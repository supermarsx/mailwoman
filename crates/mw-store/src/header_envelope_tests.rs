use super::*;
use crate::{AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount};
use serde_json::json;
async fn fixture(store: &Store) -> (String, String, String, Vec<u8>) {
    let username = format!("{}@headers.invalid", uuid::Uuid::new_v4());
    let account = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "mail.invalid",
                port: 993,
                tls: "implicit",
                username: &username,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: username.clone(),
                password: "fixture".into(),
            },
        )
        .await
        .unwrap();
    let mailbox = store
        .upsert_mailbox(&MailboxUpsert {
            account_id: &account,
            name: "INBOX",
            role: Some("inbox"),
            uidvalidity: 1,
            uidnext: 2,
            highestmodseq: 0,
            total: 1,
            unread: 1,
            parent_id: None,
        })
        .await
        .unwrap();
    let full=serde_json::to_vec(&json!({"subject":"original","from":[{"email":"sender@fixture.invalid","name":"Sender"}],"hasAttachment":true,"attachments":[{"partId":"2","blobId":"part-2","type":"application/pdf","size":3,"name":"file.pdf","attachmentBytes":"must never persist"}],"_mailwomanAttachmentSchema":1,"_mailwomanHeaderProjection":{"version":1,"pairs":[["delivered-to","reader@fixture.invalid"]]},"bodyValues":{"1":{"value":"large secret body ".repeat(150000)}},"htmlBody":[{"partId":"1"}],"textBody":[{"partId":"1"}],"bodyStructure":{"raw":"secret"},"unknownLargeField":"unknown content ".repeat(150000)})).unwrap();
    let id = store
        .upsert_message_with_body(
            &MessageUpsert {
                account_id: &account,
                mailbox_id: &mailbox,
                uid: 1,
                uidvalidity: 1,
                message_id: Some("message@fixture.invalid"),
                thread_id: None,
                internaldate: Some("2026-11-01T00:00:00Z"),
                size: 3,
                flags_json: "[]",
                envelope: Some(&full),
                blob_ref: None,
            },
            b"raw",
        )
        .await
        .unwrap()
        .0;
    (account, mailbox, id, full)
}
async fn ciphertext(store: &Store, id: &str) -> Vec<u8> {
    q("SELECT sealed_envelope FROM message_list_envelope WHERE stable_id=?1")
        .bind(id)
        .fetch_one(&store.backend)
        .await
        .unwrap()
        .get_blob("sealed_envelope")
}
async fn contract(store: &Store) {
    let (account, _, id, full) = fixture(store).await;
    let first = store
        .get_account_header_envelope(&account, &id)
        .await
        .unwrap()
        .unwrap();
    assert!(first.projected);
    assert!(first.envelope.len() < 1024);
    let projected: Value = serde_json::from_slice(&first.envelope).unwrap();
    for key in [
        "bodyValues",
        "htmlBody",
        "textBody",
        "bodyStructure",
        "unknownLargeField",
    ] {
        assert!(projected.get(key).is_none());
    }
    assert_eq!(projected["attachments"][0]["name"], "file.pdf");
    assert!(projected["attachments"][0].get("attachmentBytes").is_none());
    assert_eq!(projected["_mailwomanAttachmentSchema"], 1);
    assert_eq!(projected["_mailwomanHeaderProjection"]["version"], 1);
    assert_eq!(
        store
            .get_account_envelopes(&account, &[id.clone()])
            .await
            .unwrap()[0],
        Some(full.clone())
    );
    assert_eq!(
        first.cache_revision,
        store.get_message(&id).await.unwrap().cache_revision
    );
    let before = ciphertext(store, &id).await;
    store
        .set_flags_batch(&[(id.clone(), "[\"Seen\"]".into())])
        .await
        .unwrap();
    assert_eq!(
        ciphertext(store, &id).await,
        before,
        "flags must not rewrite the metadata cache"
    );
    let flags = store
        .get_account_header_envelope(&account, &id)
        .await
        .unwrap()
        .unwrap();
    assert_ne!(flags.cache_revision, first.cache_revision);
    let batch = store
        .get_account_header_envelopes(&account, &[id.clone(), "missing".into(), id.clone()])
        .await
        .unwrap();
    assert_eq!(batch.len(), 3);
    assert!(batch[1].is_none());
    assert_eq!(
        batch[0].as_ref().unwrap().envelope,
        batch[2].as_ref().unwrap().envelope
    );
    assert!(
        store
            .get_account_header_envelope("foreign", &id)
            .await
            .unwrap()
            .is_none()
    );
    for corrupt in [vec![0u8; 3], vec![0u8; MAX_HEADER_ENVELOPE_BYTES + 45]] {
        q("UPDATE message_list_envelope SET sealed_envelope=?1 WHERE stable_id=?2")
            .bind(corrupt)
            .bind(&id)
            .execute(&store.backend)
            .await
            .unwrap();
        assert!(
            store
                .get_account_header_envelope(&account, &id)
                .await
                .unwrap()
                .unwrap()
                .projected,
            "invalid/capped cache must lazily repair"
        );
    }
    let mut changed: Value = serde_json::from_slice(&full).unwrap();
    changed["subject"] = json!("raw SQL update");
    let changed = serde_json::to_vec(&changed).unwrap();
    q("UPDATE messages SET envelope_json=?1 WHERE stable_id=?2")
        .bind(store.key.seal(&changed).unwrap())
        .bind(&id)
        .execute(&store.backend)
        .await
        .unwrap();
    assert_eq!(
        q("SELECT COUNT(*) FROM message_list_envelope WHERE stable_id=?1")
            .bind(&id)
            .fetch_scalar_i64(&store.backend)
            .await
            .unwrap(),
        0
    );
    let repaired = store
        .get_account_header_envelope(&account, &id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&repaired.envelope).unwrap()["subject"],
        "raw SQL update"
    );
    let source = store.get_message(&id).await.unwrap();
    assert!(
        !store
            .repair_message_envelope_revision(
                &account,
                &id,
                source.blob_ref.as_deref().unwrap(),
                &first.cache_revision,
                &full
            )
            .await
            .unwrap()
    );
    assert!(
        store
            .repair_message_envelope_revision(
                &account,
                &id,
                source.blob_ref.as_deref().unwrap(),
                &source.cache_revision,
                &full
            )
            .await
            .unwrap()
    );
    assert_eq!(
        serde_json::from_slice::<Value>(
            &store
                .get_account_header_envelope(&account, &id)
                .await
                .unwrap()
                .unwrap()
                .envelope
        )
        .unwrap()["subject"],
        "original"
    );
    let oversize =
        serde_json::to_vec(&json!({"subject":"x".repeat(MAX_HEADER_ENVELOPE_BYTES+1)})).unwrap();
    assert!(
        store
            .repair_message_envelope(
                &account,
                &id,
                source.blob_ref.as_deref().unwrap(),
                &oversize
            )
            .await
            .unwrap()
    );
    let fallback = store
        .get_account_header_envelope(&account, &id)
        .await
        .unwrap()
        .unwrap();
    assert!(!fallback.projected);
    assert_eq!(fallback.envelope, oversize);
    q("DELETE FROM accounts WHERE id=?1")
        .bind(&account)
        .execute(&store.backend)
        .await
        .unwrap();
    assert_eq!(
        q("SELECT COUNT(*) FROM message_list_envelope WHERE stable_id=?1")
            .bind(&id)
            .fetch_scalar_i64(&store.backend)
            .await
            .unwrap(),
        0
    );
}
#[tokio::test]
async fn sqlite_projection_preserves_full_source_and_lazy_fallback_contract() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    contract(&store).await;
    store.close().await;
}
#[tokio::test]
async fn postgres_projection_preserves_full_source_and_lazy_fallback_contract() {
    let Ok(dsn) = std::env::var("MW_BG_TEST_PG") else {
        eprintln!("SKIP: MW_BG_TEST_PG absent");
        return;
    };
    let store = Store::open_postgres(&dsn, ServerKey::generate())
        .await
        .unwrap();
    contract(&store).await;
    store.close().await;
}

#[tokio::test]
async fn current_light_sql_cannot_read_full_envelope_or_body() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (account, _, id, full) = fixture(&store).await;
    assert!(full.len() > 4_000_000);
    let columns = q("PRAGMA table_info(messages)")
        .fetch_all(&store.backend)
        .await
        .unwrap();
    let envelope_column = columns
        .iter()
        .find(|row| row.get_string("name") == "envelope_json")
        .unwrap()
        .get_i64("cid");
    let message_root =
        q("SELECT rootpage FROM sqlite_schema WHERE type='table' AND name='messages'")
            .fetch_scalar_i64(&store.backend)
            .await
            .unwrap();
    let body_root = q("SELECT rootpage FROM sqlite_schema WHERE type='table' AND name='bodies'")
        .fetch_scalar_i64(&store.backend)
        .await
        .unwrap();
    let plan = q(Box::leak(
        format!("EXPLAIN {LIGHT_SQLITE}").into_boxed_str(),
    ))
    .bind(serde_json::to_string(&[id.clone()]).unwrap())
    .bind(&account)
    .bind((MAX_HEADER_ENVELOPE_BYTES + 44) as i64)
    .fetch_all(&store.backend)
    .await
    .unwrap();
    let cursors = plan
        .iter()
        .filter(|row| row.get_string("opcode") == "OpenRead" && row.get_i64("p2") == message_root)
        .map(|row| row.get_i64("p1"))
        .collect::<Vec<_>>();
    assert!(!cursors.is_empty());
    assert!(!plan.iter().any(|row| row.get_string("opcode") == "Column"
        && cursors.contains(&row.get_i64("p1"))
        && row.get_i64("p2") == envelope_column));
    assert!(
        !plan
            .iter()
            .any(|row| row.get_string("opcode") == "OpenRead" && row.get_i64("p2") == body_root)
    );
    let control =
        q("EXPLAIN SELECT envelope_json FROM messages WHERE account_id=?1 AND stable_id=?2")
            .bind(&account)
            .bind(&id)
            .fetch_all(&store.backend)
            .await
            .unwrap();
    let control_cursors = control
        .iter()
        .filter(|row| row.get_string("opcode") == "OpenRead" && row.get_i64("p2") == message_root)
        .map(|row| row.get_i64("p1"))
        .collect::<Vec<_>>();
    assert!(
        control
            .iter()
            .any(|row| row.get_string("opcode") == "Column"
                && control_cursors.contains(&row.get_i64("p1"))
                && row.get_i64("p2") == envelope_column),
        "control must detect full-envelope BLOB access"
    );
    assert!(
        store
            .get_account_header_envelope(&account, &id)
            .await
            .unwrap()
            .unwrap()
            .projected
    );
    store.close().await;
}
#[tokio::test]
async fn projection_write_failure_rolls_back_the_full_envelope_update() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let (account, _, id, full) = fixture(&store).await;
    let before = ciphertext(&store, &id).await;
    let source = store.get_message(&id).await.unwrap();
    q("CREATE TRIGGER reject_header_projection BEFORE INSERT ON message_list_envelope BEGIN SELECT RAISE(ABORT,'fixture failure'); END").execute(&store.backend).await.unwrap();
    assert!(
        store
            .repair_message_envelope_revision(
                &account,
                &id,
                source.blob_ref.as_deref().unwrap(),
                &source.cache_revision,
                b"{\"subject\":\"changed\"}"
            )
            .await
            .is_err()
    );
    assert_eq!(
        store
            .get_account_envelopes(&account, &[id.clone()])
            .await
            .unwrap()[0],
        Some(full)
    );
    assert_eq!(ciphertext(&store, &id).await, before);
    q("DROP TRIGGER reject_header_projection")
        .execute(&store.backend)
        .await
        .unwrap();
    store.close().await;
}
