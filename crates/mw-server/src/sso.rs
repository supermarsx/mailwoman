//! SSO login routes (t9-e3, plan §2/§3, SPEC §18.3). OIDC + SAML 2.0 as **login
//! backends** on top of the frozen `mw-sso` [`SsoLogin`] trait.
//!
//! `/api/sso/*` surface (all additive; the password/`/api/login` path is untouched):
//!   * `GET  /api/sso/providers` — advertise the enabled IdPs for a domain, PRE-auth (id + kind + display name only, never a secret).
//!   * `GET  /api/sso/{id}/begin` — 302 to the IdP (stores per-flow [`PendingState`]).
//!   * `GET  /api/sso/{id}/callback` — OIDC code exchange → session.
//!   * `POST /api/sso/{id}/acs` — SAML assertion consumer → session.
//!   * `GET  /api/sso/{id}/metadata` — SAML SP metadata.
//!   * `POST /api/sso/logout` — clear the session (+ optional IdP logout URL).
//!
//! ## Session reuse (task §1, no new mechanism)
//! On a successful [`SsoLogin::complete`] we resolve the [`SsoIdentity`] to a
//! Mailwoman account (honouring [`FirstLoginPolicy`] — default **deny/allowlist**,
//! NO auto-registration unless the admin opted in) and then mint the SAME opaque
//! `mw_session` cookie the password path issues, via the existing
//! [`crate::finish_login`]. The browser is 303'd back into the SPA carrying that
//! cookie. This crate never invents a session.
//!
//! ## Uniform 401 (task §1 / §9 R3)
//! Every [`SsoError`] — and a first-login denial — collapses to the SAME 401 body.
//! The route NEVER branches on the error variant (that would leak which check
//! failed); the variant only names the reason token in the content-free
//! `sso_login_audit`.
//!
//! ## Replay / CSRF (§9 R3)
//! `begin` mints per-flow [`PendingState`] (PKCE verifier + nonce for OIDC, the
//! AuthnRequest RequestID for SAML) held server-side in [`PendingFlows`], keyed by
//! the provider's opaque 256-bit `state` token. The callback/ACS resolve it **once**
//! ([`PendingFlows::take`] is one-shot): an unknown / reused / expired token is a
//! [`SsoError::Replay`] → 401. That server-side binding — not a cookie — is the
//! replay/CSRF defence, so `lib.rs` exempts exactly the callback/ACS paths from the
//! cookie-CSRF guard (they carry IdP-signed state, not an ambient session).
#![allow(clippy::result_large_err)]

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::{Extension, Form, Path as UrlPath, Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;

use mw_sso::{
    ClaimMap, FirstLoginPolicy, PendingState, SsoBackend, SsoCallback, SsoConfig, SsoIdentity,
    SsoKind, SsoLogin, SsoScope,
};
use mw_store::{Credentials, Store};

use crate::AppState;

/// How long an in-flight SSO login round-trip may take before its [`PendingState`]
/// expires (login redirects are short; a few minutes is generous).
pub(crate) const PENDING_TTL: Duration = Duration::from_secs(600);

// ─────────────────────────────────────────────────────────────────────────────
// Injected request extensions
// ─────────────────────────────────────────────────────────────────────────────

/// A fully-resolved provider ready to serve, plus the routing metadata the route
/// needs that the object-safe [`SsoLogin`] trait does not expose (the first-login
/// policy + claim map used to resolve an identity to an account).
#[derive(Clone)]
pub struct SsoEntry {
    /// The protocol implementation ([`mw_sso::OidcProvider`] / [`mw_sso::SamlProvider`]
    /// in production; a mock in the unit gate).
    pub provider: Arc<dyn SsoLogin>,
    /// Routing/resolution metadata mirrored from the backend config.
    pub meta: SsoMeta,
}

/// Non-secret backend metadata used for advertising + identity→account resolution.
#[derive(Clone)]
pub struct SsoMeta {
    /// Admin-facing label ("Sign in with Acme SSO").
    pub display_name: String,
    /// Protocol.
    pub kind: SsoKind,
    /// Deployment-wide or domain-scoped.
    pub scope: SsoScope,
    /// Whether the backend is advertised + usable.
    pub enabled: bool,
    /// First-login behaviour (default deny/allowlist).
    pub first_login_policy: FirstLoginPolicy,
    /// IdP claim/attribute → account-field mapping.
    pub claim_map: ClaimMap,
}

/// Where the route obtains providers. Production builds them on demand from the
/// 0009 `sso_config` store rows ([`Store`](Self::Store)); the unit gate injects
/// mock [`SsoLogin`] impls ([`Mock`](Self::Mock)) so the routes can be driven
/// without a live IdP.
#[derive(Clone)]
pub enum SsoProviderSource {
    /// Build from the persisted `sso_config` rows (unsealing the secret via the
    /// store `ServerKey`).
    Store,
    /// Test injection: a fixed id → [`SsoEntry`] map.
    Mock(Arc<HashMap<String, SsoEntry>>),
}

impl SsoProviderSource {
    /// Resolve one backend by id, or `None` (unknown / build failure).
    async fn resolve(&self, store: &Store, id: &str) -> Option<SsoEntry> {
        match self {
            SsoProviderSource::Mock(map) => map.get(id).cloned(),
            SsoProviderSource::Store => {
                let row = store.get_sso_config(id).await.ok()??;
                build_entry_from_row(&row).ok()
            }
        }
    }

    /// The enabled backends advertised for a login attempt. Deployment-wide backends
    /// are always included; a domain-scoped backend is included only when its domain
    /// matches `domain` (so a user can't enumerate other domains' IdPs).
    async fn advertised(&self, store: &Store, domain: Option<&str>) -> Vec<(String, SsoMeta)> {
        let mut out: Vec<(String, SsoMeta)> = Vec::new();
        match self {
            SsoProviderSource::Mock(map) => {
                for (id, entry) in map.iter() {
                    if entry.meta.enabled && scope_matches(&entry.meta.scope, domain) {
                        out.push((id.clone(), entry.meta.clone()));
                    }
                }
            }
            SsoProviderSource::Store => {
                let mut rows = store
                    .list_sso_config("deployment")
                    .await
                    .unwrap_or_default();
                if let Some(d) = domain {
                    rows.extend(
                        store
                            .list_sso_config(&format!("domain:{d}"))
                            .await
                            .unwrap_or_default(),
                    );
                }
                for row in rows {
                    if !row.enabled {
                        continue;
                    }
                    if let Ok(entry) = build_entry_from_row(&row) {
                        out.push((row.id, entry.meta));
                    }
                }
            }
        }
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }
}

/// Build a serve-ready [`SsoEntry`] (metadata + constructed provider) from a store
/// row: parse the JSON config + claim map, attach the unsealed secret, and construct
/// the matching provider. A malformed row / config is dropped (logged), never served.
fn build_entry_from_row(row: &mw_store::SsoConfigRow) -> Result<SsoEntry, mw_sso::SsoError> {
    let config: SsoConfig = serde_json::from_str(&row.config_json)
        .map_err(|e| mw_sso::SsoError::Config(format!("bad config json: {e}")))?;
    let claim_map: ClaimMap = serde_json::from_str(&row.claim_map_json).unwrap_or_default();
    let meta = SsoMeta {
        display_name: row.display_name.clone(),
        kind: config.kind(),
        scope: SsoScope::parse(&row.scope),
        enabled: row.enabled,
        first_login_policy: config.first_login_policy(),
        claim_map: claim_map.clone(),
    };
    let backend = SsoBackend {
        id: row.id.clone(),
        kind: config.kind(),
        display_name: row.display_name.clone(),
        scope: SsoScope::parse(&row.scope),
        enabled: row.enabled,
        config,
        claim_map,
        secret: row.secret.clone(),
    };
    let provider: Arc<dyn SsoLogin> = match backend.kind {
        SsoKind::Oidc => Arc::new(mw_sso::OidcProvider::new(backend)?),
        SsoKind::Saml => Arc::new(mw_sso::SamlProvider::new(backend)?),
    };
    Ok(SsoEntry { provider, meta })
}

/// Whether a backend scope is offered to a login for `domain`.
fn scope_matches(scope: &SsoScope, domain: Option<&str>) -> bool {
    match scope {
        SsoScope::Deployment => true,
        SsoScope::Domain(d) => domain.is_some_and(|q| q.eq_ignore_ascii_case(d)),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Pending-flow store (keyed by the provider's opaque `state` token; one-shot)
// ─────────────────────────────────────────────────────────────────────────────

struct PendingEntry {
    state: PendingState,
    expires_at: Instant,
}

/// In-flight [`PendingState`], keyed by the provider-minted opaque `state` token and
/// consumed **exactly once** on callback (replay/CSRF defence, §9 R3). Distinct from
/// `mw_sso::PendingStore` (which mints its own key); e3 keys by the provider token so
/// the value the IdP echoes back in `state`/`RelayState` looks the flow up directly.
pub struct PendingFlows {
    ttl: Duration,
    inner: Mutex<HashMap<String, PendingEntry>>,
}

impl PendingFlows {
    /// A store whose entries expire `ttl` after insertion.
    pub fn new(ttl: Duration) -> Self {
        PendingFlows {
            ttl,
            inner: Mutex::new(HashMap::new()),
        }
    }

    /// Persist `state` under the provider's opaque `token`. Sweeps expired entries.
    fn insert(&self, token: String, state: PendingState) {
        let expires_at = Instant::now() + self.ttl;
        let mut map = self.inner.lock().expect("PendingFlows poisoned");
        map.retain(|_, e| e.expires_at > Instant::now());
        map.insert(token, PendingEntry { state, expires_at });
    }

    /// Redeem a token exactly once (removes it). `None` ⇒ unknown / reused / expired
    /// (the caller treats all three as [`SsoError::Replay`](mw_sso::SsoError::Replay)).
    fn take(&self, token: &str) -> Option<PendingState> {
        let mut map = self.inner.lock().expect("PendingFlows poisoned");
        let entry = map.remove(token)?;
        (entry.expires_at > Instant::now()).then_some(entry.state)
    }

    /// Live entry count (tests/metrics).
    #[cfg(test)]
    fn len(&self) -> usize {
        let mut map = self.inner.lock().expect("PendingFlows poisoned");
        map.retain(|_, e| e.expires_at > Instant::now());
        map.len()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Router
// ─────────────────────────────────────────────────────────────────────────────

/// The `/api/sso/*` router. `lib.rs` merges this and layers the injected
/// [`SsoProviderSource`] + shared [`PendingFlows`].
pub(crate) fn sso_router() -> Router<AppState> {
    Router::new()
        .route("/api/sso/providers", get(providers))
        .route("/api/sso/{id}/begin", get(begin))
        .route("/api/sso/{id}/callback", get(callback))
        .route("/api/sso/{id}/acs", post(acs))
        .route("/api/sso/{id}/metadata", get(metadata))
        .route("/api/sso/logout", post(logout))
}

/// The uniform SSO failure: a single 401 body for EVERY error + first-login denial,
/// so a caller can never tell which check failed.
fn sso_unauthorized() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "authentication failed" })),
    )
        .into_response()
}

// ── GET /api/sso/providers ───────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ProvidersQuery {
    /// The mail domain the user is signing in for (from the typed email). Selects
    /// domain-scoped IdPs in addition to the deployment-wide ones.
    #[serde(default)]
    domain: Option<String>,
}

/// `GET /api/sso/providers?domain=` — the enabled IdP buttons the login screen
/// renders. PRE-auth (the login screen is unauthenticated). Returns id + kind +
/// display name ONLY — never any config or secret.
async fn providers(
    State(state): State<AppState>,
    Extension(source): Extension<SsoProviderSource>,
    Query(q): Query<ProvidersQuery>,
) -> Response {
    let list = source.advertised(&state.store, q.domain.as_deref()).await;
    let providers: Vec<_> = list
        .into_iter()
        .map(|(id, meta)| {
            json!({
                "id": id,
                "kind": meta.kind.as_db(),
                "displayName": meta.display_name,
            })
        })
        .collect();
    Json(json!({ "providers": providers })).into_response()
}

// ── GET /api/sso/{id}/begin ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct BeginQuery {
    /// Opaque post-login return target (deep-link back to the requested screen).
    #[serde(default, rename = "relayState")]
    relay_state: Option<String>,
}

/// `GET /api/sso/{id}/begin?relayState=` — start a login: build the IdP redirect,
/// stash the per-flow [`PendingState`], and 302 the browser to the IdP.
async fn begin(
    State(state): State<AppState>,
    Extension(source): Extension<SsoProviderSource>,
    Extension(pending): Extension<Arc<PendingFlows>>,
    UrlPath(id): UrlPath<String>,
    Query(q): Query<BeginQuery>,
) -> Response {
    let Some(entry) = source.resolve(&state.store, &id).await else {
        return sso_unauthorized();
    };
    if !entry.meta.enabled {
        return sso_unauthorized();
    }
    match entry.provider.begin(q.relay_state).await {
        Ok(br) => {
            pending.insert(br.state_token, br.pending);
            Redirect::to(&br.url).into_response()
        }
        Err(e) => {
            tracing::warn!(provider = %id, reason = e.audit_reason(), "sso begin failed");
            sso_unauthorized()
        }
    }
}

// ── GET /api/sso/{id}/callback (OIDC) ────────────────────────────────────────

/// `GET /api/sso/{id}/callback?code=&state=` — the OIDC redirect landing. Resolves
/// the one-shot pending flow from `state`, exchanges the code, and issues the session.
async fn callback(
    State(state): State<AppState>,
    Extension(source): Extension<SsoProviderSource>,
    Extension(pending): Extension<Arc<PendingFlows>>,
    UrlPath(id): UrlPath<String>,
    Query(params): Query<BTreeMap<String, String>>,
) -> Response {
    complete_flow(&state, &source, &pending, &id, params, "state").await
}

// ── POST /api/sso/{id}/acs (SAML) ────────────────────────────────────────────

/// `POST /api/sso/{id}/acs` — the SAML assertion consumer service. The IdP POSTs a
/// (base64) `SAMLResponse` + `RelayState` as `application/x-www-form-urlencoded`.
async fn acs(
    State(state): State<AppState>,
    Extension(source): Extension<SsoProviderSource>,
    Extension(pending): Extension<Arc<PendingFlows>>,
    UrlPath(id): UrlPath<String>,
    Form(params): Form<BTreeMap<String, String>>,
) -> Response {
    complete_flow(&state, &source, &pending, &id, params, "RelayState").await
}

/// Shared completion path for OIDC callback + SAML ACS: resolve the one-shot pending
/// flow (`state_key` names the correlator param), call [`SsoLogin::complete`], map
/// the identity to an account under the first-login policy, and — on success — mint
/// the normal session and 303 back into the app. Every failure is a uniform 401.
async fn complete_flow(
    state: &AppState,
    source: &SsoProviderSource,
    pending: &PendingFlows,
    id: &str,
    params: BTreeMap<String, String>,
    state_key: &str,
) -> Response {
    let Some(entry) = source.resolve(&state.store, id).await else {
        return sso_unauthorized();
    };
    // Resolve + consume the one-shot pending flow; missing/reused/expired ⇒ replay.
    let Some(token) = params.get(state_key) else {
        return audit_and_401(state, id, entry.meta.kind, "", "replay").await;
    };
    let Some(ps) = pending.take(token) else {
        return audit_and_401(state, id, entry.meta.kind, "", "replay").await;
    };
    let relay = ps.relay_state().map(str::to_string);
    let callback = SsoCallback {
        params,
        pending: ps,
    };

    let identity = match entry.provider.complete(callback).await {
        Ok(identity) => identity,
        Err(e) => {
            return audit_and_401(state, id, entry.meta.kind, "", e.audit_reason()).await;
        }
    };
    let subject_hash = subject_hash(&identity.subject);

    // Resolve the identity to a Mailwoman account under the first-login policy.
    let accounts = state.store.list_accounts().await.unwrap_or_default();
    let pairs: Vec<(String, String)> = accounts.into_iter().map(|a| (a.id, a.username)).collect();
    let Some(account) = resolve_account(
        &identity,
        &entry.meta.claim_map,
        &pairs,
        entry.meta.first_login_policy,
    ) else {
        // First-login denied (allowlist default): uniform 401, content-free audit.
        let _ = state
            .store
            .append_sso_login_audit(
                id,
                entry.meta.kind.as_db(),
                &subject_hash,
                "error:first_login",
            )
            .await;
        return sso_unauthorized();
    };

    // Mint the SAME opaque session the password path issues (no new mechanism). SSO
    // sessions carry no upstream Basic-auth creds to proxy (engine serves the mailbox
    // locally), so the sealed credential slot is empty.
    let creds = Credentials {
        username: account.username.clone(),
        password: String::new(),
    };
    let session_id = match state
        .store
        .create_session(&account.id, &account.username, "sso", "sso", &creds)
        .await
    {
        Ok(sid) => sid,
        Err(e) => {
            tracing::error!("failed to persist sso session: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "server error").into_response();
        }
    };
    state.sessions.begin(&session_id);
    let _ = state
        .store
        .append_sso_login_audit(id, entry.meta.kind.as_db(), &subject_hash, "ok")
        .await;

    // Reuse the existing login finisher for the cookie/CSRF mechanism, then 303 the
    // browser back into the SPA carrying those cookies (a browser navigation, not a
    // JSON fetch — so `client_type` is None).
    let login = crate::finish_login(state, &session_id, &account.id, &account.username, None).await;
    into_login_redirect(login, relay.as_deref())
}

/// Append a content-free audit row for a pre-account failure and return the uniform
/// 401. `subject_hash` is empty when the identity never resolved.
async fn audit_and_401(
    state: &AppState,
    id: &str,
    kind: SsoKind,
    subject_hash: &str,
    reason: &str,
) -> Response {
    let _ = state
        .store
        .append_sso_login_audit(id, kind.as_db(), subject_hash, &format!("error:{reason}"))
        .await;
    sso_unauthorized()
}

// ── GET /api/sso/{id}/metadata (SAML SP metadata) ────────────────────────────

/// `GET /api/sso/{id}/metadata` — publish the SAML SP metadata (entity id / ACS /
/// certs). `404` for OIDC (which advertises via IdP discovery, not SP metadata).
async fn metadata(
    State(state): State<AppState>,
    Extension(source): Extension<SsoProviderSource>,
    UrlPath(id): UrlPath<String>,
) -> Response {
    let Some(entry) = source.resolve(&state.store, &id).await else {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    };
    match entry.provider.metadata() {
        Some(md) => {
            let mut resp = md.body.into_response();
            if let Ok(v) = header::HeaderValue::from_str(&md.content_type) {
                resp.headers_mut().insert(header::CONTENT_TYPE, v);
            }
            resp
        }
        None => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}

// ── POST /api/sso/logout ─────────────────────────────────────────────────────

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LogoutReq {
    /// The backend the session logged in through (to build an upstream logout URL).
    #[serde(default)]
    provider_id: Option<String>,
}

/// `POST /api/sso/logout` — clear the mailbox session (like `/api/logout`) and, when
/// the client names the provider, return the upstream RP-initiated logout / SLO URL
/// for the app to navigate to. Session-authed + CSRF-protected (a first-party action,
/// NOT exempt like the IdP callbacks).
async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(source): Extension<SsoProviderSource>,
    Json(body): Json<LogoutReq>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let mut logout_url = None;
    if let Some(pid) = &body.provider_id
        && let Some(entry) = source.resolve(&state.store, pid).await
    {
        logout_url = entry.provider.logout(&session.username).map(|r| r.url);
    }
    if let Some(id) = crate::cookie_value(&headers) {
        let _ = state.store.delete_session(&id).await;
        state.sessions.forget(&id);
    }
    let mut resp = Json(json!({ "ok": true, "logoutUrl": logout_url })).into_response();
    resp.headers_mut()
        .append(header::SET_COOKIE, crate::clear_cookie(state.cookie_secure));
    resp
}

// ─────────────────────────────────────────────────────────────────────────────
// Pure helpers (unit-tested)
// ─────────────────────────────────────────────────────────────────────────────

/// The content-free identifier written to `sso_login_audit` for an IdP subject: a
/// SHA-256 hash (§21.1), NEVER the raw sub/NameID — the audit is append-only + free
/// of tokens, assertions, and mail content.
fn subject_hash(subject: &str) -> String {
    crate::push_relay::hash_token(subject)
}

/// A resolved Mailwoman account for an SSO identity.
struct ResolvedAccount {
    id: String,
    username: String,
}

/// The login identifier an [`SsoIdentity`] maps to: the mapped email claim, then the
/// asserted email, then the mapped username claim, then the subject.
fn resolved_login(identity: &SsoIdentity, claim_map: &ClaimMap) -> String {
    if let Some(key) = &claim_map.email
        && let Some(v) = identity.claims.get(key)
    {
        return v.clone();
    }
    if let Some(email) = &identity.email {
        return email.clone();
    }
    if let Some(key) = &claim_map.username
        && let Some(v) = identity.claims.get(key)
    {
        return v.clone();
    }
    identity.subject.clone()
}

/// Resolve an [`SsoIdentity`] to a Mailwoman account under the first-login policy.
/// `accounts` is `(id, username)` for every provisioned account. An existing account
/// (matched case-insensitively on username) always wins; otherwise the policy decides:
/// [`FirstLoginPolicy::AutoCreate`] admits the asserted identity, the default
/// [`FirstLoginPolicy::Allowlist`] denies (returns `None`) — NO auto-registration.
fn resolve_account(
    identity: &SsoIdentity,
    claim_map: &ClaimMap,
    accounts: &[(String, String)],
    policy: FirstLoginPolicy,
) -> Option<ResolvedAccount> {
    let login = resolved_login(identity, claim_map);
    if let Some((id, username)) = accounts
        .iter()
        .find(|(_, u)| u.eq_ignore_ascii_case(&login))
    {
        return Some(ResolvedAccount {
            id: id.clone(),
            username: username.clone(),
        });
    }
    match policy {
        FirstLoginPolicy::AutoCreate => Some(ResolvedAccount {
            id: login.clone(),
            username: login,
        }),
        FirstLoginPolicy::Allowlist => None,
    }
}

/// A safe post-login return target: only a same-site absolute path is honoured (no
/// scheme, no protocol-relative `//host`) — otherwise `/` (defeats open-redirect).
fn safe_relay(relay: Option<&str>) -> String {
    match relay {
        Some(r) if r.starts_with('/') && !r.starts_with("//") => r.to_string(),
        _ => "/".to_string(),
    }
}

/// Turn the existing login finisher's response into a 303 back into the SPA, carrying
/// over its `Set-Cookie` headers (session + CSRF). The browser followed a redirect
/// from the IdP, so it wants a redirect, not the JSON body.
fn into_login_redirect(login: Response, relay: Option<&str>) -> Response {
    let cookies: Vec<_> = login
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .cloned()
        .collect();
    let target = safe_relay(relay);
    let mut resp = Redirect::to(&target).into_response();
    let h = resp.headers_mut();
    for c in cookies {
        h.append(header::SET_COOKIE, c);
    }
    resp
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(subject: &str, email: Option<&str>) -> SsoIdentity {
        SsoIdentity {
            subject: subject.into(),
            email: email.map(str::to_string),
            display_name: None,
            groups: vec![],
            claims: BTreeMap::new(),
        }
    }

    #[test]
    fn allowlist_denies_unknown_subject_by_default() {
        let id = identity("sub-1", Some("stranger@acme.test"));
        let accounts = vec![("acct-a".to_string(), "known@acme.test".to_string())];
        assert!(
            resolve_account(
                &id,
                &ClaimMap::default(),
                &accounts,
                FirstLoginPolicy::Allowlist
            )
            .is_none(),
            "an unknown identity must be denied under the default allowlist policy"
        );
    }

    #[test]
    fn allowlist_admits_a_matching_account() {
        let id = identity("sub-1", Some("Known@Acme.Test"));
        let accounts = vec![("acct-a".to_string(), "known@acme.test".to_string())];
        let r = resolve_account(
            &id,
            &ClaimMap::default(),
            &accounts,
            FirstLoginPolicy::Allowlist,
        )
        .expect("matching account admitted");
        assert_eq!(r.id, "acct-a");
        assert_eq!(r.username, "known@acme.test");
    }

    #[test]
    fn autocreate_admits_the_asserted_identity() {
        let id = identity("sub-1", Some("new@acme.test"));
        let r = resolve_account(&id, &ClaimMap::default(), &[], FirstLoginPolicy::AutoCreate)
            .expect("autocreate admits");
        assert_eq!(r.username, "new@acme.test");
    }

    #[test]
    fn resolved_login_prefers_mapped_claim_then_email_then_subject() {
        let mut id = identity("the-subject", None);
        assert_eq!(resolved_login(&id, &ClaimMap::default()), "the-subject");
        id.email = Some("e@x".into());
        assert_eq!(resolved_login(&id, &ClaimMap::default()), "e@x");
        id.claims.insert("upn".into(), "mapped@x".into());
        let cm = ClaimMap {
            email: Some("upn".into()),
            ..Default::default()
        };
        assert_eq!(resolved_login(&id, &cm), "mapped@x");
    }

    #[test]
    fn safe_relay_defeats_open_redirect() {
        assert_eq!(safe_relay(Some("/inbox")), "/inbox");
        assert_eq!(safe_relay(Some("//evil.example")), "/");
        assert_eq!(safe_relay(Some("https://evil.example")), "/");
        assert_eq!(safe_relay(None), "/");
    }

    #[test]
    fn scope_matches_deployment_and_domain() {
        assert!(scope_matches(&SsoScope::Deployment, None));
        assert!(scope_matches(&SsoScope::Deployment, Some("acme.test")));
        assert!(scope_matches(
            &SsoScope::Domain("acme.test".into()),
            Some("ACME.test")
        ));
        assert!(!scope_matches(&SsoScope::Domain("acme.test".into()), None));
        assert!(!scope_matches(
            &SsoScope::Domain("acme.test".into()),
            Some("other.test")
        ));
    }

    #[test]
    fn audit_subject_is_hashed_not_raw() {
        // §21.1: the identifier written to sso_login_audit is a non-reversible hash of
        // the IdP subject, never the raw sub/NameID.
        let raw = "alice@corp.example|nameid-12345";
        let h = subject_hash(raw);
        assert_ne!(h, raw);
        assert!(!h.contains("alice"));
        assert_eq!(h.len(), 64, "sha-256 hex");
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn pending_flows_are_one_shot_and_ttl_swept() {
        let flows = PendingFlows::new(Duration::from_secs(300));
        let ps = PendingState::Oidc {
            pkce_verifier: "v".into(),
            nonce: "n".into(),
            relay_state: Some("/inbox".into()),
        };
        flows.insert("tok".into(), ps.clone());
        assert_eq!(flows.len(), 1);
        assert_eq!(flows.take("tok"), Some(ps));
        // One-shot: a second redemption fails (replay defence).
        assert_eq!(flows.take("tok"), None);
        assert_eq!(flows.len(), 0);

        // TTL of 0 ⇒ already expired on read.
        let expired = PendingFlows::new(Duration::from_millis(0));
        expired.insert(
            "t2".into(),
            PendingState::Saml {
                request_id: "_r".into(),
                relay_state: None,
            },
        );
        assert_eq!(expired.take("t2"), None);
    }
}
