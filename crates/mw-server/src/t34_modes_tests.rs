//! t34-e8: startup validation matrix and the graceful-shutdown drain.
use super::*;
use mw_engine::{MailSubmitter, PublicationAck, PublicationOptions, account::AccountRuntime, backend::*};
use mw_store::{
    AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store,
    request_protection::ProtectionSource,
    request_protection_model::{Mode as ProtectionMode, Policy, WindowPolicy},
};
use serde_json::{Value, json};

fn validate(db: &str, engine: bool, pairs: &[(&str, &str)]) -> std::result::Result<Modes, String> {
    validate_from(db, engine, |name| {
        pairs.iter().find(|(key, _)| *key == name).map(|(_, value)| (*value).to_owned())
    })
}

const SQLITE: &str = "mail.db";
const PG: &str = "postgres://db/mail";

#[test]
fn unset_is_all_off_in_both_modes_and_both_backends() {
    for db in [SQLITE, PG] {
        for engine in [true, false] {
            let modes = validate(db, engine, &[]).unwrap();
            assert_eq!(modes.guard, snapshot::Mode::Off);
            assert!(!modes.guard_live);
            assert_eq!(modes.accounting, write_behind::AccountingConfig::default());
            assert_eq!(modes.accounting_effective, write_behind::AccountingMode::Synchronous);
            assert!(modes.warnings.is_empty(), "{:?}", modes.warnings);
            match (&modes.index, engine) {
                (Some(options), true) => {
                    let defaults = PublicationOptions::default();
                    assert_eq!(options.ack, PublicationAck::Commit);
                    assert_eq!(options.read_fence_timeout, defaults.read_fence_timeout);
                    assert_eq!(options.drain_timeout, defaults.drain_timeout);
                }
                (None, false) => {}
                other => panic!("{other:?}"),
            }
            assert_eq!(
                modes.summary(),
                format!(
                    "t34 modes: guard snapshot off; index publication {}; request-protection accounting synchronous",
                    if engine { "commit (read fence 5000 ms, shutdown drain 10 s)" } else { "not used (proxy mode)" }
                )
            );
        }
    }
}

#[test]
fn every_mode_on_sqlite_is_accepted_and_reported() {
    let modes = validate(SQLITE, true, &[
        ("MW_GUARD_SNAPSHOT", "bounded"),
        ("MW_GUARD_SNAPSHOT_MAX_STALENESS_MS", "250"),
        ("MW_INDEX_PUBLICATION_ACK", "journal"),
        ("MW_INDEX_PUBLICATION_DRAIN_SECS", "30"),
        ("MW_REQUEST_PROTECTION_ACCOUNTING", "write-behind"),
        ("MW_REQUEST_PROTECTION_FLUSH_MS", "100"),
        ("MW_SQLITE_STARTUP_TIMEOUT_SECONDS", "60"),
        ("MW_SQLITE_WRITER", "off"),
    ])
    .unwrap();
    assert!(modes.guard_live && !modes.postgres && modes.warnings.is_empty());
    assert!(modes.summary().contains("guard snapshot bounded (250 ms)"), "{}", modes.summary());
    assert_eq!(modes.accounting_effective, write_behind::AccountingMode::WriteBehind);
    assert_eq!(
        modes.summary(),
        "t34 modes: guard snapshot bounded (250 ms); index publication journal (read fence 5000 ms, shutdown drain 30 s); request-protection accounting write-behind (flush 100 ms, at most 4096 queued)"
    );
}

#[test]
fn postgres_refuses_journal_and_disables_the_other_two_with_a_warning() {
    let journal = validate(PG, true, &[("MW_INDEX_PUBLICATION_ACK", "journal")]).unwrap_err();
    assert!(journal.contains("MW_INDEX_PUBLICATION_ACK=journal is SQLite-only"), "{journal}");
    // Proxy mode builds no engine: the index variables are inert, not refused.
    let proxy = validate(PG, false, &[("MW_INDEX_PUBLICATION_ACK", "journal")]).unwrap();
    assert!(proxy.index.is_none());
    assert_eq!(proxy.warnings.len(), 1);
    assert!(proxy.warnings[0].contains("not used in proxy mode"));

    let modes = validate(PG, true, &[
        ("MW_GUARD_SNAPSHOT", "bounded"),
        ("MW_REQUEST_PROTECTION_ACCOUNTING", "write-behind"),
        // SQLite-only: not checked for PostgreSQL (mw-store does not read it there).
        ("MW_SQLITE_STARTUP_TIMEOUT_SECONDS", "0"),
        ("MW_SQLITE_WRITER", "group"),
    ])
    .unwrap();
    // Live on PostgreSQL since t49pg: no warning, and the line says what it needs.
    assert!(modes.guard_live && modes.postgres);
    assert_eq!(modes.accounting_effective, write_behind::AccountingMode::Synchronous);
    assert_eq!(modes.warnings.len(), 1, "{:?}", modes.warnings);
    assert!(modes.warnings[0].contains("SQLite-only"), "{:?}", modes.warnings);
    let line = modes.summary();
    assert!(!line.contains("inert"), "{line}");
    assert!(line.contains("guard snapshot bounded (1000 ms; PostgreSQL: multi-process-safe within the bound, one extra database connection per process)"), "{line}");
    assert!(modes.summary().ends_with("request-protection accounting synchronous"));
}

#[test]
fn every_invalid_or_out_of_range_value_refuses_startup_naming_the_variable() {
    for (name, value) in [
        ("MW_GUARD_SNAPSHOT", "on"),
        ("MW_GUARD_SNAPSHOT", "Bounded"),
        ("MW_GUARD_SNAPSHOT_MAX_STALENESS_MS", "99"),
        ("MW_GUARD_SNAPSHOT_MAX_STALENESS_MS", "5001"),
        ("MW_INDEX_PUBLICATION_ACK", "async"),
        ("MW_INDEX_PUBLICATION_WINDOW_MS", "11"),
        ("MW_INDEX_PUBLICATION_MAX_BATCH_JOBS", "0"),
        ("MW_INDEX_PUBLICATION_MAX_BATCH_JOBS", "129"),
        ("MW_INDEX_READ_FENCE_TIMEOUT_MS", "99"),
        ("MW_INDEX_READ_FENCE_TIMEOUT_MS", "30001"),
        ("MW_INDEX_PUBLICATION_DRAIN_SECS", "61"),
        ("MW_REQUEST_PROTECTION_ACCOUNTING", "async"),
        ("MW_REQUEST_PROTECTION_FLUSH_MS", "49"),
        ("MW_REQUEST_PROTECTION_FLUSH_MS", "5001"),
        ("MW_REQUEST_PROTECTION_MAX_PENDING", "63"),
        ("MW_REQUEST_PROTECTION_MAX_PENDING", "65537"),
        ("MW_SQLITE_STARTUP_TIMEOUT_SECONDS", "0"),
        ("MW_SQLITE_STARTUP_TIMEOUT_SECONDS", "86401"),
        ("MW_SQLITE_STARTUP_TIMEOUT_SECONDS", ""),
        ("MW_SQLITE_WRITER", "on"),
        ("MW_SQLITE_WRITER", "group"),
    ] {
        let error = validate(SQLITE, true, &[(name, value)]).unwrap_err();
        assert!(error.contains(name), "{name}={value:?}: {error}");
    }
    // Proxy mode still refuses the switches it uses.
    assert!(validate(SQLITE, false, &[("MW_GUARD_SNAPSHOT", "on")]).is_err());
    assert!(validate(SQLITE, false, &[("MW_REQUEST_PROTECTION_ACCOUNTING", "x")]).is_err());
    // Empty means default for the t34 switches (as their own parsers say).
    assert!(validate(SQLITE, true, &[("MW_GUARD_SNAPSHOT", ""), ("MW_INDEX_PUBLICATION_ACK", " ")]).is_ok());
    for value in ["", "gate", "off"] {
        assert!(validate(SQLITE, true, &[("MW_SQLITE_WRITER", value)]).is_ok(), "{value:?}");
    }
}

// ---- shutdown: pending journal publication and queued accounting ----------

struct Remote;
#[async_trait::async_trait]
impl AccountBackend for Remote {
    async fn capabilities(&self) -> Result<BackendCaps> { Ok(BackendCaps::default()) }
    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> { Ok(vec![]) }
    async fn sync_mailbox(&self, _: &RawMailboxRef, c: &SyncCursor) -> Result<MailboxDelta> {
        Ok(MailboxDelta { added: vec![], flag_changes: vec![], removed: vec![], next_cursor: c.clone() })
    }
    async fn fetch_raw(&self, _: &[MessageRef]) -> Result<Vec<RawMessage>> { Ok(vec![]) }
    async fn store_flags(&self, _: &[MessageRef], _: &[Flag], _: &[Flag]) -> Result<()> { Ok(()) }
    async fn move_messages(&self, _: &[MessageRef], _: &RawMailboxRef) -> Result<MoveOutcome> {
        Err(EngineError::Unsupported("fixture".into()))
    }
    async fn append(&self, _: &RawMailboxRef, _: &[u8], _: &[Flag]) -> Result<MessageRef> {
        Err(EngineError::Unsupported("fixture".into()))
    }
    async fn watch(&self, _: ChangeSink) -> Result<WatchHandle> { Err(EngineError::Unsupported("fixture".into())) }
}
struct NoSend;
#[async_trait::async_trait]
impl MailSubmitter for NoSend {
    async fn submit(&self, _: mw_smtp::Outgoing) -> Result<mw_smtp::SubmissionResult> {
        Err(EngineError::Unsupported("fixture".into()))
    }
}

const MESSAGES: u32 = 12;
const NO_SOURCE: ProtectionSource<'static> = ProtectionSource { country: None, ip: None };

/// One account with `MESSAGES` indexed messages in a journal-mode engine.
async fn journal_engine(store: &Store) -> (Arc<Engine>, String, Vec<String>) {
    let account = store.create_account(&NewAccount {
        kind: AccountKind::Imap, host: "imap.example.invalid", port: 993, tls: "implicit",
        username: "owner@example.invalid", sync_policy_json: "{}",
    }, &Credentials { username: "owner@example.invalid".into(), password: "fixture-only".into() }).await.unwrap();
    let inbox = store.upsert_mailbox(&MailboxUpsert {
        account_id: &account, name: "INBOX", role: Some("inbox"), uidvalidity: 1, uidnext: MESSAGES + 1,
        highestmodseq: 0, total: 0, unread: 0, parent_id: None,
    }).await.unwrap();
    let mut ids = Vec::new();
    for uid in 1..=MESSAGES {
        let raw = format!("Message-ID: <drain{uid}@example.invalid>\r\nFrom: sender@example.invalid\r\nTo: owner@example.invalid\r\nSubject: Drain {uid}\r\nContent-Type: text/plain\r\n\r\nshutdowndrainbody");
        let blob = store.put_body(&account, raw.as_bytes()).await.unwrap();
        let email = mw_mime::parse(raw.as_bytes()).unwrap().email;
        let envelope = mw_engine::encode_message_envelope(&email, raw.as_bytes());
        let message_id = format!("<drain{uid}@example.invalid>");
        ids.push(store.upsert_message(&MessageUpsert {
            account_id: &account, mailbox_id: &inbox, uid, uidvalidity: 1,
            message_id: Some(&message_id), thread_id: None,
            internaldate: Some("2026-11-20T00:00:00Z"), size: raw.len() as u64,
            flags_json: "[]", envelope: Some(&envelope), blob_ref: Some(&blob),
        }).await.unwrap());
    }
    let options = PublicationOptions {
        ack: PublicationAck::Journal,
        // The widest grouping window, so acknowledged jobs are still queued
        // when the drain starts.
        collection_window: Duration::from_millis(10),
        ..Default::default()
    };
    let search = Arc::new(mw_search::Index::open_in_ram().unwrap());
    let engine = Arc::new(Engine::with_search_and_publication_options(store.clone(), search, options).unwrap());
    engine.register_backend(account.clone(), AccountRuntime::new(Arc::new(Remote), Arc::new(NoSend), "owner@example.invalid"));
    let startup = engine.start_index_publication().await.unwrap();
    assert!(startup.ready, "{startup:?}");
    engine.reconcile_search_index().await.unwrap();
    (engine, account, ids)
}

async fn jmap(engine: &Engine, account: &str, calls: Value) -> Value {
    engine.handle_jmap(account, &json!({
        "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
        "methodCalls": calls,
    })).await
}

fn policy() -> Policy {
    let window = |seconds| WindowPolicy { seconds, minimum_samples: 2, absolute_anomalies: 2, ratio_percent: 50 };
    Policy {
        rate_mode: ProtectionMode::Enforce,
        adaptive_mode: ProtectionMode::Enforce,
        rate_limit: 1000,
        rate_window_secs: 60,
        short: window(60),
        long: window(3600),
        normal_cap_per_bucket: 1000,
        signals: 63,
        throttle_secs: 3,
        block_secs: 300,
        decay_secs: 3600,
        max_stage: 2,
    }
}

async fn rate_count(store: &Store, value: &str, now: i64) -> Option<u32> {
    let key = store.request_protection_subject("account", value).unwrap();
    store.request_protection_load(&key, "account", now).await.unwrap().map(|row| row.state.snapshot().rate_count)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shutdown_applies_pending_journal_publication_and_queued_accounting_before_release() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (engine, account, ids) = journal_engine(&store).await;
    assert!(engine.index_publication_status().early_ack, "journal mode acknowledges before publishing");

    // Write-behind accounting with a long interval: nothing flushes on its own.
    let protection = crate::request_protection::Runtime::start_with(
        store.clone(),
        write_behind::AccountingConfig {
            mode: write_behind::AccountingMode::WriteBehind,
            flush_every: Duration::from_millis(5000),
            max_pending: 4096,
        },
    );
    let accounting = protection.write_behind().unwrap();
    let p = policy();
    for _ in 0..8 {
        assert!(!accounting.record("account", "drain", &p, 1000, None, NO_SOURCE).await.decision.enforced);
    }
    // The first observation creates the row synchronously; seven are queued.
    assert_eq!(accounting.status().pending, 7);
    assert_eq!(rate_count(&store, "drain", 1000).await, Some(1));

    // Journal mode: every Email/set returns after its SQL commit, before the
    // index commit, so the drain starts with accepted, unpublished jobs.
    let calls: Vec<Value> = ids.iter().enumerate()
        .map(|(n, id)| json!(["Email/set", {"update": {(id): {"keywords/$seen": true}}}, format!("s{n}")]))
        .collect();
    let mut sets = Vec::new();
    for call in calls {
        let engine = engine.clone();
        let account = account.clone();
        sets.push(tokio::spawn(async move { jmap(&engine, &account, json!([call])).await }));
    }
    for set in sets {
        let reply = set.await.unwrap();
        let updated = &reply["methodResponses"][0][1]["updated"];
        assert!(updated.as_object().is_some_and(|rows| rows.len() == 1), "{reply}");
    }
    let before = engine.index_publication_status();
    let rows_before = engine.index_publication_journal_rows().await.unwrap();
    eprintln!("[t34_modes] before drain: pending_jobs={} journal_rows={rows_before}", before.pending_jobs);

    // Request history buckets recorded since the last 15-second save.
    let history = crate::request_history::Recorder::new();
    let now = chrono::Utc::now().timestamp();
    for _ in 0..3 {
        history.record_at(now, 200, 1500, None);
    }
    let client_timing = crate::client_timing::Recorder::start(store.clone());

    let held = Held::new(store.clone(), Some(engine.clone()), protection, history, client_timing);
    let report = held.drain(ACCOUNTING_FLUSH_BUDGET).await;
    assert_eq!(report, DrainReport::NOTHING_HELD, "every step finished within its budget");

    // Every accepted publication was applied: no pending job, no journal row,
    // and the index answers the new keyword for every message.
    let after = engine.index_publication_status();
    assert_eq!(after.pending_jobs, 0, "{after:?}");
    assert!(after.draining, "admission closed by the drain");
    assert_eq!(engine.index_publication_journal_rows().await.unwrap(), 0);
    let query = jmap(&engine, &account, json!([["Email/query", {"filter": {"hasKeyword": "$seen"}}, "q"]])).await;
    let mut found: Vec<String> = query["methodResponses"][0][1]["ids"].as_array().unwrap_or_else(|| panic!("{query}"))
        .iter().map(|id| id.as_str().unwrap().to_owned()).collect();
    found.sort();
    let mut expected = ids.clone();
    expected.sort();
    assert_eq!(found, expected);

    // Every queued accounting observation was committed before release.
    assert_eq!(rate_count(&store, "drain", 1000).await, Some(8));
    // The request-history buckets were saved without waiting for the timer.
    let minute = now / 60 * 60;
    let saved = store.request_metrics_history(minute - 60, minute + 120, 60).await.unwrap();
    assert_eq!(saved.iter().map(|bucket| bucket.counts.requests).sum::<u64>(), 3, "{saved:?}");
}

#[tokio::test]
async fn without_a_held_app_the_drain_is_a_no_op_and_nothing_is_resurrected() {
    // No `hold_for_shutdown` in this process: nothing to drain.
    assert_eq!(drain_for_shutdown().await, DrainReport::NOTHING_HELD);
    // A registered app that was dropped before `hold_for_shutdown` is not kept.
    if let Ok(mut slot) = REGISTERED.lock() {
        *slot = Some(Weak::new());
    }
    assert!(!hold_for_shutdown());
    assert_eq!(drain_for_shutdown().await, DrainReport::NOTHING_HELD);
}
