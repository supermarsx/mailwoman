//! The purpose consent ledger: the purposes registry, the person's own decisions,
//! runtime enforcement and the admin overview.
//!
//! A purpose has a legal basis the administrator sets per domain
//! (`consent.<purpose>.basis`) and shows to the person, a text version
//! (`consent.<purpose>.version`; raising it asks again) and an optional text of
//! the administrator's own (`consent.<purpose>.text`). Decisions are appended to
//! `consent_record`; the newest row of an account is its current decision.
//!
//! Enforcement. `allowed` is the one runtime question. A purpose whose basis is
//! `consent` is allowed only after a `granted` row of the current version;
//! withdrawing is a `withdrawn` row and stops the processing on the next call.
//! For every other basis the purpose is allowed unless the person withdrew it,
//! which is how the per-feature assistant opt-out works. Purposes marked
//! `preference` keep their state in an existing preference (analytics); the
//! ledger holds the receipts, written when that preference changes.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx},
    admin_router::AdminRegistrar,
    capabilities::Capability as C,
    settings_registry,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderName, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use mw_store::{Store, privacy::ConsentRecord};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};

pub(crate) const BASES: [&str; 4] = ["consent", "legitimateInterest", "contract", "legalObligation"];

/// How a purpose is enforced.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Enforce {
    /// Checked from the ledger at runtime.
    Ledger,
    /// The state is an existing preference; the ledger holds the receipts.
    Preference,
    /// Managed in its own setting; listed so the person sees it.
    Listed,
}

pub(crate) struct Purpose {
    pub id: &'static str,
    pub enforce: Enforce,
    /// The page where a `Preference` or `Listed` purpose is switched.
    pub managed_in: &'static str,
}

pub(crate) const PURPOSES: [Purpose; 8] = [
    Purpose { id: "analytics", enforce: Enforce::Preference, managed_in: "privacy" },
    Purpose { id: "analyticsTag", enforce: Enforce::Preference, managed_in: "privacy" },
    Purpose { id: "assistant", enforce: Enforce::Ledger, managed_in: "" },
    Purpose { id: "push", enforce: Enforce::Ledger, managed_in: "" },
    Purpose { id: "openTracking", enforce: Enforce::Ledger, managed_in: "" },
    Purpose { id: "remoteContent", enforce: Enforce::Listed, managed_in: "mail" },
    Purpose { id: "integrations", enforce: Enforce::Ledger, managed_in: "" },
    Purpose { id: "telemetry", enforce: Enforce::Listed, managed_in: "privacy" },
];

/// The assistant features a person can opt out of one by one. The id of the
/// purpose is `assistant.<feature>`.
pub(crate) fn assistant_features() -> Vec<String> {
    mw_assist::AssistCapability::ALL
        .iter()
        .map(|c| {
            serde_json::to_value(c)
                .ok()
                .and_then(|v| v.as_str().map(str::to_string))
                .unwrap_or_default()
        })
        .collect()
}

fn purpose(id: &str) -> Option<&'static Purpose> {
    let base = id.split_once('.').map_or(id, |(b, _)| b);
    PURPOSES.iter().find(|p| p.id == base)
}

/// Whether `id` names a purpose or an assistant feature.
pub(crate) fn valid_purpose(id: &str) -> bool {
    match id.split_once('.') {
        None => purpose(id).is_some(),
        Some(("assistant", f)) => assistant_features().iter().any(|x| x == f),
        Some(_) => false,
    }
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

/// The administrator's settings for one purpose.
#[derive(Clone, Debug)]
pub(crate) struct Cfg {
    pub basis: String,
    pub version: i64,
    pub text: Value,
}

pub(crate) struct Config {
    pub enabled: bool,
    pub art22: bool,
    pub purposes: HashMap<&'static str, Cfg>,
}

impl Config {
    pub(crate) async fn load(store: &Store, s: &mw_store::settings_policy::SettingSubject<'_>) -> Self {
        let rows = settings_registry::effective_all(store, s, |d| d.key.starts_with("consent."))
            .await
            .unwrap_or_default();
        let get = |k: &str| rows.iter().find(|e| e.key == k).map(|e| e.value.clone());
        let mut purposes = HashMap::new();
        for p in &PURPOSES {
            purposes.insert(
                p.id,
                Cfg {
                    basis: get(&format!("consent.{}.basis", p.id))
                        .and_then(|v| v.as_str().map(str::to_string))
                        .unwrap_or_else(|| "consent".into()),
                    version: get(&format!("consent.{}.version", p.id))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(1),
                    text: get(&format!("consent.{}.text", p.id)).unwrap_or(json!({})),
                },
            );
        }
        Self {
            enabled: get("consent.enabled").and_then(|v| v.as_bool()).unwrap_or(true),
            art22: get("consent.art22.notice").and_then(|v| v.as_bool()).unwrap_or(true),
            purposes,
        }
    }
}

/// The state of one purpose for a person.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Decision {
    Granted,
    Withdrawn,
    /// No decision yet.
    Unset,
    /// Granted for an older text version; asks again.
    Stale,
}

impl Decision {
    fn name(self) -> &'static str {
        match self {
            Decision::Granted => "granted",
            Decision::Withdrawn => "withdrawn",
            Decision::Unset => "unset",
            Decision::Stale => "stale",
        }
    }
}

fn state_of(latest: Option<&ConsentRecord>, version: i64) -> Decision {
    match latest {
        None => Decision::Unset,
        Some(r) if r.decision == "withdrawn" => Decision::Withdrawn,
        Some(r) if r.decision == "granted" && r.version >= version => Decision::Granted,
        Some(r) if r.decision == "granted" => Decision::Stale,
        Some(_) => Decision::Unset,
    }
}

/// The runtime question: may `purpose` be processed for this account now?
///
/// `purpose` is a purpose id or `assistant.<feature>`. A feature id needs the
/// umbrella purpose and its own opt-out to allow it. A ledger fault allows
/// processing for non-consent bases and refuses it for consent bases.
pub(crate) async fn allowed(store: &Store, account_id: &str, purpose_id: &str) -> bool {
    let base = purpose_id.split_once('.').map_or(purpose_id, |(b, _)| b);
    let Some(p) = purpose(base) else { return true };
    if p.enforce != Enforce::Ledger {
        return true;
    }
    let cfg = match crate::notification_policy::account_subject(store, account_id).await {
        Ok(Some(subject)) => Config::load(store, &subject.as_subject()).await,
        // A session account without a row (proxy mode) still gets the server-wide policy.
        Ok(None) => {
            let s = mw_store::settings_policy::SettingSubject {
                personal_owner: None,
                account_id,
                address: "",
                domain: "",
            };
            Config::load(store, &s).await
        }
        Err(_) => return true,
    };
    if !cfg.enabled {
        return true;
    }
    let c = &cfg.purposes[p.id];
    // The more specific decision first: a feature the person switched off.
    if base != purpose_id {
        if let Ok(Some(r)) = store.consent_latest(account_id, purpose_id).await {
            if r.decision == "withdrawn" {
                return false;
            }
        }
    }
    let latest = store.consent_latest(account_id, base).await;
    match latest {
        Ok(l) => match state_of(l.as_ref(), c.version) {
            Decision::Withdrawn => false,
            Decision::Granted => true,
            Decision::Unset | Decision::Stale => c.basis != "consent",
        },
        Err(_) => c.basis != "consent",
    }
}

static LAST_PURGE: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);

/// Delete receipts past `consent.receipts.retentionDays` (0 keeps them), at most
/// once an hour per process, on the write path like the cookie consent log.
async fn maybe_purge(store: &Store) {
    use std::sync::atomic::Ordering;
    let t = now();
    let last = LAST_PURGE.load(Ordering::Relaxed);
    if t - last < 3600 || LAST_PURGE.compare_exchange(last, t, Ordering::Relaxed, Ordering::Relaxed).is_err() {
        return;
    }
    let days = settings_registry::effective_for(store, "consent.receipts.retentionDays", &Default::default())
        .await
        .ok()
        .flatten()
        .and_then(|v| v.value.as_i64())
        .unwrap_or(1825);
    if days > 0 {
        let _ = store.consent_purge(t - days * 86_400).await;
    }
}

/// A row id that sorts in insertion order: `created_at` has one-second
/// resolution, so two decisions in the same second are ordered by this id.
fn ledger_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    format!("{nanos:021}-{}", uuid::Uuid::new_v4())
}

/// Append a decision to the ledger.
pub(crate) async fn record(
    store: &Store,
    account_id: &str,
    domain: &str,
    purpose_id: &str,
    decision: &str,
    version: i64,
    basis: &str,
    source: &str,
) -> Result<(), mw_store::StoreError> {
    maybe_purge(store).await;
    store
        .consent_append(&ConsentRecord {
            id: ledger_id(),
            account_id: account_id.to_string(),
            domain: domain.to_string(),
            purpose: purpose_id.to_string(),
            decision: decision.to_string(),
            version,
            basis: basis.to_string(),
            source: source.to_string(),
            created_at: now(),
        })
        .await
}

/// The text a person sees for a purpose: the administrator's own text in their
/// language, else `None` (the client shows the built-in wording).
fn text_for(cfg: &Cfg, locale: &str) -> Option<String> {
    let map = cfg.text.as_object()?;
    map.get(locale)
        .or_else(|| map.get("en"))
        .or_else(|| map.values().next())
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .filter(|s| !s.is_empty())
}

// ─── the person's routes ──────────────────────────────────────────────────────

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/privacy/consents", get(own_list))
        .route("/api/privacy/consents/{purpose}", post(own_decide))
}

/// The state of every purpose for the signed-in person, as JSON.
pub(crate) async fn own_json(state: &AppState, session: &mw_store::Session, locale: &str) -> Result<Value, mw_store::StoreError> {
    let subject = crate::settings_routes::UserSubject::resolved(&state.store, session).await?;
    let cfg = Config::load(&state.store, &subject.as_subject()).await;
    let account = &subject.account_id;
    let analytics_owner = analytics_owner_state(state, session).await;
    let mut out = Vec::new();
    for p in &PURPOSES {
        let c = &cfg.purposes[p.id];
        let latest = state.store.consent_latest(account, p.id).await?;
        let st: Decision = match p.enforce {
            Enforce::Preference => match (p.id, &analytics_owner) {
                ("analytics", Some((counts, _))) => if *counts { Decision::Granted } else if latest.as_ref().is_some_and(|r| r.decision == "withdrawn") { Decision::Withdrawn } else { Decision::Unset },
                ("analyticsTag", Some((_, tag))) => if *tag { Decision::Granted } else if latest.as_ref().is_some_and(|r| r.decision == "withdrawn") { Decision::Withdrawn } else { Decision::Unset },
                _ => Decision::Unset,
            },
            Enforce::Listed => Decision::Unset,
            Enforce::Ledger => state_of(latest.as_ref(), c.version),
        };
        out.push(json!({
            "id": p.id,
            "basis": c.basis,
            "version": c.version,
            "text": text_for(c, locale),
            "state": st.name(),
            "enforcement": match p.enforce { Enforce::Ledger => "ledger", Enforce::Preference => "preference", Enforce::Listed => "listed" },
            "managedIn": p.managed_in,
            "decidedAt": latest.as_ref().map(|r| r.created_at),
            "decidedVersion": latest.as_ref().map(|r| r.version),
            "canToggle": p.enforce == Enforce::Ledger,
            "defaultOn": c.basis != "consent",
        }));
    }
    let mut features = Vec::new();
    for f in assistant_features() {
        let id = format!("assistant.{f}");
        let latest = state.store.consent_latest(account, &id).await?;
        features.push(json!({
            "id": id,
            "feature": f,
            "enabled": !latest.as_ref().is_some_and(|r| r.decision == "withdrawn"),
            "decidedAt": latest.as_ref().map(|r| r.created_at),
        }));
    }
    let receipts: Vec<Value> = state
        .store
        .consent_receipts(account)
        .await?
        .into_iter()
        .rev()
        .take(200)
        .map(|r| json!({"id": r.id, "purpose": r.purpose, "decision": r.decision, "version": r.version, "basis": r.basis, "source": r.source, "createdAt": r.created_at}))
        .collect();
    Ok(json!({
        "enabled": cfg.enabled,
        "art22": {"enabled": cfg.art22, "humanReview": true},
        "purposes": out,
        "assistantFeatures": features,
        "receipts": receipts,
    }))
}

/// `(counts consent, tag consent)` from the analytics preference document.
async fn analytics_owner_state(state: &AppState, session: &mw_store::Session) -> Option<(bool, bool)> {
    let owner = crate::prefs_routes::personal_storage::owner(state, session, &HeaderMap::new(), false)
        .await
        .ok()?;
    let counts = crate::analytics_runtime::consent(&state.store, &owner).await;
    let raw = state
        .store
        .get_setting(&crate::prefs_routes::personal_storage::key("ui-preferences", &owner))
        .await
        .ok()
        .flatten()?;
    let v: Value = serde_json::from_str(&raw).ok()?;
    let tag = v["document"]["preferences"]["analyticsTagConsent"]
        .as_str()
        .is_some_and(|s| !s.is_empty());
    Some((counts, tag))
}

fn locale_of(headers: &HeaderMap) -> String {
    headers
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split([',', ';', '-']).next())
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty() && s.len() <= 8)
        .unwrap_or_else(|| "en".into())
}

async fn own_list(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    match own_json(&state, &session, &locale_of(&headers)).await {
        Ok(v) => ([(header::CACHE_CONTROL, "no-store")], Json(v)).into_response(),
        Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Decide {
    decision: String,
}

async fn own_decide(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(purpose_id): Path<String>,
    Json(req): Json<Decide>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    let err = |status: StatusCode, code: &str| (status, Json(json!({"code": code}))).into_response();
    if !valid_purpose(&purpose_id) {
        return err(StatusCode::NOT_FOUND, "unknownPurpose");
    }
    if !matches!(req.decision.as_str(), "granted" | "withdrawn") {
        return err(StatusCode::BAD_REQUEST, "invalidDecision");
    }
    let base = purpose_id.split_once('.').map_or(purpose_id.as_str(), |(b, _)| b);
    if purpose(base).is_none_or(|p| p.enforce != Enforce::Ledger) {
        return err(StatusCode::CONFLICT, "managedElsewhere");
    }
    let Ok(subject) = crate::settings_routes::UserSubject::resolved(&state.store, &session).await else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    let cfg = Config::load(&state.store, &subject.as_subject()).await;
    if !cfg.enabled {
        return err(StatusCode::FORBIDDEN, "consentDisabled");
    }
    let c = &cfg.purposes[base];
    // Every decision is as easy to reverse as to give: no precondition on the
    // previous state, and a withdrawal always succeeds.
    if let Err(e) = record(&state.store, &subject.account_id, &subject.domain, &purpose_id, &req.decision, c.version, &c.basis, "settings").await {
        tracing::warn!("consent record failed: {e}");
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    match own_json(&state, &session, &locale_of(&headers)).await {
        Ok(v) => ([(header::CACHE_CONTROL, "no-store")], Json(v)).into_response(),
        Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}

/// Mirror a change of the analytics preferences into the ledger as receipts.
/// Best effort: a ledger fault never blocks saving the preference.
pub(crate) async fn mirror_analytics(
    state: &AppState,
    session: &mw_store::Session,
    before: (bool, bool),
    after: (bool, bool),
) {
    let Ok(subject) = crate::settings_routes::UserSubject::resolved(&state.store, session).await else { return };
    let cfg = Config::load(&state.store, &subject.as_subject()).await;
    for (id, was, now_on) in [("analytics", before.0, after.0), ("analyticsTag", before.1, after.1)] {
        if was == now_on {
            continue;
        }
        let c = &cfg.purposes[id];
        let decision = if now_on { "granted" } else { "withdrawn" };
        if let Err(e) = record(&state.store, &subject.account_id, &subject.domain, id, decision, c.version, &c.basis, "settings").await {
            tracing::warn!("consent receipt failed: {e}");
        }
    }
}

// ─── admin ────────────────────────────────────────────────────────────────────

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, "/consents/stats", C::SettingsRead, stats);
    r.admin_endpoint(Method::GET, "/consents/ledger.csv", C::SettingsRead, export_csv);
}

#[derive(Deserialize, Default)]
struct Range {
    from: Option<i64>,
    to: Option<i64>,
}

fn bounds(r: &Range) -> Result<(i64, i64), AdminApiError> {
    let to = r.to.unwrap_or_else(|| now() + 1);
    let from = r.from.unwrap_or(to - 30 * 86_400);
    if from >= to || to - from > 366 * 86_400 {
        return Err(AdminApiError::invalid("invalid", "The range must be 1 second to 366 days.", "/from"));
    }
    Ok((from, to))
}

/// Per purpose: people whose current decision is granted / withdrawn, the
/// withdrawal rate, and a daily series of decisions in the range.
pub(crate) fn summarize(rows: &[ConsentRecord], from: i64, to: i64) -> Value {
    let mut latest: BTreeMap<(&str, &str), &ConsentRecord> = BTreeMap::new();
    for r in rows {
        let slot = latest.entry((r.account_id.as_str(), r.purpose.as_str())).or_insert(r);
        if (r.created_at, &r.id) >= (slot.created_at, &slot.id) {
            *slot = r;
        }
    }
    let mut per: BTreeMap<String, (i64, i64, i64, i64)> = BTreeMap::new();
    for r in rows {
        let e = per.entry(r.purpose.clone()).or_default();
        match r.decision.as_str() {
            "granted" => e.2 += 1,
            "withdrawn" => e.3 += 1,
            _ => {}
        }
    }
    for ((_, p), r) in &latest {
        let e = per.entry((*p).to_string()).or_default();
        match r.decision.as_str() {
            "granted" => e.0 += 1,
            "withdrawn" => e.1 += 1,
            _ => {}
        }
    }
    let mut days: BTreeMap<i64, (i64, i64)> = BTreeMap::new();
    let mut d = from.div_euclid(86_400) * 86_400;
    while d < to && days.len() < 400 {
        days.insert(d, (0, 0));
        d += 86_400;
    }
    for r in rows {
        if let Some(day) = days.get_mut(&(r.created_at.div_euclid(86_400) * 86_400)) {
            match r.decision.as_str() {
                "granted" => day.0 += 1,
                "withdrawn" => day.1 += 1,
                _ => {}
            }
        }
    }
    let purposes: Vec<Value> = per
        .iter()
        .map(|(id, (granted, withdrawn, g_events, w_events))| {
            let total = granted + withdrawn;
            json!({
                "purpose": id, "granted": granted, "withdrawn": withdrawn,
                "withdrawalRate": if total == 0 { 0.0 } else { *withdrawn as f64 / total as f64 },
                "grantEvents": g_events, "withdrawEvents": w_events,
            })
        })
        .collect();
    json!({
        "from": from, "to": to, "total": rows.len(),
        "purposes": purposes,
        "series": days.iter().map(|(t, (g, w))| json!({"t": t, "granted": g, "withdrawn": w})).collect::<Vec<_>>(),
    })
}

async fn stats(State(state): State<AppState>, ctx: AdminCtx, Query(range): Query<Range>) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsRead)?;
    let (from, to) = bounds(&range)?;
    // The current decision of a person can predate the range, so read from the
    // start of the ledger for the "now" counts and the range for the series.
    let all = state.store.consent_between(0, to).await.map_err(AdminApiError::internal)?;
    let mut body = summarize(&all, from, to);
    let in_range: Vec<ConsentRecord> = all.iter().filter(|r| r.created_at >= from).cloned().collect();
    let ranged = summarize(&in_range, from, to);
    body["series"] = ranged["series"].clone();
    body["total"] = json!(in_range.len());
    Ok(([(header::CACHE_CONTROL, "no-store")], Json(body)).into_response())
}

pub(crate) fn to_csv(rows: &[ConsentRecord]) -> String {
    let mut out = String::from("time,domain,purpose,decision,version,basis,source,accountId\r\n");
    for r in rows {
        let time = chrono::DateTime::from_timestamp(r.created_at, 0)
            .map(|t| t.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
            .unwrap_or_default();
        let f = [
            crate::cookie_consent::csv_field(&time),
            crate::cookie_consent::csv_field(&r.domain),
            crate::cookie_consent::csv_field(&r.purpose),
            crate::cookie_consent::csv_field(&r.decision),
            r.version.to_string(),
            crate::cookie_consent::csv_field(&r.basis),
            crate::cookie_consent::csv_field(&r.source),
            crate::cookie_consent::csv_field(&r.account_id),
        ];
        out.push_str(&f.join(","));
        out.push_str("\r\n");
    }
    out
}

async fn export_csv(State(state): State<AppState>, ctx: AdminCtx, Query(range): Query<Range>) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsRead)?;
    let (from, to) = bounds(&range)?;
    let rows = state.store.consent_between(from, to).await.map_err(AdminApiError::internal)?;
    crate::admin_accounts::audit(&state, &ctx, "consent-ledger-exported", "consent:ledger", None, Some(json!({"rows": rows.len(), "from": from, "to": to}))).await;
    Ok((
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8".to_owned()),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"consent-ledger.csv\"".to_owned()),
            (header::CACHE_CONTROL, "no-store".to_owned()),
            (HeaderName::from_static("x-rows"), rows.len().to_string()),
        ],
        to_csv(&rows),
    )
        .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, settings_policy::SettingPolicyRow};

    async fn store_with_account() -> (Store, String) {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let id = store
            .create_account(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host: "imap.example.test",
                    port: 993,
                    tls: "tls",
                    username: "ana@example.test",
                    sync_policy_json: "{}",
                },
                &Credentials { username: "ana@example.test".into(), password: "x".into() },
            )
            .await
            .unwrap();
        (store, id)
    }

    async fn set(store: &Store, key: &str, value: Value) {
        store
            .put_setting_policy(&SettingPolicyRow {
                key: key.into(),
                global_value: Some(value.to_string()),
                allow_user_override: false,
                global_locked: false,
                updated_by: "test".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
    }

    fn restriction(account: &str, scopes: &[&str]) -> mw_store::privacy::RestrictionRow {
        mw_store::privacy::RestrictionRow {
            id: uuid::Uuid::new_v4().to_string(),
            account_id: account.into(),
            address: "ana@example.test".into(),
            domain: "example.test".into(),
            scopes: scopes.iter().map(|s| (*s).to_string()).collect(),
            mail_delivery: "keep".into(),
            reason: "test".into(),
            source: "admin".into(),
            request_id: None,
            created_by: "root".into(),
            created_at: now() - 5,
            expires_at: None,
            lifted_at: None,
            lifted_by: None,
            lift_reason: None,
        }
    }

    #[tokio::test]
    async fn non_consent_basis_runs_until_withdrawn_and_consent_basis_waits_for_a_yes() {
        let (store, acct) = store_with_account().await;
        // The shipped default for push is a non-consent basis: allowed with no decision.
        assert!(allowed(&store, &acct, "push").await);
        record(&store, &acct, "example.test", "push", "withdrawn", 1, "contract", "settings").await.unwrap();
        assert!(!allowed(&store, &acct, "push").await, "a withdrawal always holds");
        record(&store, &acct, "example.test", "push", "granted", 1, "contract", "settings").await.unwrap();
        assert!(allowed(&store, &acct, "push").await);

        set(&store, "consent.push.basis", json!("consent")).await;
        set(&store, "consent.push.version", json!(2)).await;
        assert!(!allowed(&store, &acct, "push").await, "a yes for version 1 does not cover version 2");
        record(&store, &acct, "example.test", "push", "granted", 2, "consent", "settings").await.unwrap();
        assert!(allowed(&store, &acct, "push").await);
    }

    #[tokio::test]
    async fn the_push_gate_honours_restriction_and_consent() {
        let (store, acct) = store_with_account().await;
        assert!(crate::push_relay::processing_allowed(&store, &acct).await);
        store.restriction_apply(&restriction(&acct, &["push"])).await.unwrap();
        assert!(!crate::push_relay::processing_allowed(&store, &acct).await, "restricted push sends no wake");
        store.restriction_lift(&acct, "root", "done", now()).await.unwrap();
        assert!(crate::push_relay::processing_allowed(&store, &acct).await);
        record(&store, &acct, "example.test", "push", "withdrawn", 1, "contract", "settings").await.unwrap();
        assert!(!crate::push_relay::processing_allowed(&store, &acct).await, "withdrawn consent sends no wake");
    }

    #[tokio::test]
    async fn the_open_tracking_gate_honours_restriction_and_consent() {
        let (store, acct) = store_with_account().await;
        assert!(!crate::open_tracking::paused(&store, &acct).await);
        store.restriction_apply(&restriction(&acct, &["analytics"])).await.unwrap();
        assert!(crate::open_tracking::paused(&store, &acct).await);
        store.restriction_lift(&acct, "root", "done", now()).await.unwrap();
        assert!(!crate::open_tracking::paused(&store, &acct).await);
        set(&store, "consent.openTracking.basis", json!("consent")).await;
        assert!(crate::open_tracking::paused(&store, &acct).await, "consent basis: off until a yes");
        record(&store, &acct, "example.test", "openTracking", "granted", 1, "consent", "settings").await.unwrap();
        assert!(!crate::open_tracking::paused(&store, &acct).await);
        record(&store, &acct, "example.test", "openTracking", "withdrawn", 1, "consent", "settings").await.unwrap();
        assert!(crate::open_tracking::paused(&store, &acct).await);
    }

    #[tokio::test]
    async fn the_feature_gate_maps_restriction_and_consent_for_assistant_and_integrations() {
        // The processing map used by require_feature.
        let (store, acct) = store_with_account().await;
        store.restriction_apply(&restriction(&acct, &["assistant"])).await.unwrap();
        assert!(store.restriction_blocks(&acct, "assistant").await);
        assert!(!store.restriction_blocks(&acct, "thirdParty").await);
        assert!(valid_purpose("assistant.summarize"));
        assert!(!valid_purpose("assistant.nope"));
        assert!(!valid_purpose("push.x"));
    }

    #[test]
    fn summarize_counts_current_decisions_and_withdrawal_rate() {
        let rec = |id: &str, acct: &str, purpose: &str, decision: &str, at: i64| ConsentRecord {
            id: id.into(), account_id: acct.into(), domain: "d".into(), purpose: purpose.into(),
            decision: decision.into(), version: 1, basis: "consent".into(), source: "settings".into(), created_at: at,
        };
        let rows = vec![
            rec("1", "a", "assistant", "granted", 100),
            rec("2", "a", "assistant", "withdrawn", 200),
            rec("3", "b", "assistant", "granted", 150),
            rec("4", "c", "push", "granted", 160),
        ];
        let s = summarize(&rows, 0, 86_400);
        let p = |id: &str| s["purposes"].as_array().unwrap().iter().find(|r| r["purpose"] == id).cloned().unwrap();
        assert_eq!(p("assistant")["granted"], 1);
        assert_eq!(p("assistant")["withdrawn"], 1);
        assert_eq!(p("assistant")["withdrawalRate"], 0.5);
        assert_eq!(p("push")["granted"], 1);
        assert_eq!(s["series"][0]["granted"], 3);
        assert_eq!(s["series"][0]["withdrawn"], 1);
    }

    #[test]
    fn csv_defuses_formulas() {
        let r = ConsentRecord {
            id: "1".into(), account_id: "=cmd".into(), domain: "d".into(), purpose: "push".into(),
            decision: "granted".into(), version: 1, basis: "consent".into(), source: "settings".into(), created_at: 0,
        };
        assert!(to_csv(&[r]).contains("\"'=cmd\""));
    }
}
