//! t32-h5 (D-6) — outbound webhook delivery goes through the egress gate.
//!
//! A webhook URL is chosen by a user, and before this fix the dispatcher POSTed to it
//! with a plain client: a webhook to `127.0.0.1`, an RFC 1918 address, the cloud
//! metadata address or a name resolving to any of them was delivered (SSRF), and a
//! redirect from an allowed endpoint was followed wherever it pointed.
//!
//! Every refusal leg here uses the production entry points ([`deliver`] and
//! [`run_webhook_dispatcher`], public unicast only). The legs that need a receiver
//! that accepts the delivery use [`deliver_with`] / [`run_webhook_dispatcher_with`]
//! with [`loopback_or_public`], because the test cannot reach a public address: the
//! loopback receiver stands in for one. That policy widens only the address range;
//! the scheme, credential, pin and redirect handling are the production code.
//!
//! Run:
//!   cargo test -p mw-server --test t32_h5_webhook_egress -- --test-threads=1

use std::net::IpAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use axum::Router;
use axum::body::Bytes;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use mw_egress::Refusal;
use mw_engine::StateChange;
use mw_server::webhooks::{
    DeliveryError, InMemoryWebhookRegistry, SIGNATURE_HEADER, WebhookEndpoint, WebhookRegistry,
    build_payload, deliver, deliver_with, run_webhook_dispatcher, run_webhook_dispatcher_with,
    verify_signature,
};
use tokio::sync::broadcast;

const SECRET: &[u8] = b"t32-h5-webhook-secret";

/// The production policy plus loopback: the stand-in for a public receiver.
fn loopback_or_public(ip: &IpAddr) -> bool {
    ip.is_loopback() || mw_egress::ip_allowed(ip)
}

fn change() -> StateChange {
    StateChange {
        account_id: "acct1".into(),
        email: "7".into(),
        mailbox: "3".into(),
        submission: "1".into(),
        thread: "7".into(),
        crypto_key: "2".into(),
        mail_rule: "1".into(),
    }
}

fn endpoint(url: impl Into<String>) -> WebhookEndpoint {
    WebhookEndpoint {
        id: "wh1".into(),
        account_id: "acct1".into(),
        url: url.into(),
        secret: SECRET.to_vec(),
        events: vec![],
    }
}

/// A receiver on `127.0.0.1` that counts hits, keeps the last signature and body, and
/// answers `200` or, with `redirect_to`, a `307` to that URL.
#[derive(Clone, Default)]
struct Receiver {
    hits: Arc<AtomicU32>,
    last: Arc<std::sync::Mutex<(String, Vec<u8>)>>,
    redirect_to: Option<String>,
}

async fn receive(
    axum::extract::State(rx): axum::extract::State<Receiver>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    rx.hits.fetch_add(1, Ordering::SeqCst);
    let sig = headers
        .get(SIGNATURE_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    *rx.last.lock().unwrap() = (sig, body.to_vec());
    match &rx.redirect_to {
        Some(to) => (StatusCode::TEMPORARY_REDIRECT, [("location", to.clone())]).into_response(),
        None => StatusCode::OK.into_response(),
    }
}

/// Start a receiver; returns its port and handle.
async fn spawn_receiver(redirect_to: Option<String>) -> (u16, Receiver) {
    let rx = Receiver {
        redirect_to,
        ..Receiver::default()
    };
    let app = Router::new()
        .route("/hook", post(receive))
        .with_state(rx.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (port, rx)
}

/// Run the production dispatcher (the entry point `lib.rs` spawns) for one change to
/// `url`, long enough for every retry to have happened.
async fn dispatch_production(url: String) {
    let registry: Arc<dyn WebhookRegistry> =
        Arc::new(InMemoryWebhookRegistry::new(vec![endpoint(url)]));
    let (tx, rx) = broadcast::channel(4);
    let handle = tokio::spawn(run_webhook_dispatcher(registry, rx));
    tx.send(change()).unwrap();
    // 3 attempts with 200 ms + 400 ms backoff fit well inside this.
    tokio::time::sleep(Duration::from_millis(1500)).await;
    drop(tx);
    let _ = handle.await;
}

fn assert_blocked(url: &str, outcome: Result<StatusCode, DeliveryError>) {
    match outcome {
        Err(DeliveryError::Refused(Refusal::Blocked)) => {}
        other => panic!("{url}: expected the egress gate to refuse, got {other:?}"),
    }
}

#[tokio::test]
async fn private_address_targets_are_refused() {
    let body = build_payload(&change());
    for url in [
        "http://127.0.0.1:9/hook",
        "http://10.0.0.1/hook",
        "http://172.16.5.4/hook",
        "http://192.168.1.10/hook",
        "http://169.254.169.254/latest/meta-data/",
        "http://100.64.0.1/hook",
        "http://0.0.0.0/hook",
        "http://[::1]/hook",
        "http://[fd00::1]/hook",
        "http://[fe80::1]/hook",
        "http://[::ffff:10.0.0.1]/hook",
        // NAT64 well-known prefix carrying 127.0.0.1.
        "http://[64:ff9b::7f00:1]/hook",
        // Teredo: public server 65.54.227.120, client 127.0.0.1 (XOR-obfuscated).
        "http://[2001:0:4136:e378::80ff:fffe]/hook",
        // ISATAP under a public prefix carrying 169.254.169.254.
        "https://[2a00:1450::5efe:a9fe:a9fe]/hook",
    ] {
        assert_blocked(url, deliver(&endpoint(url), &body).await);
    }
}

#[tokio::test]
async fn a_name_resolving_to_a_private_address_is_refused() {
    let (port, rx) = spawn_receiver(None).await;
    let url = format!("http://localhost:{port}/hook");
    assert_blocked(
        &url,
        deliver(&endpoint(&url), &build_payload(&change())).await,
    );

    dispatch_production(url).await;
    assert_eq!(
        rx.hits.load(Ordering::SeqCst),
        0,
        "the dispatcher delivered to a name that resolves to loopback"
    );
}

#[tokio::test]
async fn the_production_dispatcher_does_not_reach_a_loopback_receiver() {
    let (port, rx) = spawn_receiver(None).await;
    dispatch_production(format!("http://127.0.0.1:{port}/hook")).await;
    assert_eq!(
        rx.hits.load(Ordering::SeqCst),
        0,
        "the dispatcher delivered to 127.0.0.1"
    );
}

#[tokio::test]
async fn a_redirect_is_not_followed() {
    let (inner_port, inner) = spawn_receiver(None).await;
    let (outer_port, outer) =
        spawn_receiver(Some(format!("http://127.0.0.1:{inner_port}/hook"))).await;
    let url = format!("http://127.0.0.1:{outer_port}/hook");

    // The first hop is allowed here, so only the redirect handling is under test.
    let outcome = deliver_with(
        &endpoint(&url),
        &build_payload(&change()),
        loopback_or_public,
    )
    .await;
    match outcome {
        Err(DeliveryError::Redirect(status)) => assert_eq!(status, StatusCode::TEMPORARY_REDIRECT),
        other => panic!("expected an unfollowed redirect, got {other:?}"),
    }
    assert_eq!(
        outer.hits.load(Ordering::SeqCst),
        1,
        "a redirect is not retried"
    );
    assert_eq!(
        inner.hits.load(Ordering::SeqCst),
        0,
        "the redirect target was reached"
    );

    // And through the production dispatcher the redirect target is never reached.
    dispatch_production(url).await;
    assert_eq!(
        inner.hits.load(Ordering::SeqCst),
        0,
        "the production dispatcher followed a redirect to a private target"
    );
}

#[tokio::test]
async fn non_http_and_credentialed_urls_are_refused() {
    let body = build_payload(&change());
    for url in [
        "file:///etc/passwd",
        "gopher://example.com/",
        "http://user:pass@example.com/hook",
        "not a url",
    ] {
        match deliver(&endpoint(url), &body).await {
            Err(DeliveryError::Refused(Refusal::BadRequest(_))) => {}
            other => panic!("{url}: expected a refused URL, got {other:?}"),
        }
    }
}

#[tokio::test]
async fn an_allowed_target_still_receives_a_signed_delivery() {
    let (port, rx) = spawn_receiver(None).await;
    let url = format!("http://127.0.0.1:{port}/hook");

    let outcome = deliver_with(
        &endpoint(&url),
        &build_payload(&change()),
        loopback_or_public,
    )
    .await;
    assert_eq!(outcome.unwrap(), StatusCode::OK);
    assert_eq!(rx.hits.load(Ordering::SeqCst), 1);

    // Through the dispatcher too, and the body verifies under the endpoint secret.
    let registry: Arc<dyn WebhookRegistry> =
        Arc::new(InMemoryWebhookRegistry::new(vec![endpoint(&url)]));
    let (tx, brx) = broadcast::channel(4);
    let handle = tokio::spawn(run_webhook_dispatcher_with(
        registry,
        brx,
        loopback_or_public,
    ));
    tx.send(change()).unwrap();
    for _ in 0..50 {
        if rx.hits.load(Ordering::SeqCst) >= 2 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(
        rx.hits.load(Ordering::SeqCst),
        2,
        "the dispatcher delivered"
    );
    let (sig, body) = rx.last.lock().unwrap().clone();
    assert!(
        verify_signature(SECRET, &body, &sig),
        "HMAC verification failed"
    );
    drop(tx);
    let _ = handle.await;
}
