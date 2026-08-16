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
//! Text terms additionally support (SPEC §10.4):
//! - **fuzzy** — a trailing `~` (`helo~`) does a typo-tolerant match; an
//!   optional edit distance follows (`helo~2`, clamped to 0–2, transpositions
//!   count as one edit). Applies to bare terms and to `field:value` text
//!   operators (`subject:recieve~`). Never inside a quoted phrase.
//! - **prefix / wildcard** — a `*` glob (`proj*`, `*ject`, `p*ct`) matches any
//!   run of characters within a single indexed term. `?` is not special.
//!
//! The parser returns [`SearchError::Parse`] on malformed input and never
//! indexes into byte slices at non-char-boundaries.
//!
//! It is **recursive**, and that is bounded deliberately rather than by luck.
//! `atom := '(' or ')'` and `unary := NOT unary` both descend, and until 26.20
//! nothing capped how far. A stack overflow is not a panic and cannot be caught:
//! it takes the **process** down, every connected user with it. Since
//! `filter.text` on an `Email/query` reaches this parser unbounded, that was a
//! remote denial of service reachable with a small request. [`MAX_DEPTH`] is
//! what closes it, and it is load-bearing — see the note there before changing
//! or removing it.
//!
//! **What was measured, and what was inferred.** The numbers come from a
//! *faithful copy* of this parser — no `use` statements, no external deps —
//! built in a scratch crate on **Windows**, **uninstrumented**, under a release
//! profile matching `fuzz/`: **~975 bytes of stack per nesting level**, linear
//! across 1, 2 and 8 MiB stacks, so roughly **2 187 levels exhaust 2 MiB**. What
//! is *inferred* rather than observed is that a JMAP request is served on a
//! thread with that stack: `mw-server`'s `main.rs` is a plain `#[tokio::main]`
//! and nothing in `crates/` sets a stack size, so 2 MiB is tokio's documented
//! default rather than a value anyone read off a running server. **No
//! measurement was taken through a live server.**
//!
//! **Confirmed live on Linux (26.20), and this is the leg the guard rests on.**
//! In a throwaway `rustlang/rust:nightly` container (rustc 1.100.0-nightly,
//! `x86_64-unknown-linux-gnu`), with [`MAX_DEPTH`] removed and 50 000 nested `(`
//! parsed on a thread created with `stack_size(2 * 1024 * 1024)` — the tokio
//! worker size — the runtime printed `fatal runtime error: stack overflow` and
//! the **process aborted with SIGABRT**. Not a catchable panic, not a failing
//! test: an abort no caller can intercept. With the cap in place the same probe
//! is refused and all 14 `query::` tests pass on Linux, `malformed_never_panics`
//! among them.
//!
//! Three things that run deliberately did **not** establish, so that nobody
//! reports them as more than they were. It was **not** ASan-instrumented —
//! Rust's own stack guard page detects the overflow and aborts, which is the
//! property under test, so ASan would have bought a nicer backtrace and nothing
//! else. It was **not** driven through a real server — an in-process thread
//! sized to match a tokio worker is closer to production than the Windows probe
//! and still not a live JMAP request. And it used 50 000 levels, far above any
//! plausible threshold, so it confirms the **failure class**, not the
//! **boundary**: the ~975 B/level and ~2 187 figures above stay Windows-derived,
//! and per-level stack cost varies with platform, ABI and opt-level.
//!
//! So: the numbers are Windows and isolated; the abort behaviour on a 2 MiB
//! thread is Linux-confirmed; and the cap is justified by the second regardless
//! of the first.
//!
//! Read the hedging as being about the ceiling's exact location, not about
//! whether the cap is needed: [`MAX_DEPTH`] is correct regardless, because
//! nothing on the path from `filter.text` to here bounds input length or nesting
//! depth. The guard does not depend on where the true ceiling sits, so a later
//! measurement that moves the number is not a reason to relax it.
//!
//! **A second, independent bound: [`MAX_QUERY_BYTES`].** Depth is not the only
//! way a query can kill the process. `lex` materialises a `Vec<char>` and a
//! `Vec<Tok>` *before* the parser consults depth, so a flat, un-nested query
//! peaked at **84× its own size** — 176 MB for a 2 MiB body. That is a process
//! kill by *allocation*, and [`MAX_DEPTH`] does nothing about it. Neither bound
//! subsumes the other; both notes say why.
//!
//! Fuzzed by `fuzz/fuzz_targets/search_query.rs` (26.20), which runs the same
//! bounded CI smoke pass as the other targets. Note what that does and does not
//! buy: the fuzzer explores this parser's own behaviour, and it is the depth
//! cap — not the fuzzer — that makes deeply nested input safe.

/// System keyword for a read message (JMAP `$seen`); `is:unread` = its absence.
const KW_SEEN: &str = "$seen";
const KW_FLAGGED: &str = "$flagged";

/// Maximum nesting depth for `(` grouping and `NOT` chains.
///
/// **This bound is the only thing standing between a user-supplied query and a
/// process-killing stack overflow.** The parser is recursive descent; each level
/// costs ~975 bytes of stack, so without a cap 2 187 nested `(` exhausts the
/// 2 MiB stack of the tokio worker serving the request, and a stack overflow
/// cannot be caught — it aborts the process rather than raising a panic. Query
/// text arrives on `Email/query`'s `filter.text` with no length or shape limit.
///
/// 64 is far above anything a person types: the operator grammar nests in
/// single digits, and the deepest query in this crate's own tests is 2. Raising
/// it trades headroom against that overflow, so raise it only with a stack
/// measurement in hand. `deeply_nested_input_is_refused_not_fatal` fails if the
/// cap stops working.
///
/// **[`MAX_QUERY_BYTES`] does not make this redundant**, and neither bound
/// subsumes the other. That cap allows 4 096 bytes, hence up to 4 096 nesting
/// levels — still well above the measured 2 187 that overflows a 2 MiB tokio
/// worker. A 4 KiB query of nothing but `(` would abort the process if this
/// guard were removed.
const MAX_DEPTH: usize = 64;

/// Maximum length, in bytes, of the query text this parser will accept.
///
/// **This bound exists for allocation, not for parsing**, and [`MAX_DEPTH`]
/// does not cover it: the tokens are produced *before* depth is ever consulted,
/// so a query can exhaust memory without ever nesting.
///
/// [`lex`] first collects the whole input into a `Vec<char>` (**4 bytes per
/// ASCII byte**) and then a `Vec<Tok>` (**`size_of::<Tok>()` is 40 bytes**,
/// measured), with word tokens each carrying their own `String`. Measured with
/// a counting allocator against the 2 MiB axum default body limit that
/// `/jmap/api` runs under, one request peaked at:
///
/// | input shape | peak | ratio |
/// |---|---|---|
/// | `((((…` balanced | **176 MB** | **84×** |
/// | `aaaa…` single-char words | 93 MB | 44× |
/// | `from:a from:a …` | 43 MB | 20× |
///
/// Those are **per-request, single-threaded** measurements, which is all that
/// was taken. Nothing on the path bounds how many such requests run at once —
/// verifiable by inspection, since no concurrency limit exists between the route
/// and this parser — so the per-request figure multiplies. **No concurrent
/// exhaustion was actually observed**, and it should not be reported as though
/// it were; the single-request number is the measured fact, and it is a process
/// kill by allocation rather than by stack.
///
/// 4 KiB is far above anything a person types: it is ~600 average words, ~40×
/// the longest query text in this crate's own tests, and enough for well over a
/// hundred `from:someone@example.com OR …` terms. Measured at exactly the cap,
/// the worst shape now peaks at **182 312 B (~178 KB)** per request — the
/// bound, measured rather than extrapolated from the ratio.
///
/// Checked **before** [`lex`] runs, so an oversized query allocates nothing at
/// all rather than being measured after the fact.
/// `oversized_input_is_refused_before_it_is_lexed` fails if the cap stops
/// working.
const MAX_QUERY_BYTES: usize = 4096;

/// Default max edit distance for a bare fuzzy marker (`term~`).
const FUZZY_DEFAULT_DISTANCE: u8 = 1;
/// Upper bound on fuzzy edit distance (Tantivy's Levenshtein automaton caps at
/// 2); larger explicit distances are clamped here.
const FUZZY_MAX_DISTANCE: u8 = 2;

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
    /// Typo-tolerant fuzzy text (`term~` / `term~N`). `distance` is the max
    /// Levenshtein edit distance (0–2; transpositions count as one edit).
    Fuzzy {
        field: TextField,
        value: String,
        distance: u8,
    },
    /// Prefix / wildcard text: a `*` glob in the term (`term*`, `*term`, `t*m`).
    Wildcard { field: TextField, value: String },
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

/// Detect a trailing fuzzy marker on a text term: `term~` (default distance) or
/// `term~N`. Returns `(base_term, distance)`, or `None` when there is no valid
/// marker (a `~` that is not near the end, an empty base, or a non-numeric
/// suffix all stay literal). `distance` is clamped to [`FUZZY_MAX_DISTANCE`].
fn split_fuzzy(value: &str) -> Option<(String, u8)> {
    let tilde = value.rfind('~')?;
    let base = &value[..tilde];
    if base.is_empty() {
        return None;
    }
    let suffix = &value[tilde + 1..];
    let distance = if suffix.is_empty() {
        FUZZY_DEFAULT_DISTANCE
    } else {
        suffix.parse::<u8>().ok()?.min(FUZZY_MAX_DISTANCE)
    };
    Some((base.to_string(), distance))
}

/// Build a text-family [`Clause`] for `field`, detecting a fuzzy marker (`~`)
/// or an embedded wildcard (`*`). A quoted phrase is always literal.
fn text_clause(field: TextField, value: String, phrase: bool) -> Clause {
    if phrase {
        return Clause::Text {
            field,
            value,
            phrase: true,
        };
    }
    if let Some((base, distance)) = split_fuzzy(&value) {
        return Clause::Fuzzy {
            field,
            value: base,
            distance,
        };
    }
    if value.contains('*') {
        return Clause::Wildcard { field, value };
    }
    Clause::Text {
        field,
        value,
        phrase: false,
    }
}

/// Build the leaf [`Expr`] for a `field:value` operator (or a literal term when
/// `field` is not a known operator).
fn operator_atom(field: &str, value: String, phrase: bool) -> Expr {
    let f = field.to_ascii_lowercase();
    let clause = match f.as_str() {
        "from" => text_clause(TextField::From, value, phrase),
        "to" => text_clause(TextField::To, value, phrase),
        "cc" => text_clause(TextField::Cc, value, phrase),
        "subject" => text_clause(TextField::Subject, value, phrase),
        "body" => text_clause(TextField::Body, value, phrase),
        "filename" => text_clause(TextField::Filename, value, phrase),
        "text" => text_clause(TextField::All, value, phrase),
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
                out.push(Tok::Atom(Expr::Clause(text_clause(
                    TextField::All,
                    phrase,
                    true,
                ))));
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
    Tok::Atom(Expr::Clause(text_clause(TextField::All, word, false)))
}

// ---------------------------------------------------------------------------
// Parser (recursive descent over the token stream)
// ---------------------------------------------------------------------------

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
    /// Current recursion depth, capped at [`MAX_DEPTH`].
    depth: usize,
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
    ///
    /// Every recursive path in this parser passes through here — `NOT` chains
    /// directly, and `(` grouping via `parse_atom` → `parse_or` → `parse_and` →
    /// here — so counting depth at this one point bounds both. See [`MAX_DEPTH`].
    fn parse_unary(&mut self) -> Result<Expr, String> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            self.depth -= 1;
            return Err(format!("query nested deeper than {MAX_DEPTH} levels"));
        }
        let out = self.parse_unary_inner();
        self.depth -= 1;
        out
    }

    fn parse_unary_inner(&mut self) -> Result<Expr, String> {
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
    // Before `lex`, deliberately: it collects the whole input into a `Vec<char>`
    // and then a `Vec<Tok>`, so checking afterwards would mean doing the
    // allocation this bound exists to prevent. Every branch of `lex` advances at
    // least one char and pushes at most one token, so bounding the bytes bounds
    // the token count too — one check covers both allocations. See
    // [`MAX_QUERY_BYTES`].
    if text.len() > MAX_QUERY_BYTES {
        return Err(format!(
            "query longer than {MAX_QUERY_BYTES} bytes ({} given)",
            text.len()
        ));
    }
    let toks = lex(text);
    if toks.is_empty() {
        return Ok(Expr::All);
    }
    let mut p = Parser {
        toks,
        pos: 0,
        depth: 0,
    };
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
    fn fuzzy_marker_parses() {
        // Bare `~` uses the default distance.
        assert_eq!(
            cl("helo~"),
            Expr::Clause(Clause::Fuzzy {
                field: TextField::All,
                value: "helo".into(),
                distance: FUZZY_DEFAULT_DISTANCE
            })
        );
        // Explicit distance, on a field operator.
        assert_eq!(
            cl("subject:recieve~2"),
            Expr::Clause(Clause::Fuzzy {
                field: TextField::Subject,
                value: "recieve".into(),
                distance: 2
            })
        );
        // Distance is clamped to the automaton max.
        assert_eq!(
            cl("x~9"),
            Expr::Clause(Clause::Fuzzy {
                field: TextField::All,
                value: "x".into(),
                distance: FUZZY_MAX_DISTANCE
            })
        );
        // A `~` that isn't a trailing marker stays a literal term.
        assert_eq!(
            cl("a~b"),
            Expr::Clause(Clause::Text {
                field: TextField::All,
                value: "a~b".into(),
                phrase: false
            })
        );
    }

    #[test]
    fn wildcard_marker_parses() {
        assert_eq!(
            cl("proj*"),
            Expr::Clause(Clause::Wildcard {
                field: TextField::All,
                value: "proj*".into()
            })
        );
        assert_eq!(
            cl("from:ali*"),
            Expr::Clause(Clause::Wildcard {
                field: TextField::From,
                value: "ali*".into()
            })
        );
        // A quoted phrase keeps `*` literal (no wildcard).
        assert_eq!(
            cl("\"two * words\""),
            Expr::Clause(Clause::Text {
                field: TextField::All,
                value: "two * words".into(),
                phrase: true
            })
        );
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

    /// Regression for the 26.20 stack-overflow DoS: `filter.text` reaches this
    /// parser unbounded from an `Email/query`, and before [`MAX_DEPTH`] existed
    /// ~2 187 nested `(` exhausted a tokio worker's 2 MiB stack — aborting the
    /// **process**, not the request.
    ///
    /// **The depth is `MAX_QUERY_BYTES` levels — the deepest input that can now
    /// reach the parser at all, and 64× [`MAX_DEPTH`].** It is written as the
    /// constant rather than a literal so that raising the length cap
    /// automatically re-points this test at the new worst case instead of
    /// silently leaving it testing a depth nobody can reach any more.
    ///
    /// It used to be a flat 200 000, which stopped testing depth the moment
    /// `MAX_QUERY_BYTES` landed: the length cap refused it first, and the
    /// assertion on *which* guard fired caught that rather than passing on the
    /// wrong one. The 200 000 case now lives in
    /// `oversized_input_is_refused_before_it_is_lexed`, where it belongs.
    ///
    /// **The length cap does not subsume this guard**, which is the reason both
    /// exist: 4 096 bytes of `(` is 4 096 levels, still well above the measured
    /// 2 187 that overflows a 2 MiB tokio worker. Remove [`MAX_DEPTH`] and this
    /// input alone would abort the process.
    ///
    /// A stack overflow cannot be caught in-process, so what is asserted is that
    /// the call **returns at all** — reaching the assertion is itself the result.
    #[test]
    fn deeply_nested_input_is_refused_not_fatal() {
        // Both recursive paths: `(` grouping, and `NOT` chains via `-`.
        for text in ["(".repeat(MAX_QUERY_BYTES), "-".repeat(MAX_QUERY_BYTES)] {
            let err = parse_expr(&text).expect_err("must be refused, not accepted");
            assert!(
                err.contains("nested deeper"),
                "expected the DEPTH guard to be what refused it, not the length \
                 cap — if this reads 'longer than', the two bounds have drifted \
                 and nothing is testing depth any more. Got: {err}"
            );
        }
    }

    /// Regression for the 26.20 allocation DoS, which [`MAX_DEPTH`] does not
    /// cover: `lex` materialises a `Vec<char>` (4 B per ASCII byte) and a
    /// `Vec<Tok>` (40 B per token) **before** the parser ever consults depth, so
    /// a flat, un-nested query could peak at 84× its own size — 176 MB for the
    /// 2 MiB axum default body limit, with nothing bounding concurrency.
    ///
    /// **The input is deliberately 2 MiB — 512× the 4 KiB cap — and that is the
    /// point.** A test just past `MAX_QUERY_BYTES + 1` would keep passing if
    /// someone raised the cap to a value that reopens the hole; this one only
    /// passes while the parser refuses absurd input outright. 2 MiB is chosen
    /// because it is exactly what a client can deliver today.
    #[test]
    fn oversized_input_is_refused_before_it_is_lexed() {
        for text in [
            "(".repeat(2 * 1024 * 1024),
            "a ".repeat(1024 * 1024),
            "x".repeat(2 * 1024 * 1024),
        ] {
            let err = parse_expr(&text).expect_err("must be refused, not lexed");
            assert!(
                err.contains("longer than"),
                "expected the length cap to be what refused it, got: {err}"
            );
        }
    }

    /// Control for the length cap: it must refuse *only* absurd input, not the
    /// long-but-real queries people actually type. Without this, "oversized
    /// input returns Err" is satisfied by a parser that rejects everything.
    #[test]
    fn realistic_long_queries_still_parse() {
        // ~120 addresses OR'd together — a plausible paste, well under the cap.
        let addresses = (0..120)
            .map(|i| format!("from:person{i}@example.com"))
            .collect::<Vec<_>>()
            .join(" OR ");
        assert!(addresses.len() < MAX_QUERY_BYTES);
        assert!(
            parse_expr(&addresses).is_ok(),
            "a {}-byte real-world query must still parse",
            addresses.len()
        );

        // Exactly at the cap is accepted; one byte over is not. Pinning both
        // sides means an off-by-one in the comparison cannot pass unnoticed.
        let at_cap = "a".repeat(MAX_QUERY_BYTES);
        assert!(
            parse_expr(&at_cap).is_ok(),
            "the cap itself must be allowed"
        );
        let over_cap = "a".repeat(MAX_QUERY_BYTES + 1);
        assert!(
            parse_expr(&over_cap).is_err(),
            "one byte over must be refused"
        );
    }

    /// Control for the test above: the guard must refuse *only* absurd input.
    /// Without this, "deeply nested input returns Err" is satisfied by a parser
    /// that rejects every grouped query, and the fix would have broken search
    /// while passing its own regression test.
    #[test]
    fn ordinary_nesting_still_parses() {
        assert!(parse_expr("(a OR b) AND (c OR (d AND -e))").is_ok());
        // Right up to the cap, a well-formed query is still accepted.
        let deep = format!(
            "{}a{}",
            "(".repeat(MAX_DEPTH - 1),
            ")".repeat(MAX_DEPTH - 1)
        );
        assert!(
            parse_expr(&deep).is_ok(),
            "nesting just inside MAX_DEPTH must still parse"
        );
    }
}
