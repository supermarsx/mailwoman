use hickory_resolver::proto::{
    op::Message,
    rr::{RData, Record, RecordType, rdata::A},
};
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
mod common;

#[tokio::test]
async fn dns_admission_is_bounded_expires_revokes_and_ignores_spoofing() {
    let dns = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let resolver = dns.local_addr().unwrap();
    let mode = Arc::new(AtomicUsize::new(0));
    let queries = Arc::new(AtomicUsize::new(0));
    let (mode_task, query_task) = (mode.clone(), queries.clone());
    let dns_task = tokio::spawn(async move {
        let mut bytes = [0u8; 4096];
        loop {
            let (length, peer) = dns.recv_from(&mut bytes).await.unwrap();
            query_task.fetch_add(1, Ordering::SeqCst);
            let request = Message::from_vec(&bytes[..length]).unwrap();
            let mode = mode_task.load(Ordering::SeqCst);
            if mode == 2
                || request
                    .queries
                    .iter()
                    .any(|q| q.name().to_string() == "stalled.example.")
            {
                continue;
            } // A resolver that receives but never responds.
            let mut response = Message::response(request.metadata.id, request.metadata.op_code);
            response.metadata.recursion_available = true;
            response.metadata.recursion_desired = request.metadata.recursion_desired;
            response.queries = request.queries.clone();
            response.edns = request.edns.clone();
            for query in request.queries {
                if query.query_type() == RecordType::A {
                    let count = if mode == 3 { 3 } else { 1 };
                    for i in 0..count {
                        let ip = if mode == 1 {
                            Ipv4Addr::new(192, 0, 2, 20)
                        } else if i == 0 {
                            Ipv4Addr::LOCALHOST
                        } else {
                            Ipv4Addr::new(192, 0, 2, i)
                        };
                        response.add_answer(Record::from_rdata(
                            query.name().clone(),
                            1,
                            RData::A(A(ip)),
                        ));
                    }
                }
            }
            dns.send_to(&response.to_vec().unwrap(), peer)
                .await
                .unwrap();
        }
    });
    let db = common::test_db::unique_db_path("inbound-allowlist");
    let (app, _) = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into(),
            server_key_hex: None,
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
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
    let task = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    let client = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    let endpoint = format!("{base}/admin/security/inbound-allowlist");
    assert_eq!(client.get(&endpoint).send().await.unwrap().status(), 401);
    assert_eq!(
        client
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"hunter2"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let read = client.get(&endpoint).send().await.unwrap();
    assert_eq!(read.status(), 200);
    assert_eq!(read.headers()["cache-control"], "private, no-store");
    let tag = read.headers()["etag"].to_str().unwrap().to_owned();
    let mut config = read.json::<Value>().await.unwrap()["config"].clone();
    assert_eq!(config["admin"]["enabled"], false);
    assert_eq!(config["webmail"]["enabled"], false);
    config["dns"]["servers"] = json!([resolver.to_string()]);
    config["dns"]["timeoutMs"] = json!(2000);
    config["dns"]["maxAddresses"] = json!(2);
    config["webmail"] = json!({"enabled":true,"cidrs":["192.0.2.0/24"],"hostnames":["office.example","stalled.example"]});
    assert_eq!(
        client
            .put(&endpoint)
            .header("if-match", &tag)
            .json(&config)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        client
            .post(format!("{base}/admin/step-up"))
            .json(&json!({"password":"hunter2"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        client
            .put(&endpoint)
            .json(&config)
            .send()
            .await
            .unwrap()
            .status(),
        428
    );
    let saved = client
        .put(&endpoint)
        .header("if-match", &tag)
        .json(&config)
        .send()
        .await
        .unwrap();
    assert_eq!(saved.status(), 200);
    let current_tag = saved.headers()["etag"].to_str().unwrap().to_owned();
    assert_eq!(
        client
            .put(&endpoint)
            .header("if-match", &tag)
            .json(&config)
            .send()
            .await
            .unwrap()
            .status(),
        412
    );
    // This public API needs no account; 200 proves actual admission, rather than an auth refusal.
    let public = format!("{base}/api/public-config");
    for _ in 0..40 {
        if client.get(&public).send().await.unwrap().status() == 200 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(client.get(&public).send().await.unwrap().status(), 200);
    assert!(queries.load(Ordering::SeqCst) >= 2); // Both A and AAAA questions.
    mode.store(2, Ordering::SeqCst);
    tokio::time::sleep(Duration::from_millis(1200)).await;
    let before = std::time::Instant::now();
    let denied = client
        .get(&public)
        .header("x-forwarded-for", "192.0.2.30")
        .send()
        .await
        .unwrap();
    assert!(
        before.elapsed() < Duration::from_millis(1500),
        "request waited for DNS"
    );
    assert_eq!(denied.status(), 403);
    assert_eq!(denied.headers()["cache-control"], "no-store");
    assert_eq!(
        client
            .get(format!("{base}/healthz"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(client.get(&endpoint).send().await.unwrap().status(), 200); // Separate surface.
    // An overflowing fresh DNS response must not grant even the first address.
    mode.store(3, Ordering::SeqCst);
    config["webmail"]["hostnames"] = json!(["overflow.example"]);
    let saved = client
        .put(&endpoint)
        .header("if-match", &current_tag)
        .json(&config)
        .send()
        .await
        .unwrap();
    assert_eq!(saved.status(), 200);
    let current_tag = saved.headers()["etag"].to_str().unwrap().to_owned();
    tokio::time::sleep(Duration::from_millis(250)).await;
    assert_eq!(client.get(&public).send().await.unwrap().status(), 403);
    let status: Value = client
        .get(&endpoint)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(status["status"]["hosts"][0]["error"], "tooManyAddresses");
    // Literal policy edit takes effect on the very next request, without DNS or restart.
    config["webmail"]["cidrs"] = json!(["127.0.0.1"]);
    config["webmail"]["hostnames"] = json!([]);
    assert_eq!(
        client
            .put(&endpoint)
            .header("if-match", &current_tag)
            .json(&config)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(client.get(&public).send().await.unwrap().status(), 200);
    let tag = client.get(&endpoint).send().await.unwrap().headers()["etag"]
        .to_str()
        .unwrap()
        .to_owned();
    config["admin"] = json!({"enabled":true,"cidrs":["192.0.2.0/24"],"hostnames":[]});
    assert_eq!(
        client
            .put(&endpoint)
            .header("if-match", tag)
            .json(&config)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        client
            .get(&endpoint)
            .header("x-forwarded-for", "192.0.2.30")
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(client.get(&public).send().await.unwrap().status(), 200);
    // Simulate an authorized deployment-side recovery in this isolated test DB.
    let pool = sqlx::SqlitePool::connect(&format!("sqlite:{}", db.display()))
        .await
        .unwrap();
    config["admin"]["enabled"] = json!(false);
    sqlx::query("UPDATE settings SET value=? WHERE key='security.inbound-allowlist.v1'")
        .bind(config.to_string())
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(client.get(&endpoint).send().await.unwrap().status(), 200);
    // The storage read cap must accommodate every otherwise valid bounded policy.
    let tag = client.get(&endpoint).send().await.unwrap().headers()["etag"]
        .to_str()
        .unwrap()
        .to_owned();
    let long_host = [
        "a".repeat(63),
        "b".repeat(63),
        "c".repeat(63),
        "d".repeat(61),
    ]
    .join(".");
    config["admin"]["hostnames"] = json!(vec![long_host.clone(); 64]);
    config["webmail"]["hostnames"] = json!(vec![long_host; 64]);
    config["webmail"]["enabled"] = json!(false);
    assert!(config.to_string().len() > 32768);
    assert_eq!(
        client
            .put(&endpoint)
            .header("if-match", tag)
            .json(&config)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(client.get(&endpoint).send().await.unwrap().status(), 200);
    task.abort();
    dns_task.abort();
}
