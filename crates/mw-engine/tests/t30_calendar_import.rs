//! t30-e4: `CalendarEvent/import` refuses a file that is not a calendar.
//!
//! `parse_calendar_blob` sends every blob without `BEGIN:VCALENDAR` to
//! `mw_ics::parse_hol`, and `parse_hol` used to answer `Ok(vec![])` for any
//! text. Importing a text file was therefore a success with `count: 0`, and a
//! comma-separated file whose last column is a date became one all-day event
//! per row.
//!
//! Every test drives `Engine::handle_jmap` — the envelope the web client's
//! Calendar → Import sends — and then reads the stored events back with
//! `CalendarEvent/get`, so a refusal that persisted something anyway is seen.

use std::sync::Arc;

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

const EVENT_ICS: &str = include_str!("../../../fixtures/ics/event.ics");
const HOLIDAYS_HOL: &str = include_str!("../../../fixtures/ics/holidays.hol");

/// The `description` of the refusal. `apps/web/src/modules/calendar/mock.ts`
/// answers `CalendarEvent/import` with the same text.
const NOT_A_CALENDAR: &str = "backend protocol error: ics: not a calendar document";

// ── harness (the `tests/pim.rs` one: no mail, no DAV) ─────────────────────────

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
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            Arc::new(NoopBackend) as Arc<dyn AccountBackend>,
            Arc::new(NoopSubmitter) as Arc<dyn MailSubmitter>,
            "me@example.org",
        ),
    );
    Harness { engine, account_id }
}

impl Harness {
    async fn call(&self, method: &str, args: Value) -> Value {
        let req = json!({ "methodCalls": [[method, args, "c0"]] });
        let resp = self.engine.handle_jmap(&self.account_id, &req).await;
        resp["methodResponses"][0][1].clone()
    }

    /// Every stored event of the account, as `(title, start)`, sorted.
    async fn stored(&self) -> Vec<(String, String)> {
        let got = self.call("CalendarEvent/get", json!({ "ids": null })).await;
        let mut out: Vec<(String, String)> = got["list"]
            .as_array()
            .unwrap_or_else(|| panic!("CalendarEvent/get has no list: {got}"))
            .iter()
            .map(|e| {
                (
                    e["title"].as_str().unwrap_or_default().to_string(),
                    e["start"].as_str().unwrap_or_default().to_string(),
                )
            })
            .collect();
        out.sort();
        out
    }
}

/// What is wrong with `resp` as a refusal, if anything. A refusal is a
/// method-level failure with the agreed text and neither of the success fields:
/// `count: 0` beside an error would still read as "imported 0".
fn refusal_problem(resp: &Value) -> Option<String> {
    let refused = resp["type"] == "serverFail"
        && resp["description"] == NOT_A_CALENDAR
        && resp.get("count").is_none()
        && resp.get("imported").is_none();
    (!refused).then(|| format!("not refused: {resp}"))
}

fn assert_refused(resp: &Value, what: &str) {
    if let Some(problem) = refusal_problem(resp) {
        panic!("{what}: {problem}");
    }
}

// ── the refusal ──────────────────────────────────────────────────────────────

/// The bytes `apps/web/e2e/pim-calendar.spec.ts` uploads as `notes.txt`.
#[tokio::test]
async fn a_text_file_with_no_calendar_content_is_refused_and_stores_nothing() {
    let h = setup().await;
    assert!(h.stored().await.is_empty(), "precondition: no events yet");

    let resp = h
        .call("CalendarEvent/import", json!({ "blob": "not a calendar" }))
        .await;

    assert_refused(&resp, "plain text");
    assert!(
        h.stored().await.is_empty(),
        "a refused import stored events"
    );
}

#[tokio::test]
async fn inputs_that_are_not_a_holiday_pack_are_refused_and_store_nothing() {
    let cases: &[(&str, &str)] = &[
        ("empty", ""),
        ("blank lines", "  \r\n\r\n\t\r\n"),
        (
            "an HTML page",
            "<!doctype html>\r\n<html><body><p>Sign in, 2026</p></body></html>\r\n",
        ),
        // Each row ends in a date, so the line parser alone takes every row for
        // a holiday; no section header introduces them.
        (
            "comma-separated rows ending in a date",
            "name,joined\r\nAda Lovelace,2026-01-01\r\nGrace Hopper,2026/02/03\r\n",
        ),
        // What `mw_ics::write_hol(&[])` writes: a section and nothing in it.
        ("a pack with a section and no holidays", "[Holidays] 0\r\n"),
        (
            "a section whose lines carry no date",
            "[Notes] 2\r\nno comma here\r\nBirthday,sometime in May\r\n",
        ),
        // A holiday line only counts inside a section.
        (
            "a holiday line with the section after it",
            "New Year,2026/01/01\r\n[Holidays] 1\r\n",
        ),
    ];
    // Every case is run, so one run names all the inputs that got through.
    let mut wrong = Vec::new();
    for (what, blob) in cases {
        let h = setup().await;
        let resp = h
            .call("CalendarEvent/import", json!({ "blob": blob }))
            .await;
        if let Some(problem) = refusal_problem(&resp) {
            wrong.push(format!("{what}: {problem}"));
        }
        let stored = h.stored().await;
        if !stored.is_empty() {
            wrong.push(format!("{what}: stored {stored:?}"));
        }
    }
    assert!(wrong.is_empty(), "\n{}", wrong.join("\n"));
}

/// `event_import` reads a missing `blob` as the empty string.
#[tokio::test]
async fn an_import_without_a_blob_is_refused() {
    let h = setup().await;
    let resp = h.call("CalendarEvent/import", json!({})).await;
    assert_refused(&resp, "no blob");
    assert!(h.stored().await.is_empty());
}

/// `CalendarEvent/parse` goes through the same `parse_calendar_blob`.
#[tokio::test]
async fn parse_refuses_the_same_input_instead_of_answering_an_empty_list() {
    let h = setup().await;
    let resp = h
        .call("CalendarEvent/parse", json!({ "blob": "not a calendar" }))
        .await;
    assert_eq!(resp["type"], "serverFail", "{resp}");
    assert_eq!(resp["description"], NOT_A_CALENDAR, "{resp}");
    assert!(resp.get("parsed").is_none(), "{resp}");
}

// ── what must keep importing ─────────────────────────────────────────────────

#[tokio::test]
async fn the_ics_fixture_still_imports() {
    let h = setup().await;
    let resp = h
        .call("CalendarEvent/import", json!({ "blob": EVENT_ICS }))
        .await;
    assert_eq!(resp["count"], 1, "{resp}");
    assert_eq!(resp["imported"].as_array().unwrap().len(), 1, "{resp}");
    assert_eq!(
        h.stored().await,
        vec![("Team Sync".to_string(), "2026-03-20T09:00:00".to_string())]
    );
}

#[tokio::test]
async fn the_hol_fixture_still_imports_as_all_day_events() {
    let h = setup().await;
    let resp = h
        .call("CalendarEvent/import", json!({ "blob": HOLIDAYS_HOL }))
        .await;
    assert_eq!(resp["count"], 3, "{resp}");
    assert_eq!(
        h.stored().await,
        vec![
            ("Christmas Day".to_string(), "2026-12-25".to_string()),
            ("May Day".to_string(), "2026-05-01".to_string()),
            ("New Year".to_string(), "2026-01-01".to_string()),
        ]
    );
    let got = h.call("CalendarEvent/get", json!({ "ids": null })).await;
    for ev in got["list"].as_array().unwrap() {
        assert_eq!(ev["showWithoutTime"], true, "{ev}");
    }
}

/// The shapes a pack has outside this repository's fixture: a UTF-8 byte order
/// mark before the first section, unpadded dates, LF line ends, a `;` comment,
/// a second section, and a line the parser cannot date (skipped, not fatal).
#[tokio::test]
async fn a_pack_with_a_bom_unpadded_dates_and_two_sections_imports() {
    let h = setup().await;
    let blob = "\u{feff}[Portugal] 2\n; national\nAno Novo,2026/1/1\nDia da Liberdade,2026/4/25\n\
                [Madeira] 2\nDia da Regi\u{e3}o,2026/7/1\nTo be announced,later\n";
    let resp = h
        .call("CalendarEvent/import", json!({ "blob": blob }))
        .await;
    assert_eq!(resp["count"], 3, "{resp}");
    assert_eq!(
        h.stored().await,
        vec![
            ("Ano Novo".to_string(), "2026-01-01".to_string()),
            ("Dia da Liberdade".to_string(), "2026-04-25".to_string()),
            ("Dia da Regi\u{e3}o".to_string(), "2026-07-01".to_string()),
        ]
    );
}

/// `CalendarEvent/parse` still reads both formats.
#[tokio::test]
async fn parse_still_reads_both_fixtures() {
    let h = setup().await;
    let ics = h
        .call("CalendarEvent/parse", json!({ "blob": EVENT_ICS }))
        .await;
    assert_eq!(ics["parsed"].as_array().unwrap().len(), 1, "{ics}");
    let hol = h
        .call("CalendarEvent/parse", json!({ "blob": HOLIDAYS_HOL }))
        .await;
    assert_eq!(hol["parsed"].as_array().unwrap().len(), 3, "{hol}");
    assert!(
        h.stored().await.is_empty(),
        "CalendarEvent/parse stored events"
    );
}

// ── the rule itself, at the parser ───────────────────────────────────────────

#[test]
fn parse_hol_names_what_is_missing() {
    let err = |text: &str| match mw_ics::parse_hol(text.as_bytes()) {
        Ok(v) => panic!("{text:?} parsed as {} holidays", v.len()),
        Err(e) => e.to_string(),
    };
    assert_eq!(
        err("not a calendar"),
        "holiday pack (.hol) error: no [section] line"
    );
    assert_eq!(
        err("Ada Lovelace,2026-01-01\r\n"),
        "holiday pack (.hol) error: no [section] line"
    );
    assert_eq!(
        err("[Holidays] 0\r\n"),
        "holiday pack (.hol) error: no holiday lines"
    );
    // What the writer produces for at least one event is read back.
    let pack = mw_ics::write_hol(&[json!({ "title": "New Year", "start": "2026-01-01" })]).unwrap();
    assert_eq!(mw_ics::parse_hol(pack.as_bytes()).unwrap().len(), 1);
}
