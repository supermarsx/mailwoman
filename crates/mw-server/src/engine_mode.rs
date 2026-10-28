//! Engine mode (plan §3 e6, §1.4): the config switch that makes `mw-server`
//! answer `/jmap/session` + `/jmap/api` locally via `mw-engine` over a real
//! IMAP/POP3 account, instead of proxying to a JMAP upstream (the V0 default).
//!
//! Backend *construction* lives here rather than in `mw-engine`, because
//! `mw-imap`/`mw-pop3` depend on `mw-engine` for the frozen trait — so only this
//! crate, which depends on all three, can dial a server and hand the engine a
//! ready [`AccountRuntime`]. The web app `POST`s `{username, password}` to
//! `/api/login`; the server picks the `imap(s)://` / `pop3(s)://` server itself
//! (`upstream_resolve`, t32-h9) and hands it to [`engine_login`].

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

pub use crate::login_failure::{LoginFailure, LoginFailureReason};
use mw_engine::Engine;
use mw_engine::account::{AccountPolicy, AccountRuntime, MailSubmitter};
use mw_engine::backend::{AccountBackend, RawMailbox};
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

/// Whether a deployment that does not set `MW_SEARCH_DIR` gets an on-disk index
/// when its database path names a directory to put one in. `false` makes RAM
/// the default everywhere; `MW_SEARCH_DIR=<dir>` then remains the way to opt in
/// to a persistent index.
const SEARCH_INDEX_ON_DISK_BY_DEFAULT: bool = true;

/// Decide where the search index goes, from `MW_SEARCH_DIR` if it is set and
/// otherwise from the database path:
///
/// * `MW_SEARCH_DIR=memory`: RAM.
/// * `MW_SEARCH_DIR=<dir>`: that directory.
/// * a SQLite file `<path>`: the directory `<path>.search-index` beside it,
///   while [`SEARCH_INDEX_ON_DISK_BY_DEFAULT`] is `true`.
/// * an in-memory SQLite database, or a Postgres DSN: RAM, because neither
///   names a local directory.
pub(crate) fn search_index_place(db_path: &str, search_dir: Option<&str>) -> SearchIndexPlace {
    place_with_default(db_path, search_dir, SEARCH_INDEX_ON_DISK_BY_DEFAULT)
}

/// [`search_index_place`] with the default stated by the caller.
fn place_with_default(
    db_path: &str,
    search_dir: Option<&str>,
    on_disk_by_default: bool,
) -> SearchIndexPlace {
    match search_dir.map(str::trim).filter(|s| !s.is_empty()) {
        Some(v) if v.eq_ignore_ascii_case(SEARCH_DIR_MEMORY) => {
            return SearchIndexPlace::Memory("MW_SEARCH_DIR=memory");
        }
        Some(dir) => return SearchIndexPlace::Disk(PathBuf::from(dir)),
        None => {}
    }
    if !on_disk_by_default {
        return SearchIndexPlace::Memory("MW_SEARCH_DIR names no directory");
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
        SearchIndexPlace::Disk(dir) => match open_index_dir(store, &dir) {
            Ok(engine) => {
                // A warning, at every start: this is the one place the data
                // directory holds mail text that is not sealed.
                tracing::warn!(
                    "search index: on disk at {}. It holds the text of every indexed message (sender, recipients, subject, body, attachment text) UNSEALED, unlike the database, which seals message bodies under the server key. Protect this directory like the key itself, or set MW_SEARCH_DIR=memory to keep the index in RAM and rebuild it from the database at each start.",
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

/// Create `dir` readable by its owner only, where the platform has such a mode,
/// and open the index in it.
///
/// On Unix the directory is created with mode `0700`, and an existing directory
/// is set to `0700`; index files Tantivy creates inside keep the process umask,
/// which does not matter while the directory itself admits nobody else. On
/// other platforms (Windows) the directory inherits its parent's access control
/// list and nothing is changed here.
fn open_index_dir(store: &mw_store::Store, dir: &std::path::Path) -> Result<Engine, String> {
    restrict_to_owner(dir).map_err(|e| format!("create or restrict the directory: {e}"))?;
    Engine::open_with_search(store.clone(), dir).map_err(|e| e.to_string())
}

#[cfg(unix)]
fn restrict_to_owner(dir: &std::path::Path) -> std::io::Result<()> {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(dir)?;
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
fn restrict_to_owner(dir: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)
}

/// Start the engine's background work for a server process: the delayed
/// dispatcher, the search-index reconciliation, and the registration of stored
/// accounts. Returns at once; none of it delays start-up.
///
/// Call after the posture source and the plugin/bridge backends are attached:
/// the reconciliation reads the posture, and registration skips accounts a
/// bridge serves.
pub(crate) fn start_background(engine: &Arc<Engine>, security: crate::SecurityConfig) {
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
    tokio::spawn(async move { register_stored_accounts(&register, &security).await });
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
pub(crate) async fn register_stored_accounts(
    engine: &Arc<Engine>,
    security: &crate::SecurityConfig,
) {
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
        let matches =
            match crate::upstream_resolve::resolve_for(store, security, true, &account.username)
                .await
            {
                Ok(upstream) => account_matches_upstream(store, &account, upstream.url())
                    .await
                    .unwrap_or(false),
                Err(_) => false,
            };
        if !matches {
            tracing::warn!(
                "start-up account registration: {} has no configured upstream, not connected",
                account.id
            );
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

/// An old cached account must never reconnect to an authority an operator replaced.
pub(crate) async fn account_matches_upstream(
    store: &mw_store::Store,
    account: &mw_store::Account,
    url: &str,
) -> Result<bool, String> {
    if account.kind == AccountKind::Ews {
        let selected = crate::domain_provider::stored_selection(store, &account.id).await?;
        return Ok(selected.is_some_and(|selected| {
            selected
                .config
                .endpoint()
                .is_ok_and(|endpoint| endpoint.as_str() == url)
        }));
    }
    let expected = parse_mail_url(url).ok_or("invalid configured mail URL")?;
    let configured = crate::domain_config::upstream_for(store, &account.username).await?;
    let endpoint = configured
        .as_ref()
        .and_then(|c| c.mail())
        .map(|(endpoint, _)| endpoint);
    let binding = configured
        .as_ref()
        .map(|c| c.binding())
        .transpose()?
        .flatten();
    let saved_binding = AccountPolicy::from_json(&account.sync_policy_json).domain_binding;
    Ok(binding == saved_binding
        && account.kind == expected.kind
        && account.host.eq_ignore_ascii_case(&expected.host)
        && account.port == expected.port
        && endpoint.is_none_or(|endpoint| match &endpoint.security {
            Some(security) => security.allowed.iter().any(|method| {
                account.tls
                    == match method {
                        crate::domain_config::TlsMode::Implicit => "implicit",
                        crate::domain_config::TlsMode::Starttls => "starttls",
                        crate::domain_config::TlsMode::None => "plaintext",
                    }
            }),
            None => account.tls == endpoint.transport_security(),
        }))
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

/// One lock per account id. Whoever connects an account ([`ensure_account`], the
/// login in [`engine_login`]) holds it from before the backend is registered with
/// the engine until authenticated mailbox metadata is ready
/// ([`register_and_sync`]), and whoever asks whether an account is connected
/// takes it before looking. That gives two things:
///
/// * two callers for the same account (a request and the start-up registration,
///   a request and a login, or two requests) do not both register a backend and
///   both start a watch loop, and
/// * a caller never uses an account before its authenticated mailbox metadata
///   is available. Message indexing continues in a runtime-owned background job.
///
/// ## Why a request cannot deadlock against the start-up registration
///
/// This is the only lock either of them waits for while connecting, and nothing
/// that runs under it (the flag read, the dial, the mailbox metadata read)
/// calls back into [`ensure_account`]. The start-up task also holds one
/// of its own concurrency slots, which no request ever takes. So there is no
/// second lock to form a cycle with: a request can only wait for the holder to
/// finish, and the holder always lets go — on success, on an error, and when
/// its future is dropped (the start-up attempt is cut off after
/// [`BOOT_REGISTER_TIMEOUT`]; a request's future is dropped when its client
/// goes away). The wait is not new either: a request for an account that was not
/// registered yet already waited here for the start-up task's dial.
///
/// An account a bridge serves is registered by `v7_mount::load_plugin_backends`
/// and not under this lock; nothing here says when its first sync has run.
pub(crate) fn connect_lock(account_id: &str) -> Arc<tokio::sync::Mutex<()>> {
    static LOCKS: OnceLock<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>> = OnceLock::new();
    let mut locks = LOCKS
        .get_or_init(Default::default)
        .lock()
        .expect("connect locks");
    // Every request for the account comes through here, so the common case
    // (the lock exists) does not allocate a key.
    if let Some(lock) = locks.get(account_id) {
        return lock.clone();
    }
    locks.entry(account_id.to_string()).or_default().clone()
}

/// A parsed mail server URL, as the upstream resolver supplies it.
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
/// Whether `input` names an IMAP or POP3 server [`engine_login`] can dial.
pub(crate) fn is_mail_url(input: &str) -> bool {
    parse_mail_url(input).is_some()
}

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
fn build_submitter(
    policy: &AccountPolicy,
    username: &str,
    password: &str,
    auth_policy: Option<mw_net::PasswordAuthPolicy>,
) -> mw_smtp::Submitter {
    let security = match policy.smtp_security.as_str() {
        "implicit" => mw_smtp::Security::ImplicitTls,
        "plaintext" => mw_smtp::Security::Plaintext,
        _ => mw_smtp::Security::StartTls,
    };
    let credentials = if let Some(auth_policy) = auth_policy {
        mw_smtp::Credentials::PasswordWithPolicy {
            user: username.into(),
            pass: password.into(),
            policy: auth_policy,
        }
    } else if password.is_empty() {
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
fn endpoint_dialer(endpoint: &crate::domain_config::Endpoint) -> mw_net::SharedDialer {
    crate::domain_transport::GuardedDialer::shared(
        endpoint.host.trim_matches(['[', ']']).trim_end_matches('.'),
        endpoint.port,
        endpoint.allow_private,
    )
}

/// Resolve transport choices without credentials. Reconnects pin the method selected
/// at sign-in; changing it requires a fresh sign-in rather than replaying credentials.
async fn resolve_domain_transports(
    store: &mw_store::Store,
    config: &mut crate::domain_config::UpstreamConfig,
    identity: &str,
    pinned: Option<(&str, &str)>,
) -> Result<(mw_net::TlsOptions, mw_net::TlsOptions), LoginFailure> {
    let domain = crate::upstream_resolve::domain_of(identity)
        .ok_or("configured transport needs an email identity")?;
    let domain = store
        .canonical_policy_domain(&domain)
        .await
        .map_err(|_| "domain lookup failed")?;
    let plan = crate::image_proxy::load_plan(store)
        .await
        .map_err(|_| "configured egress route is unavailable")?;
    let mut mail_options = mw_net::TlsOptions::default();
    let mut smtp_options = mw_net::TlsOptions::default();
    for (protocol, slot) in [
        ("imap", &mut config.imap),
        ("pop3", &mut config.pop3),
        ("smtp", &mut config.smtp),
    ] {
        let Some(endpoint) = slot else {
            continue;
        };
        let mut candidate = endpoint.clone();
        if let Some((mail, smtp)) = pinned
            && let Some(security) = &mut candidate.security
            && security.mode == crate::domain_security::SecurityMode::Auto
        {
            let pinned = if protocol == "smtp" { smtp } else { mail };
            let method = match pinned {
                "implicit" => crate::domain_config::TlsMode::Implicit,
                "starttls" => crate::domain_config::TlsMode::Starttls,
                "plaintext" => crate::domain_config::TlsMode::None,
                _ => return Err("invalid persisted transport method".into()),
            };
            if !security.allowed.contains(&method) {
                return Err(
                    "persisted transport method is no longer allowed; sign in again".into(),
                );
            }
            security.mode = crate::domain_security::SecurityMode::Fixed;
            security.allowed = vec![method];
        }
        let resolved =
            crate::domain_security::resolve_endpoint(protocol, &candidate, &domain, &plan)
                .await
                .map_err(|e| LoginFailure::from_io(&e))?;
        *endpoint = resolved.endpoint;
        if protocol == "smtp" {
            smtp_options = resolved.tls_options;
        } else {
            mail_options = resolved.tls_options;
        }
    }
    Ok((mail_options, smtp_options))
}

type LoginObservation = Arc<Mutex<Option<LoginFailureReason>>>;
#[derive(Debug)]
struct ObservedDialer {
    inner: mw_net::SharedDialer,
    failure: LoginObservation,
}
impl mw_net::Dialer for ObservedDialer {
    fn dial<'a>(
        &'a self,
        host: &'a str,
        port: u16,
    ) -> mw_net::BoxFuture<'a, std::io::Result<mw_net::BoxStream>> {
        Box::pin(async move {
            self.inner.dial(host, port).await.inspect_err(|error| {
                *self
                    .failure
                    .lock()
                    .unwrap_or_else(|poison| poison.into_inner()) =
                    Some(LoginFailure::from_io(error).reason);
            })
        })
    }
}

struct BackendTransport {
    observation: LoginObservation,
    dialer: Option<mw_net::SharedDialer>,
    tls_options: mw_net::TlsOptions,
    auth_policy: Option<mw_net::PasswordAuthPolicy>,
}

/// Dial + authenticate the account backend for a stored account row.
async fn connect_backend(
    kind: AccountKind,
    host: &str,
    port: u16,
    tls: &str,
    credentials: (&str, &str),
    policy: &AccountPolicy,
    transport: BackendTransport,
) -> Result<Arc<dyn AccountBackend>, LoginFailure> {
    let BackendTransport {
        dialer,
        tls_options,
        observation,
        auth_policy,
    } = transport;
    let dialer = dialer.map(|inner| {
        Arc::new(ObservedDialer {
            inner,
            failure: observation.clone(),
        }) as mw_net::SharedDialer
    });
    let (username, password) = credentials;
    match kind {
        AccountKind::Ews => Err(LoginFailure::new(LoginFailureReason::Unsupported)),
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
                credentials: if let Some(policy) = auth_policy {
                    mw_imap::Credentials::PasswordWithPolicy {
                        username: username.into(),
                        password: password.into(),
                        policy,
                    }
                } else {
                    mw_imap::Credentials::Password {
                        username: username.to_string(),
                        password: password.to_string(),
                    }
                },
                watch_mailbox: "INBOX".to_string(),
            };
            let backend = mw_imap::ImapBackend::connect_via_with_tls(config, dialer, tls_options)
                .await
                .map_err(|e| {
                    LoginFailure::from_backend(
                        e,
                        *observation
                            .lock()
                            .unwrap_or_else(|poison| poison.into_inner()),
                    )
                })?;
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
            let mut backend = mw_pop3::Pop3Backend::new(config)
                .with_dialer(dialer)
                .with_tls_options(tls_options);
            if let Some(policy) = auth_policy {
                backend = backend.with_auth_policy(policy);
            }
            Ok(Arc::new(backend))
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
    credential_revision: Option<Vec<u8>>,
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
    let mut runtime = AccountRuntime::new(
        backend,
        Arc::new(masked) as Arc<dyn MailSubmitter>,
        identity,
    );
    runtime.credential_revision = credential_revision;
    runtime.dav = Some(crate::nextcloud_pim::config());
    runtime.dav_transport = Some(crate::nextcloud_pim_transport::ManagedTransport::runtime(
        engine.store().clone(),
        account_id.into(),
    ));
    engine.register_backend(account_id.to_string(), runtime);
}

/// Removes registration if authenticated mailbox metadata could not be cached.
struct FirstSync<'a> {
    engine: &'a Engine,
    account_id: &'a str,
    finished: bool,
}

impl Drop for FirstSync<'_> {
    fn drop(&mut self) {
        if !self.finished {
            self.engine.unregister(self.account_id);
        }
    }
}

/// Publish authenticated mailbox metadata under the connect lock, then index
/// messages in a cancelable background job. Large mailboxes must not consume
/// the credential-authentication deadline. Progress is exposed by Mailbox/get.
async fn register_and_sync(
    engine: &Arc<Engine>,
    account_id: &str,
    backend: Arc<dyn AccountBackend>,
    submitter: mw_smtp::Submitter,
    identity: &str,
    credential_revision: Option<Vec<u8>>,
    authenticated_mailboxes: Option<Vec<RawMailbox>>,
) -> Result<(), String> {
    register(
        engine,
        account_id,
        backend.clone(),
        submitter,
        identity,
        credential_revision,
    )
    .await;
    let mut first_sync = FirstSync {
        engine,
        account_id,
        finished: false,
    };
    let mailboxes = match authenticated_mailboxes {
        Some(mailboxes) => mailboxes,
        None => match backend.list_mailboxes().await {
            Ok(mailboxes) => mailboxes,
            Err(error) => {
                if matches!(error, mw_engine::backend::EngineError::Auth(_)) {
                    engine
                        .reject_runtime_credentials(account_id, &backend)
                        .await;
                }
                return Err(error.to_string());
            }
        },
    };
    engine
        .cache_mailbox_list(account_id, mailboxes)
        .await
        .map_err(|error| error.to_string())?;
    engine.start_initial_sync(account_id);
    first_sync.finished = true;
    Ok(())
}

/// Drop an account runtime whose first sync gave up, so the next
/// [`ensure_account`] dials a fresh connection and starts indexing again. The
/// failed job's connection is usually the thing that broke, and retrying on it
/// cannot succeed. Provider-backed (EWS) runtimes are only re-created by a
/// sign-in, so they keep the engine's in-place retry instead.
pub async fn redial_failed_initial_sync(engine: &Arc<Engine>, account_id: &str) {
    if engine.initial_sync_status(account_id) != "failed" || engine.is_plugin_backed(account_id) {
        return;
    }
    let lock = connect_lock(account_id);
    let _connecting = lock.lock().await;
    if engine.initial_sync_status(account_id) != "failed" {
        return;
    }
    match engine.store().get_account(account_id).await {
        Ok(account) if account.kind != AccountKind::Ews => {
            engine.unregister(account_id);
        }
        _ => {}
    }
}

/// Log in an IMAP/POP3 account: parse the URL, dial and authenticate the
/// backend, find or create the account, cache mailbox metadata and start indexing. Returns `(account_id,
/// username)` for the session cookie. Any failure is a uniform login error.
pub async fn engine_login(
    engine: &Arc<Engine>,
    server_url: &str,
    username: &str,
    password: &str,
) -> Result<(String, String), LoginFailure> {
    tokio::time::timeout(
        Duration::from_secs(25),
        engine_login_inner(engine, server_url, username, password),
    )
    .await
    .map_err(|_| LoginFailure::new(LoginFailureReason::Timeout))?
}

async fn engine_login_inner(
    engine: &Arc<Engine>,
    server_url: &str,
    username: &str,
    password: &str,
) -> Result<(String, String), LoginFailure> {
    let mut url =
        parse_mail_url(server_url).ok_or_else(|| "unrecognised mail server URL".to_string())?;
    let mut configured = crate::domain_config::upstream_for(engine.store(), username).await?;
    let binding = configured
        .as_ref()
        .map(|c| c.binding())
        .transpose()?
        .flatten();
    let (mail_tls_options, smtp_tls_options) = if let Some(config) = &mut configured {
        resolve_domain_transports(engine.store(), config, username, None).await?
    } else {
        (mw_net::TlsOptions::default(), mw_net::TlsOptions::default())
    };
    if configured
        .as_ref()
        .is_some_and(|config| config.selection.is_some() && config.mail().is_none())
    {
        return Err("no enabled mail backend is configured for this engine".into());
    }
    let domain_mail = configured.as_ref().and_then(|config| config.mail());
    if let Some((endpoint, protocol)) = domain_mail {
        url = MailUrl {
            kind: if protocol == "imap" {
                AccountKind::Imap
            } else {
                AccountKind::Pop3
            },
            host: endpoint
                .host
                .trim_matches(['[', ']'])
                .trim_end_matches('.')
                .to_ascii_lowercase(),
            port: endpoint.port,
            tls: endpoint.transport_security().into(),
        };
    }
    // Deployments fronting a plaintext test server (e.g. Greenmail in CI) can
    // force the transport without changing the configured URL.
    if domain_mail.is_none()
        && let Ok(tls) = std::env::var("MW_ENGINE_TLS")
        && matches!(tls.as_str(), "implicit" | "starttls" | "plaintext")
    {
        url.tls = tls;
    }
    let mut policy = if configured.as_ref().is_some_and(|c| c.selection.is_some()) {
        AccountPolicy::default()
    } else {
        smtp_policy(&url.host)
    };
    policy.domain_binding = binding;
    let wire_username = configured
        .as_ref()
        .map(|c| c.auth_username(crate::domain_selection::ServiceRole::Mail, username))
        .transpose()?
        .unwrap_or_else(|| username.into());
    let smtp_username = configured
        .as_ref()
        .map(|c| c.auth_username(crate::domain_selection::ServiceRole::Submission, username))
        .transpose()?
        .unwrap_or_else(|| username.into());
    if let Some(smtp) = configured.as_ref().and_then(|config| config.smtp.as_ref()) {
        policy.smtp_host = smtp.host.trim_matches(['[', ']']).to_string();
        policy.smtp_port = smtp.port;
        policy.smtp_security = smtp.transport_security().into();
    }
    let creds = Credentials {
        username: username.to_string(),
        password: password.to_string(),
    };

    // Authenticate before writing anything. A refused login must leave no row
    // behind, and must not overwrite the credentials stored for an account that
    // already exists. IMAP authenticates inside `connect`; POP3 constructs without
    // dialling, so one authenticated round-trip is what proves the password.
    let observation = Arc::new(Mutex::new(None));
    let backend = connect_backend(
        url.kind,
        &url.host,
        url.port,
        &url.tls,
        (&wire_username, password),
        &policy,
        BackendTransport {
            observation: observation.clone(),
            dialer: domain_mail.map(|(endpoint, _)| endpoint_dialer(endpoint)),
            tls_options: mail_tls_options,
            auth_policy: domain_mail
                .map(|(endpoint, protocol)| endpoint.auth_policy(protocol))
                .transpose()?
                .flatten(),
        },
    )
    .await?;
    let authenticated_mailboxes = backend.list_mailboxes().await.map_err(|e| {
        LoginFailure::from_backend(
            e,
            *observation
                .lock()
                .unwrap_or_else(|poison| poison.into_inner()),
        )
    })?;

    // An account the admin has disabled stops here. The password has been
    // checked, so the refusal takes as long as one for a wrong password and the
    // name alone does not show the flag; nothing below runs, so the refused
    // login writes no row, re-seals no credentials and leaves no connection to
    // the mail server. The lookup is read-only: an identity with no account row
    // yet is checked under the name it was typed as.
    let known = engine
        .store()
        .account_id_by_identity(url.kind, &url.host, url.port, username)
        .await
        .map_err(|_| LoginFailure::new(LoginFailureReason::Storage))?;
    let gate = crate::account_gate::for_login(
        engine.store(),
        known.as_deref().unwrap_or(""),
        username,
        username,
    )
    .await
    .map_err(|_| LoginFailure::new(LoginFailureReason::Storage))?;
    if gate.disabled {
        if let Some(id) = &known {
            engine.unregister(id);
        }
        return Err(LoginFailure::new(LoginFailureReason::Disabled));
    }

    // Serialize credential replacement with stored reconnects for an existing account.
    let identity_lock_key = known
        .clone()
        .unwrap_or_else(|| format!("login:{}:{}:{}", url.host, url.port, username));
    let lock = connect_lock(&identity_lock_key);
    let _connecting = lock.lock().await;

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
        .map_err(|_| LoginFailure::new(LoginFailureReason::Storage))?;

    let submitter = build_submitter(
        &policy,
        &smtp_username,
        password,
        configured
            .as_ref()
            .and_then(|c| c.smtp.as_ref())
            .map(|e| e.auth_policy("smtp"))
            .transpose()?
            .flatten(),
    )
    .with_tls_options(smtp_tls_options)
    .with_dialer(
        configured
            .as_ref()
            .and_then(|c| c.smtp.as_ref())
            .map(endpoint_dialer),
    );
    // Under the account's connect lock, so that a request for the account made
    // meanwhile (another session of it, after a restart) waits for this sync
    // instead of reading while it runs. The dial above is outside the lock.
    let new_account_lock = connect_lock(&account_id);
    let _new_account_connecting = if identity_lock_key != account_id {
        Some(new_account_lock.lock().await)
    } else {
        None
    };
    let revision = engine
        .store()
        .account_credential_revision(&account_id)
        .await
        .map_err(|_| LoginFailure::new(LoginFailureReason::Storage))?;
    let current_gate = crate::account_gate::for_account(engine.store(), &account_id)
        .await
        .map_err(|_| LoginFailure::new(LoginFailureReason::Storage))?;
    if current_gate.disabled {
        return Err(LoginFailure::new(LoginFailureReason::Disabled));
    }
    let current_binding = crate::domain_config::upstream_for(engine.store(), username)
        .await?
        .as_ref()
        .map(|configuration| configuration.binding())
        .transpose()?
        .flatten();
    if current_binding != policy.domain_binding {
        return Err("account authentication policy changed; sign in again".into());
    }
    register_and_sync(
        engine,
        &account_id,
        backend,
        submitter,
        username,
        Some(revision),
        Some(authenticated_mailboxes),
    )
    .await
    .map_err(|_| LoginFailure::new(LoginFailureReason::Sync))?;

    Ok((account_id, username.to_string()))
}

/// Ensure a stored account is connected in the engine, reconnecting it from its
/// sealed credentials if this process has not registered it yet (e.g. after a
/// restart). Idempotent. A disabled account is refused and not dialled.
///
/// `Ok` means the account is registered and, for an account connected by this
/// module, that authenticated mailbox metadata is ready. Message indexing may
/// continue in the background. A caller that arrives while
/// another one is connecting the account (the start-up registration, a login,
/// another request) waits for it; see `connect_lock` for why that wait cannot
/// deadlock.
pub async fn ensure_account(engine: &Arc<Engine>, account_id: &str) -> Result<(), String> {
    // The lock is taken before the account is looked up, not only when it is
    // missing. Until t30 a registered account returned from here without it,
    // and the backend is registered before the first sync runs: a request made
    // in between read an empty mailbox list.
    let lock = connect_lock(account_id);
    let _connecting = lock.lock().await;
    // A disabled account is not connected, whoever asks: a caller with no
    // session behind it (the admin metadata passthrough, start-up registration)
    // reaches this too. Unreadable flags refuse.
    let gate = crate::account_gate::for_account(engine.store(), account_id)
        .await
        .map_err(|e| e.to_string())?;
    if gate.disabled {
        return Err(format!("account {account_id} is disabled"));
    }
    let account = engine
        .store()
        .get_account(account_id)
        .await
        .map_err(|e| e.to_string())?;
    if serde_json::from_str::<serde_json::Value>(&account.sync_policy_json)
        .ok()
        .and_then(|p| p.get("credential_rejected").and_then(|v| v.as_bool()))
        .unwrap_or(false)
    {
        return Err("authentication rejected; fresh sign-in required".into());
    }
    if account.kind == AccountKind::Ews {
        if crate::domain_provider::stored_selection(engine.store(), account_id)
            .await?
            .is_none()
        {
            return Err("provider identity unavailable".into());
        }
        return if engine.is_registered(account_id) {
            Ok(())
        } else {
            Err("provider runtime unavailable; fresh sign-in required".into())
        };
    }
    let (creds, credential_revision) = engine
        .store()
        .account_credentials_with_revision(account_id)
        .await
        .map_err(|e| e.to_string())?;
    if engine.credential_revision_rejected(account_id, &credential_revision) {
        return Err("authentication rejected; fresh sign-in required".into());
    }
    let mut configured =
        crate::domain_config::upstream_for(engine.store(), &account.username).await?;
    let domain_mail = configured.as_ref().and_then(|c| c.mail());
    if let Some((endpoint, protocol)) = domain_mail
        && !account_matches_upstream(engine.store(), &account, &endpoint.mail_url(protocol)).await?
    {
        return Err("account upstream was replaced; sign in again".into());
    }
    let mut policy = AccountPolicy::from_json(&account.sync_policy_json);
    let binding = configured
        .as_ref()
        .map(|c| c.binding())
        .transpose()?
        .flatten();
    if policy.domain_binding != binding {
        engine.unregister(account_id);
        return Err("account authentication policy changed; sign in again".into());
    }
    if configured
        .as_ref()
        .is_some_and(|c| c.selection.is_some() && c.mail().is_none())
    {
        return Err("no enabled mail backend is configured".into());
    }
    if engine.is_registered(account_id) {
        return Ok(());
    }
    let (mail_tls_options, smtp_tls_options) = if let Some(config) = &mut configured {
        resolve_domain_transports(
            engine.store(),
            config,
            &account.username,
            Some((&account.tls, &policy.smtp_security)),
        )
        .await?
    } else {
        (mw_net::TlsOptions::default(), mw_net::TlsOptions::default())
    };
    let domain_mail = configured.as_ref().and_then(|c| c.mail());
    let wire_username = configured
        .as_ref()
        .map(|c| {
            c.auth_username(
                crate::domain_selection::ServiceRole::Mail,
                &account.username,
            )
        })
        .transpose()?
        .unwrap_or_else(|| creds.username.clone());
    let smtp_username = configured
        .as_ref()
        .map(|c| {
            c.auth_username(
                crate::domain_selection::ServiceRole::Submission,
                &account.username,
            )
        })
        .transpose()?
        .unwrap_or_else(|| creds.username.clone());
    if let Some(endpoint) = configured.as_ref().and_then(|c| c.smtp.as_ref()) {
        policy.smtp_host = endpoint.host.trim_matches(['[', ']']).to_string();
        policy.smtp_port = endpoint.port;
        policy.smtp_security = endpoint.transport_security().into();
    }

    let backend = connect_backend(
        account.kind,
        &account.host,
        account.port,
        &account.tls,
        (&wire_username, &creds.password),
        &policy,
        BackendTransport {
            observation: Arc::new(Mutex::new(None)),
            dialer: domain_mail.map(|(endpoint, _)| endpoint_dialer(endpoint)),
            tls_options: mail_tls_options,
            auth_policy: domain_mail
                .map(|(endpoint, protocol)| endpoint.auth_policy(protocol))
                .transpose()?
                .flatten(),
        },
    )
    .await;
    let backend = match backend {
        Ok(backend) => backend,
        Err(error) => {
            if error.reason == LoginFailureReason::Auth {
                engine.remember_rejected_credential_revision(account_id, &credential_revision);
                engine
                    .store()
                    .reject_account_credential_revision(account_id, &credential_revision)
                    .await
                    .map_err(|e| e.to_string())?;
            }
            return Err(error.to_string());
        }
    };
    let submitter = build_submitter(
        &policy,
        &smtp_username,
        &creds.password,
        configured
            .as_ref()
            .and_then(|c| c.smtp.as_ref())
            .map(|e| e.auth_policy("smtp"))
            .transpose()?
            .flatten(),
    )
    .with_tls_options(smtp_tls_options)
    .with_dialer(
        configured
            .as_ref()
            .and_then(|c| c.smtp.as_ref())
            .map(endpoint_dialer),
    );
    register_and_sync(
        engine,
        account_id,
        backend,
        submitter,
        &account.username,
        Some(credential_revision),
        None,
    )
    .await
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

    #[test]
    fn with_the_default_off_only_an_explicit_directory_puts_the_index_on_disk() {
        assert!(matches!(
            place_with_default("/data/mailwoman.db", None, false),
            SearchIndexPlace::Memory(_)
        ));
        assert_eq!(
            place_with_default("/data/mailwoman.db", Some("/var/lib/mw/idx"), false),
            SearchIndexPlace::Disk(PathBuf::from("/var/lib/mw/idx"))
        );
        // The same path with the default on, so the flag is what decided it.
        assert!(matches!(
            place_with_default("/data/mailwoman.db", None, true),
            SearchIndexPlace::Disk(_)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn the_index_directory_admits_its_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let base = std::env::temp_dir().join(format!("mw-e13-perm-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let dir = base.join("idx");
        // Created with the mode.
        restrict_to_owner(&dir).unwrap();
        let mode = |p: &std::path::Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&dir), 0o700);
        // An existing, wider directory is narrowed.
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(mode(&dir), 0o755);
        restrict_to_owner(&dir).unwrap();
        assert_eq!(mode(&dir), 0o700);
        let _ = std::fs::remove_dir_all(&base);
    }
}
