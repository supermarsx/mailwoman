//! t16-e-e2e — anonymizing image proxy: LIVE SSRF refusal + the anonymize transform.
//!
//! The image proxy fetches attacker-controlled URLs from email HTML, so its SSRF gate
//! is the headline. This leg drives the REAL `/api/image-proxy` route (mounted by
//! `build_app`, behind a real session, over a loopback socket):
//!
//!   * SSRF-BLOCK LIVE (the headline negative): a request targeting loopback,
//!     link-local cloud-metadata (`169.254.169.254`), IPv6 loopback, or a private
//!     address is REFUSED live (403) — the request never leaves for an internal host.
//!   * scheme/credential smuggling (`file://…`, `http://user:pw@…`) is refused (400).
//!   * the proxy REQUIRES a session — an unauthenticated request is 401 (never an open
//!     relay).
//!
//! The ANONYMIZE half (re-encode → metadata-stripped PNG) is proven on REAL fetched
//! bytes through the exact seam the proxy invokes (`mw_render::media_jail::reencode_image`)
//! — a truly-local origin cannot be proxied end-to-end because the SSRF gate correctly
//! forbids loopback egress, so the positive transform is exercised at that jail seam
//! (the size-cap / timeout / redirect-hop mechanics past the gate are unit-covered in
//! `image_proxy.rs`, which bypasses the gate with a pinned loopback target).

use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::{Value, json};

use mw_server::{AppConfig, build_app};

fn unique() -> String {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    format!(
        "{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

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
    let base = std::env::temp_dir().join(format!("mw-t16-imgproxy-{}", unique()));
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

/// Log in (no 2FA enrolled) so the client holds a real session cookie.
async fn login(c: &reqwest::Client, base: &str, mock: &str) {
    let resp = c
        .post(format!("{base}/api/login"))
        .json(&json!({ "jmapUrl": mock, "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS }))
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    assert_eq!(
        body["ok"],
        json!(true),
        "plain login succeeds (no 2FA): {body}"
    );
}

async fn proxy_get(c: &reqwest::Client, base: &str, target: &str) -> reqwest::StatusCode {
    let url = format!("{base}/api/image-proxy?url={}", urlencoding(target));
    c.get(url).send().await.unwrap().status()
}

/// Minimal percent-encoding for a URL query value (no external dep).
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

// ── LIVE SSRF refusal through the real router ────────────────────────────────

#[tokio::test]
async fn ssrf_targets_are_refused_live_through_the_real_proxy() {
    let mock = spawn_mock().await;
    let addr = spawn_server().await;
    let base = format!("http://{addr}");
    let c = client();
    login(&c, &base, &mock).await;

    // The headline: private / loopback / cloud-metadata targets are REFUSED live. A
    // deliberately coarse 403 ("target address is not permitted") — never a fetch.
    for target in [
        "http://127.0.0.1/x.png",
        "http://169.254.169.254/latest/meta-data/", // cloud metadata
        "http://[::1]/x.png",                       // IPv6 loopback
        "http://10.0.0.5/x.png",                    // private
        "http://192.168.1.1/logo.png",              // private
        "http://[::ffff:127.0.0.1]/x",              // IPv4-mapped loopback
    ] {
        let status = proxy_get(&c, &base, target).await;
        assert_eq!(
            status, 403,
            "SSRF: {target} must be refused live by the proxy (got {status})"
        );
    }

    // Scheme / credential smuggling is a 400 (bad request), not a fetch.
    for target in [
        "file:///etc/passwd",
        "ftp://example.com/x",
        "http://user:pw@example.com/x.png",
    ] {
        let status = proxy_get(&c, &base, target).await;
        assert_eq!(status, 400, "{target} must be a bad request (got {status})");
    }
}

#[tokio::test]
async fn proxy_requires_a_session_never_an_open_relay() {
    let addr = spawn_server().await;
    let base = format!("http://{addr}");
    // No login → no session cookie. Even a well-formed (public) URL is refused pre-auth,
    // so the proxy can never be abused as an open relay.
    let c = client();
    let status = proxy_get(&c, &base, "http://example.com/logo.png").await;
    assert_eq!(status, 401, "the proxy requires a session (got {status})");
}

// ── the anonymize transform on REAL fetched bytes ────────────────────────────

#[tokio::test]
async fn fetched_image_is_reencoded_to_a_stripped_png() {
    // A local origin serves a GIF carrying trailing "metadata" bytes. We fetch it for
    // real, then run the SAME jail re-encode the proxy applies post-fetch, and assert
    // the output is a normalized PNG with the metadata gone.
    use axum::Router;
    use axum::routing::get;

    // 1×1 GIF + a recognisable trailing comment the re-encode must drop.
    let mut gif: Vec<u8> = vec![
        0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00,
        0x00, 0xFF, 0xFF, 0xFF, 0x21, 0xF9, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2C, 0x00, 0x00,
        0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00, 0x3B,
    ];
    let marker = b"SECRET-EXIF-MARKER-XYZ";
    gif.extend_from_slice(marker);

    let body = gif.clone();
    let app: Router = Router::new().route(
        "/img.gif",
        get(move || {
            let b = body.clone();
            async move { ([("content-type", "image/gif")], b) }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let raw = reqwest::get(format!("http://{addr}/img.gif"))
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap()
        .to_vec();
    assert!(
        raw.windows(marker.len()).any(|w| w == marker),
        "the source image carries the metadata marker before re-encode"
    );

    // The proxy's post-fetch transform: wasm-jail re-encode → metadata-stripped PNG.
    let png = mw_render::media_jail::reencode_image(&raw).expect("re-encode in the jail");
    assert_eq!(
        &png[..8],
        &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A],
        "output is a PNG"
    );
    assert!(
        !png.windows(marker.len()).any(|w| w == marker),
        "the metadata marker is stripped by the re-encode (anonymized)"
    );
}
