//! t32-h7 (D-24): a search never returns another account's mail.
//!
//! One engine keeps one full-text index for every account it serves. Each
//! index-backed `Email/query` is scoped by the querying account's mailbox ids.
//! Before this fix the scope clause was left out when the account had no
//! mailboxes, so a new account (before its first sync), or one whose folders
//! were all removed, searched the whole index and got back every other
//! account's message ids.
//!
//! Three accounts share one engine here:
//! - A has an inbox holding mail with a word nobody else has;
//! - B has no mailboxes at all;
//! - C has mailboxes, none of which hold anything that matches.
//!
//! Every query goes through `Engine::handle_jmap`, the `/jmap/api` envelope the
//! web client sends, and covers each filter shape that reaches the index: a text
//! search, an `in:` operator naming A's inbox, an `inMailbox` naming A's inbox,
//! `inMailboxOtherThan`, a keyword condition, a custom sort with no condition,
//! and `calculateTotal`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::{Value, json};

use mw_engine::account::AccountRuntime;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, EngineError, Flag, MailboxDelta, MailboxRole,
    MessageRef, MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result, SyncCursor,
    WatchHandle,
};
use mw_engine::{Engine, MailSubmitter};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

const UIDVALIDITY: u32 = 100;
/// A word that only account A's mail contains.
const SECRET: &str = "quillfeather";
/// Messages in account A's inbox.
const A_MESSAGES: usize = 3;

// ── harness ───────────────────────────────────────────────────────────────────

/// `(uid, raw)` per mailbox name.
type Mail = HashMap<String, Vec<(u32, Vec<u8>)>>;

/// A backend with an `INBOX` and an `Archive`, or with no mailboxes at all.
struct FakeBackend {
    mailboxes: Vec<(&'static str, MailboxRole)>,
    mail: Mutex<Mail>,
}

impl FakeBackend {
    fn empty() -> Self {
        Self {
            mailboxes: Vec::new(),
            mail: Mutex::new(HashMap::new()),
        }
    }

    fn with_inbox(raws: Vec<Vec<u8>>) -> Self {
        let mut mail = HashMap::new();
        mail.insert(
            "INBOX".to_string(),
            raws.into_iter()
                .enumerate()
                .map(|(i, r)| (i as u32 + 1, r))
                .collect(),
        );
        mail.insert("Archive".to_string(), Vec::new());
        Self {
            mailboxes: vec![
                ("INBOX", MailboxRole::Inbox),
                ("Archive", MailboxRole::Archive),
            ],
            mail: Mutex::new(mail),
        }
    }
}

fn message(owner: &str, i: usize, subject: &str, body: &str) -> Vec<u8> {
    format!(
        "Message-ID: <{owner}-{i}@example.org>\r\n\
         From: sender@example.org\r\n\
         To: {owner}@example.org\r\n\
         Subject: {subject}\r\n\
         Date: Wed, 01 Jul 2026 09:00:00 +0000\r\n\
         \r\n\
         {body}\r\n"
    )
    .into_bytes()
}

#[async_trait]
impl AccountBackend for FakeBackend {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps::default())
    }

    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        let mail = self.mail.lock().unwrap();
        Ok(self
            .mailboxes
            .iter()
            .map(|(name, role)| {
                let total = mail.get(*name).map(Vec::len).unwrap_or(0) as u32;
                RawMailbox {
                    mailbox_ref: RawMailboxRef {
                        name: name.to_string(),
                        uidvalidity: UIDVALIDITY,
                    },
                    role: *role,
                    parent: None,
                    uidnext: total + 1,
                    highestmodseq: 0,
                    total,
                    unread: total,
                }
            })
            .collect())
    }

    async fn sync_mailbox(&self, mbox: &RawMailboxRef, c: &SyncCursor) -> Result<MailboxDelta> {
        let from = match c {
            SyncCursor::UidWindow { uidnext, .. } => *uidnext,
            _ => 1,
        };
        let mail = self.mail.lock().unwrap();
        let msgs = mail.get(&mbox.name).cloned().unwrap_or_default();
        let max_uid = msgs.iter().map(|(u, _)| *u).max().unwrap_or(0);
        Ok(MailboxDelta {
            added: msgs
                .iter()
                .filter(|(uid, _)| *uid >= from)
                .map(|(uid, _)| MessageRef::Imap {
                    mailbox: mbox.clone(),
                    uidvalidity: UIDVALIDITY,
                    uid: *uid,
                })
                .collect(),
            flag_changes: Vec::new(),
            removed: Vec::new(),
            next_cursor: SyncCursor::UidWindow {
                uidvalidity: UIDVALIDITY,
                uidnext: max_uid + 1,
            },
        })
    }

    async fn fetch_raw(&self, refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
        let mail = self.mail.lock().unwrap();
        let mut out = Vec::new();
        for r in refs {
            let MessageRef::Imap { mailbox, uid, .. } = r else {
                continue;
            };
            if let Some((_, raw)) = mail
                .get(&mailbox.name)
                .and_then(|m| m.iter().find(|(u, _)| u == uid))
            {
                out.push(RawMessage {
                    message_ref: r.clone(),
                    raw: raw.clone(),
                    flags: Vec::new(),
                    internaldate: Some("2026-07-01T09:00:00Z".into()),
                });
            }
        }
        Ok(out)
    }

    async fn store_flags(&self, _r: &[MessageRef], _a: &[Flag], _d: &[Flag]) -> Result<()> {
        Ok(())
    }
    async fn move_messages(&self, _r: &[MessageRef], _to: &RawMailboxRef) -> Result<MoveOutcome> {
        Err(EngineError::Unsupported("fake".into()))
    }
    async fn append(&self, _m: &RawMailboxRef, _raw: &[u8], _f: &[Flag]) -> Result<MessageRef> {
        Err(EngineError::Unsupported("fake".into()))
    }
    async fn watch(&self, _sink: ChangeSink) -> Result<WatchHandle> {
        Err(EngineError::Unsupported("fake".into()))
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

async fn add_account(engine: &Engine, user: &str, backend: FakeBackend) -> String {
    let email = format!("{user}@example.org");
    let id = engine
        .store()
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example.org",
                port: 993,
                tls: "implicit",
                username: &email,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: email.clone(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap();
    engine.register_backend(
        id.clone(),
        AccountRuntime::new(
            Arc::new(backend) as Arc<dyn AccountBackend>,
            Arc::new(NoopSubmitter) as Arc<dyn MailSubmitter>,
            &email,
        ),
    );
    engine.resync(&id).await.unwrap();
    id
}

struct Fixture {
    engine: Arc<Engine>,
    a: String,
    b: String,
    c: String,
    a_inbox: String,
    a_ids: Vec<String>,
}

async fn fixture() -> Fixture {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let engine = Arc::new(Engine::new(store));
    let a = add_account(
        &engine,
        "alice",
        FakeBackend::with_inbox(
            (0..A_MESSAGES)
                .map(|i| {
                    message(
                        "alice",
                        i,
                        &format!("{SECRET} plans {i}"),
                        &format!("the {SECRET} budget, part {i}"),
                    )
                })
                .collect(),
        ),
    )
    .await;
    let b = add_account(&engine, "bob", FakeBackend::empty()).await;
    let c = add_account(
        &engine,
        "carol",
        FakeBackend::with_inbox(vec![message("carol", 0, "lunch", "see you at noon")]),
    )
    .await;

    let a_inbox = mailbox_with_role(&engine, &a, "inbox")
        .await
        .expect("account A has an inbox");
    let a_ids = ids(&query(&engine, &a, json!({ "inMailbox": a_inbox }), None).await);
    assert_eq!(a_ids.len(), A_MESSAGES, "account A's inbox was synced");

    // The premise of the empty-scope case: B really has no mailboxes.
    let b_boxes = jmap(&engine, &b, json!([["Mailbox/get", {}, "mb"]])).await;
    assert_eq!(
        result(&b_boxes, "mb")["list"].as_array().map(Vec::len),
        Some(0),
        "account B must have no mailboxes for this test to mean anything: {b_boxes}"
    );

    Fixture {
        engine,
        a,
        b,
        c,
        a_inbox,
        a_ids,
    }
}

async fn jmap(engine: &Engine, account: &str, calls: Value) -> Value {
    engine
        .handle_jmap(account, &json!({ "methodCalls": calls }))
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

async fn mailbox_with_role(engine: &Engine, account: &str, role: &str) -> Option<String> {
    let mb = jmap(engine, account, json!([["Mailbox/get", {}, "mb"]])).await;
    result(&mb, "mb")["list"]
        .as_array()?
        .iter()
        .find(|m| m["role"] == role)?["id"]
        .as_str()
        .map(str::to_string)
}

/// One `Email/query` as `account`, with `calculateTotal`; returns the result.
async fn query(engine: &Engine, account: &str, filter: Value, sort: Option<Value>) -> Value {
    let mut args = json!({ "filter": filter, "calculateTotal": true, "limit": 1000 });
    if let Some(sort) = sort {
        args["sort"] = sort;
    }
    let resp = jmap(engine, account, json!([["Email/query", args, "q"]])).await;
    result(&resp, "q").clone()
}

fn ids(res: &Value) -> Vec<String> {
    res["ids"]
        .as_array()
        .unwrap_or_else(|| panic!("Email/query answered without ids: {res}"))
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

/// The filter shapes that reach the index, each with an optional sort.
fn probes(a_inbox: &str) -> Vec<(&'static str, Value, Option<Value>)> {
    vec![
        ("text search", json!({ "text": SECRET }), None),
        (
            "in: operator naming A's inbox",
            json!({ "text": format!("in:{a_inbox}") }),
            None,
        ),
        (
            "inMailbox naming A's inbox, with text",
            json!({ "inMailbox": a_inbox, "text": SECRET }),
            None,
        ),
        (
            "inMailboxOtherThan with text",
            json!({ "inMailboxOtherThan": ["nothing"], "text": SECRET }),
            None,
        ),
        (
            "keyword condition only",
            json!({ "notKeyword": "$flagged" }),
            None,
        ),
        (
            "no condition, custom sort",
            json!({}),
            Some(json!([{ "property": "size", "isAscending": true }])),
        ),
        (
            "subject field",
            json!({ "subject": SECRET }),
            Some(json!([{ "property": "subject", "isAscending": true }])),
        ),
    ]
}

/// Run every probe as `account` and collect each one that returned any of A's
/// ids or a non-zero total.
async fn leaks(f: &Fixture, account: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (name, filter, sort) in probes(&f.a_inbox) {
        let res = query(&f.engine, account, filter, sort).await;
        let got = ids(&res);
        let leaked: Vec<&String> = got.iter().filter(|id| f.a_ids.contains(id)).collect();
        if !leaked.is_empty() {
            found.push(format!(
                "{name}: {} of A's ids returned, total {}",
                leaked.len(),
                res["total"]
            ));
        }
    }
    found
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn account_with_no_mailboxes_finds_nothing_of_another_account() {
    let f = fixture().await;
    let found = leaks(&f, &f.b).await;
    assert!(
        found.is_empty(),
        "an account with no mailboxes saw another account's mail:\n{}",
        found.join("\n")
    );
    // The total is a real zero too, not a count of the other account's mail.
    for (name, filter, sort) in probes(&f.a_inbox) {
        let res = query(&f.engine, &f.b, filter, sort).await;
        assert_eq!(res["total"], json!(0), "{name}: total for B: {res}");
    }
}

#[tokio::test]
async fn account_whose_mailboxes_hold_no_match_finds_nothing_of_another_account() {
    let f = fixture().await;
    let found = leaks(&f, &f.c).await;
    assert!(
        found.is_empty(),
        "an account with unrelated mailboxes saw another account's mail:\n{}",
        found.join("\n")
    );
}

#[tokio::test]
async fn owner_still_finds_its_own_mail() {
    let f = fixture().await;
    for (name, filter, sort) in probes(&f.a_inbox) {
        let mut got = ids(&query(&f.engine, &f.a, filter, sort).await);
        got.sort();
        let mut want = f.a_ids.clone();
        want.sort();
        assert_eq!(got, want, "{name}: account A must find all of its own mail");
    }
    // And C finds its own lunch message, not A's.
    let c_ids = ids(&query(&f.engine, &f.c, json!({ "text": "lunch" }), None).await);
    assert_eq!(c_ids.len(), 1, "account C finds its own mail");
    assert!(!f.a_ids.contains(&c_ids[0]));
}
