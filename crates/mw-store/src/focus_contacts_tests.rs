use super::*;
use crate::{AccountKind, AddressBookRow, ContactRow, Credentials, NewAccount};
async fn account(store: &Store, name: &str) -> String {
    store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "contacts.invalid",
                port: 993,
                tls: "implicit",
                username: name,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: name.into(),
                password: "synthetic".into(),
            },
        )
        .await
        .unwrap()
}
async fn book(store: &Store, account: &str, id: &str) {
    store
        .upsert_address_book(&AddressBookRow {
            id: id.into(),
            account_id: account.into(),
            name: id.into(),
            is_default: true,
            carddav_url: None,
            sync_token: None,
            ctag: None,
        })
        .await
        .unwrap();
}
async fn card(store: &Store, book: &str, id: &str, name: &str, email: &str, favorite: bool) {
    store
        .upsert_contact(&ContactRow {
            id: id.into(),
            address_book_id: book.into(),
            uid: id.into(),
            etag: None,
            vcard_raw: format!("BEGIN:VCARD\r\nFN:{name}\r\nEMAIL:{email}\r\nEND:VCARD\r\n"),
            json: None,
            full_name: name.into(),
            is_favorite: favorite,
            photo_blob_id: None,
            pgp_key: None,
            smime_cert: None,
        })
        .await
        .unwrap();
}
async fn compare(store: &Store, account: &str, senders: &[String]) {
    let FocusContactLookup::Batched { cards, by_sender } = store
        .focus_contact_candidates(account, senders)
        .await
        .unwrap()
    else {
        panic!("small valid fixture should batch")
    };
    for sender in senders {
        let old = store
            .autocomplete_contacts(account, sender, 20)
            .await
            .unwrap();
        let actual = by_sender[sender]
            .iter()
            .map(|index| (&cards[*index].id, &cards[*index].vcard_raw))
            .collect::<Vec<_>>();
        assert_eq!(
            actual,
            old.iter()
                .map(|row| (&row.id, &row.vcard_raw))
                .collect::<Vec<_>>(),
            "per-sender ranking changed"
        );
    }
}
async fn contract(store: Store) {
    let a = account(&store, "a@example.test").await;
    let b = account(&store, "b@example.test").await;
    book(&store, &a, "contacts-a").await;
    book(&store, &b, "contacts-b").await;
    let senders = (0..50)
        .map(|n| format!("sender{n}@example.test"))
        .collect::<Vec<_>>();
    for (n, sender) in senders.iter().enumerate() {
        card(
            &store,
            "contacts-a",
            &format!("card-{n:03}"),
            &format!("Person {n:03}"),
            sender,
            n % 3 == 0,
        )
        .await;
    }
    card(
        &store,
        "contacts-b",
        "foreign-only",
        "Foreign",
        "foreign@example.test",
        true,
    )
    .await;
    let (result, count) =
        crate::pool_telemetry::measure_queries(store.focus_contact_candidates(&a, &senders)).await;
    let FocusContactLookup::Batched { cards, by_sender } = result.unwrap() else {
        panic!("50 inputs should batch")
    };
    assert_eq!(count.read.calls + count.other.calls, 1);
    assert_eq!(cards.len(), 50);
    assert_eq!(by_sender.len(), 50);
    compare(&store, &a, &senders).await;
    for n in 0..25 {
        card(
            &store,
            "contacts-a",
            &format!("cutoff-{n:03}"),
            "needle@example.test",
            &format!("different{n}@example.test"),
            true,
        )
        .await;
    }
    card(
        &store,
        "contacts-a",
        "cutoff-z-exact",
        "needle@example.test",
        "needle@example.test",
        false,
    )
    .await;
    for (id, address) in [
        ("unicode", "álice@bücher.test"),
        ("percent", "a%b@example.test"),
        ("underscore", "a_b@example.test"),
        ("backslash", "a\\b@example.test"),
    ] {
        card(&store, "contacts-a", id, id, address, false).await;
    }
    compare(
        &store,
        &a,
        &[
            "needle@example.test".into(),
            "álice@bücher.test".into(),
            "a%b@example.test".into(),
            "a_b@example.test".into(),
            "a\\b@example.test".into(),
            "foreign@example.test".into(),
            "needle@example.test".into(),
        ],
    )
    .await;
    let FocusContactLookup::Batched { cards, by_sender } = store
        .focus_contact_candidates(&a, &["needle@example.test".into()])
        .await
        .unwrap()
    else {
        panic!("cutoff fixture should batch")
    };
    assert_eq!(by_sender["needle@example.test"].len(), 20);
    assert!(
        !cards.iter().any(|card| card.id == "cutoff-z-exact"),
        "old top20 false-positive cutoff must not be silently widened"
    );
    // One selected card can match multiple input patterns: transfer it once.
    card(
        &store,
        "contacts-a",
        "shared",
        "Shared shared@example.test",
        "shared@example.test",
        true,
    )
    .await;
    let FocusContactLookup::Batched { cards, by_sender } = store
        .focus_contact_candidates(&a, &["shared@example.test".into(), "shared".into()])
        .await
        .unwrap()
    else {
        panic!("shared card should batch")
    };
    assert_eq!(cards.len(), 1);
    assert_eq!(by_sender["shared"], by_sender["shared@example.test"]);
    // Oversized selected output becomes one constant SQL row, not a giant
    // client-side candidate allocation. The same contract runs on both drivers.
    card(
        &store,
        "contacts-a",
        "selected-huge",
        "selected-huge@example.test",
        "selected-huge@example.test",
        false,
    )
    .await;
    q("UPDATE contacts SET vcard_raw=?1 WHERE id='selected-huge'")
        .bind("selected-huge@example.test".repeat(4000))
        .execute(&store.backend)
        .await
        .unwrap();
    let input=serde_json::json!([{"sender":"selected-huge@example.test","pattern":"%selected-huge@example.test%"}]).to_string();
    let rows = q(batch_sql(store.backend.dialect()))
        .bind(&a)
        .bind(input)
        .fetch_all(&store.backend)
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get_string("tag"), "overflow");
    assert_eq!(rows[0].get_string("payload"), "{}");
    assert!(matches!(
        store
            .focus_contact_candidates(&a, &["selected-huge@example.test".into()])
            .await
            .unwrap(),
        FocusContactLookup::LegacyFallback
    ));
    q("UPDATE contacts SET vcard_raw=?1 WHERE id='foreign-only'")
        .bind("foreign@example.test".repeat(4000))
        .execute(&store.backend)
        .await
        .unwrap();
    let FocusContactLookup::Batched { cards, .. } = store
        .focus_contact_candidates(&a, &["foreign@example.test".into()])
        .await
        .unwrap()
    else {
        panic!("foreign oversized cards must not affect owner output budget")
    };
    assert!(cards.is_empty());
    store.close().await;
}
#[tokio::test]
async fn focus_contact_batch_sqlite_matches_legacy_top20_in_one_statement() {
    contract(
        Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap(),
    )
    .await;
}
#[tokio::test]
async fn focus_contact_batch_postgres_matches_legacy_top20_in_one_statement() {
    use sqlx::Connection;
    let Ok(dsn) = std::env::var("MW_TEST_PG") else {
        eprintln!("SKIP: MW_TEST_PG absent");
        return;
    };
    let schema = format!("focus_contact_batch_{}", uuid::Uuid::new_v4().simple());
    let mut admin = sqlx::PgConnection::connect(&dsn).await.unwrap();
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&mut admin)
        .await
        .unwrap();
    let mut scoped = url::Url::parse(&dsn).unwrap();
    scoped
        .query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={schema}"));
    contract(
        Store::open(scoped.as_str(), crate::ServerKey::generate())
            .await
            .unwrap(),
    )
    .await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&mut admin)
        .await
        .unwrap();
}

#[tokio::test]
async fn focus_contact_batch_input_and_sql_output_bounds_avoid_payload_fetch() {
    let store = Store::open_in_memory(crate::ServerKey::generate())
        .await
        .unwrap();
    let a = account(&store, "budget@example.test").await;
    book(&store, &a, "budget-book").await;
    let oversized = (0..MAX_FOCUS_SENDERS + 1)
        .map(|n| format!("{n}@example.test"))
        .collect::<Vec<_>>();
    let (result, count) =
        crate::pool_telemetry::measure_queries(store.focus_contact_candidates(&a, &oversized))
            .await;
    assert!(matches!(
        result.unwrap(),
        FocusContactLookup::LegacyFallback
    ));
    assert_eq!(count.read.calls + count.other.calls, 0);
    card(
        &store,
        "budget-book",
        "huge",
        "huge@example.test",
        "huge@example.test",
        false,
    )
    .await;
    q("UPDATE contacts SET vcard_raw=?1 WHERE id='huge'")
        .bind("huge@example.test".repeat(5000))
        .execute(&store.backend)
        .await
        .unwrap();
    let input = serde_json::json!([{"sender":"huge@example.test","pattern":"%huge@example.test%"}])
        .to_string();
    let rows = q(batch_sql(store.backend.dialect()))
        .bind(&a)
        .bind(input)
        .fetch_all(&store.backend)
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get_string("tag"), "overflow");
    assert_eq!(rows[0].get_string("payload"), "{}");
    assert!(matches!(
        store
            .focus_contact_candidates(&a, &["huge@example.test".into()])
            .await
            .unwrap(),
        FocusContactLookup::LegacyFallback
    ));
}
#[tokio::test]
async fn focus_contact_batch_does_not_select_unused_photo_json_bytes() {
    let store = Store::open_in_memory(crate::ServerKey::generate())
        .await
        .unwrap();
    let a = account(&store, "photo@example.test").await;
    book(&store, &a, "photo-book").await;
    card(
        &store,
        "photo-book",
        "photo",
        "photo@example.test",
        "photo@example.test",
        false,
    )
    .await;
    q("UPDATE contacts SET json=?1 WHERE id='photo'")
        .bind(vec![0u8; 4 * 1024 * 1024])
        .execute(&store.backend)
        .await
        .unwrap();
    let FocusContactLookup::Batched { cards, .. } = store
        .focus_contact_candidates(&a, &["photo@example.test".into()])
        .await
        .unwrap()
    else {
        panic!("unused opaque JSON bytes must not force a bulk transfer")
    };
    assert_eq!(cards.len(), 1);
    assert!(cards[0].vcard_raw.len() < 1024);
    for dialect in [Dialect::Sqlite, Dialect::Postgres] {
        assert!(!batch_sql(dialect).contains("'json',c.json"));
    }
    q("UPDATE contacts SET json='invalid-storage-class' WHERE id='photo'")
        .execute(&store.backend)
        .await
        .unwrap();
    assert!(matches!(
        store
            .focus_contact_candidates(&a, &["photo@example.test".into()])
            .await
            .unwrap(),
        FocusContactLookup::LegacyFallback
    ));
}
#[tokio::test]
async fn focus_contact_batch_independent_writer_updates_are_fresh() {
    let root = std::env::temp_dir().join(format!("focus-contacts-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&root).unwrap();
    let path = root.join("mail.db");
    let key = crate::ServerKey::generate();
    let reader = Store::open(path.to_str().unwrap(), key.clone())
        .await
        .unwrap();
    let writer = Store::open(path.to_str().unwrap(), key).await.unwrap();
    let a = account(&reader, "fresh@example.test").await;
    book(&reader, &a, "fresh-book").await;
    let inputs = vec!["old@example.test".into(), "new@example.test".into()];
    card(
        &writer,
        "fresh-book",
        "fresh",
        "Old",
        "old@example.test",
        false,
    )
    .await;
    let FocusContactLookup::Batched { by_sender, .. } =
        reader.focus_contact_candidates(&a, &inputs).await.unwrap()
    else {
        panic!("fresh small read must batch")
    };
    assert_eq!(by_sender["old@example.test"].len(), 1);
    card(
        &writer,
        "fresh-book",
        "fresh",
        "New",
        "new@example.test",
        false,
    )
    .await;
    let FocusContactLookup::Batched { by_sender, .. } =
        reader.focus_contact_candidates(&a, &inputs).await.unwrap()
    else {
        panic!("fresh small update must batch")
    };
    assert!(by_sender["old@example.test"].is_empty());
    assert_eq!(by_sender["new@example.test"].len(), 1);
    writer.delete_contact("fresh").await.unwrap();
    let FocusContactLookup::Batched { by_sender, .. } =
        reader.focus_contact_candidates(&a, &inputs).await.unwrap()
    else {
        panic!("fresh empty read must batch")
    };
    assert!(by_sender.values().all(Vec::is_empty));
    writer.close().await;
    reader.close().await;
    drop(writer);
    drop(reader);
    for attempt in 0..100 {
        match std::fs::remove_dir_all(&root) {
            Ok(()) => break,
            Err(error) if attempt < 99 && error.raw_os_error() == Some(32) => {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await
            }
            Err(error) => panic!("owned fixture cleanup failed: {error}"),
        }
    }
}
#[tokio::test]
async fn focus_contact_batch_malformed_utf8_does_not_become_a_contact_match() {
    let store = Store::open_in_memory(crate::ServerKey::generate())
        .await
        .unwrap();
    let a = account(&store, "malformed@example.test").await;
    book(&store, &a, "bad-book").await;
    card(
        &store,
        "bad-book",
        "bad",
        "bad@example.test",
        "bad@example.test",
        false,
    )
    .await;
    q("UPDATE contacts SET uid=CAST(X'ff' AS TEXT) WHERE id='bad'")
        .execute(&store.backend)
        .await
        .unwrap();
    assert!(
        store
            .focus_contact_candidates(&a, &["bad@example.test".into()])
            .await
            .is_err()
    );
}

#[tokio::test]
async fn focus_contact_batch_aggregate_bound_precedes_payload_fetch() {
    let store = Store::open_in_memory(crate::ServerKey::generate())
        .await
        .unwrap();
    let a = account(&store, "aggregate@example.test").await;
    book(&store, &a, "aggregate-book").await;
    let mut senders = Vec::new();
    for n in 0..128 {
        let address = format!("aggregate{n}@example.test");
        let id = format!("aggregate-{n}");
        card(&store, "aggregate-book", &id, &id, &address, false).await;
        let mut raw = format!("EMAIL:{address}\r\n");
        raw.push_str(&"x".repeat(MAX_CARD_BYTES - raw.len()));
        q("UPDATE contacts SET vcard_raw=?1 WHERE id=?2")
            .bind(raw)
            .bind(id)
            .execute(&store.backend)
            .await
            .unwrap();
        senders.push(address);
    }
    let input=serde_json::to_string(&senders.iter().map(|sender|serde_json::json!({"sender":sender,"pattern":crate::v3::contact_autocomplete_pattern(sender)})).collect::<Vec<_>>()).unwrap();
    let rows = q(batch_sql(store.backend.dialect()))
        .bind(&a)
        .bind(input)
        .fetch_all(&store.backend)
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get_string("tag"), "overflow");
    assert_eq!(rows[0].get_string("payload"), "{}");
}

#[tokio::test]
async fn focus_contact_batch_control_heavy_json_expansion_is_prebudgeted() {
    let store = Store::open_in_memory(crate::ServerKey::generate())
        .await
        .unwrap();
    let a = account(&store, "escape@example.test").await;
    book(&store, &a, "escape-book").await;
    let mut senders = Vec::new();
    for n in 0..32 {
        let sender = format!("escape{n}@example.test");
        let id = format!("escape-{n}");
        card(&store, "escape-book", &id, &id, &sender, false).await;
        let mut raw = format!("EMAIL:{sender}\r\n");
        raw.push_str(&"\u{0001}".repeat(50 * 1024));
        assert!(raw.len() < MAX_CARD_BYTES);
        q("UPDATE contacts SET vcard_raw=?1 WHERE id=?2")
            .bind(raw)
            .bind(id)
            .execute(&store.backend)
            .await
            .unwrap();
        senders.push(sender);
    }
    let input=serde_json::to_string(&senders.iter().map(|sender|serde_json::json!({"sender":sender,"pattern":crate::v3::contact_autocomplete_pattern(sender)})).collect::<Vec<_>>()).unwrap();
    let rows = q(batch_sql(store.backend.dialect()))
        .bind(&a)
        .bind(input)
        .fetch_all(&store.backend)
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get_string("tag"), "overflow");
    assert_eq!(rows[0].get_string("payload"), "{}");
    assert!(matches!(
        store.focus_contact_candidates(&a, &senders).await.unwrap(),
        FocusContactLookup::LegacyFallback
    ));
}
