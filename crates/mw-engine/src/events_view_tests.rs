//! t78: the Events view end to end inside the engine. Messages are stored the
//! way sync stores them (raw body, no index document), the start-up
//! reconciliation builds the index from the store (the backfill a server
//! upgraded to t78 runs), and then `Email/query` on the Events view id and
//! `Email/get` with `mailwomanCalendar` answer from the index alone.

use serde_json::{Value, json};

use crate::Engine;
use crate::query::EVENTS_VIEW_ID;
use mw_store::{
    AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store,
};

fn calendar_mail(
    subject: &str,
    part: &str,
    method: Option<&str>,
    start: &str,
    end: &str,
) -> String {
    let method_line = method
        .map(|m| format!("METHOD:{m}\r\n"))
        .unwrap_or_default();
    format!(
        "From: Olga <olga@example.com>\r\nTo: me@example.org\r\nSubject: {subject}\r\nDate: Wed, 01 Jul 2026 09:00:00 +0000\r\nMIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary=b\r\n\r\n--b\r\nContent-Type: text/plain\r\n\r\nhello\r\n--b\r\n{part}\r\n\r\nBEGIN:VCALENDAR\r\nVERSION:2.0\r\n{method_line}BEGIN:VEVENT\r\nUID:{subject}@example.com\r\nDTSTAMP:20260701T090000Z\r\nDTSTART:{start}\r\nDTEND:{end}\r\nSUMMARY:{subject}\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n--b--\r\n"
    )
}

struct Seeded {
    engine: Engine,
    account: String,
    ids: Vec<(String, String)>,
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
                username: "u",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "u".into(),
                password: "p".into(),
            },
        )
        .await
        .unwrap();
    let mut mailboxes = Vec::new();
    for (name, role) in [("INBOX", Some("inbox")), ("Archive", None)] {
        mailboxes.push(
            store
                .upsert_mailbox(&MailboxUpsert {
                    account_id: &account,
                    name,
                    role,
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
    let messages: Vec<(&str, usize, String)> = vec![
        (
            "future-request",
            0,
            calendar_mail(
                "future-request",
                "Content-Type: text/calendar; method=REQUEST",
                Some("REQUEST"),
                "20991103T100000Z",
                "20991103T110000Z",
            ),
        ),
        (
            "octet-ics",
            1,
            calendar_mail(
                "octet-ics",
                "Content-Type: application/octet-stream; name=invite.ics\r\nContent-Disposition: attachment; filename=invite.ics",
                Some("CANCEL"),
                "20991104T100000Z",
                "20991104T110000Z",
            ),
        ),
        (
            "past-publish",
            0,
            calendar_mail(
                "past-publish",
                "Content-Type: application/ics; name=old.ics",
                Some("PUBLISH"),
                "20200101T100000Z",
                "20200101T110000Z",
            ),
        ),
        (
            "mentions-ics",
            0,
            "From: a@example.com\r\nTo: me@example.org\r\nSubject: mentions-ics\r\nDate: Wed, 01 Jul 2026 09:00:00 +0000\r\nContent-Type: text/plain\r\n\r\nThe invite.ics is attached to the other mail.\r\n".to_string(),
        ),
    ];
    let mut ids = Vec::new();
    for (uid, (name, mailbox, raw)) in messages.into_iter().enumerate() {
        // Distinct `Date:` headers: the index orders by them, newest first.
        let raw = raw.replace("09:00:00 +0000", &format!("09:00:0{uid} +0000"));
        let blob = store.put_body(&account, raw.as_bytes()).await.unwrap();
        let date = format!("2026-07-01T09:00:0{uid}Z");
        let mid = format!("{name}@x");
        let id = store
            .upsert_message(&MessageUpsert {
                account_id: &account,
                mailbox_id: &mailboxes[mailbox],
                uid: uid as u32 + 1,
                uidvalidity: 100,
                message_id: Some(&mid),
                thread_id: None,
                internaldate: Some(&date),
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
    // The backfill: an index with no documents for stored mail is rebuilt
    // from the store, calendar content included.
    assert!(engine.reconcile_search_index().await.unwrap().is_some());
    Seeded {
        engine,
        account,
        ids,
    }
}

impl Seeded {
    fn id(&self, name: &str) -> &str {
        &self.ids.iter().find(|(n, _)| n == name).unwrap().1
    }

    async fn query(&self, filter: Value) -> Value {
        self.engine
            .email_query(
                &self.account,
                &json!({ "filter": filter, "calculateTotal": true, "limit": 50 }),
            )
            .await
    }
}

fn ids(resp: &Value) -> Vec<String> {
    resp["ids"]
        .as_array()
        .unwrap_or_else(|| panic!("no ids: {resp}"))
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

#[tokio::test]
async fn the_events_view_lists_calendar_mail_across_folders_newest_first() {
    let s = seeded().await;
    let resp = s.query(json!({ "inMailbox": EVENTS_VIEW_ID })).await;
    assert_eq!(
        ids(&resp),
        vec![
            s.id("past-publish").to_string(),
            s.id("octet-ics").to_string(),
            s.id("future-request").to_string(),
        ],
        "{resp}"
    );
    assert_eq!(resp["total"], json!(3));

    // The same through the plain filter, and its complement.
    let flag = s.query(json!({ "hasCalendar": true })).await;
    assert_eq!(flag["total"], json!(3));
    let none = s.query(json!({ "hasCalendar": false })).await;
    assert_eq!(ids(&none), vec![s.id("mentions-ics").to_string()]);

    // Unread counting, as the sidebar badge asks it.
    let unread = s
        .query(json!({ "inMailbox": EVENTS_VIEW_ID, "notKeyword": "$seen" }))
        .await;
    assert_eq!(unread["total"], json!(3));
}

#[tokio::test]
async fn upcoming_only_drops_events_that_have_ended() {
    let s = seeded().await;
    let resp = s
        .query(json!({ "inMailbox": EVENTS_VIEW_ID, "calendarEndAfter": "2026-10-29T00:00:00Z" }))
        .await;
    assert_eq!(
        ids(&resp),
        vec![
            s.id("octet-ics").to_string(),
            s.id("future-request").to_string()
        ],
        "{resp}"
    );
    let bad = s
        .query(json!({ "inMailbox": EVENTS_VIEW_ID, "calendarEndAfter": "soon" }))
        .await;
    assert_eq!(bad["type"], json!("invalidArguments"), "{bad}");
}

#[tokio::test]
async fn email_get_reads_the_hint_from_the_index() {
    let s = seeded().await;
    let wanted = [
        s.id("future-request"),
        s.id("octet-ics"),
        s.id("mentions-ics"),
    ];
    let resp = s
        .engine
        .email_get(
            &s.account,
            &json!({ "ids": wanted, "properties": ["id", "subject", "mailwomanCalendar"] }),
        )
        .await;
    let list = resp["list"].as_array().unwrap();
    assert_eq!(list[0]["mailwomanCalendar"]["method"], json!("REQUEST"));
    assert_eq!(
        list[0]["mailwomanCalendar"]["start"],
        json!("2099-11-03T10:00:00")
    );
    assert_eq!(
        list[0]["mailwomanCalendar"]["title"],
        json!("future-request")
    );
    assert_eq!(list[1]["mailwomanCalendar"]["method"], json!("CANCEL"));
    assert_eq!(list[2]["mailwomanCalendar"], Value::Null);

    // Not asked for, not filled.
    let plain = s
        .engine
        .email_get(
            &s.account,
            &json!({ "ids": [wanted[0]], "properties": ["id"] }),
        )
        .await;
    assert!(plain["list"][0].get("mailwomanCalendar").is_none());
}
