//! 26.20 t28-e12: what a recipient receives must not name the blind-copied, and
//! a submission's `envelope` decides who it is sent to.
//!
//! `transmit_draft` took `RCPT TO` from the stored message's To, Cc and Bcc and
//! handed the submitter the stored bytes unchanged. `mw_mime::build` writes a
//! `Bcc:` header into those bytes, and nothing removed it, so every recipient of
//! a message with a Bcc could read the Bcc list in the message they received.
//! `EmailSubmission.envelope` was not read at all.
//!
//! These tests drive the real dispatch (`handle_jmap` → `EmailSubmission/set` →
//! the account submitter) and assert on what the submitter was handed: the
//! envelope, and the bytes that would follow `DATA`.

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

/// A message that did not come through `Email/set`: its `Bcc` field is folded
/// over three lines, a second one uses the obsolete `Bcc :` spelling in another
/// case, there is a `Resent-Bcc`, and the body has a line that begins `Bcc:`.
const HOSTILE: &[u8] = b"From: me@example.org\r\n\
To: visible@example.org\r\n\
Bcc:\r\n hidden-one@example.org,\r\n\thidden-two@example.org\r\n\
Subject: folded\r\n\
bCC : hidden-three@example.org\r\n\
Resent-Bcc: hidden-four@example.org\r\n\
X-After: kept\r\n\
Message-ID: <hostile@example.org>\r\n\
Date: Mon, 05 Oct 2026 10:00:00 +0000\r\n\
\r\n\
Bcc: this line is body text\r\n\
second body line\r\n";

/// An IMAP-shaped backend. INBOX holds one message, [`HOSTILE`]; APPENDs are
/// accepted and their bytes kept by folder.
#[derive(Default)]
struct FakeBackend {
    appended: Mutex<HashMap<String, Vec<Vec<u8>>>>,
}

#[async_trait]
impl AccountBackend for FakeBackend {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps {
            uidplus: true,
            special_use: true,
            ..BackendCaps::default()
        })
    }

    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        Ok([
            ("INBOX", MailboxRole::Inbox, 1),
            ("Sent", MailboxRole::Sent, 0),
            ("Drafts", MailboxRole::Drafts, 0),
        ]
        .into_iter()
        .map(|(name, role, total)| RawMailbox {
            mailbox_ref: RawMailboxRef {
                name: name.to_string(),
                uidvalidity: UIDVALIDITY,
            },
            role,
            parent: None,
            uidnext: total + 1,
            highestmodseq: 0,
            total,
            unread: 0,
        })
        .collect())
    }

    async fn sync_mailbox(
        &self,
        mbox: &RawMailboxRef,
        cursor: &SyncCursor,
    ) -> Result<MailboxDelta> {
        let seen = match cursor {
            SyncCursor::UidWindow { uidnext, .. } => *uidnext,
            _ => 1,
        };
        let added = if mbox.name == "INBOX" && seen <= 1 {
            vec![MessageRef::Imap {
                mailbox: mbox.clone(),
                uidvalidity: UIDVALIDITY,
                uid: 1,
            }]
        } else {
            Vec::new()
        };
        Ok(MailboxDelta {
            added,
            flag_changes: Vec::new(),
            removed: Vec::new(),
            next_cursor: SyncCursor::UidWindow {
                uidvalidity: UIDVALIDITY,
                uidnext: 2,
            },
        })
    }

    async fn fetch_raw(&self, refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
        Ok(refs
            .iter()
            .map(|r| RawMessage {
                message_ref: r.clone(),
                raw: HOSTILE.to_vec(),
                flags: vec![Flag::Seen],
                internaldate: Some("2026-10-05T10:00:00Z".into()),
            })
            .collect())
    }

    async fn store_flags(&self, _r: &[MessageRef], _a: &[Flag], _rm: &[Flag]) -> Result<()> {
        Ok(())
    }

    async fn move_messages(&self, _r: &[MessageRef], _to: &RawMailboxRef) -> Result<MoveOutcome> {
        Err(EngineError::Unsupported("move".into()))
    }

    async fn append(&self, mbox: &RawMailboxRef, raw: &[u8], _f: &[Flag]) -> Result<MessageRef> {
        let mut appended = self.appended.lock().unwrap();
        let folder = appended.entry(mbox.name.clone()).or_default();
        folder.push(raw.to_vec());
        Ok(MessageRef::Imap {
            mailbox: mbox.clone(),
            uidvalidity: UIDVALIDITY,
            uid: 100 + folder.len() as u32,
        })
    }

    async fn watch(&self, _sink: ChangeSink) -> Result<WatchHandle> {
        let (tx, _rx) = tokio::sync::watch::channel(false);
        Ok(WatchHandle::new(tx))
    }
}

/// Records every message it is handed and accepts all of its recipients.
#[derive(Default)]
struct RecordingSubmitter {
    handed: Mutex<Vec<Outgoing>>,
}

impl RecordingSubmitter {
    fn handed(&self) -> Vec<Outgoing> {
        self.handed.lock().unwrap().clone()
    }

    /// The one message handed over so far.
    fn only(&self) -> Outgoing {
        let handed = self.handed();
        assert_eq!(handed.len(), 1, "exactly one message handed to SMTP");
        handed.into_iter().next().unwrap()
    }
}

#[async_trait]
impl MailSubmitter for RecordingSubmitter {
    async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult> {
        let accepted = msg.rcpt_to.clone();
        self.handed.lock().unwrap().push(msg);
        Ok(SubmissionResult {
            accepted,
            rejected: Vec::new(),
        })
    }
}

struct Harness {
    engine: Arc<Engine>,
    account_id: String,
    backend: Arc<FakeBackend>,
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
    let backend = Arc::new(FakeBackend::default());
    let submitter = Arc::new(RecordingSubmitter::default());
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            backend.clone() as Arc<dyn AccountBackend>,
            submitter.clone() as Arc<dyn MailSubmitter>,
            "me@example.org",
        ),
    );
    engine.resync(&account_id).await.unwrap();
    Harness {
        engine,
        account_id,
        backend,
        submitter,
    }
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

/// Create a draft from `extra` merged over a To-only message; returns its id.
async fn create_draft(h: &Harness, extra: Value) -> String {
    let mut spec = json!({
        "from": [{ "email": "me@example.org" }],
        "to": [{ "email": "visible@example.org" }],
        "subject": "Quarterly numbers",
        "bodyValues": { "1": { "value": "the body\r\nBcc: this line is body text\r\n" } },
        "textBody": [{ "partId": "1", "type": "text/plain" }]
    });
    for (k, v) in extra.as_object().unwrap() {
        spec[k] = v.clone();
    }
    let resp = jmap(
        h,
        json!([["Email/set", { "create": { "draft": spec } }, "c"]]),
    )
    .await;
    result(&resp, "c")["created"]["draft"]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("draft created: {resp}"))
        .to_string()
}

/// Submit `email_id` with `spec` merged in; returns the `EmailSubmission/set` result.
async fn submit(h: &Harness, email_id: &str, spec: Value) -> Value {
    let mut create = json!({ "emailId": email_id });
    for (k, v) in spec.as_object().unwrap() {
        create[k] = v.clone();
    }
    let resp = jmap(
        h,
        json!([["EmailSubmission/set", { "create": { "send": create } }, "s"]]),
    )
    .await;
    result(&resp, "s").clone()
}

/// The header section of a message: everything before the first empty line,
/// unfolded into one string per field.
fn header_fields(raw: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(raw);
    let head = text
        .split_once("\r\n\r\n")
        .map_or(text.as_ref(), |(head, _)| head);
    let mut fields: Vec<String> = Vec::new();
    for line in head.split("\r\n") {
        if line.starts_with([' ', '\t']) && !fields.is_empty() {
            fields.last_mut().unwrap().push_str(line);
        } else {
            fields.push(line.to_string());
        }
    }
    fields
}

fn body_of(raw: &[u8]) -> String {
    let text = String::from_utf8_lossy(raw);
    text.split_once("\r\n\r\n")
        .map(|(_, body)| body.to_string())
        .unwrap_or_default()
}

/// The names of a message's header fields, lower-cased.
fn field_names(raw: &[u8]) -> Vec<String> {
    header_fields(raw)
        .iter()
        .filter_map(|f| {
            f.split_once(':')
                .map(|(name, _)| name.trim().to_ascii_lowercase())
        })
        .collect()
}

fn sorted(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v
}

/// Control: a message with no Bcc is handed over with its headers as built.
#[tokio::test]
async fn a_message_without_bcc_is_sent_with_its_headers_intact() {
    let h = setup().await;
    let draft = create_draft(&h, json!({ "cc": [{ "email": "copied@example.org" }] })).await;
    let set = submit(&h, &draft, json!({})).await;
    assert_eq!(set["created"]["send"]["undoStatus"], "final", "{set}");

    let sent = h.submitter.only();
    assert_eq!(
        sorted(sent.rcpt_to.clone()),
        ["copied@example.org", "visible@example.org"]
    );
    let names = field_names(&sent.raw);
    for expected in ["from", "to", "cc", "subject", "message-id", "date"] {
        assert!(
            names.contains(&expected.to_string()),
            "{expected} in {names:?}"
        );
    }
    // Byte for byte what was filed: nothing was removed from it.
    let filed = h.backend.appended.lock().unwrap()["Sent"][0].clone();
    assert_eq!(sent.raw, filed);
}

/// The defect: a Bcc recipient is in `RCPT TO`, and the bytes after `DATA` do
/// not name them. The copy filed into Sent still does.
#[tokio::test]
async fn a_bcc_recipient_gets_the_message_and_is_not_named_in_it() {
    let h = setup().await;
    let draft = create_draft(
        &h,
        json!({ "bcc": [
            { "name": "Hidden Reader", "email": "hidden@example.org" },
            { "email": "second-hidden@example.org" }
        ] }),
    )
    .await;
    // Precondition: the stored draft does carry the field, so there is
    // something to leak.
    let stored = h.backend.appended.lock().unwrap()["Drafts"][0].clone();
    assert!(
        field_names(&stored).contains(&"bcc".to_string()),
        "precondition: the built message has a Bcc header"
    );

    let set = submit(&h, &draft, json!({})).await;
    assert_eq!(set["created"]["send"]["undoStatus"], "final", "{set}");

    let sent = h.submitter.only();
    assert_eq!(
        sorted(sent.rcpt_to.clone()),
        [
            "hidden@example.org",
            "second-hidden@example.org",
            "visible@example.org"
        ],
        "the blind-copied are recipients"
    );
    let fields = header_fields(&sent.raw);
    assert!(
        !field_names(&sent.raw).contains(&"bcc".to_string()),
        "no Bcc field in what is sent: {fields:?}"
    );
    let head = fields.join("\n");
    assert!(!head.contains("hidden@example.org"), "{head}");
    assert!(!head.contains("Hidden Reader"), "{head}");
    // Everything else is as it was built, and the body is untouched.
    assert_eq!(
        field_names(&sent.raw),
        field_names(&stored)
            .into_iter()
            .filter(|n| n != "bcc")
            .collect::<Vec<_>>()
    );
    assert_eq!(body_of(&sent.raw), body_of(&stored));
    assert!(body_of(&sent.raw).contains("Bcc: this line is body text"));

    // The sender's own record keeps the field.
    let filed = h.backend.appended.lock().unwrap()["Sent"][0].clone();
    assert_eq!(filed, stored, "the Sent copy is the message as composed");
    assert!(field_names(&filed).contains(&"bcc".to_string()));
}

/// A stored message that did not come from the engine's own builder: folded,
/// oddly spelled and repeated Bcc fields all go, whole; the fields around
/// them and the body stay byte for byte.
#[tokio::test]
async fn a_folded_and_oddly_spelled_bcc_is_removed_whole() {
    let h = setup().await;
    let boxes = jmap(&h, json!([["Mailbox/get", { "ids": null }, "m"]])).await;
    let inbox = result(&boxes, "m")["list"]
        .as_array()
        .unwrap()
        .iter()
        .find(|mb| mb["role"] == "inbox")
        .and_then(|mb| mb["id"].as_str())
        .unwrap()
        .to_string();
    let q = jmap(
        &h,
        json!([["Email/query", { "filter": { "inMailbox": inbox } }, "q"]]),
    )
    .await;
    let hostile = result(&q, "q")["ids"][0]
        .as_str()
        .unwrap_or_else(|| panic!("the synced message: {q}"))
        .to_string();

    let set = submit(
        &h,
        &hostile,
        json!({ "envelope": { "rcptTo": [{ "email": "visible@example.org" }] } }),
    )
    .await;
    assert_eq!(set["created"]["send"]["undoStatus"], "final", "{set}");

    let sent = h.submitter.only();
    let expected: &[u8] = b"From: me@example.org\r\n\
To: visible@example.org\r\n\
Subject: folded\r\n\
X-After: kept\r\n\
Message-ID: <hostile@example.org>\r\n\
Date: Mon, 05 Oct 2026 10:00:00 +0000\r\n\
\r\n\
Bcc: this line is body text\r\n\
second body line\r\n";
    assert_eq!(
        String::from_utf8_lossy(&sent.raw),
        String::from_utf8_lossy(expected)
    );
    for hidden in ["hidden-one", "hidden-two", "hidden-three", "hidden-four"] {
        assert!(
            !String::from_utf8_lossy(&sent.raw).contains(hidden),
            "{hidden} is not in what is sent"
        );
    }
}

/// `envelope.rcptTo` is who the message goes to — exactly those, whatever the
/// message's own To/Cc/Bcc say.
#[tokio::test]
async fn envelope_rcpt_to_decides_the_recipients() {
    let h = setup().await;
    let draft = create_draft(&h, json!({ "bcc": [{ "email": "hidden@example.org" }] })).await;
    let set = submit(
        &h,
        &draft,
        json!({ "envelope": {
            "mailFrom": { "email": "me@example.org" },
            "rcptTo": [{ "email": "only@example.org" }, { "email": "also@example.org" }]
        } }),
    )
    .await;
    assert_eq!(set["created"]["send"]["undoStatus"], "final", "{set}");
    let sent = h.submitter.only();
    assert_eq!(sent.rcpt_to, ["only@example.org", "also@example.org"]);
    assert_eq!(sent.mail_from, "me@example.org");
    assert!(!field_names(&sent.raw).contains(&"bcc".to_string()));
}

/// The envelope is kept with the submission: a send that happens later — from
/// the dispatcher, or at a release — uses it, and strips Bcc, like an inline one.
#[tokio::test]
async fn a_delayed_and_a_released_send_use_the_envelope_and_strip_bcc() {
    let h = setup().await;
    let envelope = json!({ "rcptTo": [{ "email": "only@example.org" }] });

    let delayed = create_draft(&h, json!({ "bcc": [{ "email": "hidden@example.org" }] })).await;
    submit(
        &h,
        &delayed,
        json!({ "mailwomanHoldSeconds": 30, "envelope": envelope }),
    )
    .await;
    assert!(
        h.submitter.handed().is_empty(),
        "precondition: not sent yet"
    );
    h.engine
        .dispatch_tick_at(chrono::Utc::now() + chrono::Duration::days(1))
        .await
        .unwrap();

    let held = create_draft(&h, json!({ "bcc": [{ "email": "hidden@example.org" }] })).await;
    let set = submit(
        &h,
        &held,
        json!({ "mailwomanHold": "manual", "envelope": envelope }),
    )
    .await;
    let sub_id = set["created"]["send"]["id"].as_str().unwrap().to_string();
    // The release also replaces the onSuccess instructions; the recipients stay.
    let sent_box = {
        let m = jmap(&h, json!([["Mailbox/get", { "ids": null }, "m"]])).await;
        result(&m, "m")["list"]
            .as_array()
            .unwrap()
            .iter()
            .find(|mb| mb["role"] == "sent")
            .and_then(|mb| mb["id"].as_str())
            .unwrap()
            .to_string()
    };
    let released = jmap(
        &h,
        json!([["EmailSubmission/set", {
            "update": { &sub_id: { "sendAt": null, "mailwomanHoldSeconds": 0 } },
            "onSuccessUpdateEmail": { &sub_id: { "mailboxIds": { sent_box: true }, "keywords/$draft": null } },
        }, "s"]]),
    )
    .await;
    assert_eq!(
        result(&released, "s")["updated"][&sub_id]["undoStatus"],
        "final",
        "{released}"
    );

    let handed = h.submitter.handed();
    assert_eq!(
        handed.len(),
        2,
        "one from the dispatcher, one from the release"
    );
    for sent in &handed {
        assert_eq!(sent.rcpt_to, ["only@example.org"]);
        assert!(!field_names(&sent.raw).contains(&"bcc".to_string()));
        assert!(!String::from_utf8_lossy(&sent.raw).contains("hidden@example.org"));
    }
}

/// An envelope the engine will not act on as written is refused with the
/// create; nothing is sent.
#[tokio::test]
async fn an_envelope_that_cannot_be_honoured_refuses_the_create() {
    let h = setup().await;
    let draft = create_draft(&h, json!({})).await;
    for (envelope, context) in [
        (json!("visible@example.org"), "not an object"),
        (json!({}), "no rcptTo"),
        (json!({ "rcptTo": [] }), "an empty rcptTo"),
        (
            json!({ "rcptTo": [{ "name": "x" }] }),
            "an entry without an email",
        ),
        (
            json!({ "rcptTo": [{ "email": "not-an-address" }] }),
            "a recipient with no @",
        ),
        (
            json!({ "rcptTo": [{ "email": "a@example.org>\r\nRCPT TO:<victim@example.org" }] }),
            "a recipient carrying a second command",
        ),
        (
            json!({ "mailFrom": { "email": "someone-else@example.org" },
                    "rcptTo": [{ "email": "visible@example.org" }] }),
            "a sender that is not the message's",
        ),
        (
            json!({ "mailFrom": { "email": "" },
                    "rcptTo": [{ "email": "visible@example.org" }] }),
            "the null sender",
        ),
    ] {
        let set = submit(&h, &draft, json!({ "envelope": envelope })).await;
        assert_eq!(
            set["notCreated"]["send"]["properties"],
            json!(["envelope"]),
            "{context}: {set}"
        );
        assert!(set["created"].get("send").is_none(), "{context}");
    }
    assert!(h.submitter.handed().is_empty(), "nothing was sent");

    // Control: the same draft with a sound envelope is sent.
    let set = submit(
        &h,
        &draft,
        json!({ "envelope": { "mailFrom": { "email": "ME@example.org" },
                              "rcptTo": [{ "email": "visible@example.org" }] } }),
    )
    .await;
    assert_eq!(set["created"]["send"]["undoStatus"], "final", "{set}");
    assert_eq!(h.submitter.only().mail_from, "me@example.org");
}
