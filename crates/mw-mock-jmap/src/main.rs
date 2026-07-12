//! Standalone mock JMAP server. Port via `MW_MOCK_PORT` (default 8181).

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
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind mock port");
    tracing::info!(
        "mw-mock-jmap listening on http://{addr}  (user={})",
        mw_mock_jmap::USER
    );
    axum::serve(listener, mw_mock_jmap::router())
        .await
        .expect("serve");
}
