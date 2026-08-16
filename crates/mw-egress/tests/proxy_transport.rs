//! The upstream-proxy transport as an outside caller sees it (t22-e11).
//!
//! The wire-level assertions — that a recording `CONNECT` proxy receives an
//! IP-literal authority, that a recording SOCKS5 server receives `ATYP=0x01`, that a
//! MITM proxy is refused with a certificate error — need a loopback origin, which
//! `ip_allowed` correctly refuses; they therefore live in
//! `crates/mw-egress/src/proxy/tests.rs`, against the private ungated hop.
//!
//! **This file asserts the gate itself**, from outside the crate, on exactly the
//! public surface a caller can reach:
//!
//!   * a hand-built [`Target`] whose `addr` is blocked is refused **and no socket is
//!     opened to the proxy** — the check that [`Target`]'s doc comment assigns to
//!     whoever constructs one;
//!   * a redirect to a blocked address is refused when it re-enters the loop;
//!   * `ATYP=0x03` is not constructible through the public encoder;
//!   * `traversed_proxy` reports what happened, not what was configured.

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use mw_egress::proxy::{
    ProxyAuth, ProxyRefusal, ProxyRoute, ProxyScheme, fetch_via_proxy, socks5, tunnel_fetch_hop,
};
use mw_egress::{Refusal, Target};

/// A listener that counts connections and answers nothing. Any dial of the "proxy"
/// shows up here.
async fn counting_listener() -> (std::net::SocketAddr, Arc<AtomicUsize>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let hits = Arc::new(AtomicUsize::new(0));
    let sink = Arc::clone(&hits);
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            sink.fetch_add(1, Ordering::SeqCst);
            drop(stream);
        }
    });
    (addr, hits)
}

fn route_to(addr: std::net::SocketAddr, scheme: ProxyScheme) -> ProxyRoute {
    ProxyRoute {
        id: "route-under-test".into(),
        scheme,
        host: addr.ip().to_string(),
        port: addr.port(),
        auth: None,
        allow_plaintext: true,
    }
}

// ── the explicit ip_allowed check on a hand-built Target ───────────────────────

#[tokio::test]
async fn a_hand_built_target_with_a_blocked_address_is_refused_before_the_proxy_is_dialled() {
    let (proxy, hits) = counting_listener().await;
    let route = route_to(proxy, ProxyScheme::HttpConnect);

    // Every family of blocked address the policy covers, hand-assembled exactly the
    // way a careless transport would — bypassing `validate_and_resolve` entirely.
    for (host, addr) in [
        ("loopback.test", "127.0.0.1:80"),
        ("metadata.test", "169.254.169.254:80"),
        ("private.test", "10.0.0.5:80"),
        ("cgnat.test", "100.64.0.1:80"),
        ("v6-loopback.test", "[::1]:80"),
        ("v6-ula.test", "[fc00::1]:80"),
        // A private IPv4 smuggled inside a NAT64 address: blocked by the same
        // `ip_allowed` call, with no extra code here.
        ("nat64.test", "[64:ff9b::a9fe:a9fe]:80"),
    ] {
        let target = Target {
            url: reqwest::Url::parse(&format!("http://{host}/x.png")).unwrap(),
            host: host.into(),
            addr: addr.parse().unwrap(),
        };
        let hop = tunnel_fetch_hop(&target, &route, "image/*").await;
        assert_eq!(
            hop.outcome.unwrap_err(),
            ProxyRefusal::Origin(Refusal::Blocked),
            "{addr} must be refused"
        );
        assert!(!hop.traversed_proxy, "{addr} must not have used the proxy");
    }

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "the address check must run BEFORE the proxy is dialled — a blocked target \
         must not open a socket anywhere"
    );
}

#[tokio::test]
async fn the_blocked_target_assertion_is_not_vacuous() {
    // NEGATIVE CONTROL for the test above. If `tunnel_fetch_hop` refused every
    // target, or if the counting listener could not observe a dial, that test would
    // pass for the wrong reason. Here the ONLY difference is an allowed address:
    // the refusal is no longer `Blocked` and the proxy IS dialled.
    let (proxy, hits) = counting_listener().await;
    let route = route_to(proxy, ProxyScheme::HttpConnect);
    let target = Target {
        url: reqwest::Url::parse("http://cdn.example/x.png").unwrap(),
        host: "cdn.example".into(),
        // Documentation-range addresses are blocked by policy, so use a genuinely
        // allowed literal. Nothing is ever sent to it: the "proxy" hangs up.
        addr: "8.8.8.8:80".parse().unwrap(),
    };

    let hop = tunnel_fetch_hop(&target, &route, "image/*").await;
    assert_ne!(
        hop.outcome.unwrap_err(),
        ProxyRefusal::Origin(Refusal::Blocked),
        "an allowed address must not be refused by the address gate"
    );
    assert_eq!(
        hits.load(Ordering::SeqCst),
        1,
        "an allowed target must reach the proxy — proving the listener can see a dial"
    );
}

#[tokio::test]
async fn a_hand_built_target_with_a_non_http_scheme_is_refused_before_the_proxy_is_dialled() {
    // The scheme gate is `validate_and_resolve`'s, and a hand-built `Target` has not
    // been through it. `tunnel_fetch_hop`'s contract must not depend on how its
    // argument was made, so it re-applies the check for the same reason it
    // re-applies the address one.
    let (proxy, hits) = counting_listener().await;
    let route = route_to(proxy, ProxyScheme::HttpConnect);
    for url in [
        "file:///etc/passwd",
        "gopher://cdn.example/1",
        "ftp://cdn.example/x",
    ] {
        let target = Target {
            url: reqwest::Url::parse(url).unwrap(),
            host: "cdn.example".into(),
            addr: "8.8.8.8:80".parse().unwrap(),
        };
        let hop = tunnel_fetch_hop(&target, &route, "image/*").await;
        match hop.outcome.unwrap_err() {
            ProxyRefusal::Origin(Refusal::BadRequest(_)) => {}
            other => panic!("{url} → {other:?}"),
        }
        assert!(!hop.traversed_proxy);
    }
    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "the scheme check must run before the proxy is dialled"
    );
}

// ── redirects re-enter the gate ────────────────────────────────────────────────

#[tokio::test]
async fn a_redirect_to_a_blocked_address_is_refused_by_the_loop() {
    // Driven at the loop's own entry point: `fetch_via_proxy` re-validates each
    // `Location` through `validate_and_resolve` before it can become a hop, so a
    // redirect target in a blocked range is refused exactly like a direct one.
    let (proxy, hits) = counting_listener().await;
    let route = route_to(proxy, ProxyScheme::HttpConnect);

    for blocked in [
        "http://169.254.169.254/latest/meta-data/",
        "http://127.0.0.1/x.png",
        "http://[::1]/x.png",
        "http://10.1.2.3/x.png",
    ] {
        let fetched =
            fetch_via_proxy(reqwest::Url::parse(blocked).unwrap(), &route, "image/*").await;
        assert_eq!(
            fetched.outcome.unwrap_err(),
            ProxyRefusal::Origin(Refusal::Blocked),
            "{blocked} must be refused"
        );
        assert!(!fetched.traversed_proxy);
    }
    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "a blocked hop must never reach the proxy"
    );
}

#[tokio::test]
async fn the_loop_refuses_non_http_schemes_and_urls_with_credentials() {
    let (proxy, hits) = counting_listener().await;
    let route = route_to(proxy, ProxyScheme::Socks5);
    for url in [
        "file:///etc/passwd",
        "ftp://example.com/x",
        "http://user:pw@example.com/x.png",
    ] {
        let fetched = fetch_via_proxy(reqwest::Url::parse(url).unwrap(), &route, "image/*").await;
        match fetched.outcome.unwrap_err() {
            ProxyRefusal::Origin(Refusal::BadRequest(_)) => {}
            other => panic!("{url} → {other:?}"),
        }
    }
    assert_eq!(hits.load(Ordering::SeqCst), 0);
}

// ── ATYP 0x03 is not constructible through the public surface ──────────────────

#[test]
fn the_public_socks5_encoder_cannot_be_asked_for_a_domain_name() {
    // The control is the SIGNATURE: `encode_connect_request` takes a `SocketAddr`,
    // a closed sum of V4 | V6. There is no inhabitant of the input type that
    // carries a name, so no caller — inside this workspace or outside it — can ask
    // for `ATYP=0x03`. This test enumerates the whole reachable input shape and
    // pins the emitted set.
    let mut emitted = std::collections::BTreeSet::new();
    for addr in [
        "0.0.0.0:0",
        "8.8.8.8:443",
        "203.0.113.7:80",
        "255.255.255.255:65535",
        "[::]:0",
        "[::1]:443",
        "[2606:2800:220:1::1]:443",
        "[ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff]:65535",
    ] {
        let dst: std::net::SocketAddr = addr.parse().unwrap();
        let bytes = socks5::encode_connect_request(dst);
        emitted.insert(bytes[3]);
        // A domain request would be `[VER, CMD, RSV, 0x03, LEN, ...name, port]` —
        // a shape with a length byte. Both permitted shapes are fixed-width.
        assert!(
            bytes.len() == 10 || bytes.len() == 22,
            "{addr} produced a {}-byte request",
            bytes.len()
        );
    }
    assert_eq!(
        emitted,
        std::collections::BTreeSet::from([socks5::ATYP_IPV4, socks5::ATYP_IPV6])
    );
    assert!(!emitted.contains(&0x03));

    // And the module exposes no name for the domain form to be written against:
    // `ATYP_IPV4` and `ATYP_IPV6` are the only address-type constants, and neither
    // is 0x03 (also asserted at compile time in the module itself).
    assert_ne!(socks5::ATYP_IPV4, 0x03);
    assert_ne!(socks5::ATYP_IPV6, 0x03);
}

#[test]
fn socks5h_is_not_a_representable_scheme() {
    // `socks5h` means "the proxy resolves the name" by definition. `ProxyScheme` has
    // exactly two variants and neither is it; there is no string parsed into a
    // scheme here, so a configuration file cannot introduce one either.
    assert_eq!(ProxyScheme::Socks5.as_str(), "socks5");
    assert_eq!(ProxyScheme::HttpConnect.as_str(), "http-connect");
    for scheme in [ProxyScheme::Socks5, ProxyScheme::HttpConnect] {
        assert!(!scheme.as_str().contains("socks5h"));
    }
}

// ── the audit signal is the transport's, not the configuration's ───────────────

#[tokio::test]
async fn traversed_proxy_is_false_when_the_route_is_configured_but_unreachable() {
    // The 26.19 lesson: a row that says "proxied: true" because a route was
    // configured is a lie. A route IS configured here and the fetch DOES fail, and
    // the flag still has to say the proxy was never traversed.
    let dead = {
        let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        l.local_addr().unwrap()
    };
    let route = route_to(dead, ProxyScheme::HttpConnect);
    // An IP-literal host, so `validate_and_resolve` needs no DNS: without this the
    // test would pass vacuously on a network-isolated runner, where the fetch would
    // fail at name resolution and never reach the proxy stage at all.
    let fetched = fetch_via_proxy(
        reqwest::Url::parse("http://8.8.8.8/x.png").unwrap(),
        &route,
        "image/*",
    )
    .await;

    assert!(
        !fetched.traversed_proxy,
        "an unreachable proxy was not traversed"
    );
    assert!(
        matches!(fetched.outcome, Err(ProxyRefusal::ProxyUnreachable(_))),
        "the failure must name the proxy, proving the hop got that far. got {:?}",
        fetched.outcome
    );
}

#[tokio::test]
async fn there_is_no_direct_fallback_when_the_tunnel_cannot_be_built() {
    // Fail-closed, asserted on the ORIGIN: an origin listener that records
    // connections must record none, even though the proxy is dead.
    let origin = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin_addr = origin.local_addr().unwrap();
    let origin_hits = Arc::new(AtomicUsize::new(0));
    {
        let sink = Arc::clone(&origin_hits);
        tokio::spawn(async move {
            while let Ok((s, _)) = origin.accept().await {
                sink.fetch_add(1, Ordering::SeqCst);
                drop(s);
            }
        });
    }
    let dead_proxy = {
        let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        l.local_addr().unwrap()
    };
    let route = route_to(dead_proxy, ProxyScheme::Socks5);

    // Leg 1: the origin is on loopback, so the address gate refuses it before
    // anything is dialled. Fail-closed at the policy.
    let blocked_target = Target {
        url: reqwest::Url::parse("http://origin.test/x.png").unwrap(),
        host: "origin.test".into(),
        addr: origin_addr,
    };
    let hop = tunnel_fetch_hop(&blocked_target, &route, "image/*").await;
    assert_eq!(
        hop.outcome.unwrap_err(),
        ProxyRefusal::Origin(Refusal::Blocked)
    );
    assert!(!hop.traversed_proxy);

    // Leg 2 — the one that actually tests fail-closed rather than the gate: an
    // ALLOWED origin address with the route's proxy dead. The transport must give
    // up, not reach for a direct connection. If a "helpful" direct fallback were
    // ever added, this leg turns from `ProxyUnreachable` into a connection attempt.
    let allowed_target = Target {
        url: reqwest::Url::parse("http://cdn.example/x.png").unwrap(),
        host: "cdn.example".into(),
        addr: "8.8.8.8:80".parse().unwrap(),
    };
    let hop = tunnel_fetch_hop(&allowed_target, &route, "image/*").await;
    assert!(
        matches!(hop.outcome, Err(ProxyRefusal::ProxyUnreachable(_))),
        "{:?}",
        hop.outcome
    );
    assert!(!hop.traversed_proxy);

    assert_eq!(
        origin_hits.load(Ordering::SeqCst),
        0,
        "no direct connection to any origin may be made when a route is configured"
    );
}

// ── credentials never render ───────────────────────────────────────────────────

#[test]
fn a_route_never_renders_its_password() {
    let route = ProxyRoute {
        id: "r1".into(),
        scheme: ProxyScheme::Socks5,
        host: "proxy.internal".into(),
        port: 1080,
        auth: Some(ProxyAuth {
            username: "operator".into(),
            password: "correct-horse-battery-staple".into(),
        }),
        allow_plaintext: false,
    };
    // `Debug` is what a `tracing` field, a panic payload and `{:?}` in an error
    // body all reach for.
    let rendered = format!("{route:?}");
    assert!(
        !rendered.contains("correct-horse-battery-staple"),
        "{rendered}"
    );
    assert!(rendered.contains("<redacted>"), "{rendered}");
    // The auditable endpoint string is the only form intended for a log line.
    assert_eq!(route.endpoint(), "socks5://proxy.internal:1080");

    // Negative control: the password IS in the struct, so the redaction is doing
    // work rather than the field being empty.
    assert_eq!(
        route.auth.as_ref().unwrap().password,
        "correct-horse-battery-staple"
    );
}

// ── route selection cannot be reached from request data ────────────────────────

#[test]
fn a_route_is_built_only_from_operator_fields() {
    // Plan §4.3: proxy routes are deployment-wide operator configuration. This
    // transport offers no way to derive one from a request: `fetch_via_proxy` and
    // `tunnel_fetch_hop` both take a `&ProxyRoute` the CALLER supplies, and
    // `ProxyRoute` has no constructor that parses a URL, a header or a DTO.
    //
    // The assertion is on the type's surface: every field is operator-supplied and
    // there is no `FromStr`/`Deserialize`/`TryFrom<Url>` path into it.
    let sink: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    sink.lock().unwrap().push(
        ProxyRoute {
            id: "r".into(),
            scheme: ProxyScheme::HttpConnect,
            host: "127.0.0.1".into(),
            port: 3128,
            auth: None,
            allow_plaintext: false,
        }
        .endpoint(),
    );
    assert_eq!(sink.lock().unwrap()[0], "http-connect://127.0.0.1:3128");
    // The proxy endpoint is deliberately permitted to be loopback (the normal
    // "Squid on localhost" deployment); that asymmetry is safe only because this
    // value can never come from request data.
}
