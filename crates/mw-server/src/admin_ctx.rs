//! Who is calling an admin route, and what they may do (t32-ext §M, e24a seam).
//!
//! * [`AdminCtx`] — the signed-in admin: username, account, and the capabilities
//!   their roles grant, each for all domains or for a set of domains. The registrar
//!   in [`crate::admin_router`] resolves it before the handler runs, refuses the
//!   request when the route's capability is missing, and hands it to the handler
//!   as an extractor. A handler outside the registry cannot obtain one: the
//!   extractor fails closed.
//! * [`AdminApiError`] — the one error shape of admin routes:
//!   `{code, detail, field}` (`field` = JSON pointer into the request body), with
//!   `422` for a refused value (t32-cfg §4.6 R-CFG 1). `error` repeats `detail` for
//!   the older panel client, which reads that key.
//! * [`etag_of`] / [`check_if_match`] / [`with_etag`] — optimistic concurrency:
//!   reads send an `ETag`, writes accept `If-Match` and are refused with
//!   `412 {code:"stale", updatedBy, updatedAt}` when the object changed (R-CFG 4).

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, FromRequestParts, Request};
use axum::http::request::Parts;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::de::DeserializeOwned;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use crate::AppState;
use crate::capabilities::Capability;

// ─── errors ───────────────────────────────────────────────────────────────────

/// An admin route's refusal: status plus `{code, detail, field}`.
#[derive(Debug, Clone)]
pub struct AdminApiError {
    pub status: StatusCode,
    pub code: Cow<'static, str>,
    pub detail: String,
    /// JSON pointer to the refused field (`/smtp/port`), when one is to blame.
    pub field: Option<String>,
    /// More members of the body (`capability`, `updatedBy`, …).
    pub extra: Map<String, Value>,
}

impl AdminApiError {
    pub fn new(
        status: StatusCode,
        code: impl Into<Cow<'static, str>>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            status,
            code: code.into(),
            detail: detail.into(),
            field: None,
            extra: Map::new(),
        }
    }

    /// `422`: the value at `field` (a JSON pointer) is refused.
    pub fn invalid(
        code: impl Into<Cow<'static, str>>,
        detail: impl Into<String>,
        field: impl Into<String>,
    ) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, code, detail).at(field)
    }

    /// `401`: no admin session (or the panel is turned off).
    pub fn unauthenticated() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "unauthenticated",
            "admin authentication required",
        )
    }

    /// `403`: the admin's roles do not include `cap`.
    pub fn forbidden(cap: Capability) -> Self {
        crate::request_protection::observe(mw_store::request_protection_model::Signal::PermissionDenied);
        Self::new(
            StatusCode::FORBIDDEN,
            "forbidden",
            format!("Your role does not include {cap}."),
        )
        .with("capability", cap.as_str())
    }

    /// `403`: `cap` is granted only for some domains, and this action needs it
    /// for all of them.
    pub fn unscoped_required(cap: Capability) -> Self {
        crate::request_protection::observe(mw_store::request_protection_model::Signal::PermissionDenied);
        Self::new(
            StatusCode::FORBIDDEN,
            "unscopedGrantRequired",
            format!(
                "{cap} is granted to you for some domains only; this needs it for all domains."
            ),
        )
        .with("capability", cap.as_str())
    }

    /// `403`: `domain` is outside the domains `cap` is granted for.
    pub fn out_of_scope(cap: Capability, domain: &str) -> Self {
        crate::request_protection::observe(mw_store::request_protection_model::Signal::PermissionDenied);
        Self::new(
            StatusCode::FORBIDDEN,
            "outOfScope",
            format!("{cap} is not granted to you for {domain}."),
        )
        .with("capability", cap.as_str())
        .with("domain", domain)
    }

    /// `403`: a role or override names a capability the writer does not hold for
    /// all domains (t32-cfg §8 A19).
    pub fn capability_not_held(cap: Capability) -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            "capabilityNotHeld",
            format!("You cannot grant {cap}: you do not hold it for all domains."),
        )
        .with("capability", cap.as_str())
    }

    /// `404` with a code, so the console tells it from "no such route".
    pub fn not_found(detail: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, "notFound", detail)
    }

    /// `409`: the write conflicts with the state of the server.
    pub fn conflict(code: impl Into<Cow<'static, str>>, detail: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, code, detail)
    }

    /// `412`: the object changed since the caller read it.
    pub fn stale(updated_by: Option<&str>, updated_at: Option<i64>) -> Self {
        Self::new(
            StatusCode::PRECONDITION_FAILED,
            "stale",
            "Someone else changed this since you opened it.",
        )
        .with("updatedBy", updated_by.map(str::to_string))
        .with("updatedAt", updated_at.and_then(iso8601))
    }

    /// `500`; the cause goes to the log, not to the client.
    pub fn internal(e: impl std::fmt::Display) -> Self {
        tracing::warn!("admin error: {e}");
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            "admin operation failed",
        )
    }

    /// Point the error at `field` (a JSON pointer).
    pub fn at(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    /// Add a member to the body.
    pub fn with(mut self, key: &str, value: impl Into<Value>) -> Self {
        self.extra.insert(key.to_string(), value.into());
        self
    }
}

impl std::fmt::Display for AdminApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}: {}", self.status.as_u16(), self.code, self.detail)
    }
}

impl IntoResponse for AdminApiError {
    fn into_response(self) -> Response {
        let mut body = self.extra;
        body.insert("code".into(), Value::String(self.code.into_owned()));
        body.insert("detail".into(), Value::String(self.detail.clone()));
        body.insert("error".into(), Value::String(self.detail));
        if let Some(field) = self.field {
            body.insert("field".into(), Value::String(field));
        }
        let retry = if self.status == StatusCode::TOO_MANY_REQUESTS {
            body.get("retryAfterSecs").and_then(Value::as_u64).filter(|n|*n<=86400)
        } else { None };
        let mut response=(self.status, Json(Value::Object(body))).into_response();
        if let Some(seconds)=retry {
            response.headers_mut().insert(axum::http::header::RETRY_AFTER,seconds.to_string().parse().expect("numeric header"));
            response.headers_mut().insert(axum::http::header::CACHE_CONTROL,"no-store".parse().expect("static header"));
        }
        response
    }
}

impl From<mw_store::StoreError> for AdminApiError {
    fn from(e: mw_store::StoreError) -> Self {
        Self::internal(e)
    }
}

/// A JSON body for an admin write. A body that does not parse is a `422
/// {code:"invalidBody"}` in the admin shape instead of axum's plain-text answer.
pub struct AdminJson<T>(pub T);

impl<S, T> FromRequest<S> for AdminJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AdminApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(v)) => Ok(Self(v)),
            Err(rejection) => Err(json_rejection(rejection)),
        }
    }
}

fn json_rejection(rejection: JsonRejection) -> AdminApiError {
    match &rejection {
        JsonRejection::JsonSyntaxError(_) | JsonRejection::JsonDataError(_) => crate::request_protection::observe(mw_store::request_protection_model::Signal::Malformed),
        JsonRejection::BytesRejection(_) if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE => crate::request_protection::observe(mw_store::request_protection_model::Signal::Oversized),
        _ => {},
    }
    let status = rejection.status();
    let detail = rejection.body_text();
    if status == StatusCode::UNSUPPORTED_MEDIA_TYPE {
        AdminApiError::new(status, "unsupportedMediaType", detail)
    } else {
        AdminApiError::new(StatusCode::UNPROCESSABLE_ENTITY, "invalidBody", detail)
    }
}

// ─── optimistic concurrency ─────────────────────────────────────────────────────

/// The tag of an object as read: a strong `ETag` over its JSON form.
pub fn etag_of(value: &Value) -> String {
    let digest = Sha256::digest(value.to_string().as_bytes());
    let hex: String = digest[..16].iter().map(|b| format!("{b:02x}")).collect();
    format!("\"{hex}\"")
}

/// Refuse a write whose `If-Match` names a version other than `current` (`412
/// {code:"stale", updatedBy, updatedAt}`, `updatedAt` as unix seconds in, ISO 8601
/// out). No `If-Match` header: the write goes ahead (the console always sends
/// one; the CLI and scripts may not). `*` matches any version.
pub fn check_if_match(
    headers: &HeaderMap,
    current: &str,
    updated_by: Option<&str>,
    updated_at: Option<i64>,
) -> Result<(), AdminApiError> {
    let Some(raw) = headers.get(header::IF_MATCH) else {
        return Ok(());
    };
    let raw = raw.to_str().unwrap_or_default();
    let bare = |t: &str| t.trim().trim_start_matches("W/").to_string();
    let current = bare(current);
    if raw.split(',').map(bare).any(|t| t == "*" || t == current) {
        Ok(())
    } else {
        Err(AdminApiError::stale(updated_by, updated_at))
    }
}

/// `resp` with an `ETag` header.
pub fn with_etag(resp: impl IntoResponse, etag: &str) -> Response {
    let mut resp = resp.into_response();
    if let Ok(v) = HeaderValue::from_str(etag) {
        resp.headers_mut().insert(header::ETAG, v);
    }
    resp
}

fn iso8601(unix: i64) -> Option<String> {
    chrono::DateTime::from_timestamp(unix, 0).map(|t| t.to_rfc3339())
}

// ─── the signed-in admin ────────────────────────────────────────────────────────

/// Where a capability applies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DomainScope {
    All,
    /// Lower-case domain names.
    Domains(BTreeSet<String>),
}

impl DomainScope {
    pub fn allows(&self, domain: &str) -> bool {
        match self {
            DomainScope::All => true,
            DomainScope::Domains(set) => set.contains(&domain.to_ascii_lowercase()),
        }
    }

    /// The union of two grants of one capability.
    pub fn union(&mut self, other: &DomainScope) {
        match (&mut *self, other) {
            (DomainScope::All, _) => {}
            (_, DomainScope::All) => *self = DomainScope::All,
            (DomainScope::Domains(a), DomainScope::Domains(b)) => a.extend(b.iter().cloned()),
        }
    }
}

/// The signed-in admin, as the registrar resolved them for this request.
#[derive(Clone, Debug)]
pub struct AdminCtx {
    /// `admin_sessions.admin_id`; the audit actor.
    pub username: String,
    /// `admin_account.id`, when the admin has an account row.
    pub account_id: Option<String>,
    pub display_name: Option<String>,
    /// Role names, for `/admin/me` and the audit's `admin_role` column.
    pub roles: Vec<String>,
    /// The hash of the session token (for "this session" in session lists).
    pub session_hash: String,
    grants: BTreeMap<Capability, DomainScope>,
    /// The capability the route was registered with.
    route_cap: Option<Capability>,
}

impl AdminCtx {
    /// An admin holding every capability for all domains.
    pub fn all_capabilities(username: impl Into<String>) -> Self {
        Self::with_grants(
            username,
            Capability::ALL
                .iter()
                .map(|c| (*c, DomainScope::All))
                .collect(),
        )
    }

    /// An admin holding exactly `grants`.
    pub fn with_grants(
        username: impl Into<String>,
        grants: BTreeMap<Capability, DomainScope>,
    ) -> Self {
        Self {
            username: username.into(),
            account_id: None,
            display_name: None,
            roles: Vec::new(),
            session_hash: String::new(),
            grants,
            route_cap: None,
        }
    }

    /// Resolve the caller from the admin session cookie. `401` without a live
    /// session, or when the panel is turned off (`MW_ADMIN_ENABLED=false`).
    pub(crate) async fn resolve(
        state: &AppState,
        headers: &HeaderMap,
    ) -> Result<Self, AdminApiError> {
        if !state.v6.admin_enabled {
            return Err(AdminApiError::unauthenticated());
        }
        let token = session_cookie(headers).ok_or_else(AdminApiError::unauthenticated)?;
        let hash = crate::push_relay::hash_token(&token);
        let (idle, _) = crate::admin_accounts::session_timeouts(state).await?;
        let now = chrono::Utc::now().timestamp();
        let username = match state
            .store
            .get_admin_session_with_idle(&hash, now, idle)
            .await
        {
            Ok(Some(username)) => username,
            Ok(None) => return Err(AdminApiError::unauthenticated()),
            Err(e) => return Err(AdminApiError::internal(e)),
        };
        let mut ctx = load_grants(state, &username).await?;
        crate::request_protection::administrator(ctx.account_id.as_deref().unwrap_or(&ctx.username)).await?;
        ctx.session_hash = hash;
        Ok(ctx)
    }

    /// Does the admin hold `cap` (for any domain)?
    pub fn has(&self, cap: Capability) -> bool {
        self.grants.contains_key(&cap)
    }

    /// Where `cap` applies, or `None` when it is not held.
    pub fn scope(&self, cap: Capability) -> Option<&DomainScope> {
        self.grants.get(&cap)
    }

    /// Does the admin hold `cap` for all domains?
    pub fn has_unscoped(&self, cap: Capability) -> bool {
        matches!(self.grants.get(&cap), Some(DomainScope::All))
    }

    /// `403` unless the admin holds `cap`, and holds it for all domains when the
    /// capability is unscoped-only.
    pub fn require(&self, cap: Capability) -> Result<(), AdminApiError> {
        match self.grants.get(&cap) {
            None => Err(AdminApiError::forbidden(cap)),
            Some(DomainScope::Domains(_)) if cap.unscoped_only() => {
                Err(AdminApiError::unscoped_required(cap))
            }
            Some(_) => Ok(()),
        }
    }

    /// `403` unless the admin holds `cap` for all domains (global objects, and
    /// granting a capability to someone else).
    pub fn require_unscoped(&self, cap: Capability) -> Result<(), AdminApiError> {
        match self.grants.get(&cap) {
            None => Err(AdminApiError::forbidden(cap)),
            Some(DomainScope::Domains(_)) => Err(AdminApiError::unscoped_required(cap)),
            Some(DomainScope::All) => Ok(()),
        }
    }

    /// `403` unless the admin holds `cap` for `domain`.
    pub fn require_domain_for(&self, cap: Capability, domain: &str) -> Result<(), AdminApiError> {
        match self.grants.get(&cap) {
            None => Err(AdminApiError::forbidden(cap)),
            Some(scope) if scope.allows(domain) => Ok(()),
            Some(_) => Err(AdminApiError::out_of_scope(cap, domain)),
        }
    }

    /// `403` unless the route's capability is held for `domain`. A route
    /// registered without a capability (signed-in only) allows no domain.
    pub fn require_domain(&self, domain: &str) -> Result<(), AdminApiError> {
        match self.route_cap {
            Some(cap) => self.require_domain_for(cap, domain),
            None => Err(AdminApiError::new(
                StatusCode::FORBIDDEN,
                "forbidden",
                "This route acts on no domain.",
            )),
        }
    }

    /// `403` unless the route's capability is held for the domain of `address`.
    pub fn require_address(&self, address: &str) -> Result<(), AdminApiError> {
        self.require_domain(&domain_of(address))
    }

    /// The domains a list answered under `cap` is limited to: `None` = all.
    pub fn domain_filter(&self, cap: Capability) -> Option<&BTreeSet<String>> {
        match self.grants.get(&cap) {
            Some(DomainScope::Domains(set)) => Some(set),
            _ => None,
        }
    }

    /// The capability the route was registered with.
    pub fn route_capability(&self) -> Option<Capability> {
        self.route_cap
    }

    /// The capabilities held, in catalogue order.
    pub fn capabilities(&self) -> Vec<Capability> {
        self.grants.keys().copied().collect()
    }

    /// Every domain any grant is limited to, or `None` when some grant covers all
    /// domains (`/admin/me`'s `domainScope`).
    pub fn domain_scope(&self) -> Option<BTreeSet<String>> {
        let mut out = BTreeSet::new();
        for scope in self.grants.values() {
            match scope {
                DomainScope::All => return None,
                DomainScope::Domains(set) => out.extend(set.iter().cloned()),
            }
        }
        Some(out)
    }

    /// The audit's `admin_role` column: role names, comma-separated.
    pub fn audit_roles(&self) -> String {
        self.roles.join(",")
    }

    pub(crate) fn for_route(mut self, cap: Option<Capability>) -> Self {
        self.route_cap = cap;
        self
    }
}

impl<S: Send + Sync> FromRequestParts<S> for AdminCtx {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        match parts.extensions.get::<AdminCtx>() {
            Some(ctx) => Ok(ctx.clone()),
            // Only the registrar inserts it. A handler mounted some other way that
            // asks for one is a bug; refuse rather than run without a check.
            None => {
                tracing::error!(
                    "admin handler for {} is not mounted through admin_router::admin_endpoint",
                    parts.uri.path()
                );
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "code": "notRegistered",
                        "detail": "admin route not registered",
                        "error": "admin route not registered",
                    })),
                )
                    .into_response())
            }
        }
    }
}

/// The lower-cased domain part of `address` (the whole value when it has no `@`).
pub fn domain_of(address: &str) -> String {
    address
        .rsplit_once('@')
        .map_or(address, |(_, d)| d)
        .to_ascii_lowercase()
}

/// The admin session cookie's value.
fn session_cookie(headers: &HeaderMap) -> Option<String> {
    let name = mw_admin::config::session_cookie_name();
    crate::hardening::cookie(headers, &name)
}

/// The grants of `username` (`admin_roles::grants_for`).
async fn load_grants(state: &AppState, username: &str) -> Result<AdminCtx, AdminApiError> {
    crate::admin_roles::grants_for(state, username).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scoped(caps: &[(Capability, Option<&[&str]>)]) -> AdminCtx {
        AdminCtx::with_grants(
            "alice",
            caps.iter()
                .map(|(c, d)| {
                    let scope = match d {
                        None => DomainScope::All,
                        Some(list) => {
                            DomainScope::Domains(list.iter().map(|s| s.to_string()).collect())
                        }
                    };
                    (*c, scope)
                })
                .collect(),
        )
    }

    #[test]
    fn scope_checks() {
        let ctx = scoped(&[
            (Capability::UsersRead, Some(&["example.com"])),
            (Capability::LogsRead, Some(&["example.com"])),
            (Capability::AuditRead, None),
        ]);
        assert!(ctx.require(Capability::UsersRead).is_ok());
        assert_eq!(
            ctx.require(Capability::UsersWrite).unwrap_err().code,
            "forbidden"
        );
        // logs.read is unscoped-only: a domain-limited grant does not carry it.
        assert_eq!(
            ctx.require(Capability::LogsRead).unwrap_err().code,
            "unscopedGrantRequired"
        );
        let ctx = ctx.for_route(Some(Capability::UsersRead));
        assert!(ctx.require_address("Bob@Example.COM").is_ok());
        assert_eq!(
            ctx.require_address("bob@other.org").unwrap_err().code,
            "outOfScope"
        );
        assert_eq!(
            ctx.domain_filter(Capability::UsersRead)
                .unwrap()
                .iter()
                .collect::<Vec<_>>(),
            vec!["example.com"]
        );
        assert!(ctx.domain_filter(Capability::AuditRead).is_none());
        assert_eq!(ctx.domain_scope(), None, "audit.read covers all domains");
        assert!(ctx.require_unscoped(Capability::AuditRead).is_ok());
        assert!(ctx.require_unscoped(Capability::UsersRead).is_err());
    }

    #[test]
    fn scope_union() {
        let mut a = DomainScope::Domains(["a.example".to_string()].into());
        a.union(&DomainScope::Domains(["b.example".to_string()].into()));
        assert!(a.allows("b.example") && a.allows("A.EXAMPLE"));
        a.union(&DomainScope::All);
        assert_eq!(a, DomainScope::All);
    }

    #[test]
    fn if_match() {
        let v = json!({"a": 1});
        let tag = etag_of(&v);
        assert_ne!(tag, etag_of(&json!({"a": 2})));
        let mut h = HeaderMap::new();
        assert!(check_if_match(&h, &tag, None, None).is_ok(), "no header");
        h.insert(header::IF_MATCH, HeaderValue::from_str(&tag).unwrap());
        assert!(check_if_match(&h, &tag, None, None).is_ok());
        h.insert(
            header::IF_MATCH,
            HeaderValue::from_str(&format!("W/{tag}")).unwrap(),
        );
        assert!(check_if_match(&h, &tag, None, None).is_ok(), "weak form");
        h.insert(header::IF_MATCH, HeaderValue::from_static("\"other\""));
        let e = check_if_match(&h, &tag, Some("root"), Some(1_760_000_000)).unwrap_err();
        assert_eq!(e.status, StatusCode::PRECONDITION_FAILED);
        assert_eq!(e.code, "stale");
        assert_eq!(e.extra["updatedBy"], "root");
        assert!(e.extra["updatedAt"].as_str().unwrap().starts_with("2025-"));
        h.insert(header::IF_MATCH, HeaderValue::from_static("*"));
        assert!(check_if_match(&h, &tag, None, None).is_ok());
    }

    #[tokio::test]
    async fn error_body_shape() {
        let resp = AdminApiError::invalid("outOfRange", "Port must be 1-65535.", "/smtp/port")
            .into_response();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = axum::body::to_bytes(resp.into_body(), 1 << 16)
            .await
            .unwrap();
        let v: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            v,
            json!({
                "code": "outOfRange",
                "detail": "Port must be 1-65535.",
                "error": "Port must be 1-65535.",
                "field": "/smtp/port",
            })
        );
    }
}
