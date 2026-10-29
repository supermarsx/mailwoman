//! Diagnostics execute real bounded probes, persist history and require admin grants.
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};

#[tokio::test]
async fn server_suite_is_authenticated_redacted_persisted_and_rate_limited() {
    let dir = std::env::temp_dir().join(format!("mw-diagnostics-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let secret = "diagnostic-admin-password";
    let app = build_app_full(
        AppConfig {
            db_path: dir.join("diag.db").to_string_lossy().into_owned(),
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
            admin_password: Some(secret.into()),
            redis_url: None,
        },
    )
    .await
    .unwrap()
    .0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    // As main.rs serves it: with the peer address, which the fail2ban check reads.
    let task = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap()
    });
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .no_proxy()
        .build()
        .unwrap();
    assert_eq!(
        client
            .get(format!("{base}/admin/diag/runs"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(
        client
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":secret}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        client
            .post(format!("{base}/admin/diag/run"))
            .json(&json!({"suite":"unimplemented"}))
            .send()
            .await
            .unwrap()
            .status(),
        422
    );
    let response = client
        .post(format!("{base}/admin/diag/run"))
        .json(&json!({"suite":"server"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["cache-control"], "private, no-store");
    let text = response.text().await.unwrap();
    assert!(!text.contains(secret));
    assert!(!text.contains(&"12".repeat(32)));
    let report: Value = serde_json::from_str(&text).unwrap();
    // t39 L3/L4 added network, posture and fail2ban checks. Over plain HTTP on
    // loopback the posture checks warn (no HTTPS, no Secure cookies...), so the run
    // is `warn`; nothing may fail, and the original five store/key/egress checks pass.
    assert_eq!(report["run"]["status"], "warn", "{report}");
    let results = report["run"]["results"].as_array().unwrap();
    assert!(results.iter().all(|r| r["status"] != "fail"), "{report}");
    for id in [
        "database-connectivity",
        "database-write",
        "database-migrations",
        "server-key-roundtrip",
        "egress-private-policy",
    ] {
        let row = results
            .iter()
            .find(|r| r["id"] == id)
            .unwrap_or_else(|| panic!("{id}: {report}"));
        assert_eq!(row["status"], "pass", "{id}: {report}");
    }
    let fail2ban = results
        .iter()
        .find(|r| r["id"] == "fail2ban-client-address")
        .unwrap();
    assert_eq!(
        fail2ban["status"], "warn",
        "a loopback peer is named, but warned about"
    );
    assert_eq!(report["historySaved"], true);
    assert_eq!(report["auditSaved"], true);
    let history: Value = client
        .get(format!("{base}/admin/diag/runs"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(history["runs"][0]["id"], report["run"]["id"]);
    assert_eq!(
        client
            .post(format!("{base}/admin/diag/run"))
            .json(&json!({"suite":"server"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        client
            .post(format!("{base}/admin/diag/run"))
            .json(&json!({"suite":"server"}))
            .send()
            .await
            .unwrap()
            .status(),
        429
    );
    task.abort();
}

#[tokio::test]
async fn saved_backend_jobs_cancel_and_keep_other_sessions_out() {
    let dir = std::env::temp_dir().join(format!("mw-diagnostic-jobs-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("jobs.db");
    let key = "21".repeat(32);
    let store = mw_store::Store::open(
        &path.to_string_lossy(),
        mw_store::ServerKey::from_hex(&key).unwrap(),
    )
    .await
    .unwrap();
    let hanging = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = hanging.local_addr().unwrap().port();
    let endpoint = json!({"host":"127.0.0.1","port":port,"tls":"none","allowPrivate":true});
    store.put_domain_config(&mw_store::DomainConfigRow{name:"checks.example".into(),upstream_json:json!({"version":2,"backends":(0..8).map(|index| json!({"id":format!("backend-{index}"),"role":"mail","protocol":"imap","preferred":index==0,"priority":100+index,"endpoint":endpoint})).collect::<Vec<_>>()}).to_string(),allowlist_json:"[]".into(),blocklist_json:"[]".into(),settings_json:"{}".into(),enabled:true,preferred:false,updated_at:1}).await.unwrap();
    let app = build_app_full(
        AppConfig {
            db_path: path.to_string_lossy().into_owned(),
            server_key_hex: Some(key.clone()),
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig::default(),
        },
        V6Config {
            admin_enabled: true,
            admin_username: Some("root".into()),
            admin_password: Some("diagnostic-secret".into()),
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
    let other = reqwest::Client::builder()
        .cookie_store(true)
        .no_proxy()
        .build()
        .unwrap();
    for c in [&client, &other] {
        assert_eq!(
            c.post(format!("{base}/admin/login"))
                .json(&json!({"username":"root","password":"diagnostic-secret"}))
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
    }
    let response = client
        .post(format!("{base}/admin/diag/jobs"))
        .json(&json!({"suite":"backends"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 202);
    let job: Value = response.json().await.unwrap();
    assert_eq!(job["total"], 8, "nonpreferred backend must be included");
    let id = job["run"]["id"].as_str().unwrap();
    assert_eq!(
        other
            .get(format!("{base}/admin/diag/jobs/{id}"))
            .send()
            .await
            .unwrap()
            .status(),
        404
    );
    let (mut socket, _) = tokio::time::timeout(std::time::Duration::from_secs(3), hanging.accept())
        .await
        .unwrap()
        .unwrap();
    let mut held = Vec::new();
    for _ in 0..3 {
        held.push(
            tokio::time::timeout(std::time::Duration::from_secs(3), hanging.accept())
                .await
                .unwrap()
                .unwrap()
                .0,
        );
    }
    assert!(
        tokio::time::timeout(std::time::Duration::from_millis(100), hanging.accept())
            .await
            .is_err(),
        "at most four probes may connect concurrently"
    );
    assert_eq!(
        client
            .post(format!("{base}/admin/diag/jobs/{id}/cancel"))
            .json(&json!({}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let final_job = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            let value: Value = client
                .get(format!("{base}/admin/diag/jobs/{id}"))
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            if value["run"]["status"] != "running" {
                break value;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(final_job["run"]["status"], "warn");
    assert_eq!(final_job["termination"], "cancelled");
    assert_eq!(final_job["historySaved"], true);
    use tokio::io::AsyncReadExt;
    let mut byte = [0];
    assert_eq!(
        tokio::time::timeout(std::time::Duration::from_secs(2), socket.read(&mut byte))
            .await
            .unwrap()
            .unwrap(),
        0,
        "cancellation drops in-flight network stream"
    );
    let report = client
        .get(format!("{base}/admin/diag/report"))
        .send()
        .await
        .unwrap();
    assert_eq!(report.headers()["cache-control"], "private, no-store");
    let text = report.text().await.unwrap();
    assert!(!text.contains("diagnostic-secret"));
    assert!(!text.contains(&key));
    assert!(!text.contains("upstream_json"));
    task.abort();
}
