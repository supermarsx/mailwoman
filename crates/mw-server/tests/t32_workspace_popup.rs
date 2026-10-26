//! The detached document must exist even when a deployed web directory predates it.
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, build_app};
mod common;

#[tokio::test]
async fn workspace_popup_is_server_owned_at_root_and_prefix() {
    // One test owns every process-environment read/write in this test binary.
    let previous = std::env::var_os("MW_BASE_PATH");
    for prefix in ["", "/mail"] {
        unsafe {
            std::env::set_var("MW_BASE_PATH", prefix);
        }
        let directory = common::test_db::unique_dir("workspace-popup");
        let web = directory.join("web");
        std::fs::create_dir_all(&web).unwrap();
        std::fs::write(
            web.join("index.html"),
            "<!doctype html><title>Old SPA</title>",
        )
        .unwrap();
        let app = build_app(AppConfig {
            db_path: directory.join("test.db").to_string_lossy().into_owned(),
            server_key_hex: None,
            web_dir: Some(web.clone()),
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig::default(),
        })
        .await
        .unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let client = reqwest::Client::new();
        let base = format!("http://{address}{prefix}");
        let url = format!("{base}/workspace-popup.html");
        let response = client.get(&url).send().await.unwrap();
        assert_eq!(response.status(), 200);
        assert_eq!(
            response.headers()["content-type"],
            "text/html; charset=utf-8"
        );
        assert_eq!(response.headers()["cache-control"], "no-cache");
        let csp = response.headers()["content-security-policy"]
            .to_str()
            .unwrap()
            .to_owned();
        assert!(csp.contains("script-src 'self'"));
        assert!(!csp.contains("'unsafe-inline'"));
        let etag = response.headers()["etag"].clone();
        let body = response.text().await.unwrap();
        assert!(body.contains("data-mailwoman-workspace-popup=\"1\""));
        assert!(!body.contains("<script"));
        assert!(!body.contains("Old SPA"));
        let head = client.head(&url).send().await.unwrap();
        assert_eq!(head.status(), 200);
        assert_eq!(head.headers()["etag"], etag);
        assert_eq!(head.headers()["content-length"], body.len().to_string());
        assert!(head.bytes().await.unwrap().is_empty());
        let unchanged = client
            .get(&url)
            .header("if-none-match", &etag)
            .send()
            .await
            .unwrap();
        assert_eq!(unchanged.status(), 304);
        assert_eq!(unchanged.headers()["cache-control"], "no-cache");
        assert!(unchanged.bytes().await.unwrap().is_empty());
        // An incorrect old disk file must not override the compiled protocol shell.
        std::fs::write(
            web.join("workspace-popup.html"),
            "<h1>Wrong disk document</h1>",
        )
        .unwrap();
        let current = client.get(&url).send().await.unwrap();
        assert_eq!(current.text().await.unwrap(), body);
        assert_eq!(
            client
                .get(format!("{base}/missing-popup.html"))
                .send()
                .await
                .unwrap()
                .status(),
            404
        );
        // CSRF admits a same-origin non-browser request; the shell itself is GET/HEAD only.
        let invalid = client
            .post(&url)
            .header("origin", format!("http://{address}"))
            .send()
            .await
            .unwrap();
        assert_eq!(invalid.status(), 405);
        assert_eq!(invalid.headers()["allow"], "GET, HEAD");
        server.abort();
        let _ = server.await;
    }
    unsafe {
        match previous {
            Some(value) => std::env::set_var("MW_BASE_PATH", value),
            None => std::env::remove_var("MW_BASE_PATH"),
        }
    }
}
