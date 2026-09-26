//! A minimal, dependency-free XML reader — just enough to walk Thunderbird
//! autoconfig documents (plan §0 rung 2).
//!
//! Autoconfig XML is small, namespace-free, and regular, so a purpose-built
//! tree reader avoids pulling a general XML crate through the license floor.
//! It intentionally handles only what the format uses: elements, attributes,
//! text, comments, CDATA, the XML declaration, and self-closing tags. It is
//! **not** a general-purpose XML parser: no namespaces, no entity declarations,
//! no external references.
//!
//! **It does, however, run over fully attacker-chosen bytes** — a claim this
//! module's own header used to deny ("never run over untrusted bytes"), which
//! was wrong and is corrected here (t25-e3). `POST /api/discover` is
//! unauthenticated and CSRF-exempt by design (`mw-server/src/lib.rs`, the
//! comment on the route), and the ladder fetches
//! `https://autoconfig.<domain>/mail/config-v1.1.xml` from a host the requester
//! names. Everything this parser sees on that rung is written by that host. So
//! the two bounds below are load-bearing, not decoration:
//!
//! - [`MAX_DEPTH`] caps nesting. `parse_element` recurses per nested element and
//!   the egress fetch admits 8 MiB (`mw-egress`'s `MAX_IMAGE_BYTES`), which is
//!   ~2.8 M `<a>` tokens — see [`MAX_DEPTH`] for what was measured.
//! - Every read is bounds-clamped, so a truncated document yields `None` rather
//!   than an index panic. `<a b="` — six bytes — used to panic here.

/// One parsed element node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    pub children: Vec<Node>,
    pub text: String,
}

impl Node {
    /// Direct child elements with the given tag name.
    pub fn children_named<'s>(&'s self, name: &str) -> impl Iterator<Item = &'s Node> {
        let needle = name.to_owned();
        self.children.iter().filter(move |c| c.name == needle)
    }

    /// First direct child with the given tag name.
    pub fn child(&self, name: &str) -> Option<&Node> {
        self.children.iter().find(|c| c.name == name)
    }

    /// Value of an attribute, if present.
    pub fn attr(&self, key: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// Trimmed text of the first child element with the given name.
    pub fn child_text(&self, name: &str) -> Option<String> {
        self.child(name).map(|n| n.text.trim().to_string())
    }
}

/// Maximum element nesting this parser will descend into. Past it, [`parse`]
/// reports the document malformed rather than recursing further.
///
/// **Why a cap is needed.** `parse_element` is recursive-descent with one frame
/// per nesting level, and nothing upstream bounds nesting: the body arrives from
/// an attacker-named host on an unauthenticated route and the egress fetch
/// admits 8 MiB, which at three bytes per `<a>` is ~2.8 M levels. A stack
/// overflow is not a catchable panic — it is an abort that takes the process and
/// every other session with it, so this is a process-kill primitive, not a
/// failed request.
///
/// **What was measured (t25-e3), and on what.** The real `parse` in this file,
/// `cargo test` **debug** profile, Windows `x86_64-pc-windows-msvc`, run on a
/// thread created with `stack_size(2 * 1024 * 1024)` — tokio's documented
/// default worker stack, which is what `mw-server` gets since it is a plain
/// `#[tokio::main]` and nothing in `crates/` sets a stack size. 1 050 and 1 100
/// levels returned; **1 150 aborted the test process with
/// `STATUS_STACK_OVERFLOW` (0xc00000fd)**, printing `thread '<unknown>' has
/// overflowed its stack`. So the ceiling sits between 1 100 and 1 150 there,
/// ~1.9 KiB of stack per level.
///
/// **What that measurement is not.** It is a debug build, so frames are larger
/// than release's; it is Windows, and per-level cost varies with platform and
/// ABI; and it is an in-process thread sized like a tokio worker, not a live
/// `POST /api/discover`. The failure *class* is demonstrated; the exact boundary
/// is platform-specific. The cap is justified either way, because 8 MiB of input
/// is three orders of magnitude past any plausible ceiling.
///
/// **Why 64.** Real autoconfig documents nest about five deep
/// (`clientConfig` → `emailProvider` → `incomingServer` → a leaf), so 64 is an
/// order of magnitude of headroom over anything Thunderbird's schema can produce
/// while sitting ~17× below the measured debug ceiling. It is also the value
/// `mw-search`'s query parser chose for the same failure mode, and one number
/// across both parsers is easier to reason about than two.
pub const MAX_DEPTH: usize = 64;

/// Parse an XML document into its root element, or `None` if malformed — which
/// now includes "nested deeper than [`MAX_DEPTH`]".
pub fn parse(input: &str) -> Option<Node> {
    let mut p = Parser {
        b: input.as_bytes(),
        pos: 0,
    };
    p.skip_misc();
    let node = p.parse_element(0)?;
    Some(node)
}

struct Parser<'a> {
    b: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    /// The unconsumed tail. Total by construction: `pos` past the end yields an
    /// empty slice instead of panicking. Every advance below also keeps
    /// `pos <= len`, so this is defence in depth rather than the primary
    /// guarantee — but it is what makes `starts_with` safe to call from any
    /// state, which is the property the attribute loop relies on.
    fn rest(&self) -> &'a [u8] {
        self.b.get(self.pos..).unwrap_or(&[])
    }

    fn starts_with(&self, s: &str) -> bool {
        self.rest().starts_with(s.as_bytes())
    }

    fn skip_ws(&mut self) {
        while self.pos < self.b.len() && self.b[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    /// Skip prolog/misc: whitespace, `<?...?>` declarations, `<!-- -->`
    /// comments, and `<!DOCTYPE ...>` before the root element.
    fn skip_misc(&mut self) {
        loop {
            self.skip_ws();
            if self.starts_with("<?") {
                self.skip_until("?>");
            } else if self.starts_with("<!--") {
                self.skip_until("-->");
            } else if self.starts_with("<!") {
                self.skip_until(">");
            } else {
                break;
            }
        }
    }

    fn skip_until(&mut self, marker: &str) {
        if let Some(i) = find(self.rest(), marker.as_bytes()) {
            self.pos += i + marker.len();
        } else {
            self.pos = self.b.len();
        }
    }

    /// `depth` is the number of enclosing elements. Checked on entry so the
    /// refusal happens before the frame does any work, and so the root is
    /// depth 0.
    fn parse_element(&mut self, depth: usize) -> Option<Node> {
        if depth > MAX_DEPTH {
            return None;
        }
        if !self.starts_with("<") {
            return None;
        }
        self.pos += 1; // consume '<'
        let name = self.read_name();
        if name.is_empty() {
            return None;
        }
        let mut attrs = Vec::new();

        loop {
            self.skip_ws();
            // An open tag that ends with the document is malformed. The content
            // loop below has always checked this; this loop did not, and that
            // asymmetry is what turned `<a b="` into a panic — `read_quoted`
            // left `pos == len + 1` and the `starts_with` below indexed past the
            // end. Both halves are fixed; this check is the one that says why.
            if self.pos >= self.b.len() {
                return None;
            }
            if self.starts_with("/>") {
                self.pos += 2;
                return Some(Node {
                    name,
                    attrs,
                    children: Vec::new(),
                    text: String::new(),
                });
            }
            if self.starts_with(">") {
                self.pos += 1;
                break;
            }
            // Attribute: name="value" (or single-quoted).
            let attr_name = self.read_name();
            if attr_name.is_empty() {
                return None;
            }
            self.skip_ws();
            if !self.starts_with("=") {
                // Valueless attribute — tolerate and continue.
                attrs.push((attr_name, String::new()));
                continue;
            }
            self.pos += 1; // '='
            self.skip_ws();
            let value = self.read_quoted()?;
            attrs.push((attr_name, value));
        }

        // Content.
        let mut children = Vec::new();
        let mut text = String::new();
        loop {
            if self.pos >= self.b.len() {
                break;
            }
            if self.starts_with("</") {
                self.pos += 2;
                let _close = self.read_name();
                self.skip_until(">");
                break;
            }
            if self.starts_with("<!--") {
                self.skip_until("-->");
                continue;
            }
            if self.starts_with("<![CDATA[") {
                self.pos += "<![CDATA[".len();
                let start = self.pos;
                if let Some(i) = find(self.rest(), b"]]>") {
                    text.push_str(&decode(&self.b[start..start + i]));
                    self.pos += i + 3;
                } else {
                    self.pos = self.b.len();
                }
                continue;
            }
            if self.starts_with("<?") {
                self.skip_until("?>");
                continue;
            }
            if self.starts_with("<") {
                let child = self.parse_element(depth + 1)?;
                children.push(child);
                continue;
            }
            // Text run up to the next '<'.
            let start = self.pos;
            while self.pos < self.b.len() && self.b[self.pos] != b'<' {
                self.pos += 1;
            }
            text.push_str(&unescape(&decode(&self.b[start..self.pos])));
        }

        Some(Node {
            name,
            attrs,
            children,
            text,
        })
    }

    fn read_name(&mut self) -> String {
        self.skip_ws();
        let start = self.pos;
        while self.pos < self.b.len() {
            let c = self.b[self.pos];
            if c.is_ascii_whitespace() || c == b'>' || c == b'/' || c == b'=' {
                break;
            }
            self.pos += 1;
        }
        decode(&self.b[start..self.pos])
    }

    fn read_quoted(&mut self) -> Option<String> {
        let quote = *self.rest().first()?;
        if quote != b'"' && quote != b'\'' {
            return None;
        }
        self.pos += 1;
        let start = self.pos;
        while self.pos < self.b.len() && self.b[self.pos] != quote {
            self.pos += 1;
        }
        let v = unescape(&decode(&self.b[start..self.pos]));
        // Step over the closing quote — clamped, because on an *unterminated*
        // quote the loop above exits at `pos == len` and an unconditional `+= 1`
        // would leave `pos == len + 1`, breaking the parser's `pos <= len`
        // invariant for every later read.
        self.pos = (self.pos + 1).min(self.b.len());
        Some(v)
    }
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn decode(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn unescape(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_elements_attrs_and_text() {
        let doc = r#"<?xml version="1.0"?>
            <!-- comment -->
            <root a="1" b='two'>
              <child>hello</child>
              <self closed="yes"/>
            </root>"#;
        let root = parse(doc).unwrap();
        assert_eq!(root.name, "root");
        assert_eq!(root.attr("a"), Some("1"));
        assert_eq!(root.attr("b"), Some("two"));
        assert_eq!(root.child_text("child").as_deref(), Some("hello"));
        assert_eq!(root.child("self").unwrap().attr("closed"), Some("yes"));
    }

    #[test]
    fn unescapes_entities() {
        let root = parse(r#"<r>a &amp; b &lt;x&gt;</r>"#).unwrap();
        assert_eq!(root.text.trim(), "a & b <x>");
    }

    /// Regression for the 26.20 unauthenticated stack-overflow DoS. Before
    /// [`MAX_DEPTH`], this body — reachable as the HTTP response of an
    /// attacker-named `autoconfig.<domain>` host on the unauthenticated
    /// `POST /api/discover` — aborted the process.
    ///
    /// **It is run on a thread explicitly sized to 2 MiB**, tokio's default
    /// worker stack, and not on libtest's own thread. That is the whole point:
    /// libtest's stack is larger, so a test on the default thread would pass at
    /// depths that kill a real request, and the guard would look tested when it
    /// was not. Measured on this exact harness at 1 150 levels with the cap
    /// removed: `thread '<unknown>' has overflowed its stack`, process exit
    /// `0xc00000fd`. See [`MAX_DEPTH`].
    ///
    /// A stack overflow cannot be caught in-process, so the assertion is that
    /// the call **returns at all** — the thread joining is itself the result.
    /// 50 000 is far above the measured ceiling, so this pins the failure class
    /// rather than the boundary.
    #[test]
    fn deep_nesting_is_refused_not_fatal() {
        let refused = std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(|| parse(&"<a>".repeat(50_000)).is_none())
            .expect("spawn a 2 MiB probe thread")
            .join()
            .expect("the probe thread must return — a stack overflow aborts the process");
        assert!(refused, "over-deep nesting must parse as malformed");
    }

    /// Control for the test above: the cap must refuse only absurd documents,
    /// not real ones. Without this, "deep input returns None" would be satisfied
    /// by a parser that rejects every nested document, and the fix would have
    /// broken discovery while passing its own regression test.
    #[test]
    fn nesting_within_the_cap_still_parses() {
        // Exactly at the cap parses; one level deeper does not. Pinning both
        // sides means an off-by-one in the comparison cannot pass unnoticed.
        let doc = |n: usize| format!("{}x{}", "<a>".repeat(n), "</a>".repeat(n));
        assert!(
            parse(&doc(MAX_DEPTH + 1)).is_some(),
            "a document nested to exactly MAX_DEPTH must still parse"
        );
        assert!(
            parse(&doc(MAX_DEPTH + 2)).is_none(),
            "one level past MAX_DEPTH must be refused"
        );

        // The shape real autoconfig documents have, which is nowhere near it.
        let real = r#"<clientConfig version="1.1"><emailProvider id="x.example">
            <incomingServer type="imap"><hostname>imap.x.example</hostname>
            </incomingServer></emailProvider></clientConfig>"#;
        assert!(parse(real).is_some());
    }

    /// Regression for the six-byte remote panic: `read_quoted`'s unconditional
    /// advance over the closing quote left `pos == len + 1` on an unterminated
    /// quote, and the attribute loop — unlike the content loop — had no
    /// end-of-input check, so the next `starts_with` sliced out of range. Six
    /// bytes from an attacker's autoconfig host panicked the request task.
    ///
    /// Each of these used to panic or is a neighbour of one; all must now be
    /// ordinary `None`.
    #[test]
    fn truncated_documents_return_none_not_panic() {
        for doc in [
            "<a b=\"",    // the original: unterminated double quote
            "<a b='",     // and single-quoted
            "<a b=\"v",   // value, still unterminated
            "<a b=\"v\"", // quote closed, tag not
            "<a b",       // attribute name, nothing else
            "<a b=",      // '=' with no value at all
            "<a ",        // open tag, trailing space
            "<a",         // open tag, nothing else
            "<",          // just the bracket
            "<a><b c=\"", // the same defect one level down
        ] {
            assert!(
                parse(doc).is_none(),
                "{doc:?} must parse as malformed, not panic"
            );
        }

        // Truncation *inside content* is deliberately tolerated — the content
        // loop consumes to end-of-input and returns the element it has, which is
        // what makes a truncated-but-usable autoconfig response still usable.
        // These are here to assert they return rather than panic, and to record
        // that `Some` is the intended answer, not an oversight.
        for doc in ["<a><![CDATA[", "<a><!--", "<a><?pi", "<a>text", "<a></"] {
            assert!(
                parse(doc).is_some(),
                "{doc:?} is tolerated truncation, not malformed"
            );
        }
    }
}
