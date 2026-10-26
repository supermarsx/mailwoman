//! Real TCP/TLS/HTTP2 tests for the built-in admission and ingress contract.
use axum::{
    Router,
    body::Bytes,
    extract::{ConnectInfo, WebSocketUpgrade},
    response::{Sse, sse::Event},
    routing::{get, post},
};
use futures_util::{StreamExt, stream};
use mw_server::inbound_http::{HttpLimits, serve};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::oneshot,
};
struct Fixture {
    addr: SocketAddr,
    task: tokio::task::JoinHandle<()>,
    stop: Option<oneshot::Sender<()>>,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.task.abort();
    }
}
async fn fixture(router: Router, limits: HttpLimits) -> Fixture {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (stop, stopped) = oneshot::channel();
    let task = tokio::spawn(async move {
        serve(listener, router, limits, async {
            let _ = stopped.await;
        })
        .await
        .unwrap();
    });
    Fixture {
        addr,
        task,
        stop: Some(stop),
    }
}
fn limits() -> HttpLimits {
    HttpLimits {
        header_timeout: Duration::from_secs(1),
        body_timeout: Duration::from_secs(1),
        ..Default::default()
    }
}
fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap()
}
async fn raw(addr: SocketAddr, request: &[u8]) -> String {
    let mut socket = TcpStream::connect(addr).await.unwrap();
    socket.write_all(request).await.unwrap();
    let mut bytes = Vec::new();
    tokio::time::timeout(Duration::from_secs(4), socket.read_to_end(&mut bytes))
        .await
        .unwrap()
        .unwrap();
    String::from_utf8_lossy(&bytes).into_owned()
}
#[tokio::test]
async fn http1_rejects_oversized_headers_and_slow_ingress_then_remains_healthy() {
    let app = Router::new()
        .route("/", get(|| async { "healthy" }))
        .route("/echo", post(|body: Bytes| async move { body }));
    let server = fixture(app, limits()).await;
    let huge = format!(
        "GET / HTTP/1.1\r\nHost: localhost\r\nX-Large: {}\r\nConnection: close\r\n\r\n",
        "a".repeat(96 * 1024)
    );
    assert!(
        raw(server.addr, huge.as_bytes())
            .await
            .starts_with("HTTP/1.1 431")
    );
    let too_many = format!(
        "GET / HTTP/1.1\r\nHost: localhost\r\n{}Connection: close\r\n\r\n",
        (0..101)
            .map(|i| format!("X-Field-{i}: value\r\n"))
            .collect::<String>()
    );
    assert!(
        raw(server.addr, too_many.as_bytes())
            .await
            .starts_with("HTTP/1.1 431")
    );
    let partial = raw(server.addr, b"GET / HTTP/1.1\r\nHost: local").await;
    assert!(
        partial.is_empty() || partial.starts_with("HTTP/1.1 408"),
        "{partial}"
    );
    let body = raw(
        server.addr,
        b"POST /echo HTTP/1.1\r\nHost: localhost\r\nContent-Length: 10\r\n\r\nx",
    )
    .await;
    assert!(body.starts_with("HTTP/1.1 408"), "{body}");
    assert_eq!(
        client()
            .get(format!("http://{}/", server.addr))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap(),
        "healthy"
    );
}
async fn h2_response<I>(io: I, uri: String) -> hyper::Response<hyper::body::Incoming>
where
    I: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let (mut sender, connection) = hyper::client::conn::http2::handshake(
        hyper_util::rt::TokioExecutor::new(),
        hyper_util::rt::TokioIo::new(io),
    )
    .await
    .unwrap();
    tokio::spawn(async move {
        let _ = connection.await;
    });
    sender
        .send_request(
            hyper::Request::builder()
                .uri(uri)
                .body(http_body_util::Empty::<Bytes>::new())
                .unwrap(),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn plaintext_http2_is_negotiated_and_preserves_connect_info() {
    use http_body_util::BodyExt;
    let app = Router::new().route(
        "/",
        get(|ConnectInfo(peer): ConnectInfo<SocketAddr>| async move { peer.ip().to_string() }),
    );
    let server = fixture(app, limits()).await;
    let response = h2_response(
        TcpStream::connect(server.addr).await.unwrap(),
        format!("http://{}/", server.addr),
    )
    .await;
    assert_eq!(response.version(), hyper::Version::HTTP_2);
    assert_eq!(
        response.into_body().collect().await.unwrap().to_bytes(),
        "127.0.0.1"
    );
}

#[tokio::test]
async fn tls_alpn_negotiates_actual_http2() {
    use http_body_util::BodyExt;
    let fixtures = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tls");
    let (listener, _) = mw_server::TlsListener::bind(
        "127.0.0.1:0",
        &mw_server::TlsConfig::External {
            cert: fixtures.join("cert1.pem"),
            key: fixtures.join("key1.pem"),
        },
    )
    .await
    .unwrap();
    let addr = axum::serve::Listener::local_addr(&listener).unwrap();
    let task = tokio::spawn(serve(
        listener,
        Router::new().route("/", get(|| async { "h2" })),
        limits(),
        std::future::pending(),
    ));
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut config = rustls::ClientConfig::builder_with_provider(provider.clone())
        .with_safe_default_protocol_versions()
        .unwrap()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(TrustAny(provider)))
        .with_no_client_auth();
    config.alpn_protocols = vec![b"h2".to_vec()];
    let connector = tokio_rustls::TlsConnector::from(Arc::new(config));
    let tls = connector
        .connect(
            ServerName::try_from("localhost").unwrap(),
            TcpStream::connect(addr).await.unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(tls.get_ref().1.alpn_protocol(), Some(b"h2".as_slice()));
    let response = h2_response(tls, format!("https://localhost:{}/", addr.port())).await;
    assert_eq!(response.version(), hyper::Version::HTTP_2);
    assert_eq!(
        response.into_body().collect().await.unwrap().to_bytes(),
        "h2"
    );
    task.abort();
}

#[tokio::test]
async fn sse_response_holds_request_capacity_without_an_ingress_idle_timeout() {
    let app = Router::new()
        .route(
            "/events",
            get(|| async {
                Sse::new(
                    stream::once(async {
                        Ok::<_, std::convert::Infallible>(Event::default().data("ready"))
                    })
                    .chain(stream::pending()),
                )
            }),
        )
        .route("/", get(|| async { "healthy" }));
    let server = fixture(
        app,
        HttpLimits {
            requests: 1,
            ..limits()
        },
    )
    .await;
    let client = client();
    let mut events = client
        .get(format!("http://{}/events", server.addr))
        .send()
        .await
        .unwrap();
    assert_eq!(events.status(), 200);
    assert!(events.chunk().await.unwrap().is_some());
    tokio::time::sleep(Duration::from_millis(1200)).await;
    assert!(
        tokio::time::timeout(Duration::from_millis(100), events.chunk())
            .await
            .is_err(),
        "SSE must remain open past request-body timeout"
    );
    let overloaded = client
        .get(format!("http://{}/", server.addr))
        .send()
        .await
        .unwrap();
    assert_eq!(overloaded.status(), 503);
    assert_eq!(overloaded.headers()["retry-after"], "1");
    drop(overloaded);
    drop(events);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    loop {
        let status = client
            .get(format!("http://{}/", server.addr))
            .send()
            .await
            .unwrap()
            .status();
        if status == 200 {
            break;
        }
        assert!(tokio::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}
#[tokio::test]
async fn websocket_io_keeps_its_connection_lease_after_the_101_response() {
    let app = Router::new()
        .route(
            "/ws",
            get(|ws: WebSocketUpgrade| async {
                ws.on_upgrade(|mut socket| async move { while socket.recv().await.is_some() {} })
            }),
        )
        .route("/", get(|| async { "healthy" }));
    let server = fixture(
        app,
        HttpLimits {
            connections: 16,
            ..limits()
        },
    )
    .await;
    let mut sockets = Vec::new();
    for _ in 0..16 {
        let (socket, response) =
            tokio_tungstenite::connect_async(format!("ws://{}/ws", server.addr))
                .await
                .unwrap();
        assert_eq!(response.status(), 101);
        sockets.push(socket);
    }
    let rejected = client()
        .get(format!("http://{}/", server.addr))
        .send()
        .await;
    assert!(
        rejected.is_err(),
        "upgraded sockets must consume all16 connection slots"
    );
    drop(sockets.pop());
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        client()
            .get(format!("http://{}/", server.addr))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    server.task.abort();
    let remaining = sockets.last_mut().unwrap();
    let closed = tokio::time::timeout(Duration::from_secs(2), remaining.next())
        .await
        .unwrap();
    assert!(
        closed.is_none() || closed.unwrap().is_err(),
        "listener cancellation must wake upgraded IO"
    );
}
#[tokio::test]
async fn graceful_shutdown_allows_an_inflight_handler_to_finish() {
    let release = Arc::new(tokio::sync::Notify::new());
    let started = Arc::new(tokio::sync::Notify::new());
    let r = release.clone();
    let s = started.clone();
    let app = Router::new().route(
        "/",
        get(move || {
            let r = r.clone();
            let s = s.clone();
            async move {
                s.notify_one();
                r.notified().await;
                "finished"
            }
        }),
    );
    let mut server = fixture(app, limits()).await;
    let addr = server.addr;
    let request = tokio::spawn(async move {
        client()
            .get(format!("http://{addr}/"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
    });
    started.notified().await;
    server.stop.take().unwrap().send(()).unwrap();
    release.notify_one();
    assert_eq!(request.await.unwrap(), "finished");
    tokio::time::timeout(Duration::from_secs(2), &mut server.task)
        .await
        .unwrap()
        .unwrap();
}

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::CryptoProvider;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, SignatureScheme};
/// A client verifier that trusts any certificate — the test asserts on the
/// *identity* of the presented cert, not on a chain of trust.
#[derive(Debug)]
struct TrustAny(Arc<CryptoProvider>);

impl ServerCertVerifier for TrustAny {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

#[tokio::test]
async fn a_panicking_handler_releases_capacity_and_does_not_stop_the_listener() {
    async fn panic_handler() -> &'static str {
        panic!("deliberate fixture panic")
    }
    let app = Router::new()
        .route("/panic", get(panic_handler))
        .route("/", get(|| async { "healthy" }));
    let server = fixture(
        app,
        HttpLimits {
            requests: 1,
            ..limits()
        },
    )
    .await;
    assert!(
        client()
            .get(format!("http://{}/panic", server.addr))
            .send()
            .await
            .is_err()
    );
    assert_eq!(
        client()
            .get(format!("http://{}/", server.addr))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap(),
        "healthy"
    );
}
