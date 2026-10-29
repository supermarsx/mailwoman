//! Large-attachment share links and calendar creation against a REAL Nextcloud
//! (t39 L7). Skipped unless these are set:
//!   MW_E2E_NEXTCLOUD_ORIGIN   host:port of its HTTPS listener; the certificate
//!                             must be valid for IP 8.8.8.8 (tests/fixtures/tls/nextcloud-cert.pem)
//!   MW_E2E_NEXTCLOUD_USER     an account on it
//!   MW_E2E_NEXTCLOUD_APP_PASSWORD_FILE   a file holding that account's app password
//! The server reaches it as https://8.8.8.8/ through an operator egress route (a
//! CONNECT proxy in this test), so the address policy and TLS trust are exercised.
//!
//!   docker run nextcloud:stable behind Caddy with the fixture certificate, then
//!   cargo test -p mw-server --test t39_nextcloud_live -- --test-threads=1
use serde_json::{Value, json};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
mod common;

struct Live {
    origin: String,
    user: String,
    password: String,
}

fn live() -> Option<Live> {
    let origin = std::env::var("MW_E2E_NEXTCLOUD_ORIGIN").ok()?;
    let user = std::env::var("MW_E2E_NEXTCLOUD_USER").ok()?;
    let file = std::env::var("MW_E2E_NEXTCLOUD_APP_PASSWORD_FILE").ok()?;
    let password = std::fs::read_to_string(file).ok()?.trim().to_owned();
    Some(Live {
        origin,
        user,
        password,
    })
}

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

/// An HTTP CONNECT proxy that only tunnels `8.8.8.8:443` to the live origin, and
/// counts the tunnels it opened.
async fn connect_proxy(origin: String) -> (u16, std::sync::Arc<std::sync::atomic::AtomicUsize>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let tunnels = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter = tunnels.clone();
    tokio::spawn(async move {
        while let Ok((socket, _)) = listener.accept().await {
            let origin = origin.clone();
            let counter = counter.clone();
            tokio::spawn(async move {
                let mut socket = BufReader::new(socket);
                let mut first = String::new();
                if socket.read_line(&mut first).await.is_err() {
                    return;
                }
                if first != "CONNECT 8.8.8.8:443 HTTP/1.1\r\n" {
                    return;
                }
                loop {
                    let mut line = String::new();
                    if socket.read_line(&mut line).await.unwrap_or(0) == 0 {
                        return;
                    }
                    if line == "\r\n" {
                        break;
                    }
                }
                let Ok(mut upstream) = tokio::net::TcpStream::connect(&origin).await else {
                    return;
                };
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let mut socket = socket.into_inner();
                if socket
                    .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                    .await
                    .is_err()
                {
                    return;
                }
                let _ = tokio::io::copy_bidirectional(&mut socket, &mut upstream).await;
            });
        }
    });
    (port, tunnels)
}

/// Talk to Nextcloud directly (not through the server under test) to check results.
fn direct() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
}

#[tokio::test]
async fn large_file_share_link_and_new_calendar_reach_a_real_nextcloud() {
    use mw_store::{
        DomainConfigRow, EgressProxyRow, ServerKey, Store, nextcloud_links::NextcloudInstance,
    };
    let Some(live) = live() else {
        eprintln!("skipped: set MW_E2E_NEXTCLOUD_ORIGIN/USER/APP_PASSWORD_FILE");
        return;
    };
    let mail = common::mock_mail::MockPop3::start("alice@example.org", "testpass").await;
    let path = common::test_db::unique_db_path("t39-nextcloud-live");
    let key = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
    let store = Store::open(path.to_str().unwrap(), ServerKey::from_hex(key).unwrap())
        .await
        .unwrap();
    store.put_domain_config(&DomainConfigRow {
        name:"example.org".into(),upstream_json:json!({"pop3":{"host":"127.0.0.1","port":mail.addr.port(),"tls":"none","allowPrivate":true}}).to_string(),allowlist_json:"[]".into(),blocklist_json:"[]".into(),enabled:true,preferred:false,settings_json:"{}".into(),updated_at:1,
    }).await.unwrap();
    let (proxy_port, tunnels) = connect_proxy(live.origin.clone()).await;
    store
        .put_egress_proxy(&EgressProxyRow {
            id: "live-cloud-route".into(),
            kind: "http".into(),
            host: "127.0.0.1".into(),
            port: proxy_port,
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
            .set_active_egress_proxy(Some("live-cloud-route"))
            .await
            .unwrap()
    );
    store
        .put_nextcloud_instance(
            &NextcloudInstance {
                id: "cloud".into(),
                name: "Live cloud".into(),
                base_url: "https://8.8.8.8/".into(),
                ca_pem: include_str!("fixtures/tls/nextcloud-cert.pem").into(),
                enabled: true,
                revision: String::new(),
            },
            None,
        )
        .await
        .unwrap()
        .unwrap();
    let app = mw_server::build_app(mw_server::AppConfig {
        db_path: path.to_string_lossy().into(),
        server_key_hex: Some(key.into()),
        web_dir: None,
        cookie_secure: false,
        mode: mw_server::ServerMode::Engine,
        hardening: Default::default(),
        security: mw_server::SecurityConfig {
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
    let account = me["accountId"].as_str().unwrap().to_owned();
    let linked = user
        .put(format!("{base}/api/account/nextcloud/links/cloud"))
        .json(&json!({"accountId":account,"username":live.user,"appPassword":live.password}))
        .send()
        .await
        .unwrap();
    assert_eq!(linked.status(), 200, "{}", linked.text().await.unwrap());

    // ── large file → chunked upload → share link with password and expiry ──
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let name = format!("t39 big {stamp}.bin");
    let chunk = 5 * 1024 * 1024;
    let size = chunk + 1024 * 1024 + 17;
    let file: Vec<u8> = (0..size).map(|i| (i * 31 % 251) as u8).collect();

    // Another account cannot open a transfer in this link; nothing reaches the cloud.
    let before = tunnels.load(std::sync::atomic::Ordering::SeqCst);
    let foreign = user
        .post(format!("{base}/api/nextcloud/upload"))
        .json(&json!({"accountId":"foreign","instanceId":"cloud","name":name,"size":size}))
        .send()
        .await
        .unwrap();
    assert_eq!(foreign.status(), 409);
    assert_eq!(tunnels.load(std::sync::atomic::Ordering::SeqCst), before);

    let mut stored = Vec::new();
    for round in 0..2 {
        let started = user
            .post(format!("{base}/api/nextcloud/upload"))
            .json(&json!({"accountId":account,"instanceId":"cloud","name":name,"size":size}))
            .send()
            .await
            .unwrap();
        assert_eq!(started.status(), 200, "{}", started.text().await.unwrap());
        let started: Value = started.json().await.unwrap();
        let transfer = started["transfer"].as_str().unwrap().to_owned();
        let target = started["path"].as_str().unwrap().to_owned();
        assert_eq!(started["chunkBytes"], chunk);
        if round == 1 {
            // The first copy is not overwritten: the second gets a free name.
            assert_eq!(target, format!("/Mail attachments/t39 big {stamp} (2).bin"));
        } else {
            assert_eq!(target, format!("/Mail attachments/{name}"));
        }
        for (index, part) in file.chunks(chunk).enumerate() {
            let sent = user
                .put(format!(
                    "{base}/api/nextcloud/upload/{transfer}/{}",
                    index + 1
                ))
                .query(&[
                    ("accountId", account.as_str()),
                    ("instanceId", "cloud"),
                    ("path", target.as_str()),
                ])
                .body(part.to_vec())
                .send()
                .await
                .unwrap();
            assert_eq!(sent.status(), 204, "{}", sent.text().await.unwrap());
        }
        // Assembly checks the stored size against the declared one.
        let finished = user
            .post(format!("{base}/api/nextcloud/upload/{transfer}/finish"))
            .json(&json!({"accountId":account,"instanceId":"cloud","path":target,"size":size}))
            .send()
            .await
            .unwrap();
        assert_eq!(finished.status(), 200, "{}", finished.text().await.unwrap());
        stored.push(target);
    }
    // A path outside the attachment folder is refused before any cloud request.
    let escaped = user
        .put(format!("{base}/api/nextcloud/upload/{}/1", "a".repeat(32)))
        .query(&[
            ("accountId", account.as_str()),
            ("instanceId", "cloud"),
            ("path", "/Documents/x.bin"),
        ])
        .body(vec![1u8; 10])
        .send()
        .await
        .unwrap();
    assert_eq!(escaped.status(), 400);

    let cloud = direct();
    let dav = |path: &str| {
        format!(
            "https://{}/remote.php/dav/files/{}/{}",
            live.origin,
            live.user,
            path.trim_start_matches('/')
                .split('/')
                .map(|part| part.replace(' ', "%20"))
                .collect::<Vec<_>>()
                .join("/")
        )
    };
    for target in &stored {
        let got = cloud
            .get(dav(target))
            .header("host", "8.8.8.8")
            .basic_auth(&live.user, Some(&live.password))
            .send()
            .await
            .unwrap();
        assert_eq!(got.status(), 200, "{target}");
        assert!(
            got.bytes().await.unwrap().as_ref() == file.as_slice(),
            "{target}: content differs"
        );
    }

    let expiry = (chrono::Utc::now() + chrono::Duration::days(7))
        .format("%Y-%m-%d")
        .to_string();
    let shared = user
        .post(format!("{base}/api/nextcloud/share-link"))
        .json(&json!({"accountId":account,"instanceId":"cloud","path":stored[0],"password":"Share-Pass-39-x","expiresAt":expiry}))
        .send()
        .await
        .unwrap();
    assert_eq!(shared.status(), 200, "{}", shared.text().await.unwrap());
    let shared: Value = shared.json().await.unwrap();
    let url = shared["url"].as_str().unwrap();
    assert!(url.starts_with("https://8.8.8.8/s/"), "{url}");
    assert_eq!(shared["passwordProtected"], true);
    assert_eq!(shared["expiresAt"], expiry.as_str());
    // Nextcloud itself holds the share with that expiry and a password.
    let shares: Value = cloud
        .get(format!(
            "https://{}/ocs/v2.php/apps/files_sharing/api/v1/shares?format=json",
            live.origin
        ))
        .query(&[("path", stored[0].as_str())])
        .header("host", "8.8.8.8")
        .header("OCS-APIRequest", "true")
        .basic_auth(&live.user, Some(&live.password))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let share = shares["ocs"]["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["url"].as_str() == Some(url))
        .unwrap_or_else(|| panic!("share not listed: {shares}"));
    assert!(
        share["expiration"]
            .as_str()
            .is_some_and(|value| value.starts_with(&expiry)),
        "{share}"
    );
    assert!(
        share["password"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
            || share["share_with"]
                .as_str()
                .is_some_and(|value| !value.is_empty()),
        "{share}"
    );
    // The public link asks for the password: the file is not served without it.
    let token = url.rsplit('/').next().unwrap();
    let anonymous = cloud
        .get(format!("https://{}/s/{token}/download", live.origin))
        .header("host", "8.8.8.8")
        .send()
        .await
        .unwrap();
    let body = anonymous.bytes().await.unwrap();
    assert!(
        body.as_ref() != file.as_slice(),
        "a password-protected link served the file"
    );

    // ── a new calendar and a new task list, created on the remote and connected ──
    for (kind, display) in [
        ("calendar", format!("Team plans {stamp}")),
        ("tasks", format!("Errands {stamp}")),
    ] {
        let created = user
            .post(format!("{base}/api/account/nextcloud/pim/create/cloud"))
            .json(&json!({"accountId":account,"kind":kind,"name":display}))
            .send()
            .await
            .unwrap();
        assert_eq!(created.status(), 200, "{}", created.text().await.unwrap());
        let created: Value = created.json().await.unwrap();
        let href = created["href"].as_str().unwrap();
        assert_eq!(created["name"], display.as_str());
        assert!(created["collectionId"].as_str().is_some());
        let path = reqwest::Url::parse(href).unwrap().path().to_owned();
        let found = cloud
            .request(
                reqwest::Method::from_bytes(b"PROPFIND").unwrap(),
                format!("https://{}{path}", live.origin),
            )
            .header("host", "8.8.8.8")
            .header("Depth", "0")
            .basic_auth(&live.user, Some(&live.password))
            .send()
            .await
            .unwrap();
        assert_eq!(found.status(), 207, "{path}");
        let text = found.text().await.unwrap();
        assert!(text.contains(&display), "{text}");
        // Connected: the list shows it as a ready collection of this account.
        let listed: Value = user
            .get(format!(
                "{base}/api/account/nextcloud/pim?accountId={account}"
            ))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert!(
            listed["collections"]
                .as_array()
                .unwrap()
                .iter()
                .any(|row| row["kind"] == kind && row["ready"] == true),
            "{listed}"
        );
    }
    // Contacts cannot be created this way, and another account is refused.
    let refused = user
        .post(format!("{base}/api/account/nextcloud/pim/create/cloud"))
        .json(&json!({"accountId":account,"kind":"contacts","name":"x"}))
        .send()
        .await
        .unwrap();
    assert_eq!(refused.status(), 400);
    let foreign = user
        .post(format!("{base}/api/account/nextcloud/pim/create/cloud"))
        .json(&json!({"accountId":"foreign","kind":"calendar","name":"x"}))
        .send()
        .await
        .unwrap();
    assert_eq!(foreign.status(), 409);
    tokio::time::sleep(Duration::from_millis(10)).await;
}
