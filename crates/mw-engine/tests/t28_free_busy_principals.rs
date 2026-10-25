//! 26.20 t28-e19 (f34) / t29 §6 A3 — `Calendar/freeBusy` answers for the
//! addresses in `principals`, each block tagged `principal`.
//!
//! Before 26.20 the method ignored `principals` and returned the caller's own
//! blocks only, so the conflict view could not show an attendee's time. The
//! sources, in order: a same-server account that shared a calendar with the
//! caller (only the shared calendars, never an account that has not shared), the
//! caller's CalDAV server (RFC 6638 free/busy request, against a stub here), and
//! otherwise `{principal, status: "unknown"}`. Cancelled events are not busy.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use mw_engine::account::AccountRuntime;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, EngineError, Flag, MailboxDelta, MessageRef,
    MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result, SyncCursor, WatchHandle,
};
use mw_engine::{Engine, MailSubmitter};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

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

struct NullSubmitter;

#[async_trait]
impl MailSubmitter for NullSubmitter {
    async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult> {
        Ok(SubmissionResult {
            accepted: msg.rcpt_to,
            rejected: Vec::new(),
        })
    }
}

async fn add_account(
    store: &Store,
    engine: &Engine,
    user: &str,
    dav: Option<mw_dav::DavConfig>,
) -> String {
    let id = store
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
        .unwrap();
    let mut rt = AccountRuntime::new(
        Arc::new(NoopBackend) as Arc<dyn AccountBackend>,
        Arc::new(NullSubmitter) as Arc<dyn MailSubmitter>,
        user,
    );
    if let Some(d) = dav {
        rt = rt.with_dav(d);
    }
    engine.register_backend(id.clone(), rt);
    id
}

async fn call(engine: &Engine, account: &str, method: &str, args: Value) -> Value {
    let resp = engine
        .handle_jmap(account, &json!({ "methodCalls": [[method, args, "c0"]] }))
        .await;
    resp["methodResponses"][0][1].clone()
}

/// Create a calendar for `account`, shared with `share_with` (if any), holding
/// one event per `(title, start, status)`. Returns the calendar id.
async fn calendar_with(
    engine: &Engine,
    account: &str,
    name: &str,
    share_with: Option<&str>,
    events: &[(&str, &str, &str)],
) -> String {
    let mut cal = json!({ "name": name, "component": "VEVENT" });
    if let Some(p) = share_with {
        cal["shareWith"] = json!([{ "principal": p, "access": "read" }]);
    }
    let made = call(
        engine,
        account,
        "Calendar/set",
        json!({ "create": { "c": cal } }),
    )
    .await;
    let cal_id = made["created"]["c"]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("calendar created: {made}"))
        .to_string();
    for (i, (title, start, status)) in events.iter().enumerate() {
        let r = call(
            engine,
            account,
            "CalendarEvent/set",
            json!({ "create": { format!("e{i}"): {
                "calendarId": cal_id, "title": title, "start": start,
                "timeZone": "UTC", "duration": "PT1H", "status": status,
            }}}),
        )
        .await;
        assert!(
            r["created"].as_object().is_some_and(|c| c.len() == 1),
            "event created: {r}"
        );
    }
    cal_id
}

fn tagged<'a>(list: &'a [Value], principal: &str) -> Vec<&'a Value> {
    list.iter()
        .filter(|b| b["principal"] == principal)
        .collect()
}

const WINDOW: (&str, &str) = ("2026-07-13T00:00:00Z", "2026-07-14T00:00:00Z");

#[tokio::test]
async fn principals_get_shared_time_or_unknown_never_unshared_time() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let engine = Arc::new(Engine::new(store.clone()));
    let me = add_account(&store, &engine, "me@example.org", None).await;
    let bob = add_account(&store, &engine, "bob@example.org", None).await;
    let carol = add_account(&store, &engine, "carol@example.org", None).await;

    // Bob: a calendar NOT shared with me (busy 15:00), then — after the
    // precondition — one shared with me holding a busy 09:00 and a cancelled 11:00.
    calendar_with(
        &engine,
        &bob,
        "Private",
        None,
        &[("Dentist", "2026-07-13T15:00:00", "confirmed")],
    )
    .await;
    // Carol shares only with someone else.
    calendar_with(
        &engine,
        &carol,
        "Team",
        Some("zed@example.org"),
        &[("Standup", "2026-07-13T08:00:00", "confirmed")],
    )
    .await;

    let ask = |principals: Value| {
        let engine = engine.clone();
        let me = me.clone();
        async move {
            call(
                &engine,
                &me,
                "Calendar/freeBusy",
                json!({ "start": WINDOW.0, "end": WINDOW.1, "principals": principals }),
            )
            .await
        }
    };

    // Precondition: nothing shared with me yet → bob is unknown, and his private
    // calendar is not read.
    let before = ask(json!(["bob@example.org"])).await;
    let list = before["list"].as_array().unwrap().clone();
    assert_eq!(
        tagged(&list, "bob@example.org"),
        [&json!({ "principal": "bob@example.org", "status": "unknown" })],
        "{before}"
    );

    calendar_with(
        &engine,
        &bob,
        "Shared",
        Some("Me@Example.org"),
        &[
            ("Planning", "2026-07-13T09:00:00", "confirmed"),
            ("Cancelled call", "2026-07-13T11:00:00", "cancelled"),
        ],
    )
    .await;

    let fb = ask(json!([
        "BOB@example.org",
        "carol@example.org",
        "dave@example.org"
    ]))
    .await;
    let list = fb["list"].as_array().unwrap().clone();
    let bobs = tagged(&list, "bob@example.org");
    assert_eq!(
        bobs.len(),
        1,
        "one busy block from the shared calendar only: {fb}"
    );
    assert_eq!(bobs[0]["start"], "2026-07-13T09:00:00Z");
    assert_eq!(bobs[0]["status"], "busy");
    assert!(
        !list.iter().any(|b| b["start"] == "2026-07-13T15:00:00Z"),
        "the unshared calendar's event is not reported: {fb}"
    );
    assert!(
        !list.iter().any(|b| b["start"] == "2026-07-13T11:00:00Z"),
        "a cancelled event is not busy: {fb}"
    );
    for p in ["carol@example.org", "dave@example.org"] {
        assert_eq!(
            tagged(&list, p),
            [&json!({ "principal": p, "status": "unknown" })],
            "{p}: {fb}"
        );
    }
    // The caller's own blocks stay untagged (none here).
    assert!(list.iter().all(|b| b.get("principal").is_some()), "{fb}");
}

#[tokio::test]
async fn own_blocks_stay_untagged_and_cancelled_ones_are_not_busy() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let engine = Arc::new(Engine::new(store.clone()));
    let me = add_account(&store, &engine, "me@example.org", None).await;
    calendar_with(
        &engine,
        &me,
        "Mine",
        None,
        &[
            ("Busy", "2026-07-13T09:00:00", "confirmed"),
            ("Called off", "2026-07-13T12:00:00", "cancelled"),
        ],
    )
    .await;
    let fb = call(
        &engine,
        &me,
        "Calendar/freeBusy",
        json!({ "start": WINDOW.0, "end": WINDOW.1 }),
    )
    .await;
    let list = fb["list"].as_array().unwrap();
    assert_eq!(list.len(), 1, "{fb}");
    assert_eq!(list[0]["start"], "2026-07-13T09:00:00Z");
    assert!(list[0].get("principal").is_none());
}

/// A stub CalDAV server that does scheduling and answers for bob only.
async fn scheduling_stub() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        loop {
            let Ok((mut sock, _)) = listener.accept().await else {
                return;
            };
            tokio::spawn(async move {
                let mut buf = Vec::new();
                let mut tmp = [0u8; 4096];
                loop {
                    let head_end = loop {
                        if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                            break i + 4;
                        }
                        match sock.read(&mut tmp).await {
                            Ok(0) | Err(_) => return,
                            Ok(n) => buf.extend_from_slice(&tmp[..n]),
                        }
                    };
                    let head = String::from_utf8_lossy(&buf[..head_end]).to_string();
                    let len = head
                        .lines()
                        .find_map(|l| {
                            l.to_ascii_lowercase()
                                .strip_prefix("content-length:")
                                .map(|v| v.trim().parse::<usize>().unwrap_or(0))
                        })
                        .unwrap_or(0);
                    while buf.len() < head_end + len {
                        match sock.read(&mut tmp).await {
                            Ok(0) | Err(_) => return,
                            Ok(n) => buf.extend_from_slice(&tmp[..n]),
                        }
                    }
                    buf.drain(..head_end + len);
                    let mut first = head.lines().next().unwrap_or("").split_whitespace();
                    let method = first.next().unwrap_or("");
                    let path = first.next().unwrap_or("");
                    let reply = match (method, path) {
                        ("PROPFIND", "/.well-known/caldav") => "<d:multistatus xmlns:d=\"DAV:\"><d:response><d:propstat><d:prop><d:current-user-principal><d:href>/p/me/</d:href></d:current-user-principal></d:prop></d:propstat></d:response></d:multistatus>".to_string(),
                        ("PROPFIND", "/p/me/") => "<d:multistatus xmlns:d=\"DAV:\" xmlns:c=\"urn:ietf:params:xml:ns:caldav\"><d:response><d:propstat><d:prop><c:schedule-outbox-URL><d:href>/p/me/outbox/</d:href></c:schedule-outbox-URL></d:prop></d:propstat></d:response></d:multistatus>".to_string(),
                        ("POST", "/p/me/outbox/") => "<C:schedule-response xmlns:D=\"DAV:\" xmlns:C=\"urn:ietf:params:xml:ns:caldav\"><C:response><C:recipient><D:href>mailto:erin@remote.example</D:href></C:recipient><C:request-status>2.0;Success</C:request-status><C:calendar-data>BEGIN:VCALENDAR\r\nBEGIN:VFREEBUSY\r\nFREEBUSY;FBTYPE=BUSY:20260713T140000Z/20260713T150000Z\r\nEND:VFREEBUSY\r\nEND:VCALENDAR\r\n</C:calendar-data></C:response><C:response><C:recipient><D:href>mailto:frank@remote.example</D:href></C:recipient><C:request-status>3.7;Invalid calendar user</C:request-status></C:response></C:schedule-response>".to_string(),
                        _ => String::new(),
                    };
                    let status = if reply.is_empty() {
                        "404 Not Found"
                    } else if method == "POST" {
                        "200 OK"
                    } else {
                        "207 Multi-Status"
                    };
                    let out = format!(
                        "HTTP/1.1 {status}\r\nContent-Type: application/xml\r\nContent-Length: {}\r\n\r\n{reply}",
                        reply.len()
                    );
                    if sock.write_all(out.as_bytes()).await.is_err() {
                        return;
                    }
                }
            });
        }
    });
    base
}

#[tokio::test]
async fn an_address_off_this_server_is_asked_of_the_callers_caldav_server() {
    let base = scheduling_stub().await;
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let engine = Arc::new(Engine::new(store.clone()));
    let me = add_account(
        &store,
        &engine,
        "me@example.org",
        Some(mw_dav::DavConfig {
            base_url: base,
            username: "me@example.org".into(),
            password: "pw".into(),
            pin: None,
        }),
    )
    .await;
    let fb = call(
        &engine,
        &me,
        "Calendar/freeBusy",
        json!({ "start": WINDOW.0, "end": WINDOW.1,
                "principals": ["erin@remote.example", "frank@remote.example"] }),
    )
    .await;
    let list = fb["list"].as_array().unwrap().clone();
    let erin = tagged(&list, "erin@remote.example");
    assert_eq!(erin.len(), 1, "{fb}");
    assert_eq!(erin[0]["start"], "2026-07-13T14:00:00Z");
    assert_eq!(
        tagged(&list, "frank@remote.example"),
        [&json!({ "principal": "frank@remote.example", "status": "unknown" })],
        "the server could not answer for frank: {fb}"
    );
}
