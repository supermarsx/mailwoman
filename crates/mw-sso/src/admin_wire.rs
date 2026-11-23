//! The `/admin/sso` wire contract (t46-e1): the JSON the admin console (and any
//! other API client) sends to `POST /admin/sso` and reads from `GET /admin/sso`.
//!
//! The storage types in the crate root ([`OidcConfig`], [`SamlConfig`], ...) keep
//! their snake_case serde form: that is what the `sso_config.config` column holds
//! and what sign-in parses. This module is the one place the admin API's names are
//! defined:
//!
//! * field names are camelCase (`issuerUrl`, `idpSsoUrl`, `firstLoginPolicy`, ...),
//!   the casing the console sends and the house admin convention;
//! * the snake_case names (`issuer_url`, ...) are accepted as aliases, so a client
//!   that posts a stored config back as-is keeps working;
//! * the first-login policy is `allowlist` | `autocreate` (`auto_create` accepted);
//! * an unknown field is an error, never silently dropped: a misspelt field in
//!   authentication configuration must not save as an empty value;
//! * a request is validated before it is stored, and every problem is reported with
//!   the wire name of the field ([`FieldError::field`], e.g. `config.issuerUrl`).
//!
//! Secrets never appear on the read side: [`backend_view`] has no secret field, only
//! `hasSecret`. Sign-in validation (issuer/audience/signature/nonce) is untouched.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{ClaimMap, FirstLoginPolicy, OidcConfig, SamlConfig, SsoConfig, SsoKind};

/// The default requested NameID format (matches the console's default).
pub const DEFAULT_NAMEID_FORMAT: &str = "urn:oasis:names:tc:SAML:2.0:nameid-format:persistent";

/// First-login policy on the wire: `allowlist` | `autocreate`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WirePolicy {
    #[default]
    #[serde(rename = "allowlist")]
    Allowlist,
    #[serde(rename = "autocreate", alias = "auto_create")]
    AutoCreate,
}

impl From<WirePolicy> for FirstLoginPolicy {
    fn from(p: WirePolicy) -> Self {
        match p {
            WirePolicy::Allowlist => FirstLoginPolicy::Allowlist,
            WirePolicy::AutoCreate => FirstLoginPolicy::AutoCreate,
        }
    }
}

impl From<FirstLoginPolicy> for WirePolicy {
    fn from(p: FirstLoginPolicy) -> Self {
        match p {
            FirstLoginPolicy::Allowlist => WirePolicy::Allowlist,
            FirstLoginPolicy::AutoCreate => WirePolicy::AutoCreate,
        }
    }
}

/// OIDC config on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct WireOidc {
    #[serde(alias = "issuer_url")]
    pub issuer_url: String,
    #[serde(alias = "client_id")]
    pub client_id: String,
    #[serde(alias = "redirect_url")]
    pub redirect_url: String,
    pub scopes: Vec<String>,
    #[serde(alias = "first_login_policy")]
    pub first_login_policy: WirePolicy,
    /// Refuse sign-ins whose email is not `email_verified: true` (omitted = true).
    #[serde(alias = "require_verified_email")]
    pub require_verified_email: bool,
}

impl Default for WireOidc {
    fn default() -> Self {
        WireOidc::from(&OidcConfig::default())
    }
}

/// SAML config on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct WireSaml {
    #[serde(alias = "sp_entity_id")]
    pub sp_entity_id: String,
    #[serde(alias = "acs_url")]
    pub acs_url: String,
    #[serde(alias = "idp_metadata_url")]
    pub idp_metadata_url: Option<String>,
    #[serde(alias = "idp_metadata_xml")]
    pub idp_metadata_xml: Option<String>,
    #[serde(alias = "idp_sso_url")]
    pub idp_sso_url: String,
    #[serde(alias = "idp_slo_url")]
    pub idp_slo_url: Option<String>,
    #[serde(alias = "idp_signing_certs_pem")]
    pub idp_signing_certs_pem: Vec<String>,
    #[serde(alias = "want_assertions_signed")]
    pub want_assertions_signed: bool,
    #[serde(alias = "want_encrypted")]
    pub want_encrypted: bool,
    #[serde(alias = "nameid_format")]
    pub nameid_format: String,
    #[serde(alias = "first_login_policy")]
    pub first_login_policy: WirePolicy,
}

impl Default for WireSaml {
    fn default() -> Self {
        WireSaml {
            sp_entity_id: String::new(),
            acs_url: String::new(),
            idp_metadata_url: None,
            idp_metadata_xml: None,
            idp_sso_url: String::new(),
            idp_slo_url: None,
            idp_signing_certs_pem: Vec::new(),
            // Omitted means the documented, security-positive default: signed.
            want_assertions_signed: true,
            want_encrypted: false,
            nameid_format: DEFAULT_NAMEID_FORMAT.to_string(),
            first_login_policy: WirePolicy::Allowlist,
        }
    }
}

/// Kind-tagged config on the wire (`"kind": "oidc" | "saml"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum WireConfig {
    Oidc(WireOidc),
    Saml(WireSaml),
}

/// Claim map on the wire (the names are single words, so casing is not an issue;
/// unknown keys are still refused).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct WireClaimMap {
    pub email: Option<String>,
    pub username: Option<String>,
    pub display: Option<String>,
    pub groups: Option<String>,
}

/// The `POST /admin/sso` body.
///
/// `kind`, `hasSecret`, `createdAt`, `updatedAt` and `problems` are read-only fields
/// of a `GET` row; they are accepted (so a listed row can be posted back unchanged)
/// and ignored, except that a top-level `kind` must agree with `config.kind`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpsertRequest {
    pub id: String,
    #[serde(alias = "display_name")]
    pub display_name: String,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub enabled: bool,
    pub config: WireConfig,
    #[serde(default, alias = "claim_map")]
    pub claim_map: WireClaimMap,
    /// The OIDC client secret / SAML SP private key; omitted keeps the stored one.
    #[serde(default)]
    pub secret: Option<String>,
    #[serde(default)]
    pub kind: Option<SsoKind>,
    #[serde(default, alias = "has_secret")]
    pub has_secret: Option<Value>,
    #[serde(default, alias = "created_at")]
    pub created_at: Option<Value>,
    #[serde(default, alias = "updated_at")]
    pub updated_at: Option<Value>,
    #[serde(default)]
    pub problems: Option<Value>,
}

/// One validation problem, named by the field's wire path (`config.issuerUrl`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

impl FieldError {
    fn new(field: &str, message: impl Into<String>) -> Self {
        FieldError {
            field: field.to_string(),
            message: message.into(),
        }
    }
}

/// Why an upsert body was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpsertError {
    /// Not the expected shape (unknown field, wrong type, unknown kind, ...).
    Malformed(String),
    /// Well-formed but invalid; every problem found is listed.
    Invalid(Vec<FieldError>),
}

impl UpsertError {
    /// The JSON error body for the API (`400`).
    pub fn body(&self) -> Value {
        match self {
            UpsertError::Malformed(message) => {
                json!({ "error": "malformed_sso_config", "message": message })
            }
            UpsertError::Invalid(fields) => {
                json!({ "error": "invalid_sso_config", "fields": fields })
            }
        }
    }
}

/// A validated upsert, in storage types, ready to persist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidUpsert {
    pub id: String,
    pub display_name: String,
    /// `deployment` | `domain:<d>`.
    pub scope: String,
    pub enabled: bool,
    pub config: SsoConfig,
    pub claim_map: ClaimMap,
    pub secret: Option<String>,
}

fn opt(s: Option<String>) -> Option<String> {
    s.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

impl From<WireOidc> for OidcConfig {
    fn from(w: WireOidc) -> Self {
        OidcConfig {
            issuer_url: w.issuer_url.trim().to_string(),
            client_id: w.client_id.trim().to_string(),
            redirect_url: w.redirect_url.trim().to_string(),
            scopes: w
                .scopes
                .into_iter()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            first_login_policy: w.first_login_policy.into(),
            require_verified_email: w.require_verified_email,
        }
    }
}

impl From<&OidcConfig> for WireOidc {
    fn from(c: &OidcConfig) -> Self {
        WireOidc {
            issuer_url: c.issuer_url.clone(),
            client_id: c.client_id.clone(),
            redirect_url: c.redirect_url.clone(),
            scopes: c.scopes.clone(),
            first_login_policy: c.first_login_policy.into(),
            require_verified_email: c.require_verified_email,
        }
    }
}

impl From<WireSaml> for SamlConfig {
    fn from(w: WireSaml) -> Self {
        SamlConfig {
            sp_entity_id: w.sp_entity_id.trim().to_string(),
            acs_url: w.acs_url.trim().to_string(),
            idp_metadata_url: opt(w.idp_metadata_url),
            idp_metadata_xml: opt(w.idp_metadata_xml),
            idp_sso_url: w.idp_sso_url.trim().to_string(),
            idp_slo_url: opt(w.idp_slo_url),
            idp_signing_certs_pem: w
                .idp_signing_certs_pem
                .into_iter()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            want_assertions_signed: w.want_assertions_signed,
            want_encrypted: w.want_encrypted,
            nameid_format: w.nameid_format.trim().to_string(),
            first_login_policy: w.first_login_policy.into(),
        }
    }
}

impl From<&SamlConfig> for WireSaml {
    fn from(c: &SamlConfig) -> Self {
        WireSaml {
            sp_entity_id: c.sp_entity_id.clone(),
            acs_url: c.acs_url.clone(),
            idp_metadata_url: c.idp_metadata_url.clone(),
            idp_metadata_xml: c.idp_metadata_xml.clone(),
            idp_sso_url: c.idp_sso_url.clone(),
            idp_slo_url: c.idp_slo_url.clone(),
            idp_signing_certs_pem: c.idp_signing_certs_pem.clone(),
            want_assertions_signed: c.want_assertions_signed,
            want_encrypted: c.want_encrypted,
            nameid_format: c.nameid_format.clone(),
            first_login_policy: c.first_login_policy.into(),
        }
    }
}

impl From<WireConfig> for SsoConfig {
    fn from(w: WireConfig) -> Self {
        match w {
            WireConfig::Oidc(c) => SsoConfig::Oidc(c.into()),
            WireConfig::Saml(c) => SsoConfig::Saml(c.into()),
        }
    }
}

impl From<&SsoConfig> for WireConfig {
    fn from(c: &SsoConfig) -> Self {
        match c {
            SsoConfig::Oidc(c) => WireConfig::Oidc(c.into()),
            SsoConfig::Saml(c) => WireConfig::Saml(c.into()),
        }
    }
}

impl From<WireClaimMap> for ClaimMap {
    fn from(w: WireClaimMap) -> Self {
        ClaimMap {
            email: opt(w.email),
            username: opt(w.username),
            display: opt(w.display),
            groups: opt(w.groups),
        }
    }
}

impl From<&ClaimMap> for WireClaimMap {
    fn from(c: &ClaimMap) -> Self {
        WireClaimMap {
            email: c.email.clone(),
            username: c.username.clone(),
            display: c.display.clone(),
            groups: c.groups.clone(),
        }
    }
}

/// An absolute `http`/`https` URL with a host.
fn http_url_problem(value: &str) -> Option<&'static str> {
    if value.is_empty() {
        return Some("is required");
    }
    match openidconnect::url::Url::parse(value) {
        Ok(u)
            if (u.scheme() == "https" || u.scheme() == "http")
                && u.host_str().is_some_and(|h| !h.is_empty()) =>
        {
            None
        }
        Ok(_) => Some("must be an http or https address with a host"),
        Err(_) => Some("is not a valid address"),
    }
}

fn check_url(errors: &mut Vec<FieldError>, field: &str, value: &str) {
    if let Some(p) = http_url_problem(value) {
        errors.push(FieldError::new(field, p));
    }
}

fn check_required(errors: &mut Vec<FieldError>, field: &str, value: &str) {
    if value.trim().is_empty() {
        errors.push(FieldError::new(field, "is required"));
    }
}

/// The problems of a stored config, with wire field names. Empty when the config
/// can be used to sign in as far as configuration goes. Used both at save time and
/// to flag rows already stored (e.g. by an older console) on the read side.
pub fn config_problems(config: &SsoConfig) -> Vec<FieldError> {
    let mut e = Vec::new();
    match config {
        SsoConfig::Oidc(c) => {
            check_url(&mut e, "config.issuerUrl", &c.issuer_url);
            if http_url_problem(&c.issuer_url).is_none()
                && let Ok(u) = openidconnect::url::Url::parse(&c.issuer_url)
                && (u.query().is_some() || u.fragment().is_some())
            {
                e.push(FieldError::new(
                    "config.issuerUrl",
                    "an issuer address has no query or fragment",
                ));
            }
            check_required(&mut e, "config.clientId", &c.client_id);
            check_url(&mut e, "config.redirectUrl", &c.redirect_url);
            if !c.scopes.iter().any(|s| s == "openid") {
                e.push(FieldError::new("config.scopes", "must include openid"));
            }
        }
        SsoConfig::Saml(c) => {
            check_url(&mut e, "config.idpSsoUrl", &c.idp_sso_url);
            if let Some(slo) = &c.idp_slo_url {
                check_url(&mut e, "config.idpSloUrl", slo);
            }
            if let Some(meta) = &c.idp_metadata_url {
                check_url(&mut e, "config.idpMetadataUrl", meta);
            }
            if c.idp_signing_certs_pem.is_empty() {
                e.push(FieldError::new(
                    "config.idpSigningCertsPem",
                    "at least one IdP signing certificate is required",
                ));
            }
            check_required(&mut e, "config.spEntityId", &c.sp_entity_id);
            check_url(&mut e, "config.acsUrl", &c.acs_url);
            check_required(&mut e, "config.nameidFormat", &c.nameid_format);
            // With `want_assertions_signed = false`, sign-in (saml/acs.rs) accepts
            // a response that carries no signature at all: anyone could post an
            // assertion naming any subject. Such a provider is never saved, and a
            // stored one is listed with this problem.
            if !c.want_assertions_signed {
                e.push(FieldError::new(
                    "config.wantAssertionsSigned",
                    "must be on: unsigned SAML assertions are not accepted",
                ));
            }
        }
    }
    e
}

fn id_ok(id: &str) -> bool {
    let b = id.as_bytes();
    !b.is_empty()
        && b.len() <= 63
        && (b[0].is_ascii_lowercase() || b[0].is_ascii_digit())
        && b.iter()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
}

fn scope_ok(scope: &str) -> bool {
    scope == "deployment"
        || scope.strip_prefix("domain:").is_some_and(|d| {
            !d.is_empty()
                && !d
                    .chars()
                    .any(|c| c.is_whitespace() || c.is_ascii_uppercase())
        })
}

/// Parse and validate a `POST /admin/sso` body.
pub fn parse_upsert(body: Value) -> Result<ValidUpsert, UpsertError> {
    let req: UpsertRequest =
        serde_json::from_value(body).map_err(|e| UpsertError::Malformed(e.to_string()))?;
    let mut errors = Vec::new();
    let id = req.id.trim().to_string();
    if !id_ok(&id) {
        errors.push(FieldError::new(
            "id",
            "must be 1-63 lower-case letters, digits or hyphens, starting with a letter or digit",
        ));
    }
    let display_name = req.display_name.trim().to_string();
    check_required(&mut errors, "displayName", &display_name);
    let scope = req
        .scope
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "deployment".to_string());
    if !scope_ok(&scope) {
        errors.push(FieldError::new(
            "scope",
            "must be `deployment` or `domain:<lower-case domain>`",
        ));
    }
    let config: SsoConfig = req.config.into();
    if let Some(kind) = req.kind
        && kind != config.kind()
    {
        errors.push(FieldError::new("kind", "does not match config.kind"));
    }
    errors.extend(config_problems(&config));
    if !errors.is_empty() {
        return Err(UpsertError::Invalid(errors));
    }
    Ok(ValidUpsert {
        id,
        display_name,
        scope,
        enabled: req.enabled,
        config,
        claim_map: req.claim_map.into(),
        secret: req.secret,
    })
}

/// A stored backend as the read side shows it (no secret material).
#[derive(Debug, Clone)]
pub struct BackendView<'a> {
    pub id: &'a str,
    pub kind: &'a str,
    pub display_name: &'a str,
    pub scope: &'a str,
    pub enabled: bool,
    /// The `sso_config.config` column (snake_case storage JSON).
    pub config_json: &'a str,
    /// The `sso_config.claim_map` column.
    pub claim_map_json: &'a str,
    pub has_secret: bool,
    pub created_at: &'a str,
    pub updated_at: &'a str,
}

/// The `GET /admin/sso` row for a stored backend, in wire names. A config that does
/// not parse is `null` (the console says it cannot be read); `problems` lists what
/// would stop the stored config from signing anyone in (empty when none).
pub fn backend_view(v: &BackendView<'_>) -> Value {
    let config = serde_json::from_str::<SsoConfig>(v.config_json).ok();
    let claim_map = serde_json::from_str::<ClaimMap>(v.claim_map_json).ok();
    let problems = config.as_ref().map(config_problems).unwrap_or_default();
    json!({
        "id": v.id,
        "kind": v.kind,
        "displayName": v.display_name,
        "scope": v.scope,
        "enabled": v.enabled,
        "config": config.as_ref().map(WireConfig::from),
        "claimMap": claim_map.as_ref().map(WireClaimMap::from),
        "hasSecret": v.has_secret,
        "createdAt": v.created_at,
        "updatedAt": v.updated_at,
        "problems": problems,
    })
}

#[cfg(test)]
#[path = "admin_wire_tests.rs"]
mod tests;
