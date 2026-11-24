//! Calibration server (NOT a Mailwoman backend): answers HTTP/1.1 keep-alive and
//! h2c (prior knowledge) on literal 127.0.0.1. `constant` returns one fixed JSON
//! body for every request; `replay` returns the recorded response of an exactly
//! matching request (method, path, cookie, body) from a corpus captured from a real
//! fixture, so the goal-mix validators run on real response bytes while the server
//! does no work. Unknown requests get 404.
use bytes::Bytes;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

struct Reply {
    status: u16,
    content_type: &'static str,
    body: Bytes,
}

struct Table {
    constant: Option<Reply>,
    replay: HashMap<Vec<u8>, Reply>,
    served: AtomicU64,
    missed: AtomicU64,
}

fn key(method: &str, path: &str, cookie: &str, body: &[u8]) -> Vec<u8> {
    let mut k = Vec::with_capacity(method.len() + path.len() + cookie.len() + body.len() + 3);
    for part in [method.as_bytes(), path.as_bytes(), cookie.as_bytes(), body] {
        k.extend_from_slice(part);
        k.push(0);
    }
    k
}

impl Table {
    fn answer(
        &self,
        method: &str,
        path: &str,
        cookie: &str,
        body: &[u8],
    ) -> (u16, &'static str, Bytes) {
        self.served.fetch_add(1, Ordering::Relaxed);
        if let Some(reply) = &self.constant {
            return (reply.status, reply.content_type, reply.body.clone());
        }
        match self.replay.get(&key(method, path, cookie, body)) {
            Some(reply) => (reply.status, reply.content_type, reply.body.clone()),
            None => {
                self.missed.fetch_add(1, Ordering::Relaxed);
                (
                    404,
                    "application/json",
                    Bytes::from_static(b"{\"code\":\"not-recorded\"}"),
                )
            }
        }
    }
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        404 => "Not Found",
        _ => "Status",
    }
}

async fn http1(mut stream: TcpStream, table: Arc<Table>, first: Vec<u8>) {
    let _ = stream.set_nodelay(true);
    let mut buf = first;
    let mut out = Vec::with_capacity(8192);
    loop {
        // parse as many complete requests as are buffered
        loop {
            let mut headers = [httparse::EMPTY_HEADER; 64];
            let mut request = httparse::Request::new(&mut headers);
            let head = match request.parse(&buf) {
                Ok(httparse::Status::Complete(n)) => n,
                Ok(httparse::Status::Partial) => break,
                Err(_) => return,
            };
            let mut length = 0usize;
            let mut cookie = String::new();
            for h in request.headers.iter() {
                if h.name.eq_ignore_ascii_case("content-length") {
                    length = std::str::from_utf8(h.value)
                        .ok()
                        .and_then(|v| v.trim().parse().ok())
                        .unwrap_or(0);
                } else if h.name.eq_ignore_ascii_case("cookie") {
                    if !cookie.is_empty() {
                        cookie.push_str("; ");
                    }
                    cookie.push_str(std::str::from_utf8(h.value).unwrap_or(""));
                }
            }
            if buf.len() < head + length {
                break;
            }
            let (status, content_type, body) = table.answer(
                request.method.unwrap_or(""),
                request.path.unwrap_or(""),
                &cookie,
                &buf[head..head + length],
            );
            out.extend_from_slice(format!("HTTP/1.1 {status} {}\r\ncontent-type: {content_type}\r\ncontent-length: {}\r\n\r\n", reason(status), body.len()).as_bytes());
            out.extend_from_slice(&body);
            buf.drain(..head + length);
        }
        if !out.is_empty() {
            if stream.write_all(&out).await.is_err() {
                return;
            }
            out.clear();
        }
        let mut chunk = [0u8; 16384];
        match stream.read(&mut chunk).await {
            Ok(0) | Err(_) => return,
            Ok(n) => buf.extend_from_slice(&chunk[..n]),
        }
    }
}

async fn http2(stream: TcpStream, table: Arc<Table>, first: Vec<u8>) {
    let _ = stream.set_nodelay(true);
    let io = Prefixed {
        first,
        at: 0,
        stream,
    };
    let Ok(mut connection) = h2::server::Builder::new()
        .max_concurrent_streams(1000)
        .handshake::<_, Bytes>(io)
        .await
    else {
        return;
    };
    while let Some(Ok((request, mut respond))) = connection.accept().await {
        let table = table.clone();
        tokio::spawn(async move {
            let (parts, mut body) = request.into_parts();
            let mut data = Vec::new();
            while let Some(Ok(chunk)) = body.data().await {
                let _ = body.flow_control().release_capacity(chunk.len());
                data.extend_from_slice(&chunk);
            }
            let cookie = parts
                .headers
                .get_all("cookie")
                .iter()
                .filter_map(|v| v.to_str().ok())
                .collect::<Vec<_>>()
                .join("; ");
            let path = parts
                .uri
                .path_and_query()
                .map_or("/", |p| p.as_str())
                .to_owned();
            let (status, content_type, reply) =
                table.answer(parts.method.as_str(), &path, &cookie, &data);
            let response = http::Response::builder()
                .status(status)
                .header("content-type", content_type)
                .header("content-length", reply.len())
                .body(())
                .unwrap();
            if let Ok(mut send) = respond.send_response(response, reply.is_empty()) {
                if !reply.is_empty() {
                    let _ = send.send_data(reply, true);
                }
            }
        });
    }
}

/// A TcpStream with already-read bytes in front (the protocol sniff).
struct Prefixed {
    first: Vec<u8>,
    at: usize,
    stream: TcpStream,
}
impl tokio::io::AsyncRead for Prefixed {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        if self.at < self.first.len() {
            let n = (self.first.len() - self.at).min(buf.remaining());
            let at = self.at;
            buf.put_slice(&self.first[at..at + n]);
            self.at += n;
            return std::task::Poll::Ready(Ok(()));
        }
        std::pin::Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}
impl tokio::io::AsyncWrite for Prefixed {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        std::pin::Pin::new(&mut self.stream).poll_write(cx, buf)
    }
    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.stream).poll_flush(cx)
    }
    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}

async fn connection(mut stream: TcpStream, table: Arc<Table>) {
    let mut first = vec![0u8; 24];
    let mut have = 0;
    while have < 24 {
        match stream.read(&mut first[have..]).await {
            Ok(0) | Err(_) => return,
            Ok(n) => have += n,
        }
        if !b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n".starts_with(&first[..have.min(24)]) {
            break;
        }
    }
    first.truncate(have);
    if first.as_slice() == b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n" {
        http2(stream, table, first).await
    } else {
        http1(stream, table, first).await
    }
}

pub fn run(map: &HashMap<String, String>) -> Result<(), String> {
    let mode = map.get("mode").map_or("constant", String::as_str);
    let seconds: f64 = map
        .get("seconds")
        .map_or(Ok(600.0), |v| v.parse())
        .map_err(|_| "invalid --seconds")?;
    let threads: usize = map
        .get("threads")
        .map_or(Ok(2), |v| v.parse())
        .map_err(|_| "invalid --threads")?;
    if !(1.0..=3600.0).contains(&seconds) || !(1..=64).contains(&threads) {
        return Err("serve: --seconds 1-3600, --threads 1-64".into());
    }
    let mut table = Table {
        constant: None,
        replay: HashMap::new(),
        served: AtomicU64::new(0),
        missed: AtomicU64::new(0),
    };
    match mode {
        "constant" => {
            let size: usize = map
                .get("body-bytes")
                .map_or(Ok(0), |v| v.parse())
                .map_err(|_| "invalid --body-bytes")?;
            let body = format!("{{\"ok\":true,\"padding\":\"{}\"}}", "x".repeat(size));
            table.constant = Some(Reply {
                status: 200,
                content_type: "application/json",
                body: Bytes::from(body),
            });
        }
        "replay" => {
            let corpus: Value = serde_json::from_slice(
                &std::fs::read(map.get("corpus").ok_or("--corpus= required")?)
                    .map_err(|_| "cannot read corpus")?,
            )
            .map_err(|_| "invalid corpus")?;
            for entry in corpus.as_array().ok_or("corpus must be an array")? {
                let s = |k: &str| entry.get(k).and_then(Value::as_str).unwrap_or("");
                let body = match entry.get("responseBase64").and_then(Value::as_str) {
                    Some(b64) => crate::base64_decode(b64).ok_or("bad base64")?,
                    None => s("response").as_bytes().to_vec(),
                };
                let content_type = if s("contentType").starts_with("application/json") {
                    "application/json"
                } else {
                    "application/octet-stream"
                };
                table.replay.insert(
                    key(
                        s("method"),
                        s("path"),
                        s("cookie"),
                        s("requestBody").as_bytes(),
                    ),
                    Reply {
                        status: entry.get("status").and_then(Value::as_u64).unwrap_or(200) as u16,
                        content_type,
                        body: Bytes::from(body),
                    },
                );
            }
        }
        _ => return Err("serve: --mode constant or replay".into()),
    }
    let table = Arc::new(table);
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(threads)
        .enable_all()
        .build()
        .unwrap();
    let port_file = map.get("port-file").cloned();
    let stats_file = map.get("stats-file").cloned();
    runtime.block_on(async move {
        let listener = TcpListener::bind("127.0.0.1:0").await.map_err(|_| "bind failed")?;
        let port = listener.local_addr().unwrap().port();
        match &port_file {
            Some(path) => crate::write_private(path, port.to_string().as_bytes())?,
            None => println!("{port}"),
        }
        let accept = {
            let table = table.clone();
            async move {
                loop {
                    if let Ok((stream, _)) = listener.accept().await {
                        tokio::spawn(connection(stream, table.clone()));
                    }
                }
            }
        };
        let _ = tokio::time::timeout(Duration::from_secs_f64(seconds), accept).await;
        if let Some(path) = &stats_file {
            let cpu = crate::sys::cpu_micros();
            let stats = serde_json::json!({"served": table.served.load(Ordering::Relaxed), "missed": table.missed.load(Ordering::Relaxed), "cpuUserMs": cpu.0 as f64 / 1000.0, "cpuSystemMs": cpu.1 as f64 / 1000.0});
            crate::write_private(path, stats.to_string().as_bytes())?;
        }
        Ok::<(), String>(())
    })
}
