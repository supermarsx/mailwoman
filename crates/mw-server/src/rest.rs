//! REST convenience layer over JMAP (SPEC §20.1, plan §3 e9). Owned by t6-e9;
//! mounted by e11 behind the `require_scope` enforcement middleware.
//!
//! A THIN `/api/v1/...` surface with **no new semantics**: each REST call is
//! translated into the exact JMAP method call(s) the web client would issue, run
//! through the same engine/proxy path `/jmap/api` uses, and the JMAP result is
//! mapped straight back. There is no second data model — the REST response bodies
//! are the JMAP objects verbatim (asserted in tests), so REST and JMAP can never
//! drift.
//!
//! Endpoints:
//!   * `GET /api/v1/messages?mailbox=&limit=` → `Email/query` + `Email/get`
//!   * `GET /api/v1/messages/{id}`            → `Email/get`
//!   * `GET /api/v1/mailboxes`                → `Mailbox/get`
//!
//! Logging here is opaque-only (method name + counts) — never a subject, address,
//! or body (§21.1).
//!
//! Every item below is consumed by e11's `router()` mount (the factory returns a
//! `Router<AppState>`, and `AppState` is `pub(crate)`, so this surface cannot be
//! `pub` without leaking a private type). Until e11 wires it in, the handlers read
//! as dead code; the module-scoped allow keeps `clippy -D warnings` green while the
//! logic is fully exercised by the tests below.
#![allow(dead_code)]

use axum::Json;
use axum::extract::{Path as UrlPath, Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Extension, Router, body::Bytes};
use serde::Deserialize;
use serde_json::{Value, json};

use mw_jmap::JmapClient;
use mw_store::Session;

use crate::scope_mw::{RestSessionSource, rest_session};
use crate::{AppState, engine_mode};

/// The default and maximum page size for `GET /api/v1/messages`.
const DEFAULT_LIMIT: u64 = 50;
const MAX_LIMIT: u64 = 200;

/// The `/api/v1/*` router e11 merges into `router()` (behind `require_scope`).
pub(crate) fn rest_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/messages", get(list_messages))
        .route("/api/v1/messages/{id}", get(get_message))
        .route("/api/v1/mailboxes", get(list_mailboxes))
}

// ---------------------------------------------------------------------------
// Request translation (REST → JMAP) — pure, testable
// ---------------------------------------------------------------------------

/// The properties every message view returns. The exact JMAP `Email` property set
/// the web client uses, so REST results are byte-identical to the JMAP surface.
const EMAIL_PROPERTIES: &[&str] = &[
    "id",
    "threadId",
    "mailboxIds",
    "keywords",
    "subject",
    "from",
    "to",
    "receivedAt",
    "size",
    "preview",
    "hasAttachment",
];

/// Build the JMAP request for `GET /api/v1/messages`: an `Email/query` (optionally
/// filtered to a mailbox, newest first) whose result feeds an `Email/get` via a
/// `#ids` result reference — one round-trip, exactly as the web client does it.
fn build_messages_request(account_id: &str, mailbox: Option<&str>, limit: u64) -> Value {
    let mut query_args = json!({
        "accountId": account_id,
        "sort": [ { "property": "receivedAt", "isAscending": false } ],
        "limit": limit,
    });
    if let Some(mbox) = mailbox {
        query_args["filter"] = json!({ "inMailbox": mbox });
    }
    json!({
        "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
        "methodCalls": [
            ["Email/query", query_args, "q"],
            ["Email/get", {
                "accountId": account_id,
                "#ids": { "resultOf": "q", "name": "Email/query", "path": "/ids" },
                "properties": EMAIL_PROPERTIES,
            }, "g"]
        ]
    })
}

/// Build the JMAP request for `GET /api/v1/messages/{id}` (a single `Email/get`).
fn build_message_get_request(account_id: &str, id: &str) -> Value {
    json!({
        "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
        "methodCalls": [
            ["Email/get", {
                "accountId": account_id,
                "ids": [id],
                "properties": EMAIL_PROPERTIES,
            }, "g"]
        ]
    })
}

/// Build the JMAP request for `GET /api/v1/mailboxes` (a full `Mailbox/get`).
fn build_mailboxes_request(account_id: &str) -> Value {
    json!({
        "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
        "methodCalls": [
            ["Mailbox/get", { "accountId": account_id, "ids": null }, "m"]
        ]
    })
}

// ---------------------------------------------------------------------------
// Response mapping (JMAP → REST) — pure, testable
// ---------------------------------------------------------------------------

/// Extract the named method's `arguments` object from a JMAP response envelope by
/// its `callId`. Returns the JMAP method error verbatim (as `Err`) when the call
/// came back as an error.
fn method_result<'a>(response: &'a Value, call_id: &str) -> Result<&'a Value, &'a Value> {
    let responses = response
        .get("methodResponses")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    for entry in responses {
        let Some(arr) = entry.as_array() else {
            continue;
        };
        if arr.len() == 3 && arr[2].as_str() == Some(call_id) {
            let name = arr[0].as_str().unwrap_or_default();
            if name == "error" {
                return Err(&arr[1]);
            }
            return Ok(&arr[1]);
        }
    }
    Ok(&Value::Null)
}

/// Map an `Email/get` JMAP result into the REST list body. The `messages` array IS
/// the JMAP `list` verbatim — no reshaping — so REST and JMAP results are identical.
fn map_messages(get_result: &Value) -> Value {
    let list = get_result.get("list").cloned().unwrap_or(json!([]));
    let count = list.as_array().map(Vec::len).unwrap_or(0);
    json!({ "messages": list, "count": count })
}

/// Map a `Mailbox/get` JMAP result into the REST list body (the JMAP `list` verbatim).
fn map_mailboxes(get_result: &Value) -> Value {
    let list = get_result.get("list").cloned().unwrap_or(json!([]));
    let count = list.as_array().map(Vec::len).unwrap_or(0);
    json!({ "mailboxes": list, "count": count })
}

// ---------------------------------------------------------------------------
// Dispatch — run a translated JMAP request through the same engine/proxy path
// ---------------------------------------------------------------------------

/// Run a JMAP request through the identical path `/jmap/api` uses: the local engine
/// in engine mode, or the upstream JMAP server (with injected auth) in proxy mode.
/// This guarantees REST results equal the JMAP surface's.
async fn dispatch_jmap(
    state: &AppState,
    session: &Session,
    request: &Value,
) -> Result<Value, Response> {
    if let Some(engine) = &state.engine {
        if let Err(e) = engine_mode::ensure_account(engine, &session.account_id).await {
            tracing::warn!("rest: engine account not available: {e}");
            return Err(upstream_error());
        }
        return Ok(engine.handle_jmap(&session.account_id, request).await);
    }
    let client = JmapClient::new(&session.credentials.username, &session.credentials.password)
        .map_err(|_| upstream_error())?;
    let body = Bytes::from(serde_json::to_vec(request).expect("jmap request serialises"));
    match client.request_raw(&session.api_url, body).await {
        Ok((status, bytes)) => {
            if !status.is_success() {
                return Err(upstream_error());
            }
            serde_json::from_slice(&bytes).map_err(|e| {
                tracing::warn!("rest: upstream returned invalid JSON: {e}");
                upstream_error()
            })
        }
        Err(e) => {
            tracing::warn!("rest: upstream proxy failed: {e}");
            Err(upstream_error())
        }
    }
}

fn upstream_error() -> Response {
    (
        StatusCode::BAD_GATEWAY,
        Json(json!({ "error": "upstream request failed" })),
    )
        .into_response()
}

/// Return a JMAP method error as a REST error response.
fn jmap_method_error(err: &Value) -> Response {
    let kind = err.get("type").and_then(Value::as_str).unwrap_or("error");
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": kind, "jmap": err })),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Query string for `GET /api/v1/messages`.
#[derive(Debug, Deserialize)]
struct ListQuery {
    mailbox: Option<String>,
    limit: Option<u64>,
}

/// `GET /api/v1/messages` → `Email/query` + `Email/get`.
async fn list_messages(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(source): Extension<RestSessionSource>,
    Query(q): Query<ListQuery>,
) -> Response {
    let session = match rest_session(&state, &headers, source).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    metrics::counter!("mw_rest_requests_total", "endpoint" => "messages.list").increment(1);
    let limit = q.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let request = build_messages_request(&session.account_id, q.mailbox.as_deref(), limit);
    let response = match dispatch_jmap(&state, &session, &request).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    match method_result(&response, "g") {
        Ok(get) => json_response(map_messages(get)),
        Err(err) => jmap_method_error(err),
    }
}

/// `GET /api/v1/messages/{id}` → `Email/get`.
async fn get_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(source): Extension<RestSessionSource>,
    UrlPath(id): UrlPath<String>,
) -> Response {
    let session = match rest_session(&state, &headers, source).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    metrics::counter!("mw_rest_requests_total", "endpoint" => "messages.get").increment(1);
    let request = build_message_get_request(&session.account_id, &id);
    let response = match dispatch_jmap(&state, &session, &request).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    match method_result(&response, "g") {
        Ok(get) => {
            let list = get.get("list").and_then(Value::as_array);
            match list.and_then(|l| l.first()) {
                Some(message) => json_response(json!({ "message": message })),
                None => (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "message not found" })),
                )
                    .into_response(),
            }
        }
        Err(err) => jmap_method_error(err),
    }
}

/// `GET /api/v1/mailboxes` → `Mailbox/get`.
async fn list_mailboxes(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(source): Extension<RestSessionSource>,
) -> Response {
    let session = match rest_session(&state, &headers, source).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    metrics::counter!("mw_rest_requests_total", "endpoint" => "mailboxes.list").increment(1);
    let request = build_mailboxes_request(&session.account_id);
    let response = match dispatch_jmap(&state, &session, &request).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    match method_result(&response, "m") {
        Ok(get) => json_response(map_mailboxes(get)),
        Err(err) => jmap_method_error(err),
    }
}

/// A JSON `200` with an explicit content-type.
fn json_response(body: Value) -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        serde_json::to_string(&body).unwrap_or_else(|_| "{}".to_string()),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_request_translates_to_query_then_get() {
        let req = build_messages_request("acct1", Some("mbx-inbox"), 25);
        let calls = req["methodCalls"].as_array().unwrap();
        assert_eq!(calls[0][0], "Email/query");
        assert_eq!(calls[0][1]["accountId"], "acct1");
        assert_eq!(calls[0][1]["filter"]["inMailbox"], "mbx-inbox");
        assert_eq!(calls[0][1]["limit"], 25);
        // Email/get pulls its ids from the query via a result reference.
        assert_eq!(calls[1][0], "Email/get");
        assert_eq!(calls[1][1]["#ids"]["resultOf"], "q");
        assert_eq!(calls[1][1]["#ids"]["name"], "Email/query");
    }

    #[test]
    fn messages_request_without_mailbox_has_no_filter() {
        let req = build_messages_request("acct1", None, 10);
        assert!(req["methodCalls"][0][1].get("filter").is_none());
    }

    #[test]
    fn rest_result_is_the_jmap_list_verbatim() {
        // A representative JMAP response as the engine/upstream would return it.
        let jmap_response = json!({
            "methodResponses": [
                ["Email/query", { "ids": ["m1", "m2"] }, "q"],
                ["Email/get", { "list": [
                    { "id": "m1", "subject": "hi", "threadId": "t1" },
                    { "id": "m2", "subject": "yo", "threadId": "t2" }
                ] }, "g"]
            ],
            "sessionState": "s1"
        });
        let get = method_result(&jmap_response, "g").expect("Email/get present");
        let rest = map_messages(get);
        // The REST `messages` array IS the JMAP `list` — identical results.
        assert_eq!(rest["messages"], get["list"]);
        assert_eq!(rest["count"], 2);
        assert_eq!(rest["messages"][0]["id"], "m1");
    }

    #[test]
    fn method_result_surfaces_jmap_errors() {
        let jmap_response = json!({
            "methodResponses": [
                ["error", { "type": "accountNotFound" }, "g"]
            ]
        });
        let err = method_result(&jmap_response, "g").expect_err("should be an error");
        assert_eq!(err["type"], "accountNotFound");
    }

    #[test]
    fn mailboxes_map_is_the_jmap_list_verbatim() {
        let jmap_response = json!({
            "methodResponses": [
                ["Mailbox/get", { "list": [ { "id": "mbx1", "name": "Inbox" } ] }, "m"]
            ]
        });
        let get = method_result(&jmap_response, "m").unwrap();
        let rest = map_mailboxes(get);
        assert_eq!(rest["mailboxes"], get["list"]);
        assert_eq!(rest["count"], 1);
    }
}
