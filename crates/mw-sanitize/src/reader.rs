//! Reader-only quarantine. Inspection is an additional conservative admission
//! check; ammonia remains the final HTML security boundary for every outcome.
use std::cell::{Cell, RefCell};

use html5ever::buffer_queue::BufferQueue;
use html5ever::tendril::StrTendril;
use html5ever::tokenizer::{
    TagKind, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts, states::RawKind,
};

/// Active message HTML is rendered as inert plaintext, without remote references.
/// The input remains untouched in storage. Compose's generic sanitizer is unchanged.
pub fn sanitize_email_html_reader(input: &str, opts: &crate::SanitizeOptions) -> String {
    let tokenizer = Tokenizer::new(Inspect::default(), TokenizerOpts::default());
    let queue = BufferQueue::default();
    queue.push_back(StrTendril::from(input));
    let _ = tokenizer.feed(&queue);
    tokenizer.end();
    if tokenizer.sink.blocked.get() || unsafe_css(&tokenizer.sink.styles.borrow()) {
        // Empty tags/attributes means the entire retained body is escaped text.
        // Script/style contents are removed by ammonia's clean-content policy.
        let text = ammonia::Builder::empty().clean(input).to_string();
        format!("<pre data-mw-body-blocked=\"true\">{text}</pre>")
    } else {
        crate::sanitize_email_html_with(input, opts)
    }
}

#[derive(Default)]
struct Inspect {
    blocked: Cell<bool>,
    in_style: Cell<bool>,
    styles: RefCell<String>,
}

fn executable_url(value: &str) -> bool {
    // HTML entities have already been decoded by the HTML5 tokenizer. Browsers
    // remove ASCII tab/newline within schemes and trim leading C0 whitespace.
    let normalized: String = value.chars().filter(|c| !c.is_ascii_control()).collect();
    let scheme = normalized.trim_start().split_once(':').map(|(s, _)| s);
    scheme.is_some_and(|s| {
        matches!(
            s.to_ascii_lowercase().as_str(),
            "javascript" | "vbscript" | "data" | "blob" | "file" | "filesystem"
        )
    })
}

impl TokenSink for Inspect {
    type Handle = ();
    fn process_token(&self, token: Token, _: u64) -> TokenSinkResult<()> {
        match token {
            Token::TagToken(tag) if tag.kind == TagKind::StartTag => {
                let name = tag.name.as_ref();
                if matches!(
                    name,
                    "script"
                        | "iframe"
                        | "frame"
                        | "frameset"
                        | "object"
                        | "embed"
                        | "applet"
                        | "form"
                        | "input"
                        | "button"
                        | "textarea"
                        | "select"
                        | "link"
                        | "base"
                        | "svg"
                        | "math"
                        | "template"
                        | "noscript"
                        | "noembed"
                        | "xmp"
                        | "plaintext"
                ) {
                    self.blocked.set(true);
                }
                for attr in &tag.attrs {
                    let attr_name = attr.name.local.as_ref();
                    let value = attr.value.as_ref();
                    if attr_name.starts_with("on")
                        || matches!(attr_name, "srcdoc" | "data-mw-body-blocked")
                        || (name == "meta"
                            && attr_name == "http-equiv"
                            && value.trim().eq_ignore_ascii_case("refresh"))
                        || (matches!(
                            attr_name,
                            "href"
                                | "src"
                                | "action"
                                | "formaction"
                                | "background"
                                | "poster"
                                | "data"
                                | "xlink:href"
                        ) && executable_url(value))
                        || (attr_name == "style" && unsafe_css(value))
                        || (attr_name == "srcset"
                            && value
                                .split(|c: char| c == ',' || c.is_ascii_whitespace())
                                .any(executable_url))
                    {
                        self.blocked.set(true);
                    }
                }
                if name == "style" {
                    self.in_style.set(true);
                    return TokenSinkResult::RawData(RawKind::Rawtext);
                }
                if name == "title" {
                    return TokenSinkResult::RawData(RawKind::Rcdata);
                }
            }
            Token::TagToken(tag) if tag.kind == TagKind::EndTag && tag.name.as_ref() == "style" => {
                self.in_style.set(false);
                self.styles.borrow_mut().push('\n');
            }
            Token::CharacterTokens(text) if self.in_style.get() => {
                self.styles.borrow_mut().push_str(&text)
            }
            _ => {}
        }
        TokenSinkResult::Continue
    }
}

fn unsafe_css(css: &str) -> bool {
    fn inspect(parser: &mut cssparser::Parser<'_, '_>, depth: usize) -> bool {
        if depth > 64 {
            return true;
        }
        while let Ok(token) = parser.next_including_whitespace_and_comments().cloned() {
            use cssparser::Token;
            match token {
                Token::AtKeyword(name) if name.eq_ignore_ascii_case("import") => return true,
                Token::Ident(name)
                    if name.eq_ignore_ascii_case("behavior")
                        || name.eq_ignore_ascii_case("-moz-binding") =>
                {
                    return true;
                }
                Token::UnquotedUrl(url) | Token::QuotedString(url) if executable_url(&url) => {
                    return true;
                }
                Token::Function(name) if name.eq_ignore_ascii_case("expression") => return true,
                Token::Function(_)
                | Token::ParenthesisBlock
                | Token::SquareBracketBlock
                | Token::CurlyBracketBlock => {
                    if parser
                        .parse_nested_block(|p| {
                            Ok::<_, cssparser::ParseError<'_, ()>>(inspect(p, depth + 1))
                        })
                        .unwrap_or(true)
                    {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
    inspect(
        &mut cssparser::Parser::new(&mut cssparser::ParserInput::new(css)),
        0,
    )
}
