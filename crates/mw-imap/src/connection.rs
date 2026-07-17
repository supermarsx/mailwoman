//! The tagged command engine: unique tags, response demux, literal / SASL
//! continuation handling, and the streaming `imap-proto` read loop.
//!
//! One [`Connection`] owns a single [`ImapStream`] and a read buffer. Responses
//! are parsed incrementally: bytes are accumulated until `imap_proto` reports a
//! complete response, so literals (`{n}`) and multi-line responses are framed
//! by the parser rather than by hand.

use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use bytes::{Buf, BytesMut};
use imap_proto::{RequestId, Response, ResponseCode, Status};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::error::{ImapError, ImapResult};
use crate::sasl::SaslClient;
use crate::transport::{ImapStream, TlsMode};

/// A fully-owned parsed response (no borrow into the read buffer).
pub type OwnedResponse = Response<'static>;

/// Outcome of a tagged command: any untagged responses plus the tagged result.
#[derive(Debug)]
pub struct Tagged {
    pub untagged: Vec<OwnedResponse>,
    pub status: Status,
    pub code: Option<ResponseCode<'static>>,
    pub information: Option<String>,
}

impl Tagged {
    /// Map a non-OK tagged completion onto the appropriate [`ImapError`].
    ///
    /// `OK` returns `self`; `NO`/`BAD`/`BYE` become errors carrying the server
    /// text so `mw-engine` sees a meaningful message.
    pub fn ok(self) -> ImapResult<Self> {
        match self.status {
            Status::Ok | Status::PreAuth => Ok(self),
            Status::No => Err(ImapError::No(self.information.unwrap_or_default())),
            Status::Bad => Err(ImapError::Bad(self.information.unwrap_or_default())),
            Status::Bye => Err(ImapError::Bye(self.information.unwrap_or_default())),
        }
    }
}

enum ParseStep {
    Done(usize, OwnedResponse),
    Incomplete,
    Bad(String),
}

/// Classify one parse attempt without leaking any borrow of `buf`.
fn try_parse(buf: &[u8]) -> ParseStep {
    if buf.is_empty() {
        return ParseStep::Incomplete;
    }
    match imap_proto::Response::from_bytes(buf) {
        Ok((remaining, resp)) => {
            let consumed = buf.len() - remaining.len();
            ParseStep::Done(consumed, resp.into_owned())
        }
        Err(nom::Err::Incomplete(_)) => ParseStep::Incomplete,
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
            let near = &e.input[..e.input.len().min(48)];
            ParseStep::Bad(format!(
                "unparseable response near {:?}",
                String::from_utf8_lossy(near)
            ))
        }
    }
}

/// A single IMAP connection with its command tag counter and read buffer.
pub struct Connection {
    stream: Option<ImapStream>,
    buf: BytesMut,
    tag_seq: u32,
    host: String,
}

impl Connection {
    /// Dial `host:port` with the given TLS mode and read the server greeting.
    ///
    /// Returns the connection and the greeting's untagged response so the caller
    /// can harvest any pre-auth `[CAPABILITY ...]` advertisement.
    pub async fn connect(
        host: &str,
        port: u16,
        mode: TlsMode,
    ) -> ImapResult<(Self, OwnedResponse)> {
        let stream = ImapStream::connect(host, port, mode).await?;
        let mut conn = Connection {
            stream: Some(stream),
            buf: BytesMut::with_capacity(8 * 1024),
            tag_seq: 0,
            host: host.to_string(),
        };
        let greeting = conn.read_response().await?;
        match &greeting {
            Response::Data {
                status: Status::Ok | Status::PreAuth,
                ..
            } => Ok((conn, greeting)),
            Response::Data {
                status: Status::Bye,
                information,
                ..
            } => Err(ImapError::Bye(
                information
                    .as_deref()
                    .unwrap_or("server refused connection")
                    .to_string(),
            )),
            other => Err(ImapError::Protocol(format!(
                "unexpected greeting: {other:?}"
            ))),
        }
    }

    /// Whether the transport is currently TLS-encrypted.
    pub fn is_encrypted(&self) -> bool {
        self.stream.as_ref().is_some_and(ImapStream::is_encrypted)
    }

    fn next_tag(&mut self) -> String {
        self.tag_seq = self.tag_seq.wrapping_add(1);
        format!("A{:04}", self.tag_seq)
    }

    async fn write_all(&mut self, bytes: &[u8]) -> ImapResult<()> {
        let stream = self.stream.as_mut().ok_or(ImapError::Eof)?;
        stream.write_all(bytes).await?;
        stream.flush().await?;
        Ok(())
    }

    /// Read exactly one complete response, growing the buffer as needed.
    pub async fn read_response(&mut self) -> ImapResult<OwnedResponse> {
        loop {
            match try_parse(&self.buf) {
                ParseStep::Done(consumed, resp) => {
                    self.buf.advance(consumed);
                    return Ok(resp);
                }
                ParseStep::Bad(msg) => return Err(ImapError::Protocol(msg)),
                ParseStep::Incomplete => {}
            }
            let stream = self.stream.as_mut().ok_or(ImapError::Eof)?;
            let n = stream.read_buf(&mut self.buf).await?;
            if n == 0 {
                return Err(ImapError::Eof);
            }
        }
    }

    /// Send a tagged command and collect responses until its tagged completion.
    pub async fn execute(&mut self, command: &str) -> ImapResult<Tagged> {
        let tag = self.next_tag();
        tracing::trace!(target: "mw_imap::wire", tag = %tag, "C: {command}");
        self.write_all(format!("{tag} {command}\r\n").as_bytes())
            .await?;
        self.collect_until_tagged(&tag).await
    }

    async fn collect_until_tagged(&mut self, tag: &str) -> ImapResult<Tagged> {
        let mut untagged = Vec::new();
        loop {
            let resp = self.read_response().await?;
            match resp {
                Response::Done {
                    tag: RequestId(t),
                    status,
                    code,
                    information,
                } => {
                    if t == tag {
                        return Ok(Tagged {
                            untagged,
                            status,
                            code,
                            information: information.map(|c| c.into_owned()),
                        });
                    }
                    return Err(ImapError::Protocol(format!(
                        "tag mismatch: expected {tag}, got {t}"
                    )));
                }
                Response::Continue { .. } => {
                    return Err(ImapError::Protocol(
                        "unexpected continuation for a non-literal command".into(),
                    ));
                }
                other => untagged.push(other),
            }
        }
    }

    /// Drive a SASL `AUTHENTICATE` exchange.
    ///
    /// `responses` are the base64 payloads to emit at each server continuation,
    /// in order (one for PLAIN/XOAUTH2, two for LOGIN). On an unexpected extra
    /// continuation (e.g. a Gmail XOAUTH2 error challenge) an empty line is sent
    /// to let the server finish with a tagged `NO`.
    pub async fn authenticate(
        &mut self,
        mechanism: &str,
        responses: &[String],
    ) -> ImapResult<Tagged> {
        let tag = self.next_tag();
        tracing::trace!(target: "mw_imap::wire", tag = %tag, "C: AUTHENTICATE {mechanism}");
        self.write_all(format!("{tag} AUTHENTICATE {mechanism}\r\n").as_bytes())
            .await?;

        let mut untagged = Vec::new();
        let mut idx = 0usize;
        loop {
            let resp = self.read_response().await?;
            match resp {
                Response::Continue { .. } => {
                    let payload = responses.get(idx).cloned().unwrap_or_default();
                    idx += 1;
                    self.write_all(format!("{payload}\r\n").as_bytes()).await?;
                }
                Response::Done {
                    tag: RequestId(t),
                    status,
                    code,
                    information,
                } => {
                    if t != tag {
                        return Err(ImapError::Protocol(format!(
                            "tag mismatch: expected {tag}, got {t}"
                        )));
                    }
                    return Ok(Tagged {
                        untagged,
                        status,
                        code,
                        information: information.map(|c| c.into_owned()),
                    });
                }
                other => untagged.push(other),
            }
        }
    }

    /// Drive an interactive SASL `AUTHENTICATE` exchange (SCRAM, …).
    ///
    /// One [`SaslClient::step`] per server continuation; the first step receives
    /// an empty challenge (the server's bare `+`), matching the no-`SASL-IR`
    /// choreography [`Self::authenticate`] uses. A `step` error aborts the
    /// exchange (`*`) and surfaces an [`ImapError::Auth`].
    pub async fn authenticate_sasl(
        &mut self,
        mechanism: &str,
        client: &mut (dyn SaslClient + Send),
    ) -> ImapResult<Tagged> {
        let tag = self.next_tag();
        tracing::trace!(target: "mw_imap::wire", tag = %tag, "C: AUTHENTICATE {mechanism}");
        self.write_all(format!("{tag} AUTHENTICATE {mechanism}\r\n").as_bytes())
            .await?;

        let mut untagged = Vec::new();
        loop {
            let resp = self.read_response().await?;
            match resp {
                Response::Continue { information, .. } => {
                    let challenge = match &information {
                        Some(text) => B64.decode(text.trim().as_bytes()).map_err(|e| {
                            ImapError::Protocol(format!("invalid base64 SASL challenge: {e}"))
                        })?,
                        None => Vec::new(),
                    };
                    match client.step(&challenge) {
                        Ok(response) => {
                            let encoded = B64.encode(&response);
                            self.write_all(format!("{encoded}\r\n").as_bytes()).await?;
                        }
                        Err(msg) => {
                            // Abort the exchange; drain to the tagged completion.
                            self.write_all(b"*\r\n").await?;
                            let _ = self.collect_until_tagged(&tag).await;
                            return Err(ImapError::Auth(msg));
                        }
                    }
                }
                Response::Done {
                    tag: RequestId(t),
                    status,
                    code,
                    information,
                } => {
                    if t != tag {
                        return Err(ImapError::Protocol(format!(
                            "tag mismatch: expected {tag}, got {t}"
                        )));
                    }
                    return Ok(Tagged {
                        untagged,
                        status,
                        code,
                        information: information.map(|c| c.into_owned()),
                    });
                }
                other => untagged.push(other),
            }
        }
    }

    /// The SCRAM `-PLUS` channel binding for the current transport:
    /// `tls-server-end-point` (RFC 5929) over the server's leaf certificate.
    /// The digest tracks the certificate's own signature hash
    /// (SHA-256/384/512); that selection lives in
    /// [`crate::sasl::tls_server_end_point`]. `None` on a plaintext transport or
    /// when the server presented no certificate.
    pub fn channel_binding(&self) -> Option<Vec<u8>> {
        match self.stream.as_ref()? {
            ImapStream::Tls(tls) => {
                let (_, conn) = tls.get_ref();
                let leaf = conn.peer_certificates()?.first()?;
                Some(crate::sasl::tls_server_end_point(leaf.as_ref()))
            }
            ImapStream::Plain(_) => None,
        }
    }

    /// Execute a command whose untagged responses this crate parses by hand —
    /// used for `THREAD`, which `imap-proto` does not model. Returns the raw
    /// untagged lines (CRLF stripped); errors on a `NO`/`BAD`/`BYE` completion.
    ///
    /// Safe only for commands that never return synchronizing literals (true of
    /// `THREAD`): the reply is a sequence of CRLF-terminated lines.
    pub async fn execute_lines(&mut self, command: &str) -> ImapResult<Vec<String>> {
        let tag = self.next_tag();
        tracing::trace!(target: "mw_imap::wire", tag = %tag, "C: {command}");
        self.write_all(format!("{tag} {command}\r\n").as_bytes())
            .await?;
        let completion = format!("{tag} ");
        let mut lines = Vec::new();
        loop {
            let line = self.read_line().await?;
            if let Some(rest) = line.strip_prefix(&completion) {
                let status = rest.split_whitespace().next().unwrap_or("");
                let info = rest[status.len()..].trim().to_string();
                return match status.to_ascii_uppercase().as_str() {
                    "OK" => Ok(lines),
                    "NO" => Err(ImapError::No(info)),
                    "BAD" => Err(ImapError::Bad(info)),
                    "BYE" => Err(ImapError::Bye(info)),
                    _ => Err(ImapError::Protocol(format!(
                        "unexpected completion: {line}"
                    ))),
                };
            }
            lines.push(line);
        }
    }

    /// Read one CRLF-terminated line (CRLF stripped), growing the buffer as
    /// needed. Used by [`Self::execute_lines`].
    async fn read_line(&mut self) -> ImapResult<String> {
        loop {
            if let Some(pos) = self.buf.windows(2).position(|w| w == b"\r\n") {
                let line = self.buf.split_to(pos);
                self.buf.advance(2); // consume the CRLF
                return Ok(String::from_utf8_lossy(&line).into_owned());
            }
            let stream = self.stream.as_mut().ok_or(ImapError::Eof)?;
            let n = stream.read_buf(&mut self.buf).await?;
            if n == 0 {
                return Err(ImapError::Eof);
            }
        }
    }

    /// Send a command whose final argument is a synchronizing literal.
    ///
    /// `head` is everything up to (but not including) the `{n}` (e.g.
    /// `APPEND "Sent" (\Seen)`); `data` is the literal payload.
    pub async fn execute_with_literal(&mut self, head: &str, data: &[u8]) -> ImapResult<Tagged> {
        let tag = self.next_tag();
        tracing::trace!(target: "mw_imap::wire", tag = %tag, "C: {head} {{{}}}", data.len());
        self.write_all(format!("{tag} {head} {{{}}}\r\n", data.len()).as_bytes())
            .await?;

        // Expect exactly one continuation, then stream the literal + CRLF.
        loop {
            let resp = self.read_response().await?;
            match resp {
                Response::Continue { .. } => break,
                Response::Done {
                    status,
                    information,
                    ..
                } => {
                    // Server rejected the literal outright (e.g. over quota).
                    return Tagged {
                        untagged: Vec::new(),
                        status,
                        code: None,
                        information: information.map(|c| c.into_owned()),
                    }
                    .ok();
                }
                _ => {}
            }
        }
        self.write_all(data).await?;
        self.write_all(b"\r\n").await?;
        self.collect_until_tagged(&tag).await
    }

    /// Begin an `IDLE`; returns once the server confirms with a continuation.
    pub async fn idle_start(&mut self) -> ImapResult<String> {
        let tag = self.next_tag();
        self.write_all(format!("{tag} IDLE\r\n").as_bytes()).await?;
        loop {
            match self.read_response().await? {
                Response::Continue { .. } => return Ok(tag),
                Response::Done {
                    status,
                    information,
                    ..
                } => {
                    // Server declined IDLE (shouldn't happen when advertised).
                    return Err(ImapError::No(format!(
                        "IDLE refused ({status:?}): {}",
                        information.unwrap_or_default()
                    )));
                }
                _ => {}
            }
        }
    }

    /// Read the next untagged response during an active IDLE (no timeout here;
    /// the caller races this against its stop signal / keepalive timer).
    pub async fn idle_next(&mut self) -> ImapResult<OwnedResponse> {
        self.read_response().await
    }

    /// End an `IDLE` by sending `DONE` and draining to the tagged completion.
    pub async fn idle_done(&mut self, tag: &str) -> ImapResult<Tagged> {
        self.write_all(b"DONE\r\n").await?;
        self.collect_until_tagged(tag).await
    }

    /// Perform the `STARTTLS` upgrade: issue the command, then wrap the socket.
    pub async fn starttls(&mut self) -> ImapResult<()> {
        self.execute("STARTTLS").await?.ok()?;
        let stream = self.stream.take().ok_or(ImapError::Eof)?;
        let host = self.host.clone();
        self.stream = Some(stream.upgrade(&host).await?);
        // Any buffered pre-TLS bytes would be a protocol violation; discard.
        self.buf.clear();
        Ok(())
    }
}
