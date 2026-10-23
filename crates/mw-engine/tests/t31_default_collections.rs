//! t31-e5: an account gets one default calendar, one default task list and one
//! default address book, however many requests reach it first at once.
//!
//! `ensure_default_calendar` and `ensure_default_address_book` used to list the
//! account's collections and insert a row with a fresh random id when none was
//! found, with nothing between the list and the insert. Twenty Playwright
//! workers on one account each made the first request of a screen at the same
//! time, and each one that listed before any insert landed created its own
//! `Calendar`, `Tasks` or `Contacts` collection.
//!
//! Every request goes through `Engine::handle_jmap` — the `/jmap/api` envelope
//! the web client's Calendar, Tasks and Contacts screens send — and the counts
//! are read from the store directly, not through the engine code under test.
//! The rounds run on a file-backed store (the server's shape: a five-connection
//! WAL pool) and on the in-memory test store.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};
use tokio::sync::Barrier;

use mw_engine::account::AccountRuntime;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, EngineError, Flag, MailboxDelta, MessageRef,
    MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result, SyncCursor, WatchHandle,
};
use mw_engine::{Engine, MailSubmitter};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

/// Fresh accounts per backing. A round that does not race is not evidence, so
/// the test repeats; before the fix every round on the file store duplicated.
const ROUNDS: usize = 20;
/// Concurrent first requests per kind of collection.
const WORKERS: usize = 20;

// ── harness (the `tests/pim.rs` one: no mail, no DAV) ─────────────────────────

struct NoopBackend;

#[async_trait]
impl AccountBackend for NoopBackend {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps::default())
    }
    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        Ok(Vec::new())
    }
    async fn sync_mailbox(&self, _m: &RawMailboxRef, c: &SyncCursor) -> Result<MailboxDelta> {
        Ok(MailboxDelta {
            added: Vec::new(),
            flag_changes: Vec::new(),
            removed: Vec::new(),
            next_cursor: c.clone(),
        })
    }
    async fn fetch_raw(&self, _refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
        Ok(Vec::new())
    }
    async fn store_flags(&self, _r: &[MessageRef], _a: &[Flag], _d: &[Flag]) -> Result<()> {
        Ok(())
    }
    async fn move_messages(&self, _r: &[MessageRef], _to: &RawMailboxRef) -> Result<MoveOutcome> {
        Err(EngineError::Unsupported("noop".into()))
    }
    async fn append(&self, _m: &RawMailboxRef, _raw: &[u8], _f: &[Flag]) -> Result<MessageRef> {
        Err(EngineError::Unsupported("noop".into()))
    }
    async fn watch(&self, _sink: ChangeSink) -> Result<WatchHandle> {
        Err(EngineError::Unsupported("noop".into()))
    }
}

struct NoopSubmitter;

#[async_trait]
impl MailSubmitter for NoopSubmitter {
    async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult> {
        Ok(SubmissionResult {
            accepted: msg.rcpt_to,
            rejected: Vec::new(),
        })
    }
}

#[derive(Clone, Copy, Debug)]
enum Backing {
    File,
    Memory,
}

/// A scratch database file that is removed (with its WAL files) on drop.
struct ScratchDb(std::path::PathBuf);

impl Drop for ScratchDb {
    fn drop(&mut self) {
        for suffix in ["", "-wal", "-shm"] {
            let mut p = self.0.clone().into_os_string();
            p.push(suffix);
            let _ = std::fs::remove_file(p);
        }
    }
}

struct Harness {
    engine: Arc<Engine>,
    account_id: String,
    _db: Option<ScratchDb>,
}

async fn setup(backing: Backing, round: usize) -> Harness {
    let (store, db) = match backing {
        Backing::Memory => (
            Store::open_in_memory(ServerKey::generate()).await.unwrap(),
            None,
        ),
        Backing::File => {
            let path = std::env::temp_dir().join(format!(
                "mw-t31-default-collections-{}-{round}.db",
                std::process::id()
            ));
            let db = ScratchDb(path.clone());
            drop(ScratchDb(path.clone())); // a leftover from an aborted run
            let store = Store::open(path.to_str().unwrap(), ServerKey::generate())
                .await
                .unwrap();
            (store, Some(db))
        }
    };
    let account_id = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example.org",
                port: 993,
                tls: "implicit",
                username: "me@example.org",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "me@example.org".into(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap();
    let engine = Arc::new(Engine::new(store));
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            Arc::new(NoopBackend) as Arc<dyn AccountBackend>,
            Arc::new(NoopSubmitter) as Arc<dyn MailSubmitter>,
            "me@example.org",
        ),
    );
    Harness {
        engine,
        account_id,
        _db: db,
    }
}

/// One first request of a screen, as the web client sends it.
fn request(kind: usize, i: usize) -> (&'static str, Value) {
    let even = i % 2 == 0;
    match (kind, even) {
        // Calendar screen: list the calendars, or create an event with no calendar.
        (0, true) => ("Calendar/get", json!({ "ids": null })),
        (0, false) => (
            "CalendarEvent/set",
            json!({ "create": { "e": {
                "title": format!("event {i}"),
                "start": "2026-07-13T09:00:00",
                "timeZone": "UTC",
                "duration": "PT30M",
            }}}),
        ),
        // Tasks screen: list the lists, or create a task with no list.
        (1, true) => ("Calendar/get", json!({ "ids": null })),
        (1, false) => (
            "Task/set",
            json!({ "create": { "t": { "title": format!("task {i}") } } }),
        ),
        // Contacts screen: list the address books, or create a card with no book.
        (_, true) => ("AddressBook/get", json!({ "ids": null })),
        (_, false) => (
            "ContactCard/set",
            json!({ "create": { "c": {
                "kind": "individual",
                "name": { "full": format!("Person {i}") },
            }}}),
        ),
    }
}

/// What one round left in the store.
#[derive(Debug, Default)]
struct Counts {
    event_calendars: usize,
    task_lists: usize,
    address_books: usize,
    /// Calendars / lists / books that hold none of the round's items.
    empty_collections: usize,
}

impl Counts {
    fn one_of_each(&self) -> bool {
        self.event_calendars == 1 && self.task_lists == 1 && self.address_books == 1
    }
}

/// Fire `WORKERS` first requests per kind at a fresh account at once, check
/// every one succeeded and every created item was stored, and count what the
/// store holds.
async fn round(backing: Backing, n: usize) -> Counts {
    let h = setup(backing, n).await;
    let store = h.engine.store();

    // Precondition: the account starts with no collection of any kind.
    assert!(store.list_calendars(&h.account_id).await.unwrap().is_empty());
    assert!(
        store
            .list_address_books(&h.account_id)
            .await
            .unwrap()
            .is_empty()
    );

    let barrier = Arc::new(Barrier::new(3 * WORKERS));
    let mut joins = Vec::new();
    for kind in 0..3 {
        for i in 0..WORKERS {
            let engine = Arc::clone(&h.engine);
            let account_id = h.account_id.clone();
            let barrier = Arc::clone(&barrier);
            joins.push(tokio::spawn(async move {
                let (method, args) = request(kind, i);
                barrier.wait().await;
                let req = json!({ "methodCalls": [[method, args, "c0"]] });
                let resp = engine.handle_jmap(&account_id, &req).await;
                (method, resp["methodResponses"][0][1].clone())
            }));
        }
    }
    for j in joins {
        let (method, resp) = j.await.unwrap();
        assert!(
            resp.get("type").is_none(),
            "{backing:?} round {n}: {method} failed: {resp}"
        );
        if method.ends_with("/set") {
            assert!(
                resp["created"].as_object().is_some_and(|c| c.len() == 1),
                "{backing:?} round {n}: {method} created nothing: {resp}"
            );
        }
    }

    let creators = WORKERS / 2;
    let mut counts = Counts::default();
    let (mut events, mut tasks, mut contacts) = (0, 0, 0);
    for cal in store.list_calendars(&h.account_id).await.unwrap() {
        let held = match cal.component.as_str() {
            "VEVENT" => {
                counts.event_calendars += 1;
                let k = store.list_events(&cal.id).await.unwrap().len();
                events += k;
                k
            }
            "VTODO" => {
                counts.task_lists += 1;
                let k = store.list_tasks(&cal.id).await.unwrap().len();
                tasks += k;
                k
            }
            other => panic!("unexpected component {other}"),
        };
        if held == 0 {
            counts.empty_collections += 1;
        }
    }
    for book in store.list_address_books(&h.account_id).await.unwrap() {
        counts.address_books += 1;
        let k = store.list_contacts(&book.id).await.unwrap().len();
        contacts += k;
        if k == 0 {
            counts.empty_collections += 1;
        }
    }
    assert_eq!(
        (events, tasks, contacts),
        (creators, creators, creators),
        "{backing:?} round {n}: not every created item was stored"
    );
    counts
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn concurrent_first_requests_create_one_default_collection_of_each_kind() {
    let mut report = Vec::new();
    let mut duplicated = 0;
    for backing in [Backing::File, Backing::Memory] {
        for n in 0..ROUNDS {
            let counts = round(backing, n).await;
            if !counts.one_of_each() {
                duplicated += 1;
                report.push(format!(
                    "{backing:?} round {n}: {} event calendars, {} task lists, {} address \
                     books ({} of them empty)",
                    counts.event_calendars,
                    counts.task_lists,
                    counts.address_books,
                    counts.empty_collections
                ));
            }
        }
    }
    assert!(
        duplicated == 0,
        "{duplicated} of {} rounds left more than one default collection of a kind:\n{}",
        2 * ROUNDS,
        report.join("\n")
    );
}

/// The default is re-created after the user destroys it, under a new id: the
/// fix does not hand out an id a client has already seen destroyed.
#[tokio::test]
async fn a_destroyed_default_calendar_is_replaced_under_a_new_id() {
    let h = setup(Backing::Memory, 0).await;
    let call = |method: &'static str, args: Value| {
        let engine = Arc::clone(&h.engine);
        let account_id = h.account_id.clone();
        async move {
            let req = json!({ "methodCalls": [[method, args, "c0"]] });
            engine.handle_jmap(&account_id, &req).await["methodResponses"][0][1].clone()
        }
    };
    let event_calendar = |got: &Value| -> Vec<String> {
        got["list"]
            .as_array()
            .unwrap_or_else(|| panic!("Calendar/get has no list: {got}"))
            .iter()
            .filter(|c| c["name"] == "Calendar")
            .map(|c| c["id"].as_str().unwrap().to_string())
            .collect()
    };

    let first = event_calendar(&call("Calendar/get", json!({ "ids": null })).await);
    assert_eq!(first.len(), 1, "seeded: {first:?}");
    let gone = call("Calendar/set", json!({ "destroy": [first[0]] })).await;
    assert_eq!(gone["destroyed"], json!([first[0]]), "{gone}");

    let second = event_calendar(&call("Calendar/get", json!({ "ids": null })).await);
    assert_eq!(second.len(), 1, "re-seeded: {second:?}");
    assert_ne!(second[0], first[0], "the destroyed id came back");
}
