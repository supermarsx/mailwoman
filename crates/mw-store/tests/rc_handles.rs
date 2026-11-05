use mw_store::rc_handles::{HANDLE_LEN, is_handle};
use mw_store::{ServerKey, Store};

fn handle(n: u32) -> String {
    format!("{n:0width$}", width = HANDLE_LEN)
}

async fn store() -> Store {
    Store::open_in_memory(ServerKey::generate()).await.unwrap()
}

#[test]
fn handle_shape_excludes_legacy_tokens() {
    assert!(is_handle(&handle(1)));
    assert!(is_handle("abcDEF-_ghIJK0123456xy"));
    assert!(!is_handle("short"));
    assert!(!is_handle("abc.def"));
    assert!(!is_handle(&format!("{}.{}", "a".repeat(10), "b".repeat(11))));
}

#[tokio::test]
async fn roundtrip_expiry_and_unknown() {
    let s = store().await;
    s.rc_handles_put(&[(handle(1), 1_000, r#"{"u":"x"}"#.into())], 900, 10)
        .await
        .unwrap();
    assert_eq!(s.rc_handle_get(&handle(1), 900).await.unwrap().as_deref(), Some(r#"{"u":"x"}"#));
    assert!(s.rc_handle_get(&handle(1), 1_000).await.unwrap().is_some());
    assert!(s.rc_handle_get(&handle(1), 1_001).await.unwrap().is_none(), "expired");
    assert!(s.rc_handle_get(&handle(2), 900).await.unwrap().is_none(), "unknown");
    assert!(s.rc_handle_get("not a handle", 900).await.unwrap().is_none());
}

#[tokio::test]
async fn expired_rows_are_purged_on_the_next_write() {
    let s = store().await;
    s.rc_handles_put(&[(handle(1), 100, "{}".into()), (handle(2), 200, "{}".into())], 50, 10)
        .await
        .unwrap();
    assert_eq!(s.rc_handle_count().await.unwrap(), 2);
    s.rc_handles_put(&[(handle(3), 500, "{}".into())], 150, 10).await.unwrap();
    assert_eq!(s.rc_handle_count().await.unwrap(), 2, "handle 1 purged");
    assert!(s.rc_handle_get(&handle(2), 150).await.unwrap().is_some());
}

#[tokio::test]
async fn the_cap_evicts_the_rows_closest_to_expiry() {
    let s = store().await;
    for n in 1..=4u32 {
        s.rc_handles_put(&[(handle(n), 1_000 + i64::from(n), "{}".into())], 0, 4)
            .await
            .unwrap();
    }
    s.rc_handles_put(&[(handle(5), 2_000, "{}".into()), (handle(6), 2_001, "{}".into())], 0, 4)
        .await
        .unwrap();
    assert_eq!(s.rc_handle_count().await.unwrap(), 4);
    for gone in [1, 2] {
        assert!(s.rc_handle_get(&handle(gone), 0).await.unwrap().is_none(), "{gone}");
    }
    for kept in [3, 4, 5, 6] {
        assert!(s.rc_handle_get(&handle(kept), 0).await.unwrap().is_some(), "{kept}");
    }
    assert!(s.rc_handles_put(&(1..=5).map(|n| (handle(n), 5, "{}".into())).collect::<Vec<_>>(), 0, 4).await.is_err());
}

#[tokio::test]
async fn rewriting_a_handle_keeps_the_first_row() {
    let s = store().await;
    s.rc_handles_put(&[(handle(1), 100, r#"{"a":1}"#.into())], 0, 10).await.unwrap();
    s.rc_handles_put(&[(handle(1), 100, r#"{"a":1}"#.into())], 0, 10).await.unwrap();
    assert_eq!(s.rc_handle_count().await.unwrap(), 1);
}
