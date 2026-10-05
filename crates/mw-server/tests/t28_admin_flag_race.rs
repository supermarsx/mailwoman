//! t28-c1 (t27 verification O5) — two admin toggles of one account's flags that
//! race each other both take effect.
//!
//! `Admin::toggle_zero_access`, `force_password_change` and
//! `request_remote_cache_wipe` each mean one field of the flags record. They used
//! to read the record, change their field and write the whole record back, so of
//! two that overlapped, the later write put back the other's field as it was read.
//! They now go through `AdminBackend::update_flags`, which the store adapter
//! implements as a compare-and-set on the stored record
//! (`account_gate::update_flags`).
//!
//! The admin here is the one `mailwoman admin` gets (`mw_server::build_admin`):
//! the real adapter over a real SQLite store. The three toggles of an account
//! are polled on one task, so each reads before any of them has written.
//!
//! Run:
//!   cargo test -p mw-server --test t28_admin_flag_race --locked -- --test-threads=1

mod common;
use common::test_db;

const KEY_HEX: &str = "28c1f1a92c3d4e5f60718293a4b5c6d7e28c1f1a92c3d4e5f60718293a4b5c6d";

#[tokio::test]
async fn racing_toggles_of_different_flags_both_take_effect() {
    let db = test_db::unique_db_path("mw-t28-c1-flag-race");
    let admin = mw_server::build_admin(&db.to_string_lossy(), Some(KEY_HEX))
        .await
        .expect("build_admin");

    let mut lost = Vec::new();
    for round in 0..25 {
        let account = format!("user{round}@example.org");
        let (zero, force, wipe) = tokio::join!(
            admin.toggle_zero_access("root", &account, true),
            admin.force_password_change("root", &account, true),
            admin.request_remote_cache_wipe("root", &account),
        );
        zero.unwrap();
        force.unwrap();
        wipe.unwrap();
        let flags = admin.get_feature_flags(&account).await.unwrap();
        if !(flags.zero_access && flags.force_password_change && flags.remote_cache_wipe) {
            lost.push(format!("{account}: {flags:?}"));
        }
        assert!(!flags.disabled, "no toggle sets `disabled`");
    }
    assert!(
        lost.is_empty(),
        "every toggle is in the stored record; lost in {} of 25 rounds: {lost:#?}",
        lost.len()
    );

    // A toggle keeps a field another writer set earlier, and can clear its own.
    let account = "kept@example.org";
    admin
        .force_password_change("root", account, true)
        .await
        .unwrap();
    admin
        .toggle_zero_access("root", account, true)
        .await
        .unwrap();
    admin
        .toggle_zero_access("root", account, false)
        .await
        .unwrap();
    let flags = admin.get_feature_flags(account).await.unwrap();
    assert!(
        flags.force_password_change && !flags.zero_access,
        "{flags:?}"
    );

    // Each toggle is audited once, whoever won the race.
    let audit = admin.list_audit(1000).await.unwrap();
    assert_eq!(audit.len(), 25 * 3 + 3, "one audit entry per toggle");
}
