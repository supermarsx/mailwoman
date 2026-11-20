//! Actual owner, Store, MIME and Tantivy contracts; no SQL-only index assertions.
use crate::{Engine, PublicationOptions, index_publication::{PublicationMode, PublicationTestPhase}};
use mw_store::{Store, ServerKey, NewAccount, AccountKind, Credentials, MailboxUpsert, MessageUpsert};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::Duration;
use tokio::sync::Notify;

struct Fixture { engine: Engine, account: String, inbox: String, archive: String, id: String }
async fn fixture(mut options: PublicationOptions) -> Fixture {
    options.collection_window = Duration::ZERO;
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account = store.create_account(&NewAccount {
        kind: AccountKind::Imap, host: "imap.example.invalid", port: 993, tls: "implicit",
        username: "owner@example.invalid", sync_policy_json: "{}",
    }, &Credentials { username: "owner@example.invalid".into(), password: "fixture-only".into() }).await.unwrap();
    let mut folders = Vec::new();
    for name in ["INBOX", "Archive"] {
        folders.push(store.upsert_mailbox(&MailboxUpsert {
            account_id: &account, name, role: None, uidvalidity: 1, uidnext: 2,
            highestmodseq: 0, total: 0, unread: 0, parent_id: None,
        }).await.unwrap());
    }
    let raw = b"Message-ID: <publication@example.invalid>\r\nFrom: sender@example.invalid\r\nTo: owner@example.invalid\r\nSubject: Publication\r\nContent-Type: text/plain\r\n\r\nuniquepublicationbody";
    let blob = store.put_body(&account, raw).await.unwrap();
    let id = store.upsert_message(&MessageUpsert {
        account_id: &account, mailbox_id: &folders[0], uid: 1, uidvalidity: 1,
        message_id: Some("<publication@example.invalid>"), thread_id: None,
        internaldate: Some("2026-11-20T00:00:00Z"), size: raw.len() as u64,
        flags_json: "[]", envelope: None, blob_ref: Some(&blob),
    }).await.unwrap();
    let engine = Engine::with_search_and_publication_options(store, Arc::new(mw_search::Index::open_in_ram().unwrap()), options).unwrap();
    engine.publish_current_index(&account, vec![id.clone()], PublicationMode::Reconcile).await.unwrap();
    Fixture { engine, account, inbox: folders[0].clone(), archive: folders[1].clone(), id }
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
async fn drained(engine: &Engine) {
    tokio::time::timeout(Duration::from_secs(10), async {
        while engine.index_publication_status().pending_jobs != 0 { tokio::time::sleep(Duration::from_millis(1)).await; }
    }).await.expect("accepted jobs release all admission permits");
    assert_eq!(engine.index_publication_status().pending_bytes, 0);
}
fn ids(engine: &Engine, expression: mw_search::Expr) -> Vec<String> {
    let mut ids = engine.search().search(&mw_search::SearchQuery { raw: String::new(), expr: expression, sort: Default::default() }, 100).unwrap();
    ids.sort(); ids
}

#[tokio::test]
async fn post_validation_source_change_is_repaired_before_query_visible_ack() {
    let f = fixture(Default::default()).await;
    let hold = gate(&f.engine, PublicationTestPhase::AfterValidation);
    let waiter = f.engine.reserve_index_publication(&f.account, 1).await.unwrap().submit(vec![f.id.clone()], PublicationMode::Reconcile).unwrap();
    entered(&hold).await;
    // Independent authoritative caller writes AFTER the initial validation.
    let writer = f.engine.store().clone();
    writer.set_flags(&f.id, "[\"Seen\"]").await.unwrap();
    writer.relocate_message(&f.id, &f.archive, 2, 1).await.unwrap();
    hold.release.notify_one();
    waiter.await.unwrap();
    assert!(f.engine.index_publication_status().revalidations >= 1);
    assert_eq!(ids(&f.engine, mw_search::Expr::And(vec![
        mw_search::Expr::Clause(mw_search::Clause::Mailbox(f.archive.clone())),
        mw_search::Expr::Clause(mw_search::Clause::Keyword("$seen".into())),
    ])), [f.id.clone()]);
    assert!(ids(&f.engine, mw_search::Expr::Clause(mw_search::Clause::Mailbox(f.inbox.clone()))).is_empty());
    assert_eq!(f.engine.search().search(&mw_search::parse_query("body:uniquepublicationbody").unwrap(), 100).unwrap(), [f.id]);
    drained(&f.engine).await;
}

struct Switchable(AtomicBool);
impl crate::v6::AccountPostureSource for Switchable {
    fn posture(&self, _: &str) -> crate::v6::AccountPosture {
        if self.0.load(Ordering::Acquire) { crate::v6::AccountPosture::ZeroAccess } else { crate::v6::AccountPosture::Standard }
    }
}

#[tokio::test]
async fn post_validation_zero_access_removes_fulltext_before_ack() {
    let f = fixture(Default::default()).await;
    let posture = Arc::new(Switchable(AtomicBool::new(false)));
    f.engine.attach_v6(crate::v6::V6Hooks::new().with_posture_source(posture.clone()));
    let hold = gate(&f.engine, PublicationTestPhase::AfterValidation);
    let waiter = f.engine.reserve_index_publication(&f.account, 1).await.unwrap().submit(vec![f.id.clone()], PublicationMode::Reconcile).unwrap();
    entered(&hold).await; posture.0.store(true, Ordering::Release); hold.release.notify_one();
    let receipt = waiter.await.unwrap();
    assert!(receipt.suppressed);
    assert_eq!(f.engine.search().num_docs(), 0);
    assert!(f.engine.search().search(&mw_search::parse_query("body:uniquepublicationbody").unwrap(), 100).unwrap().is_empty());
    drained(&f.engine).await;
}

#[tokio::test]
async fn canceled_waiter_keeps_job_and_byte_permits_until_actual_completion() {
    let options = PublicationOptions { max_pending_jobs: 1, max_batch_jobs: 1, admission_timeout: Duration::from_millis(25), ..Default::default() };
    let f = fixture(options).await;
    f.engine.store().set_flags(&f.id, "[\"Seen\"]").await.unwrap();
    let hold = gate(&f.engine, PublicationTestPhase::BeforeAcknowledgement);
    let waiter = f.engine.reserve_index_publication(&f.account, 1).await.unwrap().submit(vec![f.id.clone()], PublicationMode::Reconcile).unwrap();
    entered(&hold).await; drop(waiter);
    assert_eq!(f.engine.index_publication_status().pending_jobs, 1);
    assert!(f.engine.index_publication_status().pending_bytes > 0);
    assert!(f.engine.reserve_index_publication(&f.account, 1).await.is_err());
    hold.release.notify_one(); drained(&f.engine).await;
    let ticket = f.engine.reserve_index_publication(&f.account, 1).await.unwrap();
    drop(ticket); drained(&f.engine).await;
    assert_eq!(ids(&f.engine, mw_search::Expr::Clause(mw_search::Clause::Keyword("$seen".into()))), [f.id]);
}

#[tokio::test]
async fn erasure_invalidates_previously_reserved_work_and_drop_keeps_fence_closed() {
    let f = fixture(Default::default()).await;
    let stale = f.engine.reserve_index_publication(&f.account, 1).await.unwrap();
    let erasure = f.engine.erase_account_index(&f.account).await.unwrap();
    assert_eq!(f.engine.search().num_docs(), 0);
    assert!(stale.submit(vec![f.id.clone()], PublicationMode::Rebuild).unwrap().await.is_err());
    assert!(f.engine.reserve_index_publication(&f.account, 1).await.is_err());
    drop(erasure);
    assert!(f.engine.reserve_index_publication(&f.account, 1).await.is_err());
    assert!(f.engine.search().search(&mw_search::parse_query("body:uniquepublicationbody").unwrap(), 100).unwrap().is_empty());
    drained(&f.engine).await;
}

#[tokio::test]
async fn repeated_instability_withholds_ack_and_retains_bounded_dirty_work() {
    let f = fixture(Default::default()).await;
    let writer = f.engine.store().clone(); let id = f.id.clone();
    let seen = Arc::new(AtomicBool::new(false));
    f.engine.set_publication_test_hook(Some(Arc::new(move |phase| {
        let writer = writer.clone(); let id = id.clone(); let seen = seen.clone();
        Box::pin(async move {
            if phase == PublicationTestPhase::AfterValidation {
                let was_seen = seen.fetch_xor(true, Ordering::AcqRel);
                writer.set_flags(&id, if was_seen { "[]" } else { "[\"Seen\"]" }).await.unwrap();
            }
        })
    })));
    assert!(f.engine.publish_current_index(&f.account, vec![f.id.clone()], PublicationMode::Reconcile).await.is_err());
    drained(&f.engine).await;
    let status = f.engine.index_publication_status();
    assert!(status.poisoned);
    assert_eq!(status.pending_repair_messages, 1);
    assert_eq!(status.revalidations, 2);
    assert!(f.engine.reserve_index_publication(&f.account, 0).await.is_err());
    // Repair executes behind the closed ordinary-admission barrier. It must
    // reconcile actual authoritative state before reopening new publications.
    f.engine.set_publication_test_hook(None);
    f.engine.recover_index_publication().await.unwrap();
    assert!(!f.engine.index_publication_status().poisoned);
    assert_eq!(f.engine.index_publication_status().pending_repair_messages, 0);
    assert!(ids(&f.engine, mw_search::Expr::Clause(mw_search::Clause::Keyword("$seen".into()))).is_empty());
    f.engine.store().set_flags(&f.id, "[\"Seen\"]").await.unwrap();
    f.engine.publish_current_index(&f.account, vec![f.id.clone()], PublicationMode::Reconcile).await.unwrap();
    assert_eq!(ids(&f.engine, mw_search::Expr::Clause(mw_search::Clause::Keyword("$seen".into()))), [f.id]);
    drained(&f.engine).await;
}

#[tokio::test]
async fn dropping_last_producer_and_ticket_releases_dedicated_owner() {
    let f = fixture(Default::default()).await;
    let search = Arc::downgrade(f.engine.search_handle());
    let ticket = f.engine.reserve_index_publication(&f.account, 1).await.unwrap();
    drop(f);
    assert!(search.upgrade().is_some(), "outstanding ticket retains the worker's channel");
    drop(ticket);
    tokio::time::timeout(Duration::from_secs(10), async {
        while search.upgrade().is_some() { tokio::time::sleep(Duration::from_millis(1)).await; }
    }).await.expect("owner exits without an Engine/Publisher ownership cycle");
}

#[tokio::test]
async fn queued_distinct_requests_coalesce_one_real_publication_and_ack_all() {
    let f = fixture(Default::default()).await;
    let hold = gate(&f.engine, PublicationTestPhase::BeforeExecution);
    // This already-current source is a no-op blocker, not a fake measured
    // update. The distinct following jobs are all queued before release.
    let blocker = f.engine.reserve_index_publication(&f.account, 1).await.unwrap()
        .submit(vec![f.id.clone()], PublicationMode::Reconcile).unwrap();
    entered(&hold).await;
    let before = f.engine.index_publication_status();
    let mut expected = Vec::new(); let mut waiters = Vec::new();
    for uid in 10..13 {
        let ticket = f.engine.reserve_index_publication(&f.account, 1).await.unwrap();
        let raw = format!("Message-ID: <batch-{uid}@example.invalid>\r\nFrom: sender@example.invalid\r\nSubject: Distinct queued message\r\nContent-Type: text/plain\r\n\r\nuniquecoalescedbody");
        let blob = f.engine.store().put_body(&f.account, raw.as_bytes()).await.unwrap();
        let mid = format!("<batch-{uid}@example.invalid>");
        let id = f.engine.store().upsert_message(&MessageUpsert {
            account_id: &f.account, mailbox_id: &f.inbox, uid, uidvalidity: 1,
            message_id: Some(&mid), thread_id: None, internaldate: Some("2026-11-20T00:00:00Z"),
            size: raw.len() as u64, flags_json: "[]", envelope: None, blob_ref: Some(&blob),
        }).await.unwrap();
        f.engine.store().set_flags(&id, "[\"Seen\"]").await.unwrap();
        f.engine.store().relocate_message(&id, &f.archive, uid, 1).await.unwrap();
        waiters.push(ticket.submit(vec![id.clone()], PublicationMode::Reconcile).unwrap());
        expected.push(id);
    }
    assert_eq!(f.engine.index_publication_status().queued_jobs, 3);
    hold.release.notify_one();
    assert!(blocker.await.unwrap().publication.is_none());
    let mut publications = Vec::new();
    for waiter in waiters { publications.push(waiter.await.unwrap().publication.expect("each caller receives the actual grouped publication receipt")); }
    assert!(publications.iter().all(|receipt| *receipt == publications[0]));
    assert_eq!(publications[0].changes, 3);
    let after = f.engine.index_publication_status();
    assert_eq!(after.published_batches - before.published_batches, 1);
    assert_eq!(after.published_changes - before.published_changes, 3);
    expected.sort();
    assert_eq!(ids(&f.engine, mw_search::Expr::And(vec![
        mw_search::Expr::Clause(mw_search::Clause::Mailbox(f.archive.clone())),
        mw_search::Expr::Clause(mw_search::Clause::Keyword("$seen".into())),
    ])), expected);
    let mut fulltext = f.engine.search().search(&mw_search::parse_query("body:uniquecoalescedbody").unwrap(), 100).unwrap();
    fulltext.sort(); assert_eq!(fulltext, expected);
    drained(&f.engine).await;
}

#[tokio::test]
async fn older_erasure_guard_cannot_complete_a_newer_erasure_epoch() {
    let f = fixture(Default::default()).await;
    let old = f.engine.erase_account_index(&f.account).await.unwrap();
    let new = f.engine.erase_account_index(&f.account).await.unwrap();
    let failure = old.complete().await.unwrap_err();
    assert!(failure.to_string().contains("generation"), "old guard must fail on epoch, before authoritative account removal can be considered");
    assert_eq!(f.engine.index_publication_status().erasing_accounts, 1);
    assert!(f.engine.reserve_index_publication(&f.account, 1).await.is_err());
    drop(new);
    assert!(f.engine.reserve_index_publication(&f.account, 1).await.is_err());
    drained(&f.engine).await;
}

async fn replace_envelope_without_raw(f: &Fixture, envelope: &[u8]) {
    let row = f.engine.store().get_message(&f.id).await.unwrap();
    assert!(row.blob_ref.is_none(), "this helper updates an already raw-free legacy row; None does not detach cached bodies");
    let id = f.engine.store().upsert_message(&MessageUpsert {
        account_id: &row.account_id, mailbox_id: &row.mailbox_id,
        uid: row.uid, uidvalidity: row.uidvalidity, message_id: row.message_id.as_deref(),
        thread_id: row.thread_id.as_deref(), internaldate: row.internaldate.as_deref(),
        size: row.size, flags_json: &row.flags_json, envelope: Some(envelope), blob_ref: None,
    }).await.unwrap();
    assert_eq!(id, f.id);
    assert!(f.engine.store().get_message(&f.id).await.unwrap().blob_ref.is_none());
}

async fn legacy_metadata_fixture() -> Fixture {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account = store.create_account(&NewAccount {
        kind: AccountKind::Imap, host: "imap.example.invalid", port: 993, tls: "implicit",
        username: "legacy@example.invalid", sync_policy_json: "{}",
    }, &Credentials { username: "legacy@example.invalid".into(), password: "fixture-only".into() }).await.unwrap();
    let mut folders = Vec::new();
    for name in ["INBOX", "Archive"] {
        folders.push(store.upsert_mailbox(&MailboxUpsert {
            account_id: &account, name, role: None, uidvalidity: 1, uidnext: 2,
            highestmodseq: 0, total: 0, unread: 0, parent_id: None,
        }).await.unwrap());
    }
    // Insert raw-free from the outset. An upsert with blob_ref=None preserves
    // an existing cache, so it cannot model detachment of the normal fixture.
    let id = store.upsert_message(&MessageUpsert {
        account_id: &account, mailbox_id: &folders[0], uid: 1, uidvalidity: 1,
        message_id: Some("<legacy-publication@example.invalid>"), thread_id: None,
        internaldate: Some("2026-11-20T00:00:00Z"), size: 64,
        flags_json: "[]", envelope: Some(b"{}"), blob_ref: None,
    }).await.unwrap();
    assert!(store.get_message(&id).await.unwrap().blob_ref.is_none());
    let search = Arc::new(mw_search::Index::open_in_ram().unwrap());
    search.upsert(&mw_search::IndexDoc {
        stable_id: id.clone(), account_id: account.clone(), mailbox_id: folders[0].clone(),
        subject: "Publication".into(), body: "uniqueretainedlegacybody".into(), size: 64,
        source_content_ref: None, has_attachment: false, ..Default::default()
    }).unwrap();
    // Setup models a genuine pre-witness stored document; only subsequent
    // operations run through the publication owner under test.
    let options = PublicationOptions { collection_window: Duration::ZERO, ..Default::default() };
    let engine = Engine::with_search_and_publication_options(store, search, options).unwrap();
    Fixture { engine, account, inbox: folders[0].clone(), archive: folders[1].clone(), id }
}

#[tokio::test]
async fn canceled_metadata_and_attachment_tickets_preserve_narrow_legacy_intent() {
    for attachment in [false, true] {
        let f = legacy_metadata_fixture().await;
        let mut ticket = f.engine.reserve_index_publication(&f.account, 1).await.unwrap();
        if attachment {
            ticket.track_attachment_ids(vec![f.id.clone()]).unwrap();
            replace_envelope_without_raw(&f, b"{\"hasAttachment\":true}").await;
        } else {
            ticket.track_metadata_ids(vec![f.id.clone()]).unwrap();
            f.engine.store().set_flags(&f.id, "[\"Seen\"]").await.unwrap();
        }
        let hold = gate(&f.engine, PublicationTestPhase::BeforeExecution);
        drop(ticket); // actual armed Drop handoff, not an explicit submit
        entered(&hold).await;
        hold.release.notify_one(); drained(&f.engine).await;
        let status = f.engine.index_publication_status();
        assert_eq!(status.failed_jobs, 0);
        assert!(!status.poisoned);
        let doc = f.engine.search().fetch_doc(&f.id).unwrap().unwrap();
        assert!(doc.source_content_ref.is_none(), "narrow metadata must not manufacture a raw-body identity");
        assert_eq!(doc.has_attachment, attachment);
        assert_eq!(doc.keywords.iter().any(|value| value == "$seen"), !attachment);
        assert_eq!(f.engine.search().search(&mw_search::parse_query("body:uniqueretainedlegacybody").unwrap(), 100).unwrap(), [f.id]);
    }
}

#[tokio::test]
async fn failed_legacy_metadata_recovers_as_metadata_without_raw_or_fake_witness() {
    let f = legacy_metadata_fixture().await;
    f.engine.store().set_flags(&f.id, "invalid-flags-json").await.unwrap();
    let ticket = f.engine.reserve_index_publication(&f.account, 1).await.unwrap();
    assert!(ticket.submit(vec![f.id.clone()], PublicationMode::Metadata).unwrap().await.is_err());
    drained(&f.engine).await;
    assert_eq!(f.engine.index_publication_status().pending_repair_messages, 1);
    f.engine.store().set_flags(&f.id, "[\"Seen\"]").await.unwrap();
    f.engine.recover_index_publication().await.unwrap();
    drained(&f.engine).await;
    assert!(!f.engine.index_publication_status().poisoned);
    assert_eq!(f.engine.index_publication_status().pending_repair_messages, 0);
    let doc = f.engine.search().fetch_doc(&f.id).unwrap().unwrap();
    assert!(doc.source_content_ref.is_none());
    assert!(doc.keywords.iter().any(|keyword| keyword == "$seen"));
    assert_eq!(f.engine.search().search(&mw_search::parse_query("body:uniqueretainedlegacybody").unwrap(), 100).unwrap(), [f.id]);
}

#[tokio::test]
async fn metadata_content_race_upgrades_dirty_work_and_cannot_recover_as_metadata() {
    let f = legacy_metadata_fixture().await;
    f.engine.store().set_flags(&f.id, "[\"Seen\"]").await.unwrap();
    let hold = gate(&f.engine, PublicationTestPhase::AfterValidation);
    let waiter = f.engine.reserve_index_publication(&f.account, 1).await.unwrap()
        .submit(vec![f.id.clone()], PublicationMode::Metadata).unwrap();
    entered(&hold).await;
    replace_envelope_without_raw(&f, b"{\"subject\":\"Changed source without recoverable body\"}").await;
    hold.release.notify_one();
    assert!(waiter.await.is_err(), "content race must force strict rebuild instead of a stale metadata ack");
    drained(&f.engine).await;
    f.engine.set_publication_test_hook(None);
    assert!(f.engine.recover_index_publication().await.is_err(), "strict dirty obligation cannot downgrade to legacy metadata when raw content is unavailable");
    assert!(f.engine.index_publication_status().poisoned);
    assert_eq!(f.engine.index_publication_status().pending_repair_messages, 1);
    assert!(f.engine.search().fetch_doc(&f.id).unwrap().unwrap().source_content_ref.is_none());
    drained(&f.engine).await;
}

#[tokio::test]
async fn full_rebuild_completion_repairs_residual_deleted_account_dirty_key() {
    let options = PublicationOptions { max_pending_repair_keys: 1, ..Default::default() };
    let f = fixture(options).await;
    f.engine.store().set_flags(&f.id, "invalid-flags-json").await.unwrap();
    let absent = "bounded-missing-second-key".to_string();
    let waiter = f.engine.reserve_index_publication(&f.account, 2).await.unwrap()
        .submit(vec![f.id.clone(), absent], PublicationMode::Reconcile).unwrap();
    assert!(waiter.await.is_err()); drained(&f.engine).await;
    assert!(f.engine.index_publication_status().requires_full_rebuild);
    assert_eq!(f.engine.index_publication_status().pending_repair_messages, 1);
    f.engine.store().dsr_delete_native_account(&f.account).await.unwrap();
    // The authoritative owner is deleted, while cleanup/index state still
    // retains it. Completion must reconcile this residual dirty key before
    // clearing overflow, including removal of its real fulltext document.
    let epoch = f.engine.index_publisher.begin_full_rebuild();
    f.engine.index_publisher.finish_full_rebuild(epoch).await.unwrap();
    drained(&f.engine).await;
    let status = f.engine.index_publication_status();
    assert!(!status.requires_full_rebuild);
    assert!(!status.poisoned);
    assert_eq!(status.pending_repair_messages, 0);
    assert!(f.engine.search().fetch_doc(&f.id).unwrap().is_none());
    assert!(f.engine.search().search(&mw_search::parse_query("body:uniquepublicationbody").unwrap(), 100).unwrap().is_empty());
}

#[tokio::test]
async fn new_failure_after_full_rebuild_begin_prevents_old_epoch_reopening() {
    let f = fixture(Default::default()).await;
    let ticket = f.engine.reserve_index_publication(&f.account, 1).await.unwrap();
    let epoch = f.engine.index_publisher.begin_full_rebuild();
    f.engine.store().set_flags(&f.id, "[\"Seen\"]").await.unwrap();
    // Already admitted work can fail during the scan; that failure must
    // advance the epoch even though ordinary new admission is closed.
    assert!(ticket.submit(vec![f.id.clone()], PublicationMode::Metadata).unwrap().await.is_err());
    drained(&f.engine).await;
    assert!(f.engine.index_publisher.finish_full_rebuild(epoch).await.is_err());
    assert!(f.engine.index_publication_status().poisoned);
    assert_eq!(f.engine.index_publication_status().pending_repair_messages, 1);
    let new_epoch = f.engine.index_publisher.begin_full_rebuild();
    assert_ne!(new_epoch, epoch);
    f.engine.index_publisher.finish_full_rebuild(new_epoch).await.unwrap();
    drained(&f.engine).await;
    assert!(!f.engine.index_publication_status().poisoned);
    assert_eq!(f.engine.index_publication_status().pending_repair_messages, 0);
    assert_eq!(ids(&f.engine, mw_search::Expr::Clause(mw_search::Clause::Keyword("$seen".into()))), [f.id]);
}

#[tokio::test]
async fn mixed_dirty_recovery_keeps_legacy_metadata_separate_from_strict_rebuild() {
    let f = legacy_metadata_fixture().await;
    f.engine.store().set_flags(&f.id, "invalid-flags-json").await.unwrap();
    let legacy = f.engine.reserve_index_publication(&f.account, 1).await.unwrap();
    assert!(legacy.submit(vec![f.id.clone()], PublicationMode::Metadata).unwrap().await.is_err());
    drained(&f.engine).await;

    let raw = b"Message-ID: <strict-mixed@example.invalid>\r\nFrom: sender@example.invalid\r\nSubject: Strict rebuild\r\nContent-Type: text/plain\r\n\r\nuniquemixedstrictbody";
    let blob = f.engine.store().put_body(&f.account, raw).await.unwrap();
    let strict_id = f.engine.store().upsert_message(&MessageUpsert {
        account_id: &f.account, mailbox_id: &f.inbox, uid: 30, uidvalidity: 1,
        message_id: Some("<strict-mixed@example.invalid>"), thread_id: None,
        internaldate: Some("2026-11-20T00:00:00Z"), size: raw.len() as u64,
        flags_json: "invalid-flags-json", envelope: None, blob_ref: Some(&blob),
    }).await.unwrap();
    let strict = f.engine.reserve_index_publication(&f.account, 1).await.unwrap();
    assert!(strict.submit(vec![strict_id.clone()], PublicationMode::Rebuild).unwrap().await.is_err());
    drained(&f.engine).await;
    assert_eq!(f.engine.index_publication_status().pending_repair_messages, 2);
    f.engine.store().set_flags(&f.id, "[\"Seen\"]").await.unwrap();
    f.engine.store().set_flags(&strict_id, "[\"Seen\"]").await.unwrap();
    f.engine.recover_index_publication().await.unwrap();
    drained(&f.engine).await;
    assert!(!f.engine.index_publication_status().poisoned);
    assert_eq!(f.engine.index_publication_status().pending_repair_messages, 0);
    assert!(f.engine.search().fetch_doc(&f.id).unwrap().unwrap().source_content_ref.is_none());
    assert!(f.engine.search().fetch_doc(&strict_id).unwrap().unwrap().source_content_ref.is_some());
    assert_eq!(f.engine.search().search(&mw_search::parse_query("body:uniqueretainedlegacybody").unwrap(), 100).unwrap(), [f.id]);
    assert_eq!(f.engine.search().search(&mw_search::parse_query("body:uniquemixedstrictbody").unwrap(), 100).unwrap(), [strict_id]);
}

#[tokio::test]
async fn same_id_metadata_and_attachment_coalesce_and_recover_both_obligations() {
    let f = legacy_metadata_fixture().await;
    // Empty-ID blocker observes account posture but cannot update the target.
    // Both real same-ID operations are queued before the owner is released.
    let hold = gate(&f.engine, PublicationTestPhase::BeforeExecution);
    let blocker = f.engine.reserve_index_publication(&f.account, 0).await.unwrap()
        .submit(vec![], PublicationMode::Metadata).unwrap();
    entered(&hold).await;
    let before = f.engine.index_publication_status();
    let metadata = f.engine.reserve_index_publication(&f.account, 1).await.unwrap();
    let attachment = f.engine.reserve_index_publication(&f.account, 1).await.unwrap();
    f.engine.store().set_flags(&f.id, "[\"Seen\"]").await.unwrap();
    replace_envelope_without_raw(&f, b"{\"hasAttachment\":true}").await;
    let metadata = metadata.submit(vec![f.id.clone()], PublicationMode::Metadata).unwrap();
    let attachment = attachment.submit(vec![f.id.clone()], PublicationMode::AttachmentMetadata).unwrap();
    assert_eq!(f.engine.index_publication_status().queued_jobs, 2);
    hold.release.notify_one();
    assert!(blocker.await.unwrap().publication.is_none());
    let first = metadata.await.unwrap().publication.unwrap();
    assert_eq!(attachment.await.unwrap().publication.unwrap(), first);
    assert_eq!(first.changes, 1, "one document must incorporate both orthogonal intents");
    drained(&f.engine).await;
    assert_eq!(f.engine.index_publication_status().published_batches - before.published_batches, 1);
    let doc = f.engine.search().fetch_doc(&f.id).unwrap().unwrap();
    assert!(doc.has_attachment && doc.keywords.iter().any(|keyword| keyword == "$seen"));
    assert!(doc.source_content_ref.is_none());
    assert_eq!(f.engine.search().search(&mw_search::parse_query("has:attachment is:read body:uniqueretainedlegacybody").unwrap(), 100).unwrap(), [f.id.clone()]);

    let hold = gate(&f.engine, PublicationTestPhase::BeforeExecution);
    let blocker = f.engine.reserve_index_publication(&f.account, 0).await.unwrap()
        .submit(vec![], PublicationMode::Metadata).unwrap();
    entered(&hold).await;
    let metadata = f.engine.reserve_index_publication(&f.account, 1).await.unwrap();
    let attachment = f.engine.reserve_index_publication(&f.account, 1).await.unwrap();
    f.engine.store().set_flags(&f.id, "invalid-flags-json").await.unwrap();
    replace_envelope_without_raw(&f, b"{\"hasAttachment\":false}").await;
    let metadata = metadata.submit(vec![f.id.clone()], PublicationMode::Metadata).unwrap();
    let attachment = attachment.submit(vec![f.id.clone()], PublicationMode::AttachmentMetadata).unwrap();
    assert_eq!(f.engine.index_publication_status().queued_jobs, 2);
    hold.release.notify_one();
    assert!(blocker.await.unwrap().publication.is_none());
    assert!(metadata.await.is_err());
    assert!(attachment.await.is_err());
    drained(&f.engine).await;
    assert_eq!(f.engine.index_publication_status().pending_repair_messages, 1);
    // Fix the authoritative flags. Recovery must retain BOTH dirty intents;
    // merely selecting the higher ranked Metadata would leave attachment=true.
    f.engine.store().set_flags(&f.id, "[]").await.unwrap();
    f.engine.set_publication_test_hook(None);
    f.engine.recover_index_publication().await.unwrap();
    drained(&f.engine).await;
    let status = f.engine.index_publication_status();
    assert!(!status.poisoned);
    assert_eq!(status.pending_repair_messages, 0);
    let doc = f.engine.search().fetch_doc(&f.id).unwrap().unwrap();
    assert!(!doc.has_attachment);
    assert!(!doc.keywords.iter().any(|keyword| keyword == "$seen"));
    assert!(doc.source_content_ref.is_none());
    assert!(f.engine.search().search(&mw_search::parse_query("has:attachment").unwrap(), 100).unwrap().is_empty());
    assert_eq!(f.engine.search().search(&mw_search::parse_query("is:unread body:uniqueretainedlegacybody").unwrap(), 100).unwrap(), [f.id]);
}

#[tokio::test]
async fn metadata_date_changes_use_zero_for_invalid_and_current_mime_when_cleared() {
    let f = fixture(Default::default()).await;
    let raw = b"Message-ID: <dated-publication@example.invalid>\r\nFrom: sender@example.invalid\r\nDate: Tue, 01 Jan 2030 12:00:00 +0000\r\nSubject: MIME date fallback\r\nContent-Type: text/plain\r\n\r\nuniquedatefallbackbody";
    let blob = f.engine.store().put_body(&f.account, raw).await.unwrap();
    let id = f.engine.store().upsert_message(&MessageUpsert {
        account_id: &f.account, mailbox_id: &f.inbox, uid: 40, uidvalidity: 1,
        message_id: Some("<dated-publication@example.invalid>"), thread_id: None,
        internaldate: Some("2025-01-01T00:00:00Z"), size: raw.len() as u64,
        flags_json: "[]", envelope: None, blob_ref: Some(&blob),
    }).await.unwrap();
    f.engine.publish_current_index(&f.account, vec![id.clone()], PublicationMode::Reconcile).await.unwrap();
    let initial = f.engine.search().fetch_doc(&id).unwrap().unwrap();
    assert_eq!(initial.date, chrono::DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z").unwrap().timestamp());
    assert!(initial.source_content_ref.is_some());
    let query = mw_search::SearchQuery { raw: String::new(), expr: mw_search::Expr::All, sort: mw_search::Sort::received_desc() };
    assert_eq!(f.engine.search().search(&query, 100).unwrap(), [f.id.clone(), id.clone()]);

    for (date, expected) in [
        (Some("invalid-internaldate"), 0),
        (None, chrono::DateTime::parse_from_rfc2822("Tue, 01 Jan 2030 12:00:00 +0000").unwrap().timestamp()),
    ] {
        let same = f.engine.store().upsert_message(&MessageUpsert {
            account_id: &f.account, mailbox_id: &f.inbox, uid: 40, uidvalidity: 1,
            message_id: Some("<dated-publication@example.invalid>"), thread_id: None,
            internaldate: date, size: raw.len() as u64, flags_json: "[]", envelope: None,
            blob_ref: Some(&blob),
        }).await.unwrap();
        assert_eq!(same, id);
        assert_eq!(f.engine.store().get_message(&id).await.unwrap().internaldate.as_deref(), date);
        f.engine.publish_current_index(&f.account, vec![id.clone()], PublicationMode::Metadata).await.unwrap();
        let doc = f.engine.search().fetch_doc(&id).unwrap().unwrap();
        assert_eq!(doc.date, expected);
        assert_eq!(doc.source_content_ref, initial.source_content_ref, "date fallback must retain the same immutable content identity");
        let expected_order = if date.is_some() { vec![f.id.clone(), id.clone()] } else { vec![id.clone(), f.id.clone()] };
        assert_eq!(f.engine.search().search(&query, 100).unwrap(), expected_order);
        assert_eq!(f.engine.search().search(&mw_search::parse_query("body:uniquedatefallbackbody").unwrap(), 100).unwrap(), [id.clone()]);
    }
    drained(&f.engine).await;
}

#[tokio::test]
async fn proven_content_reconcile_keeps_orthogonal_attachment_obligation_through_recovery() {
    let f = fixture(Default::default()).await;
    let raw = b"Message-ID: <orthogonal@example.invalid>\r\nFrom: sender@example.invalid\r\nSubject: Orthogonal attachment\r\nMIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary=proof\r\n\r\n--proof\r\nContent-Type: text/plain\r\n\r\nuniqueorthogonalbody\r\n--proof\r\nContent-Type: text/plain; name=evidence.txt\r\nContent-Disposition: attachment; filename=evidence.txt\r\nContent-Transfer-Encoding: base64\r\n\r\naGVsbG8=\r\n--proof--\r\n";
    let parsed = mw_mime::parse(raw).unwrap();
    assert!(parsed.email.has_attachment, "fixture has an actual MIME attachment");
    let envelope = crate::encode_message_envelope(&parsed.email, raw);
    let blob = f.engine.store().put_body(&f.account, raw).await.unwrap();
    let id = f.engine.store().upsert_message(&MessageUpsert {
        account_id: &f.account, mailbox_id: &f.inbox, uid: 50, uidvalidity: 1,
        message_id: Some("<orthogonal@example.invalid>"), thread_id: None,
        internaldate: Some("2026-11-20T00:00:00Z"), size: raw.len() as u64,
        flags_json: "[]", envelope: Some(&envelope), blob_ref: Some(&blob),
    }).await.unwrap();
    f.engine.publish_current_index(&f.account, vec![id.clone()], PublicationMode::Reconcile).await.unwrap();
    let proven = f.engine.search().fetch_doc(&id).unwrap().unwrap();
    assert!(proven.has_attachment);
    assert!(proven.source_content_ref.is_some());
    let mut old_projection = proven.clone(); old_projection.has_attachment = false;
    f.engine.search().upsert(&old_projection).unwrap();

    for fail_then_recover in [false, true] {
        if fail_then_recover {
            let mut stale = f.engine.search().fetch_doc(&id).unwrap().unwrap();
            stale.has_attachment = false;
            f.engine.search().upsert(&stale).unwrap();
        }
        let hold = gate(&f.engine, PublicationTestPhase::BeforeExecution);
        let blocker = f.engine.reserve_index_publication(&f.account, 0).await.unwrap()
            .submit(vec![], PublicationMode::Metadata).unwrap();
        entered(&hold).await;
        let before = f.engine.index_publication_status();
        let source = f.engine.reserve_index_publication(&f.account, 1).await.unwrap();
        let attachment = f.engine.reserve_index_publication(&f.account, 1).await.unwrap();
        f.engine.store().set_flags(&id, if fail_then_recover { "invalid-flags-json" } else { "[\"Seen\"]" }).await.unwrap();
        let source = source.submit(vec![id.clone()], PublicationMode::Reconcile).unwrap();
        let attachment = attachment.submit(vec![id.clone()], PublicationMode::AttachmentMetadata).unwrap();
        assert_eq!(f.engine.index_publication_status().queued_jobs, 2);
        hold.release.notify_one();
        assert!(blocker.await.unwrap().publication.is_none());
        if fail_then_recover {
            assert!(source.await.is_err()); assert!(attachment.await.is_err());
            drained(&f.engine).await;
            assert_eq!(f.engine.index_publication_status().pending_repair_messages, 1);
            f.engine.store().set_flags(&id, "[]").await.unwrap();
            f.engine.set_publication_test_hook(None);
            f.engine.recover_index_publication().await.unwrap();
        } else {
            let publication = source.await.unwrap().publication.unwrap();
            assert_eq!(attachment.await.unwrap().publication.unwrap(), publication);
            assert_eq!(publication.changes, 1);
            assert_eq!(f.engine.index_publication_status().published_batches - before.published_batches, 1);
        }
        drained(&f.engine).await;
        let doc = f.engine.search().fetch_doc(&id).unwrap().unwrap();
        assert!(doc.has_attachment, "same proven raw body still requires the independent attachment upgrade");
        assert_eq!(doc.source_content_ref, proven.source_content_ref);
        assert_eq!(doc.body, proven.body);
        assert_eq!(doc.filenames, proven.filenames);
        assert_eq!(doc.keywords.iter().any(|keyword| keyword == "$seen"), !fail_then_recover);
        assert_eq!(f.engine.index_publication_status().pending_repair_messages, 0);
        assert!(!f.engine.index_publication_status().poisoned);
        let query = if fail_then_recover { "has:attachment is:unread body:uniqueorthogonalbody" } else { "has:attachment is:read body:uniqueorthogonalbody" };
        assert_eq!(f.engine.search().search(&mw_search::parse_query(query).unwrap(), 100).unwrap(), [id.clone()]);
        assert_eq!(f.engine.store().get_message(&id).await.unwrap().blob_ref.as_deref(), Some(blob.as_str()));
    }
}
