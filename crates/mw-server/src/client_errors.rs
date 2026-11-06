//! Browser error reports: ingest, grouping, symbolication and the admin routes.
//!
//! The web app catches the errors that would otherwise only reach the browser
//! console (a section's error boundary, `window.onerror`, unhandled rejections,
//! failed lazy chunks, worker errors, CSP violations) and posts them, batched, to
//! `POST /api/observability/client-errors` (anonymous, reads no cookie, so it is
//! CSRF-exempt like `/api/cookies/consent`) or, when signed in,
//! `POST /api/observability/client-errors/session` (session-bound, CSRF-checked,
//! honours the person's `privacy.errorReports` choice, and the only route that
//! can add the salted account hash).
//!
//! Privacy. The browser scrubs before sending; this module scrubs again, because
//! the route is unauthenticated: query strings and hashes are removed from every
//! URL, addresses and long opaque tokens from messages, and every field is
//! validated against a short alphabet and length. No user id, address, IP, token,
//! message text, attachment or file name, or URL query is stored. The only
//! person-derived value is `account_hash` (a keyed hash), and only when the
//! administrator turned `observability.client.includeAccountHash` on and the
//! report came over the session route. Reports are never audited one by one.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson},
    admin_router::AdminRegistrar,
    capabilities::Capability as C,
    client_errors_symbolicate as sym,
    scope_mw::rate_limit::KeyedRateLimiter,
};
use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path, Query, State},
    http::{HeaderMap, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use mw_store::{
    Store,
    client_errors::{ClientErrorEvent, ClientErrorGroup, GroupFilter, NewClientError},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::HashSet,
    sync::{Mutex, OnceLock},
};

pub(crate) const ENABLED_KEY: &str = "observability.client.enabled";
pub(crate) const SAMPLE_KEY: &str = "observability.client.sampleRate";
pub(crate) const MAX_SESSION_KEY: &str = "observability.client.maxPerSession";
pub(crate) const ACCOUNT_HASH_KEY: &str = "observability.client.includeAccountHash";
pub(crate) const CONSOLE_KEY: &str = "observability.client.captureConsole";
pub(crate) const CSP_KEY: &str = "observability.client.captureCsp";
pub(crate) const SOURCE_MAPS_KEY: &str = "observability.client.sourceMaps";
pub(crate) const FORWARD_KEY: &str = "observability.client.forward";
pub(crate) const EVENT_DAYS_KEY: &str = "retention.clientErrors.eventDays";
pub(crate) const GROUP_DAYS_KEY: &str = "retention.clientErrors.groupDays";
pub(crate) const USER_KEY: &str = "privacy.errorReports";

/// 20 reports of 16 KiB, plus the envelope.
const BODY_CAP: usize = 20 * 16 * 1024 + 4096;
const ANON_BODY_CAP: usize = 64 * 1024;
const MAX_REPORT: usize = 16 * 1024;
const MAX_BATCH: usize = 20;
const ANON_MAX_BATCH: usize = 5;
/// Events kept per group per hour; further reports only add to the counts.
const MAX_EVENTS_PER_HOUR: i64 = 20;
/// Sessions: ten batches at once, then one per six seconds. Anonymous visitors
/// (per address): four, then one per twenty seconds.
const SESSION_BURST: f64 = 10.0;
const SESSION_REFILL: f64 = 1.0 / 6.0;
const ANON_BURST: f64 = 4.0;
const ANON_REFILL: f64 = 1.0 / 20.0;

pub(crate) const KINDS: [&str; 7] = [
    "boundary",
    "window.onerror",
    "unhandledrejection",
    "chunk",
    "worker",
    "csp",
    "console",
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Policy {
    pub enabled: bool,
    pub sample_rate: f64,
    pub max_per_session: i64,
    pub include_account_hash: bool,
    pub capture_console: bool,
    pub capture_csp: bool,
    pub source_maps: bool,
    pub forward: bool,
}

impl Policy {
    pub(crate) async fn load(store: &Store) -> Self {
        async fn read(store: &Store, key: &str) -> Option<Value> {
            let row = store.get_setting_policy(key).await.ok()??;
            serde_json::from_str(&row.global_value?).ok()
        }
        let flag = |v: Option<Value>, d: bool| v.and_then(|v| v.as_bool()).unwrap_or(d);
        Self {
            enabled: flag(read(store, ENABLED_KEY).await, true),
            sample_rate: read(store, SAMPLE_KEY)
                .await
                .and_then(|v| v.as_f64())
                .filter(|r| (0.0..=1.0).contains(r))
                .unwrap_or(1.0),
            max_per_session: read(store, MAX_SESSION_KEY)
                .await
                .and_then(|v| v.as_i64())
                .filter(|n| (1..=1000).contains(n))
                .unwrap_or(20),
            include_account_hash: flag(read(store, ACCOUNT_HASH_KEY).await, false),
            capture_console: flag(read(store, CONSOLE_KEY).await, false),
            capture_csp: flag(read(store, CSP_KEY).await, true),
            source_maps: flag(read(store, SOURCE_MAPS_KEY).await, true),
            forward: flag(read(store, FORWARD_KEY).await, false),
        }
    }
    fn public(&self) -> Value {
        json!({
            "enabled": self.enabled,
            "sampleRate": self.sample_rate,
            "maxPerSession": self.max_per_session,
            "captureConsole": self.capture_console,
            "captureCsp": self.capture_csp,
        })
    }
}

fn limiter(session: bool) -> &'static Mutex<KeyedRateLimiter> {
    static SESSION: OnceLock<Mutex<KeyedRateLimiter>> = OnceLock::new();
    static ANON: OnceLock<Mutex<KeyedRateLimiter>> = OnceLock::new();
    if session {
        SESSION.get_or_init(|| Mutex::new(KeyedRateLimiter::new(SESSION_BURST, SESSION_REFILL, 20_000)))
    } else {
        ANON.get_or_init(|| Mutex::new(KeyedRateLimiter::new(ANON_BURST, ANON_REFILL, 20_000)))
    }
}

fn map_cache() -> &'static sym::MapCache {
    static C: OnceLock<sym::MapCache> = OnceLock::new();
    C.get_or_init(sym::MapCache::default)
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/observability/client-errors/policy", get(public_policy))
        .route(
            "/api/observability/client-errors",
            post(ingest_anonymous).layer(DefaultBodyLimit::max(ANON_BODY_CAP)),
        )
        .route(
            "/api/observability/client-errors/session",
            post(ingest_session).layer(DefaultBodyLimit::max(BODY_CAP)),
        )
}

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, "/client-errors", C::ObservabilityRead, list_groups);
    r.admin_endpoint(Method::GET, "/client-errors/export", C::ObservabilityRead, export);
    r.admin_endpoint(Method::GET, "/client-errors/{id}", C::ObservabilityRead, group_detail);
    r.admin_endpoint(Method::POST, "/client-errors/{id}/status", C::SettingsWrite, set_status);
    r.admin_endpoint(Method::DELETE, "/client-errors/{id}", C::SettingsWrite, delete_group);
}

// ---- scrubbing ------------------------------------------------------------

/// Remove `?query` and `#hash` from every URL-like or path-like word.
pub(crate) fn strip_queries(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for (i, word) in s.split(' ').enumerate() {
        if i > 0 {
            out.push(' ');
        }
        let pathish = word.contains('/') || word.contains("://");
        let cut = word.find(['?', '#']).filter(|p| {
            // A path-like word, or a `?` followed by key=value pairs.
            pathish || word[*p..].contains('=')
        });
        match cut {
            Some(p) => {
                out.push_str(&word[..p]);
                // Keep a closing bracket or location suffix after the removed part.
                if let Some(rest) = word[p..].find([')', ':']) {
                    if word[p..p + rest].chars().all(|c| !c.is_whitespace()) {
                        out.push_str(&word[p + rest..]);
                    }
                }
            }
            None => out.push_str(word),
        }
    }
    out
}

/// Remove `scheme://authority` from every URL: a frame keeps only its path.
pub(crate) fn strip_origins(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find("://") {
        let scheme_start = rest[..i]
            .rfind(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-')))
            .map_or(0, |p| p + 1);
        let after = &rest[i + 3..];
        let end = after
            .find(|c: char| c == '/' || c.is_whitespace() || matches!(c, ')' | '(' | '"' | '\'' | '?' | '#'))
            .unwrap_or(after.len());
        out.push_str(&rest[..scheme_start]);
        rest = &after[end..];
    }
    out.push_str(rest);
    out
}

/// A run of 24 or more token characters holding both a digit and a letter, a JWT,
/// or a `Bearer` credential.
pub(crate) fn redact_tokens(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut run = String::new();
    let flush = |run: &mut String, out: &mut String| {
        if run.len() >= 24
            && run.bytes().any(|b| b.is_ascii_digit())
            && run.bytes().any(|b| b.is_ascii_alphabetic())
        {
            out.push_str("[token]");
        } else {
            out.push_str(run);
        }
        run.clear();
    };
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '+' | '=') {
            run.push(c);
        } else {
            flush(&mut run, &mut out);
            out.push(c);
        }
    }
    flush(&mut run, &mut out);
    out
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_owned();
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &s[..end])
}

/// A message: no queries or hashes, no addresses, no long tokens, bounded.
pub(crate) fn scrub_message(s: &str, max: usize) -> String {
    let no_addr = crate::errors::scrub_addresses(s).unwrap_or_else(|| s.to_owned());
    truncate(&redact_tokens(&strip_queries(&strip_origins(&no_addr))), max)
}

/// A stack: queries, hashes and addresses removed; tokens are left alone because
/// hashed chunk names look like tokens.
pub(crate) fn scrub_stack(s: &str, max: usize) -> String {
    let no_addr = crate::errors::scrub_addresses(s).unwrap_or_else(|| s.to_owned());
    let lines: Vec<String> = no_addr.lines().take(60).map(|l| strip_queries(&strip_origins(l.trim_end()))).collect();
    truncate(&lines.join("\n"), max)
}

fn token(s: &str, max: usize, extra: &[char]) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric() || extra.contains(c))
        .take(max)
        .collect()
}

// ---- parsing --------------------------------------------------------------

#[derive(Deserialize, Default)]
#[serde(default)]
struct Batch {
    reports: Vec<Raw>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Raw {
    id: String,
    kind: String,
    name: String,
    message: String,
    stack: String,
    boundary: String,
    #[serde(rename = "componentStack")]
    component_stack: String,
    route: String,
    build: String,
    fingerprint: String,
    count: i64,
    visit: String,
    #[serde(rename = "uaFamily")]
    ua_family: String,
    #[serde(rename = "uaVersion")]
    ua_version: String,
    #[serde(rename = "osFamily")]
    os_family: String,
    viewport: String,
    locale: String,
    online: Option<bool>,
    #[serde(rename = "reducedMotion")]
    reduced_motion: Option<bool>,
    theme: String,
}

/// Validate and re-scrub one report. `Err` is a short reason, never echoed input.
fn accept(raw: &Raw, time: i64, account_hash: Option<String>) -> Result<NewClientError, &'static str> {
    let kind = KINDS.iter().find(|k| **k == raw.kind).ok_or("unknown kind")?;
    if raw.stack.len() > MAX_REPORT || raw.message.len() > MAX_REPORT {
        return Err("report too large");
    }
    let name = token(&raw.name, 80, &['_', '.', '$', ':', ' ']);
    if name.is_empty() {
        return Err("missing name");
    }
    let fingerprint = raw.fingerprint.to_ascii_lowercase();
    if !(8..=40).contains(&fingerprint.len()) || !fingerprint.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("bad fingerprint");
    }
    let id = if raw.id.len() == 36 && raw.id.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-') {
        raw.id.to_ascii_lowercase()
    } else {
        uuid::Uuid::new_v4().to_string()
    };
    let route = scrub_route(&raw.route);
    let extra = json!({
        "route": route,
        "uaVersion": token(&raw.ua_version, 12, &['.']),
        "online": raw.online,
        "reducedMotion": raw.reduced_motion,
        "theme": token(&raw.theme, 32, &['-', '_']),
        "componentStack": truncate(&scrub_stack(&raw.component_stack, 600), 600),
    });
    Ok(NewClientError {
        id,
        fingerprint,
        kind: (*kind).to_owned(),
        name,
        message: scrub_message(&raw.message, 300),
        boundary: token(&raw.boundary, 120, &['/', '.', ':', '>', '-', '_']),
        route_template: route,
        time,
        build: token(&raw.build, 64, &['+', '.', '-', '_']),
        occurrences: raw.count.clamp(1, 1000),
        visit: token(&raw.visit, 16, &[]),
        stack_raw: scrub_stack(&raw.stack, mw_store::client_errors::MAX_STACK),
        stack_symbolicated: None,
        ua_family: token(&raw.ua_family, 24, &[' ']),
        os_family: token(&raw.os_family, 24, &[' ']),
        viewport: token(&raw.viewport, 12, &['-']),
        locale: token(&raw.locale, 16, &['-', '_']),
        account_hash,
        extra: extra.to_string(),
    })
}

/// A route template: path segments only; an id-looking segment becomes `:id`.
pub(crate) fn scrub_route(route: &str) -> String {
    let path = route.trim_start_matches(['#', '/']).split(['?', '#']).next().unwrap_or("");
    let segs: Vec<String> = path
        .split('/')
        .filter(|s| !s.is_empty())
        .take(8)
        .map(|s| {
            let clean = token(s, 40, &['-', '_', ':']);
            let digits = clean.bytes().filter(u8::is_ascii_digit).count();
            if clean.len() > 16 || digits >= 3 || (digits >= 1 && clean.len() >= 8) || clean.contains('%') {
                ":id".to_owned()
            } else {
                clean
            }
        })
        .collect();
    format!("/{}", segs.join("/"))
}

// ---- ingest ---------------------------------------------------------------

async fn public_policy(State(state): State<AppState>) -> Response {
    let policy = Policy::load(&state.store).await;
    ([(header::CACHE_CONTROL, "no-store")], Json(policy.public())).into_response()
}

async fn ingest_anonymous(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: axum::http::Extensions,
    body: Bytes,
) -> Response {
    let key = crate::scope_mw::proxy::client_ip(&headers, &ext)
        .map(|ip| ip.to_canonical().to_string())
        .unwrap_or_else(|| "unknown".into());
    ingest(&state, &key, false, None, &body).await
}

async fn ingest_session(State(state): State<AppState>, headers: HeaderMap, body: Bytes) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(session) => session,
        Err(response) => return response,
    };
    // The person's own choice (admin default and lock, then the person).
    if let Ok(identity) = crate::settings_routes::UserSubject::resolved(&state.store, &session).await {
        let subject = identity.as_subject();
        let off = crate::settings_registry::effective_for(&state.store, USER_KEY, &subject)
            .await
            .ok()
            .flatten()
            .is_some_and(|v| v.value.as_str() == Some("off"));
        if off {
            return Json(json!({ "reporting": "off" })).into_response();
        }
    }
    let hash = if Policy::load(&state.store).await.include_account_hash {
        account_hash(&state.store, &session.account_id).await
    } else {
        None
    };
    let response = ingest(&state, &session.id, true, hash, &body).await;
    // The browser learns the person's choice from the answer, and shows it.
    if response.status() == StatusCode::NO_CONTENT {
        return Json(json!({ "reporting": "on" })).into_response();
    }
    response
}

async fn account_hash(store: &Store, account_id: &str) -> Option<String> {
    let (generation, key) = store.scrub_key_current(now()).await.ok()?;
    let p = mw_store::scrub::pseudonym(&key, generation, "client-error-account", account_id, "hmac-sha256-16");
    Some(p.rsplit('-').next().unwrap_or(&p).to_owned())
}

/// 204 on success (also when collection is off, so a stale client stays quiet),
/// 400 on a malformed body, 413 on an oversized one, 429 when over budget.
async fn ingest(
    state: &AppState,
    key: &str,
    session: bool,
    account_hash: Option<String>,
    body: &[u8],
) -> Response {
    let policy = Policy::load(&state.store).await;
    if !policy.enabled {
        return StatusCode::NO_CONTENT.into_response();
    }
    let cap = if session { BODY_CAP } else { ANON_BODY_CAP };
    if body.len() > cap {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    }
    let allowed = limiter(session)
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .check(key);
    crate::rate_leeway::note(
        "clientErrors",
        if allowed { crate::rate_leeway::Outcome::Allowed } else { crate::rate_leeway::Outcome::Rejected },
    );
    if !allowed {
        let wait = if session { 6 } else { 20 };
        return (StatusCode::TOO_MANY_REQUESTS, [(header::RETRY_AFTER, wait.to_string())]).into_response();
    }
    let Ok(batch) = serde_json::from_slice::<Batch>(body) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let limit = if session { MAX_BATCH } else { ANON_MAX_BATCH };
    if batch.reports.is_empty() || batch.reports.len() > limit {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let time = now();
    let mut accepted = Vec::new();
    for raw in &batch.reports {
        let Ok(mut report) = accept(raw, time, account_hash.clone()) else {
            return StatusCode::BAD_REQUEST.into_response();
        };
        // CSP and console reports are separate switches.
        if (report.kind == "csp" && !policy.capture_csp) || (report.kind == "console" && !policy.capture_console) {
            continue;
        }
        if policy.source_maps {
            report.stack_symbolicated = symbolicate_stack(state, &report.stack_raw).await;
        }
        accepted.push(report);
    }
    for report in accepted {
        match state.store.record_client_error(&report, MAX_EVENTS_PER_HOUR).await {
            Ok(recorded) => {
                note(&report, recorded.new_group);
                if recorded.new_group || recorded.regressed {
                    refresh_open_gauge(&state.store).await;
                }
                if policy.forward {
                    forward(&report);
                }
            }
            Err(error) => {
                tracing::warn!(target: "client_error", "report not stored: {error}");
                return StatusCode::SERVICE_UNAVAILABLE.into_response();
            }
        }
    }
    StatusCode::NO_CONTENT.into_response()
}

/// Run the symbolicator off the async threads (a map parse can take a while).
async fn symbolicate_stack(state: &AppState, stack: &str) -> Option<String> {
    let (state, stack) = (state.clone(), stack.to_owned());
    tokio::task::spawn_blocking(move || {
        sym::symbolicate(&stack, map_cache(), |path| crate::read_asset(&state, path)).map(|(text, _)| text)
    })
    .await
    .ok()
    .flatten()
}

fn boundary_label(boundary: &str) -> String {
    static SEEN: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    let first: String = boundary.split(['/', '>']).next().unwrap_or("").chars().take(32).collect();
    if first.is_empty() {
        return "none".into();
    }
    let mut seen = SEEN.get_or_init(Default::default).lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if seen.contains(&first) || seen.len() < 40 && seen.insert(first.clone()) {
        first
    } else {
        "other".into()
    }
}

fn note(report: &NewClientError, new_group: bool) {
    let kind: &'static str = KINDS.iter().find(|k| **k == report.kind).copied().unwrap_or("boundary");
    metrics::counter!("mw_client_errors_total", "kind" => kind, "boundary" => boundary_label(&report.boundary))
        .increment(report.occurrences.max(1).unsigned_abs());
    tracing::info!(
        target: "client_error",
        fingerprint = %report.fingerprint,
        name = %report.name,
        kind = %report.kind,
        boundary = %report.boundary,
        build = %report.build,
        route = %report.route_template,
        new_group,
        "browser error reported"
    );
}

async fn refresh_open_gauge(store: &Store) {
    if let Ok(n) = store.count_open_client_error_groups().await {
        metrics::gauge!("mw_client_error_groups_open").set(n as f64);
    }
}

/// Hand the scrubbed report to the operator's `/errors` sink, if one is configured.
fn forward(report: &NewClientError) {
    let value = json!({
        "source": "mailwoman-web",
        "kind": report.kind, "name": report.name, "message": report.message,
        "stack": report.stack_raw, "boundary": report.boundary, "route": report.route_template,
        "build": report.build, "fingerprint": report.fingerprint, "level": "error",
    });
    tokio::spawn(async move {
        crate::errors::forward_scrubbed(value).await;
    });
}

// ---- admin ----------------------------------------------------------------

fn group_json(g: &ClientErrorGroup) -> Value {
    json!({
        "id": g.id, "fingerprint": g.fingerprint, "kind": g.kind, "name": g.name,
        "message": g.message, "boundary": g.boundary, "routeTemplate": g.route_template,
        "firstSeen": g.first_seen, "lastSeen": g.last_seen, "count": g.count,
        "lastBuild": g.last_build, "status": g.status, "resolvedBuild": g.resolved_build,
        "reopenedCount": g.reopened_count, "sessions": g.visits,
    })
}

fn event_json(e: &ClientErrorEvent, symbolicated: Option<&str>) -> Value {
    let extra: Value = serde_json::from_str(&e.extra).unwrap_or(Value::Null);
    json!({
        "id": e.id, "time": e.time, "build": e.build, "occurrences": e.occurrences,
        "stackRaw": e.stack_raw,
        "stackSymbolicated": symbolicated.or(e.stack_symbolicated.as_deref()),
        "uaFamily": e.ua_family, "osFamily": e.os_family, "viewport": e.viewport,
        "locale": e.locale, "accountHash": e.account_hash, "extra": extra,
    })
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct ListQuery {
    status: Option<String>,
    build: Option<String>,
    boundary: Option<String>,
    range_hours: Option<i64>,
    limit: Option<i64>,
    format: Option<String>,
}

fn filter_of(q: &ListQuery) -> Result<(GroupFilter, i64), AdminApiError> {
    if let Some(s) = &q.status
        && !["open", "resolved", "ignored", ""].contains(&s.as_str())
    {
        return Err(AdminApiError::invalid("invalidStatus", "Unknown status.", "/status"));
    }
    let hours = q.range_hours.unwrap_or(24 * 7).clamp(1, 24 * 400);
    let since = now() - hours * 3600;
    Ok((
        GroupFilter {
            status: q.status.clone().filter(|s| !s.is_empty()),
            build: q.build.clone().filter(|s| !s.is_empty()),
            boundary: q.boundary.clone().filter(|s| !s.is_empty()),
            since: Some(since),
            limit: q.limit.unwrap_or(200),
        },
        since,
    ))
}

async fn list_groups(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Query(q): Query<ListQuery>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::ObservabilityRead)?;
    let (filter, since) = filter_of(&q)?;
    let groups = state.store.list_client_error_groups(&filter).await.map_err(AdminApiError::internal)?;
    let mut rows = Vec::with_capacity(groups.len());
    for g in groups.iter().take(100) {
        let series = state.store.client_error_series(&g.id, since).await.map_err(AdminApiError::internal)?;
        let mut row = group_json(g);
        row["series"] = json!(series);
        rows.push(row);
    }
    rows.extend(groups.iter().skip(100).map(group_json));
    let policy = Policy::load(&state.store).await;
    Ok(([(header::CACHE_CONTROL, "no-store")], Json(json!({
        "groups": rows, "since": since,
        "policy": { "enabled": policy.enabled, "sourceMaps": policy.source_maps, "forward": policy.forward,
                    "includeAccountHash": policy.include_account_hash },
    })))
        .into_response())
}

async fn group_detail(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::ObservabilityRead)?;
    let group = state
        .store
        .get_client_error_group(&id)
        .await
        .map_err(AdminApiError::internal)?
        .ok_or_else(|| AdminApiError::not_found("No such error group."))?;
    let events = state.store.client_error_events(&id, 100).await.map_err(AdminApiError::internal)?;
    let policy = Policy::load(&state.store).await;
    let mut out = Vec::with_capacity(events.len());
    for (i, e) in events.iter().enumerate() {
        // A map shipped after the event arrived still symbolicates it on view.
        let lazy = if i < 5 && policy.source_maps && e.stack_symbolicated.is_none() {
            symbolicate_stack(&state, &e.stack_raw).await
        } else {
            None
        };
        out.push(event_json(e, lazy.as_deref()));
    }
    let series = state
        .store
        .client_error_series(&id, group.first_seen.max(now() - 30 * 86400))
        .await
        .map_err(AdminApiError::internal)?;
    let mut breakdown = serde_json::Map::new();
    for (name, dim) in [("browser", "ua_family"), ("os", "os_family"), ("build", "build"), ("viewport", "viewport")] {
        let rows = state.store.client_error_breakdown(&id, dim).await.map_err(AdminApiError::internal)?;
        breakdown.insert(name.into(), json!(rows));
    }
    let mut routes: std::collections::BTreeMap<String, i64> = Default::default();
    for e in &events {
        let route = serde_json::from_str::<Value>(&e.extra)
            .ok()
            .and_then(|v| v.get("route").and_then(Value::as_str).map(str::to_owned))
            .unwrap_or_default();
        *routes.entry(route).or_default() += e.occurrences;
    }
    breakdown.insert("route".into(), json!(routes.into_iter().collect::<Vec<_>>()));
    Ok(([(header::CACHE_CONTROL, "no-store")], Json(json!({
        "group": group_json(&group), "events": out, "series": series, "breakdown": breakdown,
    })))
        .into_response())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StatusBody {
    status: String,
}

async fn set_status(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
    AdminJson(body): AdminJson<StatusBody>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsWrite)?;
    if !["open", "resolved", "ignored"].contains(&body.status.as_str()) {
        return Err(AdminApiError::invalid("invalidStatus", "Unknown status.", "/status"));
    }
    if !state.store.set_client_error_status(&id, &body.status, now()).await.map_err(AdminApiError::internal)? {
        return Err(AdminApiError::not_found("No such error group."));
    }
    refresh_open_gauge(&state.store).await;
    crate::admin_accounts::audit(&state, &ctx, "client-error-status", "observability:client-errors", None,
        Some(json!({"group": id, "status": body.status}))).await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn delete_group(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsWrite)?;
    if !state.store.delete_client_error_group(&id).await.map_err(AdminApiError::internal)? {
        return Err(AdminApiError::not_found("No such error group."));
    }
    refresh_open_gauge(&state.store).await;
    crate::admin_accounts::audit(&state, &ctx, "client-error-deleted", "observability:client-errors", None,
        Some(json!({"group": id}))).await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn export(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Query(q): Query<ListQuery>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::ObservabilityRead)?;
    let (mut filter, _) = filter_of(&q)?;
    filter.limit = 1000;
    let groups = state.store.list_client_error_groups(&filter).await.map_err(AdminApiError::internal)?;
    if q.format.as_deref() == Some("csv") {
        let mut text = String::from("id,status,kind,name,boundary,route,count,sessions,firstSeen,lastSeen,lastBuild,message\r\n");
        for g in &groups {
            let cells = [
                g.id.clone(), g.status.clone(), g.kind.clone(), g.name.clone(), g.boundary.clone(),
                g.route_template.clone(), g.count.to_string(), g.visits.to_string(),
                g.first_seen.to_string(), g.last_seen.to_string(), g.last_build.clone(), g.message.clone(),
            ];
            text.push_str(&cells.iter().map(|c| crate::cookie_consent::csv_field(c)).collect::<Vec<_>>().join(","));
            text.push_str("\r\n");
        }
        return Ok((
            [
                (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
                (header::CONTENT_DISPOSITION, "attachment; filename=\"client-errors.csv\""),
                (header::CACHE_CONTROL, "no-store"),
            ],
            text,
        )
            .into_response());
    }
    Ok((
        [
            (header::CONTENT_DISPOSITION, "attachment; filename=\"client-errors.json\""),
            (header::CACHE_CONTROL, "no-store"),
        ],
        Json(json!({ "groups": groups.iter().map(group_json).collect::<Vec<_>>() })),
    )
        .into_response())
}

// ---- retention ------------------------------------------------------------

/// Prune events and groups past their retention (called by the sweeper).
pub(crate) async fn prune(store: &Store, now: i64, dry_run: bool) -> Result<u64, mw_store::StoreError> {
    async fn days(store: &Store, key: &str, default: i64, max: i64) -> i64 {
        match store.get_setting_policy(key).await {
            Ok(Some(row)) => row
                .global_value
                .and_then(|v| serde_json::from_str::<Value>(&v).ok())
                .and_then(|v| v.as_i64())
                .filter(|n| (1..=max).contains(n))
                .unwrap_or(default),
            _ => default,
        }
    }
    let event_days = days(store, EVENT_DAYS_KEY, 30, 3650).await;
    let group_days = days(store, GROUP_DAYS_KEY, 180, 3650).await.max(event_days);
    if dry_run {
        return Ok(0);
    }
    let n = store.prune_client_errors(now - event_days * 86400, now - group_days * 86400).await?;
    refresh_open_gauge(store).await;
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queries_and_hashes_are_removed_from_urls() {
        assert_eq!(
            strip_queries("at f (https://h/assets/a.js?v=1&t=SECRET#frag:1:2)"),
            "at f (https://h/assets/a.js:1:2)"
        );
        assert_eq!(strip_queries("GET /api/x?token=abc failed"), "GET /api/x failed");
        assert_eq!(strip_queries("what? really"), "what? really");
    }

    #[test]
    fn messages_lose_addresses_and_tokens() {
        let m = scrub_message("bad mail alice@example.com key eyJhbGciOiJIUzI1NiJ9abcdef123456789 ok", 300);
        assert!(!m.contains("alice@example.com") && !m.contains("eyJhbG"), "{m}");
        assert!(m.contains("[token]"));
        assert!(scrub_message(&"x".repeat(2000), 300).len() <= 304);
    }

    #[test]
    fn origins_are_dropped_from_frames() {
        assert_eq!(strip_origins("at f (https://mail.example.test/assets/a.js:1:2)"), "at f (/assets/a.js:1:2)");
        assert_eq!(strip_origins("no urls here"), "no urls here");
        assert_eq!(
            scrub_stack("E
    at f (https://h.test:8443/assets/a.js?v=1:3:4)", 1000),
            "E
    at f (/assets/a.js:3:4)"
        );
    }

    #[test]
    fn routes_become_templates() {
        assert_eq!(scrub_route("#/mail/INBOX/m-0a1b2c3d4e5f6a7b?q=secret"), "/mail/INBOX/:id");
        assert_eq!(scrub_route("/settings/privacy"), "/settings/privacy");
        assert_eq!(scrub_route("#/message/12345"), "/message/:id");
    }
}
