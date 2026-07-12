#![forbid(unsafe_code)]
//! `mw-search` — the engine-side full-text index (plan §0.1, §1.1, SPEC §23).
//!
//! Tantivy-backed, written at the single `Engine::ingest` choke point and
//! re-keyed at `store.relocate_message`/`delete_message`. Query operators
//! (`from/to/subject/body/text/has:attachment/filename/before/after/in/
//! is:unread/larger/smaller/tag/pinned`, boolean) parse into a Tantivy query;
//! the searcher returns ordered stable ids. **p95 < 50 ms over 100k** is the
//! `cargo bench` gate (SPEC §23).
//!
//! ## Scaffolder note (e0)
//! e0 authors ONLY this frozen seam: the [`IndexDoc`] DTO the engine ingests
//! and the [`Index`] handle shape. e1 owns the entire crate — Tantivy schema,
//! [`Indexer`]/`Searcher`, the operator query parser (+ its fuzz target), and
//! the 100k bench. Bodies are `todo!()`; **do not** implement logic here.

use serde::{Deserialize, Serialize};

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

/// A parsed operator query (plan §0.1). e1 owns the concrete AST + parser.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchQuery {
    /// The raw operator text as typed (`from:a subject:"hi" has:attachment`).
    pub raw: String,
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

pub type Result<T> = std::result::Result<T, SearchError>;

/// The search index handle: open once per store data dir, mutate at ingest /
/// relocate / delete, query on `Email/query`.
pub struct Index {
    _private: (),
}

impl Index {
    /// Open (or create) the index under the store's protected data dir. The
    /// index holds plaintext-derived terms and is documented as auto-excluded
    /// from the V6 zero-access surface (plan §6 risk #3).
    #[allow(unused_variables)]
    pub fn open(data_dir: &std::path::Path) -> Result<Self> {
        todo!("e1: build the Tantivy schema + open/create the index")
    }

    /// Add or replace a document (called at `Engine::ingest`).
    #[allow(unused_variables)]
    pub fn upsert(&self, doc: &IndexDoc) -> Result<()> {
        todo!("e1")
    }

    /// Delete a document by stable id (called at `store.delete_message`).
    #[allow(unused_variables)]
    pub fn delete(&self, stable_id: &str) -> Result<()> {
        todo!("e1")
    }

    /// Re-key a document onto a new mailbox after a stable-id-preserving move
    /// (called at `store.relocate_message`, plan §1.4).
    #[allow(unused_variables)]
    pub fn relocate(&self, stable_id: &str, new_mailbox_id: &str) -> Result<()> {
        todo!("e1")
    }

    /// Run a parsed query, returning matching stable ids in result order.
    #[allow(unused_variables)]
    pub fn search(&self, query: &SearchQuery, limit: usize) -> Result<Vec<String>> {
        todo!("e1")
    }
}

/// Parse operator text into a [`SearchQuery`] (plan §0.1). e1 owns the grammar
/// and its `cargo-fuzz` target (plan §1.12).
#[allow(unused_variables)]
pub fn parse_query(text: &str) -> Result<SearchQuery> {
    todo!("e1: operator query parser")
}
