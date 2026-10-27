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

use russh::{
    Channel,
    keys::{Algorithm, HashAlg, PrivateKey},
    server,
};
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
#[derive(Clone)]
struct Forward {
    port: u16,
    auth: Arc<AtomicUsize>,
    channels: Arc<AtomicUsize>,
}
impl server::Handler for Forward {
    type Error = russh::Error;
    async fn auth_password(
        &mut self,
        user: &str,
        password: &str,
    ) -> Result<server::Auth, Self::Error> {
        self.auth.fetch_add(1, Ordering::SeqCst);
        Ok(if user == "proof" && password == "ssh-test-secret" {
            server::Auth::Accept
        } else {
            server::Auth::reject()
        })
    }
    async fn channel_open_direct_tcpip(
        &mut self,
        channel: Channel<server::Msg>,
        host: &str,
        port: u32,
        _: &str,
        _: u32,
        reply: server::ChannelOpenHandle,
        _: &mut server::Session,
    ) -> Result<(), Self::Error> {
        self.channels.fetch_add(1, Ordering::SeqCst);
        if host != "127.0.0.1" || port != u32::from(self.port) {
            return Ok(());
        }
        let mut target = tokio::net::TcpStream::connect(("127.0.0.1", self.port))
            .await
            .unwrap();
        reply.accept().await;
        tokio::spawn(async move {
            let mut stream = channel.into_stream();
            let _ = tokio::io::copy_bidirectional(&mut stream, &mut target).await;
        });
        Ok(())
    }
}
async fn ssh(
    port: u16,
) -> (
    u16,
    Value,
    Arc<AtomicUsize>,
    Arc<AtomicUsize>,
    tokio::task::JoinHandle<()>,
) {
    let host = PrivateKey::random(&mut rand10::rng(), Algorithm::Ed25519).unwrap();
    let pin = json!({"algorithm":"ssh-ed25519","sha256":host.public_key().fingerprint(HashAlg::Sha256).to_string()});
    let config = Arc::new(server::Config {
        keys: vec![host],
        auth_rejection_time: Duration::ZERO,
        auth_rejection_time_initial: Some(Duration::ZERO),
        ..Default::default()
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let ssh_port = listener.local_addr().unwrap().port();
    let auth = Arc::new(AtomicUsize::new(0));
    let channels = Arc::new(AtomicUsize::new(0));
    let handler = Forward {
        port,
        auth: auth.clone(),
        channels: channels.clone(),
    };
    let task = tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let (config, handler) = (config.clone(), handler.clone());
            tokio::spawn(async move {
                if let Ok(session) = server::run_stream(config, stream, handler).await {
                    let _ = session.await;
                }
            });
        }
    });
    (ssh_port, pin, auth, channels, task)
}
async fn body(response: reqwest::Response, status: u16) -> (String, Value) {
    let actual = response.status();
    let etag = response
        .headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let text = response.text().await.unwrap();
    assert_eq!(actual, status, "{text}");
    let value = if text.is_empty() {
        Value::Null
    } else {
        serde_json::from_str(&text).unwrap()
    };
    (etag, value)
}
#[tokio::test]
async fn saved_tunnel_pins_and_https_authority_are_enforced_end_to_end() {
    let (port, records, panel_task) = panel().await;
    let (ssh_port, pin, auth, channels, ssh_task) = ssh(port).await;
    let dir = std::env::temp_dir().join(format!("mw-ssh-http-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let app = build_app_full(
        AppConfig {
            db_path: dir.join("test.db").to_string_lossy().into_owned(),
            server_key_hex: Some("13".repeat(32)),
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig::default(),
        },
        V6Config {
            admin_enabled: true,
            admin_username: Some("root".into()),
            admin_password: Some("ssh-admin-password".into()),
            redis_url: None,
        },
    )
    .await
    .unwrap()
    .0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let app_task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let client = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    let url = format!("{base}/admin/tunnels");
    assert_eq!(client.get(&url).send().await.unwrap().status(), 401);
    body(
        client
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"ssh-admin-password"}))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    let mut config = json!({"name":"SSH test","host":"127.0.0.1","port":ssh_port,"username":"proof","enabled":false,"allowPrivate":true,"hostKeyPins":[],"targets":[{"host":"localhost","port":port,"allowPrivate":true}],"keepaliveSeconds":30,"connectTimeoutMs":3000,"maxChannels":2,"credential":{"action":"set","kind":"password","password":"ssh-test-secret"}});
    assert_eq!(
        client
            .post(&url)
            .json(&config)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    body(
        client
            .post(format!("{base}/admin/step-up"))
            .json(&json!({"password":"ssh-admin-password"}))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    // Discover an unsaved authority without credentials or saved trust.
    let inspection =
        json!({"host":"127.0.0.1","port":ssh_port,"allowPrivate":true,"connectTimeoutMs":3000});
    let (_, discovered) = body(
        client
            .post(format!("{url}/inspect-host-key"))
            .json(&inspection)
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    assert_eq!(discovered, pin);
    assert_eq!(auth.load(Ordering::SeqCst), 0);
    let (_, unsaved) = body(client.get(&url).send().await.unwrap(), 200).await;
    assert!(unsaved["tunnels"].as_array().unwrap().is_empty());
    let mut refused = inspection.clone();
    refused["allowPrivate"] = json!(false);
    assert_eq!(
        client
            .post(format!("{url}/inspect-host-key"))
            .json(&refused)
            .send()
            .await
            .unwrap()
            .status(),
        409
    );
    let (etag, row) = body(client.post(&url).json(&config).send().await.unwrap(), 201).await;
    assert!(!row.to_string().contains("ssh-test-secret"));
    let saved = format!("{url}/{}", row["id"].as_str().unwrap());
    let (_, observed) = body(
        client
            .post(format!("{saved}/fetch-host-key"))
            .json(&json!({}))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    assert_eq!(observed, pin);
    assert_eq!(auth.load(Ordering::SeqCst), 0);
    config["enabled"] = json!(true);
    config["hostKeyPins"] = json!([pin]);
    config["credential"] = json!({"action":"keep"});
    assert_eq!(
        client
            .put(&saved)
            .header("If-Match", &etag)
            .json(&config)
            .send()
            .await
            .unwrap()
            .status(),
        422
    );
    config["confirmAuthorityChange"] = json!(true);
    let (etag, _) = body(
        client
            .put(&saved)
            .header("If-Match", etag)
            .json(&config)
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    assert_eq!(
        client
            .put(&saved)
            .header("If-Match", "\"stale\"")
            .json(&config)
            .send()
            .await
            .unwrap()
            .status(),
        412
    );
    let (_, probe) = body(
        client
            .post(format!("{saved}/test"))
            .json(&json!({}))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    assert_eq!(probe["ok"], true);
    assert_eq!(auth.load(Ordering::SeqCst), 1);
    assert_eq!(channels.load(Ordering::SeqCst), 1);
    let panel_url = format!("{base}/admin/integrations/hosting/connections");
    let mut panel_config = json!({"id":"","name":"Through SSH","provider":"hestia","baseUrl":format!("https://localhost:{port}"),"enabled":true,"username":"","owner":"owner","accessKey":"public-test","allowPrivate":true,"connection":{"kind":"ssh","tunnelId":row["id"]},"tls":{"caPem":CERT,"serverName":null,"minVersion":"1.2"},"allowDomainLifecycle":false});
    let(panel_etag,panel_row)=body(client.post(&panel_url).json(&json!({"config":panel_config,"credential":{"action":"set","value":"panel-secret"}})).send().await.unwrap(),201).await;
    let (_,role)=body(client.post(format!("{base}/admin/roles")).json(&json!({"name":"Panel editor without tunnel binding","capabilities":["integrations.read","integrations.write","tunnels.read"]})).send().await.unwrap(),201).await;
    body(client.post(format!("{base}/admin/admins")).json(&json!({"username":"limited","password":"limited-ssh-editor-password","roles":[{"roleId":role["id"],"domains":null}]})).send().await.unwrap(),201).await;
    let limited = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    body(
        limited
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"limited","password":"limited-ssh-editor-password"}))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    body(
        limited
            .post(format!("{base}/admin/step-up"))
            .json(&json!({"password":"limited-ssh-editor-password"}))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    assert_eq!(
        limited
            .post(&panel_url)
            .json(
                &json!({"config":panel_config,"credential":{"action":"set","value":"panel-secret"}})
            )
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    let panel_saved = format!("{panel_url}/{}", panel_row["id"].as_str().unwrap());
    let action = format!("{panel_saved}/actions");
    let (_, reply) = body(
        client
            .post(&action)
            .header("If-Match", &panel_etag)
            .json(&json!({"operation":{"action":"listDomains"}}))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    assert!(!reply.to_string().contains("must-not-expose"));
    assert_eq!(records.lock().unwrap().len(), 1);
    assert_eq!(
        auth.load(Ordering::SeqCst),
        1,
        "pooled authenticated SSH session reused"
    );
    assert_eq!(channels.load(Ordering::SeqCst), 2);
    assert_eq!(
        client
            .delete(&saved)
            .header("If-Match", &etag)
            .send()
            .await
            .unwrap()
            .status(),
        409
    );
    // The hosting permission is intersected with the tunnel's private target permission.
    panel_config["id"] = panel_row["id"].clone();
    panel_config["allowPrivate"] = json!(false);
    let (panel_etag, _) = body(
        client
            .put(&panel_saved)
            .header("If-Match", panel_etag)
            .json(&json!({"config":panel_config,"credential":{"action":"keep"}}))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    assert_eq!(
        client
            .post(&action)
            .header("If-Match", &panel_etag)
            .json(&json!({"operation":{"action":"listDomains"}}))
            .send()
            .await
            .unwrap()
            .status(),
        409
    );
    assert_eq!(channels.load(Ordering::SeqCst), 2);
    // A validly shaped but unapproved host key must fail before password authentication.
    config["hostKeyPins"][0]["sha256"] =
        json!("SHA256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
    let (etag, _) = body(
        client
            .put(&saved)
            .header("If-Match", etag)
            .json(&config)
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    let (_, probe) = body(
        client
            .post(format!("{saved}/test"))
            .json(&json!({}))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    assert_eq!(probe["ok"], false);
    assert_eq!(probe["code"], "host_key_mismatch");
    assert_eq!(auth.load(Ordering::SeqCst), 1);
    body(
        client
            .delete(&panel_saved)
            .header("If-Match", panel_etag)
            .send()
            .await
            .unwrap(),
        204,
    )
    .await;
    body(
        client
            .delete(&saved)
            .header("If-Match", etag)
            .send()
            .await
            .unwrap(),
        204,
    )
    .await;
    app_task.abort();
    ssh_task.abort();
    panel_task.abort();
}
