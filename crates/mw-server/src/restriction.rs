//! Art.18 restriction of processing: apply, lift, the admin routes and the
//! person's own status route.
//!
//! A restriction pauses processing categories (scopes) for an account while the
//! data stays stored. Enforcement sits at the places that process the data:
//!
//! * `indexing` - the search index and message embeddings (`mw-engine`)
//! * `assistant` - every AI assistant route (`user_router::require_feature`)
//! * `analytics` - the analytics tag and counts, open tracking, client timing
//! * `classification` - local Sieve rules and notification rules at ingest
//! * `push` - web push, UnifiedPush and APNs wakes
//! * `thirdParty` - Nextcloud and UI plugin features, and sends to other services
//!
//! Mail still arrives and the person can read, search by hand in the mail they
//! hold, and delete. `restriction.mailDelivery` records what the administrator
//! chose for new mail (`keep`, `queue`, `reject`); this server does not receive
//! mail itself, so the value is reported to the person and the upstream mail
//! server is the one that acts on it (see `docs/admin/consent.md`).
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx},
    admin_router::AdminRegistrar,
    audit::{SecurityEvent, security_event},
    capabilities::Capability as C,
    settings_registry,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use mw_store::{
    privacy::{MAIL_DELIVERY, RESTRICTION_SCOPES, RestrictionRow},
    settings_policy::SettingSubject,
};
use serde::Deserialize;
use serde_json::{Value, json};

const DAY: i64 = 86_400;

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

/// The effective `restriction.*` values for one person.
#[derive(Clone, Debug)]
pub(crate) struct Policy {
    pub enabled: bool,
    pub auto_apply: bool,
    pub mail_delivery: String,
    pub default_scopes: Vec<String>,
    pub expiry_days: i64,
    pub notify_user: bool,
}

impl Policy {
    pub(crate) async fn load(state: &AppState, domain: &str, address: &str, account_id: &str) -> Self {
        let subject = SettingSubject {
            personal_owner: None,
            domain,
            account_id,
            address,
        };
        let rows = settings_registry::effective_all(&state.store, &subject, |d| {
            d.key.starts_with("restriction.")
        })
        .await
        .unwrap_or_default();
        let get = |k: &str| rows.iter().find(|e| e.key == k).map(|e| e.value.clone());
        let flag = |k: &str, d: bool| get(k).and_then(|v| v.as_bool()).unwrap_or(d);
        let default_scopes = RESTRICTION_SCOPES
            .iter()
            .filter(|s| flag(&format!("restriction.scope.{s}"), true))
            .map(|s| (*s).to_string())
            .collect();
        Self {
            enabled: flag("restriction.enabled", true),
            auto_apply: flag("restriction.autoApplyOnApproval", true),
            mail_delivery: get("restriction.mailDelivery")
                .and_then(|v| v.as_str().map(str::to_string))
                .unwrap_or_else(|| "keep".into()),
            default_scopes,
            expiry_days: get("restriction.defaultExpiryDays")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            notify_user: flag("restriction.notifyUser", true),
        }
    }
}

/// What to apply.
pub(crate) struct Apply<'a> {
    pub address: &'a str,
    pub domain: &'a str,
    pub account_ids: Vec<String>,
    pub scopes: Option<Vec<String>>,
    pub reason: &'a str,
    pub source: &'a str,
    pub request_id: Option<String>,
    pub by: &'a str,
    pub expires_in_days: Option<i64>,
    pub mail_delivery: Option<String>,
    pub locale: &'a str,
}

/// Restrict every account id of a person. Returns the rows written.
pub(crate) async fn apply(
    state: &AppState,
    policy: &Policy,
    a: Apply<'_>,
) -> Result<Vec<RestrictionRow>, mw_store::StoreError> {
    let scopes: Vec<String> = a
        .scopes
        .unwrap_or_else(|| policy.default_scopes.clone())
        .into_iter()
        .filter(|s| RESTRICTION_SCOPES.contains(&s.as_str()))
        .collect();
    let delivery = a
        .mail_delivery
        .filter(|d| MAIL_DELIVERY.contains(&d.as_str()))
        .unwrap_or_else(|| policy.mail_delivery.clone());
    let days = a.expires_in_days.unwrap_or(policy.expiry_days);
    let at = now();
    let mut out = Vec::new();
    for account_id in &a.account_ids {
        let row = RestrictionRow {
            id: uuid::Uuid::new_v4().to_string(),
            account_id: account_id.clone(),
            address: a.address.to_string(),
            domain: a.domain.to_string(),
            scopes: scopes.clone(),
            mail_delivery: delivery.clone(),
            reason: a.reason.chars().take(500).collect(),
            source: a.source.to_string(),
            request_id: a.request_id.clone(),
            created_by: a.by.to_string(),
            created_at: at,
            expires_at: (days > 0).then(|| at + days * DAY),
            lifted_at: None,
            lifted_by: None,
            lift_reason: None,
        };
        state.store.restriction_apply(&row).await?;
        security_event(
            state,
            SecurityEvent::admin(a.by, "restriction-applied")
                .target(format!("account:{account_id}"))
                .detail(json!({"scopes": row.scopes, "source": row.source, "requestId": row.request_id, "expiresAt": row.expires_at, "mailDelivery": row.mail_delivery})),
        )
        .await;
        out.push(row);
    }
    if policy.notify_user {
        if let Some(first) = out.first() {
            let text = format!(
                "Processing of your account data has been restricted. We keep your data, but pause: {}. You can still read and delete your mail.{}",
                first.scopes.join(", "),
                first.expires_at.map(|e| format!(" The restriction ends on {}.", chrono::DateTime::from_timestamp(e, 0).map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default())).unwrap_or_default()
            );
            notify(state, &first.account_id, a.locale, "Processing of your data is restricted", &text);
        }
    }
    Ok(out)
}

/// Lift the restriction of every account id of a person, with a notice.
pub(crate) async fn lift(
    state: &AppState,
    policy: &Policy,
    account_ids: &[String],
    by: &str,
    reason: &str,
    locale: &str,
) -> Result<Vec<RestrictionRow>, mw_store::StoreError> {
    let at = now();
    let mut out = Vec::new();
    for id in account_ids {
        if let Some(row) = state
            .store
            .restriction_lift(id, by, &reason.chars().take(500).collect::<String>(), at)
            .await?
        {
            security_event(
                state,
                SecurityEvent::admin(by, "restriction-lifted")
                    .target(format!("account:{id}"))
                    .detail(json!({"restrictionId": row.id, "reason": reason})),
            )
            .await;
            out.push(row);
        }
    }
    if policy.notify_user {
        if let Some(first) = out.first() {
            notify(
                state,
                &first.account_id,
                locale,
                "Processing of your data is no longer restricted",
                "The restriction on your account data was lifted. The paused features work again.",
            );
        }
    }
    Ok(out)
}

fn notify(state: &AppState, account: &str, locale: &str, subject: &str, description: &str) {
    let (state, account) = (state.clone(), account.to_string());
    let locale = if locale.is_empty() { "en".to_string() } else { locale.to_string() };
    let (subject, description) = (subject.to_string(), description.to_string());
    tokio::spawn(async move {
        if let Err(e) = crate::alerts::deliver_text(&state, &account, &locale, &subject, &description, "").await {
            tracing::warn!("restriction notice failed: {e}");
        }
    });
}

/// Whether `scope` is paused for the account. The one call the enforcement
/// points use.
pub(crate) async fn blocks(store: &mw_store::Store, account_id: &str, scope: &str) -> bool {
    store.restriction_blocks(account_id, scope).await
}

fn view(r: &RestrictionRow) -> Value {
    let at = now();
    json!({
        "id": r.id, "accountId": r.account_id, "address": r.address, "domain": r.domain,
        "scopes": r.scopes, "mailDelivery": r.mail_delivery, "reason": r.reason,
        "source": r.source, "requestId": r.request_id, "createdBy": r.created_by,
        "createdAt": r.created_at, "expiresAt": r.expires_at, "liftedAt": r.lifted_at,
        "liftedBy": r.lifted_by, "liftReason": r.lift_reason, "active": r.active_at(at),
    })
}

// ─── the person's own status ──────────────────────────────────────────────────

pub(crate) fn router() -> Router<AppState> {
    Router::new().route("/api/privacy/restriction", get(own_status))
}

/// What the person sees: whether processing is paused, what is paused, since
/// when and until when. The administrator's reason is not shown.
pub(crate) async fn status_json(state: &AppState, account_id: &str) -> Value {
    match state.store.restriction_active(account_id, now()).await {
        Ok(Some(r)) => json!({
            "active": true, "scopes": r.scopes, "since": r.created_at, "expiresAt": r.expires_at,
            "mailDelivery": r.mail_delivery, "source": r.source,
        }),
        _ => json!({"active": false, "scopes": [], "since": null, "expiresAt": null}),
    }
}

async fn own_status(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    (
        [(header::CACHE_CONTROL, "no-store")],
        Json(status_json(&state, &session.account_id).await),
    )
        .into_response()
}

// ─── admin ────────────────────────────────────────────────────────────────────

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, "/restrictions", C::DataExportRun, admin_list);
    r.admin_endpoint(Method::POST, "/restrictions", C::DataExportRun, admin_apply);
    r.admin_endpoint(Method::POST, "/restrictions/lift", C::DataExportRun, admin_lift);
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ListQuery {
    domain: Option<String>,
    all: Option<bool>,
}

async fn admin_list(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Query(q): Query<ListQuery>,
) -> Result<Response, AdminApiError> {
    if let Some(d) = &q.domain {
        ctx.require_domain(d)?;
    }
    let rows = state
        .store
        .restriction_list(q.domain.as_deref(), !q.all.unwrap_or(false), now())
        .await?;
    let rows: Vec<Value> = rows
        .iter()
        .filter(|r| ctx.require_domain(&r.domain).is_ok())
        .map(view)
        .collect();
    Ok((
        [(header::CACHE_CONTROL, "no-store")],
        Json(json!({"restrictions": rows, "scopes": RESTRICTION_SCOPES, "mailDelivery": MAIL_DELIVERY})),
    )
        .into_response())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ApplyBody {
    address: String,
    #[serde(default)]
    scopes: Option<Vec<String>>,
    reason: String,
    #[serde(default)]
    expires_in_days: Option<i64>,
    #[serde(default)]
    mail_delivery: Option<String>,
}

async fn admin_apply(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Json(req): Json<ApplyBody>,
) -> Result<Response, AdminApiError> {
    let address = state.store.canonical_policy_address(&req.address).await?;
    let domain = crate::admin_ctx::domain_of(&address);
    ctx.require_domain(&domain)?;
    let reason = req.reason.trim().to_string();
    if reason.is_empty() {
        return Err(AdminApiError::invalid("invalid", "A reason is required.", "/reason"));
    }
    if let Some(s) = &req.scopes {
        if s.is_empty() || s.iter().any(|x| !RESTRICTION_SCOPES.contains(&x.as_str())) {
            return Err(AdminApiError::invalid("invalid", "Unknown or empty scope list.", "/scopes"));
        }
    }
    if req.mail_delivery.as_ref().is_some_and(|d| !MAIL_DELIVERY.contains(&d.as_str())) {
        return Err(AdminApiError::invalid("invalid", "Unknown mail delivery mode.", "/mailDelivery"));
    }
    if req.expires_in_days.is_some_and(|d| !(0..=3650).contains(&d)) {
        return Err(AdminApiError::invalid("invalid", "Expiry must be 0 to 3650 days.", "/expiresInDays"));
    }
    let ids = state.store.lifecycle_account_bindings(&address).await?;
    if ids.is_empty() {
        return Err(AdminApiError::not_found("No account of this address has signed in to this server."));
    }
    let policy = Policy::load(&state, &domain, &address, &ids[0]).await;
    if !policy.enabled {
        return Err(AdminApiError::new(StatusCode::FORBIDDEN, "restrictionDisabled", "Restrictions are switched off (restriction.enabled)."));
    }
    let rows = apply(
        &state,
        &policy,
        Apply {
            address: &address,
            domain: &domain,
            account_ids: ids,
            scopes: req.scopes,
            reason: &reason,
            source: "admin",
            request_id: None,
            by: &ctx.username,
            expires_in_days: req.expires_in_days,
            mail_delivery: req.mail_delivery,
            locale: "en",
        },
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(json!({"restrictions": rows.iter().map(view).collect::<Vec<_>>()})),
    )
        .into_response())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LiftBody {
    address: String,
    reason: String,
}

async fn admin_lift(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Json(req): Json<LiftBody>,
) -> Result<Response, AdminApiError> {
    let address = state.store.canonical_policy_address(&req.address).await?;
    let domain = crate::admin_ctx::domain_of(&address);
    ctx.require_domain(&domain)?;
    let reason = req.reason.trim().to_string();
    if reason.is_empty() {
        return Err(AdminApiError::invalid("invalid", "A reason is required.", "/reason"));
    }
    let ids = state.store.lifecycle_account_bindings(&address).await?;
    let policy = Policy::load(&state, &domain, &address, ids.first().map(String::as_str).unwrap_or("")).await;
    let rows = lift(&state, &policy, &ids, &ctx.username, &reason, "en").await?;
    if rows.is_empty() {
        return Err(AdminApiError::not_found("This person has no active restriction."));
    }
    Ok(Json(json!({"restrictions": rows.iter().map(view).collect::<Vec<_>>()})).into_response())
}
