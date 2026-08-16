//! Engine integration tests for the V3 PIM surface (plan §3 e8 acceptance):
//! calendar/event round-trips, DST-correct recurrence expansion, conflict
//! detection, the iTIP REQUEST/REPLY flow (asserted through a capturing
//! submitter), VTODO tasks + My Day, sealed notes CRUD + search, contact vCard
//! round-trip + merge + CSV import + autocomplete, and PIM `*/changes` diffs.
//!
//! Everything is driven through `Engine::handle_jmap` — the exact envelope the
//! web client speaks — over a no-op mail backend (PIM never touches it).

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
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

// ── harness ──────────────────────────────────────────────────────────────────

/// A backend that serves no mail — PIM methods never call it, but a registered
/// runtime is required for `handle_jmap` to dispatch.
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

/// A submitter that records every outgoing message so iTIP framing can be
/// asserted (the REQUEST/REPLY that `CalendarEvent/set`/`respond` emit).
#[derive(Default)]
struct CapturingSubmitter {
    sent: Mutex<Vec<Outgoing>>,
}

#[async_trait]
impl MailSubmitter for CapturingSubmitter {
    async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult> {
        let accepted = msg.rcpt_to.clone();
        self.sent.lock().unwrap().push(msg);
        Ok(SubmissionResult {
            accepted,
            rejected: Vec::new(),
        })
    }
}

struct Harness {
    engine: Arc<Engine>,
    account_id: String,
    submitter: Arc<CapturingSubmitter>,
}

async fn setup() -> Harness {
    setup_inner(None).await
}

/// A harness whose account carries a CalDAV/CardDAV config (env-gated live tests).
async fn setup_with_dav(dav: mw_dav::DavConfig) -> Harness {
    setup_inner(Some(dav)).await
}

async fn setup_inner(dav: Option<mw_dav::DavConfig>) -> Harness {
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
    let submitter = Arc::new(CapturingSubmitter::default());
    let mut runtime = AccountRuntime::new(
        Arc::new(NoopBackend) as Arc<dyn AccountBackend>,
        submitter.clone() as Arc<dyn MailSubmitter>,
        "me@example.org",
    );
    if let Some(cfg) = dav {
        runtime = runtime.with_dav(cfg);
    }
    engine.register_backend(account_id.clone(), runtime);
    Harness {
        engine,
        account_id,
        submitter,
    }
}

/// Build a CalDAV/CardDAV config from `RADICALE_URL` / `RADICALE_USER` /
/// `RADICALE_PASS`; `None` when unset (the live tests then no-op).
fn radicale_config() -> Option<mw_dav::DavConfig> {
    let base_url = std::env::var("RADICALE_URL").ok()?;
    Some(mw_dav::DavConfig {
        base_url,
        username: std::env::var("RADICALE_USER").unwrap_or_default(),
        password: std::env::var("RADICALE_PASS").unwrap_or_default(),
    })
}

/// A standalone runtime carrying a DAV config, for calling `Engine::sync_pim`
/// (which reads only `rt.dav`).
fn dav_runtime(cfg: mw_dav::DavConfig) -> AccountRuntime {
    AccountRuntime::new(
        Arc::new(NoopBackend) as Arc<dyn AccountBackend>,
        Arc::new(CapturingSubmitter::default()) as Arc<dyn MailSubmitter>,
        "me@example.org",
    )
    .with_dav(cfg)
}

impl Harness {
    /// Invoke one JMAP method and return its response arguments object.
    async fn call(&self, method: &str, args: Value) -> Value {
        let req = json!({ "methodCalls": [[method, args, "c0"]] });
        let resp = self.engine.handle_jmap(&self.account_id, &req).await;
        resp["methodResponses"][0][1].clone()
    }

    async fn session_state(&self) -> String {
        let resp = self
            .engine
            .handle_jmap(&self.account_id, &json!({ "methodCalls": [] }))
            .await;
        resp["sessionState"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    }
}

// ── calendars + events ────────────────────────────────────────────────────────

#[tokio::test]
async fn calendar_seeds_defaults_and_event_round_trips() {
    let h = setup().await;
    // Calendar/get seeds a default event calendar + task list.
    let cals = h.call("Calendar/get", json!({})).await;
    let list = cals["list"].as_array().unwrap();
    assert!(list.iter().any(|c| c["component"] == "VEVENT"));
    assert!(list.iter().any(|c| c["component"] == "VTODO"));
    let cal_id = list.iter().find(|c| c["component"] == "VEVENT").unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Create an event.
    let set = h
        .call(
            "CalendarEvent/set",
            json!({ "create": { "new1": {
                "calendarId": cal_id,
                "title": "Standup",
                "start": "2026-07-13T09:00:00",
                "timeZone": "UTC",
                "duration": "PT30M",
            }}}),
        )
        .await;
    let id = set["created"]["new1"]["id"].as_str().unwrap().to_string();

    // Get it back.
    let got = h.call("CalendarEvent/get", json!({ "ids": [id] })).await;
    assert_eq!(got["list"][0]["title"], "Standup");
    assert_eq!(got["list"][0]["start"], "2026-07-13T09:00:00");

    // Query within a window finds it; outside does not.
    let in_win = h
        .call(
            "CalendarEvent/query",
            json!({ "filter": { "after": "2026-07-13T00:00:00Z", "before": "2026-07-14T00:00:00Z" }}),
        )
        .await;
    assert_eq!(in_win["ids"].as_array().unwrap().len(), 1);
    let out_win = h
        .call(
            "CalendarEvent/query",
            json!({ "filter": { "after": "2026-08-01T00:00:00Z", "before": "2026-08-02T00:00:00Z" }}),
        )
        .await;
    assert_eq!(out_win["ids"].as_array().unwrap().len(), 0);

    // Destroy round-trips.
    let del = h
        .call("CalendarEvent/set", json!({ "destroy": [id] }))
        .await;
    assert_eq!(del["destroyed"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn recurrence_expands_across_dst_spring_forward() {
    let h = setup().await;
    // A weekly 09:00 event in Europe/London crossing the 2026-03-29 GMT→BST
    // boundary: the wall clock holds at 09:00, the UTC offset shifts (§1.12).
    let set = h
        .call(
            "CalendarEvent/set",
            json!({ "create": { "e": {
                "title": "Weekly",
                "start": "2026-03-25T09:00:00",
                "timeZone": "Europe/London",
                "duration": "PT1H",
                "recurrenceRules": [{ "rrule": "FREQ=WEEKLY;COUNT=3" }],
            }}}),
        )
        .await;
    let id = set["created"]["e"]["id"].as_str().unwrap().to_string();

    let expanded = h
        .call(
            "CalendarEvent/expand",
            json!({ "ids": [id], "start": "2026-03-01T00:00:00Z", "end": "2026-05-01T00:00:00Z" }),
        )
        .await;
    let insts = expanded["list"].as_array().unwrap();
    assert_eq!(insts.len(), 3);
    // Mar 25 is GMT (UTC+0) → 09:00Z; Apr 1 + Apr 8 are BST (UTC+1) → 08:00Z.
    assert_eq!(insts[0]["instanceStart"], "2026-03-25T09:00:00Z");
    assert_eq!(insts[1]["instanceStart"], "2026-04-01T08:00:00Z");
    assert_eq!(insts[2]["instanceStart"], "2026-04-08T08:00:00Z");
}

#[tokio::test]
async fn detect_conflicts_finds_overlaps() {
    let h = setup().await;
    for (t, start) in [("A", "2026-07-13T09:00:00"), ("B", "2026-07-13T09:30:00")] {
        h.call(
            "CalendarEvent/set",
            json!({ "create": { t: {
                "title": t,
                "start": start,
                "timeZone": "UTC",
                "duration": "PT1H",
            }}}),
        )
        .await;
    }
    let conflicts = h
        .call(
            "Calendar/detectConflicts",
            json!({ "start": "2026-07-13T00:00:00Z", "end": "2026-07-14T00:00:00Z" }),
        )
        .await;
    // A (09:00–10:00) overlaps B (09:30–10:30).
    assert_eq!(conflicts["list"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn free_busy_aggregates_busy_blocks() {
    let h = setup().await;
    h.call(
        "CalendarEvent/set",
        json!({ "create": { "e": {
            "title": "Busy",
            "start": "2026-07-13T09:00:00",
            "timeZone": "UTC",
            "duration": "PT1H",
            "freeBusyStatus": "busy",
        }}}),
    )
    .await;
    let fb = h
        .call(
            "Calendar/freeBusy",
            json!({ "start": "2026-07-13T00:00:00Z", "end": "2026-07-14T00:00:00Z" }),
        )
        .await;
    let list = fb["list"].as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["start"], "2026-07-13T09:00:00Z");
}

#[tokio::test]
async fn itip_request_on_create_and_reply_on_respond() {
    let h = setup().await;
    // Create an event where WE are an attendee invited by an external organizer.
    let set = h
        .call(
            "CalendarEvent/set",
            json!({ "create": { "inv": {
                "title": "Review",
                "start": "2026-07-20T15:00:00",
                "timeZone": "UTC",
                "duration": "PT1H",
                "participants": {
                    "boss@example.com": { "name": "Boss", "email": "boss@example.com", "role": "organizer", "participationStatus": "accepted", "expectReply": false },
                    "me@example.org": { "name": "Me", "email": "me@example.org", "role": "attendee", "participationStatus": "needs-action", "expectReply": true }
                }
            }}}),
        )
        .await;
    let id = set["created"]["inv"]["id"].as_str().unwrap().to_string();
    // No REQUEST fired (the only expectReply participant is ourselves).
    assert_eq!(h.submitter.sent.lock().unwrap().len(), 0);

    // Accept the invite → an iMIP REPLY goes to the organizer.
    let resp = h
        .call(
            "CalendarEvent/respond",
            json!({ "eventId": id, "action": "accept" }),
        )
        .await;
    assert_eq!(
        resp["updated"]["participants"]["me@example.org"]["participationStatus"],
        "accepted"
    );
    let sent = h.submitter.sent.lock().unwrap();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].rcpt_to, vec!["boss@example.com".to_string()]);
    let raw = String::from_utf8_lossy(&sent[0].raw);
    assert!(raw.contains("METHOD:REPLY"), "expected a REPLY: {raw}");
}

#[tokio::test]
async fn ics_import_and_export_round_trip() {
    let h = setup().await;
    let ics = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:imp-1\r\nSUMMARY:Imported\r\nDTSTART:20260714T120000Z\r\nDTEND:20260714T130000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
    let imported = h.call("CalendarEvent/import", json!({ "blob": ics })).await;
    assert_eq!(imported["count"], 1);
    let export = h.call("CalendarEvent/export", json!({})).await;
    let blob = export["blob"].as_str().unwrap();
    assert!(blob.contains("SUMMARY:Imported"));
    assert!(blob.contains("UID:imp-1"));
}

// ── tasks ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn task_round_trip_and_my_day_and_from_event() {
    let h = setup().await;
    // A task due today lands in My Day.
    let today = chrono::Utc::now()
        .date_naive()
        .format("%Y-%m-%d")
        .to_string();
    let set = h
        .call(
            "Task/set",
            json!({ "create": { "t": {
                "title": "Pay invoice",
                "due": format!("{today}T17:00:00"),
                "timeZone": "UTC",
            }}}),
        )
        .await;
    let tid = set["created"]["t"]["id"].as_str().unwrap().to_string();
    let got = h.call("Task/get", json!({ "ids": [tid] })).await;
    assert_eq!(got["list"][0]["title"], "Pay invoice");
    assert_eq!(got["list"][0]["status"], "needs-action");

    let my_day = h
        .call("Task/query", json!({ "filter": { "myDay": true }}))
        .await;
    assert_eq!(my_day["ids"].as_array().unwrap().len(), 1);

    // Complete it and confirm the status update round-trips.
    h.call(
        "Task/set",
        json!({ "update": { tid.clone(): { "status": "completed", "percentComplete": 100 }}}),
    )
    .await;
    let done = h.call("Task/get", json!({ "ids": [tid] })).await;
    assert_eq!(done["list"][0]["status"], "completed");
    assert_eq!(done["list"][0]["percentComplete"], 100);

    // event→task convenience seeds the title from the event.
    let ev = h
        .call(
            "CalendarEvent/set",
            json!({ "create": { "e": { "title": "Launch", "start": "2026-07-20T10:00:00", "timeZone": "UTC", "duration": "PT1H" }}}),
        )
        .await;
    let eid = ev["created"]["e"]["id"].as_str().unwrap().to_string();
    let from = h
        .call(
            "Task/set",
            json!({ "create": { "ft": { "fromEvent": { "eventId": eid }}}}),
        )
        .await;
    let ftid = from["created"]["ft"]["id"].as_str().unwrap().to_string();
    let ft = h.call("Task/get", json!({ "ids": [ftid] })).await;
    assert_eq!(ft["list"][0]["title"], "Launch");
}

// ── notes ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn note_crud_seals_and_searches() {
    let h = setup().await;
    let set = h
        .call(
            "Note/set",
            json!({ "create": { "n": {
                "title": "Recipe",
                "tags": ["cooking", "dinner"],
                "color": "#ffaa00",
                "pinned": true,
                "bodyText": "secret sauce with paprika",
                "bodyHtml": "<p>secret sauce with paprika</p>",
            }}}),
        )
        .await;
    let nid = set["created"]["n"]["id"].as_str().unwrap().to_string();

    // Get decrypts the sealed body.
    let got = h.call("Note/get", json!({ "ids": [nid] })).await;
    assert_eq!(got["list"][0]["bodyText"], "secret sauce with paprika");
    assert_eq!(got["list"][0]["pinned"], true);

    // Query by tag.
    let by_tag = h
        .call("Note/query", json!({ "filter": { "tags": ["cooking"] }}))
        .await;
    assert_eq!(by_tag["ids"].as_array().unwrap().len(), 1);
    // Query by pinned.
    let pinned = h
        .call("Note/query", json!({ "filter": { "pinned": true }}))
        .await;
    assert_eq!(pinned["ids"].as_array().unwrap().len(), 1);
    // Body substring search (decrypt-scan).
    let by_body = h
        .call("Note/query", json!({ "filter": { "text": "paprika" }}))
        .await;
    assert_eq!(by_body["ids"].as_array().unwrap().len(), 1);
    // A non-matching term finds nothing.
    let miss = h
        .call("Note/query", json!({ "filter": { "text": "nutmeg" }}))
        .await;
    assert_eq!(miss["ids"].as_array().unwrap().len(), 0);
}

// ── contacts ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn contact_round_trip_import_merge_autocomplete() {
    let h = setup().await;
    // Create a contact via the projection.
    let set = h
        .call(
            "ContactCard/set",
            json!({ "create": { "c": {
                "kind": "individual",
                "name": { "full": "Ada Lovelace", "given": "Ada", "surname": "Lovelace" },
                "emails": [{ "context": "work", "value": "ada@example.org", "pref": 1 }],
                "isFavorite": true,
            }}}),
        )
        .await;
    let cid = set["created"]["c"]["id"].as_str().unwrap().to_string();
    let got = h.call("ContactCard/get", json!({ "ids": [cid] })).await;
    assert_eq!(got["list"][0]["name"]["full"], "Ada Lovelace");
    assert_eq!(got["list"][0]["emails"][0]["value"], "ada@example.org");
    assert_eq!(got["list"][0]["isFavorite"], true);

    // vCard import.
    let vcf = "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:g-1\r\nFN:Grace Hopper\r\nEMAIL:grace@example.org\r\nEND:VCARD\r\n";
    let imp = h.call("ContactCard/import", json!({ "blob": vcf })).await;
    assert_eq!(imp["count"], 1);

    // CSV import.
    let csv = "full_name,email\nAlan Turing,alan@example.org\n";
    let csv_imp = h.call("ContactCard/import", json!({ "blob": csv })).await;
    assert_eq!(csv_imp["count"], 1);

    // Autocomplete ranks the favourite (Ada) and matches by prefix.
    let ac = h
        .call(
            "ContactCard/autocomplete",
            json!({ "prefix": "a", "limit": 10 }),
        )
        .await;
    let list = ac["list"].as_array().unwrap();
    assert!(!list.is_empty());
    assert_eq!(list[0]["email"], "ada@example.org"); // favourite first

    // Merge two duplicates → merged card + tombstones.
    let dup = h
        .call(
            "ContactCard/set",
            json!({ "create": { "d": {
                "name": { "full": "Ada Lovelace" },
                "emails": [{ "context": "home", "value": "ada@home.test", "pref": 0 }],
            }}}),
        )
        .await;
    let did = dup["created"]["d"]["id"].as_str().unwrap().to_string();
    let merged = h
        .call("ContactCard/merge", json!({ "ids": [cid, did] }))
        .await;
    let mid = merged["merged"].as_str().unwrap().to_string();
    let mcard = h.call("ContactCard/get", json!({ "ids": [mid] })).await;
    let emails = mcard["list"][0]["emails"].as_array().unwrap();
    assert_eq!(emails.len(), 2, "merge unions both emails");
    // The sources are tombstoned.
    let gone = h
        .call("ContactCard/get", json!({ "ids": [cid, did] }))
        .await;
    assert_eq!(gone["notFound"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn contact_groups_round_trip() {
    let h = setup().await;
    let set = h
        .call(
            "ContactGroup/set",
            json!({ "create": { "g": { "name": "Team", "memberIds": ["x", "y"] }}}),
        )
        .await;
    let gid = set["created"]["g"]["id"].as_str().unwrap().to_string();
    let got = h.call("ContactGroup/get", json!({ "ids": [gid] })).await;
    assert_eq!(got["list"][0]["name"], "Team");
    assert_eq!(got["list"][0]["memberIds"].as_array().unwrap().len(), 2);
}

// ── ContactCard/merge — the two request shapes (26.20, t22-e13) ───────────────
//
// The web client emits `{accountId, keepId, mergeIds}`
// (`apps/web/src/modules/contacts/api.ts`); the engine read `args["ids"]`,
// defaulted it to empty and rejected anything shorter than two, so **every merge
// the product could issue failed** with a `serverFail` the UI swallowed. Two
// divergences sat behind that one, and a fix that only renamed the argument
// would have shipped both:
//
// * **Survivor.** The engine minted a *new* card and tombstoned every source,
//   `keepId` included — so neither `keepId` nor `ids[0]` survived.
// * **Response.** The engine answered `{merged: <id string>}`; the client is
//   typed for a card object and spreads it into its store. A request-parsing-only
//   fix passes a server-side test and still leaves a string where a card belongs.
//
// Assertions below are on the **server's response and the store afterwards**.
// Asserting the client's request shape proves nothing — the client was already
// correct and was already sending exactly this.
//
// `contact_round_trip_import_merge_autocomplete` above is deliberately left
// untouched: it drives the legacy `ids` form and is the control that the old
// shape still works.

/// Helpers for the merge tests. A separate `impl` block so nothing above is
/// disturbed; the harness itself is reused rather than copied.
impl Harness {
    /// Create a card and return its server-assigned id.
    async fn card(&self, full: &str, email: &str) -> String {
        let set = self
            .call(
                "ContactCard/set",
                json!({ "create": { "c": {
                    "name": { "full": full },
                    "emails": [{ "context": "work", "value": email, "pref": 1 }],
                }}}),
            )
            .await;
        set["created"]["c"]["id"]
            .as_str()
            .unwrap_or_else(|| panic!("create rejected: {set}"))
            .to_string()
    }

    /// Ids the account currently holds, via `ContactCard/get` with no `ids`.
    async fn all_contact_ids(&self) -> Vec<String> {
        let got = self
            .call("ContactCard/get", json!({ "ids": Value::Null }))
            .await;
        got["list"]
            .as_array()
            .expect("list")
            .iter()
            .filter_map(|c| c["id"].as_str().map(String::from))
            .collect()
    }

    /// Whether `id` still resolves to a stored card.
    async fn card_exists(&self, id: &str) -> bool {
        let got = self.call("ContactCard/get", json!({ "ids": [id] })).await;
        got["list"].as_array().is_some_and(|l| !l.is_empty())
    }

    /// Every email value on a stored card, sorted.
    async fn card_emails(&self, id: &str) -> Vec<String> {
        let got = self.call("ContactCard/get", json!({ "ids": [id] })).await;
        let mut v: Vec<String> = got["list"][0]["emails"]
            .as_array()
            .unwrap_or_else(|| panic!("no emails on {id}: {got}"))
            .iter()
            .filter_map(|e| e["value"].as_str().map(String::from))
            .collect();
        v.sort();
        v
    }
}

/// Fail loudly on a `serverFail`, so a broken merge cannot be read as an empty
/// success by a later `.get(..)` that quietly returns `None`.
fn merge_ok(resp: &Value, what: &str) -> Value {
    assert_ne!(
        resp["type"].as_str(),
        Some("serverFail"),
        "{what} returned an error: {resp}"
    );
    resp.clone()
}

/// The id of the card that survived a merge, whichever response shape carried
/// it: the `keepId` form answers with the survivor as a full card object, the
/// legacy form with a bare id string. Panics rather than defaulting — a helper
/// that answered `""` for a missing field would make every caller vacuously true.
fn survivor_id(resp: &Value) -> String {
    match &resp["merged"] {
        Value::String(s) => s.clone(),
        Value::Object(o) => o
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("merged card carries no id: {resp}"))
            .to_string(),
        other => panic!("merge response has no usable `merged`: {other} in {resp}"),
    }
}

fn id_list(resp: &Value, field: &str) -> Vec<String> {
    resp[field]
        .as_array()
        .unwrap_or_else(|| panic!("`{field}` missing from {resp}"))
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect()
}

#[tokio::test]
async fn merge_with_keep_id_keeps_that_card_and_destroys_only_the_others() {
    let h = setup().await;
    let keep = h.card("Ada Lovelace", "ada@example.org").await;
    let dup1 = h.card("Ada Lovelace", "ada@home.test").await;
    let dup2 = h.card("A. Lovelace", "ada@work.test").await;

    // Exactly the request `contactMerge()` in the web client builds.
    let resp = merge_ok(
        &h.call(
            "ContactCard/merge",
            json!({ "keepId": keep, "mergeIds": [dup1, dup2] }),
        )
        .await,
        "merge",
    );

    // The survivor is the card the caller named — not a freshly minted id, and
    // not `mergeIds[0]`.
    assert_eq!(
        survivor_id(&resp),
        keep,
        "the survivor must be keepId: {resp}"
    );
    assert_eq!(resp["keptId"].as_str(), Some(keep.as_str()));

    // The server's own account of what it destroyed.
    let mut destroyed = id_list(&resp, "destroyed");
    destroyed.sort();
    let mut expected = vec![dup1.clone(), dup2.clone()];
    expected.sort();
    assert_eq!(destroyed, expected, "destroyed must be exactly mergeIds");
    assert!(
        !destroyed.contains(&keep),
        "the kept card must never appear in `destroyed`"
    );

    // And what the store actually holds: the survivor and nothing else. This is
    // what separates "kept keepId" from "kept keepId *and* also minted a new
    // card" — a response-only assertion cannot see the difference.
    assert!(
        h.card_exists(&keep).await,
        "the kept card must still be stored"
    );
    assert!(!h.card_exists(&dup1).await, "dup1 must be tombstoned");
    assert!(!h.card_exists(&dup2).await, "dup2 must be tombstoned");
    assert_eq!(
        h.all_contact_ids().await,
        vec![keep.clone()],
        "exactly one card must remain, and it must be the kept one"
    );

    // The merge did its actual job: the survivor carries every source's email.
    assert_eq!(
        h.card_emails(&keep).await,
        vec![
            "ada@example.org".to_string(),
            "ada@home.test".to_string(),
            "ada@work.test".to_string()
        ],
        "the survivor must union all three addresses"
    );

    // The response carries the survivor as a card, not an id — the client
    // patches this object straight into its store.
    assert!(
        resp["merged"].is_object(),
        "`merged` must be the card itself: {resp}"
    );
    assert_eq!(
        resp["merged"]["name"]["full"].as_str(),
        Some("Ada Lovelace")
    );
    assert_eq!(
        resp["merged"]["emails"].as_array().map(Vec::len),
        Some(3),
        "the returned card must be the merged one, not the pre-merge card"
    );
}

/// The legacy `{ids}` form, and the assertion that pins **how it differs**.
///
/// Its survivor is a new card, and `ids[0]` is destroyed like every other
/// source. That divergence from the `keepId` form is deliberate back-compat, so
/// it is asserted rather than left implicit: if someone "unifies" the two paths
/// on the reasonable assumption that they are aliases, this test fails.
#[tokio::test]
async fn legacy_ids_form_mints_a_new_card_and_does_not_preserve_ids_first() {
    let h = setup().await;
    let a = h.card("Grace Hopper", "grace@example.org").await;
    let b = h.card("Grace Hopper", "grace@navy.test").await;

    let resp = merge_ok(
        &h.call("ContactCard/merge", json!({ "ids": [a, b] })).await,
        "legacy merge",
    );

    let new_id = survivor_id(&resp);
    assert!(resp["merged"].is_string(), "legacy `merged` stays an id");
    assert_ne!(
        new_id, a,
        "the legacy form must NOT preserve ids[0] — that is what makes it \
         different from the keepId form, and unifying the two must fail here"
    );
    assert_ne!(new_id, b);
    assert!(
        !h.card_exists(&a).await,
        "ids[0] is tombstoned like any other source"
    );
    assert!(!h.card_exists(&b).await);
    assert_eq!(h.all_contact_ids().await, vec![new_id.clone()]);

    // Both pre-existing response fields still populated as they were, plus the
    // additively-added `destroyed`.
    assert_eq!(id_list(&resp, "tombstoned"), vec![a.clone(), b.clone()]);
    assert_eq!(
        id_list(&resp, "destroyed"),
        vec![a.clone(), b.clone()],
        "`destroyed` is reported by both shapes"
    );
    assert_eq!(
        h.card_emails(&new_id).await,
        vec![
            "grace@example.org".to_string(),
            "grace@navy.test".to_string()
        ],
    );
}

/// NEGATIVE CONTROL for `survivor_id`. Every assertion above rests on this
/// helper reporting the id the engine really kept. Run it over the two shapes in
/// one test and require the answers to differ in exactly the way the two designs
/// differ: the `keepId` form's survivor **is** an input id, the legacy form's
/// **is not** any input id. A helper that echoed its input, or that read a
/// missing field as a match, cannot satisfy both halves.
#[tokio::test]
async fn the_survivor_helper_can_tell_two_survivors_apart() {
    let h = setup().await;

    let keep = h.card("Kept", "kept@example.org").await;
    let gone = h.card("Gone", "gone@example.org").await;
    let kept_resp = h
        .call(
            "ContactCard/merge",
            json!({ "keepId": keep, "mergeIds": [gone] }),
        )
        .await;
    let kept_survivor = survivor_id(&merge_ok(&kept_resp, "keepId merge"));

    let x = h.card("X", "x@example.org").await;
    let y = h.card("Y", "y@example.org").await;
    let legacy_resp = h.call("ContactCard/merge", json!({ "ids": [x, y] })).await;
    let legacy_survivor = survivor_id(&merge_ok(&legacy_resp, "legacy merge"));

    assert_eq!(
        kept_survivor, keep,
        "keepId form: survivor is the named input"
    );
    assert!(
        legacy_survivor != x && legacy_survivor != y,
        "legacy form: survivor is a new card, not an input — if this reads as a \
         match, `survivor_id` is not reading a real value"
    );
    assert_ne!(
        kept_survivor, legacy_survivor,
        "the helper must distinguish the two survivors"
    );
}

#[tokio::test]
async fn merge_refuses_without_a_second_card_and_keeps_the_first() {
    let h = setup().await;
    let only = h.card("Solo", "solo@example.org").await;

    for args in [
        json!({ "keepId": only, "mergeIds": [] }),
        // `keepId` listed among the ids to merge away is not a licence to
        // tombstone the survivor.
        json!({ "keepId": only, "mergeIds": [only] }),
        json!({ "keepId": only }),
    ] {
        let resp = h.call("ContactCard/merge", args.clone()).await;
        assert_eq!(
            resp["type"].as_str(),
            Some("serverFail"),
            "{args} should have been refused, got {resp}"
        );
        assert!(
            h.card_exists(&only).await,
            "a refused merge must leave the card alone ({args})"
        );
    }
    assert_eq!(h.all_contact_ids().await, vec![only]);
}

#[tokio::test]
async fn merge_with_an_unknown_id_destroys_nothing() {
    let h = setup().await;
    let keep = h.card("Ada", "ada@example.org").await;
    let dup = h.card("Ada", "ada@home.test").await;

    let resp = h
        .call(
            "ContactCard/merge",
            json!({ "keepId": keep, "mergeIds": [dup, "contact_does_not_exist"] }),
        )
        .await;
    assert_eq!(resp["type"].as_str(), Some("serverFail"), "{resp}");

    // Every source is resolved before anything is written, so a bad id in the
    // list cannot leave a half-applied merge behind.
    assert!(h.card_exists(&keep).await, "keep survived the refusal");
    assert!(
        h.card_exists(&dup).await,
        "dup was not tombstoned by a failed merge"
    );
    assert_eq!(
        h.card_emails(&keep).await,
        vec!["ada@example.org".to_string()]
    );
}

// ── state / changes ───────────────────────────────────────────────────────────

#[tokio::test]
async fn pim_state_advances_and_changes_diff_is_correct() {
    let h = setup().await;
    let s0 = h.session_state().await;

    let set = h
        .call("Note/set", json!({ "create": { "n": { "title": "First" }}}))
        .await;
    let nid = set["created"]["n"]["id"].as_str().unwrap().to_string();
    let old_state = set["oldState"].as_str().unwrap().to_string();

    // sessionState advanced on the PIM change.
    let s1 = h.session_state().await;
    assert_ne!(s0, s1);

    // Note/changes since the create's oldState reports the created id.
    let changes = h
        .call("Note/changes", json!({ "sinceState": old_state }))
        .await;
    assert_eq!(changes["created"], json!([nid]));

    // Update then destroy; the diff since old_state folds to "destroyed".
    h.call(
        "Note/set",
        json!({ "update": { nid.clone(): { "title": "Second" }}}),
    )
    .await;
    let after_update = h
        .call("Note/changes", json!({ "sinceState": old_state }))
        .await;
    // The latest op per id wins (same fold as the mail `*/changes`): a
    // create-then-update since the same base state reads as an update.
    assert_eq!(after_update["updated"], json!([nid]));
    assert_eq!(after_update["created"], json!([]));
}

// ── live CalDAV round-trip (env-gated; e11 runs it against Radicale) ──────────

/// Push an event to a real CalDAV server through engine A, then pull it into a
/// fresh engine B over the same collection and assert it round-trips (plan §3 e8
/// acceptance). Gated on `RADICALE_URL`; `#[ignore]` so it never runs in the
/// deterministic suite (e11 wires the Radicale service + drops the ignore).
#[tokio::test]
#[ignore = "requires a live CalDAV server (RADICALE_URL)"]
async fn caldav_event_round_trip_and_repull() {
    let Some(cfg) = radicale_config() else {
        eprintln!("RADICALE_URL unset — skipping live CalDAV round-trip");
        return;
    };

    // Discover a real calendar collection href to back both engines' calendars.
    let client = mw_dav::DavClient::new(cfg.clone()).expect("dav client");
    let cols = client
        .discover_calendars()
        .await
        .expect("discover calendars");
    let href = cols
        .iter()
        .find(|c| c.components.iter().any(|x| x == "VEVENT"))
        .or_else(|| cols.first())
        .map(|c| c.href.clone())
        .expect("at least one calendar collection");

    // Engine A pushes an event to the server.
    let ha = setup_with_dav(cfg.clone()).await;
    let cal_a = ha
        .call(
            "Calendar/set",
            json!({ "create": { "c": { "name": "MW Test", "component": "VEVENT", "caldavUrl": href }}}),
        )
        .await;
    let cal_a_id = cal_a["created"]["c"]["id"].as_str().unwrap().to_string();
    let uid = format!("mw-e8-{}", chrono::Utc::now().timestamp_millis());
    ha.call(
        "CalendarEvent/set",
        json!({ "create": { "e": {
            "calendarId": cal_a_id,
            "uid": uid,
            "title": "Round Trip",
            "start": "2026-09-01T10:00:00",
            "timeZone": "UTC",
            "duration": "PT1H",
        }}}),
    )
    .await;

    // Engine B (a fresh store) pulls the same collection and sees the event.
    let hb = setup_with_dav(cfg.clone()).await;
    let cal_b = hb
        .call(
            "Calendar/set",
            json!({ "create": { "c": { "name": "MW Test", "component": "VEVENT", "caldavUrl": href }}}),
        )
        .await;
    let _cal_b_id = cal_b["created"]["c"]["id"].as_str().unwrap().to_string();
    hb.engine
        .sync_pim(&hb.account_id, &dav_runtime(cfg.clone()))
        .await
        .expect("sync_pim");

    let got = hb.call("CalendarEvent/get", json!({})).await;
    let titles: Vec<String> = got["list"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e["title"].as_str().map(String::from))
        .collect();
    assert!(
        titles.contains(&"Round Trip".to_string()),
        "pulled events {titles:?} should include the pushed event"
    );

    // Cleanup: delete the pushed resource from the server.
    let _ = client
        .delete_resource(&format!("{}/{}.ics", href.trim_end_matches('/'), uid), None)
        .await;
}
