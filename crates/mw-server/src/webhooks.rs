//! Outbound + inbound webhooks (SPEC §20.2, plan §3 e9). Owned by t6-e9; the
//! dispatcher is spawned and the routes mounted by e11.
//!
//! ## Outbound (a SECOND consumer of the engine `StateChange` broadcast)
//! Exactly the pattern the V5 push relay uses ([`crate::push_relay::run_dispatcher`]):
//! [`run_webhook_dispatcher`] drains a dedicated broadcast receiver
//! ([`crate::PushHandle::subscribe_relay`]) so it never inflates the WS/SSE
//! subscriber count. For each [`StateChange`] it looks up the account's webhooks
//! (via the [`WebhookRegistry`] e11 backs with the sealed-secret `webhooks` table),
//! signs an **HMAC-SHA256** over the JSON body, and POSTs it with retry + backoff on
//! 5xx / transport failure.
//!
//! **No mail content ever transits a webhook.** The payload is built from
//! [`StateChange::to_wire`], which carries only opaque state tokens + IDs (Email,
//! Mailbox, EmailSubmission, Thread, CryptoKey, MailRule) — never a subject, body,
//! or address. Asserted in tests.
//!
//! ## Inbound (webhook-rule actions)
//! [`inbound_webhook`] verifies an HMAC-SHA256 signature against a shared secret
//! before accepting a rule-action trigger, then hands off to the engine (wired by
//! e11). The signature check ([`verify_signature`]) is constant-time.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use axum::body::Bytes;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use tokio::sync::broadcast;

use mw_engine::StateChange;

/// The signature header on outbound deliveries (and expected on inbound calls):
/// `X-Mailwoman-Signature: sha256=<hex>` over the raw request body.
pub const SIGNATURE_HEADER: &str = "x-mailwoman-signature";
/// Names the event that produced an outbound delivery.
pub const EVENT_HEADER: &str = "x-mailwoman-event";

/// How many times a delivery is attempted before it is dropped (1 initial + retries).
const MAX_ATTEMPTS: u32 = 3;
/// Base backoff between retry attempts; doubles each attempt.
const RETRY_BASE: Duration = Duration::from_millis(200);
/// Per-attempt request timeout.
const DELIVERY_TIMEOUT: Duration = Duration::from_secs(10);

type HmacSha256 = Hmac<Sha256>;

// ---------------------------------------------------------------------------
// Registry seam (e11 backs it with the 0007 `webhooks` table)
// ---------------------------------------------------------------------------

/// A registered outbound webhook, with its HMAC secret ALREADY UNSEALED. e11's
/// registry implementation opens `webhooks.secret_sealed` with the store
/// `ServerKey` before handing an endpoint to the dispatcher, so the plaintext
/// secret exists only transiently in memory at signing time.
#[derive(Debug, Clone)]
pub struct WebhookEndpoint {
    /// Row id (`webhooks.id`) — an opaque delivery identifier, safe to log.
    pub id: String,
    /// Owning account (`webhooks.account_id`) — an opaque id, safe to log.
    pub account_id: String,
    /// Destination URL (`webhooks.url`).
    pub url: String,
    /// The unsealed HMAC-SHA256 secret. Never logged.
    pub secret: Vec<u8>,
    /// Event filter (`webhooks.event_filter`, decoded). Empty → deliver every
    /// change; otherwise deliver only when a listed event matches.
    pub events: Vec<String>,
}

impl WebhookEndpoint {
    /// Whether this endpoint wants a delivery for `change`. The broadcast carries a
    /// coalesced `StateChange` (state tokens, not per-type deltas), so the filter is
    /// coarse: empty = all, else it must name the generic `"StateChange"` event or
    /// the account.
    fn wants(&self, change: &StateChange) -> bool {
        self.events.is_empty()
            || self
                .events
                .iter()
                .any(|e| e == "StateChange" || e == "*" || *e == change.account_id)
    }
}

/// The lookup e11 fills against the sealed-secret `webhooks` table (a `Store`
/// method does not exist yet — same seam pattern as e3's `OAuthStore`). A test
/// double ([`InMemoryWebhookRegistry`]) exercises the dispatcher without a store.
#[async_trait]
pub trait WebhookRegistry: Send + Sync {
    /// The active webhooks for an account, secrets already unsealed.
    async fn list_for_account(&self, account_id: &str) -> Vec<WebhookEndpoint>;
}

/// An in-memory registry for tests + the default (empty) production wiring until
/// e11 backs it with the store.
#[derive(Debug, Default, Clone)]
pub struct InMemoryWebhookRegistry {
    endpoints: Vec<WebhookEndpoint>,
}

impl InMemoryWebhookRegistry {
    /// A registry serving a fixed endpoint set.
    pub fn new(endpoints: Vec<WebhookEndpoint>) -> Self {
        Self { endpoints }
    }
}

#[async_trait]
impl WebhookRegistry for InMemoryWebhookRegistry {
    async fn list_for_account(&self, account_id: &str) -> Vec<WebhookEndpoint> {
        self.endpoints
            .iter()
            .filter(|e| e.account_id == account_id)
            .cloned()
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Signing
// ---------------------------------------------------------------------------

/// The `sha256=<hex>` signature of `body` under `secret` (RFC 2104 HMAC-SHA256) —
/// the value of [`SIGNATURE_HEADER`]. The scheme mirrors GitHub/Stripe webhooks so
/// receivers can reuse existing verification code.
pub fn sign(secret: &[u8], body: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(body);
    let tag = mac.finalize().into_bytes();
    let mut hex = String::with_capacity(7 + tag.len() * 2);
    hex.push_str("sha256=");
    for b in tag {
        hex.push_str(&format!("{b:02x}"));
    }
    hex
}

/// Constant-time verification of a `sha256=<hex>` signature header against `body`
/// and `secret`. Uses the HMAC crate's built-in constant-time comparison.
pub fn verify_signature(secret: &[u8], body: &[u8], signature: &str) -> bool {
    let Some(hex) = signature.strip_prefix("sha256=") else {
        return false;
    };
    let Some(expected) = decode_hex(hex) else {
        return false;
    };
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(body);
    mac.verify_slice(&expected).is_ok()
}

/// Decode a lowercase/uppercase hex string to bytes; `None` on any invalid nibble.
fn decode_hex(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

/// The JSON body of an outbound delivery, built from a [`StateChange`]. Wraps the
/// frozen RFC 8887 wire object (IDs + state tokens ONLY — no mail content) with a
/// delivery id + timestamp. Serialised once so signature + transmission agree.
pub fn build_payload(change: &StateChange) -> Vec<u8> {
    let payload = serde_json::json!({
        "type": "StateChange",
        "accountId": change.account_id,
        "state": change.to_wire(),
        "timestamp": crate::push_relay::now_rfc3339(),
    });
    serde_json::to_vec(&payload).expect("StateChange payload serialises")
}

// ---------------------------------------------------------------------------
// Outbound dispatcher — second consumer of the StateChange broadcast
// ---------------------------------------------------------------------------

/// Drain the webhook broadcast for the life of the process, delivering a signed
/// POST to every matching endpoint of a changed account. Spawned once by e11 in
/// `build_app` (fed by [`crate::PushHandle::subscribe_relay`], distinct from the
/// WS/SSE and push-relay receivers). Never carries message content.
pub async fn run_webhook_dispatcher(
    registry: Arc<dyn WebhookRegistry>,
    mut rx: broadcast::Receiver<StateChange>,
    http: reqwest::Client,
) {
    loop {
        match rx.recv().await {
            Ok(change) => dispatch(&registry, &http, &change).await,
            Err(broadcast::error::RecvError::Lagged(n)) => {
                tracing::warn!("webhook dispatcher lagged {n} changes");
            }
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
}

/// Sign + deliver a change to every matching endpoint of its account.
async fn dispatch(
    registry: &Arc<dyn WebhookRegistry>,
    http: &reqwest::Client,
    change: &StateChange,
) {
    let endpoints = registry.list_for_account(&change.account_id).await;
    if endpoints.is_empty() {
        return;
    }
    let body = build_payload(change);
    for endpoint in endpoints {
        if !endpoint.wants(change) {
            continue;
        }
        let signature = sign(&endpoint.secret, &body);
        match deliver_with_retry(http, &endpoint, &body, &signature).await {
            Ok(status) => {
                metrics::counter!("mw_webhook_deliveries_total", "result" => "ok").increment(1);
                // Only opaque ids are logged — never the URL host's data or secret.
                tracing::debug!(
                    "webhook {} delivered ({}) for account {}",
                    endpoint.id,
                    status,
                    endpoint.account_id
                );
            }
            Err(e) => {
                metrics::counter!("mw_webhook_deliveries_total", "result" => "fail").increment(1);
                tracing::warn!("webhook {} gave up after retries: {e}", endpoint.id);
            }
        }
    }
}

/// POST `body` to `endpoint`, retrying on a 5xx response or a transport error with
/// exponential backoff. Returns the final 2xx/4xx status, or an error if every
/// attempt failed. A 4xx is NOT retried (the receiver rejected the request shape).
async fn deliver_with_retry(
    http: &reqwest::Client,
    endpoint: &WebhookEndpoint,
    body: &[u8],
    signature: &str,
) -> anyhow::Result<StatusCode> {
    let mut last_err: Option<String> = None;
    for attempt in 0..MAX_ATTEMPTS {
        if attempt > 0 {
            tokio::time::sleep(RETRY_BASE * 2u32.pow(attempt - 1)).await;
        }
        let sent = http
            .post(&endpoint.url)
            .header(SIGNATURE_HEADER, signature)
            .header(EVENT_HEADER, "StateChange")
            .header("content-type", "application/json")
            .timeout(DELIVERY_TIMEOUT)
            .body(body.to_vec())
            .send()
            .await;
        match sent {
            Ok(resp) => {
                let status =
                    StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
                if status.is_server_error() {
                    last_err = Some(format!("server error {status}"));
                    continue; // retry
                }
                return Ok(status); // 2xx or 4xx: delivered / rejected, do not retry
            }
            Err(e) => {
                last_err = Some(e.to_string());
                continue; // transport error: retry
            }
        }
    }
    anyhow::bail!(
        "{MAX_ATTEMPTS} attempts failed: {}",
        last_err.unwrap_or_else(|| "unknown".into())
    )
}

// ---------------------------------------------------------------------------
// Inbound webhook-rule actions
// ---------------------------------------------------------------------------

/// The shared secret guarding the inbound endpoint. Set by e11 (env
/// `MW_WEBHOOK_INBOUND_SECRET`); `None` → inbound webhooks are disabled (`404`).
static INBOUND_SECRET: std::sync::RwLock<Option<Vec<u8>>> = std::sync::RwLock::new(None);

/// The dispatcher that runs an inbound webhook's mapped rule/Sieve action against
/// the engine. `None` (the default) → the endpoint authenticates but reports that no
/// action dispatcher is wired (a no-op `202`). The engine-backed sink is injected at
/// mount, exactly like the outbound [`WebhookRegistry`] adapter.
static INBOUND_SINK: std::sync::RwLock<Option<Arc<dyn WebhookActionSink>>> =
    std::sync::RwLock::new(None);

/// Configure the inbound-webhook shared secret (wired by e11).
pub fn set_inbound_secret(secret: Option<Vec<u8>>) {
    *INBOUND_SECRET.write().expect("inbound secret lock") = secret;
}

/// Install (or clear) the inbound-action dispatcher. The mount lane injects an
/// engine-backed [`WebhookActionSink`] here (see the outbound
/// [`WebhookRegistry`] wiring for the parallel); tests install an in-memory double.
pub fn set_inbound_dispatcher(sink: Option<Arc<dyn WebhookActionSink>>) {
    *INBOUND_SINK.write().expect("inbound sink lock") = sink;
}

/// The minimal envelope an inbound `run_rules` action evaluates the account's stored
/// mail rules against. Only header-shaped fields — never a full body dump in a log.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookEnvelope {
    #[serde(default)]
    pub from: String,
    #[serde(default)]
    pub to: String,
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub message_id: Option<String>,
}

/// A rule/Sieve action an authenticated inbound webhook asks the server to run.
///
/// `run_rules` re-evaluates the account's stored GUI/Sieve rules against a supplied
/// envelope; `sieve` carries one Sieve action (`{"kind":"move","mailbox":"…"}`, the
/// `mw_sieve::Action` wire shape) to execute on a target message — the Sieve webhook
/// action.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InboundAction {
    RunRules {
        envelope: WebhookEnvelope,
    },
    Sieve {
        /// A single `mw_sieve::Action` in its serde form, validated by the sink.
        action: serde_json::Value,
        #[serde(default)]
        message_id: Option<String>,
    },
}

/// The parsed inbound webhook body: which account, which action.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub struct InboundWebhookRequest {
    pub account: String,
    pub action: InboundAction,
}

/// The outcome of a dispatched inbound action — an opaque count safe to return
/// (never mail content).
#[derive(Debug, Clone, Copy, Default)]
pub struct WebhookDispatch {
    /// How many rule/Sieve actions the dispatch fired.
    pub fired: u64,
}

/// The engine seam an inbound webhook drives. The mount lane backs it with the real
/// engine (evaluate stored rules / apply a Sieve action); the in-memory
/// [`InMemoryWebhookActionSink`] exercises the boundary in tests.
#[async_trait]
pub trait WebhookActionSink: Send + Sync {
    /// Run `action` for `account`. Returns the fired-action count, or an error
    /// string (never containing mail content) that maps to a `502`.
    async fn fire(&self, account: &str, action: &InboundAction) -> Result<WebhookDispatch, String>;
}

/// An in-memory [`WebhookActionSink`] for tests: records every dispatched action and
/// reports a fixed fired-count so a test can assert an inbound webhook fired a rule.
#[derive(Debug, Default)]
pub struct InMemoryWebhookActionSink {
    fired: std::sync::Mutex<Vec<(String, InboundAction)>>,
}

impl InMemoryWebhookActionSink {
    pub fn new() -> Self {
        Self::default()
    }

    /// The (account, action) pairs dispatched so far.
    pub fn dispatched(&self) -> Vec<(String, InboundAction)> {
        self.fired.lock().expect("sink lock").clone()
    }
}

#[async_trait]
impl WebhookActionSink for InMemoryWebhookActionSink {
    async fn fire(&self, account: &str, action: &InboundAction) -> Result<WebhookDispatch, String> {
        self.fired
            .lock()
            .expect("sink lock")
            .push((account.to_string(), action.clone()));
        Ok(WebhookDispatch { fired: 1 })
    }
}

/// `POST /api/webhooks/inbound` — accept an external system's signed call and run the
/// mapped rule/Sieve action. The body is HMAC-verified against the configured shared
/// secret before anything runs (constant-time). Disabled (`404`) when no secret is
/// set; rejected (`401`) on a bad/absent signature; `400` on an unrecognized action;
/// `502` when the dispatcher errors. On success returns `202` with the opaque
/// fired-action count. When no dispatcher is wired the boundary still authenticates
/// and returns `202` (`no_dispatcher`) so the mount can be completed independently.
pub async fn inbound_webhook(headers: HeaderMap, body: Bytes) -> Response {
    let secret = INBOUND_SECRET.read().expect("inbound secret lock").clone();
    let Some(secret) = secret else {
        return (StatusCode::NOT_FOUND, "inbound webhooks are not configured").into_response();
    };
    let signature = headers
        .get(SIGNATURE_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    if !verify_signature(&secret, &body, signature) {
        return (StatusCode::UNAUTHORIZED, "invalid webhook signature").into_response();
    }

    let request: InboundWebhookRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(_) => {
            metrics::counter!("mw_webhook_inbound_total", "result" => "bad_request").increment(1);
            return (
                StatusCode::BAD_REQUEST,
                "unrecognized inbound webhook action",
            )
                .into_response();
        }
    };

    let sink = INBOUND_SINK.read().expect("inbound sink lock").clone();
    let Some(sink) = sink else {
        // Authenticated + parsed, but no engine dispatcher is wired yet.
        metrics::counter!("mw_webhook_inbound_total", "result" => "no_dispatcher").increment(1);
        return (
            StatusCode::ACCEPTED,
            axum::Json(serde_json::json!({ "dispatched": false })),
        )
            .into_response();
    };

    match sink.fire(&request.account, &request.action).await {
        Ok(outcome) => {
            metrics::counter!("mw_webhook_inbound_total", "result" => "dispatched").increment(1);
            (
                StatusCode::ACCEPTED,
                axum::Json(serde_json::json!({ "dispatched": true, "fired": outcome.fired })),
            )
                .into_response()
        }
        Err(e) => {
            metrics::counter!("mw_webhook_inbound_total", "result" => "error").increment(1);
            // The error string is dispatcher-authored + mail-content-free by contract.
            tracing::warn!(
                "inbound webhook action failed for account {}: {e}",
                request.account
            );
            (StatusCode::BAD_GATEWAY, "inbound action dispatch failed").into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use std::sync::atomic::{AtomicU32, Ordering};

    use axum::Router;
    use axum::http::HeaderValue;
    use axum::routing::post;

    fn change(account: &str) -> StateChange {
        StateChange {
            account_id: account.into(),
            email: "7".into(),
            mailbox: "3".into(),
            submission: "1".into(),
            thread: "7".into(),
            crypto_key: "2".into(),
            mail_rule: "1".into(),
        }
    }

    #[test]
    fn payload_carries_no_mail_content_only_ids() {
        let body = build_payload(&change("acct1"));
        let text = String::from_utf8(body).unwrap();
        // The wire object is IDs + state tokens; assert the mail-content field keys
        // never appear (a structural §21.1 guarantee for the webhook path). Matched
        // as JSON keys (`"subject"`) so the check cannot false-positive on a
        // substring like `"Crypto"` containing `to`.
        for banned in [
            "\"subject\"",
            "\"body\"",
            "\"from\"",
            "\"to\"",
            "\"cc\"",
            "\"snippet\"",
            "\"preview\"",
            "@example",
        ] {
            assert!(!text.contains(banned), "payload leaked {banned}: {text}");
        }
        assert!(text.contains("acct1"));
        assert!(text.contains("StateChange"));
    }

    #[test]
    fn sign_and_verify_round_trip() {
        let secret = b"hunter2-shared-secret";
        let body = b"{\"type\":\"StateChange\"}";
        let sig = sign(secret, body);
        assert!(sig.starts_with("sha256="));
        assert!(verify_signature(secret, body, &sig));
        // Wrong secret / tampered body / malformed header all fail closed.
        assert!(!verify_signature(b"wrong", body, &sig));
        assert!(!verify_signature(secret, b"tampered", &sig));
        assert!(!verify_signature(secret, body, "not-a-signature"));
        assert!(!verify_signature(secret, body, "sha256=zzzz"));
    }

    /// A fake receiver that fails the first `fail_times` calls with 500, then 200,
    /// recording the signature it saw and how many times it was hit.
    #[derive(Clone)]
    struct Recorder {
        hits: Arc<AtomicU32>,
        fail_times: Arc<AtomicU32>,
        last_sig: Arc<std::sync::Mutex<String>>,
        last_body: Arc<std::sync::Mutex<Vec<u8>>>,
    }

    async fn recv(
        axum::extract::State(rec): axum::extract::State<Recorder>,
        headers: HeaderMap,
        body: Bytes,
    ) -> Response {
        rec.hits.fetch_add(1, Ordering::SeqCst);
        *rec.last_sig.lock().unwrap() = headers
            .get(SIGNATURE_HEADER)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_string();
        *rec.last_body.lock().unwrap() = body.to_vec();
        if rec
            .fail_times
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| {
                (n > 0).then_some(n - 1)
            })
            .is_ok()
        {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        StatusCode::OK.into_response()
    }

    async fn spawn_recorder(fail_times: u32) -> (String, Recorder) {
        let rec = Recorder {
            hits: Arc::new(AtomicU32::new(0)),
            fail_times: Arc::new(AtomicU32::new(fail_times)),
            last_sig: Arc::new(std::sync::Mutex::new(String::new())),
            last_body: Arc::new(std::sync::Mutex::new(Vec::new())),
        };
        let app = Router::new()
            .route("/hook", post(recv))
            .with_state(rec.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr: SocketAddr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        (format!("http://{addr}/hook"), rec)
    }

    #[tokio::test]
    async fn state_change_fires_a_signed_webhook_with_retry() {
        // Receiver returns 500 twice, then 200 → the dispatcher must retry to success.
        let (url, rec) = spawn_recorder(2).await;
        let secret = b"webhook-secret".to_vec();
        let registry: Arc<dyn WebhookRegistry> =
            Arc::new(InMemoryWebhookRegistry::new(vec![WebhookEndpoint {
                id: "wh1".into(),
                account_id: "acct1".into(),
                url: url.clone(),
                secret: secret.clone(),
                events: vec![],
            }]));

        let (tx, rx) = broadcast::channel(8);
        let http = reqwest::Client::new();
        let handle = tokio::spawn(run_webhook_dispatcher(registry, rx, http));

        tx.send(change("acct1")).unwrap();
        // Also send a change for an account with no webhook → no delivery.
        tx.send(change("other")).unwrap();

        // Wait for the retried delivery to land (2 failures + 1 success = 3 hits).
        for _ in 0..50 {
            if rec.hits.load(Ordering::SeqCst) >= 3 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert_eq!(
            rec.hits.load(Ordering::SeqCst),
            3,
            "expected 2 retries + success"
        );

        // The delivered body was HMAC-signed with the endpoint secret, and carries
        // no mail content.
        let sig = rec.last_sig.lock().unwrap().clone();
        let body = rec.last_body.lock().unwrap().clone();
        assert!(
            verify_signature(&secret, &body, &sig),
            "HMAC verification failed"
        );
        let text = String::from_utf8(body).unwrap();
        assert!(text.contains("acct1"));
        assert!(!text.contains("subject") && !text.contains("body"));

        drop(tx);
        let _ = handle.await;
    }

    /// Sign `body` and return the header map carrying the signature.
    fn signed_headers(secret: &[u8], body: &[u8]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            SIGNATURE_HEADER,
            HeaderValue::from_str(&sign(secret, body)).unwrap(),
        );
        headers
    }

    /// One sequential test drives auth → parse → dispatch. It is the only test that
    /// touches the process-global inbound secret + sink, so there is no cross-test
    /// race on those statics.
    #[tokio::test]
    async fn inbound_webhook_auth_parse_and_dispatch() {
        // Disabled when no secret configured.
        set_inbound_secret(None);
        set_inbound_dispatcher(None);
        let disabled = inbound_webhook(HeaderMap::new(), Bytes::from_static(b"{}")).await;
        assert_eq!(disabled.status(), StatusCode::NOT_FOUND);

        let secret = b"inbound-shared".to_vec();
        set_inbound_secret(Some(secret.clone()));

        let run_rules = serde_json::to_vec(&serde_json::json!({
            "account": "acct1",
            "action": { "kind": "run_rules", "envelope": { "from": "a@b.c", "subject": "hi" } },
        }))
        .unwrap();

        // Bad/absent signature → 401.
        let bad = inbound_webhook(HeaderMap::new(), Bytes::from(run_rules.clone())).await;
        assert_eq!(bad.status(), StatusCode::UNAUTHORIZED);

        // Valid signature but an unrecognized action → 400.
        let junk = Bytes::from_static(b"{\"action\":\"resync\"}");
        let junk_sig = signed_headers(&secret, &junk);
        let bad_body = inbound_webhook(junk_sig, junk).await;
        assert_eq!(bad_body.status(), StatusCode::BAD_REQUEST);

        // Valid + parsed, but no dispatcher wired → 202 (dispatched:false).
        let ok_no_sink = inbound_webhook(
            signed_headers(&secret, &run_rules),
            Bytes::from(run_rules.clone()),
        )
        .await;
        assert_eq!(ok_no_sink.status(), StatusCode::ACCEPTED);

        // Install the in-memory sink; a valid call now FIRES the rule action.
        let sink = Arc::new(InMemoryWebhookActionSink::new());
        set_inbound_dispatcher(Some(sink.clone()));
        let fired = inbound_webhook(
            signed_headers(&secret, &run_rules),
            Bytes::from(run_rules.clone()),
        )
        .await;
        assert_eq!(fired.status(), StatusCode::ACCEPTED);
        let dispatched = sink.dispatched();
        assert_eq!(dispatched.len(), 1, "the inbound webhook fired one action");
        assert_eq!(dispatched[0].0, "acct1");
        assert!(matches!(dispatched[0].1, InboundAction::RunRules { .. }));

        // The Sieve webhook action variant also dispatches.
        let sieve = serde_json::to_vec(&serde_json::json!({
            "account": "acct1",
            "action": {
                "kind": "sieve",
                "action": { "kind": "move", "mailbox": "Archive" },
                "message_id": "m1",
            },
        }))
        .unwrap();
        let sieve_resp =
            inbound_webhook(signed_headers(&secret, &sieve), Bytes::from(sieve.clone())).await;
        assert_eq!(sieve_resp.status(), StatusCode::ACCEPTED);
        let after = sink.dispatched();
        assert_eq!(after.len(), 2);
        assert!(matches!(after[1].1, InboundAction::Sieve { .. }));

        set_inbound_secret(None);
        set_inbound_dispatcher(None);
    }
}
