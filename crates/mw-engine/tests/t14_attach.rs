//! t14 (26.14) WS4a — `Email/set` create honors attachments whose `blobId`
//! resolves to an existing stored message/part via `Engine::fetch_blob`
//! (forward / attach-from-mail). Proves: an existing-part blobId yields a
//! multipart draft carrying that part (name + content-type + bytes); an
//! unresolved blobId is a clean `notCreated` set-error, never a panic.

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
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

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
            vec![(1, invoice_msg(), vec![], "2026-07-01T09:00:00Z".into())],
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

fn invoice_msg() -> Vec<u8> {
    concat!(
        "Message-ID: <invoice@x>\r\n",
        "From: alice@example.org\r\n",
        "To: me@example.org\r\n",
        "Subject: Invoice 2026\r\n",
        "Date: Wed, 01 Jul 2026 09:00:00 +0000\r\n",
        "MIME-Version: 1.0\r\n",
        "Content-Type: multipart/mixed; boundary=\"b1\"\r\n",
        "\r\n",
        "--b1\r\n",
        "Content-Type: text/plain; charset=utf-8\r\n",
        "\r\n",
        "Please pay the attached invoice\r\n",
        "--b1\r\n",
        "Content-Type: application/pdf; name=\"invoice.pdf\"\r\n",
        "Content-Disposition: attachment; filename=\"invoice.pdf\"\r\n",
        "Content-Transfer-Encoding: base64\r\n",
        "\r\n",
        "JVBERi0xLjQK\r\n",
        "--b1--\r\n",
    )
    .as_bytes()
    .to_vec()
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

/// The attachment blobId (`<stableId>.<partId>`) of the seeded invoice message.
async fn invoice_attachment_blob(h: &Harness) -> String {
    let mb = jmap(h, json!([["Mailbox/get", {}, "mb"]])).await;
    let inbox = result(&mb, "mb")["list"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["role"] == "inbox")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let q = jmap(
        h,
        json!([[
            "Email/query",
            { "filter": { "inMailbox": inbox, "hasAttachment": true } },
            "q"
        ]]),
    )
    .await;
    let id = result(&q, "q")["ids"][0].as_str().unwrap().to_string();
    let g = jmap(
        h,
        json!([[
            "Email/get",
            { "ids": [id], "properties": ["attachments"] },
            "g"
        ]]),
    )
    .await;
    result(&g, "g")["list"][0]["attachments"][0]["blobId"]
        .as_str()
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn compose_with_existing_part_blob_yields_multipart_carrying_part() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();
    let att_blob = invoice_attachment_blob(&h).await;

    // Compose a forward-style draft referencing the stored PDF part's blobId.
    let set = jmap(
        &h,
        json!([[
            "Email/set",
            {
                "create": {
                    "draft1": {
                        "to": [{ "email": "boss@example.org" }],
                        "subject": "Fwd: Invoice 2026",
                        "textBody": [{ "partId": "t", "type": "text/plain" }],
                        "bodyValues": { "t": { "value": "forwarding the invoice" } },
                        "attachments": [{
                            "blobId": att_blob,
                            "type": "application/pdf",
                            "name": "invoice.pdf"
                        }]
                    }
                }
            },
            "s"
        ]]),
    )
    .await;

    let created = &result(&set, "s")["created"]["draft1"];
    assert!(
        result(&set, "s").get("notCreated").is_none(),
        "no notCreated expected: {:?}",
        result(&set, "s")
    );
    let blob_id = created["blobId"].as_str().expect("draft blobId");

    // Download the composed draft's whole-message bytes and re-parse them.
    let whole = h
        .engine
        .fetch_blob(&h.account_id, blob_id)
        .await
        .unwrap()
        .expect("draft blob resolves");
    let parsed = mw_mime::parse(&whole.bytes).expect("re-parse composed draft");
    assert_eq!(parsed.email.subject.as_deref(), Some("Fwd: Invoice 2026"));

    let att = parsed
        .email
        .attachments
        .iter()
        .find(|p| p.name.as_deref() == Some("invoice.pdf"))
        .expect("attachment part carried on the composed draft");
    assert_eq!(att.r#type.as_deref(), Some("application/pdf"));
    let part_id: u32 = att.part_id.as_deref().unwrap().parse().unwrap();
    let part = mw_mime::part_blob(&whole.bytes, part_id).expect("decode carried part");
    assert_eq!(part.content_type, "application/pdf");
    assert_eq!(part.filename.as_deref(), Some("invoice.pdf"));
    // The PDF bytes round-trip through the base64 the stored part decoded to.
    assert_eq!(part.bytes, b"%PDF-1.4\n");
}

#[tokio::test]
async fn compose_with_unknown_blob_is_clean_not_created() {
    let h = setup().await;
    h.engine.resync(&h.account_id).await.unwrap();

    let set = jmap(
        &h,
        json!([[
            "Email/set",
            {
                "create": {
                    "draft1": {
                        "to": [{ "email": "boss@example.org" }],
                        "subject": "broken forward",
                        "textBody": [{ "partId": "t", "type": "text/plain" }],
                        "bodyValues": { "t": { "value": "oops" } },
                        "attachments": [{ "blobId": "0000deadbeef.9" }]
                    }
                }
            },
            "s"
        ]]),
    )
    .await;

    let resp = result(&set, "s");
    // No panic; the create is rejected with a set-error, nothing created.
    assert!(
        resp["created"]
            .as_object()
            .map(|m| m.is_empty())
            .unwrap_or(true),
        "nothing should be created: {resp:?}"
    );
    let not_created = resp["notCreated"].as_object().expect("notCreated present");
    let err = &not_created["draft1"];
    assert_eq!(err["type"].as_str(), Some("serverFail"));
    assert!(
        err["description"]
            .as_str()
            .unwrap_or_default()
            .contains("0000deadbeef.9"),
        "error names the unresolved blobId: {err:?}"
    );
}
