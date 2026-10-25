//! A small byte cursor for the hand-written response parsers (ESEARCH, the
//! extended `STATUS` reply that NOTIFY and STATUS=SIZE produce).
//!
//! `imap-proto` 0.16.7 rejects these responses outright, which fails the whole
//! connection. The parsers built on this cursor read from the start of the
//! connection's read buffer and report one of three outcomes, mirroring the
//! streaming `imap-proto` read loop in [`crate::connection`]: a complete
//! response and its length, "need more bytes", or malformed input. They never
//! panic on arbitrary input (each has a fuzz target).

/// Outcome of a streaming parse attempt over a read buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Parsed<T> {
    /// A complete response: `consumed` bytes (through the final CRLF) produced
    /// `value`.
    Done { consumed: usize, value: T },
    /// The buffer ends before the response does; read more and retry.
    Incomplete,
    /// The bytes are not a well-formed response of this kind.
    Bad(String),
}

/// Largest literal a hand parser accepts inside a small response (a mailbox
/// name or a search correlator). Bigger literals are refused rather than
/// buffered.
pub(crate) const MAX_SMALL_LITERAL: usize = 64 * 1024;

/// Deepest parenthesis nesting accepted when skipping an unknown extension
/// value.
const MAX_NESTING: usize = 32;

#[derive(Debug)]
pub(crate) enum LexError {
    Incomplete,
    Bad(String),
}

pub(crate) type LexResult<T> = Result<T, LexError>;

pub(crate) fn bad<T>(msg: impl Into<String>) -> LexResult<T> {
    Err(LexError::Bad(msg.into()))
}

impl<T> From<LexResult<(usize, T)>> for Parsed<T> {
    fn from(r: LexResult<(usize, T)>) -> Self {
        match r {
            Ok((consumed, value)) => Parsed::Done { consumed, value },
            Err(LexError::Incomplete) => Parsed::Incomplete,
            Err(LexError::Bad(m)) => Parsed::Bad(m),
        }
    }
}

/// `atom-specials` plus CTL and the high half: bytes that end an atom.
fn is_atom_char(b: u8) -> bool {
    b > 0x20 && b < 0x7f && !matches!(b, b'(' | b')' | b'{' | b'%' | b'*' | b'"' | b'\\' | b']')
}

pub(crate) struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(buf: &'a [u8]) -> Self {
        Cursor { buf, pos: 0 }
    }

    pub(crate) fn pos(&self) -> usize {
        self.pos
    }

    pub(crate) fn peek(&self) -> LexResult<u8> {
        self.buf.get(self.pos).copied().ok_or(LexError::Incomplete)
    }

    fn bump(&mut self) -> LexResult<u8> {
        let b = self.peek()?;
        self.pos += 1;
        Ok(b)
    }

    /// Consume `b` exactly.
    pub(crate) fn byte(&mut self, b: u8) -> LexResult<()> {
        let got = self.bump()?;
        if got == b {
            Ok(())
        } else {
            self.pos -= 1;
            bad(format!(
                "expected {:?} at byte {}, found {:?}",
                b as char, self.pos, got as char
            ))
        }
    }

    pub(crate) fn sp(&mut self) -> LexResult<()> {
        self.byte(b' ')
    }

    /// True (and consumed) when the next byte is `b`.
    pub(crate) fn eat(&mut self, b: u8) -> LexResult<bool> {
        if self.peek()? == b {
            self.pos += 1;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub(crate) fn crlf(&mut self) -> LexResult<()> {
        self.byte(b'\r')?;
        self.byte(b'\n')
    }

    /// True when the cursor sits on the CRLF that ends the response.
    pub(crate) fn at_crlf(&self) -> LexResult<bool> {
        Ok(self.peek()? == b'\r')
    }

    /// Consume a keyword, ASCII case-insensitively.
    pub(crate) fn keyword(&mut self, kw: &str) -> LexResult<()> {
        let start = self.pos;
        for &want in kw.as_bytes() {
            match self.bump() {
                Ok(got) if got.eq_ignore_ascii_case(&want) => {}
                Ok(_) => {
                    self.pos = start;
                    return bad(format!("expected {kw:?} at byte {start}"));
                }
                Err(e) => {
                    self.pos = start;
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    /// An `atom` (one or more ATOM-CHARs), returned as written.
    pub(crate) fn atom(&mut self) -> LexResult<&'a str> {
        let start = self.pos;
        while is_atom_char(self.peek()?) {
            self.pos += 1;
        }
        if self.pos == start {
            return bad(format!("expected an atom at byte {start}"));
        }
        // ATOM-CHARs are all ASCII.
        Ok(std::str::from_utf8(&self.buf[start..self.pos]).unwrap_or_default())
    }

    /// `number64`: one or more digits, no overflow.
    pub(crate) fn number64(&mut self) -> LexResult<u64> {
        let start = self.pos;
        let mut n: u64 = 0;
        while self.peek()?.is_ascii_digit() {
            let d = u64::from(self.bump()? - b'0');
            n = n
                .checked_mul(10)
                .and_then(|n| n.checked_add(d))
                .ok_or_else(|| LexError::Bad(format!("number overflows at byte {start}")))?;
        }
        if self.pos == start {
            return bad(format!("expected a number at byte {start}"));
        }
        Ok(n)
    }

    /// `number`: a 32-bit unsigned integer.
    pub(crate) fn number(&mut self) -> LexResult<u32> {
        let start = self.pos;
        let n = self.number64()?;
        u32::try_from(n).map_err(|_| LexError::Bad(format!("number too large at byte {start}")))
    }

    /// `nz-number`: a non-zero 32-bit integer.
    pub(crate) fn nz_number(&mut self) -> LexResult<u32> {
        let start = self.pos;
        match self.number()? {
            0 => bad(format!("expected a non-zero number at byte {start}")),
            n => Ok(n),
        }
    }

    /// `quoted`: a double-quoted string with `\"` and `\\` escapes.
    fn quoted(&mut self) -> LexResult<Vec<u8>> {
        self.byte(b'"')?;
        let mut out = Vec::new();
        loop {
            match self.bump()? {
                b'"' => return Ok(out),
                b'\\' => match self.bump()? {
                    c @ (b'"' | b'\\') => out.push(c),
                    c => return bad(format!("bad quoted escape {:?}", c as char)),
                },
                b'\r' | b'\n' | 0 => return bad("CR, LF or NUL inside a quoted string"),
                c => out.push(c),
            }
        }
    }

    /// `literal`: `{n}` CRLF then n bytes (servers never send the
    /// non-synchronising `{n+}` form).
    fn literal(&mut self) -> LexResult<Vec<u8>> {
        self.byte(b'{')?;
        let n = self.number64()?;
        self.byte(b'}')?;
        self.crlf()?;
        let n = usize::try_from(n).unwrap_or(usize::MAX);
        if n > MAX_SMALL_LITERAL {
            return bad(format!("literal of {n} bytes is larger than allowed here"));
        }
        let end = self.pos.checked_add(n).ok_or(LexError::Incomplete)?;
        if end > self.buf.len() {
            return Err(LexError::Incomplete);
        }
        let out = self.buf[self.pos..end].to_vec();
        self.pos = end;
        Ok(out)
    }

    /// `astring`: an atom (where `]` is allowed), a quoted string or a literal.
    pub(crate) fn astring(&mut self) -> LexResult<Vec<u8>> {
        match self.peek()? {
            b'"' => self.quoted(),
            b'{' => self.literal(),
            _ => {
                let start = self.pos;
                while {
                    let b = self.peek()?;
                    is_atom_char(b) || b == b']'
                } {
                    self.pos += 1;
                }
                if self.pos == start {
                    return bad(format!("expected an astring at byte {start}"));
                }
                Ok(self.buf[start..self.pos].to_vec())
            }
        }
    }

    /// A `sequence-set` of numbers (`*` reads as `u32::MAX`), returned as
    /// inclusive ranges with `lo <= hi`.
    pub(crate) fn sequence_set(&mut self) -> LexResult<Vec<(u32, u32)>> {
        let mut out = Vec::new();
        loop {
            let a = self.seq_number()?;
            let b = if self.eat(b':')? {
                self.seq_number()?
            } else {
                a
            };
            out.push((a.min(b), a.max(b)));
            if !self.eat(b',')? {
                return Ok(out);
            }
        }
    }

    fn seq_number(&mut self) -> LexResult<u32> {
        if self.eat(b'*')? {
            Ok(u32::MAX)
        } else {
            self.nz_number()
        }
    }

    /// Skip a `tagged-ext-val` this client does not know: a balanced
    /// parenthesised group (quoted strings and literals respected) or a simple
    /// token running to the next space or CRLF.
    pub(crate) fn skip_ext_value(&mut self) -> LexResult<()> {
        if self.peek()? != b'(' {
            let start = self.pos;
            while !matches!(self.peek()?, b' ' | b'\r' | b'\n' | b'(' | b')') {
                self.pos += 1;
            }
            if self.pos == start {
                return bad(format!("expected an extension value at byte {start}"));
            }
            return Ok(());
        }
        let mut depth = 0usize;
        loop {
            match self.peek()? {
                b'(' => {
                    depth += 1;
                    if depth > MAX_NESTING {
                        return bad("extension value nested too deeply");
                    }
                    self.pos += 1;
                }
                b')' => {
                    self.pos += 1;
                    depth -= 1;
                    if depth == 0 {
                        return Ok(());
                    }
                }
                b'"' => {
                    self.quoted()?;
                }
                b'{' => {
                    self.literal()?;
                }
                b'\r' | b'\n' => return bad("unbalanced parenthesis in extension value"),
                _ => self.pos += 1,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn astring_forms() {
        let mut c = Cursor::new(b"\"a\\\"b\" atom] {3}\r\nxyz ");
        assert_eq!(c.astring().unwrap(), b"a\"b");
        c.sp().unwrap();
        assert_eq!(c.astring().unwrap(), b"atom]");
        c.sp().unwrap();
        assert_eq!(c.astring().unwrap(), b"xyz");
    }

    #[test]
    fn truncated_literal_is_incomplete() {
        let mut c = Cursor::new(b"{5}\r\nab");
        assert!(matches!(c.astring(), Err(LexError::Incomplete)));
    }

    #[test]
    fn oversized_literal_is_refused() {
        let mut c = Cursor::new(b"{99999999}\r\n");
        assert!(matches!(c.astring(), Err(LexError::Bad(_))));
    }

    #[test]
    fn sequence_sets_normalise_and_star() {
        let mut c = Cursor::new(b"5:3,7,9:* ");
        assert_eq!(
            c.sequence_set().unwrap(),
            vec![(3, 5), (7, 7), (9, u32::MAX)]
        );
        assert!(Cursor::new(b"0 ").sequence_set().is_err());
    }

    #[test]
    fn numbers_reject_overflow() {
        assert!(matches!(
            Cursor::new(b"99999999999 ").number(),
            Err(LexError::Bad(_))
        ));
        assert!(matches!(
            Cursor::new(b"999999999999999999999 ").number64(),
            Err(LexError::Bad(_))
        ));
        assert_eq!(Cursor::new(b"42 ").number().unwrap(), 42);
    }

    #[test]
    fn skip_ext_value_handles_nesting_and_strings() {
        let mut c = Cursor::new(b"(a (b \")\" {1}\r\n)) c) rest");
        c.skip_ext_value().unwrap();
        assert_eq!(c.peek().unwrap(), b' ');
        let deep = format!("{}\r\n", "(".repeat(100));
        assert!(matches!(
            Cursor::new(deep.as_bytes()).skip_ext_value(),
            Err(LexError::Bad(_))
        ));
    }
}
