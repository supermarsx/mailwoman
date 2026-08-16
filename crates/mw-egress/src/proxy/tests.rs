//! Wire-level tests for the upstream-proxy transport.
//!
//! # Why these live in `src/` and not in `tests/proxy_transport.rs`
//! Every assertion here needs a **real** proxy and a **real** origin, and both can
//! only be bound on loopback — which [`crate::ip_allowed`] correctly refuses. They
//! therefore drive the private, ungated [`super::hop_over_tunnel`] directly, in the
//! same spirit as (and with the same justification as) the loopback fetch-mechanics
//! tests in `crate::tests`. The **public** entry point [`super::tunnel_fetch_hop`]
//! does apply the address check, and `tests/proxy_transport.rs` asserts that from
//! outside the crate — including that a blocked target never opens a socket.
//!
//! Keeping the ungated function private is the point: there is no `_unchecked`
//! public function for a caller to reach for by accident.
//!
//! # What the recording servers buy
//! The claims are asserted against **the bytes a third party actually received**,
//! not against our own encoder. An encoder unit test alone would not notice a
//! transport that encoded correctly and then sent something else.

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use super::*;
use crate::{Hop, Refusal, Target};

// ── helpers ────────────────────────────────────────────────────────────────────

/// Drive the ungated hop and report `(traversed_proxy, outcome)`.
async fn hop(
    target: &Target,
    route: &ProxyRoute,
    accept: &str,
) -> (bool, Result<Hop, ProxyRefusal>) {
    let traversed = Arc::new(AtomicBool::new(false));
    let outcome = super::hop_over_tunnel(target, route, accept, &traversed).await;
    (traversed.load(Ordering::SeqCst), outcome)
}

fn http_route(scheme: ProxyScheme, proxy: SocketAddr) -> ProxyRoute {
    ProxyRoute {
        id: "test-route".into(),
        scheme,
        host: proxy.ip().to_string(),
        port: proxy.port(),
        auth: None,
        // The wire-level tests use plaintext origins because a loopback origin
        // cannot present a certificate that chains to the Mozilla root set. The TLS
        // assertions below are refusals, which need no trusted origin.
        allow_plaintext: true,
    }
}

/// A target whose hostname CANNOT resolve in DNS anywhere (RFC 6761 reserves
/// `.invalid`), pinned to a loopback origin. If the transport ever handed the name
/// to the proxy, the proxy's own `connect` would fail and no body could arrive.
fn pinned_invalid_target(origin: SocketAddr, path: &str) -> Target {
    Target {
        url: reqwest::Url::parse(&format!("http://pinned.invalid{path}")).unwrap(),
        host: "pinned.invalid".into(),
        addr: origin,
    }
}

/// A plaintext HTTP origin serving `body` at `/img`, and a 302 at `/redirect`.
async fn spawn_plain_origin(body: Vec<u8>, redirect_to: Option<String>) -> SocketAddr {
    use axum::response::Response;
    use axum::routing::get;
    use axum::{Router, http::StatusCode};

    let img = move || async move { Response::new(axum::body::Body::from(body)) };
    let redirect = move || async move {
        let mut resp = Response::new(axum::body::Body::empty());
        *resp.status_mut() = StatusCode::FOUND;
        if let Some(loc) = redirect_to {
            resp.headers_mut()
                .insert(axum::http::header::LOCATION, loc.parse().unwrap());
        }
        resp
    };
    let app: Router = Router::new()
        .route("/img", get(img))
        .route("/redirect", get(redirect));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    addr
}

/// Read an HTTP head (through `\r\n\r\n`) one byte at a time, leaving anything that
/// follows on the stream.
async fn read_head(stream: &mut TcpStream) -> Vec<u8> {
    let mut head = Vec::new();
    let mut byte = [0u8; 1];
    while stream.read(&mut byte).await.unwrap_or(0) == 1 {
        head.push(byte[0]);
        if head.ends_with(b"\r\n\r\n") || head.len() > 8192 {
            break;
        }
    }
    head
}

/// A real HTTP `CONNECT` proxy that records **every `CONNECT` head it receives**,
/// then opens the requested socket and pipes bytes both ways.
async fn recording_connect_proxy() -> (SocketAddr, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&log);
    tokio::spawn(async move {
        while let Ok((mut client, _)) = listener.accept().await {
            let sink = Arc::clone(&sink);
            tokio::spawn(async move {
                let head = read_head(&mut client).await;
                let head = String::from_utf8_lossy(&head).to_string();
                sink.lock().unwrap().push(head.clone());
                // Parse the authority out of `CONNECT <authority> HTTP/1.1`.
                let authority = head
                    .lines()
                    .next()
                    .and_then(|l| l.split_whitespace().nth(1))
                    .unwrap_or_default()
                    .to_string();
                // Connect to whatever we were asked for. A NAME here would fail to
                // resolve (`pinned.invalid`) and the fetch would produce no body —
                // which is the second half of the "never asked to resolve" proof.
                match TcpStream::connect(&authority).await {
                    Ok(mut upstream) => {
                        let _ = client
                            .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                            .await;
                        let _ = tokio::io::copy_bidirectional(&mut client, &mut upstream).await;
                    }
                    Err(_) => {
                        let _ = client.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await;
                    }
                }
            });
        }
    });
    (addr, log)
}

/// What a SOCKS5 request carried, as the **server** saw it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Socks5Record {
    atyp: u8,
    /// The destination as the server decoded it: an IP literal for 0x01/0x04, or
    /// the raw name for 0x03.
    destination: String,
    port: u16,
}

/// A real SOCKS5 server that records the `ATYP` and destination of every request,
/// then (for IPv4) connects and pipes. `ATYP=0x03` is recorded and refused, which
/// is what makes the negative control below meaningful.
async fn recording_socks5_proxy() -> (SocketAddr, Arc<Mutex<Vec<Socks5Record>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let log: Arc<Mutex<Vec<Socks5Record>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&log);
    tokio::spawn(async move {
        while let Ok((mut client, _)) = listener.accept().await {
            let sink = Arc::clone(&sink);
            tokio::spawn(async move {
                // Greeting.
                let mut head = [0u8; 2];
                if client.read_exact(&mut head).await.is_err() {
                    return;
                }
                let mut methods = vec![0u8; head[1] as usize];
                if client.read_exact(&mut methods).await.is_err() {
                    return;
                }
                if client.write_all(&[0x05, 0x00]).await.is_err() {
                    return;
                }
                // Request.
                let mut req = [0u8; 4];
                if client.read_exact(&mut req).await.is_err() {
                    return;
                }
                let atyp = req[3];
                let destination = match atyp {
                    0x01 => {
                        let mut o = [0u8; 4];
                        let _ = client.read_exact(&mut o).await;
                        std::net::Ipv4Addr::from(o).to_string()
                    }
                    0x04 => {
                        let mut o = [0u8; 16];
                        let _ = client.read_exact(&mut o).await;
                        std::net::Ipv6Addr::from(o).to_string()
                    }
                    0x03 => {
                        let mut n = [0u8; 1];
                        let _ = client.read_exact(&mut n).await;
                        let mut name = vec![0u8; n[0] as usize];
                        let _ = client.read_exact(&mut name).await;
                        String::from_utf8_lossy(&name).to_string()
                    }
                    _ => String::new(),
                };
                let mut port = [0u8; 2];
                let _ = client.read_exact(&mut port).await;
                let port = u16::from_be_bytes(port);
                sink.lock().unwrap().push(Socks5Record {
                    atyp,
                    destination: destination.clone(),
                    port,
                });

                if atyp != 0x01 {
                    // Anything but IPv4 (including a domain) is refused here — the
                    // recorder's job for those is to prove it SAW them.
                    let _ = client
                        .write_all(&[0x05, 0x08, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
                        .await;
                    return;
                }
                let dst = format!("{destination}:{port}");
                match TcpStream::connect(&dst).await {
                    Ok(mut upstream) => {
                        let _ = client
                            .write_all(&[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
                            .await;
                        let _ = tokio::io::copy_bidirectional(&mut client, &mut upstream).await;
                    }
                    Err(_) => {
                        let _ = client
                            .write_all(&[0x05, 0x05, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
                            .await;
                    }
                }
            });
        }
    });
    (addr, log)
}

/// A hostile `CONNECT` proxy: it answers `200` and then, instead of piping to the
/// requested address, terminates TLS itself with a certificate for its own name.
async fn mitm_connect_proxy(its_own_name: &'static str) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let issued = rcgen::generate_simple_self_signed(vec![its_own_name.to_string()]).unwrap();
    let cert = issued.cert.der().clone();
    let key =
        rustls_pki_types::PrivateKeyDer::try_from(issued.signing_key.serialize_der()).unwrap();
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let server_config = rustls::ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)
        .unwrap();
    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(server_config));

    tokio::spawn(async move {
        while let Ok((mut client, _)) = listener.accept().await {
            let acceptor = acceptor.clone();
            tokio::spawn(async move {
                let _ = read_head(&mut client).await;
                if client
                    .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                    .await
                    .is_err()
                {
                    return;
                }
                // Substitute ourselves for the origin. Our client must refuse.
                let _ = acceptor.accept(client).await;
            });
        }
    });
    addr
}

/// A `CONNECT` proxy that answers `200` and then speaks **plaintext HTTP** at a
/// client that is expecting TLS (plan §10.3's "200 to CONNECT then plaintext").
async fn plaintext_after_connect_proxy() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        while let Ok((mut client, _)) = listener.accept().await {
            tokio::spawn(async move {
                let _ = read_head(&mut client).await;
                let _ = client
                    .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                    .await;
                let _ = client
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nowned")
                    .await;
            });
        }
    });
    addr
}

// ── the proxy is never asked to resolve a name ─────────────────────────────────

#[tokio::test]
async fn connect_authority_is_an_ip_literal_and_the_hostname_never_reaches_the_proxy() {
    let origin = spawn_plain_origin(b"from-the-pinned-origin".to_vec(), None).await;
    let (proxy, log) = recording_connect_proxy().await;
    let route = http_route(ProxyScheme::HttpConnect, proxy);
    let target = pinned_invalid_target(origin, "/img");

    let (traversed, outcome) = hop(&target, &route, "image/*").await;

    // 1. The body arrived. `pinned.invalid` cannot resolve in DNS anywhere, and the
    //    recording proxy connects to exactly what it was given — so a body can only
    //    exist if the proxy was handed the literal address WE resolved.
    assert!(traversed, "the tunnel must have been established");
    match outcome {
        Ok(Hop::Body(b)) => assert_eq!(b, b"from-the-pinned-origin"),
        other => panic!("expected a body through the tunnel, got {other:?}"),
    }

    // 2. And directly: the authority the proxy received is the IP literal.
    let heads = log.lock().unwrap().clone();
    assert_eq!(heads.len(), 1, "exactly one CONNECT");
    let head = &heads[0];
    let request_line = head.lines().next().unwrap();
    assert_eq!(
        request_line,
        format!("CONNECT {origin} HTTP/1.1"),
        "the CONNECT authority must be the resolved IP literal"
    );
    assert!(
        !head.contains("pinned.invalid"),
        "the origin hostname must not appear anywhere in the CONNECT head: {head}"
    );
}

#[tokio::test]
async fn socks5_request_carries_atyp_ipv4_and_never_a_domain() {
    let origin = spawn_plain_origin(b"socks-body".to_vec(), None).await;
    let (proxy, log) = recording_socks5_proxy().await;
    let route = http_route(ProxyScheme::Socks5, proxy);
    let target = pinned_invalid_target(origin, "/img");

    let (traversed, outcome) = hop(&target, &route, "image/*").await;
    assert!(traversed);
    match outcome {
        Ok(Hop::Body(b)) => assert_eq!(b, b"socks-body"),
        other => panic!("expected a body through the SOCKS5 tunnel, got {other:?}"),
    }

    let records = log.lock().unwrap().clone();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].atyp, socks5::ATYP_IPV4);
    assert_eq!(records[0].destination, origin.ip().to_string());
    assert_eq!(records[0].port, origin.port());
    assert_ne!(records[0].destination, "pinned.invalid");
}

#[tokio::test]
async fn the_socks5_recorder_would_have_caught_a_domain_atyp() {
    // NEGATIVE CONTROL. The assertion above ("the server saw ATYP=0x01") is only
    // evidence if the same server would have recorded a 0x03 had one been sent.
    // Here the test itself hand-writes the domain form the transport cannot build,
    // and the recorder reports it — so the previous test is measuring something.
    let (proxy, log) = recording_socks5_proxy().await;
    let mut client = TcpStream::connect(proxy).await.unwrap();
    client.write_all(&[0x05, 0x01, 0x00]).await.unwrap();
    let mut sel = [0u8; 2];
    client.read_exact(&mut sel).await.unwrap();

    let name = b"pinned.invalid";
    let mut request = vec![0x05, 0x01, 0x00, 0x03, name.len() as u8];
    request.extend_from_slice(name);
    request.extend_from_slice(&80u16.to_be_bytes());
    client.write_all(&request).await.unwrap();
    let mut reply = [0u8; 10];
    let _ = client.read_exact(&mut reply).await;

    let records = log.lock().unwrap().clone();
    assert_eq!(records.len(), 1);
    assert_eq!(
        records[0].atyp, 0x03,
        "the recorder must be able to see a domain ATYP — otherwise the positive \
         assertion is vacuous"
    );
    assert_eq!(records[0].destination, "pinned.invalid");
}

// ── origin TLS is ours through the tunnel ──────────────────────────────────────

#[tokio::test]
async fn a_mitm_proxy_presenting_its_own_certificate_is_refused() {
    let proxy = mitm_connect_proxy("mitm.example").await;
    let mut route = http_route(ProxyScheme::HttpConnect, proxy);
    route.allow_plaintext = false;
    let target = Target {
        url: reqwest::Url::parse("https://cdn.example/x.png").unwrap(),
        host: "cdn.example".into(),
        // Never dialled: the MITM proxy answers 200 and substitutes itself.
        addr: "203.0.113.7:443".parse().unwrap(),
    };

    let (traversed, outcome) = hop(&target, &route, "image/*").await;
    assert!(traversed, "the proxy did accept the tunnel");
    let err = outcome.expect_err("a substituted peer must not produce a body");
    match err {
        ProxyRefusal::OriginTls(reason) => {
            // Fails as a CERTIFICATE problem, not as a generic transport error. If
            // anyone ever adds a permissive verifier this flips to `Ok`.
            let lower = reason.to_ascii_lowercase();
            assert!(
                lower.contains("certificate") || lower.contains("not valid for name"),
                "expected a certificate failure, got: {reason}"
            );
        }
        other => panic!("expected OriginTls, got {other:?}"),
    }
}

#[tokio::test]
async fn a_proxy_that_answers_200_then_speaks_plaintext_is_refused() {
    let proxy = plaintext_after_connect_proxy().await;
    let mut route = http_route(ProxyScheme::HttpConnect, proxy);
    route.allow_plaintext = false;
    let target = Target {
        url: reqwest::Url::parse("https://cdn.example/x.png").unwrap(),
        host: "cdn.example".into(),
        addr: "203.0.113.7:443".parse().unwrap(),
    };

    let (traversed, outcome) = hop(&target, &route, "image/*").await;
    assert!(traversed);
    let err = outcome.expect_err("cleartext must not be accepted where TLS is expected");
    assert!(
        matches!(err, ProxyRefusal::OriginTls(_)),
        "expected a TLS failure, got {err:?}"
    );
}

// ── redirects re-enter the gate ────────────────────────────────────────────────

#[tokio::test]
async fn a_redirect_through_the_tunnel_is_surfaced_not_followed() {
    let origin = spawn_plain_origin(
        Vec::new(),
        Some("http://169.254.169.254/latest/meta-data/".into()),
    )
    .await;
    let (proxy, _log) = recording_connect_proxy().await;
    let route = http_route(ProxyScheme::HttpConnect, proxy);
    let target = pinned_invalid_target(origin, "/redirect");

    let (_, outcome) = hop(&target, &route, "image/*").await;
    match outcome {
        Ok(Hop::Redirect(loc)) => assert_eq!(loc, "http://169.254.169.254/latest/meta-data/"),
        other => panic!("a 3xx must be surfaced for re-validation, got {other:?}"),
    }
    // The transport did NOT follow it: nothing here dialled the metadata address.
    // `fetch_via_proxy`'s loop is what re-validates the Location, and
    // `tests/proxy_transport.rs` asserts that it refuses this one.
}

// ── failure carries the truth about whether the proxy was used ─────────────────

#[tokio::test]
async fn an_unreachable_proxy_reports_that_nothing_traversed_it() {
    // A port nothing is listening on. Bind then drop, so the port is almost
    // certainly free and the connect is refused rather than filtered.
    let dead = {
        let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
        l.local_addr().unwrap()
    };
    let route = http_route(ProxyScheme::HttpConnect, dead);
    let target = pinned_invalid_target("127.0.0.1:9".parse().unwrap(), "/img");

    let (traversed, outcome) = hop(&target, &route, "image/*").await;
    assert!(
        !traversed,
        "nothing traversed a proxy that was never reached — the audit row must be \
         able to say so"
    );
    assert!(
        matches!(outcome, Err(ProxyRefusal::ProxyUnreachable(_))),
        "{outcome:?}"
    );
}

#[tokio::test]
async fn a_proxy_that_refuses_the_tunnel_is_not_reported_as_traversed() {
    // A "proxy" that accepts TCP and then refuses the CONNECT.
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy = listener.local_addr().unwrap();
    tokio::spawn(async move {
        while let Ok((mut client, _)) = listener.accept().await {
            tokio::spawn(async move {
                let _ = read_head(&mut client).await;
                let _ = client
                    .write_all(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n")
                    .await;
            });
        }
    });
    let route = http_route(ProxyScheme::HttpConnect, proxy);
    let target = pinned_invalid_target("127.0.0.1:9".parse().unwrap(), "/img");

    let (traversed, outcome) = hop(&target, &route, "image/*").await;
    assert!(!traversed, "a refused tunnel carried no bytes for us");
    match outcome {
        Err(ProxyRefusal::ProxyRejected(m)) => assert!(m.contains("403"), "{m}"),
        other => panic!("expected ProxyRejected, got {other:?}"),
    }
}

// ── plaintext needs an explicit opt-in ─────────────────────────────────────────

#[tokio::test]
async fn a_plaintext_origin_is_refused_unless_the_route_opts_in() {
    let origin = spawn_plain_origin(b"cleartext".to_vec(), None).await;
    let (proxy, log) = recording_connect_proxy().await;
    let mut route = http_route(ProxyScheme::HttpConnect, proxy);
    route.allow_plaintext = false;
    let target = pinned_invalid_target(origin, "/img");

    let (traversed, outcome) = hop(&target, &route, "image/*").await;
    assert!(!traversed);
    assert_eq!(outcome.unwrap_err(), ProxyRefusal::PlaintextRefused);
    assert!(
        log.lock().unwrap().is_empty(),
        "the refusal must happen before the proxy is dialled"
    );
    // And the opt-in genuinely changes the answer — otherwise this test would pass
    // against a transport that refused everything.
    route.allow_plaintext = true;
    let (traversed, outcome) = hop(&target, &route, "image/*").await;
    assert!(traversed);
    assert!(matches!(outcome, Ok(Hop::Body(_))), "{outcome:?}");
}

// ── credentials reach the proxy and stop there ─────────────────────────────────

#[tokio::test]
async fn proxy_credentials_go_to_the_proxy_and_never_into_the_tunnelled_request() {
    let origin = spawn_plain_origin(b"authed".to_vec(), None).await;
    let (proxy, log) = recording_connect_proxy().await;
    let mut route = http_route(ProxyScheme::HttpConnect, proxy);
    route.auth = Some(ProxyAuth {
        username: "operator".into(),
        password: "s3kr1t-not-in-logs".into(),
    });
    let target = pinned_invalid_target(origin, "/img");

    let (_, outcome) = hop(&target, &route, "image/*").await;
    assert!(matches!(outcome, Ok(Hop::Body(_))), "{outcome:?}");

    let head = log.lock().unwrap()[0].clone();
    assert!(
        head.contains("Proxy-Authorization: Basic "),
        "the credential belongs on the CONNECT head: {head}"
    );
    // `Debug` on the route must not leak it — this is what a `tracing` event, a
    // panic message or an error body would print.
    let rendered = format!("{route:?}");
    assert!(
        !rendered.contains("s3kr1t-not-in-logs"),
        "ProxyRoute Debug leaked the password: {rendered}"
    );
    assert!(rendered.contains("<redacted>"), "{rendered}");
    assert!(
        !route.endpoint().contains("s3kr1t-not-in-logs") && !route.endpoint().contains("operator"),
        "the auditable endpoint must carry no credential: {}",
        route.endpoint()
    );
}

// ── the taxonomy a caller maps to a status code ────────────────────────────────

#[test]
fn refusals_map_onto_the_direct_paths_taxonomy() {
    assert_eq!(
        Refusal::from(ProxyRefusal::Origin(Refusal::Blocked)),
        Refusal::Blocked
    );
    assert_eq!(
        Refusal::from(ProxyRefusal::Origin(Refusal::TooLarge)),
        Refusal::TooLarge
    );
    assert_eq!(
        Refusal::from(ProxyRefusal::ProxyUnreachable("x")),
        Refusal::Upstream
    );
    assert_eq!(
        Refusal::from(ProxyRefusal::OriginTls("bad cert".into())),
        Refusal::Upstream
    );
    assert!(matches!(
        Refusal::from(ProxyRefusal::PlaintextRefused),
        Refusal::BadRequest(_)
    ));
}
