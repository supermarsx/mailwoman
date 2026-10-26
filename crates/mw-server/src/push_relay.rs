//! V5 push relay + native bearer-auth (plan §2.2/§2.3/§3 e5).
//!
//! Everything here is ADDITIVE — the browser cookie/same-origin path is
//! byte-identical. The native bearer mode lights up when a client opts in (bearer
//! token); CORS headers go only to the native shell origins ([`TAURI_ORIGINS`] plus
//! `MW_NATIVE_ORIGINS`), never to a browser origin.
//!
//! Deliverables (plan §3 e5):
//!   * VAPID keygen on first boot ([`ensure_vapid`]) + `GET /api/push/vapid` serving
//!     the PUBLIC key (private sealed at rest via `mw_store::Store`),
//!   * `POST /api/push/subscribe|unsubscribe` over `mw_store::PushSubscriptionRow`,
//!   * the native bearer-auth helpers (token hashing + the `native_sessions` marker;
//!     bearer requests skip the cookie-only CSRF guard) + the native-origin
//!     allowlist for CORS and cookie-less sign-in,
//!   * the push DISPATCHER ([`run_dispatcher`]) = a second consumer of the engine
//!     `StateChange` broadcast that sends OPAQUE wakes (no message content) —
//!     WebPush (RFC 8291 encryption, RFC 8292 VAPID) to browser/desktop endpoints,
//!     UnifiedPush (a plain POST) for Android, APNs mocked (needs an Apple account).

use anyhow::anyhow;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::{Json, extract::State};
use base64::Engine as _;
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use tokio::sync::broadcast;

use mw_engine::StateChange;
use mw_store::{PushSubscriptionRow, Store};
use p256::elliptic_curve::zeroize::{Zeroize, Zeroizing};

use crate::AppState;

/// The opaque wake payload. Push carries **NO message content** (plan §2.3 / risk
/// #8) — only this fixed marker whose sole meaning is "your account changed, wake
/// and re-fetch `/changes`". The subject/body of a message never transits push.
const WAKE_MARKER: &[u8] = b"mw-wake";

/// WebPush TTL (seconds) — how long the push service should hold an undelivered
/// wake. Four weeks: a wake that lands after the device comes back online still
/// triggers a refetch.
const WAKE_TTL: u32 = 2_419_200;

/// The RFC 8188 record size written into the `aes128gcm` header. The whole payload
/// goes in one record.
const RECORD_SIZE: u32 = 4096;

/// Largest push message body a push service must accept (RFC 8291 §4).
const MAX_MESSAGE_BYTES: usize = 4096;

/// Largest payload that fits one message: the body is the 86-byte header
/// (salt 16, rs 4, idlen 1, key 65), then the payload, the 0x02 delimiter and
/// the 16-byte tag.
const MAX_WAKE_PAYLOAD: usize = MAX_MESSAGE_BYTES - 86 - 1 - 16;

/// Lifetime of the VAPID JWT. RFC 8292 §2 caps `exp` at 24 hours after the
/// request (push services refuse longer); 12 hours leaves room for clock skew.
/// It is unrelated to [`WAKE_TTL`], which is how long the push service holds a wake.
const VAPID_JWT_LIFETIME_SECS: i64 = 12 * 60 * 60;

/// The origins a Tauri 2 webview loads the SPA from: `http://tauri.localhost` on
/// Windows and Android, `tauri://localhost` on macOS, iOS and Linux, and
/// `https://tauri.localhost` when a shell sets `useHttpsScheme`. The native shells
/// send one of these as `Origin` on every request to the user's server.
///
/// They are accepted by default (t31, decision D10) and nothing else is. A web page
/// cannot send any of them: a browser sets `Origin` from the page's own scheme and
/// host, and no page is served from `tauri:` or from the `tauri.localhost` host a
/// webview resolves internally. Accepting them therefore opens nothing to
/// cross-site requests from the web.
pub const TAURI_ORIGINS: [&str; 3] = [
    "http://tauri.localhost",
    "https://tauri.localhost",
    "tauri://localhost",
];

/// The native shell origins the server answers with CORS headers and lets sign in
/// without a cookie: [`TAURI_ORIGINS`], plus any the operator lists in
/// `MW_NATIVE_ORIGINS` (comma-separated, e.g. for a shell with another scheme).
/// A browser deployment is not affected: no browser origin is on the list, so a
/// browser request never gets an `Access-Control-*` header.
#[derive(Debug, Clone, Default)]
pub struct NativeAuthConfig {
    /// Origins `MW_NATIVE_ORIGINS` adds to [`TAURI_ORIGINS`]. Empty by default.
    pub extra_origins: Vec<String>,
}

impl NativeAuthConfig {
    /// Populate from `MW_NATIVE_ORIGINS` (comma-separated). Absent/empty → only
    /// [`TAURI_ORIGINS`].
    pub fn from_env() -> Self {
        let extra_origins = std::env::var("MW_NATIVE_ORIGINS")
            .ok()
            .filter(|s| !s.is_empty())
            .map(|s| {
                s.split(',')
                    .map(|o| o.trim().to_string())
                    .filter(|o| !o.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        Self { extra_origins }
    }

    /// Whether `origin` is a native shell origin (exact match).
    pub fn allows(&self, origin: &str) -> bool {
        TAURI_ORIGINS.contains(&origin) || self.extra_origins.iter().any(|o| o == origin)
    }
}

/// Extract a `Authorization: Bearer <token>` value, if present. The seam the
/// bearer-accept path uses (in addition to the cookie) on the authed routes;
/// bearer requests skip the cookie-only CSRF guard (no ambient authority to
/// protect — origin-agnostic).
pub fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let token = raw
        .strip_prefix("Bearer ")
        .or_else(|| raw.strip_prefix("bearer "))?;
    let token = token.trim();
    (!token.is_empty()).then(|| token.to_string())
}

/// Hash a bearer token for the `native_sessions` marker table. Only the HASH is
/// stored (plan §2.4), never the token itself.
pub(crate) fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let digest = hasher.finalize();
    let mut s = String::with_capacity(digest.len() * 2);
    for b in digest {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

// ---------------------------------------------------------------------------
// VAPID keypair (generated on first boot; private sealed at rest)
// ---------------------------------------------------------------------------

/// Ensure a VAPID keypair exists, generating + persisting one (private SEALED via
/// the store's `ServerKey`) on first boot. Idempotent: a no-op once a keypair is
/// stored. Called from `build_app` so `GET /api/push/vapid` and the dispatcher
/// always have a key.
pub(crate) async fn ensure_vapid(store: &Store) -> anyhow::Result<()> {
    if store.vapid_public().await?.is_some() {
        return Ok(());
    }
    let (public_b64, private_scalar) = generate_vapid_keypair()?;
    let created_at = now_rfc3339();
    store
        .store_vapid_keypair(&public_b64, &private_scalar, &created_at)
        .await?;
    tracing::info!("generated a fresh VAPID keypair for Web Push");
    Ok(())
}

/// Generate a P-256 VAPID keypair. Returns `(applicationServerKey, private scalar)`:
/// the public key is the base64url (no-pad) of the uncompressed SEC1 point (what the
/// browser passes to `pushManager.subscribe`); the private key is the raw 32-byte
/// EC scalar (what [`vapid_authorization`] signs with), only ever stored SEALED,
/// and wiped from memory when the returned buffer drops.
fn generate_vapid_keypair() -> anyhow::Result<(String, Zeroizing<Vec<u8>>)> {
    use p256::elliptic_curve::sec1::ToSec1Point;

    let secret = random_p256_secret();
    let point = secret.public_key().to_sec1_point(false);
    let public_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(point.as_bytes());
    let mut raw: [u8; 32] = secret.to_bytes().into();
    let private = Zeroizing::new(raw.to_vec());
    raw.zeroize();
    Ok((public_b64, private))
}

// ---------------------------------------------------------------------------
// HTTP endpoints
// ---------------------------------------------------------------------------

/// `GET /api/push/vapid` → `{ publicKey }` (public-only). Cookie- or bearer-authed
/// like every other endpoint; the browser needs this key to subscribe in-page.
pub(crate) async fn push_vapid(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(resp) = crate::authed(&state, &headers).await {
        return resp;
    }
    match state.store.vapid_public().await {
        Ok(Some(public_key)) => Json(json!({ "publicKey": public_key })).into_response(),
        Ok(None) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "VAPID key not initialized" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("VAPID public key load failed: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "server error").into_response()
        }
    }
}

/// The Web-Push subscription keys (§2.3), present for the `webpush` transport only.
#[derive(Debug, Deserialize)]
struct PushKeys {
    p256dh: String,
    auth: String,
}

/// `PushSubscriptionInfo` request body (plan §2.3). camelCase over the wire.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubscribeReq {
    transport: String,
    endpoint: String,
    #[serde(default)]
    keys: Option<PushKeys>,
    #[serde(default)]
    app_id: Option<String>,
    #[serde(default)]
    expires_at: Option<String>,
}

/// `POST /api/push/subscribe` (body = `PushSubscriptionInfo`) → `{ id, vapidPublicKey }`.
/// Stores the subscription idempotently (keyed by endpoint). Cookie- or bearer-authed.
pub(crate) async fn push_subscribe(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let subject = match crate::settings_routes::UserSubject::resolved(&state.store, &session).await
    {
        Ok(subject) => subject,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let allowed = crate::notification_policy::allowed(
        &state.store,
        &subject.as_subject(),
        "notifications.pushAllowed",
    )
    .await;
    let config = crate::notification_policy::for_subject(&state.store, &subject.as_subject()).await;
    match (allowed, config) {
        (Ok(true), Ok(config)) if config.enabled => {}
        (Err(_), _) | (_, Err(_)) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
        _ => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({"code":"notificationsDisabled"})),
            )
                .into_response();
        }
    }
    let req: SubscribeReq = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("invalid subscription: {e}") })),
            )
                .into_response();
        }
    };
    if !matches!(req.transport.as_str(), "webpush" | "unifiedpush" | "apns") {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "transport must be webpush|unifiedpush|apns" })),
        )
            .into_response();
    }
    if matches!(req.transport.as_str(), "webpush" | "unifiedpush")
        && !valid_push_endpoint(&req.endpoint)
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"code":"invalidPushEndpoint"})),
        )
            .into_response();
    }
    if let Some(expiry) = &req.expires_at
        && chrono::DateTime::parse_from_rfc3339(expiry).is_err()
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"code":"invalidPushExpiry"})),
        )
            .into_response();
    }
    let existing = match state
        .store
        .list_push_subscriptions(&session.account_id)
        .await
    {
        Ok(rows) => rows,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    if existing.len() >= 32 && !existing.iter().any(|row| row.endpoint == req.endpoint) {
        return (
            StatusCode::CONFLICT,
            Json(json!({"code":"pushSubscriptionLimit"})),
        )
            .into_response();
    }
    if req.transport == "webpush" && req.keys.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "webpush subscriptions require keys.p256dh + keys.auth" })),
        )
            .into_response();
    }
    let (p256dh, auth) = match req.keys {
        Some(k) => (Some(k.p256dh), Some(k.auth)),
        None => (None, None),
    };
    let row = PushSubscriptionRow {
        id: uuid::Uuid::new_v4().to_string(),
        account_id: session.account_id.clone(),
        transport: req.transport,
        endpoint: req.endpoint,
        p256dh,
        auth,
        app_id: req.app_id,
        expires_at: req.expires_at,
        created_at: now_rfc3339(),
        last_wake_at: None,
    };
    if let Err(e) = state.store.upsert_push_subscription(&row).await {
        if matches!(e, mw_store::StoreError::NotFound) {
            return (
                StatusCode::CONFLICT,
                Json(json!({"code":"pushEndpointAlreadyBound"})),
            )
                .into_response();
        }
        tracing::error!("failed to store push subscription: {e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "server error").into_response();
    }
    let id = existing
        .iter()
        .find(|old| old.endpoint == row.endpoint)
        .map(|old| old.id.as_str())
        .unwrap_or(&row.id);
    let vapid_public = state.store.vapid_public().await.ok().flatten();
    Json(json!({ "id": id, "vapidPublicKey": vapid_public })).into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PushPermissionQuery {
    endpoint_hash: String,
}

/// Recheck current account ownership/policy when an old queued wake reaches a browser.
pub(crate) async fn push_permission(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(query): axum::extract::Query<PushPermissionQuery>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(session) => session,
        Err(response) => return response,
    };
    if query.endpoint_hash.len() != 64
        || !query
            .endpoint_hash
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let allowed = async {
        let subject = crate::settings_routes::UserSubject::resolved(&state.store, &session).await?;
        let config =
            crate::notification_policy::for_subject(&state.store, &subject.as_subject()).await?;
        if !config.enabled
            || !config.rules.is_empty()
            || crate::notification_policy::quiet_at(&config.quiet_hours, chrono::Utc::now())
            || quiet_now(&state.store).await
            || !crate::notification_policy::allowed(
                &state.store,
                &subject.as_subject(),
                "notifications.pushAllowed",
            )
            .await?
        {
            return Ok::<bool, mw_store::StoreError>(false);
        }
        Ok(state
            .store
            .list_push_subscriptions(&session.account_id)
            .await?
            .iter()
            .any(|row| {
                let hash = Sha256::digest(row.endpoint.as_bytes())
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>();
                hash == query.endpoint_hash
                    && row.expires_at.as_ref().is_none_or(|expiry| {
                        chrono::DateTime::parse_from_rfc3339(expiry)
                            .is_ok_and(|expiry| expiry > chrono::Utc::now())
                    })
            }))
    }
    .await;
    match allowed {
        Ok(allowed) => (
            [(header::CACHE_CONTROL, "private, no-store")],
            Json(json!({ "allowed": allowed })),
        )
            .into_response(),
        Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}

/// `POST /api/push/unsubscribe {id|endpoint}`. Removes the subscription. Cookie- or
/// bearer-authed.
#[derive(Debug, Deserialize)]
struct UnsubscribeReq {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    endpoint: Option<String>,
}

pub(crate) async fn push_unsubscribe(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(session) => session,
        Err(resp) => return resp,
    };
    let req: UnsubscribeReq = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("invalid request: {e}") })),
            )
                .into_response();
        }
    };
    if req.endpoint.is_none() && req.id.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error":"id or endpoint required"})),
        )
            .into_response();
    }
    let outcome = state
        .store
        .delete_account_push_subscription(
            &session.account_id,
            req.endpoint.as_deref(),
            req.id.as_deref(),
        )
        .await;
    match outcome {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("failed to remove push subscription: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "server error").into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// Push dispatcher — second consumer of the engine StateChange broadcast
// ---------------------------------------------------------------------------

/// Drain the push-relay broadcast for the life of the process, sending an opaque
/// wake to every active subscription of a changed account (plan §2.3). Spawned
/// once by `build_app`. Never carries message content — the wake only prompts the
/// client to foreground-fetch `/changes` (the same refetch the WS/SSE path does).
pub(crate) async fn run_dispatcher(store: Store, mut rx: broadcast::Receiver<StateChange>) {
    loop {
        match rx.recv().await {
            Ok(change) => dispatch_wakes(&store, &change).await,
            Err(broadcast::error::RecvError::Lagged(n)) => {
                tracing::warn!("push dispatcher lagged {n} changes");
            }
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
}

/// Fan an account change out to its subscriptions as opaque wakes, respecting a
/// (cheap, optional) global quiet-hours window (§17.3).
async fn dispatch_wakes(store: &Store, change: &StateChange) {
    if quiet_now(store).await {
        return;
    }
    // Resolve the same canonical account policy used by the settings API.
    // Unavailable or corrupt policy must never send an unsolicited notification.
    let Ok(Some(subject)) =
        crate::notification_policy::account_subject(store, &change.account_id).await
    else {
        return;
    };
    let Ok(config) = crate::notification_policy::for_subject(store, &subject.as_subject()).await
    else {
        return;
    };
    if !crate::notification_policy::allowed(
        store,
        &subject.as_subject(),
        "notifications.pushAllowed",
    )
    .await
    .unwrap_or(false)
    {
        return;
    }
    if !config.rules.is_empty() {
        return;
    } // Opaque wakes cannot safely evaluate content rules.
    if !config.enabled
        || crate::notification_policy::quiet_at(&config.quiet_hours, chrono::Utc::now())
    {
        return;
    }
    let subs = match store.list_push_subscriptions(&change.account_id).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("push dispatcher: subscription lookup failed: {e}");
            return;
        }
    };
    if subs.is_empty() {
        return;
    }
    // Load the sealed VAPID keypair once per change (only the WebPush transport
    // needs it). `None` before first-boot init — WebPush is skipped, others proceed.
    let vapid = vapid_for_dispatch(store.load_vapid_keypair().await.ok().flatten());
    let vapid_private = vapid.as_ref().map(|(_, private)| private.as_slice());
    for sub in subs {
        if sub.expires_at.as_ref().is_some_and(|expiry| {
            chrono::DateTime::parse_from_rfc3339(expiry)
                .map_or(true, |expiry| expiry <= chrono::Utc::now())
        }) {
            let _ = store
                .delete_account_push_subscription(&sub.account_id, None, Some(&sub.id))
                .await;
            continue;
        }
        let result = match sub.transport.as_str() {
            "webpush" => send_webpush(Some(&subject.domain), &sub, vapid_private).await,
            "unifiedpush" => send_unifiedpush(Some(&subject.domain), &sub).await,
            "apns" => send_apns_mock(&sub),
            other => {
                tracing::warn!("push dispatcher: unknown transport {other:?}");
                continue;
            }
        };
        match result {
            // Gone / not found: the endpoint expired — drop the subscription.
            Ok(status) if status == 404 || status == 410 => {
                let _ = store
                    .delete_account_push_subscription(&sub.account_id, None, Some(&sub.id))
                    .await;
            }
            Ok(_) => {
                let _ = store.touch_push_wake(&sub.id, &now_rfc3339()).await;
            }
            // The subscription id, never the endpoint: endpoints are capability
            // URLs, and send errors carry no URL (`without_url`).
            Err(e) => tracing::warn!("push send for subscription {} failed: {e:#}", sub.id),
        }
    }
}

/// The loaded VAPID keypair with the private scalar in a buffer that is wiped
/// when the dispatch of one change is done.
fn vapid_for_dispatch(loaded: Option<(String, Vec<u8>)>) -> Option<(String, Zeroizing<Vec<u8>>)> {
    loaded.map(|(public, private)| (public, Zeroizing::new(private)))
}

/// Send a VAPID-signed, RFC 8291-encrypted opaque wake to a Web Push endpoint.
/// [`build_wake_request`] builds the `http::Request` (headers + encrypted body);
/// it goes out over the in-tree `reqwest` client. The encrypted body is the fixed
/// [`WAKE_MARKER`] — never message content.
async fn send_webpush(
    domain: Option<&str>,
    sub: &PushSubscriptionRow,
    vapid_private: Option<&[u8]>,
) -> anyhow::Result<u16> {
    let Some(private_scalar) = vapid_private else {
        anyhow::bail!("no VAPID keypair available");
    };
    let (Some(p256dh), Some(auth)) = (sub.p256dh.as_deref(), sub.auth.as_deref()) else {
        anyhow::bail!("webpush subscription missing p256dh/auth");
    };
    let request = build_wake_request(
        &sub.endpoint,
        p256dh,
        auth,
        private_scalar,
        &vapid_contact(),
        WAKE_MARKER,
    )?;

    let (parts, body) = request.into_parts();
    let authorization = parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| anyhow!("invalid Web Push authorization"))?;
    let request = mw_egress::plan::Request::web_push(authorization, &body)
        .map_err(|_| anyhow!("invalid Web Push request"))?;
    send_guarded(&sub.endpoint, domain, request).await
}

/// Build the encrypted, VAPID-authenticated Web Push request for one subscription:
/// the body is `payload` encrypted to the subscriber's key (RFC 8291 over RFC 8188
/// `aes128gcm`), the `Authorization` header is a VAPID ES256 JWT (RFC 8292).
fn build_wake_request(
    endpoint: &str,
    p256dh: &str,
    auth: &str,
    vapid_private: &[u8],
    contact: &str,
    payload: &[u8],
) -> anyhow::Result<axum::http::Request<Vec<u8>>> {
    let uri: axum::http::Uri = endpoint.parse().map_err(|e| anyhow!("endpoint: {e}"))?;
    let audience = push_origin(&uri)?;
    let ua_public = p256::PublicKey::from_sec1_bytes(&b64url_decode(p256dh)?)
        .map_err(|e| anyhow!("p256dh: {e}"))?;
    let auth = b64url_decode(auth)?;
    if auth.len() != 16 {
        anyhow::bail!("webpush auth secret must be 16 bytes");
    }
    let vapid_key =
        p256::SecretKey::from_slice(vapid_private).map_err(|e| anyhow!("VAPID key load: {e}"))?;

    // A fresh salt and ephemeral key from the OS RNG for every message.
    let mut salt = [0u8; 16];
    rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut salt);
    let body = encrypt_aes128gcm(payload, &random_p256_secret(), &ua_public, &auth, &salt)?;
    let expires = chrono::Utc::now().timestamp() + VAPID_JWT_LIFETIME_SECS;
    let authorization = vapid_authorization(&vapid_key, &audience, contact, expires)?;

    axum::http::Request::builder()
        .method(axum::http::Method::POST)
        .uri(uri)
        .header("TTL", WAKE_TTL)
        .header(header::CONTENT_ENCODING, "aes128gcm")
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::AUTHORIZATION, authorization)
        .body(body)
        .map_err(|e| anyhow!("web push request: {e}"))
}

/// The RFC 6454 origin of a push endpoint, the VAPID `aud` claim (RFC 8292 §2):
/// scheme and host, plus the port when it is not the scheme's default.
fn push_origin(uri: &axum::http::Uri) -> anyhow::Result<String> {
    let scheme = uri
        .scheme_str()
        .ok_or_else(|| anyhow!("endpoint: no scheme"))?;
    let default_port = match scheme {
        "https" => 443,
        "http" => 80,
        other => anyhow::bail!("endpoint: unsupported scheme {other:?}"),
    };
    let host = uri.host().ok_or_else(|| anyhow!("endpoint: no host"))?;
    Ok(match uri.port_u16() {
        Some(port) if port != default_port => format!("{scheme}://{host}:{port}"),
        _ => format!("{scheme}://{host}"),
    })
}

/// RFC 8291 §3.4 encryption into one RFC 8188 `aes128gcm` record:
/// `salt(16) ‖ rs(4) ‖ idlen(1) ‖ keyid(65 = the sender's ephemeral public key) ‖
/// AES-128-GCM(payload ‖ 0x02)`. `as_secret` is a fresh key per message.
fn encrypt_aes128gcm(
    payload: &[u8],
    as_secret: &p256::SecretKey,
    ua_public: &p256::PublicKey,
    auth: &[u8],
    salt: &[u8; 16],
) -> anyhow::Result<Vec<u8>> {
    use aes_gcm::aead::{Aead, KeyInit};
    use hkdf::Hkdf;
    use p256::elliptic_curve::sec1::ToSec1Point;

    if payload.len() > MAX_WAKE_PAYLOAD {
        anyhow::bail!(
            "web push payload is {} bytes; at most {MAX_WAKE_PAYLOAD} fit in one \
             {MAX_MESSAGE_BYTES}-byte push message",
            payload.len()
        );
    }
    let ua_point = ua_public.to_sec1_point(false);
    let as_point = as_secret.public_key().to_sec1_point(false);
    let shared = p256::ecdh::diffie_hellman(as_secret.to_nonzero_scalar(), ua_public.as_affine());

    // IKM = HKDF(auth, ecdh_secret, "WebPush: info" ‖ 0 ‖ ua_public ‖ as_public, 32).
    let mut key_info = Vec::with_capacity(14 + 65 + 65);
    key_info.extend_from_slice(b"WebPush: info\0");
    key_info.extend_from_slice(ua_point.as_bytes());
    key_info.extend_from_slice(as_point.as_bytes());
    // Derived key material lives in buffers that are wiped on drop.
    let mut ikm = Zeroizing::new([0u8; 32]);
    Hkdf::<Sha256>::new(Some(auth), shared.raw_secret_bytes().as_slice())
        .expand(&key_info, ikm.as_mut())
        .map_err(|e| anyhow!("hkdf: {e}"))?;

    // CEK and nonce from the record salt (RFC 8188 §2.2, §2.3).
    let prk = Hkdf::<Sha256>::new(Some(salt), ikm.as_ref());
    let mut cek = Zeroizing::new([0u8; 16]);
    prk.expand(b"Content-Encoding: aes128gcm\0", cek.as_mut())
        .map_err(|e| anyhow!("hkdf: {e}"))?;
    let mut nonce = [0u8; 12];
    prk.expand(b"Content-Encoding: nonce\0", &mut nonce)
        .map_err(|e| anyhow!("hkdf: {e}"))?;

    let mut record = Zeroizing::new(Vec::with_capacity(payload.len() + 1));
    record.extend_from_slice(payload);
    record.push(0x02); // last-record delimiter, no padding
    let sealed = aes_gcm::Aes128Gcm::new_from_slice(cek.as_ref())
        .map_err(|e| anyhow!("aes-gcm key: {e}"))?
        .encrypt(&nonce.into(), record.as_slice())
        .map_err(|e| anyhow!("aes-gcm: {e}"))?;

    let mut out = Vec::with_capacity(21 + 65 + sealed.len());
    out.extend_from_slice(salt);
    out.extend_from_slice(&RECORD_SIZE.to_be_bytes());
    out.push(65);
    out.extend_from_slice(as_point.as_bytes());
    out.extend_from_slice(&sealed);
    Ok(out)
}

/// The RFC 8292 §3 header value `vapid t=<JWT>, k=<public key>`: an ES256 JWT
/// (raw r ‖ s signature) with `aud`, `exp` and `sub`, and the base64url
/// uncompressed public key it verifies with.
fn vapid_authorization(
    key: &p256::SecretKey,
    audience: &str,
    contact: &str,
    expires: i64,
) -> anyhow::Result<String> {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use p256::ecdsa::signature::Signer;
    use p256::elliptic_curve::sec1::ToSec1Point;

    let head = URL_SAFE_NO_PAD.encode(br#"{"typ":"JWT","alg":"ES256"}"#);
    let claims = serde_json::to_vec(&json!({ "aud": audience, "exp": expires, "sub": contact }))?;
    let signing_input = format!("{head}.{}", URL_SAFE_NO_PAD.encode(claims));
    let signature: p256::ecdsa::Signature =
        p256::ecdsa::SigningKey::from(key).sign(signing_input.as_bytes());
    let public = URL_SAFE_NO_PAD.encode(key.public_key().to_sec1_point(false).as_bytes());
    Ok(format!(
        "vapid t={signing_input}.{}, k={public}",
        URL_SAFE_NO_PAD.encode(signature.to_bytes())
    ))
}

/// A uniformly random P-256 secret key from the OS RNG. `SecretKey` wipes itself
/// on drop; the draw buffer is wiped here.
fn random_p256_secret() -> p256::SecretKey {
    // A valid non-zero scalar < group order; retry on the (astronomically rare)
    // invalid draw rather than coupling to a specific RNG-crate version.
    let mut bytes = Zeroizing::new([0u8; 32]);
    loop {
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, bytes.as_mut());
        if let Ok(sk) = p256::SecretKey::from_slice(bytes.as_ref()) {
            return sk;
        }
    }
}

/// Decode a base64url subscription field (browsers may or may not pad).
fn b64url_decode(s: &str) -> anyhow::Result<Vec<u8>> {
    use base64::engine::general_purpose::{URL_SAFE, URL_SAFE_NO_PAD};
    URL_SAFE_NO_PAD
        .decode(s)
        .or_else(|_| URL_SAFE.decode(s))
        .map_err(|e| anyhow!("base64url: {e}"))
}

/// Send an opaque wake to a UnifiedPush endpoint (Android, self-hostable, no
/// Google). UnifiedPush delivers the POST body to the app; the app treats any
/// delivery as "sync now", so the body is the fixed [`WAKE_MARKER`] — no content.
async fn send_unifiedpush(domain: Option<&str>, sub: &PushSubscriptionRow) -> anyhow::Result<u16> {
    send_guarded(
        &sub.endpoint,
        domain,
        mw_egress::plan::Request::post(
            mw_egress::TrafficClass::Push,
            "application/octet-stream",
            WAKE_MARKER,
        )
        .response_limit(8192),
    )
    .await
}

/// APNs opaque wake — MOCKED (plan §1.7 / risk #5): live APNs needs an Apple
/// provider certificate + account, unavailable on this Windows CI. A real send
/// would be a `content-available` background push carrying NO content. Recorded
/// here so the dispatcher path is exercised end-to-end without Apple infra.
fn send_apns_mock(sub: &PushSubscriptionRow) -> anyhow::Result<u16> {
    tracing::info!(
        "APNs opaque wake (mocked) for subscription {} — no content transits push",
        sub.id
    );
    Ok(200)
}

/// The VAPID `sub` claim (a contact URI push services use to reach the operator on
/// delivery problems). From `MW_VAPID_CONTACT`, else a neutral default.
fn vapid_contact() -> String {
    vapid_contact_from(std::env::var("MW_VAPID_CONTACT").ok())
}

/// [`vapid_contact`] for a given `MW_VAPID_CONTACT` value.
fn vapid_contact_from(value: Option<String>) -> String {
    value
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "mailto:push@mailwoman.invalid".to_string())
}

// ---------------------------------------------------------------------------
// Quiet hours (cheap, optional — §17.3)
// ---------------------------------------------------------------------------

fn valid_push_endpoint(endpoint: &str) -> bool {
    if endpoint.len() > 4096 {
        return false;
    }
    let Ok(url) = reqwest::Url::parse(endpoint) else {
        return false;
    };
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return false;
    }
    let Some(host) = url.host_str() else {
        return false;
    };
    host.trim_matches(['[', ']'])
        .parse::<std::net::IpAddr>()
        .map(|ip| mw_egress::ip_allowed(&ip))
        .unwrap_or(true)
}

async fn send_guarded<'a>(
    endpoint: &str,
    domain: Option<&'a str>,
    mut request: mw_egress::plan::Request<'a>,
) -> anyhow::Result<u16> {
    let url = reqwest::Url::parse(endpoint).map_err(|_| anyhow!("invalid push endpoint"))?;
    if url.scheme() != "https" {
        anyhow::bail!("push endpoint requires HTTPS");
    }
    if let Some(domain) = domain {
        request = request.for_domain(domain);
    }
    let response = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        mw_egress::plan::send(url, &request),
    )
    .await
    .map_err(|_| anyhow!("push timeout"))?;
    match response.outcome {
        Ok(_) => Ok(200),
        Err(mw_egress::Refusal::Status(status)) => Ok(status),
        Err(_) => Err(anyhow!("push delivery refused or unavailable")),
    }
}

/// Whether the current UTC hour falls in the operator's quiet-hours window. The
/// window is a `"start-end"` (UTC hours) `settings` value under `push.quiet_hours`;
/// absent/unparseable → never quiet (the default). One indexed settings read per
/// change — cheap enough to honor per the plan's "if cheap".
async fn quiet_now(store: &Store) -> bool {
    let subject = mw_store::settings_policy::SettingSubject {
        account_id: "",
        address: "",
        domain: "",
    };
    let effective =
        match crate::settings_registry::effective_for(store, "push.quietHours", &subject).await {
            Ok(Some(value)) => value,
            _ => return true,
        };
    if let (Some(start), Some(end)) = (
        effective
            .value
            .get("start")
            .and_then(serde_json::Value::as_u64),
        effective
            .value
            .get("end")
            .and_then(serde_json::Value::as_u64),
    ) {
        return in_quiet_window(start as u32, end as u32, current_utc_hour());
    }
    if effective.source != mw_store::settings_policy::SettingSource::Default {
        return false;
    }
    // Compatibility only when the canonical registry key has never been configured.
    match store.get_setting("push.quiet_hours").await {
        Ok(Some(raw)) => parse_quiet_window(&raw)
            .is_some_and(|(start, end)| in_quiet_window(start, end, current_utc_hour())),
        Ok(None) => false,
        Err(_) => true,
    }
}

/// Parse a `"start-end"` (24h UTC) quiet-hours window, e.g. `"22-7"`.
fn parse_quiet_window(raw: &str) -> Option<(u32, u32)> {
    let (start, end) = raw.trim().split_once('-')?;
    let start: u32 = start.trim().parse().ok()?;
    let end: u32 = end.trim().parse().ok()?;
    (start < 24 && end < 24).then_some((start, end))
}

/// Whether `hour` is inside `[start, end)`, wrapping past midnight when `start > end`.
fn in_quiet_window(start: u32, end: u32, hour: u32) -> bool {
    if start <= end {
        hour >= start && hour < end
    } else {
        hour >= start || hour < end
    }
}

fn current_utc_hour() -> u32 {
    use chrono::Timelike;
    chrono::Utc::now().hour()
}

/// An RFC 3339 timestamp for the store's textual `*_at` columns.
pub(crate) fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn bearer_token_parses_case_insensitively() {
        let mut h = HeaderMap::new();
        h.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer abc123"),
        );
        assert_eq!(bearer_token(&h).as_deref(), Some("abc123"));
        h.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("bearer  xyz "),
        );
        assert_eq!(bearer_token(&h).as_deref(), Some("xyz"));
    }

    #[test]
    fn bearer_token_absent_without_header() {
        assert!(bearer_token(&HeaderMap::new()).is_none());
    }

    #[test]
    fn native_auth_allows_only_the_tauri_origins_by_default() {
        let cfg = NativeAuthConfig::default();
        for origin in TAURI_ORIGINS {
            assert!(cfg.allows(origin), "{origin}");
        }
        for origin in [
            "https://evil.example",
            "http://tauri.localhost.evil.example",
            "http://tauri.localhost:8080",
            "tauri://evil",
            "TAURI://LOCALHOST",
            "",
        ] {
            assert!(!cfg.allows(origin), "{origin}");
        }
    }

    #[test]
    fn native_auth_adds_the_configured_origins() {
        let cfg = NativeAuthConfig {
            extra_origins: vec!["app://mailwoman".into()],
        };
        assert!(cfg.allows("app://mailwoman"));
        assert!(cfg.allows("tauri://localhost"));
        assert!(!cfg.allows("https://evil.example"));
    }

    #[test]
    fn token_hash_is_stable_and_not_the_token() {
        let h1 = hash_token("secret-token");
        let h2 = hash_token("secret-token");
        assert_eq!(h1, h2);
        assert_ne!(h1, "secret-token");
        assert_eq!(h1.len(), 64); // SHA-256 hex
    }

    #[test]
    fn generated_vapid_public_is_uncompressed_p256() {
        let (public_b64, scalar) = generate_vapid_keypair().unwrap();
        let raw = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(&public_b64)
            .unwrap();
        // Uncompressed SEC1 point: 0x04 || X(32) || Y(32).
        assert_eq!(raw.len(), 65);
        assert_eq!(raw[0], 0x04);
        // The stored private scalar is the raw 32-byte EC key the VAPID signer loads.
        assert_eq!(scalar.len(), 32);
        assert!(p256::ecdsa::SigningKey::from_slice(&scalar).is_ok());
    }

    #[test]
    fn encryption_matches_the_rfc_8291_example() {
        // RFC 8291 Appendix A: fixed sender key, salt and record size 4096 give this
        // exact message body.
        let d = |v: &str| {
            base64::engine::general_purpose::URL_SAFE_NO_PAD
                .decode(v)
                .unwrap()
        };
        let as_secret =
            p256::SecretKey::from_slice(&d("yfWPiYE-n46HLnH0KqZOF1fJJU3MYrct3AELtAQ-oRw")).unwrap();
        let ua_public = p256::PublicKey::from_sec1_bytes(&d(
            "BCVxsr7N_eNgVRqvHtD0zTZsEc6-VV-JvLexhqUzORcxaOzi6-AYWXvTBHm4bjyPjs7Vd8pZGH6SRpkNtoIAiw4",
        ))
        .unwrap();
        let ua_secret =
            p256::SecretKey::from_slice(&d("q1dXpw3UpT5VOmu_cf_v6ih07Aems3njxI-JWgLcM94")).unwrap();
        let auth = d("BTBZMqHH6r4Tts7J_aSIgg");
        let salt: [u8; 16] = d("DGv6ra1nlYgDCS1FRnbzlw").try_into().unwrap();
        let plaintext = b"When I grow up, I want to be a watermelon";
        let body = encrypt_aes128gcm(plaintext, &as_secret, &ua_public, &auth, &salt).unwrap();
        assert_eq!(
            body,
            d(
                "DGv6ra1nlYgDCS1FRnbzlwAAEABBBP4z9KsN6nGRTbVYI_c7VJSPQTBtkgcy27mlmlMoZIIgDll6e3vCYLocInmYWAmS6TlzAC8wEqKK6PBru3jl7A_yl95bQpu6cVPTpK4Mqgkf1CXztLVBSt2Ks3oZwbuwXPXLWyouBWLVWGNWQexSgSxsj_Qulcy4a-fN"
            )
        );
        assert_eq!(decrypt_wake(&body, &ua_secret, &auth), plaintext);
    }

    #[test]
    fn quiet_window_parsing_and_membership() {
        assert_eq!(parse_quiet_window("22-7"), Some((22, 7)));
        assert_eq!(parse_quiet_window(" 9 - 17 "), Some((9, 17)));
        assert_eq!(parse_quiet_window("nonsense"), None);
        assert_eq!(parse_quiet_window("25-7"), None);
        // Wrapping window 22:00–07:00.
        assert!(in_quiet_window(22, 7, 23));
        assert!(in_quiet_window(22, 7, 3));
        assert!(!in_quiet_window(22, 7, 12));
        // Same-day window 09:00–17:00.
        assert!(in_quiet_window(9, 17, 12));
        assert!(!in_quiet_window(9, 17, 8));
        assert!(!in_quiet_window(9, 17, 17));
    }

    // ── Receiver-side checks (written from RFC 8291 / RFC 8292, not from the
    // sender code): decrypt the body with the subscriber's key and verify the VAPID
    // JWT with the server's public key.

    use aes_gcm::aead::{Aead, KeyInit};
    use hkdf::Hkdf;
    use p256::elliptic_curve::sec1::ToSec1Point;

    const CONTACT: &str = "mailto:ops@example.test";

    fn ua_keypair() -> (p256::SecretKey, String) {
        let secret = loop {
            let mut b = [0u8; 32];
            rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut b);
            if let Ok(sk) = p256::SecretKey::from_slice(&b) {
                break sk;
            }
        };
        let public = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(secret.public_key().to_sec1_point(false).as_bytes());
        (secret, public)
    }

    fn b64(s: &str) -> Vec<u8> {
        b64url_decode(s).unwrap()
    }

    /// RFC 8291 §3.4 + RFC 8188 §2: one aes128gcm record, last-record delimiter 0x02.
    fn decrypt_wake(body: &[u8], ua_secret: &p256::SecretKey, auth: &[u8]) -> Vec<u8> {
        let salt = &body[..16];
        let rs = u32::from_be_bytes(body[16..20].try_into().unwrap()) as usize;
        let idlen = usize::from(body[20]);
        let keyid = &body[21..21 + idlen];
        let record = &body[21 + idlen..];
        assert!(record.len() <= rs, "one record of at most rs bytes");
        let as_public = p256::PublicKey::from_sec1_bytes(keyid).unwrap();
        let shared =
            p256::ecdh::diffie_hellman(ua_secret.to_nonzero_scalar(), as_public.as_affine());
        let mut key_info = b"WebPush: info\0".to_vec();
        key_info.extend_from_slice(ua_secret.public_key().to_sec1_point(false).as_bytes());
        key_info.extend_from_slice(keyid);
        let mut ikm = [0u8; 32];
        Hkdf::<Sha256>::new(Some(auth), shared.raw_secret_bytes().as_slice())
            .expand(&key_info, &mut ikm)
            .unwrap();
        let hk = Hkdf::<Sha256>::new(Some(salt), &ikm);
        let mut cek = [0u8; 16];
        hk.expand(b"Content-Encoding: aes128gcm\0", &mut cek)
            .unwrap();
        let mut nonce = [0u8; 12];
        hk.expand(b"Content-Encoding: nonce\0", &mut nonce).unwrap();
        let cipher = aes_gcm::Aes128Gcm::new_from_slice(&cek).unwrap();
        let mut plain = cipher.decrypt(&nonce.into(), record).unwrap();
        while plain.last() == Some(&0) {
            plain.pop();
        }
        assert_eq!(plain.pop(), Some(2), "last-record delimiter");
        plain
    }

    struct Vapid {
        header: serde_json::Value,
        claims: serde_json::Value,
        key: String,
    }

    /// RFC 8292 §3: `Authorization: vapid t=<JWT>, k=<public key>`; the JWT is
    /// ES256 (raw r||s) and must verify with `k`.
    fn verify_vapid(req: &axum::http::Request<Vec<u8>>) -> Vapid {
        use p256::ecdsa::signature::Verifier;
        let value = req.headers()[header::AUTHORIZATION].to_str().unwrap();
        let params = value.strip_prefix("vapid ").expect("vapid scheme");
        let mut token = None;
        let mut key = None;
        for part in params.split(',') {
            let part = part.trim();
            if let Some(v) = part.strip_prefix("t=") {
                token = Some(v);
            } else if let Some(v) = part.strip_prefix("k=") {
                key = Some(v);
            }
        }
        let (token, key) = (token.expect("t="), key.expect("k="));
        let (signing_input, sig) = token.rsplit_once('.').unwrap();
        let (head, claims) = signing_input.split_once('.').unwrap();
        let vk = p256::ecdsa::VerifyingKey::from_sec1_bytes(&b64(key)).unwrap();
        let sig = p256::ecdsa::Signature::from_slice(&b64(sig)).unwrap();
        vk.verify(signing_input.as_bytes(), &sig)
            .expect("VAPID JWT verifies with k");
        Vapid {
            header: serde_json::from_slice(&b64(head)).unwrap(),
            claims: serde_json::from_slice(&b64(claims)).unwrap(),
            key: key.to_string(),
        }
    }

    fn now_secs() -> i64 {
        chrono::Utc::now().timestamp()
    }

    fn auth_b64(byte: u8) -> String {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([byte; 16])
    }

    #[test]
    fn wake_request_decrypts_to_the_marker_with_the_subscriber_key() {
        let (vapid_public, scalar) = generate_vapid_keypair().unwrap();
        let (ua_secret, ua_public) = ua_keypair();
        let req = build_wake_request(
            "https://push.example/sub/1",
            &ua_public,
            &auth_b64(9),
            &scalar,
            CONTACT,
            WAKE_MARKER,
        )
        .unwrap();
        assert_eq!(req.method(), axum::http::Method::POST);
        assert_eq!(req.uri(), "https://push.example/sub/1");
        assert_eq!(req.headers()[header::CONTENT_ENCODING], "aes128gcm");
        assert_eq!(req.headers()["TTL"], WAKE_TTL.to_string().as_str());
        assert_eq!(
            decrypt_wake(req.body(), &ua_secret, &[9u8; 16]),
            WAKE_MARKER
        );
        // The JWT is signed with the stored VAPID key, and `k` is its public half.
        assert_eq!(verify_vapid(&req).key, vapid_public);
    }

    #[test]
    fn vapid_jwt_is_es256_and_expires_within_24_hours() {
        // RFC 8292 §2: "An exp claim MUST NOT be more than 24 hours from the time of
        // the request." The 4-week TTL header is a separate thing.
        let (_, scalar) = generate_vapid_keypair().unwrap();
        let (_, ua_public) = ua_keypair();
        let before = now_secs();
        let req = build_wake_request(
            "https://push.example/sub/1",
            &ua_public,
            &auth_b64(1),
            &scalar,
            CONTACT,
            WAKE_MARKER,
        )
        .unwrap();
        let vapid = verify_vapid(&req);
        assert_eq!(vapid.header["alg"], "ES256");
        assert_eq!(vapid.header["typ"], "JWT");
        assert_eq!(vapid.claims["sub"], CONTACT);
        let exp = vapid.claims["exp"].as_i64().expect("numeric exp");
        assert!(exp > before, "exp in the future");
        assert!(
            exp <= now_secs() + 24 * 60 * 60,
            "exp {exp} is more than 24 h after {before}"
        );
    }

    #[test]
    fn vapid_audience_is_the_endpoint_origin_including_a_port() {
        // RFC 8292 §2: aud is the origin of the push resource URL (RFC 6454
        // serialisation: scheme, host, and the port when it is not the default).
        let (_, scalar) = generate_vapid_keypair().unwrap();
        let (_, ua_public) = ua_keypair();
        for (endpoint, aud) in [
            ("https://push.example/wake/abc", "https://push.example"),
            ("https://push.example:443/wake/abc", "https://push.example"),
            (
                "https://push.example:8443/wake/abc",
                "https://push.example:8443",
            ),
            ("http://127.0.0.1:40850/push/1", "http://127.0.0.1:40850"),
        ] {
            let req = build_wake_request(
                endpoint,
                &ua_public,
                &auth_b64(2),
                &scalar,
                CONTACT,
                WAKE_MARKER,
            )
            .unwrap();
            assert_eq!(verify_vapid(&req).claims["aud"], aud, "{endpoint}");
        }
    }

    #[test]
    fn wake_request_refuses_bad_subscription_keys() {
        let (_, scalar) = generate_vapid_keypair().unwrap();
        let (_, ua_public) = ua_keypair();
        let short_auth = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([3u8; 8]);
        let ep = "https://push.example/x";
        let a = auth_b64(3);
        assert!(build_wake_request(ep, "not-a-point", &a, &scalar, CONTACT, b"x").is_err());
        assert!(build_wake_request(ep, &ua_public, &short_auth, &scalar, CONTACT, b"x").is_err());
        assert!(build_wake_request(ep, &ua_public, &a, &[0u8; 32], CONTACT, b"x").is_err());
        assert!(build_wake_request("not a uri", &ua_public, &a, &scalar, CONTACT, b"x").is_err());
        assert!(
            build_wake_request(
                "ftp://push.example/x",
                &ua_public,
                &a,
                &scalar,
                CONTACT,
                b"x"
            )
            .is_err()
        );
        assert!(build_wake_request("/relative", &ua_public, &a, &scalar, CONTACT, b"x").is_err());
    }

    // ── Coordinator conditions for the hand-built Web Push crypto (f25).

    fn subscriber() -> (p256::SecretKey, p256::PublicKey) {
        let secret = random_p256_secret();
        let public = secret.public_key();
        (secret, public)
    }

    #[test]
    fn two_encryptions_of_the_same_body_differ() {
        // Condition 1: a fresh ephemeral key and salt per message.
        let (_, scalar) = generate_vapid_keypair().unwrap();
        let (ua_secret, ua_public) = ua_keypair();
        let build = || {
            build_wake_request(
                "https://push.example/sub/1",
                &ua_public,
                &auth_b64(4),
                &scalar,
                CONTACT,
                WAKE_MARKER,
            )
            .unwrap()
            .into_body()
        };
        let (a, b) = (build(), build());
        assert_ne!(a[..16], b[..16], "salt");
        assert_ne!(a[21..86], b[21..86], "ephemeral public key");
        assert_ne!(a[86..], b[86..], "ciphertext");
        assert_eq!(decrypt_wake(&a, &ua_secret, &[4u8; 16]), WAKE_MARKER);
        assert_eq!(decrypt_wake(&b, &ua_secret, &[4u8; 16]), WAKE_MARKER);
    }

    #[test]
    fn body_is_one_4096_record_with_the_0x02_delimiter_and_no_padding() {
        // Condition 3: rs = 4096, keyid = the 65-byte ephemeral key, one record of
        // payload + 0x02 + 16-byte tag (decrypt_wake checks the delimiter).
        let (ua_secret, ua_public) = subscriber();
        let body = encrypt_aes128gcm(
            b"hello",
            &random_p256_secret(),
            &ua_public,
            &[5u8; 16],
            &[6u8; 16],
        )
        .unwrap();
        assert_eq!(&body[..16], &[6u8; 16]);
        assert_eq!(u32::from_be_bytes(body[16..20].try_into().unwrap()), 4096);
        assert_eq!(body[20], 65);
        assert_eq!(body[21], 0x04);
        assert_eq!(body.len(), 86 + b"hello".len() + 1 + 16);
        assert_eq!(decrypt_wake(&body, &ua_secret, &[5u8; 16]), b"hello");
    }

    #[test]
    fn a_body_over_4096_bytes_is_refused_with_a_clear_error() {
        // Condition 3: push services take at most 4096 bytes of message body
        // (RFC 8291 §4): 86 header bytes + payload + 0x02 + 16-byte tag.
        let (ua_secret, ua_public) = subscriber();
        let k = random_p256_secret();
        let largest = vec![7u8; 4096 - 86 - 17];
        let body = encrypt_aes128gcm(&largest, &k, &ua_public, &[1u8; 16], &[2u8; 16]).unwrap();
        assert_eq!(body.len(), 4096);
        assert_eq!(decrypt_wake(&body, &ua_secret, &[1u8; 16]), largest);
        let over = vec![7u8; largest.len() + 1];
        let err = encrypt_aes128gcm(&over, &k, &ua_public, &[1u8; 16], &[2u8; 16])
            .unwrap_err()
            .to_string();
        assert!(err.contains("4096"), "{err}");
        assert!(err.contains(&over.len().to_string()), "{err}");
    }

    #[test]
    fn vapid_signature_is_raw_r_s_not_der() {
        // Condition 4: JWS ES256 is the 64-byte r ‖ s, not an ASN.1 DER sequence.
        let (_, scalar) = generate_vapid_keypair().unwrap();
        let (_, ua_public) = ua_keypair();
        let req = build_wake_request(
            "https://push.example/x",
            &ua_public,
            &auth_b64(8),
            &scalar,
            CONTACT,
            WAKE_MARKER,
        )
        .unwrap();
        let value = req.headers()[header::AUTHORIZATION].to_str().unwrap();
        let token = value
            .strip_prefix("vapid t=")
            .unwrap()
            .split(", k=")
            .next()
            .unwrap();
        let sig = b64(token.rsplit('.').next().unwrap());
        assert_eq!(sig.len(), 64);
        assert!(p256::ecdsa::DerSignature::try_from(sig.as_slice()).is_err());
    }

    #[test]
    fn vapid_authorization_header_has_the_rfc_8292_form() {
        // Condition 4: `vapid t=<b64url>.<b64url>.<b64url>, k=<b64url 65-byte point>`,
        // JWT header {"typ":"JWT","alg":"ES256"}, claims exactly aud, exp, sub.
        let (vapid_public, scalar) = generate_vapid_keypair().unwrap();
        let (_, ua_public) = ua_keypair();
        let before = now_secs();
        let req = build_wake_request(
            "https://push.example:8443/x",
            &ua_public,
            &auth_b64(9),
            &scalar,
            CONTACT,
            WAKE_MARKER,
        )
        .unwrap();
        let after = now_secs();
        let value = req.headers()[header::AUTHORIZATION].to_str().unwrap();
        let rest = value
            .strip_prefix("vapid t=")
            .expect("starts with `vapid t=`");
        let (token, key) = rest.split_once(", k=").expect("`, k=` separator");
        let b64url = |s: &str| {
            !s.is_empty()
                && s.bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        };
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3, "{token}");
        assert!(parts.iter().all(|p| b64url(p)), "{token}");
        assert!(b64url(key), "{key}");
        assert_eq!(key, vapid_public);
        let point = b64(key);
        assert_eq!((point.len(), point[0]), (65, 0x04));
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&b64(parts[0])).unwrap(),
            json!({ "typ": "JWT", "alg": "ES256" })
        );
        let claims: serde_json::Map<String, serde_json::Value> =
            serde_json::from_slice(&b64(parts[1])).unwrap();
        let mut names: Vec<&str> = claims.keys().map(String::as_str).collect();
        names.sort_unstable();
        assert_eq!(names, ["aud", "exp", "sub"]);
        assert_eq!(claims["aud"], "https://push.example:8443");
        assert_eq!(claims["sub"], CONTACT);
        // 12 h after the request.
        let exp = claims["exp"].as_i64().unwrap();
        assert!(
            exp >= before + 12 * 3600 && exp <= after + 12 * 3600,
            "{exp}"
        );
    }

    #[test]
    fn vapid_contact_is_unchanged() {
        // Condition 4: `sub` is MW_VAPID_CONTACT, else the old neutral default.
        assert_eq!(vapid_contact_from(None), "mailto:push@mailwoman.invalid");
        assert_eq!(
            vapid_contact_from(Some(String::new())),
            "mailto:push@mailwoman.invalid"
        );
        assert_eq!(
            vapid_contact_from(Some("mailto:ops@example.org".into())),
            "mailto:ops@example.org"
        );
    }

    #[test]
    fn vapid_private_key_is_held_in_zeroizing_buffers() {
        // Condition 5: the generated scalar and the one loaded for dispatch are
        // `Zeroizing`, so they are wiped on drop (p256::SecretKey and SigningKey
        // zeroize themselves).
        let (_, scalar): (String, Zeroizing<Vec<u8>>) = generate_vapid_keypair().unwrap();
        let loaded: Option<(String, Zeroizing<Vec<u8>>)> =
            vapid_for_dispatch(Some(("pub".into(), scalar.to_vec())));
        assert_eq!(loaded.unwrap().1.len(), 32);
    }

    #[tokio::test]
    async fn send_errors_do_not_carry_the_endpoint_url() {
        // Condition 5: endpoints are capability URLs; a failed send is logged, so
        // its error must not contain the URL. The local listener closes every
        // connection without answering.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            while let Ok((conn, _)) = listener.accept().await {
                drop(conn);
            }
        });
        let (_, scalar) = generate_vapid_keypair().unwrap();
        let (_, ua_public) = ua_keypair();
        let sub = |transport: &str| PushSubscriptionRow {
            id: "sub-1".into(),
            account_id: "acct".into(),
            transport: transport.into(),
            endpoint: format!("http://127.0.0.1:{port}/capability-secret-7f3a"),
            p256dh: Some(ua_public.clone()),
            auth: Some(auth_b64(1)),
            app_id: None,
            expires_at: None,
            created_at: now_rfc3339(),
            last_wake_at: None,
        };
        let errors = [
            send_unifiedpush(None, &sub("unifiedpush"))
                .await
                .unwrap_err(),
            send_webpush(None, &sub("webpush"), Some(scalar.as_slice()))
                .await
                .unwrap_err(),
        ];
        for e in errors {
            let text = format!("{e:#}");
            assert!(!text.contains("capability-secret-7f3a"), "{text}");
            assert!(!text.contains(&format!("127.0.0.1:{port}")), "{text}");
        }
    }

    #[test]
    fn push_logs_name_no_endpoint_key_or_token() {
        // Condition 5: no tracing call in this module formats an endpoint, a key,
        // the JWT or the Authorization header (message text may name them).
        let src = include_str!("push_relay.rs");
        let code = &src[..src.find("#[cfg(test)]").unwrap()];
        let mut rest = code;
        while let Some(at) = rest.find("tracing::") {
            let call = tracing_call(&rest[at..]);
            let values = formatted_values(call).to_ascii_lowercase();
            for word in [
                "endpoint",
                "scalar",
                "private",
                "authorization",
                "token",
                "jwt",
                "p256dh",
                "auth",
            ] {
                assert!(
                    !values.contains(word),
                    "tracing call formats `{word}`: {call}"
                );
            }
            rest = &rest[at + call.len()..];
        }
    }

    // A tracing macro can end a match arm with a comma, without a semicolon.
    // Stop at its matching parenthesis so nearby code/comments are not treated
    // as values included in the log message.
    fn tracing_call(source: &str) -> &str {
        let start = source.find('(').expect("tracing macro arguments");
        let (mut depth, mut quoted, mut escaped) = (0usize, false, false);
        for (offset, ch) in source[start..].char_indices() {
            if quoted {
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    quoted = false;
                }
                continue;
            }
            match ch {
                '"' => quoted = true,
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        return &source[..start + offset + 1];
                    }
                }
                _ => {}
            }
        }
        panic!("unterminated tracing macro")
    }

    #[test]
    fn tracing_scanner_stops_at_match_arm_and_keeps_nested_values() {
        let call = r#"tracing::warn!("failed ({})", failure.kind()), next => endpoint(secret);"#;
        assert_eq!(
            tracing_call(call),
            r#"tracing::warn!("failed ({})", failure.kind())"#
        );
        let sensitive = r#"tracing::warn!("failed: {}", endpoint(secret)),"#;
        assert!(formatted_values(tracing_call(sensitive)).contains("endpoint(secret)"));
    }

    /// `call` with string-literal text blanked, keeping `{…}` captures inside
    /// literals and everything outside them: the values the call formats.
    fn formatted_values(call: &str) -> String {
        let (mut out, mut in_str, mut in_capture) = (String::new(), false, false);
        for ch in call.chars() {
            match (in_str, ch) {
                (false, '"') => in_str = true,
                (true, '"') => in_str = false,
                (true, '{') => in_capture = true,
                (true, '}') => {
                    in_capture = false;
                    out.push(' ');
                }
                (true, c) if in_capture => out.push(c),
                (true, _) => {}
                (false, c) => out.push(c),
            }
        }
        out
    }
}
