//! Who may sign in for the first time (`signin.newUsers` and its per-type keys).
//!
//! The rule governs the creation of a NEW account only. An identity that already
//! has an account (engine accounts row, directory row with a first sign-in) is
//! never refused by it, so tightening the rule cannot lock an existing user out.
//!
//! # Values
//! * `open` (default): any identity that authenticates against its mail server or
//!   identity provider is created on first sign-in.
//! * `preprovisioned`: a matching pending entry added in advance by an admin is
//!   required (`mail_user` origin `provisioned`, never signed in, not expired; or
//!   a `POST /admin/users` provisioned user; or, when `signin.preprovision.domainWide`
//!   is on, a live capped `*@domain` entry). The entry is consumed once, by the
//!   first sign-in.
//! * `provisioned`: a pending entry of any origin (admin-added, imported or
//!   directory-synced) or a live domain-wide entry.
//! * `closed`: no new accounts at all.
//!
//! # Matrix
//! Types are `mail` (IMAP/POP3/EWS password sign-in), `sso` (OIDC/OAuth/SAML and
//! trusted-header sign-in) and `linked` (a mailbox linked to a signed-in user, per
//! provider kind `imap`, `pop3`, `ews`). Precedence, first non-`inherit` wins:
//! type@domain, type@global, [linked: kind@domain, kind@global before these],
//! `signin.newUsers`@domain, `signin.newUsers`@global, then `open`.
//!
//! A linked mailbox is evaluated against the linked mailbox's own domain.
//!
//! # Refusals
//! The check runs after the credentials or the identity provider have been
//! accepted and before anything is written, so a refusal leaves no account row,
//! stored credential, session or engine registration. `signin.refusalDetail`
//! `uniform` answers it exactly like a wrong password (401); `explain` answers
//! 403 `signinNotProvisioned`. Every refusal is written to the security audit log
//! (`signin-new-user-refused`, reason `new_user_refused`, rule, type, domain).
use std::future::Future;

use axum::extract::{Path as UrlPath, State};
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::{Value, json};

use mw_store::settings_policy::{SettingPolicyRow, SettingScope};
use mw_store::{Store, StoreError};

use crate::AppState;
use crate::admin_ctx::{AdminApiError, AdminCtx, AdminJson, check_if_match, etag_of, with_etag};
use crate::admin_router::AdminRegistrar;
use crate::capabilities::Capability;

/// The stable refusal code of the `explain` answer.
pub const REFUSAL_CODE: &str = "signinNotProvisioned";

pub const BASE_KEY: &str = "signin.newUsers";
const TYPE_KEYS: [&str; 6] = [
    "signin.newUsers.mail",
    "signin.newUsers.sso",
    "signin.newUsers.linked",
    "signin.newUsers.linked.imap",
    "signin.newUsers.linked.pop3",
    "signin.newUsers.linked.ews",
];
const OTHER_KEYS: [&str; 6] = [
    "signin.domainMode",
    "signin.refusalDetail",
    "signin.preprovision.ttlDays",
    "signin.preprovision.maxTtlDays",
    "signin.preprovision.domainWide",
    "signin.preprovision.retentionDays",
];

// ─── types and resolution ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    Open,
    Preprovisioned,
    Provisioned,
    Closed,
}

impl Rule {
    pub fn as_str(self) -> &'static str {
        match self {
            Rule::Open => "open",
            Rule::Preprovisioned => "preprovisioned",
            Rule::Provisioned => "provisioned",
            Rule::Closed => "closed",
        }
    }
    /// `known` was the name of `preprovisioned` before it had an enforcer; a stored
    /// `known` reads as `preprovisioned`. `inherit` is `None`.
    pub fn parse(value: &str) -> Option<Rule> {
        match value {
            "open" => Some(Rule::Open),
            "preprovisioned" | "known" => Some(Rule::Preprovisioned),
            "provisioned" => Some(Rule::Provisioned),
            "closed" => Some(Rule::Closed),
            _ => None,
        }
    }
}

/// How the identity is signing in (the call site decides; see [`scoped`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Mail,
    Sso,
    Linked,
}

/// The type a rule is resolved for: the [`Kind`] plus, for linked mailboxes, the
/// provider kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SigninType {
    Mail,
    Sso,
    Linked(&'static str),
}

impl SigninType {
    pub fn label(self) -> String {
        match self {
            SigninType::Mail => "mail".into(),
            SigninType::Sso => "sso".into(),
            SigninType::Linked(kind) => format!("linked.{kind}"),
        }
    }
    /// Keys to consult, most specific first (before the base key).
    fn keys(self) -> Vec<&'static str> {
        match self {
            SigninType::Mail => vec!["signin.newUsers.mail"],
            SigninType::Sso => vec!["signin.newUsers.sso"],
            SigninType::Linked(kind) => {
                let specific = match kind {
                    "imap" => Some("signin.newUsers.linked.imap"),
                    "pop3" => Some("signin.newUsers.linked.pop3"),
                    "ews" => Some("signin.newUsers.linked.ews"),
                    _ => None,
                };
                specific
                    .into_iter()
                    .chain(["signin.newUsers.linked"])
                    .collect()
            }
        }
    }
    /// The provider kind of a mail login: `imap`, `pop3` or `ews`.
    pub fn for_kind(kind: Kind, account: mw_store::AccountKind) -> SigninType {
        match kind {
            Kind::Mail => SigninType::Mail,
            Kind::Sso => SigninType::Sso,
            Kind::Linked => SigninType::Linked(match account {
                mw_store::AccountKind::Pop3 => "pop3",
                mw_store::AccountKind::Ews => "ews",
                _ => "imap",
            }),
        }
    }
}

type Capture = std::sync::Arc<std::sync::Mutex<Option<Refused>>>;
tokio::task_local! { static CTX: (Kind, Capture); }

/// Run `fut` (a sign-in) as `kind`; also returns the refusal the sign-in raised, if
/// any, so the caller (which holds the `AppState`) can log it. Without a scope a
/// sign-in is `mail`.
pub async fn scoped<F: Future>(kind: Kind, fut: F) -> (F::Output, Option<Refused>) {
    let capture: Capture = Default::default();
    let out = CTX.scope((kind, capture.clone()), fut).await;
    let refused = capture.lock().unwrap_or_else(|e| e.into_inner()).take();
    (out, refused)
}

pub fn current_kind() -> Kind {
    CTX.try_with(|(k, _)| *k).unwrap_or(Kind::Mail)
}

/// Remember the refusal for the enclosing [`scoped`] call.
pub fn note_refusal(refused: &Refused) {
    let _ = CTX.try_with(|(_, c)| {
        *c.lock().unwrap_or_else(|e| e.into_inner()) = Some(refused.clone());
    });
}

/// Request-extension marker set by the linked-mailbox route: the login that follows
/// links a mailbox instead of signing a user in.
#[derive(Clone, Copy, Debug)]
pub struct Linking;

fn parse_json_string(raw: &str) -> Option<String> {
    serde_json::from_str::<String>(raw).ok()
}

/// One key's own value at `domain` (when given) or globally; `None` = unset/inherit.
async fn own_value(
    store: &Store,
    key: &str,
    domain: Option<&str>,
) -> Result<Option<Rule>, StoreError> {
    let raw = match domain {
        Some(domain) => store
            .get_setting_value(SettingScope::Domain, domain, key)
            .await?
            .map(|row| row.value),
        None => store
            .get_setting_policy(key)
            .await?
            .and_then(|p| p.global_value),
    };
    Ok(raw.as_deref().and_then(parse_json_string).and_then(|v| Rule::parse(&v)))
}

/// The effective rule for `(domain, type)` and the key and scope that supplied it.
pub async fn resolve(
    store: &Store,
    domain: &str,
    ty: SigninType,
) -> Result<(Rule, &'static str, &'static str), StoreError> {
    let domain = if domain.is_empty() {
        None
    } else {
        Some(store.canonical_policy_domain(domain).await?)
    };
    let mut keys = ty.keys();
    keys.push(BASE_KEY);
    for key in keys {
        if let Some(domain) = &domain
            && let Some(rule) = own_value(store, key, Some(domain)).await?
        {
            return Ok((rule, key, "domain"));
        }
        if let Some(rule) = own_value(store, key, None).await? {
            return Ok((rule, key, "global"));
        }
    }
    Ok((Rule::Open, BASE_KEY, "default"))
}

// ─── the decision ───────────────────────────────────────────────────────────────

/// What a successful admission has to spend once the account exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Spend {
    Nothing,
    /// A pending directory entry (the spellings it may be stored under).
    Pending(Vec<String>),
    /// A domain-wide entry (id).
    Allow(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refused {
    pub rule: Rule,
    pub key: &'static str,
    pub ty: String,
    pub domain: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Admission {
    Admit(Spend),
    Refuse(Refused),
}

impl Admission {
    pub fn is_admit(&self) -> bool {
        matches!(self, Admission::Admit(_))
    }
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

async fn bool_value(store: &Store, key: &str) -> Result<bool, StoreError> {
    Ok(store
        .get_setting_policy(key)
        .await?
        .and_then(|p| p.global_value)
        .is_some_and(|v| v == "true"))
}

/// Decide whether `login` may be created as a new account of type `ty`.
/// `existing` is the caller's own knowledge that the account already exists; the
/// directory is consulted too. Errors fail closed (the caller refuses the sign-in).
pub async fn admit(
    store: &Store,
    login: &str,
    ty: SigninType,
    existing: bool,
) -> Result<Admission, StoreError> {
    if existing {
        return Ok(Admission::Admit(Spend::Nothing));
    }
    let canonical = crate::domain_config::canonical(store, login, None)
        .await
        .map_err(StoreError::Corrupt)?;
    let folded = mw_store::Store::fold_account_name(login);
    let spellings = [canonical.address.as_str(), folded.as_str(), login.trim()];
    let standing = store.directory_standing(&spellings).await?;
    if standing.as_ref().is_some_and(|s| s.has_signed_in()) {
        return Ok(Admission::Admit(Spend::Nothing));
    }
    let (rule, key, _) = resolve(store, &canonical.domain, ty).await?;
    let refuse = || {
        Admission::Refuse(Refused {
            rule,
            key,
            ty: ty.label(),
            domain: canonical.domain.clone(),
        })
    };
    let at = now();
    let pending = |s: &mw_store::signin_admission::DirectoryStanding| match rule {
        Rule::Preprovisioned => s.is_pending_admin_entry(at),
        _ => s.is_pending(at),
    };
    match rule {
        Rule::Open => Ok(Admission::Admit(Spend::Nothing)),
        Rule::Closed => Ok(refuse()),
        Rule::Preprovisioned | Rule::Provisioned => {
            if standing.as_ref().is_some_and(pending) {
                let mut names: Vec<String> = spellings.iter().map(|s| (*s).to_string()).collect();
                names.dedup();
                return Ok(Admission::Admit(Spend::Pending(names)));
            }
            if bool_value(store, "signin.preprovision.domainWide").await? {
                let local = canonical
                    .address
                    .rsplit_once('@')
                    .map_or(canonical.address.as_str(), |(l, _)| l);
                if let Some(entry) = store.signin_allow_match(&canonical.domain, local, at).await? {
                    return Ok(Admission::Admit(Spend::Allow(entry.id)));
                }
            }
            Ok(refuse())
        }
    }
}

/// Before the account is created: spend a domain-wide use (compare-and-swap, so
/// concurrent sign-ins cannot exceed its cap). `false` = lost the race, refuse.
pub async fn reserve(store: &Store, spend: &Spend) -> Result<bool, StoreError> {
    match spend {
        Spend::Allow(id) => store.consume_signin_allow(id, now()).await,
        _ => Ok(true),
    }
}

/// After the account exists: consume the pending entry (compare-and-swap on the
/// entry never having signed in; best-effort, the account is already created and
/// the entry would otherwise just read as signed-in at registration).
pub async fn complete(store: &Store, spend: &Spend) {
    if let Spend::Pending(names) = spend {
        for name in names {
            if store.consume_pending_user(name, now()).await.unwrap_or(false) {
                break;
            }
        }
    }
}

// ─── answers and logging ────────────────────────────────────────────────────────

pub async fn explains(store: &Store) -> bool {
    match store.get_setting_policy("signin.refusalDetail").await {
        Ok(Some(p)) => p.global_value.as_deref().and_then(parse_json_string).as_deref() != Some("uniform"),
        _ => true,
    }
}

/// The answer to a refused new user: 401 like a wrong password (`uniform`) or 403
/// with a stable code (`explain`).
pub fn refusal_response(explain: bool) -> Response {
    if explain {
        (
            StatusCode::FORBIDDEN,
            Json(json!({
                "code": REFUSAL_CODE,
                "error": "new accounts are not created for this address",
            })),
        )
            .into_response()
    } else {
        crate::unauthorized()
    }
}

/// Write the refusal to the security audit log (no credential, no token).
pub async fn record_refusal(state: &AppState, refused: &Refused) {
    tracing::info!(
        "sign-in refused: new user, rule {} type {}",
        refused.rule.as_str(),
        refused.ty
    );
    crate::audit::security_event(
        state,
        crate::audit::SecurityEvent::system("signin-new-user-refused")
            .target(format!("domain:{}", refused.domain))
            .detail(json!({
                "reason": "new_user_refused",
                "rule": refused.rule.as_str(),
                "key": refused.key,
                "type": refused.ty,
                "domain": refused.domain,
            })),
    )
    .await;
}

/// For paths that hold an `AppState` (SSO, proxy mode, header auth): decide, log,
/// and return the response to send when refused. `None` = admitted (the spend, if
/// any, is returned for the caller to settle).
pub async fn gate(
    state: &AppState,
    login: &str,
    ty: SigninType,
    existing: bool,
) -> Result<Spend, Response> {
    match admit(&state.store, login, ty, existing).await {
        Ok(Admission::Admit(spend)) => match reserve(&state.store, &spend).await {
            Ok(true) => Ok(spend),
            Ok(false) => Err(refusal_response(explains(&state.store).await)),
            Err(e) => Err(crate::account_gate::unavailable("sign-in policy", e)),
        },
        Ok(Admission::Refuse(refused)) => {
            record_refusal(state, &refused).await;
            Err(refusal_response(explains(&state.store).await))
        }
        Err(e) => Err(crate::account_gate::unavailable("sign-in policy", e)),
    }
}

// ─── admin API: /signin-policy ──────────────────────────────────────────────────

pub(crate) const PREPROVISION_ROUTE: &str = "/preprovisioned-users";

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, crate::settings_registry::SIGNIN_POLICY_ROUTE, Capability::SettingsRead, read);
    r.admin_endpoint(Method::PUT, crate::settings_registry::SIGNIN_POLICY_ROUTE, Capability::SettingsWrite, save);
    r.admin_endpoint(Method::GET, "/signin-policy/preview", Capability::SettingsRead, preview);
    r.admin_endpoint(Method::GET, PREPROVISION_ROUTE, Capability::UsersRead, pending_list);
    r.admin_endpoint(Method::POST, PREPROVISION_ROUTE, Capability::UsersCreate, pending_add);
    r.admin_endpoint(Method::PUT, "/preprovisioned-users/{address}", Capability::UsersWrite, pending_extend);
    r.admin_endpoint(Method::DELETE, "/preprovisioned-users/{address}", Capability::UsersDelete, pending_revoke);
}

fn all_keys() -> impl Iterator<Item = &'static str> {
    std::iter::once(BASE_KEY).chain(TYPE_KEYS).chain(OTHER_KEYS)
}

fn value_of(raw: Option<&str>) -> Value {
    raw.and_then(|r| serde_json::from_str::<Value>(r).ok()).unwrap_or(Value::Null)
}

/// The global values, the domain overrides the admin may see, and warnings.
async fn view(state: &AppState, ctx: &AdminCtx) -> Result<Value, AdminApiError> {
    let mut global = serde_json::Map::new();
    let mut domains = serde_json::Map::new();
    let mut defaults = serde_json::Map::new();
    for key in all_keys() {
        let def = crate::settings_registry::lookup(key).expect("registered signin key");
        defaults.insert(key.into(), def.default_value());
        let policy = state.store.get_setting_policy(key).await?;
        global.insert(key.into(), value_of(policy.as_ref().and_then(|p| p.global_value.as_deref())));
        if def.scopes == crate::settings_registry::Scopes::GlobalDomain {
            let mut rows = serde_json::Map::new();
            for row in state.store.list_setting_values(key, SettingScope::Domain, None, 500).await? {
                let visible = ctx
                    .scope(Capability::SettingsRead)
                    .is_none_or(|s| s.allows(&row.scope_id));
                if visible {
                    rows.insert(row.scope_id.clone(), value_of(Some(&row.value)));
                }
            }
            domains.insert(key.into(), Value::Object(rows));
        }
    }
    let mut warnings = Vec::new();
    for key in std::iter::once(BASE_KEY).chain(TYPE_KEYS) {
        let v = global.get(key).and_then(Value::as_str).unwrap_or("");
        if matches!(Rule::parse(v), Some(Rule::Preprovisioned | Rule::Provisioned | Rule::Closed)) {
            warnings.push(json!({"code": "existingAccountsUnaffected", "key": key}));
            break;
        }
    }
    Ok(json!({
        "keys": {
            "rules": std::iter::once(BASE_KEY).chain(TYPE_KEYS).collect::<Vec<_>>(),
            "other": OTHER_KEYS,
        },
        "global": global,
        "domains": domains,
        "defaults": defaults,
        "warnings": warnings,
    }))
}

fn refusal(r: crate::settings_registry::Refusal) -> AdminApiError {
    let status = StatusCode::from_u16(r.status).unwrap_or(StatusCode::UNPROCESSABLE_ENTITY);
    let mut e = AdminApiError::new(status, r.code, r.detail);
    if let Some(field) = r.field {
        e = e.at(field);
    }
    e
}

fn private(value: Value) -> Response {
    let tag = etag_of(&value);
    with_etag(([(header::CACHE_CONTROL, "private, no-store")], Json(value)), &tag)
}

async fn read(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require(Capability::SettingsRead)?;
    Ok(private(view(&state, &ctx).await?))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Change {
    key: String,
    /// `global` (default) or `domain`.
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    domain: Option<String>,
    /// `null` returns the key to its default (global) or removes the override.
    value: Value,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Save {
    changes: Vec<Change>,
    #[serde(default)]
    dry_run: bool,
}

async fn save(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    AdminJson(body): AdminJson<Save>,
) -> Result<Response, AdminApiError> {
    ctx.require(Capability::SettingsWrite)?;
    if body.changes.is_empty() || body.changes.len() > 64 {
        return Err(AdminApiError::invalid("invalidChanges", "between 1 and 64 changes are required", "/changes"));
    }
    let before = view(&state, &ctx).await?;
    check_if_match(&headers, &etag_of(&before), None, None)?;
    // Validate everything before writing anything.
    let mut plan = Vec::new();
    for (i, c) in body.changes.iter().enumerate() {
        let field = format!("/changes/{i}/value");
        let def = crate::settings_registry::lookup(&c.key)
            .filter(|d| d.dedicated_route == Some(crate::settings_registry::SIGNIN_POLICY_ROUTE))
            .ok_or_else(|| AdminApiError::invalid("unknownKey", format!("{} is not a sign-in policy key", c.key), format!("/changes/{i}/key")))?;
        let value = crate::settings_registry::validate(def, &c.value, &field)
            .map_err(refusal)?;
        let domain = match c.scope.as_deref().unwrap_or("global") {
            "global" => {
                ctx.require_unscoped(Capability::SettingsWrite)?;
                crate::settings_registry::refuse_if_pinned(def, &crate::settings_registry::process_env)
                    .map_err(refusal)?;
                None
            }
            "domain" => {
                if def.scopes != crate::settings_registry::Scopes::GlobalDomain {
                    return Err(AdminApiError::invalid("notDomainScoped", format!("{} has no domain values", c.key), format!("/changes/{i}/scope")));
                }
                let domain = c.domain.as_deref().unwrap_or("");
                let domain = crate::domain_config::normalize_domain(domain)
                    .map_err(|e| AdminApiError::invalid("invalidDomain", e, format!("/changes/{i}/domain")))?;
                ctx.require_domain_for(Capability::SettingsWrite, &domain)?;
                Some(domain)
            }
            _ => return Err(AdminApiError::invalid("invalidScope", "scope is global or domain", format!("/changes/{i}/scope"))),
        };
        plan.push((c.key.clone(), domain, value));
    }
    if body.dry_run {
        return Ok(Json(json!({
            "dryRun": true,
            "changes": plan.iter().map(|(k, d, v)| json!({"key": k, "domain": d, "value": v})).collect::<Vec<_>>(),
            "warnings": [{"code": "existingAccountsUnaffected"}],
        }))
        .into_response());
    }
    let at = chrono::Utc::now().timestamp();
    for (key, domain, value) in &plan {
        match domain {
            Some(domain) if value.is_null() => {
                state.store.delete_setting_value(SettingScope::Domain, domain, key).await?;
            }
            Some(domain) => {
                state
                    .store
                    .put_setting_value(&mw_store::settings_policy::SettingValueRow {
                        scope: SettingScope::Domain,
                        scope_id: domain.clone(),
                        key: key.clone(),
                        value: value.to_string(),
                        locked: false,
                        updated_by: ctx.username.clone(),
                        updated_at: at,
                    })
                    .await?;
            }
            None => {
                let current = state.store.get_setting_policy(key).await?;
                let replacement = if value.is_null() {
                    None
                } else {
                    Some(SettingPolicyRow {
                        key: key.clone(),
                        global_value: Some(value.to_string()),
                        allow_user_override: current.as_ref().is_some_and(|p| p.allow_user_override),
                        global_locked: current.as_ref().is_some_and(|p| p.global_locked),
                        updated_by: ctx.username.clone(),
                        updated_at: at,
                    })
                };
                if !state.store.compare_setting_policy(key, current.as_ref(), replacement.as_ref()).await? {
                    return Err(AdminApiError::stale(None, None));
                }
            }
        }
    }
    let after = view(&state, &ctx).await?;
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "signin-policy-changed",
        "setting:signin.newUsers",
        Some(before),
        Some(after.clone()),
    )
    .await;
    Ok(private(after))
}

#[derive(Deserialize)]
struct PreviewQuery {
    address: String,
    #[serde(rename = "type")]
    ty: Option<String>,
}

/// Read-only simulator: what a NEW sign-in of `address` would get under the stored
/// rules. Nothing is written.
async fn preview(
    State(state): State<AppState>,
    ctx: AdminCtx,
    axum::extract::Query(q): axum::extract::Query<PreviewQuery>,
) -> Result<Response, AdminApiError> {
    ctx.require(Capability::SettingsRead)?;
    let ty = match q.ty.as_deref().unwrap_or("mail") {
        "mail" => SigninType::Mail,
        "sso" => SigninType::Sso,
        "linked.imap" | "linked" => SigninType::Linked("imap"),
        "linked.pop3" => SigninType::Linked("pop3"),
        "linked.ews" => SigninType::Linked("ews"),
        _ => return Err(AdminApiError::invalid("invalidType", "unknown type", "/type")),
    };
    let canonical = crate::domain_config::canonical(&state.store, &q.address, None)
        .await
        .map_err(|e| AdminApiError::invalid("invalidAddress", e, "/address"))?;
    ctx.require_domain_for(Capability::SettingsRead, &canonical.domain)?;
    let (rule, key, scope) = resolve(&state.store, &canonical.domain, ty).await?;
    let admission = admit(&state.store, &q.address, ty, false).await?;
    Ok(Json(json!({
        "address": canonical.address,
        "type": ty.label(),
        "rule": rule.as_str(),
        "ruleKey": key,
        "ruleScope": scope,
        "wouldBeRefused": !admission.is_admit(),
    }))
    .into_response())
}

// ─── admin API: pre-provisioned users ───────────────────────────────────────────

const BULK_CAP: usize = 500;
const DAY: i64 = 86_400;

async fn int_key(store: &Store, key: &str, default: i64) -> i64 {
    store
        .get_setting_policy(key)
        .await
        .ok()
        .flatten()
        .and_then(|p| p.global_value)
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// `(default ttl days, max ttl days)`.
async fn ttl_limits(store: &Store) -> (i64, i64) {
    let max = int_key(store, "signin.preprovision.maxTtlDays", 90).await.max(1);
    let default = int_key(store, "signin.preprovision.ttlDays", 30).await.clamp(1, max);
    (default, max)
}

fn entry_json(p: &mw_store::signin_admission::PendingUser, at: i64) -> Value {
    let state = if p.first_signin_at.is_some() {
        "consumed"
    } else if p.expires_at.is_some_and(|t| t <= at) {
        "expired"
    } else {
        "pending"
    };
    json!({
        "address": p.address, "domain": p.domain, "origin": p.origin, "note": p.note,
        "createdAt": p.created_at, "createdBy": p.created_by, "expiresAt": p.expires_at,
        "consumedAt": p.first_signin_at, "state": state,
    })
}

async fn pending_list(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require(Capability::UsersRead)?;
    let filter: Option<Vec<String>> = ctx
        .domain_filter(Capability::UsersRead)
        .map(|s| s.iter().cloned().collect());
    let at = chrono::Utc::now().timestamp();
    // Bounded cleanup rides on the list read (no separate scheduler to configure).
    let _ = cleanup(&state.store).await;
    let rows = state.store.list_pending_users(filter.as_deref(), 1000).await?;
    let (default, max) = ttl_limits(&state.store).await;
    Ok(private(json!({
        "entries": rows.iter().map(|p| entry_json(p, at)).collect::<Vec<_>>(),
        "ttlDays": default, "maxTtlDays": max, "bulkCap": BULK_CAP,
    })))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct AddReq {
    /// Addresses, one per entry (a pasted list or CSV is split by the client; the
    /// server also splits on newlines, commas and semicolons).
    addresses: Vec<String>,
    #[serde(default)]
    note: Option<String>,
    #[serde(default)]
    ttl_days: Option<i64>,
    /// `*@domain` entries with a use cap (admin-only; needs `domainWide`).
    #[serde(default)]
    domain_wide_max_uses: Option<i64>,
}

async fn pending_add(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(body): AdminJson<AddReq>,
) -> Result<Response, AdminApiError> {
    ctx.require(Capability::UsersCreate)?;
    let (default, max) = ttl_limits(&state.store).await;
    let ttl = body.ttl_days.unwrap_or(default);
    if !(1..=max).contains(&ttl) {
        return Err(AdminApiError::invalid("invalidTtl", format!("ttlDays must be between 1 and {max}"), "/ttlDays"));
    }
    if body.note.as_deref().is_some_and(|n| n.chars().count() > 200 || n.chars().any(char::is_control)) {
        return Err(AdminApiError::invalid("invalidNote", "note is at most 200 characters", "/note"));
    }
    let items: Vec<String> = body
        .addresses
        .iter()
        .flat_map(|a| a.split(['\n', ',', ';']).map(|s| s.trim().to_string()).collect::<Vec<_>>())
        .filter(|s| !s.is_empty())
        .collect();
    if items.is_empty() || items.len() > BULK_CAP {
        return Err(AdminApiError::invalid("invalidAddresses", format!("between 1 and {BULK_CAP} addresses are required"), "/addresses"));
    }
    let at = chrono::Utc::now().timestamp();
    let expires = Some(at + ttl * DAY);
    let mut results = Vec::new();
    for item in items {
        // Domain-wide entry: `*@example.org`.
        if let Some(domain) = item.strip_prefix("*@") {
            ctx.require_unscoped(Capability::UsersCreate)?;
            if !bool_value(&state.store, "signin.preprovision.domainWide").await? {
                results.push(json!({"address": item, "result": "domainWideDisabled"}));
                continue;
            }
            let Some(cap) = body.domain_wide_max_uses.filter(|c| (1..=100_000).contains(c)) else {
                return Err(AdminApiError::invalid("invalidCap", "domainWideMaxUses (1-100000) is required for *@domain entries", "/domainWideMaxUses"));
            };
            let domain = match crate::domain_config::normalize_domain(domain) {
                Ok(d) => d,
                Err(_) => {
                    results.push(json!({"address": item, "result": "invalidAddress"}));
                    continue;
                }
            };
            state
                .store
                .put_signin_allow(&uuid::Uuid::new_v4().to_string(), &domain, "*", body.note.as_deref(), expires, Some(cap), &ctx.username, at)
                .await?;
            results.push(json!({"address": item, "result": "added"}));
            continue;
        }
        let canonical = match crate::domain_config::canonical(&state.store, &item, None).await {
            Ok(c) if item.contains('@') && !c.domain.is_empty() => c,
            _ => {
                results.push(json!({"address": item, "result": "invalidAddress"}));
                continue;
            }
        };
        if let Err(e) = ctx.require_domain(&canonical.domain) {
            results.push(json!({"address": item, "result": "outOfScope", "code": e.code.to_string()}));
            continue;
        }
        let added = state
            .store
            .put_provisioned_user(&canonical.address, "provisioned", body.note.as_deref(), expires, &ctx.username, at)
            .await?;
        results.push(json!({"address": canonical.address, "result": if added { "added" } else { "exists" }}));
    }
    let added = results.iter().filter(|r| r["result"] == "added").count();
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "preprovisioned-users-added",
        "setting:signin.newUsers",
        None,
        Some(json!({"requested": results.len(), "added": added, "ttlDays": ttl})),
    )
    .await;
    Ok((StatusCode::OK, Json(json!({"results": results, "added": added}))).into_response())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ExtendReq {
    ttl_days: i64,
}

async fn pending_extend(
    State(state): State<AppState>,
    ctx: AdminCtx,
    UrlPath(address): UrlPath<String>,
    AdminJson(body): AdminJson<ExtendReq>,
) -> Result<Response, AdminApiError> {
    ctx.require(Capability::UsersWrite)?;
    let canonical = crate::domain_config::canonical(&state.store, &address, None)
        .await
        .map_err(|e| AdminApiError::invalid("invalidAddress", e, "/address"))?;
    ctx.require_domain(&canonical.domain)?;
    let (_, max) = ttl_limits(&state.store).await;
    if !(1..=max).contains(&body.ttl_days) {
        return Err(AdminApiError::invalid("invalidTtl", format!("ttlDays must be between 1 and {max}"), "/ttlDays"));
    }
    let at = chrono::Utc::now().timestamp();
    if !state.store.extend_pending_user(&canonical.address, Some(at + body.ttl_days * DAY)).await? {
        return Err(AdminApiError::not_found("no pending entry for this address"));
    }
    crate::admin_accounts::audit(&state, &ctx, "preprovisioned-user-extended", &format!("user:{}", canonical.address), None, Some(json!({"ttlDays": body.ttl_days}))).await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn pending_revoke(
    State(state): State<AppState>,
    ctx: AdminCtx,
    UrlPath(address): UrlPath<String>,
) -> Result<Response, AdminApiError> {
    ctx.require(Capability::UsersDelete)?;
    let canonical = crate::domain_config::canonical(&state.store, &address, None)
        .await
        .map_err(|e| AdminApiError::invalid("invalidAddress", e, "/address"))?;
    ctx.require_domain(&canonical.domain)?;
    if !state.store.revoke_pending_user(&canonical.address).await? {
        return Err(AdminApiError::not_found("no pending entry for this address"));
    }
    crate::admin_accounts::audit(&state, &ctx, "preprovisioned-user-revoked", &format!("user:{}", canonical.address), None, None).await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Bounded cleanup of expired pending entries older than the retention window
/// (`signin.preprovision.retentionDays`). Called from the maintenance tick.
pub(crate) async fn cleanup(store: &Store) -> Result<u64, StoreError> {
    let days = int_key(store, "signin.preprovision.retentionDays", 30).await;
    store
        .purge_expired_pending(chrono::Utc::now().timestamp() - days * DAY)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use mw_store::settings_policy::SettingValueRow;

    async fn store() -> Store {
        Store::open_in_memory(mw_store::ServerKey::generate()).await.unwrap()
    }

    async fn set_global(s: &Store, key: &str, v: &str) {
        s.put_setting_policy(&SettingPolicyRow {
            key: key.into(),
            global_value: Some(serde_json::to_string(v).unwrap()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "t".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
    }

    async fn set_domain(s: &Store, key: &str, domain: &str, v: &str) {
        s.put_setting_value(&SettingValueRow {
            scope: SettingScope::Domain,
            scope_id: domain.into(),
            key: key.into(),
            value: serde_json::to_string(v).unwrap(),
            locked: false,
            updated_by: "t".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
    }

    async fn rule(s: &Store, domain: &str, ty: SigninType) -> Rule {
        resolve(s, domain, ty).await.unwrap().0
    }

    #[tokio::test]
    async fn default_is_open_everywhere() {
        let s = store().await;
        for ty in [SigninType::Mail, SigninType::Sso, SigninType::Linked("imap")] {
            assert_eq!(rule(&s, "x.test", ty).await, Rule::Open);
        }
    }

    #[tokio::test]
    async fn precedence_type_domain_beats_type_global_beats_base() {
        let s = store().await;
        set_global(&s, BASE_KEY, "closed").await;
        assert_eq!(rule(&s, "x.test", SigninType::Mail).await, Rule::Closed);
        set_domain(&s, BASE_KEY, "x.test", "open").await;
        assert_eq!(rule(&s, "x.test", SigninType::Mail).await, Rule::Open, "domain base beats global base");
        assert_eq!(rule(&s, "y.test", SigninType::Mail).await, Rule::Closed);
        set_global(&s, "signin.newUsers.mail", "provisioned").await;
        assert_eq!(rule(&s, "x.test", SigninType::Mail).await, Rule::Provisioned, "type global beats domain base");
        assert_eq!(rule(&s, "x.test", SigninType::Sso).await, Rule::Open, "other types keep the base");
        set_domain(&s, "signin.newUsers.mail", "x.test", "closed").await;
        assert_eq!(rule(&s, "x.test", SigninType::Mail).await, Rule::Closed);
        set_domain(&s, "signin.newUsers.mail", "x.test", "inherit").await;
        assert_eq!(rule(&s, "x.test", SigninType::Mail).await, Rule::Provisioned, "inherit defers to the next level");
    }

    #[tokio::test]
    async fn linked_kinds_fall_back_to_linked_then_base() {
        let s = store().await;
        set_global(&s, "signin.newUsers.linked", "closed").await;
        set_global(&s, "signin.newUsers.linked.imap", "open").await;
        assert_eq!(rule(&s, "x.test", SigninType::Linked("imap")).await, Rule::Open);
        assert_eq!(rule(&s, "x.test", SigninType::Linked("pop3")).await, Rule::Closed);
        assert_eq!(rule(&s, "x.test", SigninType::Mail).await, Rule::Open);
    }

    #[tokio::test]
    async fn legacy_known_reads_as_preprovisioned() {
        let s = store().await;
        set_global(&s, BASE_KEY, "known").await;
        assert_eq!(rule(&s, "x.test", SigninType::Mail).await, Rule::Preprovisioned);
    }

    #[tokio::test]
    async fn decisions_for_new_and_existing_users() {
        let s = store().await;
        let mail = SigninType::Mail;
        assert!(admit(&s, "new@x.test", mail, false).await.unwrap().is_admit(), "open admits");
        set_global(&s, BASE_KEY, "closed").await;
        assert!(!admit(&s, "new@x.test", mail, false).await.unwrap().is_admit());
        assert!(admit(&s, "new@x.test", mail, true).await.unwrap().is_admit(), "existing account is never refused");
        s.put_provisioned_user("signed@x.test", "provisioned", None, None, "root", 1).await.unwrap();
        s.consume_pending_user("signed@x.test", 2).await.unwrap();
        assert!(admit(&s, "signed@x.test", mail, false).await.unwrap().is_admit(), "prior sign-in is existing");
        // closed refuses even a pending entry
        s.put_provisioned_user("pend@x.test", "provisioned", None, None, "root", 1).await.unwrap();
        assert!(!admit(&s, "pend@x.test", mail, false).await.unwrap().is_admit());

        set_global(&s, BASE_KEY, "preprovisioned").await;
        let a = admit(&s, "Pend@X.test", mail, false).await.unwrap();
        assert!(matches!(a, Admission::Admit(Spend::Pending(_))));
        assert!(!admit(&s, "other@x.test", mail, false).await.unwrap().is_admit());
        s.put_provisioned_user("imp@x.test", "import", None, None, "sync", 1).await.unwrap();
        assert!(!admit(&s, "imp@x.test", mail, false).await.unwrap().is_admit(), "preprovisioned needs an admin entry");
        set_global(&s, BASE_KEY, "provisioned").await;
        assert!(admit(&s, "imp@x.test", mail, false).await.unwrap().is_admit(), "provisioned also admits imported entries");
        assert!(!admit(&s, "other@x.test", mail, false).await.unwrap().is_admit());
    }

    #[tokio::test]
    async fn expired_entry_does_not_admit_and_consumption_is_single_use() {
        let s = store().await;
        set_global(&s, BASE_KEY, "preprovisioned").await;
        let past = chrono::Utc::now().timestamp() - 10;
        s.put_provisioned_user("old@x.test", "provisioned", None, Some(past), "root", 1).await.unwrap();
        assert!(!admit(&s, "old@x.test", SigninType::Mail, false).await.unwrap().is_admit());
        s.put_provisioned_user("new@x.test", "provisioned", None, None, "root", 1).await.unwrap();
        let Admission::Admit(spend) = admit(&s, "new@x.test", SigninType::Mail, false).await.unwrap() else { panic!() };
        complete(&s, &spend).await;
        complete(&s, &spend).await;
        let st = s.directory_standing(&["new@x.test"]).await.unwrap().unwrap();
        assert!(st.has_signed_in());
        // after consumption the entry no longer admits a "new" user (an account exists by then)
        assert!(admit(&s, "new@x.test", SigninType::Mail, false).await.unwrap().is_admit(), "signed-in user is existing");
    }

    #[tokio::test]
    async fn domain_wide_entry_needs_the_switch_and_respects_its_cap() {
        let s = store().await;
        set_global(&s, BASE_KEY, "preprovisioned").await;
        s.put_signin_allow("w", "x.test", "*", None, None, Some(1), "root", 1).await.unwrap();
        assert!(!admit(&s, "a@x.test", SigninType::Mail, false).await.unwrap().is_admit(), "off by default");
        set_global(&s, "signin.preprovision.domainWide", "true").await;
        // stored as JSON bool text for Bool kinds
        s.put_setting_policy(&SettingPolicyRow {
            key: "signin.preprovision.domainWide".into(),
            global_value: Some("true".into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "t".into(),
            updated_at: 1,
        }).await.unwrap();
        let Admission::Admit(spend) = admit(&s, "a@x.test", SigninType::Mail, false).await.unwrap() else { panic!("admit") };
        assert!(reserve(&s, &spend).await.unwrap());
        assert!(!reserve(&s, &spend).await.unwrap(), "cap of one use");
        assert!(!admit(&s, "b@x.test", SigninType::Mail, false).await.unwrap().is_admit());
    }

    #[test]
    fn refusal_modes() {
        assert_eq!(refusal_response(false).status(), StatusCode::UNAUTHORIZED);
        assert_eq!(refusal_response(true).status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn explain_is_the_default_and_uniform_switches_it() {
        let s = store().await;
        assert!(explains(&s).await);
        set_global(&s, "signin.refusalDetail", "uniform").await;
        assert!(!explains(&s).await);
    }

    #[tokio::test]
    async fn cleanup_removes_only_long_expired_entries() {
        let s = store().await;
        let now = chrono::Utc::now().timestamp();
        s.put_provisioned_user("gone@x.test", "provisioned", None, Some(now - 90 * DAY), "root", 1).await.unwrap();
        s.put_provisioned_user("recent@x.test", "provisioned", None, Some(now - DAY), "root", 1).await.unwrap();
        assert_eq!(cleanup(&s).await.unwrap(), 1);
        assert!(s.directory_standing(&["recent@x.test"]).await.unwrap().is_some());
    }

    #[test]
    fn every_policy_key_is_registered_on_the_dedicated_route() {
        for key in all_keys() {
            let def = crate::settings_registry::lookup(key).expect(key);
            assert_eq!(def.dedicated_route, Some(crate::settings_registry::SIGNIN_POLICY_ROUTE), "{key}");
        }
    }
}
