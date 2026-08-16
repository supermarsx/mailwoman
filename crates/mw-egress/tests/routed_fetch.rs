//! `fetch_remote_routed` — route selection, and the fail-closed property (t22-e14).
//!
//! Transport-level behaviour is `proxy_transport.rs`'s and `src/proxy/tests.rs`'s.
//! **This file asserts the dispatch**: what happens when a route is configured, what
//! happens when one is not, and — the reason the file exists — what must *not*
//! happen when a configured route cannot be reached.
//!
//! # The bug these tests are shaped around
//! [`mw_egress::proxy::ProxyFetch`] makes the wrong caller easy to write:
//!
//! ```ignore
//! Err(_) => fetch_remote(url).await,   // reads as graceful degradation
//! ```
//!
//! That arm is a gate bypass twice over — it defeats the egress control an operator
//! configured, and because `fetch_via_proxy` returns `ProxyRefusal::Origin(r)` when
//! the **address policy** refused, it also silently retries, directly, a target the
//! SSRF gate has already said no to.
//!
//! # Why the counting listener is on loopback and the policy is permissive
//! An assertion that a fetch "returned an error" would pass for an implementation
//! that dialled the origin directly and failed for some other reason. The only thing
//! that separates the two is **whether the origin was contacted**, which needs a
//! listener we can count — and a listener we can bind is on loopback, which the
//! strict profile correctly refuses on *both* arms. Under the strict profile a
//! falling-back implementation and a fail-closed one are indistinguishable.
//!
//! So the fail-closed test runs under a permissive policy, where the direct path
//! genuinely *can* reach the counter. Its positive control proves that: with no
//! route the same call reaches the origin and `hits` becomes 1. Only then does
//! `hits` staying at 1 with a route configured mean anything.
//!
//! # No packet leaves the machine
//! The proxied-fetch tests use `http://1.2.3.4/…` as the origin because it is
//! ordinary public unicast — it passes `ip_allowed`, which a loopback origin cannot
//! — and the stand-in proxy answers `200 Connection established` and then serves the
//! response **itself** rather than connecting upstream. `1.2.3.4` is therefore only
//! ever a string in a `CONNECT` line; nothing dials it.
//!
//! Run:
//!   cargo test -p mw-egress --test routed_fetch -- --test-threads=1

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use mw_egress::proxy::{ProxyAuth, ProxyRoute, ProxyScheme};
use mw_egress::{Refusal, fetch_remote_routed, ip_allowed};

/// An address the strict profile permits, so a target built on it survives both
/// `validate_and_resolve` and `tunnel_fetch_hop`'s own re-check. Asserted rather
/// than assumed — if the policy ever changed, every proxied-fetch test below would
/// start refusing at the gate and would still *look* like a transport result.
const PUBLIC_ORIGIN: &str = "1.2.3.4";

/// A permissive address policy, used only where the test needs the direct path to
/// be able to reach a loopback listener. This is what makes the fail-closed
/// assertion discriminating; see the module docs.
fn allow_everything(_ip: &std::net::IpAddr) -> bool {
    true
}

/// An origin listener that counts every accepted connection. Nothing else — the
/// question it answers is "was this contacted at all", and a connection is the
/// whole of that.
async fn counting_origin() -> (SocketAddr, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let hits = Arc::new(AtomicUsize::new(0));
    let sink = Arc::clone(&hits);
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            sink.fetch_add(1, Ordering::SeqCst);
            // Answer, so the positive control completes as a fetch rather than as a
            // transport error that happens to have incremented the counter.
            tokio::spawn(async move {
                let mut scratch = [0u8; 1024];
                let _ = stream.read(&mut scratch).await;
                let _ = stream
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 6\r\n\r\ndirect")
                    .await;
                let _ = stream.flush().await;
            });
        }
    });
    (addr, hits)
}

/// A port with nothing behind it: bound to obtain a free number, then dropped, so
/// the connect is refused rather than filtered.
async fn dead_address() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    listener.local_addr().unwrap()
}

/// Read an HTTP head through `\r\n\r\n`, one byte at a time, leaving whatever
/// follows on the stream.
async fn read_head(stream: &mut tokio::net::TcpStream) -> String {
    let mut head = Vec::new();
    let mut byte = [0u8; 1];
    while stream.read(&mut byte).await.unwrap_or(0) == 1 {
        head.push(byte[0]);
        if head.ends_with(b"\r\n\r\n") || head.len() > 8192 {
            break;
        }
    }
    String::from_utf8_lossy(&head).into_owned()
}

/// A stand-in `CONNECT` proxy that records every `CONNECT` head, accepts the
/// tunnel, and then serves `body` **itself**. It never connects to the authority it
/// was given, which is what keeps these tests hermetic while still exercising a
/// complete tunnel → HTTP exchange.
///
/// `require_auth` makes it demand `Proxy-Authorization`, answering `407` when it is
/// absent. That is not decoration: a capture taken across a fetch through an
/// *unauthenticated* proxy proves nothing about a credential that was never
/// transmitted.
async fn standin_proxy(
    require_auth: bool,
    body: &'static str,
) -> (SocketAddr, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&log);
    tokio::spawn(async move {
        while let Ok((mut client, _)) = listener.accept().await {
            let sink = Arc::clone(&sink);
            tokio::spawn(async move {
                let head = read_head(&mut client).await;
                sink.lock().unwrap().push(head.clone());
                if require_auth && !head.contains("Proxy-Authorization:") {
                    let _ = client
                        .write_all(
                            b"HTTP/1.1 407 Proxy Authentication Required\r\n\
                              Proxy-Authenticate: Basic realm=\"standin\"\r\n\
                              Content-Length: 0\r\n\r\n",
                        )
                        .await;
                    return;
                }
                let _ = client
                    .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                    .await;
                // Consume the tunnelled request head, then answer it.
                let _ = read_head(&mut client).await;
                let _ = client
                    .write_all(
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
                            body.len()
                        )
                        .as_bytes(),
                    )
                    .await;
                let _ = client.flush().await;
            });
        }
    });
    (addr, log)
}

fn route_via(proxy: SocketAddr) -> ProxyRoute {
    ProxyRoute {
        id: "operator-route".into(),
        scheme: ProxyScheme::HttpConnect,
        host: proxy.ip().to_string(),
        port: proxy.port(),
        auth: None,
        // The origins here are plaintext because a stand-in proxy cannot present a
        // certificate chaining to the Mozilla roots. The opt-in is exercised on its
        // own in `src/proxy/tests.rs`.
        allow_plaintext: true,
    }
}

// ── the gate the whole file rests on ───────────────────────────────────────────

#[test]
fn the_public_origin_used_below_really_does_pass_the_strict_policy() {
    // If this ever stops holding, the proxied-fetch tests would be refusing at the
    // address gate while still producing an `Err` that reads like a transport
    // result. Asserting it here means that failure names itself.
    assert!(
        ip_allowed(&PUBLIC_ORIGIN.parse().unwrap()),
        "{PUBLIC_ORIGIN} must be ordinary public unicast for these tests to exercise \
         the transport rather than the gate"
    );
    assert!(
        !ip_allowed(&"127.0.0.1".parse().unwrap()),
        "and loopback must NOT pass, or the permissive-policy reasoning in this \
         file's docs is wrong"
    );
}

// ── FAIL-CLOSED: the demonstration ─────────────────────────────────────────────

#[tokio::test]
async fn a_configured_route_that_cannot_be_reached_never_reaches_the_origin() {
    let (origin, hits) = counting_origin().await;
    let url = reqwest::Url::parse(&format!("http://{origin}/img")).unwrap();

    // ── positive control, FIRST ───────────────────────────────────────────────
    // With no route the direct path reaches this origin under the permissive
    // policy. Without this leg, `hits == 0` below would be satisfied by an origin
    // nothing could ever have reached, and a broken implementation would pass.
    let direct = fetch_remote_routed(url.clone(), "image/*", allow_everything, None).await;
    assert_eq!(
        direct.outcome.as_deref(),
        Ok(&b"direct"[..]),
        "the control leg must SUCCEED, or the counter below proves nothing"
    );
    assert!(
        !direct.traversed_proxy,
        "a direct fetch traversed no proxy, and the audit row has to be able to say so"
    );
    assert_eq!(
        hits.load(Ordering::SeqCst),
        1,
        "the origin listener must actually count connections"
    );

    // ── the assertion ─────────────────────────────────────────────────────────
    // Same URL, same policy, same origin — the only change is that a route is
    // configured and its proxy is unreachable.
    let dead_proxy = dead_address().await;
    let route = route_via(dead_proxy);
    let routed = fetch_remote_routed(url, "image/*", allow_everything, Some(&route)).await;

    // The counting assertion goes FIRST, deliberately. `is_err()` would also catch
    // a fallback that happened to fail, and it would report the wrong thing; the
    // count is the claim — *the origin was not contacted* — so it is the assertion
    // that should fire, and be read, when this breaks.
    assert_eq!(
        hits.load(Ordering::SeqCst),
        1,
        "THE FAIL-CLOSED ASSERTION: the origin was contacted once (by the control \
         leg above) and not again. A second hit here means the implementation fell \
         back to a direct fetch when the tunnel failed — the gate bypass this \
         function exists to prevent, and exactly what an `Err(_) => \
         fetch_remote(url)` arm produces."
    );
    assert!(
        routed.outcome.is_err(),
        "a configured route that cannot be reached must FAIL the fetch, not complete \
         it some other way: {:?} bytes",
        routed.outcome.as_ref().map(|b| b.len())
    );
    assert!(
        !routed.traversed_proxy,
        "nothing traversed a proxy that was never reached"
    );
}

#[tokio::test]
async fn a_target_the_gate_already_refused_is_not_retried_directly() {
    // The second, subtler half of the same bug. `fetch_via_proxy` reports an
    // ORIGIN-POLICY refusal as `ProxyRefusal::Origin(_)`, so a caller matching on a
    // bare `Err(_)` would treat "the SSRF gate said no" as "the proxy is down" and
    // retry the refused target on the direct path. Under a permissive policy that
    // retry would SUCCEED, which is the whole problem.
    let (origin, hits) = counting_origin().await;
    let url = reqwest::Url::parse(&format!("http://{origin}/img")).unwrap();

    // A live proxy this time: the failure is the gate's, not the transport's.
    let (proxy, log) = standin_proxy(false, "tunnelled").await;
    let route = route_via(proxy);

    let routed = fetch_remote_routed(url, "image/*", allow_everything, Some(&route)).await;
    assert_eq!(
        routed.outcome.unwrap_err(),
        Refusal::Blocked,
        "the routed path resolves through the strict profile, so a loopback origin \
         is refused by the address gate"
    );
    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "a target the gate refused must not then be fetched directly"
    );
    assert!(
        log.lock().unwrap().is_empty(),
        "and the refusal happened before the proxy was dialled at all"
    );
}

// ── the documented asymmetry, pinned so it cannot drift quietly ────────────────

#[tokio::test]
async fn the_routed_path_is_stricter_than_the_policy_asks() {
    // `policy` is honoured on the direct arm and NOT consulted on the routed arm,
    // because `tunnel_fetch_hop` re-applies `ip_allowed` unconditionally by design.
    // The consequence is a real behaviour difference — a permissive caller reaches
    // LESS through a configured route — and it is in the fail-safe direction. It is
    // asserted here so that widening it later is a deliberate act with a red test,
    // rather than something a reader has to infer from two files.
    let (origin, _hits) = counting_origin().await;
    let url = reqwest::Url::parse(&format!("http://{origin}/img")).unwrap();
    let (proxy, _log) = standin_proxy(false, "tunnelled").await;
    let route = route_via(proxy);

    // Same permissive policy, same URL: allowed directly …
    let direct = fetch_remote_routed(url.clone(), "image/*", allow_everything, None).await;
    assert!(direct.outcome.is_ok(), "{:?}", direct.outcome);

    // … refused through the route.
    let routed = fetch_remote_routed(url, "image/*", allow_everything, Some(&route)).await;
    assert_eq!(routed.outcome.unwrap_err(), Refusal::Blocked);
}

// ── traversed_proxy is a measurement, and it moves ─────────────────────────────

#[tokio::test]
async fn a_completed_proxied_fetch_reports_that_it_traversed_the_proxy() {
    // The positive leg. Without it, every `!traversed_proxy` assertion in this file
    // would also pass against a field that was hardcoded `false`.
    let (proxy, log) = standin_proxy(false, "through-the-tunnel").await;
    let route = route_via(proxy);
    let url = reqwest::Url::parse(&format!("http://{PUBLIC_ORIGIN}/img")).unwrap();

    let routed = fetch_remote_routed(url, "image/*", ip_allowed, Some(&route)).await;
    assert_eq!(
        routed.outcome.as_deref(),
        Ok(&b"through-the-tunnel"[..]),
        "the body must arrive THROUGH the tunnel"
    );
    assert!(
        routed.traversed_proxy,
        "the bytes went through the proxy and the audit row must say so"
    );

    // And it really was the tunnel: the proxy saw a CONNECT carrying an IP-literal
    // authority, never the hostname form.
    let head = log.lock().unwrap()[0].clone();
    assert!(
        head.starts_with(&format!("CONNECT {PUBLIC_ORIGIN}:80 ")),
        "the proxy must have been asked for a literal address: {head}"
    );
}

#[tokio::test]
async fn a_failed_proxied_fetch_still_reports_traversal_truthfully() {
    // Three failures, three different points in the tunnel's life, one question:
    // does `traversed_proxy` describe what happened, or what was configured?
    let url = reqwest::Url::parse(&format!("http://{PUBLIC_ORIGIN}/img")).unwrap();

    // (a) proxy never reached → nothing traversed it.
    let route = route_via(dead_address().await);
    let routed = fetch_remote_routed(url.clone(), "image/*", ip_allowed, Some(&route)).await;
    assert_eq!(routed.outcome.unwrap_err(), Refusal::Upstream);
    assert!(
        !routed.traversed_proxy,
        "a route was configured, but no byte of ours reached the proxy — reporting \
         `true` here would be intent recorded as fact"
    );

    // (b) proxy reached but refused the tunnel → still not traversed.
    let (proxy, _log) = standin_proxy(true, "never served").await;
    let unauthenticated = route_via(proxy); // auth: None, and the proxy demands it
    let routed =
        fetch_remote_routed(url.clone(), "image/*", ip_allowed, Some(&unauthenticated)).await;
    assert_eq!(routed.outcome.unwrap_err(), Refusal::Upstream);
    assert!(
        !routed.traversed_proxy,
        "a refused tunnel carried nothing for us"
    );

    // (c) the same proxy WITH credentials → the tunnel opens. This is what makes
    // (b)'s `false` a measurement rather than a constant.
    let mut authenticated = route_via(proxy);
    authenticated.auth = Some(ProxyAuth {
        username: "operator".into(),
        password: "never-logged".into(),
    });
    let routed = fetch_remote_routed(url, "image/*", ip_allowed, Some(&authenticated)).await;
    assert!(routed.traversed_proxy, "{:?}", routed.outcome);
    assert_eq!(routed.outcome.as_deref(), Ok(&b"never served"[..]));
}
