//! Operational incidents and NIS2 preparation. Detection never determines legal
//! significance, declares recovery verified, or submits a regulatory report.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx},
    admin_router::AdminRegistrar,
    capabilities::Capability as C,
};
use axum::{
    Json,
    extract::{Path, State},
    http::{Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use mw_store::compliance::ComplianceRecord;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
const KIND: &str = "operational-incident";
const MONITOR: &str = "operational-monitor";
fn now() -> i64 {
    chrono::Utc::now().timestamp()
}
fn invalid(message: &str) -> AdminApiError {
    AdminApiError::invalid("invalidIncident", message, "/incident")
}
fn internal(error: impl std::fmt::Display) -> AdminApiError {
    AdminApiError::internal(error.to_string())
}
#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct Entry {
    at: i64,
    kind: String,
    note: String,
    actor: String,
}
#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct Report {
    phase: String,
    submitted_at: i64,
    reference: String,
    by: String,
}
#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct Incident {
    title: String,
    category: String,
    source: String,
    component: String,
    status: String,
    detected_at: i64,
    aware_at: Option<i64>,
    significance: String,
    impact: String,
    owner: String,
    root_cause: String,
    measures: String,
    recovery_plan: String,
    recovery_evidence: String,
    recovery_time_minutes: Option<u32>,
    recovery_point_minutes: Option<u32>,
    first_healthy_at: Option<i64>,
    resolved_at: Option<i64>,
    verified_at: Option<i64>,
    samples: u64,
    healthy_samples: u32,
    last_sample_at: i64,
    emailed: Vec<String>,
    audit_reference: String,
    events: Vec<Entry>,
    reports: Vec<Report>,
    reminded: Vec<String>,
}
impl Incident {
    fn event(&mut self, kind: &str, note: &str, actor: &str, at: i64) {
        self.events.push(Entry {
            at,
            kind: kind.into(),
            note: note.into(),
            actor: actor.into(),
        });
        if self.events.len() > 500 {
            self.events.drain(..self.events.len() - 500);
        }
    }
}
fn decode(row: &ComplianceRecord) -> Result<Incident, AdminApiError> {
    serde_json::from_str(&row.body_json).map_err(internal)
}
fn record(
    id: String,
    body: &Incident,
    actor: &str,
    at: i64,
) -> Result<ComplianceRecord, AdminApiError> {
    Ok(ComplianceRecord {
        id,
        kind: KIND.into(),
        name: body.title.clone(),
        body_json: serde_json::to_string(body).map_err(internal)?,
        updated_by: actor.into(),
        updated_at: at,
    })
}
async fn load(state: &AppState, id: &str) -> Result<(ComplianceRecord, Incident), AdminApiError> {
    let row = state
        .store
        .get_compliance_record(id)
        .await
        .map_err(internal)?
        .filter(|r| r.kind == KIND)
        .ok_or_else(|| {
            AdminApiError::new(StatusCode::NOT_FOUND, "notFound", "Incident not found")
        })?;
    let body = decode(&row)?;
    Ok((row, body))
}
async fn update(
    state: &AppState,
    old: &ComplianceRecord,
    body: &Incident,
    actor: &str,
    at: i64,
) -> Result<(), AdminApiError> {
    let new = record(old.id.clone(), body, actor, at)?;
    if !state
        .store
        .transition_compliance_record(old, &new)
        .await
        .map_err(internal)?
    {
        return Err(AdminApiError::new(
            StatusCode::CONFLICT,
            "incidentChanged",
            "Incident changed; reload before saving.",
        ));
    }
    Ok(())
}
async fn audit(state: &AppState, id: &str, action: &'static str) {
    crate::audit::security_event(
        state,
        crate::audit::SecurityEvent::system(action).target(id),
    )
    .await;
}
async fn audit_admin(state: &AppState, ctx: &AdminCtx, id: &str, action: &'static str) {
    crate::audit::security_event(
        state,
        crate::audit::SecurityEvent::admin(&ctx.username, action).target(id),
    )
    .await;
}
fn month_after(ts: i64) -> Option<i64> {
    chrono::DateTime::from_timestamp(ts, 0)?
        .checked_add_months(chrono::Months::new(1))
        .map(|date| date.timestamp())
}
fn deadlines(body: &Incident, trust_service: bool) -> Value {
    let Some(aware) = body.aware_at.filter(|_| body.significance == "significant") else {
        return json!({});
    };
    let notification = body
        .reports
        .iter()
        .filter(|r| r.phase == "notification")
        .map(|r| r.submitted_at)
        .min();
    let mut value = json!({"early-warning":aware.saturating_add(24*3600),"notification":aware.saturating_add(if trust_service{24}else{72}*3600)});
    if let Some(sent) = notification {
        if let Some(initial) = month_after(sent) {
            match body.resolved_at {
                None => value["progress"] = json!(initial),
                Some(handled) if handled > initial => {
                    value["progress"] = json!(initial);
                    value["final"] = json!(month_after(handled));
                }
                Some(_) => value["final"] = json!(initial),
            }
        }
    }
    value
}
fn view(row: &ComplianceRecord, body: &Incident, trust: bool) -> Value {
    json!({"id":row.id,"updatedAt":row.updated_at,"version":crate::push_relay::hash_token(&row.body_json),"incident":body,"deadlines":deadlines(body,trust),"observedRecoverySeconds":body.first_healthy_at.map(|t|t.saturating_sub(body.detected_at)),"recoveryVerified":body.verified_at.is_some()})
}
async fn cfg(state: &AppState) -> BTreeMap<String, Value> {
    let subject = mw_store::settings_policy::SettingSubject {
        personal_owner: None,
        domain: "",
        address: "",
        account_id: "",
    };
    crate::settings_registry::effective_all(&state.store, &subject, |d| {
        d.key.starts_with("incidents.") || d.key == "dsr.legalHold.subjects"
    })
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|r| (r.key, r.value))
    .collect()
}
fn enabled(cfg: &BTreeMap<String, Value>, key: &str) -> bool {
    cfg.get(key) == Some(&json!(true))
}
pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, "/incidents", C::PrivacyRead, list);
    r.admin_endpoint(Method::POST, "/incidents", C::PrivacyWrite, create);
    r.admin_endpoint(Method::PUT, "/incidents/{id}", C::PrivacyWrite, edit);
    r.admin_endpoint(
        Method::POST,
        "/incidents/{id}/reports",
        C::PrivacyWrite,
        report,
    );
    r.admin_endpoint(
        Method::GET,
        "/incidents/{id}/export",
        C::PrivacyRead,
        export,
    );
    r.admin_endpoint(Method::POST, "/incidents/scan", C::PrivacyWrite, scan);
    r.admin_endpoint(
        Method::POST,
        "/incidents/observations",
        C::PrivacyWrite,
        observation,
    );
}
async fn list(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::PrivacyRead)?;
    let config = cfg(&state).await;
    let trust = enabled(&config, "incidents.nis2.trustService");
    let rows = state
        .store
        .list_compliance_records(KIND)
        .await
        .map_err(internal)?;
    let mut items = rows
        .iter()
        .map(|row| Ok(view(row, &decode(row)?, trust)))
        .collect::<Result<Vec<_>, AdminApiError>>()?;
    items.sort_by_key(|r| std::cmp::Reverse(r["updatedAt"].as_i64().unwrap_or(0)));
    Ok(([(header::CACHE_CONTROL,"no-store")],Json(json!({"items":items,"automatic":enabled(&config,"incidents.auto.enabled"),"trustService":trust,"checkedAt":now()}))).into_response())
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Edit {
    title: String,
    category: String,
    #[serde(default)]
    impact: String,
    #[serde(default)]
    owner: String,
    #[serde(default)]
    root_cause: String,
    #[serde(default)]
    measures: String,
    #[serde(default)]
    recovery_plan: String,
    #[serde(default)]
    recovery_evidence: String,
    #[serde(default)]
    recovery_time_minutes: Option<u32>,
    #[serde(default)]
    recovery_point_minutes: Option<u32>,
    #[serde(default)]
    aware_at: Option<i64>,
    #[serde(default)]
    handled_at: Option<i64>,
    #[serde(default)]
    significance: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    updated_at: Option<i64>,
    #[serde(default)]
    version: Option<String>,
}
fn apply(body: &mut Incident, req: Edit, at: i64, actor: &str) -> Result<(), AdminApiError> {
    if req.title.trim().is_empty() || req.title.chars().count() > 200 {
        return Err(invalid("Use a title of 1 to 200 characters."));
    }
    if ![
        "outage",
        "account-compromise",
        "service-disruption",
        "vulnerability",
    ]
    .contains(&req.category.as_str())
    {
        return Err(invalid("Choose an incident category."));
    }
    for text in [
        &req.title,
        &req.impact,
        &req.owner,
        &req.root_cause,
        &req.measures,
        &req.recovery_plan,
        &req.recovery_evidence,
    ] {
        if text.chars().count() > 4000 || text.chars().any(|c| c.is_control() && c != '\n') {
            return Err(invalid("Use plain text up to 4000 characters."));
        }
    }
    let significance = if req.significance.is_empty() {
        "unassessed"
    } else {
        &req.significance
    };
    if !["unassessed", "significant", "not-significant"].contains(&significance) {
        return Err(invalid("Choose significance."));
    }
    if req.aware_at.is_some_and(|t| t < 0 || t > at + 300) {
        return Err(invalid("Awareness time must not be in the future."));
    }
    let status = if req.status.is_empty() {
        "detected"
    } else {
        &req.status
    };
    if !["detected", "active", "recovering", "resolved", "closed"].contains(&status) {
        return Err(invalid("Choose a valid incident status."));
    }
    if significance == "significant" && req.aware_at.is_none() {
        return Err(invalid("Record awareness time for a significant incident."));
    }
    if matches!(status, "resolved" | "closed")
        && (req.recovery_evidence.trim().is_empty() || req.root_cause.trim().is_empty())
    {
        return Err(invalid(
            "Verify recovery and record root cause before resolving or closing.",
        ));
    }
    if matches!(status, "resolved" | "closed") {
        let handled = req.handled_at.or(body.resolved_at).unwrap_or(at);
        if handled < 0 || handled > at + 300 || req.aware_at.is_some_and(|aware| handled < aware) {
            return Err(invalid(
                "Handling time must follow awareness and must not be in the future.",
            ));
        }
        body.resolved_at = Some(handled);
        if body.verified_at.is_none() {
            body.verified_at = Some(at);
            body.event("recovery-verified", &req.recovery_evidence, actor, at);
        }
    }
    if !matches!(status, "resolved" | "closed") && body.verified_at.is_some() {
        body.verified_at = None;
        body.resolved_at = None;
        body.event("reopened", "Recovery needs verification again.", actor, at);
    }
    body.title = req.title.trim().into();
    body.category = req.category;
    body.impact = req.impact;
    body.owner = req.owner;
    body.root_cause = req.root_cause;
    body.measures = req.measures;
    body.recovery_plan = req.recovery_plan;
    body.recovery_evidence = req.recovery_evidence;
    body.recovery_time_minutes = req.recovery_time_minutes;
    body.recovery_point_minutes = req.recovery_point_minutes;
    body.aware_at = req.aware_at;
    body.significance = significance.into();
    body.status = status.into();
    Ok(())
}
async fn create(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Json(req): Json<Edit>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::PrivacyWrite)?;
    let at = now();
    let mut body = Incident {
        detected_at: at,
        source: "manual".into(),
        ..Default::default()
    };
    apply(&mut body, req, at, &ctx.username)?;
    body.event("created", "Incident opened.", &ctx.username, at);
    let id = uuid::Uuid::new_v4().to_string();
    state
        .store
        .put_compliance_record(&record(id.clone(), &body, &ctx.username, at)?)
        .await
        .map_err(internal)?;
    audit_admin(&state, &ctx, &id, "operational-incident-created").await;
    Ok((StatusCode::CREATED, Json(json!({"id":id}))).into_response())
}
async fn edit(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
    Json(req): Json<Edit>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::PrivacyWrite)?;
    let (old, mut body) = load(&state, &id).await?;
    if req.updated_at != Some(old.updated_at)
        || req.version.as_deref() != Some(crate::push_relay::hash_token(&old.body_json).as_str())
    {
        return Err(AdminApiError::new(
            StatusCode::CONFLICT,
            "incidentChanged",
            "Reload the current incident before saving.",
        ));
    }
    let at = now();
    apply(&mut body, req, at, &ctx.username)?;
    body.event("updated", "Incident details updated.", &ctx.username, at);
    update(&state, &old, &body, &ctx.username, at).await?;
    audit_admin(&state, &ctx, &id, "operational-incident-updated").await;
    Ok(Json(json!({"ok":true})).into_response())
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReportReq {
    phase: String,
    submitted_at: i64,
    reference: String,
}
async fn report(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
    Json(req): Json<ReportReq>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::PrivacyWrite)?;
    let (old, mut body) = load(&state, &id).await?;
    let at = now();
    if !["early-warning", "notification", "progress", "final"].contains(&req.phase.as_str())
        || req.reference.trim().is_empty()
        || req.reference.len() > 500
        || req.reference.chars().any(char::is_control)
        || req.submitted_at < 0
        || req.submitted_at > at + 300
        || body.aware_at.is_none_or(|t| req.submitted_at < t)
        || body.significance != "significant"
    {
        return Err(invalid(
            "Record a significant incident, awareness time, valid reporting phase, actual submission time and authority reference.",
        ));
    }
    if req.phase == "final"
        && body
            .resolved_at
            .is_some_and(|handled| req.submitted_at < handled)
    {
        return Err(invalid(
            "Final submission cannot precede incident handling.",
        ));
    }
    if req.phase == "final" && body.resolved_at.is_none() {
        return Err(invalid(
            "While an incident is ongoing, record a progress report; final follows handling and verified recovery.",
        ));
    }
    if matches!(req.phase.as_str(), "progress" | "final")
        && !body.reports.iter().any(|r| r.phase == "notification")
    {
        return Err(invalid(
            "Record the incident notification before progress or final reports.",
        ));
    }
    if body.reports.len() >= 100 {
        return Err(invalid("Reporting history is full."));
    }
    body.event("report-recorded", &req.phase, &ctx.username, at);
    body.reports.push(Report {
        phase: req.phase,
        submitted_at: req.submitted_at,
        reference: req.reference,
        by: ctx.username.clone(),
    });
    update(&state, &old, &body, &ctx.username, at).await?;
    audit_admin(&state, &ctx, &id, "operational-report-recorded").await;
    Ok(Json(json!({"ok":true})).into_response())
}
async fn export(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::PrivacyRead)?;
    let (row, body) = load(&state, &id).await?;
    let config = cfg(&state).await;
    let mut output = view(&row, &body, enabled(&config, "incidents.nis2.trustService"));
    output["preparationOnly"] = json!(true);
    output["generatedAt"] = json!(now());
    audit_admin(&state, &ctx, &id, "operational-incident-exported").await;
    Ok((
        [
            (header::CACHE_CONTROL, "no-store"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=incident-report.json",
            ),
        ],
        Json(output),
    )
        .into_response())
}
async fn scan(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::PrivacyWrite)?;
    tick(&state).await;
    Ok(Json(json!({"ok":true})).into_response())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Observation {
    component: String,
    status: String,
}
async fn observation(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Json(req): Json<Observation>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::PrivacyWrite)?;
    if req.component.is_empty()
        || req.component.len() > 40
        || !req
            .component
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        || !["ok", "fail", "warn", "disabled"].contains(&req.status.as_str())
    {
        return Err(invalid(
            "Use a component identifier and a valid health state.",
        ));
    }
    if !enabled(&cfg(&state).await, "incidents.auto.enabled") {
        return Err(invalid("Incident observation automation is disabled."));
    }
    let component = format!("external-{}", req.component);
    let mut components = serde_json::Map::new();
    components.insert(component.clone(), json!({"status":req.status}));
    health_pass(&state, &json!({"checkedAt":now(),"components":components})).await?;
    audit_admin(&state, &ctx, &component, "operational-observation-received").await;
    Ok(Json(json!({"ok":true})).into_response())
}

// Health observations use the existing probe cycle; no extra probes or external
// requests are introduced. Samples are persisted for restart-safe debounce.
pub(crate) async fn observe_health(state: &AppState, snapshot: &Value) {
    if let Err(error) = health_pass(state, snapshot).await {
        tracing::warn!("Operational incident observation failed: {}", error.detail);
    }
}
async fn health_pass(state: &AppState, snapshot: &Value) -> Result<(), AdminApiError> {
    let config = cfg(state).await;
    if !enabled(&config, "incidents.auto.enabled") {
        return Ok(());
    }
    let at = snapshot["checkedAt"].as_i64().unwrap_or_else(now);
    if now().saturating_sub(at) > 120 {
        return Ok(());
    }
    let fail_limit = config
        .get("incidents.auto.failureSamples")
        .and_then(Value::as_u64)
        .unwrap_or(3);
    let Some(components) = snapshot["components"].as_object() else {
        return Ok(());
    };
    for (name, value) in components {
        if !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            continue;
        }
        let status = value["status"].as_str().unwrap_or("unknown");
        let id = format!("operational-monitor:{name}");
        let old = state
            .store
            .get_compliance_record(&id)
            .await
            .map_err(internal)?;
        let mut monitor = old
            .as_ref()
            .and_then(|r| serde_json::from_str::<Value>(&r.body_json).ok())
            .unwrap_or(json!({"lastAt":0,"failures":0,"healthy":0}));
        if at - monitor["lastAt"].as_i64().unwrap_or(0) < 30 {
            continue;
        }
        monitor["lastAt"] = json!(at);
        if status == "fail" {
            if monitor["failures"] == 0 {
                monitor["since"] = json!(at);
            }
            monitor["failures"] = json!(monitor["failures"].as_u64().unwrap_or(0) + 1);
            monitor["healthy"] = json!(0);
        } else if status == "ok" {
            if monitor["healthy"] == 0 {
                monitor["healthySince"] = json!(at);
            }
            monitor["healthy"] = json!(monitor["healthy"].as_u64().unwrap_or(0) + 1);
            monitor["failures"] = json!(0);
        } else {
            monitor["healthy"] = json!(0);
            monitor["failures"] = json!(0);
        }
        let next = ComplianceRecord {
            id: id.clone(),
            kind: MONITOR.into(),
            name: name.clone(),
            body_json: monitor.to_string(),
            updated_by: "system".into(),
            updated_at: at,
        };
        let claimed = if let Some(old) = old {
            state
                .store
                .transition_compliance_record(&old, &next)
                .await
                .map_err(internal)?
        } else {
            state
                .store
                .insert_compliance_record_once(&next)
                .await
                .map_err(internal)?
        };
        if !claimed {
            continue;
        }
        let active_id = monitor["incidentId"].as_str().map(str::to_owned);
        if status == "fail"
            && monitor["failures"].as_u64().unwrap_or(0) >= fail_limit
            && active_id.is_none()
        {
            let since = monitor["since"].as_i64().unwrap_or(at);
            let incident_id = format!("operational-health:{name}:{since}");
            let mut body = Incident {
                title: format!("Service disruption: {name}"),
                category: "service-disruption".into(),
                component: name.clone(),
                source: "health".into(),
                status: "detected".into(),
                significance: "unassessed".into(),
                detected_at: since,
                samples: monitor["failures"].as_u64().unwrap_or(0),
                last_sample_at: at,
                ..Default::default()
            };
            body.event(
                "detected",
                "Repeated failed health observations; review impact and significance.",
                "system",
                at,
            );
            state
                .store
                .insert_compliance_record_once(&record(incident_id.clone(), &body, "system", at)?)
                .await
                .map_err(internal)?;
            monitor["incidentId"] = json!(incident_id);
            let mut updated = next.clone();
            updated.body_json = monitor.to_string();
            state
                .store
                .transition_compliance_record(&next, &updated)
                .await
                .map_err(internal)?;
            audit(state, &incident_id, "operational-incident-detected").await;
        } else if let Some(incident_id) = active_id {
            if let Ok((row, mut body)) = load(state, &incident_id).await {
                if !matches!(body.status.as_str(), "resolved" | "closed") {
                    body.samples = body.samples.saturating_add(1);
                    body.last_sample_at = at;
                    body.healthy_samples = monitor["healthy"]
                        .as_u64()
                        .unwrap_or(0)
                        .min(u32::MAX as u64) as u32;
                    if body.healthy_samples >= 2 && body.first_healthy_at.is_none() {
                        body.first_healthy_at =
                            Some(monitor["healthySince"].as_i64().unwrap_or(at));
                        body.status = "recovering".into();
                        body.event("recovery-observed","Two healthy samples observed; operator verification is still required.","system",at);
                    }
                    if status == "fail" && body.first_healthy_at.take().is_some() {
                        body.status = "active".into();
                        body.event("recovery-interrupted", "Health failed again.", "system", at);
                    }
                    let _ = update(state, &row, &body, "system", at).await;
                } else if monitor["failures"].as_u64().unwrap_or(0) >= fail_limit {
                    // New sustained failures after closure start a new episode next cycle.
                    monitor["incidentId"] = Value::Null;
                    monitor["since"] = json!(at);
                    let mut updated = next.clone();
                    updated.body_json = monitor.to_string();
                    let _ = state
                        .store
                        .transition_compliance_record(&next, &updated)
                        .await;
                }
            }
        }
    }
    Ok(())
}
async fn tick(state: &AppState) {
    if let Err(error) = pass(state).await {
        tracing::warn!("Operational incident automation failed: {}", error.detail);
    }
}
async fn pass(state: &AppState) -> Result<(), AdminApiError> {
    let config = cfg(state).await;
    let at = now();
    if enabled(&config, "incidents.auto.enabled") {
        let heartbeat_id = "operational:heartbeat";
        let previous = state
            .store
            .get_compliance_record(heartbeat_id)
            .await
            .map_err(internal)?;
        let next = ComplianceRecord {
            id: heartbeat_id.into(),
            kind: MONITOR.into(),
            name: "heartbeat".into(),
            body_json: json!({"at":at}).to_string(),
            updated_by: "system".into(),
            updated_at: at,
        };
        let won = if let Some(old) = &previous {
            state
                .store
                .transition_compliance_record(old, &next)
                .await
                .map_err(internal)?
        } else {
            state
                .store
                .insert_compliance_record_once(&next)
                .await
                .map_err(internal)?
        };
        if won {
            if let Some(old) = previous.filter(|old| at.saturating_sub(old.updated_at) > 180) {
                let id = format!("operational-gap:{}", old.updated_at);
                let mut body=Incident{title:"Service monitoring gap".into(),category:"service-disruption".into(),source:"monitoring-gap".into(),component:"monitor".into(),status:"detected".into(),significance:"unassessed".into(),detected_at:old.updated_at,last_sample_at:at,impact:"No heartbeat was recorded during this interval. It can indicate planned downtime, a stopped monitor, database unavailability or an outage; service availability is unknown.".into(),..Default::default()};
                body.event(
                    "detected",
                    "Monitoring resumed after a gap; this is not proof of an outage.",
                    "system",
                    at,
                );
                state
                    .store
                    .insert_compliance_record_once(&record(id, &body, "system", at)?)
                    .await
                    .map_err(internal)?;
            }
        }
        // The audit ID breaks equal-timestamp ties; unlike a time-only cursor this
        // cannot skip events at a page boundary. The first pass starts from now.
        let cursor_id = "operational:audit-cursor";
        let cursor = state
            .store
            .get_compliance_record(cursor_id)
            .await
            .map_err(internal)?;
        if let Some(old) = cursor {
            let cursor: Value = serde_json::from_str(&old.body_json).map_err(internal)?;
            let rows = state
                .store
                .operational_audit_after(
                    cursor["ts"].as_str().unwrap_or(""),
                    cursor["id"].as_str().unwrap_or(""),
                    200,
                )
                .await
                .map_err(internal)?;
            for row in &rows {
                let detail: Value = serde_json::from_str(&row.detail_json).unwrap_or(Value::Null);
                let suspected = row.action == "session-security-alert"
                    && matches!(
                        detail["kind"].as_str(),
                        Some("impossibleTravel" | "concurrentUse" | "rotationReuse")
                    );
                if !suspected {
                    continue;
                }
                let id = format!("operational-security:{}", row.id);
                let detected = chrono::DateTime::parse_from_rfc3339(&row.ts)
                    .map(|t| t.timestamp())
                    .unwrap_or(at);
                let mut body = Incident {
                    title: "Suspected account compromise".into(),
                    category: "account-compromise".into(),
                    source: "security-audit".into(),
                    status: "detected".into(),
                    significance: "unassessed".into(),
                    detected_at: detected,
                    audit_reference: row.id.clone(),
                    ..Default::default()
                };
                body.event("detected","Session-security signal detected. Inspect the referenced audit event; this is not a confirmed compromise.","system",at);
                if state
                    .store
                    .insert_compliance_record_once(&record(id.clone(), &body, "system", at)?)
                    .await
                    .map_err(internal)?
                {
                    audit(state, &id, "operational-incident-detected").await;
                }
            }
            if let Some(last) = rows.last() {
                let mut next = old.clone();
                next.body_json = json!({"ts":last.ts,"id":last.id}).to_string();
                next.updated_at = at;
                state
                    .store
                    .transition_compliance_record(&old, &next)
                    .await
                    .map_err(internal)?;
            }
        } else {
            state.store.insert_compliance_record_once(&ComplianceRecord{id:cursor_id.into(),kind:MONITOR.into(),name:"audit-cursor".into(),body_json:json!({"ts":chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos,false),"id":""}).to_string(),updated_by:"system".into(),updated_at:at}).await.map_err(internal)?;
        }
    }
    let trust = enabled(&config, "incidents.nis2.trustService");
    for row in state
        .store
        .list_compliance_records(KIND)
        .await
        .map_err(internal)?
    {
        let mut body = decode(&row)?;
        let times = deadlines(&body, trust);
        let mut changed = false;
        for phase in ["early-warning", "notification", "progress", "final"] {
            let Some(due) = times[phase].as_i64() else {
                continue;
            };
            if due - at > 6 * 3600
                || body.reminded.iter().any(|p| p == phase)
                || body.reports.iter().any(|r| r.phase == phase)
            {
                continue;
            }
            if phase == "progress" && body.resolved_at.is_some() {
                continue;
            }
            if phase == "final" && body.resolved_at.is_none() {
                continue;
            }
            body.reminded.push(phase.into());
            body.event("report-due", phase, "system", at);
            changed = true;
        }
        if enabled(&config, "incidents.notify.email")
            && !(body.status == "closed" && row.updated_at < at - 86400)
        {
            let mut notifications = body.reminded.clone();
            notifications.push("detected".into());
            if body.first_healthy_at.is_some() {
                notifications.push("recovery-observed".into());
            }
            if body.verified_at.is_some() {
                notifications.push("recovery-verified".into());
            }
            let addresses: Vec<_> = config
                .get("incidents.notify.operatorAddresses")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .collect();
            for phase in notifications {
                if body.emailed.contains(&phase)
                    || body.reports.iter().any(|r| r.phase == phase)
                    || addresses.is_empty()
                {
                    continue;
                }
                let mut queued = true;
                for address in &addresses {
                    queued &= crate::compliance_mail::enqueue(state,&format!("operational:{}:{phase}",row.id),address,"Mailwoman operational incident update",&format!("Incident {}: {phase}. Review Administration > Compliance > Operational incidents. This is an operational notice, not a regulatory submission.",row.id)).await.is_ok();
                }
                if queued {
                    body.emailed.push(phase);
                    changed = true;
                }
            }
        }
        if changed {
            update(state, &row, &body, "system", at).await?;
            audit(state, &row.id, "operational-incident-reminder").await;
        }
        let configured = config
            .get("incidents.retention.days")
            .and_then(Value::as_i64)
            .unwrap_or(1095);
        let (classes, enforce) = crate::retention_tiers::class_days(&state.store).await;
        let days = crate::retention_tiers::clamp(
            crate::auth_logs::sweeper::Unit::Age,
            Some(crate::retention_tiers::Class::Long),
            configured,
            &classes,
            enforce,
        )
        .max(1);

        let holds = config
            .get("dsr.legalHold.subjects")
            .and_then(Value::as_array)
            .is_some_and(|holds| !holds.is_empty());
        if !changed && !holds && body.status == "closed" && row.updated_at < at - days * 86400 {
            state
                .store
                .delete_compliance_record(&row.id)
                .await
                .map_err(internal)?;
            audit(state, &row.id, "operational-incident-expired").await;
        }
    }
    Ok(())
}
pub(crate) fn start(state: AppState) {
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            tick(&state).await;
        }
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn final_uses_calendar_month_after_notification() {
        let mut b = Incident {
            aware_at: Some(1_769_817_600),
            significance: "significant".into(),
            ..Default::default()
        };
        b.reports.push(Report {
            phase: "notification".into(),
            submitted_at: 1_769_817_600,
            ..Default::default()
        });
        assert_eq!(
            deadlines(&b, false)["progress"],
            json!(month_after(1_769_817_600))
        );
        assert!(deadlines(&b, false).get("final").is_none());
        b.resolved_at = Some(1_769_817_600 + 86400);
        assert_eq!(
            deadlines(&b, false)["final"],
            json!(month_after(1_769_817_600))
        );
        let late = month_after(1_769_817_600).unwrap() + 86400;
        b.resolved_at = Some(late);
        assert_eq!(deadlines(&b, false)["final"], json!(month_after(late)));
        assert_eq!(
            deadlines(&b, true)["notification"],
            json!(1_769_817_600 + 86400)
        );
    }
    #[test]
    fn detection_is_not_significance_or_verified_recovery() {
        let b = Incident {
            detected_at: 10,
            first_healthy_at: Some(20),
            ..Default::default()
        };
        assert_eq!(deadlines(&b, false), json!({}));
        assert!(b.verified_at.is_none());
    }
}
