//! Real admin CRUD and a controlled HTTPS panel, including authority and retry guards.
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
const CERT: &str = include_str!("fixtures/tls/localhost-cert.pem");
async fn panel() -> (u16, Arc<Mutex<Vec<String>>>, tokio::task::JoinHandle<()>) {
    use rustls::pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
    let cert = CertificateDer::from_pem_slice(CERT.as_bytes()).unwrap();
    let key =
        PrivateKeyDer::from_pem_slice(include_bytes!("fixtures/tls/localhost-key.pem")).unwrap();
    let config = rustls::ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .unwrap()
    .with_no_client_auth()
    .with_single_cert(vec![cert], key)
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(config));
    let records = Arc::new(Mutex::new(Vec::new()));
    let observed = records.clone();
    let task = tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let acceptor = acceptor.clone();
            let observed = observed.clone();
            tokio::spawn(async move {
                let Ok(tls) = acceptor.accept(stream).await else {
                    return;
                };
                let mut reader = tokio::io::BufReader::new(tls);
                let mut head = String::new();
                let mut length = 0;
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
                        return;
                    }
                    if line == "\r\n" {
                        break;
                    }
                    if let Some(value) = line.to_lowercase().strip_prefix("content-length:") {
                        length = value.trim().parse::<usize>().unwrap();
                    }
                    head.push_str(&line);
                }
                let mut payload = vec![0; length];
                reader.read_exact(&mut payload).await.unwrap();
                let body = String::from_utf8(payload).unwrap();
                head.push_str(&body);
                observed.lock().unwrap().push(head);
                let (status, response) = if body.contains("v-list-mail-domains") {
                    (
                        "200 OK",
                        r#"{"example.test":{"SUSPENDED":"no","PASSWORD":"must-not-expose-provider-hash"}}"#,
                    )
                } else {
                    ("503 Service Unavailable", "provider-lost-confirmation")
                };
                let response = format!(
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{response}",
                    response.len()
                );
                let _ = reader.get_mut().write_all(response.as_bytes()).await;
            });
        }
    });
    (port, records, task)
}
#[tokio::test]
async fn hosting_crud_seals_credentials_and_binds_remote_actions_to_reviewed_authority() {
    let (port, records, panel_task) = panel().await;
    let dir = std::env::temp_dir().join(format!("mw-hosting-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let app = build_app_full(
        AppConfig {
            db_path: dir.join("hosting.db").to_string_lossy().into_owned(),
            server_key_hex: Some("12".repeat(32)),
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig::default(),
        },
        V6Config {
            admin_enabled: true,
            admin_username: Some("root".into()),
            admin_password: Some("hosting-admin-password".into()),
            redis_url: None,
        },
    )
    .await
    .unwrap()
    .0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .no_proxy()
        .build()
        .unwrap();
    let path = format!("{base}/admin/integrations/hosting/connections");
    assert_eq!(client.get(&path).send().await.unwrap().status(), 401);
    assert_eq!(
        client
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"hosting-admin-password"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let mut config = json!({"id":"","name":"Test panel","provider":"hestia","baseUrl":format!("https://127.0.0.1:{port}"),"enabled":true,"username":"","owner":"owner","accessKey":"public-test","allowPrivate":true,"connection":{"kind":"direct"},"tls":{"serverName":"localhost","caPem":CERT,"minVersion":"1.2"},"allowDomainLifecycle":false});
    let input = json!({"config":config,"credential":{"action":"set","value":"panel-secret"}});
    assert_eq!(
        client
            .post(&path)
            .json(&input)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        client
            .post(format!("{base}/admin/step-up"))
            .json(&json!({"password":"hosting-admin-password"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let role_response = client.post(format!("{base}/admin/roles")).json(&json!({"name":"Panel readers","capabilities":["integrations.read","integrations.write"]})).send().await.unwrap();
    assert_eq!(role_response.status(), 201);
    let role: Value = role_response.json().await.unwrap();
    for (name, roles) in [
        (
            "scoped",
            json!([{"roleId":role["id"],"domains":["example.org"]}]),
        ),
        ("no-grants", json!([])),
    ] {
        assert_eq!(
            client
                .post(format!("{base}/admin/admins"))
                .json(&json!({"username":name,"password":"long test password","roles":roles}))
                .send()
                .await
                .unwrap()
                .status(),
            201
        );
        let actor = reqwest::Client::builder()
            .cookie_store(true)
            .no_proxy()
            .build()
            .unwrap();
        assert_eq!(
            actor
                .post(format!("{base}/admin/login"))
                .json(&json!({"username":name,"password":"long test password"}))
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
        assert_eq!(
            actor.get(&path).send().await.unwrap().status(),
            403,
            "{name}"
        );
        assert_eq!(
            actor
                .post(&path)
                .json(&input)
                .send()
                .await
                .unwrap()
                .status(),
            403,
            "{name}"
        );
    }
    let response = client.post(&path).json(&input).send().await.unwrap();
    assert_eq!(response.status(), 201);
    assert_eq!(response.headers()["cache-control"], "private, no-store");
    let text = response.text().await.unwrap();
    assert!(!text.contains("panel-secret"));
    let mut row: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(row["credential"]["set"], true);
    config["id"] = row["id"].clone();
    let item = format!("{path}/{}", row["id"].as_str().unwrap());
    let action = format!("{item}/actions");
    let initial = row["etag"].as_str().unwrap().to_owned();
    assert_eq!(
        client
            .put(&item)
            .json(&json!({"config":config}))
            .send()
            .await
            .unwrap()
            .status(),
        428
    );
    config["name"] = json!("Renamed panel");
    let response = client
        .put(&item)
        .header("If-Match", &initial)
        .json(&json!({"config":config}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    row = response.json().await.unwrap();
    let mut etag = row["etag"].as_str().unwrap().to_owned();
    assert_ne!(etag, initial);
    assert_eq!(
        client
            .put(&item)
            .header("If-Match", &initial)
            .json(&json!({"config":config}))
            .send()
            .await
            .unwrap()
            .status(),
        412
    );
    let mut retarget = config.clone();
    retarget["baseUrl"] = json!("https://other.example");
    assert_eq!(
        client
            .put(&item)
            .header("If-Match", &etag)
            .json(&json!({"config":retarget}))
            .send()
            .await
            .unwrap()
            .status(),
        422
    );
    let response = client
        .post(&action)
        .header("If-Match", &etag)
        .json(&json!({"operation":{"action":"listDomains"}}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let text = response.text().await.unwrap();
    assert!(!text.contains("must-not-expose-provider-hash"));
    let inventory: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(inventory["domains"][0]["name"], "example.test");
    assert_eq!(records.lock().unwrap().len(), 1);
    assert_eq!(client.post(format!("{base}/admin/egress/proxies")).json(&json!({"id":"hosting-proxy","scheme":"http","host":"127.0.0.1","port":9,"allowPlaintext":true})).send().await.unwrap().status(),200);
    assert_eq!(
        client
            .post(format!(
                "{base}/admin/egress/proxies/hosting-proxy/activate"
            ))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let refused = client
        .post(&action)
        .json(&json!({"operation":{"action":"listDomains"}}))
        .send()
        .await
        .unwrap();
    assert_eq!(refused.status(), 503);
    assert_eq!(
        refused.json::<Value>().await.unwrap()["code"],
        "unsupported-route"
    );
    assert_eq!(
        records.lock().unwrap().len(),
        1,
        "active proxy must never silently fall back to direct"
    );
    assert_eq!(
        client
            .post(format!("{base}/admin/egress/proxies/deactivate"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );

    assert_eq!(
        client
            .post(&action)
            .header("If-Match", &etag)
            .json(&json!({"operation":{"action":"deleteDomain","domain":"example.test"}}))
            .send()
            .await
            .unwrap()
            .status(),
        422
    );
    assert_eq!(records.lock().unwrap().len(), 1);
    let response=client.post(&action).header("If-Match",&etag).json(&json!({"operation":{"action":"deleteDomain","domain":"example.test"},"confirmation":"example.test"})).send().await.unwrap();
    assert_eq!(response.status(), 502);
    let result: Value = response.json().await.unwrap();
    assert_eq!(result["resultUncertain"], true);
    assert_eq!(records.lock().unwrap().len(), 2, "mutation must not retry");
    config["allowPrivate"] = json!(false);
    row = client
        .put(&item)
        .header("If-Match", &etag)
        .json(&json!({"config":config}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    etag = row["etag"].as_str().unwrap().to_owned();
    let response = client
        .post(&action)
        .json(&json!({"operation":{"action":"listDomains"}}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 502);
    assert_eq!(
        records.lock().unwrap().len(),
        2,
        "private target must not receive credentials without opt-in"
    );
    config["enabled"] = json!(false);
    row = client
        .put(&item)
        .header("If-Match", &etag)
        .json(&json!({"config":config,"credential":{"action":"clear"}}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    etag = row["etag"].as_str().unwrap().to_owned();
    assert_eq!(row["credential"]["set"], false);
    assert_eq!(
        client
            .post(&action)
            .json(&json!({"operation":{"action":"listDomains"}}))
            .send()
            .await
            .unwrap()
            .status(),
        409
    );
    assert_eq!(
        client
            .delete(&item)
            .header("If-Match", &initial)
            .send()
            .await
            .unwrap()
            .status(),
        412
    );
    assert_eq!(
        client
            .delete(&item)
            .header("If-Match", &etag)
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    assert_eq!(client.get(&item).send().await.unwrap().status(), 404);
    task.abort();
    panel_task.abort();
}
