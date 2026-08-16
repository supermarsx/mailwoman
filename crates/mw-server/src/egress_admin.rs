//! Egress-proxy admin API (26.20 t22-e12).
//!
//! The operator surface for the **deliberate, locally-resolved** upstream proxy
//! stored in `egress_proxy` (0026) — as distinct from the ambient
//! `HTTP_PROXY`/`HTTPS_PROXY`/`ALL_PROXY` that t22-e8 refuses on every client in the
//! workspace. One is configuration; the other is an accident of the environment.
//!
//! Every route is **admin-session-gated** (`super::require_admin`, the same
//! `mw_admin_session` cookie as `/admin/*`) and **audited**. It is a CHILD module of
//! `v7_mount` (declared there via `#[path]`) and merged into the already-mounted
//! `extra_v7_router()`, mirroring `admin_plugins` — so it needs no `lib.rs` mount
//! edit, and no fourth copy of the admin gate.
//!
//! Routes:
//!   * `GET  /admin/egress/proxies` — list configured routes. **Never returns the
//!     password**, in any form, to any caller.
//!   * `POST /admin/egress/proxies` — create or replace a route.
//!   * `POST /admin/egress/proxies/{id}/delete` — remove a route.
//!
//! # The password must not leak, and the leak has four exits
//! Sealing the column closes one of them. The others are `tracing` events, panic
//! messages and **error bodies** — the last being the one people forget, because an
//! error body is not thought of as a log. So: the request type's `Debug` is
//! hand-written, the response type has no password field at all rather than an
//! emptied one, and no handler here formats a store error that could have a
//! credential in it back to the caller.
//!
//! # `detail_json` is forever
//! `audit_log` is append-only **by design** — `mw-store`'s `v6.rs` records that no
//! update or delete method exists. A secret written into `detail_json` therefore
//! cannot be redacted later, ever, in the one table built to be permanent. The
//! precedent is `mw-oauth`'s enforcement event, documented as "deliberately carries
//! no secret material". Treated here as a hard rule: the audit detail carries the
//! route's `scheme://host:port`, whether it has credentials, and the username — never
//! the password.
//!
//! # Truthful rows
//! The delete route audits **what happened**, not what was asked: `delete_egress_proxy`
//! reports whether a row actually went, and a delete of an absent id is audited as a
//! miss rather than as a deletion.

use axum::Router;
use axum::extract::{Json, Path as UrlPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;

use crate::AppState;
use mw_store::EgressProxyRow;

/// The egress admin routes. Merged into `v7_mount::extra_v7_router()`.
pub(crate) fn egress_admin_router() -> Router<AppState> {
    Router::new()
        .route("/admin/egress/proxies", get(list_proxies).post(put_proxy))
        .route("/admin/egress/proxies/{id}/delete", post(delete_proxy))
}

/// A create/replace request.
///
/// `Debug` is hand-written for the same reason as `mw_store::EgressProxyRow`'s and
/// `mw_egress::proxy::ProxyAuth`'s: a derived one puts the plaintext into every
/// `tracing` event, panic message and error body that ever formats this struct. The
/// request type matters as much as the stored one — it is what an axum rejection or
/// a `?` in a handler is most likely to render.
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PutProxyReq {
    id: String,
    scheme: String,
    host: String,
    port: u16,
    #[serde(default)]
    username: String,
    #[serde(default)]
    password: Option<String>,
    #[serde(default)]
    allow_plaintext: bool,
}

impl fmt::Debug for PutProxyReq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PutProxyReq")
            .field("id", &self.id)
            .field("scheme", &self.scheme)
            .field("host", &self.host)
            .field("port", &self.port)
            .field("username", &self.username)
            .field(
                "password",
                &self
                    .password
                    .as_ref()
                    .map(|_| "<redacted>")
                    .unwrap_or("None"),
            )
            .field("allow_plaintext", &self.allow_plaintext)
            .finish()
    }
}

/// What the admin UI is told about a route.
///
/// There is **no password field**, not even an emptied or masked one: a field that
/// is sometimes populated is a field that will one day be populated by mistake, and
/// `hasCredentials` answers the only question the UI actually has.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProxyView {
    id: String,
    scheme: String,
    host: String,
    port: u16,
    username: String,
    has_credentials: bool,
    allow_plaintext: bool,
    created_at: String,
    updated_at: String,
}

impl From<&EgressProxyRow> for ProxyView {
    fn from(r: &EgressProxyRow) -> Self {
        Self {
            id: r.id.clone(),
            scheme: r.scheme.clone(),
            host: r.host.clone(),
            port: r.port,
            username: r.username.clone(),
            has_credentials: r.password.is_some(),
            allow_plaintext: r.allow_plaintext,
            created_at: r.created_at.clone(),
            updated_at: r.updated_at.clone(),
        }
    }
}

/// `scheme://host:port` — the only form of a route that may be logged or audited.
/// Carries no credential.
///
/// **This is the seam.** When `t22-e11`'s `mw_egress::proxy::ProxyRoute` lands, the
/// mapping from a stored [`EgressProxyRow`] to that type belongs here and nowhere
/// else, so adopting it is one function rather than a hunt for field renames across
/// the lane. `ProxyRoute::endpoint()` produces exactly this string, which is the
/// point at which the two representations should be reconciled.
fn endpoint(r: &EgressProxyRow) -> String {
    format!("{}://{}:{}", r.scheme, r.host, r.port)
}

/// The audit detail for a route. Deliberately carries no secret material: the
/// endpoint, whether credentials are configured, and the non-secret username.
fn audit_detail(r: &EgressProxyRow) -> serde_json::Value {
    json!({
        "endpoint": endpoint(r),
        "hasCredentials": r.password.is_some(),
        "username": r.username,
        "allowPlaintext": r.allow_plaintext,
    })
}

/// Schemes an egress route may use. Validated here so a typo becomes a `400` rather
/// than a row that fails at fetch time.
const SCHEMES: [&str; 2] = ["http", "socks5"];

fn bad_request(msg: &str) -> Response {
    (StatusCode::BAD_REQUEST, Json(json!({ "error": msg }))).into_response()
}

/// A store failure never renders the underlying error to the caller: it is the one
/// value in this module that may have been constructed from a row containing a
/// credential, and an error body is a leak with a shorter path than a log.
fn store_failed(op: &str, e: &impl fmt::Display) -> Response {
    tracing::warn!("egress admin {op} failed: {e}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": "egress configuration store failure" })),
    )
        .into_response()
}

async fn list_proxies(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(resp) = super::require_admin(&state, &headers).await {
        return resp;
    }
    match state.store.list_egress_proxies().await {
        Ok(rows) => {
            let views: Vec<ProxyView> = rows.iter().map(ProxyView::from).collect();
            Json(json!({ "proxies": views })).into_response()
        }
        Err(e) => store_failed("list", &e),
    }
}

async fn put_proxy(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<PutProxyReq>,
) -> Response {
    let admin = match super::require_admin(&state, &headers).await {
        Ok(a) => a,
        Err(resp) => return resp,
    };
    if body.id.trim().is_empty() {
        return bad_request("id is required");
    }
    if !SCHEMES.contains(&body.scheme.as_str()) {
        return bad_request("scheme must be http or socks5");
    }
    if body.host.trim().is_empty() {
        return bad_request("host is required");
    }
    if body.port == 0 {
        return bad_request("port is required");
    }
    // A password with no username is a misconfiguration that would authenticate as
    // nobody; refusing it here beats a route that fails only at fetch time.
    if body.password.is_some() && body.username.trim().is_empty() {
        return bad_request("username is required when a password is set");
    }

    let row = EgressProxyRow {
        id: body.id.clone(),
        scheme: body.scheme.clone(),
        host: body.host.clone(),
        port: body.port,
        username: body.username.clone(),
        password: body.password.clone(),
        allow_plaintext: body.allow_plaintext,
        created_at: String::new(),
        updated_at: String::new(),
    };
    if let Err(e) = state.store.put_egress_proxy(&row).await {
        return store_failed("put", &e);
    }
    append_egress_audit(&state, &admin, &row.id, audit_detail(&row)).await;
    Json(json!({ "ok": true, "proxy": ProxyView::from(&row) })).into_response()
}

async fn delete_proxy(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
) -> Response {
    let admin = match super::require_admin(&state, &headers).await {
        Ok(a) => a,
        Err(resp) => return resp,
    };
    match state.store.delete_egress_proxy(&id).await {
        // Audit what HAPPENED, not what was asked: deleting an absent id removed
        // nothing, and a row claiming otherwise would be a false record in an
        // append-only table.
        Ok(removed) => {
            append_egress_audit(&state, &admin, &id, json!({ "removed": removed })).await;
            Json(json!({ "ok": true, "removed": removed })).into_response()
        }
        Err(e) => store_failed("delete", &e),
    }
}

/// Append an egress-configuration audit row.
///
/// Uses the existing `SecurityPolicyChanged` kind: an egress route decides where all
/// outbound traffic goes, which is security policy in effect. A dedicated
/// `EgressRouteChanged` variant would read better, but `mw_admin::AuditKind` is
/// outside this lane's files — worth adding when someone is next in that crate.
/// Best-effort, mirroring `append_plugin_audit`: a failed audit is logged and never
/// changes the outcome of the operation.
async fn append_egress_audit(state: &AppState, admin: &str, id: &str, detail: serde_json::Value) {
    let entry = mw_admin::AuditEvent::new(
        admin,
        mw_admin::ActorKind::Admin,
        mw_admin::AuditKind::SecurityPolicyChanged,
    )
    .target(id)
    .detail(detail)
    .into_entry();
    let row = mw_store::AuditRow {
        id: entry.id,
        ts: entry.ts,
        actor: entry.actor,
        actor_kind: "admin".to_string(),
        action: entry.action,
        target: entry.target,
        detail_json: entry.detail_json,
        ip: entry.ip,
    };
    if let Err(e) = state.store.append_audit(&row).await {
        tracing::warn!("egress audit append failed ({}): {e}", row.action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PASSWORD: &str = "correct-horse-battery-staple";

    fn row() -> EgressProxyRow {
        EgressProxyRow {
            id: "corp".into(),
            scheme: "http".into(),
            host: "proxy.corp.example".into(),
            port: 3128,
            username: "svc-mail".into(),
            password: Some(PASSWORD.into()),
            allow_plaintext: false,
            created_at: "t0".into(),
            updated_at: "t0".into(),
        }
    }

    #[test]
    fn the_audit_detail_carries_no_secret() {
        // `audit_log` is append-only by design, so a secret here could never be
        // redacted. This is the assertion that keeps that true.
        let detail = audit_detail(&row()).to_string();
        assert!(
            !detail.contains(PASSWORD),
            "the password reached an append-only audit row: {detail}"
        );
        assert!(
            detail.contains("proxy.corp.example") && detail.contains("svc-mail"),
            "asserted after the negative, so a detail that carried NOTHING could not \
             pass as a detail that carried no secret: {detail}"
        );
        assert!(
            detail.contains("hasCredentials"),
            "the audit must record THAT credentials exist even though it must not \
             record what they are"
        );
    }

    #[test]
    fn the_serialized_view_has_no_password_field_at_all() {
        let json = serde_json::to_string(&ProxyView::from(&row())).unwrap();
        assert!(
            !json.contains(PASSWORD),
            "the API echoed the password: {json}"
        );
        assert!(
            !json.contains("password"),
            "there must be no password FIELD, not even empty or masked — a field that \
             is sometimes populated is one that will be populated by mistake: {json}"
        );
        assert!(json.contains("\"hasCredentials\":true"));
    }

    #[test]
    fn debug_on_the_request_type_redacts_the_password() {
        // The request struct is what an axum rejection or a `?` in a handler is most
        // likely to render — so it needs the same care as the stored row.
        let req = PutProxyReq {
            id: "corp".into(),
            scheme: "http".into(),
            host: "proxy.corp.example".into(),
            port: 3128,
            username: "svc-mail".into(),
            password: Some(PASSWORD.into()),
            allow_plaintext: false,
        };
        let rendered = format!("{req:?}");
        assert!(!rendered.contains(PASSWORD), "Debug leaked: {rendered}");
        assert!(rendered.contains("<redacted>"));
    }

    #[test]
    fn the_endpoint_seam_carries_no_credential() {
        assert_eq!(endpoint(&row()), "http://proxy.corp.example:3128");
        assert!(!endpoint(&row()).contains(PASSWORD));
        assert!(
            !endpoint(&row()).contains("svc-mail"),
            "even the username stays out of the endpoint string, so it is safe to log \
             verbatim without thinking about it"
        );
    }
}
