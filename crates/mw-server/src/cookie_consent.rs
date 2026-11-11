//! Cookie consent: the public configuration a browser resolves for its domain and
//! region, consent recording, the storage scan report, and the admin routes for
//! the inventory, classification rules, unclassified storage, statistics and the
//! consent log export.
//!
//! The server is not in the path of the browser's storage. It tells the browser
//! what the administrator configured, keeps the log, and stores names the browser
//! reports. Enforcement (blocking and purging non-consented storage) happens in
//! the web client from this configuration.
//!
//! Privacy. A scan report carries names and kinds only, never values. A consent
//! row holds the time, domain, policy version, decision, category choices, a
//! region code and the privacy signal; the account id is added only when the
//! person was signed in (`POST /api/cookies/consent/account`). No address, IP or
//! user agent is stored. The anonymous routes read no cookie.

use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson},
    admin_router::AdminRegistrar,
    capabilities::Capability as C,
    cookie_inventory::{CATEGORIES, INVENTORY, KINDS, classify, valid_name, valid_pattern},
    domain_config, settings_registry,
};
use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path, Query, State},
    http::{HeaderMap, HeaderName, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use mw_store::cookies::{ConsentRow, CookieRule};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::{
    collections::BTreeMap,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicI64, Ordering},
    },
};

const BODY_CAP: usize = 8 * 1024;
const MAX_SCAN_ITEMS: usize = 100;
const MAX_RULES: usize = 200;
const DECISIONS: [&str; 4] = ["acceptAll", "rejectAll", "custom", "notice"];
const SIGNALS: [&str; 3] = ["none", "gpc", "dnt"];
/// The categories a visitor can switch; `necessary` is always on.
const OPTIONAL: [&str; 3] = ["functional", "analytics", "marketing"];

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/cookies/config", get(public_config))
        .route(
            "/api/cookies/consent",
            post(record_anonymous).layer(DefaultBodyLimit::max(BODY_CAP)),
        )
        .route(
            "/api/cookies/consent/account",
            get(latest_for_account)
                .post(record_for_account)
                .layer(DefaultBodyLimit::max(BODY_CAP)),
        )
        .route(
            "/api/cookies/detected",
            post(report_detected).layer(DefaultBodyLimit::max(BODY_CAP)),
        )
}

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, "/cookies/inventory", C::SettingsRead, inventory);
    r.admin_endpoint(Method::POST, "/cookies/rules", C::SettingsWrite, add_rule);
    r.admin_endpoint(Method::DELETE, "/cookies/rules/{id}", C::SettingsWrite, delete_rule);
    r.admin_endpoint(Method::GET, "/cookies/detected", C::SettingsRead, list_detected);
    r.admin_endpoint(Method::POST, "/cookies/detected", C::SettingsWrite, admin_scan);
    r.admin_endpoint(
        Method::POST,
        "/cookies/detected/{id}/classify",
        C::SettingsWrite,
        classify_detected,
    );
    r.admin_endpoint(
        Method::POST,
        "/cookies/detected/{id}/ignore",
        C::SettingsWrite,
        ignore_detected,
    );
    r.admin_endpoint(
        Method::DELETE,
        "/cookies/detected/{id}",
        C::SettingsWrite,
        delete_detected,
    );
    r.admin_endpoint(Method::GET, "/cookies/stats", C::SettingsRead, stats);
    r.admin_endpoint(Method::GET, "/cookies/consents.csv", C::SettingsRead, export_csv);
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

/// The per-address budget of the anonymous consent routes and the `(burst, per
/// minute)` it was built for (`ratelimit.anon.*`; a change starts a fresh one).
fn limiter() -> &'static Mutex<((u32, u32), crate::scope_mw::rate_limit::KeyedRateLimiter)> {
    static L: OnceLock<Mutex<((u32, u32), crate::scope_mw::rate_limit::KeyedRateLimiter)>> =
        OnceLock::new();
    L.get_or_init(|| {
        Mutex::new((
            (0, 0),
            crate::scope_mw::rate_limit::KeyedRateLimiter::new(1.0, 1.0, 1),
        ))
    })
}

/// Anonymous and per address on purpose: this route has no session. The budget is
/// sized for an office behind one address (a page view is one or two calls), not
/// for a script; the address comes only from the trusted proxy resolver.
async fn allowed(state: &AppState, headers: &HeaderMap, ext: &axum::http::Extensions) -> bool {
    let key = crate::scope_mw::proxy::client_ip(headers, ext)
        .map(|ip| ip.to_canonical().to_string())
        .unwrap_or_else(|| "unknown".into());
    let knobs = crate::rate_leeway::knobs(&state.store, "").await;
    let wanted = (knobs.anon_burst, knobs.anon_per_minute);
    let mut guard = limiter()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if guard.0 != wanted {
        *guard = (
            wanted,
            crate::scope_mw::rate_limit::KeyedRateLimiter::new(
                f64::from(wanted.0),
                f64::from(wanted.1) / 60.0,
                20_000,
            ),
        );
    }
    let ok = guard.1.check(&key);
    drop(guard);
    crate::rate_leeway::note(
        "cookieConsent",
        if ok {
            crate::rate_leeway::Outcome::Allowed
        } else {
            crate::rate_leeway::Outcome::Rejected
        },
    );
    ok
}

/// The `429` of a spent consent budget, with the wait until one call is allowed again.
async fn too_many(state: &AppState) -> Response {
    let knobs = crate::rate_leeway::knobs(&state.store, "").await;
    let wait = (60 / u64::from(knobs.anon_per_minute.max(1))).clamp(1, 60);
    crate::rate_leeway::tag(
        (StatusCode::TOO_MANY_REQUESTS, [(header::RETRY_AFTER, wait.to_string())]).into_response(),
        crate::rate_leeway::Refused {
            class: "cookieConsent",
            reason: "addressBudget",
            key_type: "address",
            limit: Some(knobs.anon_burst),
            retry_after_secs: wait,
        },
    )
}

// ---------------------------------------------------------------------------
// Settings and region
// ---------------------------------------------------------------------------

type Values = BTreeMap<String, Value>;

async fn values(state: &AppState, domain: &str) -> Result<Values, ()> {
    let subject = mw_store::settings_policy::SettingSubject {
        personal_owner: None,
        domain,
        address: "",
        account_id: "",
    };
    settings_registry::effective_all(&state.store, &subject, |d| d.key.starts_with("cookies."))
        .await
        .map(|all| all.into_iter().map(|v| (v.key, v.value)).collect())
        .map_err(|_| ())
}

fn flag(v: &Values, key: &str) -> bool {
    v.get(key).and_then(Value::as_bool).unwrap_or(false)
}
fn text<'a>(v: &'a Values, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}
fn int(v: &Values, key: &str) -> i64 {
    v.get(key).and_then(Value::as_i64).unwrap_or(0)
}

/// The visitor country: the configured header (set by a trusted proxy), else the
/// region subtag of the first `Accept-Language` entry. Upper-case two letters.
pub(crate) fn region_of(headers: &HeaderMap, header_name: &str) -> Option<String> {
    let valid = |s: &str| {
        let s = s.trim().to_ascii_uppercase();
        // `XX` is "unknown" and `T1` is Tor in CDN country headers.
        (s.len() == 2
            && s.chars().all(|c| c.is_ascii_alphabetic())
            && s != "XX"
            && s != "ZZ")
            .then_some(s)
    };
    if let Ok(name) = HeaderName::from_bytes(header_name.as_bytes())
        && let Some(found) = headers
            .get(&name)
            .and_then(|v| v.to_str().ok())
            .and_then(valid)
    {
        return Some(found);
    }
    let first = headers
        .get(header::ACCEPT_LANGUAGE)?
        .to_str()
        .ok()?
        .split(',')
        .next()?
        .split(';')
        .next()?
        .trim();
    first.split('-').skip(1).find_map(valid)
}

/// Whether the region uses the strict model, and the model that applies.
pub(crate) fn resolve_model(region: Option<&str>, v: &Values) -> (bool, String) {
    let list: Vec<&str> = v
        .get("cookies.strictRegions")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let strict = match region {
        Some(r) => list.iter().any(|c| c.eq_ignore_ascii_case(r)),
        None => text(v, "cookies.unknownRegion") != "normal",
    };
    let base = text(v, "cookies.mode");
    let model = if strict {
        match text(v, "cookies.regionMode") {
            "same" | "" => base,
            other => other,
        }
    } else {
        base
    };
    (strict, if model.is_empty() { "optIn" } else { model }.to_owned())
}

fn categories_json(v: &Values) -> Value {
    let mut out = vec![json!({
        "id": "necessary", "enabled": true, "default": true,
        "label": v.get("cookies.label.necessary").cloned().unwrap_or(json!({})),
        "description": v.get("cookies.description.necessary").cloned().unwrap_or(json!({})),
    })];
    for c in OPTIONAL {
        out.push(json!({
            "id": c,
            "enabled": flag(v, &format!("cookies.{c}.enabled")),
            "default": flag(v, &format!("cookies.{c}.default")),
            "label": v.get(&format!("cookies.label.{c}")).cloned().unwrap_or(json!({})),
            "description": v.get(&format!("cookies.description.{c}")).cloned().unwrap_or(json!({})),
        }));
    }
    Value::Array(out)
}

fn rule_json(r: &CookieRule) -> Value {
    json!({"id": r.id, "kind": r.kind, "pattern": r.pattern, "category": r.category, "note": r.note, "createdAt": r.created_at})
}

fn inventory_json() -> Value {
    Value::Array(
        INVENTORY
            .iter()
            .map(|e| {
                json!({"id": e.id, "kind": e.kind, "pattern": e.pattern, "category": e.category,
                    "purpose": e.purpose, "duration": e.duration, "party": e.party, "provider": e.provider})
            })
            .collect(),
    )
}

/// The consent control's configuration for one request.
pub(crate) fn config_view(v: &Values, region: Option<&str>, rules: &[CookieRule]) -> Value {
    if !flag(v, "cookies.enabled") {
        return json!({"enabled": false});
    }
    let (strict, model) = resolve_model(region, v);
    let url = |k: &str| v.get(k).filter(|x| x.is_string()).cloned().unwrap_or(Value::Null);
    json!({
        "enabled": true,
        "mode": model,
        "region": region,
        "strictRegion": strict,
        "version": int(v, "cookies.policyVersion"),
        "lifetimeDays": int(v, "cookies.lifetimeDays"),
        "reprompt": flag(v, "cookies.repromptOnVersion"),
        "layout": text(v, "cookies.layout"),
        "position": text(v, "cookies.position"),
        "rejectProminent": flag(v, "cookies.rejectProminent"),
        "policyUrl": url("cookies.policyUrl"),
        "privacyUrl": url("cookies.privacyUrl"),
        "honourGpc": flag(v, "cookies.honourGpc"),
        "honourDnt": flag(v, "cookies.honourDnt"),
        "scanIntervalHours": int(v, "cookies.scanIntervalHours"),
        "unclassified": text(v, "cookies.unclassified"),
        "log": flag(v, "cookies.logConsent"),
        "text": {
            "title": v.get("cookies.text.title").cloned().unwrap_or(json!({})),
            "body": v.get("cookies.text.body").cloned().unwrap_or(json!({})),
        },
        "categories": categories_json(v),
        "inventory": inventory_json(),
        "rules": rules.iter().map(rule_json).collect::<Vec<_>>(),
    })
}

pub(crate) async fn request_domain(state: &AppState, headers: &HeaderMap, ext: &axum::http::Extensions) -> Option<String> {
    let host = state.external.host(headers, ext);
    domain_config::canonical(&state.store, "hint", host.as_deref())
        .await
        .ok()
        .map(|identity| identity.domain)
}

/// The configuration plus whether the generated policy pages are on, so the client links to them only while they exist.
fn with_policy_pages(mut view: Value, pages: bool) -> Value {
    view["policyPages"] = json!(pages);
    view
}

async fn public_config(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: axum::http::Extensions,
) -> Response {
    let unavailable = || StatusCode::SERVICE_UNAVAILABLE.into_response();
    let Some(domain) = request_domain(&state, &headers, &ext).await else {
        return unavailable();
    };
    let Ok(v) = values(&state, &domain).await else {
        return unavailable();
    };
    let region = region_of(&headers, text(&v, "cookies.regionHeader"));
    let rules = if flag(&v, "cookies.enabled") {
        match state.store.list_cookie_rules().await {
            Ok(rules) => rules,
            Err(_) => return unavailable(),
        }
    } else {
        Vec::new()
    };
    (
        [
            (header::CACHE_CONTROL, "no-store"),
            (header::VARY, "Accept-Language, X-Country-Code"),
        ],
        Json(with_policy_pages(config_view(&v, region.as_deref(), &rules), crate::compliance::flag(&state, &domain, "policy.publicPages").await)),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Consent recording
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ConsentBody {
    version: i64,
    decision: String,
    #[serde(default)]
    categories: BTreeMap<String, bool>,
    #[serde(default)]
    signal: Option<String>,
}

/// A validated decision with the categories normalised: necessary on, categories
/// the administrator does not offer off, and the bulk decisions applied.
pub(crate) fn normalise(
    body_decision: &str,
    categories: &BTreeMap<String, bool>,
    v: &Values,
) -> Result<(String, Value), &'static str> {
    if !DECISIONS.contains(&body_decision) {
        return Err("unknown decision");
    }
    if categories.keys().any(|k| !OPTIONAL.contains(&k.as_str())) {
        return Err("unknown category");
    }
    let mut out = Map::new();
    out.insert("necessary".into(), json!(true));
    for c in OPTIONAL {
        let offered = flag(v, &format!("cookies.{c}.enabled"));
        let on = match body_decision {
            "acceptAll" => offered,
            "rejectAll" => false,
            _ => offered && categories.get(c).copied().unwrap_or(false),
        };
        out.insert(c.into(), json!(on));
    }
    Ok((body_decision.to_owned(), Value::Object(out)))
}

async fn record(
    state: &AppState,
    headers: &HeaderMap,
    ext: &axum::http::Extensions,
    body: &[u8],
    account_id: Option<String>,
) -> Response {
    if !allowed(state, headers, ext).await {
        return too_many(state).await;
    }
    if body.len() > BODY_CAP {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    }
    let Ok(parsed) = serde_json::from_slice::<ConsentBody>(body) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let signal = parsed.signal.as_deref().unwrap_or("none");
    if !(1..=1_000_000).contains(&parsed.version) || !SIGNALS.contains(&signal) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let Some(domain) = request_domain(state, headers, ext).await else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    let Ok(v) = values(state, &domain).await else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    let Ok((decision, categories)) = normalise(&parsed.decision, &parsed.categories, &v) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    // Nothing is kept while the control or the log is off; the browser keeps the
    // decision itself, so a stale client stays quiet.
    if !flag(&v, "cookies.enabled") || !flag(&v, "cookies.logConsent") {
        return StatusCode::NO_CONTENT.into_response();
    }
    let region = region_of(headers, text(&v, "cookies.regionHeader")).unwrap_or_else(|| "??".into());
    let row = ConsentRow {
        id: uuid::Uuid::new_v4().to_string(),
        account_id,
        domain,
        policy_version: parsed.version,
        decision,
        categories_json: categories.to_string(),
        region,
        signal: signal.to_owned(),
        created_at: now(),
    };
    if state.store.record_cookie_consent(&row).await.is_err() {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    maybe_purge(state).await;
    StatusCode::NO_CONTENT.into_response()
}

async fn record_anonymous(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: axum::http::Extensions,
    body: Bytes,
) -> Response {
    record(&state, &headers, &ext, &body, None).await
}

async fn record_for_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: axum::http::Extensions,
    body: Bytes,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(session) => session,
        Err(response) => return response,
    };
    record(&state, &headers, &ext, &body, Some(session.account_id.clone())).await
}

/// The newest decision this account recorded, so another device can adopt it.
async fn latest_for_account(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(session) => session,
        Err(response) => return response,
    };
    match state.store.latest_cookie_consent(&session.account_id).await {
        Ok(Some(row)) => (
            [(header::CACHE_CONTROL, "no-store")],
            Json(json!({
                "version": row.policy_version,
                "decision": row.decision,
                "categories": serde_json::from_str::<Value>(&row.categories_json).unwrap_or(Value::Null),
                "signal": row.signal,
                "decidedAt": row.created_at,
            })),
        )
            .into_response(),
        Ok(None) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}

static LAST_PURGE: AtomicI64 = AtomicI64::new(0);

/// Delete consent rows past the retention, at most once an hour per process.
async fn maybe_purge(state: &AppState) {
    let t = now();
    let last = LAST_PURGE.load(Ordering::Relaxed);
    if t - last < 3600
        || LAST_PURGE
            .compare_exchange(last, t, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
    {
        return;
    }
    let Ok(domains) = state.store.cookie_consent_domains().await else { return };
    for domain in domains {
        let subject = mw_store::settings_policy::SettingSubject {
            domain: &domain,
            ..Default::default()
        };
        let days = settings_registry::effective_for(&state.store, "cookies.logRetentionDays", &subject)
            .await
            .ok()
            .flatten()
            .and_then(|setting| setting.value.as_i64())
            .unwrap_or(0);
        if days > 0 {
            let _ = state.store.purge_domain_cookie_consents(&domain, t - days * 86_400).await;
        }
    }
}

// ---------------------------------------------------------------------------
// Storage scan report
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Item {
    kind: String,
    name: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Report {
    items: Vec<Item>,
}

/// Record the names in `report` that no rule or inventory entry classifies.
/// Returns how many were new reports (classified and invalid names are skipped).
async fn ingest(state: &AppState, report: Report) -> Result<usize, StatusCode> {
    if report.items.len() > MAX_SCAN_ITEMS {
        return Err(StatusCode::BAD_REQUEST);
    }
    let rules = state
        .store
        .list_cookie_rules()
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let mut recorded = 0;
    for item in &report.items {
        if !KINDS.contains(&item.kind.as_str()) || !valid_name(&item.name) {
            continue;
        }
        if classify(&item.kind, &item.name, &rules).is_some() {
            continue;
        }
        state
            .store
            .record_detected_storage(&uuid::Uuid::new_v4().to_string(), &item.kind, &item.name, now())
            .await
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        recorded += 1;
    }
    Ok(recorded)
}

async fn report_detected(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: axum::http::Extensions,
    body: Bytes,
) -> Response {
    if !allowed(&state, &headers, &ext).await {
        return too_many(&state).await;
    }
    // Only while the control is on: a page that is not offering consent does not
    // need a review list, and an anonymous route should not grow it for nothing.
    let Some(domain) = request_domain(&state, &headers, &ext).await else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    match values(&state, &domain).await {
        Ok(v) if flag(&v, "cookies.enabled") => {}
        Ok(_) => return StatusCode::NO_CONTENT.into_response(),
        Err(()) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
    let Ok(report) = serde_json::from_slice::<Report>(&body) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    match ingest(&state, report).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(status) => status.into_response(),
    }
}

// ---------------------------------------------------------------------------
// Admin
// ---------------------------------------------------------------------------

async fn inventory(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsRead)?;
    let rules = state.store.list_cookie_rules().await.map_err(AdminApiError::internal)?;
    Ok(([(header::CACHE_CONTROL, "no-store")], Json(json!({
        "categories": CATEGORIES, "kinds": KINDS,
        "builtin": inventory_json(),
        "rules": rules.iter().map(rule_json).collect::<Vec<_>>(),
    })))
        .into_response())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleBody {
    kind: String,
    pattern: String,
    category: String,
    #[serde(default)]
    note: String,
}

fn check_rule(body: &RuleBody) -> Result<(), AdminApiError> {
    if !KINDS.contains(&body.kind.as_str()) {
        return Err(AdminApiError::invalid("invalidKind", "Unknown storage kind.", "/kind"));
    }
    if !valid_pattern(&body.pattern) {
        return Err(AdminApiError::invalid(
            "invalidPattern",
            "Use 1 to 128 printable characters; * stands for any run of characters.",
            "/pattern",
        ));
    }
    if !CATEGORIES.contains(&body.category.as_str()) {
        return Err(AdminApiError::invalid("invalidCategory", "Unknown category.", "/category"));
    }
    if body.note.chars().count() > 200 || body.note.chars().any(char::is_control) {
        return Err(AdminApiError::invalid("invalidNote", "Notes are plain text up to 200 characters.", "/note"));
    }
    Ok(())
}

async fn add_rule(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(body): AdminJson<RuleBody>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsWrite)?;
    check_rule(&body)?;
    let existing = state.store.list_cookie_rules().await.map_err(AdminApiError::internal)?;
    let replaces = existing.iter().any(|r| r.kind == body.kind && r.pattern == body.pattern);
    if !replaces && existing.len() >= MAX_RULES {
        return Err(AdminApiError::new(StatusCode::CONFLICT, "tooManyRules", "Remove a rule before adding another."));
    }
    let rule = CookieRule {
        id: uuid::Uuid::new_v4().to_string(),
        kind: body.kind,
        pattern: body.pattern,
        category: body.category,
        note: body.note,
        created_at: now(),
    };
    state.store.upsert_cookie_rule(&rule).await.map_err(AdminApiError::internal)?;
    crate::admin_accounts::audit(&state, &ctx, "cookie-rule-saved", "cookies:rules", None,
        Some(json!({"kind": rule.kind, "pattern": rule.pattern, "category": rule.category}))).await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn delete_rule(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsWrite)?;
    if !state.store.delete_cookie_rule(&id).await.map_err(AdminApiError::internal)? {
        return Err(AdminApiError::not_found("No such rule."));
    }
    crate::admin_accounts::audit(&state, &ctx, "cookie-rule-deleted", "cookies:rules", None, None).await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn list_detected(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsRead)?;
    let rows = state.store.list_detected_storage().await.map_err(AdminApiError::internal)?;
    Ok(([(header::CACHE_CONTROL, "no-store")], Json(json!({
        "items": rows.iter().map(|d| json!({
            "id": d.id, "kind": d.kind, "name": d.name, "firstSeen": d.first_seen,
            "lastSeen": d.last_seen, "seenCount": d.seen_count, "status": d.status,
        })).collect::<Vec<_>>(),
    })))
        .into_response())
}

/// A scan run from the admin console's own browser, which reports like any other.
async fn admin_scan(
    State(state): State<AppState>,
    ctx: AdminCtx,
    AdminJson(report): AdminJson<Report>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsWrite)?;
    let recorded = ingest(&state, report)
        .await
        .map_err(|s| AdminApiError::new(s, "scanRefused", "The scan report was refused."))?;
    Ok(Json(json!({"recorded": recorded})).into_response())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ClassifyBody {
    category: String,
    #[serde(default)]
    note: String,
}

async fn classify_detected(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
    AdminJson(body): AdminJson<ClassifyBody>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsWrite)?;
    let Some(found) = state.store.get_detected_storage(&id).await.map_err(AdminApiError::internal)? else {
        return Err(AdminApiError::not_found("No such detected item."));
    };
    let rule = RuleBody { kind: found.kind.clone(), pattern: found.name.clone(), category: body.category, note: body.note };
    // The name is stored literally; a `*` in it must not become a wildcard.
    if found.name.contains('*') {
        return Err(AdminApiError::invalid("invalidPattern", "Add a rule with an explicit pattern instead.", "/id"));
    }
    check_rule(&rule)?;
    state.store.upsert_cookie_rule(&CookieRule {
        id: uuid::Uuid::new_v4().to_string(),
        kind: rule.kind, pattern: rule.pattern, category: rule.category.clone(), note: rule.note, created_at: now(),
    }).await.map_err(AdminApiError::internal)?;
    state.store.delete_detected_storage(&id).await.map_err(AdminApiError::internal)?;
    crate::admin_accounts::audit(&state, &ctx, "cookie-classified", "cookies:rules", None,
        Some(json!({"kind": found.kind, "name": found.name, "category": rule.category}))).await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn ignore_detected(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsWrite)?;
    if !state.store.set_detected_storage_status(&id, "ignored").await.map_err(AdminApiError::internal)? {
        return Err(AdminApiError::not_found("No such detected item."));
    }
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn delete_detected(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsWrite)?;
    if !state.store.delete_detected_storage(&id).await.map_err(AdminApiError::internal)? {
        return Err(AdminApiError::not_found("No such detected item."));
    }
    Ok(StatusCode::NO_CONTENT.into_response())
}

// ---------------------------------------------------------------------------
// Statistics and export
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Range {
    from: Option<i64>,
    to: Option<i64>,
}

fn bounds(range: &Range) -> Result<(i64, i64), AdminApiError> {
    let to = range.to.unwrap_or_else(|| now() + 1);
    let from = range.from.unwrap_or(to - 30 * 86_400);
    if from < 0 || to <= from || to - from > 3660 * 86_400 {
        return Err(AdminApiError::new(StatusCode::BAD_REQUEST, "invalidRange", "The period is empty or too long."));
    }
    Ok((from, to))
}

/// Aggregate consent rows: totals by decision, acceptance per category, regions,
/// signals, policy versions and a per-day series by decision.
pub(crate) fn summarize(rows: &[ConsentRow], from: i64, to: i64) -> Value {
    let mut by_decision: BTreeMap<&str, i64> = DECISIONS.iter().map(|d| (*d, 0)).collect();
    let mut cat: BTreeMap<&str, i64> = OPTIONAL.iter().map(|c| (*c, 0)).collect();
    let mut regions: BTreeMap<String, i64> = BTreeMap::new();
    let mut signals: BTreeMap<&str, i64> = SIGNALS.iter().map(|s| (*s, 0)).collect();
    let mut versions: BTreeMap<i64, i64> = BTreeMap::new();
    let mut days: BTreeMap<i64, BTreeMap<&str, i64>> = BTreeMap::new();
    let first_day = from.div_euclid(86_400) * 86_400;
    let mut d = first_day;
    while d < to && days.len() < 400 {
        days.insert(d, DECISIONS.iter().map(|k| (*k, 0)).collect());
        d += 86_400;
    }
    for r in rows {
        if let Some(n) = by_decision.get_mut(r.decision.as_str()) {
            *n += 1;
        }
        if let Ok(Value::Object(c)) = serde_json::from_str::<Value>(&r.categories_json) {
            for k in OPTIONAL {
                if c.get(k) == Some(&json!(true)) {
                    *cat.entry(k).or_default() += 1;
                }
            }
        }
        *regions.entry(r.region.clone()).or_default() += 1;
        if let Some(n) = signals.get_mut(r.signal.as_str()) {
            *n += 1;
        }
        *versions.entry(r.policy_version).or_default() += 1;
        if let Some(day) = days.get_mut(&(r.created_at.div_euclid(86_400) * 86_400))
            && let Some(n) = day.get_mut(r.decision.as_str())
        {
            *n += 1;
        }
    }
    let mut top: Vec<(String, i64)> = regions.into_iter().collect();
    top.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    top.truncate(10);
    json!({
        "from": from, "to": to, "total": rows.len(),
        "decisions": by_decision,
        "categories": cat.iter().map(|(k, n)| (k.to_string(), json!({"accepted": n, "total": rows.len()}))).collect::<Map<_, _>>(),
        "regions": top.iter().map(|(r, n)| json!({"region": r, "count": n})).collect::<Vec<_>>(),
        "signals": signals,
        "versions": versions.iter().map(|(v, n)| json!({"version": v, "count": n})).collect::<Vec<_>>(),
        "series": days.iter().map(|(t, c)| json!({"t": t, "decisions": c})).collect::<Vec<_>>(),
    })
}

async fn stats(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Query(range): Query<Range>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsRead)?;
    let (from, to) = bounds(&range)?;
    let (rows, truncated) = state.store.cookie_consents_between(from, to).await.map_err(AdminApiError::internal)?;
    let mut body = summarize(&rows, from, to);
    body["truncated"] = json!(truncated);
    Ok(([(header::CACHE_CONTROL, "no-store")], Json(body)).into_response())
}

/// A CSV field: quoted, quotes doubled, and a leading formula character defused.
pub(crate) fn csv_field(s: &str) -> String {
    let guarded = if s.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        format!("'{s}")
    } else {
        s.to_owned()
    };
    format!("\"{}\"", guarded.replace('"', "\"\""))
}

pub(crate) fn to_csv(rows: &[ConsentRow]) -> String {
    let mut out = String::from("time,domain,policyVersion,decision,necessary,functional,analytics,marketing,region,signal,accountId\r\n");
    for r in rows {
        let c: Value = serde_json::from_str(&r.categories_json).unwrap_or(Value::Null);
        let b = |k: &str| if c.get(k) == Some(&json!(true)) { "true" } else { "false" };
        let time = chrono::DateTime::from_timestamp(r.created_at, 0)
            .map(|t| t.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
            .unwrap_or_default();
        let fields = [
            csv_field(&time),
            csv_field(&r.domain),
            r.policy_version.to_string(),
            csv_field(&r.decision),
            b("necessary").into(),
            b("functional").into(),
            b("analytics").into(),
            b("marketing").into(),
            csv_field(&r.region),
            csv_field(&r.signal),
            csv_field(r.account_id.as_deref().unwrap_or("")),
        ];
        out.push_str(&fields.join(","));
        out.push_str("\r\n");
    }
    out
}

async fn export_csv(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Query(range): Query<Range>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsRead)?;
    let (from, to) = bounds(&range)?;
    let (rows, truncated) = state.store.cookie_consents_between(from, to).await.map_err(AdminApiError::internal)?;
    crate::admin_accounts::audit(&state, &ctx, "cookie-consent-exported", "cookies:consents", None,
        Some(json!({"rows": rows.len(), "from": from, "to": to}))).await;
    Ok((
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8".to_owned()),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"cookie-consents.csv\"".to_owned()),
            (header::CACHE_CONTROL, "no-store".to_owned()),
            (HeaderName::from_static("x-truncated"), truncated.to_string()),
        ],
        to_csv(&rows),
    )
        .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vals(pairs: &[(&str, Value)]) -> Values {
        let mut v: Values = settings_registry::REGISTRY
            .iter()
            .filter(|d| d.key.starts_with("cookies."))
            .map(|d| (d.key.to_owned(), d.default_value()))
            .collect();
        for (k, x) in pairs {
            v.insert((*k).to_owned(), x.clone());
        }
        v
    }
    fn headers(pairs: &[(&str, &str)]) -> HeaderMap {
        let mut h = HeaderMap::new();
        for (k, v) in pairs {
            h.insert(HeaderName::from_bytes(k.as_bytes()).unwrap(), v.parse().unwrap());
        }
        h
    }

    #[test]
    fn region_prefers_the_trusted_header_then_accept_language() {
        let h = headers(&[("x-country-code", "pt"), ("accept-language", "en-US,en;q=0.9")]);
        assert_eq!(region_of(&h, "X-Country-Code").as_deref(), Some("PT"));
        let h = headers(&[("accept-language", "de-AT,de;q=0.9")]);
        assert_eq!(region_of(&h, "X-Country-Code").as_deref(), Some("AT"));
        let h = headers(&[("accept-language", "fr")]);
        assert_eq!(region_of(&h, "X-Country-Code"), None);
        let h = headers(&[("x-country-code", "XX")]);
        assert_eq!(region_of(&h, "X-Country-Code"), None);
        let h = headers(&[("x-country-code", "zh-Hans-CN")]);
        assert_eq!(region_of(&h, "X-Country-Code"), None, "a header is a country code, not a tag");
        assert_eq!(region_of(&h, "not a header name"), None);
    }

    #[test]
    fn model_follows_region_rules() {
        let v = vals(&[("cookies.mode", json!("optOut")), ("cookies.regionMode", json!("optIn"))]);
        assert_eq!(resolve_model(Some("PT"), &v), (true, "optIn".into()));
        assert_eq!(resolve_model(Some("US"), &v), (false, "optOut".into()));
        assert_eq!(resolve_model(None, &v), (true, "optIn".into()), "unknown is strict by default");
        let v = vals(&[("cookies.mode", json!("noticeOnly")), ("cookies.unknownRegion", json!("normal"))]);
        assert_eq!(resolve_model(None, &v), (false, "noticeOnly".into()));
        let v = vals(&[("cookies.mode", json!("optOut")), ("cookies.regionMode", json!("same"))]);
        assert_eq!(resolve_model(Some("DE"), &v), (true, "optOut".into()));
    }

    #[test]
    fn config_is_minimal_when_off_and_complete_when_on() {
        assert_eq!(config_view(&vals(&[]), Some("PT"), &[]), json!({"enabled": false}));
        let v = vals(&[("cookies.enabled", json!(true)), ("cookies.analytics.enabled", json!(true))]);
        let c = config_view(&v, Some("PT"), &[]);
        assert_eq!(c["mode"], "optIn");
        assert_eq!(c["strictRegion"], true);
        assert_eq!(c["categories"].as_array().unwrap().len(), 4);
        assert_eq!(c["categories"][0]["id"], "necessary");
        assert_eq!(c["categories"][2]["enabled"], true);
        assert_eq!(c["categories"][3]["enabled"], false);
        assert!(c["inventory"].as_array().unwrap().len() > 10);
        // No value or account data is part of the configuration.
        assert!(c.get("accountId").is_none());
    }

    #[test]
    fn decisions_are_normalised_against_what_is_offered() {
        let v = vals(&[("cookies.enabled", json!(true)), ("cookies.analytics.enabled", json!(true))]);
        let ask = |d: &str, c: &[(&str, bool)]| {
            normalise(d, &c.iter().map(|(k, b)| ((*k).to_owned(), *b)).collect(), &v)
                .map(|(_, c)| c)
        };
        assert_eq!(
            ask("acceptAll", &[]).unwrap(),
            json!({"necessary": true, "functional": true, "analytics": true, "marketing": false})
        );
        assert_eq!(
            ask("rejectAll", &[("analytics", true)]).unwrap(),
            json!({"necessary": true, "functional": false, "analytics": false, "marketing": false})
        );
        assert_eq!(
            ask("custom", &[("marketing", true), ("analytics", true)]).unwrap(),
            json!({"necessary": true, "functional": false, "analytics": true, "marketing": false}),
            "marketing is not offered"
        );
        assert!(ask("custom", &[("necessary", false)]).is_err());
        assert!(ask("maybe", &[]).is_err());
    }

    fn row(at: i64, decision: &str, cats: Value, region: &str, signal: &str) -> ConsentRow {
        ConsentRow {
            id: at.to_string(),
            account_id: None,
            domain: "example.test".into(),
            policy_version: 1,
            decision: decision.into(),
            categories_json: cats.to_string(),
            region: region.into(),
            signal: signal.into(),
            created_at: at,
        }
    }

    #[test]
    fn statistics_count_decisions_categories_and_days() {
        let rows = vec![
            row(100, "acceptAll", json!({"functional": true, "analytics": true}), "PT", "none"),
            row(200, "rejectAll", json!({}), "PT", "gpc"),
            row(90_000, "custom", json!({"functional": true}), "US", "none"),
        ];
        let s = summarize(&rows, 0, 2 * 86_400);
        assert_eq!(s["total"], 3);
        assert_eq!(s["decisions"]["acceptAll"], 1);
        assert_eq!(s["decisions"]["notice"], 0);
        assert_eq!(s["categories"]["functional"]["accepted"], 2);
        assert_eq!(s["categories"]["analytics"]["accepted"], 1);
        assert_eq!(s["signals"]["gpc"], 1);
        assert_eq!(s["regions"][0]["region"], "PT");
        assert_eq!(s["series"].as_array().unwrap().len(), 2);
        assert_eq!(s["series"][1]["decisions"]["custom"], 1);
    }

    #[test]
    fn csv_defuses_formulas_and_quotes() {
        assert_eq!(csv_field("=cmd"), "\"'=cmd\"");
        assert_eq!(csv_field("a\"b"), "\"a\"\"b\"");
        let csv = to_csv(&[row(0, "custom", json!({"necessary": true, "functional": true}), "PT", "none")]);
        let mut lines = csv.lines();
        assert!(lines.next().unwrap().starts_with("time,domain,"));
        assert_eq!(
            lines.next().unwrap(),
            "\"1970-01-01T00:00:00Z\",\"example.test\",1,\"custom\",true,true,false,false,\"PT\",\"none\",\"\""
        );
    }
}
