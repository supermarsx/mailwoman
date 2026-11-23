//! `router_for` serves another account: its credentials are the only ones
//! accepted, and the session and sending identity carry its address and name.

use std::net::SocketAddr;

use base64::Engine as _;
use mw_mock_jmap::{MockAccount, PASS, USER, router_for};
use serde_json::{Value, json};

async fn spawn(account: MockAccount) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router_for(account)).await.unwrap();
    });
    addr
}

fn basic(user: &str, pass: &str) -> String {
    format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD.encode(format!("{user}:{pass}"))
    )
}

fn client() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().build().unwrap()
}

#[tokio::test]
async fn another_account_signs_in_with_its_own_credentials_only() {
    let addr = spawn(MockAccount {
        user: "alex@example.com".into(),
        pass: "demo-pass".into(),
        name: "Alex Rivera".into(),
    })
    .await;
    let url = format!("http://{addr}/jmap/session");

    let refused = client()
        .get(&url)
        .header("authorization", basic(USER, PASS))
        .send()
        .await
        .unwrap();
    assert_eq!(refused.status(), 401, "the default account is not accepted");

    let session: Value = client()
        .get(&url)
        .header("authorization", basic("alex@example.com", "demo-pass"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(session["username"], "alex@example.com");

    let r: Value = client()
        .post(format!("http://{addr}/jmap"))
        .header("authorization", basic("alex@example.com", "demo-pass"))
        .json(&json!({ "using": [], "methodCalls": [["Identity/get", {}, "i"]] }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let identity = &r["methodResponses"][0][1]["list"][0];
    assert_eq!(identity["name"], "Alex Rivera");
    assert_eq!(identity["email"], "alex@example.com");
}
