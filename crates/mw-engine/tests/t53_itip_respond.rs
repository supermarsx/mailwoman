//! t53 — answering invitations from the reader's event card.
//!
//! `CalendarEvent/respond {emailId, action, …}` answers the REQUEST in a
//! received message: it stores the answer and sends the REPLY from the address
//! that was invited (an alias included), never bumps SEQUENCE, refuses an
//! out-of-date or forged update, and sends nothing when asked not to.
//! `CalendarEvent/itip` reads PUBLISH and plain `.ics` attachments (every event,
//! bounded) and adds them only on the user's action.

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

// ── t53 fixtures ─────────────────────────────────────────────────────────────

/// An alias of the account (an enabled masked address): an invitation sent to
/// it is the account's, and the REPLY must come from it.
const ALIAS: &str = "meet-x@example.org";

impl Harness {
    async fn add_alias(&self) {
        self.engine
            .store()
            .put_masked_email(&mw_store::MaskedEmailRow {
                id: "mask-1".into(),
                account_id: self.account_id.clone(),
                alias_addr: ALIAS.into(),
                target_desc: "meetings".into(),
                state: "enabled".into(),
                created_at: "2026-10-01T00:00:00Z".into(),
                last_used_at: None,
            })
            .await
            .unwrap();
    }

    async fn respond(&self, email_id: &str, extra: Value) -> Value {
        let mut args = json!({ "emailId": email_id });
        for (k, v) in extra.as_object().unwrap() {
            args[k] = v.clone();
        }
        self.call("CalendarEvent/respond", args).await
    }

    fn sent(&self) -> Vec<Outgoing> {
        self.sent.0.lock().unwrap().clone()
    }
}

fn invite(seq: u32, dtstamp: &str, title: &str, attendee: &str) -> String {
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Test//EN\r\nMETHOD:REQUEST\r\n\
         BEGIN:VEVENT\r\nUID:mtg-53@example.com\r\nDTSTAMP:{dtstamp}\r\n\
         DTSTART:20261110T100000Z\r\nDURATION:PT1H\r\nSUMMARY:{title}\r\nSEQUENCE:{seq}\r\n\
         ORGANIZER;CN=Org:mailto:{ORGANIZER}\r\n\
         ATTENDEE;PARTSTAT=ACCEPTED:mailto:{ORGANIZER}\r\n\
         ATTENDEE;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:{attendee}\r\n\
         ATTENDEE;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:bob@example.net\r\n\
         END:VEVENT\r\nEND:VCALENDAR\r\n"
    )
}

fn cancel_of(seq: u32, dtstamp: &str, attendee: &str) -> String {
    invite(seq, dtstamp, "Planning", attendee)
        .replace("METHOD:REQUEST", "METHOD:CANCEL")
        .replace("END:VEVENT", "STATUS:CANCELLED\r\nEND:VEVENT")
}

/// A message from `from` with `ics` as an attachment of type `ct`.
fn mail_attachment(from: &str, ics: &str, ct: &str, name: &str) -> String {
    format!(
        "From: Sender <{from}>\r\nTo: {ME}\r\nSubject: Events\r\n\
         Message-ID: <att{}@example.com>\r\nDate: Sun, 25 Oct 2026 09:00:00 +0000\r\n\
         MIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary=\"b\"\r\n\r\n\
         --b\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nSee attached.\r\n\
         --b\r\nContent-Type: {ct}; name=\"{name}\"\r\n\
         Content-Disposition: attachment; filename=\"{name}\"\r\n\r\n{ics}\r\n\
         --b--\r\n",
        ics.len()
    )
}

fn plain_file(method: Option<&str>, n: usize) -> String {
    let method = method
        .map(|m| format!("METHOD:{m}\r\n"))
        .unwrap_or_default();
    let mut body = format!("BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Test//EN\r\n{method}");
    for i in 1..=n {
        body.push_str(&format!(
            "BEGIN:VEVENT\r\nUID:pub-{i}@example.com\r\nDTSTAMP:20261020T090000Z\r\n\
             DTSTART:2026111{i}T090000Z\r\nDURATION:PT30M\r\nSUMMARY:Talk {i}\r\nEND:VEVENT\r\n"
        ));
    }
    body.push_str("BEGIN:VTODO\r\nUID:todo-1@example.com\r\nSUMMARY:Slides\r\nEND:VTODO\r\n");
    body.push_str("END:VCALENDAR\r\n");
    body
}

// ── answering an invitation from the message ─────────────────────────────────

#[tokio::test]
async fn accepting_a_new_invitation_adds_it_and_replies_from_the_invited_address() {
    let h = setup().await;
    h.add_alias().await;
    let email = h
        .deliver(
            ORGANIZER,
            &mail(ORGANIZER, &invite(1, "20261020T090000Z", "Planning", ALIAS)),
        )
        .await;
    let preview = h
        .call("CalendarEvent/itip", json!({ "emailId": email }))
        .await;
    assert_eq!(preview["outcome"], "new", "{preview}");
    assert_eq!(preview["me"], ALIAS, "{preview}");
    assert!(h.sent().is_empty() && h.events().await.is_empty());

    let resp = h
        .respond(
            &email,
            json!({ "action": "accept", "comment": "Thanks, see you" }),
        )
        .await;
    assert_eq!(resp["sent"], true, "{resp}");
    assert_eq!(resp["from"], ALIAS);
    assert_eq!(resp["to"], ORGANIZER);
    let ev = h.only_event().await;
    assert_eq!(partstat(&ev, ALIAS), "accepted");
    assert_eq!(ev["sequence"], 1, "an attendee never bumps SEQUENCE");
    assert_eq!(resp["eventId"], ev["id"]);

    let sent = h.sent();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].mail_from, ALIAS);
    assert_eq!(sent[0].rcpt_to, vec![ORGANIZER.to_string()]);
    let raw = String::from_utf8_lossy(&sent[0].raw).replace("\r\n ", "");
    assert!(raw.contains("METHOD:REPLY"), "{raw}");
    assert!(raw.contains(&format!("From: {ALIAS}")), "{raw}");
    assert!(
        raw.contains(&format!("PARTSTAT=ACCEPTED:mailto:{ALIAS}")),
        "{raw}"
    );
    assert!(
        !raw.contains("bob@example.net"),
        "only the replying attendee: {raw}"
    );
    assert!(raw.contains("COMMENT:Thanks\\, see you"), "{raw}");
    assert!(raw.contains("SEQUENCE:1"), "{raw}");

    // The card now sees the stored answer.
    let after = h
        .call("CalendarEvent/itip", json!({ "emailId": email }))
        .await;
    assert_eq!(after["outcome"], "current", "{after}");
    assert_eq!(
        after["stored"]["participationStatus"], "accepted",
        "{after}"
    );
}

#[tokio::test]
async fn an_answer_without_a_response_is_stored_and_nothing_is_sent() {
    let h = setup().await;
    let email = h
        .deliver(
            ORGANIZER,
            &mail(ORGANIZER, &invite(1, "20261020T090000Z", "Planning", ME)),
        )
        .await;
    let resp = h
        .respond(
            &email,
            json!({ "action": "tentative", "sendResponse": false }),
        )
        .await;
    assert_eq!(resp["sent"], false, "{resp}");
    assert!(h.sent().is_empty());
    assert_eq!(partstat(&h.only_event().await, ME), "tentative");
}

#[tokio::test]
async fn declining_a_new_invitation_adds_nothing_and_replies() {
    let h = setup().await;
    let email = h
        .deliver(
            ORGANIZER,
            &mail(ORGANIZER, &invite(1, "20261020T090000Z", "Planning", ME)),
        )
        .await;
    let resp = h.respond(&email, json!({ "action": "decline" })).await;
    assert_eq!(resp["sent"], true, "{resp}");
    assert!(resp["eventId"].is_null(), "{resp}");
    assert!(h.events().await.is_empty());
    let raw = String::from_utf8_lossy(&h.sent()[0].raw).into_owned();
    assert!(raw.contains("PARTSTAT=DECLINED"), "{raw}");
}

#[tokio::test]
async fn an_invitation_to_none_of_my_addresses_is_not_answered() {
    let h = setup().await;
    let email = h
        .deliver(
            ORGANIZER,
            &mail(
                ORGANIZER,
                &invite(1, "20261020T090000Z", "Planning", "someone@example.net"),
            ),
        )
        .await;
    let resp = h.respond(&email, json!({ "action": "accept" })).await;
    assert_eq!(resp["code"], "notInvited", "{resp}");
    assert!(h.sent().is_empty());
    assert!(h.events().await.is_empty());
}

#[tokio::test]
async fn a_cancel_after_accepting_is_applied_not_taken_for_stale() {
    let h = setup().await;
    let email = h
        .deliver(
            ORGANIZER,
            &mail(ORGANIZER, &invite(1, "20261020T090000Z", "Planning", ME)),
        )
        .await;
    h.respond(&email, json!({ "action": "accept" })).await;
    let cancel = h
        .itip(ORGANIZER, &cancel_of(1, "20261021T090000Z", ME), false)
        .await;
    assert_eq!(cancel["outcome"], "cancel", "{cancel}");
    let applied = h
        .itip(ORGANIZER, &cancel_of(1, "20261021T090000Z", ME), true)
        .await;
    assert_eq!(applied["applied"], true, "{applied}");
    assert_eq!(status_of(&h.only_event().await), "cancelled");
}

#[tokio::test]
async fn an_older_request_is_out_of_date_and_cannot_be_answered() {
    let h = setup().await;
    let newer = h
        .deliver(
            ORGANIZER,
            &mail(ORGANIZER, &invite(2, "20261022T090000Z", "Planning v2", ME)),
        )
        .await;
    h.respond(&newer, json!({ "action": "accept" })).await;
    let older = h
        .deliver(
            ORGANIZER,
            &mail(ORGANIZER, &invite(1, "20261020T090000Z", "Planning v1", ME)),
        )
        .await;
    let preview = h
        .call("CalendarEvent/itip", json!({ "emailId": older }))
        .await;
    assert_eq!(preview["outcome"], "stale", "{preview}");
    let resp = h.respond(&older, json!({ "action": "decline" })).await;
    assert_eq!(resp["code"], "stale", "{resp}");
    let ev = h.only_event().await;
    assert_eq!(ev["title"], "Planning v2");
    assert_eq!(partstat(&ev, ME), "accepted");
    assert_eq!(h.sent().len(), 1, "only the first answer was sent");
}

#[tokio::test]
async fn a_newer_request_after_accepting_updates_and_can_be_answered_again() {
    let h = setup().await;
    let first = h
        .deliver(
            ORGANIZER,
            &mail(ORGANIZER, &invite(1, "20261020T090000Z", "Planning", ME)),
        )
        .await;
    h.respond(&first, json!({ "action": "accept" })).await;
    let second = h
        .deliver(
            ORGANIZER,
            &mail(
                ORGANIZER,
                &invite(2, "20261021T090000Z", "Planning (moved)", ME),
            ),
        )
        .await;
    let preview = h
        .call("CalendarEvent/itip", json!({ "emailId": second }))
        .await;
    assert_eq!(preview["outcome"], "update", "{preview}");
    assert_eq!(preview["stored"]["participationStatus"], "accepted");
    let resp = h.respond(&second, json!({ "action": "tentative" })).await;
    assert_eq!(resp["sent"], true, "{resp}");
    let ev = h.only_event().await;
    assert_eq!(ev["title"], "Planning (moved)");
    assert_eq!(ev["sequence"], 2);
    assert_eq!(partstat(&ev, ME), "tentative");
}

#[tokio::test]
async fn an_update_from_someone_other_than_the_organizer_cannot_be_answered() {
    let h = setup().await;
    let first = h
        .deliver(
            ORGANIZER,
            &mail(ORGANIZER, &invite(1, "20261020T090000Z", "Planning", ME)),
        )
        .await;
    h.respond(&first, json!({ "action": "accept" })).await;
    let forged = h
        .deliver(
            "mallory@example.net",
            &mail(
                "mallory@example.net",
                &invite(5, "20261023T090000Z", "Moved by Mallory", ME),
            ),
        )
        .await;
    let resp = h.respond(&forged, json!({ "action": "accept" })).await;
    assert_eq!(resp["code"], "notOrganizer", "{resp}");
    assert_eq!(h.only_event().await["title"], "Planning");
}

#[tokio::test]
async fn a_counter_proposes_a_time_and_changes_nothing_stored() {
    let h = setup().await;
    let email = h
        .deliver(
            ORGANIZER,
            &mail(ORGANIZER, &invite(1, "20261020T090000Z", "Planning", ME)),
        )
        .await;
    let resp = h
        .respond(
            &email,
            json!({ "action": "counter", "counter": { "start": "2026-11-10T14:00:00", "duration": "PT1H" } }),
        )
        .await;
    assert_eq!(resp["sent"], true, "{resp}");
    assert!(h.events().await.is_empty(), "a counter adds nothing");
    let raw = String::from_utf8_lossy(&h.sent()[0].raw).into_owned();
    assert!(raw.contains("METHOD:COUNTER"), "{raw}");
    assert!(raw.contains("DTSTART:20261110T140000Z"), "{raw}");
    assert!(!raw.contains("bob@example.net"), "{raw}");
}

#[tokio::test]
async fn responding_by_event_id_keeps_the_sequence_and_sends_one_attendee() {
    let h = setup().await;
    h.itip(
        ORGANIZER,
        &invite(3, "20261020T090000Z", "Planning", ME),
        true,
    )
    .await;
    let id = h.only_event().await["id"].as_str().unwrap().to_string();
    let resp = h
        .call(
            "CalendarEvent/respond",
            json!({ "eventId": id, "action": "accept" }),
        )
        .await;
    assert_eq!(resp["sent"], true, "{resp}");
    assert_eq!(resp["updated"]["sequence"], 3, "{resp}");
    let raw = String::from_utf8_lossy(&h.sent()[0].raw).into_owned();
    assert!(
        raw.contains("METHOD:REPLY") && !raw.contains("bob@example.net"),
        "{raw}"
    );
}

// ── an organizer reading a REPLY ─────────────────────────────────────────────

#[tokio::test]
async fn a_reply_says_who_answered_what() {
    let h = setup().await;
    // We organize; bob is invited.
    h.call(
        "CalendarEvent/set",
        json!({ "create": { "m": {
            "uid": "mine-1@example.org", "title": "Review", "start": "2026-11-12T10:00:00",
            "timeZone": "UTC", "duration": "PT1H",
            "participants": {
                ME: { "email": ME, "role": "organizer", "participationStatus": "accepted", "expectReply": false },
                "bob@example.net": { "email": "bob@example.net", "role": "attendee", "participationStatus": "needs-action", "expectReply": false }
            }
        }}}),
    )
    .await;
    let uid = h.only_event().await["uid"].as_str().unwrap().to_string();
    let reply = format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Test//EN\r\nMETHOD:REPLY\r\n\
         BEGIN:VEVENT\r\nUID:{uid}\r\nDTSTAMP:20261025T090000Z\r\nDTSTART:20261112T100000Z\r\n\
         ORGANIZER:mailto:{ME}\r\nATTENDEE;PARTSTAT=ACCEPTED:mailto:bob@example.net\r\n\
         END:VEVENT\r\nEND:VCALENDAR\r\n"
    );
    let preview = h.itip("bob@example.net", &reply, false).await;
    assert_eq!(preview["outcome"], "reply", "{preview}");
    assert_eq!(preview["answer"], "accepted", "{preview}");
    let applied = h.itip("bob@example.net", &reply, true).await;
    assert_eq!(applied["applied"], true);
    assert_eq!(
        partstat(&h.only_event().await, "bob@example.net"),
        "accepted"
    );
    // The same REPLY from someone else is refused.
    let forged = h.itip("mallory@example.net", &reply, true).await;
    assert_eq!(forged["outcome"], "notAttendee", "{forged}");
}

// ── PUBLISH and plain .ics attachments ───────────────────────────────────────

#[tokio::test]
async fn a_plain_ics_attachment_with_several_events_is_added_only_when_asked() {
    let h = setup().await;
    let email = h
        .deliver(
            ORGANIZER,
            &mail_attachment(
                ORGANIZER,
                &plain_file(None, 3),
                "application/octet-stream",
                "talks.ics",
            ),
        )
        .await;
    let preview = h
        .call("CalendarEvent/itip", json!({ "emailId": email }))
        .await;
    assert_eq!(preview["outcome"], "publish", "{preview}");
    assert!(preview["method"].is_null());
    assert_eq!(preview["items"].as_array().unwrap().len(), 3);
    assert_eq!(preview["todos"], 1);
    assert!(h.events().await.is_empty());

    // Add one, then all: the one already held is not added twice.
    let one = h
        .call(
            "CalendarEvent/itip",
            json!({ "emailId": email, "apply": true, "uids": ["pub-2@example.com"] }),
        )
        .await;
    assert_eq!(one["imported"].as_array().unwrap().len(), 1, "{one}");
    assert!(one["items"][1]["eventId"].is_string(), "{one}");
    let all = h
        .call(
            "CalendarEvent/itip",
            json!({ "emailId": email, "apply": true }),
        )
        .await;
    assert_eq!(all["imported"].as_array().unwrap().len(), 2, "{all}");
    assert_eq!(h.events().await.len(), 3);
    let again = h
        .call("CalendarEvent/itip", json!({ "emailId": email }))
        .await;
    assert_eq!(again["outcome"], "current", "{again}");
    assert!(h.sent().is_empty(), "adding events sends nothing");
}

#[tokio::test]
async fn a_publish_part_is_named_and_its_blob_id_must_be_of_this_message() {
    let h = setup().await;
    let email = h
        .deliver(
            ORGANIZER,
            &mail_attachment(
                ORGANIZER,
                &plain_file(Some("PUBLISH"), 1),
                "text/calendar",
                "a.ics",
            ),
        )
        .await;
    let by_part = h
        .call(
            "CalendarEvent/itip",
            json!({ "emailId": email, "blobId": format!("{email}.2") }),
        )
        .await;
    assert_eq!(by_part["method"], "PUBLISH", "{by_part}");
    assert_eq!(by_part["outcome"], "publish");
    let other = h
        .deliver(
            ORGANIZER,
            &mail(ORGANIZER, &invite(1, "20261020T090000Z", "x", ME)),
        )
        .await;
    let foreign = h
        .call(
            "CalendarEvent/itip",
            json!({ "emailId": email, "blobId": format!("{other}.2") }),
        )
        .await;
    assert_eq!(foreign["type"], "notFound", "{foreign}");
    // A PUBLISH is not an invitation to answer.
    let resp = h.respond(&email, json!({ "action": "accept" })).await;
    assert_eq!(resp["code"], "notRequest", "{resp}");
}

#[tokio::test]
async fn an_oversized_calendar_part_is_refused() {
    let h = setup().await;
    let mut ics = invite(1, "20261020T090000Z", "Planning", ME);
    ics = ics.replace(
        "SUMMARY:Planning",
        &format!(
            "SUMMARY:Planning\r\nDESCRIPTION:{}",
            "x".repeat(mw_ics::MAX_CALENDAR_BYTES)
        ),
    );
    let resp = h.itip(ORGANIZER, &ics, true).await;
    assert_eq!(
        resp["type"],
        "invalidArguments",
        "{}",
        &resp.to_string()[..200.min(resp.to_string().len())]
    );
    assert!(h.events().await.is_empty());
}
