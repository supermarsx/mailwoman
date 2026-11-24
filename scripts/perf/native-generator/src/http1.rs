//! HTTP/1.1 keep-alive client on a plain TcpStream: one write of the prebuilt
//! request, then the status line, headers and a Content-Length, chunked or
//! close-delimited body. Bodies are decoded (gzip/deflate) like fetch decodes them.
use std::io;
use std::net::SocketAddr;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct Limits {
    /// abort above this many decoded bytes (`response too large`, 8 MiB)
    pub total: usize,
    /// keep the body for validation; abort above `keep_limit`
    pub keep: bool,
    pub keep_limit: usize,
}

pub struct Response {
    pub status: u16,
    pub encoding: Option<String>,
    pub content_length: Option<u64>,
    /// decoded body bytes consumed (counted even when not kept)
    pub bytes: u64,
    /// on-wire body bytes (after transfer coding, before content coding)
    #[allow(dead_code)]
    pub wire: u64,
    pub body: Vec<u8>,
    /// None when a 503 body exceeded 4096 bytes
    pub body_503: Option<Vec<u8>>,
}

pub enum Failure {
    /// no usable response (connect, reset, malformed): status `network`
    Network,
    /// a response arrived but its body failed or exceeded a bound; status stays
    Body { status: u16, bytes: u64 },
}

pub struct Conn {
    stream: TcpStream,
    buf: Vec<u8>,
    start: usize,
    end: usize,
    pub write: Vec<u8>,
}

impl Conn {
    pub async fn connect(addr: SocketAddr) -> io::Result<Conn> {
        let stream = TcpStream::connect(addr).await?;
        stream.set_nodelay(true)?;
        Ok(Conn {
            stream,
            buf: vec![0; 64 * 1024],
            start: 0,
            end: 0,
            write: Vec::with_capacity(4096),
        })
    }

    async fn fill(&mut self) -> io::Result<usize> {
        if self.start == self.end {
            self.start = 0;
            self.end = 0;
        } else if self.end == self.buf.len() {
            if self.start > 0 {
                self.buf.copy_within(self.start..self.end, 0);
                self.end -= self.start;
                self.start = 0;
            } else {
                self.buf.resize(self.buf.len() * 2, 0);
            }
        }
        let n = self.stream.read(&mut self.buf[self.end..]).await?;
        self.end += n;
        Ok(n)
    }

    /// Sends `self.write` and reads one response. Ok((response, reusable)).
    pub async fn exchange(
        &mut self,
        head_request: bool,
        limits: &Limits,
    ) -> Result<(Response, bool), Failure> {
        self.stream
            .write_all(&self.write)
            .await
            .map_err(|_| Failure::Network)?;
        // status line + headers
        let (status, keep_alive, length, chunked, encoding, head_len) = loop {
            if self.end > self.start {
                let mut headers = [httparse::EMPTY_HEADER; 64];
                let mut parsed = httparse::Response::new(&mut headers);
                match parsed.parse(&self.buf[self.start..self.end]) {
                    Ok(httparse::Status::Complete(len)) => {
                        let status = parsed.code.unwrap_or(0);
                        let mut keep = parsed.version == Some(1);
                        let (mut length, mut chunked, mut encoding) = (None, false, None);
                        for h in parsed.headers.iter() {
                            let value = std::str::from_utf8(h.value).unwrap_or("").trim();
                            if h.name.eq_ignore_ascii_case("content-length") {
                                length = Some(value.parse::<u64>().map_err(|_| Failure::Network)?);
                            } else if h.name.eq_ignore_ascii_case("transfer-encoding") {
                                chunked = value
                                    .to_ascii_lowercase()
                                    .split(',')
                                    .any(|t| t.trim() == "chunked");
                            } else if h.name.eq_ignore_ascii_case("connection") {
                                let v = value.to_ascii_lowercase();
                                if v.contains("close") {
                                    keep = false;
                                } else if v.contains("keep-alive") {
                                    keep = true;
                                }
                            } else if h.name.eq_ignore_ascii_case("content-encoding") {
                                encoding = Some(value.to_owned());
                            }
                        }
                        if (100..200).contains(&status) {
                            // interim response: skip it
                            self.start += len;
                            continue;
                        }
                        break (status, keep, length, chunked, encoding, len);
                    }
                    Ok(httparse::Status::Partial) => {}
                    Err(_) => return Err(Failure::Network),
                }
            }
            if self.fill().await.map_err(|_| Failure::Network)? == 0 {
                return Err(Failure::Network);
            }
        };
        self.start += head_len;
        let no_body = head_request || status == 204 || status == 304;
        let mut wire: Vec<u8> = Vec::new();
        let coded = encoding
            .as_deref()
            .is_some_and(|e| !e.is_empty() && !e.eq_ignore_ascii_case("identity"));
        let mut sink = Sink::new(limits, status, coded);
        let mut reusable = keep_alive;
        if !no_body {
            if chunked {
                loop {
                    // chunk-size line
                    let line_end = loop {
                        if let Some(at) = self.buf[self.start..self.end]
                            .windows(2)
                            .position(|w| w == b"\r\n")
                        {
                            break at;
                        }
                        if self.fill().await.map_err(|_| sink.failure())? == 0 {
                            return Err(sink.failure());
                        }
                    };
                    let line = std::str::from_utf8(&self.buf[self.start..self.start + line_end])
                        .map_err(|_| sink.failure())?;
                    let size =
                        usize::from_str_radix(line.split(';').next().unwrap_or("").trim(), 16)
                            .map_err(|_| sink.failure())?;
                    self.start += line_end + 2;
                    if size == 0 {
                        // trailers until an empty line
                        loop {
                            let at = loop {
                                if let Some(at) = self.buf[self.start..self.end]
                                    .windows(2)
                                    .position(|w| w == b"\r\n")
                                {
                                    break at;
                                }
                                if self.fill().await.map_err(|_| sink.failure())? == 0 {
                                    return Err(sink.failure());
                                }
                            };
                            self.start += at + 2;
                            if at == 0 {
                                break;
                            }
                        }
                        break;
                    }
                    let mut left = size;
                    while left > 0 {
                        if self.start == self.end
                            && self.fill().await.map_err(|_| sink.failure())? == 0
                        {
                            return Err(sink.failure());
                        }
                        let take = left.min(self.end - self.start);
                        sink.push(&self.buf[self.start..self.start + take], &mut wire)?;
                        self.start += take;
                        left -= take;
                    }
                    while self.end - self.start < 2 {
                        if self.fill().await.map_err(|_| sink.failure())? == 0 {
                            return Err(sink.failure());
                        }
                    }
                    self.start += 2;
                }
            } else if let Some(length) = length {
                let mut left = length as usize;
                while left > 0 {
                    if self.start == self.end && self.fill().await.map_err(|_| sink.failure())? == 0
                    {
                        return Err(sink.failure());
                    }
                    let take = left.min(self.end - self.start);
                    sink.push(&self.buf[self.start..self.start + take], &mut wire)?;
                    self.start += take;
                    left -= take;
                }
            } else {
                // close-delimited
                reusable = false;
                loop {
                    if self.start < self.end {
                        let (s, e) = (self.start, self.end);
                        sink.push(&self.buf[s..e], &mut wire)?;
                        self.start = e;
                    }
                    match self.fill().await {
                        Ok(0) => break,
                        Ok(_) => {}
                        Err(_) => return Err(sink.failure()),
                    }
                }
            }
        }
        if self.start != self.end {
            reusable = false; // unexpected bytes after the response
        }
        let response = sink.finish(wire, encoding, length)?;
        Ok((response, reusable))
    }
}

/// Body accounting with fetch's bounds. Coded bodies are buffered on the wire and decoded at the end.
struct Sink<'a> {
    limits: &'a Limits,
    status: u16,
    coded: bool,
    bytes: u64,
    wire_bytes: u64,
    body: Vec<u8>,
    body_503: Option<Vec<u8>>,
}
impl<'a> Sink<'a> {
    fn new(limits: &'a Limits, status: u16, coded: bool) -> Self {
        Sink {
            limits,
            status,
            coded,
            bytes: 0,
            wire_bytes: 0,
            body: Vec::new(),
            body_503: if status == 503 {
                Some(Vec::new())
            } else {
                None
            },
        }
    }
    fn failure(&self) -> Failure {
        Failure::Body {
            status: self.status,
            bytes: self.bytes,
        }
    }
    fn decoded(&mut self, part: &[u8]) -> Result<(), Failure> {
        self.bytes += part.len() as u64;
        if let Some(b) = &mut self.body_503 {
            if self.bytes <= 4096 {
                b.extend_from_slice(part);
            } else {
                self.body_503 = None;
            }
        }
        if self.bytes > self.limits.total as u64 {
            return Err(self.failure());
        }
        if self.limits.keep {
            if self.bytes > self.limits.keep_limit as u64 {
                return Err(self.failure());
            }
            self.body.extend_from_slice(part);
        }
        Ok(())
    }
    fn push(&mut self, part: &[u8], wire: &mut Vec<u8>) -> Result<(), Failure> {
        self.wire_bytes += part.len() as u64;
        if self.coded {
            if wire.len() + part.len() > self.limits.total {
                return Err(self.failure());
            }
            wire.extend_from_slice(part);
            Ok(())
        } else {
            self.decoded(part)
        }
    }
    fn finish(
        mut self,
        wire: Vec<u8>,
        encoding: Option<String>,
        length: Option<u64>,
    ) -> Result<Response, Failure> {
        if self.coded {
            let decoded =
                decode(encoding.as_deref().unwrap_or(""), &wire).ok_or(Failure::Body {
                    status: self.status,
                    bytes: 0,
                })?;
            for chunk in decoded.chunks(16 * 1024) {
                self.decoded(chunk)?;
            }
        }
        Ok(Response {
            status: self.status,
            encoding,
            content_length: length,
            bytes: self.bytes,
            wire: self.wire_bytes,
            body: self.body,
            body_503: self.body_503,
        })
    }
}

pub fn decode(encoding: &str, wire: &[u8]) -> Option<Vec<u8>> {
    use std::io::Read;
    let mut out = Vec::new();
    match encoding.trim().to_ascii_lowercase().as_str() {
        "gzip" | "x-gzip" => flate2::read::MultiGzDecoder::new(wire)
            .read_to_end(&mut out)
            .ok()?,
        "deflate" => flate2::read::ZlibDecoder::new(wire)
            .read_to_end(&mut out)
            .ok()?,
        "identity" | "" => return Some(wire.to_vec()),
        _ => return None,
    };
    Some(out)
}

/// Idle keep-alive connections of one target (LIFO, as a connection pool reuses its warmest socket).
pub struct Pool {
    pub addr: SocketAddr,
    idle: Mutex<Vec<Conn>>,
    pub opened: AtomicU64,
    pub reused: AtomicU64,
}
impl Pool {
    pub fn new(addr: SocketAddr) -> Pool {
        Pool {
            addr,
            idle: Mutex::new(Vec::new()),
            opened: AtomicU64::new(0),
            reused: AtomicU64::new(0),
        }
    }
    pub async fn take(&self) -> io::Result<Conn> {
        if let Some(conn) = self.idle.lock().unwrap().pop() {
            self.reused.fetch_add(1, Ordering::Relaxed);
            return Ok(conn);
        }
        self.opened.fetch_add(1, Ordering::Relaxed);
        Conn::connect(self.addr).await
    }
    pub fn give(&self, conn: Conn) {
        self.idle.lock().unwrap().push(conn);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tokio::net::TcpListener;

    async fn one(response: Vec<u8>, limits: Limits) -> Result<(Response, bool), Failure> {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut s, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 1024];
            let _ = s.read(&mut buf).await;
            s.write_all(&response).await.unwrap();
        });
        let mut conn = Conn::connect(addr).await.unwrap();
        conn.write = b"GET / HTTP/1.1\r\nhost: x\r\n\r\n".to_vec();
        conn.exchange(false, &limits).await
    }
    fn keep() -> Limits {
        Limits {
            total: 8 << 20,
            keep: true,
            keep_limit: 65536,
        }
    }

    #[tokio::test]
    async fn chunked_gzip_and_bounds() {
        let (r, reusable) = one(b"HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\n\r\n5\r\nhello\r\n6;x=1\r\n world\r\n0\r\n\r\n".to_vec(), keep()).await.ok().unwrap();
        assert_eq!(
            (r.status, r.body.as_slice(), reusable),
            (200, &b"hello world"[..], true)
        );
        let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        gz.write_all(b"{\"ok\":true}").unwrap();
        let gz = gz.finish().unwrap();
        let mut response = format!(
            "HTTP/1.1 200 OK\r\ncontent-encoding: gzip\r\ncontent-length: {}\r\n\r\n",
            gz.len()
        )
        .into_bytes();
        response.extend_from_slice(&gz);
        let (r, _) = one(response, keep()).await.ok().unwrap();
        assert_eq!(
            (r.body.as_slice(), r.bytes, r.content_length),
            (&b"{\"ok\":true}"[..], 11, Some(gz.len() as u64))
        );
        let big = format!(
            "HTTP/1.1 200 OK\r\ncontent-length: 70000\r\n\r\n{}",
            "x".repeat(70000)
        )
        .into_bytes();
        assert!(matches!(
            one(big, keep()).await,
            Err(Failure::Body { status: 200, .. })
        ));
        let (r, reusable) = one(
            b"HTTP/1.1 503 Service Unavailable\r\nconnection: close\r\ncontent-length: 2\r\n\r\n{}"
                .to_vec(),
            keep(),
        )
        .await
        .ok()
        .unwrap();
        assert_eq!(
            (r.status, r.body_503.as_deref(), reusable),
            (503, Some(&b"{}"[..]), false)
        );
    }
}
