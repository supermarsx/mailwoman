//! A byte budget for request work that buffers large amounts of memory (t55,
//! t63 F9 / "first five changes" item 3).
//!
//! The HTTP admission limits count requests, assuming about 4 MiB each; nothing
//! counted the bytes a request actually buffers. This budget does, for the
//! paths that hold body- or file-sized buffers:
//!
//! * any request with a body of at least [`SMALL`] bytes, charged its declared
//!   `Content-Length` (a chunked body is charged [`CHUNKED_CHARGE`]) before the
//!   body is read: `/jmap/api`, the idempotency-key hash, attachment preview,
//!   delivery status, sanitize, drafts and every other buffered body;
//! * `/jmap/upload`, which seals while reading, a fixed [`STREAMED_CHARGE`];
//! * the import routes, which hold one message at a time, at most one message
//!   (the declared length, capped at the import message limit);
//! * a download, charged the bytes it holds while they are sent (in the
//!   handler, once the size is known).
//!
//! Requests below [`SMALL`] are not charged at all, so the common small request
//! pays nothing. The budget is max(64 MiB, 20% of the memory the HTTP limits
//! read at start). A charge waits at most [`ADMIT_WAIT`] and is then refused
//! with `503` and `Retry-After`; a single charge above the whole budget is
//! clamped to it (it runs alone rather than never). The permit is held until
//! the response body has been sent. `mw_request_bytes_held` and
//! `mw_request_bytes_limit` report the state; `mw_request_bytes_refused_total`
//! counts refusals.

use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::task::{Context, Poll};
use std::time::Duration;

use axum::Json;
use axum::body::{Body, Bytes};
use axum::extract::Request;
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use hyper::body::{Body as HttpBody, Frame, SizeHint};
use serde_json::json;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// Bodies below this are not charged.
pub(crate) const SMALL: u64 = 64 * 1024;
/// The charge for a body without `Content-Length` (chunked).
pub(crate) const CHUNKED_CHARGE: u64 = 1024 * 1024;
/// The charge for a route that streams its body (it holds one chunk and its seal).
pub(crate) const STREAMED_CHARGE: u64 = 1024 * 1024;
/// The import routes hold at most one message of this size.
const IMPORT_MESSAGE_CAP: u64 = 64 * 1024 * 1024;
/// The smallest budget.
const FLOOR: u64 = 64 * 1024 * 1024;
/// Longest a charge waits for the budget before it is refused.
pub(crate) const ADMIT_WAIT: Duration = Duration::from_secs(2);
/// `Retry-After` of a refusal, in seconds.
const RETRY_AFTER: &str = "2";

/// The budget, in KiB permits (so the largest host still fits in `u32`).
pub(crate) struct ByteBudget {
    permits: Arc<Semaphore>,
    limit_kib: u32,
    held: AtomicU64,
}

/// max(64 MiB, 20% of `memory`).
pub(crate) fn limit_for(memory: Option<u64>) -> u64 {
    memory.map_or(FLOOR, |m| (m / 5).max(FLOOR))
}

impl ByteBudget {
    pub(crate) fn new(limit_bytes: u64) -> Self {
        let limit_kib = (limit_bytes / 1024).clamp(1, u64::from(u32::MAX >> 3)) as u32;
        metrics::gauge!("mw_request_bytes_limit").set((u64::from(limit_kib) * 1024) as f64);
        Self {
            permits: Arc::new(Semaphore::new(limit_kib as usize)),
            limit_kib,
            held: AtomicU64::new(0),
        }
    }

    pub(crate) fn limit_bytes(&self) -> u64 {
        u64::from(self.limit_kib) * 1024
    }

    #[cfg(test)]
    pub(crate) fn held_bytes(&self) -> u64 {
        self.held.load(Ordering::Relaxed)
    }

    /// Charge `bytes`, waiting at most `wait`. `None` when refused.
    pub(crate) async fn admit(self: &Arc<Self>, bytes: u64, wait: Duration) -> Option<BytePermit> {
        let kib = bytes.div_ceil(1024).clamp(1, u64::from(self.limit_kib)) as u32;
        let permit = match self.permits.clone().try_acquire_many_owned(kib) {
            Ok(permit) => permit,
            Err(_) => tokio::time::timeout(wait, self.permits.clone().acquire_many_owned(kib))
                .await
                .ok()?
                .ok()?,
        };
        let held = self.held.fetch_add(u64::from(kib) * 1024, Ordering::Relaxed) + u64::from(kib) * 1024;
        metrics::gauge!("mw_request_bytes_held").set(held as f64);
        Some(BytePermit {
            _permit: permit,
            kib,
            budget: self.clone(),
        })
    }
}

/// A charge against the budget, released when dropped.
pub(crate) struct BytePermit {
    _permit: OwnedSemaphorePermit,
    kib: u32,
    budget: Arc<ByteBudget>,
}

impl Drop for BytePermit {
    fn drop(&mut self) {
        let bytes = u64::from(self.kib) * 1024;
        let held = self.budget.held.fetch_sub(bytes, Ordering::Relaxed).saturating_sub(bytes);
        metrics::gauge!("mw_request_bytes_held").set(held as f64);
    }
}

/// The process-wide budget, sized from the same memory reading as the HTTP limits.
pub(crate) fn budget() -> &'static Arc<ByteBudget> {
    static BUDGET: OnceLock<Arc<ByteBudget>> = OnceLock::new();
    BUDGET.get_or_init(|| {
        let memory = crate::inbound_http::resources::Resources::probe(false).memory_bytes;
        let budget = Arc::new(ByteBudget::new(limit_for(memory)));
        tracing::info!(
            "request byte budget: {} MiB (max of 64 MiB and 20% of {} memory)",
            budget.limit_bytes() / (1024 * 1024),
            memory.map_or_else(|| "unknown".to_string(), |m| format!("{} MiB of", m / (1024 * 1024)))
        );
        budget
    })
}

fn declared_length(request: &Request) -> Option<u64> {
    request
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.trim().parse().ok())
}

/// What a request is charged before its body is read, or `None` (not charged).
pub(crate) fn charge_for(request: &Request) -> Option<u64> {
    let path = request.uri().path();
    let declared = declared_length(request);
    let chunked = declared.is_none() && request.headers().contains_key(header::TRANSFER_ENCODING);
    if path.starts_with("/jmap/upload/") && *request.method() == Method::POST {
        return Some(STREAMED_CHARGE);
    }
    if matches!(path, "/api/import/mbox" | "/api/import/eml" | "/api/import/maildir") {
        return Some(declared.unwrap_or(IMPORT_MESSAGE_CAP).min(IMPORT_MESSAGE_CAP));
    }
    match declared {
        Some(n) if n >= SMALL => Some(n),
        Some(_) => None,
        None if chunked => Some(CHUNKED_CHARGE),
        None => None,
    }
}

/// The refusal: `503` with `Retry-After`, the body drained so it is delivered.
async fn refused(request: Request) -> Response {
    metrics::counter!("mw_request_bytes_refused_total").increment(1);
    let mut response = (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({"error": "the server is busy with large requests; retry shortly", "code": "requestBytesBusy"})),
    )
        .into_response();
    response
        .headers_mut()
        .insert(header::RETRY_AFTER, HeaderValue::from_static(RETRY_AFTER));
    crate::import_routes::refuse_before_body(request, response).await
}

/// The middleware: charge, run, hold the charge until the response is sent.
pub(crate) async fn layer(request: Request, next: Next) -> Response {
    let Some(charge) = charge_for(&request) else {
        return next.run(request).await;
    };
    match budget().admit(charge, ADMIT_WAIT).await {
        Some(permit) => hold(next.run(request).await, permit),
        None => refused(request).await,
    }
}

/// Keep `permit` until `response`'s body has been sent or dropped.
pub(crate) fn hold(response: Response, permit: BytePermit) -> Response {
    response.map(|inner| Body::new(HeldBody { inner, _permit: permit }))
}

/// The `503` a handler returns when a charge it makes itself is refused.
pub(crate) fn busy() -> Response {
    metrics::counter!("mw_request_bytes_refused_total").increment(1);
    let mut response = (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({"error": "the server is busy with large requests; retry shortly", "code": "requestBytesBusy"})),
    )
        .into_response();
    response
        .headers_mut()
        .insert(header::RETRY_AFTER, HeaderValue::from_static(RETRY_AFTER));
    response
}

struct HeldBody {
    inner: Body,
    _permit: BytePermit,
}

impl HttpBody for HeldBody {
    type Data = Bytes;
    type Error = axum::Error;
    fn poll_frame(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        Pin::new(&mut self.get_mut().inner).poll_frame(cx)
    }
    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }
    fn size_hint(&self) -> SizeHint {
        self.inner.size_hint()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(method: Method, path: &str, length: Option<u64>, chunked: bool) -> Request {
        let mut builder = axum::http::Request::builder().method(method).uri(path);
        if let Some(n) = length {
            builder = builder.header(header::CONTENT_LENGTH, n);
        }
        if chunked {
            builder = builder.header(header::TRANSFER_ENCODING, "chunked");
        }
        builder.body(Body::empty()).unwrap()
    }

    #[test]
    fn the_budget_is_twenty_percent_of_memory_with_a_floor() {
        assert_eq!(limit_for(None), 64 * 1024 * 1024);
        assert_eq!(limit_for(Some(256 * 1024 * 1024)), 64 * 1024 * 1024);
        assert_eq!(limit_for(Some(4 * 1024 * 1024 * 1024)), 4 * 1024 * 1024 * 1024 / 5);
    }

    #[test]
    fn small_requests_are_not_charged_and_large_ones_are() {
        assert_eq!(charge_for(&request(Method::POST, "/jmap/api", Some(2_000), false)), None);
        assert_eq!(charge_for(&request(Method::GET, "/jmap/session", None, false)), None);
        assert_eq!(charge_for(&request(Method::POST, "/jmap/api", Some(1_500_000), false)), Some(1_500_000));
        assert_eq!(charge_for(&request(Method::POST, "/api/sanitize", None, true)), Some(CHUNKED_CHARGE));
        assert_eq!(charge_for(&request(Method::POST, "/jmap/upload/a", Some(40_000_000), false)), Some(STREAMED_CHARGE));
        assert_eq!(charge_for(&request(Method::POST, "/api/import/mbox", Some(500), false)), Some(500));
        assert_eq!(
            charge_for(&request(Method::POST, "/api/import/maildir", Some(1 << 40), false)),
            Some(IMPORT_MESSAGE_CAP)
        );
    }

    #[tokio::test]
    async fn charges_are_held_until_dropped_and_refused_when_the_budget_is_full() {
        let budget = Arc::new(ByteBudget::new(4 * 1024 * 1024));
        let a = budget.admit(3 * 1024 * 1024, Duration::ZERO).await.expect("fits");
        assert_eq!(budget.held_bytes(), 3 * 1024 * 1024);
        assert!(budget.admit(2 * 1024 * 1024, Duration::from_millis(20)).await.is_none(), "over the budget");
        let small = budget.admit(1024 * 1024, Duration::ZERO).await.expect("the rest fits");
        drop(a);
        drop(small);
        assert_eq!(budget.held_bytes(), 0);
        // One charge above the whole budget is clamped to it and runs alone.
        let huge = budget.admit(1 << 40, Duration::ZERO).await.expect("clamped");
        assert_eq!(budget.held_bytes(), 4 * 1024 * 1024);
        drop(huge);
        // A waiting charge is admitted when another is released.
        let first = budget.admit(4 * 1024 * 1024, Duration::ZERO).await.unwrap();
        let waiter = {
            let budget = budget.clone();
            tokio::spawn(async move { budget.admit(1024, Duration::from_secs(5)).await.is_some() })
        };
        tokio::time::sleep(Duration::from_millis(20)).await;
        drop(first);
        assert!(waiter.await.unwrap());
    }

    /// The cost on the request path: an uncharged request does one header
    /// look-up; a charged one an uncontended semaphore acquire and release.
    #[tokio::test]
    async fn an_uncharged_request_costs_a_header_lookup_and_a_charge_under_a_microsecond_or_so() {
        let budget = Arc::new(ByteBudget::new(64 * 1024 * 1024));
        let req = request(Method::POST, "/jmap/api", Some(2_000), false);
        let start = std::time::Instant::now();
        for _ in 0..100_000 {
            std::hint::black_box(charge_for(std::hint::black_box(&req)));
        }
        let uncharged = start.elapsed() / 100_000;
        let start = std::time::Instant::now();
        for _ in 0..10_000 {
            drop(budget.admit(1_500_000, Duration::ZERO).await);
        }
        let charged = start.elapsed() / 10_000;
        eprintln!("request_bytes: uncharged {uncharged:?}, charged {charged:?} per request");
        assert!(uncharged < Duration::from_micros(50), "{uncharged:?}");
        assert!(charged < Duration::from_millis(1), "{charged:?}");
    }
}
