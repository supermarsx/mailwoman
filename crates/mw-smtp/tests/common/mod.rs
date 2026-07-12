//! Test-only mock SMTP server: a `tokio` `TcpListener` that replays a scripted
//! sequence of server reply blocks and captures every line the client sent, so
//! the transcript fixtures in `fixtures/smtp/` can drive the real `Submitter`
//! over a cleartext socket (plan §3 e4 acceptance).

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

/// A running mock server bound to an ephemeral port.
pub struct Mock {
    pub addr: SocketAddr,
    captured: Arc<Mutex<Vec<String>>>,
    handle: JoinHandle<()>,
}

impl Mock {
    /// Every client-sent line captured so far (commands and DATA body lines,
    /// CRLF stripped).
    pub fn captured(&self) -> Vec<String> {
        self.captured.lock().unwrap().clone()
    }

    /// Wait for the server task to finish handling the connection.
    pub async fn join(self) {
        let _ = self.handle.await;
    }
}

/// Start a mock server that replays `script` — a list of server reply blocks,
/// block 0 being the opening greeting. After the greeting, each block is sent in
/// response to one client command line; a block whose code is `354` additionally
/// swallows the DATA body up to the lone `.` before sending the following block.
pub async fn start(script: Vec<String>) -> Mock {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let captured = Arc::new(Mutex::new(Vec::new()));
    let cap = captured.clone();
    let handle = tokio::spawn(async move {
        if let Ok((mut sock, _)) = listener.accept().await {
            run(&mut sock, script, cap).await;
        }
    });
    Mock {
        addr,
        captured,
        handle,
    }
}

async fn read_line(sock: &mut TcpStream, buf: &mut Vec<u8>) -> Option<String> {
    loop {
        if let Some(pos) = buf.iter().position(|&b| b == b'\n') {
            let mut line: Vec<u8> = buf.drain(..=pos).collect();
            line.pop();
            if line.last() == Some(&b'\r') {
                line.pop();
            }
            return Some(String::from_utf8_lossy(&line).into_owned());
        }
        let mut tmp = [0u8; 1024];
        let n = sock.read(&mut tmp).await.ok()?;
        if n == 0 {
            return None;
        }
        buf.extend_from_slice(&tmp[..n]);
    }
}

async fn run(sock: &mut TcpStream, script: Vec<String>, cap: Arc<Mutex<Vec<String>>>) {
    if script.is_empty() {
        return;
    }
    let mut buf = Vec::new();
    // Block 0 is the unsolicited greeting.
    if sock.write_all(script[0].as_bytes()).await.is_err() {
        return;
    }
    let mut i = 1;
    while i < script.len() {
        let Some(line) = read_line(sock, &mut buf).await else {
            return;
        };
        cap.lock().unwrap().push(line);
        let reply = &script[i];
        i += 1;
        if sock.write_all(reply.as_bytes()).await.is_err() {
            return;
        }
        if reply.starts_with("354") {
            // Consume the dot-stuffed body up to the terminating ".".
            loop {
                let Some(l) = read_line(sock, &mut buf).await else {
                    return;
                };
                let done = l == ".";
                cap.lock().unwrap().push(l);
                if done {
                    break;
                }
            }
            if i < script.len() {
                if sock.write_all(script[i].as_bytes()).await.is_err() {
                    return;
                }
                i += 1;
            }
        }
    }
}

/// Load a transcript fixture into a script of reply blocks.
///
/// Format: lines prefixed `S: ` are server output; consecutive `S:` lines form
/// one reply block; a blank line separates blocks; `#` lines are comments. Each
/// server line is terminated with CRLF on the wire.
pub fn load_script(path: &str) -> Vec<String> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read fixture {path}: {e}"));
    let mut blocks = Vec::new();
    let mut current = String::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("S: ") {
            current.push_str(rest);
            current.push_str("\r\n");
        } else if let Some(rest) = line.strip_prefix("S:") {
            // `S:` with an empty payload (e.g. a bare continuation).
            current.push_str(rest);
            current.push_str("\r\n");
        } else if line.trim().is_empty() && !current.is_empty() {
            blocks.push(std::mem::take(&mut current));
        }
        // any other line (comments, `#`) is ignored
    }
    if !current.is_empty() {
        blocks.push(current);
    }
    blocks
}
