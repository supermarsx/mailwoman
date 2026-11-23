//! `Email/query` honours `filter.text`.
//!
//! The web reader asks `Email/query {text: "supersedes:<Message-ID>"}` for every
//! opened message and shows "Superseded by a newer message" when the total is
//! above zero. A mock that ignored the text answered with the whole Inbox, so
//! every message looked superseded. These tests pin the two cases: the
//! `supersedes:` keyword matches only a message whose `Supersedes` header names
//! the id, and any other text is a case-insensitive substring search. A query
//! without `text` lists the mailbox exactly as before.

use std::net::SocketAddr;

use base64::Engine as _;
use mw_mock_jmap::{PASS, USER, router};
use serde_json::{Value, json};

async fn spawn() -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router()).await.unwrap();
    });
    addr
}

async fn call(addr: SocketAddr, calls: Value) -> Value {
    let auth = format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD.encode(format!("{USER}:{PASS}"))
    );
    reqwest::Client::builder()
        .no_proxy()
        .build()
        .unwrap()
        .post(format!("http://{addr}/jmap"))
        .header("authorization", auth)
        .json(&json!({ "using": ["urn:ietf:params:jmap:mail"], "methodCalls": calls }))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap()
}

async fn query(addr: SocketAddr, filter: Value) -> Vec<String> {
    let r = call(addr, json!([["Email/query", { "filter": filter }, "q"]])).await;
    let resp = &r["methodResponses"][0][1];
    let ids: Vec<String> = resp["ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert_eq!(resp["total"], json!(ids.len()));
    ids
}

/// Seed one replacement message (in the Inbox) that supersedes `old@example.org`.
async fn seed_replacement(addr: SocketAddr) -> String {
    let r = call(
        addr,
        json!([["Email/set", { "create": { "n": {
            "mailboxIds": { "mb-inbox": true },
            "from": [{ "name": "Dana Okafor", "email": "dana@example.org" }],
            "to": [{ "name": "Test User", "email": USER }],
            "subject": "Corrected agenda",
            "preview": "The room changed to 4B.",
            "header:Supersedes:asText": "<old@example.org>"
        } } }, "s"]]),
    )
    .await;
    r["methodResponses"][0][1]["created"]["n"]["id"]
        .as_str()
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn supersedes_matches_only_the_replacing_message() {
    let addr = spawn().await;
    let id = seed_replacement(addr).await;

    assert_eq!(
        query(addr, json!({ "text": "supersedes:old@example.org" })).await,
        vec![id.clone()]
    );
    // Brackets and case do not matter, as in the engine's index.
    assert_eq!(
        query(addr, json!({ "text": "supersedes:<OLD@example.org>" })).await,
        vec![id]
    );
    // A message nobody replaces: nothing, not the whole Inbox.
    assert!(
        query(addr, json!({ "text": "supersedes:e1@mock.example" }))
            .await
            .is_empty()
    );
}

#[tokio::test]
async fn other_text_is_a_case_insensitive_substring_search() {
    let addr = spawn().await;
    let id = seed_replacement(addr).await;

    // Subject of a seeded message.
    assert_eq!(query(addr, json!({ "text": "LUNCH" })).await, vec!["e3"]);
    // Sender name, preview, and a field operator.
    assert_eq!(
        query(addr, json!({ "text": "dana" })).await,
        vec![id.clone()]
    );
    assert_eq!(
        query(addr, json!({ "text": "room changed" })).await,
        vec![id.clone()]
    );
    assert_eq!(
        query(addr, json!({ "text": "from:carlos" })).await,
        vec!["e3"]
    );
    // Body text.
    assert_eq!(
        query(addr, json!({ "text": "welcome aboard" })).await,
        vec!["e1"]
    );
    // No match.
    assert!(
        query(addr, json!({ "text": "nonexistent-word" }))
            .await
            .is_empty()
    );
    // Text combined with a mailbox stays inside the mailbox.
    assert!(
        query(addr, json!({ "inMailbox": "mb-sent", "text": "lunch" }))
            .await
            .is_empty()
    );
}

#[tokio::test]
async fn without_text_the_listing_is_unchanged() {
    let addr = spawn().await;
    let id = seed_replacement(addr).await;
    assert_eq!(
        query(addr, json!({ "inMailbox": "mb-inbox" })).await,
        vec![id.clone(), "e1".into(), "e2".into(), "e3".into()]
    );
    // No filter at all still means the Inbox; an empty text is no filter.
    assert_eq!(query(addr, json!({})).await.len(), 4);
    assert_eq!(query(addr, json!({ "text": "  " })).await.len(), 4);
}
