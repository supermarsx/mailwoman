#![forbid(unsafe_code)]
//! `mw-sso` — OIDC + SAML 2.0 SSO **login backends** (t9 plan §2/§3, SPEC §18.3).
//!
//! Pure protocol + crypto, **no axum**: this crate mirrors how `mw-directory` /
//! `mw-oauth` are protocol crates with a thin `mw-server` route module
//! (`sso.rs`, owned by e3). The server resolves an [`SsoIdentity`] to a Mailwoman
//! account and mints the SAME opaque `mw_session` cookie / native bearer as the
//! password path — this crate never issues sessions.
//!
//! ## The frozen contract (authored by e0; e1/e2 implement, e3/e4 consume)
//! * [`SsoKind`] — `Oidc | Saml`.
//! * [`SsoLogin`] — the provider trait: [`begin`](SsoLogin::begin) →
//!   [`complete`](SsoLogin::complete) → [`SsoIdentity`], plus
//!   [`metadata`](SsoLogin::metadata) (SAML SP metadata) and
//!   [`logout`](SsoLogin::logout) (RP-initiated logout / SLO).
//! * [`SsoConfigRow`] — the serde DTO the server maps to/from the 0009 `sso_config`
//!   store row (secrets live in the sealed `secret_sealed` column, never in this
//!   JSON). Kind-specific config is [`OidcConfig`] / [`SamlConfig`].
//! * [`ClaimMap`] — IdP claims/attributes → Mailwoman account fields.
//! * [`SsoBackend`] — a fully-resolved, ready-to-serve backend (config + unsealed
//!   secret) that the server hands to [`oidc::OidcProvider::new`] /
//!   [`saml::SamlProvider::new`].
//! * [`SsoError`] — **every variant maps to a uniform 401 at the route** so a
//!   caller never learns which check failed (the detail string is for the server
//!   log + the content-free audit only, never the wire).
//! * [`state::PendingState`] / [`state::PendingStore`] — the server-side PKCE+nonce
//!   (OIDC) / RequestID+RelayState (SAML) guard, keyed by an opaque short-TTL token.
//!
//! `#![forbid(unsafe_code)]`. VET: the `openidconnect`(rustls) + `flate2`(miniz_oxide)
//! trees add no openssl / no `-sys` C — see the crate `Cargo.toml` note.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub mod oidc;
pub mod saml;
pub mod state;

pub use oidc::OidcProvider;
pub use saml::SamlProvider;
pub use state::{PendingState, PendingStore};

/// Which SSO protocol a backend speaks (plan §3). Serialized lowercase to match the
/// 0009 `sso_config.kind` column (`'oidc' | 'saml'`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SsoKind {
    Oidc,
    Saml,
}

impl SsoKind {
    /// The `sso_config.kind` textual value.
    pub fn as_db(self) -> &'static str {
        match self {
            SsoKind::Oidc => "oidc",
            SsoKind::Saml => "saml",
        }
    }

    /// Parse the `sso_config.kind` column; unknown values default to `Oidc` with a
    /// warning (callers should validate on write).
    pub fn parse(s: &str) -> Option<SsoKind> {
        match s {
            "oidc" => Some(SsoKind::Oidc),
            "saml" => Some(SsoKind::Saml),
            _ => None,
        }
    }
}

/// A backend's configuration scope (plan §1/§4): deployment-wide or a single mail
/// domain. Textual form matches the 0009 `sso_config.scope` column:
/// `'deployment'` or `'domain:<d>'`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SsoScope {
    /// Applies to every domain the deployment serves.
    Deployment,
    /// Applies only to logins for the named mail domain.
    Domain(String),
}

impl SsoScope {
    /// The `sso_config.scope` textual value (`'deployment'` | `'domain:<d>'`).
    pub fn as_db(&self) -> String {
        match self {
            SsoScope::Deployment => "deployment".to_string(),
            SsoScope::Domain(d) => format!("domain:{d}"),
        }
    }

    /// Parse the `sso_config.scope` column.
    pub fn parse(s: &str) -> SsoScope {
        match s.strip_prefix("domain:") {
            Some(d) => SsoScope::Domain(d.to_string()),
            None => SsoScope::Deployment,
        }
    }
}

/// First-login behaviour when an IdP identity has no existing Mailwoman account
/// (plan §9 R9, task §1). **Default = [`Allowlist`](FirstLoginPolicy::Allowlist)**:
/// deny unknown subjects — NO open auto-registration. An admin must explicitly opt
/// into [`AutoCreate`](FirstLoginPolicy::AutoCreate).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FirstLoginPolicy {
    /// Deny logins whose subject maps to no existing account (the safe default).
    #[default]
    Allowlist,
    /// Auto-create a Mailwoman account on first successful SSO login (opt-in).
    AutoCreate,
}

/// Maps IdP claims/attributes onto Mailwoman account fields (plan §3). Each field
/// names the claim (OIDC) or attribute (SAML) to read; `None` falls back to the
/// protocol default (`email`/`sub` for OIDC, the configured NameID for SAML).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ClaimMap {
    /// Claim/attribute carrying the account email.
    pub email: Option<String>,
    /// Claim/attribute carrying the login username / local-part.
    pub username: Option<String>,
    /// Claim/attribute carrying the display name.
    pub display: Option<String>,
    /// Claim/attribute carrying group memberships (multi-valued).
    pub groups: Option<String>,
}

/// OIDC-specific configuration (plan §3). The `client_secret` is **not** here — it
/// lives sealed in the 0009 `secret_sealed` column and is injected into
/// [`SsoBackend::secret`] after unsealing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct OidcConfig {
    /// The issuer base URL (`.well-known/openid-configuration` is derived from it).
    pub issuer_url: String,
    /// The registered OIDC client / relying-party id.
    pub client_id: String,
    /// The redirect/callback URL registered with the IdP.
    pub redirect_url: String,
    /// Requested scopes (defaults to `openid email profile`).
    pub scopes: Vec<String>,
    /// First-login policy (default deny / allowlist).
    pub first_login_policy: FirstLoginPolicy,
}

impl Default for OidcConfig {
    fn default() -> Self {
        OidcConfig {
            issuer_url: String::new(),
            client_id: String::new(),
            redirect_url: String::new(),
            scopes: vec!["openid".into(), "email".into(), "profile".into()],
            first_login_policy: FirstLoginPolicy::default(),
        }
    }
}

/// SAML-specific configuration (plan §3/§5). The SP private key (if any) is **not**
/// here — it lives sealed in the 0009 `secret_sealed` column and is injected into
/// [`SsoBackend::secret`] after unsealing.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SamlConfig {
    /// This SP's entity id (the audience the IdP asserts to).
    pub sp_entity_id: String,
    /// The ACS (assertion consumer service) URL the IdP POSTs the response to.
    pub acs_url: String,
    /// The IdP metadata URL (fetched to learn SSO/SLO endpoints + signing certs).
    pub idp_metadata_url: Option<String>,
    /// Inline IdP metadata XML (alternative to `idp_metadata_url`).
    pub idp_metadata_xml: Option<String>,
    /// The IdP SSO (AuthnRequest) endpoint (HTTP-Redirect binding).
    pub idp_sso_url: String,
    /// The IdP SLO (single logout) endpoint, if any.
    pub idp_slo_url: Option<String>,
    /// PEM-encoded IdP signing certificates (pinned; the assertion signature must
    /// verify against one of these).
    pub idp_signing_certs_pem: Vec<String>,
    /// Require the assertion to be signed (default `true` — a security-positive).
    pub want_assertions_signed: bool,
    /// Accept `EncryptedAssertion` (default `false`).
    pub want_encrypted: bool,
    /// The requested NameID format (e.g. `...:persistent` / `...:emailAddress`).
    pub nameid_format: String,
    /// First-login policy (default deny / allowlist).
    pub first_login_policy: FirstLoginPolicy,
}

/// Kind-specific configuration, tagged by protocol (plan §3). Serializes into the
/// 0009 `sso_config.config` JSON column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum SsoConfig {
    Oidc(OidcConfig),
    Saml(SamlConfig),
}

impl SsoConfig {
    /// The protocol of this config.
    pub fn kind(&self) -> SsoKind {
        match self {
            SsoConfig::Oidc(_) => SsoKind::Oidc,
            SsoConfig::Saml(_) => SsoKind::Saml,
        }
    }

    /// The configured first-login policy (deny/allowlist by default).
    pub fn first_login_policy(&self) -> FirstLoginPolicy {
        match self {
            SsoConfig::Oidc(c) => c.first_login_policy,
            SsoConfig::Saml(c) => c.first_login_policy,
        }
    }
}

/// The serde DTO the server maps to/from the 0009 `sso_config` store row (plan §3).
/// Secrets are **excluded** — they live sealed in the store's `secret_sealed`
/// column and are attached to [`SsoBackend::secret`] after unsealing. `scope` is
/// the raw `'deployment' | 'domain:<d>'` textual form (see [`SsoScope`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SsoConfigRow {
    /// Stable id, e.g. `"corp-oidc"`.
    pub id: String,
    /// Admin-facing label, e.g. `"Sign in with Acme SSO"`.
    pub display_name: String,
    /// `'deployment'` or `'domain:<d>'`.
    pub scope: String,
    /// Whether the backend is live (advertised on the login screen).
    pub enabled: bool,
    /// Kind-specific config (secrets excluded).
    pub config: SsoConfig,
    /// IdP claim/attribute → account-field mapping.
    #[serde(default)]
    pub claim_map: ClaimMap,
}

/// A fully-resolved, ready-to-serve backend (plan §3): the persisted config plus
/// the **unsealed** secret. The server builds this from the 0009 store row (after
/// [`Store::get_sso_config`](../mw_store/struct.Store.html) unseals `secret_sealed`)
/// and hands it to [`OidcProvider::new`] / [`SamlProvider::new`].
#[derive(Debug, Clone)]
pub struct SsoBackend {
    /// Stable id.
    pub id: String,
    /// Protocol.
    pub kind: SsoKind,
    /// Admin-facing label.
    pub display_name: String,
    /// Configuration scope.
    pub scope: SsoScope,
    /// Whether the backend is enabled.
    pub enabled: bool,
    /// Kind-specific configuration.
    pub config: SsoConfig,
    /// Claim/attribute mapping.
    pub claim_map: ClaimMap,
    /// Unsealed secret: the OIDC `client_secret` or the SAML SP private key. `None`
    /// for public/confidential-less clients or metadata-only SAML SPs.
    pub secret: Option<Vec<u8>>,
}

/// The redirect the server issues to start an SSO login (plan §3). The server
/// persists [`pending`](BeginRedirect::pending) under
/// [`state_token`](BeginRedirect::state_token) (via [`PendingStore`]), sets the
/// token as a short-TTL cookie / hidden form field, and 302s to
/// [`url`](BeginRedirect::url).
#[derive(Debug, Clone)]
pub struct BeginRedirect {
    /// The IdP URL to redirect the browser to (OIDC authorize URL, or the SAML
    /// HTTP-Redirect AuthnRequest URL).
    pub url: String,
    /// The opaque state token the server persists + echoes back to bind the
    /// callback to this flow.
    pub state_token: String,
    /// The per-flow secret material (PKCE verifier + nonce, or RequestID +
    /// RelayState) the server stores under `state_token`.
    pub pending: PendingState,
}

/// The IdP callback the server hands to [`SsoLogin::complete`] (plan §3). The
/// server resolves the opaque state token → [`PendingState`] via [`PendingStore`]
/// BEFORE calling `complete`, and packs it here alongside the raw upstream params —
/// so the provider stays storage-free and the replay/CSRF binding is enforced
/// server-side.
#[derive(Debug, Clone)]
pub struct SsoCallback {
    /// Raw callback parameters: OIDC `code`/`state`, or SAML `SAMLResponse`/
    /// `RelayState` (base64), keyed by name.
    pub params: BTreeMap<String, String>,
    /// The [`PendingState`] the server resolved from the flow's opaque state token.
    pub pending: PendingState,
}

impl SsoCallback {
    /// Read a raw callback parameter.
    pub fn param(&self, name: &str) -> Option<&str> {
        self.params.get(name).map(String::as_str)
    }
}

/// SAML SP metadata to publish at `GET /api/sso/{id}/metadata` (plan §3). `None`
/// for OIDC (which advertises via the IdP's discovery doc, not SP metadata).
#[derive(Debug, Clone)]
pub struct Metadata {
    /// The response content type (e.g. `application/samlmetadata+xml`).
    pub content_type: String,
    /// The metadata document body.
    pub body: String,
}

/// A logout redirect (RP-initiated OIDC logout or SAML SLO), plan §3.
#[derive(Debug, Clone)]
pub struct Redirect {
    /// The URL to redirect the browser to for upstream logout.
    pub url: String,
}

/// The identity an SSO login resolves to (plan §3). The server maps this to a
/// Mailwoman account (per [`ClaimMap`] + [`FirstLoginPolicy`]) and mints its normal
/// session — this crate returns identity only, never a session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SsoIdentity {
    /// The stable IdP subject (OIDC `sub` / SAML `NameID`).
    pub subject: String,
    /// The asserted email, if any.
    pub email: Option<String>,
    /// The asserted display name, if any.
    pub display_name: Option<String>,
    /// Group memberships asserted by the IdP.
    pub groups: Vec<String>,
    /// The full raw claim/attribute set (single-valued view), for advanced mapping
    /// + the audit trail. NEVER contains tokens or assertions.
    pub claims: BTreeMap<String, String>,
}

/// Errors surfaced by an SSO login (plan §3, task §1). **Every variant maps to a
/// uniform HTTP 401 at the route** ([`login_status`](SsoError::login_status)) so a
/// caller can never distinguish *which* check failed — the detail string is for the
/// server log + the content-free `sso_login_audit` (`outcome = 'error:<reason>'`)
/// ONLY, never the wire.
#[derive(Debug, thiserror::Error)]
pub enum SsoError {
    /// OIDC discovery / IdP-metadata fetch or parse failed.
    #[error("discovery failed")]
    Discovery(String),
    /// ID-token / assertion validation failed (issuer, nonce, structure).
    #[error("token validation failed")]
    TokenValidation(String),
    /// The XML-DSig / JWS signature did not verify against the pinned key.
    #[error("signature invalid")]
    SignatureInvalid(String),
    /// The token/assertion audience did not match this SP/RP.
    #[error("audience mismatch")]
    AudienceMismatch,
    /// The token/assertion is outside its validity window.
    #[error("token expired")]
    Expired,
    /// The state token / `InResponseTo` was reused or is unknown (replay/CSRF).
    #[error("replay detected")]
    Replay,
    /// The backend is misconfigured (bad URL, missing cert, malformed config).
    #[error("configuration error")]
    Config(String),
    /// The upstream IdP returned an error or was unreachable.
    #[error("upstream error")]
    Upstream(String),
}

impl SsoError {
    /// The uniform status every SSO failure returns at the route: **401**. Callers
    /// MUST NOT branch on the variant to vary the response — that would leak which
    /// check failed.
    pub fn login_status(&self) -> u16 {
        401
    }

    /// The short reason token for the content-free `sso_login_audit.outcome`
    /// (`'error:<reason>'`). Carries the variant name only — no detail, no content.
    pub fn audit_reason(&self) -> &'static str {
        match self {
            SsoError::Discovery(_) => "discovery",
            SsoError::TokenValidation(_) => "token_validation",
            SsoError::SignatureInvalid(_) => "signature_invalid",
            SsoError::AudienceMismatch => "audience_mismatch",
            SsoError::Expired => "expired",
            SsoError::Replay => "replay",
            SsoError::Config(_) => "config",
            SsoError::Upstream(_) => "upstream",
        }
    }
}

/// The frozen SSO provider trait (plan §3, task §1). e1 implements it for OIDC
/// ([`OidcProvider`]), e2 for SAML ([`SamlProvider`]); e3 drives it from the route
/// module. Object-safe so the server can hold a `Box<dyn SsoLogin>` per backend.
#[async_trait::async_trait]
pub trait SsoLogin: Send + Sync {
    /// Begin a login: build the IdP redirect + the per-flow [`PendingState`] the
    /// server persists. `relay_state` is an opaque post-login return target the
    /// server round-trips (deep-link back to the requested screen).
    async fn begin(&self, relay_state: Option<String>) -> Result<BeginRedirect, SsoError>;

    /// Complete a login from the IdP [`SsoCallback`] (which already carries the
    /// server-resolved [`PendingState`]): validate signatures/nonce/audience/expiry
    /// + replay, then return the [`SsoIdentity`]. Any failure is a uniform 401.
    async fn complete(&self, callback: SsoCallback) -> Result<SsoIdentity, SsoError>;

    /// SAML SP metadata to publish, or `None` for OIDC.
    fn metadata(&self) -> Option<Metadata>;

    /// An upstream logout redirect (OIDC RP-initiated logout / SAML SLO) for the
    /// given subject, or `None` if the backend does not support it.
    fn logout(&self, subject: &str) -> Option<Redirect>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_db_round_trips() {
        assert_eq!(SsoKind::Oidc.as_db(), "oidc");
        assert_eq!(SsoKind::Saml.as_db(), "saml");
        assert_eq!(SsoKind::parse("saml"), Some(SsoKind::Saml));
        assert_eq!(SsoKind::parse("nope"), None);
    }

    #[test]
    fn scope_db_round_trips() {
        assert_eq!(SsoScope::Deployment.as_db(), "deployment");
        assert_eq!(
            SsoScope::Domain("acme.test".into()).as_db(),
            "domain:acme.test"
        );
        assert_eq!(SsoScope::parse("deployment"), SsoScope::Deployment);
        assert_eq!(
            SsoScope::parse("domain:acme.test"),
            SsoScope::Domain("acme.test".into())
        );
    }

    #[test]
    fn first_login_policy_defaults_to_allowlist_deny() {
        // The security-critical invariant: NO open auto-registration by default.
        assert_eq!(FirstLoginPolicy::default(), FirstLoginPolicy::Allowlist);
        assert_eq!(
            OidcConfig::default().first_login_policy,
            FirstLoginPolicy::Allowlist
        );
        assert_eq!(
            SamlConfig::default().first_login_policy,
            FirstLoginPolicy::Allowlist
        );
    }

    #[test]
    fn oidc_config_default_scopes() {
        assert_eq!(
            OidcConfig::default().scopes,
            vec!["openid".to_string(), "email".into(), "profile".into()]
        );
    }

    #[test]
    fn sso_config_row_json_round_trip_oidc() {
        let row = SsoConfigRow {
            id: "corp-oidc".into(),
            display_name: "Acme SSO".into(),
            scope: "deployment".into(),
            enabled: true,
            config: SsoConfig::Oidc(OidcConfig {
                issuer_url: "https://idp.example".into(),
                client_id: "mailwoman".into(),
                redirect_url: "https://mail.example/api/sso/corp-oidc/callback".into(),
                scopes: vec!["openid".into(), "email".into()],
                first_login_policy: FirstLoginPolicy::AutoCreate,
            }),
            claim_map: ClaimMap {
                email: Some("email".into()),
                ..Default::default()
            },
        };
        let json = serde_json::to_string(&row).unwrap();
        let back: SsoConfigRow = serde_json::from_str(&json).unwrap();
        assert_eq!(row, back);
        assert_eq!(back.config.kind(), SsoKind::Oidc);
        assert_eq!(
            back.config.first_login_policy(),
            FirstLoginPolicy::AutoCreate
        );
    }

    #[test]
    fn sso_config_row_json_round_trip_saml() {
        let row = SsoConfigRow {
            id: "corp-saml".into(),
            display_name: "Acme SAML".into(),
            scope: "domain:acme.test".into(),
            enabled: false,
            config: SsoConfig::Saml(SamlConfig {
                sp_entity_id: "https://mail.example/sp".into(),
                acs_url: "https://mail.example/api/sso/corp-saml/acs".into(),
                idp_sso_url: "https://idp.example/sso".into(),
                want_assertions_signed: true,
                nameid_format: "urn:oasis:names:tc:SAML:2.0:nameid-format:persistent".into(),
                ..Default::default()
            }),
            claim_map: ClaimMap::default(),
        };
        let json = serde_json::to_string(&row).unwrap();
        let back: SsoConfigRow = serde_json::from_str(&json).unwrap();
        assert_eq!(row, back);
        assert_eq!(back.config.kind(), SsoKind::Saml);
    }

    #[test]
    fn every_error_is_uniform_401_with_a_reason() {
        for e in [
            SsoError::Discovery("x".into()),
            SsoError::TokenValidation("x".into()),
            SsoError::SignatureInvalid("x".into()),
            SsoError::AudienceMismatch,
            SsoError::Expired,
            SsoError::Replay,
            SsoError::Config("x".into()),
            SsoError::Upstream("x".into()),
        ] {
            assert_eq!(e.login_status(), 401, "SSO failures must be a uniform 401");
            assert!(!e.audit_reason().is_empty());
        }
    }
}
