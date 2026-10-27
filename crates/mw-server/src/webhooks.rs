//! Outbound + inbound webhooks (SPEC §20.2, plan §3 e9; t28-e18).
//!
//! ## Outbound
//! An admin registers an endpoint for one account with the event types it wants
//! (`POST /admin/webhooks`, [`admin_create`]). Two streams feed the dispatcher:
//!
//! * typed engine events ([`EngineEvent`]: `message.received`, `message.flags`,
//!   `message.moved`, `message.deleted`, `submission.sent`, `submission.failed`,
//!   `pim.changed`), through the sink [`spawn`] attaches to the engine. An endpoint
//!   receives the kinds it lists and no others, so one for new mail is not called
//!   for a flag change.
//! * the coalesced `StateChange` broadcast ([`crate::PushHandle::subscribe_relay`],
//!   distinct from the WS/SSE receivers), for endpoints registered before 26.20
//!   whose filter is empty or names `StateChange`.
//!
//! Each delivery is signed with **HMAC-SHA256** over the JSON body and POSTed with
//! retry + backoff on 5xx / transport failure.
//!
//! ## Egress (t32-h5)
//! A webhook URL is chosen by a person, so every delivery passes the same SSRF gate
//! as the image proxy: [`mw_egress::validate_and_resolve_with`] resolves the host
//! once and refuses it unless an address passes the policy (public unicast,
//! [`mw_egress::ip_allowed`], in production), and the POST goes through a client
//! pinned to that address by [`mw_egress::harden_client`] (no proxy, no redirects).
//! A redirect is never followed: it ends the delivery as [`DeliveryError::Redirect`],
//! so the signed body never reaches the `Location`. Registration runs the same gate
//! (t28-e18), so a URL that would be refused at delivery is refused when it is
//! entered, with the reason; delivery still checks, because a name can resolve
//! elsewhere later.
//!
//! **No mail content ever transits a webhook.** Payloads carry opaque ids and state
//! tokens only — never a subject, body, or address. Asserted in tests.
//!
//! ## Inbound (webhook-rule actions)
//! Each inbound webhook is a stored object (0034) with its own sealed secret, the one
//! account it acts on, and the actions it may run. A call names the webhook
//! ([`WEBHOOK_ID_HEADER`]) and carries a timestamp and a nonce; the signature covers
//! `"<timestamp>.<nonce>.<body>"` ([`sign_inbound`]). A timestamp more than
//! [`SIGNATURE_WINDOW_SECS`] from the server clock is refused (`401`), and a nonce
//! is accepted once per webhook (`409` on a replay) — on every process sharing the
//! database, because the store's primary key decides. The process-wide
//! `MW_WEBHOOK_INBOUND_SECRET` of earlier releases is gone; setting it stops start-up
//! ([`reject_legacy_inbound_secret`]).

use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use axum::Json;
use axum::Router;
use axum::body::Bytes;
use axum::extract::{Path as UrlPath, State};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use hmac::{Hmac, KeyInit, Mac};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::Sha256;
use tokio::sync::{broadcast, mpsc};

use mw_engine::StateChange;
use mw_engine::change::{EngineEvent, EventKind, EventSink};

use crate::AppState;
use crate::admin_ctx::{AdminApiError, AdminCtx, AdminJson, check_if_match, etag_of, with_etag};
use crate::admin_router::AdminRegistrar;
use crate::capabilities::Capability;

/// The signature header on outbound deliveries and inbound calls:
/// `X-Mailwoman-Signature: sha256=<hex>`. Outbound: over the raw body. Inbound:
/// over `"<timestamp>.<nonce>.<body>"`.
pub const SIGNATURE_HEADER: &str = "x-mailwoman-signature";
/// Names the event that produced an outbound delivery.
pub const EVENT_HEADER: &str = "x-mailwoman-event";
/// Names the inbound webhook a call is for (its id, shown when it was created).
pub const WEBHOOK_ID_HEADER: &str = "x-mailwoman-webhook";
/// Unix seconds when an inbound call was signed.
pub const TIMESTAMP_HEADER: &str = "x-mailwoman-timestamp";
/// A value the caller never repeats for the same webhook (16 to 128 of
/// `A-Z a-z 0-9 - _`).
pub const NONCE_HEADER: &str = "x-mailwoman-nonce";
/// How far an inbound call's timestamp may be from the server clock, either way.
pub const SIGNATURE_WINDOW_SECS: i64 = 300;
/// The action kinds an inbound webhook can be allowed to run.
pub const INBOUND_ACTIONS: [&str; 2] = ["run_rules", "sieve"];

/// How many times a delivery is attempted before it is dropped (1 initial + retries).
const MAX_ATTEMPTS: u32 = 3;
/// Base backoff between retry attempts; doubles each attempt.
const RETRY_BASE: Duration = Duration::from_millis(200);
/// Per-attempt request timeout.
const DELIVERY_TIMEOUT: Duration = Duration::from_secs(10);
/// Typed events waiting for the dispatcher. A full queue drops the event (counted)
/// rather than slow the mutation that emitted it.
const EVENT_QUEUE: usize = 1024;

type HmacSha256 = Hmac<Sha256>;

// ---------------------------------------------------------------------------
// Registry seam (backed by the 0007 `webhooks` table)
// ---------------------------------------------------------------------------

/// A registered outbound webhook, with its HMAC secret ALREADY UNSEALED. The
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
    /// Event filter (`webhooks.event_filter`, decoded): typed event names
    /// ([`EventKind::as_str`]), or — for endpoints registered before 26.20 —
    /// empty / `StateChange` for the coalesced state stream.
    pub events: Vec<String>,
}

impl WebhookEndpoint {
    /// Whether this endpoint takes the coalesced `StateChange` stream: an empty
    /// filter (every endpoint before 26.20), or one naming `StateChange` or `*`.
    fn wants_state_change(&self) -> bool {
        self.events.is_empty()
            || self
                .events
                .iter()
                .any(|e| e == "StateChange" || e == "*" || e.is_empty())
    }

    /// Whether this endpoint lists the typed event `kind` (or `*`).
    fn wants_event(&self, kind: EventKind) -> bool {
        self.events.iter().any(|e| e == kind.as_str() || e == "*")
    }
}

/// The lookup the store backs (`stores_v6::WebhookRegistryAdapter`). A test double
/// ([`InMemoryWebhookRegistry`]) exercises the dispatcher without a store.
#[async_trait]
pub trait WebhookRegistry: Send + Sync {
    /// The active webhooks for an account, secrets already unsealed.
    async fn list_for_account(&self, account_id: &str) -> Vec<WebhookEndpoint>;
}

/// An in-memory registry for tests.
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
/// the value of [`SIGNATURE_HEADER`] on an outbound delivery. The scheme mirrors
/// GitHub/Stripe webhooks so receivers can reuse existing verification code.
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

/// The bytes an inbound call's signature covers: `"<timestamp>.<nonce>.<body>"`.
fn inbound_signed_bytes(timestamp: &str, nonce: &str, body: &[u8]) -> Vec<u8> {
    let mut signed = Vec::with_capacity(timestamp.len() + nonce.len() + 2 + body.len());
    signed.extend_from_slice(timestamp.as_bytes());
    signed.push(b'.');
    signed.extend_from_slice(nonce.as_bytes());
    signed.push(b'.');
    signed.extend_from_slice(body);
    signed
}

/// The `sha256=<hex>` signature an inbound caller sends: HMAC-SHA256 under the
/// webhook's secret over `"<timestamp>.<nonce>.<body>"`, `timestamp` being the
/// decimal unix seconds sent in [`TIMESTAMP_HEADER`].
pub fn sign_inbound(secret: &[u8], timestamp: &str, nonce: &str, body: &[u8]) -> String {
    sign(secret, &inbound_signed_bytes(timestamp, nonce, body))
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
        .map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok())
        .collect()
}

/// The JSON body of a `StateChange` delivery. Wraps the frozen RFC 8887 wire object
/// (IDs + state tokens ONLY — no mail content) with a timestamp. Serialised once so
/// signature + transmission agree.
pub fn build_payload(change: &StateChange) -> Vec<u8> {
    let payload = json!({
        "type": "StateChange",
        "accountId": change.account_id,
        "state": change.to_wire(),
        "timestamp": crate::push_relay::now_rfc3339(),
    });
    serde_json::to_vec(&payload).expect("StateChange payload serialises")
}

/// The JSON body of a typed-event delivery: the event name, the account, the opaque
/// ids it touched and a timestamp. No mail content.
pub fn build_event_payload(event: &EngineEvent) -> Vec<u8> {
    let payload = json!({
        "type": event.kind.as_str(),
        "accountId": event.account_id,
        "ids": event.ids,
        "timestamp": crate::push_relay::now_rfc3339(),
    });
    serde_json::to_vec(&payload).expect("event payload serialises")
}

// ---------------------------------------------------------------------------
// Outbound dispatcher
// ---------------------------------------------------------------------------

/// The address predicate a delivery's resolved address must pass (the shape
/// [`mw_egress::validate_and_resolve_with`] takes).
pub type AddressPolicy = fn(&IpAddr) -> bool;

/// Why a delivery did not reach a 2xx/4xx answer.
#[derive(Debug)]
pub enum DeliveryError {
    /// The URL failed the egress gate: not http(s), credentials in it, no host, or
    /// no resolved address the policy allows. Not retried.
    Refused(mw_egress::Refusal),
    /// The endpoint answered with a redirect, which is not followed. Not retried.
    Redirect(StatusCode),
    /// Every attempt ended in a 5xx or a transport error.
    Failed(String),
}

impl std::fmt::Display for DeliveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Refused(r) => write!(f, "refused by the egress policy ({r:?})"),
            Self::Redirect(s) => write!(f, "endpoint answered {s}; redirects are not followed"),
            Self::Failed(e) => f.write_str(e),
        }
    }
}

impl std::error::Error for DeliveryError {}

/// The engine-side end of the typed-event queue: [`EventSink::emit`] never blocks.
struct ChannelSink(mpsc::Sender<EngineEvent>);

impl EventSink for ChannelSink {
    fn emit(&self, event: EngineEvent) {
        if self.0.try_send(event).is_err() {
            metrics::counter!("mw_webhook_deliveries_total", "result" => "dropped").increment(1);
        }
    }
}

/// Start the outbound dispatchers for the life of the process: the `StateChange`
/// one on `state_changes`, and — in engine mode — the typed-event one, whose sink is
/// attached to `engine` here. Deliveries go only to public unicast addresses
/// ([`mw_egress::ip_allowed`]).
pub fn spawn(
    registry: Arc<dyn WebhookRegistry>,
    engine: Option<&Arc<mw_engine::Engine>>,
    state_changes: broadcast::Receiver<StateChange>,
) {
    spawn_with(registry, engine, state_changes, mw_egress::ip_allowed);
}

/// [`spawn`] under a caller-chosen address policy. Only the address range varies;
/// the URL checks, the pin and the redirect refusal do not.
pub fn spawn_with(
    registry: Arc<dyn WebhookRegistry>,
    engine: Option<&Arc<mw_engine::Engine>>,
    state_changes: broadcast::Receiver<StateChange>,
    policy: AddressPolicy,
) {
    tokio::spawn(run_webhook_dispatcher_with(
        registry.clone(),
        state_changes,
        policy,
    ));
    if let Some(engine) = engine {
        let (tx, rx) = mpsc::channel(EVENT_QUEUE);
        engine.attach_event_sink(Arc::new(ChannelSink(tx)));
        tokio::spawn(run_event_dispatcher_with(registry, rx, policy));
    }
}

/// Drain the `StateChange` broadcast for the life of the process, delivering a
/// signed POST to every endpoint of a changed account that takes the coalesced
/// stream. Never carries message content. Public unicast addresses only.
pub async fn run_webhook_dispatcher(
    registry: Arc<dyn WebhookRegistry>,
    rx: broadcast::Receiver<StateChange>,
) {
    run_webhook_dispatcher_with(registry, rx, mw_egress::ip_allowed).await
}

/// [`run_webhook_dispatcher`] under a caller-chosen address policy.
pub async fn run_webhook_dispatcher_with(
    registry: Arc<dyn WebhookRegistry>,
    mut rx: broadcast::Receiver<StateChange>,
    policy: AddressPolicy,
) {
    loop {
        match rx.recv().await {
            Ok(change) => {
                let endpoints = registry.list_for_account(&change.account_id).await;
                let wanted: Vec<WebhookEndpoint> = endpoints
                    .into_iter()
                    .filter(WebhookEndpoint::wants_state_change)
                    .collect();
                if !wanted.is_empty() {
                    let body = build_payload(&change);
                    deliver_all(&wanted, "StateChange", &body, policy).await;
                }
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                tracing::warn!("webhook dispatcher lagged {n} changes");
            }
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
}

/// Drain the typed-event queue for the life of the process, delivering each event
/// to the endpoints of its account that list its kind.
pub async fn run_event_dispatcher_with(
    registry: Arc<dyn WebhookRegistry>,
    mut rx: mpsc::Receiver<EngineEvent>,
    policy: AddressPolicy,
) {
    while let Some(event) = rx.recv().await {
        let endpoints = registry.list_for_account(&event.account_id).await;
        let wanted: Vec<WebhookEndpoint> = endpoints
            .into_iter()
            .filter(|e| e.wants_event(event.kind))
            .collect();
        if wanted.is_empty() {
            continue;
        }
        let body = build_event_payload(&event);
        deliver_all(&wanted, event.kind.as_str(), &body, policy).await;
    }
}

/// Deliver one signed body to each endpoint, counting the outcome.
async fn deliver_all(
    endpoints: &[WebhookEndpoint],
    event: &str,
    body: &[u8],
    policy: AddressPolicy,
) {
    for endpoint in endpoints {
        match deliver_event_with(endpoint, event, body, policy).await {
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
                let result = match e {
                    DeliveryError::Refused(_) | DeliveryError::Redirect(_) => "refused",
                    DeliveryError::Failed(_) => "fail",
                };
                metrics::counter!("mw_webhook_deliveries_total", "result" => result).increment(1);
                // The URL is not logged: the refusal names no host or address.
                tracing::warn!("webhook {} not delivered: {e}", endpoint.id);
            }
        }
    }
}

/// Sign `body` with the endpoint secret and deliver it, to public unicast addresses
/// only ([`mw_egress::ip_allowed`]). See [`deliver_with`].
pub async fn deliver(endpoint: &WebhookEndpoint, body: &[u8]) -> Result<StatusCode, DeliveryError> {
    deliver_with(endpoint, body, mw_egress::ip_allowed).await
}

/// [`deliver_event_with`] for a `StateChange` body.
pub async fn deliver_with(
    endpoint: &WebhookEndpoint,
    body: &[u8],
    policy: AddressPolicy,
) -> Result<StatusCode, DeliveryError> {
    deliver_event_with(endpoint, "StateChange", body, policy).await
}

/// Sign `body` with the endpoint secret and POST it to the endpoint under `policy`
/// with `event` in [`EVENT_HEADER`], retrying on a 5xx response or a transport error
/// with exponential backoff. Returns the final 2xx/4xx status (a 4xx is NOT retried:
/// the receiver rejected the request shape). A URL the egress gate refuses and a
/// redirect answer end the delivery at once.
///
/// Each attempt resolves the host once through the gate and connects only to the
/// address that passed, so a name cannot rebind between the check and the connect.
pub async fn deliver_event_with(
    endpoint: &WebhookEndpoint,
    event: &str,
    body: &[u8],
    policy: AddressPolicy,
) -> Result<StatusCode, DeliveryError> {
    let url = reqwest::Url::parse(&endpoint.url).map_err(|_| {
        DeliveryError::Refused(mw_egress::Refusal::BadRequest("malformed webhook URL"))
    })?;
    let signature = sign(&endpoint.secret, body);
    let mut last_err: Option<String> = None;
    for attempt in 0..MAX_ATTEMPTS {
        if attempt > 0 {
            tokio::time::sleep(RETRY_BASE * 2u32.pow(attempt - 1)).await;
        }
        let target = mw_egress::validate_and_resolve_with(url.clone(), policy)
            .await
            .map_err(DeliveryError::Refused)?;
        // `harden_client` pins host → the checked address, turns redirects off and
        // refuses ambient proxies (which would resolve the name themselves).
        let http = mw_egress::harden_client(reqwest::Client::builder(), &target.host, target.addr)
            .timeout(DELIVERY_TIMEOUT)
            .build()
            .map_err(|e| DeliveryError::Failed(e.to_string()))?;
        let sent = http
            .post(target.url)
            .header(SIGNATURE_HEADER, &signature)
            .header(EVENT_HEADER, event)
            .header("content-type", "application/json")
            .body(body.to_vec())
            .send()
            .await;
        match sent {
            Ok(resp) => {
                let status =
                    StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
                if status.is_redirection() {
                    return Err(DeliveryError::Redirect(status));
                }
                if status.is_server_error() {
                    last_err = Some(format!("server error {status}"));
                    continue; // retry
                }
                return Ok(status); // 2xx or 4xx: delivered / rejected, do not retry
            }
            Err(e) => {
                last_err = Some(e.without_url().to_string());
                continue; // transport error: retry
            }
        }
    }
    Err(DeliveryError::Failed(format!(
        "{MAX_ATTEMPTS} attempts failed: {}",
        last_err.unwrap_or_else(|| "unknown".into())
    )))
}

// ---------------------------------------------------------------------------
// Inbound webhook-rule actions
// ---------------------------------------------------------------------------

/// Refuse to start when `MW_WEBHOOK_INBOUND_SECRET` is set (even empty). Before
/// 26.20 it was the one secret every inbound call was checked against; inbound
/// webhooks are now stored objects with a secret each, and silently ignoring the
/// variable would leave an operator believing their old secret still works.
pub fn reject_legacy_inbound_secret() -> anyhow::Result<()> {
    reject_legacy_inbound_secret_with(|k| std::env::var_os(k).is_some())
}

fn reject_legacy_inbound_secret_with(is_set: impl Fn(&str) -> bool) -> anyhow::Result<()> {
    if is_set("MW_WEBHOOK_INBOUND_SECRET") {
        anyhow::bail!(
            "MW_WEBHOOK_INBOUND_SECRET is set but is no longer read: each inbound webhook \
             now has its own secret, created in the admin console (Integrations, \
             Webhooks) or with POST {}. Unset it to start.",
            mw_admin::config::admin_route("/webhooks")
        );
    }
    Ok(())
}

/// The dispatcher that runs an inbound webhook's action against the engine. `None`
/// (the default, and proxy mode) → an authenticated call is answered `202
/// {"dispatched": false}`. The engine-backed sink is injected at mount.
static INBOUND_SINK: std::sync::RwLock<Option<Arc<dyn WebhookActionSink>>> =
    std::sync::RwLock::new(None);

/// Install (or clear) the inbound-action dispatcher. The mount injects an
/// engine-backed [`WebhookActionSink`]; tests install an in-memory double.
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
/// `mw_sieve::Action` wire shape) to execute on a target message.
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

impl InboundAction {
    /// The kind name an inbound webhook's allowed-action list holds.
    pub fn kind(&self) -> &'static str {
        match self {
            InboundAction::RunRules { .. } => "run_rules",
            InboundAction::Sieve { .. } => "sieve",
        }
    }
}

/// The parsed inbound webhook body. The account is the webhook's own; a body that
/// names an account must name that one.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub struct InboundWebhookRequest {
    #[serde(default)]
    pub account: Option<String>,
    pub action: InboundAction,
}

/// The outcome of a dispatched inbound action — an opaque count safe to return
/// (never mail content).
#[derive(Debug, Clone, Copy, Default)]
pub struct WebhookDispatch {
    /// How many rule/Sieve actions the dispatch fired.
    pub fired: u64,
}

/// The engine seam an inbound webhook drives. The mount backs it with the real
/// engine; the in-memory [`InMemoryWebhookActionSink`] exercises the boundary in
/// tests.
#[async_trait]
pub trait WebhookActionSink: Send + Sync {
    /// Run `action` for `account`. Returns the fired-action count, or an error
    /// string (never containing mail content) that maps to a `502`.
    async fn fire(&self, account: &str, action: &InboundAction) -> Result<WebhookDispatch, String>;
}

/// An in-memory [`WebhookActionSink`] for tests: records every dispatched action and
/// reports a fixed fired-count.
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

/// What an inbound call is refused with before its action runs.
#[derive(Debug, PartialEq, Eq)]
enum InboundRefusal {
    /// No such webhook, a missing header, or a signature that does not verify.
    /// One answer for all three, so a caller learns nothing about which ids exist.
    Unauthenticated,
    /// Signed correctly but outside the time window.
    Stale,
    /// This nonce was already accepted for this webhook.
    Replayed,
}

fn header_str<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
}

fn nonce_ok(nonce: &str) -> bool {
    (16..=128).contains(&nonce.len())
        && nonce
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn unix_now() -> i64 {
    chrono::Utc::now().timestamp()
}

/// An inbound webhook as this module uses it (the store row without its secret).
struct Inbound {
    id: String,
    account_id: String,
    actions_json: String,
    label: String,
    created_by: String,
    created_at: i64,
    last_used_at: i64,
}

/// The store's inbound row, minus the secret. (The row type is not re-exported by
/// `mw-store`, so it is taken apart here rather than named.)
macro_rules! inbound_from {
    ($row:expr) => {{
        let r = $row;
        Inbound {
            id: r.id,
            account_id: r.account_id,
            actions_json: r.actions_json,
            label: r.label,
            created_by: r.created_by,
            created_at: r.created_at,
            last_used_at: r.last_used_at,
        }
    }};
}

/// Authenticate an inbound call: the webhook exists, the signature over
/// `timestamp.nonce.body` verifies under its secret, the timestamp is inside the
/// window, and the nonce is new. The signature is checked before the nonce is
/// recorded, so a caller without the secret cannot use up nonces.
async fn authenticate_inbound(
    state: &AppState,
    headers: &HeaderMap,
    body: &[u8],
    now: i64,
) -> Result<Inbound, InboundRefusal> {
    let (Some(id), Some(ts), Some(nonce), Some(signature)) = (
        header_str(headers, WEBHOOK_ID_HEADER),
        header_str(headers, TIMESTAMP_HEADER),
        header_str(headers, NONCE_HEADER),
        header_str(headers, SIGNATURE_HEADER),
    ) else {
        return Err(InboundRefusal::Unauthenticated);
    };
    if !nonce_ok(nonce) {
        return Err(InboundRefusal::Unauthenticated);
    }
    let hook = match state.store.get_inbound_webhook(id).await {
        Ok(Some(hook)) => hook,
        Ok(None) => return Err(InboundRefusal::Unauthenticated),
        Err(e) => {
            tracing::warn!("inbound webhook lookup failed: {e}");
            return Err(InboundRefusal::Unauthenticated);
        }
    };
    if !verify_signature(
        &hook.secret,
        &inbound_signed_bytes(ts, nonce, body),
        signature,
    ) {
        return Err(InboundRefusal::Unauthenticated);
    }
    let Ok(signed_at) = ts.parse::<i64>() else {
        return Err(InboundRefusal::Stale);
    };
    if (now - signed_at).abs() > SIGNATURE_WINDOW_SECS {
        return Err(InboundRefusal::Stale);
    }
    // A pair is remembered for twice the window, so a call signed at the far edge
    // of the window cannot come back after its nonce was forgotten.
    match state
        .store
        .remember_webhook_nonce(&hook.id, nonce, now, now - 2 * SIGNATURE_WINDOW_SECS)
        .await
    {
        Ok(true) => Ok(inbound_from!(hook)),
        Ok(false) => Err(InboundRefusal::Replayed),
        Err(e) => {
            // Fail closed: a nonce that could not be recorded might be a replay.
            tracing::warn!("inbound webhook nonce could not be recorded: {e}");
            Err(InboundRefusal::Replayed)
        }
    }
}

fn error_json(status: StatusCode, error: &str) -> Response {
    (status, Json(json!({ "error": error }))).into_response()
}

/// `POST /api/webhooks/inbound` — run an inbound webhook's action. Headers:
/// [`WEBHOOK_ID_HEADER`], [`TIMESTAMP_HEADER`], [`NONCE_HEADER`] and
/// [`SIGNATURE_HEADER`] (see [`sign_inbound`]). Body: `{"action": {...}}`, optionally
/// with `"account"` naming the webhook's own account.
///
/// `401` unknown webhook, missing header or bad signature (one answer), or a
/// timestamp outside [`SIGNATURE_WINDOW_SECS`]; `409` a nonce already used; `400`
/// an unrecognised body; `403` another account or an action the webhook may not
/// run; `502` the dispatcher failed; `202` otherwise, with the opaque fired count.
pub(crate) async fn inbound_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let hook = match authenticate_inbound(&state, &headers, &body, unix_now()).await {
        Ok(hook) => hook,
        Err(refusal) => {
            let (status, result, error) = match refusal {
                InboundRefusal::Unauthenticated => (
                    StatusCode::UNAUTHORIZED,
                    "unauthenticated",
                    "invalid webhook signature",
                ),
                InboundRefusal::Stale => (
                    StatusCode::UNAUTHORIZED,
                    "stale",
                    "webhook timestamp outside the allowed window",
                ),
                InboundRefusal::Replayed => (
                    StatusCode::CONFLICT,
                    "replayed",
                    "webhook nonce already used",
                ),
            };
            metrics::counter!("mw_webhook_inbound_total", "result" => result).increment(1);
            return error_json(status, error);
        }
    };

    let request: InboundWebhookRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(_) => {
            metrics::counter!("mw_webhook_inbound_total", "result" => "bad_request").increment(1);
            return error_json(
                StatusCode::BAD_REQUEST,
                "unrecognized inbound webhook action",
            );
        }
    };
    let allowed: Vec<String> = serde_json::from_str(&hook.actions_json).unwrap_or_default();
    let other_account = request
        .account
        .as_deref()
        .is_some_and(|a| a != hook.account_id);
    if other_account || !allowed.iter().any(|a| a == request.action.kind()) {
        metrics::counter!("mw_webhook_inbound_total", "result" => "forbidden").increment(1);
        return error_json(
            StatusCode::FORBIDDEN,
            "this webhook may not run that action for that account",
        );
    }
    if let Err(e) = state
        .store
        .touch_inbound_webhook(&hook.id, unix_now())
        .await
    {
        tracing::warn!("inbound webhook {} last-used update failed: {e}", hook.id);
    }

    let sink = INBOUND_SINK.read().expect("inbound sink lock").clone();
    let Some(sink) = sink else {
        // Authenticated + parsed, but no engine dispatcher (proxy mode).
        metrics::counter!("mw_webhook_inbound_total", "result" => "no_dispatcher").increment(1);
        return (StatusCode::ACCEPTED, Json(json!({ "dispatched": false }))).into_response();
    };

    match sink.fire(&hook.account_id, &request.action).await {
        Ok(outcome) => {
            metrics::counter!("mw_webhook_inbound_total", "result" => "dispatched").increment(1);
            (
                StatusCode::ACCEPTED,
                Json(json!({ "dispatched": true, "fired": outcome.fired })),
            )
                .into_response()
        }
        Err(e) => {
            metrics::counter!("mw_webhook_inbound_total", "result" => "error").increment(1);
            // The error string is dispatcher-authored + mail-content-free by contract.
            tracing::warn!("inbound webhook {} action failed: {e}", hook.id);
            error_json(StatusCode::BAD_GATEWAY, "inbound action dispatch failed")
        }
    }
}

// ---------------------------------------------------------------------------
// Admin: list / create / delete (t28-e18)
// ---------------------------------------------------------------------------

/// The inbound endpoint, merged by `lib.rs`. The admin routes are registered
/// through the admin registry ([`register_admin`]).
pub(crate) fn router() -> Router<AppState> {
    Router::new().route("/api/webhooks/inbound", post(inbound_webhook))
}

/// The admin webhook routes, registered by `admin_router::endpoints`:
///   * `GET  {admin}/webhooks` (`integrations.read`) — every webhook, outbound and
///     inbound, with an `etag` (also the `ETag` header). Secrets are never returned.
///   * `POST {admin}/webhooks` (`integrations.write`) — create one; the secret is in
///     the answer, once. `"dryRun": true` validates (including the egress check of
///     an outbound URL) and answers what would be created without writing.
///   * `DELETE {admin}/webhooks/{id}` (`integrations.write`) — delete one.
///
/// Writes accept `If-Match: <etag>` and answer `412 {code: "stale", updatedBy,
/// updatedAt}` when the list changed since; invalid input is `422 {code, detail,
/// field}` with `field` a JSON pointer. Every write is audited with target
/// `webhook:<id>`.
pub(crate) fn register_admin(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/webhooks",
        Capability::IntegrationsRead,
        admin_list,
    );
    r.admin_endpoint(
        Method::POST,
        "/webhooks",
        Capability::IntegrationsWrite,
        admin_create,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/webhooks/{id}",
        Capability::IntegrationsWrite,
        admin_delete,
    );
}

/// One webhook as the admin API shows it.
fn outbound_dto(row: &mw_store::WebhookRow) -> Value {
    let events: Vec<String> = serde_json::from_str(&row.event_filter_json).unwrap_or_default();
    json!({
        "id": row.id,
        "direction": "outbound",
        "accountId": row.account_id,
        "label": row.label,
        "url": row.url,
        "events": events,
        "createdAt": row.created_at,
        "createdBy": row.created_by,
    })
}

fn inbound_dto(row: &Inbound) -> Value {
    let actions: Vec<String> = serde_json::from_str(&row.actions_json).unwrap_or_default();
    json!({
        "id": row.id,
        "direction": "inbound",
        "accountId": row.account_id,
        "label": row.label,
        "actions": actions,
        "createdAt": rfc3339(row.created_at),
        "createdBy": row.created_by,
        "lastUsedAt": (row.last_used_at > 0).then(|| rfc3339(row.last_used_at)),
    })
}

fn rfc3339(unix: i64) -> String {
    chrono::DateTime::from_timestamp(unix, 0)
        .unwrap_or_default()
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// Every webhook, newest first, with the list's etag and its latest write.
struct Listing {
    items: Vec<Value>,
    etag: String,
    updated_by: Option<String>,
    updated_at: Option<i64>,
}

async fn listing(state: &AppState) -> Result<Listing, AdminApiError> {
    let outbound = state.store.list_all_webhooks().await?;
    let inbound = state.store.list_inbound_webhooks().await?;
    let mut items: Vec<Value> = outbound.iter().map(outbound_dto).collect();
    items.extend(inbound.into_iter().map(|r| inbound_dto(&inbound_from!(r))));
    items.sort_by(|a, b| {
        let at = |v: &Value| v["createdAt"].as_str().unwrap_or_default().to_string();
        at(b)
            .cmp(&at(a))
            .then_with(|| a["id"].as_str().cmp(&b["id"].as_str()))
    });
    let ids: Vec<&str> = items.iter().filter_map(|i| i["id"].as_str()).collect();
    let etag = etag_of(&json!(ids));
    let latest = items.first();
    Ok(Listing {
        etag,
        updated_by: latest.and_then(|i| i["createdBy"].as_str().map(str::to_string)),
        updated_at: latest
            .and_then(|i| i["createdAt"].as_str())
            .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
            .map(|t| t.timestamp()),
        items,
    })
}

async fn audit(state: &AppState, ctx: &AdminCtx, action: &str, target: &str, detail: Value) {
    let entry = mw_admin::AuditLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        ts: chrono::Utc::now().to_rfc3339(),
        actor: ctx.username.clone(),
        actor_kind: mw_admin::ActorKind::Admin,
        action: action.to_string(),
        target: Some(target.to_string()),
        detail_json: mw_admin::redact_detail(&detail),
        ip: None,
    };
    if let Err(e) = state.v6.admin.audit(entry).await {
        tracing::warn!("webhook audit append failed: {e}");
    }
}

/// `GET {admin}/webhooks`.
async fn admin_list(State(state): State<AppState>, _ctx: AdminCtx) -> Response {
    match listing(&state).await {
        Ok(list) => {
            let body = json!({
                "webhooks": list.items,
                "eventKinds": EventKind::ALL.iter().map(|k| k.as_str()).collect::<Vec<_>>(),
                "actionKinds": INBOUND_ACTIONS,
                "etag": list.etag,
            });
            with_etag(Json(body), &list.etag)
        }
        Err(e) => e.into_response(),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateReq {
    #[serde(default)]
    direction: String,
    #[serde(default)]
    account_id: String,
    #[serde(default)]
    label: String,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    events: Vec<String>,
    #[serde(default)]
    actions: Vec<String>,
    #[serde(default)]
    dry_run: bool,
}

/// A fresh 32-byte secret, as the 64 hex characters handed to the admin.
fn new_secret() -> String {
    let mut bytes = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Check an outbound URL through the same egress gate deliveries use. The detail
/// names the class of refusal, never the resolved address.
async fn check_outbound_url(raw: &str, policy: AddressPolicy) -> Result<String, AdminApiError> {
    let Ok(url) = reqwest::Url::parse(raw.trim()) else {
        return Err(AdminApiError::invalid(
            "webhookUrlInvalid",
            "Enter a full URL starting with https:// or http://.",
            "/url",
        ));
    };
    match mw_egress::validate_and_resolve_with(url.clone(), policy).await {
        Ok(_) => Ok(url.to_string()),
        Err(refusal) => {
            let detail = match refusal {
                mw_egress::Refusal::BadRequest(why) => {
                    format!("The URL cannot be used for a webhook: {why}.")
                }
                _ => "The URL's host does not resolve to a public internet address, so \
                      deliveries to it would be refused. Private, loopback and link-local \
                      addresses are not allowed."
                    .to_string(),
            };
            Err(AdminApiError::invalid("webhookUrlRefused", detail, "/url"))
        }
    }
}

/// `POST {admin}/webhooks`.
async fn admin_create(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    AdminJson(req): AdminJson<CreateReq>,
) -> Response {
    match create(&state, &ctx, &headers, req, mw_egress::ip_allowed).await {
        Ok(r) => r,
        Err(e) => e.into_response(),
    }
}

async fn create(
    state: &AppState,
    ctx: &AdminCtx,
    headers: &HeaderMap,
    req: CreateReq,
    policy: AddressPolicy,
) -> Result<Response, AdminApiError> {
    let list = listing(state).await?;
    check_if_match(
        headers,
        &list.etag,
        list.updated_by.as_deref(),
        list.updated_at,
    )?;
    let account_id = req.account_id.trim().to_string();
    if account_id.is_empty() {
        return Err(AdminApiError::invalid(
            "required",
            "Enter the account this webhook is for.",
            "/accountId",
        ));
    }
    let label = req.label.trim().to_string();
    if label.chars().count() > 120 {
        return Err(AdminApiError::invalid(
            "tooLong",
            "Use at most 120 characters.",
            "/label",
        ));
    }
    let created_by = format!("admin:{}", ctx.username);
    match req.direction.as_str() {
        "outbound" => {
            let Some(raw_url) = req.url.as_deref().filter(|u| !u.trim().is_empty()) else {
                return Err(AdminApiError::invalid(
                    "required",
                    "Enter the URL deliveries are sent to.",
                    "/url",
                ));
            };
            let mut events = req.events.clone();
            events.sort();
            events.dedup();
            if events.is_empty() {
                return Err(AdminApiError::invalid(
                    "required",
                    "Choose at least one event.",
                    "/events",
                ));
            }
            if let Some(bad) = events.iter().find(|e| EventKind::parse(e).is_none()) {
                return Err(AdminApiError::invalid(
                    "unknownEvent",
                    format!("{bad} is not an event a webhook can receive."),
                    "/events",
                ));
            }
            let url = check_outbound_url(raw_url, policy).await?;
            let id = format!("wh_{}", uuid::Uuid::new_v4().simple());
            let impact = json!({
                "direction": "outbound", "accountId": account_id, "events": events,
                "host": reqwest::Url::parse(&url).ok().and_then(|u| u.host_str().map(str::to_string)),
            });
            if req.dry_run {
                return Ok(Json(json!({ "dryRun": true, "impact": impact })).into_response());
            }
            let secret = new_secret();
            let row = mw_store::WebhookRow {
                id: id.clone(),
                account_id,
                url,
                secret_sealed: state.store.seal_webhook_secret(secret.as_bytes())?,
                event_filter_json: serde_json::to_string(&events).unwrap_or_else(|_| "[]".into()),
                created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                label,
                created_by,
            };
            state.store.put_webhook(&row).await?;
            audit(
                state,
                ctx,
                "webhook.create",
                &format!("webhook:{id}"),
                impact,
            )
            .await;
            created(state, outbound_dto(&row), secret).await
        }
        "inbound" => {
            let mut actions = req.actions.clone();
            actions.sort();
            actions.dedup();
            if actions.is_empty() {
                return Err(AdminApiError::invalid(
                    "required",
                    "Choose at least one action.",
                    "/actions",
                ));
            }
            if let Some(bad) = actions
                .iter()
                .find(|a| !INBOUND_ACTIONS.contains(&a.as_str()))
            {
                return Err(AdminApiError::invalid(
                    "unknownAction",
                    format!("{bad} is not an action an inbound webhook can run."),
                    "/actions",
                ));
            }
            let id = format!("whin_{}", uuid::Uuid::new_v4().simple());
            let impact = json!({
                "direction": "inbound", "accountId": account_id, "actions": actions,
            });
            if req.dry_run {
                return Ok(Json(json!({ "dryRun": true, "impact": impact })).into_response());
            }
            let secret = new_secret();
            let actions_json = serde_json::to_string(&actions).unwrap_or_else(|_| "[]".into());
            state
                .store
                .put_inbound_webhook(
                    &id,
                    &account_id,
                    secret.as_bytes(),
                    &actions_json,
                    &label,
                    &created_by,
                    unix_now(),
                )
                .await?;
            let row = state
                .store
                .get_inbound_webhook(&id)
                .await?
                .map(|r| inbound_from!(r))
                .ok_or_else(|| AdminApiError::internal("inbound webhook vanished after insert"))?;
            audit(
                state,
                ctx,
                "webhook.create",
                &format!("webhook:{id}"),
                impact,
            )
            .await;
            created(state, inbound_dto(&row), secret).await
        }
        _ => Err(AdminApiError::invalid(
            "invalidDirection",
            "Choose outbound (Mailwoman calls your URL) or inbound (your system calls Mailwoman).",
            "/direction",
        )),
    }
}

/// `201` with the new webhook, its secret (this once) and the list's new etag.
async fn created(
    state: &AppState,
    webhook: Value,
    secret: String,
) -> Result<Response, AdminApiError> {
    let etag = listing(state).await?.etag;
    Ok(with_etag(
        (
            StatusCode::CREATED,
            Json(json!({ "webhook": webhook, "secret": secret, "etag": etag })),
        ),
        &etag,
    ))
}

/// `DELETE {admin}/webhooks/{id}`.
async fn admin_delete(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
) -> Response {
    match delete_webhook(&state, &ctx, &headers, &id).await {
        Ok(r) => r,
        Err(e) => e.into_response(),
    }
}

async fn delete_webhook(
    state: &AppState,
    ctx: &AdminCtx,
    headers: &HeaderMap,
    id: &str,
) -> Result<Response, AdminApiError> {
    let list = listing(state).await?;
    check_if_match(
        headers,
        &list.etag,
        list.updated_by.as_deref(),
        list.updated_at,
    )?;
    let Some(item) = list.items.iter().find(|i| i["id"] == id) else {
        return Err(AdminApiError::not_found("No webhook has this id."));
    };
    let direction = item["direction"].as_str().unwrap_or_default().to_string();
    let detail = json!({ "direction": direction, "accountId": item["accountId"] });
    if direction == "inbound" {
        state.store.delete_inbound_webhook(id).await?;
    } else {
        state.store.delete_webhook(id).await?;
    }
    audit(
        state,
        ctx,
        "webhook.delete",
        &format!("webhook:{id}"),
        detail,
    )
    .await;
    let etag = listing(state).await?.etag;
    Ok(with_etag(StatusCode::NO_CONTENT, &etag))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use std::sync::atomic::{AtomicU32, Ordering};

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
        let event = EngineEvent {
            account_id: "acct1".into(),
            kind: EventKind::MessageReceived,
            ids: vec!["m1".into()],
        };
        for body in [build_payload(&change("acct1")), build_event_payload(&event)] {
            let text = String::from_utf8(body).unwrap();
            // Matched as JSON keys (`"subject"`) so the check cannot false-positive
            // on a substring like `"Crypto"` containing `to`.
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
        }
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
        // A multi-byte character at an even offset must not panic the decoder.
        assert!(!verify_signature(secret, body, "sha256=aé"));
    }

    #[test]
    fn the_inbound_signature_covers_timestamp_nonce_and_body() {
        let secret = b"s";
        let sig = sign_inbound(secret, "1767225600", "nonce-0123456789ab", b"{}");
        let signed = inbound_signed_bytes("1767225600", "nonce-0123456789ab", b"{}");
        assert!(verify_signature(secret, &signed, &sig));
        for (ts, nonce, body) in [
            ("1767225601", "nonce-0123456789ab", &b"{}"[..]),
            ("1767225600", "nonce-0123456789ac", &b"{}"[..]),
            ("1767225600", "nonce-0123456789ab", &b"{ }"[..]),
        ] {
            assert!(
                !verify_signature(secret, &inbound_signed_bytes(ts, nonce, body), &sig),
                "changing any part breaks the signature"
            );
        }
    }

    #[test]
    fn nonces_have_a_bounded_alphabet_and_length() {
        assert!(nonce_ok("abcdefghijklmnop"));
        assert!(nonce_ok("A-b_0123456789-xyz"));
        assert!(!nonce_ok("short"));
        assert!(!nonce_ok(&"a".repeat(129)));
        assert!(!nonce_ok("abcdefghijklmnop."));
        assert!(!nonce_ok("abcdefghijklmno p"));
    }

    #[test]
    fn the_legacy_global_secret_stops_start_up() {
        assert!(reject_legacy_inbound_secret_with(|_| false).is_ok());
        let err = reject_legacy_inbound_secret_with(|k| k == "MW_WEBHOOK_INBOUND_SECRET")
            .unwrap_err()
            .to_string();
        assert!(err.starts_with("MW_WEBHOOK_INBOUND_SECRET is set"), "{err}");
        assert!(err.ends_with("Unset it to start."), "{err}");
    }

    #[test]
    fn endpoints_take_the_events_they_list() {
        let ep = |events: &[&str]| WebhookEndpoint {
            id: "w".into(),
            account_id: "a".into(),
            url: "https://hooks.example/x".into(),
            secret: vec![],
            events: events.iter().map(|s| s.to_string()).collect(),
        };
        let received = ep(&["message.received"]);
        assert!(received.wants_event(EventKind::MessageReceived));
        assert!(!received.wants_event(EventKind::MessageFlags));
        assert!(!received.wants_state_change());
        // Endpoints from before 26.20 keep the coalesced stream and nothing else.
        let legacy = ep(&[]);
        assert!(legacy.wants_state_change());
        assert!(!legacy.wants_event(EventKind::MessageReceived));
        let star = ep(&["*"]);
        assert!(star.wants_state_change() && star.wants_event(EventKind::PimChanged));
    }

    /// A fake receiver that fails the first `fail_times` calls with 500, then 200,
    /// recording the signature, event header and body it saw.
    #[derive(Clone)]
    struct Recorder {
        hits: Arc<AtomicU32>,
        fail_times: Arc<AtomicU32>,
        last_sig: Arc<std::sync::Mutex<String>>,
        events: Arc<std::sync::Mutex<Vec<String>>>,
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
        rec.events.lock().unwrap().push(
            headers
                .get(EVENT_HEADER)
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default()
                .to_string(),
        );
        *rec.last_body.lock().unwrap() = body.to_vec();
        if rec
            .fail_times
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1))
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
            events: Arc::new(std::sync::Mutex::new(Vec::new())),
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

    /// The receiver is on loopback, which production refuses; this policy lets it
    /// stand in for a public endpoint (the refusal is covered by t32_h5).
    fn loopback_ok(ip: &IpAddr) -> bool {
        ip.is_loopback() || mw_egress::ip_allowed(ip)
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
        let handle = tokio::spawn(run_webhook_dispatcher_with(registry, rx, loopback_ok));

        tx.send(change("acct1")).unwrap();
        // Also send a change for an account with no webhook → no delivery.
        tx.send(change("other")).unwrap();

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

    #[tokio::test]
    async fn typed_events_reach_only_the_endpoints_that_list_them() {
        let (received_url, received) = spawn_recorder(0).await;
        let (flags_url, flags) = spawn_recorder(0).await;
        let (legacy_url, legacy) = spawn_recorder(0).await;
        let ep = |id: &str, url: &str, events: &[&str]| WebhookEndpoint {
            id: id.into(),
            account_id: "acct1".into(),
            url: url.into(),
            secret: b"s".to_vec(),
            events: events.iter().map(|s| s.to_string()).collect(),
        };
        let registry: Arc<dyn WebhookRegistry> = Arc::new(InMemoryWebhookRegistry::new(vec![
            ep("r", &received_url, &["message.received"]),
            ep("f", &flags_url, &["message.flags"]),
            ep("l", &legacy_url, &[]),
        ]));
        let (tx, rx) = mpsc::channel(8);
        let handle = tokio::spawn(run_event_dispatcher_with(registry, rx, loopback_ok));
        for kind in [EventKind::MessageReceived, EventKind::MessageFlags] {
            tx.send(EngineEvent {
                account_id: "acct1".into(),
                kind,
                ids: vec!["m1".into()],
            })
            .await
            .unwrap();
        }
        drop(tx);
        handle.await.unwrap();
        assert_eq!(*received.events.lock().unwrap(), ["message.received"]);
        assert_eq!(*flags.events.lock().unwrap(), ["message.flags"]);
        assert_eq!(
            legacy.hits.load(Ordering::SeqCst),
            0,
            "an endpoint with the old empty filter gets StateChange only"
        );
        let body: Value = serde_json::from_slice(&received.last_body.lock().unwrap()).unwrap();
        assert_eq!(body["type"], "message.received");
        assert_eq!(body["ids"], json!(["m1"]));
    }
}
