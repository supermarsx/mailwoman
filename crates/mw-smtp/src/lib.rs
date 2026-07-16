#![forbid(unsafe_code)]
//! `mw-smtp` — submission client (plan §0/§3, SPEC §6.1): 465 implicit-TLS and
//! 587 STARTTLS, EHLO capability parse, SASL PLAIN/LOGIN/XOAUTH2 plus
//! SCRAM-SHA-256 and OAUTHBEARER, MAIL FROM / RCPT TO with SIZE, 8BITMIME,
//! SMTPUTF8, REQUIRETLS and DSN, and both DATA and CHUNKING (`BDAT`) send paths.
//!
//! [`Submitter::submit`] runs the base flow; [`Submitter::submit_with`] layers
//! the per-message ESMTP extensions ([`SubmitOptions`]). It exists because the
//! "daily-drivable" exit criterion is impossible without send.
//!
//! ## Shape (the frozen sibling of the account-backend trait)
//! SMTP is a *sibling* to `mw-engine`'s `AccountBackend`, not part of it. On a
//! successful [`Submitter::submit`] the engine calls `backend.append(Sent, …)`
//! to file the sent copy (unless the server auto-files). This crate accepts the
//! already-serialized MIME bytes ([`Outgoing::raw`], built by `mw-mime`) so it
//! stays decoupled — it depends on no other workspace crate.

mod conn;
mod sasl;
mod tls;

use tokio::net::TcpStream;

use conn::{Connection, MailParams, RcptOutcome};

/// A message ready for submission: envelope + already-serialized MIME bytes.
///
/// The `raw` bytes are produced by `mw-mime` (mail-builder); this crate never
/// parses or re-encodes them, it only frames them into the `DATA` phase with
/// dot-stuffing.
#[derive(Debug, Clone)]
pub struct Outgoing {
    /// Envelope sender (`MAIL FROM`).
    pub mail_from: String,
    /// Envelope recipients (`RCPT TO`, one command each).
    pub rcpt_to: Vec<String>,
    /// Serialized RFC 5322 message bytes (`DATA`).
    pub raw: Vec<u8>,
}

/// Result of a submission: which recipients the server accepted and which it
/// rejected (with the reason), so the engine can surface a partial success.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmissionResult {
    /// Recipients the server accepted at `RCPT TO`.
    pub accepted: Vec<String>,
    /// Recipients the server rejected, paired with its reply text.
    pub rejected: Vec<(String, String)>,
}

/// Errors from SMTP submission.
#[derive(Debug, thiserror::Error)]
pub enum SmtpError {
    /// Transport-level failure (connect / TLS / socket I/O).
    #[error("smtp transport error: {0}")]
    Transport(String),
    /// SASL authentication was rejected.
    #[error("smtp authentication failed: {0}")]
    Auth(String),
    /// Malformed or unexpected protocol reply.
    #[error("smtp protocol error: {0}")]
    Protocol(String),
}

impl From<std::io::Error> for SmtpError {
    fn from(e: std::io::Error) -> Self {
        SmtpError::Transport(e.to_string())
    }
}

/// How the submission port is secured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Security {
    /// Implicit TLS from the first byte (submissions port 465, RFC 8314).
    ImplicitTls,
    /// Cleartext connect, then `STARTTLS` upgrade before AUTH (port 587).
    #[default]
    StartTls,
    /// No TLS at all — for local test servers only (e.g. Greenmail on 3025).
    Plaintext,
}

/// SASL credentials for the submission server.
#[derive(Debug, Clone, Default)]
pub enum Credentials {
    /// No authentication (unauthenticated relay / local test server).
    #[default]
    None,
    /// SASL `PLAIN` (RFC 4616).
    Plain { user: String, pass: String },
    /// SASL `LOGIN` (challenge/response username then password).
    Login { user: String, pass: String },
    /// SASL `XOAUTH2` bearer token (Gmail / Outlook).
    XOAuth2 { user: String, token: String },
    /// SASL `SCRAM-SHA-256` (RFC 5802 / RFC 7677) — salted challenge/response.
    Scram { user: String, pass: String },
    /// SASL `OAUTHBEARER` (RFC 7628) bearer token.
    OAuthBearer { user: String, token: String },
}

/// DSN `RET=` (RFC 3461): how much of the original message a bounce returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DsnRet {
    /// Return the entire message in any DSN.
    Full,
    /// Return only the headers.
    Hdrs,
}

impl DsnRet {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            DsnRet::Full => "FULL",
            DsnRet::Hdrs => "HDRS",
        }
    }
}

/// DSN `NOTIFY=` conditions (RFC 3461). `Never` is mutually exclusive with the
/// others; emit it alone to suppress notifications entirely.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DsnNotify {
    /// `NEVER` — request no delivery-status notifications.
    Never,
    /// `SUCCESS` — notify on successful delivery.
    Success,
    /// `FAILURE` — notify on delivery failure.
    Failure,
    /// `DELAY` — notify if delivery is delayed.
    Delay,
}

impl DsnNotify {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            DsnNotify::Never => "NEVER",
            DsnNotify::Success => "SUCCESS",
            DsnNotify::Failure => "FAILURE",
            DsnNotify::Delay => "DELAY",
        }
    }
}

/// Delivery-status-notification request applied to a submission (RFC 3461).
#[derive(Debug, Clone, Default)]
pub struct Dsn {
    /// `RET=` on `MAIL FROM`.
    pub ret: Option<DsnRet>,
    /// `ENVID=` on `MAIL FROM` (xtext-encoded on the wire).
    pub envid: Option<String>,
    /// `NOTIFY=` applied to every `RCPT TO`.
    pub notify: Vec<DsnNotify>,
    /// Emit `ORCPT=rfc822;<addr>` on each `RCPT TO`.
    pub orcpt: bool,
}

/// Per-submission ESMTP-extension options layered over the base flow. The
/// default is byte-for-byte the historical `submit` behaviour (no DSN, no
/// REQUIRETLS, `DATA` framing); `SMTPUTF8` is auto-negotiated regardless.
#[derive(Debug, Clone, Default)]
pub struct SubmitOptions {
    /// Request delivery-status notifications (requires server `DSN`).
    pub dsn: Option<Dsn>,
    /// Require TLS for this message (RFC 8689; requires server `REQUIRETLS`).
    pub require_tls: bool,
    /// Prefer `BDAT`/CHUNKING framing when the server advertises `CHUNKING`.
    pub use_chunking: bool,
}

/// Everything needed to reach and authenticate to one submission server.
#[derive(Debug, Clone)]
pub struct SubmitConfig {
    /// Submission host (also used as the TLS SNI / certificate name).
    pub host: String,
    /// Submission port (465 implicit-TLS, 587 STARTTLS, …).
    pub port: u16,
    /// Transport security to negotiate.
    pub security: Security,
    /// SASL credentials.
    pub credentials: Credentials,
    /// The name announced in `EHLO` (the client's own hostname).
    pub ehlo_name: String,
}

impl Default for SubmitConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 587,
            security: Security::default(),
            credentials: Credentials::default(),
            ehlo_name: "localhost".to_string(),
        }
    }
}

/// Submission client bound to one account's submission server (plan §2.1).
#[derive(Debug, Clone, Default)]
pub struct Submitter {
    config: SubmitConfig,
}

impl Submitter {
    /// Construct a submitter for the given server configuration.
    pub fn new(config: SubmitConfig) -> Self {
        Self { config }
    }

    /// Submit one message with the default (no-extension) flow. Equivalent to
    /// [`submit_with`](Self::submit_with) with [`SubmitOptions::default`].
    pub async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult, SmtpError> {
        self.submit_with(msg, SubmitOptions::default()).await
    }

    /// Submit one message, applying the ESMTP-extension options. Establishes the
    /// transport (implicit-TLS / STARTTLS / cleartext), runs `EHLO → [STARTTLS →
    /// EHLO] → AUTH → MAIL → RCPT* → DATA|BDAT → QUIT`, and reports the
    /// per-recipient outcome. The engine calls `AccountBackend::append(Sent, …)`
    /// on success unless the server auto-files.
    pub async fn submit_with(
        &self,
        msg: Outgoing,
        opts: SubmitOptions,
    ) -> Result<SubmissionResult, SmtpError> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let tcp = TcpStream::connect(&addr).await?;

        match self.config.security {
            Security::ImplicitTls => {
                let tls = tls::connect(tcp, &self.config.host).await?;
                let mut conn = Connection::new(tls);
                conn.read_greeting().await?;
                self.session(&mut conn, &msg, &opts).await
            }
            Security::StartTls => {
                // Cleartext probe: greet, EHLO, verify STARTTLS, upgrade, then
                // re-EHLO over TLS inside `session`.
                let mut conn = Connection::new(tcp);
                conn.read_greeting().await?;
                let caps = conn.ehlo(&self.config.ehlo_name).await?;
                if !caps.starttls {
                    return Err(SmtpError::Protocol(
                        "server does not advertise STARTTLS on this port".into(),
                    ));
                }
                conn.starttls().await?;
                let tcp = conn.into_inner()?;
                let tls = tls::connect(tcp, &self.config.host).await?;
                let mut conn = Connection::new(tls);
                self.session(&mut conn, &msg, &opts).await
            }
            Security::Plaintext => {
                let mut conn = Connection::new(tcp);
                conn.read_greeting().await?;
                self.session(&mut conn, &msg, &opts).await
            }
        }
    }

    /// The secured-channel portion of the flow: `EHLO → AUTH → MAIL → RCPT* →
    /// DATA|BDAT → QUIT`. Runs over whichever stream the caller has established.
    async fn session<S>(
        &self,
        conn: &mut Connection<S>,
        msg: &Outgoing,
        opts: &SubmitOptions,
    ) -> Result<SubmissionResult, SmtpError>
    where
        S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    {
        let caps = conn.ehlo(&self.config.ehlo_name).await?;
        conn.authenticate(&self.config.credentials, &caps).await?;

        // Fail closed if the caller asked for an extension the server did not
        // advertise, rather than silently dropping the guarantee.
        if opts.require_tls && !caps.requiretls {
            return Err(SmtpError::Protocol(
                "server does not advertise REQUIRETLS".into(),
            ));
        }
        if opts.dsn.is_some() && !caps.dsn {
            return Err(SmtpError::Protocol("server does not advertise DSN".into()));
        }

        // SMTPUTF8 is auto-negotiated: send it when the envelope needs UTF-8 and
        // the server supports it. SIZE only when advertised; BODY=8BITMIME only
        // when the raw MIME needs it and the server supports it.
        let smtputf8 = caps.smtputf8
            && (!msg.mail_from.is_ascii() || msg.rcpt_to.iter().any(|r| !r.is_ascii()));
        let (ret, envid) = match &opts.dsn {
            Some(d) => (d.ret, d.envid.clone()),
            None => (None, None),
        };
        let params = MailParams {
            size: caps.size.map(|_| msg.raw.len()),
            body_8bit: caps.eightbitmime && msg.raw.iter().any(|&b| b >= 0x80),
            smtputf8,
            require_tls: opts.require_tls,
            ret,
            envid,
        };
        conn.mail_from(&msg.mail_from, &params).await?;

        let (notify, orcpt) = match &opts.dsn {
            Some(d) => (d.notify.clone(), d.orcpt),
            None => (Vec::new(), false),
        };
        let mut accepted = Vec::new();
        let mut rejected = Vec::new();
        for rcpt in &msg.rcpt_to {
            match conn.rcpt_to(rcpt, &notify, orcpt).await? {
                RcptOutcome::Accepted => accepted.push(rcpt.clone()),
                RcptOutcome::Rejected { reason } => rejected.push((rcpt.clone(), reason)),
            }
        }

        // With no accepted recipient there is nothing to send; close cleanly and
        // report the rejections.
        if accepted.is_empty() {
            conn.quit().await;
            return Ok(SubmissionResult { accepted, rejected });
        }

        if opts.use_chunking && caps.chunking {
            conn.bdat(&msg.raw).await?;
        } else {
            conn.data(&msg.raw).await?;
        }
        conn.quit().await;
        Ok(SubmissionResult { accepted, rejected })
    }
}
