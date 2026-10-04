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
//!   * `POST /admin/egress/proxies/{id}/activate` — make this the one live route.
//!   * `POST /admin/egress/proxies/deactivate` — return egress to direct.
//!   * `POST /admin/egress/proxies/{id}/test` — fetch the probe URL through the
//!     active route and report how far it got. See [`test_proxy`].
//!
//! Configuring a route does **not** make it live: activation is a separate,
//! deliberate act (0027), so adding a route can never silently reroute traffic.
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
use std::time::Duration;

use mw_egress::Refusal;
use mw_egress::proxy::{ProxyRefusal, fetch_via_proxy};

use crate::AppState;
use mw_store::EgressProxyRow;

/// The egress admin routes. Merged into `v7_mount::extra_v7_router()`.
pub(crate) fn egress_admin_router() -> Router<AppState> {
    Router::new()
        .route("/admin/egress/proxies", get(list_proxies).post(put_proxy))
        .route("/admin/egress/proxies/{id}/delete", post(delete_proxy))
        .route("/admin/egress/proxies/{id}/activate", post(activate_proxy))
        .route("/admin/egress/proxies/deactivate", post(deactivate_proxies))
        .route("/admin/egress/proxies/{id}/test", post(test_proxy))
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
    /// Whether this is the live route. At most one row may be, by 0027's partial
    /// unique index — see `Store::active_egress_proxy` for why that is a security
    /// property. Not a secret, so the UI may show it.
    active: bool,
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
            active: r.active,
            created_at: r.created_at.clone(),
            updated_at: r.updated_at.clone(),
        }
    }
}

/// `scheme://host:port` — the only form of a route that may be logged or audited.
/// Carries no credential.
///
/// **The `EgressProxyRow` → `mw_egress::proxy::ProxyRoute` mapping does NOT live
/// here — it lives in `image_proxy.rs` (t22-e14).** This comment previously claimed
/// the mapping belonged in this module "and nowhere else". That was right when it
/// was written and wrong by the time it mattered: `image_proxy.rs` has since become
/// the crate's egress facade, re-exporting `ip_allowed`/`embedded_ipv4s`/
/// `fetch_url_hardened` for in-tree callers, and **both** consumers of a route reach
/// it there — the image proxy itself and the `webcal://`/ICS importer at
/// `import_routes.rs:485`. This module is a pure configuration surface: it never
/// fetches anything, so a fetch-time mapping placed here would have to be imported
/// backwards by the code that actually egresses.
///
/// What remains here is only the audit/display form below, built from the stored
/// row so this module can render a route without one having been mapped to the
/// transport. It is not the same string as `ProxyRoute::endpoint()`: that one
/// spells the scheme the transport's way (`http-connect://…`), this one the way the
/// row and the admin UI do (`http://…`).
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
        // Ignored by `put_egress_proxy` — activation is a deliberate, separate act
        // (0027), never a side effect of an edit. Set explicitly rather than by a
        // `..Default::default()` so this stays visible at the call site.
        active: false,
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

/// `POST /admin/egress/proxies/{id}/activate` — make this the one live route.
///
/// Existence is checked FIRST, and a missing id is a `404` that changes nothing.
/// `Store::set_active_egress_proxy` deliberately deactivates before it activates, so
/// calling it with a typo'd id would leave egress direct — the fail-safe direction,
/// but still a change the operator did not ask for and might not notice. Guarding at
/// the boundary means a typo costs them nothing; the store's behaviour stays as
/// defence in depth for any other caller.
async fn activate_proxy(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
) -> Response {
    let admin = match super::require_admin(&state, &headers).await {
        Ok(a) => a,
        Err(resp) => return resp,
    };
    let row = match state.store.get_egress_proxy(&id).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "no such egress route" })),
            )
                .into_response();
        }
        Err(e) => return store_failed("activate lookup", &e),
    };
    match state.store.set_active_egress_proxy(Some(&id)).await {
        Ok(activated) => {
            append_egress_audit(
                &state,
                &admin,
                &id,
                json!({ "activated": activated, "endpoint": endpoint(&row) }),
            )
            .await;
            Json(json!({ "ok": true, "activated": activated })).into_response()
        }
        Err(e) => store_failed("activate", &e),
    }
}

/// `POST /admin/egress/proxies/deactivate` — return egress to direct.
///
/// Deliberately not `{id}/deactivate`: there is at most one live route, so "stop
/// using a proxy" is one deployment-wide state rather than an operation on a
/// particular row, and naming an id would imply a per-route toggle that 0027 does
/// not permit.
async fn deactivate_proxies(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let admin = match super::require_admin(&state, &headers).await {
        Ok(a) => a,
        Err(resp) => return resp,
    };
    // What was live BEFORE, so the audit row names what actually changed rather than
    // recording an anonymous "egress deactivated" that a reader cannot act on.
    let previous = match state.store.active_egress_proxy().await {
        Ok(p) => p,
        Err(e) => return store_failed("deactivate lookup", &e),
    };
    if let Err(e) = state.store.set_active_egress_proxy(None).await {
        return store_failed("deactivate", &e);
    }
    let target = previous.as_ref().map_or("", |r| r.id.as_str());
    append_egress_audit(
        &state,
        &admin,
        target,
        json!({
            "deactivated": previous.is_some(),
            "endpoint": previous.as_ref().map(endpoint),
        }),
    )
    .await;
    Json(json!({ "ok": true, "deactivated": previous.is_some() })).into_response()
}

// ── "test this route" ────────────────────────────────────────────────────────

/// What a route test fetches when `MW_EGRESS_PROBE_URL` is not set. `example.com`
/// is reserved by IANA for documentation and answers over https.
const DEFAULT_PROBE_URL: &str = "https://example.com/";

/// How long the test waits for the proxy's own name to resolve, and separately for
/// a TCP connection to it, before reporting that stage as the failure.
const PROBE_STAGE_TIMEOUT: Duration = Duration::from_secs(5);

/// The URL a route test fetches: `MW_EGRESS_PROBE_URL` when set, else
/// [`DEFAULT_PROBE_URL`].
///
/// It comes from the deployment's environment and never from the request, so the
/// endpoint cannot be used to make the server fetch a URL of the caller's choosing.
/// It is subject to the same address policy as any other proxied fetch.
fn probe_url() -> String {
    std::env::var("MW_EGRESS_PROBE_URL")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_PROBE_URL.to_string())
}

/// The verdict of a route test. `outcome` and `stage` are the wire vocabulary the
/// admin UI renders (`apps/web/src/screens/Admin/Egress.tsx`).
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct TestVerdict {
    /// `connected` · `authRejected` · `refusedByPolicy` · `dnsFailed` ·
    /// `unreachable` · `originTlsFailed` · `routeInvalid`.
    outcome: &'static str,
    /// Where the attempt stopped: `dns` · `connect` · `tunnel` · `origin`.
    stage: &'static str,
    /// `scheme://host:port` of the route. No credential.
    endpoint: String,
    /// From the transport's own progress: true once the proxy accepted a tunnel.
    traversed_proxy: bool,
    /// One sentence. Written here or a `&'static str` from the transport; never
    /// text received from the proxy or the origin, and never a credential.
    detail: String,
    /// The URL that was fetched.
    probe_url: String,
}

/// Map the transport's result onto the wire vocabulary.
///
/// `traversed` decides between `tunnel` and `origin` for the failures that can
/// happen on either side of the proxy accepting the tunnel.
fn classify(
    outcome: &Result<Vec<u8>, ProxyRefusal>,
    traversed: bool,
) -> (&'static str, &'static str, String) {
    let past_tunnel = if traversed { "origin" } else { "tunnel" };
    match outcome {
        Ok(_) => (
            "connected",
            "origin",
            "The probe URL was fetched through the route.".to_string(),
        ),
        // The origin answered HTTP through the tunnel. The route carried a request
        // to it and a response back, which is what was being tested; the status is
        // the probe target's business.
        Err(ProxyRefusal::Origin(Refusal::Status(code))) => (
            "connected",
            "origin",
            format!("The route reached the probe URL, which answered HTTP {code}."),
        ),
        Err(ProxyRefusal::Origin(Refusal::TooLarge)) => (
            "connected",
            "origin",
            "The route reached the probe URL; its response was larger than the fetch limit."
                .to_string(),
        ),
        Err(ProxyRefusal::Origin(Refusal::Blocked)) => (
            "refusedByPolicy",
            "tunnel",
            "Mailwoman's address policy refused the probe URL's address, or its name did not \
             resolve; the proxy was not contacted for it."
                .to_string(),
        ),
        Err(ProxyRefusal::Origin(Refusal::BadRequest(m))) => (
            "refusedByPolicy",
            "tunnel",
            format!("The probe URL was refused before the proxy was contacted: {m}."),
        ),
        Err(ProxyRefusal::PlaintextRefused) => (
            "refusedByPolicy",
            "tunnel",
            "The probe URL is plaintext http and this route does not allow plaintext origins."
                .to_string(),
        ),
        Err(ProxyRefusal::Origin(Refusal::Timeout)) => (
            "unreachable",
            past_tunnel,
            "The fetch timed out.".to_string(),
        ),
        Err(ProxyRefusal::Origin(Refusal::Upstream)) => (
            "unreachable",
            past_tunnel,
            "The exchange with the probe URL failed.".to_string(),
        ),
        Err(ProxyRefusal::OriginHttp(_)) => (
            "unreachable",
            "origin",
            "The HTTP exchange with the probe URL failed inside the tunnel.".to_string(),
        ),
        Err(ProxyRefusal::RouteInvalid(m)) => ("routeInvalid", "connect", format!("{m}.")),
        Err(ProxyRefusal::ProxyUnreachable(m)) => ("unreachable", "tunnel", format!("{m}.")),
        Err(ProxyRefusal::ProxyAuthRejected(m)) => ("authRejected", "tunnel", format!("{m}.")),
        // The payload is the proxy's own status line or reply code; it is not
        // forwarded, so nothing a proxy says can reach the admin's browser.
        Err(ProxyRefusal::ProxyRejected(_)) => (
            "refusedByPolicy",
            "tunnel",
            "The proxy answered and refused to open a tunnel to the probe URL.".to_string(),
        ),
        Err(ProxyRefusal::OriginTls(_)) => (
            "originTlsFailed",
            "origin",
            "TLS to the probe URL failed inside the tunnel.".to_string(),
        ),
    }
}

/// `POST /admin/egress/proxies/{id}/test` — fetch the probe URL through one route.
///
/// **The status says whether the test ran; the verdict is in the body.** `200`
/// carries a [`TestVerdict`] for every outcome, the failures included; `404` means
/// there is no such route; `409` the route is not the active one; `401` not an
/// admin; `500` the route could not be loaded.
///
/// **Only the active route can be tested.** The one mapping from a stored row to
/// the transport's `ProxyRoute` is private to `image_proxy.rs`, and the only
/// accessor it shares is `active_route`, which returns the live route. A route
/// that exists but is not active answers `409` and is not probed. Testing a staged
/// route before switching to it needs a by-id accessor in that module
/// (`route_by_id`, for which `mw-egress/tests/route_construction_sites.rs` already
/// carries a named exception); until it exists this endpoint cannot do it.
///
/// Testing changes nothing — it does not go through the image proxy's cache or its
/// transition audit.
///
/// Three stages, so the answer says where to look:
/// 1. `dns` — the proxy's own host is resolved here.
/// 2. `connect` — a TCP connection to that address is opened and dropped.
/// 3. `tunnel` / `origin` — the transport (`mw_egress::proxy::fetch_via_proxy`)
///    negotiates the tunnel and fetches the probe URL, under the same address
///    policy and plaintext rule as a real fetch.
///
/// Stages 1 and 2 exist because the transport reports "does not resolve",
/// "connection refused" and "died mid-negotiation" as one `ProxyUnreachable`.
async fn test_proxy(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
) -> Response {
    let admin = match super::require_admin(&state, &headers).await {
        Ok(a) => a,
        Err(resp) => return resp,
    };
    let row = match state.store.get_egress_proxy(&id).await {
        Ok(Some(row)) => row,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "no such egress route" })),
            )
                .into_response();
        }
        Err(e) => return store_failed("test lookup", &e),
    };
    if !row.active {
        return (
            StatusCode::CONFLICT,
            Json(json!({ "error": "only the active egress route can be tested" })),
        )
            .into_response();
    }
    // `active_route` takes no id, so check it returned the row that was asked
    // about: another admin may have switched routes since the lookup above.
    let route = match crate::image_proxy::active_route(&state.store).await {
        Ok(Some(route)) if route.id == id => route,
        Ok(_) => {
            return (
                StatusCode::CONFLICT,
                Json(json!({ "error": "only the active egress route can be tested" })),
            )
                .into_response();
        }
        Err(()) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "egress route could not be loaded" })),
            )
                .into_response();
        }
    };
    let endpoint = endpoint(&row);
    let probe = probe_url();
    let verdict = |outcome, stage, traversed_proxy, detail: String| TestVerdict {
        outcome,
        stage,
        endpoint: endpoint.clone(),
        traversed_proxy,
        detail,
        probe_url: probe.clone(),
    };

    let verdict = 'probe: {
        // 1. dns — the proxy's own name.
        let lookup = tokio::time::timeout(
            PROBE_STAGE_TIMEOUT,
            tokio::net::lookup_host((route.host.as_str(), route.port)),
        )
        .await;
        let proxy_addr = match lookup {
            Ok(Ok(mut addrs)) => addrs.next(),
            _ => None,
        };
        let Some(proxy_addr) = proxy_addr else {
            break 'probe verdict(
                "dnsFailed",
                "dns",
                false,
                "The proxy host did not resolve to an address.".to_string(),
            );
        };
        // 2. connect — is anything listening there.
        match tokio::time::timeout(
            PROBE_STAGE_TIMEOUT,
            tokio::net::TcpStream::connect(proxy_addr),
        )
        .await
        {
            Ok(Ok(stream)) => drop(stream),
            _ => {
                break 'probe verdict(
                    "unreachable",
                    "connect",
                    false,
                    "No TCP connection could be opened to the proxy.".to_string(),
                );
            }
        }
        // 3. tunnel + origin — the real transport.
        let Ok(url) = reqwest::Url::parse(&probe) else {
            break 'probe verdict(
                "refusedByPolicy",
                "tunnel",
                false,
                "MW_EGRESS_PROBE_URL is not a URL.".to_string(),
            );
        };
        let fetch = fetch_via_proxy(url, &route, "*/*").await;
        let (outcome, stage, detail) = classify(&fetch.outcome, fetch.traversed_proxy);
        verdict(outcome, stage, fetch.traversed_proxy, detail)
    };

    append_egress_audit(
        &state,
        &admin,
        &id,
        json!({
            "tested": true,
            "endpoint": verdict.endpoint,
            "outcome": verdict.outcome,
            "stage": verdict.stage,
            "traversedProxy": verdict.traversed_proxy,
        }),
    )
    .await;
    Json(verdict).into_response()
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
            active: false,
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

    /// Every transport result maps onto the seven-outcome / four-stage vocabulary
    /// the UI renders, and success is claimed only where the origin was reached.
    #[test]
    fn classify_maps_each_refusal_to_an_outcome_and_a_stage() {
        let c = |r: Result<Vec<u8>, ProxyRefusal>, traversed| {
            let (outcome, stage, _) = classify(&r, traversed);
            (outcome, stage)
        };
        assert_eq!(c(Ok(vec![1]), true), ("connected", "origin"));
        assert_eq!(
            c(Err(ProxyRefusal::Origin(Refusal::Status(404))), true),
            ("connected", "origin")
        );
        assert_eq!(
            c(Err(ProxyRefusal::ProxyAuthRejected("x")), false),
            ("authRejected", "tunnel")
        );
        assert_eq!(
            c(Err(ProxyRefusal::ProxyRejected("403".into())), false),
            ("refusedByPolicy", "tunnel")
        );
        assert_eq!(
            c(Err(ProxyRefusal::PlaintextRefused), false),
            ("refusedByPolicy", "tunnel")
        );
        assert_eq!(
            c(Err(ProxyRefusal::Origin(Refusal::Blocked)), false),
            ("refusedByPolicy", "tunnel")
        );
        assert_eq!(
            c(Err(ProxyRefusal::ProxyUnreachable("x")), false),
            ("unreachable", "tunnel")
        );
        assert_eq!(
            c(Err(ProxyRefusal::Origin(Refusal::Timeout)), false),
            ("unreachable", "tunnel")
        );
        assert_eq!(
            c(Err(ProxyRefusal::Origin(Refusal::Timeout)), true),
            ("unreachable", "origin")
        );
        assert_eq!(
            c(Err(ProxyRefusal::OriginTls("bad cert".into())), true),
            ("originTlsFailed", "origin")
        );
        assert_eq!(
            c(Err(ProxyRefusal::RouteInvalid("x")), false),
            ("routeInvalid", "connect")
        );
    }

    /// Text the proxy or the origin sent is not copied into the verdict.
    #[test]
    fn third_party_text_does_not_reach_the_detail() {
        const MARK: &str = "<script>from-the-proxy</script>";
        for refusal in [
            ProxyRefusal::ProxyRejected(MARK.into()),
            ProxyRefusal::OriginTls(MARK.into()),
            ProxyRefusal::OriginHttp(MARK.into()),
        ] {
            let (_, _, detail) = classify(&Err(refusal), true);
            assert!(!detail.contains("from-the-proxy"), "{detail}");
            assert!(!detail.is_empty());
        }
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
