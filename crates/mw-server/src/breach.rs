//! Personal-data breach register: the admin endpoints, the notice clock worker,
//! the supervisory-authority notice and data-subject communication, automations
//! and the containment playbooks.
//!
//! An incident is opened by hand, by an automation rule (`breach.auto.*`) or from
//! a data request. The notice clock starts when the awareness time is set
//! (`aware_at`); the deadline is that time plus `breach.noticeHours`. Every change
//! is an entry in the incident's trail and a security event. Mailwoman prepares
//! the authority notice and sends data-subject notices through the channels the
//! settings allow; it never submits anything to an authority. The questionnaire
//! and the clock are aids: whether an incident must be notified is for the
//! operator to decide. See `docs/admin/breach-register.md`.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx},
    admin_router::AdminRegistrar,
    audit::{SecurityEvent, security_event},
    breach_model::{
        self as m, AuthorityCfg, AuthorityRecord, Body, Entry, Evidence, Gate, Phase, Reminder, SubjectNoticeRecord,
    },
    capabilities::Capability as C,
    settings_registry,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use mw_store::{
    breach::{BreachDelivery, BreachRow},
    settings_policy::SettingSubject,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Mutex, OnceLock};

const DAY: i64 = 86_400;
const NOTICE_PREFIX: &str = "breach-notice:v1:";
const CURSOR_KEY: &str = "breach:auto:cursor";
const MAX_TEXT: usize = 8000;

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

fn clip(s: &str, max: usize) -> String {
    s.trim().chars().take(max).collect()
}

fn bad(code: &'static str, detail: &str, field: &str) -> AdminApiError {
    AdminApiError::invalid(code, detail.to_string(), field.to_string())
}

fn store_err(e: mw_store::StoreError) -> AdminApiError {
    tracing::warn!("breach register store error: {e}");
    AdminApiError::new(StatusCode::SERVICE_UNAVAILABLE, "unavailable", "The breach register is unavailable.")
}

// ─── policy ───────────────────────────────────────────────────────────────────

/// The effective `breach.*` values for one domain (or the instance).
#[derive(Clone, Debug, Default)]
pub(crate) struct Policy {
    map: HashMap<String, Value>,
}

impl Policy {
    pub(crate) async fn load(state: &AppState, domain: &str) -> Self {
        let subject = SettingSubject { personal_owner: None, domain, account_id: "", address: "" };
        let rows = settings_registry::effective_all(&state.store, &subject, |d| d.key.starts_with("breach."))
            .await
            .unwrap_or_default();
        Self { map: rows.into_iter().map(|e| (e.key, e.value)).collect() }
    }
    fn get(&self, key: &str) -> Value {
        self.map
            .get(key)
            .cloned()
            .or_else(|| settings_registry::lookup(key).map(|d| d.default_value()))
            .unwrap_or(Value::Null)
    }
    pub(crate) fn b(&self, key: &str) -> bool {
        self.get(key).as_bool().unwrap_or(false)
    }
    pub(crate) fn n(&self, key: &str) -> i64 {
        self.get(key).as_i64().unwrap_or(0)
    }
    pub(crate) fn s(&self, key: &str) -> String {
        self.get(key).as_str().unwrap_or_default().to_string()
    }
    fn strings(&self, key: &str) -> Vec<String> {
        self.get(key)
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(str::trim).filter(|s| !s.is_empty()).map(str::to_string)).collect())
            .unwrap_or_default()
    }
    fn ints(&self, key: &str) -> Vec<i64> {
        self.get(key).as_array().map(|a| a.iter().filter_map(Value::as_i64).collect()).unwrap_or_default()
    }
    fn hours(&self) -> i64 {
        self.n("breach.noticeHours").clamp(1, 720)
    }
    fn authority(&self) -> AuthorityCfg {
        AuthorityCfg {
            name: self.s("breach.authority.name"),
            url: self.s("breach.authority.url"),
            contact: self.s("breach.authority.contact"),
            dpo_name: self.s("breach.dpo.name"),
            dpo_email: self.s("breach.dpo.email"),
            dpo_phone: self.s("breach.dpo.phone"),
        }
    }
    /// A localizable template: the setting for the language, then English, then
    /// the built-in text for the language.
    fn template(&self, kind: &str, locale: &str) -> String {
        let key = format!("breach.template.{kind}");
        let v = self.get(&key);
        let loc = m::locale_of(locale);
        if let Some(map) = v.as_object() {
            for k in [loc, "en"] {
                if let Some(t) = map.get(k).and_then(Value::as_str).filter(|t| !t.trim().is_empty()) {
                    return t.to_string();
                }
            }
        }
        m::default_template(kind, loc).to_string()
    }
}

// ─── incidents ────────────────────────────────────────────────────────────────

#[derive(Deserialize, Default, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct NewIncident {
    pub title: String,
    pub description: String,
    pub source: String,
    pub incident_type: String,
    pub domain: String,
    pub detected_at: Option<i64>,
    /// Start the clock at this time. `None` with `draft` false starts it at the
    /// detection time.
    pub aware_at: Option<i64>,
    pub draft: bool,
    pub reporter: String,
    pub assignee: Option<String>,
    pub affected: Vec<String>,
    pub affected_domains: Vec<String>,
    pub subject_categories: Vec<String>,
    pub data_categories: Vec<String>,
    pub approx_subjects: Option<i64>,
    pub approx_records: Option<i64>,
    pub consequences: String,
    pub measures_taken: String,
    pub measures_proposed: String,
    #[serde(skip)]
    pub auto: Option<Value>,
}

async fn resolve_affected(state: &AppState, addresses: &[String]) -> Vec<m::Affected> {
    let mut out: Vec<m::Affected> = Vec::new();
    for raw in addresses.iter().take(2000) {
        let a = raw.trim();
        if a.is_empty() {
            continue;
        }
        let address = state.store.canonical_policy_address(a).await.unwrap_or_else(|_| a.to_lowercase());
        if out.iter().any(|x| x.address == address) {
            continue;
        }
        let account_id = state.store.lifecycle_account_bindings(&address).await.ok().and_then(|v| v.first().cloned());
        out.push(m::Affected { address, account_id });
    }
    out
}

fn entry(actor: &str, kind: &str, text: &str, at: i64) -> Entry {
    Entry { id: uuid::Uuid::new_v4().to_string(), ts: at, actor: actor.into(), kind: kind.into(), text: clip(text, MAX_TEXT) }
}

async fn event(state: &AppState, id: &str, actor: &str, kind: &str, detail: Value) {
    if let Err(e) = state.store.breach_event_add(id, actor, kind, &detail, now()).await {
        tracing::warn!("breach trail entry not recorded: {e}");
    }
}

/// A change to an incident, as a security event in the audit log.
async fn audit(state: &AppState, actor: &str, id: &str, kind: &str, detail: Value) {
    let base = if actor == "system" { SecurityEvent::system("breach-changed") } else { SecurityEvent::admin(actor, "breach-changed") };
    security_event(state, base.target(format!("breach:{id}")).detail(json!({"kind": kind, "detail": detail}))).await;
}

/// Write the incident under its `rev`, add the trail entry and the audit event.
async fn save(state: &AppState, row: &mut BreachRow, body: &Body, actor: &str, kind: &str, detail: Value) -> Result<(), AdminApiError> {
    row.body_json = serde_json::to_string(body).map_err(AdminApiError::internal)?;
    let at = now();
    if !state.store.breach_update(row, at).await.map_err(store_err)? {
        return Err(AdminApiError::conflict("changed", "The incident changed while you were working on it; reload and try again."));
    }
    row.rev += 1;
    row.updated_at = at;
    event(state, &row.id, actor, kind, detail.clone()).await;
    audit(state, actor, &row.id, kind, detail).await;
    Ok(())
}

/// Open an incident. The clock starts when an awareness time is known.
pub(crate) async fn open_incident(state: &AppState, n: NewIncident, actor: &str, at: i64) -> Result<(BreachRow, Body), AdminApiError> {
    let title = clip(&n.title, 200);
    if title.is_empty() {
        return Err(bad("titleRequired", "Give the incident a title.", "/title"));
    }
    let source = if n.source.is_empty() { "manual".to_string() } else { n.source.clone() };
    if !m::SOURCES.contains(&source.as_str()) {
        return Err(bad("invalidSource", "Unknown source.", "/source"));
    }
    let incident_type = if n.incident_type.is_empty() { "other".to_string() } else { n.incident_type.clone() };
    if !m::TYPES.contains(&incident_type.as_str()) {
        return Err(bad("invalidType", "Unknown incident type.", "/incidentType"));
    }
    let domain = n.domain.trim().to_lowercase();
    let policy = Policy::load(state, &domain).await;
    if !policy.b("breach.enabled") {
        return Err(AdminApiError::conflict("disabled", "The breach register is switched off for this domain."));
    }
    let detected_at = n.detected_at.unwrap_or(at);
    let aware_at = if n.draft { None } else { Some(n.aware_at.unwrap_or(detected_at)) };
    let id = uuid::Uuid::new_v4().to_string();
    let mut body = Body {
        incident_type,
        description: clip(&n.description, MAX_TEXT),
        reporter: if n.reporter.trim().is_empty() { actor.to_string() } else { clip(&n.reporter, 200) },
        subject_categories: n.subject_categories.iter().map(|s| clip(s, 120)).filter(|s| !s.is_empty()).take(50).collect(),
        data_categories: n.data_categories.iter().map(|s| clip(s, 120)).filter(|s| !s.is_empty()).take(50).collect(),
        approx_subjects: n.approx_subjects.filter(|v| *v >= 0),
        approx_records: n.approx_records.filter(|v| *v >= 0),
        consequences: clip(&n.consequences, MAX_TEXT),
        measures_taken: clip(&n.measures_taken, MAX_TEXT),
        measures_proposed: clip(&n.measures_proposed, MAX_TEXT),
        affected: resolve_affected(state, &n.affected).await,
        affected_domains: n.affected_domains.iter().map(|s| clip(s, 253).to_lowercase()).filter(|s| !s.is_empty()).take(100).collect(),
        auto: n.auto.clone(),
        ..Body::default()
    };
    body.timeline.push(entry(actor, "detection", &format!("Detected ({source})."), detected_at));
    if aware_at.is_some() {
        body.confirmed_by = Some(actor.to_string());
        body.confirmed_at = Some(at);
    }
    let mut row = BreachRow {
        id: id.clone(),
        title,
        domain,
        status: "detected".into(),
        severity: "unassessed".into(),
        source,
        detected_at,
        aware_at,
        deadline_at: aware_at.map(|a| m::deadline(a, policy.hours())),
        assignee: n.assignee.map(|a| clip(&a, 120)).filter(|a| !a.is_empty()),
        created_at: at,
        updated_at: at,
        rev: 1,
        body_json: String::new(),
    };
    row.body_json = serde_json::to_string(&body).map_err(AdminApiError::internal)?;
    state.store.breach_insert(&row).await.map_err(store_err)?;
    let detail = json!({"source": row.source, "draft": aware_at.is_none(), "awareAt": aware_at, "deadlineAt": row.deadline_at});
    event(state, &id, actor, "created", detail.clone()).await;
    audit(state, actor, &id, "created", detail).await;
    Ok((row, body))
}

// ─── views ────────────────────────────────────────────────────────────────────

fn notified_at(body: &Body) -> Option<i64> {
    body.authority_notified_at()
}

fn clock_of(row: &BreachRow, body: &Body, at: i64) -> m::ClockView {
    m::clock(row.aware_at, row.deadline_at, notified_at(body), &row.status, at)
}

fn summary(row: &BreachRow, body: &Body, at: i64) -> Value {
    let c = clock_of(row, body, at);
    json!({
        "id": row.id, "reference": m::short_ref(&row.id), "title": row.title, "domain": row.domain,
        "status": row.status, "severity": row.severity, "source": row.source, "type": body.incident_type,
        "detectedAt": row.detected_at, "awareAt": row.aware_at, "deadlineAt": row.deadline_at,
        "assignee": row.assignee, "updatedAt": row.updated_at, "clock": c,
        "authorityNotifiedAt": notified_at(body), "affectedCount": body.affected.len(),
        "draft": row.aware_at.is_none() && m::is_open(&row.status),
    })
}

fn view(row: &BreachRow, body: &Body, policy: &Policy, at: i64) -> Value {
    let c = clock_of(row, body, at);
    let suggestion = m::suggest(&body.answers);
    let needs_delay = row.deadline_at.is_some_and(|d| at > d) && body.authority_notices.is_empty() && m::is_open(&row.status);
    let items: Vec<Value> = m::playbook(&body.incident_type)
        .into_iter()
        .map(|id| {
            let s = body.playbook.get(id).cloned().unwrap_or_default();
            json!({"id": id, "done": s.done, "by": s.by, "at": s.at})
        })
        .collect();
    let mut v = summary(row, body, at);
    let obj = v.as_object_mut().expect("object");
    obj.insert("rev".into(), json!(row.rev));
    obj.insert("createdAt".into(), json!(row.created_at));
    obj.insert("body".into(), serde_json::to_value(body).unwrap_or(Value::Null));
    obj.insert("suggestion".into(), serde_json::to_value(suggestion).unwrap_or(Value::Null));
    obj.insert("playbook".into(), json!(items));
    obj.insert("noticeHours".into(), json!(policy.hours()));
    obj.insert("delayReasonsRequired".into(), json!(needs_delay));
    obj.insert("overdue".into(), json!(c.phase == Phase::Overdue));
    v
}

async fn load(state: &AppState, id: &str) -> Result<(BreachRow, Body, Policy), AdminApiError> {
    let row = state.store.breach_get(id).await.map_err(store_err)?.ok_or_else(|| AdminApiError::not_found("No such incident."))?;
    let body = Body::of(&row);
    let policy = Policy::load(state, &row.domain).await;
    Ok((row, body, policy))
}

fn no_store(r: impl IntoResponse) -> Response {
    ([(header::CACHE_CONTROL, "no-store")], r).into_response()
}

// ─── admin: list, create, read ────────────────────────────────────────────────

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ListQuery {
    status: Option<String>,
    severity: Option<String>,
    domain: Option<String>,
    q: Option<String>,
    due: Option<String>,
    year: Option<i32>,
    format: Option<String>,
}

fn year_of(ts: i64) -> i32 {
    use chrono::Datelike;
    chrono::DateTime::from_timestamp(ts, 0).map(|d| d.year()).unwrap_or(1970)
}

async fn filtered(state: &AppState, q: &ListQuery) -> Result<Vec<(BreachRow, Body)>, AdminApiError> {
    let at = now();
    let needle = q.q.as_deref().map(str::to_lowercase).filter(|s| !s.is_empty());
    Ok(state
        .store
        .breach_list()
        .await
        .map_err(store_err)?
        .into_iter()
        .filter(|r| match q.status.as_deref() {
            None | Some("") => true,
            Some("open") => m::is_open(&r.status),
            Some("draft") => r.aware_at.is_none() && m::is_open(&r.status),
            Some(s) => r.status == s,
        })
        .filter(|r| q.severity.as_deref().is_none_or(|s| s.is_empty() || s == r.severity))
        .filter(|r| q.domain.as_deref().is_none_or(|d| d.is_empty() || d == r.domain))
        .filter(|r| q.year.is_none_or(|y| year_of(r.detected_at) == y))
        .filter(|r| needle.as_ref().is_none_or(|n| r.title.to_lowercase().contains(n.as_str()) || r.id.starts_with(n.as_str()) || m::short_ref(&r.id).to_lowercase().contains(n.as_str())))
        .map(|r| {
            let b = Body::of(&r);
            (r, b)
        })
        .filter(|(r, b)| match q.due.as_deref() {
            Some("overdue") => clock_of(r, b, at).phase == Phase::Overdue,
            Some("soon") => clock_of(r, b, at).remaining_secs.is_some_and(|s| (0..=24 * 3600).contains(&s)),
            _ => true,
        })
        .collect())
}

fn stats(all: &[(BreachRow, Body)], at: i64) -> Value {
    let mut by_status: BTreeMap<String, i64> = BTreeMap::new();
    let mut by_severity: BTreeMap<String, i64> = BTreeMap::new();
    let (mut open, mut overdue, mut soon, mut drafts, mut late) = (0, 0, 0, 0, 0);
    let mut created = vec![0_i64; 30];
    for (r, b) in all {
        *by_status.entry(r.status.clone()).or_default() += 1;
        *by_severity.entry(r.severity.clone()).or_default() += 1;
        let c = clock_of(r, b, at);
        if m::is_open(&r.status) {
            open += 1;
        }
        if r.aware_at.is_none() && m::is_open(&r.status) {
            drafts += 1;
        }
        match c.phase {
            Phase::Overdue => overdue += 1,
            Phase::Running if c.remaining_secs.is_some_and(|s| s <= 24 * 3600) => soon += 1,
            Phase::StoppedLate => late += 1,
            _ => {}
        }
        let day = (at - r.detected_at) / DAY;
        if (0..30).contains(&day) {
            created[(29 - day) as usize] += 1;
        }
    }
    let today = at - at.rem_euclid(DAY);
    json!({
        "total": all.len(), "open": open, "overdue": overdue, "dueSoon": soon, "drafts": drafts, "notifiedLate": late,
        "byStatus": by_status, "bySeverity": by_severity,
        "created30d": created.iter().enumerate().map(|(i, c)| json!({"t": today - (29 - i as i64) * DAY, "count": c})).collect::<Vec<_>>(),
    })
}

async fn admin_list(State(state): State<AppState>, _ctx: AdminCtx, Query(q): Query<ListQuery>) -> Result<Response, AdminApiError> {
    let at = now();
    let everything = filtered(&state, &ListQuery::default()).await?;
    let rows = filtered(&state, &q).await?;
    let list: Vec<Value> = rows.iter().take(500).map(|(r, b)| summary(r, b, at)).collect();
    Ok(no_store(Json(json!({"incidents": list, "matched": rows.len(), "stats": stats(&everything, at)}))))
}

async fn admin_meta(State(state): State<AppState>, _ctx: AdminCtx) -> Result<Response, AdminApiError> {
    let policy = Policy::load(&state, "").await;
    let questions: Vec<Value> = m::QUESTIONS
        .iter()
        .map(|(id, options)| json!({"id": id, "options": options.iter().map(|(name, _)| name).collect::<Vec<_>>()}))
        .collect();
    Ok(no_store(Json(json!({
        "statuses": m::STATUSES, "sources": m::SOURCES, "severities": m::SEVERITIES, "types": m::TYPES,
        "phases": m::PHASES, "questions": questions, "variables": m::VARIABLES,
        "rules": m::RULES.iter().map(|r| json!({"id": r.id, "key": r.key, "type": r.incident_type})).collect::<Vec<_>>(),
        "defaults": {
            "subject": {"en": m::DEFAULT_SUBJECT_EN, "pt-PT": m::DEFAULT_SUBJECT_PT},
            "body": {"en": m::DEFAULT_BODY_EN, "pt-PT": m::DEFAULT_BODY_PT},
        },
        "enabled": policy.b("breach.enabled"), "noticeHours": policy.hours(),
    }))))
}

async fn admin_create(State(state): State<AppState>, ctx: AdminCtx, Json(req): Json<NewIncident>) -> Result<Response, AdminApiError> {
    let at = now();
    let (row, body) = open_incident(&state, req, &ctx.username, at).await?;
    let policy = Policy::load(&state, &row.domain).await;
    Ok((StatusCode::CREATED, no_store(Json(json!({"incident": view(&row, &body, &policy, at)})))).into_response())
}

async fn admin_get(State(state): State<AppState>, _ctx: AdminCtx, Path(id): Path<String>) -> Result<Response, AdminApiError> {
    let (row, body, policy) = load(&state, &id).await?;
    let events = state.store.breach_events(&id).await.map_err(store_err)?;
    let deliveries = state.store.breach_deliveries(&id).await.map_err(store_err)?;
    Ok(no_store(Json(json!({"incident": view(&row, &body, &policy, now()), "trail": events, "deliveries": deliveries}))))
}

async fn admin_export(State(state): State<AppState>, ctx: AdminCtx, Query(q): Query<ListQuery>) -> Result<Response, AdminApiError> {
    let rows = filtered(&state, &q).await?;
    let register: Vec<Value> = rows.iter().map(|(r, b)| m::register_row(r, b)).collect();
    let name = q.year.map_or("breach-register".to_string(), |y| format!("breach-register-{y}"));
    audit(&state, &ctx.username, "register", "exported", json!({"year": q.year, "rows": register.len(), "format": q.format})).await;
    let (ctype, ext, bytes): (&str, &str, Vec<u8>) = match q.format.as_deref() {
        Some("json") => ("application/json", "json", serde_json::to_vec_pretty(&json!({"generatedAt": now(), "year": q.year, "count": register.len(), "incidents": register})).unwrap_or_default()),
        Some("html") => {
            let mut s = String::from("<!doctype html><html><head><meta charset=\"utf-8\"><title>Breach register</title><style>body{font:13px system-ui,sans-serif;margin:1.5rem}table{border-collapse:collapse}th,td{border:1px solid #999;padding:.25rem .45rem;text-align:start;vertical-align:top}</style></head><body><h1>Breach register</h1><table><tr>");
            for c in m::CSV_COLUMNS {
                s.push_str(&format!("<th>{c}</th>"));
            }
            s.push_str("</tr>");
            for r in &register {
                s.push_str("<tr>");
                for c in m::CSV_COLUMNS {
                    let t = match &r[c] {
                        Value::String(t) => t.clone(),
                        Value::Null => String::new(),
                        other => other.to_string(),
                    };
                    s.push_str(&format!("<td>{}</td>", t.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")));
                }
                s.push_str("</tr>");
            }
            s.push_str("</table></body></html>");
            ("text/html; charset=utf-8", "html", s.into_bytes())
        }
        _ => ("text/csv; charset=utf-8", "csv", crate::dsr_data::csv(&register, &m::CSV_COLUMNS)),
    };
    Ok((
        [
            (header::CONTENT_TYPE, ctype.to_string()),
            (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{name}.{ext}\"")),
            (header::CACHE_CONTROL, "no-store".to_string()),
        ],
        bytes,
    )
        .into_response())
}

#[derive(Deserialize, Default)]
struct FormatQuery {
    format: Option<String>,
    locale: Option<String>,
    phase: Option<String>,
}

async fn admin_report(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>, Query(q): Query<FormatQuery>) -> Result<Response, AdminApiError> {
    let (row, body, policy) = load(&state, &id).await?;
    let events = state.store.breach_events(&id).await.map_err(store_err)?;
    let deliveries = state.store.breach_deliveries(&id).await.map_err(store_err)?;
    audit(&state, &ctx.username, &id, "report-exported", json!({"format": q.format})).await;
    let reference = m::short_ref(&id);
    let (ctype, ext, bytes): (&str, &str, Vec<u8>) = match q.format.as_deref() {
        Some("json") => (
            "application/json",
            "json",
            serde_json::to_vec_pretty(&json!({"incident": view(&row, &body, &policy, now()), "trail": events, "deliveries": deliveries})).unwrap_or_default(),
        ),
        Some("csv") => ("text/csv; charset=utf-8", "csv", crate::dsr_data::csv(&[m::register_row(&row, &body)], &m::CSV_COLUMNS)),
        _ => ("text/html; charset=utf-8", "html", m::report_html(&row, &body, &events, q.locale.as_deref().unwrap_or("en")).into_bytes()),
    };
    Ok((
        [
            (header::CONTENT_TYPE, ctype.to_string()),
            (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{reference}.{ext}\"")),
            (header::CACHE_CONTROL, "no-store".to_string()),
        ],
        bytes,
    )
        .into_response())
}

// ─── admin: edits ─────────────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct Details {
    title: Option<String>,
    description: Option<String>,
    incident_type: Option<String>,
    domain: Option<String>,
    reporter: Option<String>,
    subject_categories: Option<Vec<String>>,
    data_categories: Option<Vec<String>>,
    approx_subjects: Option<Option<i64>>,
    approx_records: Option<Option<i64>>,
    consequences: Option<String>,
    measures_taken: Option<String>,
    measures_proposed: Option<String>,
    affected: Option<Vec<String>>,
    affected_domains: Option<Vec<String>>,
    delay_reasons: Option<String>,
}

async fn admin_details(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>, Json(req): Json<Details>) -> Result<Response, AdminApiError> {
    let (mut row, mut body, _) = load(&state, &id).await?;
    let mut changed: Vec<&str> = Vec::new();
    if let Some(t) = req.title {
        let t = clip(&t, 200);
        if t.is_empty() {
            return Err(bad("titleRequired", "Give the incident a title.", "/title"));
        }
        row.title = t;
        changed.push("title");
    }
    if let Some(t) = req.incident_type {
        if !m::TYPES.contains(&t.as_str()) {
            return Err(bad("invalidType", "Unknown incident type.", "/incidentType"));
        }
        body.incident_type = t;
        changed.push("incidentType");
    }
    if let Some(d) = req.domain {
        row.domain = d.trim().to_lowercase();
        changed.push("domain");
    }
    macro_rules! text {
        ($field:ident, $name:literal) => {
            if let Some(v) = req.$field {
                body.$field = clip(&v, MAX_TEXT);
                changed.push($name);
            }
        };
    }
    text!(description, "description");
    text!(reporter, "reporter");
    text!(consequences, "consequences");
    text!(measures_taken, "measuresTaken");
    text!(measures_proposed, "measuresProposed");
    text!(delay_reasons, "delayReasons");
    if let Some(v) = req.subject_categories {
        body.subject_categories = v.iter().map(|s| clip(s, 120)).filter(|s| !s.is_empty()).take(50).collect();
        changed.push("subjectCategories");
    }
    if let Some(v) = req.data_categories {
        body.data_categories = v.iter().map(|s| clip(s, 120)).filter(|s| !s.is_empty()).take(50).collect();
        changed.push("dataCategories");
    }
    if let Some(v) = req.approx_subjects {
        body.approx_subjects = v.filter(|n| *n >= 0);
        changed.push("approxSubjects");
    }
    if let Some(v) = req.approx_records {
        body.approx_records = v.filter(|n| *n >= 0);
        changed.push("approxRecords");
    }
    if let Some(v) = req.affected {
        body.affected = resolve_affected(&state, &v).await;
        changed.push("affected");
    }
    if let Some(v) = req.affected_domains {
        body.affected_domains = v.iter().map(|s| clip(s, 253).to_lowercase()).filter(|s| !s.is_empty()).take(100).collect();
        changed.push("affectedDomains");
    }
    if changed.is_empty() {
        return Err(bad("nothingToChange", "No field was given.", "/"));
    }
    // A domain change can change the window; the clock follows the new policy.
    let policy = Policy::load(&state, &row.domain).await;
    if let Some(a) = row.aware_at {
        row.deadline_at = Some(m::deadline(a, policy.hours()));
    }
    save(&state, &mut row, &body, &ctx.username, "updated", json!({"fields": changed})).await?;
    Ok(no_store(Json(json!({"incident": view(&row, &body, &policy, now())}))))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AssignReq {
    assignee: Option<String>,
}

async fn admin_assign(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>, Json(req): Json<AssignReq>) -> Result<Response, AdminApiError> {
    let (mut row, body, policy) = load(&state, &id).await?;
    row.assignee = req.assignee.map(|a| clip(&a, 120)).filter(|a| !a.is_empty());
    let detail = json!({"assignee": row.assignee});
    save(&state, &mut row, &body, &ctx.username, "assigned", detail).await?;
    Ok(no_store(Json(json!({"incident": view(&row, &body, &policy, now())}))))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct AwareReq {
    aware_at: Option<i64>,
    reason: String,
}

/// Start the clock now or at a given time (a draft becomes a running incident).
async fn admin_confirm(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>, Json(req): Json<AwareReq>) -> Result<Response, AdminApiError> {
    let (mut row, mut body, policy) = load(&state, &id).await?;
    if row.aware_at.is_some() {
        return Err(AdminApiError::conflict("alreadyStarted", "The clock is already running; change the awareness time instead."));
    }
    let at = now();
    let aware = req.aware_at.unwrap_or(at);
    if aware > at + 300 {
        return Err(bad("futureTime", "The awareness time cannot be in the future.", "/awareAt"));
    }
    row.aware_at = Some(aware);
    row.deadline_at = Some(m::deadline(aware, policy.hours()));
    body.confirmed_by = Some(ctx.username.clone());
    body.confirmed_at = Some(at);
    body.timeline.push(entry(&ctx.username, "note", "Incident confirmed; notice clock started.", at));
    let detail = json!({"awareAt": aware, "deadlineAt": row.deadline_at});
    save(&state, &mut row, &body, &ctx.username, "confirmed", detail).await?;
    notify_background(&state, &policy, &row, "confirmed", &body);
    Ok(no_store(Json(json!({"incident": view(&row, &body, &policy, now())}))))
}

async fn admin_aware_at(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>, Json(req): Json<AwareReq>) -> Result<Response, AdminApiError> {
    let (mut row, mut body, policy) = load(&state, &id).await?;
    let Some(new) = req.aware_at else {
        return Err(bad("awareAtRequired", "Give the new awareness time.", "/awareAt"));
    };
    if req.reason.trim().chars().count() < 5 {
        return Err(bad("reasonRequired", "Record why the awareness time changes.", "/reason"));
    }
    if new > now() + 300 {
        return Err(bad("futureTime", "The awareness time cannot be in the future.", "/awareAt"));
    }
    let from = row.aware_at;
    row.aware_at = Some(new);
    row.deadline_at = Some(m::deadline(new, policy.hours()));
    // Reminders are measured from the new deadline.
    body.reminded.clear();
    body.overdue_reminded_at = None;
    body.timeline.push(entry(&ctx.username, "note", &format!("Awareness time changed: {}", clip(&req.reason, 500)), now()));
    let detail = json!({"from": from, "to": new, "deadlineAt": row.deadline_at, "reason": clip(&req.reason, 500)});
    save(&state, &mut row, &body, &ctx.username, "aware-at-changed", detail).await?;
    Ok(no_store(Json(json!({"incident": view(&row, &body, &policy, now())}))))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct AssessReq {
    answers: BTreeMap<String, String>,
    severity: Option<String>,
    rationale: String,
}

async fn admin_assess(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>, Json(req): Json<AssessReq>) -> Result<Response, AdminApiError> {
    let (mut row, mut body, policy) = load(&state, &id).await?;
    for (q, a) in &req.answers {
        let ok = m::QUESTIONS.iter().any(|(id, opts)| id == q && opts.iter().any(|(name, _)| name == a));
        if !ok {
            return Err(bad("invalidAnswer", "Unknown question or answer.", &format!("/answers/{q}")));
        }
    }
    body.answers.extend(req.answers.clone());
    let s = m::suggest(&body.answers);
    let chosen = match req.severity.as_deref() {
        Some(c) if m::SEVERITIES.contains(&c) => c.to_string(),
        Some(_) => return Err(bad("invalidSeverity", "Unknown risk level.", "/severity")),
        None => s.level.to_string(),
    };
    let differs = chosen != s.level;
    if differs && req.rationale.trim().chars().count() < 10 {
        return Err(bad("rationaleRequired", "Explain why the level differs from the questionnaire result.", "/rationale"));
    }
    body.assessment = Some(m::Assessment {
        score: s.score,
        suggested: s.level.to_string(),
        chosen: chosen.clone(),
        rationale: clip(&req.rationale, MAX_TEXT),
        by: ctx.username.clone(),
        at: now(),
        complete: s.complete,
    });
    let before = row.severity.clone();
    row.severity = chosen.clone();
    if row.status == "detected" {
        row.status = "assessing".into();
    }
    let detail = json!({"from": before, "to": chosen, "suggested": s.level, "score": s.score, "complete": s.complete});
    save(&state, &mut row, &body, &ctx.username, "assessed", detail).await?;
    Ok(no_store(Json(json!({"incident": view(&row, &body, &policy, now())}))))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StatusReq {
    status: String,
    #[serde(default)]
    reason: String,
}

async fn admin_status(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>, Json(req): Json<StatusReq>) -> Result<Response, AdminApiError> {
    let (mut row, mut body, policy) = load(&state, &id).await?;
    let subjects_informed = body.subject_notices.iter().any(|n| !n.dry_run && n.recipients > 0) || body.timeline.iter().any(|e| e.kind == "subjects-informed");
    let gate = Gate {
        severity: &row.severity,
        authority_notified: !body.authority_notices.is_empty(),
        subjects_notified: subjects_informed,
        reason: &req.reason,
        body: &body,
    };
    if let Err(code) = m::check_move(&row.status, &req.status, &gate) {
        return Err(AdminApiError::new(StatusCode::CONFLICT, code, format!("The move to {} is not allowed yet ({code}).", req.status)));
    }
    let from = row.status.clone();
    match req.status.as_str() {
        "not-notifiable" => body.not_notifiable_reason = clip(&req.reason, MAX_TEXT),
        "closed" => {
            if !req.reason.trim().is_empty() {
                body.close_reason = clip(&req.reason, MAX_TEXT);
            }
            body.timeline.push(entry(&ctx.username, "closure", &req.reason, now()));
        }
        _ => {}
    }
    row.status = req.status.clone();
    let detail = json!({"from": from, "to": req.status, "reason": clip(&req.reason, 500)});
    save(&state, &mut row, &body, &ctx.username, "status-changed", detail).await?;
    Ok(no_store(Json(json!({"incident": view(&row, &body, &policy, now())}))))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TimelineReq {
    kind: String,
    text: String,
}

async fn admin_timeline(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>, Json(req): Json<TimelineReq>) -> Result<Response, AdminApiError> {
    if !["detection", "containment", "closure", "note", "subjects-informed"].contains(&req.kind.as_str()) {
        return Err(bad("invalidKind", "Unknown timeline entry type.", "/kind"));
    }
    if req.text.trim().is_empty() {
        return Err(bad("textRequired", "Write what happened.", "/text"));
    }
    let (mut row, mut body, policy) = load(&state, &id).await?;
    if body.timeline.len() >= 500 {
        return Err(AdminApiError::conflict("tooMany", "The timeline is full."));
    }
    body.timeline.push(entry(&ctx.username, &req.kind, &req.text, now()));
    save(&state, &mut row, &body, &ctx.username, "timeline-added", json!({"kind": req.kind})).await?;
    Ok(no_store(Json(json!({"incident": view(&row, &body, &policy, now())}))))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EvidenceReq {
    kind: String,
    label: String,
    value: String,
}

async fn admin_evidence(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>, Json(req): Json<EvidenceReq>) -> Result<Response, AdminApiError> {
    if !["text", "link"].contains(&req.kind.as_str()) {
        return Err(bad("invalidKind", "Evidence is text or a link.", "/kind"));
    }
    if req.kind == "link" && !(req.value.starts_with("https://") || req.value.starts_with("http://")) {
        return Err(bad("invalidLink", "A link starts with http:// or https://.", "/value"));
    }
    if req.label.trim().is_empty() || req.value.trim().is_empty() {
        return Err(bad("textRequired", "Give a label and a value.", "/value"));
    }
    let (mut row, mut body, policy) = load(&state, &id).await?;
    if body.evidence.len() >= 200 {
        return Err(AdminApiError::conflict("tooMany", "The evidence list is full."));
    }
    body.evidence.push(Evidence {
        id: uuid::Uuid::new_v4().to_string(),
        ts: now(),
        actor: ctx.username.clone(),
        kind: req.kind.clone(),
        label: clip(&req.label, 200),
        value: clip(&req.value, MAX_TEXT),
    });
    save(&state, &mut row, &body, &ctx.username, "evidence-added", json!({"kind": req.kind, "label": clip(&req.label, 80)})).await?;
    Ok(no_store(Json(json!({"incident": view(&row, &body, &policy, now())}))))
}

// ─── admin: authority notice ──────────────────────────────────────────────────

async fn admin_authority_notice(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>, Query(q): Query<FormatQuery>) -> Result<Response, AdminApiError> {
    let (row, body, policy) = load(&state, &id).await?;
    let phase = q.phase.as_deref().unwrap_or(if body.authority_notices.is_empty() { "initial" } else { "update" });
    if !m::PHASES.contains(&phase) {
        return Err(bad("invalidPhase", "Unknown phase.", "/phase"));
    }
    let notice = m::authority_notice(&row, &body, &policy.authority(), phase, body.authority_notices.len(), now());
    let reference = m::short_ref(&id);
    audit(&state, &ctx.username, &id, "authority-notice-generated", json!({"phase": phase, "format": q.format})).await;
    match q.format.as_deref() {
        Some("html") => Ok((
            [
                (header::CONTENT_TYPE, "text/html; charset=utf-8".to_string()),
                (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{reference}-authority-{phase}.html\"")),
                (header::CACHE_CONTROL, "no-store".to_string()),
            ],
            m::authority_html(&notice, q.locale.as_deref().unwrap_or("en")),
        )
            .into_response()),
        Some("download") | Some("json-file") => Ok((
            [
                (header::CONTENT_TYPE, "application/json".to_string()),
                (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{reference}-authority-{phase}.json\"")),
                (header::CACHE_CONTROL, "no-store".to_string()),
            ],
            serde_json::to_vec_pretty(&notice).unwrap_or_default(),
        )
            .into_response()),
        _ => Ok(no_store(Json(json!({"notice": notice, "phase": phase})))),
    }
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct RecordReq {
    phase: String,
    submitted_at: Option<i64>,
    reference: String,
    delay_reasons: String,
}

/// Record that the operator submitted a notice to the authority. Mailwoman does
/// not submit it; this keeps the evidence and stops the clock.
async fn admin_authority_record(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>, Json(req): Json<RecordReq>) -> Result<Response, AdminApiError> {
    let (mut row, mut body, policy) = load(&state, &id).await?;
    let Some(deadline) = row.deadline_at else {
        return Err(AdminApiError::conflict("clockNotStarted", "Confirm the incident (set when you became aware) before recording a notice."));
    };
    let phase = if req.phase.is_empty() { if body.authority_notices.is_empty() { "initial" } else { "update" } } else { req.phase.as_str() };
    if !m::PHASES.contains(&phase) {
        return Err(bad("invalidPhase", "Unknown phase.", "/phase"));
    }
    if phase != "initial" && body.authority_notices.is_empty() {
        return Err(bad("initialFirst", "Record the initial notice first.", "/phase"));
    }
    let at = now();
    let submitted = req.submitted_at.unwrap_or(at);
    if submitted > at + 300 {
        return Err(bad("futureTime", "The submission time cannot be in the future.", "/submittedAt"));
    }
    let late = submitted > deadline;
    if late && phase == "initial" && req.delay_reasons.trim().chars().count() < 10 && body.delay_reasons.trim().chars().count() < 10 {
        return Err(bad("delayReasonsRequired", "The notice is late; record the reasons for the delay.", "/delayReasons"));
    }
    if !req.delay_reasons.trim().is_empty() {
        body.delay_reasons = clip(&req.delay_reasons, MAX_TEXT);
    }
    let snapshot = m::authority_notice(&row, &body, &policy.authority(), phase, body.authority_notices.len(), submitted);
    body.authority_notices.push(AuthorityRecord {
        id: uuid::Uuid::new_v4().to_string(),
        phase: phase.to_string(),
        recorded_at: at,
        recorded_by: ctx.username.clone(),
        submitted_at: submitted,
        reference: clip(&req.reference, 200),
        late,
        delay_reasons: clip(&req.delay_reasons, MAX_TEXT),
        snapshot,
    });
    body.timeline.push(entry(&ctx.username, "note", &format!("Authority notice recorded ({phase}){}.", if late { ", after the deadline" } else { "" }), at));
    if matches!(row.status.as_str(), "detected" | "assessing" | "contained") {
        row.status = "notified-authority".into();
    }
    let detail = json!({"phase": phase, "submittedAt": submitted, "late": late, "reference": clip(&req.reference, 200)});
    save(&state, &mut row, &body, &ctx.username, "authority-notice-recorded", detail).await?;
    Ok(no_store(Json(json!({"incident": view(&row, &body, &policy, now())}))))
}

// ─── admin: data subject notice ───────────────────────────────────────────────

#[derive(Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase", default)]
struct Audience {
    /// all, selected or domain.
    kind: String,
    addresses: Vec<String>,
    domain: String,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct SubjectReq {
    audience: Audience,
    locale: String,
    dry_run: bool,
    advice: String,
    /// Replaces the stored template for this send (a preview of an edit).
    subject_template: Option<String>,
    body_template: Option<String>,
}

fn pick_audience(body: &Body, a: &Audience) -> Result<Vec<m::Affected>, AdminApiError> {
    match a.kind.as_str() {
        "" | "all" => Ok(body.affected.clone()),
        "domain" => {
            let d = a.domain.trim().to_lowercase();
            if d.is_empty() {
                return Err(bad("domainRequired", "Name the domain.", "/audience/domain"));
            }
            Ok(body.affected.iter().filter(|x| crate::admin_ctx::domain_of(&x.address) == d).cloned().collect())
        }
        "selected" => {
            let wanted: Vec<String> = a.addresses.iter().map(|s| s.trim().to_lowercase()).collect();
            let picked: Vec<m::Affected> = body.affected.iter().filter(|x| wanted.contains(&x.address.to_lowercase())).cloned().collect();
            if picked.len() != wanted.len() {
                return Err(bad("notAffected", "Every selected address must be on the affected list.", "/audience/addresses"));
            }
            Ok(picked)
        }
        _ => Err(bad("invalidAudience", "Unknown audience.", "/audience/kind")),
    }
}

fn variables(row: &BreachRow, body: &Body, policy: &Policy, advice: &str, recipient: &str) -> BTreeMap<String, String> {
    let cfg = policy.authority();
    let dpo = [cfg.dpo_name.as_str(), cfg.dpo_email.as_str(), cfg.dpo_phone.as_str()].iter().filter(|s| !s.is_empty()).copied().collect::<Vec<_>>().join(", ");
    let measures = [body.measures_taken.as_str(), body.measures_proposed.as_str()].iter().filter(|s| !s.is_empty()).copied().collect::<Vec<_>>().join(" ");
    let date = row.aware_at.and_then(|a| chrono::DateTime::from_timestamp(a, 0)).map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default();
    let reference = m::short_ref(&row.id);
    [
        ("incident", format!("{} ({reference})", row.title)),
        ("reference", reference),
        ("title", row.title.clone()),
        ("date", date),
        ("description", body.description.clone()),
        ("categories", body.data_categories.join(", ")),
        ("consequences", body.consequences.clone()),
        ("measures", measures),
        ("advice", advice.to_string()),
        ("dpo", dpo),
        ("dpoName", cfg.dpo_name),
        ("dpoEmail", cfg.dpo_email),
        ("dpoPhone", cfg.dpo_phone),
        ("authority", cfg.name),
        ("domain", row.domain.clone()),
        ("recipient", recipient.to_string()),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect()
}

async fn ui_locale(state: &AppState, account: &str) -> &'static str {
    let Some(address) = state.store.security_account_address(account).await.ok().flatten() else { return "en" };
    let domain = crate::admin_ctx::domain_of(&address);
    let subject = SettingSubject { personal_owner: None, domain: &domain, account_id: account, address: &address };
    let v = settings_registry::effective_for(&state.store, "ui.language", &subject).await.ok().flatten();
    m::locale_of(v.as_ref().and_then(|e| e.value.as_str()).unwrap_or("en"))
}

fn account_key(account: &str) -> String {
    crate::push_relay::hash_token(account)[..24].to_owned()
}

struct Plan {
    address: String,
    account_id: Option<String>,
    locale: String,
}

async fn admin_subject_notice(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>, Json(req): Json<SubjectReq>) -> Result<Response, AdminApiError> {
    let (mut row, mut body, policy) = load(&state, &id).await?;
    let audience = pick_audience(&body, &req.audience)?;
    if audience.is_empty() {
        return Err(bad("noRecipients", "The audience has no recipients.", "/audience"));
    }
    let in_app = policy.b("breach.subjects.inApp");
    let inbox = policy.b("breach.subjects.inbox");
    let webhook = policy.b("breach.subjects.webhook") && !policy.strings("breach.notify.webhookUrls").is_empty();
    let mut channels: Vec<&str> = Vec::new();
    if in_app {
        channels.push("in-app");
    }
    if inbox {
        channels.push("inbox");
    }
    if webhook {
        channels.push("webhook");
    }
    if channels.is_empty() {
        return Err(AdminApiError::conflict("noChannels", "Every data subject channel is switched off in the settings."));
    }
    let mut plans: Vec<Plan> = Vec::new();
    for a in &audience {
        let locale = if req.locale.trim().is_empty() {
            match &a.account_id {
                Some(acc) => ui_locale(&state, acc).await.to_string(),
                None => "en".to_string(),
            }
        } else {
            m::locale_of(&req.locale).to_string()
        };
        plans.push(Plan { address: a.address.clone(), account_id: a.account_id.clone(), locale });
    }
    let subject_tpl = |loc: &str| req.subject_template.clone().unwrap_or_else(|| policy.template("subject", loc));
    let body_tpl = |loc: &str| req.body_template.clone().unwrap_or_else(|| policy.template("body", loc));
    let first = &plans[0];
    let vars = variables(&row, &body, &policy, &clip(&req.advice, 2000), &first.address);
    let (st, bt) = (subject_tpl(&first.locale), body_tpl(&first.locale));
    let preview = json!({
        "subject": m::render(&st, &vars), "body": m::render(&bt, &vars), "locale": first.locale,
        "unknownVariables": { "subject": m::unknown_variables(&st), "body": m::unknown_variables(&bt) },
    });
    let recipients: Vec<Value> = plans
        .iter()
        .map(|p| {
            let mut ch: Vec<&str> = Vec::new();
            let mut skipped: Vec<Value> = Vec::new();
            for c in &channels {
                match (*c, &p.account_id) {
                    ("in-app" | "inbox", None) => skipped.push(json!({"channel": c, "reason": "no account"})),
                    _ => ch.push(c),
                }
            }
            json!({"address": p.address, "accountId": p.account_id, "locale": p.locale, "channels": ch, "skipped": skipped})
        })
        .collect();
    if req.dry_run {
        event(&state, &id, &ctx.username, "subject-notice-previewed", json!({"recipients": plans.len(), "audience": req.audience.kind})).await;
        return Ok(no_store(Json(json!({"dryRun": true, "recipients": recipients, "channels": channels, "preview": preview}))));
    }
    let notice_id = uuid::Uuid::new_v4().to_string();
    body.subject_notices.push(SubjectNoticeRecord {
        id: notice_id.clone(),
        ts: now(),
        by: ctx.username.clone(),
        audience: if req.audience.kind.is_empty() { "all".into() } else { req.audience.kind.clone() },
        locale: req.locale.clone(),
        recipients: plans.len() as i64,
        dry_run: false,
    });
    body.timeline.push(entry(&ctx.username, "note", &format!("Data subject notice queued for {} recipient(s).", plans.len()), now()));
    save(&state, &mut row, &body, &ctx.username, "subject-notice-sent", json!({"noticeId": notice_id, "recipients": plans.len(), "channels": channels})).await?;
    let job = NoticeJob {
        incident: row.clone(),
        body: body.clone(),
        notice_id: notice_id.clone(),
        plans,
        in_app,
        inbox,
        webhook,
        advice: clip(&req.advice, 2000),
        subject_template: req.subject_template.clone(),
        body_template: req.body_template.clone(),
    };
    let st = state.clone();
    let pol = policy.clone();
    tokio::spawn(async move { run_notice(&st, &pol, job).await });
    Ok((StatusCode::ACCEPTED, no_store(Json(json!({"noticeId": notice_id, "recipients": recipients, "channels": channels, "incident": view(&row, &body, &policy, now())})))).into_response())
}

struct NoticeJob {
    incident: BreachRow,
    body: Body,
    notice_id: String,
    plans: Vec<Plan>,
    in_app: bool,
    inbox: bool,
    webhook: bool,
    advice: String,
    subject_template: Option<String>,
    body_template: Option<String>,
}

async fn log_delivery(state: &AppState, job: &NoticeJob, recipient: &str, channel: &str, status: &str, detail: &str) {
    let d = BreachDelivery {
        id: uuid::Uuid::new_v4().to_string(),
        incident_id: job.incident.id.clone(),
        notice_id: job.notice_id.clone(),
        recipient: recipient.to_string(),
        channel: channel.to_string(),
        status: status.to_string(),
        detail: clip(detail, 300),
        ts: now(),
    };
    if let Err(e) = state.store.breach_delivery_add(&d).await {
        tracing::warn!("breach delivery log entry not recorded: {e}");
    }
}

/// Deliver one notice to every planned recipient, logging each channel's outcome.
async fn run_notice(state: &AppState, policy: &Policy, job: NoticeJob) {
    let mut rendered_first: Option<(String, String)> = None;
    for p in &job.plans {
        let vars = variables(&job.incident, &job.body, policy, &job.advice, &p.address);
        let subject = m::render(&job.subject_template.clone().unwrap_or_else(|| policy.template("subject", &p.locale)), &vars);
        let text = m::render(&job.body_template.clone().unwrap_or_else(|| policy.template("body", &p.locale)), &vars);
        if rendered_first.is_none() {
            rendered_first = Some((subject.clone(), text.clone()));
        }
        if job.in_app {
            match &p.account_id {
                None => log_delivery(state, &job, &p.address, "in-app", "skipped", "no account has signed in for this address").await,
                Some(acc) => {
                    let key = format!("{NOTICE_PREFIX}{}:{}", account_key(acc), job.notice_id);
                    let row = json!({"id": job.notice_id, "ts": now(), "reference": m::short_ref(&job.incident.id), "subject": subject, "body": text, "locale": p.locale, "acknowledged": false});
                    match state.store.set_setting(&key, &row.to_string()).await {
                        Ok(()) => log_delivery(state, &job, &p.address, "in-app", "sent", "").await,
                        Err(e) => log_delivery(state, &job, &p.address, "in-app", "failed", &e.to_string()).await,
                    }
                }
            }
        }
        if job.inbox {
            match &p.account_id {
                None => log_delivery(state, &job, &p.address, "inbox", "skipped", "no account has signed in for this address").await,
                Some(acc) => match crate::alerts::deliver_text(state, acc, &p.locale, &subject, &text, "").await {
                    Ok(()) => log_delivery(state, &job, &p.address, "inbox", "sent", "").await,
                    Err(e) => log_delivery(state, &job, &p.address, "inbox", "failed", &e).await,
                },
            }
        }
    }
    if job.webhook
        && let Some((subject, text)) = rendered_first
    {
        let payload = json!({
            "event": "breach.subject-notice", "reference": m::short_ref(&job.incident.id), "noticeId": job.notice_id,
            "recipients": job.plans.len(), "subject": subject, "body": text,
        });
        for url in policy.strings("breach.notify.webhookUrls") {
            let host = reqwest::Url::parse(&url).ok().and_then(|u| u.host_str().map(str::to_string)).unwrap_or_else(|| "invalid".into());
            match send_webhook(state, &url, "BreachSubjectNotice", &payload, mw_egress::ip_allowed).await {
                Ok(code) => log_delivery(state, &job, &format!("webhook:{host}"), "webhook", if code < 300 { "sent" } else { "failed" }, &format!("HTTP {code}")).await,
                Err(e) => log_delivery(state, &job, &format!("webhook:{host}"), "webhook", "failed", &e).await,
            }
        }
    }
    event(state, &job.incident.id, "system", "subject-notice-finished", json!({"noticeId": job.notice_id, "recipients": job.plans.len()})).await;
}

pub(crate) async fn send_webhook(state: &AppState, url: &str, event_name: &str, payload: &Value, policy: crate::webhooks::AddressPolicy) -> Result<u16, String> {
    let endpoint = crate::webhooks::WebhookEndpoint {
        id: "breach".into(),
        account_id: String::new(),
        url: url.to_string(),
        secret: state.store.derive_key("mw-breach-webhook-v1").to_vec(),
        events: vec![],
    };
    crate::webhooks::deliver_event_with(&endpoint, event_name, payload.to_string().as_bytes(), policy)
        .await
        .map(|s| s.as_u16())
        .map_err(|e| e.to_string())
}

// ─── admin: playbooks and lockdown ────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CheckReq {
    item: String,
    done: bool,
}

async fn admin_playbook(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>, Json(req): Json<CheckReq>) -> Result<Response, AdminApiError> {
    let (mut row, mut body, policy) = load(&state, &id).await?;
    if !m::playbook(&body.incident_type).contains(&req.item.as_str()) {
        return Err(bad("invalidItem", "That step is not in this incident's checklist.", "/item"));
    }
    body.playbook.insert(req.item.clone(), m::CheckState { done: req.done, by: ctx.username.clone(), at: now() });
    save(&state, &mut row, &body, &ctx.username, "playbook-step", json!({"item": req.item, "done": req.done})).await?;
    Ok(no_store(Json(json!({"incident": view(&row, &body, &policy, now())}))))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct LockdownReq {
    /// revoke-sessions or force-password-reset.
    action: String,
    dry_run: bool,
    /// Must be true for a real run.
    confirm: bool,
    /// A subset of the affected addresses; empty means all of them.
    addresses: Vec<String>,
}

async fn admin_lockdown(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>, Json(req): Json<LockdownReq>) -> Result<Response, AdminApiError> {
    let (mut row, mut body, policy) = load(&state, &id).await?;
    if !["revoke-sessions", "force-password-reset"].contains(&req.action.as_str()) {
        return Err(bad("invalidAction", "Unknown action.", "/action"));
    }
    let wanted: Vec<String> = req.addresses.iter().map(|s| s.trim().to_lowercase()).collect();
    let targets: Vec<m::Affected> = body.affected.iter().filter(|a| wanted.is_empty() || wanted.contains(&a.address.to_lowercase())).cloned().collect();
    if targets.is_empty() {
        return Err(bad("noTargets", "The incident has no affected accounts to act on.", "/addresses"));
    }
    if !req.dry_run && !req.confirm {
        return Err(bad("confirmRequired", "Confirm the action, or run a dry run first.", "/confirm"));
    }
    let mut results: Vec<Value> = Vec::new();
    for t in &targets {
        let Some(account) = &t.account_id else {
            results.push(json!({"address": t.address, "status": "skipped", "detail": "no account has signed in for this address"}));
            continue;
        };
        let sessions = state.store.sessions_by_account(account).await.map(|s| s.len()).unwrap_or(0);
        if req.dry_run {
            results.push(json!({"address": t.address, "status": "would-apply", "sessions": sessions}));
            continue;
        }
        let outcome = if req.action == "revoke-sessions" {
            let mut n = state.store.delete_sessions_for_account(account).await;
            if let Ok(a) = &mut n {
                *a += state.store.delete_sessions_for_username(&t.address).await.unwrap_or(0);
            }
            n.map(|n| json!({"sessionsRemoved": n})).map_err(|e| e.to_string())
        } else {
            let mut done = crate::account_gate::update_flags(&state.store, account, |f| f.force_password_change = true).await.map_err(|e| e.to_string());
            if done.is_ok() {
                done = crate::account_gate::update_flags(&state.store, &t.address, |f| f.force_password_change = true).await.map_err(|e| e.to_string());
            }
            done.map(|_| json!({"passwordChangeRequired": true}))
        };
        match outcome {
            Ok(d) => results.push(json!({"address": t.address, "status": "applied", "detail": d})),
            Err(e) => results.push(json!({"address": t.address, "status": "failed", "detail": e})),
        }
    }
    if req.dry_run {
        return Ok(no_store(Json(json!({"dryRun": true, "action": req.action, "results": results}))));
    }
    let applied = results.iter().filter(|r| r["status"] == "applied").count();
    let item = if req.action == "revoke-sessions" { "revoke-sessions" } else { "force-password-reset" };
    if applied > 0 {
        body.playbook.insert(item.into(), m::CheckState { done: true, by: ctx.username.clone(), at: now() });
        body.timeline.push(entry(&ctx.username, "containment", &format!("{}: applied to {applied} account(s).", req.action), now()));
    }
    save(&state, &mut row, &body, &ctx.username, "lockdown", json!({"action": req.action, "applied": applied, "targets": targets.len()})).await?;
    Ok(no_store(Json(json!({"dryRun": false, "action": req.action, "results": results, "incident": view(&row, &body, &policy, now())}))))
}

// ─── notifications to the people running the instance ─────────────────────────

/// Tell the operators through the configured channels. Returns one entry per
/// channel tried.
pub(crate) async fn notify_operators(state: &AppState, policy: &Policy, row: &BreachRow, what: &str, subject: &str, text: &str) -> Vec<Value> {
    let mut out = Vec::new();
    let payload = json!({
        "event": format!("breach.{what}"), "reference": m::short_ref(&row.id), "title": row.title,
        "status": row.status, "severity": row.severity, "deadlineAt": row.deadline_at, "message": text,
    });
    if policy.b("breach.notify.audit") {
        security_event(state, SecurityEvent::system("breach-notice").target(format!("breach:{}", row.id)).detail(json!({"what": what, "deadlineAt": row.deadline_at}))).await;
        out.push(json!({"channel": "audit", "ok": true}));
    }
    for url in policy.strings("breach.notify.webhookUrls") {
        let r = send_webhook(state, &url, "BreachEvent", &payload, mw_egress::ip_allowed).await;
        let host = reqwest::Url::parse(&url).ok().and_then(|u| u.host_str().map(str::to_string)).unwrap_or_default();
        out.push(json!({"channel": "webhook", "target": host, "ok": matches!(r, Ok(c) if c < 300), "detail": r.err()}));
    }
    for address in policy.strings("breach.notify.operatorAddresses") {
        let canonical = state.store.canonical_policy_address(&address).await.unwrap_or(address.to_lowercase());
        let account = state.store.lifecycle_account_bindings(&canonical).await.ok().and_then(|v| v.first().cloned());
        match account {
            None => out.push(json!({"channel": "inbox", "target": canonical, "ok": false, "detail": "no account has signed in for this address"})),
            Some(acc) => {
                let r = crate::alerts::deliver_text(state, &acc, "en", subject, text, "").await;
                out.push(json!({"channel": "inbox", "target": canonical, "ok": r.is_ok(), "detail": r.err()}));
            }
        }
    }
    event(state, &row.id, "system", "operators-notified", json!({"what": what, "channels": out})).await;
    out
}

fn notify_background(state: &AppState, policy: &Policy, row: &BreachRow, what: &'static str, body: &Body) {
    let (state, policy, row) = (state.clone(), policy.clone(), row.clone());
    let text = format!("{} {}: {}", m::short_ref(&row.id), row.title, what_text(what, &row, body, now()));
    tokio::spawn(async move {
        let subject = format!("Breach register: {}", row.title);
        notify_operators(&state, &policy, &row, what, &subject, &text).await;
    });
}

fn what_text(what: &str, row: &BreachRow, _body: &Body, at: i64) -> String {
    let left = row.deadline_at.map(|d| (d - at) / m::HOUR);
    match (what, left) {
        ("confirmed", Some(h)) => format!("clock started; {h} hours to the notice deadline."),
        ("overdue", Some(h)) => format!("the notice deadline passed {} hours ago and no authority notice is recorded.", -h),
        ("auto-opened", _) => "a draft was opened from a detection and needs review.".into(),
        ("escalated", _) => "a draft has waited for review past the escalation time.".into(),
        (w, Some(h)) if w.starts_with("reminder") => format!("{h} hours left to the notice deadline."),
        _ => what.to_string(),
    }
}

// ─── worker ───────────────────────────────────────────────────────────────────

pub(crate) fn start(state: AppState) {
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(20)).await;
        loop {
            pass_at(&state, now()).await;
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    });
}

/// One worker pass at `at`: reminders, escalation, the automation scan and the
/// register's retention.
pub(crate) async fn pass_at(state: &AppState, at: i64) {
    let rows = match state.store.breach_list().await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("breach worker pass failed: {e}");
            return;
        }
    };
    for row in rows.into_iter().filter(|r| m::is_open(&r.status)) {
        if let Err(e) = remind_one(state, row, at).await {
            tracing::warn!("breach reminder failed: {}", e.detail);
        }
    }
    scan_audit(state, at).await;
    purge(state, at).await;
}

async fn remind_one(state: &AppState, mut row: BreachRow, at: i64) -> Result<(), AdminApiError> {
    let policy = Policy::load(state, &row.domain).await;
    if !policy.b("breach.enabled") {
        return Ok(());
    }
    let mut body = Body::of(&row);
    // Escalation of a draft (or an unassigned incident) nobody has picked up.
    let after = policy.n("breach.auto.escalateAfterHours");
    if after > 0 && body.escalated_at.is_none() && (row.aware_at.is_none() || row.assignee.is_none()) && at - row.created_at >= after * m::HOUR && row.source == "auto" {
        body.escalated_at = Some(at);
        let to = policy.s("breach.auto.escalateTo");
        if !to.is_empty() {
            row.assignee = Some(to.clone());
        }
        let detail = json!({"assignee": row.assignee, "afterHours": after});
        save(state, &mut row, &body, "system", "escalated", detail).await?;
        let text = format!("{} {}: {}", m::short_ref(&row.id), row.title, what_text("escalated", &row, &body, at));
        notify_operators(state, &policy, &row, "escalated", &format!("Breach register: {}", row.title), &text).await;
        return Ok(());
    }
    let (Some(_), Some(deadline)) = (row.aware_at, row.deadline_at) else { return Ok(()) };
    if !body.authority_notices.is_empty() {
        return Ok(());
    }
    let step = m::due_reminder(at, deadline, &policy.ints("breach.reminders.hours"), &body.reminded, body.overdue_reminded_at, policy.n("breach.reminders.overdueRepeatHours"));
    let Some(which) = step.announce else {
        if !step.fired.is_empty() {
            body.reminded.extend(step.fired);
            save(state, &mut row, &body, "system", "reminder-marked", json!({})).await?;
        }
        return Ok(());
    };
    body.reminded.extend(step.fired);
    if let Some(o) = step.overdue_at {
        body.overdue_reminded_at = Some(o);
    }
    let (kind, label) = match which {
        Reminder::Before(h) => ("reminder", format!("reminder-{h}h")),
        Reminder::Overdue => ("overdue", "overdue".to_string()),
    };
    save(state, &mut row, &body, "system", kind, json!({"which": label, "deadlineAt": deadline})).await?;
    let text = format!("{} {}: {}", m::short_ref(&row.id), row.title, what_text(if kind == "overdue" { "overdue" } else { "reminder" }, &row, &body, at));
    notify_operators(state, &policy, &row, &label, &format!("Breach register: {}", row.title), &text).await;
    Ok(())
}

async fn purge(state: &AppState, at: i64) {
    static LAST: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
    let last = LAST.load(std::sync::atomic::Ordering::Relaxed);
    if at - last < 3600 {
        return;
    }
    LAST.store(at, std::sync::atomic::Ordering::Relaxed);
    let policy = Policy::load(state, "").await;
    let years = policy.n("breach.retention.years").clamp(1, 50);
    match state.store.breach_purge_closed_before(at - years * 365 * DAY).await {
        Ok(0) => {}
        Ok(n) => {
            security_event(state, SecurityEvent::system("breach-register-purged").detail(json!({"incidents": n, "years": years}))).await;
        }
        Err(e) => tracing::warn!("breach register retention failed: {e}"),
    }
}

/// `POST /breaches/run`: check deadlines and detections now instead of waiting for
/// the next minute.
async fn admin_run(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    pass_at(&state, now()).await;
    audit(&state, &ctx.username, "register", "pass-run", json!({})).await;
    Ok(no_store(Json(json!({"ok": true}))))
}

// ─── automations ──────────────────────────────────────────────────────────────

fn counters() -> &'static Mutex<HashMap<(u128, &'static str), m::Counter>> {
    static C: OnceLock<Mutex<HashMap<(u128, &'static str), m::Counter>>> = OnceLock::new();
    C.get_or_init(Default::default)
}

fn instance_key(state: &AppState) -> u128 {
    state.started_at.duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0)
}

/// Read the audit rows added since the last scan and count them toward the
/// rules. The first scan only sets the cursor, so enabling automation does not
/// replay history.
async fn scan_audit(state: &AppState, at: i64) {
    let policy = Policy::load(state, "").await;
    if !policy.b("breach.auto.enabled") {
        return;
    }
    let now_ts = chrono::DateTime::from_timestamp(at, 0).unwrap_or_default().to_rfc3339_opts(chrono::SecondsFormat::Nanos, false);
    let Some(cursor) = state.store.get_setting(CURSOR_KEY).await.ok().flatten() else {
        let _ = state.store.set_setting(CURSOR_KEY, &now_ts).await;
        return;
    };
    let rows = match state.store.breach_audit_after(&cursor, 500).await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("breach automation scan failed: {e}");
            return;
        }
    };
    let Some(last) = rows.last().map(|r| r.ts.clone()) else { return };
    let window = policy.n("breach.auto.windowMinutes").clamp(1, 1440) * 60;
    for r in &rows {
        let detail: Value = serde_json::from_str(&r.detail_json).unwrap_or(Value::Null);
        let Some(rule_id) = m::rule_for(&r.action, &detail) else { continue };
        let Some(rule) = m::RULES.iter().find(|x| x.id == rule_id) else { continue };
        let threshold = policy.n(rule.key);
        if threshold <= 0 {
            continue;
        }
        let ts = chrono::DateTime::parse_from_rfc3339(&r.ts).map(|d| d.timestamp()).unwrap_or(at);
        let hit = {
            let mut map = counters().lock().unwrap_or_else(|e| e.into_inner());
            map.entry((instance_key(state), rule.id)).or_default().hit(ts, window, threshold)
        };
        if hit {
            let sample = json!({"action": r.action, "target": r.target, "ts": r.ts});
            if let Err(e) = open_auto(state, &policy, rule, threshold, window, sample, at).await {
                tracing::warn!("breach automation could not open a draft: {}", e.detail);
            }
        }
    }
    let _ = state.store.set_setting(CURSOR_KEY, &last).await;
}

/// Open a draft for a rule, or add to the open draft the same rule opened in the
/// last day.
async fn open_auto(state: &AppState, policy: &Policy, rule: &m::Rule, threshold: i64, window: i64, sample: Value, at: i64) -> Result<(), AdminApiError> {
    let existing = state.store.breach_list().await.map_err(store_err)?.into_iter().find(|r| {
        r.source == "auto" && r.status == "detected" && r.aware_at.is_none() && at - r.created_at < DAY && Body::of(r).auto.as_ref().and_then(|a| a["rule"].as_str()) == Some(rule.id)
    });
    if let Some(mut row) = existing {
        let mut body = Body::of(&row);
        body.timeline.push(entry("system", "note", &format!("The rule matched again ({threshold} events in {} minutes).", window / 60), at));
        save(state, &mut row, &body, "system", "auto-matched-again", json!({"rule": rule.id})).await?;
        return Ok(());
    }
    let start_now = policy.s("breach.auto.startClock") == "onDetect";
    let n = NewIncident {
        title: format!("{} ({threshold} events in {} minutes)", rule.title, window / 60),
        description: format!("Opened by the {} rule: {threshold} matching events within {} minutes. Review before confirming; this is a detection, not a finding.", rule.id, window / 60),
        source: "auto".into(),
        incident_type: rule.incident_type.into(),
        draft: !start_now,
        aware_at: start_now.then_some(at),
        detected_at: Some(at),
        reporter: "automation".into(),
        assignee: Some(policy.s("breach.auto.assignee")).filter(|s| !s.is_empty()),
        auto: Some(json!({"rule": rule.id, "threshold": threshold, "windowMinutes": window / 60, "sample": sample})),
        ..NewIncident::default()
    };
    let (row, body) = open_incident(state, n, "system", at).await?;
    let text = format!("{} {}: {}", m::short_ref(&row.id), row.title, what_text("auto-opened", &row, &body, at));
    notify_operators(state, policy, &row, "auto-opened", &format!("Breach register: {}", row.title), &text).await;
    Ok(())
}

// ─── data subject side ────────────────────────────────────────────────────────

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/privacy/breach-notices", get(my_notices))
        .route("/api/privacy/breach-notices/ack", post(ack_notices))
}

async fn my_notices(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    let prefix = format!("{NOTICE_PREFIX}{}:", account_key(&session.account_id));
    let Ok(rows) = state.store.settings_with_prefix(&prefix).await else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    let notices: Vec<Value> = rows.iter().rev().filter_map(|(_, v)| serde_json::from_str(v).ok()).collect();
    let unread = notices.iter().filter(|n| n["acknowledged"] != json!(true)).count();
    no_store(Json(json!({"notices": notices, "unread": unread})))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AckReq {
    id: Option<String>,
    #[serde(default)]
    all: bool,
}

async fn ack_notices(State(state): State<AppState>, headers: HeaderMap, Json(req): Json<AckReq>) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    let prefix = format!("{NOTICE_PREFIX}{}:", account_key(&session.account_id));
    let Ok(rows) = state.store.settings_with_prefix(&prefix).await else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    let mut changed = 0;
    for (key, value) in rows {
        let wanted = req.all || req.id.as_deref().is_some_and(|id| key == format!("{prefix}{id}"));
        if !wanted {
            continue;
        }
        if let Ok(mut row) = serde_json::from_str::<Value>(&value) {
            row["acknowledged"] = json!(true);
            if state.store.set_setting(&key, &row.to_string()).await.is_ok() {
                changed += 1;
            }
        }
    }
    no_store(Json(json!({"updated": changed})))
}

// ─── registration ─────────────────────────────────────────────────────────────

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, "/breaches", C::PrivacyRead, admin_list);
    r.admin_endpoint(Method::POST, "/breaches", C::PrivacyWrite, admin_create);
    r.admin_endpoint(Method::GET, "/breaches/meta", C::PrivacyRead, admin_meta);
    r.admin_endpoint(Method::GET, "/breaches/export", C::PrivacyRead, admin_export);
    r.admin_endpoint(Method::POST, "/breaches/run", C::PrivacyWrite, admin_run);
    r.admin_endpoint(Method::GET, "/breaches/{id}", C::PrivacyRead, admin_get);
    r.admin_endpoint(Method::GET, "/breaches/{id}/report", C::PrivacyRead, admin_report);
    r.admin_endpoint(Method::POST, "/breaches/{id}/details", C::PrivacyWrite, admin_details);
    r.admin_endpoint(Method::POST, "/breaches/{id}/assign", C::PrivacyWrite, admin_assign);
    r.admin_endpoint(Method::POST, "/breaches/{id}/confirm", C::PrivacyWrite, admin_confirm);
    r.admin_endpoint(Method::POST, "/breaches/{id}/aware-at", C::PrivacyWrite, admin_aware_at);
    r.admin_endpoint(Method::POST, "/breaches/{id}/assess", C::PrivacyWrite, admin_assess);
    r.admin_endpoint(Method::POST, "/breaches/{id}/status", C::PrivacyWrite, admin_status);
    r.admin_endpoint(Method::POST, "/breaches/{id}/timeline", C::PrivacyWrite, admin_timeline);
    r.admin_endpoint(Method::POST, "/breaches/{id}/evidence", C::PrivacyWrite, admin_evidence);
    r.admin_endpoint(Method::GET, "/breaches/{id}/authority-notice", C::PrivacyRead, admin_authority_notice);
    r.admin_endpoint(Method::POST, "/breaches/{id}/authority-notice", C::PrivacyWrite, admin_authority_record);
    r.admin_endpoint(Method::POST, "/breaches/{id}/subject-notice", C::PrivacyWrite, admin_subject_notice);
    r.admin_endpoint(Method::POST, "/breaches/{id}/playbook", C::PrivacyWrite, admin_playbook);
    r.admin_endpoint(Method::POST, "/breaches/{id}/lockdown", C::PrivacyWrite, admin_lockdown);
}
