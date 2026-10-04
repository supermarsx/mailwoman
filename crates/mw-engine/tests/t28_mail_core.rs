//! 26.20 t28-e1 — mail-core conformance, driven through `Engine::handle_jmap`.
//!
//! Five behaviours, each a case where the engine answered a request with a
//! success and did something else:
//!
//! - `Email/set` update read only a whole `keywords` object, so the patch-path
//!   form `keywords/<kw>` — the only form the web, the offline outbox and the
//!   Sieve action sink send — wrote nothing and was reported under `updated`;
//! - `inReplyTo` was read as a bare string only, so the RFC 8621 `String[]`
//!   form produced a reply with no `In-Reply-To`, in a thread of its own;
//! - a `sendAt` that was not a time, or was in the past, counted as "not a
//!   future send" and the message went out at once;
//! - `Email/query` evaluated the keyword, date and size conditions only on the
//!   index route, and did not route there for them: the whole mailbox came back;
//! - `has:attachments` (and every spelling but `has:attachment`) asked the index
//!   for the messages WITHOUT an attachment.
//!
//! Every test states its precondition first, so it cannot pass against a
//! fixture that already looked like the outcome, and every refusal is paired
//! with the accepted form beside it.
//!
//! The backend and the submitter record what reaches them: an upstream flag
//! write and a transmitted message are the two effects these cases are about,
//! and neither is visible in a JMAP response.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::{Value, json};

use mw_engine::account::AccountRuntime;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, Flag, MailboxDelta, MailboxRole, MessageRef,
    MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result, SyncCursor, WatchHandle,
};
use mw_engine::{Engine, MailSubmitter};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

const UIDVALIDITY: u32 = 100;

// ---------------------------------------------------------------------------
// Backend and submitter
// ---------------------------------------------------------------------------

type ScriptMsg = (u32, Vec<u8>, Vec<Flag>, String);

/// One upstream `STORE`: the uids it named, the flags added, the flags removed.
type FlagCall = (Vec<u32>, Vec<Flag>, Vec<Flag>);

struct RecordingBackend {
    messages: Mutex<HashMap<String, Vec<ScriptMsg>>>,
    flag_calls: Mutex<Vec<FlagCall>>,
}

/// The inbox every test starts from. Three messages that differ in exactly the
/// ways the query conditions select on:
///
/// | uid | Message-ID | day        | flags      | attachment | size   |
/// |-----|------------|------------|------------|------------|--------|
/// | 1   | `seed-0@x` | 2026-07-01 | none       | no         | small  |
/// | 2   | `seed-1@x` | 2026-07-02 | `\Flagged` | no         | small  |
/// | 3   | `seed-2@x` | 2026-07-03 | none       | PDF        | > 4 kB |
impl RecordingBackend {
    fn seeded() -> Self {
        let inbox: Vec<ScriptMsg> = vec![
            (
                1,
                plain_msg(0, 1),
                Vec::new(),
                "2026-07-01T09:00:00Z".into(),
            ),
            (
                2,
                plain_msg(1, 2),
                vec![Flag::Flagged],
                "2026-07-02T09:00:00Z".into(),
            ),
            (
                3,
                attachment_msg(2, 3),
                Vec::new(),
                "2026-07-03T09:00:00Z".into(),
            ),
        ];
        let mut messages = HashMap::new();
        messages.insert("INBOX".to_string(), inbox);
        messages.insert("Archive".to_string(), Vec::new());
        Self {
            messages: Mutex::new(messages),
            flag_calls: Mutex::new(Vec::new()),
        }
    }

    fn flag_calls(&self) -> Vec<FlagCall> {
        self.flag_calls.lock().unwrap().clone()
    }
}

fn plain_msg(i: usize, day: u32) -> Vec<u8> {
    format!(
        "Message-ID: <seed-{i}@x>\r\n\
         From: alice@example.org\r\n\
         To: me@example.org\r\n\
         Subject: Seeded {i}\r\n\
         Date: {day:02} Jul 2026 09:00:00 +0000\r\n\
         \r\n\
         body of seeded message {i}\r\n"
    )
    .into_bytes()
}

/// A multipart/mixed message with one non-inline PDF part, padded past 4 kB.
fn attachment_msg(i: usize, day: u32) -> Vec<u8> {
    let padding = "0123456789abcdef".repeat(300);
    format!(
        "Message-ID: <seed-{i}@x>\r\n\
         From: alice@example.org\r\n\
         To: me@example.org\r\n\
         Subject: Seeded {i}\r\n\
         Date: {day:02} Jul 2026 09:00:00 +0000\r\n\
         MIME-Version: 1.0\r\n\
         Content-Type: multipart/mixed; boundary=\"m0\"\r\n\
         \r\n\
         --m0\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\
         \r\n\
         see attached\r\n\
         --m0\r\n\
         Content-Type: application/pdf; name=\"report.pdf\"\r\n\
         Content-Disposition: attachment; filename=\"report.pdf\"\r\n\
         \r\n\
         %PDF-1.4 {padding}\r\n\
         --m0--\r\n"
    )
    .into_bytes()
}

#[async_trait]
impl AccountBackend for RecordingBackend {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps {
            uidplus: true,
            r#move: true,
            special_use: true,
            ..BackendCaps::default()
        })
    }

    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        let st = self.messages.lock().unwrap();
        Ok([
            ("INBOX", MailboxRole::Inbox),
            ("Archive", MailboxRole::Archive),
        ]
        .into_iter()
        .map(|(name, role)| {
            let total = st.get(name).map(|m| m.len()).unwrap_or(0) as u32;
            RawMailbox {
                mailbox_ref: RawMailboxRef {
                    name: name.to_string(),
                    uidvalidity: UIDVALIDITY,
                },
                role,
                parent: None,
                uidnext: total + 1,
                highestmodseq: 0,
                total,
                unread: total,
            }
        })
        .collect())
    }

    async fn sync_mailbox(
        &self,
        mbox: &RawMailboxRef,
        cursor: &SyncCursor,
    ) -> Result<MailboxDelta> {
        let uidnext_from = match cursor {
            SyncCursor::UidWindow { uidnext, .. } => *uidnext,
            _ => 1,
        };
        let st = self.messages.lock().unwrap();
        let msgs = st.get(&mbox.name).cloned().unwrap_or_default();
        let added: Vec<MessageRef> = msgs
            .iter()
            .filter(|(uid, ..)| *uid >= uidnext_from)
            .map(|(uid, ..)| MessageRef::Imap {
                mailbox: mbox.clone(),
                uidvalidity: UIDVALIDITY,
                uid: *uid,
            })
            .collect();
        let max_uid = msgs.iter().map(|(u, ..)| *u).max().unwrap_or(0);
        Ok(MailboxDelta {
            added,
            flag_changes: Vec::new(),
            removed: Vec::new(),
            next_cursor: SyncCursor::UidWindow {
                uidvalidity: UIDVALIDITY,
                uidnext: max_uid + 1,
            },
        })
    }

    async fn fetch_raw(&self, refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
        let st = self.messages.lock().unwrap();
        let mut out = Vec::new();
        for r in refs {
            let MessageRef::Imap { mailbox, uid, .. } = r else {
                continue;
            };
            if let Some(msgs) = st.get(&mailbox.name)
                && let Some((_, raw, flags, internaldate)) = msgs.iter().find(|(u, ..)| u == uid)
            {
                out.push(RawMessage {
                    message_ref: r.clone(),
                    raw: raw.clone(),
                    flags: flags.clone(),
                    internaldate: Some(internaldate.clone()),
                });
            }
        }
        Ok(out)
    }

    async fn store_flags(&self, refs: &[MessageRef], add: &[Flag], remove: &[Flag]) -> Result<()> {
        let uids = refs
            .iter()
            .filter_map(|r| match r {
                MessageRef::Imap { uid, .. } => Some(*uid),
                _ => None,
            })
            .collect();
        self.flag_calls
            .lock()
            .unwrap()
            .push((uids, add.to_vec(), remove.to_vec()));
        Ok(())
    }

    async fn move_messages(&self, _r: &[MessageRef], _to: &RawMailboxRef) -> Result<MoveOutcome> {
        Ok(MoveOutcome::Uidplus {
            uidvalidity: UIDVALIDITY,
            uids: vec![9001],
        })
    }

    async fn append(&self, mbox: &RawMailboxRef, raw: &[u8], flags: &[Flag]) -> Result<MessageRef> {
        let mut st = self.messages.lock().unwrap();
        let entry = st.entry(mbox.name.clone()).or_default();
        let uid = entry.iter().map(|(u, ..)| *u).max().unwrap_or(0) + 1;
        entry.push((
            uid,
            raw.to_vec(),
            flags.to_vec(),
            "2026-07-10T09:00:00Z".into(),
        ));
        Ok(MessageRef::Imap {
            mailbox: mbox.clone(),
            uidvalidity: UIDVALIDITY,
            uid,
        })
    }

    async fn watch(&self, _sink: ChangeSink) -> Result<WatchHandle> {
        let (tx, _rx) = tokio::sync::watch::channel(false);
        Ok(WatchHandle::new(tx))
    }
}

/// Accepts everything and keeps the bytes it was handed.
struct RecordingSubmitter {
    sent: Mutex<Vec<Vec<u8>>>,
}

impl RecordingSubmitter {
    fn sent(&self) -> Vec<String> {
        self.sent
            .lock()
            .unwrap()
            .iter()
            .map(|raw| String::from_utf8_lossy(raw).into_owned())
            .collect()
    }
}

#[async_trait]
impl MailSubmitter for RecordingSubmitter {
    async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult> {
        self.sent.lock().unwrap().push(msg.raw);
        Ok(SubmissionResult {
            accepted: msg.rcpt_to,
            rejected: Vec::new(),
        })
    }
}

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

struct Harness {
    engine: Arc<Engine>,
    account_id: String,
    backend: Arc<RecordingBackend>,
    submitter: Arc<RecordingSubmitter>,
}

async fn setup() -> Harness {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
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
    let backend = Arc::new(RecordingBackend::seeded());
    let submitter = Arc::new(RecordingSubmitter {
        sent: Mutex::new(Vec::new()),
    });
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            backend.clone() as Arc<dyn AccountBackend>,
            submitter.clone() as Arc<dyn MailSubmitter>,
            "me@example.org",
        ),
    );
    let h = Harness {
        engine,
        account_id,
        backend,
        submitter,
    };
    h.engine.resync(&h.account_id).await.unwrap();
    h
}

async fn jmap(h: &Harness, calls: Value) -> Value {
    h.engine
        .handle_jmap(&h.account_id, &json!({ "methodCalls": calls }))
        .await
}

fn result<'a>(resp: &'a Value, call_id: &str) -> &'a Value {
    resp["methodResponses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r[2] == call_id)
        .map(|r| &r[1])
        .unwrap_or(&Value::Null)
}

async fn inbox(h: &Harness) -> String {
    let mb = jmap(h, json!([["Mailbox/get", {}, "mb"]])).await;
    result(&mb, "mb")["list"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["role"] == "inbox")
        .expect("inbox")["id"]
        .as_str()
        .unwrap()
        .to_string()
}

/// `Email/query` with `filter`, ids sorted for set comparison.
async fn query(h: &Harness, filter: Value) -> Vec<String> {
    let q = jmap(
        h,
        json!([["Email/query", { "filter": filter, "limit": 1000 }, "q"]]),
    )
    .await;
    let mut ids: Vec<String> = result(&q, "q")["ids"]
        .as_array()
        .unwrap_or_else(|| panic!("Email/query returned no ids: {q}"))
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    ids.sort();
    ids
}

async fn email(h: &Harness, id: &str) -> Value {
    let g = jmap(h, json!([["Email/get", { "ids": [id] }, "g"]])).await;
    let list = result(&g, "g")["list"].as_array().cloned().unwrap();
    assert_eq!(list.len(), 1, "Email/get for {id}: {g}");
    list[0].clone()
}

/// The stable ids of the three seeded messages, in seed order, found by their
/// subject so no test depends on the order a query happens to return.
async fn seeded_ids(h: &Harness, inbox: &str) -> [String; 3] {
    let all = query(h, json!({ "inMailbox": inbox })).await;
    assert_eq!(all.len(), 3, "the fixture is three messages");
    let mut by_seed: [Option<String>; 3] = [None, None, None];
    for id in all {
        let e = email(h, &id).await;
        let subject = e["subject"].as_str().unwrap_or_default().to_string();
        let seed = (0..3)
            .find(|i| subject == format!("Seeded {i}"))
            .unwrap_or_else(|| panic!("unexpected message {e}"));
        by_seed[seed] = Some(id);
    }
    by_seed.map(|id| id.expect("every seed present"))
}

fn sorted(ids: &[&String]) -> Vec<String> {
    let mut v: Vec<String> = ids.iter().map(|s| (*s).clone()).collect();
    v.sort();
    v
}

async fn set_update(h: &Harness, id: &str, patch: Value) -> Value {
    let r = jmap(h, json!([["Email/set", { "update": { id: patch } }, "u"]])).await;
    result(&r, "u").clone()
}

fn assert_updated(set: &Value, id: &str, context: &str) {
    assert!(
        set["updated"].as_object().unwrap().contains_key(id),
        "{context}: expected {id} under updated, got {set}"
    );
    assert!(
        set.get("notUpdated").is_none(),
        "{context}: unexpected notUpdated in {set}"
    );
}

fn assert_not_updated(set: &Value, id: &str, error_type: &str, context: &str) {
    assert_eq!(
        set["notUpdated"][id]["type"], error_type,
        "{context}: expected notUpdated.{id} = {error_type}, got {set}"
    );
    assert!(
        !set["updated"].as_object().unwrap().contains_key(id),
        "{context}: {id} is reported as updated as well: {set}"
    );
}

// ---------------------------------------------------------------------------
// (a) Email/set update: keyword patch paths
// ---------------------------------------------------------------------------

#[tokio::test]
async fn keyword_patch_path_marks_a_message_read_upstream_and_locally() {
    let h = setup().await;
    let inbox = inbox(&h).await;
    let [plain, _, _] = seeded_ids(&h, &inbox).await;

    // Precondition: unread in the row, unread in the index, nothing written
    // upstream yet.
    assert_eq!(email(&h, &plain).await["keywords"], json!({}));
    assert!(
        query(&h, json!({ "inMailbox": inbox, "text": "is:unread" }))
            .await
            .contains(&plain)
    );
    assert!(h.backend.flag_calls().is_empty());

    let set = set_update(&h, &plain, json!({ "keywords/$seen": true })).await;
    assert_updated(&set, &plain, "keywords/$seen");
    assert_ne!(
        set["oldState"], set["newState"],
        "a keyword change advances the Email state: {set}"
    );

    // The upstream flag write, the row, and the index — all three.
    assert_eq!(
        h.backend.flag_calls(),
        vec![(vec![1], vec![Flag::Seen], Vec::new())],
        "exactly one STORE +\\Seen for uid 1"
    );
    assert_eq!(
        email(&h, &plain).await["keywords"],
        json!({ "$seen": true })
    );
    assert!(
        !query(&h, json!({ "inMailbox": inbox, "text": "is:unread" }))
            .await
            .contains(&plain),
        "the index still lists the message as unread"
    );
}

#[tokio::test]
async fn keyword_patch_path_changes_one_keyword_and_keeps_the_others() {
    let h = setup().await;
    let inbox = inbox(&h).await;
    let [_, flagged, _] = seeded_ids(&h, &inbox).await;
    assert_eq!(
        email(&h, &flagged).await["keywords"],
        json!({ "$flagged": true }),
        "precondition: flagged, unread"
    );

    // A path adds to the set; the whole-object form would have replaced it.
    let set = set_update(&h, &flagged, json!({ "keywords/$seen": true })).await;
    assert_updated(&set, &flagged, "add $seen");
    assert_eq!(
        email(&h, &flagged).await["keywords"],
        json!({ "$flagged": true, "$seen": true })
    );

    // `null` clears exactly the named keyword.
    let set = set_update(&h, &flagged, json!({ "keywords/$flagged": null })).await;
    assert_updated(&set, &flagged, "clear $flagged");
    assert_eq!(
        email(&h, &flagged).await["keywords"],
        json!({ "$seen": true })
    );

    // A user keyword, which is what the web's tag action sends.
    let set = set_update(&h, &flagged, json!({ "keywords/Work": true })).await;
    assert_updated(&set, &flagged, "add a user keyword");
    assert_eq!(
        email(&h, &flagged).await["keywords"],
        json!({ "$seen": true, "Work": true })
    );

    assert_eq!(
        h.backend.flag_calls(),
        vec![
            (vec![2], vec![Flag::Seen], Vec::new()),
            (vec![2], Vec::new(), vec![Flag::Flagged]),
            (vec![2], vec![Flag::Keyword("Work".into())], Vec::new()),
        ]
    );

    // Both forms in one patch: the object replaces, the path applies on top.
    let set = set_update(
        &h,
        &flagged,
        json!({ "keywords": { "$flagged": true }, "keywords/$answered": true }),
    )
    .await;
    assert_updated(&set, &flagged, "object + path");
    assert_eq!(
        email(&h, &flagged).await["keywords"],
        json!({ "$flagged": true, "$answered": true })
    );
}

#[tokio::test]
async fn an_update_the_engine_cannot_apply_is_not_reported_as_updated() {
    let h = setup().await;
    let inbox = inbox(&h).await;
    let [plain, _, _] = seeded_ids(&h, &inbox).await;
    let before = email(&h, &plain).await;

    // Unknown id, with a patch that is itself fine.
    let set = set_update(&h, "no-such-id", json!({ "keywords/$seen": true })).await;
    assert_eq!(
        set["notUpdated"]["no-such-id"]["type"], "serverFail",
        "unknown id: {set}"
    );
    assert!(set["updated"].as_object().unwrap().is_empty(), "{set}");

    // Patches naming nothing this method applies. Each used to come back under
    // `updated` with nothing written.
    let refused: [(Value, &str, &str); 5] = [
        (json!({}), "invalidPatch", "empty patch"),
        (
            json!({ "subject": "x" }),
            "invalidProperties",
            "unknown property",
        ),
        (
            json!({ "mailboxIds/abc": true }),
            "invalidProperties",
            "mailboxIds patch path",
        ),
        (
            json!({ "keywords/$seen": "yes" }),
            "invalidProperties",
            "keyword path with a non-boolean value",
        ),
        (
            json!({ "keywords/$seen": true, "subject": "x" }),
            "invalidProperties",
            "one applicable property beside one that is not",
        ),
    ];
    for (patch, error_type, context) in refused {
        let set = set_update(&h, &plain, patch).await;
        assert_not_updated(&set, &plain, error_type, context);
        assert_eq!(
            set["oldState"], set["newState"],
            "{context}: a refused update must not advance the state: {set}"
        );
    }
    let set = set_update(
        &h,
        &plain,
        json!({ "keywords/$seen": true, "subject": "x" }),
    )
    .await;
    assert_eq!(
        set["notUpdated"][&plain]["properties"],
        json!(["subject"]),
        "the error names the property that could not be applied: {set}"
    );

    // None of the refusals wrote anything, upstream or locally.
    assert!(
        h.backend.flag_calls().is_empty(),
        "{:?}",
        h.backend.flag_calls()
    );
    assert_eq!(email(&h, &plain).await, before);

    // Control: a patch that is applicable but changes nothing is still a
    // success — marking a read message read must not surface as an error.
    let first = set_update(&h, &plain, json!({ "keywords/$seen": true })).await;
    assert_updated(&first, &plain, "first mark-read");
    let again = set_update(&h, &plain, json!({ "keywords/$seen": true })).await;
    assert_updated(&again, &plain, "idempotent mark-read");
}

// ---------------------------------------------------------------------------
// (b) Email/set create: inReplyTo
// ---------------------------------------------------------------------------

fn reply_draft(overrides: Value) -> Value {
    let mut spec = json!({
        "from": [{ "email": "me@example.org" }],
        "to": [{ "email": "alice@example.org" }],
        "subject": "An unrelated subject",
        "bodyValues": { "1": { "value": "reply body" } },
        "textBody": [{ "partId": "1", "type": "text/plain" }],
    });
    for (k, v) in overrides.as_object().unwrap() {
        spec[k] = v.clone();
    }
    spec
}

/// Create the draft and submit it at once. Returns `(Email/set, EmailSubmission/set)`.
async fn create_and_send(h: &Harness, spec: Value) -> (Value, Value) {
    let r = jmap(
        h,
        json!([
            ["Email/set", { "create": { "draft": spec } }, "c1"],
            ["EmailSubmission/set", { "create": { "s1": {
                "emailId": "#draft", "mailwomanHoldSeconds": 0
            } } }, "c2"]
        ]),
    )
    .await;
    (result(&r, "c1").clone(), result(&r, "c2").clone())
}

/// Create the draft and leave it unsent. Returns its `threadId`.
///
/// Threading is read off the draft, not the sent copy: a send files a new
/// message in Sent and the draft's id stops resolving.
async fn draft_thread(h: &Harness, spec: Value) -> Value {
    let r = jmap(
        h,
        json!([["Email/set", { "create": { "draft": spec } }, "c"]]),
    )
    .await;
    let id = result(&r, "c")["created"]["draft"]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("draft not created: {r}"))
        .to_string();
    email(h, &id).await["threadId"].clone()
}

/// The value of header `name` in a raw message, unfolded, or `None`.
fn header(raw: &str, name: &str) -> Option<String> {
    let head = raw.split("\r\n\r\n").next().unwrap_or_default();
    let unfolded = head.replace("\r\n\t", " ").replace("\r\n ", " ");
    unfolded.lines().find_map(|line| {
        let (n, v) = line.split_once(':')?;
        n.eq_ignore_ascii_case(name)
            .then(|| v.split_whitespace().collect::<Vec<_>>().join(" "))
    })
}

#[tokio::test]
async fn in_reply_to_as_a_list_writes_the_header_and_joins_the_parent_thread() {
    let h = setup().await;
    let inbox = inbox(&h).await;
    let [parent, _, _] = seeded_ids(&h, &inbox).await;
    let parent_thread = email(&h, &parent).await["threadId"].clone();
    assert!(parent_thread.is_string());

    // Precondition: the same draft WITHOUT `inReplyTo` is sent with no
    // `In-Reply-To` and lands in a thread of its own, so the subject is not
    // what joins the two below.
    let (_, sub) = create_and_send(&h, reply_draft(json!({}))).await;
    assert_eq!(sub["created"]["s1"]["undoStatus"], "final", "{sub}");
    let sent = h.submitter.sent();
    assert_eq!(sent.len(), 1);
    assert_eq!(header(&sent[0], "In-Reply-To"), None);
    assert_ne!(
        draft_thread(&h, reply_draft(json!({}))).await,
        parent_thread
    );

    // RFC 8621: `inReplyTo` is `String[]`.
    let (set, sub) = create_and_send(&h, reply_draft(json!({ "inReplyTo": ["<seed-0@x>"] }))).await;
    assert!(set.get("notCreated").is_none(), "{set}");
    assert_eq!(sub["created"]["s1"]["undoStatus"], "final", "{sub}");
    let sent = h.submitter.sent();
    assert_eq!(sent.len(), 2);
    assert_eq!(
        header(&sent[1], "In-Reply-To").as_deref(),
        Some("<seed-0@x>"),
        "transmitted message:\n{}",
        sent[1]
    );
    assert_eq!(
        draft_thread(&h, reply_draft(json!({ "inReplyTo": ["<seed-0@x>"] }))).await,
        parent_thread,
        "the reply is filed in its parent's thread"
    );

    // The bare-string form this engine has always read still works, with or
    // without the angle brackets.
    for bare in ["<seed-0@x>", "seed-0@x"] {
        let (set, _) = create_and_send(&h, reply_draft(json!({ "inReplyTo": bare }))).await;
        assert!(set.get("notCreated").is_none(), "{bare}: {set}");
        let sent = h.submitter.sent();
        assert_eq!(
            header(sent.last().unwrap(), "In-Reply-To").as_deref(),
            Some("<seed-0@x>"),
            "{bare}"
        );
    }

    // Several ids: every one is written, each in its own angle brackets.
    let (set, _) = create_and_send(
        &h,
        reply_draft(json!({ "inReplyTo": ["<seed-0@x>", "seed-1@x"] })),
    )
    .await;
    assert!(set.get("notCreated").is_none(), "{set}");
    let sent = h.submitter.sent();
    assert_eq!(
        header(sent.last().unwrap(), "In-Reply-To").as_deref(),
        Some("<seed-0@x> <seed-1@x>")
    );
}

#[tokio::test]
async fn in_reply_to_list_elements_are_checked_like_the_bare_string() {
    let h = setup().await;

    // t27's control-character refusal applies to every element of the list,
    // wherever it sits, and to the bare string as before.
    let hostile: [(Value, &str); 5] = [
        (json!("<a@x>\r\nBcc: victim@example.net"), "bare string"),
        (json!(["<a@x>\r\nBcc: victim@example.net"]), "only element"),
        (
            json!(["<ok@x>", "<a@x>\r\nBcc: victim@example.net"]),
            "second element",
        ),
        (json!(["<a@x>\nX-Injected: 1", "<ok@x>"]), "first element"),
        (json!(["<a\u{0}@x>"]), "NUL"),
    ];
    for (value, context) in hostile {
        let (set, sub) = create_and_send(&h, reply_draft(json!({ "inReplyTo": value }))).await;
        let err = &set["notCreated"]["draft"];
        assert_eq!(err["type"], "invalidProperties", "{context}: {set}");
        assert_eq!(err["properties"], json!(["inReplyTo"]), "{context}: {set}");
        assert!(
            set["created"].as_object().unwrap().is_empty(),
            "{context}: {set}"
        );
        assert!(
            sub["created"].as_object().unwrap().is_empty(),
            "{context}: the refused draft was submitted: {sub}"
        );
    }

    // A value that is not an id list at all is refused rather than dropped.
    for (value, context) in [
        (json!(["<ok@x>", 7]), "number in the list"),
        (json!({ "id": "<ok@x>" }), "object"),
        (json!(7), "number"),
    ] {
        let (set, _) = create_and_send(&h, reply_draft(json!({ "inReplyTo": value }))).await;
        assert_eq!(
            set["notCreated"]["draft"]["properties"],
            json!(["inReplyTo"]),
            "{context}: {set}"
        );
    }

    assert!(
        h.submitter.sent().is_empty(),
        "nothing above may have reached the submitter"
    );

    // Control: `null` and an empty list are "no parent", not errors.
    for value in [Value::Null, json!([])] {
        let (set, sub) = create_and_send(&h, reply_draft(json!({ "inReplyTo": value }))).await;
        assert!(set.get("notCreated").is_none(), "{set}");
        assert_eq!(sub["created"]["s1"]["undoStatus"], "final", "{sub}");
    }
    let sent = h.submitter.sent();
    assert_eq!(sent.len(), 2);
    assert!(sent.iter().all(|m| header(m, "In-Reply-To").is_none()));
}

// ---------------------------------------------------------------------------
// (c) EmailSubmission/set create: sendAt
// ---------------------------------------------------------------------------

/// Create a draft, then submit it in a second request with `submission`
/// merged over `{emailId}`. Returns the `EmailSubmission/set` result.
async fn submit_with(h: &Harness, submission: Value) -> Value {
    let c = jmap(
        h,
        json!([["Email/set", { "create": { "draft": reply_draft(json!({})) } }, "c"]]),
    )
    .await;
    let id = result(&c, "c")["created"]["draft"]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("draft not created: {c}"))
        .to_string();
    let mut spec = json!({ "emailId": id });
    for (k, v) in submission.as_object().unwrap() {
        spec[k] = v.clone();
    }
    let r = jmap(
        h,
        json!([["EmailSubmission/set", { "create": { "s1": spec } }, "s"]]),
    )
    .await;
    result(&r, "s").clone()
}

async fn submission_count(h: &Harness) -> usize {
    let q = jmap(h, json!([["EmailSubmission/query", {}, "q"]])).await;
    result(&q, "q")["ids"]
        .as_array()
        .unwrap_or_else(|| panic!("EmailSubmission/query: {q}"))
        .len()
}

fn rfc3339_from_now(offset: chrono::Duration) -> String {
    (chrono::Utc::now() + offset).to_rfc3339()
}

#[tokio::test]
async fn a_send_at_that_is_past_or_not_a_time_is_refused_and_nothing_is_sent() {
    let h = setup().await;
    assert!(h.submitter.sent().is_empty());
    assert_eq!(submission_count(&h).await, 0);

    let refused: [(Value, &str); 6] = [
        (
            json!(rfc3339_from_now(chrono::Duration::days(-1))),
            "yesterday",
        ),
        (
            json!(rfc3339_from_now(chrono::Duration::minutes(-5))),
            "five minutes ago",
        ),
        (json!("2020-01-01T00:00:00Z"), "years ago"),
        (json!("tomorrow at nine"), "not a time"),
        (json!("2099-01-01 09:00"), "not RFC 3339"),
        (json!(4_102_444_800_u64), "a number"),
    ];
    // With and without a hold window: the hold is not what keeps it unsent.
    for hold in [0, 30] {
        for (send_at, context) in &refused {
            let set = submit_with(
                &h,
                json!({ "sendAt": send_at, "mailwomanHoldSeconds": hold }),
            )
            .await;
            let err = &set["notCreated"]["s1"];
            assert_eq!(
                err["type"], "invalidProperties",
                "{context} (hold {hold}): {set}"
            );
            assert_eq!(err["properties"], json!(["sendAt"]), "{context}: {set}");
            assert!(
                set["created"].as_object().unwrap().is_empty(),
                "{context} (hold {hold}): {set}"
            );
            assert_eq!(
                set["oldState"], set["newState"],
                "{context}: a refused create must not advance the state: {set}"
            );
        }
    }
    assert!(
        h.submitter.sent().is_empty(),
        "a refused sendAt reached the submitter"
    );
    assert_eq!(
        submission_count(&h).await,
        0,
        "a refused sendAt left a submission row for the dispatcher"
    );
}

#[tokio::test]
async fn a_valid_send_at_still_defers_and_no_send_at_still_sends() {
    let h = setup().await;

    // Future: held for the dispatcher, not transmitted.
    let set = submit_with(
        &h,
        json!({ "sendAt": rfc3339_from_now(chrono::Duration::hours(1)), "mailwomanHoldSeconds": 0 }),
    )
    .await;
    assert_eq!(set["created"]["s1"]["undoStatus"], "pending", "{set}");
    assert!(h.submitter.sent().is_empty());
    assert_eq!(submission_count(&h).await, 1);

    // A few seconds behind the server clock is a client asking for "now".
    let set = submit_with(
        &h,
        json!({ "sendAt": rfc3339_from_now(chrono::Duration::seconds(-10)), "mailwomanHoldSeconds": 0 }),
    )
    .await;
    assert_eq!(set["created"]["s1"]["undoStatus"], "final", "{set}");
    assert_eq!(h.submitter.sent().len(), 1);

    // Absent and `null` are the plain immediate send.
    let set = submit_with(&h, json!({ "mailwomanHoldSeconds": 0 })).await;
    assert_eq!(set["created"]["s1"]["undoStatus"], "final", "{set}");
    let set = submit_with(&h, json!({ "sendAt": null, "mailwomanHoldSeconds": 0 })).await;
    assert_eq!(set["created"]["s1"]["undoStatus"], "final", "{set}");
    assert_eq!(h.submitter.sent().len(), 3);
}

// ---------------------------------------------------------------------------
// (d) Email/query: keyword, date and size conditions
// ---------------------------------------------------------------------------

#[tokio::test]
async fn keyword_date_and_size_filters_select_instead_of_returning_the_mailbox() {
    let h = setup().await;
    let inbox = inbox(&h).await;
    let [plain, flagged, attach] = seeded_ids(&h, &inbox).await;
    let all = sorted(&[&plain, &flagged, &attach]);
    // Precondition: the unfiltered mailbox is all three, so "the whole mailbox"
    // is distinguishable from every expected answer below.
    assert_eq!(query(&h, json!({ "inMailbox": inbox })).await, all);

    let cases: [(Value, Vec<String>, &str); 8] = [
        (
            json!({ "hasKeyword": "$flagged" }),
            sorted(&[&flagged]),
            "hasKeyword",
        ),
        (
            json!({ "notKeyword": "$flagged" }),
            sorted(&[&plain, &attach]),
            "notKeyword",
        ),
        (
            json!({ "after": "2026-07-02T12:00:00Z" }),
            sorted(&[&attach]),
            "after",
        ),
        (
            json!({ "before": "2026-07-02T12:00:00Z" }),
            sorted(&[&plain, &flagged]),
            "before",
        ),
        (
            json!({ "after": "2026-07-01T12:00:00Z", "before": "2026-07-02T12:00:00Z" }),
            sorted(&[&flagged]),
            "after + before",
        ),
        (json!({ "minSize": 4000 }), sorted(&[&attach]), "minSize"),
        (
            json!({ "maxSize": 4000 }),
            sorted(&[&plain, &flagged]),
            "maxSize",
        ),
        (
            json!({ "hasKeyword": "$flagged", "minSize": 4000 }),
            Vec::new(),
            "hasKeyword + minSize",
        ),
    ];
    for (condition, expected, context) in cases {
        let mut filter = condition.clone();
        filter["inMailbox"] = json!(inbox);
        let got = query(&h, filter).await;
        assert_eq!(got, expected, "{context}: {condition}");
        assert_ne!(got, all, "{context} returned the whole mailbox");
    }

    // The keyword condition follows a keyword written through `Email/set`.
    let set = set_update(&h, &plain, json!({ "keywords/$flagged": true })).await;
    assert_updated(&set, &plain, "flag a second message");
    assert_eq!(
        query(&h, json!({ "inMailbox": inbox, "hasKeyword": "$flagged" })).await,
        sorted(&[&plain, &flagged])
    );
    assert_eq!(
        query(&h, json!({ "inMailbox": inbox, "notKeyword": "$flagged" })).await,
        sorted(&[&attach])
    );
}

// ---------------------------------------------------------------------------
// (e) has: through the engine's text filter
// ---------------------------------------------------------------------------

#[tokio::test]
async fn has_attachments_selects_the_messages_with_an_attachment() {
    let h = setup().await;
    let inbox = inbox(&h).await;
    let [plain, flagged, attach] = seeded_ids(&h, &inbox).await;
    let text = |q: &str| json!({ "inMailbox": inbox, "text": q });

    // Precondition: the spelling that always worked, and its complement.
    assert_eq!(query(&h, text("has:attachment")).await, sorted(&[&attach]));
    assert_eq!(
        query(&h, text("-has:attachment")).await,
        sorted(&[&plain, &flagged])
    );

    for spelling in ["has:attachments", "has:file", "has:files"] {
        assert_eq!(
            query(&h, text(spelling)).await,
            sorted(&[&attach]),
            "{spelling}"
        );
    }
    // A value `has:` does not define selects neither side of the split.
    let unknown = query(&h, text("has:zzqx")).await;
    assert_ne!(unknown, sorted(&[&plain, &flagged]), "has:zzqx inverted");
    assert_ne!(unknown, sorted(&[&attach]));
}
