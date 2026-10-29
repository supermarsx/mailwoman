//! Per-phase fetch timeouts and the redirect cap (t56), against slow local origins.
//!
//! Each origin stalls at one phase — after accepting (a TLS handshake that never
//! starts), after reading the request (no headers), or mid-body — and each test
//! asserts that the matching bound fires, within tolerance, and is reported.
//!
//! # Why the origin is on this machine's LAN address
//! The direct path refuses loopback for every class, so the only origin a test can
//! reach through the real gate is one on a private address that the internal
//! allowlist admits (here for `siem`, an eligible class). The tests find the
//! machine's own RFC1918 address (a UDP `connect` sends nothing) and listen there;
//! on a machine without one they say so and return. No packet leaves the machine.
//!
//! The bounds are not class-specific: `Request::timeouts` applies to whatever class
//! sets it, and the remote-image proxy sets it for `remote_content`.
//!
//! Run:
//!   cargo test -p mw-egress --test fetch_timeouts -- --test-threads=1

use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use mw_egress::plan::{self, EgressPlan, Request};
use mw_egress::{
    AllowDestination, AllowEntry, FetchTimeouts, InternalAllowList, Refusal, TimeoutPhase,
    TimeoutReport, TrafficClass,
};

fn lan_address() -> Option<IpAddr> {
    let s = UdpSocket::bind("0.0.0.0:0").ok()?;
    s.connect("192.0.2.1:9").ok()?;
    let ip = s.local_addr().ok()?.ip();
    match ip {
        IpAddr::V4(v4) if v4.is_private() => Some(ip),
        _ => None,
    }
}

fn plan_for(ip: IpAddr) -> EgressPlan {
    EgressPlan::new(
        Vec::new(),
        InternalAllowList::new(vec![
            AllowEntry::new(
                TrafficClass::Siem,
                AllowDestination::parse(&ip.to_string()).unwrap(),
            )
            .unwrap(),
        ]),
    )
    .unwrap()
}

async fn read_head(stream: &mut TcpStream) {
    let mut head = Vec::new();
    let mut byte = [0u8; 1];
    while stream.read(&mut byte).await.unwrap_or(0) == 1 {
        head.push(byte[0]);
        if head.ends_with(b"\r\n\r\n") || head.len() > 16384 {
            break;
        }
    }
}

/// What an origin does with each connection.
#[derive(Clone, Copy)]
enum Stall {
    /// Accept and say nothing: an https client waits for a ServerHello forever.
    Handshake,
    /// Read the request and never answer.
    Headers,
    /// Send headers and 10 of 100 body bytes, then stall.
    MidBody,
    /// Send 100 body bytes one at a time, `gap` apart.
    Trickle(Duration),
    /// Redirect to the same path with `n+1`, forever.
    RedirectLoop,
    /// Redirect to a loopback address (the gate must refuse the hop).
    RedirectToLoopback,
}

async fn origin(ip: IpAddr, stall: Stall) -> (SocketAddr, Arc<AtomicUsize>) {
    let listener = TcpListener::bind(SocketAddr::new(ip, 0)).await.unwrap();
    let addr = listener.local_addr().unwrap();
    let hits = Arc::new(AtomicUsize::new(0));
    let h = Arc::clone(&hits);
    tokio::spawn(async move {
        while let Ok((mut s, _)) = listener.accept().await {
            h.fetch_add(1, Ordering::SeqCst);
            tokio::spawn(async move {
                match stall {
                    Stall::Handshake => {
                        tokio::time::sleep(Duration::from_secs(60)).await;
                    }
                    Stall::Headers => {
                        read_head(&mut s).await;
                        tokio::time::sleep(Duration::from_secs(60)).await;
                    }
                    Stall::MidBody => {
                        read_head(&mut s).await;
                        let _ = s
                            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\n0123456789")
                            .await;
                        let _ = s.flush().await;
                        tokio::time::sleep(Duration::from_secs(60)).await;
                    }
                    Stall::Trickle(gap) => {
                        read_head(&mut s).await;
                        let _ = s.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\n").await;
                        for _ in 0..100 {
                            if s.write_all(b"x").await.is_err() {
                                return;
                            }
                            let _ = s.flush().await;
                            tokio::time::sleep(gap).await;
                        }
                    }
                    Stall::RedirectLoop => {
                        read_head(&mut s).await;
                        let _ = s
                            .write_all(b"HTTP/1.1 302 Found\r\nLocation: /again\r\nContent-Length: 0\r\n\r\n")
                            .await;
                    }
                    Stall::RedirectToLoopback => {
                        read_head(&mut s).await;
                        let _ = s
                            .write_all(b"HTTP/1.1 302 Found\r\nLocation: http://127.0.0.1:9/x.png\r\nContent-Length: 0\r\n\r\n")
                            .await;
                    }
                }
            });
        }
    });
    (addr, hits)
}

fn bounds(connect: u64, headers: u64, idle: u64, total: u64, max_redirects: usize) -> FetchTimeouts {
    FetchTimeouts {
        connect: Duration::from_millis(connect),
        headers: Duration::from_millis(headers),
        idle: Duration::from_millis(idle),
        total: Duration::from_millis(total),
        max_redirects,
    }
}

/// Fetch `url` under `limits`; the refusal, the phase reported and the elapsed time.
async fn timed(
    plan: &EgressPlan,
    url: &str,
    limits: FetchTimeouts,
) -> (Result<Vec<u8>, Refusal>, Option<TimeoutPhase>, Duration) {
    let report = TimeoutReport::new();
    let req = Request::get(TrafficClass::Siem)
        .timeouts(limits)
        .timeout_report(&report);
    let started = Instant::now();
    let fetched = plan::send_with(plan, reqwest::Url::parse(url).unwrap(), &req).await;
    (fetched.outcome, report.phase(), started.elapsed())
}

/// `elapsed` is the bound, give or take scheduling: not early, and well short of
/// the next-larger bound in each test.
fn near(elapsed: Duration, bound_ms: u64) {
    let bound = Duration::from_millis(bound_ms);
    assert!(
        elapsed + Duration::from_millis(50) >= bound && elapsed < bound + Duration::from_millis(1500),
        "fired after {elapsed:?}, expected about {bound:?}"
    );
}

macro_rules! lan_or_skip {
    () => {
        match lan_address() {
            Some(ip) => ip,
            None => {
                eprintln!("no RFC1918 address on this machine; skipping");
                return;
            }
        }
    };
}

#[tokio::test]
async fn a_stalled_tls_handshake_is_the_connect_bound() {
    let ip = lan_or_skip!();
    let (addr, hits) = origin(ip, Stall::Handshake).await;
    let plan = plan_for(ip);
    // connect 700 ms, everything else far larger: only the connect clock can fire.
    let (outcome, phase, elapsed) = timed(
        &plan,
        &format!("https://{addr}/img.png"),
        bounds(700, 8_000, 8_000, 10_000, 3),
    )
    .await;
    assert!(matches!(outcome, Err(Refusal::Timeout)), "{outcome:?}");
    assert_eq!(phase, Some(TimeoutPhase::Connect));
    near(elapsed, 700);
    assert_eq!(hits.load(Ordering::SeqCst), 1, "TCP connected; TLS is what stalled");
}

#[tokio::test]
async fn no_response_headers_is_the_headers_bound() {
    let ip = lan_or_skip!();
    let (addr, _) = origin(ip, Stall::Headers).await;
    let (outcome, phase, elapsed) = timed(
        &plan_for(ip),
        &format!("http://{addr}/img.png"),
        bounds(5_000, 800, 8_000, 10_000, 3),
    )
    .await;
    assert!(matches!(outcome, Err(Refusal::Timeout)), "{outcome:?}");
    assert_eq!(phase, Some(TimeoutPhase::Headers));
    near(elapsed, 800);
}

#[tokio::test]
async fn a_body_that_stalls_mid_stream_is_the_idle_bound() {
    let ip = lan_or_skip!();
    let (addr, _) = origin(ip, Stall::MidBody).await;
    let (outcome, phase, elapsed) = timed(
        &plan_for(ip),
        &format!("http://{addr}/img.png"),
        bounds(5_000, 5_000, 900, 10_000, 3),
    )
    .await;
    assert!(matches!(outcome, Err(Refusal::Timeout)), "{outcome:?}");
    assert_eq!(phase, Some(TimeoutPhase::Idle));
    near(elapsed, 900);
}

#[tokio::test]
async fn a_trickling_body_never_idle_still_meets_the_total_deadline() {
    let ip = lan_or_skip!();
    // A byte every 100 ms never trips a 1 s idle clock; 100 bytes would take 10 s.
    let (addr, _) = origin(ip, Stall::Trickle(Duration::from_millis(100))).await;
    let (outcome, phase, elapsed) = timed(
        &plan_for(ip),
        &format!("http://{addr}/img.png"),
        bounds(5_000, 5_000, 1_000, 1_500, 3),
    )
    .await;
    assert!(matches!(outcome, Err(Refusal::Timeout)), "{outcome:?}");
    assert_eq!(phase, Some(TimeoutPhase::Total));
    near(elapsed, 1_500);
}

#[tokio::test]
async fn a_fast_origin_is_unaffected_and_reports_nothing() {
    let ip = lan_or_skip!();
    let (addr, _) = origin(ip, Stall::Trickle(Duration::ZERO)).await;
    let (outcome, phase, _) = timed(
        &plan_for(ip),
        &format!("http://{addr}/img.png"),
        bounds(1_000, 1_000, 1_000, 3_000, 3),
    )
    .await;
    assert_eq!(outcome.unwrap().len(), 100);
    assert_eq!(phase, None);
}

#[tokio::test]
async fn the_redirect_cap_is_the_callers_and_every_hop_reconnects() {
    let ip = lan_or_skip!();
    for cap in [0usize, 2, 3] {
        let (addr, hits) = origin(ip, Stall::RedirectLoop).await;
        let (outcome, phase, _) = timed(
            &plan_for(ip),
            &format!("http://{addr}/start"),
            bounds(1_000, 1_000, 1_000, 5_000, cap),
        )
        .await;
        assert!(matches!(outcome, Err(Refusal::Upstream)), "cap {cap}: {outcome:?}");
        assert_eq!(phase, None, "a redirect cap is not a timeout");
        assert_eq!(hits.load(Ordering::SeqCst), cap + 1, "cap {cap}: the first hop plus {cap} redirects");
    }
}

#[tokio::test]
async fn a_redirect_into_loopback_is_refused_by_the_gate_on_that_hop() {
    let ip = lan_or_skip!();
    let (addr, hits) = origin(ip, Stall::RedirectToLoopback).await;
    let (outcome, _, _) = timed(
        &plan_for(ip),
        &format!("http://{addr}/start"),
        bounds(1_000, 1_000, 1_000, 5_000, 3),
    )
    .await;
    assert!(matches!(outcome, Err(Refusal::Blocked)), "{outcome:?}");
    assert_eq!(hits.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn a_redirect_into_teredo_isatap_or_nat64_is_refused_with_timeouts_set() {
    // The gate itself, through the timed path: literals that embed a private or
    // loopback IPv4 are refused before any connection, exactly as without bounds.
    let plan = EgressPlan::direct();
    for url in [
        "http://[2001:0:4136:e378:8000:63bf:f5ff:fffe]/x", // Teredo -> 10.0.0.1
        "http://[fe80::5efe:7f00:1]/x",                    // ISATAP -> 127.0.0.1
        "http://[64:ff9b::a9fe:a9fe]/x",                   // NAT64 -> 169.254.169.254
        "http://127.0.0.1/x",
        "http://10.0.0.1/x",
    ] {
        let (outcome, phase, elapsed) = timed(&plan, url, bounds(1_000, 1_000, 1_000, 2_000, 3)).await;
        assert!(matches!(outcome, Err(Refusal::Blocked)), "{url}: {outcome:?}");
        assert_eq!(phase, None, "{url}");
        assert!(elapsed < Duration::from_millis(500), "{url}: refused without network");
    }
}

#[tokio::test]
async fn without_bounds_the_legacy_behaviour_is_unchanged() {
    let ip = lan_or_skip!();
    let (addr, hits) = origin(ip, Stall::RedirectLoop).await;
    let fetched = plan::send_with(
        &plan_for(ip),
        reqwest::Url::parse(&format!("http://{addr}/start")).unwrap(),
        &Request::get(TrafficClass::Siem),
    )
    .await;
    assert!(matches!(fetched.outcome, Err(Refusal::Upstream)));
    assert_eq!(hits.load(Ordering::SeqCst), mw_egress::MAX_REDIRECTS + 1);
}

// ── the same bounds through a configured egress route ───────────────────────────
//
// The stand-in `CONNECT` proxy listens on loopback (a route's own host is exempt
// from the address gate by design) and, after the tunnel, plays the origin with
// the given stall. The origin URL is `http://1.2.3.4/…`, which passes the gate and
// is only ever a string in the `CONNECT` line: no packet leaves the machine.

use mw_egress::plan::{PlanRoute, RouteScope};
use mw_egress::proxy::{ProxyRoute, ProxyScheme};

/// `None` = the proxy accepts TCP and never answers the CONNECT.
async fn stalling_proxy(stall: Option<Stall>) -> (SocketAddr, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let connects = Arc::new(AtomicUsize::new(0));
    let c = Arc::clone(&connects);
    tokio::spawn(async move {
        while let Ok((mut s, _)) = listener.accept().await {
            c.fetch_add(1, Ordering::SeqCst);
            tokio::spawn(async move {
                read_head(&mut s).await;
                let Some(stall) = stall else {
                    tokio::time::sleep(Duration::from_secs(60)).await;
                    return;
                };
                let _ = s.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n").await;
                match stall {
                    Stall::Headers => {
                        read_head(&mut s).await;
                        tokio::time::sleep(Duration::from_secs(60)).await;
                    }
                    Stall::MidBody => {
                        read_head(&mut s).await;
                        let _ = s
                            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\n0123456789")
                            .await;
                        let _ = s.flush().await;
                        tokio::time::sleep(Duration::from_secs(60)).await;
                    }
                    Stall::RedirectLoop => {
                        read_head(&mut s).await;
                        let _ = s
                            .write_all(b"HTTP/1.1 302 Found\r\nLocation: /again\r\nContent-Length: 0\r\n\r\n")
                            .await;
                    }
                    _ => unreachable!("not used through the proxy"),
                }
            });
        }
    });
    (addr, connects)
}

fn routed_plan(proxy: SocketAddr) -> EgressPlan {
    EgressPlan::new(
        vec![PlanRoute {
            scope: RouteScope::Global,
            classes: vec![TrafficClass::RemoteContent],
            route: ProxyRoute {
                id: "t56".into(),
                scheme: ProxyScheme::HttpConnect,
                host: proxy.ip().to_string(),
                port: proxy.port(),
                auth: None,
                allow_plaintext: true,
            },
        }],
        InternalAllowList::empty(),
    )
    .unwrap()
}

async fn routed(proxy: SocketAddr, limits: FetchTimeouts) -> (mw_egress::RoutedFetch, Option<TimeoutPhase>, Duration) {
    let report = TimeoutReport::new();
    let req = Request::get(TrafficClass::RemoteContent)
        .timeouts(limits)
        .timeout_report(&report);
    let started = Instant::now();
    let fetched = plan::send_with(
        &routed_plan(proxy),
        reqwest::Url::parse("http://1.2.3.4/img.png").unwrap(),
        &req,
    )
    .await;
    (fetched, report.phase(), started.elapsed())
}

#[tokio::test]
async fn through_a_route_a_silent_proxy_is_the_connect_bound_and_not_traversed() {
    let (proxy, _) = stalling_proxy(None).await;
    let (fetched, phase, elapsed) = routed(proxy, bounds(700, 8_000, 8_000, 10_000, 3)).await;
    assert!(matches!(fetched.outcome, Err(Refusal::Timeout)), "{:?}", fetched.outcome);
    assert_eq!(phase, Some(TimeoutPhase::Connect));
    assert!(!fetched.traversed_proxy, "the tunnel was never accepted");
    near(elapsed, 700);
}

#[tokio::test]
async fn through_a_route_stalled_headers_and_a_stalled_body_fire_their_bounds() {
    let (proxy, _) = stalling_proxy(Some(Stall::Headers)).await;
    let (fetched, phase, elapsed) = routed(proxy, bounds(5_000, 800, 8_000, 10_000, 3)).await;
    assert!(matches!(fetched.outcome, Err(Refusal::Timeout)));
    assert_eq!(phase, Some(TimeoutPhase::Headers));
    assert!(fetched.traversed_proxy, "a timeout after the tunnel still reports traversal");
    near(elapsed, 800);

    let (proxy, _) = stalling_proxy(Some(Stall::MidBody)).await;
    let (fetched, phase, elapsed) = routed(proxy, bounds(5_000, 5_000, 900, 10_000, 3)).await;
    assert!(matches!(fetched.outcome, Err(Refusal::Timeout)));
    assert_eq!(phase, Some(TimeoutPhase::Idle));
    near(elapsed, 900);
}

#[tokio::test]
async fn through_a_route_the_total_deadline_spans_hops_and_the_cap_is_the_callers() {
    // Headers stall on every hop: with a 1.2 s total and a 5 s headers bound the
    // total must fire, not the per-hop legacy 10 s.
    let (proxy, _) = stalling_proxy(Some(Stall::Headers)).await;
    let (fetched, phase, elapsed) = routed(proxy, bounds(5_000, 5_000, 5_000, 1_200, 3)).await;
    assert!(matches!(fetched.outcome, Err(Refusal::Timeout)));
    assert_eq!(phase, Some(TimeoutPhase::Total));
    near(elapsed, 1_200);

    let (proxy, connects) = stalling_proxy(Some(Stall::RedirectLoop)).await;
    let (fetched, _, _) = routed(proxy, bounds(1_000, 1_000, 1_000, 5_000, 1)).await;
    assert!(matches!(fetched.outcome, Err(Refusal::Upstream)), "{:?}", fetched.outcome);
    assert_eq!(connects.load(Ordering::SeqCst), 2, "one redirect, one fresh tunnel each");
}
