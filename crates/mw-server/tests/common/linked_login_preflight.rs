//! Real HTTP admission probe outside measured work. Never consumes measured cookies.
use mw_store::{Store, identity_principal::VerifiedPrimary, settings_policy::SettingPolicyRow};
use serde_json::{Value, json};
const KEY: &str = "accounts.linked.directLogin";
fn session_id(principal: &Value) -> &str {
    principal["cookie"]
        .as_str()
        .unwrap()
        .strip_prefix("mw_session=")
        .unwrap()
        .split(';')
        .next()
        .unwrap()
}
async fn policy(store: &Store, allowed: bool) {
    store
        .put_setting_policy(&SettingPolicyRow {
            key: KEY.into(),
            global_value: Some(allowed.to_string()),
            allow_user_override: false,
            global_locked: true,
            updated_by: "native-load-fixture-preflight".into(),
            updated_at: chrono::Utc::now().timestamp(),
        })
        .await
        .unwrap();
}
async fn me(client: &reqwest::Client, base: &str, principal: &Value) -> reqwest::Response {
    client
        .get(format!("{base}/api/me"))
        .header("cookie", principal["cookie"].as_str().unwrap())
        .header(
            "x-forwarded-for",
            principal["clientAddress"].as_str().unwrap(),
        )
        .send()
        .await
        .unwrap()
}
pub async fn run(
    store: &Store,
    client: &reqwest::Client,
    base: &str,
    principals: &[Value],
) -> Value {
    if principals.len() < 2 {
        return json!({"verified":false,"reason":"single-principal","testedPrincipals":0});
    }
    let owner = &principals[0];
    let child = &principals[1];
    let owner_id = session_id(owner);
    let child_id = session_id(child);
    let original = store.get_setting_policy(KEY).await.unwrap();
    let child_session = store.get_primary_session(child_id).await.unwrap();
    assert!(store.linked_accounts(owner_id).await.unwrap().is_empty());
    assert!(
        !store
            .is_linked_mailbox(&child_session.account_id)
            .await
            .unwrap()
    );
    // Linking promotes its source to internal kind1. Use an extra legitimately
    // bound source, preserving the measured child's independent primary cookie.
    let source = store
        .create_session(
            &child_session.account_id,
            &child_session.username,
            "engine",
            "engine",
            &child_session.credentials,
        )
        .await
        .unwrap();
    store
        .set_session_authentication(&source, "password")
        .await
        .unwrap();
    let bound = store
        .bind_verified_primary(
            &source,
            VerifiedPrimary {
                account_id: &child_session.account_id,
                kind: "mailbox",
                authority: "isolated-native-load",
                subject: &child_session.account_id,
            },
            chrono::Utc::now().timestamp(),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        bound.principal_id,
        store
            .primary_principal(child_id)
            .await
            .unwrap()
            .unwrap()
            .principal_id
    );
    let link = store
        .attach_verified_link(
            owner_id,
            &source,
            "Fixture direct-login policy probe",
            2,
            chrono::Utc::now().timestamp(),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(store.linked_session_kind(&source).await.unwrap(), Some(1));
    policy(store, false).await;
    let denied = me(client, base, child).await;
    assert_eq!(
        denied.status(),
        401,
        "locked direct-login policy must deny the existing primary child cookie"
    );
    assert_eq!(
        me(client, base, owner).await.status(),
        200,
        "owner must retain independent admission"
    );
    policy(store, true).await;
    assert_eq!(
        me(client, base, child).await.status(),
        200,
        "the same cookie must observe policy restoration immediately"
    );
    assert!(
        store
            .unlink_account(owner_id, &link.id, &link.revision)
            .await
            .unwrap()
    );
    if let Some(original) = &original {
        store.put_setting_policy(original).await.unwrap();
    } else {
        assert!(store.delete_setting_policy(KEY).await.unwrap());
    }
    assert_eq!(
        store.get_setting_policy(KEY).await.unwrap(),
        original,
        "restore exact prior policy row or absence"
    );
    assert!(
        store.get_session(&source).await.is_err(),
        "no retained source session after unlink"
    );
    assert_eq!(store.linked_session_kind(&source).await.unwrap(), None,
        "sourceRemoved must prove the row is absent, not merely inaccessible");
    assert!(store.linked_accounts(owner_id).await.unwrap().is_empty());
    assert!(
        !store
            .is_linked_mailbox(&child_session.account_id)
            .await
            .unwrap()
    );
    for principal in [owner, child] {
        let id = session_id(principal);
        let restored = store.get_primary_session(id).await.unwrap();
        assert_eq!(store.linked_session_kind(id).await.unwrap(), Some(0));
        assert_eq!(
            restored.account_id,
            principal["accountId"].as_str().unwrap()
        );
        assert_eq!(
            me(client, base, principal).await.status(),
            200,
            "original primary cookie must remain valid after cleanup"
        );
    }
    json!({"verified":true,"testedPrincipals":2,"deniedStatus":401,"ownerStatus":200,"restoredStatus":200,"originalPolicyRestored":true,"sourceRemoved":true,"linksRemoved":true,"measuredPrimariesUnchanged":true})
}
