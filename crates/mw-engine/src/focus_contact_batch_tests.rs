use super::*;
use crate::Engine;
use mw_store::{
    AccountKind, AddressBookRow, ContactRow, Credentials, NewAccount, ServerKey, Store,
};
#[tokio::test]
async fn fifty_focus_senders_use_one_contact_statement_and_keep_reason_precedence() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let a = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "focus.invalid",
                port: 993,
                tls: "implicit",
                username: "owner@example.test",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "owner@example.test".into(),
                password: "synthetic".into(),
            },
        )
        .await
        .unwrap();
    store
        .upsert_address_book(&AddressBookRow {
            id: "focus-book".into(),
            account_id: a.clone(),
            name: "Focus".into(),
            is_default: true,
            carddav_url: None,
            sync_token: None,
            ctag: None,
        })
        .await
        .unwrap();
    let mut emails = Vec::new();
    for n in 0..50 {
        let sender = format!("sender{n}@example.test");
        let id = format!("message-{n}");
        store
            .upsert_contact(&ContactRow {
                id: format!("contact-{n}"),
                address_book_id: "focus-book".into(),
                uid: format!("uid-{n}"),
                etag: None,
                vcard_raw: format!("BEGIN:VCARD\r\nEMAIL:{sender}\r\nEND:VCARD\r\n"),
                json: None,
                full_name: format!("Sender {n}"),
                is_favorite: false,
                photo_blob_id: None,
                pgp_key: None,
                smime_cert: None,
            })
            .await
            .unwrap();
        emails.push(Some(
            serde_json::json!({"id":id,"from":[{"email":sender,"name":"News notification"}]}),
        ));
    }
    emails.push(emails[0].clone()); // positional duplicates remain in the response
    let engine = Engine::with_search(
        store,
        std::sync::Arc::new(mw_search::Index::open_in_ram().unwrap()),
    );
    let mut projections = std::collections::HashMap::new();
    let doc = mw_search::IndexDoc {
        stable_id: "message-0".into(),
        account_id: a.clone(),
        list_signal: Some("list-id".into()),
        ..Default::default()
    };
    let projection: mw_search::ListProjection =
        serde_json::from_value(serde_json::to_value(doc).unwrap()).unwrap();
    projections.insert("message-0".into(), projection);
    let (result, count) = mw_store::pool_telemetry::measure_queries(engine.fill_focus(
        &a,
        &mut emails,
        &projections,
        false,
    ))
    .await;
    result.unwrap();
    assert_eq!(
        count.read.calls + count.other.calls,
        2,
        "one mailbox-context lookup plus one page-wide contact statement"
    );
    for email in emails.iter().flatten() {
        let decision = &email[FOCUS_PROPERTY];
        if email["id"] == "message-0" {
            assert_eq!(decision["reason"], "listHeader");
            assert_eq!(decision["tab"], "other");
        } else {
            assert_eq!(decision["reason"], "contact");
            assert_eq!(decision["tab"], "focused");
        }
    }
    assert_eq!(emails[0], emails[50]);
}
