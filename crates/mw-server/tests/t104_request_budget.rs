//! t104: a signed-in admin gets a real diagnostics burst, an identical running
//! job is answered instead of refused, every 429 carries Retry-After, and the
//! `rateLimit.*` keys are written only through the step-up route with
//! validation.
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};

const SECRET: &str = "t104-admin-password";

async fn server(
    store_setup: impl AsyncFnOnce(&mw_store::Store),
) -> (String, tokio::task::JoinHandle<()>) {
    let dir = std::env::temp_dir().join(format!("mw-t104-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("t104.db");
    let key = "34".repeat(32);
    let store = mw_store::Store::open(
        &path.to_string_lossy(),
        mw_store::ServerKey::from_hex(&key).unwrap(),
    )
    .await
    .unwrap();
    store_setup(&store).await;
    drop(store);
    let app = build_app_full(
        AppConfig {
            db_path: path.to_string_lossy().into_owned(),
            server_key_hex: Some(key),
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig::default(),
        },
        V6Config {
            admin_enabled: true,
            admin_username: Some("root".into()),
            admin_password: Some(SECRET.into()),
            redis_url: None,
        },
    )
    .await
    .unwrap()
    .0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap()
    });
    (base, task)
}

async fn admin(base: &str) -> reqwest::Client {
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .no_proxy()
        .build()
        .unwrap();
    let status = client
        .post(format!("{base}/admin/login"))
        .json(&json!({"username":"root","password":SECRET}))
        .send()
        .await
        .unwrap()
        .status();
    assert_eq!(status, 200);
    client
}

#[tokio::test]
async fn an_admin_session_runs_several_checks_in_a_row_and_the_limit_answers_retry_after() {
    let (base, task) = server(async |_| {}).await;
    let client = admin(&base).await;
    // The old limit refused the third run inside a minute.
    for n in 0..5 {
        let response = client
            .post(format!("{base}/admin/diag/run"))
            .json(&json!({"suite":"server"}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200, "run {n}");
    }

    // The keys go through the dedicated step-up route only.
    let settings = client
        .put(format!("{base}/admin/settings/rateLimit.diagnosticsBurst"))
        .json(&json!({"globalValue": 1}))
        .send()
        .await
        .unwrap();
    assert!(
        settings.status().is_client_error(),
        "the generic settings route refuses a dedicated key: {}",
        settings.status()
    );
    let read: Value = client
        .get(format!("{base}/admin/rate-limits"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(read["limits"]["diagnosticsBurst"], 10, "{read}");
    let refused = client
        .put(format!("{base}/admin/rate-limits"))
        .json(&json!({"diagnosticsBurst": 0}))
        .send()
        .await
        .unwrap();
    assert_eq!(refused.status(), 422, "a zero burst is refused");
    let body: Value = refused.json().await.unwrap();
    assert_eq!(body["field"], "/diagnosticsBurst", "{body}");
    let out_of_range = client
        .put(format!("{base}/admin/rate-limits"))
        .json(&json!({"diagnosticsPerMinute": 0}))
        .send()
        .await
        .unwrap();
    assert_eq!(out_of_range.status(), 422);
    let unknown = client
        .put(format!("{base}/admin/rate-limits"))
        .json(&json!({"everything": 1}))
        .send()
        .await
        .unwrap();
    assert!(unknown.status().is_client_error());

    // A valid change needs a fresh confirmation.
    let without = client
        .put(format!("{base}/admin/rate-limits"))
        .json(&json!({"diagnosticsBurst": 1, "diagnosticsPerMinute": 1}))
        .send()
        .await
        .unwrap();
    assert_eq!(without.status(), 403);
    let body: Value = without.json().await.unwrap();
    assert_eq!(body["code"], "step_up_required", "{body}");
    assert_eq!(
        client
            .post(format!("{base}/admin/step-up"))
            .json(&json!({"password":SECRET}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let saved = client
        .put(format!("{base}/admin/rate-limits"))
        .json(&json!({"diagnosticsBurst": 1, "diagnosticsPerMinute": 1}))
        .send()
        .await
        .unwrap();
    assert_eq!(saved.status(), 200);
    let saved: Value = saved.json().await.unwrap();
    assert_eq!(saved["limits"]["diagnosticsBurst"], 1, "{saved}");

    // A fresh session at the low limit: one run, then 429 with Retry-After.
    let fresh = admin(&base).await;
    assert_eq!(
        fresh
            .post(format!("{base}/admin/diag/run"))
            .json(&json!({"suite":"server"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let limited = fresh
        .post(format!("{base}/admin/diag/run"))
        .json(&json!({"suite":"server"}))
        .send()
        .await
        .unwrap();
    assert_eq!(limited.status(), 429);
    let after: u64 = limited.headers()["retry-after"]
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!((1..=60).contains(&after), "Retry-After {after}");
    let body: Value = limited.json().await.unwrap();
    assert_eq!(body["code"], "rate_limited");
    assert_eq!(body["retryAfterSecs"], after);
    // The budget is per session: another admin session is not spent by this one.
    let other = admin(&base).await;
    assert_eq!(
        other
            .post(format!("{base}/admin/diag/run"))
            .json(&json!({"suite":"server"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    // Back to defaults (null = default).
    let reset = fresh
        .put(format!("{base}/admin/rate-limits"))
        .json(&json!({"diagnosticsBurst": null, "diagnosticsPerMinute": null}))
        .send()
        .await
        .unwrap();
    assert!(
        reset.status() == 200 || reset.status() == 403,
        "{}",
        reset.status()
    );
    task.abort();
}

#[tokio::test]
async fn an_identical_running_job_is_answered_instead_of_refused() {
    let hanging = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = hanging.local_addr().unwrap().port();
    let (base, task) = server(async move |store| {
        let endpoint = json!({"host":"127.0.0.1","port":port,"tls":"none","allowPrivate":true});
        store
            .put_domain_config(&mw_store::DomainConfigRow {
                name: "dedupe.example".into(),
                upstream_json: json!({"version":2,"backends":[{"id":"primary","role":"mail","protocol":"imap","preferred":true,"priority":100,"endpoint":endpoint}]}).to_string(),
                allowlist_json: "[]".into(),
                blocklist_json: "[]".into(),
                settings_json: "{}".into(),
                enabled: true,
                preferred: false,
                updated_at: 1,
            })
            .await
            .unwrap();
    })
    .await;
    let client = admin(&base).await;
    let first = client
        .post(format!("{base}/admin/diag/jobs"))
        .json(&json!({"suite":"backends"}))
        .send()
        .await
        .unwrap();
    assert_eq!(first.status(), 202);
    let first: Value = first.json().await.unwrap();
    let id = first["run"]["id"].as_str().unwrap().to_owned();
    // The probe hangs on the accepted socket, so the job stays running.
    let _held = tokio::time::timeout(std::time::Duration::from_secs(3), hanging.accept())
        .await
        .unwrap()
        .unwrap();
    for _ in 0..12 {
        let again = client
            .post(format!("{base}/admin/diag/jobs"))
            .json(&json!({"suite":"backends"}))
            .send()
            .await
            .unwrap();
        assert_eq!(again.status(), 200, "a repeat is answered, never 429");
        let again: Value = again.json().await.unwrap();
        assert_eq!(again["run"]["id"], id.as_str(), "{again}");
        assert_eq!(again["deduplicated"], true);
    }
    // Another session does not see this session's job.
    let other = admin(&base).await;
    let theirs = other
        .post(format!("{base}/admin/diag/jobs"))
        .json(&json!({"suite":"backends"}))
        .send()
        .await
        .unwrap();
    assert_eq!(theirs.status(), 202);
    let theirs: Value = theirs.json().await.unwrap();
    assert_ne!(theirs["run"]["id"], id.as_str());
    assert!(theirs.get("deduplicated").is_none());
    task.abort();
}

#[tokio::test]
async fn sign_in_pacing_is_not_widened_by_a_signed_in_session() {
    let (base, task) = server(async |_| {}).await;
    let client = admin(&base).await;
    // A signed-in browser that keeps sending wrong passwords is paced like any
    // other source: the session buys no extra guesses.
    let mut statuses = Vec::new();
    for _ in 0..4 {
        statuses.push(
            client
                .post(format!("{base}/admin/login"))
                .json(&json!({"username":"root","password":"wrong"}))
                .send()
                .await
                .unwrap(),
        );
    }
    let limited = statuses
        .iter()
        .find(|r| r.status() == 429)
        .expect("repeated failed sign-ins are refused with 429");
    assert!(
        limited.headers().contains_key("retry-after"),
        "the sign-in refusal says when to try again"
    );
    task.abort();
}
