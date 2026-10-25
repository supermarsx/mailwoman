//! 26.20 f24 (REQUEST f01): an administrator's hold on `mail.readReceipts` wins
//! over the policy the user stored before the hold.
//!
//! The REST route refuses a change while the setting is held, but a user who
//! had stored `always` earlier kept it, and `MDN/send {automatic: true}` reads
//! the engine's policy: receipts would still go out automatically after an
//! administrator locked them to `never`.

use mw_engine::Engine;
use mw_store::settings_policy::SettingPolicyRow;
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

async fn setup() -> (Engine, String) {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account_id = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example.org",
                port: 993,
                tls: "implicit",
                username: "Me@Example.org",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "Me@Example.org".into(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap();
    store
        .set_setting(&Engine::mdn_policy_key(&account_id), "always")
        .await
        .unwrap();
    (Engine::new(store), account_id)
}

fn policy(value: &str, locked: bool, allow_user_override: bool) -> SettingPolicyRow {
    SettingPolicyRow {
        key: "mail.readReceipts".into(),
        global_value: Some(value.into()),
        allow_user_override,
        global_locked: locked,
        updated_by: "admin".into(),
        updated_at: 0,
    }
}

/// Precondition: with no admin policy the stored `always` is the policy.
#[tokio::test]
async fn without_a_hold_the_stored_policy_applies() {
    let (engine, account) = setup().await;
    assert_eq!(engine.mdn_policy(&account).await.unwrap(), "always");
}

#[tokio::test]
async fn a_global_lock_at_never_overrides_a_stored_always() {
    let (engine, account) = setup().await;
    engine
        .store()
        .put_setting_policy(&policy("\"never\"", true, true))
        .await
        .unwrap();
    assert_eq!(engine.mdn_policy(&account).await.unwrap(), "never");
}

#[tokio::test]
async fn no_user_override_holds_the_global_value_too() {
    let (engine, account) = setup().await;
    engine
        .store()
        .put_setting_policy(&policy("\"ask\"", false, false))
        .await
        .unwrap();
    assert_eq!(engine.mdn_policy(&account).await.unwrap(), "ask");
}

/// A global default the user may change is not a hold: the stored value stays.
#[tokio::test]
async fn an_unlocked_global_default_does_not_override_the_user() {
    let (engine, account) = setup().await;
    engine
        .store()
        .put_setting_policy(&policy("\"never\"", false, true))
        .await
        .unwrap();
    assert_eq!(engine.mdn_policy(&account).await.unwrap(), "always");
}
