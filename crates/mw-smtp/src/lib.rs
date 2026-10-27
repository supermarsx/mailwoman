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
//! stays decoupled — the only workspace crate it depends on is `mw-net`, the
//! tokio-only dial seam (t32-ext §J).

mod addr;
mod conn;
mod dsn_support;
mod sasl;
mod tls;

pub use addr::{validate_mailbox, validate_reverse_path};
use conn::{Connection, MailParams, RcptOutcome};
pub use tls::{MIN_TLS_REFUSED, MinTls, min_tls, set_min_tls};

/// A message ready for submission: envelope + already-serialized MIME bytes.
///
/// The `raw` bytes are produced by `mw-mime` (mail-builder); this crate never
/// parses or re-encodes them. It sends them as lines: every line end in `raw`
/// (CRLF, a bare LF or a bare CR) goes out as CRLF, and in the `DATA` phase a
/// line that begins with `.` is dot-stuffed. Bytes that already have CRLF line
/// ends are otherwise sent as they are. Header content inside `raw` is
/// therefore the caller's to get right; the envelope fields are checked here
/// (see [`Outgoing::validate`]).
#[derive(Debug, Clone)]
pub struct Outgoing {
    /// Envelope sender (`MAIL FROM`).
    pub mail_from: String,
    /// Envelope recipients (`RCPT TO`, one command each).
    pub rcpt_to: Vec<String>,
    /// Serialized RFC 5322 message bytes (`DATA`).
    pub raw: Vec<u8>,
}

impl Outgoing {
    /// Check the envelope: `mail_from` with [`validate_reverse_path`] and every
    /// `rcpt_to` entry with [`validate_mailbox`]. The first failure is returned
    /// and nothing is repaired, so one bad recipient refuses the whole message.
    ///
    /// [`Submitter::submit_with`] calls this before it opens a connection. It is
    /// public so that a caller which hands the message to something other than
    /// this crate's `Submitter` can apply the same rule.
    pub fn validate(&self) -> Result<(), SmtpError> {
        validate_reverse_path(&self.mail_from)?;
        self.rcpt_to.iter().try_for_each(|r| validate_mailbox(r))
    }
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
    /// Content transmission began, but no definitive acceptance or refusal arrived.
    /// Retrying can deliver a duplicate.
    #[error("delivery status unknown; check Sent before sending again: {0}")]
    DeliveryUncertain(String),
    /// SASL authentication was rejected.
    #[error("smtp authentication failed: {0}")]
    Auth(String),
    /// Malformed or unexpected protocol reply.
    #[error("smtp protocol error: {0}")]
    Protocol(String),
    /// An envelope address was refused before it was written to the wire (see
    /// [`validate_mailbox`]). The text holds the value debug-escaped and
    /// truncated, never raw.
    #[error("invalid envelope address {0}")]
    InvalidAddress(String),
    /// An envelope address is not ASCII and the server did not offer
    /// `SMTPUTF8` (RFC 6531), so the message was not sent to anyone. The text
    /// is that address, debug-escaped. The same message to the same server
    /// fails the same way again: this is not worth a retry.
    #[error(
        "the server does not offer SMTPUTF8, which the non-ASCII address {0} needs; the message was not sent"
    )]
    SmtpUtf8Required(String),
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
#[derive(Clone, Default)]
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
    /// Exact SCRAM variant; unlike the legacy variant this never upgrades it.
    ScramExact {
        user: String,
        pass: String,
        plus: bool,
    },
    /// Select once from this connection's advertised mechanisms, before auth.
    PasswordWithPolicy {
        user: String,
        pass: String,
        policy: mw_net::PasswordAuthPolicy,
    },
    /// SASL `OAUTHBEARER` (RFC 7628) bearer token.
    OAuthBearer { user: String, token: String },
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match self {
            Self::None => "None",
            Self::Plain { .. } => "Plain",
            Self::Login { .. } => "Login",
            Self::Scram { .. } => "Scram",
            Self::ScramExact { .. } => "ScramExact",
            Self::PasswordWithPolicy { .. } => "PasswordWithPolicy",
            Self::XOAuth2 { .. } => "XOAuth2",
            Self::OAuthBearer { .. } => "OAuthBearer",
        };
        f.debug_struct(kind).finish_non_exhaustive()
    }
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
    /// How the connection reaches the server (`None` = direct TCP).
    dialer: Option<mw_net::SharedDialer>,
    tls_options: mw_net::TlsOptions,
    dsn_support: std::sync::Arc<dsn_support::Support>,
}

type Connected = (
    Connection<mw_net::BoxStream>,
    Option<(&'static str, Vec<u8>)>,
);

impl Submitter {
    /// Construct a submitter for the given server configuration.
    pub fn new(config: SubmitConfig) -> Self {
        Self {
            config,
            dialer: None,
            tls_options: mw_net::TlsOptions::default(),
            dsn_support: Default::default(),
        }
    }

    /// Open every submission connection through `dialer` (`None` = direct TCP,
    /// the default). TLS and STARTTLS run over the dialled stream as over a
    /// direct socket, validated against the configured host.
    pub fn with_dialer(mut self, dialer: Option<mw_net::SharedDialer>) -> Self {
        self.dsn_support = Default::default();
        self.dialer = dialer;
        self
    }

    pub fn with_tls_options(mut self, options: mw_net::TlsOptions) -> Self {
        self.dsn_support = Default::default();
        self.tls_options = options;
        self
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
    ///
    /// The envelope and the `EHLO` name are checked first. A malformed address
    /// is `Err(SmtpError::InvalidAddress)` and no connection is made, so no
    /// recipient of that message is sent to.
    pub async fn submit_with(
        &self,
        msg: Outgoing,
        opts: SubmitOptions,
    ) -> Result<SubmissionResult, SmtpError> {
        msg.validate()?;
        let (mut conn, binding) = self.connect().await?;
        self.session(
            &mut conn,
            &msg,
            &opts,
            binding
                .as_ref()
                .map(|(name, bytes)| (*name, bytes.as_slice())),
        )
        .await
    }

    /// Recently observed DSN support; unknown or expired observations are false.
    pub fn cached_dsn_support(&self) -> bool {
        self.dsn_support.get() == Some(true)
    }

    /// Observe EHLO over the configured TLS, egress and authentication path.
    /// Never issues MAIL/RCPT/DATA; concurrent callers share a bounded probe.
    pub async fn probe_dsn(&self) -> Result<bool, SmtpError> {
        use std::time::Duration;
        if let Some(known) = self.dsn_support.get() {
            return Ok(known);
        }
        let _guard = tokio::time::timeout(Duration::from_secs(2), self.dsn_support.probe.lock())
            .await
            .map_err(|_| SmtpError::Transport("SMTP capability probe busy".into()))?;
        if let Some(known) = self.dsn_support.get() {
            return Ok(known);
        }
        let result = tokio::time::timeout(Duration::from_secs(2), async {
            let (mut conn, binding) = self.connect().await?;
            let caps = conn.ehlo(&self.config.ehlo_name).await?;
            conn.authenticate(
                &self.config.credentials,
                &caps,
                binding
                    .as_ref()
                    .map(|(name, bytes)| (*name, bytes.as_slice())),
            )
            .await?;
            conn.quit().await;
            Ok(caps.dsn)
        })
        .await
        .unwrap_or_else(|_| {
            Err(SmtpError::Transport(
                "SMTP capability probe timed out".into(),
            ))
        });
        self.dsn_support
            .record(result.as_ref().copied().unwrap_or(false));
        result
    }

    async fn connect(&self) -> Result<Connected, SmtpError> {
        conn::check_ehlo_name(&self.config.ehlo_name)?;
        let tcp = mw_net::dial(self.dialer.as_deref(), &self.config.host, self.config.port).await?;
        match self.config.security {
            Security::ImplicitTls => {
                let up = tls::connect(tcp, &self.config.host, &self.tls_options).await?;
                let mut conn = Connection::new(Box::new(up.stream) as mw_net::BoxStream);
                conn.read_greeting().await?;
                Ok((conn, up.channel_binding))
            }
            Security::StartTls => {
                let mut conn = Connection::new(tcp);
                conn.read_greeting().await?;
                let caps = conn.ehlo(&self.config.ehlo_name).await?;
                if !caps.starttls {
                    return Err(SmtpError::Protocol(
                        "server does not advertise STARTTLS on this port".into(),
                    ));
                }
                conn.starttls().await?;
                let up =
                    tls::connect(conn.into_inner()?, &self.config.host, &self.tls_options).await?;
                Ok((
                    Connection::new(Box::new(up.stream) as mw_net::BoxStream),
                    up.channel_binding,
                ))
            }
            Security::Plaintext => {
                let mut conn = Connection::new(tcp);
                conn.read_greeting().await?;
                Ok((conn, None))
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
        channel_binding: Option<(&str, &[u8])>,
    ) -> Result<SubmissionResult, SmtpError>
    where
        S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    {
        let caps = conn.ehlo(&self.config.ehlo_name).await?;

        // A non-ASCII envelope address needs SMTPUTF8 (RFC 6531). Without it
        // the message cannot be sent to this server, so stop here, before
        // AUTH and MAIL FROM, rather than part-way through the recipients.
        let needs_utf8 = std::iter::once(&msg.mail_from)
            .chain(&msg.rcpt_to)
            .find(|a| !a.is_ascii());
        if let Some(addr) = needs_utf8
            && !caps.smtputf8
        {
            conn.quit().await;
            return Err(SmtpError::SmtpUtf8Required(format!("{addr:?}")));
        }

        conn.authenticate(&self.config.credentials, &caps, channel_binding)
            .await?;
        self.dsn_support.record(caps.dsn);

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

        // SMTPUTF8 is sent when the envelope needs it; the server offers it,
        // or this point is not reached. SIZE only when advertised, and it is
        // the size of what is transmitted, not of `raw`. BODY=8BITMIME only
        // when the raw MIME needs it and the server supports it.
        let smtputf8 = needs_utf8.is_some();
        let (ret, envid) = match &opts.dsn {
            Some(d) => (d.ret, d.envid.clone()),
            None => (None, None),
        };
        let params = MailParams {
            size: caps.size.map(|_| conn::transmitted_len(&msg.raw)),
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
