//! t52w-e4: a signed-in user's write of the appearance customization sends the
//! opaque `Settings` push signal to that account's streams and to no other
//! account's, so a client with a live stream needs no 30 s timer to see it.
//! A write refused by the `If-Match` check sends nothing.
//!
//! Run:
//!   cargo test -p mw-server --test t52w_appearance_customization_push --locked -- --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use futures_util::StreamExt;
use serde_json::{Value, json};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config};

mod common;
use common::mock_mail::MockPop3;
use common::test_db;

const KEY_HEX: &str = "32a1b2c3d4e5f6071829304a5b6c7d8e32a1b2c3d4e5f6071829304a5b6c7d8e";
const A_USER: &str = "alice@example.org";
const A_PASS: &str = "Alice-Passw0rd!";
const B_USER: &str = "bob@example.org";
const B_PASS: &str = "Bob-Passw0rd!";

type Ws = WebSocketStream<MaybeTlsStream<TcpStream>>;

struct Server {
    base: String,
    addr: SocketAddr,
    _pop: MockPop3,
}

struct User {
    cookie: String,
    account_id: String,
    authority: String,
}

async fn spawn() -> Server {
    // SAFETY: this binary holds one test.
    unsafe { std::env::set_var("MW_ENGINE_TLS", "plaintext") };
    let pop = MockPop3::start_accounts(&[(A_USER, A_PASS), (B_USER, B_PASS)]).await;
    let dir = test_db::unique_dir("mw-t52w-e4");
    let web: PathBuf = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), "<!doctype html><title>t</title>").unwrap();
    let db = dir.join("mw.db").to_string_lossy().into_owned();
    let config = AppConfig {
        db_path: db.clone(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: ServerMode::Engine,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
            jmap_upstreams: Some(Vec::new()),
            upstream_default: Some(pop.url()),
            ..SecurityConfig::default()
        },
    };
    let v6 = V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("hunter2".into()),
        redis_url: None,
    };
    // Every sign-in comes from 127.0.0.1; 0 switches the sign-in pacing off.
    mw_store::Store::open(&db, mw_store::ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap()
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: "signin.retryCooldownSecs".into(),
            global_value: Some("0".into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "test-fixture".into(),
            updated_at: 0,
        })
        .await
        .unwrap();
    let (app, _push) = mw_server::build_app_full(config, v6)
        .await
        .expect("build_app_full");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    Server {
        base: format!("http://{addr}"),
        addr,
        _pop: pop,
    }
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

async fn login(srv: &Server, user: &str, pass: &str) -> User {
    let r = client()
        .post(format!("{}/api/login", srv.base))
        .json(&json!({ "username": user, "password": pass }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "login for {user}");
    let cookie = r
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .filter_map(|c| c.split(';').next())
        .find(|c| c.starts_with("mw_session=") && c.len() > "mw_session=".len())
        .expect("login sets mw_session")
        .to_string();
    let body: Value = r.json().await.unwrap();
    let account_id = body["accountId"].as_str().unwrap().to_string();
    let me: Value = client()
        .get(format!("{}/api/me", srv.base))
        .header("cookie", &cookie)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    User {
        cookie,
        account_id,
        authority: me["storageAuthority"].as_str().unwrap().to_string(),
    }
}

async fn open_ws(srv: &Server, user: &User) -> Ws {
    let mut req = format!("ws://{}/jmap/ws", srv.addr)
        .into_client_request()
        .unwrap();
    req.headers_mut()
        .insert("cookie", user.cookie.parse().unwrap());
    tokio_tungstenite::connect_async(req)
        .await
        .expect("ws upgrade")
        .0
}

/// The `changed` object of every `StateChange` frame arriving within `within`.
async fn frames(ws: &mut Ws, within: Duration) -> Vec<Value> {
    let deadline = tokio::time::Instant::now() + within;
    let mut seen = Vec::new();
    while let Ok(Some(Ok(message))) = tokio::time::timeout_at(deadline, ws.next()).await {
        if let Message::Text(text) = message {
            let frame: Value = serde_json::from_str(text.as_str()).unwrap();
            if frame["@type"] == "StateChange" {
                seen.push(frame["changed"].clone());
            }
        }
    }
    seen
}

fn settings_for(frames: &[Value], account: &str) -> usize {
    frames
        .iter()
        .filter(|changed| changed[account]["Settings"].is_string())
        .count()
}

fn url(srv: &Server, user: &User) -> String {
    format!(
        "{}/api/account/appearance/customization?accountId={}",
        srv.base, user.account_id
    )
}

async fn write(srv: &Server, user: &User, tag: &str, css: &str) -> u16 {
    client()
        .put(url(srv, user))
        .header("cookie", &user.cookie)
        .header("x-mailwoman-storage-authority", &user.authority)
        .header("if-match", tag)
        .json(&json!({"ownerId": user.account_id, "document": {"version": 1, "css": css, "script": ""}}))
        .send()
        .await
        .unwrap()
        .status()
        .as_u16()
}

async fn etag(srv: &Server, user: &User) -> String {
    let r = client()
        .get(url(srv, user))
        .header("cookie", &user.cookie)
        .header("x-mailwoman-storage-authority", &user.authority)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    r.headers()["etag"].to_str().unwrap().to_owned()
}

#[tokio::test]
async fn a_customization_write_signals_its_own_account_only() {
    let srv = spawn().await;
    let a = login(&srv, A_USER, A_PASS).await;
    let b = login(&srv, B_USER, B_PASS).await;
    assert_ne!(a.account_id, b.account_id);
    let mut a_stream = open_ws(&srv, &a).await;
    let mut b_stream = open_ws(&srv, &b).await;
    // Whatever the streams carry on open is not the write's doing.
    let _ = frames(&mut a_stream, Duration::from_millis(800)).await;
    let _ = frames(&mut b_stream, Duration::from_millis(800)).await;

    // A refused write (stale If-Match) changes nothing and signals nothing.
    let tag = etag(&srv, &a).await;
    assert_eq!(write(&srv, &a, "\"stale\"", "body{color:red}").await, 412);
    let refused = frames(&mut a_stream, Duration::from_millis(1200)).await;
    assert_eq!(settings_for(&refused, &a.account_id), 0, "{refused:?}");

    assert_eq!(write(&srv, &a, &tag, "body{color:red}").await, 200);
    let a_seen = frames(&mut a_stream, Duration::from_millis(2500)).await;
    let b_seen = frames(&mut b_stream, Duration::from_millis(500)).await;
    assert!(
        settings_for(&a_seen, &a.account_id) >= 1,
        "A's stream carries the Settings signal: {a_seen:?}"
    );
    assert!(
        !serde_json::to_string(&b_seen)
            .unwrap()
            .contains(&a.account_id)
            && settings_for(&b_seen, &b.account_id) == 0,
        "B's stream carries nothing of A's write: {b_seen:?}"
    );
}
