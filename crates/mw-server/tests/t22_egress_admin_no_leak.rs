//! t22-e12 — **the proxy password must not escape through any of its four exits.**
//!
//! Sealing the column closes one. The others are `tracing` output, panic messages
//! and **error bodies** — the last being the one people forget, because an error body
//! is not thought of as a log even though it travels further than one.
//!
//! Unit tests in `egress_admin` and `mw_store::egress_config` assert each exit in
//! isolation. They cannot catch the interesting case: some *other* line of code,
//! anywhere in the request path, formatting a value that happens to carry the
//! credential. So this file drives the real admin routes over real HTTP with a
//! `tracing` subscriber capturing **everything at TRACE level**, and asserts the
//! password appears nowhere in what was captured, nor in any response body.
//!
//! # What this does NOT cover, stated rather than implied
//! The acceptance criterion names a **full proxied fetch**. That leg needs
//! `t22-e11`'s tunnelling transport, which is not committed — so a fetch cannot be
//! driven through a configured route yet. **This file covers the configuration path
//! only: store, admin API, audit.** Whoever wires the transport to this config owns
//! extending the capture across an actual fetch; the pattern here is meant to be
//! copied rather than reinvented.
//!
//! Run:
//!   cargo test -p mw-server --test t22_egress_admin_no_leak -- --test-threads=1

use std::io;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde_json::json;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;

use mw_server::{AppConfig, V6Config, build_app_full};

mod common;
use common::test_db;

const KEY_HEX: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";
const ADMIN_USER: &str = "root";
const ADMIN_PASS: &str = "hunter2";

/// Distinctive enough that a substring match cannot collide with anything else the
/// server might legitimately log.
const PASSWORD: &str = "zq7-Proxy-Secret-Do-Not-Log-4f2b";

/// A `tracing` writer that appends every byte into a shared buffer.
#[derive(Clone)]
struct CapturedWriter(Arc<Mutex<Vec<u8>>>);

impl io::Write for CapturedWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().expect("capture lock").extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for CapturedWriter {
    type Writer = CapturedWriter;
    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

fn temp_db() -> String {
    test_db::unique_dir("mw-t22-egress-leak")
        .join("mw.db")
        .to_string_lossy()
        .into_owned()
}

async fn spawn_server(db_path: &str) -> SocketAddr {
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
        security: mw_server::SecurityConfig::default(),
    };
    let v6 = V6Config {
        admin_enabled: true,
        admin_username: Some(ADMIN_USER.into()),
        admin_password: Some(ADMIN_PASS.into()),
        redis_url: None,
    };
    let app = build_app_full(config, v6).await.expect("build_app").0;
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
    addr
}

#[tokio::test]
async fn the_password_reaches_neither_the_logs_nor_any_response_body() {
    let captured = Arc::new(Mutex::new(Vec::<u8>::new()));
    let writer = CapturedWriter(Arc::clone(&captured));
    let _guard = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(writer)
                .with_ansi(false),
        )
        .with(tracing_subscriber::filter::LevelFilter::TRACE)
        .set_default();

    let db = temp_db();
    let addr = spawn_server(&db).await;
    let base = format!("http://{addr}");
    let c = reqwest::Client::builder().no_proxy().build().unwrap();

    // Log in through the REAL admin route rather than seeding a session row: the
    // login itself handles a password, so it belongs inside the capture window.
    let resp = c
        .post(format!("{base}/admin/login"))
        .json(&json!({ "username": ADMIN_USER, "password": ADMIN_PASS }))
        .send()
        .await
        .expect("admin login");
    assert_eq!(resp.status(), 200, "admin login succeeds with valid creds");
    let set_cookie = resp
        .headers()
        .get(reqwest::header::SET_COOKIE)
        .expect("login sets a session cookie")
        .to_str()
        .unwrap()
        .to_string();
    let cookie = set_cookie
        .split(';')
        .next()
        .expect("cookie name=value")
        .to_string();

    let mut bodies = Vec::new();

    // Create a route carrying the credential.
    let resp = c
        .post(format!("{base}/admin/egress/proxies"))
        .header("cookie", &cookie)
        .json(&json!({
            "id": "corp",
            "scheme": "http",
            "host": "proxy.corp.example",
            "port": 3128,
            "username": "svc-mail",
            "password": PASSWORD,
            "allowPlaintext": false,
        }))
        .send()
        .await
        .expect("put proxy");
    let status = resp.status();
    bodies.push(resp.text().await.unwrap());
    assert_eq!(status, 200, "put failed: {}", bodies[0]);

    // Read it back, and delete it — every route that handles the row.
    for (method, url) in [
        ("get", format!("{base}/admin/egress/proxies")),
        ("post", format!("{base}/admin/egress/proxies/corp/delete")),
    ] {
        let req = if method == "get" {
            c.get(&url)
        } else {
            c.post(&url)
        };
        let resp = req.header("cookie", &cookie).send().await.expect("request");
        bodies.push(resp.text().await.unwrap());
    }

    // A malformed request, so the rejection/error path is exercised too — an error
    // body is the exit that gets forgotten.
    let resp = c
        .post(format!("{base}/admin/egress/proxies"))
        .header("cookie", &cookie)
        .json(&json!({
            "id": "",
            "scheme": "gopher",
            "host": "",
            "port": 0,
            "username": "",
            "password": PASSWORD,
        }))
        .send()
        .await
        .expect("bad request");
    bodies.push(resp.text().await.unwrap());

    // Give any spawned logging a moment to land before reading the buffer.
    tokio::task::yield_now().await;

    let logs = String::from_utf8_lossy(&captured.lock().unwrap().clone()).into_owned();

    // Asserted FIRST: a capture that recorded nothing would satisfy "the password is
    // absent" trivially, and would look exactly like a clean run.
    assert!(
        !logs.is_empty(),
        "no tracing output was captured at all — this test cannot detect a log leak \
         and its clean result means nothing"
    );

    assert!(
        !logs.contains(PASSWORD),
        "the proxy password appears in captured tracing output:\n{logs}"
    );
    for (i, body) in bodies.iter().enumerate() {
        assert!(
            !body.contains(PASSWORD),
            "the proxy password appears in response body {i}: {body}"
        );
    }
}
