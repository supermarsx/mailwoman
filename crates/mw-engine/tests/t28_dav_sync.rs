//! 26.20 t28-e19 (f34) — the four CalDAV/CardDAV sync hazards found in
//! `pim/sync.rs` (t28 §1.10, re-verified at the start of t28-e19), each against a
//! fake DAV server that records every request:
//!
//! 1. a webcal overlay calendar shares `caldav_url`: no REPORT, PUT or DELETE may
//!    ever go to it;
//! 2. hrefs were built as `<collection>/<uid>.ics` instead of stored: an edit of a
//!    resource the server named otherwise went to a new href (a duplicate) and a
//!    delete missed;
//! 3. no ctag/etag fallback when `sync-collection` is unsupported, and a server-side
//!    delete was never seen by the CardDAV fallback;
//! 4. the pull re-emitted a lossy serialisation that the next local edit PUT back:
//!    the server's body is kept, and a local edit is merged into it.
//!
//! Plus: a CardDAV pull keeps the fields the engine owns (favourite).

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

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
use mw_store::{
    AccountKind, AddressBookRow, CalendarRow, Credentials, NewAccount, ServerKey, Store,
};

// ── a fake DAV server ─────────────────────────────────────────────────────────

#[derive(Default)]
struct FakeState {
    /// href → (etag, body), every collection together.
    resources: BTreeMap<String, (String, String)>,
    /// hrefs deleted on the server since the last sync-collection answer.
    tombstones: Vec<String>,
    /// Whether REPORT sync-collection is supported.
    sync_supported: bool,
    /// Bumped on every change; the ctag and the etag source.
    version: u64,
    /// Every request: (method, path, body).
    log: Vec<(String, String, String)>,
}

#[derive(Clone, Default)]
struct FakeDav(Arc<Mutex<FakeState>>);

impl FakeDav {
    fn put(&self, href: &str, body: &str) {
        let mut st = self.0.lock().unwrap();
        st.version += 1;
        let etag = format!("\"v{}\"", st.version);
        st.resources.insert(href.into(), (etag, body.into()));
    }
    fn delete(&self, href: &str) {
        let mut st = self.0.lock().unwrap();
        st.version += 1;
        st.resources.remove(href);
        st.tombstones.push(href.into());
    }
    fn body(&self, href: &str) -> Option<String> {
        self.0
            .lock()
            .unwrap()
            .resources
            .get(href)
            .map(|r| r.1.clone())
    }
    fn hrefs(&self) -> Vec<String> {
        self.0.lock().unwrap().resources.keys().cloned().collect()
    }
    fn requests(&self, method: &str, path_prefix: &str) -> Vec<(String, String)> {
        self.0
            .lock()
            .unwrap()
            .log
            .iter()
            .filter(|(m, p, _)| m == method && p.starts_with(path_prefix))
            .map(|(_, p, b)| (p.clone(), b.clone()))
            .collect()
    }
    fn reports(&self, path_prefix: &str, kind: &str) -> usize {
        self.requests("REPORT", path_prefix)
            .iter()
            .filter(|(_, b)| b.contains(kind))
            .count()
    }

    fn answer(&self, method: &str, path: &str, body: &str) -> (u16, Vec<(String, String)>, String) {
        let mut st = self.0.lock().unwrap();
        st.log.push((method.into(), path.into(), body.into()));
        let xml_resp = |href: &str, etag: Option<&str>, data: Option<(&str, &str)>| {
            let mut prop = String::new();
            if let Some(e) = etag {
                prop.push_str(&format!("<d:getetag>{}</d:getetag>", xml(e)));
            }
            if let Some((elem, d)) = data {
                prop.push_str(&format!("<c:{elem}>{}</c:{elem}>", xml(d)));
            }
            format!(
                "<d:response><d:href>{href}</d:href><d:propstat><d:prop>{prop}</d:prop>\
                 <d:status>HTTP/1.1 200 OK</d:status></d:propstat></d:response>"
            )
        };
        let ms = |inner: String, extra: &str| {
            format!(
                "<?xml version=\"1.0\"?><d:multistatus xmlns:d=\"DAV:\" \
                 xmlns:c=\"urn:ietf:params:xml:ns:caldav\" \
                 xmlns:cs=\"http://calendarserver.org/ns/\">{inner}{extra}</d:multistatus>"
            )
        };
        match method {
            "REPORT" if body.contains("sync-collection") => {
                if !st.sync_supported {
                    return (501, vec![], String::new());
                }
                let mut inner = String::new();
                for (href, (etag, _)) in st.resources.iter().filter(|(h, _)| h.starts_with(path)) {
                    inner.push_str(&xml_resp(href, Some(etag), None));
                }
                let gone: Vec<String> = st
                    .tombstones
                    .iter()
                    .filter(|h| h.starts_with(path))
                    .cloned()
                    .collect();
                for href in gone {
                    inner.push_str(&format!(
                        "<d:response><d:href>{href}</d:href><d:status>HTTP/1.1 404 Not Found</d:status></d:response>"
                    ));
                }
                st.tombstones.retain(|h| !h.starts_with(path));
                let token = format!("<d:sync-token>tok-{}</d:sync-token>", st.version);
                (207, vec![], ms(inner, &token))
            }
            "REPORT" if body.contains("multiget") => {
                let elem = if body.contains("addressbook-multiget") {
                    "address-data"
                } else {
                    "calendar-data"
                };
                let mut inner = String::new();
                for part in body.split("<d:href>").skip(1) {
                    let href = part.split("</d:href>").next().unwrap_or_default();
                    if let Some((etag, data)) = st.resources.get(href) {
                        inner.push_str(&xml_resp(href, Some(etag), Some((elem, data))));
                    }
                }
                (207, vec![], ms(inner, ""))
            }
            "PROPFIND" => {
                let mut inner = format!(
                    "<d:response><d:href>{path}</d:href><d:propstat><d:prop>\
                     <cs:getctag>ctag-{}</cs:getctag></d:prop>\
                     <d:status>HTTP/1.1 200 OK</d:status></d:propstat></d:response>",
                    st.version
                );
                for (href, (etag, _)) in st.resources.iter().filter(|(h, _)| h.starts_with(path)) {
                    inner.push_str(&xml_resp(href, Some(etag), None));
                }
                (207, vec![], ms(inner, ""))
            }
            "PUT" => {
                st.version += 1;
                let etag = format!("\"v{}\"", st.version);
                st.resources
                    .insert(path.into(), (etag.clone(), body.into()));
                (201, vec![("ETag".into(), etag)], String::new())
            }
            "DELETE" => {
                st.version += 1;
                st.resources.remove(path);
                (204, vec![], String::new())
            }
            _ => (404, vec![], String::new()),
        }
    }
}

fn xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

async fn spawn_fake(sync_supported: bool) -> (String, FakeDav) {
    let fake = FakeDav::default();
    fake.0.lock().unwrap().sync_supported = sync_supported;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = fake.clone();
    tokio::spawn(async move {
        loop {
            let Ok((mut sock, _)) = listener.accept().await else {
                return;
            };
            let server = server.clone();
            tokio::spawn(async move {
                let mut buf = Vec::new();
                let mut tmp = [0u8; 8192];
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
                    let body = String::from_utf8_lossy(&buf[head_end..head_end + len]).to_string();
                    buf.drain(..head_end + len);
                    let mut first = head.lines().next().unwrap_or("").split_whitespace();
                    let method = first.next().unwrap_or("").to_string();
                    let path = first.next().unwrap_or("").to_string();
                    let (status, headers, reply) = server.answer(&method, &path, &body);
                    let mut out =
                        format!("HTTP/1.1 {status} X\r\nContent-Length: {}\r\n", reply.len());
                    for (k, v) in headers {
                        out.push_str(&format!("{k}: {v}\r\n"));
                    }
                    out.push_str("Content-Type: application/xml\r\n\r\n");
                    out.push_str(&reply);
                    if sock.write_all(out.as_bytes()).await.is_err() {
                        return;
                    }
                }
            });
        }
    });
    (base, fake)
}

// ── engine harness ────────────────────────────────────────────────────────────

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

struct Harness {
    engine: Arc<Engine>,
    store: Store,
    account: String,
    rt: AccountRuntime,
    fake: FakeDav,
}

async fn harness(sync_supported: bool) -> Harness {
    let (base, fake) = spawn_fake(sync_supported).await;
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account = store
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
    let engine = Arc::new(Engine::new(store.clone()));
    let rt = AccountRuntime::new(
        Arc::new(NoopBackend) as Arc<dyn AccountBackend>,
        Arc::new(NullSubmitter) as Arc<dyn MailSubmitter>,
        "me@example.org",
    )
    .with_dav(mw_dav::DavConfig {
        base_url: base,
        username: "me@example.org".into(),
        password: "pw".into(),
        pin: None,
    });
    engine.register_backend(account.clone(), rt.clone());
    Harness {
        engine,
        store,
        account,
        rt,
        fake,
    }
}

impl Harness {
    async fn calendar(&self, id: &str, url: &str, overlay: bool) {
        self.store
            .upsert_calendar(&CalendarRow {
                id: id.into(),
                account_id: self.account.clone(),
                name: id.into(),
                color: "#3366ff".into(),
                sort_order: 0,
                is_visible: true,
                role: None,
                caldav_url: Some(url.into()),
                sync_token: None,
                ctag: None,
                is_overlay: overlay,
                component: "VEVENT".into(),
            })
            .await
            .unwrap();
    }

    async fn call(&self, method: &str, args: Value) -> Value {
        let resp = self
            .engine
            .handle_jmap(
                &self.account,
                &json!({ "methodCalls": [[method, args, "c0"]] }),
            )
            .await;
        resp["methodResponses"][0][1].clone()
    }

    async fn sync(&self) {
        self.engine.sync_pim(&self.account, &self.rt).await.unwrap();
    }

    async fn event_ids(&self, cal: &str) -> Vec<String> {
        self.store
            .list_events(cal)
            .await
            .unwrap()
            .into_iter()
            .map(|e| e.id)
            .collect()
    }
}

fn vevent(uid: &str, summary: &str, extra: &str) -> String {
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Fake//EN\r\nX-WR-CALNAME:Work\r\n\
         BEGIN:VEVENT\r\nUID:{uid}\r\nDTSTAMP:20260701T000000Z\r\n\
         DTSTART:20260713T090000Z\r\nDTEND:20260713T100000Z\r\nSUMMARY:{summary}\r\n{extra}\
         END:VEVENT\r\nEND:VCALENDAR\r\n"
    )
}

// ── 1. overlays ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn a_webcal_overlay_gets_no_report_put_or_delete() {
    let h = harness(true).await;
    h.calendar("dav", "/cal/", false).await;
    h.calendar("feed", "/feeds/holidays.ics", true).await;
    h.fake
        .put("/feeds/holidays.ics/x.ics", &vevent("x", "Holiday", ""));

    h.sync().await;
    assert!(
        h.fake.reports("/cal/", "sync-collection") >= 1,
        "control: the DAV calendar is pulled"
    );
    assert_eq!(
        h.fake.reports("/feeds/", ""),
        0,
        "no REPORT reaches the overlay"
    );

    // A local event in the overlay calendar is not pushed, and its delete sends nothing.
    let made = h
        .call(
            "CalendarEvent/set",
            json!({ "create": { "e": { "calendarId": "feed", "title": "Local", "start": "2026-07-14T09:00:00", "timeZone": "UTC", "duration": "PT1H" } } }),
        )
        .await;
    let id = made["created"]["e"]["id"]
        .as_str()
        .expect("created")
        .to_string();
    h.call("CalendarEvent/set", json!({ "destroy": [id] }))
        .await;
    assert!(
        h.fake.requests("PUT", "/feeds/").is_empty(),
        "no PUT to the overlay"
    );
    assert!(
        h.fake.requests("DELETE", "/feeds/").is_empty(),
        "no DELETE to the overlay"
    );

    // Control: the same in the DAV calendar does reach the server.
    let made = h
        .call(
            "CalendarEvent/set",
            json!({ "create": { "e": { "calendarId": "dav", "title": "Pushed", "start": "2026-07-14T09:00:00", "timeZone": "UTC", "duration": "PT1H" } } }),
        )
        .await;
    assert!(made["created"]["e"]["id"].is_string());
    assert_eq!(h.fake.requests("PUT", "/cal/").len(), 1);
}

// ── 2 + 4. server hrefs and server bodies ─────────────────────────────────────

#[tokio::test]
async fn an_edit_goes_to_the_servers_href_and_keeps_what_the_projection_does_not_model() {
    let h = harness(true).await;
    h.calendar("dav", "/cal/", false).await;
    // The server names the resource its own way, not `<uid>.ics`.
    h.fake.put(
        "/cal/7f3a-server-name.ics",
        &vevent(
            "uid-1@example.org",
            "Planning",
            "X-MW-KEEP:unknown property\r\nCATEGORIES:Board\r\n",
        ),
    );
    h.sync().await;
    let ids = h.event_ids("dav").await;
    assert_eq!(ids.len(), 1, "pulled");
    let row = h.store.get_event(&ids[0]).await.unwrap().unwrap();
    assert!(
        row.ical_raw.contains("X-MW-KEEP:unknown property"),
        "the pull stores the server's body, not a re-emission: {}",
        row.ical_raw
    );

    // Local edit: PUT to the server's href, with the unknown property kept.
    let r = h
        .call(
            "CalendarEvent/set",
            json!({ "update": { &ids[0]: { "title": "Planning (moved)" } } }),
        )
        .await;
    assert!(
        r["updated"]
            .as_object()
            .is_some_and(|u| u.contains_key(&ids[0])),
        "{r}"
    );
    let puts = h.fake.requests("PUT", "/cal/");
    assert_eq!(puts.len(), 1, "{puts:?}");
    assert_eq!(
        puts[0].0, "/cal/7f3a-server-name.ics",
        "the edit goes to the server's href"
    );
    assert!(
        puts[0].1.contains("SUMMARY:Planning (moved)"),
        "{}",
        puts[0].1
    );
    assert!(
        puts[0].1.contains("X-MW-KEEP:unknown property"),
        "kept: {}",
        puts[0].1
    );
    assert!(
        puts[0].1.contains("CATEGORIES:Board"),
        "kept: {}",
        puts[0].1
    );
    assert!(
        puts[0].1.contains("X-WR-CALNAME:Work"),
        "calendar-level kept: {}",
        puts[0].1
    );
    assert_eq!(
        h.fake.hrefs(),
        vec!["/cal/7f3a-server-name.ics".to_string()],
        "no duplicate resource was created"
    );

    // Local delete: the server's href.
    h.call("CalendarEvent/set", json!({ "destroy": [&ids[0]] }))
        .await;
    assert_eq!(
        h.fake.requests("DELETE", "/cal/"),
        vec![("/cal/7f3a-server-name.ics".to_string(), String::new())]
    );
}

#[tokio::test]
async fn a_delete_on_the_server_removes_the_event_by_its_stored_href() {
    let h = harness(true).await;
    h.calendar("dav", "/cal/", false).await;
    h.fake
        .put("/cal/abc.ics", &vevent("uid-not-the-name", "One", ""));
    h.sync().await;
    assert_eq!(h.event_ids("dav").await.len(), 1);
    h.fake.delete("/cal/abc.ics");
    h.sync().await;
    assert!(
        h.event_ids("dav").await.is_empty(),
        "the tombstone href matched the stored href, not a basename uid"
    );
}

// ── 3. ctag / etag fallback ──────────────────────────────────────────────────

#[tokio::test]
async fn without_sync_collection_the_pull_diffs_etags() {
    let h = harness(false).await;
    h.calendar("dav", "/cal/", false).await;
    h.fake.put("/cal/a.ics", &vevent("a", "A", ""));
    h.fake.put("/cal/b.ics", &vevent("b", "B", ""));

    h.sync().await;
    assert_eq!(
        h.event_ids("dav").await.len(),
        2,
        "both pulled through the etag listing"
    );
    assert_eq!(h.fake.reports("/cal/", "multiget"), 1);

    // Nothing changed (same ctag): no multiget.
    h.sync().await;
    assert_eq!(
        h.fake.reports("/cal/", "multiget"),
        1,
        "an unchanged collection is not re-fetched"
    );

    // One changed, one deleted: only the changed one is fetched, the deleted one goes.
    h.fake.put("/cal/a.ics", &vevent("a", "A changed", ""));
    h.fake.delete("/cal/b.ics");
    h.sync().await;
    let multigets = h.fake.requests("REPORT", "/cal/");
    let last = &multigets.last().unwrap().1;
    assert!(
        last.contains("/cal/a.ics") && !last.contains("/cal/b.ics"),
        "{last}"
    );
    let ids = h.event_ids("dav").await;
    assert_eq!(ids.len(), 1, "b removed");
    let a = h.store.get_event(&ids[0]).await.unwrap().unwrap();
    assert!(a.ical_raw.contains("SUMMARY:A changed"));
}

// ── CardDAV: deletes seen without sync-collection; engine-owned fields kept ───

fn vcard(uid: &str, name: &str, extra: &str) -> String {
    format!(
        "BEGIN:VCARD\r\nVERSION:3.0\r\nUID:{uid}\r\nFN:{name}\r\nN:{name};;;;\r\n\
         EMAIL:{uid}@example.org\r\n{extra}END:VCARD\r\n"
    )
}

#[tokio::test]
async fn carddav_pull_keeps_favourites_and_the_photo_and_sees_deletes() {
    let h = harness(false).await;
    h.store
        .upsert_address_book(&AddressBookRow {
            id: "book".into(),
            account_id: h.account.clone(),
            name: "Contacts".into(),
            is_default: false,
            carddav_url: Some("/book/".into()),
            sync_token: None,
            ctag: None,
        })
        .await
        .unwrap();
    h.fake.put(
        "/book/c1.vcf",
        &vcard(
            "ann",
            "Ann",
            "PHOTO;VALUE=uri:https://img.example/ann.jpg\r\nX-CUSTOM:keep\r\n",
        ),
    );
    h.fake.put("/book/c2.vcf", &vcard("bob", "Bob", ""));
    h.sync().await;
    let contacts = h.store.list_contacts("book").await.unwrap();
    assert_eq!(contacts.len(), 2);
    let ann = contacts.iter().find(|c| c.uid == "ann").unwrap().clone();

    // Favourite locally, then the server changes the card: the favourite stays.
    h.call(
        "ContactCard/set",
        json!({ "update": { &ann.id: { "isFavorite": true } } }),
    )
    .await;
    let puts_before = h.fake.requests("PUT", "/book/").len();
    let put = h.fake.requests("PUT", "/book/").last().cloned();
    if let Some((href, body)) = put {
        assert_eq!(href, "/book/c1.vcf", "the edit goes to the server's href");
        assert!(body.contains("X-CUSTOM:keep"), "kept: {body}");
        assert!(
            body.contains("PHOTO;VALUE=uri:https://img.example/ann.jpg"),
            "kept: {body}"
        );
    }
    h.fake.put(
        "/book/c1.vcf",
        &vcard("ann", "Ann Lee", "X-CUSTOM:keep\r\n"),
    );
    h.sync().await;
    let ann_after = h.store.get_contact(&ann.id).await.unwrap().unwrap();
    assert!(
        ann_after.full_name.contains("Ann Lee"),
        "the server change was pulled"
    );
    assert!(
        ann_after.is_favorite,
        "the engine-owned favourite survives a pull"
    );
    assert!(puts_before >= 1, "the favourite edit was pushed");

    // Deleted on a server without sync-collection: removed here.
    h.fake.delete("/book/c2.vcf");
    h.sync().await;
    let left: Vec<String> = h
        .store
        .list_contacts("book")
        .await
        .unwrap()
        .into_iter()
        .map(|c| c.uid)
        .collect();
    assert_eq!(left, vec!["ann".to_string()]);
    assert!(h.fake.body("/book/c1.vcf").is_some());
}
