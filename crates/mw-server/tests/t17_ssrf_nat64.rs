//! t17-e-e2e — SSRF NAT64 / 6to4 embedded-IPv4 refusal (L3) LIVE through the real
//! image proxy.
//!
//! 26.16's SSRF gate blocked literal private/loopback/metadata targets but not those
//! same addresses SMUGGLED inside an IPv6 NAT64 (`64:ff9b::/96`) or 6to4 (`2002::/16`)
//! embedding. 26.17 decodes the embedded IPv4 and re-checks it. `image_proxy.rs`
//! unit-tests the decode; THIS leg drives the REAL `/api/image-proxy` route (behind a
//! real session) and asserts a private/loopback/metadata v4 smuggled through NAT64/6to4
//! is REFUSED live by the SSRF gate, while a public v4 via NAT64 is PERMITTED by it.
//!
//! # Why every assertion here reads the BODY (26.20, t22-e7 P3)
//! The route now has TWO gates that both answer `403` on the path this suite drives:
//! the egress/SSRF policy and the remote-image grant gate (`ungranted_response`),
//! which refuses because this suite's session holds no grant. A bare
//! `assert_eq!(status, 403)` therefore no longer distinguishes them — **every
//! smuggling leg below would stay green with the NAT64/6to4 decode deleted**, refused
//! by the grant gate instead of by the thing this file exists to test. The two
//! refusals carry different bodies ([`EGRESS_REFUSAL`] / [`GRANT_REFUSAL`]), so the
//! body is what makes each leg mean what its name says. The last assertion in the
//! test checks that the two bodies actually differ *in this run*, because a reword or
//! a refactor routing both gates through one message would silently degrade every
//! assertion above back to "this route returns 403".
//!
//! Run:
//!   cargo test -p mw-server --test t17_ssrf_nat64 -- --nocapture --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::{Value, json};

use mw_server::{AppConfig, build_app};

mod common;
use common::test_db;

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

async fn spawn_mock() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    format!("http://{addr}")
}

async fn spawn_server() -> SocketAddr {
    let base = test_db::unique_dir("mw-t17-nat64");
    let web = base.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let config = AppConfig {
        db_path: base.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: None,
        web_dir: Some(web as PathBuf),
        cookie_secure: false,
        mode: mw_server::ServerMode::Proxy,
        hardening: mw_server::HardeningConfig::default(),
        security: mw_server::SecurityConfig::default(),
    };
    let app = build_app(config).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap()
}

async fn login(c: &reqwest::Client, base: &str, mock: &str) {
    let body: Value = c
        .post(format!("{base}/api/login"))
        .json(&json!({ "jmapUrl": mock, "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(body["ok"], json!(true), "plain login (no 2FA): {body}");
}

fn urlencoding(s: &str) -> String {
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

/// The refusal the EGRESS/SSRF policy returns for a blocked address
/// (`image_proxy.rs::refusal_response`, `Refusal::Blocked`). This is the one this
/// file is about.
const EGRESS_REFUSAL: &str = "target address is not permitted";
/// The refusal the 26.20 remote-image GRANT gate returns
/// (`image_proxy.rs::ungranted_response`). Reaching it proves the egress policy
/// PERMITTED the address: `ungranted_response` runs that policy first and returns its
/// refusal if there is one, so this body is unreachable for a blocked address.
const GRANT_REFUSAL: &str = "no remote-image grant covers this message";

/// Drive the real route and return both halves of the answer. The status alone is no
/// longer a sufficient instrument — see the module docs.
async fn proxy_get(c: &reqwest::Client, base: &str, target: &str) -> (reqwest::StatusCode, String) {
    let url = format!("{base}/api/image-proxy?url={}", urlencoding(target));
    let resp = c.get(url).send().await.unwrap();
    let status = resp.status();
    (status, resp.text().await.unwrap())
}

#[tokio::test]
async fn nat64_and_6to4_smuggled_private_targets_are_refused_live() {
    let mock = spawn_mock().await;
    let addr = spawn_server().await;
    let base = format!("http://{addr}");
    let c = client();
    login(&c, &base, &mock).await;

    // Each of these embeds a forbidden IPv4 (127.0.0.1 / 169.254.169.254 / 10.0.0.1 /
    // 192.168.1.1) inside a NAT64 (64:ff9b::/96) or 6to4 (2002::/16) address. All must
    // be refused live (403) BY THE SSRF GATE — the decode + re-check closes the
    // smuggling path. One refusal body is kept for the instrument check below.
    let mut denied_body = String::new();
    for (target, what) in [
        ("http://[64:ff9b::7f00:1]/x.png", "NAT64 loopback 127.0.0.1"),
        (
            "http://[64:ff9b::a9fe:a9fe]/latest/meta-data/",
            "NAT64 metadata 169.254.169.254",
        ),
        ("http://[64:ff9b::a00:1]/x.png", "NAT64 private 10.0.0.1"),
        ("http://[2002:7f00:1::]/x.png", "6to4 loopback 127.0.0.1"),
        (
            "http://[2002:a9fe:a9fe::]/x",
            "6to4 metadata 169.254.169.254",
        ),
        (
            "http://[2002:c0a8:101::]/logo.png",
            "6to4 private 192.168.1.1",
        ),
    ] {
        let (status, body) = proxy_get(&c, &base, target).await;
        assert_eq!(
            status, 403,
            "SSRF: {what} ({target}) must be refused live (got {status})"
        );
        // The half that makes the leg non-vacuous: refused BY THE SSRF GATE. Without
        // this, deleting the NAT64/6to4 decode leaves the address permitted by egress
        // and refused by the grant gate — same 403, test still green, control gone.
        assert_eq!(
            body, EGRESS_REFUSAL,
            "SSRF: {what} ({target}) must be refused by the EGRESS gate, not by another \
             403 (got {body:?}); a {GRANT_REFUSAL:?} here means the decode PERMITTED the \
             smuggled address and this leg is no longer testing it"
        );
        denied_body = body;
    }

    // A PUBLIC v4 via NAT64 (8.8.8.8) is PERMITTED by the SSRF gate. It used to be
    // asserted as "not a 403" — since 26.20 the request is then refused by the grant
    // gate (this session holds none), so the address-level statement moves to the body,
    // where it is strictly more specific than the status ever was: not merely "some
    // other status", but "refused by the OTHER gate, having passed this one".
    let (public_status, public_body) =
        proxy_get(&c, &base, "http://[64:ff9b::808:808]/x.png").await;
    assert_ne!(
        public_body, EGRESS_REFUSAL,
        "a public v4 via NAT64 (8.8.8.8) must NOT be refused by the SSRF gate \
         (got {public_status} {public_body:?})"
    );
    assert_eq!(
        public_body, GRANT_REFUSAL,
        "a public v4 via NAT64 (8.8.8.8) must reach the grant gate, which only runs \
         once the egress policy has permitted the address (got {public_status} \
         {public_body:?})"
    );

    // THE INSTRUMENT ITSELF. Everything above distinguishes two refusals by their
    // body text; that is a valid instrument only while the two texts actually differ.
    // Both of these were produced by the real route moments ago, so if a reword or a
    // refactor ever routes both gates through one message, this fails here rather than
    // quietly turning every assertion above back into "the route returns 403".
    assert_ne!(
        denied_body, public_body,
        "the egress refusal and the grant refusal must be distinguishable, or every \
         assertion in this test degrades to a bare status check"
    );

    // Neither body is reachable without a session: `proxy_image` calls `crate::authed`
    // before both gates, so an unauthenticated caller is told nothing about the
    // address. (Between two AUTHENTICATED sessions the distinction IS visible — that
    // is precisely what the assertions above depend on.)
    let (anon_status, anon_body) =
        proxy_get(&client(), &base, "http://[64:ff9b::7f00:1]/x.png").await;
    assert_eq!(
        anon_status, 401,
        "the image proxy requires a session (got {anon_status})"
    );
    assert_ne!(
        anon_body, EGRESS_REFUSAL,
        "no refusal detail without a session"
    );
    assert_ne!(
        anon_body, GRANT_REFUSAL,
        "no refusal detail without a session"
    );
}
