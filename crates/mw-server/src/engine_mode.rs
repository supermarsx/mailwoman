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

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

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

/// Where the engine's search index lives for this process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SearchIndexPlace {
    /// On disk, in this directory.
    Disk(PathBuf),
    /// In RAM, for the stated reason. Rebuilt from the store at every start.
    Memory(&'static str),
}

/// The value of `MW_SEARCH_DIR` that keeps the index in RAM.
const SEARCH_DIR_MEMORY: &str = "memory";

/// Decide where the search index goes, from `MW_SEARCH_DIR` if it is set and
/// otherwise from the database path:
///
/// * `MW_SEARCH_DIR=memory`: RAM.
/// * `MW_SEARCH_DIR=<dir>`: that directory.
/// * a SQLite file `<path>`: the directory `<path>.search-index` beside it.
/// * an in-memory SQLite database, or a Postgres DSN: RAM, because neither
///   names a local directory.
pub(crate) fn search_index_place(db_path: &str, search_dir: Option<&str>) -> SearchIndexPlace {
    match search_dir.map(str::trim).filter(|s| !s.is_empty()) {
        Some(v) if v.eq_ignore_ascii_case(SEARCH_DIR_MEMORY) => {
            return SearchIndexPlace::Memory("MW_SEARCH_DIR=memory");
        }
        Some(dir) => return SearchIndexPlace::Disk(PathBuf::from(dir)),
        None => {}
    }
    if db_path.starts_with("postgres://") || db_path.starts_with("postgresql://") {
        return SearchIndexPlace::Memory(
            "the database is Postgres and MW_SEARCH_DIR names no directory",
        );
    }
    // `Store::open` accepts a bare path or a `sqlite:` URL.
    let file = db_path
        .strip_prefix("sqlite://")
        .or_else(|| db_path.strip_prefix("sqlite:"))
        .unwrap_or(db_path);
    let file = file.split('?').next().unwrap_or(file);
    if file.is_empty() || file.contains(":memory:") {
        return SearchIndexPlace::Memory("the database is in memory");
    }
    SearchIndexPlace::Disk(PathBuf::from(format!("{file}.search-index")))
}

/// Build the engine for this process over `store`, with its search index where
/// [`search_index_place`] says. A directory that cannot be opened (missing and
/// not creatable, read-only, held by another process, or holding an index
/// Tantivy cannot read) does not stop start-up: the index is kept in RAM for
/// this run and the reason is logged.
pub(crate) fn build_engine(store: &mw_store::Store, db_path: &str) -> Arc<Engine> {
    let search_dir = std::env::var("MW_SEARCH_DIR").ok();
    match search_index_place(db_path, search_dir.as_deref()) {
        SearchIndexPlace::Disk(dir) => match Engine::open_with_search(store.clone(), &dir) {
            Ok(engine) => {
                tracing::info!(
                    "search index: on disk at {} (holds message text unsealed; set MW_SEARCH_DIR=memory to keep it in RAM)",
                    dir.display()
                );
                Arc::new(engine)
            }
            Err(e) => {
                tracing::warn!(
                    "search index: {} could not be opened ({e}); keeping the index in RAM for this run and rebuilding it from the store. If the directory holds a damaged index, delete it and restart.",
                    dir.display()
                );
                Arc::new(Engine::new(store.clone()))
            }
        },
        SearchIndexPlace::Memory(why) => {
            tracing::info!(
                "search index: in RAM ({why}); it is rebuilt from the store at every start"
            );
            Arc::new(Engine::new(store.clone()))
        }
    }
}

/// Start the engine's background work for a server process: the delayed
/// dispatcher, the search-index reconciliation, and the registration of stored
/// accounts. Returns at once; none of it delays start-up.
///
/// Call after the posture source and the plugin/bridge backends are attached:
/// the reconciliation reads the posture, and registration skips accounts a
/// bridge serves.
pub(crate) fn start_background(engine: &Arc<Engine>) {
    // Without this the dispatcher started only as a side effect of the first
    // login's `start_watch`.
    engine.start_dispatcher();

    let reconcile = Arc::clone(engine);
    tokio::spawn(async move {
        match reconcile.reconcile_search_index().await {
            Ok(Some(_)) => {}
            Ok(None) => tracing::info!("search index: agrees with the store, no rebuild"),
            Err(e) => tracing::warn!("search index: start-up rebuild failed: {e}"),
        }
    });

    let register = Arc::clone(engine);
    tokio::spawn(async move { register_stored_accounts(&register).await });
}

/// How many stored accounts are connected at the same time at start-up.
const BOOT_REGISTER_CONCURRENCY: usize = 4;

/// The longest one start-up connection attempt (dial, authenticate, first
/// resync) may take before it is abandoned.
const BOOT_REGISTER_TIMEOUT: Duration = Duration::from_secs(600);

/// Waits before the second, third and fourth attempt to connect an account
/// that could not be reached at start-up. After the last one the account stays
/// unregistered until someone logs in to it.
const BOOT_REGISTER_RETRY: [Duration; 3] = [
    Duration::from_secs(30),
    Duration::from_secs(120),
    Duration::from_secs(600),
];

/// Connect every stored account that may be used, so that mail keeps syncing
/// and due submissions are sent after a restart without anyone logging in.
///
/// Skipped: an account the admin has disabled or is holding for a password
/// change (and any account whose flags cannot be read), and an account bound to
/// a bridge, which `v7_mount::load_plugin_backends` registers.
///
/// An attempt the mail server refused as an authentication failure is not
/// repeated: the stored password is wrong, and repeating it can lock the
/// mailbox. Any other failure is retried per [`BOOT_REGISTER_RETRY`].
pub(crate) async fn register_stored_accounts(engine: &Arc<Engine>) {
    let store = engine.store();
    let accounts = match store.list_accounts().await {
        Ok(a) => a,
        Err(e) => {
            tracing::warn!("start-up account registration: cannot list accounts: {e}");
            return;
        }
    };
    let bridged: HashSet<String> = store
        .list_bridge_accounts()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|b| b.account_id)
        .collect();

    let slots = Arc::new(tokio::sync::Semaphore::new(BOOT_REGISTER_CONCURRENCY));
    let mut tasks = tokio::task::JoinSet::new();
    for account in accounts {
        if bridged.contains(&account.id) || engine.is_registered(&account.id) {
            continue;
        }
        match crate::account_gate::for_account(store, &account.id).await {
            Ok(gate) if !gate.disabled && !gate.password_change_required => {}
            Ok(_) => {
                tracing::info!(
                    "start-up account registration: {} is disabled or held, not connected",
                    account.id
                );
                continue;
            }
            Err(e) => {
                tracing::warn!(
                    "start-up account registration: flags of {} unreadable ({e}), not connected",
                    account.id
                );
                continue;
            }
        }
        let engine = Arc::clone(engine);
        let slots = Arc::clone(&slots);
        tasks.spawn(async move { register_one(&engine, &account.id, &slots).await });
    }
    let mut connected = 0usize;
    let mut failed = 0usize;
    while let Some(done) = tasks.join_next().await {
        match done {
            Ok(true) => connected += 1,
            _ => failed += 1,
        }
    }
    if connected + failed > 0 {
        tracing::info!(
            "start-up account registration: {connected} account(s) connected, {failed} not"
        );
    }
}

/// Connect one stored account, retrying per [`BOOT_REGISTER_RETRY`]. Returns
/// whether it ended up registered.
async fn register_one(
    engine: &Arc<Engine>,
    account_id: &str,
    slots: &tokio::sync::Semaphore,
) -> bool {
    let mut waits = BOOT_REGISTER_RETRY.iter();
    loop {
        let outcome = {
            let _slot = slots.acquire().await;
            tokio::time::timeout(BOOT_REGISTER_TIMEOUT, ensure_account(engine, account_id)).await
        };
        let error = match outcome {
            Ok(Ok(())) => return true,
            Ok(Err(e)) => e,
            Err(_) => "timed out".to_string(),
        };
        // A login made meanwhile may have registered it.
        if engine.is_registered(account_id) {
            return true;
        }
        if error.to_ascii_lowercase().contains("auth") {
            tracing::warn!(
                "start-up account registration: {account_id} refused ({error}); not retried, it connects at the next login"
            );
            return false;
        }
        let Some(wait) = waits.next() else {
            tracing::warn!(
                "start-up account registration: {account_id} not connected ({error}); giving up until the next login"
            );
            return false;
        };
        tracing::warn!(
            "start-up account registration: {account_id} not connected ({error}); retrying in {}s",
            wait.as_secs()
        );
        tokio::time::sleep(*wait).await;
    }
}

/// One lock per account id, so that two callers of [`ensure_account`] for the
/// same account (a request and the start-up registration, or two requests) do
/// not both dial the server and both start a watch loop.
fn connect_lock(account_id: &str) -> Arc<tokio::sync::Mutex<()>> {
    static LOCKS: OnceLock<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>> = OnceLock::new();
    LOCKS
        .get_or_init(Default::default)
        .lock()
        .expect("connect locks")
        .entry(account_id.to_string())
        .or_default()
        .clone()
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
    // Hostnames are case-insensitive and a trailing dot names the same host, so
    // store the canonical spelling (the account-identity lookup ignores both too).
    let host = host.trim_end_matches('.').to_ascii_lowercase();
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
    // Wrap the SMTP submitter with masked-email on-send From enforcement (26.10
    // follow-up a): a submission whose envelope `From` is one of this account's
    // enabled masked aliases is presented as the alias (real address hidden), and
    // an alias the account may not send as is refused fail-closed. A non-alias
    // `From` is forwarded byte-unchanged. Bridge/plugin accounts are not wrapped
    // (they send through the provider's own identity, see `v7_mount`).
    let base: Arc<dyn MailSubmitter> = Arc::new(submitter);
    let masked =
        crate::masked::MaskedSubmitter::new(engine.store().clone(), account_id.to_string(), base);
    let runtime = AccountRuntime::new(
        backend,
        Arc::new(masked) as Arc<dyn MailSubmitter>,
        identity,
    );
    engine.register_backend(account_id.to_string(), runtime);
}

/// Log in an IMAP/POP3 account: parse the URL, dial and authenticate the
/// backend, find or create the account, register it, and run an initial sync. Returns `(account_id,
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

    // Authenticate before writing anything. A refused login must leave no row
    // behind, and must not overwrite the credentials stored for an account that
    // already exists. IMAP authenticates inside `connect`; POP3 constructs without
    // dialling, so one authenticated round-trip is what proves the password.
    let backend = connect_backend(
        url.kind, &url.host, url.port, &url.tls, username, password, &policy,
    )
    .await?;
    backend.list_mailboxes().await.map_err(|e| e.to_string())?;

    // Find the account this identity logged in as before, or create it. The id
    // must be stable across logins: second factors, and everything else the
    // account owns, are keyed by it.
    let account_id = engine
        .store()
        .upsert_account_by_identity(
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
    let lock = connect_lock(account_id);
    let _connecting = lock.lock().await;
    // Whoever held the lock may have just registered it.
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
        // The host is stored in one spelling whatever case or trailing dot was typed.
        let u = parse_mail_url("IMAPS://Mail.Example.ORG.:993/").unwrap();
        assert_eq!(u.host, "mail.example.org");
        assert_eq!(u.port, 993);
        // Unknown scheme is rejected.
        assert!(parse_mail_url("ftp://x").is_none());
    }

    #[test]
    fn search_index_goes_beside_a_sqlite_file_and_nowhere_else_by_default() {
        let disk = |p: &str| SearchIndexPlace::Disk(PathBuf::from(p));
        assert_eq!(
            search_index_place("/data/mailwoman.db", None),
            disk("/data/mailwoman.db.search-index")
        );
        assert_eq!(
            search_index_place("sqlite:///data/mw.db?mode=rwc", None),
            disk("/data/mw.db.search-index")
        );
        // No local directory can be derived from these.
        for db in [
            "postgres://u:p@db/mw",
            "postgresql://db/mw",
            "sqlite::memory:",
            ":memory:",
        ] {
            assert!(
                matches!(search_index_place(db, None), SearchIndexPlace::Memory(_)),
                "{db}"
            );
        }
        // An empty or blank override is the same as none.
        assert_eq!(
            search_index_place("mailwoman.db", Some("  ")),
            disk("mailwoman.db.search-index")
        );
        // The override wins over the database path, Postgres included.
        assert_eq!(
            search_index_place("postgres://db/mw", Some("/var/lib/mw/idx")),
            disk("/var/lib/mw/idx")
        );
        assert!(matches!(
            search_index_place("/data/mailwoman.db", Some("Memory")),
            SearchIndexPlace::Memory(_)
        ));
    }
}
