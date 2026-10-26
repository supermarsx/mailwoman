//! Real EWS guest, guarded HTTPS transport and domain-selected enrollment.
use mw_server::{AppConfig, SecurityConfig, ServerMode, build_app};
use mw_store::{DomainConfigRow, ServerKey, Store};
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
mod common;
const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
const HIER: &str = include_str!("../../../plugins/bridge-ews/fixtures/sync_folder_hierarchy.xml");
const ITEMS: &str = include_str!("../../../plugins/bridge-ews/fixtures/sync_folder_items.xml");
const ITEM: &str = include_str!("../../../plugins/bridge-ews/fixtures/get_item_mime.xml");

async fn exchange() -> (
    u16,
    String,
    Arc<AtomicUsize>,
    Arc<AtomicUsize>,
    tokio::task::JoinHandle<()>,
) {
    use rustls::pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
    let certified = rcgen::generate_simple_self_signed(vec!["8.8.8.8".into()]).unwrap();
    let ca = certified.cert.pem();
    let tls = rustls::ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .unwrap()
    .with_no_client_auth()
    .with_single_cert(
        vec![certified.cert.der().clone()],
        PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
            certified.signing_key.serialize_der(),
        )),
    )
    .unwrap();
    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(tls));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let count = Arc::new(AtomicUsize::new(0));
    let observed = count.clone();
    let proxy_attempts = Arc::new(AtomicUsize::new(0));
    let proxy_observed = proxy_attempts.clone();
    let task = tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let acceptor = acceptor.clone();
            let observed = observed.clone();
            let proxy_observed = proxy_observed.clone();
            tokio::spawn(async move {
                let mut outer = tokio::io::BufReader::new(stream);
                let mut first = String::new();
                outer.read_line(&mut first).await.unwrap();
                assert_eq!(first, "CONNECT 8.8.8.8:443 HTTP/1.1\r\n");
                loop {
                    let mut line = String::new();
                    if outer.read_line(&mut line).await.unwrap_or(0) == 0 {
                        return;
                    }
                    if line == "\r\n" {
                        break;
                    }
                }
                outer
                    .get_mut()
                    .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                    .await
                    .unwrap();
                let Ok(tls) = acceptor.accept(outer.into_inner()).await else {
                    return;
                };
                let mut reader = tokio::io::BufReader::new(tls);
                let mut request_line = String::new();
                if reader.read_line(&mut request_line).await.unwrap_or(0) == 0 {
                    return;
                }
                if request_line.split_whitespace().nth(1) == Some("/proxy-forbidden/EWS.asmx") {
                    proxy_observed.fetch_add(1, Ordering::SeqCst);
                }
                let mut length = 0usize;
                let mut authenticated = false;
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
                        return;
                    }
                    if line == "\r\n" {
                        break;
                    }
                    if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                        length = value.trim().parse().unwrap();
                    }
                    if let Some((name, value)) = line.split_once(':')
                        && name.eq_ignore_ascii_case("authorization")
                    {
                        use base64::Engine;
                        authenticated = value.trim()
                            == format!(
                                "Basic {}",
                                base64::engine::general_purpose::STANDARD
                                    .encode("alice@example.test:testpass")
                            );
                    }
                }
                assert!(length <= 1024 * 1024);
                let mut bytes = vec![0; length];
                reader.read_exact(&mut bytes).await.unwrap();
                observed.fetch_add(1, Ordering::SeqCst);
                let body = String::from_utf8(bytes).unwrap();
                let (status, payload) = if !authenticated {
                    ("401 Unauthorized", "")
                } else if body.contains("SyncFolderHierarchy") {
                    ("200 OK", HIER)
                } else if body.contains("SyncFolderItems") {
                    ("200 OK", ITEMS)
                } else if body.contains("GetItem") {
                    ("200 OK", ITEM)
                } else {
                    ("200 OK", "<soap:Envelope/>")
                };
                let response = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: text/xml\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
                    payload.len()
                );
                let _ = reader.get_mut().write_all(response.as_bytes()).await;
            });
        }
    });
    (port, ca, count, proxy_attempts, task)
}

fn domain(path: &str, ca: Option<&str>) -> DomainConfigRow {
    DomainConfigRow {
        name: "example.test".into(),
        enabled: true,
        preferred: false,
        upstream_json:
            json!({"version":2,"backends":[{"id":"exchange","role":"mail","protocol":"ews","ews":{
                "endpointUrl":format!("https://8.8.8.8/{path}"),"authentication":"basic","caPem":ca
            }}]})
            .to_string(),
        allowlist_json: "[]".into(),
        blocklist_json: "[]".into(),
        settings_json: "{}".into(),
        updated_at: 1,
    }
}
async fn app(db: &str, mode: ServerMode) -> String {
    let app = build_app(AppConfig {
        db_path: db.into(),
        server_key_hex: Some(KEY.into()),
        mode,
        cookie_secure: false,
        web_dir: None,
        hardening: Default::default(),
        security: SecurityConfig::default(),
    })
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    base
}
async fn login(client: &reqwest::Client, base: &str, password: &str) -> reqwest::Response {
    client
        .post(format!("{base}/api/login"))
        .json(&json!({"username":"alice@example.test","password":password}))
        .send()
        .await
        .unwrap()
}

async fn jmap(client: &reqwest::Client, base: &str, method: &str, args: Value) -> Value {
    let response = client
        .post(format!("{base}/jmap/api"))
        .json(
            &json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],
            "methodCalls":[[method,args,"proof"]]}),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let result: Value = response.json().await.unwrap();
    assert_eq!(result["methodResponses"][0][0], method, "{result}");
    result["methodResponses"][0][1].clone()
}

async fn indexed_mail(client: &reqwest::Client, base: &str, account: &str) {
    let mut last = Value::Null;
    let completed = tokio::time::timeout(std::time::Duration::from_secs(30), async {
        loop {
            let boxes = jmap(client, base, "Mailbox/get", json!({"accountId":account})).await;
            assert_ne!(boxes["mailwomanInitialSync"], "failed", "{boxes}");
            let inbox = boxes["list"]
                .as_array()
                .unwrap()
                .iter()
                .find(|mailbox| mailbox["role"] == "inbox")
                .unwrap()["id"]
                .clone();
            let mail = jmap(
                client,
                base,
                "Email/query",
                json!({"accountId":account,"filter":{"inMailbox":inbox}}),
            )
            .await;
            if boxes["mailwomanInitialSync"] == "ready"
                && mail["ids"].as_array().is_some_and(|ids| !ids.is_empty())
            {
                let messages = jmap(
                    client,
                    base,
                    "Email/get",
                    json!({"accountId":account,"ids":mail["ids"],"fetchTextBodyValues":true}),
                )
                .await;
                let message = &messages["list"][0];
                assert_eq!(message["subject"], "Quarterly numbers", "{messages}");
                assert!(
                    message["bodyValues"]
                        .as_object()
                        .is_some_and(|bodies| bodies.values().any(|body| body["value"]
                            .as_str()
                            .is_some_and(|text| text.contains("quarterly numbers")))),
                    "{messages}"
                );
                return;
            }
            last = json!({"mailboxes":boxes,"query":mail});
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    })
    .await;
    assert!(
        completed.is_ok(),
        "EWS activation must finish actual indexing: {last}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn guarded_ews_login_requires_proof_and_exact_domain_authority() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .with_test_writer()
        .try_init();
    let (port, ca, requests, proxy_attempts, exchange_task) = exchange().await;
    let db = common::test_db::unique_dir("domain-ews")
        .join("mw.db")
        .to_string_lossy()
        .into_owned();
    let store = Store::open(&db, ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    let mut private = domain("EWS/Exchange.asmx", Some(&ca));
    private.upstream_json = private.upstream_json.replace("8.8.8.8", "127.0.0.1");
    store.put_domain_config(&private).await.unwrap();
    store
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: "signin.retryCooldownSecs".into(),
            global_value: Some("0".into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "fixture".into(),
            updated_at: 0,
        })
        .await
        .unwrap();
    store
        .put_plugin(&mw_store::PluginRow {
            id: "bridge-ews".into(),
            name: "EWS".into(),
            version: "1".into(),
            signature_hex: None,
            approved_by: Some("fixture".into()),
            enabled: true,
            capabilities_json: "[\"net\",\"account-backend\"]".into(),
            net_allowlist_json: "[]".into(),
            limits_json: "{\"memoryMb\":128,\"deadlineMs\":10000}".into(),
            created_at: "2026-10-26T00:00:00Z".into(),
        })
        .await
        .unwrap();
    for capability in ["net", "account-backend"] {
        store
            .put_plugin_grant(&mw_store::PluginGrantRow {
                plugin_id: "bridge-ews".into(),
                account_id: String::new(),
                capability: capability.into(),
                granted_by: "fixture".into(),
                created_at: "2026-10-26T00:00:00Z".into(),
            })
            .await
            .unwrap();
    }
    let base = app(&db, ServerMode::Engine).await;
    let client = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .timeout(std::time::Duration::from_secs(40))
        .build()
        .unwrap();
    // Neither an enabled plugin nor a CA grants access to a private destination.
    assert_eq!(login(&client, &base, "testpass").await.status(), 401);
    assert_eq!(requests.load(Ordering::SeqCst), 0);
    assert!(store.list_accounts().await.unwrap().is_empty());
    store
        .put_egress_proxy(&mw_store::EgressProxyRow {
            id: "ews-controlled-proxy".into(),
            kind: "http".into(),
            host: "127.0.0.1".into(),
            port,
            username: String::new(),
            password: None,
            allow_plaintext: false,
            active: true,
            tls_to_proxy: false,
            remote_dns: false,
            remote_dns_risk_accepted_by: None,
            remote_dns_risk_accepted_at: None,
            tunnel_id: None,
            chain_next_id: None,
            scope: "global".into(),
            traffic_classes: vec!["plugins".into()],
            created_at: "2026-10-26T00:00:00Z".into(),
            updated_at: "2026-10-26T00:00:00Z".into(),
        })
        .await
        .unwrap();
    assert!(
        store
            .set_active_egress_proxy(Some("ews-controlled-proxy"))
            .await
            .unwrap()
    );
    assert!(
        store
            .active_egress_proxy()
            .await
            .unwrap()
            .is_some_and(|route| route.id == "ews-controlled-proxy")
    );
    store
        .put_domain_config(&domain("EWS/Exchange.asmx", None))
        .await
        .unwrap();
    let before_tls = requests.load(Ordering::SeqCst);
    assert_eq!(login(&client, &base, "testpass").await.status(), 401);
    assert_eq!(
        requests.load(Ordering::SeqCst),
        before_tls,
        "untrusted TLS must fail before sending credentials"
    );
    store
        .put_domain_config(&domain("EWS/Exchange.asmx", Some(&ca)))
        .await
        .unwrap();
    assert_eq!(login(&client, &base, "wrong").await.status(), 401);
    assert!(
        requests.load(Ordering::SeqCst) > before_tls,
        "wrong password must be rejected by the actual HTTPS endpoint"
    );
    assert!(
        store.list_accounts().await.unwrap().is_empty(),
        "failed proof must not provision an account"
    );
    let response = login(&client, &base, "testpass").await;
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    let first: Value = response.json().await.unwrap();
    let id = first["accountId"].as_str().unwrap();
    assert!(
        first["storageAuthority"]
            .as_str()
            .unwrap()
            .starts_with("msa1_")
    );
    assert_eq!(
        store.get_account(id).await.unwrap().kind,
        mw_store::AccountKind::Ews
    );
    assert!(!store.list_mailboxes(id).await.unwrap().is_empty());
    indexed_mail(&client, &base, id).await;
    assert_eq!(
        client
            .get(format!("{base}/api/me"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let again: Value = login(&client, &base, "testpass")
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(again["accountId"], first["accountId"]);
    assert_eq!(again["storageAuthority"], first["storageAuthority"]);
    let native = reqwest::Client::builder().no_proxy().build().unwrap();
    let response = native
        .post(format!("{base}/api/login"))
        .json(&json!({"username":"alice@example.test","password":"testpass","clientType":"native"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let token: Value = response.json().await.unwrap();
    assert_eq!(token["accountId"], first["accountId"]);
    assert_eq!(token["storageAuthority"], first["storageAuthority"]);
    assert_eq!(
        native
            .get(format!("{base}/api/me"))
            .bearer_auth(token["token"].as_str().unwrap())
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    // A new application instance restores only the current verified binding;
    // an existing native session keeps the same account/private-storage identity.
    let restarted = app(&db, ServerMode::Engine).await;
    let resumed = native
        .get(format!("{restarted}/api/me"))
        .bearer_auth(token["token"].as_str().unwrap())
        .send()
        .await
        .unwrap();
    assert_eq!(resumed.status(), 200);
    let resumed: Value = resumed.json().await.unwrap();
    assert_eq!(resumed["accountId"], first["accountId"]);
    assert_eq!(resumed["storageAuthority"], first["storageAuthority"]);
    store
        .put_totp_secret(id, b"controlled-ews-totp-secret", true)
        .await
        .unwrap();
    let fresh = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    let response = login(&fresh, &base, "testpass").await;
    assert_eq!(response.status(), 200);
    assert!(
        !response
            .headers()
            .get_all("set-cookie")
            .iter()
            .any(|cookie| cookie.to_str().unwrap().starts_with("mw_session="))
    );
    let challenge: Value = response.json().await.unwrap();
    assert_eq!(
        challenge["twofaRequired"], true,
        "provider sign-in must preserve enrolled factors"
    );
    assert_eq!(
        fresh
            .get(format!("{base}/api/me"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    store
        .put_domain_config(&domain("tenant-two/EWS.asmx", Some(&ca)))
        .await
        .unwrap();
    assert_eq!(
        client
            .get(format!("{base}/api/me"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    let other: Value = login(&client, &base, "testpass")
        .await
        .json()
        .await
        .unwrap();
    assert_ne!(other["accountId"], first["accountId"]);
    assert_ne!(other["storageAuthority"], first["storageAuthority"]);
    store
        .put_domain_config(&domain("proxy-forbidden/EWS.asmx", Some(&ca)))
        .await
        .unwrap();
    let proxy = app(&db, ServerMode::Proxy).await;
    assert_eq!(login(&client, &proxy, "testpass").await.status(), 404);
    assert_eq!(
        proxy_attempts.load(Ordering::SeqCst),
        0,
        "proxy mode must never issue a provider request; background engine requests use different paths"
    );
    exchange_task.abort();
}
