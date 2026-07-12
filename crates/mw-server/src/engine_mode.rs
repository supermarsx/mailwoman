//! Engine mode (plan §3 e6, §1.4): the config switch that makes `mw-server`
//! answer `/jmap/session` + `/jmap/api` locally via `mw-engine` over a real
//! IMAP/POP3 account, instead of proxying to a JMAP upstream (the V0 default).
//!
//! Backend *construction* lives here rather than in `mw-engine`, because
//! `mw-imap`/`mw-pop3` depend on `mw-engine` for the frozen trait — so only this
//! crate, which depends on all three, can dial a server and hand the engine a
//! ready [`AccountRuntime`]. The web app is unchanged: it still `POST`s the same
//! `{jmapUrl, username, password}` to `/api/login`; in engine mode the `jmapUrl`
//! field is read as an `imap(s)://` / `pop3(s)://` server URL.

use std::sync::Arc;

use mw_engine::Engine;
use mw_engine::account::{AccountPolicy, AccountRuntime, MailSubmitter};
use mw_engine::backend::AccountBackend;
use mw_store::{AccountKind, Credentials, NewAccount};

/// Which upstream the server presents on `/jmap/*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ServerMode {
    /// V0: transparently proxy a JMAP upstream (unchanged default).
    #[default]
    Proxy,
    /// V1: drive an IMAP/POP3 account locally through `mw-engine`.
    Engine,
}

impl ServerMode {
    /// Read the mode from `MW_MODE` (`proxy` | `engine`), defaulting to proxy.
    pub fn from_env() -> Self {
        match std::env::var("MW_MODE").ok().as_deref() {
            Some("engine") => ServerMode::Engine,
            _ => ServerMode::Proxy,
        }
    }
}

/// A parsed mail server URL from the login form's `jmapUrl` field.
#[derive(Debug, Clone, PartialEq, Eq)]
struct MailUrl {
    kind: AccountKind,
    host: String,
    port: u16,
    /// Canonical TLS string persisted on the account row (`implicit`/`starttls`).
    tls: String,
}

/// Parse `imaps://host[:port]` / `imap://…` / `pop3s://…` / `pop3://…`; a bare
/// host defaults to IMAPS. Returns `None` for an unrecognised scheme.
fn parse_mail_url(input: &str) -> Option<MailUrl> {
    let input = input.trim();
    let (scheme, rest) = match input.split_once("://") {
        Some((s, r)) => (s.to_ascii_lowercase(), r),
        None => ("imaps".to_string(), input),
    };
    let rest = rest.trim_end_matches('/');
    let (host, explicit_port) = match rest.rsplit_once(':') {
        Some((h, p)) if p.chars().all(|c| c.is_ascii_digit()) && !p.is_empty() => {
            (h.to_string(), p.parse::<u16>().ok())
        }
        _ => (rest.to_string(), None),
    };
    if host.is_empty() {
        return None;
    }
    let (kind, tls, default_port) = match scheme.as_str() {
        "imaps" => (AccountKind::Imap, "implicit", 993),
        "imap" => (AccountKind::Imap, "starttls", 143),
        "pop3s" | "pops" => (AccountKind::Pop3, "implicit", 995),
        "pop3" | "pop" => (AccountKind::Pop3, "starttls", 110),
        _ => return None,
    };
    Some(MailUrl {
        kind,
        host,
        port: explicit_port.unwrap_or(default_port),
        tls: tls.to_string(),
    })
}

/// The SMTP submission endpoint, read from the environment (with sensible
/// fallbacks to the IMAP host). Engine mode needs a send path to be
/// daily-drivable (plan §0).
fn smtp_policy(imap_host: &str) -> AccountPolicy {
    let host = std::env::var("MW_SMTP_HOST").unwrap_or_else(|_| imap_host.to_string());
    let security = std::env::var("MW_SMTP_SECURITY").unwrap_or_else(|_| "starttls".to_string());
    let default_port = match security.as_str() {
        "implicit" => 465,
        "plaintext" => 25,
        _ => 587,
    };
    let port = std::env::var("MW_SMTP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(default_port);
    AccountPolicy {
        smtp_host: host,
        smtp_port: port,
        smtp_security: security,
        ..AccountPolicy::default()
    }
}

/// Build the submitter for an account from its policy + credentials.
fn build_submitter(policy: &AccountPolicy, username: &str, password: &str) -> mw_smtp::Submitter {
    let security = match policy.smtp_security.as_str() {
        "implicit" => mw_smtp::Security::ImplicitTls,
        "plaintext" => mw_smtp::Security::Plaintext,
        _ => mw_smtp::Security::StartTls,
    };
    let credentials = if password.is_empty() {
        mw_smtp::Credentials::None
    } else {
        mw_smtp::Credentials::Plain {
            user: username.to_string(),
            pass: password.to_string(),
        }
    };
    mw_smtp::Submitter::new(mw_smtp::SubmitConfig {
        host: policy.smtp_host.clone(),
        port: policy.smtp_port,
        security,
        credentials,
        ehlo_name: "mailwoman".to_string(),
    })
}

/// Dial + authenticate the account backend for a stored account row.
async fn connect_backend(
    kind: AccountKind,
    host: &str,
    port: u16,
    tls: &str,
    username: &str,
    password: &str,
    policy: &AccountPolicy,
) -> Result<Arc<dyn AccountBackend>, String> {
    match kind {
        AccountKind::Imap => {
            let tls_mode = match tls {
                "implicit" => mw_imap::TlsMode::Implicit,
                "plaintext" => mw_imap::TlsMode::Plaintext,
                _ => mw_imap::TlsMode::StartTls,
            };
            let config = mw_imap::ImapConfig {
                host: host.to_string(),
                port,
                tls: tls_mode,
                credentials: mw_imap::Credentials::Password {
                    username: username.to_string(),
                    password: password.to_string(),
                },
                watch_mailbox: "INBOX".to_string(),
            };
            let backend = mw_imap::ImapBackend::connect(config)
                .await
                .map_err(|e| e.to_string())?;
            Ok(Arc::new(backend))
        }
        AccountKind::Pop3 => {
            let tls_mode = match tls {
                "implicit" => mw_pop3::TlsMode::Implicit,
                "plaintext" => mw_pop3::TlsMode::Plain,
                _ => mw_pop3::TlsMode::StartTls,
            };
            let config = mw_pop3::Pop3Config {
                host: host.to_string(),
                port,
                tls: tls_mode,
                auth: mw_pop3::Pop3Auth::UserPass,
                username: username.to_string(),
                secret: password.to_string(),
                leave_policy: mw_pop3::LeavePolicy::Keep,
                poll_interval: std::time::Duration::from_secs(policy.poll_secs.max(1)),
            };
            Ok(Arc::new(mw_pop3::Pop3Backend::new(config)))
        }
    }
}

/// Register a connected backend + submitter into the engine for `account_id`.
async fn register(
    engine: &Arc<Engine>,
    account_id: &str,
    backend: Arc<dyn AccountBackend>,
    submitter: mw_smtp::Submitter,
    identity: &str,
) {
    let runtime = AccountRuntime::new(
        backend,
        Arc::new(submitter) as Arc<dyn MailSubmitter>,
        identity,
    );
    engine.register_backend(account_id.to_string(), runtime);
}

/// Log in an IMAP/POP3 account: parse the URL, persist the account, dial the
/// backend, register it, and run an initial sync. Returns `(account_id,
/// username)` for the session cookie. Any failure is a uniform login error.
pub async fn engine_login(
    engine: &Arc<Engine>,
    server_url: &str,
    username: &str,
    password: &str,
) -> Result<(String, String), String> {
    let mut url =
        parse_mail_url(server_url).ok_or_else(|| "unrecognised mail server URL".to_string())?;
    // Deployments fronting a plaintext test server (e.g. Greenmail in CI) can
    // force the transport without changing the URL the browser posts.
    if let Ok(tls) = std::env::var("MW_ENGINE_TLS")
        && matches!(tls.as_str(), "implicit" | "starttls" | "plaintext")
    {
        url.tls = tls;
    }
    let policy = smtp_policy(&url.host);
    let creds = Credentials {
        username: username.to_string(),
        password: password.to_string(),
    };

    // Persist the account (sealed creds) before connecting so a reconnect after
    // restart has everything it needs.
    let account_id = engine
        .store()
        .create_account(
            &NewAccount {
                kind: url.kind,
                host: &url.host,
                port: url.port,
                tls: &url.tls,
                username,
                sync_policy_json: &policy.to_json(),
            },
            &creds,
        )
        .await
        .map_err(|e| e.to_string())?;

    let backend = connect_backend(
        url.kind, &url.host, url.port, &url.tls, username, password, &policy,
    )
    .await?;
    let submitter = build_submitter(&policy, username, password);
    register(engine, &account_id, backend, submitter, username).await;

    engine
        .resync(&account_id)
        .await
        .map_err(|e| e.to_string())?;
    // Change ingestion keeps the cache fresh for the next browser poll.
    let _ = engine.start_watch(&account_id).await;

    Ok((account_id, username.to_string()))
}

/// Ensure a stored account is connected in the engine, reconnecting it from its
/// sealed credentials if this process has not registered it yet (e.g. after a
/// restart). Idempotent.
pub async fn ensure_account(engine: &Arc<Engine>, account_id: &str) -> Result<(), String> {
    if engine.is_registered(account_id) {
        return Ok(());
    }
    let account = engine
        .store()
        .get_account(account_id)
        .await
        .map_err(|e| e.to_string())?;
    let creds = engine
        .store()
        .account_credentials(account_id)
        .await
        .map_err(|e| e.to_string())?;
    let policy = AccountPolicy::from_json(&account.sync_policy_json);

    let backend = connect_backend(
        account.kind,
        &account.host,
        account.port,
        &account.tls,
        &creds.username,
        &creds.password,
        &policy,
    )
    .await?;
    let submitter = build_submitter(&policy, &creds.username, &creds.password);
    register(engine, account_id, backend, submitter, &account.username).await;
    engine.resync(account_id).await.map_err(|e| e.to_string())?;
    let _ = engine.start_watch(account_id).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_scheme_and_port() {
        let u = parse_mail_url("imaps://imap.example.org").unwrap();
        assert_eq!(u.kind, AccountKind::Imap);
        assert_eq!(u.port, 993);
        assert_eq!(u.tls, "implicit");

        let u = parse_mail_url("imap://host:1143").unwrap();
        assert_eq!(u.port, 1143);
        assert_eq!(u.tls, "starttls");

        let u = parse_mail_url("pop3s://pop.example.org").unwrap();
        assert_eq!(u.kind, AccountKind::Pop3);
        assert_eq!(u.port, 995);

        // Bare host defaults to IMAPS.
        assert_eq!(parse_mail_url("mail.example.org").unwrap().port, 993);
        // Unknown scheme is rejected.
        assert!(parse_mail_url("ftp://x").is_none());
    }
}
