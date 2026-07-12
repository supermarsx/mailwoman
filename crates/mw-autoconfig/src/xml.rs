//! A minimal, dependency-free XML reader — just enough to walk Thunderbird
//! autoconfig documents (plan §0 rung 2).
//!
//! Autoconfig XML is small, namespace-free, and regular, so a purpose-built
//! tree reader avoids pulling a general XML crate through the license floor.
//! It intentionally handles only what the format uses: elements, attributes,
//! text, comments, CDATA, the XML declaration, and self-closing tags. It is
//! **not** a general-purpose or security-hardened parser and is never run over
//! untrusted message bytes (that is `mw-mime`'s job, inside the render jail).

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

/// Parse an XML document into its root element, or `None` if malformed.
pub fn parse(input: &str) -> Option<Node> {
    let mut p = Parser {
        b: input.as_bytes(),
        pos: 0,
    };
    p.skip_misc();
    let node = p.parse_element()?;
    Some(node)
}

struct Parser<'a> {
    b: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn rest(&self) -> &'a [u8] {
        &self.b[self.pos..]
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

    fn parse_element(&mut self) -> Option<Node> {
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
                let child = self.parse_element()?;
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
        self.pos += 1; // closing quote
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
}
