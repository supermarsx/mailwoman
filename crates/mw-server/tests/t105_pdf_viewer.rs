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

#[tokio::test]
async fn viewer_assets_are_embedded_with_script_and_style_types_even_over_a_stale_web_dir() {
    let f = fixture().await;
    for (path, mime) in [
        ("pdfjs/bootstrap.mjs", "text/javascript"),
        ("pdfjs/mailwoman.css", "text/css"),
        ("pdfjs/web/viewer.css", "text/css"),
        ("pdfjs/web/viewer.mjs", "text/javascript"),
        ("pdfjs/build/pdf.mjs", "text/javascript"),
        ("pdf.worker.mjs", "text/javascript"),
        ("pdfjs/web/wasm/qcms_bg.wasm", "application/wasm"),
        ("pdfjs/web/cmaps/78-H.bcmap", "application/octet-stream"),
        ("pdfjs/web/locale/locale.json", "application/json"),
    ] {
        let r = f.client.get(format!("{}/{path}", f.base)).send().await.unwrap();
        assert_eq!(r.status(), StatusCode::OK, "{path}");
        let ct = r.headers()["content-type"].to_str().unwrap().to_owned();
        assert!(ct.starts_with(mime), "{path}: {ct}");
        assert_eq!(r.headers()["x-content-type-options"], "nosniff");
        let etag = r.headers()["etag"].to_str().unwrap().to_owned();
        assert!(!r.bytes().await.unwrap().is_empty(), "{path}");
        let cached = f
            .client
            .get(format!("{}/{path}", f.base))
            .header("if-none-match", etag)
            .send()
            .await
            .unwrap();
        assert_eq!(cached.status(), StatusCode::NOT_MODIFIED, "{path}");
    }
}

#[tokio::test]
async fn a_viewer_asset_miss_is_plain_text_404_never_json() {
    let f = fixture().await;
    for path in ["pdfjs/web/missing.css", "pdfjs/nope.mjs", "pdfjs/"] {
        let r = f
            .client
            .get(format!("{}/{path}", f.base))
            .header("accept", "*/*")
            .send()
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::NOT_FOUND, "{path}");
        assert!(
            r.headers()["content-type"]
                .to_str()
                .unwrap()
                .starts_with("text/plain"),
            "{path}"
        );
        assert!(r.text().await.unwrap().contains("PDF viewer asset not found"));
    }
    let post = f
        .client
        .post(format!("{}/pdfjs/bootstrap.mjs", f.base))
        .send()
        .await
        .unwrap();
    assert_eq!(post.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn a_stale_web_dir_copy_of_the_library_cannot_replace_the_embedded_one() {
    let web = test_db::unique_dir("pdf-viewer-stale-web");
    std::fs::write(web.join("index.html"), "<!doctype html><title>Mail</title>").unwrap();
    std::fs::create_dir_all(web.join("pdfjs/web")).unwrap();
    std::fs::write(web.join("pdfjs/bootstrap.mjs"), "STALE").unwrap();
    std::fs::write(web.join("pdfjs/web/viewer.css"), "STALE").unwrap();
    std::fs::write(web.join("pdfjs/manifest.json"), "{\"version\":\"0.0.1\"}").unwrap();
    let db = test_db::unique_db_path("pdf-viewer-stale");
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
    let client = Client::builder().no_proxy().build().unwrap();
    for path in ["pdfjs/bootstrap.mjs", "pdfjs/web/viewer.css"] {
        let body = client
            .get(format!("{base}/{path}"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        assert_ne!(body, "STALE", "{path}");
        assert!(body.len() > 100, "{path}");
    }
    task.abort();
}
