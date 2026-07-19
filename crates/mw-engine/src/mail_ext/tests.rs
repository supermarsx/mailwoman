//! Unit coverage for the mail-family completeness surface (t16 J1–J5): one test
//! per method plus the dispatch guard + capability advertisement. Messages are
//! ingested through the real [`Engine::ingest`] path so `Thread/get` exercises
//! genuine JWZ threads, and copy/import/parse round-trip real stored bytes.

use serde_json::{Value, json};

use mw_store::{AccountKind, Credentials, MailboxUpsert, NewAccount, QuotaRow, ServerKey, Store};

use crate::backend::{MessageRef, RawMailboxRef, RawMessage};
use crate::engine::Engine;

// ---- fixtures -----------------------------------------------------------

async fn engine() -> Engine {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    Engine::new(store)
}

/// Create a connected account and return its store id. Mailboxes carry a foreign
/// key to `accounts`, so any test that upserts a mailbox must seed one first.
async fn account(engine: &Engine) -> String {
    engine
        .store()
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example.org",
                port: 993,
                tls: "implicit",
                username: "me@example.org",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "me@example.org".into(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap()
}

async fn mailbox(engine: &Engine, account: &str, name: &str, role: Option<&str>) -> String {
    engine
        .store()
        .upsert_mailbox(&MailboxUpsert {
            account_id: account,
            name,
            role,
            uidvalidity: 1,
            uidnext: 0,
            highestmodseq: 0,
            total: 0,
            unread: 0,
            parent_id: None,
        })
        .await
        .unwrap()
}

/// A minimal RFC822 message. `mid` carries its angle brackets (`<a@x>`).
fn raw_msg(
    mid: &str,
    subject: &str,
    in_reply_to: Option<&str>,
    refs: &[&str],
    body: &str,
) -> Vec<u8> {
    let mut s = String::new();
    s.push_str(&format!("Message-ID: {mid}\r\n"));
    s.push_str("From: sender@example.com\r\n");
    s.push_str("To: rcpt@example.com\r\n");
    s.push_str(&format!("Subject: {subject}\r\n"));
    if let Some(irt) = in_reply_to {
        s.push_str(&format!("In-Reply-To: {irt}\r\n"));
    }
    if !refs.is_empty() {
        s.push_str(&format!("References: {}\r\n", refs.join(" ")));
    }
    s.push_str("Date: Mon, 01 Jan 2024 00:00:00 +0000\r\n\r\n");
    s.push_str(body);
    s.into_bytes()
}

async fn ingest(
    engine: &Engine,
    account: &str,
    mailbox_id: &str,
    raw: Vec<u8>,
    uid: u32,
    when: &str,
) -> String {
    let msg = RawMessage {
        message_ref: MessageRef::Imap {
            mailbox: RawMailboxRef {
                name: "INBOX".into(),
                uidvalidity: 1,
            },
            uidvalidity: 1,
            uid,
        },
        raw,
        flags: Vec::new(),
        internaldate: Some(when.to_string()),
    };
    engine.ingest(account, mailbox_id, &msg).await.unwrap()
}

// ---- Thread/get + Thread/changes (J1) -----------------------------------

#[tokio::test]
async fn thread_get_returns_real_jwz_thread_oldest_first() {
    let e = engine().await;
    let acct = account(&e).await;
    let acct = acct.as_str();
    let mb = mailbox(&e, acct, "Archive", Some("archive")).await;

    let root = ingest(
        &e,
        acct,
        &mb,
        raw_msg("<o@x>", "Plan", None, &[], "hi"),
        10,
        "2024-01-01T00:00:00Z",
    )
    .await;
    let reply = ingest(
        &e,
        acct,
        &mb,
        raw_msg("<r@x>", "Re: Plan", Some("<o@x>"), &["<o@x>"], "reply"),
        11,
        "2024-01-02T00:00:00Z",
    )
    .await;

    let thread_id = e
        .store()
        .get_message(&root)
        .await
        .unwrap()
        .thread_id
        .unwrap();
    assert_eq!(
        e.store()
            .get_message(&reply)
            .await
            .unwrap()
            .thread_id
            .as_deref(),
        Some(thread_id.as_str()),
        "reply converges onto the root's JWZ thread"
    );

    let resp = e.thread_get(acct, &json!({ "ids": [thread_id] })).await;
    assert_eq!(resp["list"].as_array().unwrap().len(), 1);
    let email_ids: Vec<&str> = resp["list"][0]["emailIds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(
        email_ids,
        vec![root.as_str(), reply.as_str()],
        "oldest first"
    );
    assert!(resp["notFound"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn thread_get_reports_unknown_thread_ids_as_not_found() {
    let e = engine().await;
    let resp = e.thread_get("acct-a", &json!({ "ids": ["nope"] })).await;
    assert!(resp["list"].as_array().unwrap().is_empty());
    assert_eq!(resp["notFound"][0], "nope");
}

#[tokio::test]
async fn thread_changes_maps_email_changes_to_threads() {
    let e = engine().await;
    let acct = account(&e).await;
    let acct = acct.as_str();
    let mb = mailbox(&e, acct, "Archive", Some("archive")).await;
    let sid = ingest(
        &e,
        acct,
        &mb,
        raw_msg("<o@x>", "Plan", None, &[], "hi"),
        10,
        "2024-01-01T00:00:00Z",
    )
    .await;
    let thread_id = e
        .store()
        .get_message(&sid)
        .await
        .unwrap()
        .thread_id
        .unwrap();

    let resp = e.thread_changes(acct, &json!({ "sinceState": "0" })).await;
    let updated: Vec<&str> = resp["updated"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(updated.contains(&thread_id.as_str()));
    assert_eq!(resp["hasMoreChanges"], json!(false));
}

// ---- SearchSnippet/get (J2) ---------------------------------------------

#[tokio::test]
async fn search_snippet_highlights_query_terms() {
    let e = engine().await;
    let acct = account(&e).await;
    let acct = acct.as_str();
    let mb = mailbox(&e, acct, "Archive", Some("archive")).await;
    let sid = ingest(
        &e,
        acct,
        &mb,
        raw_msg("<o@x>", "Invoice for March", None, &[], "body"),
        10,
        "2024-01-01T00:00:00Z",
    )
    .await;

    let resp = e
        .search_snippet_get(
            acct,
            &json!({ "filter": { "text": "invoice" }, "emailIds": [sid] }),
        )
        .await;
    let subject = resp["list"][0]["subject"].as_str().unwrap();
    assert!(
        subject.contains("<mark>Invoice</mark>"),
        "case-insensitive term is marked: {subject}"
    );
    assert_eq!(resp["list"][0]["emailId"], json!(sid));
}

#[tokio::test]
async fn search_snippet_escapes_html_and_flags_unknown_ids() {
    let e = engine().await;
    let acct = account(&e).await;
    let acct = acct.as_str();
    let mb = mailbox(&e, acct, "Archive", Some("archive")).await;
    let sid = ingest(
        &e,
        acct,
        &mb,
        raw_msg("<o@x>", "a <b> & c", None, &[], "body"),
        10,
        "2024-01-01T00:00:00Z",
    )
    .await;
    let resp = e
        .search_snippet_get(acct, &json!({ "emailIds": [sid, "missing"] }))
        .await;
    let subject = resp["list"][0]["subject"].as_str().unwrap();
    assert!(subject.contains("&lt;b&gt;") && subject.contains("&amp;"));
    assert_eq!(resp["notFound"][0], "missing");
}

// ---- VacationResponse/get|set (J4) --------------------------------------

#[tokio::test]
async fn vacation_response_defaults_disabled_singleton() {
    let e = engine().await;
    let resp = e.vacation_response_get("acct-a", &json!({})).await;
    assert_eq!(resp["list"][0]["id"], "singleton");
    assert_eq!(resp["list"][0]["isEnabled"], json!(false));
}

#[tokio::test]
async fn vacation_response_set_persists_and_advances_state() {
    let e = engine().await;
    let acct = "acct-a";
    let set = e
        .vacation_response_set(
            acct,
            &json!({
                "update": { "singleton": { "isEnabled": true, "subject": "Away", "textBody": "Back Monday" } }
            }),
        )
        .await;
    assert_eq!(set["updated"]["singleton"], Value::Null);
    assert_ne!(
        set["oldState"], set["newState"],
        "state advances on a write"
    );

    let get = e.vacation_response_get(acct, &json!({})).await;
    assert_eq!(get["list"][0]["isEnabled"], json!(true));
    assert_eq!(get["list"][0]["subject"], "Away");
    assert_eq!(get["state"], set["newState"]);
}

#[tokio::test]
async fn vacation_response_refuses_create_and_destroy() {
    let e = engine().await;
    let resp = e
        .vacation_response_set(
            "acct-a",
            &json!({ "create": { "c1": {} }, "destroy": ["singleton"] }),
        )
        .await;
    assert_eq!(resp["notCreated"]["c1"]["type"], "singleton");
    assert_eq!(resp["notDestroyed"]["singleton"]["type"], "singleton");
}

// ---- Quota/get (J3) ------------------------------------------------------

#[tokio::test]
async fn quota_get_reports_usage_against_configured_limit() {
    let e = engine().await;
    let acct = account(&e).await;
    let acct = acct.as_str();
    let mb = mailbox(&e, acct, "Archive", Some("archive")).await;
    ingest(
        &e,
        acct,
        &mb,
        raw_msg("<o@x>", "Plan", None, &[], "some body bytes"),
        10,
        "2024-01-01T00:00:00Z",
    )
    .await;
    e.store()
        .set_quota(
            acct,
            QuotaRow {
                bytes_limit: 1_000_000,
                msg_limit: 500,
            },
        )
        .await
        .unwrap();

    let resp = e.quota_get(acct, &json!({})).await;
    let list = resp["list"].as_array().unwrap();
    assert_eq!(list.len(), 2, "octets + count");
    let storage = list.iter().find(|q| q["resourceType"] == "octets").unwrap();
    assert_eq!(storage["hardLimit"], json!(1_000_000));
    assert!(storage["used"].as_u64().unwrap() > 0);
    let count = list.iter().find(|q| q["resourceType"] == "count").unwrap();
    assert_eq!(count["used"], json!(1));
}

#[tokio::test]
async fn quota_get_is_empty_without_a_configured_limit() {
    let e = engine().await;
    let resp = e.quota_get("acct-a", &json!({})).await;
    assert!(resp["list"].as_array().unwrap().is_empty());
}

// ---- Email/parse|import|copy (J5) ---------------------------------------

#[tokio::test]
async fn email_parse_returns_email_without_importing() {
    let e = engine().await;
    let acct = account(&e).await;
    let acct = acct.as_str();
    let mb = mailbox(&e, acct, "Archive", Some("archive")).await;
    let sid = ingest(
        &e,
        acct,
        &mb,
        raw_msg("<o@x>", "Parse me", None, &[], "body"),
        10,
        "2024-01-01T00:00:00Z",
    )
    .await;

    // The whole-message blobId is the stable id itself.
    let resp = e.email_parse(acct, &json!({ "blobIds": [sid] })).await;
    assert_eq!(resp["parsed"][&sid]["subject"], "Parse me");
    assert_eq!(resp["parsed"][&sid]["blobId"], json!(sid));
    assert!(resp["notParsable"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn email_import_ingests_blob_into_target_mailbox() {
    let e = engine().await;
    let acct = account(&e).await;
    let acct = acct.as_str();
    let src = mailbox(&e, acct, "Archive", Some("archive")).await;
    let dst = mailbox(&e, acct, "Imported", Some("archive")).await;
    let sid = ingest(
        &e,
        acct,
        &src,
        raw_msg("<o@x>", "Import me", None, &[], "body"),
        10,
        "2024-01-01T00:00:00Z",
    )
    .await;

    let resp = e
        .email_import(
            acct,
            &json!({
                "emails": { "c1": { "blobId": sid, "mailboxIds": { dst.clone(): true } } }
            }),
        )
        .await;
    let new_id = resp["created"]["c1"]["id"].as_str().unwrap();
    assert!(resp["created"]["c1"]["threadId"].is_string());
    let imported = e.store().get_message(new_id).await.unwrap();
    assert_eq!(imported.mailbox_id, dst);
    assert_eq!(imported.account_id, acct);
}

#[tokio::test]
async fn email_import_missing_mailbox_is_clean_not_created() {
    let e = engine().await;
    let acct = account(&e).await;
    let acct = acct.as_str();
    let mb = mailbox(&e, acct, "Archive", Some("archive")).await;
    let sid = ingest(
        &e,
        acct,
        &mb,
        raw_msg("<o@x>", "x", None, &[], "body"),
        10,
        "2024-01-01T00:00:00Z",
    )
    .await;
    let resp = e
        .email_import(
            acct,
            &json!({ "emails": { "c1": { "blobId": sid, "mailboxIds": { "no-such-mb": true } } } }),
        )
        .await;
    assert!(resp["created"].as_object().unwrap().is_empty());
    assert!(resp["notCreated"]["c1"]["type"].is_string());
}

#[tokio::test]
async fn email_copy_round_trips_and_can_destroy_original() {
    let e = engine().await;
    let acct = account(&e).await;
    let acct = acct.as_str();
    let src = mailbox(&e, acct, "Archive", Some("archive")).await;
    let dst = mailbox(&e, acct, "Copies", Some("archive")).await;
    let sid = ingest(
        &e,
        acct,
        &src,
        raw_msg("<o@x>", "Copy me", None, &[], "body"),
        10,
        "2024-01-01T00:00:00Z",
    )
    .await;

    let resp = e
        .email_copy(
            acct,
            &json!({
                "fromAccountId": acct,
                "onSuccessDestroyOriginal": true,
                "create": { "c1": { "id": sid, "mailboxIds": { dst.clone(): true } } }
            }),
        )
        .await;
    let new_id = resp["created"]["c1"]["id"].as_str().unwrap().to_string();
    assert_ne!(new_id, sid);
    assert_eq!(
        e.store().get_message(&new_id).await.unwrap().mailbox_id,
        dst
    );
    // Original destroyed.
    assert!(matches!(
        e.store().get_message(&sid).await,
        Err(mw_store::StoreError::NotFound)
    ));
}

#[tokio::test]
async fn email_copy_requires_from_account() {
    let e = engine().await;
    let resp = e.email_copy("acct-a", &json!({ "create": {} })).await;
    assert_eq!(resp["type"], "invalidArguments");
}

// ---- dispatch guard + capability advertisement --------------------------

#[test]
fn dispatch_guard_matches_mail_ext_but_not_core_email() {
    use crate::jmap::mail_ext::dispatch::is_mail_ext_method;
    for m in [
        "Thread/get",
        "Thread/changes",
        "SearchSnippet/get",
        "VacationResponse/get",
        "VacationResponse/set",
        "Quota/get",
        "Email/copy",
        "Email/import",
        "Email/parse",
    ] {
        assert!(is_mail_ext_method(m), "{m} routes to mail_ext");
    }
    for m in [
        "Email/get",
        "Email/set",
        "Email/query",
        "Mailbox/get",
        "Calendar/get",
    ] {
        assert!(!is_mail_ext_method(m), "{m} must NOT route to mail_ext");
    }
}

#[test]
fn session_advertises_added_capabilities() {
    let session = crate::jmap::session_json("acct-a", "user@example.com");
    let caps = &session["capabilities"];
    assert!(caps.get("urn:ietf:params:jmap:vacationresponse").is_some());
    assert!(caps.get("urn:ietf:params:jmap:quota").is_some());
    assert_eq!(
        session["primaryAccounts"]["urn:ietf:params:jmap:quota"],
        "acct-a"
    );
}
