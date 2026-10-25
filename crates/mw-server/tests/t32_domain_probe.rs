//! Real HTTP admin probe coverage: saved targets, no credentials, private opt-in and rate bounds.
use mw_server::{AppConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
use std::{
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
mod common;
fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap()
}
async fn server() -> (String, reqwest::Client) {
    let (app, _) = build_app_full(
        AppConfig {
            db_path: common::test_db::unique_db_path("domain-probe")
                .to_string_lossy()
                .into(),
            server_key_hex: None,
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Engine,
            hardening: Default::default(),
            security: SecurityConfig::default(),
        },
        V6Config {
            admin_enabled: true,
            admin_username: Some("root".into()),
            admin_password: Some("hunter2".into()),
            redis_url: None,
        },
    )
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    let root = client();
    assert_eq!(
        root.post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"hunter2"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        root.post(format!("{base}/admin/step-up"))
            .json(&json!({"password":"hunter2"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    (base, root)
}
async fn imap() -> (u16, Arc<AtomicUsize>, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let counter = Arc::new(AtomicUsize::new(0));
    let seen = counter.clone();
    let task = tokio::spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            seen.fetch_add(1, Ordering::SeqCst);
            tokio::spawn(async move {
                let mut stream = BufReader::new(stream);
                stream
                    .get_mut()
                    .write_all(b"* OK fixture\r\n")
                    .await
                    .unwrap();
                let mut line = String::new();
                stream.read_line(&mut line).await.unwrap();
                assert_eq!(line, "a2 CAPABILITY\r\n", "probe never authenticates");
                stream
                    .get_mut()
                    .write_all(b"* CAPABILITY IMAP4rev1 AUTH=PLAIN\r\na2 OK done\r\n")
                    .await
                    .unwrap();
            });
        }
    });
    (port, counter, task)
}
#[tokio::test]
async fn probes_only_saved_operator_targets_without_credentials_and_bounds_cost() {
    let (base, root) = server().await;
    let (port, opened, task) = imap().await;
    let url = format!("{base}/admin/diag/upstream");
    for route in ["upstream", "dns"] {
        assert_eq!(
            client()
                .post(format!("{base}/admin/diag/{route}"))
                .json(&json!({"domain":"example.test"}))
                .send()
                .await
                .unwrap()
                .status(),
            401
        );
    }
    let response = root.put(format!("{base}/admin/domains/example.test")).json(&json!({"upstream":{"imap":{"host":"127.0.0.1","port":port,"tls":"none","auth":"password","allowPrivate":true}}})).send().await.unwrap();
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    assert_eq!(
        root.post(&url)
            .json(&json!({"domain":"example.test","host":"169.254.169.254"}))
            .send()
            .await
            .unwrap()
            .status(),
        422
    );
    let response = root
        .post(&url)
        .json(&json!({"domain":"example.test"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["credentialsSent"], false);
    assert_eq!(body["results"][0]["reachable"], true);
    assert!(
        body["results"][0]["capabilities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str().unwrap().contains("IMAP4rev1"))
    );
    assert_eq!(opened.load(Ordering::SeqCst), 1);
    assert!(
        mw_server::domain_probe::cached_reports()
            .iter()
            .any(|report| report["domain"] == "example.test")
    );
    // Hostnames are rechecked after DNS, not merely checked as string literals at save time.
    let response = root.put(format!("{base}/admin/domains/example.test")).json(&json!({"upstream":{"imap":{"host":"localhost.","port":port,"tls":"none","auth":"password","allowPrivate":false}}})).send().await.unwrap();
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    for _ in 0..5 {
        let response = root
            .post(&url)
            .json(&json!({"domain":"example.test"}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        let body: Value = response.json().await.unwrap();
        assert_eq!(body["results"][0]["reachable"], false);
    }
    assert_eq!(
        root.post(&url)
            .json(&json!({"domain":"example.test"}))
            .send()
            .await
            .unwrap()
            .status(),
        429
    );
    assert_eq!(
        opened.load(Ordering::SeqCst),
        1,
        "denied DNS answers never open a socket"
    );
    task.abort();
}
#[tokio::test]
async fn diagnostic_permission_does_not_grant_upstream_network_authority() {
    let (base, root) = server().await;
    assert_eq!(
        root.put(format!("{base}/admin/domains/example.test"))
            .json(&json!({}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let role: Value = root
        .post(format!("{base}/admin/roles"))
        .json(&json!({"name":"Probe viewer", "capabilities":["diagnostics.run"]}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let response = root.post(format!("{base}/admin/admins")).json(&json!({"username":"probeviewer","password":"long probe password","roles":[{"roleId":role["id"]}]})).send().await.unwrap();
    assert_eq!(response.status(), 201, "{}", response.text().await.unwrap());
    let viewer = client();
    assert_eq!(
        viewer
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"probeviewer","password":"long probe password"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    for route in ["upstream", "dns"] {
        let response = viewer
            .post(format!("{base}/admin/diag/{route}"))
            .json(&json!({"domain":"example.test"}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 403);
    }
}
