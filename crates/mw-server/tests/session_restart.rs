//! A real process restart retains a default-key cookie, not logout or expiry.
use serde_json::json;
use std::{
    path::Path,
    process::{Child, Command, Stdio},
    time::Duration,
};
mod common;

struct Running(Child);
impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
async fn start(db: &Path, bind: &str, upstream: &str) -> Running {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mailwoman"));
    command
        .arg("serve")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    for (key, _) in std::env::vars().filter(|(key, _)| key.starts_with("MW_")) {
        command.env_remove(key);
    }
    command
        .env("MW_MODE", "proxy")
        .env("MW_BIND", bind)
        .env("MW_DB_PATH", db)
        .env("MW_UPSTREAM_DEFAULT", upstream)
        .env("MW_JMAP_UPSTREAMS", upstream);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = Running(command.spawn().unwrap());
    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();
    for _ in 0..120 {
        assert!(
            child.0.try_wait().unwrap().is_none(),
            "server exited during startup"
        );
        if client
            .get(format!("http://{bind}/api/me"))
            .send()
            .await
            .is_ok()
        {
            return child;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("server did not start");
}
async fn login(client: &reqwest::Client, base: &str) -> String {
    let response = client
        .post(format!("{base}/api/login"))
        .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    response
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|h| h.to_str().ok())
        .find_map(|h| {
            h.strip_prefix("mw_session=")
                .map(|v| v.split(';').next().unwrap().to_owned())
        })
        .unwrap()
}
async fn status(client: &reqwest::Client, base: &str, session: &str) -> u16 {
    client
        .get(format!("{base}/api/me"))
        .header("Cookie", format!("mw_session={session}"))
        .send()
        .await
        .unwrap()
        .status()
        .as_u16()
}
#[tokio::test]
async fn default_key_sessions_survive_real_process_restart_but_logout_and_expiry_do_not() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let upstream = format!("http://{}", listener.local_addr().unwrap());
    let mock = tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    let reserve = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let bind = reserve.local_addr().unwrap().to_string();
    drop(reserve);
    let base = format!("http://{bind}");
    let db = common::test_db::unique_db_path("persistent-session-restart");
    let process = start(&db, &bind, &upstream).await;
    let pool = sqlx::SqlitePool::connect(&format!("sqlite:{}", db.display()))
        .await
        .unwrap();
    sqlx::query("INSERT INTO setting_policy(key,global_value,updated_by,updated_at) VALUES('signin.retryCooldownSecs','0','restart-fixture',0)").execute(&pool).await.unwrap();
    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();
    let session = login(&client, &base).await;
    assert_eq!(status(&client, &base, &session).await, 200);
    let before: (String, String) =
        sqlx::query_as("SELECT created_at,activity_at FROM sessions WHERE id=?")
            .bind(&session)
            .fetch_one(&pool)
            .await
            .unwrap();
    drop(process);
    let process = start(&db, &bind, &upstream).await;
    assert_eq!(
        status(&client, &base, &session).await,
        200,
        "same cookie survives a fresh process without MW_SERVER_KEY"
    );
    let after: (String, String) =
        sqlx::query_as("SELECT created_at,activity_at FROM sessions WHERE id=?")
            .bind(&session)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(before, after, "restart must not extend session lifetime");
    let response = client
        .post(format!("{base}/api/logout"))
        .header("Origin", &base)
        .header("Cookie", format!("mw_session={session}"))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());
    drop(process);
    let process = start(&db, &bind, &upstream).await;
    assert_eq!(
        status(&client, &base, &session).await,
        401,
        "logout survives restart"
    );
    let expired = login(&client, &base).await;
    sqlx::query("UPDATE sessions SET created_at='2000-01-01T00:00:00Z' WHERE id=?")
        .bind(&expired)
        .execute(&pool)
        .await
        .unwrap();
    drop(process);
    let process = start(&db, &bind, &upstream).await;
    assert_eq!(
        status(&client, &base, &expired).await,
        401,
        "absolute expiry survives restart"
    );
    drop(process);
    pool.close().await;
    mock.abort();
}
