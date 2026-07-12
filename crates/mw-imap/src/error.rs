//! Internal error type and its mapping onto the frozen [`EngineError`] seam.
//!
//! `mw-imap` speaks a rich internal error internally (distinguishing tagged
//! `NO`/`BAD`, transport faults, and unmet capabilities) and collapses it onto
//! the coarse [`EngineError`] variants at the `AccountBackend` boundary so
//! `mw-engine` can apply uniform retry/degrade policy (plan §6.1).

use mw_engine::backend::EngineError;

/// Result alias for the crate's internal command/transport layer.
pub type ImapResult<T> = std::result::Result<T, ImapError>;

/// Errors produced by the IMAP transport and command engine.
#[derive(Debug)]
pub enum ImapError {
    /// Socket / TLS I/O failure.
    Io(std::io::Error),
    /// TLS setup or handshake failure.
    Tls(String),
    /// The server closed the connection or sent a truncated response.
    Eof,
    /// Malformed or unexpected protocol bytes (parser rejected the response).
    Protocol(String),
    /// A tagged `NO` completion (command refused by the server).
    No(String),
    /// A tagged `BAD` completion (client sent something invalid).
    Bad(String),
    /// An untagged `BYE` — the server is disconnecting.
    Bye(String),
    /// Authentication (LOGIN / AUTHENTICATE) was rejected.
    Auth(String),
    /// A required capability is not advertised by this server.
    Unsupported(String),
    /// A referenced mailbox does not exist upstream.
    MailboxNotFound(String),
}

impl std::fmt::Display for ImapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImapError::Io(e) => write!(f, "io error: {e}"),
            ImapError::Tls(m) => write!(f, "tls error: {m}"),
            ImapError::Eof => write!(f, "connection closed by server"),
            ImapError::Protocol(m) => write!(f, "protocol error: {m}"),
            ImapError::No(m) => write!(f, "server refused command (NO): {m}"),
            ImapError::Bad(m) => write!(f, "server rejected command (BAD): {m}"),
            ImapError::Bye(m) => write!(f, "server disconnecting (BYE): {m}"),
            ImapError::Auth(m) => write!(f, "authentication failed: {m}"),
            ImapError::Unsupported(m) => write!(f, "capability not supported: {m}"),
            ImapError::MailboxNotFound(m) => write!(f, "mailbox not found: {m}"),
        }
    }
}

impl std::error::Error for ImapError {}

impl From<std::io::Error> for ImapError {
    fn from(e: std::io::Error) -> Self {
        ImapError::Io(e)
    }
}

impl From<ImapError> for EngineError {
    fn from(e: ImapError) -> Self {
        match e {
            ImapError::Io(err) => EngineError::Transport(err.to_string()),
            ImapError::Tls(m) => EngineError::Transport(m),
            ImapError::Eof => EngineError::Transport("connection closed by server".into()),
            ImapError::Bye(m) => EngineError::Transport(format!("server disconnected: {m}")),
            ImapError::Protocol(m) | ImapError::Bad(m) => EngineError::Protocol(m),
            ImapError::No(m) => EngineError::Protocol(m),
            ImapError::Auth(m) => EngineError::Auth(m),
            ImapError::Unsupported(m) => EngineError::Unsupported(m),
            ImapError::MailboxNotFound(m) => EngineError::MailboxNotFound(m),
        }
    }
}
