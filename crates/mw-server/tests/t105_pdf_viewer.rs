//! The full PDF viewer is a fixed, independently isolated document. Attachment
//! bytes arrive through its parent bridge, never a file= query parameter.
use std::net::SocketAddr;

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use reqwest::{Client, StatusCode};

mod common;
use common::test_db;

struct Fixture {
    base: String,
    client: Client,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.task.abort();
    }
}
async fn fixture() -> Fixture {
    let web = test_db::unique_dir("pdf-viewer-web");
    std::fs::write(web.join("index.html"), "<!doctype html><title>Mail</title>").unwrap();
    let db = test_db::unique_db_path("pdf-viewer");
    let (app, _) = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into_owned(),
            server_key_hex: Some(
                "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff".into(),
            ),
            web_dir: Some(web),
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig::default(),
        },
        V6Config::default(),
    )
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    Fixture {
        base,
        client: Client::builder().no_proxy().build().unwrap(),
        task,
    }
}

#[tokio::test]
async fn pdf_document_has_its_own_narrow_policy_and_query_cannot_select_a_file() {
    let f = fixture().await;
    let url = format!("{}/pdfjs/web/mailwoman.html", f.base);
    let response = f.client.get(&url).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().get("x-frame-options").is_none());
    let csp = response.headers()["content-security-policy"]
        .to_str()
        .unwrap()
        .to_owned();
    assert!(csp.contains("frame-ancestors 'self'"));
    assert!(csp.contains("connect-src 'self' blob: data:"));
    assert!(csp.contains("require-trusted-types-for 'script'"));
    assert!(!csp.contains("connect-src *"));
    let body = response.text().await.unwrap();
    assert!(body.contains("../bootstrap.mjs"));
    assert!(!body.contains("<script src=\"viewer.mjs\""));
    assert!(!body.contains("<script src=\"../build/pdf.mjs\""));
    let with_query = f
        .client
        .get(format!("{url}?file=https://external.invalid/document.pdf"))
        .send()
        .await
        .unwrap();
    assert_eq!(with_query.status(), StatusCode::OK);
    assert_eq!(with_query.text().await.unwrap(), body);
}

#[tokio::test]
async fn pdf_head_and_conditional_requests_preserve_isolation_and_other_methods_are_refused() {
    let f = fixture().await;
    let url = format!("{}/pdfjs/web/mailwoman.html", f.base);
    let get = f.client.get(&url).send().await.unwrap();
    let etag = get.headers()["etag"].to_str().unwrap().to_owned();
    let body = get.bytes().await.unwrap();
    let head = f.client.head(&url).send().await.unwrap();
    assert_eq!(head.status(), StatusCode::OK);
    assert_eq!(
        head.headers()["content-length"]
            .to_str()
            .unwrap()
            .parse::<usize>()
            .unwrap(),
        body.len()
    );
    assert!(head.bytes().await.unwrap().is_empty());
    let cached = f
        .client
        .get(&url)
        .header("if-none-match", etag)
        .send()
        .await
        .unwrap();
    assert_eq!(cached.status(), StatusCode::NOT_MODIFIED);
    assert!(cached.headers().get("x-frame-options").is_none());
    assert!(
        cached.headers()["content-security-policy"]
            .to_str()
            .unwrap()
            .contains("frame-ancestors 'self'")
    );
    assert!(cached.bytes().await.unwrap().is_empty());
    let post = f.client.post(&url).send().await.unwrap();
    assert_eq!(post.status(), StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(post.headers()["allow"], "GET, HEAD");
}

#[tokio::test]
async fn request_headers_cannot_turn_the_mail_shell_into_an_embeddable_pdf_document() {
    let f = fixture().await;
    let response = f
        .client
        .get(format!("{}/index.html", f.base))
        .header("x-mw-pdf-viewer", "1")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["x-frame-options"], "DENY");
    assert!(
        !response.headers()["content-security-policy"]
            .to_str()
            .unwrap()
            .contains("frame-ancestors 'self'")
    );
}
