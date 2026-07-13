//! Mailwoman-native calendar / address-book sharing (plan §3 e9, SPEC §11/§13):
//! serve an on-server collection to another principal, ACL-checked against the
//! grantor's `shareWith` grants (the `calendar_shares` table, surfaced on the
//! frozen `Calendar` object).
//!
//! ## Data path
//! These endpoints read the owner's collection through the **frozen engine PIM
//! surface** — `Engine::handle_jmap` with `Calendar/get` / `CalendarEvent/*`
//! (and `AddressBook/get` / `ContactCard/*`) — the same envelope the web client
//! speaks. No new store accessor is introduced: the grants ride on
//! `Calendar.shareWith`, and the events/cards come back as the standard JMAP
//! `get` list. Those handlers are filled by **e8** (`dispatch_pim`); until e8
//! lands the ACL + projection logic here is exercised by the unit tests below,
//! and the live end-to-end path is proven by e12 against Radicale + the engine.
//!
//! ## What the frozen contract does and does not cover
//! - **Calendars** carry `shareWith: [{principal, access}]` (§2.1) → real
//!   grantee ACL: an owner reads their own collection; a grantee with
//!   `read`/`readWrite` may fetch; everyone else is `403`.
//! - **Address books** have **no** share-ACL in the V3 frozen model (no
//!   `shareWith` on `AddressBook`, no `addressbook_shares` table, §2.1/§2.4), so
//!   the address-book endpoint is **owner-only** (a principal may fetch a book
//!   in their own account; cross-principal is `403`). Cross-principal
//!   address-book grants are a documented follow-up (see `.orchestration/logs`).

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

use mw_engine::Engine;

/// Access level a principal holds on a shared collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Access {
    Read,
    ReadWrite,
}

impl Access {
    fn as_str(self) -> &'static str {
        match self {
            Access::Read => "read",
            Access::ReadWrite => "readWrite",
        }
    }
}

/// Resolve the access a `principal` holds from a calendar's `shareWith` grants
/// (the JSON array as returned by `Calendar/get`). `readWrite` supersedes
/// `read`; returns `None` when the principal is not a grantee.
pub(crate) fn resolve_access(share_with: &Value, principal: &str) -> Option<Access> {
    let grants = share_with.as_array()?;
    let mut best: Option<Access> = None;
    for grant in grants {
        let matches = grant
            .get("principal")
            .and_then(Value::as_str)
            .is_some_and(|p| p.eq_ignore_ascii_case(principal));
        if !matches {
            continue;
        }
        let access = match grant.get("access").and_then(Value::as_str) {
            Some("readWrite") => Access::ReadWrite,
            _ => Access::Read,
        };
        // Keep the strongest grant if a principal is listed more than once.
        best = Some(match (best, access) {
            (Some(Access::ReadWrite), _) | (_, Access::ReadWrite) => Access::ReadWrite,
            _ => Access::Read,
        });
    }
    best
}

/// Pull the result object of the (first) response whose method name matches
/// `method` out of an `Engine::handle_jmap` reply.
fn method_result<'a>(response: &'a Value, method: &str) -> Option<&'a Value> {
    response
        .get("methodResponses")?
        .as_array()?
        .iter()
        .find(|entry| entry.get(0).and_then(Value::as_str) == Some(method))
        .and_then(|entry| entry.get(1))
}

/// `GET /dav/calendars/{accountId}/{calendarId}` core: serve `owner_account`'s
/// calendar `calendar_id` to the requesting principal, ACL-checked.
///
/// Returns a JSON projection `{calendar, events, access}` (200) for the owner or
/// a grantee; `403` when the principal is not granted; `404` when the calendar
/// does not exist or is not a Mailwoman-native (non-CalDAV) collection.
pub(crate) async fn serve_shared_calendar(
    engine: &Engine,
    owner_account: &str,
    calendar_id: &str,
    requester_account: &str,
    requester_principal: &str,
) -> Response {
    let get_req = json!({
        "using": ["urn:mailwoman:calendars"],
        "methodCalls": [
            ["Calendar/get", { "accountId": owner_account, "ids": [calendar_id] }, "0"]
        ]
    });
    let resp = engine.handle_jmap(owner_account, &get_req).await;
    let Some(calendar) = method_result(&resp, "Calendar/get")
        .and_then(|r| r.get("list"))
        .and_then(Value::as_array)
        .and_then(|list| {
            list.iter()
                .find(|c| c.get("id").and_then(Value::as_str) == Some(calendar_id))
        })
    else {
        return not_found("calendar");
    };

    // Owner reads their own collection unconditionally; otherwise the grantee
    // must appear in shareWith with read or readWrite.
    let access = if requester_account == owner_account {
        Access::ReadWrite
    } else {
        let share_with = calendar.get("shareWith").cloned().unwrap_or(json!([]));
        match resolve_access(&share_with, requester_principal) {
            Some(a) => a,
            None => return forbidden(),
        }
    };

    let events = fetch_collection(engine, owner_account, calendar_id, EventKind::Calendar).await;
    Json(json!({
        "calendar": calendar,
        "events": events,
        "access": access.as_str(),
    }))
    .into_response()
}

/// `GET /dav/addressbooks/{accountId}/{addressBookId}` core. Owner-only in V3
/// (no address-book share-ACL in the frozen model — see module docs): a
/// principal may fetch a book in their own account; cross-principal is `403`.
pub(crate) async fn serve_shared_addressbook(
    engine: &Engine,
    owner_account: &str,
    address_book_id: &str,
    requester_account: &str,
) -> Response {
    if requester_account != owner_account {
        return forbidden();
    }
    let get_req = json!({
        "using": ["urn:mailwoman:contacts"],
        "methodCalls": [
            ["AddressBook/get", { "accountId": owner_account, "ids": [address_book_id] }, "0"]
        ]
    });
    let resp = engine.handle_jmap(owner_account, &get_req).await;
    let Some(book) = method_result(&resp, "AddressBook/get")
        .and_then(|r| r.get("list"))
        .and_then(Value::as_array)
        .and_then(|list| {
            list.iter()
                .find(|b| b.get("id").and_then(Value::as_str) == Some(address_book_id))
        })
    else {
        return not_found("address book");
    };
    let cards = fetch_collection(
        engine,
        owner_account,
        address_book_id,
        EventKind::AddressBook,
    )
    .await;
    Json(json!({
        "addressBook": book,
        "cards": cards,
        "access": Access::ReadWrite.as_str(),
    }))
    .into_response()
}

/// Which collection family a fetch targets.
#[derive(Clone, Copy)]
enum EventKind {
    Calendar,
    AddressBook,
}

/// Fetch a collection's members (events / cards) via a `query` + result-ref
/// `get`, then defensively filter to the requested collection id. Returns the
/// member JSON list (empty on any engine error — the ACL gate already passed).
async fn fetch_collection(
    engine: &Engine,
    owner_account: &str,
    collection_id: &str,
    kind: EventKind,
) -> Vec<Value> {
    let (capability, query_method, get_method, filter_key, id_field) = match kind {
        EventKind::Calendar => (
            "urn:mailwoman:calendars",
            "CalendarEvent/query",
            "CalendarEvent/get",
            "inCalendar",
            "calendarId",
        ),
        EventKind::AddressBook => (
            "urn:mailwoman:contacts",
            "ContactCard/query",
            "ContactCard/get",
            "inAddressBook",
            "addressBookId",
        ),
    };
    let req = json!({
        "using": [capability],
        "methodCalls": [
            [query_method, { "accountId": owner_account, "filter": { filter_key: collection_id } }, "q"],
            [get_method, {
                "accountId": owner_account,
                "#ids": { "resultOf": "q", "name": query_method, "path": "/ids" }
            }, "g"]
        ]
    });
    let resp = engine.handle_jmap(owner_account, &req).await;
    method_result(&resp, get_method)
        .and_then(|r| r.get("list"))
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter(|m| {
                    // If the engine already honored the filter, id_field matches;
                    // if it ignored the filter, this keeps the result correct.
                    m.get(id_field).and_then(Value::as_str) == Some(collection_id)
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

fn forbidden() -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({ "error": "not authorized to read this collection" })),
    )
        .into_response()
}

fn not_found(what: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "error": format!("{what} not found") })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shares() -> Value {
        json!([
            { "principal": "alice@example.org", "access": "read" },
            { "principal": "bob@example.org", "access": "readWrite" }
        ])
    }

    #[test]
    fn grantee_access_is_resolved_and_case_insensitive() {
        assert_eq!(
            resolve_access(&shares(), "alice@example.org"),
            Some(Access::Read)
        );
        assert_eq!(
            resolve_access(&shares(), "BOB@example.org"),
            Some(Access::ReadWrite)
        );
    }

    #[test]
    fn non_grantee_is_denied() {
        assert_eq!(resolve_access(&shares(), "carol@example.org"), None);
        assert_eq!(resolve_access(&json!([]), "alice@example.org"), None);
        assert_eq!(
            resolve_access(&json!("not-an-array"), "alice@example.org"),
            None
        );
    }

    #[test]
    fn strongest_grant_wins_for_duplicate_principal() {
        let dup = json!([
            { "principal": "eve@example.org", "access": "read" },
            { "principal": "eve@example.org", "access": "readWrite" }
        ]);
        assert_eq!(
            resolve_access(&dup, "eve@example.org"),
            Some(Access::ReadWrite)
        );
    }

    #[test]
    fn method_result_extracts_the_named_response() {
        let reply = json!({
            "methodResponses": [
                ["Calendar/get", { "list": [{ "id": "c1" }] }, "0"],
                ["CalendarEvent/get", { "list": [] }, "1"]
            ],
            "sessionState": "s"
        });
        let cal = method_result(&reply, "Calendar/get").unwrap();
        assert_eq!(cal["list"][0]["id"], "c1");
        assert!(method_result(&reply, "Note/get").is_none());
    }
}
