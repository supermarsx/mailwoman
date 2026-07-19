//! Bridging helpers between the engine and `mw-search` (plan §1.1, §3 e9):
//! building an [`IndexDoc`] from a parsed message at ingest, and compiling the
//! frozen [`EmailFilter`] + sort into a `mw-search` [`SearchQuery`].
//!
//! Free functions (no engine state) so both `engine.rs` (ingest) and `jmap.rs`
//! (`Email/query`) can share them.

use mail_parser::{MessageParser, MimeHeaders};
use mw_jmap::{Email, EmailAddress};
use mw_search::{Clause, Expr, IndexDoc, SearchQuery, Sort, SortField, TextField};

use crate::query::{Comparator, EmailFilter, SortProperty};

/// Join a JMAP address list into one indexable string (`"Name <email>"`).
fn addr_str(list: &Option<Vec<EmailAddress>>) -> String {
    let Some(list) = list else {
        return String::new();
    };
    list.iter()
        .map(|a| match &a.name {
            Some(n) if !n.is_empty() => format!("{n} {}", a.email),
            _ => a.email.clone(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Concatenate a message's decoded text bodies for the full-text `body:` field.
fn body_text(email: &Email) -> String {
    let mut out = String::new();
    for v in email.body_values.values() {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(&v.value);
    }
    out
}

/// RFC3339 → Unix seconds (0 on absent/unparseable), for the `date` fast field.
fn unix_secs(rfc3339: Option<&str>) -> i64 {
    rfc3339
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.timestamp())
        .unwrap_or(0)
}

/// Attachment filenames from raw RFC822 bytes, for the `filename:` operator.
/// Best-effort and panic-free (`mail-parser` is extremely lenient).
pub(crate) fn attachment_filenames(raw: &[u8]) -> Vec<String> {
    let Some(msg) = MessageParser::default().parse(raw) else {
        return Vec::new();
    };
    msg.attachments()
        .filter_map(|p| p.attachment_name().map(str::to_string))
        .collect()
}

/// W19: decoded text of a message's `text/*` attachments, concatenated, for
/// full-text search over attachment content. Binary attachments (PDF/DOCX/images)
/// yield no text here and are skipped — extracting them needs format parsers that
/// are out of this milestone's scope. Best-effort + panic-free. Bounded so one
/// pathological message can't bloat the index.
pub(crate) fn attachment_text(raw: &[u8]) -> String {
    /// Cap on total indexed attachment text per message (2 MiB of tokens is ample).
    const MAX_ATTACH_TEXT: usize = 2 * 1024 * 1024;
    let Some(msg) = MessageParser::default().parse(raw) else {
        return String::new();
    };
    let mut out = String::new();
    for part in msg.attachments() {
        let Some(text) = part.text_contents() else {
            continue; // not a text part (binary attachment) — skip.
        };
        if !out.is_empty() {
            out.push(' ');
        }
        let remaining = MAX_ATTACH_TEXT.saturating_sub(out.len());
        if remaining == 0 {
            break;
        }
        // Push at most `remaining` bytes, respecting UTF-8 char boundaries.
        let take = text
            .char_indices()
            .take_while(|(i, _)| *i < remaining)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);
        out.push_str(&text[..take]);
    }
    out
}

/// Build the `mw-search` [`IndexDoc`] for one message at `Engine::ingest`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_index_doc(
    stable_id: &str,
    account_id: &str,
    mailbox_id: &str,
    email: &Email,
    keywords: Vec<String>,
    filenames: Vec<String>,
    attachment_text: String,
    pinned: bool,
) -> IndexDoc {
    IndexDoc {
        stable_id: stable_id.to_string(),
        account_id: account_id.to_string(),
        mailbox_id: mailbox_id.to_string(),
        from: addr_str(&email.from),
        to: addr_str(&email.to),
        cc: addr_str(&email.cc),
        subject: email.subject.clone().unwrap_or_default(),
        body: body_text(email),
        date: unix_secs(email.received_at.as_deref()),
        has_attachment: email.has_attachment,
        keywords,
        size: email.size,
        filenames,
        attachment_text,
        pinned,
    }
}

/// Map the frozen [`SortProperty`] + comparator direction onto a `mw-search`
/// [`Sort`]. Defaults to `receivedAt` desc when no comparator is supplied.
pub(crate) fn sort_from_comparator(cmp: Option<&Comparator>) -> Sort {
    match cmp {
        None => Sort::received_desc(),
        Some(c) => {
            let field = match c.property {
                SortProperty::ReceivedAt => SortField::ReceivedAt,
                SortProperty::Size => SortField::Size,
                SortProperty::From => SortField::From,
                SortProperty::Subject => SortField::Subject,
            };
            Sort {
                field,
                ascending: c.is_ascending,
            }
        }
    }
}

/// Compile the frozen [`EmailFilter`] + `sort` into a `mw-search` query,
/// scoped to the querying account's mailboxes so a global full-text search
/// cannot leak another account's ids (the index is shared per engine).
///
/// `account_mailboxes` are every mailbox id the account owns; `scope_mailbox`
/// is the single `inMailbox` id when the filter pins one (the common case).
pub(crate) fn build_search_query(
    filter: &EmailFilter,
    sort: Sort,
    account_mailboxes: &[String],
    scope_mailbox: Option<&str>,
) -> SearchQuery {
    let mut clauses: Vec<Expr> = Vec::new();

    // Account/mailbox scope.
    match scope_mailbox {
        Some(mb) => clauses.push(Expr::Clause(Clause::Mailbox(mb.to_string()))),
        None => {
            let scope: Vec<Expr> = account_mailboxes
                .iter()
                .map(|m| Expr::Clause(Clause::Mailbox(m.clone())))
                .collect();
            if !scope.is_empty() {
                clauses.push(Expr::Or(scope));
            }
        }
    }
    if let Some(exclude) = &filter.in_mailbox_other_than {
        for mb in exclude {
            clauses.push(Expr::Not(Box::new(Expr::Clause(Clause::Mailbox(
                mb.clone(),
            )))));
        }
    }

    let text = |field: TextField, value: &str| {
        Expr::Clause(Clause::Text {
            field,
            value: value.to_string(),
            phrase: false,
        })
    };
    if let Some(v) = &filter.text {
        // The web packs the whole operator string (`subject:foo`,
        // `from:anna has:attachment`, booleans, quoted phrases) into
        // `filter.text`. Route it through the shared operator parser instead of
        // treating it as one literal all-fields term — otherwise `subject:foo`
        // searches for the literal words "subject" and "foo" across all fields
        // and every operator query returns nothing. A bare word still parses to
        // an all-fields clause, so plain full-text search is preserved. On a
        // parse error, fall back to the old literal all-fields term.
        match mw_search::parse_query(v) {
            Ok(parsed) if !matches!(parsed.expr, Expr::All) => clauses.push(parsed.expr),
            Ok(_) => {} // empty / whitespace-only text — no added constraint
            Err(_) => clauses.push(text(TextField::All, v)),
        }
    }
    if let Some(v) = &filter.from {
        clauses.push(text(TextField::From, v));
    }
    if let Some(v) = &filter.to {
        clauses.push(text(TextField::To, v));
    }
    if let Some(v) = &filter.cc {
        clauses.push(text(TextField::Cc, v));
    }
    if let Some(v) = &filter.subject {
        clauses.push(text(TextField::Subject, v));
    }
    if let Some(v) = &filter.body {
        clauses.push(text(TextField::Body, v));
    }
    if let Some(v) = &filter.filename {
        clauses.push(text(TextField::Filename, v));
    }
    if let Some(b) = filter.has_attachment {
        clauses.push(Expr::Clause(Clause::HasAttachment(b)));
    }
    if let Some(k) = &filter.has_keyword {
        clauses.push(Expr::Clause(Clause::Keyword(k.clone())));
    }
    if let Some(k) = &filter.not_keyword {
        clauses.push(Expr::Clause(Clause::NotKeyword(k.clone())));
    }
    if filter.before.is_some() || filter.after.is_some() {
        clauses.push(Expr::Clause(Clause::DateRange {
            after: filter.after.as_deref().and_then(parse_date_bound),
            before: filter.before.as_deref().and_then(parse_date_bound),
        }));
    }
    if filter.min_size.is_some() || filter.max_size.is_some() {
        clauses.push(Expr::Clause(Clause::SizeRange {
            larger: filter.min_size,
            smaller: filter.max_size,
        }));
    }

    let expr = match clauses.len() {
        0 => Expr::All,
        1 => clauses.pop().expect("len == 1"),
        _ => Expr::And(clauses),
    };
    SearchQuery {
        raw: String::new(),
        expr,
        sort,
    }
}

/// Parse a filter date bound (RFC3339 or `YYYY-MM-DD` or bare unix-seconds).
fn parse_date_bound(s: &str) -> Option<i64> {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Some(dt.timestamp());
    }
    if let Ok(secs) = s.parse::<i64>() {
        return Some(secs);
    }
    // `YYYY-MM-DD` at UTC midnight.
    let d = chrono::NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").ok()?;
    Some(d.and_hms_opt(0, 0, 0)?.and_utc().timestamp())
}
