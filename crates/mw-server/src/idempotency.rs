//! `Idempotency-Key` for user-facing writes (t91).
//!
//! The web client retries a write after a transient failure (`api/recovery.ts`).
//! A write that is not idempotent on its own (a create, a link with a password)
//! is retried only when it carries an `Idempotency-Key` header: the same key on
//! every attempt of that one user action. This layer makes such a retry safe.
//!
//!   - A request without the header passes through untouched.
//!   - The record is keyed by the caller's credential (a hash of the session
//!     cookie or the bearer token, and of the linked-mailbox selector), the
//!     method, the path and the key. Another session sending the same key gets
//!     its own record, never someone else's answer.
//!   - The first request with a key runs. A 2xx answer is kept for
//!     [`TTL`] (status, headers except `Set-Cookie`, body up to
//!     [`MAX_STORED_BODY`]); a repeat with the same body gets that answer back
//!     with `Idempotent-Replayed: true` and runs nothing. Any other answer is
//!     not kept, so a retry after a refusal or a server error runs again (and
//!     the key may then carry another body).
//!   - A repeat that arrives while the first is still running waits for it (at
//!     most [`IN_FLIGHT_WAIT`]) and then gets its answer.
//!   - The same key with a different body is refused `422
//!     idempotencyKeyReused`: a key names one request, not a slot.
//!
//! The store is in memory and per process, bounded by [`MAX_ENTRIES`] (expired
//! records first, then the oldest, are dropped). It protects the client's
//! retries of one action within minutes, which reach the process that answered
//! (or failed to answer) the first attempt behind a sticky or single-instance
//! deployment; it is not a durable or cluster-wide ledger.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use axum::Json;
use axum::body::{Body, Bytes};
use axum::extract::Request;
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use sha2::{Digest, Sha256};

/// The request header.
pub(crate) const HEADER: &str = "idempotency-key";
/// Set on an answer that was replayed from the store.
pub(crate) const REPLAYED: &str = "idempotent-replayed";
/// How long a completed answer is kept.
pub(crate) const TTL: Duration = Duration::from_secs(10 * 60);
/// How long a repeat waits for the first request with its key to finish.
pub(crate) const IN_FLIGHT_WAIT: Duration = Duration::from_secs(30);
/// Records kept at once.
pub(crate) const MAX_ENTRIES: usize = 20_000;
/// Largest request body hashed (anything larger is not deduplicated).
const MAX_REQUEST_BODY: usize = 16 * 1024 * 1024;
/// Largest answer body kept for a replay.
pub(crate) const MAX_STORED_BODY: usize = 1024 * 1024;

/// A stored answer.
#[derive(Clone)]
struct Stored {
    status: StatusCode,
    headers: Vec<(HeaderName, HeaderValue)>,
    body: Bytes,
}

/// One key's record. The async mutex serialises requests with the same key.
struct Slot {
    request_hash: [u8; 32],
    created: Instant,
    answer: tokio::sync::Mutex<Option<Stored>>,
}

#[derive(Default)]
struct Ledger {
    slots: HashMap<[u8; 32], Arc<Slot>>,
}

fn ledger() -> &'static Mutex<Ledger> {
    static LEDGER: OnceLock<Mutex<Ledger>> = OnceLock::new();
    LEDGER.get_or_init(Mutex::default)
}

/// A key the client may send: 1 to 128 visible ASCII characters.
pub(crate) fn valid_key(key: &str) -> bool {
    !key.is_empty() && key.len() <= 128 && key.bytes().all(|b| b.is_ascii_graphic())
}

/// The credential the record belongs to: the session cookie or bearer token,
/// with the linked-mailbox selector. `None` for an anonymous request, which is
/// not deduplicated.
fn owner(headers: &HeaderMap) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    if let Some(token) = crate::push_relay::bearer_token(headers) {
        out.extend_from_slice(b"bearer\0");
        out.extend_from_slice(token.as_bytes());
    } else {
        let cookie = crate::cookie_value(headers)?;
        out.extend_from_slice(b"cookie\0");
        out.extend_from_slice(cookie.as_bytes());
    }
    for name in [crate::linked_accounts::SELECTOR, "x-mailwoman-account"] {
        out.push(0);
        if let Some(value) = headers.get(name) {
            out.extend_from_slice(value.as_bytes());
        }
    }
    Some(out)
}

fn record_id(owner: &[u8], method: &str, path: &str, key: &str) -> [u8; 32] {
    let mut hash = Sha256::new();
    for part in [owner, method.as_bytes(), path.as_bytes(), key.as_bytes()] {
        hash.update((part.len() as u64).to_be_bytes());
        hash.update(part);
    }
    hash.finalize().into()
}

/// The slot for `id`, made when there is none (or the one there expired or was
/// made for another body: then `Err(())` when it is still live).
fn slot(id: [u8; 32], request_hash: [u8; 32], now: Instant) -> Result<Arc<Slot>, ()> {
    let mut ledger = ledger().lock().unwrap_or_else(|e| e.into_inner());
    if let Some(existing) = ledger.slots.get(&id)
        && now.duration_since(existing.created) < TTL
    {
        if existing.request_hash == request_hash {
            return Ok(existing.clone());
        }
        // Another body under a live key is refused, unless the key's request
        // did not succeed (nothing kept, nothing running): then the key is free.
        let free = existing
            .answer
            .try_lock()
            .is_ok_and(|answer| answer.is_none());
        if !free {
            return Err(());
        }
    }
    if ledger.slots.len() >= MAX_ENTRIES {
        ledger
            .slots
            .retain(|_, slot| now.duration_since(slot.created) < TTL);
        while ledger.slots.len() >= MAX_ENTRIES {
            let Some(oldest) = ledger
                .slots
                .iter()
                .min_by_key(|(_, slot)| slot.created)
                .map(|(id, _)| *id)
            else {
                break;
            };
            ledger.slots.remove(&oldest);
        }
    }
    let fresh = Arc::new(Slot {
        request_hash,
        created: now,
        answer: tokio::sync::Mutex::new(None),
    });
    ledger.slots.insert(id, fresh.clone());
    Ok(fresh)
}

fn refusal(status: StatusCode, code: &str, error: &str) -> Response {
    (status, Json(json!({ "error": error, "code": code }))).into_response()
}

fn replay(stored: &Stored) -> Response {
    let mut response = Response::new(Body::from(stored.body.clone()));
    *response.status_mut() = stored.status;
    for (name, value) in &stored.headers {
        response.headers_mut().append(name.clone(), value.clone());
    }
    response
        .headers_mut()
        .insert(REPLAYED, HeaderValue::from_static("true"));
    response
}

/// The middleware. Innermost in the router (after the origin, CSRF and linked
/// guards), so a request it sees has passed them.
pub(crate) async fn layer(request: Request, next: Next) -> Response {
    let Some(raw) = request.headers().get(HEADER) else {
        return next.run(request).await;
    };
    let Some(key) = raw
        .to_str()
        .ok()
        .filter(|k| valid_key(k))
        .map(str::to_owned)
    else {
        return refusal(
            StatusCode::BAD_REQUEST,
            "invalidIdempotencyKey",
            "Idempotency-Key must be 1 to 128 visible ASCII characters",
        );
    };
    let Some(owner) = owner(request.headers()) else {
        return next.run(request).await;
    };
    let method = request.method().as_str().to_owned();
    let path = request.uri().path().to_owned();
    let (parts, body) = request.into_parts();
    let bytes = match axum::body::to_bytes(body, MAX_REQUEST_BODY).await {
        Ok(bytes) => bytes,
        Err(_) => {
            return refusal(
                StatusCode::PAYLOAD_TOO_LARGE,
                "requestTooLarge",
                "the request body is too large",
            );
        }
    };
    let request_hash: [u8; 32] = Sha256::digest(&bytes).into();
    let id = record_id(&owner, &method, &path, &key);
    let Ok(slot) = slot(id, request_hash, Instant::now()) else {
        return refusal(
            StatusCode::UNPROCESSABLE_ENTITY,
            "idempotencyKeyReused",
            "this Idempotency-Key was used for a different request",
        );
    };
    let Ok(mut answer) = tokio::time::timeout(IN_FLIGHT_WAIT, slot.answer.lock()).await else {
        return refusal(
            StatusCode::CONFLICT,
            "idempotencyKeyInFlight",
            "a request with this Idempotency-Key is still running",
        );
    };
    if let Some(stored) = answer.as_ref() {
        return replay(stored);
    }
    let response = next
        .run(Request::from_parts(parts, Body::from(bytes)))
        .await;
    if !response.status().is_success() {
        // Not kept: the slot stays empty, so a retry runs again.
        return response;
    }
    let (parts, body) = response.into_parts();
    let body = match axum::body::to_bytes(body, MAX_STORED_BODY).await {
        Ok(body) => body,
        Err(_) => {
            // Too large to keep. The write happened; a repeat cannot be answered
            // with its result, so it is refused rather than run twice.
            *answer = Some(Stored {
                status: StatusCode::CONFLICT,
                headers: vec![(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/json"),
                )],
                body: Bytes::from_static(
                    br#"{"error":"the answer to this request is too large to replay","code":"idempotencyReplayUnavailable"}"#,
                ),
            });
            return refusal(
                StatusCode::BAD_GATEWAY,
                "idempotencyReplayUnavailable",
                "the answer to this request is too large to return",
            );
        }
    };
    let stored = Stored {
        status: parts.status,
        headers: parts
            .headers
            .iter()
            .filter(|(name, _)| *name != header::SET_COOKIE && *name != header::CONTENT_LENGTH)
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect(),
        body: body.clone(),
    };
    *answer = Some(stored);
    Response::from_parts(parts, Body::from(body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_short_visible_ascii() {
        assert!(valid_key("6f1c0d2e-8a3b-4c5d-9e0f-112233445566"));
        assert!(!valid_key(""));
        assert!(!valid_key("has space"));
        assert!(!valid_key(&"k".repeat(129)));
        assert!(!valid_key("caf\u{e9}"));
    }

    #[test]
    fn a_record_is_per_owner_method_path_and_key() {
        let a = record_id(b"cookie\0one", "POST", "/jmap/api", "k");
        assert_ne!(a, record_id(b"cookie\0two", "POST", "/jmap/api", "k"));
        assert_ne!(a, record_id(b"cookie\0one", "PUT", "/jmap/api", "k"));
        assert_ne!(
            a,
            record_id(b"cookie\0one", "POST", "/api/accounts/link", "k")
        );
        assert_ne!(a, record_id(b"cookie\0one", "POST", "/jmap/api", "k2"));
        // Length-prefixed: shifting bytes between parts is another record.
        assert_ne!(
            record_id(b"ab", "c", "/p", "k"),
            record_id(b"a", "bc", "/p", "k")
        );
    }

    #[test]
    fn the_same_key_with_another_body_is_refused_until_it_expires() {
        let id = record_id(b"cookie\0slot-test", "POST", "/t", "k");
        let now = Instant::now();
        let first = slot(id, [1; 32], now).unwrap();
        assert!(Arc::ptr_eq(&first, &slot(id, [1; 32], now).unwrap()));
        // Its request succeeded (an answer is kept): another body is refused.
        *first.answer.try_lock().unwrap() = Some(Stored {
            status: StatusCode::OK,
            headers: Vec::new(),
            body: Bytes::new(),
        });
        assert!(slot(id, [2; 32], now).is_err());
        let later = now + TTL + Duration::from_secs(1);
        let renewed = slot(id, [2; 32], later).unwrap();
        assert!(!Arc::ptr_eq(&first, &renewed));
    }

    #[test]
    fn a_key_whose_request_did_not_succeed_takes_another_body() {
        let id = record_id(b"cookie\0failed", "POST", "/t", "k");
        let now = Instant::now();
        let first = slot(id, [1; 32], now).unwrap();
        // Running: the key is taken.
        let running = first.answer.try_lock().unwrap();
        assert!(slot(id, [2; 32], now).is_err());
        drop(running);
        // Finished without a kept answer: free for a corrected request.
        let next = slot(id, [2; 32], now).unwrap();
        assert!(!Arc::ptr_eq(&first, &next));
        // Finished with a kept answer: taken.
        *next.answer.try_lock().unwrap() = Some(Stored {
            status: StatusCode::OK,
            headers: Vec::new(),
            body: Bytes::new(),
        });
        assert!(slot(id, [3; 32], now).is_err());
        assert!(Arc::ptr_eq(&next, &slot(id, [2; 32], now).unwrap()));
    }
}
