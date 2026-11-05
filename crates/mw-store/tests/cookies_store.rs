//! Cookie consent storage: consent log, classification rules, detected names.

use mw_store::cookies::{ConsentRow, CookieRule, MAX_DETECTED};
use mw_store::{ServerKey, Store};

fn key() -> ServerKey {
    ServerKey::from_bytes(&[7u8; 32]).unwrap()
}

fn consent(id: &str, account: Option<&str>, at: i64, decision: &str) -> ConsentRow {
    ConsentRow {
        id: id.into(),
        account_id: account.map(str::to_owned),
        domain: "example.test".into(),
        policy_version: 1,
        decision: decision.into(),
        categories_json: "{\"functional\":true}".into(),
        region: "PT".into(),
        signal: "none".into(),
        created_at: at,
    }
}

#[tokio::test]
async fn consent_log_round_trips_purges_and_erases() {
    let store = Store::open_in_memory(key()).await.unwrap();
    store.record_cookie_consent(&consent("a", Some("acct-1"), 100, "custom")).await.unwrap();
    store.record_cookie_consent(&consent("b", Some("acct-1"), 200, "acceptAll")).await.unwrap();
    store.record_cookie_consent(&consent("c", None, 300, "rejectAll")).await.unwrap();

    let latest = store.latest_cookie_consent("acct-1").await.unwrap().unwrap();
    assert_eq!(latest.id, "b");
    assert!(store.latest_cookie_consent("nobody").await.unwrap().is_none());

    let (rows, truncated) = store.cookie_consents_between(0, 250).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert!(!truncated);

    assert_eq!(store.purge_cookie_consents(150).await.unwrap(), 1);
    assert_eq!(store.delete_cookie_consents_for_account("acct-1").await.unwrap(), 1);
    let (rows, _) = store.cookie_consents_between(0, 1000).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert!(rows[0].account_id.is_none());
}

#[tokio::test]
async fn rules_upsert_by_kind_and_pattern() {
    let store = Store::open_in_memory(key()).await.unwrap();
    let rule = |id: &str, category: &str| CookieRule {
        id: id.into(),
        kind: "cookie".into(),
        pattern: "vendor_*".into(),
        category: category.into(),
        note: String::new(),
        created_at: 1,
    };
    store.upsert_cookie_rule(&rule("r1", "analytics")).await.unwrap();
    store.upsert_cookie_rule(&rule("r2", "marketing")).await.unwrap();
    let rules = store.list_cookie_rules().await.unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].category, "marketing");
    assert!(store.delete_cookie_rule(&rules[0].id).await.unwrap());
    assert!(store.list_cookie_rules().await.unwrap().is_empty());
}

#[tokio::test]
async fn detected_names_count_are_capped_and_keep_review_status() {
    let store = Store::open_in_memory(key()).await.unwrap();
    store.record_detected_storage("d1", "localStorage", "vendor.key", 10).await.unwrap();
    store.record_detected_storage("d2", "localStorage", "vendor.key", 20).await.unwrap();
    let list = store.list_detected_storage().await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!((list[0].seen_count, list[0].first_seen, list[0].last_seen), (2, 10, 20));
    assert_eq!(list[0].status, "unclassified");

    assert!(store.set_detected_storage_status(&list[0].id, "ignored").await.unwrap());
    store.record_detected_storage("d3", "localStorage", "vendor.key", 30).await.unwrap();
    assert_eq!(store.list_detected_storage().await.unwrap()[0].status, "ignored");

    for n in 0..(MAX_DETECTED + 20) {
        store
            .record_detected_storage(&format!("x{n}"), "cookie", &format!("n{n}"), 40)
            .await
            .unwrap();
    }
    assert_eq!(store.list_detected_storage().await.unwrap().len() as i64, MAX_DETECTED);
}
