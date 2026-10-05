//! t28-e7 — `POST /api/ui-plugins/{id}/rpc` requires a mailbox session.
//!
//! The broker had no session check: it gated on the plugin being approved and on
//! its grants, both properties of the plugin, and on nothing about the caller. So
//! anyone who could reach the server could read and overwrite the values a plugin
//! had stored and, for a plugin granted `net:host-allowlist`, have the server fetch
//! from the granted hosts. The plugin ids needed for that are listed, without a
//! session, by `GET /api/ui-plugins`.
//!
//! Driven over HTTP against the real router. The refusals are preceded by the same
//! request succeeding for the same plugin, and the anonymous write is shown not to
//! have landed.
//!
//! Run:
//!   cargo test -p mw-server --test t28_ui_plugin_rpc_auth --locked -- --test-threads=1

use std::net::SocketAddr;

use base64::Engine as _;
use serde_json::{Value, json};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config};

mod common;
use common::test_db;

const KEY_HEX: &str = "28e7c1b2c3d4e5f60718293a4b5c6d7e28e7c1b2c3d4e5f60718293a4b5c6d7e";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW_T28</div>";
const PLUGIN: &str = "t28-notes";

async fn spawn_mock_jmap() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    format!("http://{addr}")
}

async fn spawn_server(mock: &str) -> String {
    let dir = test_db::unique_dir("mw-t28-e7-uiplugin");
    let web = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let config = AppConfig {
        db_path: dir.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
            jmap_upstreams: Some(vec![mock.to_string()]),
            ..SecurityConfig::default()
        },
    };
    let v6 = V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("hunter2".into()),
        redis_url: None,
    };
    let app = mw_server::build_app_full(config, v6)
        .await
        .expect("build_app_full")
        .0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    format!("http://{addr}")
}

fn browser() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

async fn rpc(c: &reqwest::Client, base: &str, method: &str, args: Value) -> reqwest::Response {
    c.post(format!("{base}/api/ui-plugins/{PLUGIN}/rpc"))
        .json(&json!({
            "v": 1, "id": "r1", "cap": "store:kv-scoped", "method": method, "args": args,
        }))
        .send()
        .await
        .unwrap()
}

async fn set_flags(admin: &reqwest::Client, base: &str, id: &str, disabled: bool, force: bool) {
    let r = admin
        .put(format!("{base}/admin/users/{id}/flags"))
        .json(&json!({
            "zeroAccess": false,
            "forcePasswordChange": force,
            "remoteCacheWipe": false,
            "disabled": disabled,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 204, "the panel's flag write is accepted");
}

#[tokio::test]
async fn the_rpc_broker_serves_a_session_and_refuses_everyone_else() {
    let mock = spawn_mock_jmap().await;
    let base = spawn_server(&mock).await;
    let (user, pass) = (mw_mock_jmap::USER, mw_mock_jmap::PASS);

    // An approved plugin with the KV capability granted.
    let admin = browser();
    let r = admin
        .post(format!("{base}/admin/login"))
        .json(&json!({ "username": "root", "password": "hunter2" }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "admin login");
    let r = admin
        .post(format!("{base}/admin/ui-plugins"))
        .json(&json!({
            "manifest": {
                "id": PLUGIN,
                "name": "T28 Notes",
                "version": "1.0.0",
                "signature": null,
                "extensionPoints": ["message-toolbar"],
                "capabilities": ["ui:message-toolbar", "store:kv-scoped"],
                "csp": "default-src 'none'",
            },
            "bundle": base64::engine::general_purpose::STANDARD.encode("bundle"),
            "allowUnsigned": true,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 201, "register the plugin");
    let r = admin
        .post(format!("{base}/admin/ui-plugins/{PLUGIN}/approve"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 204, "approve it");
    let r = admin
        .post(format!("{base}/admin/ui-plugins/{PLUGIN}/grant"))
        .json(&json!({ "capability": "store:kv-scoped", "params": {} }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "grant the capability");

    // Precondition: the signed-in SPA's call works, exactly as the web client
    // sends it (cookie, JSON envelope).
    let c = browser();
    let login = c
        .post(format!("{base}/api/login"))
        .json(&json!({ "jmapUrl": mock, "username": user, "password": pass }))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200, "mailbox login");
    let put = rpc(&c, &base, "put", json!(["note", "written by the session"])).await;
    assert_eq!(put.status(), 200);
    let put: Value = put.json().await.unwrap();
    assert_eq!(put["ok"], json!({ "ok": true }), "{put}");
    let get: Value = rpc(&c, &base, "get", json!(["note"]))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(get["ok"], json!("written by the session"), "{get}");
    // The broker's own deny-by-default still answers inside a session.
    let denied: Value = rpc(&c, &base, "delete", json!(["note"]))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(denied["err"]["code"], json!("method-denied"), "{denied}");

    // No cookie: refused, for a read and for a write.
    let anon = browser();
    for (method, args) in [
        ("get", json!(["note"])),
        ("put", json!(["note", "written by nobody"])),
    ] {
        let r = rpc(&anon, &base, method, args).await;
        assert_eq!(r.status(), 401, "{method} without a session");
        let body = r.text().await.unwrap();
        assert!(
            !body.contains("written by the session"),
            "the refusal carries no stored value: {body}"
        );
    }
    // A cookie that names no session is no better.
    let forged = reqwest::Client::new()
        .post(format!("{base}/api/ui-plugins/{PLUGIN}/rpc"))
        .header("cookie", "mw_session=not-a-session")
        .json(&json!({ "v": 1, "id": "r1", "cap": "store:kv-scoped", "method": "get", "args": ["note"] }))
        .send()
        .await
        .unwrap();
    assert_eq!(forged.status(), 401, "a forged cookie");
    // And the anonymous write did not land.
    let get: Value = rpc(&c, &base, "get", json!(["note"]))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(
        get["ok"],
        json!("written by the session"),
        "the stored value is what the session wrote"
    );

    // Held for a password change: refused with the gate's body, like any other
    // session route, and released with the flag.
    set_flags(&admin, &base, user, false, true).await;
    let held = rpc(&c, &base, "get", json!(["note"])).await;
    assert_eq!(held.status(), 403, "a held account");
    assert_eq!(
        held.json::<Value>().await.unwrap(),
        json!({ "error": "password change required", "passwordChangeRequired": true })
    );
    set_flags(&admin, &base, user, false, false).await;
    assert_eq!(rpc(&c, &base, "get", json!(["note"])).await.status(), 200);

    // Disabled: the cookie that worked a moment ago is refused.
    set_flags(&admin, &base, user, true, false).await;
    let disabled = rpc(&c, &base, "get", json!(["note"])).await;
    assert_eq!(disabled.status(), 401, "a disabled account");
}
