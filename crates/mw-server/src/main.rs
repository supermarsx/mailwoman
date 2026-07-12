//! Mailwoman CLI: `mailwoman serve` runs the HTTP server; `mailwoman
//! healthcheck` probes a running instance (used by the Docker HEALTHCHECK).

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use mw_server::{AppConfig, build_app};

#[derive(Parser)]
#[command(name = "mailwoman", version, about = "Mailwoman server")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the HTTP server.
    Serve(ServeArgs),
    /// Probe `/healthz` on a running server; exit 0 if healthy, 1 otherwise.
    Healthcheck(HealthArgs),
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
        Command::Serve(args) => match serve(args).await {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(e) => {
                tracing::error!("fatal: {e:#}");
                std::process::ExitCode::FAILURE
            }
        },
        Command::Healthcheck(args) => healthcheck(args).await,
    }
}

async fn serve(args: ServeArgs) -> anyhow::Result<()> {
    let config = AppConfig {
        db_path: args.db_path,
        server_key_hex: args.server_key,
        web_dir: args.web_dir,
        cookie_secure: args.cookie_secure,
    };
    let app = build_app(config).await?;
    let listener = tokio::net::TcpListener::bind(&args.bind).await?;
    tracing::info!("mailwoman listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
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
