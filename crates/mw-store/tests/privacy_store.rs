//! Art.18 restrictions and the purpose consent ledger.

use mw_store::privacy::{ConsentRecord, RestrictionRow};
use mw_store::{ServerKey, Store};

fn key() -> ServerKey {
    ServerKey::from_bytes(&[9u8; 32]).unwrap()
}

fn restriction(id: &str, account: &str, scopes: &[&str], at: i64) -> RestrictionRow {
    RestrictionRow {
        id: id.into(),
        account_id: account.into(),
        address: format!("{account}@example.test"),
        domain: "example.test".into(),
        scopes: scopes.iter().map(|s| (*s).to_string()).collect(),
        mail_delivery: "keep".into(),
        reason: "dispute".into(),
        source: "admin".into(),
        request_id: None,
        created_by: "root".into(),
        created_at: at,
        expires_at: None,
        lifted_at: None,
        lifted_by: None,
        lift_reason: None,
    }
}

fn record(id: &str, account: &str, purpose: &str, decision: &str, version: i64, at: i64) -> ConsentRecord {
    ConsentRecord {
        id: id.into(),
        account_id: account.into(),
        domain: "example.test".into(),
        purpose: purpose.into(),
        decision: decision.into(),
        version,
        basis: "consent".into(),
        source: "settings".into(),
        created_at: at,
    }
}

#[tokio::test]
async fn a_restriction_blocks_only_its_scopes_until_lifted() {
    let store = Store::open_in_memory(key()).await.unwrap();
    store.restriction_apply(&restriction("r1", "a1", &["assistant", "push"], 100)).await.unwrap();
    assert!(store.restriction_blocks("a1", "assistant").await);
    assert!(store.restriction_blocks("a1", "push").await);
    assert!(!store.restriction_blocks("a1", "indexing").await, "other scopes keep running");
    assert!(!store.restriction_blocks("a2", "assistant").await, "other accounts are untouched");

    let lifted = store.restriction_lift("a1", "root", "settled", 200).await.unwrap().unwrap();
    assert_eq!(lifted.lift_reason.as_deref(), Some("settled"));
    assert!(!store.restriction_blocks("a1", "assistant").await);
    assert!(store.restriction_lift("a1", "root", "again", 300).await.unwrap().is_none());
}

#[tokio::test]
async fn all_scope_covers_every_scope_and_expiry_ends_the_restriction() {
    let store = Store::open_in_memory(key()).await.unwrap();
    store.restriction_apply(&restriction("r1", "a1", &["all"], 100)).await.unwrap();
    for scope in mw_store::privacy::RESTRICTION_SCOPES {
        assert!(store.restriction_blocks("a1", scope).await, "{scope}");
    }
    let mut short = restriction("r2", "a2", &["push"], 100);
    short.expires_at = Some(150);
    store.restriction_apply(&short).await.unwrap();
    assert!(store.restriction_active("a2", 120).await.unwrap().is_some());
    assert!(store.restriction_active("a2", 150).await.unwrap().is_none(), "ends at the expiry time");
}

#[tokio::test]
async fn a_new_restriction_replaces_the_active_one_and_history_is_kept() {
    let store = Store::open_in_memory(key()).await.unwrap();
    store.restriction_apply(&restriction("r1", "a1", &["push"], 100)).await.unwrap();
    store.restriction_apply(&restriction("r2", "a1", &["assistant"], 200)).await.unwrap();
    assert!(!store.restriction_blocks("a1", "push").await);
    assert!(store.restriction_blocks("a1", "assistant").await);
    let all = store.restriction_list(None, false, 300).await.unwrap();
    assert_eq!(all.len(), 2);
    let active = store.restriction_list(None, true, 300).await.unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, "r2");
    assert_eq!(store.restriction_get("r1").await.unwrap().unwrap().lift_reason.as_deref(), Some("replaced"));
    assert_eq!(store.restriction_delete_for_account("a1").await.unwrap(), 2);
}

#[tokio::test]
async fn the_ledger_keeps_every_receipt_and_the_newest_decides() {
    let store = Store::open_in_memory(key()).await.unwrap();
    store.consent_append(&record("c1", "a1", "assistant", "granted", 1, 100)).await.unwrap();
    store.consent_append(&record("c2", "a1", "assistant", "withdrawn", 1, 200)).await.unwrap();
    store.consent_append(&record("c3", "a1", "push", "granted", 1, 150)).await.unwrap();
    store.consent_append(&record("c4", "a2", "assistant", "granted", 1, 120)).await.unwrap();
    assert_eq!(store.consent_latest("a1", "assistant").await.unwrap().unwrap().decision, "withdrawn");
    assert_eq!(store.consent_latest("a1", "push").await.unwrap().unwrap().decision, "granted");
    assert!(store.consent_latest("a1", "telemetry").await.unwrap().is_none());
    assert_eq!(store.consent_receipts("a1").await.unwrap().len(), 3);
    assert_eq!(store.consent_between(0, 160).await.unwrap().len(), 3);

    assert_eq!(store.consent_purge(110).await.unwrap(), 1);
    assert_eq!(store.consent_delete_for_account("a1").await.unwrap(), 2);
    assert_eq!(store.consent_receipts("a2").await.unwrap().len(), 1);
}

#[tokio::test]
async fn the_footprint_counts_what_is_stored_for_one_account() {
    let store = Store::open_in_memory(key()).await.unwrap();
    store.consent_append(&record("c1", "a1", "assistant", "granted", 1, 100)).await.unwrap();
    store.consent_append(&record("c2", "a2", "assistant", "granted", 1, 100)).await.unwrap();
    let fp = store.privacy_footprint("a1").await.unwrap();
    assert_eq!(fp.items["consents"].0, 1);
    assert_eq!(fp.items["mail"], (0, 0));
    for category in ["contacts", "calendars", "notes", "sessions", "signInHistory", "push", "apiKeys", "searchEmbeddings", "tracking", "requests", "audit"] {
        assert!(fp.items.contains_key(category), "{category}");
    }
}
