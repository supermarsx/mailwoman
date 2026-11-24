//! A small JSON reader for the validators, with `JSON.parse` acceptance rules:
//! RFC 8259 grammar, no trailing data, strings may not hold raw control characters,
//! escapes must be valid, a lone UTF-16 surrogate escape is accepted (as U+FFFD,
//! which keeps its UTF-16 length of one; no validator compares such a string with
//! anything it could equal), duplicate keys keep the last value. Strings borrow the
//! input unless they contain escapes; objects are vectors searched from the end, so
//! no hashing and one allocation per array or object. serde_json's `Value` with
//! ordered maps cost about 10 µs per KiB here, this about 1.
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq)]
pub enum Node<'a> {
    Null,
    Bool(bool),
    Num(f64),
    Str(Cow<'a, str>),
    Arr(Vec<Node<'a>>),
    Obj(Vec<(Cow<'a, str>, Node<'a>)>),
}

impl<'a> Node<'a> {
    /// last value of `key` (JSON.parse keeps the last duplicate)
    pub fn field(entries: &'a [(Cow<'a, str>, Node<'a>)], key: &str) -> Option<&'a Node<'a>> {
        entries.iter().rev().find(|(k, _)| k == key).map(|(_, v)| v)
    }
    /// entries without the earlier duplicates
    pub fn unique(
        entries: &'a [(Cow<'a, str>, Node<'a>)],
    ) -> impl Iterator<Item = &'a (Cow<'a, str>, Node<'a>)> {
        entries
            .iter()
            .enumerate()
            .filter(move |(i, (k, _))| !entries[i + 1..].iter().any(|(later, _)| later == k))
            .map(|(_, e)| e)
    }
}

const MAX_DEPTH: usize = 4000;

pub fn parse(text: &str) -> Option<Node<'_>> {
    let mut p = Parser {
        s: text.as_bytes(),
        text,
        i: 0,
    };
    p.ws();
    let v = p.value(0)?;
    p.ws();
    (p.i == p.s.len()).then_some(v)
}

struct Parser<'a> {
    s: &'a [u8],
    text: &'a str,
    i: usize,
}

impl<'a> Parser<'a> {
    fn ws(&mut self) {
        while let Some(b' ' | b'\t' | b'\n' | b'\r') = self.s.get(self.i) {
            self.i += 1;
        }
    }
    fn lit(&mut self, word: &[u8], v: Node<'a>) -> Option<Node<'a>> {
        if self.s[self.i..].starts_with(word) {
            self.i += word.len();
            Some(v)
        } else {
            None
        }
    }
    fn value(&mut self, depth: usize) -> Option<Node<'a>> {
        if depth > MAX_DEPTH {
            return None;
        }
        match *self.s.get(self.i)? {
            b'{' => {
                self.i += 1;
                let mut entries = Vec::new();
                self.ws();
                if self.s.get(self.i) == Some(&b'}') {
                    self.i += 1;
                    return Some(Node::Obj(entries));
                }
                loop {
                    self.ws();
                    if self.s.get(self.i) != Some(&b'"') {
                        return None;
                    }
                    let key = self.string()?;
                    self.ws();
                    if self.s.get(self.i) != Some(&b':') {
                        return None;
                    }
                    self.i += 1;
                    self.ws();
                    let v = self.value(depth + 1)?;
                    entries.push((key, v));
                    self.ws();
                    match self.s.get(self.i)? {
                        b',' => self.i += 1,
                        b'}' => {
                            self.i += 1;
                            return Some(Node::Obj(entries));
                        }
                        _ => return None,
                    }
                }
            }
            b'[' => {
                self.i += 1;
                let mut items = Vec::new();
                self.ws();
                if self.s.get(self.i) == Some(&b']') {
                    self.i += 1;
                    return Some(Node::Arr(items));
                }
                loop {
                    self.ws();
                    items.push(self.value(depth + 1)?);
                    self.ws();
                    match self.s.get(self.i)? {
                        b',' => self.i += 1,
                        b']' => {
                            self.i += 1;
                            return Some(Node::Arr(items));
                        }
                        _ => return None,
                    }
                }
            }
            b'"' => Some(Node::Str(self.string()?)),
            b't' => self.lit(b"true", Node::Bool(true)),
            b'f' => self.lit(b"false", Node::Bool(false)),
            b'n' => self.lit(b"null", Node::Null),
            b'-' | b'0'..=b'9' => self.number(),
            _ => None,
        }
    }
    fn digits(&mut self) -> usize {
        let start = self.i;
        while let Some(b'0'..=b'9') = self.s.get(self.i) {
            self.i += 1;
        }
        self.i - start
    }
    fn number(&mut self) -> Option<Node<'a>> {
        let start = self.i;
        if self.s[self.i] == b'-' {
            self.i += 1;
        }
        match self.s.get(self.i)? {
            b'0' => self.i += 1,
            b'1'..=b'9' => {
                self.digits();
            }
            _ => return None,
        }
        if self.s.get(self.i) == Some(&b'.') {
            self.i += 1;
            if self.digits() == 0 {
                return None;
            }
        }
        if let Some(b'e' | b'E') = self.s.get(self.i) {
            self.i += 1;
            if let Some(b'+' | b'-') = self.s.get(self.i) {
                self.i += 1;
            }
            if self.digits() == 0 {
                return None;
            }
        }
        self.text[start..self.i].parse::<f64>().ok().map(Node::Num)
    }
    fn hex4(&mut self) -> Option<u32> {
        let h = self.text.get(self.i..self.i + 4)?;
        if !h.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        self.i += 4;
        u32::from_str_radix(h, 16).ok()
    }
    fn string(&mut self) -> Option<Cow<'a, str>> {
        self.i += 1; // opening quote
        let start = self.i;
        // fast path: no escape
        loop {
            match *self.s.get(self.i)? {
                b'"' => {
                    let s = &self.text[start..self.i];
                    self.i += 1;
                    return Some(Cow::Borrowed(s));
                }
                b'\\' => break,
                c if c < 0x20 => return None,
                _ => self.i += 1,
            }
        }
        let mut out = String::from(&self.text[start..self.i]);
        loop {
            match *self.s.get(self.i)? {
                b'"' => {
                    self.i += 1;
                    return Some(Cow::Owned(out));
                }
                b'\\' => {
                    self.i += 1;
                    let e = *self.s.get(self.i)?;
                    self.i += 1;
                    match e {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let u = self.hex4()?;
                            if (0xd800..0xdc00).contains(&u) && self.s[self.i..].starts_with(b"\\u")
                            {
                                let save = self.i;
                                self.i += 2;
                                match self.hex4() {
                                    Some(low) if (0xdc00..0xe000).contains(&low) => {
                                        out.push(
                                            char::from_u32(
                                                0x10000 + ((u - 0xd800) << 10) + (low - 0xdc00),
                                            )
                                            .unwrap(),
                                        );
                                        continue;
                                    }
                                    _ => self.i = save,
                                }
                            }
                            out.push(char::from_u32(u).unwrap_or('\u{fffd}'));
                        }
                        _ => return None,
                    }
                }
                c if c < 0x20 => return None,
                _ => {
                    // copy one UTF-8 scalar
                    let ch = self.text[self.i..].chars().next()?;
                    out.push(ch);
                    self.i += ch.len_utf8();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn json_parse_rules() {
        for ok in [
            "0",
            "-0.5e+3",
            "\"a\\u00e9\\ud83d\\ude00\"",
            " {\"a\":[1,2,{}],\"a\":true} ",
            "\"\\ud800x\"",
            "[]",
            "null",
        ] {
            assert!(parse(ok).is_some(), "{ok}");
        }
        for bad in [
            "",
            "01",
            "1.",
            ".5",
            "+1",
            "[1,]",
            "{\"a\":1,}",
            "\"\u{1}\"",
            "\"\\x\"",
            "tru",
            "{} x",
            "NaN",
            "'a'",
            "{a:1}",
            "\"\\u12\"",
        ] {
            assert!(parse(bad).is_none(), "{bad}");
        }
        let Some(Node::Obj(e)) = parse("{\"a\":1,\"b\":2,\"a\":3}") else {
            panic!()
        };
        assert_eq!(Node::field(&e, "a"), Some(&Node::Num(3.0)));
        assert_eq!(Node::unique(&e).count(), 2);
    }
}
