//! 26.20 t28-e11b — `Email/get` returns `messageId`, `inReplyTo` and
//! `references` (RFC 8621 §4.1.2.3), driven through `Engine::handle_jmap`.
//!
//! The engine answers `Email/get` from the JSON it stored for the message at
//! ingest and applies no `properties` filter, so the three values reach a
//! client when the parser fills them and the type serialises them. These tests
//! hold that whole path: synced mail, a draft the engine composed itself, and
//! a row stored before the properties existed.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
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
use mw_store::{AccountKind, Credentials, MessageUpsert, NewAccount, ServerKey, Store};

const UIDVALIDITY: u32 = 100;

type ScriptMsg = (u32, Vec<u8>, Vec<Flag>, String);

struct Scripted {
    mailboxes: Vec<(String, MailboxRole)>,
    messages: HashMap<String, Vec<ScriptMsg>>,
}

struct FakeBackend {
    state: Mutex<Scripted>,
}

impl FakeBackend {
    fn new() -> Self {
        let mailboxes = vec![
            ("INBOX".to_string(), MailboxRole::Inbox),
            ("Drafts".to_string(), MailboxRole::Drafts),
        ];
        let mut messages: HashMap<String, Vec<ScriptMsg>> = HashMap::new();
        messages.insert(
            "INBOX".to_string(),
            vec![
                (1, root_msg(), vec![], "2026-07-01T09:00:00Z".into()),
                (2, reply_msg(), vec![], "2026-07-02T09:00:00Z".into()),
                (3, odd_msg(), vec![], "2026-07-03T09:00:00Z".into()),
            ],
        );
        Self {
            state: Mutex::new(Scripted {
                mailboxes,
                messages,
            }),
        }
    }
}

#[async_trait]
impl AccountBackend for FakeBackend {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps {
            uidplus: true,
            r#move: true,
            special_use: true,
            ..BackendCaps::default()
        })
    }

    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        let st = self.state.lock().unwrap();
        Ok(st
            .mailboxes
            .iter()
            .map(|(name, role)| {
                let total = st.messages.get(name).map(|m| m.len()).unwrap_or(0) as u32;
                RawMailbox {
                    mailbox_ref: RawMailboxRef {
                        name: name.clone(),
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

    async fn sync_mailbox(
        &self,
        mbox: &RawMailboxRef,
        cursor: &SyncCursor,
    ) -> Result<MailboxDelta> {
        let uidnext_from = match cursor {
            SyncCursor::UidWindow { uidnext, .. } => *uidnext,
            _ => 1,
        };
        let st = self.state.lock().unwrap();
        let msgs = st.messages.get(&mbox.name).cloned().unwrap_or_default();
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
        let st = self.state.lock().unwrap();
        let mut out = Vec::new();
        for r in refs {
            let MessageRef::Imap { mailbox, uid, .. } = r else {
                continue;
            };
            if let Some(msgs) = st.messages.get(&mailbox.name)
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

    async fn store_flags(&self, _r: &[MessageRef], _a: &[Flag], _rm: &[Flag]) -> Result<()> {
        Ok(())
    }

    async fn move_messages(&self, _r: &[MessageRef], _to: &RawMailboxRef) -> Result<MoveOutcome> {
        Ok(MoveOutcome::Uidplus {
            uidvalidity: UIDVALIDITY,
            uids: vec![9001],
        })
    }

    async fn append(&self, mbox: &RawMailboxRef, raw: &[u8], flags: &[Flag]) -> Result<MessageRef> {
        let mut st = self.state.lock().unwrap();
        let entry = st.messages.entry(mbox.name.clone()).or_default();
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

struct FakeSubmitter {
    calls: AtomicUsize,
}

#[async_trait]
impl MailSubmitter for FakeSubmitter {
    async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(SubmissionResult {
            accepted: msg.rcpt_to,
            rejected: Vec::new(),
        })
    }
}

struct Harness {
    engine: Arc<Engine>,
    account_id: String,
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
    let submitter = Arc::new(FakeSubmitter {
        calls: AtomicUsize::new(0),
    });
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            Arc::new(FakeBackend::new()) as Arc<dyn AccountBackend>,
            submitter.clone() as Arc<dyn MailSubmitter>,
            "me@example.org",
        ),
    );
    Harness { engine, account_id }
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

fn root_msg() -> Vec<u8> {
    concat!(
        "Message-ID: <root@example.org>\r\n",
        "From: alice@example.org\r\n",
        "To: me@example.org\r\n",
        "Subject: Root\r\n",
        "Date: Wed, 01 Jul 2026 09:00:00 +0000\r\n",
        "\r\n",
        "first\r\n",
    )
    .as_bytes()
    .to_vec()
}

fn reply_msg() -> Vec<u8> {
    concat!(
        "Message-ID: <reply@example.org>\r\n",
        "In-Reply-To: <middle@example.org>\r\n",
        "References: <root@example.org>\r\n",
        " (a comment) <middle@example.org>\r\n",
        "From: bob@example.org\r\n",
        "To: me@example.org\r\n",
        "Subject: Re: Root\r\n",
        "Date: Thu, 02 Jul 2026 09:00:00 +0000\r\n",
        "\r\n",
        "second\r\n",
    )
    .as_bytes()
    .to_vec()
}

/// No `Message-ID`, an `In-Reply-To` that is prose, a `References` with a
/// control character inside an id.
fn odd_msg() -> Vec<u8> {
    concat!(
        "In-Reply-To: your note of Tuesday\r\n",
        "References: <a\u{1b}[31m@example.org>\r\n",
        "From: carol@example.org\r\n",
        "To: me@example.org\r\n",
        "Subject: Odd\r\n",
        "Date: Fri, 03 Jul 2026 09:00:00 +0000\r\n",
        "\r\n",
        "third\r\n",
    )
    .as_bytes()
    .to_vec()
}

async fn inbox_id(h: &Harness) -> String {
    let mb = jmap(h, json!([["Mailbox/get", {}, "mb"]])).await;
    result(&mb, "mb")["list"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["role"] == "inbox")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

/// The synced inbox as `Email/get` returns it, keyed by subject. `properties`
/// is passed through as given (`Null` asks for the default set).
async fn inbox_by_subject(h: &Harness, properties: Value) -> HashMap<String, Value> {
    let inbox = inbox_id(h).await;
    let q = jmap(
        h,
        json!([["Email/query", { "filter": { "inMailbox": inbox } }, "q"]]),
    )
    .await;
    let ids = result(&q, "q")["ids"].clone();
    assert_eq!(
        ids.as_array().map(Vec::len),
        Some(3),
        "three seeded messages"
    );
    let mut args = json!({ "ids": ids });
    if !properties.is_null() {
        args["properties"] = properties;
    }
    let g = jmap(h, json!([["Email/get", args, "g"]])).await;
    result(&g, "g")["list"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| (e["subject"].as_str().unwrap().to_string(), e.clone()))
        .collect()
}

#[tokio::test]
async fn synced_mail_returns_its_ids_without_brackets() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    let list = inbox_by_subject(
        &h,
        json!(["subject", "messageId", "inReplyTo", "references"]),
    )
    .await;

    let root = &list["Root"];
    assert_eq!(root["messageId"], json!(["root@example.org"]));
    assert_eq!(root["inReplyTo"], Value::Null);
    assert_eq!(root["references"], Value::Null);

    let reply = &list["Re: Root"];
    assert_eq!(reply["messageId"], json!(["reply@example.org"]));
    assert_eq!(reply["inReplyTo"], json!(["middle@example.org"]));
    assert_eq!(
        reply["references"],
        json!(["root@example.org", "middle@example.org"])
    );
}

#[tokio::test]
async fn the_default_property_set_carries_them_too() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    let list = inbox_by_subject(&h, Value::Null).await;
    let reply = &list["Re: Root"];
    assert_eq!(reply["messageId"], json!(["reply@example.org"]));
    assert_eq!(reply["inReplyTo"], json!(["middle@example.org"]));
    assert_eq!(
        reply["references"],
        json!(["root@example.org", "middle@example.org"])
    );
}

#[tokio::test]
async fn a_missing_or_malformed_header_is_an_explicit_null() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    let list = inbox_by_subject(&h, Value::Null).await;
    let odd = list["Odd"].as_object().unwrap();
    // Present and null: the key is there, which is what tells a client this
    // message was parsed for the properties and has nothing to report.
    for key in ["messageId", "inReplyTo", "references"] {
        assert_eq!(odd.get(key), Some(&Value::Null), "{key}");
    }
}

#[tokio::test]
async fn a_reply_draft_reads_back_the_headers_it_was_composed_with() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    let set = jmap(
        &h,
        json!([[
            "Email/set",
            {
                "create": {
                    "d": {
                        "to": [{ "email": "bob@example.org" }],
                        "subject": "Re: Root",
                        "inReplyTo": ["reply@example.org"],
                        "references": [
                            "root@example.org",
                            "middle@example.org",
                            "reply@example.org"
                        ],
                        "textBody": [{ "partId": "t", "type": "text/plain" }],
                        "bodyValues": { "t": { "value": "third" } }
                    }
                }
            },
            "s"
        ]]),
    )
    .await;
    let id = result(&set, "s")["created"]["d"]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("draft created: {:?}", result(&set, "s")))
        .to_string();

    let g = jmap(&h, json!([["Email/get", { "ids": [id] }, "g"]])).await;
    let draft = &result(&g, "g")["list"][0];
    assert_eq!(draft["inReplyTo"], json!(["reply@example.org"]));
    assert_eq!(
        draft["references"],
        json!([
            "root@example.org",
            "middle@example.org",
            "reply@example.org"
        ])
    );
    // The engine gave the draft one Message-ID of its own.
    let own = draft["messageId"].as_array().expect("messageId is a list");
    assert_eq!(own.len(), 1);
    let own = own[0].as_str().unwrap();
    assert!(!own.is_empty() && !own.contains(['<', '>']), "{own:?}");
}

/// Store a row the way ingest wrote one before 26.20: an envelope with no
/// threading keys. `raw` is its stored message, if it has one.
async fn store_old_row(h: &Harness, uid: u32, raw: Option<&[u8]>) -> String {
    let inbox = inbox_id(h).await;
    let blob = match raw {
        Some(raw) => Some(h.engine.store().put_body(&h.account_id, raw).await.unwrap()),
        None => None,
    };
    h.engine
        .store()
        .upsert_message(&MessageUpsert {
            account_id: &h.account_id,
            mailbox_id: &inbox,
            uid,
            uidvalidity: UIDVALIDITY,
            message_id: Some(&format!("old-{uid}@example.org")),
            thread_id: None,
            internaldate: Some("2026-06-01T09:00:00Z"),
            size: 3,
            flags_json: "[]",
            envelope: Some(br#"{"subject":"Old","size":3}"#),
            blob_ref: blob.as_deref(),
        })
        .await
        .unwrap()
}

/// 26.20 t28-e12. This case used to pin the opposite: a row stored before the
/// properties existed came back without the three keys, and a client had to
/// parse the raw headers itself. `Email/get` now completes such a row from its
/// stored message when the request reads the properties.
#[tokio::test]
async fn a_row_stored_before_the_properties_existed_is_completed_from_its_message() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    let with_ids = store_old_row(&h, 50, Some(&reply_msg())).await;
    let without = store_old_row(&h, 51, Some(&odd_msg())).await;

    // Precondition: the stored envelope really has none of the keys.
    let stored: Value = serde_json::from_slice(
        &h.engine
            .store()
            .get_envelope(&with_ids)
            .await
            .unwrap()
            .expect("a stored envelope"),
    )
    .unwrap();
    assert_eq!(stored, json!({ "subject": "Old", "size": 3 }));

    for properties in [
        json!(null),
        json!(["id", "subject", "messageId"]),
        json!(["references"]),
    ] {
        let g = jmap(
            &h,
            json!([["Email/get", { "ids": [with_ids, without], "properties": properties }, "g"]]),
        )
        .await;
        let list = result(&g, "g")["list"].as_array().expect("list").clone();
        // The same values, in the same form, as for newly synced mail.
        assert_eq!(
            list[0]["subject"], "Old",
            "the stored envelope is still served"
        );
        assert_eq!(list[0]["messageId"], json!(["reply@example.org"]));
        assert_eq!(list[0]["inReplyTo"], json!(["middle@example.org"]));
        assert_eq!(
            list[0]["references"],
            json!(["root@example.org", "middle@example.org"])
        );
        // Present and null where the header is absent or not a clean id list.
        let odd = list[1].as_object().expect("found");
        for key in ["messageId", "inReplyTo", "references"] {
            assert_eq!(odd.get(key), Some(&Value::Null), "{key} with {properties}");
        }
    }

    // Nothing was written back: the row is completed on each request.
    let after: Value = serde_json::from_slice(
        &h.engine
            .store()
            .get_envelope(&with_ids)
            .await
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(after, stored);
}

/// A request that names its properties and none of the three is not charged
/// the body read: the keys stay absent, as stored.
#[tokio::test]
async fn an_old_row_is_not_completed_for_a_request_that_does_not_read_the_properties() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    let old = store_old_row(&h, 50, Some(&reply_msg())).await;
    let g = jmap(
        &h,
        json!([["Email/get", { "ids": [old], "properties": ["id", "subject", "from"] }, "g"]]),
    )
    .await;
    let email = result(&g, "g")["list"][0]
        .as_object()
        .expect("found")
        .clone();
    assert_eq!(email["subject"], "Old");
    for key in ["messageId", "inReplyTo", "references"] {
        assert!(!email.contains_key(key), "{key} was not asked for");
    }
}

/// A row with no stored message has nothing to read them from: the keys stay
/// absent, which is the engine saying it does not know, not that there are none.
#[tokio::test]
async fn an_old_row_without_a_stored_message_still_has_no_such_keys() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    let old = store_old_row(&h, 50, None).await;
    let g = jmap(&h, json!([["Email/get", { "ids": [old] }, "g"]])).await;
    let email = result(&g, "g")["list"][0]
        .as_object()
        .expect("found")
        .clone();
    assert_eq!(email["subject"], "Old");
    for key in ["messageId", "inReplyTo", "references"] {
        assert!(!email.contains_key(key), "{key} on a row with no body");
    }
}
