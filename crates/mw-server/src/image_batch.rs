//! `POST /api/rc/batch`: one authenticated request that fetches a message's
//! signed remote images together.
//!
//! The consent model is not redesigned. The request carries the signed image
//! tokens `POST /api/rc/sign` already issued (the last path segment of each
//! `/api/rc/i/{token}` source). What changes is how often the checks run:
//!
//! 1. The session is authenticated once (`crate::authed`), never from a token.
//! 2. Every token is verified (MAC, kind, expiry) with no database read, and
//!    must have been minted for *this* session and account, so a token cannot
//!    be replayed by another session. All accepted tokens must name the same
//!    message; others are refused individually.
//! 3. The message policy and the grant/membership check
//!    (`image_allowed`) run once for that message. Without a grant nothing is
//!    fetched and no rate budget is spent.
//! 4. Each URL then goes through the same `fetch_image` the single-URL route
//!    uses: cache, single-flight, per-message / per-account / global slots,
//!    the rate budget, the per-URL egress policy (SSRF refusal), the size
//!    caps and the media jail. One URL's failure never fails the others.
//!
//! The reply is JSON, one entry per requested token in request order, with the
//! status the single-URL route would have answered, its content type, ETag and,
//! when `bodies` is true and the size caps allow, the PNG as base64. A batch
//! without `bodies` warms the server's image cache: the browser's own `<img>`
//! requests that follow are then answered from it, before any rate or slot
//! accounting.

use std::collections::HashMap;

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use base64::Engine as _;
use futures_util::StreamExt as _;
use serde::Deserialize;
use serde_json::{Value, json};

use super::{Claims, Kind, fetch_image, image_allowed, now_secs, signing_key};
use crate::AppState;

/// Most tokens one batch may carry.
pub(super) const MAX_BATCH: usize = 100;
/// Largest single body returned inline (re-encoded PNG bytes).
const MAX_INLINE_BODY: usize = 4 * 1024 * 1024;
/// Largest total of inline bodies in one reply.
const MAX_INLINE_TOTAL: usize = 16 * 1024 * 1024;
/// Longest token accepted (matches the router's path token bound).
const MAX_TOKEN_LEN: usize = 16 * 1024;
/// Upper bound on fetches in flight from one batch, whatever the policy allows.
/// The per-message and per-account slots still apply inside each fetch.
const MAX_PARALLEL: usize = 16;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BatchReq {
    #[serde(default)]
    tokens: Vec<String>,
    /// Return the image bytes (base64) as well as the status.
    #[serde(default)]
    bodies: bool,
}

fn refused(index: usize, status: u16, reason: &str) -> Value {
    json!({ "index": index, "status": status, "error": reason })
}

/// The token itself, whether the caller sent it bare or as its signed path.
fn token_of(raw: &str) -> &str {
    raw.rsplit('/').next().unwrap_or(raw)
}

pub(crate) async fn batch(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<BatchReq>,
) -> Response {
    // Cheap shape checks first: nothing below runs for an over-cap request.
    if body.tokens.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "tokens are required" })),
        )
            .into_response();
    }
    if body.tokens.len() > MAX_BATCH {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "too many images in one batch", "max": MAX_BATCH })),
        )
            .into_response();
    }
    let session = match crate::authed(&state, &headers).await {
        Ok(session) => session,
        Err(response) => return response,
    };
    if let Err(response) = crate::request_protection::session_account(&session).await {
        return response;
    }

    let key = signing_key(&state);
    let now = now_secs();
    let sid = super::authority_tag(&key, &session);
    let mut results: Vec<Option<Value>> = vec![None; body.tokens.len()];
    let mut message: Option<String> = None;
    // (request index, claims); one fetch per distinct URL, duplicates share it.
    let mut accepted: Vec<(usize, Claims)> = Vec::new();
    for (index, raw) in body.tokens.iter().enumerate() {
        let token = token_of(raw);
        if token.is_empty() || token.len() > MAX_TOKEN_LEN {
            results[index] = Some(refused(index, 403, "invalid or expired link"));
            continue;
        }
        let claims = match super::resolve(&state, &key, token, now).await {
            Ok(c) if c.k == Kind::Image && c.a == session.account_id && c.s == sid => c,
            _ => {
                results[index] = Some(refused(index, 403, "invalid or expired link"));
                continue;
            }
        };
        match &message {
            None => message = Some(claims.e.clone()),
            Some(first) if *first != claims.e => {
                results[index] = Some(refused(index, 403, "token is for another message"));
                continue;
            }
            Some(_) => {}
        }
        accepted.push((index, claims));
    }

    let Some(email) = message else {
        return respond(results);
    };
    let policy = match crate::remote_policy::effective_for_message(&state, &session, &email).await
    {
        Ok(policy) => std::sync::Arc::new(policy),
        Err(response) => return response,
    };
    // The grant and membership are checked once for the message. A refusal
    // answers every token and fetches nothing.
    let message_allowed = match image_allowed(&state, &session, &email, &policy).await {
        Ok(allowed) => allowed,
        Err(()) => return super::internal("remote-content grant check"),
    };
    // Blocked hosts never load; without a grant only trusted hosts do.
    let (accepted, denied): (Vec<_>, Vec<_>) = accepted.into_iter().partition(|(_, claims)| {
        !policy.host_blocked(&claims.u) && (message_allowed || policy.host_trusted(&claims.u))
    });
    for (index, claims) in &denied {
        results[*index] = Some(refused(
            *index,
            403,
            if policy.host_blocked(&claims.u) {
                "image host blocked by the user"
            } else {
                "remote content is not allowed for this message"
            },
        ));
    }
    if accepted.is_empty() {
        return respond(results);
    }

    let mut first_for_url: HashMap<String, usize> = HashMap::new();
    let mut duplicates: Vec<(usize, usize)> = Vec::new();
    let mut work: Vec<(usize, Claims)> = Vec::new();
    for (index, claims) in accepted {
        if policy.trackers && mw_sanitize::remote::is_tracker_url(&claims.u) {
            results[index] = Some(refused(index, 403, "tracker blocked by policy"));
            continue;
        }
        match first_for_url.get(&claims.u) {
            Some(&first) => duplicates.push((index, first)),
            None => {
                first_for_url.insert(claims.u.clone(), index);
                work.push((index, claims));
            }
        }
    }

    let parallel = policy
        .fetch
        .per_message
        .clamp(1, MAX_PARALLEL);
    let want_bodies = body.bodies;
    let fetched: Vec<(usize, Value, Option<Vec<u8>>)> = futures_util::stream::iter(work)
        .map(|(index, claims)| {
            let state = state.clone();
            let session = session.clone();
            let policy = policy.clone();
            async move {
                let response = fetch_image(state, session, claims, policy, HeaderMap::new()).await;
                describe(index, response, want_bodies).await
            }
        })
        .buffer_unordered(parallel)
        .collect()
        .await;

    let mut total = 0usize;
    for (index, mut entry, bytes) in fetched {
        if let Some(bytes) = bytes {
            if bytes.len() <= MAX_INLINE_BODY && total + bytes.len() <= MAX_INLINE_TOTAL {
                total += bytes.len();
                entry["body"] = Value::String(
                    base64::engine::general_purpose::STANDARD.encode(&bytes),
                );
            } else {
                entry["bodyOmitted"] = Value::Bool(true);
            }
        }
        results[index] = Some(entry);
    }
    for (index, first) in duplicates {
        if let Some(mut copy) = results[first].clone() {
            copy["index"] = json!(index);
            results[index] = Some(copy);
        }
    }
    respond(results)
}

fn respond(results: Vec<Option<Value>>) -> Response {
    let results: Vec<Value> = results
        .into_iter()
        .enumerate()
        .map(|(i, r)| r.unwrap_or_else(|| refused(i, 500, "no result")))
        .collect();
    let mut response = Json(json!({ "results": results })).into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("private, no-store"),
    );
    response
}

/// Reduce a single-URL response to its batch entry (and its bytes, if wanted).
async fn describe(
    index: usize,
    response: Response,
    want_body: bool,
) -> (usize, Value, Option<Vec<u8>>) {
    let status = response.status();
    let h = response.headers();
    let text = |name: &str| h.get(name).and_then(|v| v.to_str().ok()).map(str::to_owned);
    let mut entry = json!({ "index": index, "status": status.as_u16() });
    if let Some(v) = text("content-type") {
        entry["contentType"] = json!(v);
    }
    if let Some(v) = text("etag") {
        entry["etag"] = json!(v);
    }
    if let Some(v) = text("retry-after").and_then(|v| v.parse::<u64>().ok()) {
        entry["retryAfter"] = json!(v);
    }
    if text("x-mw-remote-retryable").as_deref() == Some("1") {
        entry["retryable"] = json!(true);
    }
    if text("x-mw-remote-content").as_deref() == Some("tracker") {
        entry["tracker"] = json!(true);
    }
    if !status.is_success() {
        entry["error"] = json!(
            status
                .canonical_reason()
                .unwrap_or("image could not be loaded")
        );
        return (index, entry, None);
    }
    if !want_body {
        return (index, entry, None);
    }
    match axum::body::to_bytes(response.into_body(), MAX_INLINE_BODY).await {
        Ok(bytes) => (index, entry, Some(bytes.to_vec())),
        Err(_) => {
            entry["bodyOmitted"] = Value::Bool(true);
            (index, entry, None)
        }
    }
}
