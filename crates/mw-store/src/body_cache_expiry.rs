//! Renewable cached-MIME leases. Local originals are never expiry candidates.
//! The engine supplies currently registered, explicitly refetch-capable sources.
use crate::{
    ServerKey, Store, StoreError,
    backend::{Backend, Row},
    q,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedBodyExpiryOptions {
    pub enabled: bool,
    pub retention_days: u32,
    pub batch_rows: u32,
    pub batch_bytes: u64,
    pub max_run_ms: u64,
    pub max_wal_bytes: u64,
}
impl Default for CachedBodyExpiryOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_days: 60,
            batch_rows: 64,
            batch_bytes: 16 * 1024 * 1024,
            max_run_ms: 250,
            max_wal_bytes: 32 * 1024 * 1024,
        }
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedBodyExpiryCursor {
    pub scan: String,
    pub expire: String,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedBodyExpiryReport {
    pub status: String,
    pub at: i64,
    pub cutoff_at: i64,
    pub scanned_messages: u64,
    pub initialized_messages: u64,
    pub detached_messages: u64,
    pub removed_bodies: u64,
    pub removed_bytes: u64,
    #[serde(default)]
    pub cleaned_envelopes: u64,
    #[serde(default)]
    pub removed_envelope_bytes: u64,
    pub protected_messages: u64,
    pub oversized_bodies: u64,
    pub cursor: CachedBodyExpiryCursor,
    pub elapsed_ms: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedBodyExpiryStats {
    pub tracked_messages: u64,
    pub oldest_use_at: Option<i64>,
    pub latest_use_at: Option<i64>,
}

fn eligible(row: &Row, sources: &HashSet<String>) -> bool {
    sources.contains(&row.get_string("account_id"))
        && row.get_string("kind") == "imap"
        && row.get_i64("uidvalidity") > 0
        && row.get_i64("uid") > 0
        && !matches!(
            row.get_opt_string("role").as_deref(),
            Some("drafts" | "outbox")
        )
        && serde_json::from_str::<Vec<String>>(&row.get_string("flags_json")).is_ok_and(|flags| {
            !flags.iter().any(|flag| {
                ["Draft", "\\Draft", "$draft"]
                    .iter()
                    .any(|draft| flag.eq_ignore_ascii_case(draft))
            })
        })
}
impl Store {
    /// An explicit user body-read intent. Ordinary reads, sync and prefetch do not
    /// call this. Lease writes are throttled to one per message per hour in SQL.
    pub async fn renew_cached_body_use(&self, account: &str, id: &str) -> Result<u64, StoreError> {
        let now = chrono::Utc::now().timestamp();
        let current=q("SELECT last_used_at FROM message_body_cache_lease WHERE stable_id=?1 AND account_id=?2").bind(id).bind(account).fetch_optional(&self.backend).await?;
        if current.is_none_or(|row| row.get_i64("last_used_at") > now - 3600) {
            return Ok(0);
        }
        Ok(q("UPDATE message_body_cache_lease SET last_used_at=?1 WHERE stable_id=?2 AND account_id=?3 AND last_used_at<=?4 AND EXISTS(SELECT 1 FROM messages m WHERE m.stable_id=?2 AND m.account_id=?3 AND m.uidvalidity=message_body_cache_lease.uidvalidity AND m.uid=message_body_cache_lease.uid AND (m.blob_ref=message_body_cache_lease.blob_ref OR m.blob_ref IS NULL))").bind(now).bind(id).bind(account).bind(now-3600).execute(&self.backend).await?)
    }
    /// Record a genuine remote cache fill, or adopt a legacy remote message. The
    /// same content/coordinates retain their original lease during background sync.
    pub async fn register_refetchable_cached_body(
        &self,
        account: &str,
        id: &str,
    ) -> Result<u64, StoreError> {
        register(&self.backend, account, id, chrono::Utc::now().timestamp()).await
    }
    pub async fn cached_body_expiry_stats(&self) -> Result<CachedBodyExpiryStats, StoreError> {
        let row=q("SELECT COUNT(*) AS n,MIN(last_used_at) AS oldest,MAX(last_used_at) AS latest FROM message_body_cache_lease").fetch_one(&self.backend).await?;
        Ok(CachedBodyExpiryStats {
            tracked_messages: row.get_i64("n").max(0) as u64,
            oldest_use_at: row.get_opt_i64("oldest"),
            latest_use_at: row.get_opt_i64("latest"),
        })
    }
    pub async fn expire_cached_bodies(
        &self,
        options: CachedBodyExpiryOptions,
        cursor: CachedBodyExpiryCursor,
        refetchable_accounts: Vec<String>,
    ) -> Result<CachedBodyExpiryReport, StoreError> {
        let original = self.backend.clone();
        let key = self.key.clone();
        let id = self.instance_id();
        tokio::spawn(async move {
            let report = run(original, key, options, cursor, refetchable_accounts).await?;
            crate::database_growth::record_cleanup(id, crate::database_growth::Cleanup {
                removed_bodies: report.removed_bodies, removed_body_bytes: report.removed_bytes,
                cleaned_envelopes: report.cleaned_envelopes, removed_envelope_bytes: report.removed_envelope_bytes,
                ..Default::default()
            });
            Ok(report)
        })
            .await
            .map_err(|error| StoreError::from(sqlx::Error::Protocol(error.to_string())))?
    }
}
async fn register(backend: &Backend, account: &str, id: &str, now: i64) -> Result<u64, StoreError> {
    if q("SELECT l.stable_id FROM message_body_cache_lease l JOIN messages m ON m.stable_id=l.stable_id WHERE l.stable_id=?1 AND l.account_id=?2 AND l.blob_ref=m.blob_ref AND l.uidvalidity=m.uidvalidity AND l.uid=m.uid").bind(id).bind(account).fetch_optional(backend).await?.is_some(){return Ok(0);}
    let mut tx = backend.begin().await?;
    crate::background_maintenance::lock_body_account(&mut tx, account).await?;
    let query = "INSERT INTO message_body_cache_lease(stable_id,account_id,blob_ref,uidvalidity,uid,last_used_at) SELECT m.stable_id,m.account_id,m.blob_ref,m.uidvalidity,m.uid,?3 FROM messages m JOIN accounts a ON a.id=m.account_id JOIN mailboxes mb ON mb.id=m.mailbox_id WHERE m.stable_id=?1 AND m.account_id=?2 AND m.blob_ref IS NOT NULL AND NOT(m.uidvalidity<=0 OR m.uid<=0 OR a.kind<>'imap' OR COALESCE(mb.role,'') IN ('drafts','outbox') OR EXISTS(SELECT 1 FROM submissions s WHERE s.email_id=m.stable_id) OR EXISTS(SELECT 1 FROM account_draft d WHERE d.email_id=m.stable_id OR d.cleanup_email_id=m.stable_id)) ON CONFLICT(stable_id) DO UPDATE SET account_id=excluded.account_id,blob_ref=excluded.blob_ref,uidvalidity=excluded.uidvalidity,uid=excluded.uid,last_used_at=excluded.last_used_at WHERE message_body_cache_lease.blob_ref<>excluded.blob_ref OR message_body_cache_lease.uidvalidity<>excluded.uidvalidity OR message_body_cache_lease.uid<>excluded.uid";
    let n = q(query)
        .bind(id)
        .bind(account)
        .bind(now)
        .execute_tx(&mut tx)
        .await?;
    tx.commit().await?;
    Ok(n)
}
async fn run(
    original: Backend,
    key: ServerKey,
    mut options: CachedBodyExpiryOptions,
    cursor: CachedBodyExpiryCursor,
    sources: Vec<String>,
) -> Result<CachedBodyExpiryReport, StoreError> {
    let started = Instant::now();
    let now = chrono::Utc::now().timestamp();
    options.retention_days = options.retention_days.clamp(1, 3650);
    options.batch_rows = options.batch_rows.clamp(1, 512);
    options.max_run_ms = options.max_run_ms.clamp(25, 2000);
    options.batch_bytes = options.batch_bytes.clamp(1024 * 1024, 256 * 1024 * 1024);
    let mut report = CachedBodyExpiryReport {
        status: "complete".into(),
        at: now,
        cutoff_at: now - i64::from(options.retention_days) * 86400,
        cursor,
        ..Default::default()
    };
    if !options.enabled {
        report.status = "disabled".into();
        return Ok(report);
    }
    let sources = sources.into_iter().take(10000).collect::<HashSet<_>>();
    if sources.is_empty() {
        report.status = "no-refetchable-sources".into();
        return Ok(report);
    }
    let mut pin = match &original {
        Backend::Sqlite(pool) => Some(pool.pin().await),
        _ => None,
    };
    // Keep a physical foreground connection alive so closing the dedicated
    // worker cannot become SQLite's implicit last-connection checkpoint.
    let mut source_connection = match &pin {
        Some(pin) => match crate::background_maintenance::idle_keepalive(pin).await {
            Some(connection) => Some(connection),
            None => {
                report.status = "busy".into();
                return Ok(report);
            }
        },
        None => None,
    };
    if let Backend::Sqlite(pool) = &original {
        let wal = std::fs::metadata(format!("{}-wal", pool.logical_path().display()))
            .map(|m| m.len())
            .unwrap_or(0);
        if wal > options.max_wal_bytes {
            report.status = "wal-pressure".into();
            return Ok(report);
        }
    }
    let mut worker_lease = None;
    let backend = match original {
        Backend::Sqlite(_) => {
            let pool = sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(1)
                .acquire_timeout(Duration::from_secs(2))
                .connect_lazy_with(
                    (*pin.as_ref().unwrap().active_options())
                        .clone()
                        .busy_timeout(Duration::from_millis(50))
                        .pragma("wal_autocheckpoint", "0"),
                );
            worker_lease = Some(crate::background_maintenance::WorkerLease::new(
                pool.clone(),
                pin.take().unwrap(),
                source_connection.take().unwrap(),
            ));
            Backend::Sqlite(pool.into())
        }
        Backend::Postgres(pool) => Backend::Postgres(
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(1)
                .acquire_timeout(Duration::from_secs(2))
                .after_connect(|connection, _| {
                    Box::pin(async move {
                        sqlx::query("SET lock_timeout='50ms'")
                            .execute(&mut *connection)
                            .await?;
                        sqlx::query("SET statement_timeout='500ms'")
                            .execute(&mut *connection)
                            .await?;
                        Ok(())
                    })
                })
                .connect_with((*pool.connect_options()).clone())
                .await?,
        ),
    };
    let deadline = Instant::now() + Duration::from_millis(options.max_run_ms);
    if let Backend::Sqlite(pool) = &backend {
        pool.acquire()
            .await?
            .lock_handle()
            .await?
            .set_progress_handler(1000, move || Instant::now() < deadline);
    }
    let outcome = collect(&backend, &key, &options, &sources, deadline, &mut report).await;
    match &backend {
        Backend::Sqlite(pool) => {
            if let Ok(mut c) = pool.acquire().await {
                if let Ok(mut h) = c.lock_handle().await {
                    h.remove_progress_handler();
                }
            }
            pool.close().await;
        }
        Backend::Postgres(pool) => pool.close().await,
    }
    if let Some(lease) = &mut worker_lease {
        lease.close().await;
    }
    report.elapsed_ms = started.elapsed().as_millis() as u64;
    if let Err(error) = outcome {
        let text = error.to_string().to_ascii_lowercase();
        if Instant::now() >= deadline
            || text.contains("locked")
            || text.contains("busy")
            || text.contains("statement timeout")
            || text.contains("lock timeout")
        {
            report.status = if Instant::now() >= deadline {
                "budget"
            } else {
                "busy"
            }
            .into();
        } else {
            return Err(error);
        }
    }
    Ok(report)
}
async fn collect(
    backend: &Backend,
    key: &ServerKey,
    options: &CachedBodyExpiryOptions,
    sources: &HashSet<String>,
    deadline: Instant,
    report: &mut CachedBodyExpiryReport,
) -> Result<(), StoreError> {
    let query = "SELECT m.stable_id,m.account_id,m.blob_ref,m.uidvalidity,m.uid,m.flags_json,a.kind,mb.role FROM messages m JOIN accounts a ON a.id=m.account_id JOIN mailboxes mb ON mb.id=m.mailbox_id LEFT JOIN message_body_cache_lease l ON l.stable_id=m.stable_id WHERE m.stable_id>?1 AND m.blob_ref IS NOT NULL AND (l.stable_id IS NULL OR l.blob_ref<>m.blob_ref OR l.uidvalidity<>m.uidvalidity OR l.uid<>m.uid) ORDER BY m.stable_id LIMIT ?2";
    let rows = q(query)
        .bind(&report.cursor.scan)
        .bind(i64::from(options.batch_rows))
        .fetch_all(backend)
        .await?;
    for row in &rows {
        if Instant::now() >= deadline {
            report.status = "budget".into();
            break;
        }
        report.scanned_messages += 1;
        if eligible(row, sources) {
            report.initialized_messages += register(
                backend,
                &row.get_string("account_id"),
                &row.get_string("stable_id"),
                report.at,
            )
            .await?;
        } else {
            report.protected_messages += 1;
        }
        // Each adoption transaction commits before advancing its cursor or deleting.
        report.cursor.scan = row.get_string("stable_id");
    }
    if rows.len() < options.batch_rows as usize && report.scanned_messages == rows.len() as u64 {
        report.cursor.scan.clear();
    }
    if Instant::now() >= deadline {
        report.status = "budget".into();
        return Ok(());
    }
    let query = "SELECT m.stable_id,m.account_id,m.blob_ref,m.uidvalidity,m.uid,m.flags_json,a.kind,mb.role,l.last_used_at,l.uidvalidity AS lease_validity,l.uid AS lease_uid,l.blob_ref AS lease_blob,CAST(COALESCE(octet_length(b.sealed_bytes),0) AS BIGINT) AS bytes,CAST(COALESCE(octet_length(m.envelope_json),0) AS BIGINT) AS envelope_bytes FROM message_body_cache_lease l JOIN messages m ON m.stable_id=l.stable_id JOIN accounts a ON a.id=m.account_id JOIN mailboxes mb ON mb.id=m.mailbox_id LEFT JOIN bodies b ON b.blob_ref=m.blob_ref WHERE l.last_used_at<=?1 AND l.stable_id>?2 ORDER BY l.stable_id LIMIT ?3";
    let rows = q(query)
        .bind(report.cutoff_at)
        .bind(&report.cursor.expire)
        .bind(i64::from(options.batch_rows))
        .fetch_all(backend)
        .await?;
    let mut visited = 0;
    let mut work_bytes = 0u64;
    for row in &rows {
        if Instant::now() >= deadline {
            report.status = "budget".into();
            break;
        }
        let id = row.get_string("stable_id");
        let account = row.get_string("account_id");
        let blob = row.get_opt_string("blob_ref");
        let lease_blob = row.get_string("lease_blob");
        let size = row.get_i64("bytes").max(0) as u64;
        let envelope_size = row.get_i64("envelope_bytes").max(0) as u64;
        let candidate_bytes = size.saturating_add(envelope_size);
        if !eligible(row, sources)
            || blob.as_ref().is_some_and(|blob| &lease_blob != blob)
            || row.get_i64("lease_validity") != row.get_i64("uidvalidity")
            || row.get_i64("lease_uid") != row.get_i64("uid")
        {
            report.protected_messages += 1;
        } else if candidate_bytes > options.batch_bytes {
            report.oversized_bodies += 1;
        } else if work_bytes.saturating_add(candidate_bytes) > options.batch_bytes {
            break;
        } else {
            let mut tx = backend.begin().await?;
            crate::background_maintenance::lock_body_account(&mut tx, &account).await?;
            // Length is checked in SQL before retrieving this single sealed blob.
            // The bound also handles a concurrent metadata expansion since the scan.
            let envelope_budget = options
                .batch_bytes
                .saturating_sub(work_bytes)
                .saturating_sub(size);
            let envelopes=q("SELECT envelope_json FROM messages WHERE stable_id=?1 AND account_id=?2 AND (blob_ref=?3 OR (blob_ref IS NULL AND ?3 IS NULL)) AND COALESCE(octet_length(envelope_json),0)<=?4")
                .bind(&id).bind(&account).bind(blob.clone()).bind(envelope_budget.min(i64::MAX as u64) as i64).fetch_all_tx(&mut tx).await?;
            let envelope = envelopes
                .first()
                .map(|row| row.get_opt_blob("envelope_json"));
            if let Some(envelope) = envelope {
                work_bytes = work_bytes
                    .saturating_add(size)
                    .saturating_add(envelope.as_ref().map_or(0, |bytes| bytes.len() as u64));
                match strip_body_values(key, &envelope) {
                    Ok((cleaned, changed, saved)) if blob.is_some() || changed => {
                        let query = "UPDATE messages SET blob_ref=NULL,envelope_json=?6 WHERE stable_id=?1 AND account_id=?2 AND (blob_ref=?3 OR (blob_ref IS NULL AND ?3 IS NULL)) AND uidvalidity>0 AND uid>0 AND EXISTS(SELECT 1 FROM accounts a WHERE a.id=?2 AND a.kind='imap') AND EXISTS(SELECT 1 FROM message_body_cache_lease l WHERE l.stable_id=?1 AND l.blob_ref=?8 AND l.uidvalidity=messages.uidvalidity AND l.uid=messages.uid AND l.last_used_at<=?4) AND NOT EXISTS(SELECT 1 FROM submissions s WHERE s.email_id=?1) AND NOT EXISTS(SELECT 1 FROM account_draft d WHERE d.email_id=?1 OR d.cleanup_email_id=?1) AND mailbox_id IN(SELECT id FROM mailboxes WHERE COALESCE(role,'') NOT IN('drafts','outbox')) AND flags_json=?5 AND (envelope_json=?7 OR (envelope_json IS NULL AND ?7 IS NULL))";
                        let updated = q(query)
                            .bind(&id)
                            .bind(&account)
                            .bind(blob.clone())
                            .bind(report.cutoff_at)
                            .bind(row.get_string("flags_json"))
                            .bind(cleaned)
                            .bind(envelope)
                            .bind(&lease_blob)
                            .execute_tx(&mut tx)
                            .await?;
                        let removed = if updated > 0 && blob.is_some() {
                            q("DELETE FROM bodies WHERE blob_ref=?1 AND account_id=?2 AND NOT EXISTS(SELECT 1 FROM messages WHERE blob_ref=?1)").bind(blob.clone()).bind(&account).execute_tx(&mut tx).await?
                        } else {
                            0
                        };
                        tx.commit().await?;
                        report.detached_messages += if blob.is_some() { updated } else { 0 };
                        report.removed_bodies += removed;
                        report.removed_bytes += removed * size;
                        report.cleaned_envelopes += if changed { updated } else { 0 };
                        report.removed_envelope_bytes += updated * saved;
                    }
                    Ok(_) => {
                        tx.commit().await?;
                    }
                    Err(_) => {
                        report.protected_messages += 1;
                        tx.commit().await?;
                    }
                }
            } else {
                // Changed/deleted rows and newly oversized metadata are retried on
                // another bounded scan, never detached using a stale projection.
                report.protected_messages += 1;
                tx.commit().await?;
            }
        }
        report.cursor.expire = id;
        visited += 1;
    }
    if rows.len() < options.batch_rows as usize && visited == rows.len() {
        report.cursor.expire.clear();
    }
    Ok(())
}

/// Deliberate narrow exception to the opaque-envelope seam: expiry removes only
/// the standardized full-body value map, preserving header and part metadata.
fn strip_body_values(
    key: &ServerKey,
    sealed: &Option<Vec<u8>>,
) -> Result<(Option<Vec<u8>>, bool, u64), StoreError> {
    let Some(sealed) = sealed else {
        return Ok((None, false, 0));
    };
    let raw = key.open(sealed)?;
    let mut value: serde_json::Value = serde_json::from_slice(&raw)
        .map_err(|_| StoreError::Corrupt("cached envelope is not JSON".into()))?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| StoreError::Corrupt("cached envelope is not an object".into()))?;
    if object.remove("bodyValues").is_none() {
        return Ok((Some(sealed.clone()), false, 0));
    }
    let encoded = serde_json::to_vec(&value)
        .map_err(|_| StoreError::Corrupt("cached envelope could not be encoded".into()))?;
    let cleaned = key.seal(&encoded)?;
    let saved = sealed.len().saturating_sub(cleaned.len()) as u64;
    Ok((Some(cleaned), true, saved))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey};
    #[tokio::test]
    async fn postgres_expiry_preserves_headers_and_refetch_contract() {
        let Ok(dsn) = std::env::var("MW_BG_TEST_PG") else {
            eprintln!("SKIP live PostgreSQL expiry: MW_BG_TEST_PG is unset");
            return;
        };
        let store = Store::open_postgres(&dsn, ServerKey::generate())
            .await
            .unwrap();
        let username = format!("cache-{}@fixture.invalid", uuid::Uuid::new_v4());
        let account = store
            .create_account(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host: "fixture.invalid",
                    port: 993,
                    tls: "implicit",
                    username: &username,
                    sync_policy_json: "{}",
                },
                &Credentials {
                    username: username.clone(),
                    password: "disposable-fixture".into(),
                },
            )
            .await
            .unwrap();
        let mailbox = store
            .upsert_mailbox(&MailboxUpsert {
                account_id: &account,
                name: "INBOX",
                role: Some("inbox"),
                uidvalidity: 1,
                uidnext: 2,
                highestmodseq: 1,
                total: 1,
                unread: 1,
                parent_id: None,
            })
            .await
            .unwrap();
        let id = message(
            &store,
            &account,
            &mailbox,
            1,
            1,
            b"PostgreSQL refetchable MIME",
        )
        .await;
        let original =
            parsed_envelope(&store, &account, &id, "PostgreSQL parsed body values").await;
        run(&store, &account).await;
        q("UPDATE message_body_cache_lease SET last_used_at=1 WHERE account_id=?1")
            .bind(&account)
            .execute(&store.backend)
            .await
            .unwrap();
        let before = store.get_message(&id).await.unwrap();
        let blob = before.blob_ref.clone().unwrap();
        let report = run(&store, &account).await;
        assert_eq!(report.detached_messages, 1);
        assert_eq!(report.removed_bodies, 1);
        assert_eq!(report.cleaned_envelopes, 1);
        let after = store.get_message(&id).await.unwrap();
        assert!(after.blob_ref.is_none());
        assert_ne!(before.cache_revision, after.cache_revision);
        let mut expected: serde_json::Value = serde_json::from_slice(&original).unwrap();
        expected.as_object_mut().unwrap().remove("bodyValues");
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(
                &store.get_envelope(&id).await.unwrap().unwrap()
            )
            .unwrap(),
            expected
        );
        assert!(store.get_body(&blob).await.unwrap().is_none());
        assert!(
            store
                .cache_refetched_message_body(&after, b"PostgreSQL refetchable MIME")
                .await
                .unwrap()
        );
        assert_eq!(
            store.get_body(&blob).await.unwrap().unwrap(),
            b"PostgreSQL refetchable MIME"
        );
        store.close().await;
    }
    async fn fixture(kind: AccountKind) -> (Store, String, String) {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let account = store
            .create_account(
                &NewAccount {
                    kind,
                    host: "example.invalid",
                    port: 993,
                    tls: "implicit",
                    username: "cache@example.invalid",
                    sync_policy_json: "{}",
                },
                &Credentials {
                    username: "cache@example.invalid".into(),
                    password: "fixture".into(),
                },
            )
            .await
            .unwrap();
        let mailbox = store
            .upsert_mailbox(&MailboxUpsert {
                account_id: &account,
                name: "INBOX",
                role: Some("inbox"),
                uidvalidity: 1,
                uidnext: 100,
                highestmodseq: 1,
                total: 0,
                unread: 0,
                parent_id: None,
            })
            .await
            .unwrap();
        (store, account, mailbox)
    }
    async fn message(
        store: &Store,
        account: &str,
        mailbox: &str,
        uid: u32,
        validity: u32,
        raw: &[u8],
    ) -> String {
        store
            .upsert_message_with_body(
                &MessageUpsert {
                    account_id: account,
                    mailbox_id: mailbox,
                    uid,
                    uidvalidity: validity,
                    message_id: None,
                    thread_id: None,
                    internaldate: None,
                    size: raw.len() as u64,
                    flags_json: "[]",
                    envelope: Some(b"{\"subject\":\"kept header\"}"),
                    blob_ref: None,
                },
                raw,
            )
            .await
            .unwrap()
            .0
    }
    fn options() -> CachedBodyExpiryOptions {
        CachedBodyExpiryOptions {
            max_run_ms: 2000,
            ..Default::default()
        }
    }
    async fn age(store: &Store) {
        q("UPDATE message_body_cache_lease SET last_used_at=1")
            .execute(&store.backend)
            .await
            .unwrap();
    }
    async fn run(store: &Store, account: &str) -> CachedBodyExpiryReport {
        store
            .expire_cached_bodies(options(), Default::default(), vec![account.into()])
            .await
            .unwrap()
    }
    #[tokio::test]
    async fn legacy_adoption_gets_full_sixty_day_grace_and_disabled_mode_is_inert() {
        assert_eq!(CachedBodyExpiryOptions::default().retention_days, 60);
        let (store, account, mailbox) = fixture(AccountKind::Imap).await;
        let id = message(&store, &account, &mailbox, 1, 1, b"legacy").await;
        let disabled = store
            .expire_cached_bodies(
                CachedBodyExpiryOptions {
                    enabled: false,
                    ..options()
                },
                Default::default(),
                vec![account.clone()],
            )
            .await
            .unwrap();
        assert_eq!(disabled.status, "disabled");
        assert_eq!(
            store
                .cached_body_expiry_stats()
                .await
                .unwrap()
                .tracked_messages,
            0
        );
        let report = run(&store, &account).await;
        assert_eq!(report.initialized_messages, 1, "{report:?}");
        assert_eq!(report.detached_messages, 0);
        assert!(store.get_message(&id).await.unwrap().blob_ref.is_some());
        assert!(
            store
                .cached_body_expiry_stats()
                .await
                .unwrap()
                .oldest_use_at
                .unwrap()
                > report.cutoff_at
        );
    }
    #[tokio::test]
    async fn expired_remote_cache_detaches_then_deletes_without_losing_headers() {
        let (store, account, mailbox) = fixture(AccountKind::Imap).await;
        let id = message(&store, &account, &mailbox, 1, 1, b"remote cached MIME").await;
        run(&store, &account).await;
        age(&store).await;
        let old = store.get_message(&id).await.unwrap();
        let blob = old.blob_ref.clone().unwrap();
        let report = run(&store, &account).await;
        assert_eq!(report.detached_messages, 1);
        assert_eq!(report.removed_bodies, 1);
        assert!(report.removed_bytes > 0);
        let after = store.get_message(&id).await.unwrap();
        assert!(after.blob_ref.is_none());
        assert_ne!(after.cache_revision, old.cache_revision);
        assert!(store.get_body(&blob).await.unwrap().is_none());
        assert_eq!(
            store.get_envelope(&id).await.unwrap().unwrap(),
            b"{\"subject\":\"kept header\"}"
        );
        assert!(
            store
                .cache_refetched_message_body(&after, b"remote cached MIME")
                .await
                .unwrap()
        );
        assert_eq!(
            store.get_message(&id).await.unwrap().blob_ref,
            Some(blob.clone())
        );
        assert_eq!(
            store.get_body(&blob).await.unwrap().unwrap(),
            b"remote cached MIME"
        );
    }
    #[tokio::test]
    async fn only_explicit_use_renews_and_repeated_sync_or_reads_do_not() {
        let (store, account, mailbox) = fixture(AccountKind::Imap).await;
        let id = message(&store, &account, &mailbox, 1, 1, b"unchanged raw").await;
        run(&store, &account).await;
        age(&store).await;
        let blob = store.get_message(&id).await.unwrap().blob_ref.unwrap();
        assert!(store.get_body(&blob).await.unwrap().is_some());
        message(&store, &account, &mailbox, 1, 1, b"unchanged raw").await;
        assert_eq!(
            store
                .register_refetchable_cached_body(&account, &id)
                .await
                .unwrap(),
            0
        );
        assert_eq!(
            store
                .cached_body_expiry_stats()
                .await
                .unwrap()
                .oldest_use_at,
            Some(1)
        );
        assert_eq!(
            store
                .renew_cached_body_use("foreign-account", &id)
                .await
                .unwrap(),
            0
        );
        assert_eq!(store.renew_cached_body_use(&account, &id).await.unwrap(), 1);
        assert_eq!(store.renew_cached_body_use(&account, &id).await.unwrap(), 0);
        assert_eq!(run(&store, &account).await.detached_messages, 0);
        age(&store).await;
        assert_eq!(run(&store, &account).await.detached_messages, 1);
    }
    #[tokio::test]
    async fn unsupported_sources_pop3_local_and_draft_copies_are_protected() {
        for kind in [AccountKind::Imap, AccountKind::Pop3] {
            let (store, account, mailbox) = fixture(kind).await;
            let remote = message(&store, &account, &mailbox, 1, 1, b"shared immutable raw").await;
            let local = message(&store, &account, &mailbox, 2, 0, b"shared immutable raw").await;
            let draft = message(&store, &account, &mailbox, 3, 1, b"draft raw").await;
            store.set_flags(&draft, "[\"\\\\Draft\"]").await.unwrap();
            run(&store, &account).await;
            age(&store).await;
            let absent = store
                .expire_cached_bodies(options(), Default::default(), Vec::new())
                .await
                .unwrap();
            assert_eq!(absent.status, "no-refetchable-sources");
            let report = run(&store, &account).await;
            assert_eq!(
                report.detached_messages,
                if kind == AccountKind::Imap { 1 } else { 0 }
            );
            assert_eq!(report.removed_bodies, 0);
            let original = store.get_message(&local).await.unwrap().blob_ref.unwrap();
            assert!(store.get_body(&original).await.unwrap().is_some());
            assert!(store.get_message(&draft).await.unwrap().blob_ref.is_some());
            if kind == AccountKind::Pop3 {
                assert!(store.get_message(&remote).await.unwrap().blob_ref.is_some());
            }
        }
    }
    #[tokio::test]
    async fn changed_coordinates_and_stale_refill_cannot_reuse_old_lease() {
        let (store, account, mailbox) = fixture(AccountKind::Imap).await;
        let id = message(&store, &account, &mailbox, 1, 1, b"old raw").await;
        run(&store, &account).await;
        age(&store).await;
        let old = store.get_message(&id).await.unwrap();
        store.set_flags(&id, "[\"\\\\Seen\"]").await.unwrap();
        assert!(
            !store
                .cache_refetched_message_body(&old, b"stale download")
                .await
                .unwrap()
        );
        assert_eq!(
            store
                .get_body(old.blob_ref.as_deref().unwrap())
                .await
                .unwrap()
                .unwrap(),
            b"old raw"
        );
        message(&store, &account, &mailbox, 1, 1, b"changed raw").await;
        assert_eq!(
            store
                .register_refetchable_cached_body(&account, &id)
                .await
                .unwrap(),
            1
        );
        assert!(
            store
                .cached_body_expiry_stats()
                .await
                .unwrap()
                .oldest_use_at
                .unwrap()
                > 1
        );
        assert_eq!(run(&store, &account).await.detached_messages, 0);
    }
    #[tokio::test]
    async fn oversized_remote_bodies_are_reported_and_remain_intact() {
        let (store, account, mailbox) = fixture(AccountKind::Imap).await;
        let id = message(
            &store,
            &account,
            &mailbox,
            1,
            1,
            &vec![b'X'; 1024 * 1024 + 1],
        )
        .await;
        run(&store, &account).await;
        age(&store).await;
        let report = store
            .expire_cached_bodies(
                CachedBodyExpiryOptions {
                    batch_bytes: 1024 * 1024,
                    ..options()
                },
                Default::default(),
                vec![account],
            )
            .await
            .unwrap();
        assert_eq!(report.oversized_bodies, 1);
        assert_eq!(report.detached_messages, 0);
        assert!(store.get_message(&id).await.unwrap().blob_ref.is_some());
    }
    #[tokio::test]
    async fn unchanged_sync_does_not_refill_expired_body_but_reader_and_new_content_can() {
        let (store, account, mailbox) = fixture(AccountKind::Imap).await;
        let id = message(&store, &account, &mailbox, 1, 1, b"unchanged remote MIME").await;
        run(&store, &account).await;
        age(&store).await;
        let blob = store.get_message(&id).await.unwrap().blob_ref.unwrap();
        assert_eq!(run(&store, &account).await.detached_messages, 1);
        assert_eq!(
            message(&store, &account, &mailbox, 1, 1, b"unchanged remote MIME").await,
            id
        );
        let expired = store.get_message(&id).await.unwrap();
        assert!(
            expired.blob_ref.is_none(),
            "a sync replay must not create an expire/refill loop"
        );
        assert!(store.get_body(&blob).await.unwrap().is_none());
        assert_eq!(
            store
                .cached_body_expiry_stats()
                .await
                .unwrap()
                .oldest_use_at,
            Some(1)
        );
        assert!(
            store
                .cache_refetched_message_body(&expired, b"unchanged remote MIME")
                .await
                .unwrap()
        );
        assert_eq!(
            store.get_message(&id).await.unwrap().blob_ref.as_deref(),
            Some(blob.as_str())
        );
        assert_eq!(
            message(
                &store,
                &account,
                &mailbox,
                1,
                1,
                b"changed authoritative MIME"
            )
            .await,
            id
        );
        assert_ne!(
            store.get_message(&id).await.unwrap().blob_ref.as_deref(),
            Some(blob.as_str())
        );
        assert_eq!(
            store
                .register_refetchable_cached_body(&account, &id)
                .await
                .unwrap(),
            1
        );
        assert!(
            store
                .cached_body_expiry_stats()
                .await
                .unwrap()
                .oldest_use_at
                .unwrap()
                > 1
        );
    }
    async fn parsed_envelope(store: &Store, account: &str, id: &str, value: &str) -> Vec<u8> {
        let envelope = serde_json::to_vec(&serde_json::json!({
            "subject":"kept header","from":[{"email":"sender@example.org"}],
            "preview":"kept preview","bodyValues":{"1":{"value":value,"isTruncated":false}},
            "textBody":[{"partId":"1","type":"text/plain"}],
            "htmlBody":[{"partId":"2","type":"text/html"}],
            "attachments":[{"name":"kept.pdf","blobId":"attachment","type":"application/pdf"}],
            "hasAttachment":true,"_mailwomanAttachmentSchema":1
        }))
        .unwrap();
        let blob = store.get_message(id).await.unwrap().blob_ref.unwrap();
        assert!(
            store
                .repair_message_envelope(account, id, &blob, &envelope)
                .await
                .unwrap()
        );
        envelope
    }
    #[tokio::test]
    async fn parsed_body_values_expire_without_losing_headers_and_sync_cannot_restore_them() {
        let (store, account, mailbox) = fixture(AccountKind::Imap).await;
        let raw = b"unchanged parsed MIME";
        let id = message(&store, &account, &mailbox, 1, 1, raw).await;
        let original = parsed_envelope(&store, &account, &id, "private full body text").await;
        run(&store, &account).await;
        age(&store).await;
        let report = run(&store, &account).await;
        assert_eq!(report.detached_messages, 1);
        assert_eq!(report.cleaned_envelopes, 1);
        assert!(report.removed_envelope_bytes > 0);
        let clean = store.get_envelope(&id).await.unwrap().unwrap();
        let mut expected: serde_json::Value = serde_json::from_slice(&original).unwrap();
        expected.as_object_mut().unwrap().remove("bodyValues");
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&clean).unwrap(),
            expected
        );
        store
            .upsert_message_with_body(
                &MessageUpsert {
                    account_id: &account,
                    mailbox_id: &mailbox,
                    uid: 1,
                    uidvalidity: 1,
                    message_id: None,
                    thread_id: None,
                    internaldate: None,
                    size: raw.len() as u64,
                    flags_json: "[]",
                    envelope: Some(&original),
                    blob_ref: None,
                },
                raw,
            )
            .await
            .unwrap();
        assert!(store.get_message(&id).await.unwrap().blob_ref.is_none());
        assert_eq!(store.get_envelope(&id).await.unwrap().unwrap(), clean);
        assert_eq!(
            store
                .cached_body_expiry_stats()
                .await
                .unwrap()
                .oldest_use_at,
            Some(1)
        );
    }
    #[tokio::test]
    async fn legacy_detached_parsed_values_are_cleaned_once_without_renewing_lease() {
        let (store, account, mailbox) = fixture(AccountKind::Imap).await;
        let id = message(&store, &account, &mailbox, 1, 1, b"legacy parsed MIME").await;
        parsed_envelope(&store, &account, &id, "historical full body").await;
        run(&store, &account).await;
        age(&store).await;
        q("UPDATE messages SET blob_ref=NULL WHERE stable_id=?1")
            .bind(&id)
            .execute(&store.backend)
            .await
            .unwrap();
        let before = store.get_message(&id).await.unwrap().cache_revision;
        let report = run(&store, &account).await;
        assert_eq!(report.detached_messages, 0);
        assert_eq!(report.cleaned_envelopes, 1);
        assert_eq!(report.removed_bodies, 0);
        assert_ne!(store.get_message(&id).await.unwrap().cache_revision, before);
        let clean = store.get_envelope(&id).await.unwrap().unwrap();
        assert!(
            serde_json::from_slice::<serde_json::Value>(&clean)
                .unwrap()
                .get("bodyValues")
                .is_none()
        );
        let again = run(&store, &account).await;
        assert_eq!(again.cleaned_envelopes, 0);
        assert_eq!(store.get_envelope(&id).await.unwrap().unwrap(), clean);
        assert_eq!(
            store
                .cached_body_expiry_stats()
                .await
                .unwrap()
                .oldest_use_at,
            Some(1)
        );
    }
    #[tokio::test]
    async fn malformed_or_unopenable_envelopes_protect_cached_raw_bodies() {
        for unreadable in [false, true] {
            let (store, account, mailbox) = fixture(AccountKind::Imap).await;
            let id = message(&store, &account, &mailbox, 1, 1, b"protected MIME").await;
            run(&store, &account).await;
            age(&store).await;
            let envelope = if unreadable {
                vec![1, 2, 3]
            } else {
                store.key.seal(b"not JSON").unwrap()
            };
            q("UPDATE messages SET envelope_json=?1 WHERE stable_id=?2")
                .bind(envelope.clone())
                .bind(&id)
                .execute(&store.backend)
                .await
                .unwrap();
            let report = run(&store, &account).await;
            assert_eq!(report.detached_messages, 0);
            assert_eq!(report.cleaned_envelopes, 0);
            assert_eq!(report.protected_messages, 1);
            assert!(store.get_message(&id).await.unwrap().blob_ref.is_some());
            assert_eq!(
                q("SELECT envelope_json FROM messages WHERE stable_id=?1")
                    .bind(&id)
                    .fetch_one(&store.backend)
                    .await
                    .unwrap()
                    .get_opt_blob("envelope_json"),
                Some(envelope)
            );
        }
    }
    #[tokio::test]
    async fn oversized_parsed_envelope_is_bounded_before_fetch_and_preserved() {
        let (store, account, mailbox) = fixture(AccountKind::Imap).await;
        let id = message(&store, &account, &mailbox, 1, 1, b"tiny raw MIME").await;
        parsed_envelope(&store, &account, &id, &"x".repeat(1024 * 1024)).await;
        run(&store, &account).await;
        age(&store).await;
        let report = store
            .expire_cached_bodies(
                CachedBodyExpiryOptions {
                    batch_bytes: 1024 * 1024,
                    ..options()
                },
                Default::default(),
                vec![account.clone()],
            )
            .await
            .unwrap();
        assert_eq!(report.oversized_bodies, 1);
        assert_eq!(report.detached_messages, 0);
        assert!(store.get_message(&id).await.unwrap().blob_ref.is_some());
    }
    #[tokio::test]
    async fn canonical_and_protocol_draft_flags_protect_messages_outside_drafts_folder() {
        for flags in ["[\"Draft\"]", "[\"\\\\Draft\"]", "[\"$draft\"]"] {
            let (store, account, mailbox) = fixture(AccountKind::Imap).await;
            let id = message(
                &store,
                &account,
                &mailbox,
                1,
                1,
                b"protected upstream draft",
            )
            .await;
            store.set_flags(&id, flags).await.unwrap();
            run(&store, &account).await;
            // Force an existing legacy lease to test the expiry check as well
            // as protection against initial lease adoption.
            let blob = store.get_message(&id).await.unwrap().blob_ref.unwrap();
            q("INSERT INTO message_body_cache_lease(stable_id,account_id,blob_ref,uidvalidity,uid,last_used_at) VALUES(?1,?2,?3,1,1,1) ON CONFLICT(stable_id) DO UPDATE SET last_used_at=1").bind(&id).bind(&account).bind(&blob).execute(&store.backend).await.unwrap();
            let report = run(&store, &account).await;
            assert_eq!(report.detached_messages, 0, "{flags}");
            assert_eq!(report.protected_messages, 1, "{flags}");
            assert!(store.get_message(&id).await.unwrap().blob_ref.is_some());
        }
    }
}
