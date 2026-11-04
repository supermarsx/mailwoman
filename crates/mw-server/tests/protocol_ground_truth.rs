//! Ground truth for the protocol anomaly guard: which malformed or ambiguous
//! HTTP/1.1 and HTTP/2 requests the framework (hyper 1.x / h2) already rejects or
//! normalises before any middleware runs, and which reach the application. Every
//! vector is sent as raw bytes to the real listener (`inbound_http::serve`),
//! against a router whose only handler records what it was given. The
//! expectations are asserted, so the table in `docs/admin/protocol-guard.md` is a
//! recorded fact rather than an assumption.
use axum::{Router, extract::Request};
use mw_server::inbound_http::{HttpLimits, serve};
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
async fn fixture() -> Fixture {
    let seen: Seen = Default::default();
    let recorder = seen.clone();
    let app = Router::new().fallback(move |request: Request| {
        let recorder = recorder.clone();
        async move {
            let (parts, body) = request.into_parts();
            let header = |name: &str| {
                parts
                    .headers
                    .get_all(name)
                    .iter()
                    .map(|v| String::from_utf8_lossy(v.as_bytes()).into_owned())
                    .collect::<Vec<_>>()
                    .join("|")
            };
            let body = axum::body::to_bytes(body, 1 << 20).await;
            recorder.lock().unwrap().push(format!(
                "{} {} {:?} cl=[{}] te=[{}] host=[{}] body={}",
                parts.method,
                parts.uri,
                parts.version,
                header("content-length"),
                header("transfer-encoding"),
                header("host"),
                body.map(|b| b.len().to_string())
                    .unwrap_or_else(|_| "err".into()),
            ));
            "hit"
        }
    });
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (stop, stopped) = oneshot::channel::<()>();
    let limits = HttpLimits {
        header_timeout: Duration::from_secs(1),
        body_timeout: Duration::from_secs(1),
        ..Default::default()
    };
    let task = tokio::spawn(async move {
        let _ = serve(listener, app, limits, async {
            let _ = stopped.await;
        })
        .await;
    });
    Fixture {
        addr,
        seen,
        _stop: stop,
        task,
    }
}
/// Send `request`, read until close or 1.5 s of silence.
async fn raw(addr: SocketAddr, request: &[u8]) -> Vec<u8> {
    let mut socket = TcpStream::connect(addr).await.unwrap();
    socket.write_all(request).await.unwrap();
    let mut out = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        match tokio::time::timeout(Duration::from_millis(1500), socket.read(&mut chunk)).await {
            Ok(Ok(0)) | Ok(Err(_)) | Err(_) => break,
            Ok(Ok(n)) => out.extend_from_slice(&chunk[..n]),
        }
    }
    out
}
fn statuses(response: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(response)
        .split("HTTP/1.")
        .skip(1)
        .map(|part| part.chars().take(5).collect::<String>())
        .collect()
}

#[derive(Debug, PartialEq)]
enum Outcome {
    /// Rejected before the middleware stack; the status the client saw.
    Rejected(&'static str),
    /// Reached the handler (so a middleware could see it) with this many hits.
    Reached(usize),
}
struct Vector {
    name: &'static str,
    bytes: Vec<u8>,
    expect: Outcome,
}
fn v(name: &'static str, bytes: &[u8], expect: Outcome) -> Vector {
    Vector {
        name,
        bytes: bytes.to_vec(),
        expect,
    }
}
fn post(name: &'static str, headers: &str, tail: &str, expect: Outcome) -> Vector {
    v(
        name,
        format!("POST / HTTP/1.1\r\nHost: example.test\r\n{headers}\r\n{tail}").as_bytes(),
        expect,
    )
}
fn get(name: &'static str, headers: &str, expect: Outcome) -> Vector {
    v(
        name,
        format!("GET / HTTP/1.1\r\nHost: example.test\r\n{headers}Connection: close\r\n\r\n")
            .as_bytes(),
        expect,
    )
}

async fn run(vectors: Vec<Vector>) -> String {
    let mut report = String::new();
    let mut failures = Vec::new();
    for vector in vectors {
        // A fresh listener per vector keeps hit counts attributable.
        let server = fixture().await;
        let response = raw(server.addr, &vector.bytes).await;
        let codes = statuses(&response);
        let hits = server.seen.lock().unwrap().clone();
        let observed = if hits.is_empty() {
            Outcome::Rejected(match codes.first().map(|s| &s[2..5]) {
                Some("400") => "400",
                Some("431") => "431",
                Some("501") => "501",
                Some("505") => "505",
                Some("408") => "408",
                Some("414") => "414",
                None => "closed",
                _ => "other",
            })
        } else {
            Outcome::Reached(hits.len())
        };
        report.push_str(&format!(
            "{:<40} responses={:?} handler={:?}\n",
            vector.name, codes, hits
        ));
        if observed != vector.expect {
            failures.push(format!(
                "{}: expected {:?}, observed {:?}",
                vector.name, vector.expect, observed
            ));
        }
    }
    eprintln!("{report}");
    assert!(
        failures.is_empty(),
        "ground truth drifted:\n{}",
        failures.join("\n")
    );
    report
}

#[tokio::test]
async fn framework_ground_truth_for_http1_anomalies() {
    use Outcome::{Reached, Rejected};
    let long_path = "a".repeat(40 * 1024);
    let vectors = vec![
        get("control: plain GET", "", Reached(1)),
        post("CL+TE together (CL.TE)", "Content-Length: 4\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n", "0\r\n\r\n", Reached(1)),
        post("TE.CL smuggled second request", "Content-Length: 4\r\nTransfer-Encoding: chunked\r\n", "5c\r\nGPOST /smuggled HTTP/1.1\r\nHost: x\r\nContent-Length: 15\r\n\r\nx=1\r\n0\r\n\r\n", Reached(1)),
        post("duplicate conflicting CL", "Content-Length: 5\r\nContent-Length: 6\r\n", "hello!", Rejected("400")),
        post("duplicate identical CL", "Content-Length: 5\r\nContent-Length: 5\r\nConnection: close\r\n", "hello", Reached(1)),
        post("CL comma list, differing", "Content-Length: 5, 6\r\n", "hello!", Rejected("400")),
        post("CL comma list, identical", "Content-Length: 5, 5\r\nConnection: close\r\n", "hello", Rejected("400")),
        post("CL negative", "Content-Length: -1\r\n", "", Rejected("400")),
        post("CL plus sign", "Content-Length: +5\r\n", "hello", Rejected("400")),
        post("CL non-numeric", "Content-Length: abc\r\n", "", Rejected("400")),
        post("CL overflow", "Content-Length: 99999999999999999999999\r\n", "", Rejected("400")),
        post("TE xchunked", "Transfer-Encoding: xchunked\r\nConnection: close\r\n", "", Rejected("400")),
        post("TE chunked, identity", "Transfer-Encoding: chunked, identity\r\nConnection: close\r\n", "0\r\n\r\n", Rejected("400")),
        post("TE identity, chunked", "Transfer-Encoding: identity, chunked\r\nConnection: close\r\n", "0\r\n\r\n", Reached(1)),
        post("TE gzip, chunked", "Transfer-Encoding: gzip, chunked\r\nConnection: close\r\n", "0\r\n\r\n", Reached(1)),
        post("TE chunked twice (two headers)", "Transfer-Encoding: chunked\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n", "0\r\n\r\n", Reached(1)),
        post("TE with leading tab", "Transfer-Encoding:\tchunked\r\nConnection: close\r\n", "0\r\n\r\n", Reached(1)),
        post("TE name, space before colon", "Transfer-Encoding : chunked\r\n", "0\r\n\r\n", Rejected("400")),
        post("TE obs-fold continuation", "Transfer-Encoding: chunked\r\n , identity\r\n", "0\r\n\r\n", Rejected("400")),
        v("TE on HTTP/1.0", b"POST / HTTP/1.0\r\nHost: example.test\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\n", Rejected("400")),
        get("header name, space before colon", "X-A : b\r\n", Rejected("400")),
        get("obs-fold continuation line", "X-A: b\r\n c\r\n", Rejected("400")),
        v("bare LF line endings", b"GET / HTTP/1.1\nHost: example.test\nConnection: close\n\n", Reached(1)),
        get("bare CR inside header value", "X-A: b\rc\r\n", Rejected("400")),
        v("NUL in header value", b"GET / HTTP/1.1\r\nHost: example.test\r\nX-A: b\0c\r\nConnection: close\r\n\r\n", Rejected("400")),
        v("control char 0x01 in header value", b"GET / HTTP/1.1\r\nHost: example.test\r\nX-A: b\x01c\r\nConnection: close\r\n\r\n", Rejected("400")),
        v("DEL 0x7f in header value", b"GET / HTTP/1.1\r\nHost: example.test\r\nX-A: b\x7fc\r\nConnection: close\r\n\r\n", Rejected("400")),
        v("NUL in path", b"GET /a\0b HTTP/1.1\r\nHost: example.test\r\nConnection: close\r\n\r\n", Rejected("400")),
        v("space in path", b"GET /a b HTTP/1.1\r\nHost: example.test\r\nConnection: close\r\n\r\n", Rejected("400")),
        v("raw tab in path", b"GET /a\tb HTTP/1.1\r\nHost: example.test\r\nConnection: close\r\n\r\n", Rejected("400")),
        v("UTF-8 in header value", "GET / HTTP/1.1\r\nHost: example.test\r\nX-A: caf\u{e9}\r\nConnection: close\r\n\r\n".as_bytes(), Reached(1)),
        v("invalid UTF-8 (0xff) in header value", b"GET / HTTP/1.1\r\nHost: example.test\r\nX-A: b\xffc\r\nConnection: close\r\n\r\n", Reached(1)),
        v("UTF-8 in path", "GET /caf\u{e9} HTTP/1.1\r\nHost: example.test\r\nConnection: close\r\n\r\n".as_bytes(), Reached(1)),
        v("absolute-form, other authority", b"GET http://evil.test/x HTTP/1.1\r\nHost: example.test\r\nConnection: close\r\n\r\n", Reached(1)),
        v("HTTP/0.9 request line", b"GET /\r\n\r\n", Rejected("400")),
        v("HTTP/2.0 on a 1.x connection", b"GET / HTTP/2.0\r\nHost: example.test\r\n\r\n", Rejected("400")),
        v("HTTP/9.9", b"GET / HTTP/9.9\r\nHost: example.test\r\n\r\n", Rejected("400")),
        v("HTTP/1.1x", b"GET / HTTP/1.1x\r\nHost: example.test\r\n\r\n", Rejected("400")),
        v("lowercase method", b"get / HTTP/1.1\r\nHost: example.test\r\nConnection: close\r\n\r\n", Reached(1)),
        v("unknown method", b"BREW / HTTP/1.1\r\nHost: example.test\r\nConnection: close\r\n\r\n", Reached(1)),
        v("TRACE", b"TRACE / HTTP/1.1\r\nHost: example.test\r\nConnection: close\r\n\r\n", Reached(1)),
        v("CONNECT authority-form", b"CONNECT example.test:443 HTTP/1.1\r\nHost: example.test:443\r\nConnection: close\r\n\r\n", Reached(1)),
        v("overlong request line (40 KiB)", format!("GET /{long_path} HTTP/1.1\r\nHost: example.test\r\n\r\n").as_bytes(), Rejected("431")),
        post("Expect: 100-continue", "Expect: 100-continue\r\nContent-Length: 2\r\nConnection: close\r\n", "hi", Reached(1)),
        post("Expect: unknown token", "Expect: banana\r\nContent-Length: 2\r\nConnection: close\r\n", "hi", Reached(1)),
        v("GET with a body", b"GET / HTTP/1.1\r\nHost: example.test\r\nContent-Length: 2\r\nConnection: close\r\n\r\nhi", Reached(1)),
        v("duplicate Host", b"GET / HTTP/1.1\r\nHost: a.test\r\nHost: b.test\r\nConnection: close\r\n\r\n", Reached(1)),
        v("empty Host", b"GET / HTTP/1.1\r\nHost:\r\nConnection: close\r\n\r\n", Reached(1)),
        v("userinfo in Host", b"GET / HTTP/1.1\r\nHost: user@evil.test\r\nConnection: close\r\n\r\n", Reached(1)),
        v("missing Host on HTTP/1.1", b"GET / HTTP/1.1\r\nConnection: close\r\n\r\n", Reached(1)),
        v("pipelined legitimate pair", b"GET /one HTTP/1.1\r\nHost: example.test\r\n\r\nGET /two HTTP/1.1\r\nHost: example.test\r\nConnection: close\r\n\r\n", Reached(2)),
        v("dot segments / %2e / \\ / %00 in path", b"GET /a/../b/%2e%2e/c//d\\e%00f HTTP/1.1\r\nHost: example.test\r\nConnection: close\r\n\r\n", Reached(1)),
        v("fragment in request target", b"GET /a#frag HTTP/1.1\r\nHost: example.test\r\nConnection: close\r\n\r\n", Reached(1)),
    ];
    run(vectors).await;
}

// ── HTTP/2 (plaintext prior-knowledge, hand-built frames) ────────────────────

fn hpack_literal(name: &str, value: &[u8]) -> Vec<u8> {
    let mut out = vec![0x00, name.len() as u8];
    out.extend_from_slice(name.as_bytes());
    out.push(value.len() as u8);
    out.extend_from_slice(value);
    out
}
fn frame(kind: u8, flags: u8, stream: u32, payload: &[u8]) -> Vec<u8> {
    let len = payload.len() as u32;
    let mut out = vec![(len >> 16) as u8, (len >> 8) as u8, len as u8, kind, flags];
    out.extend_from_slice(&stream.to_be_bytes());
    out.extend_from_slice(payload);
    out
}
/// Frame types the server sent back, and whether the handler ran.
async fn h2_probe(block: Vec<u8>, end_stream: bool) -> (Vec<u8>, Vec<String>) {
    let server = fixture().await;
    let mut socket = TcpStream::connect(server.addr).await.unwrap();
    let mut bytes = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n".to_vec();
    bytes.extend(frame(4, 0, 0, &[]));
    bytes.extend(frame(1, 0x4 | if end_stream { 0x1 } else { 0 }, 1, &block));
    socket.write_all(&bytes).await.unwrap();
    let mut out = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        match tokio::time::timeout(Duration::from_millis(1200), socket.read(&mut chunk)).await {
            Ok(Ok(0)) | Ok(Err(_)) | Err(_) => break,
            Ok(Ok(n)) => out.extend_from_slice(&chunk[..n]),
        }
    }
    let mut kinds = Vec::new();
    let mut at = 0;
    while at + 9 <= out.len() {
        let len = ((out[at] as usize) << 16) | ((out[at + 1] as usize) << 8) | out[at + 2] as usize;
        kinds.push(out[at + 3]);
        at += 9 + len;
    }
    let hits = server.seen.lock().unwrap().clone();
    (kinds, hits)
}
fn base() -> Vec<u8> {
    let mut block = vec![0x82, 0x86, 0x84];
    block.extend([0x01, 12]);
    block.extend_from_slice(b"example.test");
    block
}

#[tokio::test]
async fn framework_ground_truth_for_http2_anomalies() {
    // 0 DATA, 1 HEADERS, 3 RST_STREAM, 4 SETTINGS, 7 GOAWAY
    let with = |extra: Vec<u8>| {
        let mut block = base();
        block.extend(extra);
        block
    };
    let cases: Vec<(&str, Vec<u8>, bool)> = vec![
        ("control", base(), true),
        ("uppercase header name", with(hpack_literal("X-Upper", b"1")), true),
        ("transfer-encoding: chunked", with(hpack_literal("transfer-encoding", b"chunked")), true),
        ("connection: keep-alive", with(hpack_literal("connection", b"keep-alive")), true),
        ("te: trailers", with(hpack_literal("te", b"trailers")), true),
        ("te: gzip", with(hpack_literal("te", b"gzip")), true),
        ("CRLF in header value", with(hpack_literal("x-a", b"b\r\nx-b: c")), true),
        ("NUL in header value", with(hpack_literal("x-a", b"b\0c")), true),
        ("content-length 5, no body", with(hpack_literal("content-length", b"5")), true),
        ("host differs from :authority", with(hpack_literal("host", b"other.test")), true),
        ("pseudo-header after regular header", {
            let mut block = vec![0x82, 0x86];
            block.extend(hpack_literal("x-a", b"1"));
            block.push(0x84);
            block
        }, true),
        ("missing :path", vec![0x82, 0x86], true),
        ("duplicate :path", {
            let mut block = base();
            block.push(0x84);
            block
        }, true),
        ("NUL in :path", {
            let mut block = vec![0x82, 0x86];
            block.extend([0x04, 4]);
            block.extend_from_slice(b"/a\0b");
            block
        }, true),
    ];
    let mut report = String::new();
    for (name, block, end) in cases {
        let (kinds, hits) = h2_probe(block, end).await;
        report.push_str(&format!("h2 {:<36} frames={:?} handler={:?}\n", name, kinds, hits));
        // What the h2 stack did: hand the request on, reset the stream
        // (RST_STREAM), or end the connection (GOAWAY).
        let observed = if !hits.is_empty() {
            "reached"
        } else if kinds.contains(&7) {
            "goaway"
        } else if kinds.contains(&3) {
            "reset"
        } else {
            "silent"
        };
        let expected = match name {
            "control" | "te: trailers" | "host differs from :authority" | "missing :path" => {
                "reached"
            }
            "uppercase header name" | "CRLF in header value" | "NUL in header value" => "goaway",
            _ => "reset",
        };
        assert_eq!(observed, expected, "h2 {name}: {kinds:?} {hits:?}");
    }
    eprintln!("{report}");
}
