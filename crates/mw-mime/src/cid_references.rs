//! Discover inline resource references; this does not sanitize or authorize HTML.
use cssparser::{Parser, ParserInput, Token};
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{NodeData, RcDom};

/// Actual HTML/CSS `cid:` URLs in document order, deduplicated and percent-decoded
/// (RFC 2392). HTML parsing decodes attribute entities; CSS parsing handles quoted
/// URLs and escapes without treating comments or arbitrary strings as resources.
/// Returned IDs are still untrusted: the engine validates them and requires an
/// exact match with owned inline image parts before creating any MIME message.
#[must_use]
pub fn html_cid_references(html: &str) -> Vec<String> {
    let dom = html5ever::parse_document(RcDom::default(), Default::default()).one(html);
    let mut nodes = vec![dom.document.clone()];
    let mut out = Vec::new();
    while let Some(node) = nodes.pop() {
        if let NodeData::Element { name, attrs, .. } = &node.data {
            for attr in attrs.borrow().iter() {
                let value = attr.value.as_ref();
                match attr.name.local.as_ref() {
                    "style" => css_urls(value, &mut out),
                    "srcset" | "imagesrcset" => {
                        for candidate in value.split(',') {
                            if let Some(url) = candidate.split_ascii_whitespace().next() {
                                add_url(url, &mut out);
                            }
                        }
                    }
                    "src" | "href" | "background" | "poster" | "data" | "action" | "formaction"
                    | "cite" | "longdesc" | "usemap" | "profile" | "manifest" => {
                        add_url(value, &mut out)
                    }
                    _ => {}
                }
            }
            if name.local.as_ref() == "style" {
                let mut css = String::new();
                for child in node.children.borrow().iter() {
                    if let NodeData::Text { contents } = &child.data {
                        css.push_str(&contents.borrow());
                    }
                }
                css_urls(&css, &mut out);
            }
        }
        // Template contents and raw-text elements are inert in the HTML DOM;
        // only real child elements are visited. Avoid recursive traversal.
        nodes.extend(node.children.borrow().iter().rev().cloned());
    }
    out
}

fn add_url(value: &str, out: &mut Vec<String>) {
    let value = value.trim_matches(|c: char| c.is_ascii_whitespace());
    if !value
        .get(..4)
        .is_some_and(|scheme| scheme.eq_ignore_ascii_case("cid:"))
    {
        return;
    }
    let cid = percent_decode(&value.as_bytes()[4..]);
    if !cid.is_empty() && !out.contains(&cid) {
        out.push(cid);
    }
}

fn css_urls(css: &str, out: &mut Vec<String>) {
    fn collect(parser: &mut Parser<'_, '_>, out: &mut Vec<String>, depth: usize) {
        // CSS parsing is bounded even for maliciously nested functions. An
        // impossible content ID makes the engine's exact-set check fail closed.
        if depth > 64 {
            out.push("\0invalid-css-nesting".into());
            return;
        }
        while let Ok(token) = parser.next().cloned() {
            match token {
                Token::UnquotedUrl(url) => add_url(&url, out),
                Token::Function(name) if name.eq_ignore_ascii_case("url") => {
                    let _ = parser.parse_nested_block(|nested| {
                        let value = nested.expect_string_cloned()?;
                        nested.expect_exhausted()?;
                        add_url(&value, out);
                        Ok::<_, cssparser::ParseError<'_, ()>>(())
                    });
                }
                Token::Function(_)
                | Token::ParenthesisBlock
                | Token::SquareBracketBlock
                | Token::CurlyBracketBlock => {
                    let _ = parser.parse_nested_block(|nested| {
                        collect(nested, out, depth + 1);
                        Ok::<_, cssparser::ParseError<'_, ()>>(())
                    });
                }
                _ => {}
            }
        }
    }
    collect(&mut Parser::new(&mut ParserInput::new(css)), out, 0);
}

/// Decode complete percent escapes, preserving malformed escapes and replacing
/// non-UTF8 bytes as before. Content-ID validity is checked by the caller.
fn percent_decode(raw: &[u8]) -> String {
    let hex = |b: u8| char::from(b).to_digit(16).map(|d| d as u8);
    let mut out = Vec::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        let escape = (raw[i] == b'%')
            .then(|| Some(hex(*raw.get(i + 1)?)? << 4 | hex(*raw.get(i + 2)?)?))
            .flatten();
        match escape {
            Some(byte) => {
                out.push(byte);
                i += 3;
            }
            None => {
                out.push(raw[i]);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}
