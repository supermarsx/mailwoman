//! 26.20 t32-h4 — a downloaded attachment can never render as a page of this
//! origin (t32-ext.md §2, D-5).
//!
//! # The defect these tests pin
//! In engine mode `/jmap/download` served a MIME part with the type the sender
//! wrote on it, so a `text/html` or `application/xhtml+xml` attachment came back
//! typed as a document. Proxy mode already reduced the upstream's type, but no
//! download in either mode carried a sandbox policy. The web client turns the
//! response into a `blob:` URL whose type is the response's `Content-Type`, and a
//! `blob:` URL belongs to the origin that minted it: opening it ran the sender's
//! script as Mailwoman. A direct navigation to the download URL is the same page.
//!
//! # What is asserted
//! * engine mode: HTML, XHTML and SVG parts are served as
//!   `application/octet-stream`; a PNG part keeps `image/png`; every download,
//!   the whole-message one and the EML export carry
//!   `Content-Security-Policy: sandbox; default-src 'none'`, `nosniff` and an
//!   `attachment` disposition;
//! * proxy mode: the same headers on a relayed download, for an upstream that
//!   labels its bytes `text/html`, `application/xhtml+xml` or `image/png`;
//! * the sandbox policy is added to download responses only: the SPA shell keeps
//!   its single policy.
//!
//! Run:
//!   cargo test -p mw-server --test t32_download_sandbox --locked -- --test-threads=1

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::OnceLock;

use axum::Router;
use axum::extract::Path as UrlPath;
use axum::http::header;
use axum::response::IntoResponse;
use axum::routing::get;
use serde_json::{Value, json};
use tokio::sync::{Mutex, MutexGuard};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, build_app};

mod common;
use common::mock_imap::MockImap;
use common::test_db;

/// The policy every download response must carry.
const SANDBOX: &str = "sandbox; default-src 'none'";
const OPAQUE: &str = "application/octet-stream";

const ENGINE_USER: &str = "owner@example.org";
const ENGINE_PASS: &str = "Passw0rd!";

/// A 1x1 PNG.
const PNG_B64: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==";

/// A received message whose attachments are a page, an XHTML page, an SVG with
/// a script and a PNG.
fn parts_message() -> String {
    format!(
        "From: Ann <ann@example.org>\r\n\
To: owner@example.org\r\n\
Subject: parts\r\n\
Message-ID: <parts-1@example.org>\r\n\
Date: Mon, 05 Oct 2026 10:00:00 +0000\r\n\
MIME-Version: 1.0\r\n\
Content-Type: multipart/mixed; boundary=\"b1\"\r\n\
\r\n\
--b1\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
\r\n\
see attached\r\n\
--b1\r\n\
Content-Type: text/html; charset=utf-8\r\n\
Content-Disposition: attachment; filename=\"page.html\"\r\n\
\r\n\
<script>fetch('/api/me')</script>\r\n\
--b1\r\n\
Content-Type: application/xhtml+xml\r\n\
Content-Disposition: attachment; filename=\"page.xhtml\"\r\n\
\r\n\
<html xmlns=\"http://www.w3.org/1999/xhtml\"><script>fetch('/api/me')</script></html>\r\n\
--b1\r\n\
Content-Type: image/svg+xml\r\n\
Content-Disposition: attachment; filename=\"pic.svg\"\r\n\
\r\n\
<svg xmlns=\"http://www.w3.org/2000/svg\"><script>fetch('/api/me')</script></svg>\r\n\
--b1\r\n\
Content-Type: image/png\r\n\
Content-Disposition: attachment; filename=\"dot.png\"\r\n\
Content-Transfer-Encoding: base64\r\n\
\r\n\
{PNG_B64}\r\n\
--b1--\r\n"
    )
}

// ── harness ──────────────────────────────────────────────────────────────────

/// Serialises the tests in this binary: they set process environment that the
/// server reads when it is built and when it dials the mail server.
async fn serial() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let guard = LOCK.get_or_init(Default::default).lock().await;
    let uploads = test_db::unique_dir("mw-t32-h4-uploads");
    // SAFETY: the guard serialises every mutation and every dependent read here.
    unsafe {
        std::env::set_var("MW_ENGINE_TLS", "plaintext");
        std::env::set_var("MW_UPLOAD_DIR", &uploads);
        for k in [
            "MW_HEADER_AUTH",
            "MW_LDAP_BIND_AUTH",
            "MW_PASSWD_BACKEND",
            "MW_MCP_RESOURCE",
        ] {
            std::env::remove_var(k);
        }
    }
    guard
}

async fn spawn(mode: ServerMode, security: SecurityConfig) -> String {
    let dir = test_db::unique_dir("mw-t32-h4");
    let web: PathBuf = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), "<!doctype html><title>t</title>").unwrap();
    let config = AppConfig {
        db_path: dir.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: None,
        web_dir: Some(web),
        cookie_secure: false,
        mode,
        hardening: HardeningConfig::default(),
        security,
    };
    let app = build_app(config).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    format!("http://{addr}")
}

fn browser() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

/// One JMAP method call that must be answered; the method's result.
async fn jmap(c: &reqwest::Client, base: &str, method: &str, args: Value) -> Value {
    let r = c
        .post(format!("{base}/jmap/api"))
        .json(&json!({
            "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
            "methodCalls": [[method, args, "c"]]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "{method} reaches the engine");
    let body: Value = r.json().await.unwrap();
    body["methodResponses"][0][1].clone()
}

/// Upload [`parts_message`] and import it into the inbox, over HTTP. Returns the
/// Email id.
async fn import_parts(c: &reqwest::Client, base: &str, account: &str) -> String {
    let mailboxes = jmap(c, base, "Mailbox/get", json!({ "accountId": account })).await;
    let inbox = mailboxes["list"]
        .as_array()
        .and_then(|list| {
            list.iter().find(|m| {
                m["role"]
                    .as_str()
                    .is_some_and(|r| r.eq_ignore_ascii_case("inbox"))
            })
        })
        .unwrap_or_else(|| panic!("an inbox: {mailboxes}"))["id"]
        .as_str()
        .unwrap()
        .to_string();
    let r = c
        .post(format!("{base}/jmap/upload/{account}"))
        .header(reqwest::header::CONTENT_TYPE, "message/rfc822")
        .body(parts_message())
        .send()
        .await
        .unwrap();
    let status = r.status();
    let uploaded: Value = r.json().await.unwrap();
    assert_eq!(status, 200, "upload: {uploaded}");
    let imported = jmap(
        c,
        base,
        "Email/import",
        json!({ "accountId": account, "emails": { "i": {
            "blobId": uploaded["blobId"], "mailboxIds": { inbox: true }
        } } }),
    )
    .await;
    imported["created"]["i"]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("the message was imported: {imported}"))
        .to_string()
}

/// Every `Content-Security-Policy` value on a response.
fn policies(r: &reqwest::Response) -> Vec<String> {
    r.headers()
        .get_all(reqwest::header::CONTENT_SECURITY_POLICY)
        .iter()
        .map(|v| v.to_str().unwrap().to_string())
        .collect()
}

fn header_str(r: &reqwest::Response, name: reqwest::header::HeaderName) -> String {
    r.headers()
        .get(name)
        .map(|v| v.to_str().unwrap().to_string())
        .unwrap_or_default()
}

/// GET a download-shaped URL; assert its `Content-Type` is `served` and that it
/// has the headers no download may lack.
async fn sandboxed_download(c: &reqwest::Client, url: &str, served: &str) {
    let r = c.get(url).send().await.unwrap();
    assert_eq!(r.status(), 200, "{url}");
    assert_eq!(
        header_str(&r, reqwest::header::CONTENT_TYPE),
        served,
        "{url} Content-Type"
    );
    let csp = policies(&r);
    assert!(
        csp.iter().any(|p| p == SANDBOX),
        "{url} must carry `{SANDBOX}`; got {csp:?}"
    );
    assert_eq!(
        header_str(&r, reqwest::header::X_CONTENT_TYPE_OPTIONS),
        "nosniff",
        "{url}"
    );
    assert!(
        header_str(&r, reqwest::header::CONTENT_DISPOSITION).starts_with("attachment"),
        "{url} is served as an attachment"
    );
}

// ── engine mode ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn engine_mode_parts_are_typed_safely_and_sandboxed() {
    let _g = serial().await;
    let pop = MockImap::start(ENGINE_USER, ENGINE_PASS).await;
    let base = &spawn(
        ServerMode::Engine,
        SecurityConfig {
            upstream_default: Some(pop.url()),
            ..SecurityConfig::default()
        },
    )
    .await;
    let c = browser();
    let login = c
        .post(format!("{base}/api/login"))
        .json(&json!({ "username": ENGINE_USER, "password": ENGINE_PASS }))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200, "engine login");
    let account = login.json::<Value>().await.unwrap()["accountId"]
        .as_str()
        .unwrap()
        .to_string();

    let id = import_parts(&c, base, &account).await;
    let got = jmap(
        &c,
        base,
        "Email/get",
        json!({ "ids": [id], "properties": ["attachments"] }),
    )
    .await;
    let attachments: HashMap<String, (String, String)> = got["list"][0]["attachments"]
        .as_array()
        .unwrap_or_else(|| panic!("attachments listed: {got}"))
        .iter()
        .map(|a| {
            (
                a["name"].as_str().unwrap_or_default().to_string(),
                (
                    a["blobId"].as_str().unwrap().to_string(),
                    a["type"].as_str().unwrap_or_default().to_string(),
                ),
            )
        })
        .collect();

    for (name, metadata_type, served) in [
        ("page.html", "text/html", OPAQUE),
        ("page.xhtml", "application/xhtml+xml", OPAQUE),
        ("pic.svg", "image/svg+xml", OPAQUE),
        ("dot.png", "image/png", "image/png"),
    ] {
        let (blob, ty) = attachments
            .get(name)
            .unwrap_or_else(|| panic!("{name} is an attachment: {attachments:?}"));
        // The JMAP metadata still names the sender's type; only the bytes'
        // HTTP framing is reduced.
        assert_eq!(ty, metadata_type, "{name} metadata type");
        let url = format!("{base}/jmap/download/{account}/{blob}/{name}");
        sandboxed_download(&c, &url, served).await;
    }

    // The whole message and its export are downloads too.
    sandboxed_download(
        &c,
        &format!("{base}/jmap/download/{account}/{id}/message.eml"),
        "message/rfc822",
    )
    .await;
    let proof = c
        .post(format!("{base}/api/security/step-up"))
        .json(&json!({"password":ENGINE_PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(proof.status(), 200, "{}", proof.text().await.unwrap());
    sandboxed_download(
        &c,
        &format!("{base}/api/export/{id}?format=eml"),
        "message/rfc822",
    )
    .await;

    // The shell keeps its one policy; the sandbox is not applied site-wide.
    let shell = c.get(format!("{base}/")).send().await.unwrap();
    let csp = policies(&shell);
    assert_eq!(csp.len(), 1, "the shell has one policy: {csp:?}");
    assert!(!csp[0].contains("sandbox"), "{csp:?}");
}

// ── proxy mode ───────────────────────────────────────────────────────────────

const ACCOUNT: &str = "acct-h4";

/// A JMAP upstream whose download endpoint labels each blob with the type named
/// by its blobId.
async fn spawn_upstream() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    let session = json!({
        "capabilities": { "urn:ietf:params:jmap:core": {}, "urn:ietf:params:jmap:mail": {} },
        "accounts": { ACCOUNT: { "name": "h4", "isPersonal": true, "isReadOnly": false } },
        "primaryAccounts": { "urn:ietf:params:jmap:mail": ACCOUNT },
        "username": "h4@example.org",
        "apiUrl": format!("{origin}/jmap"),
        "downloadUrl": format!("{origin}/download/{{accountId}}/{{blobId}}/{{name}}"),
        "uploadUrl": format!("{origin}/upload/{{accountId}}"),
        "eventSourceUrl": format!("{origin}/events"),
        "state": "s0"
    });
    let router = Router::new()
        .route(
            "/.well-known/jmap",
            get(move || {
                let session = session.clone();
                async move { axum::Json(session) }
            }),
        )
        .route(
            "/download/{a}/{b}/{c}",
            get(
                |UrlPath((_a, blob, _c)): UrlPath<(String, String, String)>| async move {
                    let ty = match blob.as_str() {
                        "html" => "text/html; charset=utf-8",
                        "xhtml" => "application/xhtml+xml",
                        _ => "image/png",
                    };
                    (
                        [
                            (header::CONTENT_TYPE, ty),
                            (header::CONTENT_DISPOSITION, "inline"),
                        ],
                        "<script>fetch('/api/me')</script>",
                    )
                        .into_response()
                },
            ),
        );
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    origin
}

#[tokio::test]
async fn proxy_mode_downloads_carry_the_same_headers() {
    let _g = serial().await;
    let upstream = spawn_upstream().await;
    let base = &spawn(
        ServerMode::Proxy,
        SecurityConfig {
            jmap_upstreams: Some(vec![upstream.clone()]),
            ..SecurityConfig::default()
        },
    )
    .await;
    let c = browser();
    let login = c
        .post(format!("{base}/api/login"))
        .json(&json!({ "username": "h4", "password": "pw" }))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200, "proxy login");

    for (blob, served) in [("html", OPAQUE), ("xhtml", OPAQUE), ("png", "image/png")] {
        let url = format!("{base}/jmap/download/{ACCOUNT}/{blob}/file");
        sandboxed_download(&c, &url, served).await;
    }
}
