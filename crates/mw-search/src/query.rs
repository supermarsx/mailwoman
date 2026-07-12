//! Operator query parser + AST (plan §0.1).
//!
//! Grammar (case-insensitive `AND`/`OR`/`NOT`, implicit `AND` between adjacent
//! atoms, `(`/`)` grouping, quoted phrases, `-atom` = `NOT atom`):
//!
//! ```text
//! or    := and (OR and)*
//! and   := unary (AND? unary)*
//! unary := NOT unary | '-' unary | atom
//! atom  := '(' or ')' | field ':' value | WORD | '"' PHRASE '"'
//! ```
//!
//! Supported operators (plan §0.1 / §2.1):
//! `from: to: cc: subject: body:` (field text) · `text:`/bare (all-fields) ·
//! `has:attachment` · `filename:` · `before:`/`after:` (date) · `in:` (mailbox) ·
//! `is:unread|read|flagged|unflagged|pinned` · `larger:`/`smaller:` (size) ·
//! `tag:` (keyword) · `pinned:true|false`.
//!
//! The parser is **panic-free** (fuzzed, plan §1.12): it only ever returns
//! [`SearchError::Parse`] on malformed input and never indexes into byte slices
//! at non-char-boundaries.

/// System keyword for a read message (JMAP `$seen`); `is:unread` = its absence.
const KW_SEEN: &str = "$seen";
const KW_FLAGGED: &str = "$flagged";

/// Which field(s) a text clause searches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextField {
    From,
    To,
    Cc,
    Subject,
    Body,
    Filename,
    /// All user-visible text fields (`text:` / bare terms).
    All,
}

/// A single leaf predicate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Clause {
    /// Free/field text; `phrase` forces adjacency.
    Text {
        field: TextField,
        value: String,
        phrase: bool,
    },
    /// Exact JMAP keyword present (`tag:`, `is:read`, `is:flagged`).
    Keyword(String),
    /// Exact JMAP keyword absent (`is:unread` = not `$seen`).
    NotKeyword(String),
    /// `in:` — message lives in this mailbox id.
    Mailbox(String),
    /// `has:attachment`.
    HasAttachment(bool),
    /// `pinned:` / `is:pinned`.
    Pinned(bool),
    /// `before:`/`after:` — Unix-second bounds (`after` inclusive, `before` exclusive).
    DateRange {
        after: Option<i64>,
        before: Option<i64>,
    },
    /// `larger:`/`smaller:` — strict byte bounds.
    SizeRange {
        larger: Option<u64>,
        smaller: Option<u64>,
    },
}

/// Boolean AST over [`Clause`]s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    /// Matches every document (empty query).
    All,
    Clause(Clause),
    And(Vec<Expr>),
    Or(Vec<Expr>),
    Not(Box<Expr>),
}

/// Field a result set is ordered by (plan §2.1 sort set).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    ReceivedAt,
    Size,
    From,
    Subject,
}

/// Result ordering. `receivedAt` defaults to descending (newest first); the
/// string/size sorts default to ascending.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sort {
    pub field: SortField,
    pub ascending: bool,
}

impl Sort {
    /// The plan's default: newest received first.
    pub fn received_desc() -> Self {
        Sort {
            field: SortField::ReceivedAt,
            ascending: false,
        }
    }

    /// Natural default ordering for a field (`receivedAt` desc, others asc).
    pub fn for_field(field: SortField) -> Self {
        Sort {
            field,
            ascending: !matches!(field, SortField::ReceivedAt),
        }
    }
}

impl Default for Sort {
    fn default() -> Self {
        Sort::received_desc()
    }
}

// ---------------------------------------------------------------------------
// Lexer
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    LParen,
    RParen,
    And,
    Or,
    Not,
    Atom(Expr),
}

/// Known operator prefixes (`name:`), so an unknown `foo:bar` stays a literal term.
fn is_operator(name: &str) -> bool {
    matches!(
        name,
        "from"
            | "to"
            | "cc"
            | "subject"
            | "body"
            | "text"
            | "has"
            | "filename"
            | "before"
            | "after"
            | "in"
            | "is"
            | "larger"
            | "smaller"
            | "tag"
            | "pinned"
    )
}

/// Parse a `YYYY-MM-DD` (or bare unix-seconds) date to a Unix timestamp at UTC
/// midnight. Returns `None` on malformed input (never panics).
fn parse_date(s: &str) -> Option<i64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    // Bare integer = already unix seconds.
    if let Ok(secs) = s.parse::<i64>() {
        return Some(secs);
    }
    let mut parts = s.split('-');
    let y: i64 = parts.next()?.parse().ok()?;
    let m: i64 = parts.next()?.parse().ok()?;
    let d: i64 = parts.next()?.parse().ok()?;
    if parts.next().is_some() || !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    // Days-from-civil (Howard Hinnant's algorithm) — leap-year correct, no deps.
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    let days = era * 146097 + doe - 719468;
    Some(days * 86400)
}

/// Parse a size with an optional `k`/`m`/`g` suffix (1000-based). `None` on error.
fn parse_size(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let (num, mult) = match s.chars().last() {
        Some('k') | Some('K') => (&s[..s.len() - 1], 1_000u64),
        Some('m') | Some('M') => (&s[..s.len() - 1], 1_000_000),
        Some('g') | Some('G') => (&s[..s.len() - 1], 1_000_000_000),
        _ => (s, 1),
    };
    num.trim().parse::<u64>().ok()?.checked_mul(mult)
}

/// Build the leaf [`Expr`] for a `field:value` operator (or a literal term when
/// `field` is not a known operator).
fn operator_atom(field: &str, value: String, phrase: bool) -> Expr {
    let f = field.to_ascii_lowercase();
    let clause = match f.as_str() {
        "from" => Clause::Text {
            field: TextField::From,
            value,
            phrase,
        },
        "to" => Clause::Text {
            field: TextField::To,
            value,
            phrase,
        },
        "cc" => Clause::Text {
            field: TextField::Cc,
            value,
            phrase,
        },
        "subject" => Clause::Text {
            field: TextField::Subject,
            value,
            phrase,
        },
        "body" => Clause::Text {
            field: TextField::Body,
            value,
            phrase,
        },
        "filename" => Clause::Text {
            field: TextField::Filename,
            value,
            phrase,
        },
        "text" => Clause::Text {
            field: TextField::All,
            value,
            phrase,
        },
        "tag" => Clause::Keyword(value),
        "in" => Clause::Mailbox(value),
        "has" => Clause::HasAttachment(value.eq_ignore_ascii_case("attachment")),
        "pinned" => Clause::Pinned(!value.eq_ignore_ascii_case("false") && value != "0"),
        "is" => match value.to_ascii_lowercase().as_str() {
            "unread" => Clause::NotKeyword(KW_SEEN.to_string()),
            "read" => Clause::Keyword(KW_SEEN.to_string()),
            "flagged" | "starred" => Clause::Keyword(KW_FLAGGED.to_string()),
            "unflagged" => Clause::NotKeyword(KW_FLAGGED.to_string()),
            "pinned" => Clause::Pinned(true),
            // Unknown `is:x` — treat the token as a keyword filter.
            other => Clause::Keyword(other.to_string()),
        },
        "before" => Clause::DateRange {
            after: None,
            before: parse_date(&value),
        },
        "after" => Clause::DateRange {
            after: parse_date(&value),
            before: None,
        },
        "larger" => Clause::SizeRange {
            larger: parse_size(&value),
            smaller: None,
        },
        "smaller" => Clause::SizeRange {
            larger: None,
            smaller: parse_size(&value),
        },
        // Not a known operator: keep the whole `field:value` as a literal term.
        _ => Clause::Text {
            field: TextField::All,
            value: format!("{field}:{value}"),
            phrase,
        },
    };
    Expr::Clause(clause)
}

/// Split raw text into tokens, honouring quotes, parens, and `field:value`.
fn lex(input: &str) -> Vec<Tok> {
    let chars: Vec<char> = input.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        match c {
            '(' => {
                out.push(Tok::LParen);
                i += 1;
            }
            ')' => {
                out.push(Tok::RParen);
                i += 1;
            }
            '-' => {
                // Leading '-' negates the following atom (only when it prefixes one).
                if i + 1 < chars.len() && !chars[i + 1].is_whitespace() && chars[i + 1] != ')' {
                    out.push(Tok::Not);
                    i += 1;
                } else {
                    i += 1;
                }
            }
            '"' => {
                let (phrase, next) = read_quoted(&chars, i + 1);
                out.push(Tok::Atom(Expr::Clause(Clause::Text {
                    field: TextField::All,
                    value: phrase,
                    phrase: true,
                })));
                i = next;
            }
            _ => {
                let (word, next) = read_word(&chars, i);
                i = next;
                out.push(classify_word(&chars, word, &mut i));
            }
        }
    }
    out
}

/// Read a run of non-space, non-paren, non-quote chars starting at `start`.
fn read_word(chars: &[char], start: usize) -> (String, usize) {
    let mut i = start;
    let mut s = String::new();
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() || c == '(' || c == ')' || c == '"' {
            break;
        }
        s.push(c);
        i += 1;
    }
    (s, i)
}

/// Read a quoted phrase body (the opening quote already consumed). Tolerates a
/// missing closing quote (runs to end of input).
fn read_quoted(chars: &[char], start: usize) -> (String, usize) {
    let mut i = start;
    let mut s = String::new();
    while i < chars.len() {
        if chars[i] == '"' {
            i += 1;
            break;
        }
        s.push(chars[i]);
        i += 1;
    }
    (s, i)
}

/// Turn a bare word into a keyword token or an atom. Handles `field:value` and
/// `field:"quoted value"` (the quote is consumed via `i`).
fn classify_word(chars: &[char], word: String, i: &mut usize) -> Tok {
    match word.as_str() {
        "AND" | "and" | "&&" => return Tok::And,
        "OR" | "or" | "||" => return Tok::Or,
        "NOT" | "not" => return Tok::Not,
        _ => {}
    }
    if let Some(colon) = word.find(':') {
        let (field, rest) = word.split_at(colon);
        let value = &rest[1..]; // skip ':'
        if is_operator(&field.to_ascii_lowercase()) {
            // `field:` with an immediately-following quoted value.
            if value.is_empty() && *i < chars.len() && chars[*i] == '"' {
                let (phrase, next) = read_quoted(chars, *i + 1);
                *i = next;
                return Tok::Atom(operator_atom(field, phrase, true));
            }
            return Tok::Atom(operator_atom(field, value.to_string(), false));
        }
    }
    Tok::Atom(Expr::Clause(Clause::Text {
        field: TextField::All,
        value: word,
        phrase: false,
    }))
}

// ---------------------------------------------------------------------------
// Parser (recursive descent over the token stream)
// ---------------------------------------------------------------------------

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn bump(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    /// `or := and (OR and)*`
    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut terms = vec![self.parse_and()?];
        while matches!(self.peek(), Some(Tok::Or)) {
            self.bump();
            terms.push(self.parse_and()?);
        }
        Ok(if terms.len() == 1 {
            terms.pop().expect("len checked")
        } else {
            Expr::Or(terms)
        })
    }

    /// `and := unary (AND? unary)*` — adjacency is implicit AND.
    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut terms = vec![self.parse_unary()?];
        loop {
            match self.peek() {
                Some(Tok::And) => {
                    self.bump();
                    terms.push(self.parse_unary()?);
                }
                // Implicit AND: another atom/NOT/'(' follows without OR/')'.
                Some(Tok::Not) | Some(Tok::LParen) | Some(Tok::Atom(_)) => {
                    terms.push(self.parse_unary()?);
                }
                _ => break,
            }
        }
        Ok(if terms.len() == 1 {
            terms.pop().expect("len checked")
        } else {
            Expr::And(terms)
        })
    }

    /// `unary := NOT unary | atom`
    fn parse_unary(&mut self) -> Result<Expr, String> {
        if matches!(self.peek(), Some(Tok::Not)) {
            self.bump();
            let inner = self.parse_unary()?;
            return Ok(Expr::Not(Box::new(inner)));
        }
        self.parse_atom()
    }

    /// `atom := '(' or ')' | ATOM`
    fn parse_atom(&mut self) -> Result<Expr, String> {
        match self.bump() {
            Some(Tok::LParen) => {
                let inner = self.parse_or()?;
                match self.bump() {
                    Some(Tok::RParen) => Ok(inner),
                    _ => Err("unbalanced parenthesis".to_string()),
                }
            }
            Some(Tok::Atom(e)) => Ok(e),
            Some(Tok::RParen) => Err("unexpected ')'".to_string()),
            Some(Tok::And) | Some(Tok::Or) => Err("dangling boolean operator".to_string()),
            Some(Tok::Not) => Err("dangling NOT".to_string()),
            None => Err("unexpected end of query".to_string()),
        }
    }
}

/// Parse operator text into an [`Expr`]. Empty/whitespace input → [`Expr::All`].
pub(crate) fn parse_expr(text: &str) -> Result<Expr, String> {
    let toks = lex(text);
    if toks.is_empty() {
        return Ok(Expr::All);
    }
    let mut p = Parser { toks, pos: 0 };
    let expr = p.parse_or()?;
    if p.pos != p.toks.len() {
        return Err("trailing tokens after query".to_string());
    }
    Ok(expr)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cl(text: &str) -> Expr {
        parse_expr(text).expect("parse")
    }

    #[test]
    fn empty_is_all() {
        assert_eq!(cl(""), Expr::All);
        assert_eq!(cl("   "), Expr::All);
    }

    #[test]
    fn bare_term_is_all_fields() {
        assert_eq!(
            cl("hello"),
            Expr::Clause(Clause::Text {
                field: TextField::All,
                value: "hello".into(),
                phrase: false
            })
        );
    }

    #[test]
    fn field_operators() {
        assert_eq!(
            cl("from:alice"),
            Expr::Clause(Clause::Text {
                field: TextField::From,
                value: "alice".into(),
                phrase: false
            })
        );
        assert_eq!(
            cl("has:attachment"),
            Expr::Clause(Clause::HasAttachment(true))
        );
        assert_eq!(
            cl("in:INBOX"),
            Expr::Clause(Clause::Mailbox("INBOX".into()))
        );
        assert_eq!(cl("tag:Work"), Expr::Clause(Clause::Keyword("Work".into())));
        assert_eq!(
            cl("is:unread"),
            Expr::Clause(Clause::NotKeyword("$seen".into()))
        );
        assert_eq!(cl("pinned:true"), Expr::Clause(Clause::Pinned(true)));
        assert_eq!(cl("pinned:false"), Expr::Clause(Clause::Pinned(false)));
    }

    #[test]
    fn quoted_field_phrase() {
        assert_eq!(
            cl("subject:\"quarterly report\""),
            Expr::Clause(Clause::Text {
                field: TextField::Subject,
                value: "quarterly report".into(),
                phrase: true
            })
        );
    }

    #[test]
    fn date_and_size_ranges() {
        assert_eq!(
            cl("after:2020-01-01"),
            Expr::Clause(Clause::DateRange {
                after: Some(1_577_836_800),
                before: None
            })
        );
        assert_eq!(
            cl("larger:1m"),
            Expr::Clause(Clause::SizeRange {
                larger: Some(1_000_000),
                smaller: None
            })
        );
    }

    #[test]
    fn boolean_precedence() {
        // a b  =>  AND(a,b)
        assert!(matches!(cl("a b"), Expr::And(v) if v.len() == 2));
        // a OR b  => OR
        assert!(matches!(cl("a OR b"), Expr::Or(v) if v.len() == 2));
        // a AND b OR c  => OR(AND(a,b), c)
        match cl("a AND b OR c") {
            Expr::Or(v) => {
                assert_eq!(v.len(), 2);
                assert!(matches!(v[0], Expr::And(_)));
            }
            other => panic!("expected OR, got {other:?}"),
        }
    }

    #[test]
    fn not_and_minus() {
        assert!(matches!(cl("NOT spam"), Expr::Not(_)));
        assert!(matches!(cl("-spam"), Expr::Not(_)));
        // from:a -from:b  => AND(from:a, NOT from:b)
        match cl("from:a -from:b") {
            Expr::And(v) => {
                assert_eq!(v.len(), 2);
                assert!(matches!(v[1], Expr::Not(_)));
            }
            other => panic!("expected AND, got {other:?}"),
        }
    }

    #[test]
    fn parens_group() {
        match cl("(a OR b) c") {
            Expr::And(v) => {
                assert_eq!(v.len(), 2);
                assert!(matches!(v[0], Expr::Or(_)));
            }
            other => panic!("expected AND, got {other:?}"),
        }
    }

    #[test]
    fn unknown_colon_is_literal() {
        assert_eq!(
            cl("weird:thing"),
            Expr::Clause(Clause::Text {
                field: TextField::All,
                value: "weird:thing".into(),
                phrase: false
            })
        );
    }

    #[test]
    fn malformed_never_panics() {
        // These must return Err, not panic.
        for q in ["(", ")", "a OR", "AND b", "((()", "\"unterminated", "from:"] {
            let _ = parse_expr(q);
        }
    }
}
