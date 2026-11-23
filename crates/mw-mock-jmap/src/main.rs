//! Standalone mock JMAP server. Port via `MW_MOCK_PORT` (default 8181). The
//! account is `testuser@example.org` / `testpass` unless `MW_MOCK_USER`,
//! `MW_MOCK_PASS` and `MW_MOCK_NAME` (the sending identity's name) say otherwise.

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let port: u16 = std::env::var("MW_MOCK_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8181);
    let defaults = mw_mock_jmap::MockAccount::default();
    let var = |name: &str, default: String| std::env::var(name).unwrap_or(default);
    let account = mw_mock_jmap::MockAccount {
        user: var("MW_MOCK_USER", defaults.user),
        pass: var("MW_MOCK_PASS", defaults.pass),
        name: var("MW_MOCK_NAME", defaults.name),
    };
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind mock port");
    tracing::info!(
        "mw-mock-jmap listening on http://{addr}  (user={})",
        account.user
    );
    axum::serve(listener, mw_mock_jmap::router_for(account))
        .await
        .expect("serve");
}
