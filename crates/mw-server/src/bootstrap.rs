//! `GET /api/bootstrap` (t52w): the mail app's load-time reads in one response.
//!
//! Each section is the answer of one existing `GET` route to an internal
//! request that carries the caller's headers. The internal request goes through
//! the same router, with every layer, as a direct call would: it is sent through
//! [`Dispatcher`] (the application router the WebSocket actions use) from a
//! freshly spawned task, so no task-local frame of the outer request (request
//! inputs, guard context, request protection, measurement) is visible to it,
//! exactly as for a request arriving on its own connection. Each route therefore
//! runs its own guards and handler checks, and a section the caller may not read
//! carries that route's own status and error body, never its data.
//!
//! The set of routes is the fixed list [`SECTIONS`]. Nothing in the outer
//! request (query string, headers, body) selects or changes a path; the one
//! query value sent (`accountId` for the appearance customization) is taken from
//! the `me` section's answer for the same caller and percent-encoded.
//!
//! Sub-requests are counted by request protection, metrics and every other
//! layer like the direct calls they replace; the outer request is counted too.

use crate::websocket_actions::Dispatcher;
use axum::{
    body::{Body, Bytes, to_bytes},
    extract::{ConnectInfo, Request},
    http::{HeaderMap, HeaderValue, Method, StatusCode, Uri, Version, header, uri::PathAndQuery},
    response::{IntoResponse, Response},
};
use std::net::SocketAddr;
use tower::ServiceExt;

/// Format version of the response document.
pub(crate) const VERSION: u32 = 1;
/// A section body larger than this is answered as `{"body":null,"error":"bodyTooLarge"}`.
pub(crate) const SECTION_BODY_LIMIT: usize = 4 * 1024 * 1024;
/// Sub-requests in flight at once after `me` (a browser opens about six).
const PARALLEL: usize = 4;
const VARY: &str = "Cookie";
const CACHE_CONTROL: &str = "private, no-cache";

/// How a section's request target is built.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Target {
    /// The path, no query.
    Path,
    /// The path plus `?accountId=<me.accountId>` when the `me` section
    /// answered 200 with a string `accountId`; otherwise the path alone.
    WithMeAccountId,
}

#[derive(Debug)]
pub(crate) struct Section {
    pub name: &'static str,
    pub path: &'static str,
    pub target: Target,
}

const fn section(name: &'static str, path: &'static str) -> Section {
    Section { name, path, target: Target::Path }
}

/// The sections, in response order. `me` is first and is dispatched alone,
/// before the others, because one section's target depends on its answer.
pub(crate) const SECTIONS: &[Section] = &[
    section("me", "/api/me"),
    section("jmapSession", "/jmap/session"),
    section("publicConfig", "/api/public-config"),
    section("cookiesConfig", "/api/cookies/config"),
    section("accounts", "/api/accounts"),
    section("settings", "/api/settings"),
    section("appearance", "/api/account/appearance"),
    section("preferences", "/api/account/preferences"),
    section("features", "/api/features"),
    section("notifications", "/api/account/notifications"),
    section("policy", "/api/policy"),
    section("restriction", "/api/privacy/restriction"),
    section("uiPlugins", "/api/ui-plugins"),
    section("assistConfig", "/api/assist/config"),
    section("version", "/api/version"),
    section("focusedSenders", "/api/account/focused-senders"),
    section("drafts", "/api/account/drafts"),
    section("notificationCenter", "/api/account/notification-center"),
    Section {
        name: "appearanceCustomization",
        path: "/api/account/appearance/customization",
        target: Target::WithMeAccountId,
    },
];

/// Outer request headers that are not passed to the sections: validators and
/// ranges (each section answers in full; the combined tag is checked here),
/// body framing (sections are bodiless GETs), hop-by-hop fields, and
/// `accept-encoding` (section bodies are embedded, never re-encoded).
const DROPPED_HEADERS: &[&str] = &[
    "if-none-match",
    "if-match",
    "if-modified-since",
    "if-unmodified-since",
    "if-range",
    "range",
    "content-length",
    "content-type",
    "content-encoding",
    "transfer-encoding",
    "expect",
    "idempotency-key",
    "accept-encoding",
    "connection",
    "keep-alive",
    "proxy-connection",
    "te",
    "trailer",
    "upgrade",
];

/// Percent-encode a query value (RFC 3986 unreserved characters kept).
fn encode_query_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// The path and query a section is requested at. The path is always the
/// section's fixed path.
pub(crate) fn path_and_query(section: &Section, me_account: Option<&str>) -> PathAndQuery {
    let text = match (section.target, me_account) {
        (Target::WithMeAccountId, Some(account)) => {
            format!("{}?accountId={}", section.path, encode_query_value(account))
        }
        _ => section.path.to_owned(),
    };
    // Every byte is either from a static path or percent-encoded.
    PathAndQuery::try_from(text).unwrap_or_else(|_| PathAndQuery::from_static(section.path))
}

/// The request a section is answered from. Its target mirrors the outer
/// request's form (absolute for HTTP/2, origin-form otherwise) and version, so
/// the protocol guard sees what it would for a direct call.
pub(crate) fn sub_request(
    section: &Section,
    me_account: Option<&str>,
    outer_uri: &Uri,
    version: Version,
    headers: &HeaderMap,
    peer: Option<ConnectInfo<SocketAddr>>,
) -> Request {
    let pq = path_and_query(section, me_account);
    let uri = match (outer_uri.scheme(), outer_uri.authority()) {
        (Some(scheme), Some(authority)) => Uri::builder()
            .scheme(scheme.clone())
            .authority(authority.clone())
            .path_and_query(pq.clone())
            .build()
            .unwrap_or_else(|_| Uri::from(pq)),
        _ => Uri::from(pq),
    };
    let mut request = Request::new(Body::empty());
    *request.method_mut() = Method::GET;
    *request.uri_mut() = uri;
    *request.version_mut() = version;
    let out = request.headers_mut();
    for (name, value) in headers {
        if !DROPPED_HEADERS.contains(&name.as_str()) {
            out.append(name.clone(), value.clone());
        }
    }
    if let Some(peer) = peer {
        request.extensions_mut().insert(peer);
    }
    request
}

/// One section's answer.
pub(crate) struct Outcome {
    pub status: StatusCode,
    pub etag: Option<String>,
    pub json: bool,
    /// `None`: the body exceeded [`SECTION_BODY_LIMIT`] or failed to read.
    pub body: Option<Bytes>,
    pub set_cookie: Vec<HeaderValue>,
}

async fn collect(response: Response) -> Outcome {
    let status = response.status();
    let headers = response.headers();
    let etag = headers
        .get(header::ETAG)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);
    let json = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| {
            let v = v.trim_start().to_ascii_lowercase();
            v.starts_with("application/json") || v.starts_with("application/problem+json")
        });
    let set_cookie = headers.get_all(header::SET_COOKIE).iter().cloned().collect();
    let body = to_bytes(response.into_body(), SECTION_BODY_LIMIT).await.ok();
    Outcome { status, etag, json, body, set_cookie }
}

/// Send one sub-request through the router from a fresh task.
async fn dispatch(router: axum::Router, request: Request) -> Outcome {
    let response = router.oneshot(request).await.unwrap_or_else(|never| match never {});
    collect(response).await
}

fn strong_hash(bytes: &[u8]) -> String {
    use base64::Engine;
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&digest[..16])
}

/// The section's tag: the route's own `ETag` when it sets one, else a strong
/// hash of the body bytes (same scheme as `private_etag`).
fn section_etag(outcome: &Outcome) -> Option<String> {
    if let Some(etag) = &outcome.etag {
        return Some(etag.clone());
    }
    outcome.body.as_ref().map(|b| format!("\"{}\"", strong_hash(b)))
}

/// The `me` section's account id, used for the one dependent target.
fn me_account(outcome: &Outcome) -> Option<String> {
    if outcome.status != StatusCode::OK || !outcome.json {
        return None;
    }
    let value: serde_json::Value = serde_json::from_slice(outcome.body.as_ref()?).ok()?;
    value.get("accountId")?.as_str().map(str::to_owned)
}

/// The `me` section's CSRF cookie, the only `Set-Cookie` forwarded. Cookies
/// that layers set on a sub-response are set by the same layers on the outer
/// response, so they are not copied.
pub(crate) fn forwarded_cookies(name: &str, outcome: &Outcome) -> Vec<HeaderValue> {
    if name != "me" {
        return Vec::new();
    }
    let prefix = format!("{}=", crate::hardening::CSRF_COOKIE);
    outcome
        .set_cookie
        .iter()
        .filter(|v| v.to_str().is_ok_and(|s| s.trim_start().starts_with(&prefix)))
        .cloned()
        .collect()
}

/// The response document, built from raw bytes so each JSON body is embedded
/// exactly as the route sent it. Returns the document and the combined tag
/// (`None` when a section has no usable body).
pub(crate) fn document(outcomes: &[(&'static str, Outcome)]) -> (Vec<u8>, Option<String>) {
    use sha2::{Digest, Sha256};
    let mut out = Vec::with_capacity(
        64 + outcomes
            .iter()
            .map(|(_, o)| o.body.as_ref().map_or(0, Bytes::len) + 96)
            .sum::<usize>(),
    );
    let mut combined = Sha256::new();
    combined.update(format!("bootstrap-v{VERSION}\n").as_bytes());
    let mut complete = true;
    out.extend_from_slice(format!("{{\"v\":{VERSION},\"sections\":{{").as_bytes());
    for (index, (name, outcome)) in outcomes.iter().enumerate() {
        if index > 0 {
            out.push(b',');
        }
        let etag = section_etag(outcome);
        let quote = |s: &str| serde_json::to_string(s).unwrap_or_else(|_| "\"\"".into());
        out.extend_from_slice(quote(name).as_bytes());
        out.extend_from_slice(
            format!(
                ":{{\"status\":{},\"etag\":{},\"body\":",
                outcome.status.as_u16(),
                etag.as_deref().map_or_else(|| "null".to_owned(), quote)
            )
            .as_bytes(),
        );
        match &outcome.body {
            None => {
                complete = false;
                out.extend_from_slice(b"null,\"error\":\"bodyTooLarge\"");
            }
            Some(body) if body.is_empty() => out.extend_from_slice(b"null"),
            Some(body)
                if outcome.json
                    && serde_json::from_slice::<serde::de::IgnoredAny>(body).is_ok() =>
            {
                out.extend_from_slice(body);
            }
            Some(body) => out.extend_from_slice(quote(&String::from_utf8_lossy(body)).as_bytes()),
        }
        out.push(b'}');
        combined.update(name.as_bytes());
        combined.update(b"\0");
        combined.update(outcome.status.as_u16().to_string().as_bytes());
        combined.update(b"\0");
        combined.update(etag.as_deref().unwrap_or("").as_bytes());
        combined.update(b"\n");
    }
    out.extend_from_slice(b"}}");
    let tag = complete.then(|| {
        use base64::Engine;
        let digest = combined.finalize();
        format!(
            "\"bs{VERSION}-{}\"",
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&digest[..16])
        )
    });
    (out, tag)
}

/// Does `If-None-Match` name `etag` (or `*`)? A weak form matches too.
fn none_match(raw: Option<&HeaderValue>, etag: &str) -> bool {
    let Some(raw) = raw.and_then(|v| v.to_str().ok()) else {
        return false;
    };
    raw.split(',').any(|candidate| {
        let candidate = candidate.trim();
        let candidate = candidate.strip_prefix("W/").unwrap_or(candidate);
        candidate == "*" || candidate == etag
    })
}

/// Run every section for this request, `me` first, the rest at most
/// [`PARALLEL`] at a time. Outcomes are in [`SECTIONS`] order.
pub(crate) async fn run_sections(
    router: &axum::Router,
    outer_uri: &Uri,
    version: Version,
    headers: &HeaderMap,
    peer: Option<ConnectInfo<SocketAddr>>,
) -> Vec<(&'static str, Outcome)> {
    let mut slots: Vec<Option<Outcome>> = SECTIONS.iter().map(|_| None).collect();
    // `me` alone first. A JoinSet aborts its tasks if this request is dropped.
    let mut tasks = tokio::task::JoinSet::new();
    let me_request = sub_request(&SECTIONS[0], None, outer_uri, version, headers, peer);
    let me_router = router.clone();
    tasks.spawn(async move { (0, dispatch(me_router, me_request).await) });
    let me = match tasks.join_next().await {
        Some(Ok((_, outcome))) => outcome,
        _ => failed(),
    };
    let account = me_account(&me);
    slots[0] = Some(me);
    let mut next = 1;
    loop {
        while next < SECTIONS.len() && tasks.len() < PARALLEL {
            let request = sub_request(
                &SECTIONS[next],
                account.as_deref(),
                outer_uri,
                version,
                headers,
                peer,
            );
            let router = router.clone();
            let index = next;
            tasks.spawn(async move { (index, dispatch(router, request).await) });
            next += 1;
        }
        let Some(joined) = tasks.join_next().await else { break };
        if let Ok((index, outcome)) = joined {
            slots[index] = Some(outcome);
        }
    }
    SECTIONS
        .iter()
        .zip(slots)
        .map(|(section, outcome)| (section.name, outcome.unwrap_or_else(failed)))
        .collect()
}

/// A sub-request task that did not finish (panic): reported as a 500 section.
fn failed() -> Outcome {
    Outcome {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        etag: None,
        json: false,
        body: Some(Bytes::new()),
        set_cookie: Vec::new(),
    }
}

/// `GET /api/bootstrap`.
pub(crate) async fn get(request: Request) -> Response {
    let Some(Dispatcher(router)) = request.extensions().get::<Dispatcher>().cloned() else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    let peer = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .cloned();
    let if_none_match = request.headers().get(header::IF_NONE_MATCH).cloned();
    let outcomes = run_sections(
        &router,
        request.uri(),
        request.version(),
        request.headers(),
        peer,
    )
    .await;
    let cookies = outcomes
        .iter()
        .flat_map(|(name, outcome)| forwarded_cookies(name, outcome))
        .collect::<Vec<_>>();
    let (body, tag) = document(&outcomes);
    respond(body, tag, cookies, if_none_match.as_ref())
}

fn respond(
    body: Vec<u8>,
    tag: Option<String>,
    cookies: Vec<HeaderValue>,
    if_none_match: Option<&HeaderValue>,
) -> Response {
    let tag = tag.and_then(|t| HeaderValue::from_str(&t).ok());
    let not_modified = cookies.is_empty()
        && tag
            .as_ref()
            .and_then(|t| t.to_str().ok())
            .is_some_and(|t| none_match(if_none_match, t));
    let mut response = if not_modified {
        StatusCode::NOT_MODIFIED.into_response()
    } else {
        let mut response = Body::from(body).into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        response
    };
    let headers = response.headers_mut();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static(CACHE_CONTROL));
    headers.insert(header::VARY, HeaderValue::from_static(VARY));
    if let Some(tag) = tag {
        headers.insert(header::ETAG, tag);
    }
    for cookie in cookies {
        headers.append(header::SET_COOKIE, cookie);
    }
    response
}

#[cfg(test)]
#[path = "bootstrap_tests.rs"]
mod tests;
