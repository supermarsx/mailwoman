use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
use std::net::SocketAddr;
mod common;
async fn serve(router: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    format!("http://{address}")
}
#[tokio::test]
async fn password_login_requires_a_fresh_surface_bound_proof_for_browser_and_native() {
    let upstream = serve(mw_mock_jmap::router()).await;
    let db = common::test_db::unique_db_path("signin-challenge");
    let (router, _) = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into(),
            server_key_hex: None,
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig {
                csrf_strict: true,
                ..HardeningConfig::default()
            },
            security: SecurityConfig {
                upstream_default: Some(upstream.clone()),
                jmap_upstreams: Some(vec![upstream]),
                ..SecurityConfig::default()
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
    let base = serve(router).await;
    let pool = sqlx::SqlitePool::connect(&format!("sqlite:{}", db.display()))
        .await
        .unwrap();
    sqlx::query("INSERT INTO setting_policy(key,global_value,updated_by,updated_at) VALUES('signin.retryCooldownSecs','0','test',0),('signin.antiBotMode','\"all\"','test',0),('signin.antiBotDifficulty','100','test',0)").execute(&pool).await.unwrap();
    let client = reqwest::Client::new();
    let public: Value = client
        .get(format!("{base}/api/public-config"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(public["antiBot"], json!({"mailbox":true,"admin":true}));
    for path in ["/api/login", "/admin/login"] {
        let response = client
            .post(format!("{base}{path}"))
            .json(&json!({"username":"root","password":"hunter2","clientType":"native"}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 403);
        let body: Value = response.json().await.unwrap();
        assert_eq!(body["code"], "signinChallengeRequired");
    }
    let first = proof(&client, &base, "mailbox").await;
    // A proof for one login surface cannot be spent on another.
    assert_eq!(
        client
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"hunter2","altcha":first}))
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    let native = json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS,"clientType":"native","altcha":first});
    let (a, b) = tokio::join!(
        client
            .post(format!("{base}/api/login"))
            .json(&native)
            .send(),
        client
            .post(format!("{base}/api/login"))
            .json(&native)
            .send()
    );
    let mut statuses = [a.unwrap().status().as_u16(), b.unwrap().status().as_u16()];
    statuses.sort();
    assert_eq!(statuses, [200, 403]);
    let second = proof(&client, &base, "admin").await;
    assert_eq!(
        client
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"wrong","altcha":second}))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(
        client
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"hunter2","altcha":second}))
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    tokio::time::sleep(std::time::Duration::from_millis(3100)).await;
    let third = proof(&client, &base, "admin").await;
    use base64::Engine as _;
    let mut tampered: Value = serde_json::from_slice(
        &base64::engine::general_purpose::STANDARD
            .decode(&third)
            .unwrap(),
    )
    .unwrap();
    tampered["solution"]["derivedKey"] = json!("0".repeat(64));
    let bad = base64::engine::general_purpose::STANDARD.encode(tampered.to_string());
    assert_eq!(
        client
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"hunter2","altcha":bad}))
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        client
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"hunter2","altcha":third}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    tokio::time::sleep(std::time::Duration::from_millis(3100)).await;
    let fourth = proof(&client, &base, "admin").await;
    sqlx::query("UPDATE signin_challenge SET expires_at=0")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        client
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"hunter2","altcha":fourth}))
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    sqlx::query("UPDATE setting_policy SET global_value='\"off\"' WHERE key='signin.antiBotMode'")
        .execute(&pool)
        .await
        .unwrap();
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
}
async fn proof(client: &reqwest::Client, base: &str, surface: &str) -> String {
    use base64::Engine as _;
    let response = client
        .get(format!("{base}/api/signin/challenge?surface={surface}"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["cache-control"], "no-store");
    let challenge: altcha::Challenge = response.json().await.unwrap();
    let solution = altcha::solve_challenge(altcha::SolveChallengeOptions::new(&challenge))
        .unwrap()
        .unwrap();
    base64::engine::general_purpose::STANDARD.encode(
        serde_json::to_vec(&altcha::Payload {
            challenge,
            solution,
        })
        .unwrap(),
    )
}
