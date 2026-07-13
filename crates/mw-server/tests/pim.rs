//! Integration tests for the V3 `mw-server` PIM endpoints (plan §3 e9): the
//! holiday feed, the PIM realtime push path, and the sharing-endpoint auth
//! surface. Everything runs over loopback against the in-repo mock JMAP upstream
//! (proxy mode) — the sharing *data* path rides the frozen engine PIM surface
//! (`handle_jmap` → `dispatch_pim`, filled by e8) and is proven end-to-end by
//! e12; here we cover the parts that do not depend on e8.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use futures_util::StreamExt;
use serde_json::{Value, json};

use mw_engine::StateChange;
use mw_server::{AppConfig, HardeningConfig, PushHandle, build_app_with_push};

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

async fn spawn_mock() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    format!("http://{addr}")
}

/// Spawn mw-server (proxy mode) and return (base URL, SocketAddr, push handle).
async fn spawn_server() -> (String, SocketAddr, PushHandle) {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let unique = format!(
        "{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    );
    let base = std::env::temp_dir().join(format!("mw-pim-test-{unique}"));
    let web_dir = base.join("web");
    std::fs::create_dir_all(&web_dir).unwrap();
    std::fs::write(web_dir.join("index.html"), INDEX_HTML).unwrap();

    let config = AppConfig {
        db_path: base.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: None,
        web_dir: Some(PathBuf::from(&web_dir)),
        cookie_secure: false,
        mode: mw_server::ServerMode::Proxy,
        hardening: HardeningConfig::default(),
    };
    let (app, push) = build_app_with_push(config).await.unwrap();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}"), addr, push)
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap()
}

async fn login(c: &reqwest::Client, server: &str, mock: &str) -> String {
    let resp = c
        .post(format!("{server}/api/login"))
        .json(&json!({
            "jmapUrl": mock,
            "username": mw_mock_jmap::USER,
            "password": mw_mock_jmap::PASS,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "login should succeed");
    set_cookie_pair(&resp, "mw_session").expect("login sets mw_session")
}

fn set_cookie_pair(resp: &reqwest::Response, name: &str) -> Option<String> {
    for v in resp.headers().get_all(reqwest::header::SET_COOKIE) {
        let s = v.to_str().ok()?;
        if let Some(rest) = s.split(';').next()
            && rest.trim_start().starts_with(&format!("{name}="))
        {
            return Some(rest.trim().to_string());
        }
    }
    None
}

// ── Holiday feed ────────────────────────────────────────────────────────────

#[tokio::test]
async fn holiday_regions_lists_subscribable_packs() {
    let mock = spawn_mock().await;
    let (server, _addr, _push) = spawn_server().await;
    let c = client();
    login(&c, &server, &mock).await;

    let resp = c
        .get(format!("{server}/api/holidays"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let regions = body["regions"].as_array().expect("regions array");
    assert!(!regions.is_empty(), "at least one bundled region");
    let us = regions
        .iter()
        .find(|r| r["id"] == "us")
        .expect("a 'us' region is bundled");
    assert!(us["count"].as_u64().unwrap() >= 1);
    assert_eq!(us["url"], "/api/holidays/us");
}

#[tokio::test]
async fn holiday_feed_serves_a_valid_ics() {
    let mock = spawn_mock().await;
    let (server, _addr, _push) = spawn_server().await;
    let c = client();
    login(&c, &server, &mock).await;

    let resp = c
        .get(format!("{server}/api/holidays/us"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let ctype = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(ctype.starts_with("text/calendar"), "got {ctype}");
    let ics = resp.text().await.unwrap();
    assert!(ics.starts_with("BEGIN:VCALENDAR"));
    assert!(ics.contains("BEGIN:VEVENT"));
    assert!(ics.contains("RRULE:FREQ=YEARLY"));
    assert!(ics.trim_end().ends_with("END:VCALENDAR"));
}

#[tokio::test]
async fn holiday_feed_unknown_region_is_404() {
    let mock = spawn_mock().await;
    let (server, _addr, _push) = spawn_server().await;
    let c = client();
    login(&c, &server, &mock).await;

    let resp = c
        .get(format!("{server}/api/holidays/atlantis"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn holiday_feed_requires_auth() {
    let (server, _addr, _push) = spawn_server().await;
    let c = client();
    let resp = c
        .get(format!("{server}/api/holidays"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401, "holiday endpoints are cookie-authed");
}

// ── PIM realtime push ───────────────────────────────────────────────────────

/// A change broadcast — the wire object is datatype-agnostic, so a PIM mutation
/// (e8 advances `Calendar`/`CalendarEvent`/… state and broadcasts a
/// `StateChange`) rides this exact verified path to the client. When e8 extends
/// `StateChange::to_wire` with the PIM `changed` keys they flow unchanged.
fn a_change() -> StateChange {
    StateChange {
        account_id: mw_mock_jmap::ACCOUNT_ID.to_string(),
        email: "7".into(),
        mailbox: "4".into(),
        submission: "1".into(),
        thread: "7".into(),
    }
}

#[tokio::test]
async fn pim_state_change_reaches_a_ws_client() {
    use tokio_tungstenite::tungstenite::Message;
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;
    use tokio_tungstenite::tungstenite::http::header::COOKIE;

    let mock = spawn_mock().await;
    let (server, addr, push) = spawn_server().await;
    let c = client();
    let cookie = login(&c, &server, &mock).await;

    let mut req = format!("ws://{addr}/jmap/ws")
        .into_client_request()
        .unwrap();
    req.headers_mut().insert(COOKIE, cookie.parse().unwrap());
    let (mut ws, _resp) = tokio_tungstenite::connect_async(req).await.unwrap();

    // Simulate the engine broadcasting after a PIM mutation.
    assert_eq!(push.send(a_change()), 1, "one WS subscriber connected");

    let frame = loop {
        let msg = tokio::time::timeout(Duration::from_secs(5), ws.next())
            .await
            .expect("a frame arrives")
            .expect("stream open")
            .expect("no ws error");
        match msg {
            Message::Text(t) => break t.to_string(),
            Message::Ping(_) | Message::Pong(_) => continue,
            other => panic!("unexpected frame: {other:?}"),
        }
    };
    let wire: Value = serde_json::from_str(&frame).unwrap();
    assert_eq!(wire["@type"], "StateChange");
    assert!(
        wire["changed"][mw_mock_jmap::ACCOUNT_ID].is_object(),
        "the change carries a per-account changed map"
    );
    drop(ws);
}

#[tokio::test]
async fn pim_state_change_reaches_an_sse_client() {
    let mock = spawn_mock().await;
    let (server, _addr, push) = spawn_server().await;
    let c = client();
    login(&c, &server, &mock).await;

    let resp = c
        .get(format!("{server}/jmap/eventsource"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers()
            .get(reqwest::header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap(),
        "text/event-stream"
    );

    assert_eq!(push.send(a_change()), 1);

    let mut stream = resp.bytes_stream();
    let mut buf = String::new();
    let found = tokio::time::timeout(Duration::from_secs(5), async {
        while let Some(chunk) = stream.next().await {
            buf.push_str(&String::from_utf8_lossy(&chunk.unwrap()));
            if buf.contains("StateChange") {
                return true;
            }
        }
        false
    })
    .await
    .unwrap_or(false);
    assert!(found, "SSE never delivered a StateChange; got: {buf:?}");
    assert!(buf.contains("data:"), "SSE frame must use data: lines");
}

// ── Sharing endpoints (auth surface; live data path is e8/e12) ──────────────

#[tokio::test]
async fn sharing_endpoints_require_auth() {
    let (server, _addr, _push) = spawn_server().await;
    let c = client();

    for path in ["/dav/calendars/acct/cal1", "/dav/addressbooks/acct/book1"] {
        let resp = c.get(format!("{server}{path}")).send().await.unwrap();
        assert_eq!(resp.status(), 401, "{path} must be cookie-authed");
    }
}

#[tokio::test]
async fn sharing_in_proxy_mode_is_not_implemented() {
    // Mailwoman-native collections live in the local engine store; a proxy
    // upstream has none to serve, so an authed request gets a clean 501.
    let mock = spawn_mock().await;
    let (server, _addr, _push) = spawn_server().await;
    let c = client();
    login(&c, &server, &mock).await;

    let resp = c
        .get(format!(
            "{server}/dav/calendars/{}/cal1",
            mw_mock_jmap::ACCOUNT_ID
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 501, "proxy mode has no native collections");
}
