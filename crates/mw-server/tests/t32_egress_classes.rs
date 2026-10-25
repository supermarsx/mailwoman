//! t32 e31b — the server honours the egress plan: a route carries only the traffic
//! classes it lists, the plan is installed when the server starts, and an admin
//! change re-installs it at once.
//!
//! # How the route choice is observed without a network
//! The image fetch below targets `http://127.0.0.1:9/img`, which the address gate
//! refuses on either path, so nothing is dialled. What differs is the route the
//! server **chose**: the image proxy writes an audit row naming the configured
//! route (or `direct`) whenever that changes, before it answers. The stand-in proxy
//! records every connection, so it also shows that a refused target never reached
//! it.
//!
//! Run:
//!   cargo test -p mw-server --test t32_egress_classes -- --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{Value, json};
use tokio::net::TcpListener;

use mw_egress::TrafficClass;
use mw_server::{AppConfig, build_app};
use mw_store::{ServerKey, Store};

mod common;
use common::test_db;

const KEY_HEX: &str = "1f2e3d4c5b6a79889796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f1";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";
const ADMIN_TOKEN: &str = "t32-f17-admin-token";

static MOCK_UPSTREAMS: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

async fn spawn_mock() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, mw_mock_jmap::router()).await;
    });
    let origin = format!("http://{addr}");
    MOCK_UPSTREAMS.lock().unwrap().push(origin.clone());
    origin
}

async fn spawn_server(db_path: &str, mock: &str) -> SocketAddr {
    let web = PathBuf::from(db_path)
        .parent()
        .unwrap()
        .join(format!("web-{}", test_db::unique_tag()));
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let config = AppConfig {
        db_path: db_path.to_string(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: mw_server::ServerMode::Proxy,
        hardening: mw_server::HardeningConfig::default(),
        security: mw_server::SecurityConfig {
            jmap_upstreams: Some(MOCK_UPSTREAMS.lock().unwrap().clone()),
            upstream_default: Some(mock.to_string()),
            ..mw_server::SecurityConfig::default()
        },
    };
    let app = build_app(config).await.expect("build_app");
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await;
    });
    addr
}

async fn store_at(db_path: &str) -> Store {
    Store::open(db_path, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .expect("open store")
}

fn hash_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(token.as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// A listener that only counts connections: a route pointing here shows whether
/// anything was ever sent to it.
async fn counting_proxy() -> (SocketAddr, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let hits = Arc::new(AtomicUsize::new(0));
    let sink = Arc::clone(&hits);
    tokio::spawn(async move {
        while let Ok((_s, _)) = listener.accept().await {
            sink.fetch_add(1, Ordering::SeqCst);
        }
    });
    (addr, hits)
}

/// The `configuredRoute` of the newest egress transition row (`list_audit` is
/// newest first; `ts` carries sub-second precision).
async fn last_configured_route(store: &Store) -> Option<String> {
    store
        .list_audit(50)
        .await
        .unwrap()
        .into_iter()
        .find(|r| r.actor == "egress")
        .and_then(|r| {
            serde_json::from_str::<Value>(&r.detail_json)
                .ok()?
                .get("configuredRoute")?
                .as_str()
                .map(str::to_string)
        })
}

async fn set_classes(store: &Store, id: &str, classes: &[&str]) {
    let mut row = store.get_egress_proxy(id).await.unwrap().unwrap();
    row.traffic_classes = classes.iter().map(|c| c.to_string()).collect();
    row.password = None; // keep the stored credential
    store.put_egress_proxy(&row).await.unwrap();
}

#[tokio::test]
async fn a_route_carries_only_its_classes_and_the_plan_follows_admin_changes() {
    let (proxy, proxy_hits) = counting_proxy().await;
    let db = test_db::unique_dir("mw-t32-egress-classes")
        .join("mw.db")
        .to_string_lossy()
        .into_owned();
    {
        let store = store_at(&db).await;
        store
            .put_admin_session(
                &hash_token(ADMIN_TOKEN),
                "root",
                &chrono::Utc::now().to_rfc3339(),
            )
            .await
            .unwrap();
    }

    // Nothing in this test binary has started a server yet: no plan.
    assert!(
        mw_egress::plan::current().is_none(),
        "the plan must come from the server's start, not from somewhere else"
    );
    let mock = spawn_mock().await;
    let addr = spawn_server(&db, &mock).await;
    let base = format!("http://{addr}");
    let installed = mw_egress::plan::current().expect("the server installs a plan when it starts");
    assert!(installed.routes().is_empty(), "no route is live yet");

    // A user session with a granted message, so the image proxy actually fetches.
    let user = reqwest::Client::builder()
        .cookie_store(true)
        .no_proxy()
        .build()
        .unwrap();
    let login: Value = user
        .post(format!("{base}/api/login"))
        .json(&json!({ "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(login["ok"], json!(true), "login failed: {login}");
    let account = login["accountId"].as_str().unwrap().to_string();
    let email_id = {
        let opts = sqlx::sqlite::SqliteConnectOptions::new().filename(&db);
        let raw = sqlx::SqlitePool::connect_with(opts).await.unwrap();
        sqlx::query(
            "INSERT INTO accounts (id, kind, host, port, tls, username, sealed_creds, sync_policy_json)
             VALUES (?1, 'imap', 'mail.example', 993, 1, ?1, X'00', '{}')",
        )
        .bind(&account)
        .execute(&raw)
        .await
        .unwrap();
        raw.close().await;
        let store = store_at(&db).await;
        let mailbox = store
            .upsert_mailbox(&mw_store::MailboxUpsert {
                account_id: &account,
                name: "INBOX",
                role: Some("inbox"),
                uidvalidity: 1,
                uidnext: 2,
                highestmodseq: 1,
                total: 1,
                unread: 0,
                parent_id: None,
            })
            .await
            .unwrap();
        let id = store
            .upsert_message(&mw_store::MessageUpsert {
                account_id: &account,
                mailbox_id: &mailbox,
                uid: 1,
                uidvalidity: 1,
                message_id: Some("<egress-classes@test>"),
                thread_id: None,
                internaldate: Some("2026-10-25T12:00:00Z"),
                size: 42,
                flags_json: "[]",
                envelope: None,
                blob_ref: None,
            })
            .await
            .unwrap();
        store.grant_remote_image(&account, "all", "").await.unwrap();
        id
    };

    // An operator route, through the real admin API, made live.
    let admin = reqwest::Client::builder().no_proxy().build().unwrap();
    let put = admin
        .post(format!("{base}/admin/egress/proxies"))
        .header("cookie", format!("mw_admin_session={ADMIN_TOKEN}"))
        .json(&json!({
            "id": "corp",
            "scheme": "http",
            "host": proxy.ip().to_string(),
            "port": proxy.port(),
            "allowPlaintext": true,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(put.status(), 200, "{}", put.text().await.unwrap());
    let act = admin
        .post(format!("{base}/admin/egress/proxies/corp/activate"))
        .header("cookie", format!("mw_admin_session={ADMIN_TOKEN}"))
        .send()
        .await
        .unwrap();
    assert_eq!(act.status(), 200);

    // The activation re-installed the plan in this process at once, with the
    // migration's default classes for a route created through the 26.19 form.
    let plan = mw_egress::plan::current().unwrap();
    assert_eq!(
        plan.selected_route(TrafficClass::RemoteContent, None)
            .map(|r| r.id.as_str()),
        Some("corp")
    );
    assert_eq!(
        plan.selected_route(TrafficClass::IcsFeeds, None)
            .map(|r| r.id.as_str()),
        Some("corp")
    );
    assert!(plan.selected_route(TrafficClass::Webhooks, None).is_none());

    let store = store_at(&db).await;
    let image = format!(
        "{base}/api/image-proxy?url={}&emailId={}",
        urlencode("http://127.0.0.1:9/img"),
        urlencode(&email_id)
    );

    // remote_content listed: the image proxy chooses the route.
    let r = user.get(&image).send().await.unwrap();
    assert_eq!(
        r.status(),
        403,
        "the loopback target is refused by the gate"
    );
    assert_eq!(last_configured_route(&store).await.as_deref(), Some("corp"));

    // The route stops listing remote_content: the next image goes direct.
    set_classes(&store, "corp", &["ics_feeds"]).await;
    let r = user.get(&image).send().await.unwrap();
    assert_eq!(r.status(), 403);
    assert_eq!(
        last_configured_route(&store).await.as_deref(),
        Some("direct"),
        "a route that does not list remote_content must not carry remote images"
    );

    // And back.
    set_classes(&store, "corp", &["remote_content"]).await;
    let r = user.get(&image).send().await.unwrap();
    assert_eq!(r.status(), 403);
    assert_eq!(last_configured_route(&store).await.as_deref(), Some("corp"));

    assert_eq!(
        proxy_hits.load(Ordering::SeqCst),
        0,
        "a target the gate refuses must never reach the proxy"
    );

    // Deactivation re-installs a plan with no live route.
    let off = admin
        .post(format!("{base}/admin/egress/proxies/deactivate"))
        .header("cookie", format!("mw_admin_session={ADMIN_TOKEN}"))
        .send()
        .await
        .unwrap();
    assert_eq!(off.status(), 200);
    assert!(mw_egress::plan::current().unwrap().routes().is_empty());
}
