//! OAuth 2.0 Dynamic Client Registration (RFC 7591) + client-configuration
//! (RFC 7592) — admin/policy-gated, **default disabled** (t10 plan §1.5/§3 e8).
//!
//! This module is transport-agnostic and additive: it does not touch the existing
//! [`AuthServer`](crate::AuthServer) behaviour, the [`Scope`] model, or the 0007
//! schema. A DCR-issued client is a perfectly ordinary [`OAuthClient`] persisted
//! into the 0007 `oauth_clients` table via the frozen [`OAuthStore`] seam; the
//! RFC-7591 extras (software id/version, contacts, and the **hash** of the
//! registration-access-token) are recorded by the caller in the 0010
//! `oauth_client_meta` side table.
//!
//! ## Gate (all enforced by the caller from the 0010 `oauth_dcr` policy row)
//! - **Disabled by default.** Absent/`enabled=false` policy ⇒ [`DcrError::Disabled`]
//!   (the `mw-server` handler maps this to `403`).
//! - **Redirect-host allowlist.** Every `redirect_uri` host must match a configured
//!   `allowed_redirect_host_suffixes` entry; an empty allowlist denies everything.
//! - **No scope escalation.** A DCR client is granted exactly the policy's
//!   `default_scope` — the client's requested `scope` is advisory only and can never
//!   widen it. The [`Scope`] grant/deny matrix is unchanged.
//! - **Optional initial-access-token.** `require_initial_access_token` is honoured by
//!   the handler (it gates the endpoint, not this pure core).
//!
//! ## Client-configuration (RFC 7592)
//! Read/update/delete of a registered client are authorised by the per-client
//! **registration-access-token** minted here. Only its hash is stored; verify a
//! presented token with [`verify_registration_access_token`]. That token authorises
//! *only* its own client's configuration — it grants no API/mail scope.

use serde::{Deserialize, Serialize};

use crate::store::OAuthStore;
use crate::util::{b64url, ct_eq, random_bytes, sha256_hex};
use crate::{OAuthClient, OAuthError, Scope, ScopeSelector};

/// The `created_via` marker written to `oauth_client_meta.created_via` for
/// DCR-issued clients, and the `approved_by` sentinel stored on the 0007 client row
/// (there is no human approver — the policy gate is the approval).
pub const DCR_CREATED_VIA: &str = "dcr";
/// The `approved_by` sentinel for a self-registered (DCR) client.
pub const DCR_APPROVED_BY: &str = "dcr:self-registered";

/// The admin/policy gate for DCR, materialised from the 0010 `oauth_dcr` singleton
/// (plus the request-derived issuer base URL). Constructed by the `mw-server`
/// handler; this crate never reads the DB.
#[derive(Debug, Clone)]
pub struct DcrPolicy {
    /// Master switch. `false` (the default) ⇒ every registration is refused.
    pub enabled: bool,
    /// When `true`, the endpoint additionally requires a valid initial-access-token
    /// (enforced by the handler, not by [`register`]).
    pub require_initial_access_token: bool,
    /// Allowed `redirect_uri` host suffixes (e.g. `["example.com"]` matches
    /// `app.example.com` and `example.com`). **Empty ⇒ deny all** (deny-by-default).
    pub allowed_redirect_host_suffixes: Vec<String>,
    /// The [`Scope`] granted to a DCR client — the ceiling; never widened by the
    /// client's requested scope.
    pub default_scope: Scope,
    /// Absolute base URL used to build `registration_client_uri`
    /// (e.g. `https://mail.example.com`).
    pub issuer_base_url: String,
}

/// An RFC 7591 client-registration request (a subset of client metadata — the parts
/// this AS honours). Unknown members are ignored (`serde` default).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct ClientRegistrationRequest {
    pub redirect_uris: Vec<String>,
    pub client_name: Option<String>,
    pub grant_types: Vec<String>,
    pub response_types: Vec<String>,
    pub token_endpoint_auth_method: Option<String>,
    /// Advisory only — the granted scope is always the policy default (no escalation).
    pub scope: Option<String>,
    pub software_id: Option<String>,
    pub software_version: Option<String>,
    pub contacts: Vec<String>,
}

/// An RFC 7591 client-information response (also the RFC 7592 read/update body). The
/// `#[serde(skip)]` tail carries persistence artifacts the `mw-server` handler needs
/// but that are never serialised back to the client.
#[derive(Debug, Clone, Serialize)]
pub struct ClientRegistrationResponse {
    pub client_id: String,
    pub client_id_issued_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_name: Option<String>,
    pub redirect_uris: Vec<String>,
    pub grant_types: Vec<String>,
    pub response_types: Vec<String>,
    pub token_endpoint_auth_method: String,
    pub scope: String,
    /// The registration-access-token, in plaintext, returned **once** at
    /// registration time (RFC 7592). `None` on read/update responses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_access_token: Option<String>,
    pub registration_client_uri: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub contacts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_version: Option<String>,

    // ── persistence artifacts (never serialised to the client) ──────────────────
    /// Hash of the registration-access-token to persist in `oauth_client_meta`.
    #[serde(skip)]
    pub registration_access_token_hash: String,
    /// The scope actually granted (`= policy.default_scope`) — proves no escalation.
    #[serde(skip)]
    pub granted_scope: Scope,
    /// RFC-3339 creation timestamp for the 0007 client row + meta row.
    #[serde(skip)]
    pub created_at: String,
    /// The `approved_by` sentinel for the 0007 client row.
    #[serde(skip)]
    pub approved_by: String,
}

/// Failure modes of a DCR request. The `mw-server` handler maps these onto RFC 7591
/// HTTP responses (`403` disabled, `401` bad initial-access-token, `400`
/// `invalid_redirect_uri` / `invalid_client_metadata`, `500` store).
#[derive(Debug, thiserror::Error)]
pub enum DcrError {
    #[error("dynamic client registration is disabled")]
    Disabled,
    #[error("a valid initial access token is required")]
    InitialAccessTokenRequired,
    #[error("invalid redirect uri: {0}")]
    InvalidRedirectUri(String),
    #[error("invalid client metadata: {0}")]
    InvalidClientMetadata(String),
    #[error("store error: {0}")]
    Store(String),
}

/// The all-false [`Scope`] — the safe fallback when a policy row carries no usable
/// `default_scope`. Grants nothing (a DCR client so scoped can never escalate).
pub fn no_scope() -> Scope {
    Scope {
        read: false,
        send: false,
        delete: false,
        accounts: ScopeSelector::Subset(Vec::new()),
        folders: ScopeSelector::Subset(Vec::new()),
        mail: false,
        pim: false,
        ip_allowlist: Vec::new(),
        expires_at: None,
        rate_limit: None,
        mcp_tools: Vec::new(),
        unattended_send: false,
    }
}

/// Hex SHA-256 of a registration-access-token — the at-rest transform (the token is
/// a 256-bit random, so a single SHA-256 lookup key is correct, not a slow KDF).
pub fn registration_access_token_hash(token: &str) -> String {
    sha256_hex(token)
}

/// Constant-time check of a presented registration-access-token against the stored
/// hash (RFC 7592 client-configuration auth).
pub fn verify_registration_access_token(presented: &str, stored_hash: &str) -> bool {
    ct_eq(
        registration_access_token_hash(presented).as_bytes(),
        stored_hash.as_bytes(),
    )
}

/// Normalised, policy-validated client metadata (the values this AS will honour).
#[derive(Debug, Clone)]
pub struct ValidatedMetadata {
    pub redirect_uris: Vec<String>,
    pub grant_types: Vec<String>,
    pub response_types: Vec<String>,
    pub token_endpoint_auth_method: String,
    /// Always the policy default — the requested scope never widens it.
    pub granted_scope: Scope,
    /// A human-readable rendering of `granted_scope` for the RFC-7591 `scope` field.
    pub scope_display: String,
}

/// Validate + normalise a registration request against `policy` (shared by register
/// and update). Enforces the redirect-host allowlist and the supported grant/response
/// /auth-method sets; grants exactly `policy.default_scope`.
pub fn validate_metadata(
    request: &ClientRegistrationRequest,
    policy: &DcrPolicy,
) -> Result<ValidatedMetadata, DcrError> {
    // redirect_uris: required (this AS only issues the authorization_code grant).
    if request.redirect_uris.is_empty() {
        return Err(DcrError::InvalidRedirectUri(
            "at least one redirect_uri is required".into(),
        ));
    }
    for uri in &request.redirect_uris {
        let host = redirect_host(uri).ok_or_else(|| {
            DcrError::InvalidRedirectUri(format!("unparseable redirect_uri: {uri}"))
        })?;
        if !host_allowed(&host, &policy.allowed_redirect_host_suffixes) {
            return Err(DcrError::InvalidRedirectUri(format!(
                "redirect_uri host `{host}` is not within the allowed host suffixes"
            )));
        }
    }

    // grant_types: default authorization_code; only code + refresh are supported.
    let grant_types = if request.grant_types.is_empty() {
        vec!["authorization_code".to_string()]
    } else {
        request.grant_types.clone()
    };
    for g in &grant_types {
        if g != "authorization_code" && g != "refresh_token" {
            return Err(DcrError::InvalidClientMetadata(format!(
                "unsupported grant_type: {g}"
            )));
        }
    }

    // response_types: default code; only code is supported (no implicit flow).
    let response_types = if request.response_types.is_empty() {
        vec!["code".to_string()]
    } else {
        request.response_types.clone()
    };
    for r in &response_types {
        if r != "code" {
            return Err(DcrError::InvalidClientMetadata(format!(
                "unsupported response_type: {r}"
            )));
        }
    }

    // token_endpoint_auth_method: this AS is PKCE public-client only — `none`. We do
    // not issue client secrets, so we refuse to advertise secret-based methods.
    let token_endpoint_auth_method = request
        .token_endpoint_auth_method
        .clone()
        .unwrap_or_else(|| "none".to_string());
    if token_endpoint_auth_method != "none" {
        return Err(DcrError::InvalidClientMetadata(format!(
            "unsupported token_endpoint_auth_method `{token_endpoint_auth_method}` \
             (this server issues public PKCE clients only)"
        )));
    }

    // The granted scope is ALWAYS the policy default — never the requested scope.
    let granted_scope = policy.default_scope.clone();
    let scope_display = scope_display(&granted_scope);

    Ok(ValidatedMetadata {
        redirect_uris: request.redirect_uris.clone(),
        grant_types,
        response_types,
        token_endpoint_auth_method,
        granted_scope,
        scope_display,
    })
}

/// A freshly minted client identity + registration-access-token.
struct Minted {
    client_id: String,
    registration_access_token: String,
    registration_access_token_hash: String,
    registration_client_uri: String,
    client_id_issued_at: i64,
    created_at: String,
}

fn mint(policy: &DcrPolicy) -> Minted {
    let client_id = format!("dcr_{}", b64url(&random_bytes::<16>()));
    let token = b64url(&random_bytes::<32>());
    let now = chrono::Utc::now();
    Minted {
        registration_access_token_hash: registration_access_token_hash(&token),
        registration_client_uri: registration_client_uri(&policy.issuer_base_url, &client_id),
        client_id_issued_at: now.timestamp(),
        created_at: now.to_rfc3339(),
        registration_access_token: token,
        client_id,
    }
}

/// Build the `registration_client_uri` for a client (`{base}/oauth/register/{id}`).
pub fn registration_client_uri(issuer_base_url: &str, client_id: &str) -> String {
    format!(
        "{}/oauth/register/{}",
        issuer_base_url.trim_end_matches('/'),
        client_id
    )
}

/// Register a new client (RFC 7591). Validates against `policy`, mints a `client_id`,
/// a registration-access-token, and a `registration_client_uri`, then persists the
/// client into the 0007 `oauth_clients` table via the frozen [`OAuthStore`]. The returned
/// response carries (in its `#[serde(skip)]` tail) the token hash / created-at /
/// granted-scope the caller writes to the 0010 `oauth_client_meta` side table.
///
/// Refuses with [`DcrError::Disabled`] when the policy is disabled (defence in depth;
/// the handler also short-circuits before calling).
pub async fn register<S>(
    store: &S,
    request: ClientRegistrationRequest,
    policy: DcrPolicy,
) -> Result<ClientRegistrationResponse, DcrError>
where
    S: OAuthStore + ?Sized,
{
    if !policy.enabled {
        return Err(DcrError::Disabled);
    }
    let meta = validate_metadata(&request, &policy)?;
    let minted = mint(&policy);

    let client = OAuthClient {
        client_id: minted.client_id.clone(),
        name: request.client_name.clone().unwrap_or_default(),
        redirect_uris: meta.redirect_uris.clone(),
        approved_by: DCR_APPROVED_BY.to_string(),
        created_at: minted.created_at.clone(),
    };
    store
        .put_client(client)
        .await
        .map_err(|e: OAuthError| DcrError::Store(e.to_string()))?;

    Ok(build_response(
        &minted.client_id,
        &request,
        &meta,
        minted.registration_client_uri,
        minted.client_id_issued_at,
        Some(minted.registration_access_token),
        minted.registration_access_token_hash,
        minted.created_at,
    ))
}

/// Assemble a [`ClientRegistrationResponse`] from validated metadata. Used by
/// [`register`] (with a fresh token) and by the RFC-7592 read/update handlers (token
/// `None`, hash empty — they never re-mint).
#[allow(clippy::too_many_arguments)]
pub fn build_response(
    client_id: &str,
    request: &ClientRegistrationRequest,
    meta: &ValidatedMetadata,
    registration_client_uri: String,
    client_id_issued_at: i64,
    registration_access_token: Option<String>,
    registration_access_token_hash: String,
    created_at: String,
) -> ClientRegistrationResponse {
    ClientRegistrationResponse {
        client_id: client_id.to_string(),
        client_id_issued_at,
        client_name: request.client_name.clone().filter(|s| !s.is_empty()),
        redirect_uris: meta.redirect_uris.clone(),
        grant_types: meta.grant_types.clone(),
        response_types: meta.response_types.clone(),
        token_endpoint_auth_method: meta.token_endpoint_auth_method.clone(),
        scope: meta.scope_display.clone(),
        registration_access_token,
        registration_client_uri,
        contacts: request.contacts.clone(),
        software_id: request.software_id.clone(),
        software_version: request.software_version.clone(),
        registration_access_token_hash,
        granted_scope: meta.granted_scope.clone(),
        created_at,
        approved_by: DCR_APPROVED_BY.to_string(),
    }
}

// ── helpers ─────────────────────────────────────────────────────────────────────

/// Render a [`Scope`] as an RFC-7591 space-delimited scope string (best-effort, for
/// display only — the authoritative grant is the typed [`Scope`]).
fn scope_display(s: &Scope) -> String {
    let mut parts = Vec::new();
    if s.read {
        parts.push("read");
    }
    if s.send {
        parts.push("send");
    }
    if s.delete {
        parts.push("delete");
    }
    if s.mail {
        parts.push("mail");
    }
    if s.pim {
        parts.push("pim");
    }
    parts.join(" ")
}

/// Extract the lower-cased host from a `redirect_uri`. Returns `None` if there is no
/// `scheme://` authority or the host is empty. Handles userinfo, ports, and IPv6
/// literals.
fn redirect_host(uri: &str) -> Option<String> {
    let after_scheme = uri.split_once("://").map(|(_, rest)| rest)?;
    let authority = after_scheme.split(['/', '?', '#']).next().unwrap_or("");
    // strip userinfo
    let hostport = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    let host = if let Some(rest) = hostport.strip_prefix('[') {
        // IPv6 literal: [::1]:port
        rest.split(']').next().unwrap_or("")
    } else {
        hostport.split(':').next().unwrap_or("")
    };
    if host.is_empty() {
        None
    } else {
        Some(host.to_ascii_lowercase())
    }
}

/// Whether `host` matches any allowed suffix (`example.com` matches `example.com` and
/// `*.example.com`, but not `notexample.com`). An empty allowlist matches nothing.
fn host_allowed(host: &str, suffixes: &[String]) -> bool {
    suffixes.iter().any(|s| {
        let s = s.trim().trim_start_matches('.').to_ascii_lowercase();
        !s.is_empty() && (host == s || host.ends_with(&format!(".{s}")))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::InMemoryOAuthStore;

    fn policy(enabled: bool, suffixes: &[&str], scope: Scope) -> DcrPolicy {
        DcrPolicy {
            enabled,
            require_initial_access_token: false,
            allowed_redirect_host_suffixes: suffixes.iter().map(|s| s.to_string()).collect(),
            default_scope: scope,
            issuer_base_url: "https://mail.example.com".into(),
        }
    }

    fn req(redirect: &str) -> ClientRegistrationRequest {
        ClientRegistrationRequest {
            redirect_uris: vec![redirect.to_string()],
            client_name: Some("Test App".into()),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn disabled_policy_refuses() {
        let store = InMemoryOAuthStore::new();
        let err = register(
            &store,
            req("https://app.example.com/cb"),
            policy(false, &["example.com"], Scope::read_only("acct")),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, DcrError::Disabled));
    }

    #[tokio::test]
    async fn enabled_mints_and_persists_a_client() {
        let store = InMemoryOAuthStore::new();
        let resp = register(
            &store,
            req("https://app.example.com/cb"),
            policy(true, &["example.com"], Scope::read_only("acct")),
        )
        .await
        .unwrap();

        assert!(resp.client_id.starts_with("dcr_"));
        assert!(resp.registration_access_token.is_some());
        assert_eq!(resp.token_endpoint_auth_method, "none");
        assert_eq!(resp.grant_types, vec!["authorization_code"]);
        assert_eq!(resp.response_types, vec!["code"]);
        assert!(
            resp.registration_client_uri
                .ends_with(&format!("/oauth/register/{}", resp.client_id))
        );

        // The client is a perfectly ordinary 0007 OAuthClient now.
        let client = store.get_client(&resp.client_id).await.unwrap().unwrap();
        assert_eq!(client.redirect_uris, vec!["https://app.example.com/cb"]);
        assert_eq!(client.approved_by, DCR_APPROVED_BY);
    }

    #[tokio::test]
    async fn redirect_host_outside_allowlist_is_rejected() {
        let store = InMemoryOAuthStore::new();
        let err = register(
            &store,
            req("https://evil.attacker.test/cb"),
            policy(true, &["example.com"], Scope::read_only("acct")),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, DcrError::InvalidRedirectUri(_)));

        // A near-miss (suffix is not a dotted boundary) is also rejected.
        let err2 = validate_metadata(
            &req("https://notexample.com/cb"),
            &policy(true, &["example.com"], Scope::read_only("acct")),
        )
        .unwrap_err();
        assert!(matches!(err2, DcrError::InvalidRedirectUri(_)));
    }

    #[tokio::test]
    async fn empty_allowlist_denies_everything() {
        let err = validate_metadata(
            &req("https://app.example.com/cb"),
            &policy(true, &[], Scope::read_only("acct")),
        )
        .unwrap_err();
        assert!(matches!(err, DcrError::InvalidRedirectUri(_)));
    }

    #[test]
    fn subdomain_and_apex_match_but_sibling_does_not() {
        let suffixes = vec!["example.com".to_string()];
        assert!(host_allowed("example.com", &suffixes));
        assert!(host_allowed("app.example.com", &suffixes));
        assert!(host_allowed("a.b.example.com", &suffixes));
        assert!(!host_allowed("notexample.com", &suffixes));
        assert!(!host_allowed("example.com.evil.test", &suffixes));
        assert!(!host_allowed("example.com", &[]));
    }

    #[test]
    fn redirect_host_parsing() {
        assert_eq!(
            redirect_host("https://user:pw@app.example.com:8443/cb?x=1"),
            Some("app.example.com".into())
        );
        assert_eq!(redirect_host("http://[::1]:9000/cb"), Some("::1".into()));
        assert_eq!(redirect_host("not-a-uri"), None);
    }

    #[test]
    fn registration_access_token_round_trips_by_hash() {
        let token = "abc.def.reg-access-token";
        let hash = registration_access_token_hash(token);
        assert!(verify_registration_access_token(token, &hash));
        assert!(!verify_registration_access_token("wrong-token", &hash));
        // The hash is not the token.
        assert_ne!(hash, token);
    }

    #[tokio::test]
    async fn scope_is_never_escalated_beyond_policy_default() {
        let store = InMemoryOAuthStore::new();
        // The client asks for a broad scope string...
        let mut request = req("https://app.example.com/cb");
        request.scope = Some("read send delete mail pim".into());
        // ...but the policy default is strictly read-only mail.
        let default = Scope::read_only("acct");
        let resp = register(
            &store,
            request,
            policy(true, &["example.com"], default.clone()),
        )
        .await
        .unwrap();

        // The granted scope equals the policy default — not the requested widening.
        assert_eq!(resp.granted_scope, default);
        assert!(resp.granted_scope.read);
        assert!(!resp.granted_scope.send);
        assert!(!resp.granted_scope.delete);
        assert!(!resp.granted_scope.pim);
        assert_eq!(resp.scope, "read mail");
    }

    #[test]
    fn unsupported_grant_and_auth_method_are_rejected() {
        let mut r = req("https://app.example.com/cb");
        r.grant_types = vec!["client_credentials".into()];
        assert!(matches!(
            validate_metadata(&r, &policy(true, &["example.com"], no_scope())),
            Err(DcrError::InvalidClientMetadata(_))
        ));

        let mut r2 = req("https://app.example.com/cb");
        r2.token_endpoint_auth_method = Some("client_secret_basic".into());
        assert!(matches!(
            validate_metadata(&r2, &policy(true, &["example.com"], no_scope())),
            Err(DcrError::InvalidClientMetadata(_))
        ));
    }
}
