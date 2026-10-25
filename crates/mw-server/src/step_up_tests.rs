//! Step-up over HTTP (t32-ext §14.3 tests "step-up required and expiring"): a real
//! server in proxy mode over the in-repo mock JMAP upstream, a probe route that
//! takes the extractor (mounted only in this crate's unit tests), and the mint
//! routes.

use std::net::SocketAddr;

use axum::http::StatusCode;
use base64::Engine as _;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use mw_mfa::totp::{self, TotpParams};
use mw_store::{ServerKey, Store, WebauthnCredentialRow};

use super::{RequireAdminStepUp, RequireStepUp, action};
use crate::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};

#[allow(dead_code)]
#[path = "../tests/common/mock_mail.rs"]
mod security_mock_mail;

const KEY_HEX: &str = "4b1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0";
const ADMIN_USER: &str = "root";
const ADMIN_PASS: &str = "hunter2-step-up";

/// The probe the tests guard: a user data export.
pub(super) async fn user_probe(_s: RequireStepUp<action::DataExport>) -> StatusCode {
    StatusCode::NO_CONTENT
}

/// The admin probe: a configuration import.
pub(super) async fn admin_probe(_s: RequireAdminStepUp<action::ConfigImport>) -> StatusCode {
    StatusCode::NO_CONTENT
}

fn unique() -> String {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    format!(
        "{}_{}_{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )
}

struct Server {
    base: String,
    addr: SocketAddr,
    db: String,
}

impl Server {
    async fn store(&self) -> Store {
        Store::open(&self.db, ServerKey::from_hex(KEY_HEX).unwrap())
            .await
            .unwrap()
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }
}

async fn spawn() -> Server {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mock = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });

    let dir = std::env::temp_dir().join(format!("mw-f31-stepup-{}", unique()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("index.html"), "<!doctype html><title>MW</title>").unwrap();
    let db = dir.join("mw.db").to_string_lossy().into_owned();
    let config = AppConfig {
        db_path: db.clone(),
        server_key_hex: Some(KEY_HEX.into()),
        web_dir: Some(dir),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
            jmap_upstreams: Some(vec![mock.clone()]),
            upstream_default: Some(mock),
            ..SecurityConfig::default()
        },
    };
    let v6 = V6Config {
        admin_enabled: true,
        admin_username: Some(ADMIN_USER.into()),
        admin_password: Some(ADMIN_PASS.into()),
        redis_url: None,
    };
    let app = build_app_full(config, v6).await.expect("server boots").0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    Server {
        base: format!("http://{addr}"),
        addr,
        db,
    }
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap()
}

async fn post(c: &reqwest::Client, url: String, body: Value) -> (u16, Value) {
    let resp = c.post(url).json(&body).send().await.unwrap();
    let status = resp.status().as_u16();
    let body = resp.json::<Value>().await.unwrap_or(Value::Null);
    (status, body)
}

async fn get(c: &reqwest::Client, url: String) -> (u16, Value) {
    let resp = c.get(url).send().await.unwrap();
    let status = resp.status().as_u16();
    (status, resp.json::<Value>().await.unwrap_or(Value::Null))
}

/// Sign in to the mailbox; returns the login body (a pending 2FA step when the
/// account has a factor).
async fn login(s: &Server, c: &reqwest::Client) -> Value {
    let (status, body) = post(
        c,
        s.url("/api/login"),
        json!({ "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS }),
    )
    .await;
    assert_eq!(status, 200, "login: {body}");
    body
}

async fn probe(s: &Server, c: &reqwest::Client) -> (u16, Value) {
    post(c, s.url("/api/security/step-up/probe"), json!({})).await
}

/// Security events written for the mock account, oldest first.
async fn events(s: &Server) -> Vec<(String, Option<String>)> {
    let store = s.store().await;
    let mut out = Vec::new();
    for row in store.list_audit(200).await.unwrap().into_iter().rev() {
        if row.target.as_deref() == Some(&format!("account:{}", mw_mock_jmap::ACCOUNT_ID)) {
            let cat = store.audit_category(&row.id).await.unwrap();
            out.push((row.action, cat));
        }
    }
    out
}

#[tokio::test]
async fn password_proof_unlocks_the_action_for_this_session_only() {
    let s = spawn().await;
    let a = client();
    login(&s, &a).await;

    let (status, body) = probe(&s, &a).await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["code"], "step_up_required");
    assert_eq!(body["action"], "dataExport");
    assert_eq!(body["methods"], json!(["password"]));
    let (_, st) = get(&a, s.url("/api/security/step-up")).await;
    assert_eq!(st["active"], false);

    let (status, body) = post(
        &a,
        s.url("/api/security/step-up"),
        json!({ "password": "not-it" }),
    )
    .await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["code"], "stepUpFailed");
    assert_eq!(body["attemptsLeft"], 4);
    assert_eq!(probe(&s, &a).await.0, 403, "a wrong password mints nothing");

    let (status, body) = post(
        &a,
        s.url("/api/security/step-up"),
        json!({ "password": mw_mock_jmap::PASS }),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["expiresIn"], 300);
    assert_eq!(probe(&s, &a).await.0, 204, "the proof unlocks the action");
    let (_, st) = get(&a, s.url("/api/security/step-up")).await;
    assert_eq!(st["active"], true);
    assert!(st["expiresIn"].as_u64().unwrap() > 290);

    // Another session of the same account has no proof.
    let b = client();
    login(&s, &b).await;
    assert_eq!(
        probe(&s, &b).await.0,
        403,
        "the proof is bound to its session"
    );

    // Signing out ends the proof with the session.
    let (status, _) = post(&a, s.url("/api/logout"), json!({})).await;
    assert!(status < 300);
    assert_eq!(probe(&s, &a).await.0, 401);
    login(&s, &a).await;
    assert_eq!(
        probe(&s, &a).await.0,
        403,
        "a new session starts without one"
    );

    let ev = events(&s).await;
    let security = Some("security".to_string());
    assert!(
        ev.contains(&("step-up-refused".into(), security.clone())),
        "{ev:?}"
    );
    assert!(ev.contains(&("step-up-granted".into(), security)), "{ev:?}");
}

#[tokio::test]
async fn real_sensitive_user_routes_refuse_before_mutation_and_key_mint_retries() {
    let server = spawn().await;
    let client = client();
    login(&server, &client).await;
    for (path, body) in [
        ("/api/account/2fa/totp/disable", json!({})),
        (
            "/api/account/2fa/passkey/remove",
            json!({"handle":"missing"}),
        ),
        ("/api/account/2fa/recovery/regenerate", json!({})),
        (
            "/api/keys",
            json!({"scope":mw_oauth::Scope::read_only(mw_mock_jmap::ACCOUNT_ID)}),
        ),
    ] {
        let (status, body) = post(&client, server.url(path), body).await;
        assert_eq!(status, 403, "{path}: {body}");
        assert_eq!(body["code"], "step_up_required", "{path}: {body}");
    }
    assert!(
        server
            .store()
            .await
            .list_api_keys()
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        server
            .store()
            .await
            .list_unused_recovery_codes(mw_mock_jmap::ACCOUNT_ID)
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        post(
            &client,
            server.url("/api/security/step-up"),
            json!({"password":mw_mock_jmap::PASS})
        )
        .await
        .0,
        200
    );
    let (status, body) = post(
        &client,
        server.url("/api/keys"),
        json!({"scope":mw_oauth::Scope::read_only(mw_mock_jmap::ACCOUNT_ID)}),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(server.store().await.list_api_keys().await.unwrap().len(), 1);
}

#[tokio::test]
async fn enrolling_a_factor_revokes_an_existing_password_proof() {
    let server = spawn().await;
    let client = client();
    login(&server, &client).await;
    let (status, body) = post(
        &client,
        server.url("/api/security/step-up"),
        json!({"password":mw_mock_jmap::PASS}),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(probe(&server, &client).await.0, 204);
    server
        .store()
        .await
        .put_totp_secret(mw_mock_jmap::ACCOUNT_ID, &totp::generate_secret(), true)
        .await
        .unwrap();
    let (status, body) = probe(&server, &client).await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["methods"], json!(["password", "totp"]));
    assert_eq!(
        get(&client, server.url("/api/security/step-up")).await.1["active"],
        false
    );
}

#[tokio::test]
async fn an_account_with_totp_confirms_with_password_and_code() {
    let s = spawn().await;
    let secret = totp::generate_secret();
    s.store()
        .await
        .put_totp_secret(mw_mock_jmap::ACCOUNT_ID, &secret, true)
        .await
        .unwrap();
    let a = client();
    let pending = login(&s, &a).await;
    let now = now_unix();
    let code = totp::totp_at(&secret, now, &TotpParams::default());
    let (status, body) = post(
        &a,
        s.url("/api/login/2fa"),
        json!({ "pendingToken": pending["pendingToken"], "method": "totp", "code": code }),
    )
    .await;
    assert_eq!(status, 200, "{body}");

    let (status, body) = probe(&s, &a).await;
    assert_eq!(status, 403);
    assert_eq!(body["methods"], json!(["password", "totp"]));

    // The password alone is not enough, and the answer does not say whether it
    // was right.
    for pw in [mw_mock_jmap::PASS, "not-it"] {
        let (status, body) = post(
            &a,
            s.url("/api/security/step-up"),
            json!({ "password": pw }),
        )
        .await;
        assert_eq!(status, 422, "{body}");
        assert_eq!(body["code"], "stepUpIncomplete");
    }
    // The code the sign-in used is refused (replay guard).
    let (status, body) = post(
        &a,
        s.url("/api/security/step-up"),
        json!({ "password": mw_mock_jmap::PASS, "code": code }),
    )
    .await;
    assert_eq!(status, 403, "{body}");
    // A code without the password is incomplete for a password session.
    let next = totp::totp_at(&secret, now + 30, &TotpParams::default());
    let (status, _) = post(&a, s.url("/api/security/step-up"), json!({ "code": next })).await;
    assert_eq!(status, 422);
    let (status, body) = post(
        &a,
        s.url("/api/security/step-up"),
        json!({ "password": mw_mock_jmap::PASS, "code": next }),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(probe(&s, &a).await.0, 204);
}

#[tokio::test]
async fn five_wrong_attempts_lock_step_up() {
    let s = spawn().await;
    let a = client();
    login(&s, &a).await;
    for left in [4, 3, 2, 1] {
        let (status, body) = post(
            &a,
            s.url("/api/security/step-up"),
            json!({ "password": "x" }),
        )
        .await;
        assert_eq!(status, 403);
        assert_eq!(body["attemptsLeft"], left);
    }
    let (status, body) = post(
        &a,
        s.url("/api/security/step-up"),
        json!({ "password": "x" }),
    )
    .await;
    assert_eq!(status, 429, "{body}");
    assert_eq!(body["code"], "stepUpLocked");
    let (status, _) = post(
        &a,
        s.url("/api/security/step-up"),
        json!({ "password": mw_mock_jmap::PASS }),
    )
    .await;
    assert_eq!(status, 429, "the right password does not lift the lock");
    assert_eq!(probe(&s, &a).await.0, 403);
    let ev = events(&s).await;
    assert!(
        ev.iter().any(|(a, _)| a == "step-up-locked"),
        "the lock is audited: {ev:?}"
    );
}

#[tokio::test]
async fn a_passkey_alone_needs_user_verification() {
    let s = spawn().await;
    let authn = Es256Authenticator::new();
    let cred_id = b64url(b"f31-step-up-cred");
    s.store()
        .await
        .add_webauthn_credential(&WebauthnCredentialRow {
            credential_id: cred_id.clone(),
            account_id: mw_mock_jmap::ACCOUNT_ID.to_string(),
            cose_public_key: authn.cose(),
            sign_count: 1,
            transports: "internal".into(),
            label: "Virtual".into(),
            created_at: String::new(),
        })
        .await
        .unwrap();
    let origin = format!("http://{}", s.addr);
    let rp_id = s.addr.ip().to_string();

    // Sign in with the passkey as the second factor.
    let a = client();
    let pending = login(&s, &a).await;
    let ch = decode(pending["webauthn"]["challenge"].as_str().unwrap());
    let asr = authn.assert(&rp_id, &origin, &ch, 2, true);
    let (status, body) = post(
        &a,
        s.url("/api/login/2fa"),
        json!({
            "pendingToken": pending["pendingToken"],
            "method": "webauthn",
            "credentialId": cred_id,
            "clientDataJson": b64url(&asr.client_data_json),
            "authenticatorData": b64url(&asr.authenticator_data),
            "signature": b64url(&asr.signature),
        }),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    let (_, body) = probe(&s, &a).await;
    assert_eq!(body["methods"], json!(["password", "passkey"]));

    let begin = |c: reqwest::Client| {
        let url = s.url("/api/security/step-up/passkey");
        async move {
            let (status, body) = post(&c, url, json!({})).await;
            assert_eq!(status, 200, "{body}");
            assert_eq!(body["credentialIds"], json!([b64url(b"f31-step-up-cred")]));
            decode(body["challenge"].as_str().unwrap())
        }
    };
    let assertion = |asr: &Assertion| {
        json!({
            "credentialId": cred_id,
            "clientDataJson": b64url(&asr.client_data_json),
            "authenticatorData": b64url(&asr.authenticator_data),
            "signature": b64url(&asr.signature),
        })
    };

    // Presence only, no verification: refused on its own.
    let ch = begin(a.clone()).await;
    let up_only = authn.assert(&rp_id, &origin, &ch, 3, false);
    let (status, body) = post(
        &a,
        s.url("/api/security/step-up"),
        json!({ "passkey": assertion(&up_only) }),
    )
    .await;
    assert_eq!(status, 403, "{body}");

    // The same assertion again: its challenge is spent.
    let (status, _) = post(
        &a,
        s.url("/api/security/step-up"),
        json!({ "password": mw_mock_jmap::PASS, "passkey": assertion(&up_only) }),
    )
    .await;
    assert_eq!(status, 403, "a challenge is single use");

    // Presence plus the password: granted.
    let ch = begin(a.clone()).await;
    let up_only = authn.assert(&rp_id, &origin, &ch, 4, false);
    let (status, body) = post(
        &a,
        s.url("/api/security/step-up"),
        json!({ "password": mw_mock_jmap::PASS, "passkey": assertion(&up_only) }),
    )
    .await;
    assert_eq!(status, 200, "{body}");

    // A verified passkey on its own: granted (a fresh session, so no proof yet).
    let b = client();
    let pending = login(&s, &b).await;
    let ch = decode(pending["webauthn"]["challenge"].as_str().unwrap());
    let asr = authn.assert(&rp_id, &origin, &ch, 5, true);
    let (status, _) = post(
        &b,
        s.url("/api/login/2fa"),
        json!({
            "pendingToken": pending["pendingToken"],
            "method": "webauthn",
            "credentialId": cred_id,
            "clientDataJson": b64url(&asr.client_data_json),
            "authenticatorData": b64url(&asr.authenticator_data),
            "signature": b64url(&asr.signature),
        }),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(probe(&s, &b).await.0, 403);
    let ch = begin(b.clone()).await;
    let uv = authn.assert(&rp_id, &origin, &ch, 6, true);
    let (status, body) = post(
        &b,
        s.url("/api/security/step-up"),
        json!({ "passkey": assertion(&uv) }),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(probe(&s, &b).await.0, 204);
}

#[tokio::test]
async fn admin_routes_need_an_admin_proof() {
    let s = spawn().await;
    let admin_path = mw_admin::config::admin_path();
    let a = client();
    let (status, _) = post(
        &a,
        s.url(&format!("{admin_path}/login")),
        json!({ "username": ADMIN_USER, "password": ADMIN_PASS }),
    )
    .await;
    assert_eq!(status, 200);
    let probe_url = s.url(&format!("{admin_path}/step-up/probe"));
    let mint_url = s.url(&format!("{admin_path}/step-up"));

    let (status, body) = post(&a, probe_url.clone(), json!({})).await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["code"], "step_up_required");
    assert_eq!(body["action"], "configImport");
    assert_eq!(body["methods"], json!(["password"]));

    let role_url = s.url(&format!("{admin_path}/roles"));
    let role_body = json!({"name":"stepup-test-role", "capabilities":["users.read"]});
    let (status, body) = post(&a, role_url.clone(), role_body.clone()).await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["code"], "step_up_required");
    let (status, body) = post(&a, mint_url.clone(), json!({ "password": "nope" })).await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["code"], "stepUpFailed");
    let (status, body) = post(&a, mint_url.clone(), json!({ "password": ADMIN_PASS })).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(post(&a, probe_url.clone(), json!({})).await.0, 204);
    let (_, st) = get(&a, mint_url.clone()).await;
    assert_eq!(st["active"], true);

    let (status, body) = post(&a, role_url, role_body).await;
    assert_eq!(status, 201, "{body}");

    // A second admin session, and a mailbox session, have no admin proof.
    let b = client();
    post(
        &b,
        s.url(&format!("{admin_path}/login")),
        json!({ "username": ADMIN_USER, "password": ADMIN_PASS }),
    )
    .await;
    assert_eq!(post(&b, probe_url.clone(), json!({})).await.0, 403);
    let m = client();
    login(&s, &m).await;
    post(
        &m,
        s.url("/api/security/step-up"),
        json!({ "password": mw_mock_jmap::PASS }),
    )
    .await;
    assert_eq!(
        post(&m, probe_url, json!({})).await.0,
        401,
        "a mailbox proof is not an admin session"
    );
}

#[tokio::test]
async fn personal_admin_proof_requires_current_password_and_nonreplayed_totp() {
    let s = spawn().await;
    let store = s.store().await;
    let password = "personal-stepup-password";
    let hash =
        mw_admin::password::hash_password(password, &mw_admin::SecurityPolicy::default()).unwrap();
    store
        .insert_admin_account(&mw_store::AdminAccountRow {
            id: "personal".into(),
            username: "personal".into(),
            display_name: "Personal".into(),
            password_hash: Some(hash),
            disabled: false,
            source: "db".into(),
            created_at: now_unix() as i64,
            updated_at: now_unix() as i64,
            last_login_at: None,
        })
        .await
        .unwrap();
    let secret = totp::generate_secret();
    store.put_admin_totp("personal", &secret, -1).await.unwrap();
    let c = client();
    let prefix = mw_admin::config::admin_path();
    let (status, pending) = post(
        &c,
        s.url(&format!("{prefix}/login")),
        json!({"username":"personal","password":password}),
    )
    .await;
    assert_eq!(status, 200, "{pending}");
    let now = now_unix();
    let code = totp::totp_at(&secret, now, &TotpParams::default());
    let (status, body) = post(
        &c,
        s.url(&format!("{prefix}/login/totp")),
        json!({"pendingToken":pending["pendingToken"],"code":code}),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    let mint = s.url(&format!("{prefix}/step-up"));
    let probe = s.url(&format!("{prefix}/step-up/probe"));
    assert_eq!(
        post(&c, mint.clone(), json!({"password":password})).await.0,
        422
    );
    assert_eq!(
        post(&c, mint.clone(), json!({"password":password,"code":code}))
            .await
            .0,
        403
    );
    let next = totp::totp_at(&secret, now + 30, &TotpParams::default());
    let (status, body) = post(&c, mint.clone(), json!({"password":password,"code":next})).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(post(&c, probe.clone(), json!({})).await.0, 204);
    assert_eq!(
        post(&c, mint, json!({"password":password,"code":next}))
            .await
            .0,
        403
    );
    store
        .put_admin_totp("personal", &totp::generate_secret(), -1)
        .await
        .unwrap();
    assert_eq!(post(&c, probe, json!({})).await.0, 403);
}

async fn security_setting(store: &Store, key: &str, value: Value) {
    store
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: key.into(),
            global_value: Some(value.to_string()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "test".into(),
            updated_at: now_unix() as i64,
        })
        .await
        .unwrap();
}
#[tokio::test]
async fn security_center_records_device_history_and_enforces_session_cap() {
    let s = spawn().await;
    let store = s.store().await;
    security_setting(&store, "session.maxConcurrent", json!(1)).await;
    let old = client();
    login(&s, &old).await;
    let fresh = client();
    let response = fresh
        .post(s.url("/api/login"))
        .header("User-Agent", "SecurityTest/1")
        .header("X-Mailwoman-Client", "browser/26.20")
        .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(get(&old, s.url("/api/me")).await.0, 401);
    let (status, body) = get(&fresh, s.url("/api/account/sessions")).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["sessions"].as_array().unwrap().len(), 1);
    assert_eq!(body["sessions"][0]["clientVersion"], "browser/26.20");
    assert_eq!(body["sessions"][0]["userAgent"], "SecurityTest/1");
    let (status, history) = get(&fresh, s.url("/api/account/security/history")).await;
    assert_eq!(status, 200, "{history}");
    assert_eq!(history["events"].as_array().unwrap().len(), 2);
    assert!(!history.to_string().contains(mw_mock_jmap::PASS));
}
#[tokio::test]
async fn remembered_device_skips_login_factor_but_factor_change_revokes_it() {
    let s = spawn().await;
    let store = s.store().await;
    let c = client();
    login(&s, &c).await;
    security_setting(&store, "twofa.rememberDays", json!(7)).await;
    let secret = totp::generate_secret();
    store
        .put_totp_secret(mw_mock_jmap::ACCOUNT_ID, &secret, true)
        .await
        .unwrap();
    let code = totp::totp_at(&secret, now_unix(), &TotpParams::default());
    assert_eq!(
        post(
            &c,
            s.url("/api/security/step-up"),
            json!({"password":mw_mock_jmap::PASS,"code":code})
        )
        .await
        .0,
        200
    );
    let (status, body) = post(
        &c,
        s.url("/api/account/security/remembered-devices"),
        json!({"label":"Laptop"}),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert!(!body.to_string().contains("token"));
    post(&c, s.url("/api/logout"), json!({})).await;
    let login_body = login(&s, &c).await;
    assert!(login_body.get("twofaRequired").is_none(), "{login_body}");
    assert_eq!(
        probe(&s, &c).await.0,
        403,
        "remembered login is not a fresh step-up"
    );
    post(&c, s.url("/api/logout"), json!({})).await;
    store
        .put_totp_secret(mw_mock_jmap::ACCOUNT_ID, &totp::generate_secret(), true)
        .await
        .unwrap();
    assert_eq!(login(&s, &c).await["twofaRequired"], true);
}

#[tokio::test]
async fn passwordless_engine_uses_sealed_credentials_and_requires_uv_once() {
    let user = "passkey@example.org";
    let password = "sealed-upstream-password";
    let pop = security_mock_mail::MockPop3::start(user, password).await;
    let dir = std::env::temp_dir().join(format!("mw-passwordless-{}", unique()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("index.html"), "<!doctype html><title>test</title>").unwrap();
    let db = dir.join("mail.db").to_string_lossy().to_string();
    let store = Store::open(&db, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap();
    store.put_domain_config(&mw_store::DomainConfigRow{name:"example.org".into(),upstream_json:json!({"pop3":{"host":"127.0.0.1","port":pop.addr.port(),"tls":"none","auth":"password","allowPrivate":true}}).to_string(),allowlist_json:"[]".into(),blocklist_json:"[]".into(),enabled:true,preferred:false,settings_json:"{}".into(),updated_at:now_unix() as i64}).await.unwrap();
    let config = AppConfig {
        db_path: db,
        server_key_hex: Some(KEY_HEX.into()),
        web_dir: Some(dir),
        cookie_secure: false,
        mode: ServerMode::Engine,
        hardening: HardeningConfig::default(),
        security: SecurityConfig::default(),
    };
    let app = build_app_full(
        config,
        V6Config {
            admin_enabled: false,
            admin_username: None,
            admin_password: None,
            redis_url: None,
        },
    )
    .await
    .unwrap()
    .0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    let origin = format!("http://{addr}");
    let first = client();
    let (status, body) = post(
        &first,
        format!("{origin}/api/login"),
        json!({"username":user,"password":password}),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    let account = body["accountId"].as_str().unwrap();
    let authn = Es256Authenticator::new();
    let credential = b64url(b"discoverable-security-test");
    store
        .add_webauthn_credential(&WebauthnCredentialRow {
            credential_id: credential.clone(),
            account_id: account.into(),
            cose_public_key: authn.cose(),
            sign_count: 1,
            transports: "internal".into(),
            label: "Virtual".into(),
            created_at: String::new(),
        })
        .await
        .unwrap();
    store
        .set_passkey_discoverable(&credential, account, true)
        .await
        .unwrap();
    let fresh = client();
    for (uv, expected, counter) in [(false, 401, 2), (true, 200, 3)] {
        let (status, ch) = post(
            &fresh,
            format!("{origin}/api/login/passkey/begin"),
            json!({}),
        )
        .await;
        assert_eq!(status, 200, "{ch}");
        let proof = authn.assert(
            &addr.ip().to_string(),
            &origin,
            &decode(ch["challenge"].as_str().unwrap()),
            counter,
            uv,
        );
        let request = json!({"pendingToken":ch["pendingToken"],"credentialId":credential,"userHandle":b64url(&crate::twofa_routes::passkey_user_handle(account)),"clientDataJson":b64url(&proof.client_data_json),"authenticatorData":b64url(&proof.authenticator_data),"signature":b64url(&proof.signature)});
        let (status, body) = post(
            &fresh,
            format!("{origin}/api/login/passkey/finish"),
            request.clone(),
        )
        .await;
        assert_eq!(status, expected, "{body}");
        if uv {
            assert_eq!(body["accountId"], account);
            assert!(!body.to_string().contains(password));
        }
        assert_eq!(
            post(
                &fresh,
                format!("{origin}/api/login/passkey/finish"),
                request
            )
            .await
            .0,
            401
        );
    }
    assert_eq!(get(&fresh, format!("{origin}/api/me")).await.0, 200);
    assert!(pop.logins_accepted() > 0);
}
#[tokio::test]
async fn passwordless_proxy_is_explicitly_unavailable() {
    let server = spawn().await;
    let client = client();
    assert_eq!(
        get(&client, server.url("/api/login/methods")).await.1["passwordless"],
        false
    );
    assert_eq!(
        post(&client, server.url("/api/login/passkey/begin"), json!({}))
            .await
            .0,
        501
    );
}

// ─── helpers ────────────────────────────────────────────────────────────────────

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn b64url(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn decode(s: &str) -> Vec<u8> {
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(s)
        .unwrap()
}

struct Assertion {
    client_data_json: Vec<u8>,
    authenticator_data: Vec<u8>,
    signature: Vec<u8>,
}

/// A virtual ES256 authenticator (as in `tests/t16_twofa.rs`).
struct Es256Authenticator {
    signing: p256::ecdsa::SigningKey,
    x: [u8; 32],
    y: [u8; 32],
}

impl Es256Authenticator {
    fn new() -> Self {
        let signing = p256::ecdsa::SigningKey::from_slice(&[0x31u8; 32]).unwrap();
        let ep = signing.verifying_key().to_sec1_point(false);
        let b = ep.as_bytes();
        let mut x = [0u8; 32];
        let mut y = [0u8; 32];
        x.copy_from_slice(&b[1..33]);
        y.copy_from_slice(&b[33..65]);
        Es256Authenticator { signing, x, y }
    }

    fn cose(&self) -> Vec<u8> {
        let mut v = vec![0xa5];
        v.extend_from_slice(&[0x01, 0x02]);
        v.extend_from_slice(&[0x03, 0x26]);
        v.extend_from_slice(&[0x20, 0x01]);
        v.extend_from_slice(&[0x21, 0x58, 0x20]);
        v.extend_from_slice(&self.x);
        v.extend_from_slice(&[0x22, 0x58, 0x20]);
        v.extend_from_slice(&self.y);
        v
    }

    /// An assertion with user presence, and user verification when `uv`.
    fn assert(
        &self,
        rp_id: &str,
        origin: &str,
        challenge: &[u8],
        counter: u32,
        uv: bool,
    ) -> Assertion {
        let mut authn_data = Vec::new();
        authn_data.extend_from_slice(&Sha256::digest(rp_id.as_bytes()));
        authn_data.push(if uv { 0x01 | 0x04 } else { 0x01 });
        authn_data.extend_from_slice(&counter.to_be_bytes());
        let cdj = format!(
            r#"{{"type":"webauthn.get","challenge":"{}","origin":"{origin}"}}"#,
            b64url(challenge)
        )
        .into_bytes();
        let mut message = authn_data.clone();
        message.extend_from_slice(&Sha256::digest(&cdj));
        let signature = {
            use p256::ecdsa::signature::Signer;
            let sig: p256::ecdsa::Signature = self.signing.sign(&message);
            sig.to_der().as_bytes().to_vec()
        };
        Assertion {
            client_data_json: cdj,
            authenticator_data: authn_data,
            signature,
        }
    }
}

#[tokio::test]
async fn admin_session_preview_does_not_revoke_and_commit_uses_reviewed_handles() {
    let server = spawn().await;
    let user = client();
    login(&server, &user).await;
    let admin = client();
    let prefix = mw_admin::config::admin_path();
    assert_eq!(
        post(
            &admin,
            server.url(&format!("{prefix}/login")),
            json!({"username":ADMIN_USER,"password":ADMIN_PASS})
        )
        .await
        .0,
        200
    );
    let (status, listed) = get(&admin, server.url(&format!("{prefix}/sessions"))).await;
    assert_eq!(status, 200, "{listed}");
    let (posture_status, posture) =
        get(&admin, server.url(&format!("{prefix}/security/posture"))).await;
    assert_eq!(posture_status, 200, "{posture}");
    assert_eq!(posture["knownAccounts"], 1);
    assert_eq!(posture["accountsWithFactor"], 0);
    assert_eq!(posture["staleApiKeys"], 0);
    assert_eq!(listed["sessions"].as_array().unwrap().len(), 1);
    let handle = listed["sessions"][0]["handle"].as_str().unwrap();
    let endpoint = server.url(&format!("{prefix}/sessions/revoke"));
    let (status, preview) =
        post(&admin, endpoint.clone(), json!({"filter":{},"dryRun":true})).await;
    assert_eq!(status, 200, "{preview}");
    assert_eq!(get(&user, server.url("/api/account/sessions")).await.0, 200);
    assert_eq!(
        post(
            &admin,
            endpoint.clone(),
            json!({"filter":{},"dryRun":false})
        )
        .await
        .0,
        400
    );
    let (status, result) = post(
        &admin,
        endpoint,
        json!({"filter":{},"dryRun":false,"handles":[handle]}),
    )
    .await;
    assert_eq!(status, 200, "{result}");
    assert_eq!(result["revoked"], 1);
    assert_eq!(get(&user, server.url("/api/account/sessions")).await.0, 401);
}

#[tokio::test]
async fn persisted_header_auth_provenance_refuses_backend_password_as_user_proof() {
    let server = spawn().await;
    let user = client();
    login(&server, &user).await;
    let store = server.store().await;
    let session = store
        .sessions_by_account(mw_mock_jmap::ACCOUNT_ID)
        .await
        .unwrap()
        .pop()
        .unwrap();
    store
        .set_session_authentication(&session.id, "header-auth")
        .await
        .unwrap();
    let (status, body) = post(
        &user,
        server.url("/api/security/step-up"),
        json!({"password":mw_mock_jmap::PASS}),
    )
    .await;
    assert_eq!(status, 403, "{body}");
}

#[tokio::test]
async fn unknown_session_provenance_requires_new_login_for_proof_and_enrollment() {
    let server = spawn().await;
    let user = client();
    login(&server, &user).await;
    let store = server.store().await;
    let session = store
        .sessions_by_account(mw_mock_jmap::ACCOUNT_ID)
        .await
        .unwrap()
        .pop()
        .unwrap();
    store
        .set_session_authentication(&session.id, "legacy")
        .await
        .unwrap();
    assert_eq!(
        post(
            &user,
            server.url("/api/security/step-up"),
            json!({"password":mw_mock_jmap::PASS})
        )
        .await
        .0,
        401
    );
    assert_eq!(
        post(&user, server.url("/api/account/2fa/totp/begin"), json!({}))
            .await
            .0,
        401
    );
    assert_eq!(probe(&server, &user).await.0, 401);
}

#[tokio::test]
async fn mfa_failure_uses_current_trusted_device_and_journals_expired_challenges() {
    let s = spawn().await;
    let original = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .user_agent("Original-password-device")
        .build()
        .unwrap();
    login(&s, &original).await;
    let store = s.store().await;
    store
        .put_totp_secret(mw_mock_jmap::ACCOUNT_ID, &totp::generate_secret(), true)
        .await
        .unwrap();
    let pending = login(&s, &original).await;
    let current = client();
    let response = current
        .post(s.url("/api/login/2fa"))
        .header("User-Agent", "Current-factor-device")
        .header("X-Forwarded-For", "198.51.100.77")
        .json(&json!({"pendingToken":pending["pendingToken"],"method":"totp","code":"invalid"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 401);
    let rows = store
        .auth_log_events(&mw_store::AuthLogQuery {
            reason: Some("factor_refused".into()),
            limit: 10,
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].event.user_agent, "Current-factor-device");
    assert_eq!(rows[0].event.ip, "127.0.0.1");
    assert_eq!(
        post(
            &current,
            s.url("/api/login/2fa"),
            json!({"pendingToken":"expired","method":"totp","code":"invalid"})
        )
        .await
        .0,
        401
    );
    let rows = store
        .auth_log_events(&mw_store::AuthLogQuery {
            reason: Some("challenge-rejected".into()),
            limit: 10,
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].event.account_id, None);
}
