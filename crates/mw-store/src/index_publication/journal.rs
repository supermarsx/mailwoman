//! Crash-replay journal for search-index publication (migration 0116).
//!
//! Rows are appended only by triggers on `messages`, `message_meta` and
//! `message_body_cache_lease`, inside the transaction of the source change, and
//! only while the control row is enabled. There is no append method: every SQL
//! writer, including raw SQL outside this crate, journals the same way, as long as
//! it uses ordinary DML with `recursive_triggers=ON` (the Store's own setting).
//!
//! A row is a "reconcile this key to current source" hint, not a delta. `mode` is
//! advisory; a replayer must reconcile every row against current SQL source.
//!
//! Acknowledgement protocol (SQLite only):
//! 1. call [`Store::index_journal_max_seq`] and keep the value as `upto`;
//! 2. only then read the source and publish;
//! 3. after the index commit, call [`Store::index_journal_complete`] with `upto`.
//!
//! SQLite has one writer, so seq order is commit order: any change committed after
//! step 1 gets a seq above `upto` and survives step 3. PostgreSQL identity values
//! are assigned at INSERT time, so that argument fails there, and enabling the
//! journal on PostgreSQL is refused.
use crate::{
    Store, StoreError,
    backend::{Dialect, q},
};

/// Upper bound on rows per page and keys per acknowledgement call. Callers with
/// more keys split the work; a partly acknowledged group only leaves rows to replay.
pub const MAX_INDEX_JOURNAL_BATCH: usize = 4096;
const MAX_KEY_BYTES: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexJournalMode {
    /// Only metadata columns (flags, mailbox, size, date, pin, follow-up) changed.
    Metadata,
    /// Content, owner or key may have changed, or the row was inserted.
    Reconcile,
    /// The source row was deleted or moved to another key.
    Delete,
}
impl IndexJournalMode {
    fn parse(value: &str) -> Result<Self, StoreError> {
        match value {
            "metadata" => Ok(Self::Metadata),
            "reconcile" => Ok(Self::Reconcile),
            "delete" => Ok(Self::Delete),
            _ => Err(StoreError::Corrupt(
                "index journal mode is unknown".to_owned(),
            )),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct IndexJournalEntry {
    pub seq: i64,
    pub account_id: String,
    pub stable_id: String,
    pub mode: IndexJournalMode,
}
impl std::fmt::Debug for IndexJournalEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndexJournalEntry")
            .field("seq", &self.seq)
            .field("mode", &self.mode)
            .finish_non_exhaustive()
    }
}

fn invalid(message: &'static str) -> StoreError {
    StoreError::Invalid(message)
}

impl Store {
    /// Turn trigger capture on or off. Durable and shared by every process on this
    /// database. Turning it off does not delete rows; replay them first.
    /// Enabling is refused on PostgreSQL (see the module documentation).
    pub async fn set_index_journal_enabled(&self, enabled: bool) -> Result<(), StoreError> {
        if enabled && self.backend.dialect() == Dialect::Postgres {
            return Err(invalid(
                "index publication journal is SQLite-only: PostgreSQL sequence order is not commit order",
            ));
        }
        q("INSERT INTO index_publication_journal_control(id,enabled) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET enabled=excluded.enabled")
            .bind(i64::from(enabled))
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Whether triggers currently append. An absent control row means disabled.
    pub async fn index_journal_enabled(&self) -> Result<bool, StoreError> {
        let enabled = q("SELECT CAST(COALESCE(MAX(enabled),0) AS BIGINT) AS n FROM index_publication_journal_control WHERE id=1")
            .fetch_scalar_i64(&self.backend)
            .await?;
        Ok(enabled == 1)
    }

    /// Highest committed seq, or 0 when the journal is empty. Capture this BEFORE
    /// reading the source that an acknowledgement will cover.
    pub async fn index_journal_max_seq(&self) -> Result<i64, StoreError> {
        Ok(
            q("SELECT CAST(COALESCE(MAX(seq),0) AS BIGINT) AS n FROM index_publication_journal")
                .fetch_scalar_i64(&self.backend)
                .await?,
        )
    }

    /// Rows with `seq > after_seq` in seq order, at most `limit`
    /// (1..=[`MAX_INDEX_JOURNAL_BATCH`]). Page with the last returned seq.
    pub async fn index_journal_page(
        &self,
        after_seq: i64,
        limit: usize,
    ) -> Result<Vec<IndexJournalEntry>, StoreError> {
        if limit == 0 || limit > MAX_INDEX_JOURNAL_BATCH {
            return Err(invalid("index journal page limit out of range"));
        }
        let rows = q("SELECT seq,account_id,stable_id,mode FROM index_publication_journal WHERE seq>?1 ORDER BY seq LIMIT ?2")
            .bind(after_seq)
            .bind(limit as i64)
            .fetch_all(&self.backend)
            .await?;
        rows.iter()
            .map(|row| {
                Ok(IndexJournalEntry {
                    seq: row.get_i64("seq"),
                    account_id: row.get_string("account_id"),
                    stable_id: row.get_string("stable_id"),
                    mode: IndexJournalMode::parse(&row.get_string("mode"))?,
                })
            })
            .collect()
    }

    /// Acknowledge published keys: for each `(account_id, stable_id, upto_seq)`,
    /// delete that key's rows with `seq <= upto_seq`. Newer rows for the same key
    /// stay. One statement, so all keys are acknowledged atomically. Returns the
    /// number of rows deleted.
    pub async fn index_journal_complete(
        &self,
        acknowledged: &[(&str, &str, i64)],
    ) -> Result<u64, StoreError> {
        if acknowledged.is_empty() {
            return Ok(0);
        }
        if acknowledged.len() > MAX_INDEX_JOURNAL_BATCH {
            return Err(invalid("index journal acknowledgement batch too large"));
        }
        if acknowledged.iter().any(|(account, id, _)| {
            account.is_empty()
                || id.is_empty()
                || account.len() > MAX_KEY_BYTES
                || id.len() > MAX_KEY_BYTES
        }) {
            return Err(invalid("index journal acknowledgement key out of range"));
        }
        let payload = serde_json::to_string(
            &acknowledged
                .iter()
                .map(|(account, id, upto)| serde_json::json!([account, id, upto]))
                .collect::<Vec<_>>(),
        )
        .map_err(|_| {
            StoreError::Corrupt("index journal acknowledgement encoding failed".to_owned())
        })?;
        let sql = match self.backend.dialect() {
            Dialect::Sqlite => {
                "DELETE FROM index_publication_journal WHERE seq IN (SELECT j.seq FROM json_each(?1) e JOIN index_publication_journal j ON j.account_id=json_extract(e.value,'$[0]') AND j.stable_id=json_extract(e.value,'$[1]') AND j.seq<=json_extract(e.value,'$[2]'))"
            }
            Dialect::Postgres => {
                "DELETE FROM index_publication_journal WHERE seq IN (SELECT j.seq FROM json_array_elements(?1::json) e JOIN index_publication_journal j ON j.account_id=(e->>0) AND j.stable_id=(e->>1) AND j.seq<=(e->>2)::bigint)"
            }
        };
        Ok(q(sql).bind(payload).execute(&self.backend).await?)
    }

    /// Delete every row with `seq <= upto_seq`, for a full rebuild that captured
    /// `upto_seq` before its source scan and then committed.
    pub async fn index_journal_complete_all(&self, upto_seq: i64) -> Result<u64, StoreError> {
        Ok(q("DELETE FROM index_publication_journal WHERE seq<=?1")
            .bind(upto_seq)
            .execute(&self.backend)
            .await?)
    }

    /// Remove all rows of an account. Account erasure calls this after the
    /// authoritative deletion, so replay cannot recreate erased index entries.
    pub async fn index_journal_purge_account(&self, account_id: &str) -> Result<u64, StoreError> {
        Ok(
            q("DELETE FROM index_publication_journal WHERE account_id=?1")
                .bind(account_id)
                .execute(&self.backend)
                .await?,
        )
    }

    /// Number of rows waiting for replay or acknowledgement.
    pub async fn index_journal_len(&self) -> Result<u64, StoreError> {
        let n = q("SELECT CAST(COUNT(*) AS BIGINT) AS n FROM index_publication_journal")
            .fetch_scalar_i64(&self.backend)
            .await?;
        u64::try_from(n)
            .map_err(|_| StoreError::Corrupt("index journal length negative".to_owned()))
    }
}

#[cfg(test)]
#[path = "journal_tests.rs"]
mod tests;
