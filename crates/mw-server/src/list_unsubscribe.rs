//! RFC 8058 one-click unsubscribe, sent by the server so the reader's address and IP
//! never reach the list host (the same reason remote images go through the proxy).
//!
//! `POST /api/mail/list-unsubscribe {"url": "https://..."}` (session required) sends
//! the fixed body `List-Unsubscribe=One-Click` to an `https` URL through the egress
//! plan's `unsubscribe` class: the SSRF address gate, pinned DNS, no redirects followed,
//! no cookies, referrer or browser headers. The URL comes from the message's own
//! `List-Unsubscribe` header, which the web client has already checked against
//! `List-Unsubscribe-Post`; the body is never client-chosen, so this is not a general
//! POST relay. At most four requests run at once.
use crate::AppState;
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
};
use mw_egress::{Refusal, TrafficClass, plan};
use serde::Deserialize;
use serde_json::json;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

const BODY: &[u8] = b"List-Unsubscribe=One-Click";
const CONTENT_TYPE: &str = "application/x-www-form-urlencoded";
const MAX_URL: usize = 2048;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    url: String,
}

pub(crate) fn router() -> Router<AppState> {
    Router::new().route(
        "/api/mail/list-unsubscribe",
        post(unsubscribe).layer(DefaultBodyLimit::max(4096)),
    )
}

/// An absolute `https` URL with a host and no credentials, else `None`.
fn target(raw: &str) -> Option<reqwest::Url> {
    if raw.len() > MAX_URL || raw.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return None;
    }
    let url = reqwest::Url::parse(raw).ok()?;
    (url.scheme() == "https"
        && url.host_str().is_some()
        && url.username().is_empty()
        && url.password().is_none())
    .then_some(url)
}

fn failure(status: StatusCode, reason: &str) -> Response {
    (
        status,
        [(axum::http::header::CACHE_CONTROL, "private, no-store")],
        Json(json!({"ok": false, "reason": reason})),
    )
        .into_response()
}

async fn unsubscribe(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<Input>,
) -> Response {
    if let Err(response) = crate::authed(&state, &headers).await {
        return response;
    }
    let Some(url) = target(&input.url) else {
        return failure(StatusCode::BAD_REQUEST, "bad-url");
    };
    static SLOTS: OnceLock<Arc<tokio::sync::Semaphore>> = OnceLock::new();
    let slots = SLOTS.get_or_init(|| Arc::new(tokio::sync::Semaphore::new(4)));
    let Ok(_permit) = slots.clone().try_acquire_owned() else {
        return failure(StatusCode::TOO_MANY_REQUESTS, "busy");
    };
    let request = plan::Request::post(TrafficClass::Unsubscribe, CONTENT_TYPE, BODY)
        .response_limit(4096)
        .user_agent("Mailwoman");
    let sent = tokio::time::timeout(Duration::from_secs(10), plan::send(url, &request)).await;
    let (status, reason) = match sent.map(|r| r.outcome) {
        Ok(Ok(_)) => {
            return (
                [(axum::http::header::CACHE_CONTROL, "private, no-store")],
                Json(json!({"ok": true})),
            )
                .into_response();
        }
        Err(_) | Ok(Err(Refusal::Timeout)) => (StatusCode::GATEWAY_TIMEOUT, "timeout"),
        Ok(Err(Refusal::Blocked | Refusal::BadRequest(_))) => (StatusCode::BAD_GATEWAY, "refused"),
        Ok(Err(_)) => (StatusCode::BAD_GATEWAY, "rejected"),
    };
    failure(status, reason)
}

#[cfg(test)]
mod tests {
    use super::target;

    #[test]
    fn only_plain_https_urls_are_targets() {
        assert!(target("https://list.example.org/leave?t=1").is_some());
        assert!(target("http://list.example.org/leave").is_none());
        assert!(target("https://user:pw@list.example.org/").is_none());
        assert!(target("https://list.example.org/a b").is_none());
        assert!(target("mailto:leave@example.org").is_none());
        assert!(target(&format!("https://e.org/{}", "a".repeat(3000))).is_none());
    }
}
