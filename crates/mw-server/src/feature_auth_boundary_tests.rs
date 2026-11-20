//! Real production-router requests with an independent writer paused exactly
//! after feature resolution, before the final authentication chain.
use super::*;
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request as HttpRequest, StatusCode},
};
use mw_store::{
    AccountKind, Credentials, NewAccount, ServerKey, Store, settings_policy::SettingPolicyRow,
};
use serde_json::{Value, json};
use std::{future::Future, time::Duration};
use tower::ServiceExt;

struct Fixture {
    router: Router,
    writer: Store,
    db: std::path::PathBuf,
    db_key: Option<mw_store::encryption::DbKey>,
    session: String,
    account: String,
}
impl Fixture {
    async fn new() -> Self {
        let directory =
            std::env::temp_dir().join(format!("mw-final-feature-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();
        let db = directory.join("mail.db");
        // Determine the configured key BEFORE opening the fresh owned database;
        // never probe an active SQLite file with a separate filesystem handle.
        let encryption = mw_store::encryption::global().unwrap();
        let db_key = encryption.key_for_open(&db).unwrap().cloned();
        let key = ServerKey::generate();
        let writer = Store::open(db.to_str().unwrap(), key.clone())
            .await
            .unwrap();
        let credentials = Credentials {
            username: "user@auth-boundary.test".into(),
            password: "fixture-only-not-a-real-password".into(),
        };
        let account = writer
            .create_account(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host: "127.0.0.1",
                    port: 9,
                    tls: "starttls",
                    username: &credentials.username,
                    sync_policy_json: "{}",
                },
                &credentials,
            )
            .await
            .unwrap();
        let session = writer
            .create_session(
                &account,
                &credentials.username,
                "engine",
                "engine",
                &credentials,
            )
            .await
            .unwrap();
        writer
            .set_session_authentication(&session, "password")
            .await
            .unwrap();
        writer
            .bind_verified_primary(
                &session,
                mw_store::identity_principal::VerifiedPrimary {
                    account_id: &account,
                    kind: "mailbox",
                    authority: "auth-boundary-fixture",
                    subject: &account,
                },
                chrono::Utc::now().timestamp(),
            )
            .await
            .unwrap()
            .unwrap();
        policy(&writer, "access.protocols", json!(["imap"])).await;
        let (router, _push) = crate::build_app_full(
            crate::AppConfig {
                db_path: db.to_string_lossy().into_owned(),
                server_key_hex: Some(key.to_hex()),
                web_dir: None,
                cookie_secure: false,
                mode: crate::ServerMode::Engine,
                hardening: Default::default(),
                security: Default::default(),
            },
            crate::V6Config {
                admin_enabled: false,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        let fixture = Self {
            router,
            writer,
            db,
            db_key,
            session,
            account,
        };
        let response = send(fixture.router.clone(), fixture.request(false)).await;
        assert_eq!(
            response.0,
            StatusCode::OK,
            "fixture startup must preserve real primary auth"
        );
        assert_eq!(response.1["methodResponses"][0][0], "Core/echo");
        fixture
    }
    fn request(&self, legacy: bool) -> HttpRequest<Body> {
        let csrf = self.writer.session_csrf_token(&self.session);
        let mut request=HttpRequest::builder().method("POST").uri("/jmap/api").header("host","localhost").header("origin","http://localhost").header("content-type","application/json")
            .header("cookie",format!("mw_session={}; mw_csrf={csrf}",self.session)).header("x-csrf-token",csrf)
            .body(Body::from(serde_json::to_vec(&json!({"using":["urn:ietf:params:jmap:core"],"methodCalls":[["Core/echo",{"proof":"fresh"},"echo"]]})).unwrap())).unwrap();
        if legacy {
            request.extensions_mut().insert(ForceLegacy);
        }
        request
    }
    async fn sql(&self, sql: &str) {
        let options = mw_store::encryption::connect_options(&self.db, self.db_key.as_ref(), false);
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();
        sqlx::query(sql).execute(&pool).await.unwrap();
        pool.close().await;
    }
}
async fn policy(store: &Store, key: &str, value: Value) {
    store
        .put_setting_policy(&SettingPolicyRow {
            key: key.into(),
            global_value: Some(value.to_string()),
            allow_user_override: false,
            global_locked: true,
            updated_by: "auth-boundary-fixture".into(),
            updated_at: chrono::Utc::now().timestamp(),
        })
        .await
        .unwrap();
}
async fn send(
    router: Router,
    request: HttpRequest<Body>,
) -> (
    StatusCode,
    Value,
    u64,
    mw_store::pool_telemetry::QuerySnapshot,
) {
    let ((response, auth_calls), sql) =
        mw_store::pool_telemetry::measure_queries(count_auth(router.oneshot(request))).await;
    let response = response.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        auth_calls,
        sql,
    )
}
async fn paused<'a, F, Fut>(
    fixture: &'a Fixture,
    mutate: F,
) -> (
    StatusCode,
    Value,
    u64,
    mw_store::pool_telemetry::QuerySnapshot,
)
where
    F: FnOnce(&'a Fixture) -> Fut,
    Fut: Future<Output = ()>,
{
    paused_at(fixture, PauseAt::AfterFeatures, mutate).await
}
async fn paused_at<'a, F, Fut>(
    fixture: &'a Fixture,
    stage: PauseAt,
    mutate: F,
) -> (
    StatusCode,
    Value,
    u64,
    mw_store::pool_telemetry::QuerySnapshot,
)
where
    F: FnOnce(&'a Fixture) -> Fut,
    Fut: Future<Output = ()>,
{
    let pause = pause_at(stage, &fixture.session);
    let router = fixture.router.clone();
    let request = fixture.request(false);
    let request = tokio::spawn(send(router, request));
    tokio::time::timeout(Duration::from_secs(10), pause.entered.notified())
        .await
        .expect("request reached actual post-feature boundary");
    mutate(fixture).await;
    pause.release.notify_one();
    tokio::time::timeout(Duration::from_secs(10), request)
        .await
        .unwrap()
        .unwrap()
}
#[tokio::test]
async fn actual_cookie_jmap_uses_one_complete_auth_chain_and_fewer_sql_reads() {
    let f = Fixture::new().await;
    // Warm both normal settings paths before comparing the exact same request.
    assert_eq!(
        send(f.router.clone(), f.request(true)).await.0,
        StatusCode::OK
    );
    assert_eq!(
        send(f.router.clone(), f.request(false)).await.0,
        StatusCode::OK
    );
    let legacy = send(f.router.clone(), f.request(true)).await;
    let current = send(f.router.clone(), f.request(false)).await;
    assert_eq!(legacy.0, StatusCode::OK);
    assert_eq!(current.0, StatusCode::OK);
    assert_eq!(legacy.1, current.1);
    eprintln!(
        "cookie JMAP boundary: legacy auth={} reads={} writes={}; candidate auth={} reads={} writes={}",
        legacy.2,
        legacy.3.read.calls,
        legacy.3.write.calls,
        current.2,
        current.3.read.calls,
        current.3.write.calls
    );
    assert_eq!(legacy.2, 2);
    assert_eq!(current.2, 1);
    assert!(
        current.3.read.calls < legacy.3.read.calls,
        "actual SQL reads must decrease"
    );
}
#[tokio::test]
async fn revocation_after_feature_lookup_still_prevents_dispatch() {
    let f = Fixture::new().await;
    let r = paused(&f, |f| async move {
        f.writer.delete_session(&f.session).await.unwrap();
    })
    .await;
    assert_eq!(r.0, StatusCode::UNAUTHORIZED);
    assert_ne!(r.1["methodResponses"][0][0], "Core/echo");
    assert_eq!(r.2, 1);
}
#[tokio::test]
async fn feature_denial_never_leaks_to_an_expired_cookie() {
    let f = Fixture::new().await;
    policy(&f.writer, "access.protocols", json!([])).await;
    let r = paused(&f, |f| async move {
        f.writer
            .record_session_activity(&f.session, "2000-01-01T00:00:00Z")
            .await
            .unwrap();
    })
    .await;
    assert_eq!(r.0, StatusCode::UNAUTHORIZED);
    assert_ne!(r.1["code"], "feature_disabled");
    assert_eq!(r.2, 1);
}
#[tokio::test]
async fn changed_feature_policy_rejects_prepared_allow_then_next_request_denies() {
    let f = Fixture::new().await;
    let r = paused(&f, |f| async move {
        policy(&f.writer, "access.protocols", json!([])).await;
    })
    .await;
    assert_eq!(r.0, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(r.2, 1);
    let fresh = send(f.router.clone(), f.request(false)).await;
    assert_eq!(fresh.0, StatusCode::FORBIDDEN);
    assert_eq!(fresh.1["code"], "feature_disabled");
}
#[tokio::test]
async fn final_domain_and_malformed_account_flags_are_still_fresh() {
    for malformed in [false, true] {
        let f = Fixture::new().await;
        let r = paused(&f, |f| async move {
            if malformed {
                f.writer
                    .set_setting("v6:admin:flags:user@auth-boundary.test", "{")
                    .await
                    .unwrap();
            } else {
                let mut domain = f
                    .writer
                    .get_domain_config("auth-boundary.test")
                    .await
                    .unwrap()
                    .unwrap();
                domain.enabled = false;
                f.writer.put_domain_config(&domain).await.unwrap();
            }
        })
        .await;
        assert!(!r.0.is_success());
        assert_ne!(r.1["methodResponses"][0][0], "Core/echo");
        assert_eq!(r.2, 1);
    }
}
#[tokio::test]
async fn credential_rotation_or_canonical_alias_change_cannot_reuse_feature_hint() {
    for rotate in [false, true] {
        let f = Fixture::new().await;
        let r=paused(&f,|f|async move {
            if rotate {f.writer.reseal_account_credentials(&f.account,"rotated-after-policy").await.unwrap();}
            else {let mut target=f.writer.get_domain_config("auth-boundary.test").await.unwrap().unwrap();target.name="retarget.test".into();target.preferred=false;f.writer.put_domain_config(&target).await.unwrap();f.sql("INSERT INTO domain_alias(alias,domain) VALUES('auth-boundary.test','retarget.test')").await;assert_eq!(f.writer.canonical_policy_domain("auth-boundary.test").await.unwrap(),"retarget.test");}
        }).await;
        assert!(!r.0.is_success());
        assert_eq!(r.2, 1);
    }
}
#[tokio::test]
async fn primary_policy_identity_retirement_is_fenced_with_session_row_unchanged() {
    let f = Fixture::new().await;
    let before = f.writer.get_primary_session(&f.session).await.unwrap();
    let r = paused(&f, |f| async move {
        f.sql("UPDATE identity_binding SET retired_at=1").await;
    })
    .await;
    assert!(!r.0.is_success());
    assert_eq!(r.2, 1);
    let after = f.writer.get_primary_session(&f.session).await.unwrap();
    assert_eq!(before.credentials, after.credentials);
    assert_eq!(before.username, after.username);
}
#[tokio::test]
async fn linked_selector_and_native_bearer_keep_original_independent_auth_boundaries() {
    let f = Fixture::new().await;
    for (header, value) in [
        (crate::linked_accounts::SELECTOR, "missing-link"),
        ("authorization", "Bearer nonexistent-native-token"),
    ] {
        let mut request = f.request(false);
        request
            .headers_mut()
            .insert(header, axum::http::HeaderValue::from_static(value));
        let r = send(f.router.clone(), request).await;
        let mut legacy = f.request(true);
        legacy
            .headers_mut()
            .insert(header, axum::http::HeaderValue::from_static(value));
        let baseline = send(f.router.clone(), legacy).await;
        assert!(!r.0.is_success());
        assert_eq!(r.0, baseline.0);
        assert_eq!(r.1["code"], baseline.1["code"]);
        assert_eq!(r.2, baseline.2);
    }
}
#[tokio::test]
async fn missing_policy_revision_retains_legacy_path_instead_of_caching_authority() {
    let f = Fixture::new().await;
    f.sql("DELETE FROM settings_revision WHERE id=1").await;
    let r = send(f.router.clone(), f.request(false)).await;
    assert_eq!(r.0, StatusCode::OK);
    assert_eq!(r.2, 2);
}

#[tokio::test]
async fn subject_changes_during_final_auth_are_fenced_even_when_session_fields_do_not_change() {
    for retire in [false, true] {
        let f = Fixture::new().await;
        let before = f.writer.get_primary_session(&f.session).await.unwrap();
        let r=paused_at(&f,PauseAt::AfterDomain,|f|async move {
            if retire {f.sql("UPDATE identity_binding SET retired_at=1").await;}
            else {let mut target=f.writer.get_domain_config("auth-boundary.test").await.unwrap().unwrap();target.name="retarget.test".into();target.preferred=false;f.writer.put_domain_config(&target).await.unwrap();f.sql("INSERT INTO domain_alias(alias,domain) VALUES('auth-boundary.test','retarget.test')").await;assert_eq!(f.writer.canonical_policy_domain("auth-boundary.test").await.unwrap(),"retarget.test");}
        }).await;
        assert_eq!(r.0, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(r.2, 1);
        let after = f.writer.get_primary_session(&f.session).await.unwrap();
        assert!(same_session(&before, &after));
    }
}

#[tokio::test]
async fn session_revoked_while_real_request_body_is_waiting_never_dispatches() {
    let f = Fixture::new().await;
    let mut request = f.request(false);
    let bytes = to_bytes(
        std::mem::replace(request.body_mut(), Body::empty()),
        1024 * 1024,
    )
    .await
    .unwrap();
    let (send_body, receive_body) = tokio::sync::oneshot::channel::<axum::body::Bytes>();
    let entered = std::sync::Arc::new(tokio::sync::Notify::new());
    let signalled = entered.clone();
    *request.body_mut() = Body::from_stream(futures_util::stream::once(async move {
        signalled.notify_one();
        let body = receive_body
            .await
            .map_err(|_| std::io::Error::other("fixture body canceled"))?;
        Ok::<_, std::io::Error>(body)
    }));
    let pending = tokio::spawn(send(f.router.clone(), request));
    tokio::time::timeout(Duration::from_secs(10), entered.notified())
        .await
        .unwrap();
    f.writer.delete_session(&f.session).await.unwrap();
    send_body.send(bytes).unwrap();
    let r = tokio::time::timeout(Duration::from_secs(10), pending)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(r.0, StatusCode::UNAUTHORIZED);
    assert_ne!(r.1["methodResponses"][0][0], "Core/echo");
}
#[tokio::test]
async fn late_linked_login_marker_still_runs_the_fresh_direct_login_guard() {
    let f = Fixture::new().await;
    // Both identities are real verified primary principals. Only the late link
    // marker is inserted independently; its credential is never read because
    // direct sign-in must be denied BEFORE any linked context/backend access.
    let credentials = Credentials {
        username: "owner@auth-boundary.test".into(),
        password: "fixture-owner".into(),
    };
    let account = f
        .writer
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "127.0.0.1",
                port: 9,
                tls: "starttls",
                username: &credentials.username,
                sync_policy_json: "{}",
            },
            &credentials,
        )
        .await
        .unwrap();
    let session = f
        .writer
        .create_session(
            &account,
            &credentials.username,
            "engine",
            "engine",
            &credentials,
        )
        .await
        .unwrap();
    let owner = f
        .writer
        .bind_verified_primary(
            &session,
            mw_store::identity_principal::VerifiedPrimary {
                account_id: &account,
                kind: "mailbox",
                authority: "auth-boundary-fixture",
                subject: &account,
            },
            chrono::Utc::now().timestamp(),
        )
        .await
        .unwrap()
        .unwrap();
    let child = f
        .writer
        .primary_principal(&f.session)
        .await
        .unwrap()
        .unwrap();
    policy(&f.writer, "accounts.linked.directLogin", json!(false)).await;
    let r=paused(&f,move |f|async move {
        let pool=sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect_with(mw_store::encryption::connect_options(&f.db,f.db_key.as_ref(),false)).await.unwrap();
        sqlx::query("INSERT INTO linked_account(id,owner_principal_id,child_principal_id,child_subject,account_id,display_name,sealed_session,revision,created_at) VALUES('late-link',?1,?2,'verified-child',?3,'fixture',X'00','fixture-revision',1)")
            .bind(&owner.principal_id).bind(&child.principal_id).bind(&f.account).execute(&pool).await.unwrap();pool.close().await;
    }).await;
    assert_eq!(r.0, StatusCode::UNAUTHORIZED);
    assert_eq!(r.2, 1);
    assert_ne!(r.1["methodResponses"][0][0], "Core/echo");
}
