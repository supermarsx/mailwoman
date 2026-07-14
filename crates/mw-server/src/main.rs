//! Mailwoman CLI: `mailwoman serve` runs the HTTP(S) server; `mailwoman fonts
//! pull` self-hosts Google Fonts; `mailwoman healthcheck` probes a running
//! instance (used by the Docker HEALTHCHECK).

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use clap::{Parser, Subcommand};

use mw_server::fonts::{self, GoogleFonts, PullOptions};
use mw_server::{
    AppConfig, HardeningConfig, ReloadableResolver, SecurityConfig, ServerMode, TlsConfig,
    TlsListener, build_app,
};

#[derive(Parser)]
#[command(name = "mailwoman", version, about = "Mailwoman server")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the HTTP(S) server.
    Serve(ServeArgs),
    /// Download and self-host fonts.
    Fonts(FontsArgs),
    /// Probe `/healthz` on a running server; exit 0 if healthy, 1 otherwise.
    Healthcheck(HealthArgs),
    // ── V6 subcommand STUBS (plan §3 e0). Parse-only; each prints "not yet
    // implemented" and is filled by its owning Batch-B executor. ──
    /// Copy a SQLite store into a Postgres backend (V6 §4.2). STUB — filled by e1.
    MigrateStore(MigrateStoreArgs),
    /// Admin panel CLI: manage domains/users/quotas/policy/observability (V6
    /// §19). STUB — the `mailwoman admin <noun> <verb>` tree is filled by e5.
    Admin(AdminArgs),
    /// Run the MCP server over stdio, proxying to a configured server (V6 §20.3).
    /// STUB — filled by e4.
    McpStdio(McpStdioArgs),
    // ── V7 subcommand STUBS (plan §3 e0). Parse-only; each prints "not yet
    // implemented" and is filled by its owning Batch-B/C executor. ──
    /// Manage WASM engine plugins (V7 §22). STUB — filled by e14.
    Plugin(PluginArgs),
    /// Change an account password via a configured backend (V7 §18.3). STUB —
    /// filled by e14 (over the `mw-passwd` backends).
    Password(PasswordArgs),
}

/// `mailwoman plugin <list|approve>` (V7 §22). STUB: e14 backs this with the
/// `mw-plugin` host + the 0008 `plugins` registry tables.
#[derive(Parser)]
struct PluginArgs {
    #[command(subcommand)]
    command: PluginCommand,
}

#[derive(Subcommand)]
enum PluginCommand {
    /// List registered plugins (id / version / approved / enabled).
    List,
    /// Admin-approve a plugin by id.
    Approve {
        /// The plugin id to approve.
        id: String,
    },
}

/// `mailwoman password` (V7 §18.3). STUB: e14 drives a `mw-passwd` backend +
/// re-seals sealed upstream credentials on success.
#[derive(Parser)]
struct PasswordArgs {
    /// Account id whose password to change (env: MW_ACCOUNT_ID).
    #[arg(long, env = "MW_ACCOUNT_ID")]
    account_id: Option<String>,
}

/// `mailwoman migrate-store` (V6 §4.2). STUB: e1 implements
/// `Store::migrate_from_sqlite`, copying rows with count + content parity.
#[derive(Parser)]
struct MigrateStoreArgs {
    /// Source SQLite DSN/path (env: MW_MIGRATE_FROM).
    #[arg(long, env = "MW_MIGRATE_FROM")]
    from: Option<String>,
    /// Destination Postgres DSN, e.g. `postgres://…` (env: MW_MIGRATE_TO).
    #[arg(long, env = "MW_MIGRATE_TO")]
    to: Option<String>,
}

/// `mailwoman admin …` (V6 §19). The full `mw_admin` noun/verb tree, backed by the
/// 0007 store tables via `mw_server::build_admin`.
#[derive(Parser)]
struct AdminArgs {
    /// SQLite/Postgres DSN (env: MW_DB_PATH).
    #[arg(long, env = "MW_DB_PATH", default_value = "mailwoman.db")]
    db_path: String,
    /// Hex-encoded 32-byte server key (env: MW_SERVER_KEY).
    #[arg(long, env = "MW_SERVER_KEY")]
    server_key: Option<String>,
    #[command(subcommand)]
    command: mw_admin::cli::AdminCommand,
}

/// `mailwoman mcp-stdio` (V6 §20.3): proxy stdin/stdout JSON-RPC to a configured
/// remote `/mcp` endpoint.
#[derive(Parser)]
struct McpStdioArgs {
    /// Upstream MCP server URL to proxy to (env: MW_MCP_SERVER).
    #[arg(long, env = "MW_MCP_SERVER")]
    server: Option<String>,
    /// Bearer token (API key / OAuth token) presented to the remote (env: MW_MCP_TOKEN).
    #[arg(long, env = "MW_MCP_TOKEN")]
    token: Option<String>,
}

#[derive(Parser)]
struct ServeArgs {
    /// Bind address (env: MW_BIND).
    #[arg(long, env = "MW_BIND", default_value = "0.0.0.0:8080")]
    bind: String,
    /// SQLite database path (env: MW_DB_PATH).
    #[arg(long, env = "MW_DB_PATH", default_value = "mailwoman.db")]
    db_path: String,
    /// Hex-encoded 32-byte server key (env: MW_SERVER_KEY). Generated if unset.
    #[arg(long, env = "MW_SERVER_KEY")]
    server_key: Option<String>,
    /// Serve static assets from disk instead of the embedded set (env: MW_WEB_DIR).
    #[arg(long, env = "MW_WEB_DIR")]
    web_dir: Option<PathBuf>,
    /// Mark the session cookie `Secure` (enable behind TLS) (env: MW_COOKIE_SECURE).
    #[arg(long, env = "MW_COOKIE_SECURE", default_value_t = false)]
    cookie_secure: bool,
    /// Upstream mode: `proxy` (JMAP, default) or `engine` (IMAP/POP3) (env: MW_MODE).
    #[arg(long, env = "MW_MODE", default_value = "proxy")]
    mode: String,

    // ---- TLS (plan §1.10) ----
    /// Acquire/renew certs via ACME (Let's Encrypt) for these domains; repeat or
    /// comma-separate (env: MW_ACME). Enables HTTPS on `--bind`.
    #[arg(long = "acme", env = "MW_ACME", value_delimiter = ',')]
    acme: Vec<String>,
    /// ACME contact email (env: MW_ACME_CONTACT).
    #[arg(long, env = "MW_ACME_CONTACT")]
    acme_contact: Option<String>,
    /// ACME account/cert cache directory (env: MW_ACME_CACHE).
    #[arg(long, env = "MW_ACME_CACHE", default_value = "acme-cache")]
    acme_cache: PathBuf,
    /// Use the Let's Encrypt *staging* directory (avoids rate limits) (env: MW_ACME_STAGING).
    #[arg(long, env = "MW_ACME_STAGING", default_value_t = false)]
    acme_staging: bool,
    /// External TLS certificate chain (PEM); hot-reloads on SIGHUP (env: MW_TLS_CERT).
    #[arg(long, env = "MW_TLS_CERT")]
    tls_cert: Option<PathBuf>,
    /// External TLS private key (PEM) (env: MW_TLS_KEY).
    #[arg(long, env = "MW_TLS_KEY")]
    tls_key: Option<PathBuf>,

    // ---- Hardening (plan §7.4) ----
    /// Emit COEP `require-corp` (crossOriginIsolated) (env: MW_COEP).
    #[arg(long, env = "MW_COEP", default_value_t = true)]
    coep: bool,
    /// Enforce the double-submit CSRF token (requires the SPA to send it) (env: MW_CSRF_STRICT).
    #[arg(long, env = "MW_CSRF_STRICT", default_value_t = false)]
    csrf_strict: bool,
    /// Idle session timeout in seconds (env: MW_SESSION_IDLE_SECS).
    #[arg(long, env = "MW_SESSION_IDLE_SECS", default_value_t = 1800)]
    session_idle_secs: u64,
    /// Absolute session lifetime in seconds (env: MW_SESSION_ABSOLUTE_SECS).
    #[arg(long, env = "MW_SESSION_ABSOLUTE_SECS", default_value_t = 43200)]
    session_absolute_secs: u64,
}

#[derive(Parser)]
struct FontsArgs {
    #[command(subcommand)]
    command: FontsCommand,
}

#[derive(Subcommand)]
enum FontsCommand {
    /// Download Google Fonts families, keep their per-unicode-range subsets, and
    /// write self-hostable woff2 + a rewritten stylesheet (`font-src 'self'`).
    Pull(FontsPullArgs),
}

#[derive(Parser)]
struct FontsPullArgs {
    /// Google Fonts family specs, e.g. `Inter:wght@400;700` `Fraunces:ital@1`.
    #[arg(required = true)]
    families: Vec<String>,
    /// Output directory (served from origin under `font-src 'self'`).
    #[arg(long, default_value = "fonts")]
    out: PathBuf,
    /// Restrict to these characters (Google `text=` subsetting).
    #[arg(long)]
    text: Option<String>,
    /// Origin-relative `url()` prefix written into the stylesheet.
    #[arg(long = "url-prefix", default_value = "/fonts")]
    url_prefix: String,
    /// Filename of the rewritten stylesheet written into `--out`.
    #[arg(long = "css-name", default_value = "fonts.css")]
    css_name: String,
}

#[derive(Parser)]
struct HealthArgs {
    /// Full health URL to probe. Defaults to http://<MW_BIND>/healthz.
    #[arg(long)]
    url: Option<String>,
}

#[tokio::main]
async fn main() -> std::process::ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cli = Cli::parse();
    match cli.command {
        Command::Serve(args) => run(serve(args).await),
        Command::Fonts(args) => run(fonts_cmd(args).await),
        Command::Healthcheck(args) => healthcheck(args).await,
        Command::MigrateStore(args) => run(migrate_store(args).await),
        Command::Admin(args) => run(admin_cmd(args).await),
        Command::McpStdio(args) => run(mcp_stdio_cmd(args).await),
        Command::Plugin(args) => run(plugin_cmd(args).await),
        Command::Password(args) => run(password_cmd(args).await),
    }
}

/// `mailwoman plugin <list|approve>` (plan §22, e14). STUB — parses and reports
/// "not yet implemented" until e14 wires the `mw-plugin` host + registry.
async fn plugin_cmd(args: PluginArgs) -> anyhow::Result<()> {
    match args.command {
        PluginCommand::List => println!("plugin list: not yet implemented (t7 e14)"),
        PluginCommand::Approve { id } => {
            println!("plugin approve {id}: not yet implemented (t7 e14)");
        }
    }
    Ok(())
}

/// `mailwoman password` (plan §18.3, e14). STUB — parses and reports "not yet
/// implemented" until e14 wires the `mw-passwd` backends.
async fn password_cmd(args: PasswordArgs) -> anyhow::Result<()> {
    let who = args.account_id.unwrap_or_else(|| "<account>".into());
    println!("password change for {who}: not yet implemented (t7 e14)");
    Ok(())
}

/// `mailwoman migrate-store` (plan §4.2, e1): copy a SQLite store into a Postgres
/// backend. Source and destination MUST share the same `ServerKey` (sealed columns
/// round-trip); the key comes from `MW_SERVER_KEY`.
async fn migrate_store(args: MigrateStoreArgs) -> anyhow::Result<()> {
    let from = args
        .from
        .ok_or_else(|| anyhow::anyhow!("--from (MW_MIGRATE_FROM) is required"))?;
    let to = args
        .to
        .ok_or_else(|| anyhow::anyhow!("--to (MW_MIGRATE_TO) is required"))?;
    let hex = std::env::var("MW_SERVER_KEY")
        .ok()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow::anyhow!("MW_SERVER_KEY must be set (src + dst share the key)"))?;
    let key = mw_store::ServerKey::from_hex(&hex)
        .map_err(|_| anyhow::anyhow!("MW_SERVER_KEY is not valid hex"))?;
    let dst = mw_store::Store::open(&to, key).await?;
    let report = dst.migrate_from_sqlite(&from).await?;
    println!(
        "migrated {} rows across {} tables from {from} → {to}",
        report.total_rows(),
        report.tables.len()
    );
    Ok(())
}

/// `mailwoman admin <noun> <verb>` (plan §19, e5): drive the admin domain logic +
/// audit log against the 0007 store tables.
async fn admin_cmd(args: AdminArgs) -> anyhow::Result<()> {
    let admin = mw_server::build_admin(&args.db_path, args.server_key.as_deref()).await?;
    let out = mw_admin::cli::run(&admin, args.command, "cli").await?;
    println!("{out}");
    Ok(())
}

/// `mailwoman mcp-stdio` (plan §20.3, e4): the stdio JSON-RPC bridge to a remote
/// `/mcp` endpoint.
async fn mcp_stdio_cmd(args: McpStdioArgs) -> anyhow::Result<()> {
    let url = args
        .server
        .ok_or_else(|| anyhow::anyhow!("--server (MW_MCP_SERVER) is required"))?;
    mw_server::mcp::run_stdio(&url, args.token).await
}

/// Map a fallible command result to a process exit code.
fn run(result: anyhow::Result<()>) -> std::process::ExitCode {
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!("fatal: {e:#}");
            std::process::ExitCode::FAILURE
        }
    }
}

async fn serve(args: ServeArgs) -> anyhow::Result<()> {
    let mode = match args.mode.as_str() {
        "engine" => ServerMode::Engine,
        _ => ServerMode::Proxy,
    };
    let hardening = HardeningConfig {
        coep: args.coep,
        csrf_strict: args.csrf_strict,
        idle_timeout: Duration::from_secs(args.session_idle_secs),
        absolute_timeout: Duration::from_secs(args.session_absolute_secs),
    };
    let config = AppConfig {
        db_path: args.db_path,
        server_key_hex: args.server_key,
        web_dir: args.web_dir,
        cookie_secure: args.cookie_secure,
        mode,
        hardening,
        security: SecurityConfig::from_env(),
    };
    let app = build_app(config).await?;

    // Decide the transport: ACME > external cert > plaintext.
    let tls = if !args.acme.is_empty() {
        Some(TlsConfig::Acme {
            domains: args.acme.clone(),
            contact: args.acme_contact.clone(),
            cache_dir: args.acme_cache.clone(),
            staging: args.acme_staging,
        })
    } else if let (Some(cert), Some(key)) = (args.tls_cert.clone(), args.tls_key.clone()) {
        Some(TlsConfig::External { cert, key })
    } else {
        None
    };

    match tls {
        None => {
            let listener = tokio::net::TcpListener::bind(&args.bind).await?;
            tracing::info!("mailwoman listening on http://{}", listener.local_addr()?);
            axum::serve(listener, app)
                .with_graceful_shutdown(shutdown_signal())
                .await?;
        }
        Some(tls_config) => {
            let (listener, resolver) = TlsListener::bind(&args.bind, &tls_config).await?;
            if let Some(resolver) = resolver {
                spawn_reload_on_sighup(resolver);
            }
            tracing::info!("mailwoman listening on https://{} (TLS)", args.bind);
            axum::serve(listener, app)
                .with_graceful_shutdown(shutdown_signal())
                .await?;
        }
    }
    Ok(())
}

/// Reload the external TLS cert on SIGHUP (Unix). Windows has no SIGHUP; a
/// restart picks up a new cert there.
#[cfg(unix)]
fn spawn_reload_on_sighup(resolver: Arc<ReloadableResolver>) {
    tokio::spawn(async move {
        use tokio::signal::unix::{SignalKind, signal};
        let mut hup = match signal(SignalKind::hangup()) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("SIGHUP handler unavailable: {e}");
                return;
            }
        };
        while hup.recv().await.is_some() {
            match resolver.reload() {
                Ok(()) => tracing::info!("reloaded TLS certificate on SIGHUP"),
                Err(e) => tracing::error!("TLS reload failed (keeping current cert): {e}"),
            }
        }
    });
}

#[cfg(not(unix))]
fn spawn_reload_on_sighup(_resolver: Arc<ReloadableResolver>) {
    tracing::info!(
        "external-cert hot-reload via signal is Unix-only; restart to reload on Windows"
    );
}

async fn fonts_cmd(args: FontsArgs) -> anyhow::Result<()> {
    match args.command {
        FontsCommand::Pull(a) => {
            let opts = PullOptions {
                families: a.families,
                text: a.text,
                out_dir: a.out,
                url_prefix: a.url_prefix,
                css_name: a.css_name,
            };
            let report = fonts::pull(&GoogleFonts::new(), &opts).await?;
            println!(
                "pulled {} font face(s) → {} (+{} woff2)",
                report.faces,
                report.css_path.display(),
                report.woff2.len()
            );
            Ok(())
        }
    }
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutdown signal received");
}

async fn healthcheck(args: HealthArgs) -> std::process::ExitCode {
    let url = args.url.unwrap_or_else(|| {
        let bind = std::env::var("MW_BIND").unwrap_or_else(|_| "0.0.0.0:8080".into());
        let host = bind.replace("0.0.0.0", "127.0.0.1");
        format!("http://{host}/healthz")
    });
    let client = reqwest::Client::new();
    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => std::process::ExitCode::SUCCESS,
        Ok(resp) => {
            eprintln!("healthcheck: {url} -> {}", resp.status());
            std::process::ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("healthcheck: {url} -> {e}");
            std::process::ExitCode::FAILURE
        }
    }
}
