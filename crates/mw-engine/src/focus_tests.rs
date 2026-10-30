//! t94: `Email/get` `mailwomanFocus` end to end inside the engine. Mail is
//! stored the way sync stores it (raw body, no index document); the start-up
//! reconciliation builds the index (the backfill), and the list headers then
//! come from the index alone.

use serde_json::{Value, json};

use crate::Engine;
use mw_store::{
    AccountKind, AddressBookRow, ContactRow, Credentials, MailboxUpsert, MessageUpsert, NewAccount,
    ServerKey, Store,
};

fn mail(from: &str, to: &str, subject: &str, extra: &str) -> String {
    format!(
        "From: {from}\r\nTo: {to}\r\nSubject: {subject}\r\nDate: Wed, 01 Jul 2026 09:00:00 +0000\r\n{extra}MIME-Version: 1.0\r\nContent-Type: text/plain\r\n\r\nhello\r\n"
    )
}

struct Seeded {
    engine: Engine,
    account: String,
    ids: Vec<(String, String)>,
}

impl Seeded {
    fn id(&self, name: &str) -> &str {
        &self.ids.iter().find(|(n, _)| n == name).unwrap().1
    }

    async fn focus(&self, names: &[&str]) -> Vec<Value> {
        let ids: Vec<&str> = names.iter().map(|n| self.id(n)).collect();
        let resp = self
            .engine
            .email_get(
                &self.account,
                &json!({ "ids": ids, "properties": ["id", "from", "threadId", "mailboxIds", "mailwomanFocus"] }),
            )
            .await;
        resp["list"]
            .as_array()
            .unwrap_or_else(|| panic!("{resp}"))
            .iter()
            .map(|e| e["mailwomanFocus"].clone())
            .collect()
    }
}

async fn seeded() -> Seeded {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "h",
                port: 993,
                tls: "implicit",
                username: "me@example.org",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "me@example.org".into(),
                password: "p".into(),
            },
        )
        .await
        .unwrap();
    let mut boxes = Vec::new();
    for (name, role) in [("INBOX", "inbox"), ("Sent", "sent")] {
        boxes.push(
            store
                .upsert_mailbox(&MailboxUpsert {
                    account_id: &account,
                    name,
                    role: Some(role),
                    uidvalidity: 100,
                    uidnext: 1,
                    highestmodseq: 0,
                    total: 0,
                    unread: 0,
                    parent_id: None,
                })
                .await
                .unwrap(),
        );
    }
    store
        .upsert_address_book(&AddressBookRow {
            id: "book".into(),
            account_id: account.clone(),
            name: "Contacts".into(),
            is_default: true,
            carddav_url: None,
            sync_token: None,
            ctag: None,
        })
        .await
        .unwrap();
    store
        .upsert_contact(&ContactRow {
            id: "carol".into(),
            address_book_id: "book".into(),
            uid: "carol".into(),
            etag: None,
            vcard_raw: "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Carol\r\nEMAIL;TYPE=home:Carol@Example.net\r\nEND:VCARD\r\n".into(),
            json: None,
            full_name: "Carol".into(),
            is_favorite: false,
            photo_blob_id: None,
            pgp_key: None,
            smime_cert: None,
        })
        .await
        .unwrap();
    let me = "me@example.org";
    // (name, mailbox index, thread, raw)
    let messages: Vec<(&str, usize, Option<&str>, String)> = vec![
        (
            "news",
            0,
            None,
            mail(
                "Shop <deals@shop.example>",
                me,
                "Deals",
                "List-Unsubscribe: <mailto:u@shop.example>\r\n",
            ),
        ),
        (
            "list",
            0,
            None,
            mail(
                "Dev <dev@lists.example>",
                me,
                "Digest",
                "List-Id: Dev <dev.lists.example>\r\n",
            ),
        ),
        (
            "bulk",
            0,
            None,
            mail(
                "Promo <promo@x.example>",
                me,
                "Sale",
                "Precedence: bulk\r\n",
            ),
        ),
        (
            "auto",
            0,
            None,
            mail(
                "Robot <robot@x.example>",
                me,
                "Report",
                "Auto-Submitted: auto-generated\r\n",
            ),
        ),
        (
            "contact",
            0,
            None,
            mail("Carol <carol@example.net>", me, "Lunch", ""),
        ),
        (
            "contact-list",
            0,
            None,
            mail(
                "Carol <carol@example.net>",
                me,
                "Club news",
                "List-Id: <club.example>\r\n",
            ),
        ),
        (
            "sent-to-dave",
            1,
            None,
            mail(
                "Me <me@example.org>",
                "Dave <dave@example.net>",
                "Hi Dave",
                "",
            ),
        ),
        (
            "from-dave",
            0,
            None,
            mail("Dave <dave@example.net>", me, "Re: Hi Dave", ""),
        ),
        (
            "sent-in-thread",
            1,
            Some("T1"),
            mail("Me <me@example.org>", "group@example.net", "Plan", ""),
        ),
        (
            "thread-reply",
            0,
            Some("T1"),
            mail(
                "Erin <erin@example.net>",
                "group@example.net",
                "Re: Plan",
                "",
            ),
        ),
        (
            "noreply",
            0,
            None,
            mail("Service <no-reply@service.example>", me, "Notice", ""),
        ),
        (
            "person",
            0,
            None,
            mail("Ana <ana@example.net>", me, "Hello", ""),
        ),
    ];
    let mut ids = Vec::new();
    for (uid, (name, mailbox, thread, raw)) in messages.into_iter().enumerate() {
        let blob = store.put_body(&account, raw.as_bytes()).await.unwrap();
        let mid = format!("{name}@x");
        let id = store
            .upsert_message(&MessageUpsert {
                account_id: &account,
                mailbox_id: &boxes[mailbox],
                uid: uid as u32 + 1,
                uidvalidity: 100,
                message_id: Some(&mid),
                thread_id: thread,
                internaldate: Some("2026-07-01T09:00:00Z"),
                size: raw.len() as u64,
                flags_json: "[]",
                envelope: None,
                blob_ref: Some(&blob),
            })
            .await
            .unwrap();
        ids.push((name.to_string(), id));
    }
    let engine = Engine::new(store);
    // The backfill: the index is rebuilt from the store, list headers included.
    assert!(engine.reconcile_search_index().await.unwrap().is_some());
    Seeded {
        engine,
        account,
        ids,
    }
}

#[tokio::test]
async fn list_headers_mean_other_and_name_the_header() {
    let s = seeded().await;
    let got = s.focus(&["news", "list", "bulk", "auto"]).await;
    for (value, header) in got.iter().zip([
        "list-unsubscribe",
        "list-id",
        "precedence",
        "auto-submitted",
    ]) {
        assert_eq!(
            value,
            &json!({"tab":"other","reason":"listHeader","header":header})
        );
    }
}

#[tokio::test]
async fn contacts_correspondents_and_threads_mean_focused() {
    let s = seeded().await;
    let got = s
        .focus(&["contact", "from-dave", "thread-reply", "person"])
        .await;
    let reasons: Vec<&str> = got.iter().map(|v| v["reason"].as_str().unwrap()).collect();
    assert_eq!(reasons, ["contact", "correspondent", "thread", "default"]);
    assert!(got.iter().all(|v| v["tab"] == json!("focused")), "{got:?}");
}

#[tokio::test]
async fn headers_win_over_a_contact_and_keywords_come_last() {
    let s = seeded().await;
    let got = s.focus(&["contact-list", "noreply"]).await;
    assert_eq!(got[0]["reason"], json!("listHeader"));
    assert_eq!(got[0]["tab"], json!("other"));
    assert_eq!(got[1], json!({"tab":"other","reason":"keyword"}));
}

#[tokio::test]
async fn the_property_is_filled_only_when_named() {
    let s = seeded().await;
    let resp = s
        .engine
        .email_get(
            &s.account,
            &json!({ "ids": [s.id("news")], "properties": ["id", "from"] }),
        )
        .await;
    assert!(resp["list"][0].get("mailwomanFocus").is_none(), "{resp}");
}
