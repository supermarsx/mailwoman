//! [`Poppassd`] — change a password via the poppassd line protocol over TCP.
//!
//! The dialogue (each server reply is a `NNN text` line; `2xx` = success):
//! ```text
//! S: 200 hello
//! C: user <username>      S: 200 ...
//! C: pass <oldpassword>   S: 200 ...   (non-2xx here ⇒ wrong current password)
//! C: newpass <newpassword> S: 200 ...  (non-2xx here ⇒ new password rejected)
//! C: quit                 S: 200 bye
//! ```
//! The protocol logic lives in [`run_dialogue`] over a [`LineTransport`] seam, so it is
//! tested with a scripted in-memory transport; [`TcpLineTransport`] is the real socket.

use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use crate::{
    BackendKind, Ctx, PasswordChangeBackend, PasswordChangeOutcome, PasswordError, PasswordPolicy,
    Result, Secret,
};

/// A CRLF line transport: send a command line, read a reply line.
#[async_trait]
pub trait LineTransport: Send {
    /// Read one reply line (without the trailing CRLF).
    async fn read_line(&mut self) -> Result<String>;
    /// Write one command line (CRLF is appended).
    async fn write_line(&mut self, line: &str) -> Result<()>;
}

/// True when a poppassd reply line indicates success (a `2xx` status code).
fn is_ok(reply: &str) -> bool {
    reply.trim_start().starts_with('2')
}

/// Drive the poppassd change dialogue over any [`LineTransport`].
///
/// Secrets are written to the wire (the protocol requires it) but never returned or
/// logged; the returned errors carry only server status text, no password material.
pub async fn run_dialogue<T: LineTransport + ?Sized>(
    t: &mut T,
    username: &str,
    old: &str,
    new: &str,
) -> Result<()> {
    // Greeting.
    let greet = t.read_line().await?;
    if !is_ok(&greet) {
        return Err(PasswordError::Protocol(format!("greeting: {greet}")));
    }
    t.write_line(&format!("user {username}")).await?;
    let r = t.read_line().await?;
    if !is_ok(&r) {
        return Err(PasswordError::Protocol(format!("user: {r}")));
    }
    t.write_line(&format!("pass {old}")).await?;
    let r = t.read_line().await?;
    if !is_ok(&r) {
        // poppassd rejects the current password here.
        return Err(PasswordError::WrongCurrent);
    }
    t.write_line(&format!("newpass {new}")).await?;
    let r = t.read_line().await?;
    if !is_ok(&r) {
        return Err(PasswordError::Protocol(format!("newpass rejected: {r}")));
    }
    t.write_line("quit").await?;
    // Best-effort read of the closing "bye"; the change already succeeded.
    let _ = t.read_line().await;
    Ok(())
}

/// Config for the poppassd backend.
#[derive(Debug, Clone)]
pub struct PoppassdConfig {
    pub host: String,
    pub port: u16,
    pub policy: PasswordPolicy,
}

impl PoppassdConfig {
    #[must_use]
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            policy: PasswordPolicy::default(),
        }
    }
}

/// A real poppassd TCP connection (buffered CRLF lines).
pub struct TcpLineTransport {
    inner: BufReader<TcpStream>,
}

impl TcpLineTransport {
    /// Connect to a poppassd server.
    pub async fn connect(host: &str, port: u16) -> Result<Self> {
        let stream = TcpStream::connect((host, port))
            .await
            .map_err(|e| PasswordError::Transport(e.to_string()))?;
        Ok(Self {
            inner: BufReader::new(stream),
        })
    }
}

#[async_trait]
impl LineTransport for TcpLineTransport {
    async fn read_line(&mut self) -> Result<String> {
        let mut line = String::new();
        let n = self
            .inner
            .read_line(&mut line)
            .await
            .map_err(|e| PasswordError::Transport(e.to_string()))?;
        if n == 0 {
            return Err(PasswordError::Transport("connection closed".into()));
        }
        Ok(line.trim_end_matches(['\r', '\n']).to_string())
    }

    async fn write_line(&mut self, line: &str) -> Result<()> {
        let stream = self.inner.get_mut();
        stream
            .write_all(line.as_bytes())
            .await
            .map_err(|e| PasswordError::Transport(e.to_string()))?;
        stream
            .write_all(b"\r\n")
            .await
            .map_err(|e| PasswordError::Transport(e.to_string()))?;
        stream
            .flush()
            .await
            .map_err(|e| PasswordError::Transport(e.to_string()))?;
        Ok(())
    }
}

/// poppassd protocol password change (plan §2.3).
pub struct Poppassd {
    config: PoppassdConfig,
}

impl Poppassd {
    #[must_use]
    pub fn new(config: PoppassdConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl PasswordChangeBackend for Poppassd {
    async fn change(&self, ctx: &Ctx, old: Secret, new: Secret) -> Result<PasswordChangeOutcome> {
        self.config.policy.validate(&new)?;
        let mut transport = TcpLineTransport::connect(&self.config.host, self.config.port).await?;
        run_dialogue(&mut transport, &ctx.username, old.expose(), new.expose()).await?;
        Ok(PasswordChangeOutcome::changed_from(ctx))
    }

    fn policy(&self) -> PasswordPolicy {
        self.config.policy.clone()
    }

    fn kind(&self) -> BackendKind {
        BackendKind::Poppassd
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    /// A scripted transport: canned server replies + a record of client-sent lines.
    struct ScriptedTransport {
        replies: VecDeque<String>,
        sent: Vec<String>,
    }
    impl ScriptedTransport {
        fn new(replies: &[&str]) -> Self {
            Self {
                replies: replies.iter().map(|s| (*s).to_string()).collect(),
                sent: Vec::new(),
            }
        }
    }
    #[async_trait]
    impl LineTransport for ScriptedTransport {
        async fn read_line(&mut self) -> Result<String> {
            self.replies
                .pop_front()
                .ok_or_else(|| PasswordError::Transport("no more replies".into()))
        }
        async fn write_line(&mut self, line: &str) -> Result<()> {
            self.sent.push(line.to_string());
            Ok(())
        }
    }

    #[tokio::test]
    async fn happy_path_full_dialogue() {
        let mut t = ScriptedTransport::new(&[
            "200 hello",
            "200 user ok",
            "200 pass ok",
            "200 password changed",
            "200 bye",
        ]);
        run_dialogue(&mut t, "alice", "oldpw", "new-strong-pw")
            .await
            .unwrap();
        assert_eq!(
            t.sent,
            vec![
                "user alice".to_string(),
                "pass oldpw".to_string(),
                "newpass new-strong-pw".to_string(),
                "quit".to_string(),
            ]
        );
    }

    #[tokio::test]
    async fn deny_path_wrong_current_password() {
        let mut t = ScriptedTransport::new(&["200 hello", "200 user ok", "500 auth failed"]);
        let err = run_dialogue(&mut t, "alice", "wrong", "new-strong-pw")
            .await
            .unwrap_err();
        assert!(matches!(err, PasswordError::WrongCurrent));
        // We never sent the new password after the failed auth.
        assert!(!t.sent.iter().any(|l| l.starts_with("newpass")));
    }

    #[tokio::test]
    async fn deny_path_new_password_rejected_by_server() {
        let mut t = ScriptedTransport::new(&[
            "200 hello",
            "200 user ok",
            "200 pass ok",
            "500 password too weak",
        ]);
        let err = run_dialogue(&mut t, "alice", "oldpw", "weak")
            .await
            .unwrap_err();
        assert!(matches!(err, PasswordError::Protocol(_)));
    }
}
