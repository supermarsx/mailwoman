//! `signin.newUsers` through the real `/api/login` route (proxy mode against the
//! mock JMAP upstream): every rule value, existing users never refused, no partial
//! state after a refusal, and uniform versus explained refusals.
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::settings_policy::{SettingPolicyRow, SettingScope, SettingValueRow};
use serde_json::{Value, json};
use std::net::SocketAddr;
mod common;

const KEY: &str = "5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b";
const DOMAIN: &str = "example.org";

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

struct App {
    base: String,
    store: mw_store::Store,
}

async fn build(tag: &str) -> App {
    let upstream = serve(mw_mock_jmap::router()).await;
    let db = common::test_db::unique_db_path(tag);
    let store = mw_store::Store::open(
        db.to_str().unwrap(),
        mw_store::ServerKey::from_hex(KEY).unwrap(),
    )
    .await
    .unwrap();
    let (router, _) = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into(),
            server_key_hex: Some(KEY.into()),
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
    // Repeated attempts from one source would otherwise meet the retry cooldown (429).
    set_global(&store, "signin.retryCooldownSecs", json!(0)).await;
    App {
        base: serve(router).await,
        store,
    }
}

async fn set_global(store: &mw_store::Store, key: &str, value: Value) {
    store
        .put_setting_policy(&SettingPolicyRow {
            key: key.into(),
            global_value: Some(value.to_string()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "fixture".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
}

async fn login(app: &App) -> reqwest::Response {
    reqwest::Client::builder()
        .no_proxy()
        .build()
        .unwrap()
        .post(format!("{}/api/login", app.base))
        .json(&json!({"username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap()
}

async fn assert_nothing_left(app: &App) {
    let standing = app
        .store
        .directory_standing(&[mw_mock_jmap::USER])
        .await
        .unwrap();
    assert!(
        standing.as_ref().is_none_or(|s| !s.has_signed_in()),
        "a refused sign-in must not register the user"
    );
}

async fn refused(app: &App, status: u16) {
    let response = login(app).await;
    assert_eq!(response.status(), status);
    assert!(
        response.headers().get("set-cookie").is_none(),
        "a refusal must not set a session cookie"
    );
    assert_nothing_left(app).await;
}

#[tokio::test]
async fn open_admits_a_new_user() {
    let app = build("signin-policy-open").await;
    set_global(&app.store, "signin.newUsers", json!("open")).await;
    assert_eq!(login(&app).await.status(), 200);
}

#[tokio::test]
async fn closed_refuses_a_new_user_even_with_a_pending_entry_but_never_an_existing_one() {
    let app = build("signin-policy-closed").await;
    set_global(&app.store, "signin.newUsers", json!("closed")).await;
    refused(&app, 403).await;
    assert!(
        app.store
            .put_provisioned_user(
                mw_mock_jmap::USER,
                "provisioned",
                None,
                None,
                "fixture",
                chrono::Utc::now().timestamp()
            )
            .await
            .unwrap()
    );
    refused(&app, 403).await;
    let existing = build("signin-policy-existing").await;
    assert_eq!(login(&existing).await.status(), 200);
    set_global(&existing.store, "signin.newUsers", json!("closed")).await;
    assert_eq!(login(&existing).await.status(), 200);
}

#[tokio::test]
async fn preprovisioned_and_provisioned_need_a_pending_entry_which_is_spent_on_first_sign_in() {
    for rule in ["preprovisioned", "provisioned"] {
        let app = build(&format!("signin-policy-{rule}")).await;
        set_global(&app.store, "signin.newUsers", json!(rule)).await;
        refused(&app, 403).await;
        assert!(
            app.store
                .put_provisioned_user(
                    mw_mock_jmap::USER,
                    "provisioned",
                    None,
                    None,
                    "fixture",
                    chrono::Utc::now().timestamp(),
                )
                .await
                .unwrap()
        );
        assert_eq!(login(&app).await.status(), 200, "{rule}");
        let standing = app
            .store
            .directory_standing(&[mw_mock_jmap::USER])
            .await
            .unwrap()
            .unwrap();
        assert!(standing.has_signed_in(), "{rule}: the entry is consumed");
        assert_eq!(login(&app).await.status(), 200, "{rule}");
    }
}

#[tokio::test]
async fn an_expired_pending_entry_is_not_an_admission() {
    let app = build("signin-policy-expired").await;
    set_global(&app.store, "signin.newUsers", json!("preprovisioned")).await;
    app.store
        .put_provisioned_user(mw_mock_jmap::USER, "provisioned", None, Some(5), "fixture", 1)
        .await
        .unwrap();
    refused(&app, 403).await;
}

#[tokio::test]
async fn uniform_refusals_look_like_a_wrong_password_and_explained_ones_carry_the_code() {
    let app = build("signin-policy-detail").await;
    set_global(&app.store, "signin.newUsers", json!("closed")).await;
    let explained = login(&app).await;
    assert_eq!(explained.status(), 403);
    let body: Value = explained.json().await.unwrap();
    assert_eq!(body["code"], "signinNotProvisioned");
    set_global(&app.store, "signin.refusalDetail", json!("uniform")).await;
    let uniform = login(&app).await;
    assert_eq!(uniform.status(), 401);
    let body: Value = uniform.json().await.unwrap_or(Value::Null);
    assert_ne!(body["code"], "signinNotProvisioned");
    assert_nothing_left(&app).await;
}

#[tokio::test]
async fn a_domain_value_overrides_the_global_rule_for_that_domain_only() {
    let app = build("signin-policy-domain").await;
    set_global(&app.store, "signin.newUsers", json!("closed")).await;
    app.store
        .put_setting_value(&SettingValueRow {
            scope: SettingScope::Domain,
            scope_id: DOMAIN.into(),
            key: "signin.newUsers".into(),
            value: json!("open").to_string(),
            locked: false,
            updated_by: "fixture".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
    assert_eq!(login(&app).await.status(), 200);
}

#[tokio::test]
async fn the_sign_in_type_key_beats_the_base_key() {
    let app = build("signin-policy-type").await;
    set_global(&app.store, "signin.newUsers", json!("open")).await;
    set_global(&app.store, "signin.newUsers.mail", json!("closed")).await;
    refused(&app, 403).await;
}
