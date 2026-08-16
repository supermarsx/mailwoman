//! t22-e9 — the `/api/discover` rate limit, LIVE through the real route.
//!
//! `scope_mw::discover_ratelimit` unit-tests the key derivation and
//! `scope_mw::rate_limit` unit-tests the bucket and its eviction bound. Neither
//! proves the thing most likely to be wrong: that the layer is actually **wired onto
//! the route**, and that the client address actually **reaches it**.
//!
//! The second half is not hypothetical. The limit keys on
//! `scope_mw::proxy::client_ip`, which returns `None` unless the serve path installed
//! `ConnectInfo` — and the existing integration harnesses in this directory call
//! `axum::serve(listener, app)` *without* it (`t18_e2e_ratelimit.rs:79`, among
//! others). A limiter tested only through those would fail open on every request and
//! the suite would be green. So this file spawns the server **both ways** and asserts
//! the two different, documented behaviours:
//!
//!   * with `into_make_service_with_connect_info` — as production serves — a source
//!     spends its burst and is then refused `429`;
//!   * without it — as an in-process transport may — nothing is counted, which is the
//!     deliberate fail-open, live rather than merely described.
//!
//! Driven with `{"email":"bad"}`: no `@`, so `split_email` refuses it before any DNS
//! or HTTP happens. The layer runs before the handler, so each request still charges
//! a token. That keeps the test off the network entirely and makes it fast and
//! deterministic — a discovery of a real domain would do live SRV and HTTPS work per
//! request, which is neither.
//!
//! Run:
//!   cargo test -p mw-server --test t22_discover_ratelimit -- --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::json;

use mw_server::{AppConfig, build_app};

mod common;
use common::test_db;

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

/// Burst from `discover_ratelimit::BURST`. Private there, restated here: an
/// integration test asserting through the public surface cannot read it, and
/// hard-coding it means a change to the budget fails this test loudly rather than
/// silently weakening it.
const EXPECTED_BURST: usize = 20;

async fn app_config(db_path: &str) -> AppConfig {
    let web = PathBuf::from(db_path)
        .parent()
        .unwrap()
        .join(format!("web-{}", test_db::unique_tag()));
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    AppConfig {
        db_path: db_path.to_string(),
        server_key_hex: None,
        web_dir: Some(web),
        cookie_secure: false,
        mode: mw_server::ServerMode::Proxy,
        hardening: mw_server::HardeningConfig::default(),
        security: mw_server::SecurityConfig::default(),
    }
}

fn temp_db(tag: &str) -> String {
    test_db::unique_dir(tag)
        .join("mw.db")
        .to_string_lossy()
        .into_owned()
}

/// Serve the way PRODUCTION serves — `ConnectInfo` installed, so `client_ip` yields
/// the peer address and the limit has a key to work with.
async fn spawn_with_connect_info(db_path: &str) -> SocketAddr {
    let app = build_app(app_config(db_path).await)
        .await
        .expect("build_app");
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

/// Serve WITHOUT `ConnectInfo` — what an in-process transport looks like, and what
/// the other integration harnesses in this directory do.
async fn spawn_without_connect_info(db_path: &str) -> SocketAddr {
    let app = build_app(app_config(db_path).await)
        .await
        .expect("build_app");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

async fn discover_statuses(base: &str, n: usize) -> Vec<u16> {
    let c = reqwest::Client::builder().no_proxy().build().unwrap();
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        let resp = c
            .post(format!("{base}/api/discover"))
            .json(&json!({ "email": "bad" }))
            .send()
            .await
            .expect("discover request");
        out.push(resp.status().as_u16());
    }
    out
}

#[tokio::test]
async fn a_source_spends_its_burst_and_is_then_refused() {
    let db = temp_db("mw-t22-discover-rl");
    let addr = spawn_with_connect_info(&db).await;
    let statuses = discover_statuses(&format!("http://{addr}"), EXPECTED_BURST + 10).await;

    // The early requests must be SERVED (whatever the handler makes of a malformed
    // address) — a limiter that refused from the first request would also "contain"
    // the abuse, and would be a broken endpoint.
    let early = &statuses[..EXPECTED_BURST - 5];
    assert!(
        early.iter().all(|s| *s != 429),
        "the first {} requests must be admitted, got {early:?}",
        EXPECTED_BURST - 5
    );

    // And the burst must actually end. This is the assertion that fails if the layer
    // is not wired onto the route, or if `ConnectInfo` never reaches it.
    assert!(
        statuses.contains(&429),
        "no request was refused across {} attempts — the rate limit is not in the \
         request path, or `client_ip` returned `None` and it failed open: {statuses:?}",
        statuses.len()
    );
}

#[tokio::test]
async fn without_connect_info_nothing_is_counted() {
    // The documented fail-open, proven rather than asserted in prose. This is the
    // whole reason `t22_connect_info_guard.rs` exists: if a production mount ever
    // lost its make-service, THIS is the behaviour it would silently get.
    let db = temp_db("mw-t22-discover-open");
    let addr = spawn_without_connect_info(&db).await;
    let statuses = discover_statuses(&format!("http://{addr}"), EXPECTED_BURST + 10).await;

    assert!(
        statuses.iter().all(|s| *s != 429),
        "with no `ConnectInfo` there is no key, so nothing may be counted — a 429 \
         here means the limiter invented a key from something the peer controls: \
         {statuses:?}"
    );
}
