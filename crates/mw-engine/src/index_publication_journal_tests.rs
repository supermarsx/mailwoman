//! Journal acknowledgement mode (t34 Workstream B): real Store triggers, real
//! owner, real Tantivy, real `handle_jmap` Email/set + Email/query.
use crate::account::{AccountRuntime, MailSubmitter};
use crate::backend::*;
use crate::index_publication::{PublicationMode, PublicationTestPhase};
use crate::{Engine, PublicationAck, PublicationOptions};
use async_trait::async_trait;
use mw_store::{AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store};
use serde_json::{Value, json};
use std::sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}};
use std::time::Duration;
use tokio::sync::Notify;

struct Remote;
#[async_trait]
impl AccountBackend for Remote {
    async fn capabilities(&self) -> Result<BackendCaps> { Ok(BackendCaps::default()) }
    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> { Ok(vec![]) }
    async fn sync_mailbox(&self, _: &RawMailboxRef, c: &SyncCursor) -> Result<MailboxDelta> {
        Ok(MailboxDelta { added: vec![], flag_changes: vec![], removed: vec![], next_cursor: c.clone() })
    }
    async fn fetch_raw(&self, _: &[MessageRef]) -> Result<Vec<RawMessage>> { Ok(vec![]) }
    async fn store_flags(&self, _: &[MessageRef], _: &[Flag], _: &[Flag]) -> Result<()> { Ok(()) }
    async fn move_messages(&self, _: &[MessageRef], _: &RawMailboxRef) -> Result<MoveOutcome> {
        static NEXT: AtomicUsize = AtomicUsize::new(100);
        Ok(MoveOutcome::Uidplus { uidvalidity: 1, uids: vec![NEXT.fetch_add(1, Ordering::SeqCst) as u32] })
    }
    async fn append(&self, _: &RawMailboxRef, _: &[u8], _: &[Flag]) -> Result<MessageRef> { panic!("not composing") }
    async fn watch(&self, _: ChangeSink) -> Result<WatchHandle> { Err(EngineError::Unsupported("fixture".into())) }
}
struct NoSend;
#[async_trait]
impl MailSubmitter for NoSend {
    async fn submit(&self, _: mw_smtp::Outgoing) -> Result<mw_smtp::SubmissionResult> { panic!("not sending") }
}

struct Switchable(AtomicBool);
impl crate::v6::AccountPostureSource for Switchable {
    fn posture(&self, _: &str) -> crate::v6::AccountPosture {
        if self.0.load(Ordering::Acquire) { crate::v6::AccountPosture::ZeroAccess } else { crate::v6::AccountPosture::Standard }
    }
}

struct Fixture { engine: Engine, store: Store, search: Arc<mw_search::Index>, account: String, inbox: String, archive: String, id: String }

fn journal_options() -> PublicationOptions {
    PublicationOptions { ack: PublicationAck::Journal, collection_window: Duration::ZERO, ..Default::default() }
}

async fn seed() -> (Store, String, String, String, String) {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account = store.create_account(&NewAccount {
        kind: AccountKind::Imap, host: "imap.example.invalid", port: 993, tls: "implicit",
        username: "owner@example.invalid", sync_policy_json: "{}",
    }, &Credentials { username: "owner@example.invalid".into(), password: "fixture-only".into() }).await.unwrap();
    let mut boxes = Vec::new();
    for (name, role) in [("INBOX", "inbox"), ("Archive", "archive")] {
        boxes.push(store.upsert_mailbox(&MailboxUpsert {
            account_id: &account, name, role: Some(role), uidvalidity: 1, uidnext: 2,
            highestmodseq: 0, total: 0, unread: 0, parent_id: None,
        }).await.unwrap());
    }
    let raw = b"Message-ID: <journal@example.invalid>\r\nFrom: sender@example.invalid\r\nTo: owner@example.invalid\r\nSubject: Journal\r\nContent-Type: text/plain\r\n\r\nuniquejournalbody";
    let blob = store.put_body(&account, raw).await.unwrap();
    let email = mw_mime::parse(raw).unwrap().email;
    let envelope = crate::encode_message_envelope(&email, raw);
    let id = store.upsert_message(&MessageUpsert {
        account_id: &account, mailbox_id: &boxes[0], uid: 1, uidvalidity: 1,
        message_id: Some("<journal@example.invalid>"), thread_id: None,
        internaldate: Some("2026-11-20T00:00:00Z"), size: raw.len() as u64,
        flags_json: "[]", envelope: Some(&envelope), blob_ref: Some(&blob),
    }).await.unwrap();
    (store, account, boxes[0].clone(), boxes[1].clone(), id)
}

async fn engine_over(store: &Store, search: &Arc<mw_search::Index>, account: &str, options: PublicationOptions) -> Engine {
    let engine = Engine::with_search_and_publication_options(store.clone(), search.clone(), options).unwrap();
    engine.register_backend(account.to_owned(), AccountRuntime::new(Arc::new(Remote), Arc::new(NoSend), "owner@example.invalid"));
    engine
}

async fn fixture(options: PublicationOptions) -> Fixture {
    let (store, account, inbox, archive, id) = seed().await;
    let search = Arc::new(mw_search::Index::open_in_ram().unwrap());
    let engine = engine_over(&store, &search, &account, options).await;
    let startup = engine.start_index_publication().await.unwrap();
    assert!(startup.ready && startup.replayed_keys == 0, "{startup:?}");
    engine.publish_current_index(&account, vec![id.clone()], PublicationMode::Rebuild).await.unwrap();
    Fixture { engine, store, search, account, inbox, archive, id }
}

struct Gate { entered: Notify, release: Notify, armed: AtomicBool }
fn gate(engine: &Engine, at: PublicationTestPhase) -> Arc<Gate> {
    let gate = Arc::new(Gate { entered: Notify::new(), release: Notify::new(), armed: AtomicBool::new(true) });
    let hold = gate.clone();
    engine.set_publication_test_hook(Some(Arc::new(move |phase| {
        let hold = hold.clone();
        Box::pin(async move {
            if phase == at && hold.armed.swap(false, Ordering::AcqRel) {
                hold.entered.notify_one(); hold.release.notified().await;
            }
        })
    })));
    gate
}
async fn entered(gate: &Gate) { tokio::time::timeout(Duration::from_secs(10), gate.entered.notified()).await.expect("owner reached deterministic phase"); }

async fn jmap(engine: &Engine, account: &str, calls: Value) -> Vec<Value> {
    let out = engine.handle_jmap(account, &json!({
        "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
        "methodCalls": calls,
    })).await;
    out["methodResponses"].as_array().unwrap_or_else(|| panic!("{out}")).clone()
}
fn set_call(id: &str, patch: Value) -> Value { json!(["Email/set", {"update": {(id): patch}}, "s"]) }
fn query_call(filter: Value) -> Value { json!(["Email/query", {"filter": filter}, "q"]) }
fn ids_of(response: &Value) -> Vec<String> {
    assert_eq!(response[0], "Email/query", "{response}");
    let mut ids: Vec<String> = response[1]["ids"].as_array().unwrap().iter().map(|id| id.as_str().unwrap().to_owned()).collect();
    ids.sort(); ids
}
fn updated(response: &Value, id: &str) {
    assert_eq!(response[0], "Email/set", "{response}");
    assert!(response[1]["updated"].as_object().is_some_and(|rows| rows.contains_key(id)), "{response}");
}
/// The mailbox the index document holds. Since t50hp-e32 slice 1 keywords,
/// pin, snooze and follow-up are SQL-only; a move is the Email/set write that
/// still publishes, so the tests below drive publication with moves.
fn doc_mailbox(f: &Fixture) -> String { f.search.fetch_doc(&f.id).unwrap().unwrap().mailbox_id }
fn move_to(mailbox: &str) -> Value { json!({"mailboxIds": {(mailbox): true}}) }
/// An index-backed query (the body term) restricted to one mailbox.
fn in_mailbox_text(mailbox: &str) -> Value { query_call(json!({"inMailbox": mailbox, "text": "uniquejournalbody"})) }
async fn add_mailbox(f: &Fixture, name: &str) -> String {
    f.store.upsert_mailbox(&MailboxUpsert {
        account_id: &f.account, name, role: None, uidvalidity: 1, uidnext: 2,
        highestmodseq: 0, total: 0, unread: 0, parent_id: None,
    }).await.unwrap()
}

#[tokio::test]
async fn journal_email_set_returns_before_index_commit_and_fenced_query_waits_for_it() {
    let f = fixture(journal_options()).await;
    assert!(f.engine.index_publication_status().early_ack);
    let hold = gate(&f.engine, PublicationTestPhase::BeforeExecution);
    // Email/set returns while the owner is held before executing its job.
    let set = tokio::time::timeout(Duration::from_secs(5), jmap(&f.engine, &f.account, json!([set_call(&f.id, move_to(&f.archive))])))
        .await.expect("journal-mode Email/set must not wait for the index commit");
    updated(&set[0], &f.id);
    entered(&hold).await;
    assert_eq!(doc_mailbox(&f), f.inbox, "index not yet published: the ack really was early");
    assert!(f.engine.index_publication_journal_rows().await.unwrap() >= 1, "the SQL commit wrote the durable journal row");
    // A read of the index that starts now must wait for the pending
    // publication. (A metadata-only query is answered from SQL since
    // t50hp-e31 and needs no fence; the text term makes this one index-backed.)
    let engine = &f.engine;
    let read = jmap(engine, &f.account, json!([in_mailbox_text(&f.archive)]));
    tokio::pin!(read);
    assert!(tokio::time::timeout(Duration::from_millis(100), &mut read).await.is_err(), "fenced read must not answer from the stale index");
    hold.release.notify_one();
    let read = tokio::time::timeout(Duration::from_secs(5), read).await.unwrap();
    assert_eq!(ids_of(&read[0]), vec![f.id.clone()]);
    assert_eq!(doc_mailbox(&f), f.archive);
    let status = f.engine.index_publication_status();
    assert!(status.read_fence_waits >= 1 && status.read_fence_timeouts == 0, "{status:?}");
    assert_eq!(f.engine.index_publication_journal_rows().await.unwrap(), 0, "owner acknowledged the journal after the index commit");
}

#[tokio::test]
async fn journal_mutate_then_read_back_in_one_batch_and_search_right_after_flag_and_move() {
    let f = fixture(journal_options()).await;
    // Same request: Email/set then Email/query (index-backed: keyword + fulltext).
    let out = jmap(&f.engine, &f.account, json!([
        set_call(&f.id, json!({"keywords/$flagged": true})),
        query_call(json!({"hasKeyword": "$flagged", "text": "uniquejournalbody"})),
    ])).await;
    updated(&out[0], &f.id);
    assert_eq!(ids_of(&out[1]), vec![f.id.clone()]);
    // Separate requests: move, then search the destination immediately.
    let out = jmap(&f.engine, &f.account, json!([set_call(&f.id, json!({"mailboxIds": {(f.archive.clone()): true}}))])).await;
    updated(&out[0], &f.id);
    let out = jmap(&f.engine, &f.account, json!([query_call(json!({"inMailbox": f.archive, "text": "uniquejournalbody"}))])).await;
    assert_eq!(ids_of(&out[0]), vec![f.id.clone()]);
    let out = jmap(&f.engine, &f.account, json!([query_call(json!({"inMailbox": f.inbox, "text": "uniquejournalbody"}))])).await;
    assert!(ids_of(&out[0]).is_empty());
    // Unflag + query in one batch.
    let out = jmap(&f.engine, &f.account, json!([
        set_call(&f.id, json!({"keywords/$flagged": null})),
        query_call(json!({"hasKeyword": "$flagged"})),
    ])).await;
    assert!(ids_of(&out[1]).is_empty());
}

#[tokio::test]
async fn journal_per_account_publications_apply_in_order_and_latest_source_wins() {
    let f = fixture(journal_options()).await;
    // A third mailbox, so the last move's target differs from the first one's
    // and from the starting mailbox: "first wins" and "nothing published"
    // would both be visible.
    let later = add_mailbox(&f, "Later").await;
    let hold = gate(&f.engine, PublicationTestPhase::BeforeExecution);
    let a = jmap(&f.engine, &f.account, json!([set_call(&f.id, move_to(&f.archive))])).await;
    updated(&a[0], &f.id);
    entered(&hold).await;
    // Two more mutations queue behind the held first one.
    for target in [&f.inbox, &later] {
        let out = jmap(&f.engine, &f.account, json!([set_call(&f.id, move_to(target))])).await;
        updated(&out[0], &f.id);
    }
    assert_eq!(f.engine.index_publication_status().queued_jobs, 2);
    let engine = &f.engine;
    let read = jmap(engine, &f.account, json!([in_mailbox_text(&later)]));
    tokio::pin!(read);
    assert!(tokio::time::timeout(Duration::from_millis(50), &mut read).await.is_err());
    hold.release.notify_one();
    let read = tokio::time::timeout(Duration::from_secs(5), read).await.unwrap();
    assert_eq!(ids_of(&read[0]), vec![f.id.clone()]);
    for earlier in [&f.inbox, &f.archive] {
        let out = jmap(&f.engine, &f.account, json!([in_mailbox_text(earlier)])).await;
        assert!(ids_of(&out[0]).is_empty(), "the last write (Later) is what the query shows: {out:?}");
    }
    assert_eq!(doc_mailbox(&f), later);
    assert_eq!(f.engine.index_publication_journal_rows().await.unwrap(), 0);
}

#[tokio::test]
async fn journal_fence_timeout_is_an_error_never_a_stale_result() {
    let f = fixture(PublicationOptions { read_fence_timeout: Duration::from_millis(100), ..journal_options() }).await;
    let hold = gate(&f.engine, PublicationTestPhase::BeforeExecution);
    // The move publishes; the keyword rides along so the metadata-only query
    // below has something to find (it is SQL and does not journal).
    let mut patch = move_to(&f.archive);
    patch["keywords/$seen"] = json!(true);
    let set = jmap(&f.engine, &f.account, json!([set_call(&f.id, patch)])).await;
    updated(&set[0], &f.id);
    entered(&hold).await;
    let read = jmap(&f.engine, &f.account, json!([in_mailbox_text(&f.archive)])).await;
    assert_eq!(read[0][0], "error", "{read:?}");
    assert_eq!(read[0][1]["type"], "serverFail", "{read:?}");
    assert_eq!(f.engine.index_publication_status().read_fence_timeouts, 1);
    // SQL-only listing is unaffected by the fence.
    let listing = jmap(&f.engine, &f.account, json!([query_call(json!({"inMailbox": f.archive}))])).await;
    assert_eq!(ids_of(&listing[0]), vec![f.id.clone()]);
    // So is a metadata-only query since t50hp-e31 (SQL, already current).
    let metadata = jmap(&f.engine, &f.account, json!([query_call(json!({"hasKeyword": "$seen"}))])).await;
    assert_eq!(ids_of(&metadata[0]), vec![f.id.clone()]);
    assert_eq!(f.engine.index_publication_status().read_fence_timeouts, 1);
    hold.release.notify_one();
    let read = jmap(&f.engine, &f.account, json!([in_mailbox_text(&f.archive)])).await;
    assert_eq!(ids_of(&read[0]), vec![f.id.clone()]);
}

#[tokio::test]
async fn journal_backpressure_refuses_email_set_before_any_sql_change() {
    let options = PublicationOptions { max_pending_jobs: 1, max_batch_jobs: 1, admission_timeout: Duration::from_millis(50), ..journal_options() };
    let f = fixture(options).await;
    let hold = gate(&f.engine, PublicationTestPhase::BeforeExecution);
    updated(&jmap(&f.engine, &f.account, json!([set_call(&f.id, move_to(&f.archive))])).await[0], &f.id);
    entered(&hold).await;
    // The single admission slot is held by the queued job: the next Email/set
    // move waits admission_timeout and fails the id (a per-id SetError since
    // t55) without touching SQL.
    let out = jmap(&f.engine, &f.account, json!([set_call(&f.id, move_to(&f.inbox))])).await;
    assert_eq!(out[0][0], "Email/set", "{out:?}");
    assert_eq!(out[0][1]["notUpdated"][&f.id]["type"], "serverFail", "{out:?}");
    assert_eq!(f.store.get_message(&f.id).await.unwrap().mailbox_id, f.archive, "the refused move changed SQL");
    hold.release.notify_one();
    let read = jmap(&f.engine, &f.account, json!([in_mailbox_text(&f.archive)])).await;
    assert_eq!(ids_of(&read[0]), vec![f.id.clone()]);
    let read = jmap(&f.engine, &f.account, json!([in_mailbox_text(&f.inbox)])).await;
    assert!(ids_of(&read[0]).is_empty(), "{read:?}");
}

#[tokio::test]
async fn crash_between_sql_commit_and_index_commit_is_replayed_at_startup() {
    let (store, account, inbox, archive, id) = seed().await;
    let search = Arc::new(mw_search::Index::open_in_ram().unwrap());
    let first = engine_over(&store, &search, &account, journal_options()).await;
    assert!(first.start_index_publication().await.unwrap().ready);
    first.publish_current_index(&account, vec![id.clone()], PublicationMode::Rebuild).await.unwrap();
    let stale = search.fetch_doc(&id).unwrap().unwrap();
    assert_eq!(stale.mailbox_id, inbox);
    // The owner never gets past BeforeExecution: the "process" dies with the
    // job queued after Email/set was acknowledged.
    let hold = gate(&first, PublicationTestPhase::BeforeExecution);
    let set = jmap(&first, &account, json!([set_call(&id, move_to(&archive))])).await;
    assert_eq!(set[0][0], "Email/set", "{set:?}");
    entered(&hold).await;
    drop(first);
    assert!(store.index_journal_len().await.unwrap() >= 1);
    // Restart: a new index holding only what was committed before the crash.
    let restarted = Arc::new(mw_search::Index::open_in_ram().unwrap());
    restarted.upsert(&stale).unwrap();
    let second = engine_over(&store, &restarted, &account, journal_options()).await;
    let startup = second.start_index_publication().await.unwrap();
    assert!(startup.ready && startup.replayed_keys >= 1 && startup.failed_keys == 0, "{startup:?}");
    assert_eq!(startup.remaining_rows, Some(0));
    assert_eq!(restarted.fetch_doc(&id).unwrap().unwrap().mailbox_id, archive, "replay published the acknowledged move");
    let read = jmap(&second, &account, json!([in_mailbox_text(&archive)])).await;
    assert_eq!(ids_of(&read[0]), vec![id.clone()]);
    let read = jmap(&second, &account, json!([in_mailbox_text(&inbox)])).await;
    assert!(ids_of(&read[0]).is_empty(), "{read:?}");
    hold.release.notify_one();
}

#[tokio::test]
async fn reads_are_held_until_startup_replay_completes() {
    let (store, account, _inbox, _archive, _id) = seed().await;
    let search = Arc::new(mw_search::Index::open_in_ram().unwrap());
    let engine = engine_over(&store, &search, &account, PublicationOptions { read_fence_timeout: Duration::from_millis(100), ..journal_options() }).await;
    let before = jmap(&engine, &account, json!([query_call(json!({"text": "uniquejournalbody"}))])).await;
    assert_eq!(before[0][1]["type"], "serverFail", "no index read before the journal was replayed: {before:?}");
    assert!(engine.start_index_publication().await.unwrap().ready);
    let after = jmap(&engine, &account, json!([query_call(json!({"text": "uniquejournalbody"}))])).await;
    assert_eq!(after[0][0], "Email/query", "{after:?}");
}

#[tokio::test]
async fn erasure_purges_journal_rows_and_a_late_publication_cannot_resurrect() {
    let f = fixture(journal_options()).await;
    let stale = f.engine.reserve_index_publication(&f.account, 1).await.unwrap();
    // A raw membership change: it journals (a flag write no longer does).
    f.store.relocate_message(&f.id, &f.archive, 50, 1).await.unwrap();
    assert!(f.store.index_journal_len().await.unwrap() >= 1);
    let erasure = f.engine.erase_account_index(&f.account).await.unwrap();
    let (_, errors) = f.store.dsr_purge_person(&mw_store::erasure::PersonIds {
        address: "owner@example.invalid".into(), account_ids: vec![f.account.clone()], owners: vec![], principals: vec![],
    }).await;
    assert!(errors.is_empty(), "{errors:?}");
    erasure.complete().await.unwrap();
    assert_eq!(f.store.index_journal_len().await.unwrap(), 0, "erasure purged the account's journal rows");
    assert!(stale.submit(vec![f.id.clone()], PublicationMode::Reconcile).unwrap().await.is_err());
    f.engine.index_publisher.sweep_for_test(i64::MAX).await;
    assert_eq!(f.search.num_docs(), 0);
    assert!(f.search.search(&mw_search::parse_query("body:uniquejournalbody").unwrap(), 10).unwrap().is_empty());
}

#[tokio::test]
async fn full_rebuild_trims_only_captured_rows_and_keeps_concurrent_mutations() {
    let f = fixture(journal_options()).await;
    // Rows the rebuild covers: a raw writer with no publication (a move out of
    // INBOX; the concurrent toggles below never return the message there).
    let later = add_mailbox(&f, "Later").await;
    f.store.relocate_message(&f.id, &f.archive, 50, 1).await.unwrap();
    let captured = f.store.index_journal_max_seq().await.unwrap();
    assert!(captured > 0);
    // Concurrent mutations: after every owner job of the rebuild (including the
    // final FinishFullRebuild, i.e. after every source read), the message moves
    // between Later and Archive.
    let writer = f.store.clone(); let id = f.id.clone();
    let (to_later, to_archive) = (later.clone(), f.archive.clone());
    let toggles = Arc::new(AtomicUsize::new(0)); let count = toggles.clone();
    f.engine.set_publication_test_hook(Some(Arc::new(move |phase| {
        let writer = writer.clone(); let id = id.clone(); let count = count.clone();
        let (to_later, to_archive) = (to_later.clone(), to_archive.clone());
        Box::pin(async move {
            if phase == PublicationTestPhase::BeforeAcknowledgement {
                let n = count.fetch_add(1, Ordering::AcqRel);
                let target = if n.is_multiple_of(2) { &to_later } else { &to_archive };
                writer.relocate_message(&id, target, 100 + n as u32, 1).await.unwrap();
            }
        })
    })));
    let summary = f.engine.rebuild_search_index_all().await.unwrap();
    assert_eq!(summary.failed, 0);
    f.engine.set_publication_test_hook(None);
    assert!(toggles.load(Ordering::Acquire) >= 2);
    let rows = f.store.index_journal_page(0, 100).await.unwrap();
    assert!(!rows.is_empty(), "the mutation after the last publication must survive the trim");
    assert!(rows.iter().all(|row| row.seq > captured), "every captured row was trimmed: {rows:?}");
    // The sweeper reconciles the survivors; source and index agree after it.
    f.engine.index_publisher.sweep_for_test(i64::MAX).await;
    assert_eq!(f.store.index_journal_len().await.unwrap(), 0);
    let source = f.store.get_message(&f.id).await.unwrap().mailbox_id;
    assert_eq!(doc_mailbox(&f), source);
    assert_ne!(doc_mailbox(&f), f.inbox, "the captured raw move was published");
}

#[tokio::test]
async fn publisher_failure_keeps_rows_refuses_reads_and_recovers() {
    let f = fixture(journal_options()).await;
    // Posture is not journaled, so a posture flip during publication keeps the
    // strict revalidation path; flipping on every attempt poisons the owner.
    let posture = Arc::new(Switchable(AtomicBool::new(false)));
    f.engine.attach_v6(crate::v6::V6Hooks::new().with_posture_source(posture.clone()));
    let flip = posture.clone();
    f.engine.set_publication_test_hook(Some(Arc::new(move |phase| {
        let flip = flip.clone();
        Box::pin(async move {
            if phase == PublicationTestPhase::AfterValidation {
                flip.0.fetch_xor(true, Ordering::AcqRel);
            }
        })
    })));
    let set = jmap(&f.engine, &f.account, json!([set_call(&f.id, move_to(&f.archive))])).await;
    updated(&set[0], &f.id);
    let read = jmap(&f.engine, &f.account, json!([in_mailbox_text(&f.archive)])).await;
    assert_eq!(read[0][1]["type"], "serverFail", "poisoned publication must refuse, not serve stale: {read:?}");
    let status = f.engine.index_publication_status();
    assert!(status.poisoned && status.read_fence_refusals >= 1, "{status:?}");
    assert!(f.store.index_journal_len().await.unwrap() >= 1, "failed publication left its rows for retry");
    f.engine.set_publication_test_hook(None);
    posture.0.store(false, Ordering::Release);
    f.engine.recover_index_publication().await.unwrap();
    let read = jmap(&f.engine, &f.account, json!([in_mailbox_text(&f.archive)])).await;
    assert_eq!(ids_of(&read[0]), vec![f.id.clone()]);
    assert_eq!(doc_mailbox(&f), f.archive);
    f.engine.index_publisher.sweep_for_test(i64::MAX).await;
    assert_eq!(f.store.index_journal_len().await.unwrap(), 0);
}

#[tokio::test]
async fn drain_closes_admission_and_waits_for_accepted_jobs() {
    let f = fixture(journal_options()).await;
    let hold = gate(&f.engine, PublicationTestPhase::BeforeExecution);
    updated(&jmap(&f.engine, &f.account, json!([set_call(&f.id, move_to(&f.archive))])).await[0], &f.id);
    entered(&hold).await;
    assert!(!f.engine.drain_index_publication_within(Duration::from_millis(30)).await);
    assert!(f.engine.index_publication_status().draining);
    let refused = f.engine.reserve_index_publication(&f.account, 1).await.err().expect("admission closed");
    assert!(refused.to_string().contains("shutting down"), "{refused}");
    hold.release.notify_one();
    assert!(f.engine.drain_index_publication_within(Duration::from_secs(5)).await);
    assert_eq!(f.store.index_journal_len().await.unwrap(), 0);
    assert_eq!(doc_mailbox(&f), f.archive, "the accepted job was published before drain returned");
}

#[tokio::test]
async fn commit_mode_start_replays_and_disables_a_left_over_journal() {
    let (store, account, _inbox, archive, id) = seed().await;
    let search = Arc::new(mw_search::Index::open_in_ram().unwrap());
    let engine = engine_over(&store, &search, &account, PublicationOptions { collection_window: Duration::ZERO, ..Default::default() }).await;
    engine.publish_current_index(&account, vec![id.clone()], PublicationMode::Rebuild).await.unwrap();
    // An earlier journal-mode run left capture on and one unpublished change.
    store.set_index_journal_enabled(true).await.unwrap();
    store.relocate_message(&id, &archive, 50, 1).await.unwrap();
    let startup = engine.start_index_publication().await.unwrap();
    assert_eq!(startup.mode, PublicationAck::Commit);
    assert!(startup.ready && startup.replayed_keys == 1 && startup.remaining_rows == Some(0), "{startup:?}");
    assert!(!store.index_journal_enabled().await.unwrap(), "commit mode must not leave capture enabled");
    assert_eq!(search.fetch_doc(&id).unwrap().unwrap().mailbox_id, archive);
    let status = engine.index_publication_status();
    assert!(!status.early_ack && !status.journal_replay_pending, "{status:?}");
    // Commit mode with an empty, disabled journal: nothing to do.
    let again = engine.start_index_publication().await.unwrap();
    assert_eq!(again.replayed_keys, 0);
}

#[tokio::test]
async fn commit_mode_email_set_still_waits_and_reads_are_unfenced() {
    let (store, account, _inbox, archive, id) = seed().await;
    let search = Arc::new(mw_search::Index::open_in_ram().unwrap());
    let engine = engine_over(&store, &search, &account, PublicationOptions { collection_window: Duration::ZERO, ..Default::default() }).await;
    engine.publish_current_index(&account, vec![id.clone()], PublicationMode::Rebuild).await.unwrap();
    let hold = gate(&engine, PublicationTestPhase::BeforeExecution);
    let set = jmap(&engine, &account, json!([set_call(&id, move_to(&archive))]));
    tokio::pin!(set);
    assert!(tokio::time::timeout(Duration::from_millis(50), &mut set).await.is_err(), "commit mode acknowledges after the index commit");
    hold.release.notify_one();
    let out = set.await;
    updated(&out[0], &id);
    assert_eq!(search.fetch_doc(&id).unwrap().unwrap().mailbox_id, archive);
    assert!(!store.index_journal_enabled().await.unwrap());
    assert_eq!(store.index_journal_len().await.unwrap(), 0);
    let status = engine.index_publication_status();
    assert_eq!(status.ack_mode, PublicationAck::Commit);
    assert_eq!((status.read_fence_waits, status.journal_rows), (0, None));
}

#[tokio::test]
async fn search_index_status_is_not_ready_while_journal_rows_are_unapplied() {
    let f = fixture(journal_options()).await;
    let status = f.engine.search_index_status().await.unwrap();
    assert!(status.ready() && status.journal_rows == 0, "{status:?}");
    // A raw writer with no publication leaves a row until the sweeper runs.
    f.store.relocate_message(&f.id, &f.archive, 50, 1).await.unwrap();
    let status = f.engine.search_index_status().await.unwrap();
    assert!(status.journal_rows >= 1 && !status.ready(), "{status:?}");
    f.engine.index_publisher.sweep_for_test(i64::MAX).await;
    let status = f.engine.search_index_status().await.unwrap();
    assert!(status.ready() && status.journal_rows == 0, "{status:?}");
    assert_eq!(doc_mailbox(&f), f.archive);
}

#[tokio::test]
async fn commit_mode_search_index_status_reports_zero_journal_rows() {
    let (store, account, _inbox, _archive, id) = seed().await;
    let search = Arc::new(mw_search::Index::open_in_ram().unwrap());
    let engine = engine_over(&store, &search, &account, PublicationOptions { collection_window: Duration::ZERO, ..Default::default() }).await;
    engine.publish_current_index(&account, vec![id], PublicationMode::Rebuild).await.unwrap();
    let status = engine.search_index_status().await.unwrap();
    assert!(status.ready() && status.journal_rows == 0, "{status:?}");
}

async fn add_message(store: &Store, account: &str, mailbox: &str, uid: u32, tag: &str) -> String {
    let raw = format!("Message-ID: <{tag}@example.invalid>\r\nFrom: sender@example.invalid\r\nTo: owner@example.invalid\r\nSubject: {tag}\r\nContent-Type: text/plain\r\n\r\n{tag}");
    let blob = store.put_body(account, raw.as_bytes()).await.unwrap();
    let email = mw_mime::parse(raw.as_bytes()).unwrap().email;
    let envelope = crate::encode_message_envelope(&email, raw.as_bytes());
    let mid = format!("<{tag}@example.invalid>");
    store.upsert_message(&MessageUpsert {
        account_id: account, mailbox_id: mailbox, uid, uidvalidity: 1,
        message_id: Some(&mid), thread_id: None, internaldate: Some("2026-11-20T00:00:00Z"),
        size: raw.len() as u64, flags_json: "[]", envelope: Some(&envelope), blob_ref: Some(&blob),
    }).await.unwrap()
}

/// A second account with INBOX (holding one message) and Archive; returns
/// (account, message id, archive).
async fn second_account(f: &Fixture, engine: &Engine) -> (String, String, String) {
    let account = f.store.create_account(&NewAccount {
        kind: AccountKind::Imap, host: "imap.example.invalid", port: 993, tls: "implicit",
        username: "other@example.invalid", sync_policy_json: "{}",
    }, &Credentials { username: "other@example.invalid".into(), password: "fixture-only".into() }).await.unwrap();
    let mut boxes = Vec::new();
    for (name, role) in [("INBOX", "inbox"), ("Archive", "archive")] {
        boxes.push(f.store.upsert_mailbox(&MailboxUpsert {
            account_id: &account, name, role: Some(role), uidvalidity: 1, uidnext: 2,
            highestmodseq: 0, total: 0, unread: 0, parent_id: None,
        }).await.unwrap());
    }
    let id = add_message(&f.store, &account, &boxes[0], 1, "uniquetenantbody").await;
    engine.register_backend(account.clone(), AccountRuntime::new(Arc::new(Remote), Arc::new(NoSend), "other@example.invalid"));
    engine.publish_current_index(&account, vec![id.clone()], PublicationMode::Rebuild).await.unwrap();
    (account, id, boxes.remove(1))
}

// U1: no caller holds a lock the owner needs while it waits on the fence, so
// concurrent read-after-write of one account converges without a timeout.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_read_after_write_for_one_account_never_times_out() {
    let f = fixture(journal_options()).await;
    let mut ids = vec![f.id.clone()];
    // Each message's own body term: the queries below carry it so they read
    // the index and pass the fence (a flag-only query is SQL since t50hp-e31).
    let mut terms = vec!["uniquejournalbody".to_owned()];
    for uid in 2..=8u32 {
        ids.push(add_message(&f.store, &f.account, &f.inbox, uid, &format!("uniqueconcurrent{uid}")).await);
        terms.push(format!("uniqueconcurrent{uid}"));
    }
    f.engine.publish_current_index(&f.account, ids.clone(), PublicationMode::Rebuild).await.unwrap();
    let account = f.account.clone();
    let (inbox, archive) = (f.inbox.clone(), f.archive.clone());
    let engine = Arc::new(f.engine);
    let started = std::time::Instant::now();
    let mut tasks = tokio::task::JoinSet::new();
    for (id, term) in ids.clone().into_iter().zip(terms) {
        let engine = engine.clone(); let account = account.clone();
        let (inbox, archive) = (inbox.clone(), archive.clone());
        tasks.spawn(async move {
            // Moves publish (a flag write is SQL-only since t50hp-e32).
            for round in 0..4 {
                let archived = round % 2 == 0;
                let target = if archived { &archive } else { &inbox };
                let out = jmap(&engine, &account, json!([
                    set_call(&id, move_to(target)),
                    query_call(json!({"inMailbox": archive, "text": term})),
                ])).await;
                updated(&out[0], &id);
                assert_eq!(ids_of(&out[1]).contains(&id), archived, "read-after-write in one batch: {out:?}");
                let again = jmap(&engine, &account, json!([query_call(json!({"inMailbox": archive, "text": term}))])).await;
                assert_eq!(ids_of(&again[0]).contains(&id), archived, "read-after-write across requests: {again:?}");
            }
        });
    }
    while let Some(done) = tasks.join_next().await { done.unwrap(); }
    let status = engine.index_publication_status();
    assert_eq!((status.read_fence_timeouts, status.read_fence_refusals), (0, 0), "{status:?}");
    assert!(started.elapsed() < Duration::from_secs(5), "{:?}", started.elapsed());
    assert_eq!(f.store.index_journal_len().await.unwrap(), 0);
}

// N4: one account flooding Email/set is capped on its own backlog; another
// account keeps its admission and its fenced read does not wait for the flood.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn one_account_flood_is_capped_and_another_accounts_read_is_not_starved() {
    let options = PublicationOptions { max_pending_jobs_per_account: 2, admission_timeout: Duration::from_millis(300), ..journal_options() };
    let f = fixture(options).await;
    let (other, other_id, other_archive) = second_account(&f, &f.engine).await;
    let hold = gate(&f.engine, PublicationTestPhase::BeforeExecution);
    updated(&jmap(&f.engine, &f.account, json!([set_call(&f.id, move_to(&f.archive))])).await[0], &f.id);
    entered(&hold).await;
    updated(&jmap(&f.engine, &f.account, json!([set_call(&f.id, move_to(&f.inbox))])).await[0], &f.id);
    // Over the cap: refused after admission_timeout, before any SQL change.
    let refused = jmap(&f.engine, &f.account, json!([set_call(&f.id, move_to(&f.archive))])).await;
    // A per-id SetError since t55, not a method-level error.
    assert_eq!((refused[0][0].as_str(), refused[0][1]["notUpdated"][&f.id]["type"].as_str()), (Some("Email/set"), Some("serverFail")), "{refused:?}");
    assert_eq!(f.store.get_message(&f.id).await.unwrap().mailbox_id, f.inbox, "the refused move changed SQL");
    // The other account is admitted at once while the first is at its cap.
    let started = std::time::Instant::now();
    updated(&jmap(&f.engine, &other, json!([set_call(&other_id, move_to(&other_archive))])).await[0], &other_id);
    assert!(started.elapsed() < Duration::from_millis(250), "{:?}", started.elapsed());
    hold.release.notify_one();
    let started = std::time::Instant::now();
    let read = jmap(&f.engine, &other, json!([query_call(json!({"inMailbox": other_archive, "text": "uniquetenantbody"}))])).await;
    assert_eq!(ids_of(&read[0]), vec![other_id.clone()]);
    assert!(started.elapsed() < Duration::from_secs(1), "{:?}", started.elapsed());
    assert_eq!(f.engine.index_publication_status().read_fence_timeouts, 0);
}

// Benchmark defect (t34 bench, snapjournal arm): with early acknowledgement the
// same client's next Email/set (mark read, then restore) commits while the
// owner is between its two source reads. Before the fix every such job retried
// with the source still moving, exhausted revalidation_attempts and poisoned
// the owner; from then on every Email/set failed admission with serverFail.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn journal_mode_back_to_back_mutations_racing_publication_do_not_poison_the_owner() {
    let f = fixture(journal_options()).await;
    let account = f.account.clone(); let id = f.id.clone(); let store = f.store.clone();
    let (inbox, archive) = (f.inbox.clone(), f.archive.clone());
    let engine = Arc::new(f.engine);
    let weak = Arc::downgrade(&engine);
    let races = Arc::new(AtomicUsize::new(0));
    let refused = Arc::new(AtomicUsize::new(0));
    let fired = races.clone(); let failed = refused.clone(); let (hook_account, hook_id) = (account.clone(), id.clone());
    let (hook_inbox, hook_archive) = (inbox.clone(), archive.clone());
    engine.set_publication_test_hook(Some(Arc::new(move |phase| {
        let weak = weak.clone(); let fired = fired.clone(); let failed = failed.clone();
        let (account, id) = (hook_account.clone(), hook_id.clone());
        let (inbox, archive) = (hook_inbox.clone(), hook_archive.clone());
        Box::pin(async move {
            if phase != PublicationTestPhase::AfterValidation { return; }
            let n = fired.fetch_add(1, Ordering::AcqRel);
            if n >= 3 { return; }
            let Some(engine) = weak.upgrade() else { return };
            // The client's next mutation, through the real Email/set path (a
            // move: the write that still publishes since t50hp-e32).
            let target = if n.is_multiple_of(2) { &inbox } else { &archive };
            let out = jmap(&engine, &account, json!([set_call(&id, move_to(target))])).await;
            if out[0][0] != "Email/set" || out[0][1].get("notUpdated").is_some() { failed.fetch_add(1, Ordering::AcqRel); }
        })
    })));
    let first = jmap(&engine, &account, json!([set_call(&id, move_to(&archive))])).await;
    updated(&first[0], &id);
    tokio::time::timeout(Duration::from_secs(10), async {
        while races.load(Ordering::Acquire) < 2 || engine.index_publication_status().pending_jobs != 0 {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }).await.expect("racing publications drain");
    engine.set_publication_test_hook(None);
    let status = engine.index_publication_status();
    assert!(!status.poisoned && status.failed_jobs == 0 && status.pending_repair_messages == 0, "{status:?}");
    assert_eq!(refused.load(Ordering::Acquire), 0, "racing Email/set calls were refused");
    assert!(status.journal_covered_revalidations >= 1 && status.admission_refusals == 0, "{status:?}");
    // Writes keep being admitted, and reads see the final source.
    let current = store.get_message(&id).await.unwrap().mailbox_id;
    let read = jmap(&engine, &account, json!([in_mailbox_text(&current)])).await;
    assert_eq!(ids_of(&read[0]), vec![id.clone()], "{read:?}");
    let other = if current == inbox { archive.clone() } else { inbox.clone() };
    let next = jmap(&engine, &account, json!([set_call(&id, move_to(&other))])).await;
    updated(&next[0], &id);
    let read = jmap(&engine, &account, json!([in_mailbox_text(&other)])).await;
    assert_eq!(ids_of(&read[0]), vec![id.clone()], "{read:?}");
    let read = jmap(&engine, &account, json!([in_mailbox_text(&current)])).await;
    assert!(ids_of(&read[0]).is_empty(), "{read:?}");
    assert_eq!(store.index_journal_len().await.unwrap(), 0);
}

// t50hp-e32 slice 1: keywords, pin, snooze and follow-up are read from SQL, so
// their writes leave the search index (and its journal) alone. In each test
// below the owner is held by a queued move, i.e. index publication is stuck.

/// Holds the owner at BeforeExecution with a move queued (journal mode).
async fn hold_owner_with_a_move(f: &Fixture) -> Arc<Gate> {
    let hold = gate(&f.engine, PublicationTestPhase::BeforeExecution);
    updated(&jmap(&f.engine, &f.account, json!([set_call(&f.id, move_to(&f.archive))])).await[0], &f.id);
    entered(&hold).await;
    hold
}

#[tokio::test]
async fn journal_keyword_and_metadata_writes_add_no_journal_rows() {
    let f = fixture(journal_options()).await;
    let hold = hold_owner_with_a_move(&f).await;
    let before = f.store.index_journal_len().await.unwrap();
    assert!(before >= 1, "the held move journaled (capture is on)");
    let out = jmap(&f.engine, &f.account, json!([set_call(&f.id, json!({"keywords/$flagged": true, "keywords/$seen": true}))])).await;
    updated(&out[0], &f.id);
    let out = jmap(&f.engine, &f.account, json!([set_call(&f.id, json!({"pinned": true, "snoozedUntil": "2026-12-01T10:00:00Z", "followUpAt": "2026-12-02T10:00:00Z"}))])).await;
    updated(&out[0], &f.id);
    assert_eq!(f.store.index_journal_len().await.unwrap(), before, "a keyword/pin/snooze/follow-up write journaled");
    assert_eq!(f.engine.index_publication_status().queued_jobs, 0, "a flag write queued an index job");
    let flags = f.store.get_message(&f.id).await.unwrap().flags_json;
    assert!(flags.contains("Flagged") && flags.contains("Seen"), "{flags}");
    assert!(f.store.get_message_meta(&f.id).await.unwrap().unwrap().pinned);
    hold.release.notify_one();
}

#[tokio::test]
async fn journal_keyword_write_returns_while_the_owner_is_held_and_waits_on_no_fence() {
    let f = fixture(journal_options()).await;
    let hold = hold_owner_with_a_move(&f).await;
    let waits = f.engine.index_publication_status().read_fence_waits;
    let out = tokio::time::timeout(Duration::from_secs(5), jmap(&f.engine, &f.account, json!([set_call(&f.id, json!({"keywords/$flagged": true}))])))
        .await.expect("a keyword write must not wait for the held owner");
    updated(&out[0], &f.id);
    let status = f.engine.index_publication_status();
    assert_eq!((status.read_fence_waits, status.read_fence_timeouts), (waits, 0), "{status:?}");
    hold.release.notify_one();
}

/// In commit mode a move's Email/set waits for its index commit; a keyword
/// write to the same message while that move is held must not.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn commit_mode_keyword_write_does_not_wait_for_a_held_publication() {
    let (store, account, _inbox, archive, id) = seed().await;
    let search = Arc::new(mw_search::Index::open_in_ram().unwrap());
    let engine = Arc::new(engine_over(&store, &search, &account, PublicationOptions { collection_window: Duration::ZERO, ..Default::default() }).await);
    engine.publish_current_index(&account, vec![id.clone()], PublicationMode::Rebuild).await.unwrap();
    let hold = gate(&engine, PublicationTestPhase::BeforeExecution);
    let mover = {
        let (engine, account, id, archive) = (engine.clone(), account.clone(), id.clone(), archive.clone());
        tokio::spawn(async move { jmap(&engine, &account, json!([set_call(&id, move_to(&archive))])).await })
    };
    entered(&hold).await;
    let out = tokio::time::timeout(Duration::from_secs(5), jmap(&engine, &account, json!([
        set_call(&id, json!({"keywords/$flagged": true})),
        query_call(json!({"hasKeyword": "$flagged"})),
    ]))).await.expect("a keyword write must not wait for a held index publication");
    updated(&out[0], &id);
    assert_eq!(ids_of(&out[1]), vec![id.clone()]);
    assert!(!mover.is_finished(), "the move still waits for its index commit");
    hold.release.notify_one();
    updated(&tokio::time::timeout(Duration::from_secs(5), mover).await.unwrap().unwrap()[0], &id);
    assert_eq!(search.fetch_doc(&id).unwrap().unwrap().mailbox_id, archive);
    assert!(store.get_message(&id).await.unwrap().flags_json.contains("Flagged"));
}

#[tokio::test]
async fn journal_keyword_write_is_not_refused_when_admission_is_full() {
    let options = PublicationOptions { max_pending_jobs: 1, max_batch_jobs: 1, admission_timeout: Duration::from_millis(50), ..journal_options() };
    let f = fixture(options).await;
    let hold = hold_owner_with_a_move(&f).await;
    // The slot is full: a move is refused...
    let refused = jmap(&f.engine, &f.account, json!([set_call(&f.id, move_to(&f.inbox))])).await;
    assert_eq!(refused[0][1]["notUpdated"][&f.id]["type"], "serverFail", "{refused:?}");
    let refusals = f.engine.index_publication_status().admission_refusals;
    assert!(refusals >= 1);
    // ...and a keyword write is applied, without waiting admission_timeout.
    let started = std::time::Instant::now();
    let out = jmap(&f.engine, &f.account, json!([set_call(&f.id, json!({"keywords/$flagged": true}))])).await;
    updated(&out[0], &f.id);
    assert!(out[0][1].get("notUpdated").is_none(), "{out:?}");
    assert!(started.elapsed() < Duration::from_millis(50), "{:?}", started.elapsed());
    assert_eq!(f.engine.index_publication_status().admission_refusals, refusals);
    assert!(f.store.get_message(&f.id).await.unwrap().flags_json.contains("Flagged"));
    hold.release.notify_one();
}

#[tokio::test]
async fn journal_keyword_write_then_has_keyword_query_in_one_request_is_current() {
    let f = fixture(journal_options()).await;
    let hold = hold_owner_with_a_move(&f).await;
    // The index cannot have changed (the owner is held); the query is right
    // because it is answered from SQL.
    let out = tokio::time::timeout(Duration::from_secs(5), jmap(&f.engine, &f.account, json!([
        set_call(&f.id, json!({"keywords/$flagged": true})),
        query_call(json!({"hasKeyword": "$flagged"})),
    ]))).await.expect("no fence wait");
    updated(&out[0], &f.id);
    assert_eq!(ids_of(&out[1]), vec![f.id.clone()]);
    let out = tokio::time::timeout(Duration::from_secs(5), jmap(&f.engine, &f.account, json!([
        set_call(&f.id, json!({"keywords/$flagged": null})),
        query_call(json!({"hasKeyword": "$flagged"})),
    ]))).await.expect("no fence wait");
    updated(&out[0], &f.id);
    assert!(ids_of(&out[1]).is_empty(), "{out:?}");
    assert_eq!(f.engine.index_publication_status().read_fence_waits, 0);
    hold.release.notify_one();
}

#[tokio::test]
async fn journal_mode_is_refused_on_options_validation_bounds() {
    let mut options = journal_options();
    options.read_fence_timeout = Duration::from_millis(99);
    assert!(options.validate().is_err());
    options.read_fence_timeout = Duration::from_secs(31);
    assert!(options.validate().is_err());
    options.read_fence_timeout = Duration::from_secs(5);
    options.drain_timeout = Duration::from_secs(61);
    assert!(options.validate().is_err());
    options.drain_timeout = Duration::ZERO;
    assert!(options.validate().is_ok());
    options.max_pending_jobs_per_account = 0;
    assert!(options.validate().is_err());
}
