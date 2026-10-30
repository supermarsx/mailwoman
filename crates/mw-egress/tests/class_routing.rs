//! Class-aware egress (t32 e31b): which route a class takes, what is sent through
//! it, and that a configured route is never bypassed.
//!
//! The plan's selection logic has unit tests in `src/plan.rs`. **This file asserts
//! the wire**: what a stand-in proxy and a stand-in origin actually receive.
//!
//! # No packet leaves the machine
//! Proxied requests use `http://1.2.3.4/…` as the origin because it passes
//! `ip_allowed`; the stand-in proxy answers the `CONNECT` and then serves the
//! response itself, so `1.2.3.4` is only ever a string in a `CONNECT` line.
//!
//! # Why some tests need this machine's LAN address
//! A fail-closed test must show that a fallback *would* have reached the origin,
//! or "the origin was not contacted" proves nothing. The direct arm refuses
//! loopback for every class, so the only origin the direct arm can reach in a test
//! is one on a private address that the internal allowlist admits for `siem`. The
//! tests find the machine's own RFC1918 address (a UDP `connect` sends nothing) and
//! listen there; on a machine without one they say so and return.
//!
//! Run:
//!   cargo test -p mw-egress --test class_routing -- --test-threads=1

use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use mw_egress::plan::{self, EgressPlan, PlanRoute, Request, RouteScope};
use mw_egress::proxy::{ProxyRoute, ProxyScheme};
use mw_egress::{AllowDestination, AllowEntry, InternalAllowList, Refusal, TrafficClass};

/// Read an HTTP head through `\r\n\r\n`, then `Content-Length` bytes of body.
async fn read_request(stream: &mut TcpStream) -> (String, Vec<u8>) {
    let mut head = Vec::new();
    let mut byte = [0u8; 1];
    while stream.read(&mut byte).await.unwrap_or(0) == 1 {
        head.push(byte[0]);
        if head.ends_with(b"\r\n\r\n") || head.len() > 16384 {
            break;
        }
    }
    let head = String::from_utf8_lossy(&head).into_owned();
    let len = head
        .lines()
        .find_map(|l| {
            let (k, v) = l.split_once(':')?;
            k.eq_ignore_ascii_case("content-length")
                .then(|| v.trim().parse::<usize>().ok())
                .flatten()
        })
        .unwrap_or(0);
    let mut body = vec![0u8; len];
    if len > 0 {
        let _ = stream.read_exact(&mut body).await;
    }
    (head, body)
}

/// What a stand-in saw: each `CONNECT` head, and each tunnelled request (head and
/// body).
#[derive(Default)]
struct Seen {
    connects: Vec<String>,
    requests: Vec<(String, Vec<u8>)>,
}

/// A stand-in `CONNECT` proxy that accepts the tunnel and answers the tunnelled
/// request itself with `reply` (a full HTTP response).
async fn standin_proxy(reply: &'static str) -> (SocketAddr, Arc<Mutex<Seen>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let seen = Arc::new(Mutex::new(Seen::default()));
    let sink = Arc::clone(&seen);
    tokio::spawn(async move {
        while let Ok((mut client, _)) = listener.accept().await {
            let sink = Arc::clone(&sink);
            tokio::spawn(async move {
                let (connect, _) = read_request(&mut client).await;
                sink.lock().unwrap().connects.push(connect);
                let _ = client
                    .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                    .await;
                let req = read_request(&mut client).await;
                sink.lock().unwrap().requests.push(req);
                let _ = client.write_all(reply.as_bytes()).await;
                let _ = client.flush().await;
            });
        }
    });
    (addr, seen)
}

/// A port with nothing listening.
async fn dead_address() -> SocketAddr {
    TcpListener::bind("127.0.0.1:0")
        .await
        .unwrap()
        .local_addr()
        .unwrap()
}

fn route_via(id: &str, proxy: SocketAddr) -> ProxyRoute {
    ProxyRoute {
        id: id.into(),
        scheme: ProxyScheme::HttpConnect,
        host: proxy.ip().to_string(),
        port: proxy.port(),
        auth: None,
        // Plaintext origins: a stand-in cannot present a certificate chaining to
        // the Mozilla roots. The TLS path is `src/proxy/tests.rs`'s.
        allow_plaintext: true,
    }
}

fn plan_with(routes: Vec<PlanRoute>, allow: InternalAllowList) -> EgressPlan {
    EgressPlan::new(routes, allow).expect("unambiguous plan")
}

fn global(route: ProxyRoute, classes: &[TrafficClass]) -> PlanRoute {
    PlanRoute {
        scope: RouteScope::Global,
        classes: classes.to_vec(),
        route,
    }
}

const OK_BODY: &str = "HTTP/1.1 200 OK\r\nContent-Length: 7\r\n\r\nproxied";

fn url(s: &str) -> reqwest::Url {
    reqwest::Url::parse(s).unwrap()
}

// ── which route a class takes ───────────────────────────────────────────────────

#[tokio::test]
async fn a_listed_class_goes_through_the_route_and_an_unlisted_one_does_not() {
    let (proxy, seen) = standin_proxy(OK_BODY).await;
    let plan = plan_with(
        vec![global(
            route_via("tor", proxy),
            &[TrafficClass::RemoteContent],
        )],
        InternalAllowList::empty(),
    );

    let routed = plan::send_with(
        &plan,
        url("http://1.2.3.4/pixel.png"),
        &Request::get(TrafficClass::RemoteContent).accept("image/*"),
    )
    .await;
    assert!(routed.traversed_proxy, "{routed:?}");
    assert_eq!(routed.outcome.as_deref(), Ok(&b"proxied"[..]));
    assert_eq!(seen.lock().unwrap().connects.len(), 1);
    assert!(
        seen.lock().unwrap().connects[0].starts_with("CONNECT 1.2.3.4:80 "),
        "the proxy is handed the resolved literal, never a name: {:?}",
        seen.lock().unwrap().connects
    );

    // `pki` is not listed, so it goes direct — under the strict gate, which refuses
    // this loopback origin before any socket opens. The proxy must not see it.
    let direct = plan::send_with(
        &plan,
        url("http://127.0.0.1:9/crl"),
        &Request::get(TrafficClass::Pki),
    )
    .await;
    assert!(!direct.traversed_proxy);
    assert_eq!(direct.outcome.unwrap_err(), Refusal::Blocked);
    assert_eq!(
        seen.lock().unwrap().connects.len(),
        1,
        "an unlisted class must not use the route"
    );
}

#[tokio::test]
async fn a_domain_route_carries_its_domains_requests_and_no_others() {
    let (global_proxy, global_seen) = standin_proxy(OK_BODY).await;
    let (corp_proxy, corp_seen) = standin_proxy(OK_BODY).await;
    let plan = plan_with(
        vec![
            global(
                route_via("global", global_proxy),
                &[TrafficClass::Keyservers],
            ),
            PlanRoute {
                scope: RouteScope::Domain("corp.example".into()),
                classes: vec![TrafficClass::Keyservers],
                route: route_via("corp", corp_proxy),
            },
        ],
        InternalAllowList::empty(),
    );
    let req = Request::get(TrafficClass::Keyservers);

    let corp = plan::send_with(
        &plan,
        url("http://1.2.3.4/wkd"),
        &req.for_domain("corp.example"),
    )
    .await;
    assert!(corp.traversed_proxy);
    let other = plan::send_with(
        &plan,
        url("http://1.2.3.4/wkd"),
        &req.for_domain("other.example"),
    )
    .await;
    assert!(other.traversed_proxy);
    let none = plan::send_with(&plan, url("http://1.2.3.4/wkd"), &req).await;
    assert!(none.traversed_proxy);

    assert_eq!(
        corp_seen.lock().unwrap().connects.len(),
        1,
        "only corp.example's request"
    );
    assert_eq!(
        global_seen.lock().unwrap().connects.len(),
        2,
        "other.example's and the account-less one"
    );
}

// ── what goes through the tunnel ────────────────────────────────────────────────

#[tokio::test]
async fn a_post_carries_its_body_and_the_class_user_agent_through_the_tunnel() {
    let (proxy, seen) = standin_proxy(OK_BODY).await;
    let plan = plan_with(
        vec![global(
            route_via("corp", proxy),
            &[TrafficClass::Unsubscribe],
        )],
        InternalAllowList::empty(),
    );
    let body = b"List-Unsubscribe=One-Click";
    let sent = plan::send_with(
        &plan,
        url("http://1.2.3.4/u/abc?x=1"),
        &Request::post(
            TrafficClass::Unsubscribe,
            "application/x-www-form-urlencoded",
            body,
        ),
    )
    .await;
    assert_eq!(sent.outcome.as_deref(), Ok(&b"proxied"[..]), "{sent:?}");

    let seen = seen.lock().unwrap();
    let (head, got) = &seen.requests[0];
    assert!(head.starts_with("POST /u/abc?x=1 HTTP/1.1\r\n"), "{head}");
    let lower = head.to_ascii_lowercase();
    assert!(
        lower.contains("content-type: application/x-www-form-urlencoded"),
        "{head}"
    );
    assert!(lower.contains("user-agent: mailwoman\r\n"), "{head}");
    assert!(
        !lower.contains("cookie:") && !lower.contains("referer:"),
        "{head}"
    );
    assert_eq!(got.as_slice(), body);
}

#[tokio::test]
async fn a_get_sends_no_body_and_the_callers_user_agent() {
    let (proxy, seen) = standin_proxy(OK_BODY).await;
    let plan = plan_with(
        vec![global(
            route_via("corp", proxy),
            &[TrafficClass::UpdateCheck],
        )],
        InternalAllowList::empty(),
    );
    let got = plan::send_with(
        &plan,
        url("http://1.2.3.4/releases"),
        &Request::get(TrafficClass::UpdateCheck)
            .accept("application/json")
            .user_agent("Mailwoman/26.20"),
    )
    .await;
    assert!(got.outcome.is_ok(), "{got:?}");
    let seen = seen.lock().unwrap();
    let head = seen.requests[0].0.to_ascii_lowercase();
    assert!(head.starts_with("get /releases http/1.1\r\n"), "{head}");
    assert!(head.contains("user-agent: mailwoman/26.20\r\n"), "{head}");
    assert!(head.contains("accept: application/json\r\n"), "{head}");
    assert!(
        !head.contains("content-length") && !head.contains("transfer-encoding"),
        "a GET carries no body framing, as before bodies existed: {head}"
    );
}

#[tokio::test]
async fn a_post_answered_with_a_redirect_is_not_followed() {
    let (proxy, seen) = standin_proxy(
        "HTTP/1.1 302 Found\r\nLocation: http://5.6.7.8/elsewhere\r\nContent-Length: 0\r\n\r\n",
    )
    .await;
    let plan = plan_with(
        vec![global(route_via("corp", proxy), &[TrafficClass::Webhooks])],
        InternalAllowList::empty(),
    );
    let sent = plan::send_with(
        &plan,
        url("http://1.2.3.4/hook"),
        &Request::post(TrafficClass::Webhooks, "application/json", b"{}"),
    )
    .await;
    assert_eq!(sent.outcome.unwrap_err(), Refusal::Upstream);
    assert_eq!(
        seen.lock().unwrap().connects.len(),
        1,
        "the body must not be re-sent to the Location"
    );
}

// ── fail-closed, demonstrated ────────────────────────────────────────────────────

/// This machine's own RFC1918 IPv4 address, if it has one. A UDP `connect` only
/// picks the outgoing interface; it sends nothing.
fn lan_address() -> Option<IpAddr> {
    let s = UdpSocket::bind("0.0.0.0:0").ok()?;
    s.connect("192.0.2.1:9").ok()?;
    let ip = s.local_addr().ok()?.ip();
    match ip {
        IpAddr::V4(v4) if v4.is_private() => Some(ip),
        _ => None,
    }
}

/// An origin on `ip` that counts connections and records the request.
async fn counting_origin(
    ip: IpAddr,
) -> (
    SocketAddr,
    Arc<AtomicUsize>,
    Arc<Mutex<Vec<(String, Vec<u8>)>>>,
) {
    let listener = TcpListener::bind(SocketAddr::new(ip, 0)).await.unwrap();
    let addr = listener.local_addr().unwrap();
    let hits = Arc::new(AtomicUsize::new(0));
    let log = Arc::new(Mutex::new(Vec::new()));
    let (h, l) = (Arc::clone(&hits), Arc::clone(&log));
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            h.fetch_add(1, Ordering::SeqCst);
            let l = Arc::clone(&l);
            tokio::spawn(async move {
                let req = read_request(&mut stream).await;
                l.lock().unwrap().push(req);
                let _ = stream
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 6\r\n\r\ndirect")
                    .await;
                let _ = stream.flush().await;
            });
        }
    });
    (addr, hits, log)
}

fn siem_allow(ip: IpAddr) -> InternalAllowList {
    InternalAllowList::new(vec![
        AllowEntry::new(
            TrafficClass::Siem,
            AllowDestination::parse(&ip.to_string()).unwrap(),
        )
        .unwrap(),
    ])
}

#[tokio::test]
async fn a_route_that_cannot_be_reached_never_reaches_the_origin_directly() {
    let Some(lan) = lan_address() else {
        eprintln!("SKIPPED: this machine has no RFC1918 address to host a reachable origin on");
        return;
    };
    let (origin, hits, _) = counting_origin(lan).await;
    let target = url(&format!("http://{origin}/ingest"));
    let req = Request::post(TrafficClass::Siem, "application/json", b"{\"e\":1}");

    // Positive control FIRST: with no route the direct arm reaches this origin,
    // because the allowlist admits it for `siem`. Without this, `hits` staying at
    // 1 below would prove nothing.
    let direct = plan::send_with(
        &plan_with(Vec::new(), siem_allow(lan)),
        target.clone(),
        &req,
    )
    .await;
    assert_eq!(direct.outcome.as_deref(), Ok(&b"direct"[..]), "{direct:?}");
    assert_eq!(hits.load(Ordering::SeqCst), 1);

    // Now a route to a dead proxy carries `siem`. The fetch fails, and the origin
    // is not contacted a second time.
    let plan = plan_with(
        vec![global(
            route_via("down", dead_address().await),
            &[TrafficClass::Siem],
        )],
        siem_allow(lan),
    );
    let routed = plan::send_with(&plan, target, &req).await;
    assert!(routed.outcome.is_err(), "{routed:?}");
    assert!(!routed.traversed_proxy);
    assert_eq!(
        hits.load(Ordering::SeqCst),
        1,
        "a configured route failed and the request went direct anyway"
    );
}

#[tokio::test]
async fn the_allowlist_admits_its_class_on_the_direct_arm_and_nothing_else() {
    let Some(lan) = lan_address() else {
        eprintln!("SKIPPED: this machine has no RFC1918 address to host a reachable origin on");
        return;
    };
    let (origin, hits, log) = counting_origin(lan).await;
    let target = url(&format!("http://{origin}/ingest"));
    let plan = plan_with(Vec::new(), siem_allow(lan));

    let ok = plan::send_with(
        &plan,
        target.clone(),
        &Request::post(TrafficClass::Siem, "application/json", b"{\"e\":2}"),
    )
    .await;
    assert_eq!(ok.outcome.as_deref(), Ok(&b"direct"[..]), "{ok:?}");
    {
        let log = log.lock().unwrap();
        assert!(
            log[0].0.starts_with("POST /ingest HTTP/1.1\r\n"),
            "{}",
            log[0].0
        );
        assert_eq!(log[0].1, b"{\"e\":2}");
    }

    for c in TrafficClass::ALL {
        if c == TrafficClass::Siem {
            continue;
        }
        let r = plan::send_with(&plan, target.clone(), &Request::get(c)).await;
        let expected = if matches!(
            c,
            TrafficClass::Push | TrafficClass::PackageTracking | TrafficClass::GeoipDatabase
        ) {
            Refusal::BadRequest("this traffic class requires HTTPS")
        } else {
            Refusal::Blocked
        };
        assert_eq!(r.outcome.unwrap_err(), expected, "{c}");
    }
    assert_eq!(
        hits.load(Ordering::SeqCst),
        1,
        "no other class reached the private origin"
    );
}

#[tokio::test]
async fn a_route_for_other_classes_leaves_an_unlisted_class_direct() {
    let Some(lan) = lan_address() else {
        eprintln!("SKIPPED: this machine has no RFC1918 address to host a reachable origin on");
        return;
    };
    let (origin, hits, _) = counting_origin(lan).await;
    let (proxy, seen) = standin_proxy(OK_BODY).await;
    let plan = plan_with(
        vec![global(
            route_via("images", proxy),
            &[TrafficClass::RemoteContent, TrafficClass::Webhooks],
        )],
        siem_allow(lan),
    );
    // `siem` is not listed: it goes direct and reaches the allowlisted origin. Had
    // it taken the route, the tunnel's strict check would have refused the private
    // address and the origin would show no hit.
    let r = plan::send_with(
        &plan,
        url(&format!("http://{origin}/ingest")),
        &Request::post(TrafficClass::Siem, "application/json", b"{}"),
    )
    .await;
    assert_eq!(r.outcome.as_deref(), Ok(&b"direct"[..]), "{r:?}");
    assert!(!r.traversed_proxy);
    assert_eq!(hits.load(Ordering::SeqCst), 1);
    assert!(
        seen.lock().unwrap().connects.is_empty(),
        "the route carried an unlisted class"
    );
}

// ── the declared media type ──────────────────────────────────────────────────────

#[tokio::test]
async fn a_body_reports_the_media_type_its_origin_declared() {
    let (typed, _) = standin_proxy(
        "HTTP/1.1 200 OK\r\nContent-Type: Image/PNG; charset=binary\r\nContent-Length: 7\r\n\r\nproxied",
    )
    .await;
    let plan = plan_with(
        vec![global(
            route_via("t", typed),
            &[TrafficClass::RemoteContent],
        )],
        InternalAllowList::empty(),
    );
    let got = plan::send_with(
        &plan,
        url("http://1.2.3.4/a.png"),
        &Request::get(TrafficClass::RemoteContent),
    )
    .await;
    assert_eq!(got.content_type.as_deref(), Some("image/png"), "{got:?}");

    let (untyped, _) = standin_proxy(OK_BODY).await;
    let plan = plan_with(
        vec![global(
            route_via("u", untyped),
            &[TrafficClass::RemoteContent],
        )],
        InternalAllowList::empty(),
    );
    let got = plan::send_with(
        &plan,
        url("http://1.2.3.4/a.png"),
        &Request::get(TrafficClass::RemoteContent),
    )
    .await;
    assert!(got.outcome.is_ok());
    assert_eq!(got.content_type, None, "no header, no type");

    // A refusal carries no type.
    let got = plan::send_with(
        &plan,
        url("http://127.0.0.1:9/a.png"),
        &Request::get(TrafficClass::Pki),
    )
    .await;
    assert_eq!(got.content_type, None);
}

#[tokio::test]
async fn the_direct_arm_reports_the_media_type_too() {
    let Some(lan) = lan_address() else {
        eprintln!("SKIPPED: this machine has no RFC1918 address to host a reachable origin on");
        return;
    };
    let listener = TcpListener::bind(SocketAddr::new(lan, 0)).await.unwrap();
    let origin = listener.local_addr().unwrap();
    tokio::spawn(async move {
        while let Ok((mut s, _)) = listener.accept().await {
            tokio::spawn(async move {
                let _ = read_request(&mut s).await;
                let _ = s
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: image/gif\r\nContent-Length: 2\r\n\r\nok")
                    .await;
            });
        }
    });
    let got = plan::send_with(
        &plan_with(Vec::new(), siem_allow(lan)),
        url(&format!("http://{origin}/x")),
        &Request::get(TrafficClass::Siem),
    )
    .await;
    assert_eq!(got.outcome.as_deref(), Ok(&b"ok"[..]), "{got:?}");
    assert_eq!(got.content_type.as_deref(), Some("image/gif"));
}
