use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
mod common;

async fn serve(router: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    format!("http://{address}")
}
type CloudRequests = Arc<Mutex<Vec<(String, Vec<u8>)>>>;
struct CloudFixture {
    port: u16,
    records: CloudRequests,
    reject: Arc<AtomicBool>,
}
async fn cloud_proxy() -> CloudFixture {
    use rustls_pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
    let cert =
        CertificateDer::from_pem_slice(include_bytes!("fixtures/tls/nextcloud-cert.pem")).unwrap();
    let key =
        PrivateKeyDer::from_pem_slice(include_bytes!("fixtures/tls/nextcloud-key.pem")).unwrap();
    let config = rustls::ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .unwrap()
    .with_no_client_auth()
    .with_single_cert(vec![cert], key)
    .unwrap();
    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(config));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let records = Arc::new(Mutex::new(Vec::new()));
    let output = records.clone();
    let reject = Arc::new(AtomicBool::new(false));
    let reject_output = reject.clone();
    tokio::spawn(async move {
        while let Ok((socket, _)) = listener.accept().await {
            let acceptor = acceptor.clone();
            let records = records.clone();
            let reject = reject.clone();
            tokio::spawn(async move {
                let mut socket = BufReader::new(socket);
                let mut first = String::new();
                socket.read_line(&mut first).await.unwrap();
                assert_eq!(first, "CONNECT 8.8.8.8:443 HTTP/1.1\r\n");
                loop {
                    let mut line = String::new();
                    socket.read_line(&mut line).await.unwrap();
                    if line == "\r\n" {
                        break;
                    }
                }
                socket
                    .get_mut()
                    .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                    .await
                    .unwrap();
                let Ok(stream) = acceptor.accept(socket.into_inner()).await else {
                    return;
                };
                let mut stream = BufReader::new(stream);
                let mut head = String::new();
                loop {
                    let mut line = String::new();
                    match stream.read_line(&mut line).await {
                        Ok(0) | Err(_) => return,
                        _ => {}
                    }
                    head.push_str(&line);
                    if line == "\r\n" {
                        break;
                    }
                }
                let lower = head.to_ascii_lowercase();
                let length = lower
                    .lines()
                    .find_map(|line| {
                        line.strip_prefix("content-length:")
                            .and_then(|value| value.trim().parse::<usize>().ok())
                    })
                    .unwrap_or(0);
                assert!(length <= 8388608);
                let mut bytes = vec![0; length];
                stream.read_exact(&mut bytes).await.unwrap();
                let method = head.split_whitespace().next().unwrap().to_owned();
                records.lock().unwrap().push((method.clone(), bytes));
                use base64::Engine as _;
                let valid = head.lines().any(|line| {
                    line.eq_ignore_ascii_case(&format!(
                        "authorization: Basic {}",
                        base64::engine::general_purpose::STANDARD.encode("alice:cloud-password")
                    ))
                });
                let (status, body) = if !valid || reject.load(Ordering::SeqCst) {
                    ("401 Unauthorized", String::new())
                } else if method == "PROPFIND" {
                    let file = if lower.contains("depth: 1") {
                        r#"<d:response><d:href>/nextcloud/remote.php/dav/files/alice/R%C3%A9sum%C3%A9.txt</d:href><d:propstat><d:prop><d:getcontentlength>10</d:getcontentlength><d:getcontenttype>text/plain</d:getcontenttype></d:prop><d:status>HTTP/1.1 200 OK</d:status></d:propstat></d:response>"#
                    } else {
                        ""
                    };
                    (
                        "207 Multi-Status",
                        format!(
                            r#"<d:multistatus xmlns:d="DAV:"><d:response><d:href>/nextcloud/remote.php/dav/files/alice/</d:href><d:propstat><d:prop><d:resourcetype><d:collection/></d:resourcetype></d:prop></d:propstat></d:response>{file}</d:multistatus>"#
                        ),
                    )
                } else if method == "PUT" {
                    ("201 Created", String::new())
                } else {
                    ("200 OK", "cloud-file".into())
                };
                let response = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: application/xml\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.get_mut().write_all(response.as_bytes()).await;
            });
        }
    });
    CloudFixture {
        port,
        records: output,
        reject: reject_output,
    }
}
async fn policy(pool: &sqlx::SqlitePool, key: &str, value: Value) {
    sqlx::query("INSERT INTO setting_policy(key,global_value,updated_by,updated_at) VALUES(?1,?2,'test',0) ON CONFLICT(key) DO UPDATE SET global_value=excluded.global_value").bind(key).bind(value.to_string()).execute(pool).await.unwrap();
}

#[tokio::test]
async fn approved_ca_account_link_files_and_attachment_policy_are_enforced_end_to_end() {
    let upstream = serve(mw_mock_jmap::router()).await;
    let db = common::test_db::unique_db_path("nextcloud");
    let (app, _) = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into(),
            server_key_hex: None,
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig {
                upstream_default: Some(upstream.clone()),
                jmap_upstreams: Some(vec![upstream]),
                ..Default::default()
            },
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
    let base = serve(app).await;
    let pool = sqlx::SqlitePool::connect(&format!("sqlite:{}", db.display()))
        .await
        .unwrap();
    let admin = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    assert_eq!(
        admin
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"hunter2"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let body = json!({"name":"Fixture cloud","baseUrl":"https://8.8.8.8/nextcloud/","enabled":true,"caPem":include_str!("fixtures/tls/nextcloud-cert.pem")});
    assert_eq!(
        admin
            .put(format!("{base}/admin/nextcloud/instances/cloud"))
            .header("if-none-match", "*")
            .json(&body)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        admin
            .post(format!("{base}/admin/step-up"))
            .json(&json!({"password":"hunter2"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let created = admin
        .put(format!("{base}/admin/nextcloud/instances/cloud"))
        .header("if-none-match", "*")
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(created.status(), 200, "{}", created.text().await.unwrap());
    let CloudFixture {
        port,
        records,
        reject,
    } = cloud_proxy().await;
    mw_egress::plan::install(
        mw_egress::plan::EgressPlan::new(
            vec![mw_egress::plan::PlanRoute {
                scope: mw_egress::plan::RouteScope::Global,
                classes: [mw_egress::TrafficClass::Nextcloud].into_iter().collect(),
                route: mw_egress::proxy::ProxyRoute {
                    id: "fixture".into(),
                    scheme: mw_egress::proxy::ProxyScheme::HttpConnect,
                    host: "127.0.0.1".into(),
                    port,
                    auth: None,
                    allow_plaintext: false,
                },
            }],
            mw_egress::InternalAllowList::empty(),
        )
        .unwrap(),
    );
    tokio::time::sleep(std::time::Duration::from_secs(4)).await;
    let user = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    let login = user
        .post(format!("{base}/api/login"))
        .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200);
    let account = login.json::<Value>().await.unwrap()["accountId"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(
        user.put(format!("{base}/api/account/nextcloud/links/cloud"))
            .json(&json!({"accountId":"foreign","username":"alice","appPassword":"cloud-password"}))
            .send()
            .await
            .unwrap()
            .status(),
        409
    );
    assert!(records.lock().unwrap().is_empty());
    let bad = user
        .put(format!("{base}/api/account/nextcloud/links/cloud"))
        .json(&json!({"accountId":account,"username":"alice","appPassword":"wrong"}))
        .send()
        .await
        .unwrap();
    assert_eq!(bad.status(), 502);
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    assert_eq!(
        records.lock().unwrap().len(),
        1,
        "bad credentials were replayed"
    );
    let linked = user
        .put(format!("{base}/api/account/nextcloud/links/cloud"))
        .json(&json!({"accountId":account,"username":"alice","appPassword":"cloud-password"}))
        .send()
        .await
        .unwrap();
    assert_eq!(linked.status(), 200, "{}", linked.text().await.unwrap());
    let files = user
        .get(format!(
            "{base}/api/nextcloud/files?instanceId=cloud&path=/"
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(files.status(), 200, "{}", files.text().await.unwrap());
    let listing: Value = user
        .get(format!(
            "{base}/api/nextcloud/files?instanceId=cloud&path=/"
        ))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(listing["entries"][0]["name"], "Résumé.txt");
    let attached = user
        .post(format!("{base}/api/nextcloud/attach"))
        .json(&json!({"accountId":account,"instanceId":"cloud","paths":["/Résumé.txt"]}))
        .send()
        .await
        .unwrap();
    assert_eq!(attached.status(), 200, "{}", attached.text().await.unwrap());
    let created:Value=user.post(format!("{base}/jmap/api")).json(&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Email/set",{"accountId":account,"create":{"x":{"subject":"cloud policy","attachments":[{"blobId":"danger","name":"malware.exe","type":"application/octet-stream","size":10},{"blobId":"safe","name":"note.txt","type":"application/octet-stream","size":10}]}}},"c"]]})).send().await.unwrap().json().await.unwrap();
    let email = created["methodResponses"][0][1]["created"]["x"]["id"]
        .as_str()
        .unwrap();
    policy(&pool, "attachments.blockedExtensions", json!(["exe"])).await;
    let before = records.lock().unwrap().len();
    let denied = user
        .post(format!("{base}/api/nextcloud/save"))
        .json(&json!({"accountId":account,"instanceId":"cloud","emailId":email,"blobId":"danger","path":"/"}))
        .send()
        .await
        .unwrap();
    assert_eq!(denied.status(), 403);
    assert_eq!(records.lock().unwrap().len(), before);
    policy(&pool, "attachments.warnExtensions", json!(["txt"])).await;
    let denied = user
        .post(format!("{base}/api/nextcloud/save"))
        .json(&json!({"accountId":account,"instanceId":"cloud","emailId":email,"blobId":"safe","path":"/"}))
        .send()
        .await
        .unwrap();
    assert_eq!(denied.status(), 403);
    assert_eq!(records.lock().unwrap().len(), before);
    let accepted=user.post(format!("{base}/api/nextcloud/save")).json(&json!({"accountId":account,"instanceId":"cloud","emailId":email,"blobId":"safe","path":"/","acknowledgeWarning":true})).send().await.unwrap();
    assert_eq!(accepted.status(), 200, "{}", accepted.text().await.unwrap());
    assert_eq!(records.lock().unwrap().last().unwrap().0, "PUT");
    reject.store(true, Ordering::SeqCst);
    let before_rejection = records.lock().unwrap().len();
    assert!(
        !user
            .get(format!("{base}/api/nextcloud/list"))
            .send()
            .await
            .unwrap()
            .status()
            .is_success()
    );
    assert_eq!(records.lock().unwrap().len(), before_rejection + 1);
    assert_eq!(
        user.get(format!("{base}/api/nextcloud/list"))
            .send()
            .await
            .unwrap()
            .status(),
        409
    );
    assert_eq!(records.lock().unwrap().len(), before_rejection + 1);
    reject.store(false, Ordering::SeqCst);
    assert_eq!(
        user.put(format!("{base}/api/account/nextcloud/links/cloud"))
            .json(&json!({"accountId":account,"username":"alice","appPassword":"cloud-password"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );

    let row: Value = admin
        .get(format!("{base}/admin/nextcloud/instances"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let etag = row["instances"][0]["etag"].as_str().unwrap();
    let mut changed = body.clone();
    changed["enabled"] = json!(false);
    assert_eq!(
        admin
            .put(format!("{base}/admin/nextcloud/instances/cloud"))
            .header("if-match", etag)
            .json(&changed)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let before = records.lock().unwrap().len();
    assert_eq!(
        user.get(format!("{base}/api/nextcloud/files?instanceId=cloud"))
            .send()
            .await
            .unwrap()
            .status(),
        404
    );
    assert_eq!(records.lock().unwrap().len(), before);
    let view: Value = user
        .get(format!("{base}/api/account/nextcloud"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(view["instances"][0]["enabled"], false);
    assert_eq!(view["links"][0]["ready"], false);
    assert_eq!(
        user.delete(format!("{base}/api/account/nextcloud/links/cloud"))
            .header("if-match", view["links"][0]["etag"].as_str().unwrap())
            .json(&json!({"accountId":account}))
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    assert_eq!(records.lock().unwrap().len(), before);
}
