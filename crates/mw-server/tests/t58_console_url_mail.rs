//! t58 — Security > Console address > Send by email.
//!
//! `POST {console}/console-path/mail` queues one message with the console's
//! address through the outbound system mailer (`compliance_mail`). These tests
//! read the queued message back from the encrypted queue with the server key:
//!
//! 1. Signed-out callers and admins without `security.read` are refused; an
//!    admin without a fresh step-up gets `step_up_required`.
//! 2. One plain address only; with mail switched off the answer is
//!    `mailNotConfigured` and nothing is queued.
//! 3. The message holds the absolute address under `MW_BASE_PATH` and behind a
//!    trusted proxy's forwarded host, plus the pending address with what happens
//!    at the restart; plain text and escaped HTML; no password or session value.
//! 4. Each send is audited without the path; the fourth send in an hour is 429.

use std::net::SocketAddr;

use base64::Engine as _;
use reqwest::Client;
use serde_json::{Value, json};
use tokio::sync::{Mutex, MutexGuard};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::settings_policy::SettingPolicyRow;
use mw_store::{ServerKey, Store};

mod common;
use common::test_db;

const KEY_HEX: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
const ADMIN_PASS: &str = "hunter2";
const PENDING_PATH: &str = "/ops-console-t58x";
const QUEUE_KINDS: [&str; 6] = [
    "compliance-mail:pending",
    "compliance-mail:sending",
    "compliance-mail:sent",
    "compliance-mail:failed",
    "compliance-mail:uncertain",
    "compliance-mail:dismissed",
];

/// Serialise the environment the app reads at build time.
async fn env_lock(base: Option<&str>, trusted: Option<&str>) -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::const_new(());
    let guard = LOCK.lock().await;
    // SAFETY: every mutation and dependent read in this binary holds the guard.
    unsafe {
        for (key, value) in [("MW_BASE_PATH", base), ("MW_TRUSTED_PROXIES", trusted)] {
            match value {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
        std::env::remove_var("MW_PUBLIC_URL");
        std::env::remove_var("MW_ADMIN_PATH");
    }
    guard
}

async fn sql_rows(db: &str, statement: &str) -> Vec<String> {
    use sqlx::{Connection, Row};
    let mut conn = sqlx::SqliteConnection::connect(&format!("sqlite://{db}"))
        .await
        .unwrap();
    sqlx::query(statement)
        .fetch_all(&mut conn)
        .await
        .unwrap()
        .iter()
        .map(|r| r.try_get::<String, _>(0).unwrap_or_default())
        .collect()
}

async fn set(store: &Store, key: &str, value: Value) {
    store
        .put_setting_policy(&SettingPolicyRow {
            key: key.into(),
            global_value: Some(value.to_string()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "test".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
}

struct Server {
    /// `http://127.0.0.1:port` + `MW_BASE_PATH`.
    base: String,
    store: Store,
    db: String,
}

/// A server whose outbound mail is enabled (`mail`) with a relay it never
/// reaches (no `system_mail` proxy route), so sends stay in the queue.
async fn server(tag: &str, mail: bool) -> Server {
    let db = test_db::unique_db_path(tag);
    let store = Store::open(db.to_str().unwrap(), ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap();
    set(&store, "signin.retryCooldownSecs", json!(0)).await;
    if mail {
        set(&store, "compliance.mail.enabled", json!(true)).await;
        set(&store, "compliance.mail.host", json!("relay.invalid")).await;
        set(
            &store,
            "compliance.mail.from",
            json!("mailwoman@example.org"),
        )
        .await;
        set(
            &store,
            "dsr.notify.operatorAddresses",
            json!(["ops@example.org"]),
        )
        .await;
    }
    let base_path = std::env::var("MW_BASE_PATH").unwrap_or_default();
    let (app, _) = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into(),
            server_key_hex: Some(KEY_HEX.into()),
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig::default(),
        },
        V6Config {
            admin_enabled: true,
            admin_username: Some("root".into()),
            admin_password: Some(ADMIN_PASS.into()),
            redis_url: None,
        },
    )
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await;
    });
    Server {
        base: format!("http://{addr}{base_path}"),
        store,
        db: db.to_string_lossy().into_owned(),
    }
}

fn client() -> Client {
    Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap()
}

async fn login(base: &str, user: &str, pass: &str) -> Client {
    let c = client();
    let r = c
        .post(format!("{base}/admin/login"))
        .json(&json!({"username": user, "password": pass}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "login {user}: {}", r.text().await.unwrap());
    c
}

async fn step_up(c: &Client, base: &str, pass: &str) {
    let r = c
        .post(format!("{base}/admin/step-up"))
        .json(&json!({"password": pass}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "step-up: {}", r.text().await.unwrap());
}

/// Store a new console path in the console; it applies at the next start, so
/// until then the server answers at `/admin` and the new path is pending.
async fn stage_pending(c: &Client, base: &str) {
    let current = c
        .get(format!("{base}/admin/console-path"))
        .send()
        .await
        .unwrap();
    let tag = current.headers()["etag"].to_str().unwrap().to_owned();
    let r = c
        .put(format!("{base}/admin/console-path"))
        .header("if-match", tag)
        .json(&json!({"path": PENDING_PATH}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "stage: {}", r.text().await.unwrap());
}

async fn send(c: &Client, base: &str, body: Value, headers: &[(&str, &str)]) -> (u16, Value) {
    let mut req = c
        .post(format!("{base}/admin/console-path/mail"))
        .json(&body);
    for (k, v) in headers {
        req = req.header(*k, *v);
    }
    let r = req.send().await.unwrap();
    let status = r.status().as_u16();
    (status, r.json().await.unwrap_or(Value::Null))
}

/// Every message in the queue, decrypted: `(recipient, subject, text, html)`.
async fn queued(store: &Store) -> Vec<(String, String, String, String)> {
    let mut out = Vec::new();
    for kind in QUEUE_KINDS {
        for row in store.list_compliance_records(kind).await.unwrap() {
            if !row.name.starts_with("admin-console-url:") {
                continue;
            }
            let sealed = base64::engine::general_purpose::STANDARD
                .decode(&row.body_json)
                .unwrap();
            let plain = store.open_compliance_delivery(&row.id, &sealed).unwrap();
            let m: Value = serde_json::from_slice(&plain).unwrap();
            let s = |k: &str| m[k].as_str().unwrap_or_default().to_owned();
            out.push((s("recipient"), s("subject"), s("text"), s("html")));
        }
    }
    out
}

#[tokio::test]
async fn only_a_stepped_up_admin_with_security_read_can_send() {
    let _g = env_lock(None, None).await;
    let s = server("t58-auth", true).await;
    let base = &s.base;

    // Signed out.
    let (status, _) = send(&client(), base, json!({"recipient": "a@example.org"}), &[]).await;
    assert_eq!(status, 401);

    // An admin whose role lacks security.read.
    let root = login(base, "root", ADMIN_PASS).await;
    step_up(&root, base, ADMIN_PASS).await;
    let role: Value = root
        .post(format!("{base}/admin/roles"))
        .json(&json!({"name": "dashboard only", "capabilities": ["dashboard.read"]}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let created = root
        .post(format!("{base}/admin/admins"))
        .json(
            &json!({"username": "viewer", "password": "long viewer password",
            "roles": [{"roleId": role["id"], "domains": null}]}),
        )
        .send()
        .await
        .unwrap();
    assert!(
        created.status().is_success(),
        "{}",
        created.text().await.unwrap()
    );
    let viewer = login(base, "viewer", "long viewer password").await;
    step_up(&viewer, base, "long viewer password").await;
    let (status, _) = send(&viewer, base, json!({"recipient": "a@example.org"}), &[]).await;
    assert_eq!(status, 403);
    let st = viewer
        .get(format!("{base}/admin/console-path/mail"))
        .send()
        .await
        .unwrap();
    assert_eq!(st.status(), 403);

    // A full admin without a fresh step-up.
    let fresh = login(base, "root", ADMIN_PASS).await;
    let (status, body) = send(&fresh, base, json!({"recipient": "a@example.org"}), &[]).await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["code"], "step_up_required");
    assert_eq!(body["action"], "consolePath", "{body}");
    assert!(queued(&s.store).await.is_empty());

    // The status lists the configured operator recipient.
    let st: Value = fresh
        .get(format!("{base}/admin/console-path/mail"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(st["configured"], true, "{st}");
    assert_eq!(st["suggestions"][0]["address"], "ops@example.org", "{st}");
    assert_eq!(st["suggestions"][0]["kind"], "operator");
}

#[tokio::test]
async fn recipients_are_one_plain_address_and_mail_must_be_set_up() {
    let _g = env_lock(None, None).await;
    let off = server("t58-off", false).await;
    let c = login(&off.base, "root", ADMIN_PASS).await;
    step_up(&c, &off.base, ADMIN_PASS).await;
    let st: Value = c
        .get(format!("{}/admin/console-path/mail", off.base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(st["configured"], false);
    assert_eq!(st["reason"], "disabled");
    let (status, body) = send(&c, &off.base, json!({"recipient": "a@example.org"}), &[]).await;
    assert_eq!(status, 409, "{body}");
    assert_eq!(body["code"], "mailNotConfigured");
    assert!(queued(&off.store).await.is_empty());

    let on = server("t58-recipients", true).await;
    let c = login(&on.base, "root", ADMIN_PASS).await;
    step_up(&c, &on.base, ADMIN_PASS).await;
    for bad in [
        "root",
        "a@example.org, b@example.org",
        "a@example.org;b@example.org",
        "Ops <ops@example.org>",
        "a@example.org\r\nBcc: x@example.org",
        "\"x y\"@example.org",
    ] {
        let (status, body) = send(&c, &on.base, json!({"recipient": bad}), &[]).await;
        assert_eq!(status, 422, "{bad:?}: {body}");
        assert_eq!(body["code"], "invalidRecipient", "{bad:?}");
    }
    let (status, _) = send(
        &c,
        &on.base,
        json!({"recipient": "a@example.org", "url": "https://evil.example/"}),
        &[],
    )
    .await;
    assert!(
        status == 400 || status == 422,
        "unknown fields are refused: {status}"
    );
    assert!(queued(&on.store).await.is_empty());
}

#[tokio::test]
async fn the_message_carries_the_absolute_address_under_a_sub_path_behind_a_proxy() {
    let _g = env_lock(Some("/mail"), Some("127.0.0.1/32")).await;
    let s = server("t58-message", true).await;
    let c = login(&s.base, "root", ADMIN_PASS).await;
    step_up(&c, &s.base, ADMIN_PASS).await;
    stage_pending(&c, &s.base).await;
    let (status, body) = send(
        &c,
        &s.base,
        json!({"recipient": "other@example.net", "locale": "de"}),
        &[
            ("x-forwarded-host", "mail.example.org"),
            ("x-forwarded-proto", "https"),
        ],
    )
    .await;
    assert_eq!(status, 202, "{body}");
    assert_eq!(body["queued"], true);
    let mail = queued(&s.store).await;
    assert_eq!(mail.len(), 1);
    let (to, subject, text, html) = &mail[0];
    assert_eq!(to, "other@example.net");
    assert_eq!(
        subject,
        "Adresse der Administrationskonsole für Mailwoman (mail.example.org)"
    );
    let lines: Vec<&str> = text.lines().collect();
    assert!(
        lines.contains(&"https://mail.example.org/mail/admin/"),
        "{text}"
    );
    assert!(
        lines.contains(&format!("https://mail.example.org/mail{PENDING_PATH}/").as_str()),
        "{text}"
    );
    assert!(text.contains("Neustart"), "{text}");
    assert!(text.contains(" UTC"), "{text}");
    assert!(
        html.contains("<a href=\"https://mail.example.org/mail/admin/\""),
        "{html}"
    );
    assert!(html.contains("lang=\"de\""));
    for secret in [ADMIN_PASS, "mw_admin_session", "mw_csrf"] {
        assert!(!text.contains(secret) && !html.contains(secret), "{secret}");
    }

    // The audit names the send and the recipient's domain, never the path.
    let audit: Value = c
        .get(format!("{}/admin/audit", s.base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let all = audit.to_string();
    assert!(all.contains("admin-console-path-emailed"), "{all}");
    assert!(!all.contains("ops-console-t58x"), "{all}");
    // The before/after detail is in the row itself (not in the list API).
    let entry = sql_rows(
        &s.db,
        "SELECT COALESCE(after_json,'') || ' ' || detail_json FROM audit_log WHERE action = 'admin-console-path-emailed'",
    )
    .await
    .join("
");
    assert!(
        entry.contains("\"recipientDomain\":\"example.net\""),
        "{entry}"
    );
    assert!(entry.contains("\"recipientKind\":\"other\""), "{entry}");
    assert!(
        !entry.contains("other@example.net"),
        "the redactor masks the address: {entry}"
    );
    let everything = sql_rows(
        &s.db,
        "SELECT COALESCE(after_json,'') || COALESCE(before_json,'') || detail_json FROM audit_log",
    )
    .await
    .join(
        "
",
    );
    assert!(!everything.contains("ops-console-t58x"), "{everything}");
    let rows = s.store.list_audit(100).await.unwrap();
    let row = rows
        .iter()
        .find(|r| r.action == "admin-console-path-emailed")
        .expect("audit row");
    assert_eq!(row.actor, "root");
    assert!(!format!("{row:?}").contains("ops-console-t58x"));
}

#[tokio::test]
async fn an_untrusted_forwarded_host_is_ignored_and_the_name_is_escaped() {
    let _g = env_lock(None, None).await;
    let s = server("t58-escape", true).await;
    let c = login(&s.base, "root", ADMIN_PASS).await;
    step_up(&c, &s.base, ADMIN_PASS).await;
    let current = c
        .get(format!("{}/admin/branding", s.base))
        .send()
        .await
        .unwrap();
    let tag = current.headers()["etag"].to_str().unwrap().to_owned();
    let view: Value = current.json().await.unwrap();
    let saved = c
        .put(format!("{}/admin/branding", s.base))
        .header("if-match", tag)
        .json(&json!({"brandName": "<b>Ops</b> & \"Mail\"", "loginNotice": null, "images": view["images"]}))
        .send()
        .await
        .unwrap();
    assert!(
        saved.status().is_success(),
        "{}",
        saved.text().await.unwrap()
    );
    let (status, body) = send(
        &c,
        &s.base,
        json!({"recipient": "ops@example.org"}),
        &[("x-forwarded-host", "evil.example")],
    )
    .await;
    assert_eq!(status, 202, "{body}");
    let mail = queued(&s.store).await;
    let (_, subject, text, html) = &mail[0];
    assert!(!text.contains("evil.example") && !html.contains("evil.example"));
    assert!(
        text.lines()
            .any(|l| l.starts_with("http://127.0.0.1:") && l.ends_with("/admin/")),
        "{text}"
    );
    assert!(
        subject.contains("<b>Ops</b>"),
        "the subject is plain text: {subject}"
    );
    assert!(!html.contains("<b>Ops</b>"), "{html}");
    assert!(
        html.contains("&lt;b&gt;Ops&lt;/b&gt; &amp; &quot;Mail&quot;"),
        "{html}"
    );
}

#[tokio::test]
async fn sends_are_rate_limited_per_admin() {
    let _g = env_lock(None, None).await;
    let s = server("t58-rate", true).await;
    let c = login(&s.base, "root", ADMIN_PASS).await;
    step_up(&c, &s.base, ADMIN_PASS).await;
    for i in 0..3 {
        let (status, body) = send(
            &c,
            &s.base,
            json!({"recipient": format!("r{i}@example.org")}),
            &[],
        )
        .await;
        assert_eq!(status, 202, "send {i}: {body}");
    }
    let (status, body) = send(&c, &s.base, json!({"recipient": "r4@example.org"}), &[]).await;
    assert_eq!(status, 429, "{body}");
    assert_eq!(body["code"], "rateLimited");
    assert!(body["retryAfterSecs"].as_u64().unwrap() >= 1);
    assert_eq!(queued(&s.store).await.len(), 3);
}
