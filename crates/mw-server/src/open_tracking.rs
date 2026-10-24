//! Open tracking (26.20 t29-e8): opt-in, per message, off unless the sender asks.
//!
//! The sender's client mints a token before sending, puts a 1x1 image pointing at
//! `<MW_PUBLIC_URL><MW_BASE_PATH>/t/<token>.gif` into the outgoing HTML, sends, and
//! binds the sent email id to the token. When a mail program loads the image, the
//! time is recorded. The sender later reads how many loads there were and when the
//! first and last were.
//!
//! What is recorded is the time and nothing else: the image route reads no address,
//! user agent or header, so the sender learns nothing about the reader beyond "the
//! image was loaded at T". This server fetches nothing for this feature — it only
//! answers the image request — so no SSRF or egress rule is involved.
//!
//! Limits the UI has to state: many mail programs block remote images, and some load
//! them in advance through a proxy, so a recorded open does not prove the message was
//! read and no record does not prove it was not. The recipient is not told.
//!
//! ## Routes
//! * `POST /api/track/open` (session; `{subject, to}`) → `{token, url}`, or 409
//!   `{"error":"publicUrlRequired"}` when the feature is unavailable (see [`availability`]).
//! * `PUT /api/track/open/{token}` (session, owner only; `{emailId}`) → 204; 404 for a
//!   token that is not the caller's; 409 `alreadyBound` for a token bound to another id.
//! * `GET /api/track/open?emailId=` (session, account-scoped) → `{count, first, last}`;
//!   404 when the account has no token bound to that id.
//! * `GET /t/{token}.gif` — no session. Always the same 43-byte GIF with
//!   `Cache-Control: no-store, private`, whether or not the token exists, so the
//!   route cannot be used to test tokens. Loads beyond [`BURST`] per token are served
//!   but not recorded until the bucket refills.
//!
//! ## The on/off switch
//! [`availability`] is the only place that decides whether open tracking is offered.
//! Today it requires `MW_PUBLIC_URL`: the image URL must be absolute and must not be
//! derived from a request's `Host`, which is what made the removed feature fail. The
//! mint route, the image route and `/api/policy`'s `openTracking` all go through it.

use std::sync::{Mutex, OnceLock};

use axum::Json;
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, put};
use base64::Engine as _;
use rand::RngCore;
use serde::Deserialize;
use serde_json::json;

use mw_store::{OpenBind, open_token_hash};

use crate::scope_mw::rate_limit::KeyedRateLimiter;
use crate::{AppState, ExternalBase};

/// The image: a transparent 1x1 GIF89a, 43 bytes.
pub(crate) const PIXEL_GIF: [u8; 43] = [
    0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xff, 0xff, 0xff, 0x21, 0xf9, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x2c, 0x00, 0x00, 0x00, 0x00,
    0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00, 0x3b,
];

/// Random bytes in a token (128 bits), base64url without padding = 22 characters.
const TOKEN_BYTES: usize = 16;
const TOKEN_CHARS: usize = 22;

/// Loads one token may record at once, and the sustained refill (one per 6 s).
const BURST: f64 = 10.0;
const REFILL_PER_SEC: f64 = 1.0 / 6.0;
/// Retained token keys. Bounded because the image route is unauthenticated and the
/// path is caller-chosen; see [`crate::scope_mw::rate_limit`].
const KEY_CAPACITY: usize = 10_000;

/// Longest subject kept, in characters (RFC 5322's line limit).
const MAX_SUBJECT_CHARS: usize = 998;
/// Most recipients kept for one token.
const MAX_RECIPIENTS: usize = 500;

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/track/open", get(summary).post(mint))
        .route("/api/track/open/{token}", put(bind))
        .route("/t/{file}", get(pixel))
}

/// Why open tracking is not offered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Unavailable {
    /// `MW_PUBLIC_URL` is not set, so there is no absolute URL for the image.
    PublicUrlRequired,
}

impl Unavailable {
    fn code(self) -> &'static str {
        match self {
            Unavailable::PublicUrlRequired => "publicUrlRequired",
        }
    }
}

/// Whether open tracking is offered, and if so the absolute base the image URL is
/// built on (`scheme://host[:port]` plus `MW_BASE_PATH`). The one on/off switch for
/// the feature; an admin setting belongs here.
pub(crate) async fn availability(state: &AppState) -> Result<String, Unavailable> {
    let origin = state
        .external
        .public_origin()
        .ok_or(Unavailable::PublicUrlRequired)?;
    Ok(format!("{origin}{}", state.base_path))
}

/// What `/api/policy` reports as `openTracking`.
pub(crate) async fn enabled(state: &AppState) -> bool {
    availability(state).await.is_ok()
}

impl ExternalBase {
    /// `scheme://host[:port]` from `MW_PUBLIC_URL`, and only from it: never from a
    /// request header.
    pub(crate) fn public_origin(&self) -> Option<String> {
        let host = self.public_host.as_deref()?;
        let scheme = if self.public_https? { "https" } else { "http" };
        Some(format!("{scheme}://{host}"))
    }
}

fn new_token() -> String {
    let mut b = [0u8; TOKEN_BYTES];
    rand::thread_rng().fill_bytes(&mut b);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b)
}

/// A string shaped like a token this server mints. Anything else is never looked up.
fn token_shaped(s: &str) -> bool {
    s.len() == TOKEN_CHARS
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn error(status: StatusCode, code: &str) -> Response {
    (status, Json(json!({ "error": code }))).into_response()
}

fn store_failed(route: &str, e: mw_store::StoreError) -> Response {
    tracing::error!("{route}: {e}");
    error(StatusCode::INTERNAL_SERVER_ERROR, "storeUnavailable")
}

#[derive(Debug, Deserialize)]
struct MintReq {
    #[serde(default)]
    subject: String,
    #[serde(default)]
    to: Vec<String>,
}

/// `POST /api/track/open`.
async fn mint(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<MintReq>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let base = match availability(&state).await {
        Ok(b) => b,
        Err(why) => return error(StatusCode::CONFLICT, why.code()),
    };
    if req.subject.chars().count() > MAX_SUBJECT_CHARS || req.to.len() > MAX_RECIPIENTS {
        return error(StatusCode::BAD_REQUEST, "tooLarge");
    }
    let recipients = serde_json::to_string(&req.to).unwrap_or_else(|_| "[]".into());
    let token = new_token();
    if let Err(e) = state
        .store
        .create_open_token(
            &open_token_hash(&token),
            &session.account_id,
            &req.subject,
            &recipients,
            chrono::Utc::now(),
        )
        .await
    {
        return store_failed("POST /api/track/open", e);
    }
    let url = format!("{base}/t/{token}.gif");
    (
        [(header::CACHE_CONTROL, "no-store")],
        Json(json!({ "token": token, "url": url })),
    )
        .into_response()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BindReq {
    email_id: String,
}

/// `PUT /api/track/open/{token}`.
async fn bind(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(token): Path<String>,
    Json(req): Json<BindReq>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    if req.email_id.trim().is_empty() {
        return error(StatusCode::BAD_REQUEST, "emailIdRequired");
    }
    if !token_shaped(&token) {
        return error(StatusCode::NOT_FOUND, "notFound");
    }
    match state
        .store
        .bind_open_token(&session.account_id, &open_token_hash(&token), &req.email_id)
        .await
    {
        Ok(OpenBind::Bound) => StatusCode::NO_CONTENT.into_response(),
        Ok(OpenBind::NotFound) => error(StatusCode::NOT_FOUND, "notFound"),
        Ok(OpenBind::AlreadyBound) => error(StatusCode::CONFLICT, "alreadyBound"),
        Err(e) => store_failed("PUT /api/track/open/{token}", e),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SummaryQuery {
    #[serde(default)]
    email_id: String,
}

/// `GET /api/track/open?emailId=`.
async fn summary(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<SummaryQuery>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    if q.email_id.is_empty() {
        return error(StatusCode::BAD_REQUEST, "emailIdRequired");
    }
    match state
        .store
        .open_summary(&session.account_id, &q.email_id)
        .await
    {
        Ok(Some(s)) => (
            [(header::CACHE_CONTROL, "no-store")],
            Json(json!({ "count": s.count, "first": s.first, "last": s.last })),
        )
            .into_response(),
        Ok(None) => error(StatusCode::NOT_FOUND, "notFound"),
        Err(e) => store_failed("GET /api/track/open", e),
    }
}

fn per_token() -> &'static Mutex<KeyedRateLimiter> {
    static RL: OnceLock<Mutex<KeyedRateLimiter>> = OnceLock::new();
    RL.get_or_init(|| Mutex::new(KeyedRateLimiter::new(BURST, REFILL_PER_SEC, KEY_CAPACITY)))
}

/// `GET /t/{token}.gif`. Takes no peer address and no headers: there is nothing
/// about the reader here to record.
async fn pixel(State(state): State<AppState>, Path(file): Path<String>) -> Response {
    if let Some(token) = file.strip_suffix(".gif")
        && token_shaped(token)
        && enabled(&state).await
    {
        let hash = open_token_hash(token);
        let allowed = per_token()
            .lock()
            .map(|mut rl| rl.check(&hash))
            .unwrap_or(false);
        if allowed && let Err(e) = state.store.record_open(&hash, chrono::Utc::now()).await {
            tracing::warn!("open tracking: could not record a load: {e}");
        }
    }
    pixel_response()
}

fn pixel_response() -> Response {
    let mut resp = PIXEL_GIF.into_response();
    let h = resp.headers_mut();
    h.insert(header::CONTENT_TYPE, HeaderValue::from_static("image/gif"));
    h.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, private"),
    );
    // The image is loaded by mail programs on other origins. `security_headers`
    // keeps a handler's own value instead of its `same-origin` default, which a
    // browser would answer by refusing the image.
    h.insert(
        "cross-origin-resource-policy",
        HeaderValue::from_static("cross-origin"),
    );
    resp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_128_bits_of_base64url() {
        let t = new_token();
        assert_eq!(t.len(), TOKEN_CHARS);
        assert!(token_shaped(&t));
        assert_ne!(t, new_token());
    }

    #[test]
    fn only_token_shaped_paths_are_looked_up() {
        assert!(!token_shaped(""));
        assert!(!token_shaped("short"));
        assert!(!token_shaped("aaaaaaaaaaaaaaaaaaaaa/"));
        assert!(!token_shaped("aaaaaaaaaaaaaaaaaaaaaaa"));
        assert!(token_shaped("aaaaaaaaaaaaaaaaaaaa-_"));
    }

    #[test]
    fn pixel_is_a_43_byte_gif() {
        assert_eq!(PIXEL_GIF.len(), 43);
        assert_eq!(&PIXEL_GIF[..6], b"GIF89a");
        assert_eq!(PIXEL_GIF[42], 0x3b);
    }

    #[test]
    fn public_origin_comes_only_from_public_url() {
        let none = ExternalBase::default();
        assert_eq!(none.public_origin(), None);
        let set = ExternalBase {
            public_https: Some(true),
            public_host: Some("mail.example.org:8443".into()),
            ..ExternalBase::default()
        };
        assert_eq!(
            set.public_origin().as_deref(),
            Some("https://mail.example.org:8443")
        );
    }
}
