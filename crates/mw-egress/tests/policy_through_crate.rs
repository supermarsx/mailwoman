//! t22-e6 — drive the egress policy through `mw-egress`'s **public** surface.
//!
//! The unit tests inside `src/lib.rs` moved here with the code they cover, so they
//! prove the policy still behaves — but they see private items and they run inside
//! the crate. That is exactly the shape that can pass while the *public* API a
//! consumer actually reaches for behaves differently: `mw-autoconfig`,
//! `mw-crypto`, the proxy transport and the image proxy all call this crate from
//! outside, and nothing in an in-crate test exercises that path.
//!
//! This file is an external consumer. It links `mw_egress` the way those crates
//! will and asserts, through nothing but the published API:
//!
//!   * the deny-by-default address policy, including the v6-smuggled forms;
//!   * that the URL/scheme/credential gate refuses before any socket is opened;
//!   * that `fetch_url_hardened` — the one-call reuse hook the ungated surfaces
//!     will adopt — refuses a literal metadata/loopback target;
//!   * that **`.no_proxy()` survived the extraction** (26.19 `fe09384`). This is
//!     the regression that would be silent: with an ambient or explicit proxy the
//!     `.resolve()` pin is never consulted, a third party resolves the name, and
//!     every fetch still succeeds — so only an assertion on the *proxy's own
//!     socket* can see it.

use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use mw_egress::{Refusal, embedded_ipv4s, fetch_url_hardened, harden_client, ip_allowed};

// ── the address policy, through the public fn ─────────────────────────────────

#[test]
fn public_ip_policy_denies_by_default() {
    for s in [
        "127.0.0.1",
        "10.0.0.1",
        "172.16.0.1",
        "192.168.1.1",
        "169.254.169.254", // cloud metadata
        "100.64.0.1",      // CGNAT
        "0.0.0.0",
        "::1",
        "fc00::1",
        "fe80::1",
        "::ffff:127.0.0.1",   // IPv4-mapped loopback
        "64:ff9b::a9fe:a9fe", // NAT64-embedded metadata
        "2002:7f00:1::",      // 6to4-embedded loopback
        // Teredo, public server, client v4 = 169.254.169.254 (obfuscated).
        "2001:0:4136:e378:8000:ffff:5601:5601",
        "2001:470::5efe:7f00:1", // ISATAP-embedded loopback
    ] {
        let ip: IpAddr = s.parse().unwrap();
        assert!(
            !ip_allowed(&ip),
            "{s} must be denied through the public API"
        );
    }
    for s in [
        "1.1.1.1",
        "8.8.8.8",
        "2606:2800:220:1::1",
        "64:ff9b::808:808",
    ] {
        let ip: IpAddr = s.parse().unwrap();
        assert!(
            ip_allowed(&ip),
            "{s} must be allowed through the public API"
        );
    }
}

#[test]
fn public_ip_policy_denies_rfc1918_which_is_what_makes_the_sieve_policy_narrower() {
    // `mw-server`'s ManageSieve caller deliberately KEEPS RFC1918 reachable — an
    // internal ManageSieve server is a legitimate target — while still refusing
    // metadata/loopback/link-local. That asymmetry only exists if THIS policy is
    // the strict one. If a future edit relaxed private ranges here, the Sieve
    // policy would silently stop being narrower and this assertion is what fails.
    for s in ["10.0.0.1", "172.31.255.255", "192.168.1.1", "fc00::1"] {
        let ip: IpAddr = s.parse().unwrap();
        assert!(
            !ip_allowed(&ip),
            "{s}: the general egress policy must refuse private ranges"
        );
    }
}

#[test]
fn public_embedded_decode_is_reachable_for_the_narrower_callers() {
    // `sieve_sync` applies its own rule to these decoded addresses, so the decode
    // itself has to be public, not just `ip_allowed`'s use of it.
    let teredo: Ipv6Addr = "2001:0:4136:e378:8000:ffff:80ff:fffe".parse().unwrap();
    let decoded = embedded_ipv4s(&teredo);
    assert_eq!(decoded.len(), 2, "Teredo carries a server AND a client v4");
    assert_eq!(decoded[1].to_string(), "127.0.0.1");

    let isatap: Ipv6Addr = "2001:470::5efe:a9fe:a9fe".parse().unwrap();
    assert_eq!(
        embedded_ipv4s(&isatap)
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>(),
        vec!["169.254.169.254".to_string()]
    );

    let plain: Ipv6Addr = "2606:2800:220:1::1".parse().unwrap();
    assert!(
        embedded_ipv4s(&plain).is_empty(),
        "a non-transitional address decodes to nothing"
    );
}

// ── the one-call reuse hook the ungated surfaces will adopt ───────────────────

#[tokio::test]
async fn fetch_url_hardened_refuses_the_scheme_credential_and_address_gates() {
    // Non-http(s) scheme.
    assert_eq!(
        fetch_url_hardened("file:///etc/passwd", "*/*")
            .await
            .unwrap_err(),
        "only http/https URLs are proxied"
    );
    // Credentials in the URL.
    assert_eq!(
        fetch_url_hardened("http://user:pw@example.com/x", "*/*")
            .await
            .unwrap_err(),
        "credentials in URL are not allowed"
    );
    // Literal metadata / loopback / private targets — refused at the gate, so no
    // socket is opened at all.
    for u in [
        "http://169.254.169.254/latest/meta-data/",
        "http://127.0.0.1/x",
        "http://[::1]/x",
        "http://10.0.0.5/x",
    ] {
        assert_eq!(
            fetch_url_hardened(u, "*/*").await.unwrap_err(),
            "target address is not permitted",
            "{u} must be refused by the address gate"
        );
    }
}

#[tokio::test]
async fn refusal_discriminants_are_public_and_distinguish_bad_request_from_blocked() {
    // A consumer that maps refusals onto its own error type needs the variants,
    // not just the strings.
    let err = mw_egress::validate_and_resolve(reqwest::Url::parse("gopher://x/1").unwrap())
        .await
        .unwrap_err();
    assert!(matches!(err, Refusal::BadRequest(_)), "{err:?}");

    let err = mw_egress::validate_and_resolve(
        reqwest::Url::parse("http://169.254.169.254/latest/").unwrap(),
    )
    .await
    .unwrap_err();
    assert_eq!(err, Refusal::Blocked);
}

// ── `.no_proxy()` survived the move (26.19 fe09384) ───────────────────────────

async fn spawn_origin(body: &'static [u8]) -> SocketAddr {
    let app = axum::Router::new().route(
        "/img",
        axum::routing::get(move || async move { axum::body::Bytes::from_static(body) }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

#[tokio::test]
async fn hardened_client_from_outside_the_crate_still_refuses_every_proxy() {
    // The proven bypass this closes: with a proxy in play the request is sent to
    // the proxy BY NAME, so the proxy resolves it and `.resolve()` is dead code.
    // The origin host is `pinned.invalid` (RFC 6761 — unresolvable in DNS
    // anywhere), so a successful body proves the pin, and a proxy hit count of 0
    // proves nothing was handed the hostname.
    let origin = spawn_origin(b"from-the-pinned-origin").await;

    let observer = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = observer.local_addr().unwrap();
    let hits = Arc::new(AtomicUsize::new(0));
    {
        let hits = Arc::clone(&hits);
        tokio::spawn(async move {
            while let Ok((stream, _)) = observer.accept().await {
                hits.fetch_add(1, Ordering::SeqCst);
                drop(stream);
            }
        });
    }

    let client = harden_client(
        reqwest::Client::builder()
            .proxy(reqwest::Proxy::all(format!("http://{proxy_addr}")).unwrap()),
        "pinned.invalid",
        origin,
    )
    .build()
    .unwrap();

    let resp = client
        .get("http://pinned.invalid/img")
        .send()
        .await
        .expect("the pinned address must be contacted, not the proxy");
    assert_eq!(resp.status().as_u16(), 200);
    assert_eq!(
        resp.bytes().await.unwrap().as_ref(),
        b"from-the-pinned-origin"
    );
    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "no proxy may receive the request — it would resolve the name itself"
    );
}

#[tokio::test]
async fn hardened_client_disables_redirect_following_so_every_hop_is_re_validated() {
    // Redirect re-validation is the caller's loop, and it only works because the
    // client itself never follows a hop. A builder that regained the default
    // redirect policy would follow a `Location` past the gate.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let app = axum::Router::new().route(
            "/img",
            axum::routing::get(|| async {
                (
                    axum::http::StatusCode::FOUND,
                    [(axum::http::header::LOCATION, "http://169.254.169.254/")],
                )
            }),
        );
        axum::serve(listener, app).await.unwrap();
    });

    let client = harden_client(reqwest::Client::builder(), "pinned.invalid", addr)
        .build()
        .unwrap();
    let resp = client
        .get("http://pinned.invalid/img")
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status().as_u16(),
        302,
        "the redirect must be surfaced, not followed"
    );
}
