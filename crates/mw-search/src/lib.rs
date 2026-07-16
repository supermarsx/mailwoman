#![forbid(unsafe_code)]
//! `mw-search` — the engine-side full-text index (plan §0.1, §1.1, SPEC §23).
//!
//! Tantivy-backed, written at the single `Engine::ingest` choke point and
//! re-keyed at `store.relocate_message`/`delete_message`. Query operators
//! (`from/to/subject/body/text/has:attachment/filename/before/after/in/
//! is:unread/larger/smaller/tag/pinned`, boolean) parse into a Tantivy query;
//! the searcher returns ordered stable ids. Text terms also support fuzzy
//! (`helo~`, `helo~2` → `FuzzyTermQuery`) and prefix/wildcard (`proj*` →
//! `RegexQuery`) matching (SPEC §10.4). **p95 < 50 ms over 100k** is the
//! timing-harness gate (SPEC §23, `tests/bench.rs`).
//!
//! ## Layering
//! The [`IndexDoc`] DTO is the frozen seam the engine (e9) ingests — a plain
//! struct so this crate stays decoupled from `mw-engine`. [`parse_query`]
//! turns operator text into a [`SearchQuery`]; [`Index::search`] runs it and
//! returns `stableId`s in sort order.

mod query;

pub use query::{Clause, Expr, Sort, SortField, TextField};

use std::ops::Bound;
use std::path::Path;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use tantivy::query::{
    AllQuery, BooleanQuery, EmptyQuery, FuzzyTermQuery, Occur, PhraseQuery, Query, RangeQuery,
    RegexQuery, TermQuery,
};
use tantivy::schema::{
    FAST, Field, INDEXED, IndexRecordOption, STORED, STRING, Schema, TEXT, TantivyDocument, Value,
};
use tantivy::{Index as TantivyIndex, IndexReader, IndexWriter, Order, ReloadPolicy, Term};

/// One document to index, derived from a parsed message at `Engine::ingest`.
///
/// This is a plain DTO (not a `mw-jmap`/engine type) so e1 can build the index
/// in parallel without depending on the engine crate (plan §3 e1). The engine
/// fills one of these per message and hands it across the seam.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IndexDoc {
    /// The store's opaque `stable_id` — the index's primary key (plan §1.4).
    pub stable_id: String,
    pub account_id: String,
    pub mailbox_id: String,
    pub from: String,
    pub to: String,
    pub cc: String,
    pub subject: String,
    pub body: String,
    /// `receivedAt` as a Unix timestamp (seconds), for `before:`/`after:`.
    pub date: i64,
    pub has_attachment: bool,
    /// JMAP keywords (labels + system flags) — backs `tag:`/`is:unread`.
    pub keywords: Vec<String>,
    pub size: u64,
    /// Attachment filenames — backs `filename:`.
    pub filenames: Vec<String>,
    /// Engine-local pin flag (plan §1.5) — backs `pinned:`.
    pub pinned: bool,
}

/// A parsed operator query (plan §0.1) plus its result ordering.
///
/// Build via [`parse_query`] (from operator text) or [`SearchQuery::all`];
/// set ordering with [`SearchQuery::with_sort`]. `raw` preserves the source
/// text for logging/round-tripping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchQuery {
    /// The raw operator text as typed (`from:a subject:"hi" has:attachment`).
    pub raw: String,
    /// Parsed boolean AST.
    pub expr: Expr,
    /// Result ordering (defaults to `receivedAt` desc).
    pub sort: Sort,
}

impl Default for SearchQuery {
    fn default() -> Self {
        SearchQuery::all()
    }
}

impl SearchQuery {
    /// A match-everything query in default (`receivedAt` desc) order.
    pub fn all() -> Self {
        SearchQuery {
            raw: String::new(),
            expr: Expr::All,
            sort: Sort::default(),
        }
    }

    /// Override the result ordering.
    pub fn with_sort(mut self, sort: Sort) -> Self {
        self.sort = sort;
        self
    }
}

/// Parse operator text into a [`SearchQuery`] (plan §0.1). Empty text matches
/// everything. Never panics (the parser is fuzzed, plan §1.12).
pub fn parse_query(text: &str) -> Result<SearchQuery> {
    let expr = query::parse_expr(text).map_err(SearchError::Parse)?;
    Ok(SearchQuery {
        raw: text.to_string(),
        expr,
        sort: Sort::default(),
    })
}

#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("tantivy error: {0}")]
    Tantivy(String),
    #[error("query parse error: {0}")]
    Parse(String),
    #[error("io error: {0}")]
    Io(String),
}

impl From<tantivy::TantivyError> for SearchError {
    fn from(e: tantivy::TantivyError) -> Self {
        SearchError::Tantivy(e.to_string())
    }
}

impl From<std::io::Error> for SearchError {
    fn from(e: std::io::Error) -> Self {
        SearchError::Io(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, SearchError>;

/// Resolved schema field handles (built once at open).
struct Fields {
    stable_id: Field,
    account_id: Field,
    mailbox: Field,
    from: Field,
    to: Field,
    cc: Field,
    subject: Field,
    body: Field,
    filename: Field,
    keywords: Field,
    from_sort: Field,
    subject_sort: Field,
    date: Field,
    size: Field,
    has_attachment: Field,
    pinned: Field,
    doc_json: Field,
}

/// Fast-field names used for sorting (must match the schema below).
const F_DATE: &str = "date";
const F_SIZE: &str = "size";
const F_FROM_SORT: &str = "from_sort";
const F_SUBJECT_SORT: &str = "subject_sort";

/// Cap for an unbounded (`limit == 0`) search, bounding the collector heap.
const MAX_HITS: usize = 100_000;

fn build_schema() -> (Schema, Fields) {
    let mut sb = Schema::builder();
    let stable_id = sb.add_text_field("stable_id", STRING | STORED);
    let account_id = sb.add_text_field("account_id", STRING);
    let mailbox = sb.add_text_field("mailbox", STRING);
    let from = sb.add_text_field("from", TEXT);
    let to = sb.add_text_field("to", TEXT);
    let cc = sb.add_text_field("cc", TEXT);
    let subject = sb.add_text_field("subject", TEXT);
    let body = sb.add_text_field("body", TEXT);
    let filename = sb.add_text_field("filename", TEXT);
    let keywords = sb.add_text_field("keywords", STRING);
    // Raw, lowercased, fast string columns used purely for `from`/`subject` sort.
    let from_sort = sb.add_text_field(F_FROM_SORT, STRING | FAST);
    let subject_sort = sb.add_text_field(F_SUBJECT_SORT, STRING | FAST);
    let date = sb.add_i64_field(F_DATE, INDEXED | FAST);
    let size = sb.add_u64_field(F_SIZE, INDEXED | FAST);
    let has_attachment = sb.add_u64_field("has_attachment", INDEXED);
    let pinned = sb.add_u64_field("pinned", INDEXED);
    let doc_json = sb.add_text_field("doc_json", STORED);
    let schema = sb.build();
    let fields = Fields {
        stable_id,
        account_id,
        mailbox,
        from,
        to,
        cc,
        subject,
        body,
        filename,
        keywords,
        from_sort,
        subject_sort,
        date,
        size,
        has_attachment,
        pinned,
        doc_json,
    };
    (schema, fields)
}

/// The search index handle: open once per store data dir, mutate at ingest /
/// relocate / delete, query on `Email/query`.
pub struct Index {
    inner: TantivyIndex,
    reader: IndexReader,
    writer: Mutex<IndexWriter>,
    fields: Fields,
}

impl Index {
    /// Open (or create) the index under the store's protected data dir. The
    /// index holds plaintext-derived terms and is documented as auto-excluded
    /// from the V6 zero-access surface (plan §6 risk #3).
    pub fn open(data_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(data_dir)?;
        let (schema, fields) = build_schema();
        let dir = MmapDirectory::open(data_dir)
            .map_err(|e| SearchError::Io(format!("open index dir: {e}")))?;
        let inner = TantivyIndex::open_or_create(dir, schema)?;
        Self::from_index(inner, fields)
    }

    /// Open an ephemeral in-RAM index (tests + the 100k timing harness).
    pub fn open_in_ram() -> Result<Self> {
        let (schema, fields) = build_schema();
        let inner = TantivyIndex::create_in_ram(schema);
        Self::from_index(inner, fields)
    }

    fn from_index(inner: TantivyIndex, fields: Fields) -> Result<Self> {
        let writer: IndexWriter = inner.writer::<TantivyDocument>(50_000_000)?;
        let reader = inner
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()?;
        Ok(Index {
            inner,
            reader,
            writer: Mutex::new(writer),
            fields,
        })
    }

    fn lock_writer(&self) -> Result<std::sync::MutexGuard<'_, IndexWriter>> {
        self.writer
            .lock()
            .map_err(|_| SearchError::Io("index writer poisoned".to_string()))
    }

    fn to_document(&self, doc: &IndexDoc) -> Result<TantivyDocument> {
        let f = &self.fields;
        let mut td = TantivyDocument::default();
        td.add_text(f.stable_id, &doc.stable_id);
        td.add_text(f.account_id, &doc.account_id);
        td.add_text(f.mailbox, &doc.mailbox_id);
        td.add_text(f.from, &doc.from);
        td.add_text(f.to, &doc.to);
        td.add_text(f.cc, &doc.cc);
        td.add_text(f.subject, &doc.subject);
        td.add_text(f.body, &doc.body);
        for name in &doc.filenames {
            td.add_text(f.filename, name);
        }
        for kw in &doc.keywords {
            td.add_text(f.keywords, kw);
        }
        td.add_text(f.from_sort, doc.from.to_lowercase());
        td.add_text(f.subject_sort, doc.subject.to_lowercase());
        td.add_i64(f.date, doc.date);
        td.add_u64(f.size, doc.size);
        td.add_u64(f.has_attachment, u64::from(doc.has_attachment));
        td.add_u64(f.pinned, u64::from(doc.pinned));
        let json = serde_json::to_string(doc)
            .map_err(|e| SearchError::Io(format!("serialize IndexDoc: {e}")))?;
        td.add_text(f.doc_json, json);
        Ok(td)
    }

    /// Delete-then-add so a stable id is never duplicated (Tantivy has no
    /// in-place update). Caller commits + reloads.
    fn write_one(&self, writer: &IndexWriter, doc: &IndexDoc) -> Result<()> {
        writer.delete_term(Term::from_field_text(self.fields.stable_id, &doc.stable_id));
        let td = self.to_document(doc)?;
        writer.add_document(td)?;
        Ok(())
    }

    /// Add or replace a document (called at `Engine::ingest`).
    pub fn upsert(&self, doc: &IndexDoc) -> Result<()> {
        {
            let mut w = self.lock_writer()?;
            self.write_one(&w, doc)?;
            w.commit()?;
        }
        self.reader.reload()?;
        Ok(())
    }

    /// Bulk add/replace with a single commit — used by the store's initial
    /// re-index and the 100k timing harness.
    pub fn upsert_batch(&self, docs: &[IndexDoc]) -> Result<()> {
        {
            let mut w = self.lock_writer()?;
            for doc in docs {
                self.write_one(&w, doc)?;
            }
            w.commit()?;
        }
        self.reader.reload()?;
        Ok(())
    }

    /// Delete a document by stable id (called at `store.delete_message`).
    pub fn delete(&self, stable_id: &str) -> Result<()> {
        {
            let mut w = self.lock_writer()?;
            w.delete_term(Term::from_field_text(self.fields.stable_id, stable_id));
            w.commit()?;
        }
        self.reader.reload()?;
        Ok(())
    }

    /// Re-key a document onto a new mailbox after a stable-id-preserving move
    /// (called at `store.relocate_message`, plan §1.4). Preserves every other
    /// field by reconstructing from the stored `doc_json`.
    pub fn relocate(&self, stable_id: &str, new_mailbox_id: &str) -> Result<()> {
        let Some(mut doc) = self.fetch_doc(stable_id)? else {
            return Ok(()); // Nothing indexed under this id yet — no-op.
        };
        doc.mailbox_id = new_mailbox_id.to_string();
        self.upsert(&doc)
    }

    /// Reconstruct the [`IndexDoc`] stored under `stable_id`, if present.
    fn fetch_doc(&self, stable_id: &str) -> Result<Option<IndexDoc>> {
        let searcher = self.reader.searcher();
        let q = TermQuery::new(
            Term::from_field_text(self.fields.stable_id, stable_id),
            IndexRecordOption::Basic,
        );
        let hits = searcher.search(&q, &TopDocs::with_limit(1).order_by_score())?;
        let Some((_, addr)) = hits.first() else {
            return Ok(None);
        };
        let td: TantivyDocument = searcher.doc(*addr)?;
        let Some(json) = td.get_first(self.fields.doc_json).and_then(|v| v.as_str()) else {
            return Ok(None);
        };
        Ok(serde_json::from_str(json).ok())
    }

    /// Run a parsed query, returning matching stable ids in sort order. `limit`
    /// of `0` means "all matches" (capped at [`MAX_HITS`]).
    pub fn search(&self, query: &SearchQuery, limit: usize) -> Result<Vec<String>> {
        let searcher = self.reader.searcher();
        let compiled = compile(&query.expr, &self.fields);
        let cap = if limit == 0 { MAX_HITS } else { limit };
        let order = if query.sort.ascending {
            Order::Asc
        } else {
            Order::Desc
        };

        let addrs: Vec<tantivy::DocAddress> = match query.sort.field {
            SortField::ReceivedAt => searcher
                .search(
                    &compiled,
                    &TopDocs::with_limit(cap).order_by_fast_field::<i64>(F_DATE, order),
                )?
                .into_iter()
                .map(|(_, a)| a)
                .collect(),
            SortField::Size => searcher
                .search(
                    &compiled,
                    &TopDocs::with_limit(cap).order_by_fast_field::<u64>(F_SIZE, order),
                )?
                .into_iter()
                .map(|(_, a)| a)
                .collect(),
            SortField::From => searcher
                .search(
                    &compiled,
                    &TopDocs::with_limit(cap).order_by_string_fast_field(F_FROM_SORT, order),
                )?
                .into_iter()
                .map(|(_, a)| a)
                .collect(),
            SortField::Subject => searcher
                .search(
                    &compiled,
                    &TopDocs::with_limit(cap).order_by_string_fast_field(F_SUBJECT_SORT, order),
                )?
                .into_iter()
                .map(|(_, a)| a)
                .collect(),
        };

        let mut ids = Vec::with_capacity(addrs.len());
        for addr in addrs {
            let td: TantivyDocument = searcher.doc(addr)?;
            if let Some(id) = td.get_first(self.fields.stable_id).and_then(|v| v.as_str()) {
                ids.push(id.to_string());
            }
        }
        Ok(ids)
    }

    /// Force any pending writes to be visible to subsequent searches.
    pub fn reload(&self) -> Result<()> {
        self.reader.reload()?;
        Ok(())
    }

    /// The number of live (committed, non-deleted) documents.
    pub fn num_docs(&self) -> u64 {
        self.reader.searcher().num_docs()
    }

    /// Access the underlying Tantivy index (schema introspection/tests).
    pub fn tantivy(&self) -> &TantivyIndex {
        &self.inner
    }
}

// ---------------------------------------------------------------------------
// AST → Tantivy query compilation
// ---------------------------------------------------------------------------

/// Replicate Tantivy's `default` tokenizer (split on non-alphanumeric,
/// lowercase, drop tokens > 40 bytes) so query terms match indexed terms.
fn tokenize(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty() && t.len() <= 40)
        .map(|t| t.to_lowercase())
        .collect()
}

fn compile(expr: &Expr, f: &Fields) -> Box<dyn Query> {
    match expr {
        Expr::All => Box::new(AllQuery),
        Expr::Clause(c) => clause_query(c, f),
        Expr::And(children) => {
            let subs = children
                .iter()
                .map(|e| (Occur::Must, compile(e, f)))
                .collect();
            Box::new(BooleanQuery::new(subs))
        }
        Expr::Or(children) => {
            let subs = children
                .iter()
                .map(|e| (Occur::Should, compile(e, f)))
                .collect();
            Box::new(BooleanQuery::new(subs))
        }
        Expr::Not(inner) => negate(compile(inner, f)),
    }
}

/// `NOT q` — a must-not needs a positive clause to select the candidate set.
fn negate(q: Box<dyn Query>) -> Box<dyn Query> {
    Box::new(BooleanQuery::new(vec![
        (Occur::Must, Box::new(AllQuery) as Box<dyn Query>),
        (Occur::MustNot, q),
    ]))
}

fn text_field(f: &Fields, tf: TextField) -> Vec<Field> {
    match tf {
        TextField::From => vec![f.from],
        TextField::To => vec![f.to],
        TextField::Cc => vec![f.cc],
        TextField::Subject => vec![f.subject],
        TextField::Body => vec![f.body],
        TextField::Filename => vec![f.filename],
        TextField::All => vec![f.from, f.to, f.cc, f.subject, f.body, f.filename],
    }
}

fn clause_query(c: &Clause, f: &Fields) -> Box<dyn Query> {
    match c {
        Clause::Text {
            field,
            value,
            phrase,
        } => {
            let tokens = tokenize(value);
            if tokens.is_empty() {
                return Box::new(EmptyQuery);
            }
            fan_out(text_field(f, *field), |fld| {
                field_text_query(fld, &tokens, *phrase)
            })
        }
        Clause::Fuzzy {
            field,
            value,
            distance,
        } => {
            let tokens = tokenize(value);
            if tokens.is_empty() {
                return Box::new(EmptyQuery);
            }
            fan_out(text_field(f, *field), |fld| {
                fuzzy_field_query(fld, &tokens, *distance)
            })
        }
        Clause::Wildcard { field, value } => {
            let pattern = wildcard_pattern(value);
            if pattern.is_empty() {
                return Box::new(EmptyQuery);
            }
            fan_out(text_field(f, *field), |fld| {
                wildcard_field_query(fld, &pattern)
            })
        }
        Clause::Keyword(kw) => term_query(f.keywords, kw),
        Clause::NotKeyword(kw) => negate(term_query(f.keywords, kw)),
        Clause::Mailbox(m) => term_query(f.mailbox, m),
        Clause::HasAttachment(b) => u64_term(f.has_attachment, u64::from(*b)),
        Clause::Pinned(b) => u64_term(f.pinned, u64::from(*b)),
        Clause::DateRange { after, before } => {
            let lower = after
                .map(|v| Bound::Included(Term::from_field_i64(f.date, v)))
                .unwrap_or(Bound::Unbounded);
            let upper = before
                .map(|v| Bound::Excluded(Term::from_field_i64(f.date, v)))
                .unwrap_or(Bound::Unbounded);
            if matches!(
                (lower.as_ref(), upper.as_ref()),
                (Bound::Unbounded, Bound::Unbounded)
            ) {
                return Box::new(EmptyQuery);
            }
            Box::new(RangeQuery::new(lower, upper))
        }
        Clause::SizeRange { larger, smaller } => {
            let lower = larger
                .map(|v| Bound::Excluded(Term::from_field_u64(f.size, v)))
                .unwrap_or(Bound::Unbounded);
            let upper = smaller
                .map(|v| Bound::Excluded(Term::from_field_u64(f.size, v)))
                .unwrap_or(Bound::Unbounded);
            if matches!(
                (lower.as_ref(), upper.as_ref()),
                (Bound::Unbounded, Bound::Unbounded)
            ) {
                return Box::new(EmptyQuery);
            }
            Box::new(RangeQuery::new(lower, upper))
        }
    }
}

/// Fan a per-field query builder across one or more fields: a single field
/// yields that query directly; several (the `TextField::All` set) are OR'd.
fn fan_out(fields: Vec<Field>, mut make: impl FnMut(Field) -> Box<dyn Query>) -> Box<dyn Query> {
    let mut per_field: Vec<(Occur, Box<dyn Query>)> = fields
        .into_iter()
        .map(|fld| (Occur::Should, make(fld)))
        .collect();
    if per_field.len() == 1 {
        per_field.pop().expect("len == 1").1
    } else {
        Box::new(BooleanQuery::new(per_field))
    }
}

fn field_text_query(field: Field, tokens: &[String], phrase: bool) -> Box<dyn Query> {
    if phrase && tokens.len() > 1 {
        let terms = tokens
            .iter()
            .map(|t| Term::from_field_text(field, t))
            .collect();
        Box::new(PhraseQuery::new(terms))
    } else if tokens.len() == 1 {
        Box::new(TermQuery::new(
            Term::from_field_text(field, &tokens[0]),
            IndexRecordOption::WithFreqs,
        ))
    } else {
        // Multiple tokens, not a phrase: require all (AND).
        let subs = tokens
            .iter()
            .map(|t| {
                (
                    Occur::Must,
                    Box::new(TermQuery::new(
                        Term::from_field_text(field, t),
                        IndexRecordOption::WithFreqs,
                    )) as Box<dyn Query>,
                )
            })
            .collect();
        Box::new(BooleanQuery::new(subs))
    }
}

/// Typo-tolerant query for one field: a `FuzzyTermQuery` per token (usually a
/// single token), AND'd together. Transpositions count as one edit.
fn fuzzy_field_query(field: Field, tokens: &[String], distance: u8) -> Box<dyn Query> {
    let fuzzy = |t: &str| -> Box<dyn Query> {
        Box::new(FuzzyTermQuery::new(
            Term::from_field_text(field, t),
            distance,
            true,
        ))
    };
    if tokens.len() == 1 {
        fuzzy(&tokens[0])
    } else {
        let subs = tokens
            .iter()
            .map(|t| (Occur::Must, fuzzy(t)))
            .collect::<Vec<_>>();
        Box::new(BooleanQuery::new(subs))
    }
}

/// Translate a user wildcard term into a Tantivy `RegexQuery` pattern that
/// matches a whole indexed term: `*` → `.*`, letters are lowercased to match
/// the indexed (lowercased) tokens, every other char is regex-escaped. Since
/// indexed terms are alphanumeric, escaped punctuation simply matches nothing.
fn wildcard_pattern(value: &str) -> String {
    let mut out = String::new();
    for c in value.chars() {
        if c == '*' {
            out.push_str(".*");
        } else if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
        } else {
            out.push('\\');
            out.push(c);
        }
    }
    out
}

/// Prefix/wildcard query for one field. A malformed pattern (rejected by the
/// FST regex compiler) degrades to an empty match rather than an error.
fn wildcard_field_query(field: Field, pattern: &str) -> Box<dyn Query> {
    match RegexQuery::from_pattern(pattern, field) {
        Ok(q) => Box::new(q),
        Err(_) => Box::new(EmptyQuery),
    }
}

fn term_query(field: Field, value: &str) -> Box<dyn Query> {
    Box::new(TermQuery::new(
        Term::from_field_text(field, value),
        IndexRecordOption::Basic,
    ))
}

fn u64_term(field: Field, value: u64) -> Box<dyn Query> {
    Box::new(TermQuery::new(
        Term::from_field_u64(field, value),
        IndexRecordOption::Basic,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(id: &str, subject: &str, from: &str) -> IndexDoc {
        IndexDoc {
            stable_id: id.to_string(),
            account_id: "acct".to_string(),
            mailbox_id: "INBOX".to_string(),
            from: from.to_string(),
            subject: subject.to_string(),
            body: subject.to_string(),
            ..IndexDoc::default()
        }
    }

    fn seeded() -> Index {
        let idx = Index::open_in_ram().expect("open ram index");
        idx.upsert_batch(&[
            doc("m1", "Quarterly report ready", "alice@example.com"),
            doc("m2", "Project status update", "bob@acme.org"),
            doc("m3", "Lunch on Friday", "carol@vendor.net"),
        ])
        .expect("index");
        idx
    }

    fn find(idx: &Index, q: &str) -> Vec<String> {
        let query = parse_query(q).expect("parse");
        let mut ids = idx.search(&query, 0).expect("search");
        ids.sort();
        ids
    }

    #[test]
    fn fuzzy_tolerates_a_typo() {
        let idx = seeded();
        // One-edit typo (`reprot` -> `report`) still matches.
        assert_eq!(find(&idx, "subject:reprot~"), vec!["m1".to_string()]);
        // A transposition counts as a single edit.
        assert_eq!(find(&idx, "subject:qaurterly~"), vec!["m1".to_string()]);
        // Two edits need an explicit distance.
        assert_eq!(find(&idx, "subject:proejct~2"), vec!["m2".to_string()]);
        // Distance 0 is exact: the typo no longer matches.
        assert!(find(&idx, "subject:reprot~0").is_empty());
    }

    #[test]
    fn wildcard_matches_prefix_and_middle() {
        let idx = seeded();
        // Prefix.
        assert_eq!(find(&idx, "subject:proj*"), vec!["m2".to_string()]);
        // Trailing-wildcard prefix on a field operator.
        assert_eq!(find(&idx, "from:ali*"), vec!["m1".to_string()]);
        // `*` in the middle of a term.
        assert_eq!(find(&idx, "subject:qu*ly"), vec!["m1".to_string()]);
        // Bare-term wildcard fans across all text fields.
        assert_eq!(find(&idx, "quarter*"), vec!["m1".to_string()]);
    }

    #[test]
    fn exact_terms_still_match() {
        let idx = seeded();
        assert_eq!(find(&idx, "subject:report"), vec!["m1".to_string()]);
        assert!(find(&idx, "subject:reprot").is_empty());
        assert_eq!(find(&idx, "subject:project"), vec!["m2".to_string()]);
    }
}
