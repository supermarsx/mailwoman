//! HTML → Markdown conversion for the Markdown/TXT export paths (plan §3 e3).
//!
//! Email HTML is converted, not sanitized-then-serialised: we run the real
//! `html5ever` tokenizer (never regex — same house rule as `mw-sanitize`),
//! fold the token stream into a small tolerant DOM, and walk that tree to
//! Markdown. `script`/`style`/`head`/`title`/`noscript`/`template` subtrees are
//! dropped whole so no code text leaks into the output, which gives us the
//! sanitized-shape guarantee without a dependency on `mw-sanitize`.

use std::cell::RefCell;

use html5ever::buffer_queue::BufferQueue;
use html5ever::tendril::StrTendril;
use html5ever::tokenizer::{
    Tag, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts,
};

/// Convert an HTML fragment/document to Markdown (best-effort, permissive).
#[must_use]
pub fn html_to_markdown(html: &str) -> String {
    let dom = parse(html);
    let blocks = render_blocks(&dom);
    let mut out = blocks.join("\n\n");
    // Normalise the tail: exactly one trailing newline, no leading blank lines.
    while out.ends_with(['\n', ' ', '\t', '\r']) {
        out.pop();
    }
    out
}

// ---------------------------------------------------------------------------
// Tolerant mini-DOM built from the token stream.
// ---------------------------------------------------------------------------

enum Node {
    Text(String),
    Element(Element),
}

struct Element {
    name: String,
    attrs: Vec<(String, String)>,
    children: Vec<Node>,
}

/// The `TokenSink`: `process_token` takes `&self`, so all mutable state lives
/// behind a `RefCell`. Index 0 of the stack is a synthetic root that is never
/// popped; every open element is pushed above it and folded into its parent on
/// the matching end tag (or on `finish` for unclosed tags).
struct Sink {
    stack: RefCell<Vec<Element>>,
}

impl Sink {
    fn new() -> Self {
        Self {
            stack: RefCell::new(vec![Element {
                name: String::new(),
                attrs: Vec::new(),
                children: Vec::new(),
            }]),
        }
    }

    fn open(&self, tag: &Tag) {
        let name = tag.name.as_ref().to_ascii_lowercase();
        let attrs = tag
            .attrs
            .iter()
            .map(|a| {
                (
                    a.name.local.as_ref().to_ascii_lowercase(),
                    a.value.to_string(),
                )
            })
            .collect();
        let el = Element {
            name: name.clone(),
            attrs,
            children: Vec::new(),
        };
        if tag.self_closing || is_void(&name) {
            self.attach(Node::Element(el));
        } else {
            self.stack.borrow_mut().push(el);
        }
    }

    fn close(&self, tag: &Tag) {
        let name = tag.name.as_ref().to_ascii_lowercase();
        let mut stack = self.stack.borrow_mut();
        // Find the nearest open element with this name; ignore stray end tags.
        let Some(pos) = stack.iter().rposition(|e| e.name == name) else {
            return;
        };
        if pos == 0 {
            return; // never pop the synthetic root
        }
        // Fold this element and any still-open descendants into their parents.
        while stack.len() > pos {
            let el = stack.pop().expect("len > pos > 0");
            stack
                .last_mut()
                .expect("root remains")
                .children
                .push(Node::Element(el));
        }
    }

    fn attach(&self, node: Node) {
        self.stack
            .borrow_mut()
            .last_mut()
            .expect("root remains")
            .children
            .push(node);
    }

    /// Unwind any unclosed elements and return the root's children.
    fn finish(&self) -> Vec<Node> {
        let mut stack = self.stack.borrow_mut();
        while stack.len() > 1 {
            let el = stack.pop().expect("len > 1");
            stack
                .last_mut()
                .expect("root remains")
                .children
                .push(Node::Element(el));
        }
        std::mem::take(&mut stack[0].children)
    }
}

impl TokenSink for Sink {
    type Handle = ();

    fn process_token(&self, token: Token, _line: u64) -> TokenSinkResult<()> {
        match token {
            Token::CharacterTokens(s) if !s.is_empty() => {
                self.attach(Node::Text(s.to_string()));
            }
            Token::TagToken(tag) => match tag.kind {
                TagKind::StartTag => self.open(&tag),
                TagKind::EndTag => self.close(&tag),
            },
            // Doctype/comment/null/EOF/parse-error carry nothing for Markdown.
            _ => {}
        }
        TokenSinkResult::Continue
    }
}

fn parse(html: &str) -> Vec<Node> {
    let tok = Tokenizer::new(Sink::new(), TokenizerOpts::default());
    let input = BufferQueue::default();
    input.push_back(StrTendril::from(html));
    let _ = tok.feed(&input);
    tok.end();
    tok.sink.finish()
}

// ---------------------------------------------------------------------------
// Tag classification.
// ---------------------------------------------------------------------------

/// Void elements never take an end tag (HTML spec).
fn is_void(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

/// Subtrees whose content must never reach the output (code, metadata).
fn is_dropped(name: &str) -> bool {
    matches!(
        name,
        "script" | "style" | "head" | "title" | "noscript" | "template"
    )
}

/// Block-level elements start a new Markdown block.
fn is_block(name: &str) -> bool {
    matches!(
        name,
        "p" | "div"
            | "section"
            | "article"
            | "header"
            | "footer"
            | "main"
            | "aside"
            | "figure"
            | "figcaption"
            | "blockquote"
            | "pre"
            | "hr"
            | "ul"
            | "ol"
            | "li"
            | "table"
            | "thead"
            | "tbody"
            | "tfoot"
            | "tr"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "dl"
            | "dt"
            | "dd"
    )
}

fn heading_level(name: &str) -> Option<usize> {
    match name {
        "h1" => Some(1),
        "h2" => Some(2),
        "h3" => Some(3),
        "h4" => Some(4),
        "h5" => Some(5),
        "h6" => Some(6),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Rendering: nodes → Markdown blocks / inline spans.
// ---------------------------------------------------------------------------

/// Render a container's children into a list of Markdown blocks. Runs of inline
/// content between block elements are flushed as their own paragraph block.
fn render_blocks(nodes: &[Node]) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut inline = String::new();
    for node in nodes {
        match node {
            Node::Element(el) if is_dropped(&el.name) => {}
            Node::Element(el) if is_block(&el.name) => {
                flush_inline(&mut inline, &mut blocks);
                blocks.extend(render_block(el));
            }
            _ => inline.push_str(&render_inline_node(node)),
        }
    }
    flush_inline(&mut inline, &mut blocks);
    blocks
}

fn flush_inline(inline: &mut String, blocks: &mut Vec<String>) {
    let text = inline.trim();
    if !text.is_empty() {
        blocks.push(text.to_string());
    }
    inline.clear();
}

fn render_block(el: &Element) -> Vec<String> {
    if let Some(level) = heading_level(&el.name) {
        let text = render_inline(&el.children);
        let text = text.trim();
        return if text.is_empty() {
            Vec::new()
        } else {
            vec![format!("{} {text}", "#".repeat(level))]
        };
    }
    match el.name.as_str() {
        "hr" => vec!["---".to_string()],
        "pre" => vec![format!("```\n{}\n```", text_content(el).trim_matches('\n'))],
        "blockquote" => {
            let inner = render_blocks(&el.children).join("\n\n");
            if inner.is_empty() {
                Vec::new()
            } else {
                vec![prefix_lines(&inner, "> ", "> ")]
            }
        }
        "ul" => render_list(el, false),
        "ol" => render_list(el, true),
        "li" => render_blocks(&el.children), // stray <li> outside a list
        "table" => render_table(el),
        // Generic block containers just flatten to their child blocks.
        _ => render_blocks(&el.children),
    }
}

/// Render a `<ul>`/`<ol>` as one Markdown block; nested lists indent naturally
/// because each item's continuation lines are indented by the marker width.
fn render_list(el: &Element, ordered: bool) -> Vec<String> {
    let mut items = Vec::new();
    let mut index = 1usize;
    for child in &el.children {
        let Node::Element(li) = child else { continue };
        if li.name != "li" {
            continue;
        }
        let marker = if ordered {
            let m = format!("{index}. ");
            index += 1;
            m
        } else {
            "- ".to_string()
        };
        let indent = " ".repeat(marker.len());
        let body = render_blocks(&li.children).join("\n\n");
        let body = if body.is_empty() { String::new() } else { body };
        items.push(prefix_lines(&body, &marker, &indent));
    }
    if items.is_empty() {
        Vec::new()
    } else {
        vec![items.join("\n")]
    }
}

/// Best-effort GFM pipe table. First row becomes the header; a separator row is
/// synthesised after it.
fn render_table(el: &Element) -> Vec<String> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    collect_rows(el, &mut rows);
    let rows: Vec<Vec<String>> = rows.into_iter().filter(|r| !r.is_empty()).collect();
    if rows.is_empty() {
        return Vec::new();
    }
    let cols = rows.iter().map(Vec::len).max().unwrap_or(0);
    let mut lines = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        let mut cells: Vec<String> = row.clone();
        cells.resize(cols, String::new());
        lines.push(format!("| {} |", cells.join(" | ")));
        if i == 0 {
            let sep: Vec<&str> = std::iter::repeat_n("---", cols).collect();
            lines.push(format!("| {} |", sep.join(" | ")));
        }
    }
    vec![lines.join("\n")]
}

fn collect_rows(el: &Element, rows: &mut Vec<Vec<String>>) {
    for child in &el.children {
        let Node::Element(e) = child else { continue };
        if e.name == "tr" {
            let mut cells = Vec::new();
            for c in &e.children {
                if let Node::Element(cell) = c
                    && (cell.name == "td" || cell.name == "th")
                {
                    cells.push(render_inline(&cell.children).trim().replace('|', "\\|"));
                }
            }
            rows.push(cells);
        } else {
            collect_rows(e, rows); // descend through thead/tbody/tfoot
        }
    }
}

fn render_inline(nodes: &[Node]) -> String {
    let mut s = String::new();
    for node in nodes {
        s.push_str(&render_inline_node(node));
    }
    s
}

fn render_inline_node(node: &Node) -> String {
    match node {
        Node::Text(t) => collapse_ws(t),
        Node::Element(el) if is_dropped(&el.name) => String::new(),
        Node::Element(el) => match el.name.as_str() {
            "br" => "  \n".to_string(),
            "strong" | "b" => wrap(&render_inline(&el.children), "**"),
            "em" | "i" => wrap(&render_inline(&el.children), "*"),
            "code" | "tt" | "kbd" | "samp" => {
                let t = text_content(el);
                let t = t.trim();
                if t.is_empty() {
                    String::new()
                } else {
                    format!("`{t}`")
                }
            }
            "a" => {
                let text = render_inline(&el.children);
                let text = text.trim();
                match attr(el, "href") {
                    Some(href) if !href.trim().is_empty() => {
                        let label = if text.is_empty() { href.trim() } else { text };
                        format!("[{label}]({})", href.trim())
                    }
                    _ => text.to_string(),
                }
            }
            "img" => {
                let alt = attr(el, "alt").map(str::trim).unwrap_or("");
                match attr(el, "src") {
                    Some(src) if !src.trim().is_empty() => format!("![{alt}]({})", src.trim()),
                    _ => String::new(),
                }
            }
            // Unknown/other inline containers pass their content through.
            _ => render_inline(&el.children),
        },
    }
}

/// Wrap non-empty, trimmed content in an emphasis marker, keeping surrounding
/// whitespace outside the markers (Markdown emphasis can't hug spaces).
fn wrap(inner: &str, marker: &str) -> String {
    let trimmed = inner.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let lead = if inner.starts_with(char::is_whitespace) {
        " "
    } else {
        ""
    };
    let trail = if inner.ends_with(char::is_whitespace) {
        " "
    } else {
        ""
    };
    format!("{lead}{marker}{trimmed}{marker}{trail}")
}

fn attr<'a>(el: &'a Element, name: &str) -> Option<&'a str> {
    el.attrs
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.as_str())
}

/// Concatenate all descendant text verbatim (for `<pre>`/`<code>`).
fn text_content(el: &Element) -> String {
    let mut out = String::new();
    fn walk(el: &Element, out: &mut String) {
        for child in &el.children {
            match child {
                Node::Text(t) => out.push_str(t),
                Node::Element(e) if is_dropped(&e.name) => {}
                Node::Element(e) => {
                    if e.name == "br" {
                        out.push('\n');
                    }
                    walk(e, out);
                }
            }
        }
    }
    walk(el, &mut out);
    out
}

/// Collapse every run of ASCII whitespace to a single space (HTML flow rules).
fn collapse_ws(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_ws = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            in_ws = true;
        } else {
            if in_ws && !out.is_empty() {
                out.push(' ');
            }
            in_ws = false;
            out.push(ch);
        }
    }
    if in_ws {
        out.push(' '); // preserve a boundary space between inline siblings
    }
    out
}

/// Prefix the first line of `text` with `first` and every later line with
/// `rest` (used for list markers and blockquote framing).
fn prefix_lines(text: &str, first: &str, rest: &str) -> String {
    let mut out = String::new();
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }
        if i == 0 {
            out.push_str(first);
        } else if line.is_empty() {
            out.push_str(rest.trim_end());
        } else {
            out.push_str(rest);
        }
        out.push_str(line);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::html_to_markdown as md;

    #[test]
    fn paragraphs_and_headings() {
        let out = md("<h1>Title</h1><p>Hello <strong>world</strong>.</p><p>Second.</p>");
        assert_eq!(out, "# Title\n\nHello **world**.\n\nSecond.");
    }

    #[test]
    fn links_and_images() {
        let out = md(
            r#"<p>See <a href="https://x.test">the site</a>.</p><p><img src="cid:1" alt="pic"></p>"#,
        );
        assert_eq!(out, "See [the site](https://x.test).\n\n![pic](cid:1)");
    }

    #[test]
    fn unordered_and_ordered_lists() {
        let out = md("<ul><li>one</li><li>two</li></ul><ol><li>a</li><li>b</li></ol>");
        assert_eq!(out, "- one\n- two\n\n1. a\n2. b");
    }

    #[test]
    fn nested_list_indents() {
        let out = md("<ul><li>top<ul><li>child</li></ul></li></ul>");
        assert_eq!(out, "- top\n\n  - child");
    }

    #[test]
    fn blockquote_prefixes_lines() {
        let out = md("<blockquote><p>quoted</p><p>lines</p></blockquote>");
        assert_eq!(out, "> quoted\n>\n> lines");
    }

    #[test]
    fn pre_is_fenced_and_preserved() {
        let out = md("<pre>  keep   spaces\nand lines</pre>");
        assert_eq!(out, "```\n  keep   spaces\nand lines\n```");
    }

    #[test]
    fn drops_script_and_style_content() {
        let out = md("<p>ok</p><script>alert(1)</script><style>.x{color:red}</style>");
        assert_eq!(out, "ok");
        assert!(!out.contains("alert"));
        assert!(!out.contains("color"));
    }

    #[test]
    fn collapses_whitespace() {
        let out = md("<p>a\n   b\t c</p>");
        assert_eq!(out, "a b c");
    }

    #[test]
    fn tolerates_unclosed_tags() {
        let out = md("<p>one<p>two");
        assert_eq!(out, "one\n\ntwo");
    }

    #[test]
    fn emphasis_keeps_spacing_outside_markers() {
        let out = md("<p>a <em>b </em>c</p>");
        assert_eq!(out, "a *b* c");
    }

    #[test]
    fn basic_table() {
        let out = md("<table><tr><th>H1</th><th>H2</th></tr><tr><td>a</td><td>b</td></tr></table>");
        assert_eq!(out, "| H1 | H2 |\n| --- | --- |\n| a | b |");
    }
}
