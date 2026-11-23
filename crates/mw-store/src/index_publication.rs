//! Bounded, fresh observations for optimistic index publication validation.
//! This is NOT an admission or mutation lease. No connection, transaction or
//! generation guard survives this method. Revalidate after publication and repair
//! before acknowledgement; arbitrary external writers do not participate in an
//! index physical-erasure acknowledgement protocol.
use crate::{
    Message, Store, StoreError, StoredMeta,
    backend::{Backend, Dialect, q},
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::PathBuf;
use std::sync::LazyLock;

mod journal;
pub use journal::{IndexJournalEntry, IndexJournalMode, MAX_INDEX_JOURNAL_BATCH};

pub const MAX_PUBLICATION_ACCOUNTS: usize = 64;
pub const MAX_PUBLICATION_MESSAGES: usize = 4096;
const MAX_INPUT_BYTES: usize = 1024 * 1024;
const MAX_RESTRICTIONS: usize = 4096;

#[derive(Clone)]
pub struct IndexPublicationRequest {
    pub account_id: String,
    pub stable_ids: Vec<String>,
}
#[derive(Clone, PartialEq, Eq)]
pub struct IndexPublicationAccount {
    pub exists: bool,
    pub zero_access: bool,
    pub indexing_restricted: bool,
    restrictions: Vec<RestrictionWitness>,
}
#[derive(Clone, PartialEq, Eq)]
pub struct IndexPublicationMessage {
    /// Present even when reassigned; foreign message content is never returned.
    pub current_owner: Option<String>,
    pub message: Option<Message>,
    pub meta: Option<StoredMeta>,
    /// Sealed source sizes, measured without fetching/decrypting the BLOB.
    pub envelope_bytes: Option<u64>,
    pub raw_body_bytes: Option<u64>,
    /// Immutable content identity, retained by a matching expiry lease after raw
    /// cache detachment. Reading this does not renew the inactivity lease.
    pub content_blob_ref: Option<String>,
}
pub struct IndexPublicationSnapshot {
    scope: uuid::Uuid,
    generation: Option<PathBuf>,
    accounts: BTreeMap<String, IndexPublicationAccount>,
    messages: BTreeMap<(String, String), IndexPublicationMessage>,
}
impl fmt::Debug for IndexPublicationRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IndexPublicationRequest")
            .field("message_count", &self.stable_ids.len())
            .finish_non_exhaustive()
    }
}
impl fmt::Debug for IndexPublicationAccount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IndexPublicationAccount")
            .field("exists", &self.exists)
            .field("zero_access", &self.zero_access)
            .field("indexing_restricted", &self.indexing_restricted)
            .finish_non_exhaustive()
    }
}
impl fmt::Debug for IndexPublicationMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IndexPublicationMessage")
            .field("owner_present", &self.current_owner.is_some())
            .field("message_present", &self.message.is_some())
            .field("meta_present", &self.meta.is_some())
            .finish_non_exhaustive()
    }
}
impl fmt::Debug for IndexPublicationSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IndexPublicationSnapshot")
            .field("accounts", &self.accounts.len())
            .field("messages", &self.messages.len())
            .finish_non_exhaustive()
    }
}
impl IndexPublicationSnapshot {
    pub fn account(&self, account: &str) -> Option<&IndexPublicationAccount> {
        self.accounts.get(account)
    }
    pub fn message(&self, account: &str, id: &str) -> Option<&IndexPublicationMessage> {
        self.messages.get(&(account.to_owned(), id.to_owned()))
    }
    /// Optimistic equality, not a promise that no writer can run afterwards.
    pub fn matches(&self, other: &Self) -> Result<bool, StoreError> {
        if self.scope != other.scope {
            return Err(corrupt("publication snapshots belong to different Stores"));
        }
        Ok(self.generation == other.generation
            && self.accounts == other.accounts
            && self.messages == other.messages)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct RestrictionWitness {
    id: String,
    scopes: String,
    created: i64,
    expires: Option<i64>,
}
#[derive(Deserialize)]
struct AccountPayload {
    exists: i64,
    zero_access_present: i64,
    zero_access: Option<i64>,
}
#[derive(Deserialize)]
struct MessagePayload {
    current_owner: Option<String>,
    content_blob_ref: Option<String>,
    stable_id: Option<String>,
    account_id: Option<String>,
    mailbox_id: Option<String>,
    uid: Option<u32>,
    uidvalidity: Option<u32>,
    message_id: Option<String>,
    thread_id: Option<String>,
    internaldate: Option<String>,
    size: Option<u64>,
    flags_json: Option<String>,
    blob_ref: Option<String>,
    cache_revision: Option<String>,
    meta_present: i64,
    pinned: Option<i64>,
    snoozed_until: Option<String>,
    follow_up_at: Option<String>,
    envelope_bytes: Option<u64>,
    raw_body_bytes: Option<u64>,
}
fn corrupt(message: &str) -> StoreError {
    StoreError::Corrupt(message.to_owned())
}
fn required<T>(value: Option<T>) -> Result<T, StoreError> {
    value.ok_or_else(|| corrupt("publication message shape invalid"))
}
impl MessagePayload {
    fn decode(self, requested_account: &str) -> Result<IndexPublicationMessage, StoreError> {
        if self.current_owner.as_deref() != Some(requested_account) {
            return Ok(IndexPublicationMessage {
                current_owner: self.current_owner,
                message: None,
                meta: None,
                envelope_bytes: None,
                raw_body_bytes: None,
                content_blob_ref: None,
            });
        }
        let message = Message {
            stable_id: required(self.stable_id)?,
            account_id: required(self.account_id)?,
            mailbox_id: required(self.mailbox_id)?,
            uid: required(self.uid)?,
            uidvalidity: required(self.uidvalidity)?,
            message_id: self.message_id,
            thread_id: self.thread_id,
            internaldate: self.internaldate,
            size: required(self.size)?,
            flags_json: required(self.flags_json)?,
            blob_ref: self.blob_ref,
            cache_revision: required(self.cache_revision)?,
        };
        let meta = if self.meta_present != 0 {
            Some(StoredMeta {
                pinned: required(self.pinned)? != 0,
                snoozed_until: self.snoozed_until,
                follow_up_at: self.follow_up_at,
            })
        } else {
            None
        };
        Ok(IndexPublicationMessage {
            current_owner: self.current_owner,
            message: Some(message),
            meta,
            envelope_bytes: self.envelope_bytes,
            raw_body_bytes: self.raw_body_bytes,
            content_blob_ref: self.content_blob_ref,
        })
    }
}
/// Freshly opened source bytes bound to the supplied authoritative Message.
/// No cache lease is renewed and no source metadata is repaired by this reader.
pub struct IndexPublicationSource {
    pub envelope: Option<Vec<u8>>,
    pub raw: Option<Vec<u8>>,
}
impl fmt::Debug for IndexPublicationSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IndexPublicationSource")
            .field("envelope_bytes", &self.envelope.as_ref().map(Vec::len))
            .field("raw_bytes", &self.raw.as_ref().map(Vec::len))
            .finish_non_exhaustive()
    }
}
const MAX_SOURCE_READ_BYTES: u64 = 64 * 1024 * 1024;

impl Store {
    /// Fetch/decrypt only an unchanged owned source. The SQL CASE budget guard
    /// runs before either BLOB is returned, including when an oversized body is
    /// corrupt: rejection must not attempt decryption first.
    pub async fn index_publication_source(
        &self,
        expected: &Message,
        max_bytes: u64,
    ) -> Result<Option<IndexPublicationSource>, StoreError> {
        if max_bytes > MAX_SOURCE_READ_BYTES
            || expected.account_id.len() > 1024
            || expected.stable_id.len() > 1024
            || expected.cache_revision.len() > 1024
        {
            return Err(corrupt("publication source read limit exceeded"));
        }
        let limit = i64::try_from(max_bytes)
            .map_err(|_| corrupt("publication source read limit exceeded"))?;
        let row=q("SELECT CASE WHEN COALESCE(octet_length(m.envelope_json),0)+COALESCE(octet_length(b.sealed_bytes),0)<=?8 THEN m.envelope_json ELSE NULL END AS envelope,CASE WHEN COALESCE(octet_length(m.envelope_json),0)+COALESCE(octet_length(b.sealed_bytes),0)<=?8 THEN b.sealed_bytes ELSE NULL END AS raw,CAST(CASE WHEN COALESCE(octet_length(m.envelope_json),0)+COALESCE(octet_length(b.sealed_bytes),0)<=?8 THEN 0 ELSE 1 END AS BIGINT) AS oversized FROM messages m JOIN accounts a ON a.id=m.account_id LEFT JOIN bodies b ON b.blob_ref=m.blob_ref AND b.account_id=m.account_id WHERE m.account_id=?1 AND m.stable_id=?2 AND m.cache_revision=?3 AND m.uidvalidity=?4 AND m.uid=?5 AND (m.blob_ref=?6 OR (m.blob_ref IS NULL AND ?6 IS NULL)) AND m.mailbox_id=?7")
            .bind(&expected.account_id).bind(&expected.stable_id).bind(&expected.cache_revision).bind(i64::from(expected.uidvalidity)).bind(i64::from(expected.uid)).bind(expected.blob_ref.as_deref()).bind(&expected.mailbox_id).bind(limit)
            .fetch_optional(&self.backend).await?;
        let Some(row) = row else {
            return Ok(None);
        };
        if row.get_i64("oversized") != 0 {
            return Err(corrupt("publication source byte budget exceeded"));
        }
        let envelope = row
            .get_opt_blob("envelope")
            .map(|bytes| self.key.open(&bytes))
            .transpose()?;
        let raw = row
            .get_opt_blob("raw")
            .map(|bytes| self.key.open(&bytes))
            .transpose()?;
        Ok(Some(IndexPublicationSource { envelope, raw }))
    }

    /// Open only the unchanged owned envelope, without reading or joining raw
    /// body storage. An oversized envelope is rejected in SQL before transfer.
    pub async fn index_publication_envelope(
        &self,
        expected: &Message,
        max_bytes: u64,
    ) -> Result<Option<Vec<u8>>, StoreError> {
        if max_bytes > MAX_SOURCE_READ_BYTES
            || expected.account_id.len() > 1024
            || expected.stable_id.len() > 1024
            || expected.cache_revision.len() > 1024
        {
            return Err(corrupt("publication envelope read limit exceeded"));
        }
        let limit = i64::try_from(max_bytes)
            .map_err(|_| corrupt("publication envelope read limit exceeded"))?;
        let row=q("SELECT CASE WHEN COALESCE(octet_length(m.envelope_json),0)<=?8 THEN m.envelope_json ELSE NULL END AS envelope,CAST(CASE WHEN COALESCE(octet_length(m.envelope_json),0)<=?8 THEN 0 ELSE 1 END AS BIGINT) AS oversized FROM messages m JOIN accounts a ON a.id=m.account_id WHERE m.account_id=?1 AND m.stable_id=?2 AND m.cache_revision=?3 AND m.uidvalidity=?4 AND m.uid=?5 AND (m.blob_ref=?6 OR (m.blob_ref IS NULL AND ?6 IS NULL)) AND m.mailbox_id=?7")
            .bind(&expected.account_id).bind(&expected.stable_id).bind(&expected.cache_revision).bind(i64::from(expected.uidvalidity)).bind(i64::from(expected.uid)).bind(expected.blob_ref.as_deref()).bind(&expected.mailbox_id).bind(limit).fetch_optional(&self.backend).await?;
        let Some(row) = row else {
            return Ok(None);
        };
        if row.get_i64("oversized") != 0 {
            return Err(corrupt("publication envelope byte budget exceeded"));
        }
        row.get_opt_blob("envelope")
            .map(|bytes| self.key.open(&bytes).map_err(StoreError::from))
            .transpose()
    }

    pub(crate) async fn publication_generation(&self) -> Option<PathBuf> {
        match &self.backend {
            Backend::Sqlite(live) => Some(
                live.pin()
                    .await
                    .active_options()
                    .get_filename()
                    .to_path_buf(),
            ),
            Backend::Postgres(_) => None,
        }
    }
    /// One fresh SQL snapshot shared by all requested account/message pairs.
    /// Existing account-scoped readers remain the public authenticated projection
    /// API; this internal-index observation additionally reports reassignment.
    pub async fn index_publication_snapshot(
        &self,
        requests: &[IndexPublicationRequest],
        now: i64,
    ) -> Result<IndexPublicationSnapshot, StoreError> {
        self.index_publication_snapshot_with_body(requests, now, true)
            .await
    }
    /// Metadata-only variant for an owned metadata publication. It observes the
    /// same message/owner/meta/posture/restriction/expiry identity, but never
    /// joins raw body storage or reads its size. This is not a raw-source proof.
    pub async fn index_publication_metadata_snapshot(
        &self,
        requests: &[IndexPublicationRequest],
        now: i64,
    ) -> Result<IndexPublicationSnapshot, StoreError> {
        self.index_publication_snapshot_with_body(requests, now, false)
            .await
    }
    async fn index_publication_snapshot_with_body(
        &self,
        requests: &[IndexPublicationRequest],
        now: i64,
        include_body: bool,
    ) -> Result<IndexPublicationSnapshot, StoreError> {
        if requests.len() > MAX_PUBLICATION_ACCOUNTS {
            return Err(corrupt("publication request limit exceeded"));
        }
        if requests
            .iter()
            .map(|request| request.stable_ids.len())
            .try_fold(0usize, |total, n| total.checked_add(n))
            .is_none_or(|total| total > MAX_PUBLICATION_MESSAGES)
        {
            return Err(corrupt("publication request message limit exceeded"));
        }
        let mut input = Vec::new();
        let mut accounts = BTreeSet::new();
        let mut pairs = BTreeSet::new();
        let mut bytes = 0usize;
        for request in requests {
            bytes = bytes.saturating_add(request.account_id.len());
            if request.account_id.is_empty()
                || request.account_id.len() > 1024
                || bytes > MAX_INPUT_BYTES
            {
                return Err(corrupt("publication account/input limit exceeded"));
            }
            accounts.insert(request.account_id.as_str());
            if accounts.len() > MAX_PUBLICATION_ACCOUNTS {
                return Err(corrupt("publication account limit exceeded"));
            }
            if request.stable_ids.is_empty() {
                input.push(serde_json::json!({"account":request.account_id,"id":null}));
            }
            for id in &request.stable_ids {
                bytes = bytes.saturating_add(id.len());
                if id.is_empty() || id.len() > 1024 || bytes > MAX_INPUT_BYTES {
                    return Err(corrupt("publication id/input limit exceeded"));
                }
                if pairs.insert((request.account_id.as_str(), id.as_str())) {
                    if pairs.len() > MAX_PUBLICATION_MESSAGES {
                        return Err(corrupt("publication message limit exceeded"));
                    }
                    input.push(serde_json::json!({"account":request.account_id,"id":id}));
                }
            }
        }
        let input = serde_json::to_string(&input)
            .map_err(|_| corrupt("publication input encoding failed"))?;
        if input.len() > MAX_INPUT_BYTES {
            return Err(corrupt("publication encoded input limit exceeded"));
        }
        let generation = self.publication_generation().await;
        let rows = q(publication_sql_variant(
            self.backend.dialect(),
            include_body,
        ))
        .bind(input)
        .fetch_all(&self.backend)
        .await?;
        if generation != self.publication_generation().await {
            return Err(StoreError::SourceChanged);
        }
        let mut result = IndexPublicationSnapshot {
            scope: self.cache_scope(),
            generation,
            accounts: BTreeMap::new(),
            messages: BTreeMap::new(),
        };
        let mut restriction_count = 0;
        for row in rows {
            let tag = row
                .try_get_opt_string("tag")?
                .ok_or_else(|| corrupt("publication tag missing"))?;
            let account = row
                .try_get_opt_string("requested_account")?
                .ok_or_else(|| corrupt("publication account missing"))?;
            let payload = row
                .try_get_opt_string("payload")?
                .ok_or_else(|| corrupt("publication payload missing"))?;
            match tag.as_str() {
                "overflow" => {
                    return Err(corrupt(
                        "publication source metadata budget or shape exceeded",
                    ));
                }
                "account" => {
                    let value: AccountPayload = serde_json::from_str(&payload)
                        .map_err(|_| corrupt("publication account state malformed"))?;
                    let zero_access = if value.zero_access_present != 0 {
                        required(value.zero_access)? != 0
                    } else {
                        false
                    };
                    result.accounts.insert(
                        account,
                        IndexPublicationAccount {
                            exists: value.exists != 0,
                            zero_access,
                            indexing_restricted: false,
                            restrictions: Vec::new(),
                        },
                    );
                }
                "restriction" => {
                    restriction_count += 1;
                    if restriction_count > MAX_RESTRICTIONS {
                        return Err(corrupt("publication restriction limit exceeded"));
                    }
                    let value: RestrictionWitness = serde_json::from_str(&payload)
                        .map_err(|_| corrupt("publication restriction state malformed"))?;
                    // Do not silently treat malformed restriction scopes as an
                    // unrestricted index publication. This worker is fail closed.
                    let _: Vec<String> = serde_json::from_str(&value.scopes)
                        .map_err(|_| corrupt("publication restriction scopes malformed"))?;
                    result
                        .accounts
                        .get_mut(&account)
                        .ok_or_else(|| corrupt("publication restriction account missing"))?
                        .restrictions
                        .push(value);
                }
                "message" => {
                    let id = row
                        .try_get_opt_string("stable_id")?
                        .ok_or_else(|| corrupt("publication message id missing"))?;
                    let value: MessagePayload = serde_json::from_str(&payload)
                        .map_err(|_| corrupt("publication source state malformed"))?;
                    result
                        .messages
                        .insert((account.clone(), id), value.decode(&account)?);
                }
                _ => return Err(corrupt("publication tag unknown")),
            }
        }
        for value in result.accounts.values_mut() {
            // SQL preserves the old newest-first ordering including DB tie order.
            if let Some(active) = value
                .restrictions
                .iter()
                .find(|r| r.expires.is_none_or(|expiry| expiry > now))
            {
                let scopes: Vec<String> = serde_json::from_str(&active.scopes)
                    .map_err(|_| corrupt("publication restriction scopes malformed"))?;
                value.indexing_restricted = scopes.iter().any(|s| s == "indexing" || s == "all");
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
fn publication_sql(dialect: Dialect) -> &'static str {
    publication_sql_variant(dialect, true)
}
fn publication_sql_variant(dialect: Dialect, include_body: bool) -> &'static str {
    static SQLITE: LazyLock<String> =
        LazyLock::new(|| build_publication_sql(Dialect::Sqlite, true));
    static POSTGRES: LazyLock<String> =
        LazyLock::new(|| build_publication_sql(Dialect::Postgres, true));
    static SQLITE_METADATA: LazyLock<String> =
        LazyLock::new(|| build_publication_sql(Dialect::Sqlite, false));
    static POSTGRES_METADATA: LazyLock<String> =
        LazyLock::new(|| build_publication_sql(Dialect::Postgres, false));
    match (dialect, include_body) {
        (Dialect::Sqlite, true) => SQLITE.as_str(),
        (Dialect::Postgres, true) => POSTGRES.as_str(),
        (Dialect::Sqlite, false) => SQLITE_METADATA.as_str(),
        (Dialect::Postgres, false) => POSTGRES_METADATA.as_str(),
    }
}
fn build_publication_sql(dialect: Dialect, include_body: bool) -> String {
    const SCALAR: usize = 256 * 1024;
    const BATCH: usize = 8 * 1024 * 1024;
    const SCOPES: usize = 64 * 1024;
    let (requested, json, cast) = match dialect {
        Dialect::Sqlite => (
            "SELECT json_extract(value,'$.account') AS requested_account,json_extract(value,'$.id') AS requested_id FROM json_each(?1)",
            "json_object",
            "",
        ),
        Dialect::Postgres => (
            "SELECT value->>'account' AS requested_account,value->>'id' AS requested_id FROM json_array_elements(?1::json)",
            "json_build_object",
            "::text",
        ),
    };
    let message_fields = [
        "stable_id",
        "account_id",
        "mailbox_id",
        "uid",
        "uidvalidity",
        "message_id",
        "thread_id",
        "internaldate",
        "size",
        "flags_json",
        "blob_ref",
        "cache_revision",
    ];
    let fields = message_fields
        .iter()
        .map(|name| {
            format!(
                "'{name}',CASE WHEN m.account_id=r.requested_account THEN m.{name} ELSE NULL END"
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let source_text = [
        "m.stable_id",
        "m.account_id",
        "m.mailbox_id",
        "m.message_id",
        "m.thread_id",
        "m.internaldate",
        "m.flags_json",
        "m.blob_ref",
        "m.cache_revision",
        "mm.snoozed_until",
        "mm.follow_up_at",
        "l.blob_ref",
    ];
    let length = |field: &str| format!("COALESCE(octet_length({field}),0)");
    let scoped_length = |field: &str| {
        if field == "m.account_id" {
            length(field)
        } else {
            format!(
                "CASE WHEN m.account_id=r.requested_account THEN {} ELSE 0 END",
                length(field)
            )
        }
    };
    let source_bytes = source_text
        .iter()
        .map(|field| scoped_length(field))
        .collect::<Vec<_>>()
        .join("+");
    let source_big = source_text
        .iter()
        .map(|field| format!("{}>{SCALAR}", scoped_length(field)))
        .collect::<Vec<_>>()
        .join(" OR ");
    let restriction_bytes = "ar.id_bytes+ar.account_bytes+ar.scope_bytes";
    let restriction_big =
        format!("ar.id_bytes>{SCALAR} OR ar.account_bytes>{SCALAR} OR ar.scope_bytes>{SCOPES}");
    // Type checks avoid constructing enormous JSON strings out of malformed
    // SQLite numeric cells. PostgreSQL's strongly typed columns need no scan.
    let (source_bad, restriction_bad, posture_bad) = match dialect {
        Dialect::Sqlite => (
            "m.account_id=r.requested_account AND (typeof(m.uid)!='integer' OR typeof(m.uidvalidity)!='integer' OR typeof(m.size)!='integer' OR (mm.stable_id IS NOT NULL AND typeof(mm.pinned)!='integer'))",
            "ar.invalid_shape!=0",
            "z.account_id IS NOT NULL AND typeof(z.enabled)!='integer'",
        ),
        Dialect::Postgres => ("FALSE", "FALSE", "FALSE"),
    };
    // Carry only bounded physical row locators and length/type witnesses in
    // this multiply-used CTE. Even an over-budget collection of giant textual
    // primary keys must not be materialized before the aggregate budget check.
    // Locators are statement-local; they never escape this SQL snapshot.
    let restriction_rows = match dialect {
        Dialect::Sqlite => format!(
            "SELECT ar.rowid AS row_locator,ra.requested_account AS account_id,CASE WHEN typeof(ar.created_at)='integer' THEN ar.created_at ELSE 0 END AS created_at,octet_length(ar.id) AS id_bytes,octet_length(ar.account_id) AS account_bytes,octet_length(ar.scopes_json) AS scope_bytes,CASE WHEN typeof(ar.id)!='text' OR typeof(ar.scopes_json)!='text' OR typeof(ar.created_at)!='integer' OR typeof(ar.expires_at) NOT IN ('integer','null') THEN 1 ELSE 0 END AS invalid_shape FROM account_restriction ar JOIN requested_accounts ra ON ra.requested_account=ar.account_id WHERE ar.lifted_at IS NULL ORDER BY ar.account_id,CASE WHEN typeof(ar.created_at)='integer' THEN ar.created_at ELSE 0 END DESC LIMIT 4097"
        ),
        Dialect::Postgres => format!(
            "SELECT ar.ctid AS row_locator,ra.requested_account AS account_id,ar.created_at,octet_length(ar.id) AS id_bytes,octet_length(ar.account_id) AS account_bytes,octet_length(ar.scopes_json) AS scope_bytes,0 AS invalid_shape FROM account_restriction ar JOIN requested_accounts ra ON ra.requested_account=ar.account_id WHERE ar.lifted_at IS NULL ORDER BY ar.account_id,ar.created_at DESC LIMIT 4097"
        ),
    };
    let body_join = if include_body {
        "LEFT JOIN bodies b ON b.blob_ref=m.blob_ref AND b.account_id=m.account_id AND m.account_id=r.requested_account"
    } else {
        ""
    };
    let body_length = if include_body {
        "CASE WHEN m.account_id=r.requested_account THEN octet_length(b.sealed_bytes) ELSE NULL END"
    } else {
        "NULL"
    };
    let restriction_locator = match dialect {
        Dialect::Sqlite => "ar.rowid=restricted.row_locator",
        Dialect::Postgres => "ar.ctid=restricted.row_locator",
    };
    format!(
        "WITH requested AS ({requested}), requested_accounts AS (SELECT DISTINCT requested_account FROM requested), restrictions AS ({restriction_rows}), source_budget AS (SELECT COALESCE(SUM({source_bytes}),0) AS bytes,COALESCE(MAX(CASE WHEN {source_big} OR ({source_bad}) THEN 1 ELSE 0 END),0) AS invalid FROM requested r LEFT JOIN messages m ON m.stable_id=r.requested_id LEFT JOIN message_meta mm ON mm.stable_id=m.stable_id AND m.account_id=r.requested_account LEFT JOIN message_body_cache_lease l ON l.stable_id=m.stable_id AND l.account_id=m.account_id AND l.uidvalidity=m.uidvalidity AND l.uid=m.uid), restriction_budget AS (SELECT COUNT(*) AS n,COALESCE(SUM({restriction_bytes}),0) AS bytes,COALESCE(MAX(CASE WHEN {restriction_big} OR ({restriction_bad}) THEN 1 ELSE 0 END),0) AS invalid FROM restrictions ar), posture_budget AS (SELECT COALESCE(MAX(CASE WHEN {posture_bad} THEN 1 ELSE 0 END),0) AS invalid FROM requested_accounts ra LEFT JOIN zeroaccess_accounts z ON z.account_id=ra.requested_account), budget AS (SELECT CASE WHEN sb.bytes+rb.bytes<={BATCH} AND sb.invalid=0 AND rb.invalid=0 AND rb.n<={MAX_RESTRICTIONS} AND pb.invalid=0 THEN 1 ELSE 0 END AS ok FROM source_budget sb CROSS JOIN restriction_budget rb CROSS JOIN posture_budget pb) SELECT tag,requested_account,stable_id,payload FROM (SELECT 'overflow' AS tag,'' AS requested_account,'' AS stable_id,CAST(0 AS BIGINT) AS created_at,'{{}}' AS payload FROM budget WHERE ok=0 UNION ALL SELECT 'account',ra.requested_account,'',CAST(0 AS BIGINT),{json}('exists',CASE WHEN a.id IS NULL THEN 0 ELSE 1 END,'zero_access_present',CASE WHEN z.account_id IS NULL THEN 0 ELSE 1 END,'zero_access',z.enabled){cast} FROM budget CROSS JOIN requested_accounts ra LEFT JOIN accounts a ON a.id=ra.requested_account LEFT JOIN zeroaccess_accounts z ON z.account_id=ra.requested_account WHERE budget.ok=1 UNION ALL SELECT 'restriction',ar.account_id,'',ar.created_at,{json}('id',ar.id,'scopes',ar.scopes_json,'created',ar.created_at,'expires',ar.expires_at){cast} FROM budget CROSS JOIN restrictions restricted JOIN account_restriction ar ON {restriction_locator} WHERE budget.ok=1 UNION ALL SELECT 'message',r.requested_account,r.requested_id,CAST(0 AS BIGINT),{json}('current_owner',m.account_id,'content_blob_ref',CASE WHEN m.account_id=r.requested_account THEN COALESCE(m.blob_ref,l.blob_ref) ELSE NULL END,{fields},'meta_present',CASE WHEN mm.stable_id IS NULL THEN 0 ELSE 1 END,'pinned',mm.pinned,'snoozed_until',mm.snoozed_until,'follow_up_at',mm.follow_up_at,'envelope_bytes',CASE WHEN m.account_id=r.requested_account THEN octet_length(m.envelope_json) ELSE NULL END,'raw_body_bytes',{body_length}){cast} FROM budget CROSS JOIN requested r LEFT JOIN messages m ON m.stable_id=r.requested_id LEFT JOIN message_meta mm ON mm.stable_id=m.stable_id AND m.account_id=r.requested_account LEFT JOIN message_body_cache_lease l ON l.stable_id=m.stable_id AND l.account_id=m.account_id AND l.uidvalidity=m.uidvalidity AND l.uid=m.uid {body_join} WHERE budget.ok=1 AND r.requested_id IS NOT NULL) publication_rows ORDER BY tag,requested_account,created_at DESC,stable_id"
    )
}

#[cfg(test)]
#[path = "index_publication_tests.rs"]
mod tests;
