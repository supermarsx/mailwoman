//! The t32 e31b route kinds on the wire: TLS to the proxy (`https-connect`) and
//! remote DNS (`socks5h`).
//!
//! In `src/` for the same reason as `../tests.rs`, whose child module this is: the stand-in proxies can only be
//! bound on loopback, and the TLS-to-proxy tests need a [`Dialer`] that trusts the
//! stand-in's self-signed certificate, which only this crate can build. Origins are
//! `http://1.2.3.4/…` or a name; the stand-ins answer the tunnelled request
//! themselves, so no packet leaves the machine.

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use super::super::*;
use crate::{Outgoing, PROXY_UA, Refusal};

const USER: &str = "svc-mail";
const PASSWORD: &str = "correct-horse-battery-staple";
/// `base64("svc-mail:correct-horse-battery-staple")`, as `Proxy-Authorization`
/// carries it.
const BASIC: &str = "c3ZjLW1haWw6Y29ycmVjdC1ob3JzZS1iYXR0ZXJ5LXN0YXBsZQ==";
const REPLY: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 7\r\n\r\nthrough";

fn get() -> Outgoing<'static> {
    Outgoing {
        authorized_dav: None,
        accept: "*/*",
        user_agent: PROXY_UA,
        body: None,
        response_limit: crate::MAX_IMAGE_BYTES,
    }
}

fn route(scheme: ProxyScheme, at: SocketAddr, auth: bool) -> ProxyRoute {
    ProxyRoute {
        id: "kind-test".into(),
        scheme,
        host: at.ip().to_string(),
        port: at.port(),
        auth: auth.then(|| ProxyAuth {
            username: USER.into(),
            password: PASSWORD.into(),
        }),
        // Plaintext origins: the stand-ins answer the tunnelled request themselves
        // and cannot present a publicly trusted origin certificate.
        allow_plaintext: true,
    }
}

/// Read an HTTP head through `\r\n\r\n` from any stream.
async fn read_head<S: tokio::io::AsyncRead + Unpin>(s: &mut S) -> String {
    let mut head = Vec::new();
    let mut byte = [0u8; 1];
    while s.read(&mut byte).await.unwrap_or(0) == 1 {
        head.push(byte[0]);
        if head.ends_with(b"\r\n\r\n") || head.len() > 8192 {
            break;
        }
    }
    String::from_utf8_lossy(&head).into_owned()
}

// ── TLS to the proxy ─────────────────────────────────────────────────────────────

/// A self-signed certificate for `127.0.0.1` and the root store that trusts it.
fn loopback_cert() -> (
    rustls_pki_types::CertificateDer<'static>,
    rustls_pki_types::PrivateKeyDer<'static>,
    rustls::RootCertStore,
) {
    let issued = rcgen::generate_simple_self_signed(vec!["127.0.0.1".to_string()]).unwrap();
    let cert = issued.cert.der().clone();
    let key =
        rustls_pki_types::PrivateKeyDer::try_from(issued.signing_key.serialize_der()).unwrap();
    let mut roots = rustls::RootCertStore::empty();
    roots.add(cert.clone()).unwrap();
    (cert, key, roots)
}

/// A `CONNECT` proxy that speaks TLS on its listening socket, records each
/// **decrypted** `CONNECT` head, and answers the tunnelled request itself.
async fn tls_connect_proxy() -> (SocketAddr, rustls::RootCertStore, Arc<Mutex<Vec<String>>>) {
    let (cert, key, roots) = loopback_cert();
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)
        .unwrap();
    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(config));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let log = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&log);
    tokio::spawn(async move {
        while let Ok((tcp, _)) = listener.accept().await {
            let acceptor = acceptor.clone();
            let sink = Arc::clone(&sink);
            tokio::spawn(async move {
                let Ok(mut tls) = acceptor.accept(tcp).await else {
                    return;
                };
                let head = read_head(&mut tls).await;
                sink.lock().unwrap().push(head);
                let _ = tls
                    .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                    .await;
                let _ = read_head(&mut tls).await;
                let _ = tls.write_all(REPLY).await;
                let _ = tls.flush().await;
            });
        }
    });
    (addr, roots, log)
}

/// A plain `CONNECT` proxy that records what it receives and answers itself. Like
/// a real HTTP proxy, it answers `400` to bytes that are not an HTTP request (a TLS
/// ClientHello, for one) and closes.
async fn plain_connect_proxy() -> (SocketAddr, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let log = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&log);
    tokio::spawn(async move {
        while let Ok((mut tcp, _)) = listener.accept().await {
            let sink = Arc::clone(&sink);
            tokio::spawn(async move {
                let mut first = [0u8; 4096];
                let n = tcp.read(&mut first).await.unwrap_or(0);
                let mut head = String::from_utf8_lossy(&first[..n]).into_owned();
                if !first[..n].starts_with(b"CONNECT ") {
                    sink.lock().unwrap().push(head);
                    let _ = tcp.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n").await;
                    return;
                }
                if !head.ends_with("\r\n\r\n") {
                    head.push_str(&read_head(&mut tcp).await);
                }
                sink.lock().unwrap().push(head);
                let _ = tcp
                    .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                    .await;
                let _ = read_head(&mut tcp).await;
                let _ = tcp.write_all(REPLY).await;
            });
        }
    });
    (addr, log)
}

/// A TCP relay in front of `upstream` that records every byte the client sends:
/// what a passive observer between Mailwoman and the proxy would see.
async fn sniffing_relay(upstream: SocketAddr) -> (SocketAddr, Arc<Mutex<Vec<u8>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let wire = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&wire);
    tokio::spawn(async move {
        while let Ok((client, _)) = listener.accept().await {
            let sink = Arc::clone(&sink);
            tokio::spawn(async move {
                let Ok(server) = TcpStream::connect(upstream).await else {
                    return;
                };
                let (mut cr, mut cw) = client.into_split();
                let (mut sr, mut sw) = server.into_split();
                let up = async {
                    let mut buf = [0u8; 4096];
                    loop {
                        let n = match cr.read(&mut buf).await {
                            Ok(0) | Err(_) => break,
                            Ok(n) => n,
                        };
                        sink.lock().unwrap().extend_from_slice(&buf[..n]);
                        if sw.write_all(&buf[..n]).await.is_err() {
                            break;
                        }
                    }
                };
                let down = async {
                    let _ = tokio::io::copy(&mut sr, &mut cw).await;
                };
                tokio::join!(up, down);
            });
        }
    });
    (addr, wire)
}

fn contains(hay: &[u8], needle: &str) -> bool {
    hay.windows(needle.len()).any(|w| w == needle.as_bytes())
}

fn trusting(roots: rustls::RootCertStore) -> Dialer {
    Dialer {
        proxy_tls: stream::connector_trusting(roots),
    }
}

#[tokio::test]
async fn https_connect_sends_the_credential_only_inside_tls_to_the_proxy() {
    let (proxy, roots, decrypted) = tls_connect_proxy().await;
    let (relay, wire) = sniffing_relay(proxy).await;
    let r = route(ProxyScheme::HttpsConnect, relay, true);

    let fetched = send_via_proxy_with(
        &trusting(roots),
        reqwest::Url::parse("http://1.2.3.4/img").unwrap(),
        &r,
        &get(),
    )
    .await;
    assert_eq!(
        fetched.outcome.as_deref(),
        Ok(&b"through"[..]),
        "{fetched:?}"
    );
    assert!(fetched.traversed_proxy);

    // The proxy, inside TLS, received the CONNECT and the credential.
    let heads = decrypted.lock().unwrap().clone();
    assert_eq!(heads.len(), 1, "{heads:?}");
    assert!(heads[0].starts_with("CONNECT 1.2.3.4:80 "), "{heads:?}");
    assert!(heads[0].contains(BASIC), "{heads:?}");

    // On the wire between us and the proxy, none of it is readable.
    let wire = wire.lock().unwrap().clone();
    assert!(!wire.is_empty(), "the relay saw the connection");
    for secret in ["CONNECT", "Proxy-Authorization", BASIC, PASSWORD, USER] {
        assert!(!contains(&wire, secret), "{secret:?} was sent in the clear");
    }
}

#[tokio::test]
async fn the_sniffer_does_see_the_credential_on_a_plain_connect_route() {
    // Negative control for the test above: the same relay in front of a plain
    // `http-connect` proxy sees the credential, so "not seen" above is the TLS.
    let (proxy, _) = plain_connect_proxy().await;
    let (relay, wire) = sniffing_relay(proxy).await;
    let r = route(ProxyScheme::HttpConnect, relay, true);
    let fetched = send_via_proxy_with(
        &Dialer::production(),
        reqwest::Url::parse("http://1.2.3.4/img").unwrap(),
        &r,
        &get(),
    )
    .await;
    assert!(fetched.outcome.is_ok(), "{fetched:?}");
    assert!(contains(&wire.lock().unwrap(), BASIC));
}

#[tokio::test]
async fn https_connect_to_a_proxy_that_does_not_speak_tls_sends_no_connect() {
    let (proxy, received) = plain_connect_proxy().await;
    let r = route(ProxyScheme::HttpsConnect, proxy, true);
    let fetched = send_via_proxy_with(
        &Dialer::production(),
        reqwest::Url::parse("http://1.2.3.4/img").unwrap(),
        &r,
        &get(),
    )
    .await;
    assert!(
        matches!(fetched.outcome, Err(ProxyRefusal::ProxyTls(_))),
        "{fetched:?}"
    );
    assert!(!fetched.traversed_proxy);
    for head in received.lock().unwrap().iter() {
        assert!(
            !head.contains("CONNECT") && !head.contains(BASIC),
            "a CONNECT reached a proxy over plain TCP: {head:?}"
        );
    }
}

#[tokio::test]
async fn https_connect_refuses_a_proxy_certificate_the_roots_do_not_trust() {
    // Production roots: the stand-in's self-signed certificate is not trusted.
    let (proxy, _roots, decrypted) = tls_connect_proxy().await;
    let r = route(ProxyScheme::HttpsConnect, proxy, true);
    let fetched = send_via_proxy_with(
        &Dialer::production(),
        reqwest::Url::parse("http://1.2.3.4/img").unwrap(),
        &r,
        &get(),
    )
    .await;
    match fetched.outcome {
        Err(ProxyRefusal::ProxyTls(reason)) => {
            assert!(reason.contains("certificate"), "{reason}")
        }
        other => panic!("an untrusted proxy certificate must be refused: {other:?}"),
    }
    assert!(
        decrypted.lock().unwrap().is_empty(),
        "nothing may be sent before the proxy is verified"
    );
}

#[tokio::test]
async fn https_connect_verifies_the_proxy_for_the_routes_host() {
    // The certificate is for 127.0.0.1; a route naming the proxy `localhost`
    // reaches the same socket but asks for a name the certificate does not carry.
    let (proxy, roots, decrypted) = tls_connect_proxy().await;
    let mut r = route(ProxyScheme::HttpsConnect, proxy, true);
    r.host = "localhost".into();
    let fetched = send_via_proxy_with(
        &trusting(roots),
        reqwest::Url::parse("http://1.2.3.4/img").unwrap(),
        &r,
        &get(),
    )
    .await;
    // `localhost` may resolve to ::1 first on this host, where nothing listens;
    // either way nothing may reach the proxy unverified.
    assert!(
        matches!(
            fetched.outcome,
            Err(ProxyRefusal::ProxyTls(_)) | Err(ProxyRefusal::ProxyUnreachable(_))
        ),
        "{fetched:?}"
    );
    assert!(decrypted.lock().unwrap().is_empty());
}

// ── remote DNS ───────────────────────────────────────────────────────────────────

/// A SOCKS5 server that records `(ATYP, destination, port)` of each request,
/// accepts it, and answers the tunnelled request with `reply` itself.
async fn recording_socks5h(
    reply: &'static [u8],
) -> (SocketAddr, Arc<Mutex<Vec<(u8, String, u16)>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let log = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&log);
    tokio::spawn(async move {
        while let Ok((mut c, _)) = listener.accept().await {
            let sink = Arc::clone(&sink);
            tokio::spawn(async move {
                let mut g = [0u8; 2];
                if c.read_exact(&mut g).await.is_err() {
                    return;
                }
                let mut m = vec![0u8; g[1] as usize];
                let _ = c.read_exact(&mut m).await;
                let _ = c.write_all(&[0x05, 0x00]).await;
                let mut req = [0u8; 4];
                if c.read_exact(&mut req).await.is_err() {
                    return;
                }
                let dest = match req[3] {
                    0x01 => {
                        let mut o = [0u8; 4];
                        let _ = c.read_exact(&mut o).await;
                        std::net::Ipv4Addr::from(o).to_string()
                    }
                    0x04 => {
                        let mut o = [0u8; 16];
                        let _ = c.read_exact(&mut o).await;
                        std::net::Ipv6Addr::from(o).to_string()
                    }
                    0x03 => {
                        let mut n = [0u8; 1];
                        let _ = c.read_exact(&mut n).await;
                        let mut name = vec![0u8; n[0] as usize];
                        let _ = c.read_exact(&mut name).await;
                        String::from_utf8_lossy(&name).into_owned()
                    }
                    _ => return,
                };
                let mut port = [0u8; 2];
                let _ = c.read_exact(&mut port).await;
                sink.lock()
                    .unwrap()
                    .push((req[3], dest, u16::from_be_bytes(port)));
                let _ = c
                    .write_all(&[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
                    .await;
                let _ = read_head(&mut c).await;
                let _ = c.write_all(reply).await;
            });
        }
    });
    (addr, log)
}

async fn via_socks5h(proxy: SocketAddr, url: &str) -> ProxyFetch {
    send_via_proxy_with(
        &Dialer::production(),
        reqwest::Url::parse(url).unwrap(),
        &route(ProxyScheme::Socks5h, proxy, false),
        &get(),
    )
    .await
}

#[tokio::test]
async fn socks5h_hands_the_proxy_a_checked_name() {
    let (proxy, seen) = recording_socks5h(REPLY).await;
    let fetched = via_socks5h(proxy, "http://cdn.example.com/p.png").await;
    assert_eq!(
        fetched.outcome.as_deref(),
        Ok(&b"through"[..]),
        "{fetched:?}"
    );
    assert_eq!(
        seen.lock().unwrap().as_slice(),
        &[(0x03, "cdn.example.com".to_string(), 80)]
    );
}

#[tokio::test]
async fn socks5h_still_sends_a_literal_as_an_address() {
    let (proxy, seen) = recording_socks5h(REPLY).await;
    let fetched = via_socks5h(proxy, "http://1.2.3.4:8080/p.png").await;
    assert!(fetched.outcome.is_ok(), "{fetched:?}");
    assert_eq!(
        seen.lock().unwrap().as_slice(),
        &[(0x01, "1.2.3.4".to_string(), 8080)]
    );
}

#[tokio::test]
async fn socks5h_refuses_denylisted_names_and_private_literals_without_dialling() {
    let (proxy, seen) = recording_socks5h(REPLY).await;
    for u in [
        "http://db.internal/",
        "http://metadata.google.internal/computeMetadata/v1/",
        "http://printer.local/",
        "http://intranet/",
        "http://127.0.0.1/",
        "http://10.0.0.5/",
        "http://169.254.169.254/latest/meta-data/",
        "http://[::1]/",
    ] {
        let fetched = via_socks5h(proxy, u).await;
        assert_eq!(
            fetched.outcome.unwrap_err(),
            ProxyRefusal::Origin(Refusal::Blocked),
            "{u}"
        );
        assert!(!fetched.traversed_proxy, "{u}");
    }
    assert!(
        seen.lock().unwrap().is_empty(),
        "a refused destination must never reach the proxy"
    );
}

#[tokio::test]
async fn socks5h_rechecks_every_redirect() {
    let (proxy, seen) = recording_socks5h(
        b"HTTP/1.1 302 Found\r\nLocation: http://metadata.google.internal/\r\nContent-Length: 0\r\n\r\n",
    )
    .await;
    let fetched = via_socks5h(proxy, "http://cdn.example.com/start").await;
    assert_eq!(
        fetched.outcome.unwrap_err(),
        ProxyRefusal::Origin(Refusal::Blocked)
    );
    assert!(fetched.traversed_proxy, "the first hop went through");
    assert_eq!(
        seen.lock().unwrap().len(),
        1,
        "the redirect target was refused before a second request"
    );
}

#[tokio::test]
async fn a_remote_dns_route_cannot_be_driven_with_a_locally_resolved_target() {
    let (proxy, seen) = recording_socks5h(REPLY).await;
    let target = Target {
        url: reqwest::Url::parse("http://1.2.3.4/x").unwrap(),
        host: "1.2.3.4".into(),
        addr: "1.2.3.4:80".parse().unwrap(),
    };
    let hop = tunnel_send_hop(&target, &route(ProxyScheme::Socks5h, proxy, false), &get()).await;
    assert!(
        matches!(hop.outcome, Err(ProxyRefusal::RouteInvalid(_))),
        "{hop:?}"
    );
    assert!(seen.lock().unwrap().is_empty());
}

#[test]
fn the_new_kinds_have_their_own_tokens() {
    assert_eq!(ProxyScheme::HttpsConnect.as_str(), "https-connect");
    assert_eq!(ProxyScheme::Socks5h.as_str(), "socks5h");
    assert_eq!(ProxyScheme::Socks5.as_str(), "socks5");
    assert_eq!(
        Refusal::from(ProxyRefusal::ProxyTls("x".into())),
        Refusal::Upstream
    );
}
