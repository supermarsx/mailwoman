//! Administrator self-service TOTP is independent of mailbox MFA.
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use reqwest::{Client, Response};
use serde_json::{Value, json};
use std::net::SocketAddr;
mod common;
use common::test_db;

async fn server() -> String {
    let web = test_db::unique_dir("admin-totp-web");
    std::fs::write(web.join("index.html"), "<!doctype html><title>Test</title>").unwrap();
    let db = test_db::unique_db_path("admin-totp")
        .to_string_lossy()
        .into_owned();
    // A sign-in that stops at the second factor issues no session, so it keeps its
    // sign-in pacing charge (t33 D1); these legs make several in a row and are not
    // about pacing (`signin.retryCooldownSecs`, d94472d7). 0 switches it off.
    let key = "5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b";
    mw_store::Store::open(&db, mw_store::ServerKey::from_hex(key).unwrap())
        .await
        .unwrap()
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: "signin.retryCooldownSecs".into(),
            global_value: Some("0".into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "admin-totp-fixture".into(),
            updated_at: 0,
        })
        .await
        .unwrap();
    let (app, _) = build_app_full(
        AppConfig {
            db_path: db,
            server_key_hex: Some(key.into()),
            web_dir: Some(web),
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
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    format!("http://{address}")
}
fn client() -> Client {
    Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap()
}
async fn post(client: &Client, base: &str, path: &str, body: Value) -> Response {
    client
        .post(format!("{base}/admin{path}"))
        .json(&body)
        .send()
        .await
        .unwrap()
}
async fn login(client: &Client, base: &str) -> Response {
    post(
        client,
        base,
        "/login",
        json!({"username":"root","password":"hunter2"}),
    )
    .await
}
async fn prove(client: &Client, base: &str, extra: Value) {
    let mut body = extra;
    body["password"] = json!("hunter2");
    let response = post(client, base, "/step-up", body).await;
    let status = response.status();
    assert_eq!(status, 200, "{}", response.text().await.unwrap());
}

#[tokio::test]
async fn self_service_totp_binds_enrollment_and_supports_lost_factor_recovery() {
    use mw_mfa::totp::{TotpParams, base32_decode, totp_at};
    let base = server().await;
    let a = client();
    assert_eq!(login(&a, &base).await.status(), 200);
    assert_eq!(
        post(&a, &base, "/me/totp", json!({})).await.status(),
        403,
        "fresh proof required before revealing a setup secret"
    );
    prove(&a, &base, json!({})).await;
    let started = post(&a, &base, "/me/totp", json!({})).await;
    assert_eq!(started.headers()["cache-control"], "no-store");
    let setup: Value = started.json().await.unwrap();
    assert!(
        setup["otpauthUri"]
            .as_str()
            .unwrap()
            .starts_with("otpauth://totp/")
    );
    let secret = base32_decode(setup["secret"].as_str().unwrap()).unwrap();
    let b = client();
    assert_eq!(login(&b, &base).await.status(), 200);
    prove(&b, &base, json!({})).await;
    let code = totp_at(
        &secret,
        chrono::Utc::now().timestamp() as u64,
        &TotpParams::default(),
    );
    assert_eq!(
        post(&b, &base, "/me/totp/confirm", json!({"code":code}))
            .await
            .status(),
        409,
        "another session cannot finish this setup"
    );
    for _ in 0..5 {
        assert_eq!(
            post(&a, &base, "/me/totp/confirm", json!({"code":"invalid"}))
                .await
                .status(),
            422
        );
    }
    assert_eq!(
        post(&a, &base, "/me/totp/confirm", json!({"code":code}))
            .await
            .status(),
        409,
        "five failed attempts consume setup"
    );
    let setup: Value = post(&a, &base, "/me/totp", json!({}))
        .await
        .json()
        .await
        .unwrap();
    let secret = base32_decode(setup["secret"].as_str().unwrap()).unwrap();
    let code = totp_at(
        &secret,
        chrono::Utc::now().timestamp() as u64,
        &TotpParams::default(),
    );
    let confirmed = post(&a, &base, "/me/totp/confirm", json!({"code":code})).await;
    assert_eq!(confirmed.status(), 200);
    assert_eq!(confirmed.headers()["cache-control"], "no-store");
    let confirmed: Value = confirmed.json().await.unwrap();
    let initial = confirmed["recoveryCodes"].as_array().unwrap();
    assert_eq!(initial.len(), mw_mfa::recovery::DEFAULT_RECOVERY_CODES);
    assert_eq!(
        post(&a, &base, "/me/recovery", json!({})).await.status(),
        403,
        "adding factor invalidates password-only proof"
    );
    let next_code = totp_at(
        &secret,
        chrono::Utc::now().timestamp() as u64 + 30,
        &TotpParams::default(),
    );
    prove(&a, &base, json!({"code":next_code})).await;
    let rotated = post(&a, &base, "/me/recovery", json!({})).await;
    assert_eq!(
        rotated.status(),
        200,
        "TOTP-only administrators can rotate recovery codes"
    );
    let rotated: Value = rotated.json().await.unwrap();
    let codes = rotated["recoveryCodes"].as_array().unwrap();
    assert_eq!(codes.len(), mw_mfa::recovery::DEFAULT_RECOVERY_CODES);
    let recovered = client();
    let pending = login(&recovered, &base).await;
    assert_eq!(
        pending.status(),
        200,
        "admin password challenge uses the existing 200/twofaRequired contract"
    );
    assert!(
        pending.headers().get("set-cookie").is_none(),
        "pending challenge must not mint an admin session"
    );
    let pending: Value = pending.json().await.unwrap();
    assert_eq!(pending["twofaRequired"], true);
    assert!(
        pending["factors"]
            .as_array()
            .unwrap()
            .contains(&json!("totp"))
    );
    assert!(
        pending["factors"]
            .as_array()
            .unwrap()
            .contains(&json!("recovery"))
    );
    assert_eq!(
        post(
            &recovered,
            &base,
            "/login/recovery",
            json!({"pendingToken":pending["pendingToken"],"code":codes[0]})
        )
        .await
        .status(),
        200
    );
    let replay = client();
    let pending: Value = login(&replay, &base).await.json().await.unwrap();
    assert_eq!(
        post(
            &replay,
            &base,
            "/login/recovery",
            json!({"pendingToken":pending["pendingToken"],"code":codes[0]})
        )
        .await
        .status(),
        401
    );
    let required = recovered
        .delete(format!("{base}/admin/me/totp"))
        .send()
        .await
        .unwrap();
    assert_eq!(required.status(), 403);
    let required: Value = required.json().await.unwrap();
    assert!(
        required["methods"]
            .as_array()
            .unwrap()
            .contains(&json!("recovery")),
        "the browser must be offered recovery for TOTP-only administrators"
    );
    prove(&recovered, &base, json!({"recoveryCode":codes[1]})).await;
    assert_eq!(
        recovered
            .delete(format!("{base}/admin/me/totp"))
            .send()
            .await
            .unwrap()
            .status(),
        204,
        "fresh recovery proof removes lost authenticator without requiring its code again"
    );
    let me: Value = recovered
        .get(format!("{base}/admin/me"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(me["totpEnrolled"], false);
    let password_only = client();
    let signed_in: Value = login(&password_only, &base).await.json().await.unwrap();
    assert!(signed_in.get("twofaRequired").is_none());
    assert_eq!(
        password_only
            .get(format!("{base}/admin/me"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
}

#[tokio::test]
async fn administrator_without_account_management_can_enroll_only_their_own_factor() {
    use mw_mfa::totp::{TotpParams, base32_decode, totp_at};
    let base = server().await;
    let root = client();
    assert_eq!(login(&root, &base).await.status(), 200);
    prove(&root, &base, json!({})).await;
    assert_eq!(
        post(
            &root,
            &base,
            "/admins",
            json!({"username":"limited","password":"long test password","roles":[]})
        )
        .await
        .status(),
        201
    );
    let limited = client();
    assert_eq!(
        post(
            &limited,
            &base,
            "/login",
            json!({"username":"limited","password":"long test password"})
        )
        .await
        .status(),
        200
    );
    assert_eq!(
        limited
            .get(format!("{base}/admin/admins"))
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        post(
            &limited,
            &base,
            "/step-up",
            json!({"password":"long test password"})
        )
        .await
        .status(),
        200
    );
    let setup = post(&limited, &base, "/me/totp", json!({})).await;
    assert_eq!(setup.status(), 200);
    let setup: Value = setup.json().await.unwrap();
    let secret = base32_decode(setup["secret"].as_str().unwrap()).unwrap();
    let code = totp_at(
        &secret,
        chrono::Utc::now().timestamp() as u64,
        &TotpParams::default(),
    );
    assert_eq!(
        post(
            &limited,
            &base,
            "/me/totp/confirm",
            json!({"code":code,"accountId":"root"})
        )
        .await
        .status(),
        200
    );
    let me: Value = limited
        .get(format!("{base}/admin/me"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(me["totpEnrolled"], true);
    let root_me: Value = root
        .get(format!("{base}/admin/me"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        root_me["totpEnrolled"], false,
        "body cannot retarget the signed-in principal"
    );
}
