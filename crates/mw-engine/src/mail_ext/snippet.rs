//! `SearchSnippet/get` (RFC 8621 §5): for each requested email, return the
//! subject + a body preview with the query's free-text terms wrapped in
//! `<mark></mark>`. Terms come from the same `filter` an `Email/query` uses; the
//! text is HTML-escaped before the marks are inserted (the snippets are HTML).

use serde_json::{Value, json};

use crate::backend::{EngineError, Result};
use crate::engine::Engine;
use crate::query::EmailFilter;

use super::server_fail;

impl Engine {
    /// `SearchSnippet/get` (RFC 8621 §5.1).
    pub(crate) async fn search_snippet_get(&self, account_id: &str, args: &Value) -> Value {
        let filter: EmailFilter = args
            .get("filter")
            .cloned()
            .and_then(|f| serde_json::from_value(f).ok())
            .unwrap_or_default();
        let terms = filter_terms(&filter);

        let empty = Vec::new();
        let email_ids = args
            .get("emailIds")
            .and_then(Value::as_array)
            .unwrap_or(&empty);

        let mut list = Vec::new();
        let mut not_found = Vec::new();
        for id in email_ids.iter().filter_map(Value::as_str) {
            match self.email_envelope(account_id, id).await {
                Ok(Some(email)) => {
                    let subject = email.subject.unwrap_or_default();
                    let preview = email.preview.unwrap_or_default();
                    list.push(json!({
                        "emailId": id,
                        "subject": highlight(&subject, &terms),
                        "preview": highlight(&preview, &terms),
                    }));
                }
                Ok(None) => not_found.push(json!(id)),
                Err(e) => return server_fail(e),
            }
        }
        json!({
            "accountId": account_id,
            "list": list,
            "notFound": not_found
        })
    }

    /// Load an email's stored (sealed) envelope, or re-parse its body when no
    /// envelope was cached. Account-scoped: an id owned by another account (or
    /// unknown) resolves to `None`.
    async fn email_envelope(
        &self,
        account_id: &str,
        stable_id: &str,
    ) -> Result<Option<mw_jmap::Email>> {
        match self.store().get_message(stable_id).await {
            Ok(m) if m.account_id != account_id => return Ok(None),
            Ok(_) => {}
            Err(mw_store::StoreError::NotFound) => return Ok(None),
            Err(e) => return Err(EngineError::Store(e)),
        }
        if let Some(bytes) = self.store().get_envelope(stable_id).await? {
            return Ok(serde_json::from_slice(&bytes).ok());
        }
        if let Some(blob) = self.fetch_blob(account_id, stable_id).await? {
            return Ok(mw_mime::parse(&blob.bytes).ok().map(|p| p.email));
        }
        Ok(None)
    }
}

/// The free-text search terms a snippet highlights: the words of the filter's
/// text/subject/body/from/to/cc conditions, deduped case-insensitively, ≥2 chars.
fn filter_terms(f: &EmailFilter) -> Vec<String> {
    let mut terms: Vec<String> = Vec::new();
    for v in [&f.text, &f.subject, &f.body, &f.from, &f.to, &f.cc]
        .into_iter()
        .flatten()
    {
        for w in v.split_whitespace() {
            if w.len() >= 2 && !terms.iter().any(|t| t.eq_ignore_ascii_case(w)) {
                terms.push(w.to_string());
            }
        }
    }
    terms
}

/// HTML-escape `text` and wrap case-insensitive (ASCII-folded) occurrences of
/// any `term` in `<mark></mark>`. Matches are found on the raw text and escaped
/// per-segment, so the marks never land inside an entity or nest.
fn highlight(text: &str, terms: &[String]) -> Value {
    if terms.is_empty() {
        return json!(html_escape(text));
    }
    let ranges = find_matches(text, terms);
    if ranges.is_empty() {
        return json!(html_escape(text));
    }
    let mut out = String::new();
    let mut pos = 0;
    for (start, end) in ranges {
        if start > pos {
            out.push_str(&html_escape(&text[pos..start]));
        }
        out.push_str("<mark>");
        out.push_str(&html_escape(&text[start..end]));
        out.push_str("</mark>");
        pos = end;
    }
    if pos < text.len() {
        out.push_str(&html_escape(&text[pos..]));
    }
    json!(out)
}

/// Byte ranges of every term match in `text` (ASCII case-insensitive), merged so
/// overlapping/adjacent matches don't produce nested marks.
fn find_matches(text: &str, terms: &[String]) -> Vec<(usize, usize)> {
    let tb = text.as_bytes();
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    for term in terms {
        let term_b = term.as_bytes();
        if term_b.is_empty() {
            continue;
        }
        let mut i = 0;
        while i + term_b.len() <= tb.len() {
            if tb[i..i + term_b.len()].eq_ignore_ascii_case(term_b)
                && text.is_char_boundary(i)
                && text.is_char_boundary(i + term_b.len())
            {
                ranges.push((i, i + term_b.len()));
                i += term_b.len();
            } else {
                i += 1;
            }
        }
    }
    ranges.sort_unstable();
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (start, end) in ranges {
        match merged.last_mut() {
            Some(last) if start <= last.1 => last.1 = last.1.max(end),
            _ => merged.push((start, end)),
        }
    }
    merged
}

/// Minimal HTML text escaping for snippet output.
fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}
