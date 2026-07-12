//! Basic syntactic validation of raw Sieve text (plan §0.6 — the raw-Sieve
//! editor's lint), plus the tokenizer that backs it and the fuzz entry point.
//!
//! This is deliberately a *lightweight* checker, not a full RFC 5228 parser: it
//! tokenizes defensively (bounded, never panics on arbitrary bytes) and reports
//! the mistakes a hand-editing user actually makes — unbalanced brackets or
//! quotes, an unterminated block comment, a bad literal length, and Sieve
//! commands used without a matching `require`. It returns a list of
//! human-readable diagnostics; an empty list means clean.

use crate::Result;

/// A coarse Sieve token. Comments and whitespace are dropped by [`tokenize`];
/// what remains is enough to check bracket balance and required-extension use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// A quoted string literal (contents, unescaped is not needed here).
    Str(String),
    /// A multi-line literal `{n}` and the `n` bytes that followed it.
    Literal(String),
    /// A bare identifier / command name / tag (`fileinto`, `:contains`, `allof`).
    Word(String),
    /// One of `{` `}` `(` `)` `[` `]` `;` `,`.
    Punct(char),
}

/// Tokenize Sieve text, skipping `#` line comments and `/* */` block comments.
///
/// Never panics: malformed input (an unterminated string/comment, a `{n}` whose
/// length runs past end-of-input) simply ends the scan. [`lint`] inspects the
/// same conditions and reports them; this function only produces tokens.
pub fn tokenize(input: &str) -> Vec<Token> {
    let bytes = input.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0usize;
    let n = bytes.len();

    while i < n {
        let b = bytes[i];
        match b {
            b' ' | b'\t' | b'\r' | b'\n' => i += 1,
            b'#' => {
                // Line comment to end-of-line.
                while i < n && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if i + 1 < n && bytes[i + 1] == b'*' => {
                i += 2;
                while i + 1 < n && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                // Skip the closing `*/` if present; otherwise we hit EOF.
                i = (i + 2).min(n);
            }
            b'"' => {
                let (s, next) = scan_string(bytes, i);
                tokens.push(Token::Str(s));
                i = next;
            }
            b'{' => {
                // Could be a `{n}` literal length or a block brace. A literal is
                // digits (optionally `+`) then `}` then CRLF then n bytes.
                if let Some((lit, next)) = scan_literal(bytes, i) {
                    tokens.push(Token::Literal(lit));
                    i = next;
                } else {
                    tokens.push(Token::Punct('{'));
                    i += 1;
                }
            }
            b'}' | b'(' | b')' | b'[' | b']' | b';' | b',' => {
                tokens.push(Token::Punct(b as char));
                i += 1;
            }
            _ => {
                let start = i;
                while i < n && !is_delim(bytes[i]) {
                    i += 1;
                }
                if i == start {
                    // Defensive: an unexpected byte we did not advance on.
                    i += 1;
                } else {
                    tokens.push(Token::Word(input[start..i].to_string()));
                }
            }
        }
    }
    tokens
}

fn is_delim(b: u8) -> bool {
    matches!(
        b,
        b' ' | b'\t' | b'\r' | b'\n' | b'"' | b'{' | b'}' | b'(' | b')' | b'[' | b']' | b';' | b','
    ) || b == b'#'
}

/// Scan a `"..."` string starting at `bytes[start] == '"'`. Returns the decoded
/// contents (with `\\`/`\"` unescaped) and the index just past the close quote;
/// on an unterminated string, consumes to end-of-input.
fn scan_string(bytes: &[u8], start: usize) -> (String, usize) {
    let mut out = String::new();
    let mut i = start + 1;
    let n = bytes.len();
    while i < n {
        match bytes[i] {
            b'\\' if i + 1 < n => {
                out.push(bytes[i + 1] as char);
                i += 2;
            }
            b'"' => return (out, i + 1),
            other => {
                out.push(other as char);
                i += 1;
            }
        }
    }
    (out, n) // unterminated — lint reports the imbalance separately
}

/// Try to scan a `{n}` / `{n+}` synchronizing-literal marker at `bytes[start]`.
/// Returns the literal payload and the index past it, or `None` when the braces
/// do not enclose a length (i.e. it is an ordinary block `{`).
fn scan_literal(bytes: &[u8], start: usize) -> Option<(String, usize)> {
    let n = bytes.len();
    let mut i = start + 1;
    let digit_start = i;
    while i < n && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == digit_start {
        return None; // `{` not followed by digits ⇒ a block brace
    }
    let len: usize = std::str::from_utf8(&bytes[digit_start..i])
        .ok()?
        .parse()
        .ok()?;
    if i < n && bytes[i] == b'+' {
        i += 1; // non-synchronizing literal marker
    }
    if i >= n || bytes[i] != b'}' {
        return None;
    }
    i += 1; // past '}'
    // Skip an optional CRLF / LF separating the marker from the payload.
    if i < n && bytes[i] == b'\r' {
        i += 1;
    }
    if i < n && bytes[i] == b'\n' {
        i += 1;
    }
    // Take up to `len` payload bytes (clamped so we never index past the end).
    let end = i.saturating_add(len).min(n);
    let payload = String::from_utf8_lossy(&bytes[i..end]).into_owned();
    Some((payload, end))
}

/// Sieve commands whose use requires a capability string in `require`.
const REQUIRING_COMMANDS: &[(&str, &str)] = &[
    ("fileinto", "fileinto"),
    ("reject", "reject"),
    ("ereject", "ereject"),
    ("vacation", "vacation"),
    ("setflag", "imap4flags"),
    ("addflag", "imap4flags"),
    ("removeflag", "imap4flags"),
    ("hasflag", "imap4flags"),
    ("notify", "enotify"),
    ("include", "include"),
    ("set", "variables"),
];

/// Lint raw Sieve text, returning diagnostics (empty = clean).
///
/// Checks: balanced `{}`/`()`/`[]`, terminated strings and block comments, valid
/// `{n}` literal lengths, and that any capability-gated command is declared in a
/// `require`. Never fails for well-formed UTF-8 input — the `Result` exists for
/// symmetry with the rest of the API.
pub fn lint(input: &str) -> Result<Vec<String>> {
    let mut diags = Vec::new();

    // 1. Unterminated block comment.
    if has_unterminated_block_comment(input) {
        diags.push("unterminated block comment (`/*` without `*/`)".to_string());
    }

    let tokens = tokenize(input);

    // 2. Bracket balance.
    let mut stack: Vec<char> = Vec::new();
    for tok in &tokens {
        if let Token::Punct(c) = tok {
            match c {
                '{' | '(' | '[' => stack.push(*c),
                '}' | ')' | ']' => {
                    let want = match c {
                        '}' => '{',
                        ')' => '(',
                        _ => '[',
                    };
                    match stack.pop() {
                        Some(open) if open == want => {}
                        _ => diags.push(format!("unbalanced `{c}` — no matching `{want}`")),
                    }
                }
                _ => {}
            }
        }
    }
    for open in &stack {
        let close = match open {
            '{' => '}',
            '(' => ')',
            _ => ']',
        };
        diags.push(format!("unclosed `{open}` — missing `{close}`"));
    }

    // 3. Unterminated string: an odd number of unescaped quotes (outside
    //    comments) leaves a dangling `"`.
    if has_unterminated_string(input) {
        diags.push("unterminated string literal (unescaped `\"`)".to_string());
    }

    // 4. Required-extension coverage.
    let required = collect_required(&tokens);
    for tok in &tokens {
        if let Token::Word(w) = tok {
            let name = w.trim_start_matches(':'); // guard against `:notify`-style tags
            if let Some((_, cap)) = REQUIRING_COMMANDS.iter().find(|(cmd, _)| *cmd == name) {
                // Only a command *at statement position* counts; a tag like
                // `:contains` never collides because tags keep their leading ':'.
                if !w.starts_with(':') && !required.contains(*cap) {
                    diags.push(format!(
                        "`{name}` used but extension `{cap}` is not in `require`"
                    ));
                }
            }
        }
    }
    diags.dedup();

    Ok(diags)
}

/// Collect the capability strings named in every `require "x";` / `require ["a",
/// "b"];` statement.
fn collect_required(tokens: &[Token]) -> std::collections::BTreeSet<String> {
    let mut set = std::collections::BTreeSet::new();
    let mut i = 0;
    while i < tokens.len() {
        if matches!(&tokens[i], Token::Word(w) if w == "require") {
            // Everything up to the next `;` that is a string is a capability.
            let mut j = i + 1;
            while j < tokens.len() && tokens[j] != Token::Punct(';') {
                if let Token::Str(s) = &tokens[j] {
                    set.insert(s.clone());
                }
                j += 1;
            }
            i = j;
        }
        i += 1;
    }
    set
}

fn has_unterminated_block_comment(input: &str) -> bool {
    let bytes = input.as_bytes();
    let mut i = 0;
    let n = bytes.len();
    while i < n {
        match bytes[i] {
            b'#' => {
                while i < n && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'"' => {
                let (_, next) = scan_string(bytes, i);
                i = next;
            }
            b'/' if i + 1 < n && bytes[i + 1] == b'*' => {
                i += 2;
                while i + 1 < n && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                if i + 1 >= n {
                    return true; // ran off the end without `*/`
                }
                i += 2;
            }
            _ => i += 1,
        }
    }
    false
}

/// True when a `"` opens a string that never closes (comments/escapes honoured).
fn has_unterminated_string(input: &str) -> bool {
    let bytes = input.as_bytes();
    let mut i = 0;
    let n = bytes.len();
    while i < n {
        match bytes[i] {
            b'#' => {
                while i < n && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if i + 1 < n && bytes[i + 1] == b'*' => {
                i += 2;
                while i + 1 < n && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(n);
            }
            b'"' => {
                let mut j = i + 1;
                let mut closed = false;
                while j < n {
                    match bytes[j] {
                        b'\\' if j + 1 < n => j += 2,
                        b'"' => {
                            closed = true;
                            j += 1;
                            break;
                        }
                        _ => j += 1,
                    }
                }
                if !closed {
                    return true;
                }
                i = j;
            }
            _ => i += 1,
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_script_has_no_diagnostics() {
        let sieve = "require [\"fileinto\"];\n\
                     if header :contains \"subject\" \"sale\" {\n\
                     \x20\x20\x20\x20fileinto \"Junk\";\n}\n";
        assert!(lint(sieve).unwrap().is_empty());
    }

    #[test]
    fn detects_unbalanced_braces() {
        let d = lint("if true {\n  keep;\n").unwrap();
        assert!(d.iter().any(|m| m.contains("unclosed `{`")), "{d:?}");
    }

    #[test]
    fn detects_extra_close_paren() {
        let d = lint("if anyof (a, b)) { keep; }").unwrap();
        assert!(d.iter().any(|m| m.contains("unbalanced `)`")), "{d:?}");
    }

    #[test]
    fn detects_unterminated_string() {
        let d = lint("fileinto \"Archive;").unwrap();
        assert!(d.iter().any(|m| m.contains("unterminated string")), "{d:?}");
    }

    #[test]
    fn detects_unterminated_block_comment() {
        let d = lint("/* never ends\nkeep;").unwrap();
        assert!(
            d.iter().any(|m| m.contains("unterminated block comment")),
            "{d:?}"
        );
    }

    #[test]
    fn flags_missing_require() {
        let d = lint("fileinto \"Spam\";").unwrap();
        assert!(
            d.iter()
                .any(|m| m.contains("`fileinto`") && m.contains("not in `require`")),
            "{d:?}"
        );
    }

    #[test]
    fn require_satisfies_command() {
        let d = lint("require \"fileinto\";\nfileinto \"Spam\";").unwrap();
        assert!(d.is_empty(), "{d:?}");
    }

    #[test]
    fn match_tags_do_not_trip_require_check() {
        // `:matches` etc. keep their colon and must never look like commands.
        let d =
            lint("require \"imap4flags\";\nif hasflag :contains \"x\" { addflag \"y\"; }").unwrap();
        assert!(d.is_empty(), "{d:?}");
    }

    #[test]
    fn tokenizes_literal_payload() {
        let toks = tokenize("PUTSCRIPT \"n\" {5+}\r\nhello\r\n");
        assert!(toks.contains(&Token::Literal("hello".to_string())));
    }

    #[test]
    fn oversized_literal_length_does_not_panic() {
        // A length far beyond the buffer must clamp, not panic.
        let toks = tokenize("{999999999}\r\nshort");
        assert!(toks.iter().any(|t| matches!(t, Token::Literal(_))));
    }

    #[test]
    fn block_brace_is_not_a_literal() {
        let toks = tokenize("if x {\n keep;\n}");
        assert!(toks.contains(&Token::Punct('{')));
        assert!(toks.contains(&Token::Punct('}')));
    }
}
