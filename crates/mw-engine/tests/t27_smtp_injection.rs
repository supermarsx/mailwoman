//! 26.20 t27-e2 (SEC-2): an address is validated before it becomes SMTP.
//!
//! `MAIL FROM:<…>` and `RCPT TO:<…>` are built by interpolation in `mw-smtp`,
//! and the iTIP message headers by interpolation in `pim/events.rs`. Before
//! this change a participant key, a draft recipient or a draft sender that
//! contained CR/LF, `>` or a space went onto the SMTP control channel as
//! written, and an event title or a draft subject containing CR/LF added
//! header lines to the message.
//!
//! Every test here drives `Engine::handle_jmap` with the real
//! `mw_smtp::Submitter` as the account's submitter, pointed at an in-process
//! SMTP server that records each line it receives and answers `250` to
//! anything — so an injected command would be accepted and would show up in
//! the record. "Nothing was sent" is asserted by sending a benign control
//! message through the same path afterwards and requiring the server's whole
//! record to be that one session: the server takes connections one at a time
//! in arrival order, so by the time the control session has finished, anything
//! an earlier call wrote has been recorded.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use mw_engine::account::AccountRuntime;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, EngineError, Flag, MailboxDelta, MailboxRole,
    MessageRef, MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result, SyncCursor,
    WatchHandle,
};
use mw_engine::{Engine, MailSubmitter};
use mw_smtp::{Outgoing, Security, SubmissionResult, SubmitConfig, Submitter};
use mw_store::{
    AccountKind, Credentials, EventRow, MessageUpsert, NewAccount, ServerKey, Store, SubmissionRow,
};

const ME: &str = "me@example.org";
const UIDVALIDITY: u32 = 100;

/// The injection the audit describes: close the path, end the line, add a
/// recipient.
const CRLF_RCPT: &str = "x@example.test>\r\nRCPT TO:<victim@example.test";

/// Addresses that must never reach `RCPT TO` / `MAIL FROM`.
const HOSTILE: &[&str] = &[
    CRLF_RCPT,
    "x@example.test\nRCPT TO:<victim@example.test>",
    "x@example.test\rRCPT TO:<victim@example.test>",
    "x@example.\0test",
    "x @example.test",
    "victim@example.test> NOTIFY=NEVER",
    "a@b>",
];

// ── a recording SMTP server ──────────────────────────────────────────────────

#[derive(Default)]
struct Wire {
    connections: usize,
    /// Every line received, commands and message lines alike, CRLF stripped.
    lines: Vec<String>,
}

impl Wire {
    fn starting(&self, prefix: &str) -> Vec<&str> {
        self.lines
            .iter()
            .map(String::as_str)
            .filter(|l| l.starts_with(prefix))
            .collect()
    }

    fn count(&self, line: &str) -> usize {
        self.lines.iter().filter(|l| *l == line).count()
    }
}

async fn read_line(sock: &mut TcpStream, buf: &mut Vec<u8>) -> Option<String> {
    loop {
        if let Some(pos) = buf.iter().position(|&b| b == b'\n') {
            let mut line: Vec<u8> = buf.drain(..=pos).collect();
            line.pop();
            if line.last() == Some(&b'\r') {
                line.pop();
            }
            return Some(String::from_utf8_lossy(&line).into_owned());
        }
        let mut tmp = [0u8; 1024];
        let n = sock.read(&mut tmp).await.ok()?;
        if n == 0 {
            return None;
        }
        buf.extend_from_slice(&tmp[..n]);
    }
}

/// One SMTP session. Every command is answered positively, including ones a
/// real server would not expect, so nothing an injection adds is hidden by a
/// refusal.
async fn session(mut sock: TcpStream, wire: &Mutex<Wire>) {
    let mut buf = Vec::new();
    if sock.write_all(b"220 recorder ESMTP\r\n").await.is_err() {
        return;
    }
    while let Some(line) = read_line(&mut sock, &mut buf).await {
        wire.lock().unwrap().lines.push(line.clone());
        let reply: &[u8] = match line.to_ascii_uppercase().as_str() {
            "DATA" => b"354 go ahead\r\n",
            "QUIT" => b"221 bye\r\n",
            _ => b"250 OK\r\n",
        };
        if sock.write_all(reply).await.is_err() {
            return;
        }
        if line.eq_ignore_ascii_case("QUIT") {
            return;
        }
        if line.eq_ignore_ascii_case("DATA") {
            loop {
                let Some(l) = read_line(&mut sock, &mut buf).await else {
                    return;
                };
                let done = l == ".";
                wire.lock().unwrap().lines.push(l);
                if done {
                    break;
                }
            }
            if sock.write_all(b"250 queued\r\n").await.is_err() {
                return;
            }
        }
    }
}

/// Start the server. Connections are served strictly one after another.
async fn start_recorder() -> (std::net::SocketAddr, Arc<Mutex<Wire>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let wire = Arc::new(Mutex::new(Wire::default()));
    let w = wire.clone();
    tokio::spawn(async move {
        while let Ok((sock, _)) = listener.accept().await {
            w.lock().unwrap().connections += 1;
            session(sock, &w).await;
        }
    });
    (addr, wire)
}

// ── engine harness ───────────────────────────────────────────────────────────

/// Three role folders and an `APPEND` that always succeeds: enough for
/// `Email/set` to create a draft and for a send to file its Sent copy.
struct FolderBackend {
    appended: Mutex<u32>,
}

#[async_trait]
impl AccountBackend for FolderBackend {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps {
            uidplus: true,
            special_use: true,
            ..BackendCaps::default()
        })
    }
    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        Ok([
            ("INBOX", MailboxRole::Inbox),
            ("Sent", MailboxRole::Sent),
            ("Drafts", MailboxRole::Drafts),
        ]
        .into_iter()
        .map(|(name, role)| RawMailbox {
            mailbox_ref: RawMailboxRef {
                name: name.to_string(),
                uidvalidity: UIDVALIDITY,
            },
            role,
            parent: None,
            uidnext: 1,
            highestmodseq: 0,
            total: 0,
            unread: 0,
        })
        .collect())
    }
    async fn sync_mailbox(&self, _m: &RawMailboxRef, _c: &SyncCursor) -> Result<MailboxDelta> {
        Ok(MailboxDelta {
            added: Vec::new(),
            flag_changes: Vec::new(),
            removed: Vec::new(),
            next_cursor: SyncCursor::UidWindow {
                uidvalidity: UIDVALIDITY,
                uidnext: 1,
            },
        })
    }
    async fn fetch_raw(&self, _refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
        Ok(Vec::new())
    }
    async fn store_flags(&self, _r: &[MessageRef], _a: &[Flag], _d: &[Flag]) -> Result<()> {
        Ok(())
    }
    async fn move_messages(&self, _r: &[MessageRef], _to: &RawMailboxRef) -> Result<MoveOutcome> {
        Err(EngineError::Unsupported("move".into()))
    }
    async fn append(&self, mbox: &RawMailboxRef, _raw: &[u8], _f: &[Flag]) -> Result<MessageRef> {
        let mut n = self.appended.lock().unwrap();
        *n += 1;
        Ok(MessageRef::Imap {
            mailbox: mbox.clone(),
            uidvalidity: UIDVALIDITY,
            uid: *n,
        })
    }
    async fn watch(&self, _sink: ChangeSink) -> Result<WatchHandle> {
        let (tx, _rx) = tokio::sync::watch::channel(false);
        Ok(WatchHandle::new(tx))
    }
}

struct Harness {
    engine: Arc<Engine>,
    account_id: String,
    wire: Arc<Mutex<Wire>>,
}

async fn setup() -> Harness {
    setup_with(None).await
}

/// `submitter: None` is the production `mw_smtp::Submitter` pointed at the
/// recorder. `Some` replaces it, for the tests that need a submitter which
/// does no checking of its own.
async fn setup_with(submitter: Option<Arc<dyn MailSubmitter>>) -> Harness {
    let (addr, wire) = start_recorder().await;
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account_id = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example.org",
                port: 993,
                tls: "implicit",
                username: ME,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: ME.into(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap();
    let engine = Arc::new(Engine::new(store));
    // By default the production submitter, not a fake: the property under test
    // is what it writes to a socket.
    let submitter = submitter.unwrap_or_else(|| {
        Arc::new(Submitter::new(SubmitConfig {
            host: addr.ip().to_string(),
            port: addr.port(),
            security: Security::Plaintext,
            credentials: mw_smtp::Credentials::None,
            ehlo_name: "client.test".to_string(),
        }))
    });
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            Arc::new(FolderBackend {
                appended: Mutex::new(0),
            }) as Arc<dyn AccountBackend>,
            submitter,
            ME,
        ),
    );
    engine.resync(&account_id).await.unwrap();
    Harness {
        engine,
        account_id,
        wire,
    }
}

impl Harness {
    /// One JMAP request. Bounded, so a client left waiting on a server it has
    /// desynchronised fails the test instead of hanging it.
    async fn jmap(&self, calls: Value) -> Value {
        tokio::time::timeout(
            Duration::from_secs(30),
            self.engine
                .handle_jmap(&self.account_id, &json!({ "methodCalls": calls })),
        )
        .await
        .expect("the JMAP call returned")
    }

    async fn call(&self, method: &str, args: Value) -> Value {
        let resp = self.jmap(json!([[method, args, "c"]])).await;
        result(&resp, "c").clone()
    }

    /// Everything the server has recorded since the last call, cleared.
    fn take_wire(&self) -> Wire {
        std::mem::take(&mut *self.wire.lock().unwrap())
    }

    /// Send the control invitation and require that it is the only thing the
    /// server has seen since the wire was last taken.
    async fn assert_nothing_was_sent_but_the_control(&self, context: &str) {
        let set = self
            .call(
                "CalendarEvent/set",
                json!({ "create": { "control": event_with("Control", "control@example.test") } }),
            )
            .await;
        assert!(
            set["created"]["control"]["id"].is_string(),
            "{context}: control event created: {set}"
        );
        let wire = self.take_wire();
        assert_eq!(
            wire.connections, 1,
            "{context}: one connection, the control's: {:?}",
            wire.lines
        );
        assert_eq!(
            wire.starting("MAIL FROM:"),
            vec!["MAIL FROM:<me@example.org>"],
            "{context}"
        );
        assert_eq!(
            wire.starting("RCPT TO:"),
            vec!["RCPT TO:<control@example.test>"],
            "{context}"
        );
        assert_eq!(wire.count("DATA"), 1, "{context}");
        assert!(
            !wire
                .lines
                .iter()
                .any(|l| l.contains("victim") || l.contains("X-Injected")),
            "{context}: {:?}",
            wire.lines
        );
    }

    async fn event_ids(&self) -> Vec<String> {
        let q = self.call("CalendarEvent/get", json!({})).await;
        q["list"]
            .as_array()
            .unwrap_or_else(|| panic!("CalendarEvent/get list: {q}"))
            .iter()
            .map(|e| e["id"].as_str().unwrap().to_string())
            .collect()
    }

    async fn email_ids(&self) -> Vec<String> {
        let q = self.call("Email/query", json!({})).await;
        q["ids"]
            .as_array()
            .unwrap_or_else(|| panic!("Email/query ids: {q}"))
            .iter()
            .map(|e| e.as_str().unwrap().to_string())
            .collect()
    }
}

fn result<'a>(resp: &'a Value, call_id: &str) -> &'a Value {
    resp["methodResponses"]
        .as_array()
        .unwrap_or_else(|| panic!("methodResponses: {resp}"))
        .iter()
        .find(|r| r[2] == call_id)
        .map(|r| &r[1])
        .unwrap_or(&Value::Null)
}

/// An event organised by us with one attendee who is asked to reply, which is
/// what makes `CalendarEvent/set` send an iTIP REQUEST.
fn event_with(title: &str, attendee: &str) -> Value {
    let mut participants = serde_json::Map::new();
    participants.insert(
        ME.to_string(),
        json!({ "email": ME, "role": "organizer", "participationStatus": "accepted" }),
    );
    participants.insert(
        attendee.to_string(),
        json!({ "email": attendee, "role": "attendee",
                "participationStatus": "needs-action", "expectReply": true }),
    );
    json!({
        "title": title,
        "start": "2026-07-20T15:00:00",
        "timeZone": "UTC",
        "duration": "PT1H",
        "participants": participants,
    })
}

fn draft(overrides: Value) -> Value {
    let mut spec = json!({
        "from": [{ "email": ME }],
        "to": [{ "email": "friend@example.org" }],
        "subject": "Invoice",
        "bodyValues": { "1": { "value": "please pay once" } },
        "textBody": [{ "partId": "1", "type": "text/plain" }],
    });
    for (k, v) in overrides.as_object().unwrap() {
        spec[k] = v.clone();
    }
    spec
}

/// `Email/set` create + an immediate `EmailSubmission/set` for it.
fn create_and_send(spec: Value) -> Value {
    json!([
        ["Email/set", { "create": { "draft": spec } }, "c1"],
        ["EmailSubmission/set", { "create": { "s1": {
            "emailId": "#draft", "mailwomanHoldSeconds": 0
        } } }, "c2"]
    ])
}

fn assert_invalid_properties(set: &Value, bucket: &str, key: &str, property: &str, context: &str) {
    let err = &set[bucket][key];
    assert_eq!(
        err["type"], "invalidProperties",
        "{context}: expected {bucket}.{key} = invalidProperties, got {set}"
    );
    assert_eq!(err["properties"], json!([property]), "{context}: {set}");
    let text = err["description"].as_str().unwrap_or_default();
    assert!(
        !text.chars().any(char::is_control),
        "{context}: the error text carries a raw control character: {text:?}"
    );
}

/// The lines of the one message in `wire`, headers only.
fn header_lines(wire: &Wire) -> Vec<&str> {
    let start = wire
        .lines
        .iter()
        .position(|l| l == "DATA")
        .expect("a DATA command");
    wire.lines[start + 1..]
        .iter()
        .map(String::as_str)
        .take_while(|l| !l.is_empty())
        .collect()
}

/// Replace the `to` of a stored message's envelope, which is what
/// `transmit_draft` reads the recipients from. This models a row written
/// before `Email/set` create validated addresses.
async fn rewrite_stored_recipient(store: &Store, id: &str, to: &str) {
    let msg = store.get_message(id).await.unwrap();
    let mut env: Value =
        serde_json::from_slice(&store.get_envelope(id).await.unwrap().unwrap()).unwrap();
    env["to"] = json!([{ "email": to }]);
    let env = serde_json::to_vec(&env).unwrap();
    let same = store
        .upsert_message(&MessageUpsert {
            account_id: &msg.account_id,
            mailbox_id: &msg.mailbox_id,
            uid: msg.uid,
            uidvalidity: msg.uidvalidity,
            message_id: msg.message_id.as_deref(),
            thread_id: msg.thread_id.as_deref(),
            internaldate: msg.internaldate.as_deref(),
            size: msg.size,
            flags_json: &msg.flags_json,
            envelope: Some(&env),
            blob_ref: msg.blob_ref.as_deref(),
        })
        .await
        .unwrap();
    assert_eq!(same, id, "the rewrite updated the message, not a new row");
}

// ── iTIP: the control ────────────────────────────────────────────────────────

/// The harness reaches the sink: a benign invitation produces `RCPT TO` and a
/// message on the recorded wire.
#[tokio::test]
async fn control_a_benign_invitation_reaches_the_wire() {
    let h = setup().await;
    let set = h
        .call(
            "CalendarEvent/set",
            json!({ "create": { "e": event_with("Review", "bob@example.test") } }),
        )
        .await;
    assert!(set["created"]["e"]["id"].is_string(), "{set}");

    let wire = h.take_wire();
    assert_eq!(wire.connections, 1);
    assert_eq!(
        wire.starting("MAIL FROM:"),
        vec!["MAIL FROM:<me@example.org>"]
    );
    assert_eq!(
        wire.starting("RCPT TO:"),
        vec!["RCPT TO:<bob@example.test>"]
    );
    assert_eq!(wire.count("DATA"), 1);
    let headers = header_lines(&wire);
    assert!(headers.contains(&"Subject: REQUEST: Review"), "{headers:?}");
    assert!(headers.contains(&"To: bob@example.test"), "{headers:?}");
    assert!(headers.contains(&"From: me@example.org"), "{headers:?}");
}

// ── iTIP: participant keys ───────────────────────────────────────────────────

/// D3 (a): a create whose participant key is not an address is rejected —
/// nothing stored, nothing sent.
#[tokio::test]
async fn a_hostile_participant_key_is_rejected_on_create_and_nothing_is_sent() {
    let h = setup().await;
    for bad in HOSTILE {
        let set = h
            .call(
                "CalendarEvent/set",
                json!({ "create": { "e": event_with("Review", bad) } }),
            )
            .await;
        assert!(
            set["created"].get("e").is_none(),
            "{bad:?} was created: {set}"
        );
        assert_invalid_properties(&set, "notCreated", "e", "participants", bad);
        assert!(
            h.event_ids().await.is_empty(),
            "{bad:?}: the event was stored"
        );
        h.assert_nothing_was_sent_but_the_control(bad).await;

        // Remove the control event so the next round starts from no events.
        let ids = h.event_ids().await;
        h.call("CalendarEvent/set", json!({ "destroy": ids })).await;
        h.take_wire();
    }
}

/// The same key is rejected when it arrives in an update, and the stored
/// event keeps its participants.
#[tokio::test]
async fn a_hostile_participant_key_is_rejected_on_update() {
    let h = setup().await;
    let set = h
        .call(
            "CalendarEvent/set",
            json!({ "create": { "e": event_with("Review", "bob@example.test") } }),
        )
        .await;
    let id = set["created"]["e"]["id"].as_str().unwrap().to_string();
    h.take_wire();

    let hostile = event_with("Review", CRLF_RCPT)["participants"].clone();
    let upd = h
        .call(
            "CalendarEvent/set",
            json!({ "update": { &id: { "participants": hostile } } }),
        )
        .await;
    assert!(upd["updated"].get(&id).is_none(), "{upd}");
    assert_invalid_properties(&upd, "notUpdated", &id, "participants", "update");

    let got = h.call("CalendarEvent/get", json!({ "ids": [id] })).await;
    let keys: Vec<&String> = got["list"][0]["participants"]
        .as_object()
        .unwrap()
        .keys()
        .collect();
    assert!(keys.iter().all(|k| !k.contains("victim")), "{keys:?}");
    assert!(keys.iter().any(|k| *k == "bob@example.test"), "{keys:?}");

    // Control for the update path: a valid participant map is accepted.
    let ok = event_with("Review", "carol@example.test")["participants"].clone();
    let upd = h
        .call(
            "CalendarEvent/set",
            json!({ "update": { &id: { "participants": ok } } }),
        )
        .await;
    assert!(upd["updated"].get(&id).is_some(), "{upd}");

    h.assert_nothing_was_sent_but_the_control("update").await;
}

/// An event stored before this change (or imported — import does not reject)
/// may hold a hostile organizer key. `CalendarEvent/respond` reads the
/// recipient from it; the send is refused.
#[tokio::test]
async fn a_stored_hostile_organizer_is_not_sent_to_on_respond() {
    let h = setup().await;

    // A benign invitation from an external organizer, to get a real row.
    let benign = |organizer: &str| {
        let mut participants = serde_json::Map::new();
        participants.insert(
            organizer.to_string(),
            json!({ "email": "boss@example.test", "role": "organizer",
                    "participationStatus": "accepted", "expectReply": false }),
        );
        participants.insert(
            ME.to_string(),
            json!({ "email": ME, "role": "attendee",
                    "participationStatus": "needs-action", "expectReply": true }),
        );
        json!({
            "title": "Review", "start": "2026-07-20T15:00:00", "timeZone": "UTC",
            "duration": "PT1H", "participants": participants,
        })
    };
    let set = h
        .call(
            "CalendarEvent/set",
            json!({ "create": { "inv": benign("boss@example.test") } }),
        )
        .await;
    let id = set["created"]["inv"]["id"].as_str().unwrap().to_string();

    // Control: responding to the benign invitation sends a REPLY to the organizer.
    let resp = h
        .call(
            "CalendarEvent/respond",
            json!({ "eventId": id, "action": "accept" }),
        )
        .await;
    assert_eq!(
        resp["updated"]["participants"][ME]["participationStatus"], "accepted",
        "{resp}"
    );
    let wire = h.take_wire();
    assert_eq!(
        wire.starting("RCPT TO:"),
        vec!["RCPT TO:<boss@example.test>"],
        "control: {:?}",
        wire.lines
    );

    // Rewrite the stored row so its organizer key is hostile, as a row written
    // before create-time validation existed could be.
    for bad in HOSTILE {
        let row = h.engine.store().get_event(&id).await.unwrap().unwrap();
        let mut stored = benign(bad);
        stored["id"] = json!(id);
        stored["uid"] = json!(row.uid);
        h.engine
            .store()
            .upsert_event(&EventRow {
                json: Some(serde_json::to_vec(&stored).unwrap()),
                ..row
            })
            .await
            .unwrap();

        h.call(
            "CalendarEvent/respond",
            json!({ "eventId": id, "action": "accept" }),
        )
        .await;
        h.assert_nothing_was_sent_but_the_control(bad).await;
        let ids: Vec<String> = h
            .event_ids()
            .await
            .into_iter()
            .filter(|e| *e != id)
            .collect();
        h.call("CalendarEvent/set", json!({ "destroy": ids })).await;
        h.take_wire();
    }
}

// ── iTIP: the Subject header ─────────────────────────────────────────────────

/// A title with a line break is still sent, as one `Subject:` line; the text
/// after the break does not become a header of its own.
#[tokio::test]
async fn a_title_with_a_line_break_does_not_add_a_header() {
    let h = setup().await;
    let set = h
        .call(
            "CalendarEvent/set",
            json!({ "create": { "e": event_with(
                "hi\r\nBcc: victim@example.test\r\nX-Injected: 1", "bob@example.test"
            ) } }),
        )
        .await;
    assert!(set["created"]["e"]["id"].is_string(), "{set}");

    let wire = h.take_wire();
    assert_eq!(
        wire.starting("RCPT TO:"),
        vec!["RCPT TO:<bob@example.test>"]
    );
    let headers = header_lines(&wire);
    assert!(
        !headers
            .iter()
            .any(|l| l.starts_with("Bcc:") || l.starts_with("X-Injected:")),
        "{headers:?}"
    );
    assert_eq!(
        headers
            .iter()
            .filter(|l| l.starts_with("Subject:"))
            .collect::<Vec<_>>(),
        vec![&"Subject: REQUEST: hiBcc: victim@example.testX-Injected: 1"],
        "{headers:?}"
    );
    // The header block still ends where it should: the calendar body follows.
    assert_eq!(
        headers.len(),
        7,
        "Message-ID, From, To, Subject, MIME-Version, Content-Type, CTE: {headers:?}"
    );
}

// ── ordinary mail: the envelope ──────────────────────────────────────────────

/// The harness reaches the sink on the ordinary path too.
#[tokio::test]
async fn control_a_benign_draft_is_transmitted() {
    let h = setup().await;
    let resp = h
        .jmap(create_and_send(draft(json!({
            "cc": [{ "email": "cc@example.org" }],
            "bcc": [{ "email": "bcc@example.org" }],
        }))))
        .await;
    assert_eq!(
        result(&resp, "c2")["created"]["s1"]["undoStatus"],
        "final",
        "{resp}"
    );
    let wire = h.take_wire();
    assert_eq!(wire.connections, 1);
    assert_eq!(
        wire.starting("MAIL FROM:"),
        vec!["MAIL FROM:<me@example.org>"]
    );
    assert_eq!(
        wire.starting("RCPT TO:"),
        vec![
            "RCPT TO:<friend@example.org>",
            "RCPT TO:<cc@example.org>",
            "RCPT TO:<bcc@example.org>"
        ]
    );
    assert_eq!(wire.count("DATA"), 1);
}

/// A hostile address in any address field fails the `Email/set` create, so
/// there is no draft to submit and nothing is sent.
#[tokio::test]
async fn a_hostile_draft_address_is_rejected_at_create_and_nothing_is_sent() {
    let h = setup().await;
    for field in ["to", "cc", "bcc", "from", "replyTo"] {
        for bad in HOSTILE {
            let context = format!("{field} = {bad:?}");
            let resp = h
                .jmap(create_and_send(draft(json!({ field: [{ "email": bad }] }))))
                .await;
            let set = result(&resp, "c1");
            assert!(set["created"].get("draft").is_none(), "{context}: {resp}");
            assert_invalid_properties(set, "notCreated", "draft", field, &context);
            assert!(
                result(&resp, "c2")["created"].get("s1").is_none(),
                "{context}: a submission was created: {resp}"
            );
            assert!(
                h.email_ids().await.is_empty(),
                "{context}: a draft was stored"
            );
            h.assert_nothing_was_sent_but_the_control(&context).await;
        }
    }
}

/// The header form of the same input: the address that would have closed the
/// `To:` path and opened a new header line never reaches a stored message.
#[tokio::test]
async fn an_address_carrying_a_header_line_is_rejected_at_create() {
    let h = setup().await;
    let set = h
        .call(
            "Email/set",
            json!({ "create": { "draft": draft(json!({
                "to": [{ "email": "a@example.test>\r\nX-Injected: 1\r\n<b@example.test" }]
            })) } }),
        )
        .await;
    assert_invalid_properties(&set, "notCreated", "draft", "to", "to with a header line");
    assert!(h.email_ids().await.is_empty(), "a draft was stored");
}

/// A draft stored before this change can hold a hostile recipient. Submitting
/// it sends nothing, and the failure is final rather than retried.
#[tokio::test]
async fn a_stored_hostile_draft_is_not_transmitted_and_is_not_retried() {
    let h = setup().await;
    let set = h
        .call(
            "Email/set",
            json!({ "create": { "draft": draft(json!({})) } }),
        )
        .await;
    let id = set["created"]["draft"]["id"].as_str().unwrap().to_string();

    rewrite_stored_recipient(h.engine.store(), &id, CRLF_RCPT).await;
    let store = h.engine.store();

    // Inline send.
    let sub = h
        .call(
            "EmailSubmission/set",
            json!({ "create": { "s1": { "emailId": id, "mailwomanHoldSeconds": 0 } } }),
        )
        .await;
    assert!(sub["created"].get("s1").is_none(), "{sub}");
    assert!(sub["notCreated"].get("s1").is_some(), "{sub}");
    h.assert_nothing_was_sent_but_the_control("stored draft, inline")
        .await;

    // Deferred send: one dispatcher pass is enough to give up, because a
    // malformed address does not become valid on a retry.
    store
        .insert_submission(&SubmissionRow {
            id: "sub-t27".into(),
            account_id: h.account_id.clone(),
            email_id: id.clone(),
            identity_id: None,
            send_at: None,
            undo_status: "pending".into(),
            hold_seconds: 10,
            created_at: "2000-01-01T00:00:00Z".into(),
        })
        .await
        .unwrap();
    h.engine.dispatch_tick().await.unwrap();
    let row = store.get_submission("sub-t27").await.unwrap().unwrap();
    assert_eq!(row.undo_status, "failed", "given up after one pass");
    h.assert_nothing_was_sent_but_the_control("stored draft, dispatcher")
        .await;
}

// ── ordinary mail: header text ───────────────────────────────────────────────

/// `subject` is written as an unstructured header. A CRLF in it must not end
/// the header; the message is still sent.
#[tokio::test]
async fn a_subject_with_a_line_break_does_not_add_a_header() {
    let h = setup().await;
    let resp = h
        .jmap(create_and_send(draft(json!({
            "subject": "hi\r\nBcc: victim@example.test\r\nX-Injected: 1",
            "from": [{ "name": "Me\r\nX-Injected: 2", "email": ME }],
        }))))
        .await;
    assert_eq!(
        result(&resp, "c2")["created"]["s1"]["undoStatus"],
        "final",
        "{resp}"
    );
    let wire = h.take_wire();
    assert_eq!(
        wire.starting("RCPT TO:"),
        vec!["RCPT TO:<friend@example.org>"]
    );
    let headers = header_lines(&wire);
    assert!(
        !headers
            .iter()
            .any(|l| l.starts_with("Bcc:") || l.starts_with("X-Injected:")),
        "{headers:?}"
    );
    assert!(
        headers.contains(&"Subject: hiBcc: victim@example.testX-Injected: 1"),
        "{headers:?}"
    );
}

/// `inReplyTo` and `references` are written between `<` and `>` as given, and
/// an attachment's declared type is written as the `Content-Type` value.
#[tokio::test]
async fn a_line_break_in_a_message_id_or_content_type_is_rejected_at_create() {
    let h = setup().await;
    // A stored message to attach, so the attachment case gets as far as the
    // declared type.
    let set = h
        .call(
            "Email/set",
            json!({ "create": { "draft": draft(json!({})) } }),
        )
        .await;
    let blob = set["created"]["draft"]["blobId"]
        .as_str()
        .unwrap_or_else(|| panic!("{set}"))
        .to_string();

    // Control: the same three fields with ordinary values are accepted.
    let ok = h
        .call(
            "Email/set",
            json!({ "create": { "draft": draft(json!({
                "inReplyTo": "<parent@example.org>",
                "references": ["<root@example.org>", "<parent@example.org>"],
                "attachments": [{ "blobId": blob, "type": "message/rfc822", "name": "fwd.eml" }],
            })) } }),
        )
        .await;
    assert!(ok["created"]["draft"]["id"].is_string(), "{ok}");

    for (field, spec) in [
        (
            "inReplyTo",
            json!({ "inReplyTo": "<parent@example.org>\r\nX-Injected: 1" }),
        ),
        (
            "references",
            json!({ "references": ["<root@example.org>", "x>\r\nX-Injected: 1\r\n<y"] }),
        ),
        (
            "attachments",
            json!({ "attachments": [{
                "blobId": blob, "type": "text/plain\r\nX-Injected: 1", "name": "a.txt"
            }] }),
        ),
    ] {
        let before = h.email_ids().await.len();
        let set = h
            .call("Email/set", json!({ "create": { "draft": draft(spec) } }))
            .await;
        assert_invalid_properties(&set, "notCreated", "draft", field, field);
        assert_eq!(
            h.email_ids().await.len(),
            before,
            "{field}: a draft was stored"
        );
    }
}

// ── the engine's own checks, without mw-smtp behind them ─────────────────────

/// A submitter that accepts whatever it is handed and records it. It stands
/// in for one that does not go through `mw_smtp::Submitter` and so gets none
/// of its validation — `mw-server`'s bridge submitter hands the message to a
/// plugin.
#[derive(Default)]
struct UncheckedSubmitter {
    handed: Mutex<Vec<Outgoing>>,
}

#[async_trait]
impl MailSubmitter for UncheckedSubmitter {
    async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult> {
        let accepted = msg.rcpt_to.clone();
        self.handed.lock().unwrap().push(msg);
        Ok(SubmissionResult {
            accepted,
            rejected: Vec::new(),
        })
    }
}

/// A stored hostile organizer and a stored hostile draft are refused by the
/// engine before the submitter is called at all.
#[tokio::test]
async fn stored_hostile_addresses_never_reach_a_submitter_that_does_not_check() {
    let unchecked = Arc::new(UncheckedSubmitter::default());
    let h = setup_with(Some(unchecked.clone() as Arc<dyn MailSubmitter>)).await;
    let store = h.engine.store();
    let invitation = |organizer: &str| {
        let mut participants = serde_json::Map::new();
        participants.insert(
            organizer.to_string(),
            json!({ "email": "boss@example.test", "role": "organizer",
                    "participationStatus": "accepted", "expectReply": false }),
        );
        participants.insert(
            ME.to_string(),
            json!({ "email": ME, "role": "attendee",
                    "participationStatus": "needs-action", "expectReply": true }),
        );
        json!({
            "title": "Review", "start": "2026-07-20T15:00:00", "timeZone": "UTC",
            "duration": "PT1H", "participants": participants,
        })
    };

    // Control, iTIP: a benign organizer is handed to this submitter.
    let set = h
        .call(
            "CalendarEvent/set",
            json!({ "create": { "inv": invitation("boss@example.test") } }),
        )
        .await;
    let event = set["created"]["inv"]["id"].as_str().unwrap().to_string();
    h.call(
        "CalendarEvent/respond",
        json!({ "eventId": event, "action": "accept" }),
    )
    .await;
    // Control, mail: so is a benign draft.
    let resp = h.jmap(create_and_send(draft(json!({})))).await;
    assert_eq!(
        result(&resp, "c2")["created"]["s1"]["undoStatus"],
        "final",
        "{resp}"
    );
    {
        let handed = unchecked.handed.lock().unwrap();
        let rcpts: Vec<&Vec<String>> = handed.iter().map(|m| &m.rcpt_to).collect();
        assert_eq!(
            rcpts,
            vec![
                &vec!["boss@example.test".to_string()],
                &vec!["friend@example.org".to_string()]
            ],
            "both controls reached the submitter"
        );
    }

    // iTIP: make the stored organizer key hostile, then respond.
    let row = store.get_event(&event).await.unwrap().unwrap();
    let mut stored = invitation(CRLF_RCPT);
    stored["id"] = json!(event);
    stored["uid"] = json!(row.uid);
    store
        .upsert_event(&EventRow {
            json: Some(serde_json::to_vec(&stored).unwrap()),
            ..row
        })
        .await
        .unwrap();
    h.call(
        "CalendarEvent/respond",
        json!({ "eventId": event, "action": "accept" }),
    )
    .await;

    // Mail: make a stored draft's recipient hostile, then submit it.
    let set = h
        .call(
            "Email/set",
            json!({ "create": { "draft": draft(json!({})) } }),
        )
        .await;
    let id = set["created"]["draft"]["id"].as_str().unwrap().to_string();
    rewrite_stored_recipient(store, &id, CRLF_RCPT).await;
    let sub = h
        .call(
            "EmailSubmission/set",
            json!({ "create": { "s1": { "emailId": id, "mailwomanHoldSeconds": 0 } } }),
        )
        .await;
    assert!(sub["notCreated"].get("s1").is_some(), "{sub}");

    assert_eq!(
        unchecked.handed.lock().unwrap().len(),
        2,
        "nothing beyond the two controls was handed to the submitter"
    );
}

// ── CalendarEvent/set: what the participant check covers ─────────────────────

/// The `email` inside a participant entry is checked as well as the key: it is
/// what `mw-ics` writes into `ATTENDEE` and what the stored map is keyed by
/// after the round trip.
#[tokio::test]
async fn a_hostile_participant_email_under_a_valid_key_is_rejected() {
    let h = setup().await;
    let mut spec = event_with("Review", "bob@example.test");
    spec["participants"]["bob@example.test"]["email"] = json!(CRLF_RCPT);
    let set = h
        .call("CalendarEvent/set", json!({ "create": { "e": spec } }))
        .await;
    assert_invalid_properties(&set, "notCreated", "e", "participants", "email field");
    assert!(h.event_ids().await.is_empty());
    h.assert_nothing_was_sent_but_the_control("email field")
        .await;
}

/// An event that already holds a key which is not an address (an imported
/// `invalid:nomail` attendee, say) stays editable: an update that sends the
/// same map back is accepted. Adding a new such key is not.
#[tokio::test]
async fn an_update_may_resend_a_stored_key_but_not_add_a_bad_one() {
    let h = setup().await;
    let set = h
        .call(
            "CalendarEvent/set",
            json!({ "create": { "e": {
                "title": "Imported", "start": "2026-07-20T15:00:00", "timeZone": "UTC",
                "duration": "PT1H",
            } } }),
        )
        .await;
    let id = set["created"]["e"]["id"].as_str().unwrap().to_string();
    let store = h.engine.store();
    let row = store.get_event(&id).await.unwrap().unwrap();
    let stored = json!({
        "id": id, "uid": row.uid,
        "title": "Imported", "start": "2026-07-20T15:00:00", "timeZone": "UTC",
        "duration": "PT1H",
        "participants": { "invalid:nomail": { "name": "Someone", "role": "attendee" } },
    });
    store
        .upsert_event(&EventRow {
            json: Some(serde_json::to_vec(&stored).unwrap()),
            ..row
        })
        .await
        .unwrap();

    let same = json!({ "invalid:nomail": { "name": "Someone", "role": "attendee" } });
    let upd = h
        .call(
            "CalendarEvent/set",
            json!({ "update": { &id: { "title": "Renamed", "participants": same } } }),
        )
        .await;
    assert!(upd["updated"].get(&id).is_some(), "{upd}");

    let added = json!({
        "invalid:nomail": { "name": "Someone", "role": "attendee" },
        "also not an address": { "name": "New", "role": "attendee" },
    });
    let upd = h
        .call(
            "CalendarEvent/set",
            json!({ "update": { &id: { "participants": added } } }),
        )
        .await;
    assert_invalid_properties(&upd, "notUpdated", &id, "participants", "new bad key");
}

// ── an update checks `email` under a stored key too (26.20 t27-f2) ──────────

/// The allowance for a stored key covers the key, not what the patch puts
/// under it. An update that gave a stored participant a hostile `email` used
/// to be accepted, and the participant was then left out of the stored event:
/// with the organizer gone, a later `CalendarEvent/respond` addressed its
/// REPLY to the user's own address.
#[tokio::test]
async fn an_update_cannot_put_a_hostile_email_under_a_stored_key() {
    let h = setup().await;
    let set = h
        .call(
            "CalendarEvent/set",
            json!({ "create": { "e": event_with("Review", "bob@example.test") } }),
        )
        .await;
    let id = set["created"]["e"]["id"].as_str().unwrap().to_string();
    h.take_wire();
    let before = h.call("CalendarEvent/get", json!({ "ids": [&id] })).await;
    let before = before["list"][0]["participants"].clone();
    assert_eq!(before.as_object().map(|m| m.len()), Some(2), "{before}");

    for key in [ME, "bob@example.test"] {
        for hostile in [
            "victim@example.test>\r\nRCPT TO:<v2@example.test",
            CRLF_RCPT,
            "not an address",
        ] {
            let mut participants = event_with("Review", "bob@example.test")["participants"].clone();
            participants[key]["email"] = json!(hostile);
            let upd = h
                .call(
                    "CalendarEvent/set",
                    json!({ "update": { &id: { "title": "Renamed", "participants": participants } } }),
                )
                .await;
            let context = format!("{key} <- {hostile:?}");
            assert!(upd["updated"].get(&id).is_none(), "{context}: {upd}");
            assert_invalid_properties(&upd, "notUpdated", &id, "participants", &context);

            // Nothing of the refused patch was stored, and nobody was dropped.
            let got = h.call("CalendarEvent/get", json!({ "ids": [&id] })).await;
            assert_eq!(got["list"][0]["participants"], before, "{context}");
            assert_eq!(got["list"][0]["title"], "Review", "{context}");
        }
    }

    // Control: the same map with its stored emails is accepted.
    let same = event_with("Review", "bob@example.test")["participants"].clone();
    let upd = h
        .call(
            "CalendarEvent/set",
            json!({ "update": { &id: { "title": "Renamed", "participants": same } } }),
        )
        .await;
    assert!(upd["updated"].get(&id).is_some(), "{upd}");

    h.assert_nothing_was_sent_but_the_control("email under a stored key")
        .await;
}

/// A stored entry whose `email` is not an address (an imported event) can be
/// sent back as it is, which keeps the event editable. Changing that `email`
/// to another value that is not an address is refused.
#[tokio::test]
async fn an_update_may_resend_a_stored_email_but_not_change_it_to_a_bad_one() {
    let h = setup().await;
    let set = h
        .call(
            "CalendarEvent/set",
            json!({ "create": { "e": {
                "title": "Imported", "start": "2026-07-20T15:00:00", "timeZone": "UTC",
                "duration": "PT1H",
            } } }),
        )
        .await;
    let id = set["created"]["e"]["id"].as_str().unwrap().to_string();
    let store = h.engine.store();
    let row = store.get_event(&id).await.unwrap().unwrap();
    let entry = json!({ "name": "Someone", "role": "attendee", "email": "no address here" });
    let stored = json!({
        "id": id, "uid": row.uid,
        "title": "Imported", "start": "2026-07-20T15:00:00", "timeZone": "UTC",
        "duration": "PT1H",
        "participants": { "invalid:nomail": entry },
    });
    store
        .upsert_event(&EventRow {
            json: Some(serde_json::to_vec(&stored).unwrap()),
            ..row
        })
        .await
        .unwrap();

    let mut changed = entry.clone();
    changed["email"] = json!("still no address");
    let upd = h
        .call(
            "CalendarEvent/set",
            json!({ "update": { &id: { "participants": { "invalid:nomail": changed } } } }),
        )
        .await;
    assert_invalid_properties(&upd, "notUpdated", &id, "participants", "changed email");

    let upd = h
        .call(
            "CalendarEvent/set",
            json!({ "update": { &id: {
                "title": "Renamed", "participants": { "invalid:nomail": entry },
            } } }),
        )
        .await;
    assert!(upd["updated"].get(&id).is_some(), "{upd}");
}
