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
fn session_cookie(response: &reqwest::Response) -> String {
    response
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find_map(|v| {
            v.strip_prefix("mw_session=")
                .map(|v| v.split(';').next().unwrap().to_string())
        })
        .unwrap()
}
#[tokio::test]
async fn activity_renews_only_live_mailbox_idle_and_rotation_keeps_fifteen_day_cap() {
    let upstream = serve(mw_mock_jmap::router()).await;
    let db = common::test_db::unique_db_path("mailbox-activity");
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
    let endpoint = format!("{base}/api/account/session/activity");
    let anonymous = reqwest::Client::new();
    assert_eq!(
        anonymous.post(&endpoint).send().await.unwrap().status(),
        403
    );
    assert_eq!(
        anonymous
            .post(&endpoint)
            .bearer_auth("invalid")
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    for native in [false, true] {
        let client = reqwest::Client::builder()
            .cookie_store(!native)
            .build()
            .unwrap();
        let mut body = json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS});
        if native {
            body["clientType"] = json!("native");
        }
        let login = client
            .post(format!("{base}/api/login"))
            .json(&body)
            .send()
            .await
            .unwrap();
        assert_eq!(login.status(), 200);
        let cookie = if native {
            None
        } else {
            Some(session_cookie(&login))
        };
        let data: Value = login.json().await.unwrap();
        let mut id = cookie.unwrap_or_else(|| data["token"].as_str().unwrap().to_string());
        let mut csrf = data["csrfToken"].as_str().unwrap_or("").to_string();
        let now = chrono::Utc::now();
        let created = (now - chrono::Duration::days(14)).to_rfc3339();
        let activity = (now - chrono::Duration::seconds(60)).to_rfc3339();
        sqlx::query("UPDATE sessions SET created_at=?2,activity_at=?3 WHERE id=?1")
            .bind(&id)
            .bind(&created)
            .bind(&activity)
            .execute(&pool)
            .await
            .unwrap();
        let request = client.get(format!("{base}/api/me"));
        let request = if native {
            request.bearer_auth(&id)
        } else {
            request
        };
        assert_eq!(request.send().await.unwrap().status(), 200);
        let untouched: String = sqlx::query_scalar("SELECT activity_at FROM sessions WHERE id=?1")
            .bind(&id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(untouched, activity, "background reads must not renew idle");
        if !native {
            assert_eq!(client.post(&endpoint).send().await.unwrap().status(), 403);
        }
        let request = client.post(&endpoint);
        let request = if native {
            request.bearer_auth(&id)
        } else {
            request.header("x-csrf-token", &csrf)
        };
        let renewed = request.send().await.unwrap();
        assert_eq!(renewed.status(), 200);
        assert_eq!(renewed.headers()["cache-control"], "private, no-store");
        let renewed_at: String = sqlx::query_scalar("SELECT activity_at FROM sessions WHERE id=?1")
            .bind(&id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_ne!(renewed_at, activity);
        let listing = client.get(format!("{base}/api/account/sessions"));
        let listing = if native {
            listing.bearer_auth(&id)
        } else {
            listing
        };
        let listing: Value = listing.send().await.unwrap().json().await.unwrap();
        let current = listing["sessions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["current"] == true)
            .unwrap();
        assert_eq!(current["lastActivity"], renewed_at);

        if !native {
            let response = client
                .post(format!("{base}/api/session/rotate"))
                .header("x-csrf-token", &csrf)
                .send()
                .await
                .unwrap();
            assert_eq!(response.status(), 200);
            id = session_cookie(&response);
            csrf = response.json::<Value>().await.unwrap()["csrfToken"]
                .as_str()
                .unwrap()
                .to_string();
            let clocks: (String, String) =
                sqlx::query_as("SELECT created_at,activity_at FROM sessions WHERE id=?1")
                    .bind(&id)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert_eq!(clocks, (created.clone(), renewed_at));
        }
        // Recent real activity cannot reset the original absolute sign-in deadline.
        sqlx::query("UPDATE sessions SET created_at=?2 WHERE id=?1")
            .bind(&id)
            .bind((now - chrono::Duration::days(15)).to_rfc3339())
            .execute(&pool)
            .await
            .unwrap();
        let request = client.post(&endpoint);
        let request = if native {
            request.bearer_auth(&id)
        } else {
            request.header("x-csrf-token", &csrf)
        };
        assert_eq!(request.send().await.unwrap().status(), 401);
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sessions WHERE id=?1")
            .bind(&id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }
    // Recent background traffic must not conceal an idle mailbox session.
    let idle_client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    let response = idle_client
        .post(format!("{base}/api/login"))
        .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let idle_id = session_cookie(&response);
    let now = chrono::Utc::now();
    sqlx::query("UPDATE sessions SET activity_at=?2,last_seen=?3 WHERE id=?1")
        .bind(&idle_id)
        .bind((now - chrono::Duration::seconds(1800)).to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        idle_client
            .get(format!("{base}/api/me"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );

    // The administrator policy is deliberately independent and remains strict.
    let admin = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    let response = admin
        .post(format!("{base}/admin/login"))
        .json(&json!({"username":"root","password":"hunter2"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let deadlines: (i64, i64) =
        sqlx::query_as("SELECT expires_at,absolute_expires_at FROM admin_sessions LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    let now = chrono::Utc::now().timestamp();
    assert!(deadlines.0 <= now + 1800 && deadlines.0 > now);
    assert!(deadlines.1 <= now + 43200 && deadlines.1 > now);
    sqlx::query("UPDATE admin_sessions SET expires_at=?1")
        .bind(now - 1)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        admin
            .get(format!("{base}/admin/me"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
}
