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
    conflict: Arc<AtomicBool>,
}
async fn cloud_proxy() -> CloudFixture {
    cloud_proxy_mode(false).await
}
async fn cloud_proxy_mode(pim: bool) -> CloudFixture {
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
    let conflict = Arc::new(AtomicBool::new(false));
    let conflict_output = conflict.clone();
    tokio::spawn(async move {
        while let Ok((socket, _)) = listener.accept().await {
            let acceptor = acceptor.clone();
            let records = records.clone();
            let reject = reject.clone();
            let conflict = conflict.clone();
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
                let path = head.split_whitespace().nth(1).unwrap_or("");
                let (status, body) = if !valid || reject.load(Ordering::SeqCst) {
                    ("401 Unauthorized", String::new())
                } else if conflict.load(Ordering::SeqCst)
                    && matches!(method.as_str(), "PUT" | "DELETE")
                {
                    ("412 Precondition Failed", String::new())
                } else if let Some(reply) = pim.then(|| pim_response(&method, path)).flatten() {
                    reply
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
                    "HTTP/1.1 {status}\r\nETag: remote-v2\r\nContent-Type: application/xml\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
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
        conflict: conflict_output,
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
        ..
    } = cloud_proxy().await;
    assert_eq!(admin.post(format!("{base}/admin/egress/proxies")).json(&json!({"id":"cloud-proxy","scheme":"http","host":"127.0.0.1","port":port,"scope":"global","trafficClasses":["nextcloud"]})).send().await.unwrap().status(),200);
    assert_eq!(
        admin
            .post(format!("{base}/admin/egress/proxies/cloud-proxy/activate"))
            .send()
            .await
            .unwrap()
            .status(),
        200
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
    let before_foreign = records.lock().unwrap().len();
    for path in [
        "/api/nextcloud/list?accountId=foreign",
        "/api/nextcloud/files?instanceId=cloud&accountId=foreign",
    ] {
        assert_eq!(
            user.get(format!("{base}{path}"))
                .send()
                .await
                .unwrap()
                .status(),
            409
        );
    }
    assert_eq!(records.lock().unwrap().len(), before_foreign);

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

fn pim_response(method: &str, path: &str) -> Option<(&'static str, String)> {
    let calendar = path.contains("/calendars/alice/");
    let contacts = path.contains("/addressbooks/users/alice/");
    if !calendar && !contacts {
        return None;
    }
    let root = if calendar {
        "/nextcloud/remote.php/dav/calendars/alice/work/"
    } else {
        "/nextcloud/remote.php/dav/addressbooks/users/alice/contacts/"
    };
    let body = if method == "PROPFIND" {
        let resource = if calendar {
            "<c:calendar/><c:supported-calendar-component-set><c:comp name=\"VEVENT\"/></c:supported-calendar-component-set>"
        } else {
            "<a:addressbook/>"
        };
        format!(
            "<d:response><d:href>{root}</d:href><d:propstat><d:prop><d:displayname>Cloud collection</d:displayname><d:resourcetype><d:collection/>{resource}</d:resourcetype></d:prop><d:status>HTTP/1.1 200 OK</d:status></d:propstat></d:response>"
        )
    } else if method == "REPORT" {
        let (name, data) = if calendar {
            (
                "event.ics",
                "<c:calendar-data><![CDATA[BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:cloud-event\r\nDTSTAMP:20260101T000000Z\r\nDTSTART:20261026T100000Z\r\nDTEND:20261026T110000Z\r\nSUMMARY:Cloud event\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n]]></c:calendar-data>",
            )
        } else {
            (
                "contact.vcf",
                "<a:address-data><![CDATA[BEGIN:VCARD\r\nVERSION:3.0\r\nUID:cloud-contact\r\nFN:Cloud Contact\r\nN:Contact;Cloud;;;\r\nEND:VCARD\r\n]]></a:address-data>",
            )
        };
        format!(
            "<d:response><d:href>{root}{name}</d:href><d:propstat><d:prop><d:getetag>remote-v1</d:getetag>{data}</d:prop><d:status>HTTP/1.1 200 OK</d:status></d:propstat></d:response><d:sync-token>sync-1</d:sync-token>"
        )
    } else {
        return None;
    };
    Some((
        "207 Multi-Status",
        format!(
            r#"<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav" xmlns:a="urn:ietf:params:xml:ns:carddav">{body}</d:multistatus>"#
        ),
    ))
}

#[tokio::test]
async fn cloud_pim_discovery_sync_writes_conflicts_and_revocation_use_guarded_https() {
    use mw_store::{
        DomainConfigRow, EgressProxyRow, ServerKey, Store, nextcloud_links::NextcloudInstance,
    };
    let mail = common::mock_mail::MockPop3::start("alice@example.org", "testpass").await;
    let path = common::test_db::unique_db_path("nextcloud-pim");
    let key = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
    let store = Store::open(path.to_str().unwrap(), ServerKey::from_hex(key).unwrap())
        .await
        .unwrap();
    store.put_domain_config(&DomainConfigRow {
        name:"example.org".into(),upstream_json:json!({"pop3":{"host":"127.0.0.1","port":mail.addr.port(),"tls":"none","allowPrivate":true}}).to_string(),allowlist_json:"[]".into(),blocklist_json:"[]".into(),enabled:true,preferred:false,settings_json:"{}".into(),updated_at:1,
    }).await.unwrap();
    let cloud = cloud_proxy_mode(true).await;
    store
        .put_egress_proxy(&EgressProxyRow {
            id: "pim-route".into(),
            kind: "http".into(),
            host: "127.0.0.1".into(),
            port: cloud.port,
            username: String::new(),
            password: None,
            allow_plaintext: false,
            active: false,
            tls_to_proxy: false,
            remote_dns: false,
            remote_dns_risk_accepted_by: None,
            remote_dns_risk_accepted_at: None,
            tunnel_id: None,
            chain_next_id: None,
            scope: "global".into(),
            traffic_classes: vec!["nextcloud".into()],
            created_at: String::new(),
            updated_at: String::new(),
        })
        .await
        .unwrap();
    assert!(
        store
            .set_active_egress_proxy(Some("pim-route"))
            .await
            .unwrap()
    );
    store
        .put_nextcloud_instance(
            &NextcloudInstance {
                id: "cloud".into(),
                name: "Cloud".into(),
                base_url: "https://8.8.8.8/nextcloud/".into(),
                ca_pem: include_str!("fixtures/tls/nextcloud-cert.pem").into(),
                enabled: true,
                revision: String::new(),
            },
            None,
        )
        .await
        .unwrap()
        .unwrap();
    let app = mw_server::build_app(AppConfig {
        db_path: path.to_string_lossy().into(),
        server_key_hex: Some(key.into()),
        web_dir: None,
        cookie_secure: false,
        mode: ServerMode::Engine,
        hardening: Default::default(),
        security: SecurityConfig {
            upstream_default: Some(mail.url()),
            ..Default::default()
        },
    })
    .await
    .unwrap();
    let base = serve(app).await;
    let user = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    let login = user
        .post(format!("{base}/api/login"))
        .json(&json!({"username":"alice@example.org","password":"testpass"}))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200, "{}", login.text().await.unwrap());
    let me: Value = user
        .get(format!("{base}/api/me"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let account = me["accountId"].as_str().unwrap();
    let linked = user
        .put(format!("{base}/api/account/nextcloud/links/cloud"))
        .json(&json!({"accountId":account,"username":"alice","appPassword":"cloud-password"}))
        .send()
        .await
        .unwrap();
    assert_eq!(linked.status(), 200, "{}", linked.text().await.unwrap());
    let before = cloud.records.lock().unwrap().len();
    let foreign = user
        .post(format!("{base}/api/account/nextcloud/pim/discover/cloud"))
        .json(&json!({"accountId":"foreign","kind":"calendar"}))
        .send()
        .await
        .unwrap();
    assert_eq!(foreign.status(), 409);
    assert_eq!(cloud.records.lock().unwrap().len(), before);
    for kind in ["calendar", "contacts"] {
        let discovery = user
            .post(format!("{base}/api/account/nextcloud/pim/discover/cloud"))
            .json(&json!({"accountId":account,"kind":kind}))
            .send()
            .await
            .unwrap();
        assert_eq!(
            discovery.status(),
            200,
            "{}",
            discovery.text().await.unwrap()
        );
        let found: Value = discovery.json().await.unwrap();
        let href = found["collections"][0]["href"]
            .as_str()
            .expect("discovered collection");
        let applied = user
            .post(format!("{base}/api/account/nextcloud/pim/apply/cloud"))
            .json(&json!({"accountId":account,"kind":kind,"href":href}))
            .send()
            .await
            .unwrap();
        assert_eq!(applied.status(), 200, "{}", applied.text().await.unwrap());
    }
    // The periodic worker must pull newly connected collections without a manual
    // refresh or another mailbox login. Its production scan cadence is 15 seconds.
    tokio::time::timeout(std::time::Duration::from_secs(35), async {
        loop {
            let calendars = store.list_calendars(account).await.unwrap();
            let mut loaded = false;
            for calendar in calendars {
                if calendar.caldav_url.is_some()
                    && !store.list_events(&calendar.id).await.unwrap().is_empty()
                {
                    loaded = true;
                }
            }
            if loaded {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("periodic PIM worker did not pull the connected calendar");
    let synced = user
        .post(format!("{base}/api/account/nextcloud/pim/sync"))
        .json(&json!({"accountId":account}))
        .send()
        .await
        .unwrap();
    assert_eq!(synced.status(), 200, "{}", synced.text().await.unwrap());
    let calendar = store
        .list_calendars(account)
        .await
        .unwrap()
        .into_iter()
        .find(|row| row.caldav_url.is_some())
        .unwrap();
    let book = store
        .list_address_books(account)
        .await
        .unwrap()
        .into_iter()
        .find(|row| row.carddav_url.is_some())
        .unwrap();
    let event = store.list_events(&calendar.id).await.unwrap().remove(0);
    let contact = store.list_contacts(&book.id).await.unwrap().remove(0);
    async fn update(
        user: &reqwest::Client,
        base: &str,
        account: &str,
        method: &str,
        id: &str,
        patch: Value,
    ) -> Value {
        user.post(format!("{base}/jmap/api")).json(&json!({"using":["urn:ietf:params:jmap:core","urn:mailwoman:calendars","urn:mailwoman:contacts"],"methodCalls":[[method,{"accountId":account,"update":{id:patch}},"c"]]})).send().await.unwrap().json::<Value>().await.unwrap()["methodResponses"][0][1].clone()
    }
    let changed = update(
        &user,
        &base,
        account,
        "CalendarEvent/set",
        &event.id,
        json!({"title":"Edited cloud event"}),
    )
    .await;
    assert!(changed["updated"].get(&event.id).is_some(), "{changed}");
    assert!(
        cloud
            .records
            .lock()
            .unwrap()
            .iter()
            .any(|(method, body)| method == "PUT"
                && String::from_utf8_lossy(body).contains("Edited cloud event"))
    );
    let saved = store.get_event(&event.id).await.unwrap().unwrap();
    assert_eq!(saved.etag.as_deref(), Some("remote-v2"));
    cloud.conflict.store(true, Ordering::SeqCst);
    let failed = update(
        &user,
        &base,
        account,
        "CalendarEvent/set",
        &event.id,
        json!({"title":"Conflict must not persist"}),
    )
    .await;
    assert!(failed["notUpdated"].get(&event.id).is_some(), "{failed}");
    assert_eq!(store.get_event(&event.id).await.unwrap().unwrap(), saved);
    cloud.conflict.store(false, Ordering::SeqCst);
    cloud.reject.store(true, Ordering::SeqCst);
    let before = cloud.records.lock().unwrap().len();
    let failed = update(
        &user,
        &base,
        account,
        "ContactCard/set",
        &contact.id,
        json!({"isFavorite":true}),
    )
    .await;
    assert!(failed["notUpdated"].get(&contact.id).is_some(), "{failed}");
    assert_eq!(cloud.records.lock().unwrap().len(), before + 1);
    assert!(store.nextcloud_links(account).await.unwrap().is_empty());
    let _ = update(
        &user,
        &base,
        account,
        "ContactCard/set",
        &contact.id,
        json!({"isFavorite":true}),
    )
    .await;
    assert_eq!(
        cloud.records.lock().unwrap().len(),
        before + 1,
        "revoked credentials never replayed"
    );
    assert_eq!(
        store.get_contact(&contact.id).await.unwrap().unwrap(),
        contact
    );
}
