use super::{
    check::{View, host_syntax_ok, inspect},
    config::{Class, Config, Mode, normalise_host},
    run, stats,
};
use axum::{
    Router,
    extract::{ConnectInfo, Request},
    http::{HeaderMap, HeaderName, HeaderValue, Method, Uri, Version},
    middleware,
    routing::get,
};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::oneshot,
};

fn codes_with(
    cfg: &Config,
    method: &str,
    target: &str,
    version: Version,
    headers: &[(&str, &str)],
    peer_trusted: bool,
) -> Vec<&'static str> {
    let mut map = HeaderMap::new();
    for (name, value) in headers {
        map.append(
            HeaderName::from_bytes(name.as_bytes()).unwrap(),
            HeaderValue::from_bytes(value.as_bytes()).unwrap(),
        );
    }
    let method = Method::from_bytes(method.as_bytes()).unwrap();
    let uri: Uri = target.parse().unwrap();
    let view = View {
        method: &method,
        uri: &uri,
        version,
        headers: &map,
        peer_trusted,
        proxies_configured: false,
        peer_loopback: false,
    };
    inspect(cfg, &view).into_iter().map(|f| f.code).collect()
}
fn codes(method: &str, target: &str, headers: &[(&str, &str)]) -> Vec<&'static str> {
    codes_with(&all_on(), method, target, Version::HTTP_11, headers, false)
}
/// Every class on, so a unit test sees what each finds; modes are tested apart.
fn all_on() -> Config {
    let mut cfg = Config::default();
    cfg.hosts.clear();
    cfg
}
const HOST: (&str, &str) = ("host", "mail.example.test");

#[test]
fn ordinary_requests_raise_nothing() {
    // A browser navigation, a JMAP call, a chunked upload, an XHR with cookies.
    let browser = [
        HOST,
        ("user-agent", "Mozilla/5.0 (X11; Linux x86_64) Firefox/130.0"),
        ("accept", "text/html,application/xhtml+xml"),
        ("accept-language", "pt-PT,pt;q=0.9"),
        ("accept-encoding", "gzip, deflate, br"),
        ("connection", "keep-alive"),
        ("cookie", "mw_session=abc123; mw_csrf=def456"),
        ("sec-fetch-site", "same-origin"),
        ("sec-fetch-mode", "navigate"),
        ("sec-fetch-dest", "document"),
        ("upgrade-insecure-requests", "1"),
    ];
    assert!(codes("GET", "/inbox?q=a%20b&page=2", &browser).is_empty());
    let jmap = [
        HOST,
        ("content-type", "application/json"),
        ("content-length", "128"),
        ("authorization", "Bearer token"),
        ("origin", "https://mail.example.test"),
    ];
    assert!(codes("POST", "/jmap", &jmap).is_empty());
    let chunked = [HOST, ("transfer-encoding", "chunked"), ("content-type", "application/octet-stream")];
    assert!(codes("PUT", "/api/upload", &chunked).is_empty());
    assert!(codes("PROPFIND", "/dav/calendars/a/b", &[HOST, ("depth", "1")]).is_empty());
    assert!(codes("REPORT", "/dav/addressbooks/a/b", &[HOST]).is_empty());
    assert!(codes("OPTIONS", "/jmap", &[HOST, ("origin", "tauri://localhost")]).is_empty());
    // Encoded folder path with an escaped slash is ordinary here.
    assert!(codes("GET", "/api/folders/INBOX%2Fsub/messages", &[HOST]).is_empty());
}

#[test]
fn websocket_upgrade_is_accepted_only_on_the_websocket_route() {
    let upgrade = [HOST, ("connection", "Upgrade"), ("upgrade", "websocket")];
    assert!(codes("GET", "/jmap/ws", &upgrade).is_empty());
    assert!(codes("GET", "/mail/jmap/ws", &upgrade).is_empty());
    assert!(codes("GET", "/api/other", &upgrade).contains(&"upgrade-unexpected"));
    let h2c = [HOST, ("connection", "Upgrade, HTTP2-Settings"), ("upgrade", "h2c")];
    assert!(codes("GET", "/jmap/ws", &h2c).contains(&"upgrade-unexpected"));
    assert!(codes("GET", "/x", &[HOST, ("upgrade", "websocket")]).contains(&"upgrade-connection-mismatch"));
    // SSE is a plain GET.
    assert!(codes("GET", "/jmap/eventsource", &[HOST, ("accept", "text/event-stream")]).is_empty());
}

#[test]
fn host_syntax_accepts_real_hosts_and_refuses_ambiguous_ones() {
    for ok in [
        "mail.example.test",
        "mail.example.test:8443",
        "mail.example.test.",
        "localhost",
        "127.0.0.1:8080",
        "[::1]",
        "[::1]:8080",
        "[2001:db8::1]:443",
        "xn--caf-dma.example",
        "my_service:3000",
    ] {
        assert!(host_syntax_ok(ok), "{ok}");
        assert!(codes("GET", "/", &[("host", ok)]).is_empty(), "{ok}");
    }
    for bad in [
        "",
        "user@evil.test",
        "a b",
        "evil.test/path",
        "evil.test\\x",
        "evil.test:",
        "evil.test:99999",
        "evil.test:80a",
        "[::1",
        "[]",
        "[evil]",
        ".evil",
        "a..b",
        "a#b",
        "a?b",
    ] {
        assert!(!host_syntax_ok(bad), "{bad}");
    }
    assert_eq!(codes("GET", "/", &[HOST, HOST]), ["host-multiple"]);
    assert_eq!(codes("GET", "/", &[("host", "user@evil.test")]), ["host-syntax"]);
    assert_eq!(codes("GET", "/", &[("host", "")]), ["host-empty"]);
    assert_eq!(codes("GET", "/", &[]), ["host-missing"]);
    assert!(codes_with(&all_on(), "GET", "/", Version::HTTP_10, &[], false).is_empty());
    assert_eq!(normalise_host("Mail.Example.TEST.:443"), "mail.example.test");
    assert_eq!(normalise_host("[::1]:8080"), "::1");
}

#[test]
fn host_pinning_follows_the_configured_names_only() {
    let mut cfg = all_on();
    assert!(codes_with(&cfg, "GET", "/", Version::HTTP_11, &[("host", "evil.test")], false).is_empty());
    cfg.hosts.insert("mail.example.test".into());
    cfg.hosts.insert("internal".into());
    let find = |host: &str| codes_with(&cfg, "GET", "/", Version::HTTP_11, &[("host", host)], false);
    assert!(find("mail.example.test").is_empty());
    assert!(find("MAIL.example.test:8443").is_empty());
    assert!(find("mail.example.test.").is_empty());
    assert!(find("internal").is_empty());
    assert_eq!(find("evil.test"), ["host-unpinned"]);
    assert_eq!(find("mail.example.test.evil.test"), ["host-unpinned"]);
}

#[test]
fn framing_findings_cover_the_smuggling_shapes() {
    let cl_te = [HOST, ("content-length", "4"), ("transfer-encoding", "chunked")];
    assert_eq!(codes("POST", "/", &cl_te), ["cl-and-te"]);
    for te in ["xchunked", "chunked, identity", "identity, chunked", "gzip, chunked", "chunked;x=1", ""] {
        assert!(
            codes("POST", "/", &[HOST, ("transfer-encoding", te)]).contains(&"te-value"),
            "{te:?}"
        );
    }
    let twice = [HOST, ("transfer-encoding", "chunked"), ("transfer-encoding", "chunked")];
    assert!(codes("POST", "/", &twice).contains(&"te-value"));
    for ok in ["chunked", "Chunked", " chunked "] {
        assert!(codes("POST", "/", &[HOST, ("transfer-encoding", ok)]).is_empty(), "{ok:?}");
    }
    let te = [HOST, ("transfer-encoding", "chunked")];
    assert!(codes_with(&all_on(), "POST", "/", Version::HTTP_10, &te, false).contains(&"te-version"));
    assert!(codes_with(&all_on(), "POST", "/", Version::HTTP_2, &te, false).contains(&"te-version"));
    for bad in ["-1", "+5", "abc", "5, x", "99999999999999999999999", "0x10", "5 5", ""] {
        assert!(
            codes("POST", "/", &[HOST, ("content-length", bad)]).contains(&"cl-syntax"),
            "{bad:?}"
        );
    }
    for conflicting in [
        codes("POST", "/", &[HOST, ("content-length", "5"), ("content-length", "6")]),
        codes("POST", "/", &[HOST, ("content-length", "5, 6")]),
    ] {
        assert!(conflicting.contains(&"cl-conflict"), "{conflicting:?}");
    }
    // Identical repeats are a hygiene finding, not a framing one.
    assert_eq!(
        codes("POST", "/", &[HOST, ("content-length", "5"), ("content-length", "5")]),
        ["content-length-repeated"]
    );
}

#[test]
fn request_target_findings() {
    let path = |target: &str| codes("GET", target, &[HOST]);
    assert_eq!(path("/a%00b"), ["path-nul"]);
    assert_eq!(path("/a%0d%0aSet-Cookie:x"), ["path-crlf"]);
    assert_eq!(path("/a%0Ab"), ["path-crlf"]);
    assert_eq!(path("/a/%2e%2e/b"), ["path-encoded-dot"]);
    assert_eq!(path("/a/%252e%252e/b"), ["path-double-encoded"]);
    assert_eq!(path("/a/%c0%ae%c0%ae/b"), ["path-overlong-utf8"]);
    assert_eq!(path("/a/../b"), ["path-dot-segment"]);
    assert_eq!(path("/a/./b"), ["path-dot-segment"]);
    assert_eq!(path("/a%2f../b"), ["path-dot-segment"]);
    assert_eq!(path("/a//b"), ["path-empty-segment"]);
    assert_eq!(path("/a%5cb"), ["path-backslash"]);
    // A query may hold NUL-looking or CRLF text; only the path is checked.
    assert!(path("/search?q=%0a%0d%2e%2e//").is_empty());
    assert!(path("/file.name.tar.gz").is_empty());
    assert!(path("/a%20b/%E2%9C%93").is_empty());
    // Absolute-form naming another authority than Host.
    assert_eq!(
        codes("GET", "http://evil.test/x", &[HOST]),
        ["authority-mismatch"]
    );
    assert!(codes("GET", "http://mail.example.test/x", &[HOST]).is_empty());
    // Length caps.
    let mut cfg = all_on();
    cfg.max_path = 20;
    cfg.max_query = 10;
    let long = |t: &str| codes_with(&cfg, "GET", t, Version::HTTP_11, &[HOST], false);
    assert_eq!(long(&format!("/{}", "a".repeat(30))), ["path-long"]);
    assert_eq!(long("/a?q=0123456789ab"), ["query-long"]);
    assert!(long("/a?q=1").is_empty());
}

#[test]
fn method_header_hygiene_and_body_findings() {
    for method in ["TRACE", "TRACK", "CONNECT", "BREW", "get"] {
        assert_eq!(codes(method, "/", &[HOST]), ["method-not-allowed"], "{method}");
    }
    assert_eq!(
        codes("POST", "/", &[HOST, ("content-type", "a/b"), ("content-type", "c/d")]),
        ["singleton-repeated"]
    );
    assert_eq!(
        codes("GET", "/", &[HOST, ("cookie", "a=1"), ("cookie", "b=2")]),
        ["cookie-repeated"]
    );
    // HTTP/2 splits Cookie into several headers legitimately.
    assert!(
        codes_with(&all_on(), "GET", "/", Version::HTTP_2, &[HOST, ("cookie", "a=1"), ("cookie", "b=2")], false)
            .is_empty()
    );
    let mut cfg = all_on();
    cfg.max_headers = 3;
    cfg.max_header_value = 20;
    cfg.max_cookie = 30;
    let many = codes_with(
        &cfg,
        "GET",
        "/",
        Version::HTTP_11,
        &[HOST, ("a", "1"), ("b", "2"), ("c", "3"), ("x-long", &"v".repeat(40)), ("cookie", &"c=1;".repeat(20))],
        false,
    );
    for code in ["too-many-headers", "header-value-long", "cookie-large"] {
        assert!(many.contains(&code), "{code}");
    }
    let opaque = codes_with(&all_on(), "GET", "/", Version::HTTP_11, &[HOST], false);
    assert!(opaque.is_empty());
    let mut map = HeaderMap::new();
    map.insert("host", HeaderValue::from_static("a.test"));
    map.insert("x-a", HeaderValue::from_bytes(b"b\xffc").unwrap());
    let view = View {
        method: &Method::GET,
        uri: &"/".parse().unwrap(),
        version: Version::HTTP_11,
        headers: &map,
        peer_trusted: false,
        proxies_configured: false,
        peer_loopback: false,
    };
    assert_eq!(inspect(&all_on(), &view)[0].code, "header-opaque-bytes");
    assert!(
        codes("GET", "/", &[HOST, ("connection", "close, Content-Length")])
            .contains(&"connection-strips-header")
    );
    assert_eq!(codes("GET", "/", &[HOST, ("content-length", "5")]), ["safe-method-body"]);
    assert_eq!(codes("HEAD", "/", &[HOST, ("transfer-encoding", "chunked")]), ["safe-method-body"]);
    assert_eq!(codes("POST", "/", &[HOST, ("expect", "banana"), ("content-length", "1")]), ["expect-unknown"]);
    assert_eq!(codes("POST", "/", &[HOST, ("expect", "100-continue")]), ["expect-without-body"]);
    assert!(codes("POST", "/", &[HOST, ("expect", "100-continue"), ("content-length", "9")]).is_empty());
    cfg.max_body = 100;
    assert!(
        codes_with(&cfg, "POST", "/", Version::HTTP_11, &[HOST, ("content-length", "101")], false)
            .contains(&"body-too-large")
    );
}

#[test]
fn forwarding_headers_are_flagged_from_untrusted_peers_only() {
    let spoof = [HOST, ("x-forwarded-for", "203.0.113.9"), ("x-forwarded-proto", "https"), ("forwarded", "for=1.2.3.4")];
    assert_eq!(
        codes_with(&all_on(), "GET", "/", Version::HTTP_11, &spoof, false),
        ["forwarded-from-untrusted-peer"]
    );
    assert!(codes_with(&all_on(), "GET", "/", Version::HTTP_11, &spoof, true).is_empty());
}

#[test]
fn off_mode_and_exemptions_behave() {
    let cfg = all_on().with(Class::Method, Mode::Off);
    assert!(codes_with(&cfg, "TRACE", "/", Version::HTTP_11, &[HOST], false).is_empty());
    let mut cfg = all_on();
    cfg.exempt = vec!["/api/import".into()];
    let doubled = [HOST, ("content-type", "a/b"), ("content-type", "c/d")];
    assert!(codes_with(&cfg, "POST", "/api/import", Version::HTTP_11, &doubled, false).is_empty());
    assert!(codes_with(&cfg, "POST", "/api/import/x", Version::HTTP_11, &doubled, false).is_empty());
    assert!(!codes_with(&cfg, "POST", "/api/imports", Version::HTTP_11, &doubled, false).is_empty());
    // Smuggling-class and path-control findings are never exempted.
    let cl_te = [HOST, ("content-length", "4"), ("transfer-encoding", "chunked")];
    assert_eq!(codes_with(&cfg, "POST", "/api/import", Version::HTTP_11, &cl_te, false), ["cl-and-te"]);
    assert_eq!(codes_with(&cfg, "GET", "/api/import/a%00b", Version::HTTP_11, &[HOST], false), ["path-nul"]);
}

#[test]
fn exemption_prefixes_that_would_switch_the_guard_off_are_dropped() {
    let rows = |value: serde_json::Value| {
        vec![crate::settings_registry::Effective {
            key: "security.protocol.exemptPrefixes".into(),
            value,
            source: mw_store::settings_policy::SettingSource::Default,
            locked_by: None,
            user_may_override: false,
            user_value: None,
            inherited: false,
        }]
    };
    let cfg = Config::parse(rows(serde_json::json!(["/", "api", "/", "/ok", "/api/up/"])));
    assert_eq!(cfg.exempt, ["/ok", "/api/up/"]);
    assert!(cfg.exempts("/ok"));
    assert!(cfg.exempts("/ok/deeper"));
    assert!(!cfg.exempts("/okay"));
    assert!(cfg.exempts("/api/up/file"));
    assert!(!cfg.exempts("/"));
}

#[test]
fn defaults_enforce_smuggling_and_syntax_classes_and_detect_heuristics() {
    let cfg = Config::default();
    for class in [Class::Framing, Class::Host, Class::PathControl] {
        assert_eq!(cfg.mode(class), Mode::Enforce, "{class:?}");
    }
    for class in Class::ALL {
        if !matches!(class, Class::Framing | Class::Host | Class::PathControl) {
            assert_eq!(cfg.mode(class), Mode::Detect, "{class:?}");
        }
    }
    let defaults: Vec<String> = DEFAULT_METHODS_FOR_TEST.iter().map(|m| (*m).to_owned()).collect();
    assert_eq!(cfg.methods.len(), defaults.len());
}
const DEFAULT_METHODS_FOR_TEST: &[&str] = &[
    "GET", "HEAD", "POST", "PUT", "PATCH", "DELETE", "OPTIONS", "PROPFIND", "PROPPATCH", "REPORT",
    "MKCOL", "MKCALENDAR", "COPY", "MOVE", "LOCK", "UNLOCK",
];

// ── raw-socket tests against the real listener ───────────────────────────────

type Seen = Arc<Mutex<Vec<String>>>;
struct Fixture {
    addr: SocketAddr,
    seen: Seen,
    _stop: oneshot::Sender<()>,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.task.abort();
    }
}
/// The real listener with the guard in front of a recording fallback plus
/// `/healthz` and `/readyz`. The recorder stores the resolved client address so a
/// test can prove a forwarded header was not trusted.
async fn fixture(cfg: Config) -> Fixture {
    let seen: Seen = Default::default();
    let recorder = seen.clone();
    let cfg = Arc::new(cfg);
    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/readyz", get(|| async { "ready" }))
        .fallback(move |request: Request| {
            let recorder = recorder.clone();
            async move {
                let ip = crate::scope_mw::proxy::client_ip(request.headers(), request.extensions());
                let peer = request.extensions().get::<ConnectInfo<SocketAddr>>().map(|c| c.0.ip());
                let (parts, body) = request.into_parts();
                let _ = axum::body::to_bytes(body, 1 << 20).await;
                recorder
                    .lock()
                    .unwrap()
                    .push(format!("{} {} client={ip:?} peer={peer:?}", parts.method, parts.uri));
                "hit"
            }
        })
        .layer(middleware::from_fn(move |request, next| {
            let cfg = cfg.clone();
            async move { run(cfg, None, request, next).await }
        }));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (stop, stopped) = oneshot::channel::<()>();
    let limits = crate::inbound_http::HttpLimits {
        header_timeout: Duration::from_secs(1),
        body_timeout: Duration::from_secs(1),
        ..Default::default()
    };
    let task = tokio::spawn(async move {
        let _ = crate::inbound_http::serve(listener, app, limits, async {
            let _ = stopped.await;
        })
        .await;
    });
    Fixture { addr, seen, _stop: stop, task }
}
async fn raw(addr: SocketAddr, request: &[u8]) -> String {
    let mut socket = TcpStream::connect(addr).await.unwrap();
    socket.write_all(request).await.unwrap();
    let mut out = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        match tokio::time::timeout(Duration::from_millis(1200), socket.read(&mut chunk)).await {
            Ok(Ok(0)) | Ok(Err(_)) | Err(_) => break,
            Ok(Ok(n)) => out.extend_from_slice(&chunk[..n]),
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}
fn responses(text: &str) -> usize {
    text.matches("HTTP/1.1 ").count()
}
fn status(text: &str) -> &str {
    text.get(9..12).unwrap_or("")
}

#[tokio::test]
async fn smuggling_vectors_are_not_parsed_as_a_second_request() {
    // CL.TE, TE.CL, TE.TE and request splitting through an encoded CRLF. Each
    // carries a second, complete request after the first; none may reach the
    // application, and the first is refused with the connection closed.
    let smuggled = "GET /smuggled HTTP/1.1\r\nHost: mail.example.test\r\n\r\n";
    let cl_te = format!(
        "POST / HTTP/1.1\r\nHost: mail.example.test\r\nContent-Length: {}\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\n{smuggled}",
        smuggled.len() + 5
    );
    let te_cl = format!(
        "POST / HTTP/1.1\r\nHost: mail.example.test\r\nContent-Length: 4\r\nTransfer-Encoding: chunked\r\n\r\n{:x}\r\n{smuggled}\r\n0\r\n\r\n",
        smuggled.len()
    );
    let te_te_xchunked = format!(
        "POST / HTTP/1.1\r\nHost: mail.example.test\r\nTransfer-Encoding: xchunked\r\n\r\n0\r\n\r\n{smuggled}"
    );
    let te_te_tab = format!(
        "POST / HTTP/1.1\r\nHost: mail.example.test\r\nTransfer-Encoding: chunked\r\nTransfer-Encoding:\tidentity\r\n\r\n0\r\n\r\n{smuggled}"
    );
    let te_te_list = format!(
        "POST / HTTP/1.1\r\nHost: mail.example.test\r\nTransfer-Encoding: chunked, identity\r\n\r\n0\r\n\r\n{smuggled}"
    );
    let dup_cl = format!(
        "POST / HTTP/1.1\r\nHost: mail.example.test\r\nContent-Length: 0\r\nContent-Length: {}\r\n\r\n{smuggled}",
        smuggled.len()
    );
    let split = format!(
        "GET /a%0d%0a%0d%0aGET%20/smuggled HTTP/1.1\r\nHost: mail.example.test\r\n\r\n{smuggled}"
    );
    for (name, request) in [
        ("CL.TE", cl_te),
        ("TE.CL", te_cl),
        ("TE.TE xchunked", te_te_xchunked),
        ("TE.TE second header", te_te_tab),
        ("TE.TE list", te_te_list),
        ("duplicate CL", dup_cl),
        ("CRLF in path", split),
    ] {
        let server = fixture(Config::default()).await;
        let response = raw(server.addr, request.as_bytes()).await;
        let hits = server.seen.lock().unwrap().clone();
        assert!(
            !hits.iter().any(|h| h.contains("/smuggled")),
            "{name}: smuggled request reached the application: {hits:?}"
        );
        assert!(hits.is_empty(), "{name}: {hits:?}");
        assert_eq!(responses(&response), 1, "{name}: {response}");
        assert_eq!(status(&response), "400", "{name}: {response}");
    }
}

#[tokio::test]
async fn framing_reject_closes_the_connection_and_reflects_nothing() {
    let server = fixture(Config::default()).await;
    let response = raw(
        server.addr,
        b"POST /reflectme-marker HTTP/1.1\r\nHost: mail.example.test\r\nContent-Length: 4\r\nTransfer-Encoding: chunked\r\nX-Secret: secret-token-marker\r\n\r\n0\r\n\r\n",
    )
    .await;
    assert_eq!(status(&response), "400", "{response}");
    let lower = response.to_ascii_lowercase();
    assert!(lower.contains("connection: close"), "{response}");
    assert!(lower.contains("cache-control: no-store"), "{response}");
    for marker in ["reflectme", "secret-token", "chunked"] {
        assert!(!lower.contains(marker), "{marker} reflected: {response}");
    }
    assert!(server.seen.lock().unwrap().is_empty());
}

#[tokio::test]
async fn path_nul_and_bad_host_are_refused_by_default() {
    let server = fixture(Config::default()).await;
    let nul = raw(server.addr, b"GET /a%00evilmarker HTTP/1.1\r\nHost: mail.example.test\r\nConnection: close\r\n\r\n").await;
    assert_eq!(status(&nul), "400", "{nul}");
    assert!(!nul.contains("evilmarker"), "{nul}");
    for host in ["user@evil.test", "", "a b"] {
        let response = raw(
            server.addr,
            format!("GET / HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n").as_bytes(),
        )
        .await;
        assert_eq!(status(&response), "400", "{host:?}: {response}");
        assert!(!response.contains("evil.test"), "{response}");
    }
    let duplicate = raw(
        server.addr,
        b"GET / HTTP/1.1\r\nHost: a.test\r\nHost: b.test\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert_eq!(status(&duplicate), "400", "{duplicate}");
    assert!(server.seen.lock().unwrap().is_empty());
}

#[tokio::test]
async fn legitimate_requests_pass_through_the_real_listener() {
    let server = fixture(Config::default()).await;
    let pipelined = raw(
        server.addr,
        b"GET /one HTTP/1.1\r\nHost: mail.example.test\r\n\r\nPOST /two HTTP/1.1\r\nHost: [::1]:8080\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nabc\r\n0\r\n\r\nGET /three HTTP/1.1\r\nHost: mail.example.test.:8443\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert_eq!(responses(&pipelined), 3, "{pipelined}");
    assert_eq!(server.seen.lock().unwrap().len(), 3);
}

#[tokio::test]
async fn health_and_readiness_stay_reachable_even_when_a_class_enforces() {
    let cfg = Config::default().with(Class::Method, Mode::Enforce);
    let server = fixture(cfg).await;
    for path in ["/healthz", "/readyz"] {
        // Two Host headers would be refused anywhere else (hostMode enforces).
        let response = raw(
            server.addr,
            format!("GET {path} HTTP/1.1\r\nHost: a.test\r\nHost: b.test\r\nConnection: close\r\n\r\n").as_bytes(),
        )
        .await;
        assert_eq!(status(&response), "200", "{path}: {response}");
    }
    let refused = raw(server.addr, b"GET /elsewhere HTTP/1.1\r\nHost: a.test\r\nHost: b.test\r\nConnection: close\r\n\r\n").await;
    assert_eq!(status(&refused), "400", "{refused}");
}

#[tokio::test]
async fn detect_allows_and_counts_while_enforce_refuses_with_the_right_status() {
    let before = stats::counts(Class::Method);
    let detect = fixture(Config::default()).await;
    let allowed = raw(detect.addr, b"TRACE / HTTP/1.1\r\nHost: mail.example.test\r\nConnection: close\r\n\r\n").await;
    assert_eq!(status(&allowed), "200", "{allowed}");
    assert_eq!(detect.seen.lock().unwrap().len(), 1);
    let after = stats::counts(Class::Method);
    assert_eq!(after.0, before.0 + 1, "detected counter");
    assert_eq!(after.1, before.1, "enforced counter");

    let enforce = fixture(Config::default().with(Class::Method, Mode::Enforce)).await;
    let refused = raw(enforce.addr, b"TRACE /x-marker HTTP/1.1\r\nHost: mail.example.test\r\nConnection: close\r\n\r\n").await;
    assert_eq!(status(&refused), "405", "{refused}");
    assert!(refused.to_ascii_lowercase().contains("allow: "), "{refused}");
    assert!(!refused.contains("x-marker") && !refused.contains("TRACE"), "{refused}");
    assert!(enforce.seen.lock().unwrap().is_empty());
    assert_eq!(stats::counts(Class::Method).1, after.1 + 1);

    // A raw non-ASCII byte in the path is passed on by hyper; the target class counts it.
    let target_before = stats::counts(Class::Target);
    let accented = raw(
        detect.addr,
        "GET /caf\u{e9} HTTP/1.1\r\nHost: mail.example.test\r\nConnection: close\r\n\r\n".as_bytes(),
    )
    .await;
    assert_eq!(status(&accented), "200", "{accented}");
    assert_eq!(stats::counts(Class::Target).0, target_before.0 + 1);

    // Framing in detect mode passes the request on and still counts it.
    let framing_before = stats::counts(Class::Framing);
    let loose = fixture(Config::default().with(Class::Framing, Mode::Detect)).await;
    let passed = raw(
        loose.addr,
        b"POST / HTTP/1.1\r\nHost: mail.example.test\r\nContent-Length: 4\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n0\r\n\r\n",
    )
    .await;
    assert_eq!(status(&passed), "200", "{passed}");
    assert_eq!(stats::counts(Class::Framing).0, framing_before.0 + 1);
}

#[tokio::test]
async fn size_and_length_enforcement_use_their_own_statuses() {
    let mut cfg = Config::default()
        .with(Class::PathLength, Mode::Enforce)
        .with(Class::HeaderHygiene, Mode::Enforce)
        .with(Class::Body, Mode::Enforce);
    cfg.max_path = 32;
    cfg.max_body = 10;
    cfg.max_header_value = 64;
    let server = fixture(cfg).await;
    let long = raw(
        server.addr,
        format!("GET /{} HTTP/1.1\r\nHost: a.test\r\nConnection: close\r\n\r\n", "a".repeat(64)).as_bytes(),
    )
    .await;
    assert_eq!(status(&long), "414", "{long}");
    let big = raw(
        server.addr,
        b"POST / HTTP/1.1\r\nHost: a.test\r\nContent-Length: 100000\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert_eq!(status(&big), "413", "{big}");
    let wide = raw(
        server.addr,
        format!("GET / HTTP/1.1\r\nHost: a.test\r\nX-Wide: {}\r\nConnection: close\r\n\r\n", "v".repeat(100)).as_bytes(),
    )
    .await;
    assert_eq!(status(&wide), "431", "{wide}");
    assert!(server.seen.lock().unwrap().is_empty());
}

#[tokio::test]
async fn spoofed_forwarding_headers_are_flagged_and_never_trusted() {
    // No MW_TRUSTED_PROXIES in the test environment: every peer is untrusted.
    assert!(std::env::var("MW_TRUSTED_PROXIES").is_err());
    let before = stats::counts(Class::Forwarded);
    let server = fixture(Config::default()).await;
    let response = raw(
        server.addr,
        b"GET / HTTP/1.1\r\nHost: mail.example.test\r\nX-Forwarded-For: 203.0.113.77\r\nForwarded: for=203.0.113.78\r\nX-Real-IP: 203.0.113.79\r\nConnection: close\r\n\r\n",
    )
    .await;
    assert_eq!(status(&response), "200", "{response}");
    let hits = server.seen.lock().unwrap().clone();
    assert_eq!(hits.len(), 1);
    assert!(hits[0].contains("client=Some(127.0.0.1)"), "{hits:?}");
    assert!(!hits[0].contains("203.0.113"), "{hits:?}");
    assert_eq!(stats::counts(Class::Forwarded).0, before.0 + 1);
}
