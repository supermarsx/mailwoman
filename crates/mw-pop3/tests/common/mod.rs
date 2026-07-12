//! A scripted in-process POP3 server for transcript-driven tests.
//!
//! A transcript is a recorded dialogue: lines prefixed `S: ` are sent to the
//! client, lines prefixed `C: ` mark a client command we expect to read (use
//! `C: *` to accept any single line, e.g. a base64 SASL blob). The server
//! records every client line it receives so tests can assert precisely which
//! commands were issued — crucial for the leave-on-server `DELE` branches.

#![allow(dead_code)]

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use mw_pop3::{LeavePolicy, Pop3Auth, Pop3Config, TlsMode};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

enum Event {
    Send(String),
    Expect(Option<String>),
}

fn parse_transcript(text: &str) -> Vec<Event> {
    let mut events = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("S: ") {
            events.push(Event::Send(rest.to_string()));
        } else if line == "S:" {
            events.push(Event::Send(String::new()));
        } else if let Some(rest) = line.strip_prefix("C: ") {
            let pat = if rest == "*" {
                None
            } else {
                Some(rest.to_string())
            };
            events.push(Event::Expect(pat));
        }
        // Anything else (blank lines, `#` comments) is ignored.
    }
    events
}

/// A running scripted server bound to an ephemeral loopback port.
pub struct MockServer {
    pub addr: SocketAddr,
    recorded: Arc<Mutex<Vec<String>>>,
    handle: JoinHandle<()>,
}

impl MockServer {
    /// Bind and start replaying `transcript` on the first accepted connection.
    pub async fn start(transcript: &str) -> Self {
        let events = parse_transcript(transcript);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let recorded = Arc::new(Mutex::new(Vec::new()));
        let rec = Arc::clone(&recorded);

        let handle = tokio::spawn(async move {
            let Ok((sock, _)) = listener.accept().await else {
                return;
            };
            let (rd, mut wr) = sock.into_split();
            let mut reader = BufReader::new(rd);
            for event in events {
                match event {
                    Event::Send(s) => {
                        let mut bytes = s.into_bytes();
                        bytes.extend_from_slice(b"\r\n");
                        if wr.write_all(&bytes).await.is_err() {
                            return;
                        }
                        let _ = wr.flush().await;
                    }
                    Event::Expect(pat) => {
                        let mut line = String::new();
                        let n = reader.read_line(&mut line).await.unwrap_or(0);
                        if n == 0 {
                            rec.lock().unwrap().push("<EOF>".to_string());
                            return;
                        }
                        let got = line.trim_end_matches(['\r', '\n']).to_string();
                        if let Some(expected) = pat
                            && expected != got
                        {
                            rec.lock()
                                .unwrap()
                                .push(format!("<MISMATCH want {expected:?} got {got:?}>"));
                        }
                        rec.lock().unwrap().push(got);
                    }
                }
            }
        });

        Self {
            addr,
            recorded,
            handle,
        }
    }

    /// Every client command line received so far.
    pub fn commands(&self) -> Vec<String> {
        self.recorded.lock().unwrap().clone()
    }

    /// Await the server task, surfacing any transcript mismatch it recorded.
    pub async fn finish(self) {
        let _ = self.handle.await;
        let cmds = self.recorded.lock().unwrap();
        for c in cmds.iter() {
            assert!(!c.starts_with("<MISMATCH"), "transcript mismatch: {c}");
        }
    }
}

/// Build a plaintext config pointed at a mock server.
pub fn mock_config(
    addr: SocketAddr,
    auth: Pop3Auth,
    username: &str,
    secret: &str,
    leave_policy: LeavePolicy,
) -> Pop3Config {
    Pop3Config {
        host: addr.ip().to_string(),
        port: addr.port(),
        tls: TlsMode::Plain,
        auth,
        username: username.to_string(),
        secret: secret.to_string(),
        leave_policy,
        poll_interval: Duration::from_millis(50),
    }
}
