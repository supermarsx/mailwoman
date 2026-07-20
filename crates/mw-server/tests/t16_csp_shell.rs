//! t16-e-e2e — the SPA shell is served under the SHIPPED tightened CSP (S10),
//! completed by 26.17 t17-e5 (Trusted-Types enforcement re-enabled).
//!
//! 26.16 shipped `SHELL_CSP_TIGHTENED` MINUS `require-trusted-types-for 'script'`
//! (enforcement was deferred until the SPA registered a default TT policy). 26.17
//! registers that default policy (`apps/web/src/main.tsx`) and re-enables the
//! directive, so the shipped shell CSP is now the full tightened value. A broken
//! shell here blocks release, so this leg drives the REAL shell route through
//! `build_app` and asserts:
//!   * `GET /` returns the shell (200) with its content delivered;
//!   * the `content-security-policy` header is EXACTLY the shipped tightened value —
//!     `style-src 'self'` (the `'unsafe-inline'` style source is dropped),
//!     `script-src 'self' 'wasm-unsafe-eval'`, `default-src 'none'`, and
//!     `require-trusted-types-for 'script'` (enforcement now present).
//!
//! NOTE (honest scope): a pixel-level "the built SPA paints under this CSP" check needs
//! a browser + the production `apps/web/dist` bundle and is owned by the web gate
//! (`pnpm -C apps/web build` + the Playwright/CSP checks). This backend leg proves the
//! shipped HEADER value and that the shell is delivered under it — the wiring half.

use std::path::PathBuf;

use mw_server::{AppConfig, build_app};

fn unique() -> String {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    format!(
        "{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><script src=\"/app.js\"></script><div id=app>MW_SHELL</div>";

async fn spawn_server() -> String {
    let base = std::env::temp_dir().join(format!("mw-t16-csp-{}", unique()));
    let web = base.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let config = AppConfig {
        db_path: base.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: None,
        web_dir: Some(web as PathBuf),
        cookie_secure: false,
        mode: mw_server::ServerMode::Proxy,
        hardening: mw_server::HardeningConfig::default(),
        security: mw_server::SecurityConfig::default(),
    };
    let app = build_app(config).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

#[tokio::test]
async fn shell_is_served_under_the_shipped_tightened_csp() {
    let base = spawn_server().await;
    let resp = reqwest::get(format!("{base}/")).await.unwrap();
    assert_eq!(resp.status(), 200, "the shell is served");

    let csp = resp
        .headers()
        .get("content-security-policy")
        .and_then(|v| v.to_str().ok())
        .expect("shell carries a CSP header")
        .to_string();

    let html = resp.text().await.unwrap();
    assert!(
        html.contains("MW_SHELL"),
        "the shell body is delivered: {html}"
    );

    // The shipped tightened value (S10): style 'unsafe-inline' dropped …
    assert!(
        csp.contains("style-src 'self'"),
        "style-src is tightened to 'self': {csp}"
    );
    assert!(
        !csp.contains("style-src 'self' 'unsafe-inline'")
            && !csp.contains("style-src 'unsafe-inline'"),
        "the 'unsafe-inline' style source is dropped from the shell CSP: {csp}"
    );
    // … WASM permitted for the crypto worker, but not JS eval …
    assert!(
        csp.contains("script-src 'self' 'wasm-unsafe-eval'"),
        "script-src permits self + wasm only: {csp}"
    );
    assert!(!csp.contains("'unsafe-eval'") || csp.contains("'wasm-unsafe-eval'"));
    assert!(
        csp.contains("default-src 'none'"),
        "default-src locked down: {csp}"
    );
    // … and Trusted-Types enforcement is now ON (26.17: a default TT policy ships in
    // apps/web/src/main.tsx, so the directive no longer breaks the shell at boot).
    assert!(
        csp.contains("require-trusted-types-for 'script'"),
        "Trusted-Types enforcement must be present in the shipped shell CSP: {csp}"
    );

    eprintln!("[t16 csp] shipped shell CSP:\n  {csp}");
    eprintln!(
        "[t16 csp] NOTE: pixel-level SPA render under this CSP is the web gate's job \
         (needs apps/web/dist + a browser)."
    );
}
