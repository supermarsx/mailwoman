//! 26.20 t28-e21 (f35) — iTIP guards and ingest.
//!
//! Before 26.20 `CalendarEvent/import` threw the iTIP METHOD away and created a
//! new event for every component it read: importing a file twice stored every
//! event twice, and a CANCEL was stored as one more (cancelled) event instead of
//! being refused. Nothing read an invitation that arrived by mail.
//!
//! Now:
//! - a plain import never applies a CANCEL, REPLY or COUNTER, and skips an event
//!   whose UID the calendar already holds (no duplicate, nothing overwritten);
//! - `CalendarEvent/itip {emailId, apply?}` reads the `text/calendar` part of a
//!   stored message and says what applying it would do; only `apply: true`
//!   changes anything, and only when the message's sender is the stored
//!   ORGANIZER (REQUEST, CANCEL) or a stored ATTENDEE (REPLY) and its SEQUENCE
//!   and DTSTAMP are not older than what was applied before. The sender is the
//!   message's `From`, read by the server; the client sends only the email id.
//! - nothing is sent by the ingest.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::{Value, json};

use mw_engine::account::AccountRuntime;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, EngineError, Flag, MailboxDelta, MessageRef,
    MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result, SyncCursor, WatchHandle,
};
use mw_engine::{Engine, MailSubmitter};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{
    AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store,
};

const ME: &str = "me@example.org";
const ORGANIZER: &str = "org@example.com";

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

/// Records every submission, so a test can say the ingest sent nothing.
#[derive(Default)]
struct Recorder(Mutex<Vec<Outgoing>>);

#[async_trait]
impl MailSubmitter for Recorder {
    async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult> {
        let accepted = msg.rcpt_to.clone();
        self.0.lock().unwrap().push(msg);
        Ok(SubmissionResult {
            accepted,
            rejected: Vec::new(),
        })
    }
}

struct Harness {
    engine: Arc<Engine>,
    account_id: String,
    inbox: String,
    sent: Arc<Recorder>,
    next_uid: Mutex<u32>,
}

async fn account(store: &Store, user: &str) -> String {
    store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example.org",
                port: 993,
                tls: "implicit",
                username: user,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: user.into(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap()
}

async fn inbox_of(store: &Store, account_id: &str) -> String {
    store
        .upsert_mailbox(&MailboxUpsert {
            account_id,
            name: "INBOX",
            role: Some("inbox"),
            uidvalidity: 1,
            uidnext: 1,
            highestmodseq: 0,
            total: 0,
            unread: 0,
            parent_id: None,
        })
        .await
        .unwrap()
}

async fn setup() -> Harness {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account_id = account(&store, ME).await;
    let inbox = inbox_of(&store, &account_id).await;
    let engine = Arc::new(Engine::new(store));
    let sent = Arc::new(Recorder::default());
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            Arc::new(NoopBackend) as Arc<dyn AccountBackend>,
            sent.clone() as Arc<dyn MailSubmitter>,
            ME,
        ),
    );
    Harness {
        engine,
        account_id,
        inbox,
        sent,
        next_uid: Mutex::new(1),
    }
}

impl Harness {
    async fn call(&self, method: &str, args: Value) -> Value {
        let req = json!({ "methodCalls": [[method, args, "c0"]] });
        let resp = self.engine.handle_jmap(&self.account_id, &req).await;
        resp["methodResponses"][0][1].clone()
    }

    /// Store a received message from `from` and return its email id.
    async fn deliver(&self, from: &str, raw: &str) -> String {
        let uid = {
            let mut n = self.next_uid.lock().unwrap();
            *n += 1;
            *n
        };
        let store = self.engine.store();
        let blob = store
            .put_body(&self.account_id, raw.as_bytes())
            .await
            .unwrap();
        let mid = format!("m{uid}@{}", from.split('@').nth(1).unwrap_or("x"));
        store
            .upsert_message(&MessageUpsert {
                account_id: &self.account_id,
                mailbox_id: &self.inbox,
                uid,
                uidvalidity: 1,
                message_id: Some(&mid),
                thread_id: None,
                internaldate: Some("2026-10-25T09:00:00Z"),
                size: raw.len() as u64,
                flags_json: "[]",
                envelope: Some(br#"{"subject":"Invitation"}"#),
                blob_ref: Some(&blob),
            })
            .await
            .unwrap()
    }

    /// Deliver an iTIP message from `from` and run `CalendarEvent/itip` on it.
    async fn itip(&self, from: &str, ics: &str, apply: bool) -> Value {
        let email_id = self.deliver(from, &mail(from, ics)).await;
        self.call(
            "CalendarEvent/itip",
            json!({ "emailId": email_id, "apply": apply }),
        )
        .await
    }

    /// Every stored event as its projection.
    async fn events(&self) -> Vec<Value> {
        let got = self.call("CalendarEvent/get", json!({})).await;
        got["list"].as_array().cloned().unwrap_or_default()
    }

    async fn only_event(&self) -> Value {
        let events = self.events().await;
        assert_eq!(events.len(), 1, "expected one event: {events:#?}");
        events[0].clone()
    }
}

/// An iTIP calendar for the meeting `mtg-1@example.com`.
fn itip(method: &str, seq: u32, dtstamp: &str, title: &str, extra: &str) -> String {
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Test//EN\r\nMETHOD:{method}\r\n\
         BEGIN:VEVENT\r\nUID:mtg-1@example.com\r\nDTSTAMP:{dtstamp}\r\n\
         DTSTART:20261110T100000Z\r\nDURATION:PT1H\r\nSUMMARY:{title}\r\nSEQUENCE:{seq}\r\n\
         ORGANIZER;CN=Org:mailto:{ORGANIZER}\r\n\
         ATTENDEE;PARTSTAT=ACCEPTED:mailto:{ORGANIZER}\r\n\
         {extra}END:VEVENT\r\nEND:VCALENDAR\r\n"
    )
}

fn request(seq: u32, dtstamp: &str, title: &str) -> String {
    itip(
        "REQUEST",
        seq,
        dtstamp,
        title,
        &format!(
            "ATTENDEE;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:{ME}\r\n\
             ATTENDEE;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:bob@example.net\r\n"
        ),
    )
}

fn cancel(seq: u32, dtstamp: &str) -> String {
    itip(
        "CANCEL",
        seq,
        dtstamp,
        "Planning",
        &format!("ATTENDEE:mailto:{ME}\r\nSTATUS:CANCELLED\r\n"),
    )
}

/// A multipart message from `from` carrying `ics` as its `text/calendar` part.
fn mail(from: &str, ics: &str) -> String {
    format!(
        "From: Sender <{from}>\r\nTo: {ME}\r\nSubject: Invitation\r\n\
         Message-ID: <{}@example.com>\r\nDate: Sun, 25 Oct 2026 09:00:00 +0000\r\n\
         MIME-Version: 1.0\r\nContent-Type: multipart/alternative; boundary=\"b\"\r\n\r\n\
         --b\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nYou are invited.\r\n\
         --b\r\nContent-Type: text/calendar; method=REQUEST; charset=utf-8\r\n\r\n{ics}\r\n\
         --b--\r\n",
        ics.len()
    )
}

fn status_of(ev: &Value) -> &str {
    ev["status"].as_str().unwrap_or("")
}

fn partstat(ev: &Value, who: &str) -> String {
    ev["participants"]
        .as_object()
        .and_then(|m| {
            m.iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(who))
                .map(|(_, p)| p["participationStatus"].as_str().unwrap_or("").to_string())
        })
        .unwrap_or_default()
}

// ── plain import ─────────────────────────────────────────────────────────────

const PLAIN_ICS: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Test//EN\r\n\
    BEGIN:VEVENT\r\nUID:plain-1@example.com\r\nDTSTAMP:20261020T090000Z\r\n\
    DTSTART:20261112T100000Z\r\nDURATION:PT1H\r\nSUMMARY:Dentist\r\nEND:VEVENT\r\n\
    END:VCALENDAR\r\n";

#[tokio::test]
async fn importing_the_same_file_twice_stores_each_event_once() {
    let h = setup().await;
    let first = h
        .call("CalendarEvent/import", json!({ "blob": PLAIN_ICS }))
        .await;
    assert_eq!(first["count"], 1, "{first}");
    let second = h
        .call("CalendarEvent/import", json!({ "blob": PLAIN_ICS }))
        .await;
    assert_eq!(second["count"], 0, "{second}");
    assert_eq!(second["skipped"], 1, "{second}");
    assert_eq!(h.events().await.len(), 1);
}

#[tokio::test]
async fn an_import_never_applies_a_cancel() {
    let h = setup().await;
    // Precondition: the invitation is in the calendar.
    let created = h
        .call(
            "CalendarEvent/import",
            json!({ "blob": request(1, "20261020T090000Z", "Planning") }),
        )
        .await;
    assert_eq!(created["count"], 1, "{created}");

    let resp = h
        .call(
            "CalendarEvent/import",
            json!({ "blob": cancel(2, "20261021T090000Z") }),
        )
        .await;
    assert_eq!(resp["type"], "invalidArguments", "{resp}");
    assert_eq!(resp["code"], "itipNotImportable", "{resp}");
    let ev = h.only_event().await;
    assert_ne!(status_of(&ev), "cancelled");
}

#[tokio::test]
async fn an_import_of_a_reply_is_refused() {
    let h = setup().await;
    let reply = itip(
        "REPLY",
        1,
        "20261021T090000Z",
        "Planning",
        "ATTENDEE;PARTSTAT=ACCEPTED:mailto:bob@example.net\r\n",
    );
    let resp = h
        .call("CalendarEvent/import", json!({ "blob": reply }))
        .await;
    assert_eq!(resp["code"], "itipNotImportable", "{resp}");
    assert!(h.events().await.is_empty());
}

#[tokio::test]
async fn parse_reports_the_method() {
    let h = setup().await;
    let resp = h
        .call(
            "CalendarEvent/parse",
            json!({ "blob": cancel(2, "20261021T090000Z") }),
        )
        .await;
    assert_eq!(resp["method"], "CANCEL", "{resp}");
    let plain = h
        .call("CalendarEvent/parse", json!({ "blob": PLAIN_ICS }))
        .await;
    assert_eq!(plain["method"], Value::Null, "{plain}");
}

// ── ingest from a message ────────────────────────────────────────────────────

#[tokio::test]
async fn a_request_is_only_previewed_until_the_user_applies_it() {
    let h = setup().await;
    let preview = h
        .itip(
            ORGANIZER,
            &request(1, "20261020T090000Z", "Planning"),
            false,
        )
        .await;
    assert_eq!(preview["method"], "REQUEST", "{preview}");
    assert_eq!(preview["outcome"], "new", "{preview}");
    assert_eq!(preview["sender"], ORGANIZER, "{preview}");
    assert_eq!(preview["applied"], false, "{preview}");
    assert_eq!(preview["event"]["title"], "Planning", "{preview}");
    assert!(h.events().await.is_empty(), "a preview stored an event");

    let applied = h
        .itip(ORGANIZER, &request(1, "20261020T090000Z", "Planning"), true)
        .await;
    assert_eq!(applied["outcome"], "new", "{applied}");
    assert_eq!(applied["applied"], true, "{applied}");
    let ev = h.only_event().await;
    assert_eq!(applied["eventId"], ev["id"]);
    assert_eq!(ev["title"], "Planning");
    assert_eq!(partstat(&ev, ME), "needs-action");
    assert!(
        h.sent.0.lock().unwrap().is_empty(),
        "the ingest sent a message"
    );
}

#[tokio::test]
async fn a_cancel_from_someone_other_than_the_organizer_leaves_the_event() {
    let h = setup().await;
    h.itip(ORGANIZER, &request(1, "20261020T090000Z", "Planning"), true)
        .await;

    let forged = h
        .itip("mallory@example.net", &cancel(2, "20261021T090000Z"), true)
        .await;
    assert_eq!(forged["outcome"], "notOrganizer", "{forged}");
    assert_eq!(forged["applied"], false, "{forged}");
    assert_ne!(status_of(&h.only_event().await), "cancelled");

    // Precondition the other way: the same CANCEL from the organizer cancels it.
    let real = h
        .itip(ORGANIZER, &cancel(2, "20261021T090000Z"), true)
        .await;
    assert_eq!(real["outcome"], "cancel", "{real}");
    assert_eq!(real["applied"], true, "{real}");
    assert_eq!(status_of(&h.only_event().await), "cancelled");
}

#[tokio::test]
async fn the_organizer_is_matched_without_regard_to_case() {
    let h = setup().await;
    h.itip(ORGANIZER, &request(1, "20261020T090000Z", "Planning"), true)
        .await;
    let real = h
        .itip("Org@Example.COM", &cancel(2, "20261021T090000Z"), true)
        .await;
    assert_eq!(real["outcome"], "cancel", "{real}");
}

#[tokio::test]
async fn a_replayed_request_with_a_lower_sequence_is_ignored() {
    let h = setup().await;
    h.itip(ORGANIZER, &request(1, "20261020T090000Z", "Planning"), true)
        .await;
    let moved = h
        .itip(
            ORGANIZER,
            &request(3, "20261022T090000Z", "Planning (moved)"),
            true,
        )
        .await;
    assert_eq!(moved["outcome"], "update", "{moved}");
    assert_eq!(h.only_event().await["title"], "Planning (moved)");

    let replay = h
        .itip(ORGANIZER, &request(2, "20261021T090000Z", "Planning"), true)
        .await;
    assert_eq!(replay["outcome"], "stale", "{replay}");
    assert_eq!(replay["applied"], false, "{replay}");
    assert_eq!(h.only_event().await["title"], "Planning (moved)");
}

#[tokio::test]
async fn a_request_with_the_same_sequence_and_an_older_dtstamp_is_ignored() {
    let h = setup().await;
    h.itip(ORGANIZER, &request(2, "20261022T090000Z", "Newer"), true)
        .await;
    let older = h
        .itip(ORGANIZER, &request(2, "20261021T090000Z", "Older"), true)
        .await;
    assert_eq!(older["outcome"], "stale", "{older}");
    assert_eq!(h.only_event().await["title"], "Newer");
    // The same message again is already applied, not stale.
    let same = h
        .itip(ORGANIZER, &request(2, "20261022T090000Z", "Newer"), false)
        .await;
    assert_eq!(same["outcome"], "current", "{same}");
}

#[tokio::test]
async fn an_update_from_someone_other_than_the_organizer_is_refused() {
    let h = setup().await;
    h.itip(ORGANIZER, &request(1, "20261020T090000Z", "Planning"), true)
        .await;
    let forged = h
        .itip(
            "mallory@example.net",
            &request(5, "20261023T090000Z", "Moved by Mallory"),
            true,
        )
        .await;
    assert_eq!(forged["outcome"], "notOrganizer", "{forged}");
    assert_eq!(h.only_event().await["title"], "Planning");
}

#[tokio::test]
async fn applying_an_invitation_twice_does_not_duplicate_it() {
    let h = setup().await;
    h.itip(ORGANIZER, &request(1, "20261020T090000Z", "Planning"), true)
        .await;
    let again = h
        .itip(ORGANIZER, &request(1, "20261020T090000Z", "Planning"), true)
        .await;
    assert_eq!(again["outcome"], "current", "{again}");
    assert_eq!(h.events().await.len(), 1);
}

#[tokio::test]
async fn a_reply_from_an_attendee_records_their_answer_and_no_one_elses() {
    let h = setup().await;
    // The account organises; bob is invited.
    let mine = format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Test//EN\r\nBEGIN:VEVENT\r\n\
         UID:mtg-1@example.com\r\nDTSTAMP:20261020T090000Z\r\nDTSTART:20261110T100000Z\r\n\
         DURATION:PT1H\r\nSUMMARY:Planning\r\nSEQUENCE:1\r\nORGANIZER:mailto:{ME}\r\n\
         ATTENDEE;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:bob@example.net\r\n\
         END:VEVENT\r\nEND:VCALENDAR\r\n"
    );
    let created = h
        .call("CalendarEvent/import", json!({ "blob": mine }))
        .await;
    assert_eq!(created["count"], 1, "{created}");

    let reply = |who: &str, stat: &str| {
        format!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Test//EN\r\nMETHOD:REPLY\r\n\
             BEGIN:VEVENT\r\nUID:mtg-1@example.com\r\nDTSTAMP:20261021T090000Z\r\n\
             DTSTART:20261110T100000Z\r\nSEQUENCE:1\r\nORGANIZER:mailto:{ME}\r\n\
             ATTENDEE;PARTSTAT={stat}:mailto:{who}\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
        )
    };
    let forged = h
        .itip(
            "mallory@example.net",
            &reply("mallory@example.net", "ACCEPTED"),
            true,
        )
        .await;
    assert_eq!(forged["outcome"], "notAttendee", "{forged}");
    // Someone else's answer in bob's name, sent from mallory's address.
    let spoofed = h
        .itip(
            "mallory@example.net",
            &reply("bob@example.net", "DECLINED"),
            true,
        )
        .await;
    assert_eq!(spoofed["outcome"], "notAttendee", "{spoofed}");
    assert_eq!(
        partstat(&h.only_event().await, "bob@example.net"),
        "needs-action"
    );

    let real = h
        .itip(
            "bob@example.net",
            &reply("bob@example.net", "ACCEPTED"),
            true,
        )
        .await;
    assert_eq!(real["outcome"], "reply", "{real}");
    assert_eq!(real["applied"], true, "{real}");
    let ev = h.only_event().await;
    assert_eq!(partstat(&ev, "bob@example.net"), "accepted");
    assert_eq!(ev["title"], "Planning");
}

#[tokio::test]
async fn a_cancel_for_an_event_not_in_the_calendar_changes_nothing() {
    let h = setup().await;
    let resp = h
        .itip(ORGANIZER, &cancel(2, "20261021T090000Z"), true)
        .await;
    assert_eq!(resp["outcome"], "unknownEvent", "{resp}");
    assert_eq!(resp["applied"], false, "{resp}");
    assert!(h.events().await.is_empty());
}

#[tokio::test]
async fn a_message_without_a_calendar_part_or_of_another_account_is_refused() {
    let h = setup().await;
    let plain = "From: a@example.com\r\nTo: me@example.org\r\nSubject: hi\r\n\r\nhello\r\n";
    let email_id = h.deliver("a@example.com", plain).await;
    let resp = h
        .call("CalendarEvent/itip", json!({ "emailId": email_id }))
        .await;
    assert_eq!(resp["type"], "invalidArguments", "{resp}");

    // A message stored for another account does not resolve from this one.
    let store = h.engine.store();
    let other = account(store, "other@example.org").await;
    let other_inbox = inbox_of(store, &other).await;
    let raw = mail(ORGANIZER, &request(1, "20261020T090000Z", "Planning"));
    let blob = store.put_body(&other, raw.as_bytes()).await.unwrap();
    let foreign = store
        .upsert_message(&MessageUpsert {
            account_id: &other,
            mailbox_id: &other_inbox,
            uid: 1,
            uidvalidity: 1,
            message_id: Some("foreign@example.com"),
            thread_id: None,
            internaldate: Some("2026-10-25T09:00:00Z"),
            size: raw.len() as u64,
            flags_json: "[]",
            envelope: None,
            blob_ref: Some(&blob),
        })
        .await
        .unwrap();
    let resp = h
        .call(
            "CalendarEvent/itip",
            json!({ "emailId": foreign, "apply": true }),
        )
        .await;
    assert_eq!(resp["type"], "notFound", "{resp}");
    assert!(h.events().await.is_empty());
}
