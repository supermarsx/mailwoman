//! Opt-in disposable real HTTP fixture for delivery-report browser acceptance.
use axum::{
    body::{Body, to_bytes},
    extract::Request,
    middleware::Next,
    response::Response,
};
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
mod common;
const RAW: &str = "From: daemon@example.org\r\nTo: testuser@example.org\r\nSubject: Delivery failure fixture\r\nContent-Type: multipart/report; report-type=delivery-status; boundary=b\r\n\r\n--b\r\nContent-Type: text/plain\r\n\r\nDelivery failed after retries.\r\n--b\r\nContent-Type: message/delivery-status\r\n\r\nReporting-MTA: dns; mx.example.org\r\n\r\nFinal-Recipient: rfc822; missing@example.org\r\nAction: failed\r\nStatus: 4.4.7\r\nDiagnostic-Code: smtp; retries exhausted\r\n--b--\r\n";
async fn upstream(req: Request, next: Next) -> Response {
    let download =
        req.uri().path().contains("/download/") && req.uri().path().contains("/blob-e1/");
    let response = next.run(req).await;
    if !response.status().is_success() {
        return response;
    }
    let (mut parts, body) = response.into_parts();
    let raw = to_bytes(body, 4 * 1024 * 1024).await.unwrap();
    let body = if download {
        parts
            .headers
            .insert("content-type", "message/rfc822".parse().unwrap());
        RAW.as_bytes().to_vec()
    } else if let Ok(mut value) = serde_json::from_slice::<Value>(&raw) {
        if let Some(calls) = value["methodResponses"].as_array_mut() {
            for call in calls {
                if call[0] == "Email/get"
                    && let Some(list) = call[1]["list"].as_array_mut()
                {
                    for email in list {
                        if email["id"] == "e1" {
                            email["subject"] = json!("Delivery failure fixture");
                            email["size"] = json!(RAW.len());
                        }
                    }
                }
            }
        }
        serde_json::to_vec(&value).unwrap()
    } else {
        raw.to_vec()
    };
    parts.headers.remove("content-length");
    Response::from_parts(parts, Body::from(body))
}
#[tokio::test]
#[ignore = "manual persistent browser fixture, requires MW_DSN_WEB and Greenmail"]
async fn serve_delivery_report_browser_fixture() {
    // A manual fixture that serves until it is killed: `--ignored` runs in CI reach
    // it, and without its variable it must report a skip, not panic or start.
    let Some(web) = std::env::var_os("MW_DSN_WEB").filter(|v| !v.is_empty()) else {
        common::gate::skip("[dsn browser fixture] MW_DSN_WEB unset — manual fixture not started.");
        return;
    };
    let web = std::path::PathBuf::from(web);
    assert!(web.join("index.html").is_file());
    let mock = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mock_url = format!("http://{}", mock.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(
            mock,
            mw_mock_jmap::router().layer(axum::middleware::from_fn(upstream)),
        )
        .await
        .unwrap();
    });
    for (port, mode, upstream) in [
        (
            42209,
            ServerMode::Engine,
            "imap://127.0.0.1:43202".to_owned(),
        ),
        (42210, ServerMode::Proxy, mock_url),
    ] {
        let path = common::test_db::unique_db_path(&format!("dsn-browser-{port}"));
        let key = "1111111111111111111111111111111111111111111111111111111111111111";
        let app = build_app_full(
            AppConfig {
                db_path: path.to_string_lossy().into(),
                server_key_hex: Some(key.into()),
                web_dir: Some(web.clone()),
                cookie_secure: false,
                mode,
                hardening: HardeningConfig {
                    csrf_strict: true,
                    ..Default::default()
                },
                security: SecurityConfig {
                    jmap_upstreams: Some(vec![upstream.clone()]),
                    upstream_default: Some(upstream),
                    ..Default::default()
                },
            },
            V6Config {
                admin_enabled: false,
                admin_username: None,
                admin_password: None,
                redis_url: None,
            },
        )
        .await
        .unwrap()
        .0;
        let store = mw_store::Store::open(
            &path.to_string_lossy(),
            mw_store::ServerKey::from_hex(key).unwrap(),
        )
        .await
        .unwrap();
        store
            .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
                key: "signin.retryCooldownSecs".into(),
                global_value: Some("0".into()),
                allow_user_override: false,
                global_locked: false,
                updated_by: "fixture".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
            .await
            .unwrap();
        tokio::spawn(async move {
            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .await
            .unwrap();
        });
        println!("DSN browser fixture ready {port}");
    }
    std::future::pending::<()>().await;
}
