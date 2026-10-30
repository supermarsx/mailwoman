//! Signed remote-content URLs (t32 e31a, plan R + M10).
//!
//! # Why signed URLs
//! A message body renders in a `sandbox=""` frame: an opaque origin, so the
//! `SameSite=Strict` session cookie is never sent from it, and the cookie-authed
//! `/api/image-proxy` answered `401` there (D-7). The reader therefore asks for
//! URLs that carry their own authorisation: `POST /api/rc/sign` (cookie-authed,
//! the same grant check as the image proxy) returns paths of the form
//! `/api/rc/i/<token>` (a remote image) and `/api/rc/p/<token>` (an inline
//! `cid:` part of the message), which the frame can load without a cookie, and
//! `/api/rc/f/<token>` for "download through the server" of a remote file linked
//! from the message (always a download, never rendered).
//!
//! # The token
//! `base64url(claims) "." base64url(HMAC-SHA256(key, "mw-rc-token\0" ‖ payload))`,
//! with the key derived from the server key under the label
//! `mw-remote-content-v1` (`mw_store::Store::derive_key`). The claims bind the
//! account, the session (as an HMAC tag of its id, never the id itself), the
//! message, the URL or blob, the kind, and an expiry: one hour for a remote image,
//! five minutes for a part or a linked file (M10). Nothing in a token can be changed without the
//! key, so a token cannot be pointed at another URL, message or account.
//!
//! # What is checked when a token is used, in this order
//! 1. The tag, the kind and the expiry — a tampered, retargeted or expired token
//!    is a `403` before anything else happens: no database read, no DNS.
//! 2. The session it was minted for still exists. Logging out, or an admin
//!    revoking the sessions, ends every token minted for them (M10's generation
//!    rule, kept by binding to the session row instead of a counter column).
//! 3. For a remote image, a grant still covers the message (a revoked grant stops
//!    loading at once), then the egress plan, the per-account rate limit, the
//!    response type allowlist with a magic-byte check, and the re-encode in the
//!    wasm media jail. SVG is refused (DA36). An image that decodes to 2×2 or
//!    smaller is answered with a transparent pixel and counted as a tracker.
//!
//! Responses carry `Cross-Origin-Resource-Policy: cross-origin`, because a
//! document with an opaque origin is cross-origin to everything and the shell's
//! default `same-origin` would block the image; the token is the authorisation,
//! not the origin of the requester.

use std::sync::atomic::{AtomicU64, Ordering};

use axum::Json;
use axum::extract::{Path as UrlPath, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Sha256;

use mw_egress::TrafficClass;
use mw_egress::plan::{self, Request};

use super::{
    CacheKey, DIRECT_ROUTE, audit_egress_transition, cache, etag_for, fetch_semaphore,
    if_none_match, internal, load_plan, not_modified, rate_limiter, refusal_response,
};
use crate::AppState;

/// The label the signing key is derived under.
pub(crate) const KEY_LABEL: &str = "mw-remote-content-v1";
/// How long a remote-image token is valid.
pub(crate) const IMAGE_TTL_SECS: i64 = 3600;
/// How long a part token is valid (M10).
pub(crate) const PART_TTL_SECS: i64 = 300;
/// Most URLs or parts one sign request may ask for.
const MAX_SIGN: usize = 256;
/// Longest URL that is signed.
const MAX_URL_LEN: usize = 4096;
/// Longest blob id that is signed.
const MAX_BLOB_ID_LEN: usize = 256;
const MAX_PART_RAW_BYTES: usize = 32 * 1024 * 1024;

/// Timeouts, concurrency caps, negative cache and load outcomes (t56).
#[path = "remote_fetch_limits.rs"]
pub(crate) mod fetch_limits;
#[path = "remote_image_flight.rs"]
pub(super) mod image_flight;
#[path = "remote_membership.rs"]
mod membership;
#[path = "remote_tracker_stats.rs"]
mod tracker_stats;
pub(crate) use fetch_limits::FetchLimits;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StatsQuery {
    email_id: String,
}

/// Recent decoded tracker replacements for this session's message, never a
/// cross-account aggregate. The bounded in-memory window resets on restart.
pub(crate) async fn message_stats(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<StatsQuery>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(session) => session,
        Err(response) => return response,
    };
    if query.email_id.is_empty() || query.email_id.len() > 512 {
        return (StatusCode::BAD_REQUEST, "invalid emailId").into_response();
    }
    match message_context(&state, &session, &query.email_id).await {
        Ok(Some(_)) => {}
        Ok(None) => return forbidden("message does not belong to this account"),
        Err(()) => return internal("remote-content message lookup"),
    }
    let key = state.store.derive_key(KEY_LABEL);
    let tag = authority_tag(&key, &session);
    let (count, limited) = tracker_stats::read(&session.account_id, &tag, &query.email_id);
    // t56 for t55's loading toast: distinct proxied images of this message, for
    // this session, that answered an image / failed (timeout, upstream failure,
    // non-success, too large, remembered failure or no free slot in time).
    let (loaded, failed, failed_refs, retryable_refs) =
        fetch_limits::outcome_details(&session.account_id, &tag, &query.email_id);
    (
        [(header::CACHE_CONTROL, "no-store")],
        Json(json!({
            "decodedTrackersReplaced": count,
            "limited": limited,
            "windowSecs": IMAGE_TTL_SECS,
            "imagesLoaded": loaded,
            "imagesFailed": failed,
            "failedImageRefs": failed_refs,
            "retryableImageRefs": retryable_refs,
        })),
    )
        .into_response()
}

async fn image_allowed(
    state: &AppState,
    session: &mw_store::Session,
    email: &str,
    policy: &crate::remote_policy::PrivacyPolicy,
) -> Result<bool, ()> {
    if policy.mode == "block" {
        return Ok(false);
    }
    let Some((sender, domain)) = message_context(state, session, email).await? else {
        return Ok(false);
    };
    if policy.mode != "ask" {
        return Ok(true);
    }
    state
        .store
        .remote_image_allowed(&session.account_id, email, &sender, &domain)
        .await
        .map_err(|_| ())
}

/// Proxy deployments do not own an engine cache. Verify their message identity
/// and sender with the authenticated, origin-pinned JMAP upstream.
async fn message_context(
    state: &AppState,
    session: &mw_store::Session,
    email: &str,
) -> Result<Option<(String, String)>, ()> {
    match state.store.get_message(email).await {
        Ok(message) if message.account_id == session.account_id => {
            return Ok(Some(super::sender_of(&state.store, email).await));
        }
        Ok(_) => return Ok(None),
        Err(mw_store::StoreError::NotFound) if state.engine.is_none() => {}
        Err(mw_store::StoreError::NotFound) => return Ok(None),
        Err(_) => return Err(()),
    }
    let Some(message) = message_metadata(state, session, email, &["id", "from"]).await? else {
        return Ok(None);
    };
    let sender = message["from"][0]["email"]
        .as_str()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    let domain = sender
        .rsplit_once('@')
        .map(|(_, domain)| domain.to_owned())
        .unwrap_or_default();
    Ok(Some((sender, domain)))
}

/// Only metadata belonging to the authenticated account/message is accepted.
pub(crate) async fn message_metadata(
    state: &AppState,
    session: &mw_store::Session,
    email: &str,
    properties: &[&str],
) -> Result<Option<serde_json::Value>, ()> {
    let request = json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Email/get",{"accountId":session.account_id,"ids":[email],"properties":properties,"bodyProperties":["blobId","cid","subParts","type","name","size"]},"privacy"]]});
    let _metadata_permit = fetch_semaphore().try_acquire().map_err(|_| ())?;
    let response = if let Some(engine) = &state.engine {
        engine.handle_jmap(&session.account_id, &request).await
    } else {
        let client = crate::upstream_client(
            &state.security,
            &session.jmap_url,
            &session.api_url,
            &session.credentials,
        )
        .await
        .map_err(|_| ())?;
        let (status, bytes) = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            client.request_raw_limited(
                &session.api_url,
                axum::body::Bytes::from(request.to_string()),
                256 * 1024,
            ),
        )
        .await
        .map_err(|_| ())?
        .map_err(|_| ())?;
        if !status.is_success() {
            return Err(());
        }
        serde_json::from_slice::<serde_json::Value>(&bytes).map_err(|_| ())?
    };
    let Some(result) = response["methodResponses"].as_array().and_then(|calls| {
        calls
            .iter()
            .find(|call| call[0] == "Email/get" && call[2] == "privacy")
    }) else {
        return Ok(None);
    };
    if result[1]["accountId"] != session.account_id {
        return Ok(None);
    }
    let Some(message) = result[1]["list"]
        .as_array()
        .and_then(|list| list.iter().find(|message| message["id"] == email))
    else {
        return Ok(None);
    };
    Ok(Some(message.clone()))
}

async fn part_membership(
    state: &AppState,
    session: &mw_store::Session,
    email: &str,
) -> Result<Vec<String>, ()> {
    if let Some(engine) = &state.engine {
        let _permit = fetch_semaphore().try_acquire().map_err(|_| ())?;
        return engine
            .inline_part_ids_limited(&session.account_id, email, MAX_PART_RAW_BYTES)
            .await
            .map_err(|_| ());
    }
    let properties = if state.engine.is_some() {
        vec!["id", "mailwomanInlineParts"]
    } else {
        vec!["id", "bodyStructure", "attachments"]
    };
    let Some(message) = message_metadata(state, session, email, &properties).await? else {
        return Ok(Vec::new());
    };
    fn collect(value: &serde_json::Value, out: &mut Vec<String>, depth: usize) {
        if depth > 32 || out.len() >= MAX_SIGN {
            return;
        }
        if let Some(parts) = value.as_array() {
            for part in parts {
                collect(part, out, depth + 1);
            }
        } else if value.is_object() {
            if value["cid"].as_str().is_some_and(|s| !s.is_empty())
                && let Some(blob) = value["blobId"].as_str()
            {
                out.push(blob.to_owned());
            }
            collect(&value["subParts"], out, depth + 1);
        }
    }
    let mut ids = Vec::new();
    for field in ["mailwomanInlineParts", "bodyStructure", "attachments"] {
        collect(&message[field], &mut ids, 0);
    }
    Ok(ids)
}

/// The image types a remote fetch may return (plan R).
const ALLOWED_IMAGE_TYPES: [&str; 4] = ["image/png", "image/jpeg", "image/gif", "image/webp"];

/// A 1×1 fully transparent PNG, served in place of a tracking pixel.
const TRANSPARENT_PIXEL: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0B, 0x49, 0x44, 0x41, 0x54, 0x78, 0xDA, 0x63, 0x60, 0x00, 0x02, 0x00,
    0x00, 0x05, 0x00, 0x01, 0xE9, 0xFA, 0xDC, 0xD8, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44,
    0xAE, 0x42, 0x60, 0x82,
];

/// Tracking pixels answered with [`TRANSPARENT_PIXEL`] since the process started.
/// Read by the privacy diagnostics that the admin Privacy screen lands with.
static TRACKERS_REPLACED: AtomicU64 = AtomicU64::new(0);

// ── tokens ─────────────────────────────────────────────────────────────────────

/// What a token is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Kind {
    /// A remote image, fetched through the egress plan.
    #[serde(rename = "i")]
    Image,
    /// An inline part of the message (`cid:`), read like a download.
    #[serde(rename = "p")]
    Part,
    /// A remote file linked from the message, fetched through the egress plan and
    /// served only as a download.
    #[serde(rename = "f")]
    File,
    /// A deliberately downloaded, decoded message image; never interchangeable with image display tokens.
    #[serde(rename = "di")]
    ImageDownload,
}

/// The signed claims. Short field names keep the URL short.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Claims {
    /// Format version.
    pub v: u8,
    /// Kind.
    pub k: Kind,
    /// Account id.
    pub a: String,
    /// Session tag ([`session_tag`]).
    pub s: String,
    /// Email id.
    pub e: String,
    /// Remote URL (image) or blob id (part).
    pub u: String,
    /// Expiry, unix seconds.
    pub x: i64,
    /// Cache nonce for an explicit retry; the standard signed expiry still applies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: Option<String>,
}

type HmacSha256 = Hmac<Sha256>;

fn mac(key: &[u8; 32], domain: &[u8], data: &[u8]) -> HmacSha256 {
    let mut m = HmacSha256::new_from_slice(key).expect("HMAC takes a key of any length");
    m.update(domain);
    m.update(data);
    m
}

/// The session's tag in a token: the first 16 bytes of an HMAC of its id, in hex.
/// The id itself is the session's secret and never goes into a URL.
pub(crate) fn session_tag(key: &[u8; 32], session_id: &str) -> String {
    let full = mac(key, b"mw-rc-session\0", session_id.as_bytes())
        .finalize()
        .into_bytes();
    full[..16].iter().map(|b| format!("{b:02x}")).collect()
}

fn authority_tag(key: &[u8; 32], session: &mw_store::Session) -> String {
    session_tag(
        key,
        &format!("{}\0{}\0{}", session.id, session.jmap_url, session.api_url),
    )
}

/// Mint a token for `claims`.
pub(crate) fn mint(key: &[u8; 32], claims: &Claims) -> String {
    let payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(claims).expect("claims serialize"));
    let tag = mac(key, b"mw-rc-token\0", payload.as_bytes())
        .finalize()
        .into_bytes();
    format!("{payload}.{}", URL_SAFE_NO_PAD.encode(tag))
}

/// Check a token's tag, version and expiry (and nothing that needs I/O). `Err`
/// for anything that is not a token this server minted and that is still valid.
pub(crate) fn verify(key: &[u8; 32], token: &str, now: i64) -> Result<Claims, ()> {
    let (payload, tag) = token.split_once('.').ok_or(())?;
    let tag = URL_SAFE_NO_PAD.decode(tag).map_err(|_| ())?;
    // `verify_slice` compares in constant time.
    mac(key, b"mw-rc-token\0", payload.as_bytes())
        .verify_slice(&tag)
        .map_err(|_| ())?;
    let bytes = URL_SAFE_NO_PAD.decode(payload).map_err(|_| ())?;
    let claims: Claims = serde_json::from_slice(&bytes).map_err(|_| ())?;
    if claims.v != 1 || claims.x < now {
        return Err(());
    }
    Ok(claims)
}

fn now_secs() -> i64 {
    chrono::Utc::now().timestamp()
}

fn signing_key(state: &AppState) -> [u8; 32] {
    state.store.derive_key(KEY_LABEL)
}

fn forbidden(msg: &'static str) -> Response {
    (StatusCode::FORBIDDEN, Json(json!({ "error": msg }))).into_response()
}

// ── POST /api/rc/sign ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SignReq {
    email_id: String,
    #[serde(default)]
    urls: Vec<String>,
    #[serde(default)]
    parts: Vec<String>,
    #[serde(default)]
    files: Vec<String>,
    #[serde(default)]
    image_downloads: Vec<String>,
    /// "Retry images" (t56): forget this account's remembered failures for the
    /// `urls` signed, so the new links fetch again instead of answering the
    /// cached failure.
    #[serde(default)]
    retry: bool,
}

/// `POST /api/rc/sign {emailId, urls?, parts?, files?}` — signed paths for the
/// reader.
///
/// Remote images need a grant covering the message (the image proxy's check);
/// trackers by URL (`mw_sanitize::remote::is_tracker_url`) are refused with the
/// reason `tracker`, and anything but an absolute http(s) URL with `scheme`.
/// Parts are the message's own inline parts and need only the session. A file is
/// signed when the reader asks to download a linked file through the server — a
/// deliberate click, so no grant is needed — and only for a message of the
/// session's own account.
pub(crate) async fn sign(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SignReq>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    if !body.image_downloads.is_empty()
        && headers
            .get("x-mailwoman-account")
            .and_then(|value| value.to_str().ok())
            != Some(session.account_id.as_str())
    {
        return (
            StatusCode::CONFLICT,
            Json(json!({"error":"accountChanged"})),
        )
            .into_response();
    }
    let policy = match crate::remote_policy::effective(&state, &session).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    if body.email_id.is_empty() || body.email_id.len() > 512 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "emailId is required" })),
        )
            .into_response();
    }
    if body.urls.len() > MAX_SIGN
        || body.parts.len() > MAX_SIGN
        || body.files.len() > MAX_SIGN
        || body.image_downloads.len() > MAX_SIGN
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "too many references in one request" })),
        )
            .into_response();
    }
    if !body.urls.is_empty() {
        match image_allowed(&state, &session, &body.email_id, &policy).await {
            Ok(true) => {}
            Ok(false) => return forbidden("remote content is not allowed for this message"),
            Err(()) => return internal("remote-content grant check"),
        }
    }
    if !body.files.is_empty() || !body.parts.is_empty() {
        match message_context(&state, &session, &body.email_id).await {
            Ok(Some(_)) => {}
            Ok(None) => {
                return forbidden("no such message");
            }
            Err(_) => return internal("remote-file message check"),
        }
    }

    if !body.image_downloads.is_empty() && !policy.remote_image_downloads {
        return forbidden("remote image downloads are disabled");
    }
    let references = if body.files.is_empty() && body.image_downloads.is_empty() {
        None
    } else {
        match membership::owned(&state, &session, &body.email_id, policy.clean_links).await {
            Ok(refs) => Some(refs),
            Err(response) => return response,
        }
    };

    let key = signing_key(&state);
    let now = now_secs();
    let sid = authority_tag(&key, &session);
    let claims = |k: Kind, u: &str, ttl: i64| Claims {
        v: 1,
        k,
        a: session.account_id.clone(),
        s: sid.clone(),
        e: body.email_id.clone(),
        u: u.to_string(),
        x: now + ttl,
        n: body
            .retry
            .then(|| format!("{:032x}", rand::random::<u128>())),
    };

    let mut images = Vec::new();
    let mut refused = Vec::new();
    for url in &body.urls {
        let reason = if url.len() > MAX_URL_LEN {
            Some("length")
        } else if !mw_sanitize::remote::is_http_url(url) || reqwest::Url::parse(url).is_err() {
            Some("scheme")
        } else if policy.trackers && mw_sanitize::remote::is_tracker_url(url) {
            Some("tracker")
        } else {
            None
        };
        match reason {
            Some(r) => refused.push(json!({ "url": url, "reason": r })),
            None => {
                if body.retry {
                    fetch_limits::forget(&session.account_id, [url.as_str()]);
                }
                let token = mint(&key, &claims(Kind::Image, url, IMAGE_TTL_SECS));
                images.push(json!({
                    "url": url,
                    "ref": fetch_limits::image_reference(url),
                    "src": format!("{}/api/rc/i/{token}", state.base_path),
                }));
            }
        }
    }
    // Ordinary signing is idempotent: the reader may refresh its message object
    // while the browser retains the exact same signed image URLs. Erasing those
    // outcomes would leave no subsequent fetch to restore progress or retries.
    // Only an explicit retry starts a fresh load for its accepted targets.
    if body.retry {
        fetch_limits::forget_outcomes(
            &session.account_id,
            &sid,
            &body.email_id,
            images
                .iter()
                .filter_map(|image| image["url"].as_str())
                .collect::<Vec<_>>(),
        );
    }
    let mut files = Vec::new();
    for url in &body.files {
        if url.len() > MAX_URL_LEN
            || !mw_sanitize::remote::is_http_url(url)
            || reqwest::Url::parse(url).is_err()
        {
            refused.push(json!({ "url": url, "reason": "scheme" }));
            continue;
        }
        let Some(normalized) = membership::normalized_url(url).filter(|value| {
            references
                .as_ref()
                .is_some_and(|refs| refs.links.contains(value))
        }) else {
            refused.push(json!({"url":url,"reason":"message"}));
            continue;
        };
        let token = mint(&key, &claims(Kind::File, &normalized, PART_TTL_SECS));
        files.push(json!({
            "url": url,
            "src": format!("{}/api/rc/f/{token}", state.base_path),
        }));
    }
    let mut image_downloads = Vec::new();
    for url in &body.image_downloads {
        let Some(normalized) = membership::normalized_url(url) else {
            refused.push(json!({"url":url,"reason":"scheme"}));
            continue;
        };
        let Some(tracker) = references
            .as_ref()
            .and_then(|refs| refs.images.get(&normalized))
        else {
            refused.push(json!({"url":url,"reason":"message"}));
            continue;
        };
        if policy.trackers && *tracker {
            refused.push(json!({"url":url,"reason":"tracker"}));
            continue;
        }
        let token = mint(
            &key,
            &claims(Kind::ImageDownload, &normalized, PART_TTL_SECS),
        );
        image_downloads
            .push(json!({"url":url,"src":format!("{}/api/rc/d/{token}",state.base_path)}));
    }
    let allowed_parts = if body.parts.is_empty() {
        Vec::new()
    } else {
        match part_membership(&state, &session, &body.email_id).await {
            Ok(ids) => ids,
            Err(()) => return internal("part membership check"),
        }
    };
    let mut parts = Vec::new();
    for blob in &body.parts {
        let ok = !blob.is_empty()
            && blob.len() <= MAX_BLOB_ID_LEN
            && blob.bytes().all(|b| b.is_ascii_graphic() && b != b'/');
        if !ok {
            refused.push(json!({ "blobId": blob, "reason": "blobId" }));
            continue;
        }
        if !allowed_parts.contains(blob) {
            refused.push(json!({"blobId":blob,"reason":"notMessagePart"}));
            continue;
        }
        let token = mint(&key, &claims(Kind::Part, blob, PART_TTL_SECS));
        parts.push(json!({
            "blobId": blob,
            "src": format!("{}/api/rc/p/{token}", state.base_path),
        }));
    }
    Json(json!({ "images": images, "parts": parts, "files": files, "imageDownloads": image_downloads, "refused": refused }))
        .into_response()
}

// ── using a token ──────────────────────────────────────────────────────────────

/// Steps 1 and 2 of the module docs: the token, then the session it was minted
/// for. Returns the claims and that session.
async fn admit(
    state: &AppState,
    token: &str,
    kind: Kind,
) -> Result<(Claims, mw_store::Session), Response> {
    let key = signing_key(state);
    let claims = match verify(&key, token, now_secs()) {
        Ok(c) if c.k == kind => c,
        _ => return Err(forbidden("invalid or expired link")),
    };
    let sessions = state
        .store
        .sessions_by_account(&claims.a)
        .await
        .map_err(|_| internal("remote-content session check"))?;
    let mut session = sessions
        .into_iter()
        .find(|s| authority_tag(&key, s) == claims.s)
        .ok_or_else(|| forbidden("invalid or expired link"))?;
    // A session past its idle or absolute timeout no longer loads anything. The
    // check records no activity, so loading images never keeps a session alive.
    if state
        .sessions
        .expired(&session.id, &state.hardening)
        .is_some()
    {
        return Err(forbidden("invalid or expired link"));
    }
    crate::security_center::enforce_session(state, &session)
        .await
        .map_err(|_| forbidden("invalid or expired link"))?;
    crate::domain_upgrade::refresh_session(state, &mut session)
        .await
        .map_err(|_| forbidden("invalid or expired link"))?;
    // A reconfigured upstream can change the session's account. Old signed
    // content must never survive that authority change.
    if session.account_id != claims.a || authority_tag(&key, &session) != claims.s {
        return Err(forbidden("invalid or expired link"));
    }
    let gate = crate::account_gate::for_session(&state.store, &session)
        .await
        .map_err(|_| internal("remote-content account check"))?;
    if gate.disabled || gate.password_change_required {
        return Err(forbidden("invalid or expired link"));
    }
    crate::request_protection::session_account(&session).await?;
    Ok((claims, session))
}

/// The magic-byte type of an image body, among [`ALLOWED_IMAGE_TYPES`].
fn sniff(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("image/png")
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some("image/gif")
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

/// Why a fetched body is refused before decoding, or `None` when it may be decoded:
/// the declared type must be an allowed image type and the bytes must be that type.
pub(crate) fn type_refusal(declared: Option<&str>, bytes: &[u8]) -> Option<&'static str> {
    match declared {
        Some("image/svg+xml") => Some("SVG images are not loaded"),
        Some(d) if ALLOWED_IMAGE_TYPES.contains(&d) => match sniff(bytes) {
            Some(s) if s == d => None,
            _ => Some("the image does not match its declared type"),
        },
        _ => Some("not an allowed image type"),
    }
}

/// The width and height in a PNG's header.
fn png_size(png: &[u8]) -> Option<(u32, u32)> {
    if png.len() < 24 || !png.starts_with(b"\x89PNG\r\n\x1a\n") || &png[12..16] != b"IHDR" {
        return None;
    }
    let w = u32::from_be_bytes(png[16..20].try_into().ok()?);
    let h = u32::from_be_bytes(png[20..24].try_into().ok()?);
    Some((w, h))
}

fn unsupported(msg: &'static str) -> Response {
    (StatusCode::UNSUPPORTED_MEDIA_TYPE, msg).into_response()
}

/// A signed image response. `tracker` marks a replaced tracking pixel.
fn signed_image(png: Vec<u8>, etag: String, tracker: bool) -> Response {
    let mut resp = (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "image/png".to_string()),
            // Revalidate before reuse so revocation also applies to a browser
            // cache hit; the server retains the re-encoded bytes and ETag.
            (header::CACHE_CONTROL, "private, no-cache".to_string()),
            (header::ETAG, etag),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_string()),
        ],
        png,
    )
        .into_response();
    let h = resp.headers_mut();
    // See the module docs: the requester is an opaque origin.
    h.insert(
        "cross-origin-resource-policy",
        HeaderValue::from_static("cross-origin"),
    );
    if tracker {
        h.insert("x-mw-remote-content", HeaderValue::from_static("tracker"));
    }
    resp
}

/// Re-encode `raw` in the wasm media jail and answer it, replacing a 2×2-or-smaller
/// result with [`TRANSPARENT_PIXEL`].
pub(super) async fn reencode_and_answer(
    raw: Vec<u8>,
    key: Option<CacheKey>,
    policy: &crate::remote_policy::PrivacyPolicy,
) -> Response {
    let png = match tokio::task::spawn_blocking(move || mw_render::media_jail::reencode_image(&raw))
        .await
    {
        Ok(Ok(png)) => png,
        Ok(Err(_)) => return unsupported("not a decodable image"),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "re-encode failed").into_response(),
    };
    let tracker = policy.trackers && png_size(&png).is_some_and(|(w, h)| w <= 2 && h <= 2);
    let png = if tracker {
        TRACKERS_REPLACED.fetch_add(1, Ordering::Relaxed);
        TRANSPARENT_PIXEL.to_vec()
    } else {
        png
    };
    let etag = etag_for(&png);
    if let Some(key) = key {
        cache().lock().expect("image cache lock").put_with_budget(
            key,
            etag.clone(),
            png.clone(),
            policy.cache_bytes,
        );
    }
    signed_image(png, etag, tracker)
}

/// `GET /api/rc/i/{token}` — a signed remote image.
pub(crate) async fn image(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(token): UrlPath<String>,
) -> Response {
    let (claims, session) = match admit(&state, &token, Kind::Image).await {
        Ok(v) => v,
        Err(r) => return r,
    };
    let policy = match crate::remote_policy::effective(&state, &session).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    match image_allowed(&state, &session, &claims.e, &policy).await {
        Ok(true) => {}
        Ok(false) => return forbidden("remote content is not allowed for this message"),
        Err(()) => return internal("remote-content grant check"),
    }
    if policy.trackers && mw_sanitize::remote::is_tracker_url(&claims.u) {
        return forbidden("tracker blocked by policy");
    }

    fetch_image(state, session, claims, policy, headers).await
}

async fn fetch_image(
    state: AppState,
    session: mw_store::Session,
    claims: Claims,
    policy: crate::remote_policy::PrivacyPolicy,
    headers: HeaderMap,
) -> Response {
    let plan = match load_plan(&state.store).await {
        Ok(p) => p,
        Err(()) => return internal("egress route lookup"),
    };
    let domain = match crate::remote_policy::canonical_domain(&state, &session).await {
        Ok(domain) => domain,
        Err(response) => return response,
    };
    let route_id = plan
        .selected_route(TrafficClass::RemoteContent, domain.as_deref())
        .map_or(DIRECT_ROUTE, |r| r.id.as_str())
        .to_string();

    // A cache hit is served before the rate limit, like the image proxy's.
    let key = CacheKey::new(
        &format!("{route_id}:{}:{}", policy.trackers, policy.max_bytes),
        &claims.u,
    );
    cache()
        .lock()
        .expect("image cache lock")
        .trim(policy.cache_bytes);
    let cached_response = || {
        let (etag, png) = cache().lock().expect("image cache lock").get(&key)?;
        let tracker = policy.trackers && png == TRANSPARENT_PIXEL;
        if tracker {
            tracker_stats::record(&claims.a, &claims.s, &claims.e, &claims.u, claims.x);
        }
        fetch_limits::record_outcome(&claims.a, &claims.s, &claims.e, &claims.u, true, claims.x);
        if if_none_match(&headers, &etag) {
            let mut response = not_modified(&etag);
            response.headers_mut().insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("private, no-cache"),
            );
            response.headers_mut().insert(
                "cross-origin-resource-policy",
                HeaderValue::from_static("cross-origin"),
            );
            return Some(response);
        }
        Some(signed_image(png, etag, tracker))
    };
    if let Some(response) = cached_response() {
        return response;
    }
    let deadline = std::time::Instant::now() + policy.fetch.timeouts.total;
    let Some(_flight) = image_flight::acquire(&claims.a, &key.route, &claims.u, deadline).await
    else {
        fetch_limits::record_outcome(&claims.a, &claims.s, &claims.e, &claims.u, false, claims.x);
        fetch_limits::record_retryable_failure(
            &claims.a, &claims.s, &claims.e, &claims.u, claims.x,
        );
        return busy_response();
    };
    if let Some(response) = cached_response() {
        return response;
    }
    let outcome = |loaded: bool| {
        fetch_limits::record_outcome(&claims.a, &claims.s, &claims.e, &claims.u, loaded, claims.x);
    };
    // A failure remembered for this URL answers before the rate limit: it costs
    // no network, and charging for it would let a dead host drain the budget.
    if let Some(failure) = fetch_limits::remembered(&claims.a, &claims.u) {
        outcome(false);
        if fetch_limits::retryable_refusal(&failure.refusal()) {
            fetch_limits::record_retryable_failure(
                &claims.a, &claims.s, &claims.e, &claims.u, claims.x,
            );
        }
        return failed_response(failure.refusal());
    }
    if !rate_limiter()
        .lock()
        .expect("image rate-limit lock")
        .check_with_limit(
            &claims.a,
            policy.rate_per_min as f64,
            policy.rate_per_min as f64 / 60.0,
        )
    {
        fetch_limits::record_retryable_failure(
            &claims.a, &claims.s, &claims.e, &claims.u, claims.x,
        );
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "remote content rate limit exceeded",
        )
            .into_response();
    }
    let Ok(url) = reqwest::Url::parse(&claims.u) else {
        return refusal_response(mw_egress::Refusal::BadRequest("malformed URL"));
    };

    let request = Request::get(TrafficClass::RemoteContent)
        .response_limit(policy.max_bytes)
        .accept("image/png, image/jpeg, image/gif, image/webp");
    let (raw, content_type) = match guarded_fetch(
        &state,
        &plan,
        &route_id,
        url,
        request,
        domain.as_deref(),
        (&claims.a, &claims.e, &claims.u),
        &policy,
        deadline,
    )
    .await
    {
        Ok(fetched) => fetched,
        Err(response) => {
            outcome(false);
            if response
                .headers()
                .get("x-mw-remote-retryable")
                .is_some_and(|v| v == "1")
            {
                fetch_limits::record_retryable_failure(
                    &claims.a, &claims.s, &claims.e, &claims.u, claims.x,
                );
            }
            return response;
        }
    };
    if raw.len() > policy.max_bytes {
        outcome(false);
        return refusal_response(mw_egress::Refusal::TooLarge);
    }
    if let Some(why) = type_refusal(content_type.as_deref(), &raw) {
        outcome(false);
        return unsupported(why);
    }
    let response = reencode_and_answer(raw, Some(key), &policy).await;
    if response
        .headers()
        .get("x-mw-remote-content")
        .is_some_and(|value| value == "tracker")
    {
        tracker_stats::record(&claims.a, &claims.s, &claims.e, &claims.u, claims.x);
        outcome(true);
    } else {
        outcome(response.status().is_success());
    }
    response
}

/// One remote-image fetch under the t56 limits, shared by `/api/rc/i` and the
/// legacy `/api/image-proxy`: a per-message and per-account slot (waited for,
/// inside the deadline), the replica's global ceiling (shed), the per-phase
/// timeouts and redirect cap, the egress audit, and the negative cache for a
/// network failure. Returns the body and its declared type, or the response to
/// send.
///
/// The total deadline includes single-flight admission before any fetch slot, so network work stays within
/// `privacy.fetchTotalTimeoutMs` whether it queued, fetched or both.
#[allow(clippy::too_many_arguments)]
pub(super) async fn guarded_fetch(
    state: &AppState,
    plan: &mw_egress::EgressPlan,
    route_id: &str,
    url: reqwest::Url,
    request: Request<'_>,
    domain: Option<&str>,
    (account, message, raw_url): (&str, &str, &str),
    policy: &crate::remote_policy::PrivacyPolicy,
    deadline: std::time::Instant,
) -> Result<(Vec<u8>, Option<String>), Response> {
    let limits = policy.fetch;
    let started = std::time::Instant::now();
    let Some(_slot) = fetch_limits::acquire(account, message, &limits, deadline).await else {
        tracing::debug!(
            target: "mw_server::remote_content",
            phase = "slot",
            elapsed_ms = started.elapsed().as_millis() as u64,
            "remote image gave up waiting for a fetch slot"
        );
        return Err(busy_response());
    };
    // Shed rather than queue at the replica ceiling, as before t56.
    let Ok(_permit) = fetch_semaphore().try_acquire() else {
        return Err(busy_response());
    };
    let remaining = deadline.saturating_duration_since(std::time::Instant::now());
    if remaining.is_zero() {
        return Err(busy_response());
    }
    let timeouts = mw_egress::FetchTimeouts {
        total: remaining.max(std::time::Duration::from_millis(1)),
        ..limits.timeouts
    };
    let report = mw_egress::TimeoutReport::new();
    let mut request = request.timeouts(timeouts).timeout_report(&report);
    if let Some(d) = domain {
        request = request.for_domain(d);
    }
    let fetched = plan::send_with(plan, url, &request).await;
    audit_egress_transition(state, route_id, fetched.traversed_proxy).await;
    match fetched.outcome {
        Ok(bytes) => Ok((bytes, fetched.content_type)),
        Err(refusal) => {
            if matches!(refusal, mw_egress::Refusal::Timeout) {
                // Never the URL, the host or the account.
                tracing::debug!(
                    target: "mw_server::remote_content",
                    phase = report.phase().map_or("transport", |p| p.as_str()),
                    elapsed_ms = started.elapsed().as_millis() as u64,
                    routed = route_id != DIRECT_ROUTE,
                    "remote image fetch timed out"
                );
            }
            fetch_limits::remember(account, raw_url, &refusal);
            Err(failed_response(refusal))
        }
    }
}

/// A failed fetch, as the reader's frame receives it: the usual refusal status
/// and text, `X-MW-Remote-Content: failed`, and a short private cache lifetime
/// matching the server's negative cache so the browser does not refetch it in a
/// loop either. A new signed link (Retry images) is a new URL.
pub(super) fn failed_response(refusal: mw_egress::Refusal) -> Response {
    let cacheable = fetch_limits::Failure::of(&refusal).is_some();
    let retryable = fetch_limits::retryable_refusal(&refusal);
    let mut response = refusal_response(refusal);
    let h = response.headers_mut();
    h.insert("x-mw-remote-content", HeaderValue::from_static("failed"));
    if retryable {
        h.insert("x-mw-remote-retryable", HeaderValue::from_static("1"));
    }
    h.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(if cacheable {
            "private, max-age=60"
        } else {
            "no-store"
        }),
    );
    response
}

/// No fetch slot in time, or the replica is at its ceiling. Not remembered:
/// nothing is wrong with the image.
pub(super) fn busy_response() -> Response {
    let mut response = (StatusCode::SERVICE_UNAVAILABLE, "image proxy busy").into_response();
    let h = response.headers_mut();
    h.insert("x-mw-remote-content", HeaderValue::from_static("failed"));
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    h.insert(header::RETRY_AFTER, HeaderValue::from_static("5"));
    h.insert("x-mw-remote-retryable", HeaderValue::from_static("1"));
    response
}

/// A manual, policy-enabled image download through the same guarded image decoder.
pub(crate) async fn image_download(
    State(state): State<AppState>,
    UrlPath(token): UrlPath<String>,
    headers: HeaderMap,
) -> Response {
    let (claims, session) = match admit(&state, &token, Kind::ImageDownload).await {
        Ok(value) => value,
        Err(response) => return response,
    };
    let current = match crate::authed(&state, &headers).await {
        Ok(value) => value,
        Err(response) => return response,
    };
    if current.id != session.id
        || current.account_id != session.account_id
        || headers
            .get("x-mailwoman-account")
            .is_some_and(|value| value.to_str().ok() != Some(session.account_id.as_str()))
    {
        return (
            StatusCode::CONFLICT,
            Json(json!({"error":"accountChanged"})),
        )
            .into_response();
    }
    let policy = match crate::remote_policy::effective(&state, &session).await {
        Ok(value) => value,
        Err(response) => return response,
    };
    if !policy.remote_image_downloads {
        return forbidden("remote image downloads are disabled");
    }
    let refs = match membership::owned(&state, &session, &claims.e, policy.clean_links).await {
        Ok(value) => value,
        Err(response) => return response,
    };
    let Some(url) = membership::normalized_url(&claims.u) else {
        return forbidden("invalid image URL");
    };
    let Some(tracker) = refs.images.get(&url) else {
        return forbidden("URL is not an image in this message");
    };
    if policy.trackers && *tracker {
        return forbidden("tracker blocked by policy");
    }
    let mut response = fetch_image(state, session, claims, policy, HeaderMap::new()).await;
    if !response.status().is_success() {
        return response;
    }
    if response
        .headers()
        .get("x-mw-remote-content")
        .is_some_and(|value| value == "tracker")
    {
        return forbidden("tracker blocked by policy");
    }
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"message-image.png\""),
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    response.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("sandbox; default-src 'none'"),
    );
    response
}

fn charge_fetch(
    account: &str,
    policy: &crate::remote_policy::PrivacyPolicy,
) -> Result<(), Response> {
    if rate_limiter()
        .lock()
        .expect("image rate-limit lock")
        .check_with_limit(
            account,
            policy.rate_per_min as f64,
            policy.rate_per_min as f64 / 60.0,
        )
    {
        Ok(())
    } else {
        Err((
            StatusCode::TOO_MANY_REQUESTS,
            "remote content rate limit exceeded",
        )
            .into_response())
    }
}

/// `GET /api/rc/p/{token}` — a signed inline part of the message, served as a
/// re-encoded image. Only image parts are served; anything else is `415`.
pub(crate) async fn part(
    State(state): State<AppState>,
    UrlPath(token): UrlPath<String>,
) -> Response {
    let (claims, session) = match admit(&state, &token, Kind::Part).await {
        Ok(v) => v,
        Err(r) => return r,
    };
    let policy = match crate::remote_policy::effective(&state, &session).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    if let Err(r) = charge_fetch(&session.account_id, &policy) {
        return r;
    }
    match part_membership(&state, &session, &claims.e).await {
        Ok(parts) if parts.contains(&claims.u) => {}
        Ok(_) => return forbidden("not a part of this message"),
        Err(()) => return internal("part membership check"),
    }
    let attachment_policy = match crate::attachment_policy::effective(&state, &session).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    if !attachment_policy.preview_allowed || !attachment_policy.download_allowed {
        return crate::attachment_policy::refusal("attachmentPreviewDisabled");
    }
    let attachment_query = crate::attachment_policy::DownloadQuery {
        email_id: Some(claims.e.clone()),
        preview: Some(true),
        acknowledge_warning: None,
    };
    let metadata = if state.engine.is_none() && attachment_policy.restricts_content() {
        match crate::attachment_policy::proxy_metadata(
            &state,
            &session,
            &claims.u,
            &attachment_query,
        )
        .await
        {
            Ok(parts) => parts,
            Err(r) => return r,
        }
    } else {
        Vec::new()
    };
    for part in &metadata {
        if let Err(r) =
            crate::attachment_policy::authorize(&attachment_policy, part, &attachment_query)
        {
            return r;
        }
    }
    let _permit = match fetch_semaphore().try_acquire() {
        Ok(p) => p,
        Err(_) => return (StatusCode::SERVICE_UNAVAILABLE, "remote content busy").into_response(),
    };
    let (bytes, content_type, filename) = match tokio::time::timeout(
        std::time::Duration::from_secs(15),
        read_part(
            &state,
            &session,
            &claims.u,
            if attachment_policy.max_bytes > 0 {
                policy.max_bytes.min(attachment_policy.max_bytes as usize)
            } else {
                policy.max_bytes
            },
        ),
    )
    .await
    {
        Ok(Ok(Some(v))) => v,
        Ok(Ok(None)) => return (StatusCode::NOT_FOUND, "part not found").into_response(),
        Ok(Err(r)) => return r,
        Err(_) => return (StatusCode::GATEWAY_TIMEOUT, "part download timed out").into_response(),
    };
    if let Some(name) = filename {
        let actual = crate::attachment_policy::Metadata {
            name,
            mime: content_type
                .clone()
                .unwrap_or_else(|| "application/octet-stream".into()),
            size: bytes.len() as u64,
        };
        if let Err(r) =
            crate::attachment_policy::authorize(&attachment_policy, &actual, &attachment_query)
        {
            return r;
        }
    }
    for part in &metadata {
        let actual = crate::attachment_policy::Metadata {
            name: part.name.clone(),
            mime: content_type
                .clone()
                .unwrap_or_else(|| "application/octet-stream".into()),
            size: bytes.len() as u64,
        };
        if let Err(r) =
            crate::attachment_policy::authorize(&attachment_policy, &actual, &attachment_query)
        {
            return r;
        }
    }
    let declared = mw_egress::declared_media_type(content_type.as_deref());
    if let Some(why) = type_refusal(declared.as_deref(), &bytes) {
        return unsupported(why);
    }
    if bytes.len() > policy.max_bytes {
        return refusal_response(mw_egress::Refusal::TooLarge);
    }
    reencode_and_answer(bytes, None, &policy).await
}

/// `GET /api/rc/f/{token}` — a remote file linked from the message, downloaded
/// through the server: fetched through the egress plan under the same address
/// gate and size limit as an image, and served as `application/octet-stream`
/// with `Content-Disposition: attachment` and a sandboxing CSP, so a browser saves
/// it and never renders it.
pub(crate) async fn file(
    State(state): State<AppState>,
    UrlPath(token): UrlPath<String>,
) -> Response {
    let (claims, session) = match admit(&state, &token, Kind::File).await {
        Ok(v) => v,
        Err(r) => return r,
    };
    let policy = match crate::remote_policy::effective(&state, &session).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    if !rate_limiter()
        .lock()
        .expect("image rate-limit lock")
        .check_with_limit(
            &claims.a,
            policy.rate_per_min as f64,
            policy.rate_per_min as f64 / 60.0,
        )
    {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "remote content rate limit exceeded",
        )
            .into_response();
    }
    match message_context(&state, &session, &claims.e).await {
        Ok(Some(_)) => {}
        Ok(None) => return forbidden("no such message"),
        Err(()) => return internal("remote-file message check"),
    }
    let references = match membership::owned(&state, &session, &claims.e, policy.clean_links).await
    {
        Ok(refs) => refs,
        Err(response) => return response,
    };
    if !membership::normalized_url(&claims.u).is_some_and(|url| references.links.contains(&url)) {
        return forbidden("URL is not a link in this message");
    }
    let _permit = match fetch_semaphore().try_acquire() {
        Ok(p) => p,
        Err(_) => return (StatusCode::SERVICE_UNAVAILABLE, "remote content busy").into_response(),
    };
    let plan = match load_plan(&state.store).await {
        Ok(p) => p,
        Err(()) => return internal("egress route lookup"),
    };
    let domain = match crate::remote_policy::canonical_domain(&state, &session).await {
        Ok(domain) => domain,
        Err(response) => return response,
    };
    let route_id = plan
        .selected_route(TrafficClass::RemoteContent, domain.as_deref())
        .map_or(DIRECT_ROUTE, |r| r.id.as_str())
        .to_string();
    let Ok(url) = reqwest::Url::parse(&claims.u) else {
        return refusal_response(mw_egress::Refusal::BadRequest("malformed URL"));
    };
    let name = download_name(&url);
    let mut request = Request::get(TrafficClass::RemoteContent).response_limit(policy.max_bytes);
    if let Some(d) = domain.as_deref() {
        request = request.for_domain(d);
    }
    let fetched = plan::send_with(&plan, url, &request).await;
    audit_egress_transition(&state, &route_id, fetched.traversed_proxy).await;
    let bytes = match fetched.outcome {
        Ok(b) => b,
        Err(r) => return refusal_response(r),
    };
    if bytes.len() > policy.max_bytes {
        return refusal_response(mw_egress::Refusal::TooLarge);
    }
    let mut resp = (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/octet-stream".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{name}\""),
            ),
            (header::CACHE_CONTROL, "private, no-store".to_string()),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_string()),
        ],
        bytes,
    )
        .into_response();
    resp.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("sandbox; default-src 'none'"),
    );
    resp
}

pub(crate) async fn stats(
    State(_state): State<AppState>,
    ctx: crate::admin_ctx::AdminCtx,
) -> Result<Json<serde_json::Value>, crate::admin_ctx::AdminApiError> {
    ctx.require_unscoped(crate::capabilities::Capability::EgressRead)?;
    let cache = cache().lock().expect("image cache lock");
    let traffic: Vec<_> = mw_egress::plan::counts()
        .into_iter()
        .map(|(class, counts)| {
            json!({
                "class":class.as_str(), "allowed":counts.allowed, "refused":counts.refused,
                "failed":counts.failed, "totalMs":counts.total_ms, "totalUs":counts.total_us,
            })
        })
        .collect();
    Ok(Json(
        json!({"cacheEntries":cache.map.len(),"cacheBytes":cache.bytes,
        "trackersReplaced":TRACKERS_REPLACED.load(Ordering::Relaxed),"traffic":traffic}),
    ))
}

/// The file name for a downloaded remote file: the last path segment, reduced to
/// letters, digits, `.`, `-` and `_` (anything else becomes `_`), at most 100
/// characters; `download` when nothing is left.
fn download_name(url: &reqwest::Url) -> String {
    let last = url
        .path_segments()
        .and_then(|mut s| s.next_back())
        .unwrap_or("");
    let cleaned: String = last
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .take(100)
        .collect();
    let cleaned = cleaned.trim_matches('.').to_string();
    if cleaned.is_empty() {
        "download".to_string()
    } else {
        cleaned
    }
}

/// A part's bytes and declared type, read as `/jmap/download` reads a blob: from
/// the engine in engine mode, from the upstream's download URL with the session's
/// credentials in proxy mode.
pub(crate) async fn read_part(
    state: &AppState,
    session: &mw_store::Session,
    blob_id: &str,
    limit: usize,
) -> Result<Option<(Vec<u8>, Option<String>, Option<String>)>, Response> {
    if let Some(engine) = &state.engine {
        return match engine
            .fetch_blob_limited(&session.account_id, blob_id, MAX_PART_RAW_BYTES, limit)
            .await
        {
            Ok(Some(blob)) => Ok(Some((
                blob.bytes,
                Some(blob.content_type),
                Some(blob.filename),
            ))),
            Ok(None) => Ok(None),
            Err(e) => {
                tracing::warn!("part fetch failed: {e}");
                Err(crate::upstream_error())
            }
        };
    }
    let mut discovery_url =
        mw_jmap::client::session_url(&session.jmap_url).map_err(|_| crate::upstream_error())?;
    let mut discovered = None;
    for _ in 0..=mw_egress::MAX_REDIRECTS {
        let session_client = crate::upstream_client(
            &state.security,
            &session.jmap_url,
            &discovery_url,
            &session.credentials,
        )
        .await
        .map_err(|_| crate::upstream_error())?;
        match session_client
            .session_at_limited(&discovery_url, 256 * 1024)
            .await
        {
            Ok(session) => {
                discovered = Some(session);
                break;
            }
            Err(mw_jmap::JmapError::Redirect(location)) => {
                discovery_url = reqwest::Url::parse(&discovery_url)
                    .ok()
                    .and_then(|url| url.join(&location).ok())
                    .ok_or_else(crate::upstream_error)?
                    .to_string();
            }
            Err(_) => return Err(crate::upstream_error()),
        }
    }
    let upstream = discovered.ok_or_else(crate::upstream_error)?;
    let url = upstream
        .download_url
        .replace("{accountId}", &crate::percent_encode(&session.account_id))
        .replace("{blobId}", &crate::percent_encode(blob_id))
        .replace("{name}", "part")
        .replace("{type}", "application/octet-stream");
    let abs = crate::resolve_api_url(&session.jmap_url, &url);
    let client = crate::upstream_client(
        &state.security,
        &session.jmap_url,
        &abs,
        &session.credentials,
    )
    .await
    .map_err(|r| {
        tracing::warn!("upstream part download refused: {r}");
        crate::upstream_error()
    })?;
    match client.get_bytes_limited(&abs, limit).await {
        Ok((status, content_type, bytes)) if status.as_u16() == 200 => {
            Ok(Some((bytes.to_vec(), content_type, None)))
        }
        Ok((status, ..)) if status.as_u16() == 404 => Ok(None),
        Ok(_) => Err(crate::upstream_error()),
        Err(mw_jmap::JmapError::TooLarge) => Err(refusal_response(mw_egress::Refusal::TooLarge)),
        Err(e) => {
            tracing::warn!("part download failed: {e}");
            Err(crate::upstream_error())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: [u8; 32] = [7; 32];

    fn claims(x: i64) -> Claims {
        Claims {
            v: 1,
            k: Kind::Image,
            a: "acct".into(),
            s: session_tag(&KEY, "session-secret"),
            e: "M1".into(),
            u: "https://cdn.example/a.png".into(),
            x,
            n: None,
        }
    }

    #[test]
    fn a_minted_token_verifies_until_it_expires() {
        let t = mint(&KEY, &claims(1_000));
        assert_eq!(verify(&KEY, &t, 999), Ok(claims(1_000)));
        assert_eq!(verify(&KEY, &t, 1_000), Ok(claims(1_000)));
        assert_eq!(verify(&KEY, &t, 1_001), Err(()));
    }

    #[test]
    fn retry_nonce_changes_cache_key_and_standard_expiry_is_still_enforced() {
        let original = claims(1000);
        let mut retry = original.clone();
        retry.n = Some("retry-1".into());
        assert_ne!(mint(&KEY, &original), mint(&KEY, &retry));
        assert_eq!(verify(&KEY, &mint(&KEY, &retry), 999), Ok(retry.clone()));
        assert!(verify(&KEY, &mint(&KEY, &retry), 1001).is_err());
        assert_eq!(retry.a, original.a);
        assert_eq!(retry.s, original.s);
        assert_eq!(retry.e, original.e);
    }

    #[test]
    fn a_token_cannot_be_retargeted_or_moved_to_another_key() {
        let t = mint(&KEY, &claims(1_000));
        let (payload, tag) = t.split_once('.').unwrap();
        // Retarget: same tag, claims for another URL.
        let mut other = claims(1_000);
        other.u = "http://169.254.169.254/latest/meta-data/".into();
        let forged_payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&other).unwrap());
        assert_eq!(verify(&KEY, &format!("{forged_payload}.{tag}"), 0), Err(()));
        // Extend the expiry the same way.
        let longer = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&claims(i64::MAX)).unwrap());
        assert_eq!(verify(&KEY, &format!("{longer}.{tag}"), 0), Err(()));
        // A one-bit change of the tag, a missing tag, junk.
        let mut bad = URL_SAFE_NO_PAD.decode(tag).unwrap();
        bad[0] ^= 1;
        assert_eq!(
            verify(
                &KEY,
                &format!("{payload}.{}", URL_SAFE_NO_PAD.encode(bad)),
                0
            ),
            Err(())
        );
        for junk in ["", ".", payload, "a.b", "%%%.%%%"] {
            assert_eq!(verify(&KEY, junk, 0), Err(()), "{junk:?}");
        }
        assert_eq!(verify(&[8; 32], &t, 0), Err(()), "another server key");
    }

    #[test]
    fn the_session_id_never_appears_in_a_token() {
        let t = mint(&KEY, &claims(1_000));
        let decoded = String::from_utf8(
            URL_SAFE_NO_PAD
                .decode(t.split_once('.').unwrap().0)
                .unwrap(),
        )
        .unwrap();
        assert!(!decoded.contains("session-secret"), "{decoded}");
        assert_eq!(session_tag(&KEY, "session-secret").len(), 32);
        assert_ne!(
            session_tag(&KEY, "session-secret"),
            session_tag(&KEY, "another")
        );
    }

    #[test]
    fn only_allowed_image_types_whose_bytes_match_are_decoded() {
        let png = b"\x89PNG\r\n\x1a\n....";
        let jpeg = [0xFF, 0xD8, 0xFF, 0xE0, 0, 0];
        let gif = b"GIF89a....";
        let webp = b"RIFF\0\0\0\0WEBPVP8 ";
        assert_eq!(type_refusal(Some("image/png"), png), None);
        assert_eq!(type_refusal(Some("image/jpeg"), &jpeg), None);
        assert_eq!(type_refusal(Some("image/gif"), gif), None);
        assert_eq!(type_refusal(Some("image/webp"), webp), None);
        assert_eq!(
            type_refusal(Some("image/svg+xml"), b"<svg/>"),
            Some("SVG images are not loaded")
        );
        assert_eq!(
            type_refusal(Some("image/png"), gif),
            Some("the image does not match its declared type")
        );
        for declared in [
            None,
            Some("text/html"),
            Some("image/x-icon"),
            Some("application/octet-stream"),
        ] {
            assert_eq!(
                type_refusal(declared, png),
                Some("not an allowed image type"),
                "{declared:?}"
            );
        }
    }

    #[test]
    fn a_download_name_keeps_only_safe_characters() {
        let name = |u: &str| download_name(&reqwest::Url::parse(u).unwrap());
        assert_eq!(
            name("https://files.example/a/report-2026.pdf"),
            "report-2026.pdf"
        );
        assert_eq!(
            name("https://files.example/a/x%22%3B%20y.zip"),
            "x_22_3B_20y.zip"
        );
        assert_eq!(name("https://files.example/"), "download");
        assert_eq!(name("https://files.example/.."), "download");
        assert_eq!(
            name(&format!("https://f.example/{}", "a".repeat(300))).len(),
            100
        );
    }

    #[test]
    fn the_transparent_pixel_is_a_one_by_one_png() {
        assert_eq!(png_size(TRANSPARENT_PIXEL), Some((1, 1)));
        assert_eq!(sniff(TRANSPARENT_PIXEL), Some("image/png"));
    }
}
