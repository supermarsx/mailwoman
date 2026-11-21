use super::*;
use mw_store::{AccountKind, Credentials, DomainConfigRow, NewAccount, ServerKey};
use serde_json::json;
async fn fixture() -> (Store, String, Vec<String>) {
    let s = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let credentials = Credentials {
        username: "guard@publication.test".into(),
        password: "fixture-only-no-upstream".into(),
    };
    let account = s
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "localhost",
                port: 9,
                tls: "starttls",
                username: &credentials.username,
                sync_policy_json: "{}",
            },
            &credentials,
        )
        .await
        .unwrap();
    s.put_domain_config(&DomainConfigRow {
        name: "publication.test".into(),
        upstream_json: "{}".into(),
        allowlist_json: "[]".into(),
        blocklist_json: "[]".into(),
        enabled: true,
        preferred: false,
        settings_json: "{}".into(),
        updated_at: 1,
    })
    .await
    .unwrap();
    let cookie = s
        .create_session(
            &account,
            &credentials.username,
            "engine",
            "engine",
            &credentials,
        )
        .await
        .unwrap();
    s.set_session_authentication(&cookie, "password")
        .await
        .unwrap();
    s.bind_verified_primary(
        &cookie,
        mw_store::identity_principal::VerifiedPrimary {
            account_id: &account,
            kind: "mailbox",
            authority: "guard-fixture",
            subject: &account,
        },
        1,
    )
    .await
    .unwrap()
    .unwrap();
    (s, cookie, vec!["publication.test".into()])
}
async fn policy(store: &Store, value: bool) {
    store
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: "realtime.actions.enabled".into(),
            global_value: Some(json!(value).to_string()),
            allow_user_override: false,
            global_locked: true,
            updated_by: "guard-fixture".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
}
#[tokio::test]
async fn cold_guard_data_uses_fresh_registry_source_not_old_revision_memo() {
    let (s, cookie, domains) = fixture().await;
    policy(&s, false).await;
    assert_eq!(
        registry::effective_for(&s, "realtime.actions.enabled", &Default::default())
            .await
            .unwrap()
            .unwrap()
            .value,
        json!(false)
    );
    policy(&s, true).await;
    let prepared = PreparedGuardInputs::prepare(&s, &cookie, &domains, &[])
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        prepared.global("realtime.actions.enabled").unwrap().value,
        json!(true)
    );
    assert!(
        prepared.inbound().is_ok() && prepared.client().is_ok() && prepared.maintenance().is_ok()
    );
    assert!(prepared.logical_policy_bytes() <= MAX_PREPARED_BYTES);
    let fresh = s
        .native_publication_fence(&cookie, &domains, &[])
        .await
        .unwrap()
        .unwrap();
    assert!(prepared.matches(&s, &fresh).unwrap());
    policy(&s, false).await;
    let changed = s
        .native_publication_fence(&cookie, &domains, &[])
        .await
        .unwrap()
        .unwrap();
    assert!(!prepared.matches(&s, &changed).unwrap());
    assert_eq!(
        prepared.global("realtime.actions.enabled").unwrap().value,
        json!(true),
        "immutable DATA did not mutate or certify a new allow"
    );
    s.close().await;
}
#[tokio::test]
async fn malformed_later_guard_is_deferred_instead_of_an_earlier_refusal_or_default() {
    let (s, cookie, domains) = fixture().await;
    s.set_setting("security.clientAccess.v1", "[malformed")
        .await
        .unwrap();
    let prepared = PreparedGuardInputs::prepare(&s, &cookie, &domains, &[])
        .await
        .unwrap()
        .unwrap();
    assert!(prepared.inbound().is_ok());
    assert!(prepared.client().is_err());
    assert!(prepared.maintenance().is_ok());
    assert_eq!(prepared.source_bans_raw(&s).unwrap(), None);
    s.close().await;
}
#[tokio::test]
async fn raw_guard_change_after_policy_preparation_rejects_before_any_publication() {
    let (s, cookie, domains) = fixture().await;
    let pause = Arc::new(test_control::Pause {
        entered: tokio::sync::Notify::new(),
        release: tokio::sync::Notify::new(),
    });
    let future = test_control::PAUSE.scope(
        pause.clone(),
        PreparedGuardInputs::prepare(&s, &cookie, &domains, &[]),
    );
    tokio::pin!(future);
    tokio::select! {_=pause.entered.notified()=>{},_=&mut future=>panic!("cold preparation did not reach final boundary")}
    s.set_setting(
        "maintenance.mode.v1",
        r#"{"enabled":true,"message":"Stop","retryAfterSecs":60}"#,
    )
    .await
    .unwrap();
    pause.release.notify_one();
    assert!(matches!(future.await, Err(StoreError::SourceChanged)));
    s.close().await;
}
