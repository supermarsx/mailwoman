//! Breach register storage: compare-and-set updates, the trail, the delivery log
//! and the retention purge of closed incidents.

use mw_store::breach::{BreachDelivery, BreachRow};
use mw_store::{ServerKey, Store};
use serde_json::json;

fn key() -> ServerKey {
    ServerKey::from_bytes(&[9u8; 32]).unwrap()
}

fn row(id: &str, status: &str, detected_at: i64, updated_at: i64) -> BreachRow {
    BreachRow {
        id: id.into(),
        title: format!("Incident {id}"),
        domain: "example.test".into(),
        status: status.into(),
        severity: "risk".into(),
        source: "manual".into(),
        detected_at,
        aware_at: Some(detected_at),
        deadline_at: Some(detected_at + 72 * 3600),
        assignee: None,
        created_at: detected_at,
        updated_at,
        rev: 1,
        body_json: "{}".into(),
    }
}

#[tokio::test]
async fn updates_are_compare_and_set_on_rev() {
    let store = Store::open_in_memory(key()).await.unwrap();
    let mut a = row("a", "detected", 100, 100);
    store.breach_insert(&a).await.unwrap();
    a.status = "assessing".into();
    a.aware_at = None;
    a.deadline_at = None;
    assert!(store.breach_update(&a, 200).await.unwrap());
    // The same rev again is stale.
    a.status = "contained".into();
    assert!(!store.breach_update(&a, 300).await.unwrap());
    let got = store.breach_get("a").await.unwrap().unwrap();
    assert_eq!((got.status.as_str(), got.rev, got.updated_at), ("assessing", 2, 200));
    assert_eq!((got.aware_at, got.deadline_at), (None, None), "a draft keeps no clock");
    assert!(store.breach_get("missing").await.unwrap().is_none());
}

#[tokio::test]
async fn trail_and_delivery_log_are_per_incident_and_ordered() {
    let store = Store::open_in_memory(key()).await.unwrap();
    store.breach_insert(&row("a", "detected", 100, 100)).await.unwrap();
    store.breach_insert(&row("b", "detected", 200, 200)).await.unwrap();
    store.breach_event_add("a", "root", "created", &json!({"x": 1}), 10).await.unwrap();
    store.breach_event_add("a", "root", "assessed", &json!({}), 20).await.unwrap();
    store.breach_event_add("b", "root", "created", &json!({}), 5).await.unwrap();
    let events = store.breach_events("a").await.unwrap();
    assert_eq!(events.iter().map(|e| e.kind.as_str()).collect::<Vec<_>>(), ["created", "assessed"]);
    assert_eq!(events[0].detail["x"], 1);

    for (n, status) in ["sent", "failed"].iter().enumerate() {
        store
            .breach_delivery_add(&BreachDelivery {
                id: format!("d{n}"),
                incident_id: "a".into(),
                notice_id: "n1".into(),
                recipient: "p@example.test".into(),
                channel: "inbox".into(),
                status: (*status).into(),
                detail: String::new(),
                ts: n as i64,
            })
            .await
            .unwrap();
    }
    assert_eq!(store.breach_deliveries("a").await.unwrap().len(), 2);
    assert!(store.breach_deliveries("b").await.unwrap().is_empty());
    // Newest detection first.
    assert_eq!(store.breach_list().await.unwrap()[0].id, "b");
}

#[tokio::test]
async fn retention_removes_only_old_closed_incidents_with_their_records() {
    let store = Store::open_in_memory(key()).await.unwrap();
    store.breach_insert(&row("old-closed", "closed", 10, 10)).await.unwrap();
    store.breach_insert(&row("old-no-notice", "not-notifiable", 10, 10)).await.unwrap();
    store.breach_insert(&row("old-open", "assessing", 10, 10)).await.unwrap();
    store.breach_insert(&row("new-closed", "closed", 900, 900)).await.unwrap();
    store.breach_event_add("old-closed", "root", "created", &json!({}), 10).await.unwrap();
    store
        .breach_delivery_add(&BreachDelivery {
            id: "d".into(),
            incident_id: "old-closed".into(),
            notice_id: "n".into(),
            recipient: "p@example.test".into(),
            channel: "in-app".into(),
            status: "sent".into(),
            detail: String::new(),
            ts: 10,
        })
        .await
        .unwrap();
    assert_eq!(store.breach_purge_closed_before(500).await.unwrap(), 2);
    let left: Vec<String> = store.breach_list().await.unwrap().into_iter().map(|r| r.id).collect();
    assert!(left.contains(&"old-open".to_string()), "an open incident is never purged");
    assert!(left.contains(&"new-closed".to_string()));
    assert_eq!(left.len(), 2);
    assert!(store.breach_events("old-closed").await.unwrap().is_empty());
    assert!(store.breach_deliveries("old-closed").await.unwrap().is_empty());
}

#[tokio::test]
async fn audit_rows_after_a_cursor_come_oldest_first() {
    let store = Store::open_in_memory(key()).await.unwrap();
    for (id, ts) in [("1", "2026-01-01T00:00:01+00:00"), ("2", "2026-01-01T00:00:03+00:00"), ("3", "2026-01-01T00:00:02+00:00")] {
        store
            .append_audit(&mw_store::AuditRow {
                id: id.into(),
                ts: ts.into(),
                actor: "a".into(),
                actor_kind: "system".into(),
                action: "x".into(),
                target: None,
                detail_json: "{}".into(),
                ip: None,
            })
            .await
            .unwrap();
    }
    let rows = store.breach_audit_after("2026-01-01T00:00:01+00:00", 10).await.unwrap();
    assert_eq!(rows.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), ["3", "2"]);
}
