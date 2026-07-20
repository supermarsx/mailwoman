//! t17-e-e2e — ManageSieve egress gate (L5) through the REAL sieve-sync route.
//!
//! 26.16's Sieve caller connected to any user-supplied host:port. 26.17 adds a
//! NARROWER-than-image-proxy egress gate (settled DQ-L5): refuse cloud-metadata,
//! loopback and link-local, but KEEP RFC1918/private reachable (syncing rules to your
//! own internal ManageSieve server is legitimate). `sieve_egress_permitted` is a
//! private fn; this leg drives the REAL `POST /api/account/sieve/sync` route (engine
//! mode, behind a real session) and asserts the DISTINCTION live:
//!   * a metadata / loopback / link-local host is refused at the gate → 403 (instant,
//!     never a connect);
//!   * an RFC1918 host PASSES the gate (it is NOT a 403 — it only fails afterward with
//!     a gateway error / connect timeout because nothing is listening there).
//!
//! No live ManageSieve server is needed (settled: unit/integration ok) — the gate
//! runs before any connect, so the 403-vs-not distinction is the wired proof.
//!
//! Run:
//!   cargo test -p mw-server --test t17_sieve_egress -- --nocapture --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use serde_json::json;

use mw_server::{AppConfig, build_app};
use mw_store::{Credentials, ServerKey, Store};

const KEY_HEX: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0";

fn unique() -> String {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    format!(
        "{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

fn temp_db() -> String {
    let dir = std::env::temp_dir().join(format!("mw-t17-sieve-{}", unique()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("mw.db").to_string_lossy().into_owned()
}

/// Spawn `build_app` in ENGINE mode (the sieve route requires an engine) on `db_path`.
async fn spawn_engine_server(db_path: &str) -> SocketAddr {
    let web = PathBuf::from(db_path)
        .parent()
        .unwrap()
        .join(format!("web-{}", unique()));
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let config = AppConfig {
        db_path: db_path.to_string(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: mw_server::ServerMode::Engine,
        hardening: mw_server::HardeningConfig::default(),
        security: mw_server::SecurityConfig::default(),
    };
    let app = build_app(config).await.expect("build_app engine mode");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

/// Seed a real session row (same key as the server) and return its cookie token.
async fn seed_session(db_path: &str) -> String {
    let store = Store::open(db_path, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap();
    store
        .create_session(
            "acct-sieve",
            "user@example.org",
            "http://upstream.invalid",
            "http://upstream.invalid",
            &Credentials {
                username: "user@example.org".into(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap()
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(4))
        .build()
        .unwrap()
}

async fn sieve_sync(
    c: &reqwest::Client,
    base: &str,
    cookie: &str,
    host: &str,
) -> Result<reqwest::StatusCode, reqwest::Error> {
    c.post(format!("{base}/api/account/sieve/sync"))
        .header("Cookie", format!("mw_session={cookie}"))
        .json(&json!({
            "host": host,
            "port": 4190,
            "tls": "plaintext",
            "username": "u",
            "password": "p",
        }))
        .send()
        .await
        .map(|r| r.status())
}

#[tokio::test]
async fn sieve_egress_refuses_metadata_and_loopback_but_allows_rfc1918() {
    let db = temp_db();
    let addr = spawn_engine_server(&db).await;
    let base = format!("http://{addr}");
    let cookie = seed_session(&db).await;
    let c = client();

    // Sanity: the session is accepted (a bogus host is refused at the gate, not 401).
    // Metadata / loopback / link-local → refused at the gate → 403 (instant, no connect).
    for (host, what) in [
        ("169.254.169.254", "cloud metadata"),
        ("127.0.0.1", "IPv4 loopback"),
        ("::1", "IPv6 loopback"),
        ("169.254.0.1", "link-local"),
    ] {
        let status = sieve_sync(&c, &base, &cookie, host)
            .await
            .unwrap_or_else(|e| {
                panic!("{what} host must return promptly (a gate 403), not hang: {e}")
            });
        assert_eq!(
            status.as_u16(),
            403,
            "sieve egress: {what} ({host}) must be refused at the gate (got {status})"
        );
    }

    // RFC1918 PASSES the gate: syncing to your own internal ManageSieve is legitimate.
    // Nothing is listening, so it fails AFTER the gate — never a 403. A 403 would be
    // instant; a permitted host proceeds to a connect that fails (502) or exceeds the
    // client timeout — both prove the gate allowed it.
    match sieve_sync(&c, &base, &cookie, "10.255.255.1").await {
        Ok(status) => assert_ne!(
            status.as_u16(),
            403,
            "an RFC1918 host must NOT be refused by the Sieve egress gate (got {status})"
        ),
        Err(e) if e.is_timeout() => {
            // The gate passed and the request is stuck in the post-gate connect — a
            // blocked host would have returned a 403 instantly, so this proves allow.
            eprintln!(
                "[t17 sieve] RFC1918 10.255.255.1 passed the gate (post-gate connect timed out — expected)"
            );
        }
        Err(e) => panic!("unexpected error for the RFC1918 host: {e}"),
    }
}
