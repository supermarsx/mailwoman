//! Browser error reports over the real router: anonymous and signed-in ingest,
//! validation, rate limits, grouping by fingerprint, regression reopening, the
//! person's opt-out and the administrator's switches, symbolication with a tiny
//! source map, hidden maps never served, and the admin list, detail, status and
//! delete routes with their permission checks.
//!
//! SQLite only. Settings are stored before the app boots, as the settings API
//! would store them. The ingest rate limiters are process-wide; the tests that
//! use the anonymous route share one address, so they live in one test.

use std::net::SocketAddr;

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::settings_policy::SettingPolicyRow;
use serde_json::{Value, json};
use tokio::net::TcpListener;

mod common;
use common::test_db;

const KEY_HEX: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";
/// `index-Ab12.js`: column 0 of line 0 is `main` in `src/app.ts` 1:1; column 10 is 2:5;
/// line 1 column 4 is 4:5.
const MAP: &str = r#"{"version":3,"file":"index-Ab12.js","sources":["../../src/app.ts","../../src/util.ts"],"names":["main"],"mappings":"AAAAA,UACI;IAEAA"}"#;

struct Server {
    base: String,
    db: String,
}

async fn spawn(settings: &[(&str, Value)], strict: bool) -> Server {
    let upstream_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let upstream = format!("http://{}", upstream_listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(upstream_listener, mw_mock_jmap::router()).await.unwrap();
    });
    let dir = test_db::unique_dir("mw-client-errors");
    let web = dir.join("web");
    std::fs::create_dir_all(web.join("assets")).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    std::fs::write(web.join("assets/index-Ab12.js"), "console.log(1)").unwrap();
    std::fs::write(web.join("assets/index-Ab12.js.map"), MAP).unwrap();
    let config = AppConfig {
        db_path: dir.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig { csrf_strict: strict, ..HardeningConfig::default() },
        security: SecurityConfig {
            jmap_upstreams: Some(vec![upstream.clone()]),
            upstream_default: Some(upstream),
            ..SecurityConfig::default()
        },
    };
    let db = config.db_path.clone();
    let store = mw_store::Store::open(&config.db_path, mw_store::ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap();
    for (key, value) in settings {
        store
            .put_setting_policy(&SettingPolicyRow {
                key: (*key).into(),
                global_value: Some(value.to_string()),
                allow_user_override: *key == "privacy.errorReports",
                global_locked: false,
                updated_by: "test-fixture".into(),
                updated_at: 0,
            })
            .await
            .unwrap();
    }
    drop(store);
    let v6 = V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("hunter2".into()),
        redis_url: None,
    };
    let app = build_app_full(config, v6).await.expect("server boots").0;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await;
    });
    Server { base: format!("http://{addr}"), db }
}

fn client() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().redirect(reqwest::redirect::Policy::none()).build().unwrap()
}

async fn admin(c: &reqwest::Client, base: &str) -> String {
    let resp = c
        .post(format!("{base}/admin/login"))
        .json(&json!({"username": "root", "password": "hunter2"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    resp.headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .map(|v| v.to_str().unwrap().split(';').next().unwrap().to_owned())
        .collect::<Vec<_>>()
        .join("; ")
}

/// A signed-in mail session: a cookie jar client, its CSRF token and storage authority.
async fn user(base: &str) -> (reqwest::Client, String, String) {
    let c = reqwest::Client::builder().no_proxy().cookie_store(true).build().unwrap();
    let resp = c
        .post(format!("{base}/api/login"))
        .json(&json!({"username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    (
        c,
        body["csrfToken"].as_str().unwrap().to_owned(),
        body["storageAuthority"].as_str().unwrap_or_default().to_owned(),
    )
}

fn report(fingerprint: &str, build: &str, over: Value) -> Value {
    let mut r = json!({
        "id": uuid::Uuid::new_v4().to_string(), "kind": "boundary", "name": "TypeError",
        "message": "x is undefined", "stack": "TypeError: x is undefined\n    at h (/assets/index-Ab12.js:1:1)\n    at /assets/index-Ab12.js:1:13",
        "boundary": "Reader/AttachmentViewer/text", "route": "/mail/INBOX/:id", "build": build,
        "fingerprint": fingerprint, "count": 1, "visit": "v1a2b3c4d5e6", "uaFamily": "Chrome", "uaVersion": "126",
        "osFamily": "Windows", "viewport": "wide", "locale": "en", "online": true, "reducedMotion": false, "theme": "paper",
    });
    for (k, v) in over.as_object().cloned().unwrap_or_default() {
        r[k] = v;
    }
    r
}

async fn post_session(c: &reqwest::Client, base: &str, token: &str, reports: Vec<Value>) -> reqwest::Response {
    c.post(format!("{base}/api/observability/client-errors/session"))
        .header("x-csrf-token", token)
        .json(&json!({ "reports": reports }))
        .send()
        .await
        .unwrap()
}

async fn groups(c: &reqwest::Client, base: &str, cookie: &str, query: &str) -> Value {
    let r = c.get(format!("{base}/admin/client-errors{query}")).header("Cookie", cookie).send().await.unwrap();
    assert_eq!(r.status(), 200);
    r.json().await.unwrap()
}

#[tokio::test]
async fn signed_in_reports_group_by_fingerprint_and_resolve_reopens_on_a_new_build() {
    let s = spawn(&[], false).await;
    let (c, token, _) = user(&s.base).await;
    let a = client();
    let cookie = admin(&a, &s.base).await;

    let r = post_session(&c, &s.base, &token, vec![report("aaaaaaaa11", "26.18+one", json!({})), report("aaaaaaaa11", "26.18+one", json!({"count": 4})), report("bbbbbbbb22", "26.18+one", json!({"name": "RangeError", "boundary": "Settings/privacy"}))]).await;
    assert_eq!(r.status(), 200);
    assert_eq!(r.json::<Value>().await.unwrap()["reporting"], "on");

    let list = groups(&a, &s.base, &cookie, "").await;
    let rows = list["groups"].as_array().unwrap();
    assert_eq!(rows.len(), 2, "{list}");
    let same = rows.iter().find(|g| g["fingerprint"] == "aaaaaaaa11").unwrap();
    assert_eq!(same["count"], 5);
    assert_eq!(same["sessions"], 1);
    assert_eq!(same["status"], "open");
    assert_eq!(same["routeTemplate"], "/mail/INBOX/:id");
    assert_eq!(same["lastBuild"], "26.18+one");
    assert!(same["series"].is_array());
    let id = same["id"].as_str().unwrap().to_owned();

    // Filters.
    assert_eq!(groups(&a, &s.base, &cookie, "?boundary=Settings/privacy").await["groups"].as_array().unwrap().len(), 1);
    assert_eq!(groups(&a, &s.base, &cookie, "?status=resolved").await["groups"].as_array().unwrap().len(), 0);
    assert_eq!(a.get(format!("{}/admin/client-errors?status=bogus", s.base)).header("Cookie", &cookie).send().await.unwrap().status(), 422);

    // Resolve while build one is current; the same build again does not reopen it.
    let resolve = a.post(format!("{}/admin/client-errors/{id}/status", s.base)).header("Cookie", &cookie).json(&json!({"status": "resolved"})).send().await.unwrap();
    assert_eq!(resolve.status(), 204);
    post_session(&c, &s.base, &token, vec![report("aaaaaaaa11", "26.18+one", json!({}))]).await;
    let detail: Value = a.get(format!("{}/admin/client-errors/{id}", s.base)).header("Cookie", &cookie).send().await.unwrap().json().await.unwrap();
    assert_eq!(detail["group"]["status"], "resolved");
    assert_eq!(detail["group"]["resolvedBuild"], "26.18+one");
    // A build that never saw it reopens it: a regression.
    post_session(&c, &s.base, &token, vec![report("aaaaaaaa11", "26.19+two", json!({}))]).await;
    let detail: Value = a.get(format!("{}/admin/client-errors/{id}", s.base)).header("Cookie", &cookie).send().await.unwrap().json().await.unwrap();
    assert_eq!(detail["group"]["status"], "open");
    assert_eq!(detail["group"]["reopenedCount"], 1);
    assert_eq!(detail["group"]["lastBuild"], "26.19+two");
    assert!(detail["events"].as_array().unwrap().len() >= 2);
    assert!(detail["breakdown"]["browser"].as_array().unwrap().iter().any(|row| row[0] == "Chrome"));
    assert!(detail["breakdown"]["route"].as_array().unwrap().iter().any(|row| row[0] == "/mail/INBOX/:id"));

    // Ignored groups keep counting but store no events.
    a.post(format!("{}/admin/client-errors/{id}/status", s.base)).header("Cookie", &cookie).json(&json!({"status": "ignored"})).send().await.unwrap();
    let before = detail["events"].as_array().unwrap().len();
    post_session(&c, &s.base, &token, vec![report("aaaaaaaa11", "26.19+two", json!({}))]).await;
    let detail: Value = a.get(format!("{}/admin/client-errors/{id}", s.base)).header("Cookie", &cookie).send().await.unwrap().json().await.unwrap();
    assert_eq!(detail["events"].as_array().unwrap().len(), before);
    assert_eq!(detail["group"]["count"], 8);

    // Export and delete.
    let csv = a.get(format!("{}/admin/client-errors/export?format=csv", s.base)).header("Cookie", &cookie).send().await.unwrap();
    assert_eq!(csv.status(), 200);
    assert!(csv.text().await.unwrap().starts_with("id,status,kind,name"));
    assert_eq!(a.delete(format!("{}/admin/client-errors/{id}", s.base)).header("Cookie", &cookie).send().await.unwrap().status(), 204);
    assert_eq!(a.get(format!("{}/admin/client-errors/{id}", s.base)).header("Cookie", &cookie).send().await.unwrap().status(), 404);
    assert_eq!(a.post(format!("{}/admin/client-errors/{id}/status", s.base)).header("Cookie", &cookie).json(&json!({"status": "open"})).send().await.unwrap().status(), 404);
}

#[tokio::test]
async fn admin_routes_need_an_admin_session() {
    let s = spawn(&[], false).await;
    let a = client();
    for path in ["/admin/client-errors", "/admin/client-errors/x", "/admin/client-errors/export"] {
        let r = a.get(format!("{}{path}", s.base)).send().await.unwrap();
        assert_eq!(r.status(), 401, "{path}");
    }
    let r = a.post(format!("{}/admin/client-errors/x/status", s.base)).json(&json!({"status": "open"})).send().await.unwrap();
    assert!(matches!(r.status().as_u16(), 401 | 403), "{}", r.status());
    // A mail user's session is not an admin session.
    let (c, _, _) = user(&s.base).await;
    let r = c.get(format!("{}/admin/client-errors", s.base)).send().await.unwrap();
    assert_eq!(r.status(), 401);
}

#[tokio::test]
async fn stack_frames_are_resolved_with_the_hidden_map_which_is_never_served() {
    let s = spawn(&[], false).await;
    let (c, token, _) = user(&s.base).await;
    let a = client();
    let cookie = admin(&a, &s.base).await;
    assert_eq!(post_session(&c, &s.base, &token, vec![report("cccccccc33", "26.18+one", json!({}))]).await.status(), 200);
    let id = groups(&a, &s.base, &cookie, "").await["groups"][0]["id"].as_str().unwrap().to_owned();
    let detail: Value = a.get(format!("{}/admin/client-errors/{id}", s.base)).header("Cookie", &cookie).send().await.unwrap().json().await.unwrap();
    let event = &detail["events"][0];
    assert!(event["stackRaw"].as_str().unwrap().contains("/assets/index-Ab12.js:1:1"), "{event}");
    let resolved = event["stackSymbolicated"].as_str().expect("symbolicated").to_owned();
    assert!(resolved.contains("at main (src/app.ts:1:1)"), "{resolved}");
    assert!(resolved.contains("src/app.ts:2:5"), "{resolved}");
    assert!(!resolved.contains("index-Ab12.js"), "{resolved}");

    // The bundle is served; its map is not, under any spelling.
    assert_eq!(a.get(format!("{}/assets/index-Ab12.js", s.base)).send().await.unwrap().status(), 200);
    for path in ["/assets/index-Ab12.js.map", "/assets/%69ndex-Ab12.js.map"] {
        let r = a.get(format!("{}{path}", s.base)).send().await.unwrap();
        assert!(matches!(r.status().as_u16(), 404 | 400), "{path}: {}", r.status());
    }

    // With source maps off the raw stack is kept alone.
    let off = spawn(&[("observability.client.sourceMaps", json!(false))], false).await;
    let (c2, t2, _) = user(&off.base).await;
    let cookie2 = admin(&a, &off.base).await;
    post_session(&c2, &off.base, &t2, vec![report("dddddddd44", "26.18+one", json!({}))]).await;
    let id2 = groups(&a, &off.base, &cookie2, "").await["groups"][0]["id"].as_str().unwrap().to_owned();
    let d2: Value = a.get(format!("{}/admin/client-errors/{id2}", off.base)).header("Cookie", &cookie2).send().await.unwrap().json().await.unwrap();
    assert!(d2["events"][0]["stackSymbolicated"].is_null());
}

#[tokio::test]
async fn reports_are_scrubbed_again_on_arrival() {
    let s = spawn(&[], false).await;
    let (c, token, _) = user(&s.base).await;
    let a = client();
    let cookie = admin(&a, &s.base).await;
    let dirty = report("eeeeeeee55", "26.18+one", json!({
        "message": "cannot send to ana@example.test with key eyJhbGciOiJIUzI1NiJ9abcdef0123456789 at https://h.test/x?token=SECRET#frag",
        "stack": "Error: boom\n    at f (https://mail.example.test/assets/other.js?v=1&session=SECRET:1:2)",
        "route": "#/mail/INBOX/m-0a1b2c3d4e5f6a7b?q=secret",
        "boundary": "Reader/<script>alert(1)</script>",
    }));
    assert_eq!(post_session(&c, &s.base, &token, vec![dirty]).await.status(), 200);
    let id = groups(&a, &s.base, &cookie, "").await["groups"][0]["id"].as_str().unwrap().to_owned();
    let detail = a.get(format!("{}/admin/client-errors/{id}", s.base)).header("Cookie", &cookie).send().await.unwrap().text().await.unwrap();
    for leak in ["ana@example.test", "eyJhbGci", "SECRET", "q=secret", "mail.example.test", "frag", "<script>"] {
        assert!(!detail.contains(leak), "{leak} leaked: {detail}");
    }
    assert!(detail.contains("/mail/INBOX/:id"));
}

#[tokio::test]
async fn the_session_route_validates_and_checks_csrf() {
    let s = spawn(&[], true).await;
    let (c, token, _) = user(&s.base).await;
    let ok = report("ffffffff66", "26.18+one", json!({}));
    // CSRF: the session route is checked, the anonymous one is not.
    let r = c.post(format!("{}/api/observability/client-errors/session", s.base)).json(&json!({"reports": [ok.clone()]})).send().await.unwrap();
    assert_eq!(r.status(), 403);
    // Bad bodies are 400 and never stored.
    for body in [
        json!({"reports": []}),
        json!({"reports": [report("fffffff", "b", json!({"fingerprint": "zz"}))]}),
        json!({"reports": [report("ffffffff66", "b", json!({"kind": "nope"}))]}),
        json!({"reports": [report("ffffffff66", "b", json!({"name": ""}))]}),
        json!({"reports": (0..21).map(|i| report(&format!("ab{i:06}"), "b", json!({}))).collect::<Vec<_>>()}),
    ] {
        let r = c.post(format!("{}/api/observability/client-errors/session", s.base)).header("x-csrf-token", &token).json(&body).send().await.unwrap();
        assert_eq!(r.status(), 400, "{body}");
    }
    let r = c.post(format!("{}/api/observability/client-errors/session", s.base)).header("x-csrf-token", &token).header("content-type", "application/json").body("nope").send().await.unwrap();
    assert_eq!(r.status(), 400);
    // Over 20 reports' worth of body is refused before parsing.
    let huge = json!({"reports": [report("ffffffff66", "b", json!({"stack": "x".repeat(400_000)}))]});
    let r = c.post(format!("{}/api/observability/client-errors/session", s.base)).header("x-csrf-token", &token).json(&huge).send().await.unwrap();
    assert_eq!(r.status(), 413);
    // A single report over 16 KB is a 400.
    let big = json!({"reports": [report("ffffffff66", "b", json!({"stack": "y".repeat(20_000)}))]});
    let r = c.post(format!("{}/api/observability/client-errors/session", s.base)).header("x-csrf-token", &token).json(&big).send().await.unwrap();
    assert_eq!(r.status(), 400);
    // No session: 401 (after the CSRF check passes without a cookie).
    let r = client().post(format!("{}/api/observability/client-errors/session", s.base)).json(&json!({"reports": [ok]})).send().await.unwrap();
    assert!(matches!(r.status().as_u16(), 401 | 403), "{}", r.status());
}

#[tokio::test]
async fn the_session_route_is_rate_limited_per_session() {
    let s = spawn(&[], false).await;
    let (c, token, _) = user(&s.base).await;
    let mut limited = None;
    for i in 0..14 {
        let r = post_session(&c, &s.base, &token, vec![report(&format!("cd{i:06}"), "b", json!({}))]).await;
        if r.status() == 429 {
            limited = Some((i, r.headers().get("retry-after").map(|v| v.to_str().unwrap().to_owned())));
            break;
        }
        assert_eq!(r.status(), 200);
    }
    let (at, retry) = limited.expect("a 429 within 14 batches");
    assert!((10..=11).contains(&at), "burst of ten, hit at {at}");
    assert!(retry.is_some());
    // Another session has its own budget.
    let (c2, t2, _) = user(&s.base).await;
    assert_eq!(post_session(&c2, &s.base, &t2, vec![report("ce000001", "b", json!({}))]).await.status(), 200);
}

#[tokio::test]
async fn anonymous_reports_work_without_a_cookie_with_stricter_limits() {
    let s = spawn(&[], true).await;
    let a = client();
    let cookie = admin(&a, &s.base).await;
    let anon = client();
    let url = format!("{}/api/observability/client-errors", s.base);
    // The policy is public and reads no cookie.
    let p: Value = anon.get(format!("{url}/policy")).send().await.unwrap().json().await.unwrap();
    assert_eq!(p["enabled"], true);
    assert_eq!(p["captureCsp"], true);
    assert_eq!(p["captureConsole"], false);
    // 1: accepted with no cookie and no CSRF token.
    let r = anon.post(&url).json(&json!({"reports": [report("a0000001ab", "26.18+one", json!({}))]})).send().await.unwrap();
    assert_eq!(r.status(), 204);
    // An anonymous body over 64 KB never reaches the handler.
    let r = anon.post(&url).json(&json!({"reports": [report("a0000001ab", "b", json!({"message": "m".repeat(70_000)}))]})).send().await.unwrap();
    assert_eq!(r.status(), 413);
    // 2: malformed. 3: too many reports for an anonymous batch.
    let r = anon.post(&url).header("content-type", "application/json").body("{").send().await.unwrap();
    assert_eq!(r.status(), 400);
    let six: Vec<Value> = (0..6).map(|i| report(&format!("a1{i:06}"), "b", json!({}))).collect();
    let r = anon.post(&url).json(&json!({"reports": six})).send().await.unwrap();
    assert_eq!(r.status(), 400);
    // 4: still within the burst of four.
    let r = anon.post(&url).json(&json!({"reports": [report("a0000002ab", "26.18+one", json!({}))]})).send().await.unwrap();
    assert_eq!(r.status(), 204);
    // 5: over the anonymous budget.
    let r = anon.post(&url).json(&json!({"reports": [report("a0000003ab", "26.18+one", json!({}))]})).send().await.unwrap();
    assert_eq!(r.status(), 429);
    assert!(r.headers().contains_key("retry-after"));
    // Stored: two anonymous reports, with no account hash.
    let list = groups(&a, &s.base, &cookie, "").await;
    assert_eq!(list["groups"].as_array().unwrap().len(), 2);
    let id = list["groups"][0]["id"].as_str().unwrap();
    let detail: Value = a.get(format!("{}/admin/client-errors/{id}", s.base)).header("Cookie", &cookie).send().await.unwrap().json().await.unwrap();
    assert!(detail["events"][0]["accountHash"].is_null());
}

#[tokio::test]
async fn the_admin_switches_and_the_person_opt_out_stop_storage() {
    // Administrator off: accepted and dropped.
    let off = spawn(&[("observability.client.enabled", json!(false))], false).await;
    let (c, token, _) = user(&off.base).await;
    let a = client();
    let cookie = admin(&a, &off.base).await;
    let r = post_session(&c, &off.base, &token, vec![report("0ff00001ab", "b", json!({}))]).await;
    assert_eq!(r.status(), 200);
    assert_eq!(groups(&a, &off.base, &cookie, "").await["groups"].as_array().unwrap().len(), 0);
    let p: Value = a.get(format!("{}/api/observability/client-errors/policy", off.base)).send().await.unwrap().json().await.unwrap();
    assert_eq!(p["enabled"], false);

    // CSP and console reports have their own switches.
    let s = spawn(&[("observability.client.captureCsp", json!(false))], false).await;
    let (c, token, _) = user(&s.base).await;
    let cookie = admin(&a, &s.base).await;
    post_session(&c, &s.base, &token, vec![report("c5000001ab", "b", json!({"kind": "csp"})), report("c0000001ab", "b", json!({"kind": "console"}))]).await;
    assert_eq!(groups(&a, &s.base, &cookie, "").await["groups"].as_array().unwrap().len(), 0);

    // The person opts out: reporting stays 'off' for them and nothing is stored.
    let s = spawn(&[], false).await;
    let (c, token, authority) = user(&s.base).await;
    let cookie = admin(&a, &s.base).await;
    let set = c
        .put(format!("{}/api/settings/privacy.errorReports", s.base))
        .header("x-csrf-token", &token)
        .header("x-mailwoman-storage-authority", &authority)
        .json(&json!({"value": "off"}))
        .send()
        .await
        .unwrap();
    assert_eq!(set.status(), 200, "{}", set.text().await.unwrap());
    let r = post_session(&c, &s.base, &token, vec![report("0a000001ab", "b", json!({}))]).await;
    assert_eq!(r.status(), 200);
    assert_eq!(r.json::<Value>().await.unwrap()["reporting"], "off");
    assert_eq!(groups(&a, &s.base, &cookie, "").await["groups"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn the_account_hash_is_added_only_when_the_administrator_asks_for_it() {
    let a = client();
    let off = spawn(&[], false).await;
    let (c, token, _) = user(&off.base).await;
    let cookie = admin(&a, &off.base).await;
    post_session(&c, &off.base, &token, vec![report("a5000001ab", "b", json!({}))]).await;
    let id = groups(&a, &off.base, &cookie, "").await["groups"][0]["id"].as_str().unwrap().to_owned();
    let d: Value = a.get(format!("{}/admin/client-errors/{id}", off.base)).header("Cookie", &cookie).send().await.unwrap().json().await.unwrap();
    assert!(d["events"][0]["accountHash"].is_null());

    let on = spawn(&[("observability.client.includeAccountHash", json!(true))], false).await;
    let (c, token, _) = user(&on.base).await;
    let cookie = admin(&a, &on.base).await;
    post_session(&c, &on.base, &token, vec![report("a5000002ab", "b", json!({}))]).await;
    let id = groups(&a, &on.base, &cookie, "").await["groups"][0]["id"].as_str().unwrap().to_owned();
    let d: Value = a.get(format!("{}/admin/client-errors/{id}", on.base)).header("Cookie", &cookie).send().await.unwrap().json().await.unwrap();
    let hash = d["events"][0]["accountHash"].as_str().expect("hash").to_owned();
    assert_eq!(hash.len(), 16);
    assert!(!hash.contains(mw_mock_jmap::USER));
}

#[tokio::test]
async fn retention_prunes_events_before_groups_and_the_store_survives_a_restart() {
    let s = spawn(&[("retention.clientErrors.eventDays", json!(1)), ("retention.clientErrors.groupDays", json!(2))], false).await;
    let (c, token, _) = user(&s.base).await;
    post_session(&c, &s.base, &token, vec![report("ba000001ab", "b", json!({}))]).await;
    let store = mw_store::Store::open(&s.db, mw_store::ServerKey::from_hex(KEY_HEX).unwrap()).await.unwrap();
    let list = store.list_client_error_groups(&mw_store::client_errors::GroupFilter { limit: 10, ..Default::default() }).await.unwrap();
    assert_eq!(list.len(), 1);
    let gid = list[0].id.clone();
    assert_eq!(store.client_error_events(&gid, 10).await.unwrap().len(), 1);
    // Nothing is old yet.
    assert_eq!(store.prune_client_errors(0, 0).await.unwrap(), 0);
    // An event cutoff in the future removes the event but keeps the group (its group cutoff is earlier).
    let now = chrono::Utc::now().timestamp();
    assert_eq!(store.prune_client_errors(now + 10, 0).await.unwrap(), 1);
    assert_eq!(store.client_error_events(&gid, 10).await.unwrap().len(), 0);
    assert!(store.get_client_error_group(&gid).await.unwrap().is_some());
    // A group cutoff in the future removes the group.
    assert_eq!(store.prune_client_errors(0, now + 10).await.unwrap(), 1);
    assert!(store.get_client_error_group(&gid).await.unwrap().is_none());
}

#[tokio::test]
async fn per_fingerprint_sampling_keeps_counts_and_drops_details() {
    let s = spawn(&[], false).await;
    let store = mw_store::Store::open(&s.db, mw_store::ServerKey::from_hex(KEY_HEX).unwrap()).await.unwrap();
    let now = chrono::Utc::now().timestamp();
    let mk = |i: usize| mw_store::client_errors::NewClientError {
        id: format!("evt-{i}"), fingerprint: "5a000001ab".into(), kind: "boundary".into(), name: "E".into(), message: "m".into(),
        boundary: "B".into(), route_template: "/".into(), time: now, build: "b".into(), occurrences: 1, visit: format!("v{i}"),
        stack_raw: "s".into(), stack_symbolicated: None, ua_family: "Chrome".into(), os_family: "Linux".into(), viewport: "wide".into(),
        locale: "en".into(), account_hash: None, extra: "{}".into(),
    };
    let mut dropped = 0;
    for i in 0..30 {
        if store.record_client_error(&mk(i), 20).await.unwrap().sampled_out {
            dropped += 1;
        }
    }
    assert_eq!(dropped, 10);
    let g = store.list_client_error_groups(&mw_store::client_errors::GroupFilter { limit: 10, ..Default::default() }).await.unwrap();
    assert_eq!(g[0].count, 30);
    assert_eq!(store.client_error_events(&g[0].id, 100).await.unwrap().len(), 20);
}
