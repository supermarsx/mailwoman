//! The CID path rejects oversized sealed source blobs before opening them.
use mw_engine::Engine;
use mw_store::{
    AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store,
};
#[tokio::test]
async fn bounded_inline_parts_enforce_source_decoded_and_account_limits() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "example.test",
                port: 993,
                tls: "implicit",
                username: "user",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "user".into(),
                password: "pw".into(),
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
    let raw = b"From: a@example.test\r\nMIME-Version: 1.0\r\nContent-Type: multipart/related; boundary=x\r\n\r\n--x\r\nContent-Type: text/html\r\n\r\n<img src=cid:logo>\r\n--x\r\nContent-Type: image/png\r\nContent-ID: <logo>\r\nContent-Transfer-Encoding: base64\r\n\r\nYWJjZA==\r\n--x--\r\n";
    let blob = store.put_body(&account, raw).await.unwrap();
    let message = store
        .upsert_message(&MessageUpsert {
            account_id: &account,
            mailbox_id: &mailbox,
            uid: 1,
            uidvalidity: 1,
            message_id: None,
            thread_id: None,
            internaldate: None,
            size: 1,
            flags_json: "[]",
            envelope: None,
            blob_ref: Some(&blob),
        })
        .await
        .unwrap();
    // Deliberately misleading indexed size cannot bypass the sealed SQL length guard.
    assert!(
        store
            .get_body_limited(&account, &blob, raw.len() - 1)
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        store
            .get_body_limited(&account, &blob, raw.len())
            .await
            .unwrap()
            .unwrap(),
        raw
    );
    assert!(
        store
            .get_body_limited("other", &blob, raw.len())
            .await
            .unwrap()
            .is_none()
    );
    let engine = Engine::new(store);
    let html = engine
        .message_html_limited(&account, &message, raw.len(), 1024)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(html.len(), 1);
    assert!(html[0].contains("<img src=cid:logo>"));
    assert!(
        engine
            .message_html_limited("other", &message, raw.len(), 1024)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        engine
            .message_html_limited(&account, &message, raw.len() - 1, 1024)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        engine
            .message_html_limited(&account, &message, raw.len(), 1)
            .await
            .is_err()
    );

    assert!(
        engine
            .inline_part_ids_limited(&account, &message, raw.len() - 1)
            .await
            .unwrap()
            .is_empty()
    );
    let ids = engine
        .inline_part_ids_limited(&account, &message, raw.len())
        .await
        .unwrap();
    assert_eq!(ids.len(), 1);
    assert_eq!(
        engine
            .fetch_blob_limited(&account, &ids[0], raw.len(), 4)
            .await
            .unwrap()
            .unwrap()
            .bytes,
        b"abcd"
    );
    assert!(
        engine
            .fetch_blob_limited(&account, &ids[0], raw.len(), 3)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        engine
            .fetch_blob_limited("other", &ids[0], raw.len(), 4)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        engine
            .fetch_blob_limited(&account, "Uunrelated-upload", raw.len(), 4)
            .await
            .unwrap()
            .is_none()
    );
}
