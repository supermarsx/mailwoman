//! t50hp-e26 (S3-a): the unread-count shortcut answers exactly what the
//! generic index path answers, member for member, over generated folders and
//! after every kind of row write, with the index drained before each compare.
use super::*;
use crate::account::MailSubmitter;
use crate::backend::*;
use crate::unread_count_shortcut::GENERIC_ONLY;
use async_trait::async_trait;
use mw_store::{AccountKind, Credentials, MailboxUpsert, NewAccount, ServerKey, Store};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

#[derive(Default)]
struct Remote {
    next_uid: AtomicU32,
}
#[async_trait]
impl AccountBackend for Remote {
    async fn capabilities(&self) -> Result<BackendCaps> { Ok(BackendCaps::default()) }
    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> { Ok(vec![]) }
    async fn sync_mailbox(&self, _: &RawMailboxRef, c: &SyncCursor) -> Result<MailboxDelta> {
        Ok(MailboxDelta { added: vec![], flag_changes: vec![], removed: vec![], next_cursor: c.clone() })
    }
    async fn fetch_raw(&self, _: &[MessageRef]) -> Result<Vec<RawMessage>> { Err(EngineError::Unsupported("fixture".into())) }
    async fn store_flags(&self, _: &[MessageRef], _: &[Flag], _: &[Flag]) -> Result<()> { Ok(()) }
    async fn move_messages(&self, refs: &[MessageRef], _: &RawMailboxRef) -> Result<MoveOutcome> {
        let uids = refs.iter().map(|_| self.next_uid.fetch_add(1, Ordering::SeqCst)).collect();
        Ok(MoveOutcome::Uidplus { uidvalidity: 1, uids })
    }
    async fn expunge_messages(&self, _: &[MessageRef]) -> Result<()> { Ok(()) }
    async fn append(&self, _: &RawMailboxRef, _: &[u8], _: &[Flag]) -> Result<MessageRef> { Err(EngineError::Unsupported("fixture".into())) }
    async fn watch(&self, _: ChangeSink) -> Result<WatchHandle> { Err(EngineError::Unsupported("fixture".into())) }
}
struct NoSend;
#[async_trait]
impl MailSubmitter for NoSend {
    async fn submit(&self, _: mw_smtp::Outgoing) -> Result<mw_smtp::SubmissionResult> { panic!("not sending") }
}

/// A JSON object from computed keys.
fn obj(pairs: Vec<(String, Value)>) -> Value {
    Value::Object(pairs.into_iter().collect())
}
fn boxes(id: &str) -> Value {
    obj(vec![(id.to_owned(), json!(true))])
}

struct World {
    engine: Engine,
    account: String,
    /// INBOX, Archive, Trash, Empty.
    boxes: Vec<(String, String)>,
}

async fn world(store: Store) -> World {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let login = format!("u26-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::SeqCst) as u64 + chrono::Utc::now().timestamp_millis() as u64);
    let account = store
        .create_account(
            &NewAccount { kind: AccountKind::Imap, host: "fixture.invalid", port: 993, tls: "implicit", username: &login, sync_policy_json: "{}" },
            &Credentials { username: login.clone(), password: "x".into() },
        )
        .await
        .unwrap();
    let mut boxes = Vec::new();
    for (name, role) in [("INBOX", Some("inbox")), ("Archive", Some("archive")), ("Trash", Some("trash")), ("Empty", None)] {
        let id = store
            .upsert_mailbox(&MailboxUpsert { account_id: &account, name, role, uidvalidity: 1, uidnext: 1, highestmodseq: 0, total: 0, unread: 0, parent_id: None })
            .await
            .unwrap();
        boxes.push((id, name.to_owned()));
    }
    let engine = Engine::new(store);
    engine.register_backend(account.clone(), AccountRuntime::new(Arc::new(Remote { next_uid: AtomicU32::new(10_000) }), Arc::new(NoSend), "o@fixture.invalid"));
    World { engine, account, boxes }
}

impl World {
    async fn ingest(&self, mailbox: usize, uid: u32, second: u32, seen: bool) {
        let (id, name) = &self.boxes[mailbox];
        let raw = format!(
            "From: s{uid}@fixture.invalid\r\nTo: o@fixture.invalid\r\nSubject: m {uid}\r\nMessage-ID: <{uid}.{name}@fixture.invalid>\r\nContent-Type: text/plain\r\n\r\nbody {uid}\r\n"
        );
        let message = RawMessage {
            message_ref: MessageRef::Imap { mailbox: RawMailboxRef { name: name.clone(), uidvalidity: 1 }, uidvalidity: 1, uid },
            raw: raw.into_bytes(),
            flags: if seen { vec![Flag::Seen] } else { vec![] },
            internaldate: Some(format!("2026-07-01T00:{:02}:{:02}Z", second / 60, second % 60)),
        };
        self.engine.ingest(&self.account, id, &message).await.unwrap();
    }

    async fn send(&self, calls: Value) -> Value {
        self.engine
            .handle_jmap(&self.account, &json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":calls}))
            .await
    }

    fn count_call(&self, mailbox: usize, limit: u64) -> Value {
        json!(["Email/query", {"accountId": self.account, "filter": {"inMailbox": self.boxes[mailbox].0, "notKeyword": "$seen"}, "limit": limit, "calculateTotal": true}, "unread"])
    }

    /// The generic path's answer to the same call, same state.
    async fn generic(&self, call: Value) -> Value {
        GENERIC_ONLY.with(|g| g.set(true));
        let out = self.send(json!([call])).await;
        GENERIC_ONLY.with(|g| g.set(false));
        out["methodResponses"][0].clone()
    }

    /// Unread rows by brute force, and the stored counter.
    async fn unread(&self, mailbox: usize) -> (u64, u64) {
        let id = &self.boxes[mailbox].0;
        let ids = self.engine.store().list_message_ids(id, i64::MAX, 0).await.unwrap();
        let rows = self.engine.store().get_messages(&ids).await.unwrap().into_iter().flatten();
        let brute = rows.filter(|m| !crate::mapping::flags_from_json(&m.flags_json).contains(&Flag::Seen)).count() as u64;
        (brute, u64::from(self.engine.store().get_mailbox(id).await.unwrap().unread))
    }

    /// Commit mode (the default): every write above returned after its index
    /// publication committed, so the index is current here. `check` proves
    /// it per folder (the generic total equals the row count); `drain` would
    /// close admission.
    async fn drain(&self) {}

    /// For every folder and both limits: counter == brute force, shortcut ==
    /// generic for every member, and the generic total is the brute force
    /// (so the index really holds the folder). Returns compares made.
    async fn check(&self, context: &str) -> usize {
        self.drain().await;
        let mut compares = 0;
        for mailbox in 0..self.boxes.len() {
            let (brute, counter) = self.unread(mailbox).await;
            assert_eq!(counter, brute, "{context}: stored counter of {}", self.boxes[mailbox].1);
            for limit in [0, 1] {
                let call = self.count_call(mailbox, limit);
                let before = self.engine.unread_count_shortcut_status();
                let fast = self.send(json!([call.clone()])).await["methodResponses"][0].clone();
                let after = self.engine.unread_count_shortcut_status();
                assert!(after.taken_limit0 + after.taken_limit1 > before.taken_limit0 + before.taken_limit1, "{context}: shortcut taken");
                let slow = self.generic(call).await;
                assert_eq!(fast, slow, "{context}: {} limit {limit}", self.boxes[mailbox].1);
                assert_eq!(slow[1]["total"], json!(brute), "{context}: generic total is the row count");
                let expect_ids = if limit == 1 && brute > 0 { 1 } else { 0 };
                assert_eq!(fast[1]["ids"].as_array().map(Vec::len), Some(expect_ids), "{context}: {fast}");
                compares += 1;
            }
        }
        compares
    }

    async fn ids(&self, mailbox: usize) -> Vec<String> {
        self.engine.store().list_message_ids(&self.boxes[mailbox].0, i64::MAX, 0).await.unwrap()
    }
}

async fn differential(store: Store) -> usize {
    let w = world(store).await;
    let mut compares = w.check("empty folders").await;
    // INBOX: many, mixed, with equal-second ties (three per second); Archive:
    // one unread; Trash: one read; Empty: none.
    for uid in 1..=45u32 {
        w.ingest(0, uid, uid / 3, uid % 4 == 0).await;
    }
    w.ingest(1, 1, 7, false).await;
    w.ingest(2, 1, 9, true).await;
    compares += w.check("after ingest").await;

    let inbox = w.ids(0).await;
    // Mark read and unread through Email/set.
    let update = obj(vec![
        (inbox[0].clone(), json!({"keywords/$seen": true})),
        (inbox[1].clone(), json!({"keywords/$seen": true})),
        (inbox[3].clone(), json!({"keywords/$seen": null})),
    ]);
    let r = w.send(json!([["Email/set", {"accountId": w.account, "update": update}, "s"]])).await;
    assert!(r["methodResponses"][0][1]["notUpdated"].is_null(), "{r}");
    compares += w.check("after mark read/unread").await;

    // Move two unread and one read out of INBOX, and the Archive one in.
    let archive = w.ids(1).await;
    let (a, t) = (w.boxes[1].0.clone(), w.boxes[2].0.clone());
    let update = obj(vec![
        (inbox[5].clone(), json!({"mailboxIds": boxes(&a)})),
        (inbox[6].clone(), json!({"mailboxIds": boxes(&t)})),
        (inbox[7].clone(), json!({"mailboxIds": boxes(&a)})),
        (archive[0].clone(), json!({"mailboxIds": boxes(&w.boxes[0].0)})),
    ]);
    let r = w.send(json!([["Email/set", {"accountId": w.account, "update": update}, "m"]])).await;
    assert!(r["methodResponses"][0][1]["notUpdated"].is_null(), "{r}");
    compares += w.check("after moves in and out").await;

    // Delete an unread and a read message.
    let r = w.send(json!([["Email/set", {"accountId": w.account, "destroy": [inbox[9].clone(), inbox[0].clone()]}, "d"]])).await;
    assert!(r["methodResponses"][0][1]["notDestroyed"].is_null(), "{r}");
    compares += w.check("after delete").await;

    // A flag change earlier in the SAME batch, then the count query: the
    // shortcut's answer in that batch equals the generic answer right after.
    let inbox = w.ids(0).await;
    for limit in [0u64, 1] {
        let r = w.send(json!([
            ["Email/set", {"accountId": w.account, "update": obj(vec![(inbox[2].clone(), json!({"keywords/$seen": limit == 0}))])}, "s"],
            w.count_call(0, limit)
        ])).await;
        let in_batch = r["methodResponses"][1].clone();
        w.drain().await;
        assert_eq!(in_batch, w.generic(w.count_call(0, limit)).await, "same-batch flag change, limit {limit}");
        assert_eq!(in_batch[1]["total"], json!(w.unread(0).await.0));
        compares += 1;
    }
    compares += w.check("after same-batch writes").await;

    // A folder of another account is answered exactly as today (empty, 0).
    let other = world(w.engine.store().clone()).await;
    let foreign = json!(["Email/query", {"accountId": w.account, "filter": {"inMailbox": other.boxes[0].0, "notKeyword": "$seen"}, "limit": 1, "calculateTotal": true}, "unread"]);
    let fast = w.send(json!([foreign.clone()])).await["methodResponses"][0].clone();
    assert_eq!(fast, w.generic(foreign).await);
    assert_eq!(fast[1]["total"], json!(0));
    assert_eq!(fast[1]["ids"], json!([]));
    compares + 1
}

#[tokio::test]
async fn unread_count_shortcut_equals_generic_path_sqlite() {
    let compares = differential(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
    eprintln!("t50hp-e26 differential sqlite: {compares} response compares");
}

#[tokio::test]
async fn unread_count_shortcut_equals_generic_path_postgres() {
    let Some(dsn) = mw_test_gate::pg_dsn() else {
        mw_test_gate::skip("DATABASE_URL_PG and MW_E14_PG_DSN unset: unread-count shortcut differential on PostgreSQL");
        return;
    };
    let compares = differential(Store::open(&dsn, ServerKey::generate()).await.unwrap()).await;
    eprintln!("t50hp-e26 differential postgres: {compares} response compares");
}

/// Shapes the shortcut must not take, each counted under its reason; the
/// generic answer is returned for them.
#[tokio::test]
async fn other_shapes_keep_the_generic_path() {
    let w = world(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
    for uid in 1..=5u32 {
        w.ingest(0, uid, uid, false).await;
    }
    w.drain().await;
    let inbox = w.boxes[0].0.clone();
    let filter = json!({"inMailbox": inbox, "notKeyword": "$seen"});
    for (args, reason) in [
        (json!({"filter": filter, "limit": 1, "calculateTotal": true, "sort": [{"property": "size"}]}), "sort"),
        (json!({"filter": filter, "limit": 1, "position": 1, "calculateTotal": true}), "window"),
        (json!({"filter": filter, "limit": 5, "calculateTotal": true}), "limit"),
        (json!({"filter": filter, "limit": 1}), "noTotal"),
    ] {
        let before = w.engine.unread_count_shortcut_status();
        let mut a = args.clone();
        a["accountId"] = json!(w.account);
        let fast = w.send(json!([["Email/query", a.clone(), "q"]])).await["methodResponses"][0].clone();
        let after = w.engine.unread_count_shortcut_status();
        // Counters are process-wide and only grow: another test can add to
        // them, never take away.
        let reason_count = |s: &crate::UnreadCountShortcutStatus| match reason {
            "sort" => s.fallback_sort,
            "window" => s.fallback_window,
            "limit" => s.fallback_limit,
            _ => s.fallback_no_total,
        };
        assert!(reason_count(&after) > reason_count(&before), "{reason}");
        assert_eq!(fast, w.generic(json!(["Email/query", a, "q"])).await, "{reason}");
    }
    // Events view and extra conditions are not the shape: generic answers.
    let other = w.boxes[2].0.clone();
    for f in [
        json!({"inMailbox": crate::query::EVENTS_VIEW_ID, "notKeyword": "$seen"}),
        json!({"inMailbox": inbox, "notKeyword": "$seen", "hasKeyword": "$flagged"}),
        json!({"inMailbox": inbox, "notKeyword": "$seen", "inMailboxOtherThan": [other]}),
    ] {
        let call = json!(["Email/query", {"accountId": w.account, "filter": f, "limit": 1, "calculateTotal": true}, "q"]);
        let fast = w.send(json!([call.clone()])).await["methodResponses"][0].clone();
        assert_eq!(fast, w.generic(call).await);
    }
}

