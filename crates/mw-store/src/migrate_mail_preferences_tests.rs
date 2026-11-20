use super::*;
use crate::{AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey};

async fn copy_contract(target: &Store) {
    // Store migration preserves the deployment key as well as stable IDs.
    let source = Store::open_in_memory(target.key.clone()).await.unwrap();
    let principal = uuid::Uuid::new_v4().to_string();
    let username = format!("{principal}@migration.invalid");
    let account = source
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
    let mailbox = source
        .upsert_mailbox(&MailboxUpsert {
            account_id: &account,
            name: "INBOX",
            role: Some("inbox"),
            uidvalidity: 1,
            uidnext: 3,
            highestmodseq: 0,
            total: 2,
            unread: 0,
            parent_id: None,
        })
        .await
        .unwrap();
    let mut ids = Vec::new();
    for uid in 1..=2 {
        ids.push(
            source
                .upsert_message(&MessageUpsert {
                    account_id: &account,
                    mailbox_id: &mailbox,
                    uid,
                    uidvalidity: 1,
                    message_id: None,
                    thread_id: None,
                    internaldate: None,
                    size: 0,
                    flags_json: "[]",
                    envelope: None,
                    blob_ref: None,
                })
                .await
                .unwrap(),
        );
    }
    q("INSERT INTO identity_principal(id,state,created_at) VALUES(?1,'active',123)")
        .bind(&principal)
        .execute(&source.backend)
        .await
        .unwrap();
    let hidden = r#"["linked-fixture","secondary-fixture"]"#;
    q("INSERT INTO linked_layout(owner_principal_id,order_json,nicknames_json,revision,updated_at,avatars_json,hidden_json) VALUES(?1,'[\"main\"]','{}','saved-revision',456,'{}',?2)")
        .bind(&principal).bind(hidden).execute(&source.backend).await.unwrap();
    for (index, id) in ids.iter().enumerate() {
        q("INSERT INTO scheduled_mdn(account_id,email_id,requested_by,send_at,lease_until,retry_at) VALUES(?1,?2,'sender@fixture.invalid',1700000000,?3,?4)")
            .bind(&account).bind(id)
            .bind(if index == 0 { Some(1700000123_i64) } else { None })
            .bind(if index == 0 { Some(1700000456_i64) } else { None })
            .execute(&source.backend).await.unwrap();
    }
    q("INSERT INTO mdn_sender_schedule(account_id,address,send_at) VALUES(?1,'sender@fixture.invalid',1700000789)")
        .bind(&account).execute(&source.backend).await.unwrap();

    let full_header=serde_json::to_vec(&serde_json::json!({"subject":"copied lightweight metadata","bodyValues":{"1":{"value":"private body"}}})).unwrap();
    q("UPDATE messages SET envelope_json=?1 WHERE stable_id=?2")
        .bind(source.key.seal(&full_header).unwrap())
        .bind(&ids[0])
        .execute(&source.backend)
        .await
        .unwrap();
    assert!(
        source
            .get_account_header_envelope(&account, &ids[0])
            .await
            .unwrap()
            .unwrap()
            .projected
    );
    let mut tx = target.backend.begin().await.unwrap();
    // These are the production copier's specs, including real FK parents.
    for (name, expected) in [
        ("identity_principal", 1),
        ("accounts", 1),
        ("mailboxes", 1),
        ("messages", 2),
        ("message_list_envelope", 1),
        ("linked_layout", 1),
        ("scheduled_mdn", 2),
        ("mdn_sender_schedule", 1),
    ] {
        let spec = TABLES.iter().find(|spec| spec.name == name).unwrap();
        assert_eq!(
            copy_table(&source.backend, &mut tx, spec).await.unwrap(),
            expected,
            "{name}"
        );
    }
    tx.commit().await.unwrap();
    let copied_header = target
        .get_account_header_envelope(&account, &ids[0])
        .await
        .unwrap()
        .unwrap();
    assert!(copied_header.projected);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&copied_header.envelope).unwrap()["subject"],
        "copied lightweight metadata"
    );
    assert_eq!(
        target
            .get_account_envelopes(&account, &[ids[0].clone()])
            .await
            .unwrap()[0],
        Some(full_header)
    );

    let layout =
        q("SELECT hidden_json,revision,updated_at FROM linked_layout WHERE owner_principal_id=?1")
            .bind(&principal)
            .fetch_one(&target.backend)
            .await
            .unwrap();
    assert_eq!(layout.get_string("hidden_json"), hidden);
    assert_eq!(layout.get_string("revision"), "saved-revision");
    assert_eq!(layout.get_i64("updated_at"), 456);
    for (index, id) in ids.iter().enumerate() {
        let copied = q("SELECT requested_by,send_at,lease_until,retry_at FROM scheduled_mdn WHERE account_id=?1 AND email_id=?2")
            .bind(&account).bind(id).fetch_one(&target.backend).await.unwrap();
        assert_eq!(copied.get_string("requested_by"), "sender@fixture.invalid");
        assert_eq!(copied.get_i64("send_at"), 1700000000);
        let lease: Option<i64> = match &copied {
            Row::Sqlite(row) => sqlx::Row::get(row, "lease_until"),
            Row::Postgres(row) => sqlx::Row::get(row, "lease_until"),
        };
        let retry: Option<i64> = match &copied {
            Row::Sqlite(row) => sqlx::Row::get(row, "retry_at"),
            Row::Postgres(row) => sqlx::Row::get(row, "retry_at"),
        };
        assert_eq!(lease, if index == 0 { Some(1700000123) } else { None });
        assert_eq!(retry, if index == 0 { Some(1700000456) } else { None });
    }
    let sender = q("SELECT address,send_at FROM mdn_sender_schedule WHERE account_id=?1")
        .bind(&account)
        .fetch_one(&target.backend)
        .await
        .unwrap();
    assert_eq!(sender.get_string("address"), "sender@fixture.invalid");
    assert_eq!(sender.get_i64("send_at"), 1700000789);

    q("DELETE FROM accounts WHERE id=?1")
        .bind(&account)
        .execute(&target.backend)
        .await
        .unwrap();
    q("DELETE FROM linked_layout WHERE owner_principal_id=?1")
        .bind(&principal)
        .execute(&target.backend)
        .await
        .unwrap();
    q("DELETE FROM identity_principal WHERE id=?1")
        .bind(&principal)
        .execute(&target.backend)
        .await
        .unwrap();
    source.close().await;
}

#[tokio::test]
async fn sqlite_copy_preserves_link_visibility_and_pending_receipts() {
    let target = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    copy_contract(&target).await;
    target.close().await;
}

#[tokio::test]
async fn postgres_copy_preserves_link_visibility_and_pending_receipts() {
    let Ok(dsn) = std::env::var("MW_BG_TEST_PG") else {
        eprintln!("SKIP: MW_BG_TEST_PG absent");
        return;
    };
    let target = Store::open_postgres(&dsn, ServerKey::generate())
        .await
        .unwrap();
    copy_contract(&target).await;
    target.close().await;
}
