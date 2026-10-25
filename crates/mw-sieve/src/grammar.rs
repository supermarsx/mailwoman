//! The full RFC 5228 Sieve grammar: a hand-written lexer and recursive-descent
//! parser producing an [`Script`] syntax tree with a position on every node.
//!
//! [`crate::parse`] reads back only the constrained subset [`crate::codegen`]
//! emits; this module reads any script a user or another client wrote. It is
//! what validation, the extension inventory ([`crate::extensions`]), the server
//! policy ([`crate::policy`]) and the builder's managed blocks
//! ([`crate::managed`]) stand on.
//!
//! Grammar (RFC 5228 §8.2):
//!
//! ```text
//! commands    = *command
//! command     = identifier arguments ( ";" / block )
//! block       = "{" commands "}"
//! arguments   = *argument [ test / test-list ]
//! argument    = string-list / number / tag
//! string-list = "[" string *( "," string ) "]" / string
//! test        = identifier arguments
//! test-list   = "(" test *( "," test ) ")"
//! ```
//!
//! Strings are quoted (`"..."`, `\` escapes the next character) or multi-line
//! (`text:` ... a line holding only `.`, with dot-stuffing). Numbers take an
//! optional `K`/`M`/`G` quantifier. Comments are `#` to end of line and
//! `/* ... */`.
//!
//! Every error carries the 1-based line and column (in characters) of the token
//! that caused it. Nesting depth is bounded ([`MAX_DEPTH`]) so a hostile script
//! cannot exhaust the stack; nothing here panics on any input (fuzzed).

use std::fmt;

/// Deepest allowed nesting of blocks and tests combined. Real scripts stay in
/// single digits; the bound only exists so recursion cannot overflow the stack.
pub const MAX_DEPTH: usize = 64;

/// A position range in the source. `start`/`end` are byte offsets; `line` and
/// `column` (1-based, column counted in characters) locate `start`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: u32,
    pub column: u32,
}

/// A syntax or validation error at a source position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptError {
    pub line: u32,
    pub column: u32,
    pub message: String,
}

impl ScriptError {
    pub(crate) fn at(span: Span, message: impl Into<String>) -> Self {
        ScriptError {
            line: span.line,
            column: span.column,
            message: message.into(),
        }
    }
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "line {}, column {}: {}",
            self.line, self.column, self.message
        )
    }
}

impl std::error::Error for ScriptError {}

/// A parsed script: its top-level commands in source order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Script {
    pub commands: Vec<Command>,
}

/// One command (`fileinto "x";`, `if ... { ... }`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    /// The identifier as written (compare with [`Command::is`]).
    pub name: String,
    pub span: Span,
    pub arguments: Vec<Argument>,
    /// The test or tests after the arguments (`if` takes exactly one).
    pub tests: Vec<Test>,
    /// Whether `tests` was written as a parenthesised test-list.
    pub test_list: bool,
    /// The `{ ... }` block, for `if`/`elsif`/`else` (and extension commands).
    pub block: Option<Block>,
}

impl Command {
    /// Case-insensitive name comparison (Sieve identifiers ignore case).
    pub fn is(&self, name: &str) -> bool {
        self.name.eq_ignore_ascii_case(name)
    }
}

/// A `{ ... }` block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub commands: Vec<Command>,
    pub span: Span,
}

/// A test (`header :contains "subject" "x"`, `allof (...)`, `not exists "x"`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Test {
    pub name: String,
    pub span: Span,
    pub arguments: Vec<Argument>,
    pub tests: Vec<Test>,
    pub test_list: bool,
}

impl Test {
    /// Case-insensitive name comparison.
    pub fn is(&self, name: &str) -> bool {
        self.name.eq_ignore_ascii_case(name)
    }
}

/// One positional argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Argument {
    pub kind: ArgumentKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgumentKind {
    /// A single string or a `[...]` string list (`bracketed` tells which).
    Strings {
        values: Vec<SieveString>,
        bracketed: bool,
    },
    /// A number with its quantifier applied (`10K` is 10240).
    Number(u64),
    /// A tag without its leading colon, as written (`contains`, `copy`).
    Tag(String),
}

impl Argument {
    /// The tag name when this argument is a tag.
    pub fn tag(&self) -> Option<&str> {
        match &self.kind {
            ArgumentKind::Tag(t) => Some(t),
            _ => None,
        }
    }

    /// The strings when this argument is a string or string list.
    pub fn strings(&self) -> Option<&[SieveString]> {
        match &self.kind {
            ArgumentKind::Strings { values, .. } => Some(values),
            _ => None,
        }
    }
}

/// A decoded string value with its source position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SieveString {
    pub value: String,
    pub span: Span,
    /// Written as `text:` ... `.` rather than `"..."`.
    pub multiline: bool,
}

/// Parse a complete script. Syntax only: which commands exist and whether their
/// extensions are required is [`crate::extensions::analyze`]'s job.
pub fn parse_script(source: &str) -> Result<Script, ScriptError> {
    let tokens = lex(source)?;
    let mut p = Parser {
        toks: &tokens,
        pos: 0,
        eof: eof_span(source),
    };
    let commands = p.commands(0, false)?;
    Ok(Script { commands })
}

// ── lexer ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    Ident(String),
    Tag(String),
    Number(u64),
    Str(String, bool),
    Punct(u8),
}

#[derive(Debug, Clone)]
struct Token {
    tok: Tok,
    span: Span,
}

/// A cursor over the source that tracks line and column.
struct Cursor<'a> {
    src: &'a str,
    bytes: &'a [u8],
    pos: usize,
    line: u32,
    column: u32,
}

impl<'a> Cursor<'a> {
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn peek_at(&self, off: usize) -> Option<u8> {
        self.bytes.get(self.pos + off).copied()
    }

    fn here(&self) -> Span {
        Span {
            start: self.pos,
            end: self.pos,
            line: self.line,
            column: self.column,
        }
    }

    /// Advance one character (not byte), keeping line/column right.
    fn bump(&mut self) {
        let Some(b) = self.peek() else { return };
        if b == b'\n' {
            self.line = self.line.saturating_add(1);
            self.column = 1;
            self.pos += 1;
            return;
        }
        let width = self.src[self.pos..]
            .chars()
            .next()
            .map_or(1, char::len_utf8);
        self.pos += width;
        self.column = self.column.saturating_add(1);
    }

    fn eat(&mut self, b: u8) -> bool {
        if self.peek() == Some(b) {
            self.bump();
            true
        } else {
            false
        }
    }

    /// Consume a line break (CRLF or bare LF). Returns whether one was there.
    fn eat_newline(&mut self) -> bool {
        if self.peek() == Some(b'\r') && self.peek_at(1) == Some(b'\n') {
            self.bump();
            self.bump();
            true
        } else {
            self.eat(b'\n')
        }
    }
}

fn eof_span(source: &str) -> Span {
    let mut line = 1u32;
    let mut column = 1u32;
    for c in source.chars() {
        if c == '\n' {
            line = line.saturating_add(1);
            column = 1;
        } else {
            column = column.saturating_add(1);
        }
    }
    Span {
        start: source.len(),
        end: source.len(),
        line,
        column,
    }
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn lex(source: &str) -> Result<Vec<Token>, ScriptError> {
    let mut c = Cursor {
        src: source,
        bytes: source.as_bytes(),
        pos: 0,
        line: 1,
        column: 1,
    };
    let mut out = Vec::new();
    while let Some(b) = c.peek() {
        let start = c.here();
        match b {
            b' ' | b'\t' | b'\r' | b'\n' => c.bump(),
            b'#' => {
                while let Some(x) = c.peek() {
                    if x == b'\n' {
                        break;
                    }
                    c.bump();
                }
            }
            b'/' if c.peek_at(1) == Some(b'*') => {
                c.bump();
                c.bump();
                loop {
                    match c.peek() {
                        None => {
                            return Err(ScriptError::at(
                                start,
                                "unterminated comment: `/*` without `*/`",
                            ));
                        }
                        Some(b'*') if c.peek_at(1) == Some(b'/') => {
                            c.bump();
                            c.bump();
                            break;
                        }
                        Some(_) => c.bump(),
                    }
                }
            }
            b'"' => {
                c.bump();
                let mut value = String::new();
                loop {
                    match c.peek() {
                        None => {
                            return Err(ScriptError::at(start, "unterminated string"));
                        }
                        Some(b'"') => {
                            c.bump();
                            break;
                        }
                        Some(b'\\') => {
                            c.bump();
                            match c.src[c.pos..].chars().next() {
                                Some(ch) => {
                                    value.push(ch);
                                    c.bump();
                                }
                                None => {
                                    return Err(ScriptError::at(start, "unterminated string"));
                                }
                            }
                        }
                        Some(_) => {
                            if let Some(ch) = c.src[c.pos..].chars().next() {
                                value.push(ch);
                            }
                            c.bump();
                        }
                    }
                }
                out.push(token(Tok::Str(value, false), start, c.pos));
            }
            b'[' | b']' | b'(' | b')' | b'{' | b'}' | b';' | b',' => {
                c.bump();
                out.push(token(Tok::Punct(b), start, c.pos));
            }
            b':' => {
                c.bump();
                if !c.peek().is_some_and(is_ident_start) {
                    return Err(ScriptError::at(start, "a tag needs a name after `:`"));
                }
                let name_start = c.pos;
                while c.peek().is_some_and(is_ident_char) {
                    c.bump();
                }
                let name = source[name_start..c.pos].to_string();
                out.push(token(Tok::Tag(name), start, c.pos));
            }
            b'0'..=b'9' => {
                let digits_start = c.pos;
                while c.peek().is_some_and(|x| x.is_ascii_digit()) {
                    c.bump();
                }
                let digits = &source[digits_start..c.pos];
                let mut value: u64 = digits
                    .parse()
                    .map_err(|_| ScriptError::at(start, "number too large"))?;
                let shift = match c.peek() {
                    Some(b'K' | b'k') => 10,
                    Some(b'M' | b'm') => 20,
                    Some(b'G' | b'g') => 30,
                    _ => 0,
                };
                if shift > 0 {
                    c.bump();
                    value = value
                        .checked_mul(1u64 << shift)
                        .ok_or_else(|| ScriptError::at(start, "number too large"))?;
                }
                if c.peek().is_some_and(is_ident_char) {
                    return Err(ScriptError::at(start, "malformed number"));
                }
                out.push(token(Tok::Number(value), start, c.pos));
            }
            x if is_ident_start(x) => {
                let id_start = c.pos;
                while c.peek().is_some_and(is_ident_char) {
                    c.bump();
                }
                let ident = &source[id_start..c.pos];
                if ident.eq_ignore_ascii_case("text") && c.peek() == Some(b':') {
                    c.bump();
                    let value = multiline(&mut c, start)?;
                    out.push(token(Tok::Str(value, true), start, c.pos));
                } else {
                    out.push(token(Tok::Ident(ident.to_string()), start, c.pos));
                }
            }
            _ => {
                let ch = source[c.pos..].chars().next().unwrap_or('?');
                return Err(ScriptError::at(
                    start,
                    format!("unexpected character {ch:?}"),
                ));
            }
        }
    }
    Ok(out)
}

fn token(tok: Tok, start: Span, end: usize) -> Token {
    Token {
        tok,
        span: Span { end, ..start },
    }
}

/// The body of a `text:` string, cursor just past the colon. RFC 5228 §2.4.2:
/// optional spaces/tabs and an optional `#` comment, a line break, then lines up
/// to one holding only `.`; a leading `..` stands for a literal `.`.
fn multiline(c: &mut Cursor<'_>, start: Span) -> Result<String, ScriptError> {
    while matches!(c.peek(), Some(b' ' | b'\t')) {
        c.bump();
    }
    if c.peek() == Some(b'#') {
        while c.peek().is_some_and(|x| x != b'\n') {
            c.bump();
        }
    }
    if !c.eat_newline() {
        return Err(ScriptError::at(
            c.here(),
            "`text:` must be followed by a line break",
        ));
    }
    let mut value = String::new();
    loop {
        if c.peek().is_none() {
            return Err(ScriptError::at(
                start,
                "unterminated multi-line string: no line holding only `.`",
            ));
        }
        let line_start = c.pos;
        while c.peek().is_some_and(|x| x != b'\n') {
            c.bump();
        }
        let raw = &c.src[line_start..c.pos];
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        let had_newline = c.eat_newline();
        if line == "." {
            return Ok(value);
        }
        let line = line
            .strip_prefix('.')
            .filter(|_| line.starts_with(".."))
            .unwrap_or(line);
        value.push_str(line);
        if had_newline {
            value.push_str("\r\n");
        }
    }
}

// ── parser ─────────────────────────────────────────────────────────────────

struct Parser<'a> {
    toks: &'a [Token],
    pos: usize,
    eof: Span,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&'a Token> {
        self.toks.get(self.pos)
    }

    fn span_here(&self) -> Span {
        self.peek().map_or(self.eof, |t| t.span)
    }

    fn is_punct(&self, p: u8) -> bool {
        matches!(self.peek(), Some(Token { tok: Tok::Punct(x), .. }) if *x == p)
    }

    fn expect_punct(&mut self, p: u8, what: &str) -> Result<Span, ScriptError> {
        if self.is_punct(p) {
            let span = self.toks[self.pos].span;
            self.pos += 1;
            Ok(span)
        } else {
            Err(self.unexpected(what))
        }
    }

    fn unexpected(&self, expected: &str) -> ScriptError {
        match self.peek() {
            None => ScriptError::at(
                self.eof,
                format!("unexpected end of script, expected {expected}"),
            ),
            Some(t) => ScriptError::at(
                t.span,
                format!("expected {expected}, found {}", describe(&t.tok)),
            ),
        }
    }

    fn depth_check(&self, depth: usize) -> Result<(), ScriptError> {
        if depth > MAX_DEPTH {
            Err(ScriptError::at(self.span_here(), "nesting too deep"))
        } else {
            Ok(())
        }
    }

    /// `*command`, up to end of input (top level) or a closing `}` (in a block).
    fn commands(&mut self, depth: usize, in_block: bool) -> Result<Vec<Command>, ScriptError> {
        let mut out = Vec::new();
        loop {
            match self.peek() {
                None if !in_block => return Ok(out),
                None => return Err(self.unexpected("`}`")),
                Some(Token {
                    tok: Tok::Punct(b'}'),
                    ..
                }) if in_block => return Ok(out),
                Some(_) => out.push(self.command(depth)?),
            }
        }
    }

    fn command(&mut self, depth: usize) -> Result<Command, ScriptError> {
        self.depth_check(depth)?;
        let Some(Token {
            tok: Tok::Ident(name),
            span,
        }) = self.peek()
        else {
            return Err(self.unexpected("a command"));
        };
        let (name, start) = (name.clone(), *span);
        self.pos += 1;
        let (arguments, tests, test_list) = self.arguments(depth)?;
        let block = if self.is_punct(b'{') {
            let open = self.expect_punct(b'{', "`{`")?;
            let commands = self.commands(depth + 1, true)?;
            let close = self.expect_punct(b'}', "`}`")?;
            Some(Block {
                commands,
                span: Span {
                    end: close.end,
                    ..open
                },
            })
        } else if self.is_punct(b';') {
            self.pos += 1;
            None
        } else {
            return Err(self.unexpected("`;` or `{`"));
        };
        let end = self.toks[self.pos - 1].span.end;
        Ok(Command {
            name,
            span: Span { end, ..start },
            arguments,
            tests,
            test_list,
            block,
        })
    }

    /// `*argument [ test / test-list ]`.
    #[allow(clippy::type_complexity)]
    fn arguments(&mut self, depth: usize) -> Result<(Vec<Argument>, Vec<Test>, bool), ScriptError> {
        let mut args = Vec::new();
        while let Some(t) = self.peek() {
            match &t.tok {
                Tok::Str(v, multi) => {
                    self.pos += 1;
                    args.push(Argument {
                        kind: ArgumentKind::Strings {
                            values: vec![SieveString {
                                value: v.clone(),
                                span: t.span,
                                multiline: *multi,
                            }],
                            bracketed: false,
                        },
                        span: t.span,
                    });
                }
                Tok::Number(n) => {
                    self.pos += 1;
                    args.push(Argument {
                        kind: ArgumentKind::Number(*n),
                        span: t.span,
                    });
                }
                Tok::Tag(name) => {
                    self.pos += 1;
                    args.push(Argument {
                        kind: ArgumentKind::Tag(name.clone()),
                        span: t.span,
                    });
                }
                Tok::Punct(b'[') => args.push(self.string_list()?),
                _ => break,
            }
        }
        match self.peek().map(|t| &t.tok) {
            Some(Tok::Ident(_)) => {
                let test = self.test(depth + 1)?;
                Ok((args, vec![test], false))
            }
            Some(Tok::Punct(b'(')) => {
                self.pos += 1;
                let mut tests = vec![self.test(depth + 1)?];
                while self.is_punct(b',') {
                    self.pos += 1;
                    tests.push(self.test(depth + 1)?);
                }
                self.expect_punct(b')', "`,` or `)`")?;
                Ok((args, tests, true))
            }
            _ => Ok((args, Vec::new(), false)),
        }
    }

    fn string_list(&mut self) -> Result<Argument, ScriptError> {
        let open = self.expect_punct(b'[', "`[`")?;
        let mut values = Vec::new();
        loop {
            match self.peek() {
                Some(Token {
                    tok: Tok::Str(v, multi),
                    span,
                }) => {
                    values.push(SieveString {
                        value: v.clone(),
                        span: *span,
                        multiline: *multi,
                    });
                    self.pos += 1;
                }
                _ => return Err(self.unexpected("a string")),
            }
            if self.is_punct(b',') {
                self.pos += 1;
                continue;
            }
            let close = self.expect_punct(b']', "`,` or `]`")?;
            return Ok(Argument {
                kind: ArgumentKind::Strings {
                    values,
                    bracketed: true,
                },
                span: Span {
                    end: close.end,
                    ..open
                },
            });
        }
    }

    fn test(&mut self, depth: usize) -> Result<Test, ScriptError> {
        self.depth_check(depth)?;
        let Some(Token {
            tok: Tok::Ident(name),
            span,
        }) = self.peek()
        else {
            return Err(self.unexpected("a test"));
        };
        let (name, start) = (name.clone(), *span);
        self.pos += 1;
        let (arguments, tests, test_list) = self.arguments(depth)?;
        let end = self.toks[self.pos - 1].span.end;
        Ok(Test {
            name,
            span: Span { end, ..start },
            arguments,
            tests,
            test_list,
        })
    }
}

fn describe(tok: &Tok) -> String {
    match tok {
        Tok::Ident(s) => format!("`{s}`"),
        Tok::Tag(s) => format!("`:{s}`"),
        Tok::Number(n) => format!("number {n}"),
        Tok::Str(..) => "a string".to_string(),
        Tok::Punct(p) => format!("`{}`", *p as char),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err(src: &str) -> ScriptError {
        parse_script(src).expect_err("must not parse")
    }

    #[test]
    fn parses_core_shapes() {
        let s = parse_script(
            "require [\"fileinto\", \"copy\"];\r\n\
             # comment\r\n\
             if allof (header :contains \"subject\" \"x\", not exists \"list-id\") {\r\n\
             \x20 fileinto :copy \"A\";\r\n\
             } elsif size :over 10K { discard; } else { keep; }\r\n\
             /* trailing */ stop;\r\n",
        )
        .unwrap();
        assert_eq!(s.commands.len(), 5);
        let req = &s.commands[0];
        assert!(req.is("require"));
        assert_eq!(
            req.arguments[0].strings().unwrap().len(),
            2,
            "bracketed string list"
        );
        let iff = &s.commands[1];
        assert_eq!(iff.tests.len(), 1);
        assert!(iff.tests[0].is("allof"));
        assert!(iff.tests[0].test_list);
        assert_eq!(iff.tests[0].tests.len(), 2);
        assert!(iff.tests[0].tests[1].is("not"));
        assert!(iff.tests[0].tests[1].tests[0].is("exists"));
        let fileinto = &iff.block.as_ref().unwrap().commands[0];
        assert_eq!(fileinto.arguments[0].tag(), Some("copy"));
        let elsif = &s.commands[2];
        assert_eq!(
            elsif.tests[0].arguments[1].kind,
            ArgumentKind::Number(10 * 1024)
        );
        assert!(s.commands[4].is("stop"));
        assert_eq!(s.commands[4].span.line, 6);
        assert_eq!(s.commands[4].span.column, 16);
    }

    #[test]
    fn quantifiers_scale_and_overflow_is_an_error() {
        let s = parse_script("if size :over 2M { keep; } if size :under 1G { keep; }").unwrap();
        assert_eq!(
            s.commands[0].tests[0].arguments[1].kind,
            ArgumentKind::Number(2 << 20)
        );
        assert_eq!(
            s.commands[1].tests[0].arguments[1].kind,
            ArgumentKind::Number(1 << 30)
        );
        assert!(
            err("if size :over 99999999999999999999 { keep; }")
                .message
                .contains("too large")
        );
        assert!(
            err("if size :over 18014398509481984G { keep; }")
                .message
                .contains("too large")
        );
    }

    #[test]
    fn quoted_strings_unescape() {
        let s = parse_script(r#"fileinto "a\"b\\c\d";"#).unwrap();
        assert_eq!(
            s.commands[0].arguments[0].strings().unwrap()[0].value,
            "a\"b\\cd"
        );
    }

    #[test]
    fn multiline_strings_dot_unstuff() {
        let src = "vacation text: # reason\r\nline one\r\n..dotted\r\n.\r\n;\r\n";
        let s = parse_script(src).unwrap();
        let v = &s.commands[0].arguments[0].strings().unwrap()[0];
        assert!(v.multiline);
        assert_eq!(v.value, "line one\r\n.dotted\r\n");
        // Bare LF line endings are accepted the same way.
        let s = parse_script("vacation text:\nhi\n.\n;").unwrap();
        assert_eq!(
            s.commands[0].arguments[0].strings().unwrap()[0].value,
            "hi\r\n"
        );
    }

    #[test]
    fn errors_carry_line_and_column() {
        let e = err("keep;\nfileinto \"a\"\n}");
        assert_eq!((e.line, e.column), (3, 1), "{e}");
        assert!(e.message.contains("expected `;` or `{`"), "{e}");

        let e = err("if true {\n  keep;\n");
        assert!(e.message.contains("unexpected end"), "{e}");

        let e = err("keep;\n  \"stray\";");
        assert_eq!((e.line, e.column), (2, 3), "{e}");

        let e = err("if anyof (true, ) { keep; }");
        assert_eq!(e.column, 17, "{e}");

        let e = err("fileinto \"abc");
        assert_eq!(
            (e.line, e.column, e.message.as_str()),
            (1, 10, "unterminated string")
        );

        let e = err("keep; /* open");
        assert_eq!(e.column, 7, "{e}");

        let e = err("vacation text:\nno end\n");
        assert!(e.message.contains("multi-line"), "{e}");

        let e = err("keep; @");
        assert!(e.message.contains("unexpected character"), "{e}");
    }

    #[test]
    fn columns_count_characters_not_bytes() {
        let e = err("fileinto \"é\" é");
        assert_eq!(e.column, 14, "{e}");
    }

    #[test]
    fn nesting_is_bounded() {
        let deep = format!("{}keep;{}", "if true {".repeat(200), "}".repeat(200));
        assert!(err(&deep).message.contains("too deep"));
        let deep_tests = format!("if {}true{} {{ keep; }}", "not ".repeat(500), "");
        assert!(err(&deep_tests).message.contains("too deep"));
        let shallow = format!("{}keep;{}", "if true {".repeat(20), "}".repeat(20));
        assert!(parse_script(&shallow).is_ok());
    }

    #[test]
    fn spans_cover_the_command() {
        let src = "  fileinto \"A\";  ";
        let s = parse_script(src).unwrap();
        let c = &s.commands[0];
        assert_eq!(&src[c.span.start..c.span.end], "fileinto \"A\";");
    }

    #[test]
    fn empty_and_comment_only_scripts_parse() {
        assert!(parse_script("").unwrap().commands.is_empty());
        assert!(
            parse_script("# only\r\n/* c */\r\n")
                .unwrap()
                .commands
                .is_empty()
        );
    }
}
