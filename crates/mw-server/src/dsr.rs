//! Data subject requests (access, download, correction, erasure, restriction,
//! objection, other): the user and admin endpoints, the request lifecycle and the
//! background worker.
//!
//! A person files a request in Settings; it moves through confirmation, an
//! optional identity check and approval (each switched by a `dsr.*` setting, per
//! domain), then either builds an export or schedules an erasure after a grace
//! period. Every change is an entry in the request's trail. The settings are in
//! the registry (`dsr.*`); see `docs/admin/data-requests.md`.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx},
    admin_router::AdminRegistrar,
    audit::{SecurityEvent, security_event},
    capabilities::Capability as C,
    dsr_data::{self, Options, SECTIONS, Subj},
    settings_registry::{self},
    step_up::{self, StepUpAction},
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use mw_store::{dsr::DsrRow, settings_policy::SettingSubject};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;

pub(crate) const KINDS: [&str; 7] = [
    "access",
    "portability",
    "rectification",
    "erasure",
    "restriction",
    "objection",
    "other",
];
const OPEN: [&str; 9] = [
    "awaiting_confirmation",
    "awaiting_verification",
    "awaiting_approval",
    "queued",
    "processing",
    "scheduled",
    "in_progress",
    "blocked",
    "failed",
];
const DAY: i64 = 86_400;
const CONFIRM_TTL_DAYS: i64 = 7;
const MAX_CODE_TRIES: i64 = 5;

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

fn is_open(status: &str) -> bool {
    OPEN.contains(&status)
}

// ─── failures ─────────────────────────────────────────────────────────────────

pub(crate) struct Fail {
    status: StatusCode,
    code: &'static str,
    detail: String,
    raw: Option<Response>,
}

impl Fail {
    fn new(status: StatusCode, code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            status,
            code,
            detail: detail.into(),
            raw: None,
        }
    }
    /// A response built elsewhere (a step-up demand) passed through unchanged.
    fn raw(r: Response) -> Self {
        Self {
            status: r.status(),
            code: "raw",
            detail: String::new(),
            raw: Some(r),
        }
    }
    fn conflict() -> Self {
        Self::new(
            StatusCode::CONFLICT,
            "changed",
            "The request changed while you were working on it; reload and try again.",
        )
    }
    fn state(detail: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, "wrongState", detail)
    }
}

impl From<mw_store::StoreError> for Fail {
    fn from(e: mw_store::StoreError) -> Self {
        tracing::warn!("data request store error: {e}");
        Self::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "unavailable",
            "The request store is unavailable.",
        )
    }
}

impl IntoResponse for Fail {
    fn into_response(self) -> Response {
        if let Some(r) = self.raw {
            return r;
        }
        (
            self.status,
            Json(json!({"code": self.code, "error": self.detail})),
        )
            .into_response()
    }
}

impl From<Fail> for AdminApiError {
    fn from(f: Fail) -> Self {
        AdminApiError::new(f.status, f.code, f.detail)
    }
}

// ─── policy ───────────────────────────────────────────────────────────────────

/// The effective `dsr.*` values for one person.
#[derive(Clone, Debug, Default)]
pub(crate) struct Policy {
    map: HashMap<String, Value>,
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
            d.key.starts_with("dsr.") || d.key == "dataExport.linkHours"
        })
        .await
        .unwrap_or_default();
        Self {
            map: rows.into_iter().map(|e| (e.key, e.value)).collect(),
        }
    }
    fn get(&self, key: &str) -> Value {
        self.map
            .get(key)
            .cloned()
            .or_else(|| settings_registry::lookup(key).map(|d| d.default_value()))
            .unwrap_or(Value::Null)
    }
    fn b(&self, key: &str) -> bool {
        self.get(key).as_bool().unwrap_or(false)
    }
    fn n(&self, key: &str) -> i64 {
        self.get(key).as_i64().unwrap_or(0)
    }
    fn s(&self, key: &str) -> String {
        self.get(key).as_str().unwrap_or_default().to_string()
    }
    fn strings(&self, key: &str) -> Vec<String> {
        self.get(key)
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_lowercase)).collect())
            .unwrap_or_default()
    }
    fn reminders(&self) -> Vec<i64> {
        self.get("dsr.sla.reminderDays")
            .as_array()
            .map(|a| a.iter().filter_map(Value::as_i64).collect())
            .unwrap_or_default()
    }
    /// A localizable text: the person's language, then English, then any.
    fn text(&self, key: &str, locale: &str) -> Option<String> {
        let v = self.get(key);
        let m = v.as_object()?;
        m.get(locale)
            .or_else(|| m.get("en"))
            .or_else(|| m.values().next())
            .and_then(Value::as_str)
            .map(str::to_string)
    }
    fn kind_enabled(&self, kind: &str) -> bool {
        self.b("dsr.enabled") && self.b(&format!("dsr.type.{kind}"))
    }
    fn approval(&self, kind: &str) -> String {
        match kind {
            "access" | "portability" => self.s("dsr.approval.export"),
            "erasure" => self.s("dsr.approval.erasure"),
            _ => self.s("dsr.approval.other"),
        }
    }
    /// The legal hold on an address, if any.
    fn hold(&self, body: &Body, address: &str) -> Option<Value> {
        if let Some(h) = &body.hold {
            return Some(json!({"active": true, "source": "request", "reason": h}));
        }
        let domain = crate::admin_ctx::domain_of(address);
        let subjects = self.strings("dsr.legalHold.subjects");
        let hit = subjects.iter().any(|s| {
            let s = s.trim_start_matches('@');
            s == address || (!s.contains('@') && s == domain)
        });
        hit.then(|| json!({"active": true, "source": "policy", "reason": "listed in the legal hold setting"}))
    }
    /// The settings a client may show.
    fn client_view(&self) -> Value {
        let mut types = serde_json::Map::new();
        for k in KINDS {
            types.insert(k.into(), json!(self.kind_enabled(k)));
        }
        json!({
            "enabled": self.b("dsr.enabled"),
            "types": types,
            "reauthRequired": self.b("dsr.reauth.required"),
            "confirmationRequired": self.b("dsr.confirmation.required"),
            "identityVerificationRequired": self.b("dsr.identityVerification.required"),
            "approval": {"export": self.s("dsr.approval.export"), "erasure": self.s("dsr.approval.erasure"), "other": self.s("dsr.approval.other")},
            "graceDays": self.n("dsr.erasure.graceDays"),
            "slaDays": self.n("dsr.sla.days"),
            "mailAllowed": self.b("dsr.export.mailAllowed"),
            "mailFormat": self.s("dsr.export.mailFormat"),
            "singleUseDownload": self.b("dsr.download.singleUse"),
            "linkMinutes": self.n("dsr.download.linkMinutes"),
            "exportHours": self.n("dataExport.linkHours"),
            "contact": {"name": self.get("dsr.contact.name"), "email": self.get("dsr.contact.email"), "url": self.get("dsr.contact.url")},
            "text": {"intro": self.get("dsr.text.intro")},
        })
    }
}

// ─── stored body ──────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct Body {
    details: String,
    include_mail: bool,
    requested_by: String,
    locale: String,
    owners: Vec<String>,
    confirm_hash: Option<String>,
    confirm_tries: i64,
    confirmed_at: Option<i64>,
    verified_at: Option<i64>,
    verified_by: Option<String>,
    approved_at: Option<i64>,
    approved_by: Option<String>,
    decision_reason: Option<String>,
    grace_until: Option<i64>,
    original_due_at: Option<i64>,
    extension_days: i64,
    extension_reason: Option<String>,
    hold: Option<String>,
    answered_at: Option<i64>,
    completion: Option<Value>,
    export: Option<ExportInfo>,
    download: Option<Download>,
    downloads: i64,
    lease_until: i64,
    attempts: i64,
    error: Option<String>,
    reminded: bool,
    overdue_flagged: bool,
    cancelled_by: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
struct ExportInfo {
    bytes: u64,
    expires_at: i64,
    done: Vec<String>,
    sections: Vec<Value>,
    complete: bool,
    sealed: bool,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
struct Download {
    hash: String,
    expires_at: i64,
    used: bool,
}

fn body_of(row: &DsrRow) -> Body {
    serde_json::from_str(&row.body_json).unwrap_or_default()
}

async fn save(state: &AppState, row: &mut DsrRow, body: &Body, status: &str) -> Result<(), Fail> {
    row.status = status.to_string();
    row.body_json = serde_json::to_string(body).unwrap_or_else(|_| "{}".into());
    if state.store.dsr_update(row, now()).await? {
        row.rev += 1;
        Ok(())
    } else {
        Err(Fail::conflict())
    }
}

async fn event(state: &AppState, id: &str, actor: &str, kind: &str, detail: Value) {
    if let Err(e) = state.store.dsr_event_add(id, actor, kind, &detail, now()).await {
        tracing::warn!("data request event {kind} not recorded: {e}");
    }
}

fn due_state(row: &DsrRow, policy: &Policy, at: i64) -> &'static str {
    if !is_open(&row.status) {
        return "none";
    }
    if at > row.due_at {
        return "overdue";
    }
    let widest = policy.reminders().into_iter().max().unwrap_or(0);
    if row.due_at - at <= widest * DAY { "dueSoon" } else { "ok" }
}

// ─── views ────────────────────────────────────────────────────────────────────

fn user_view(row: &DsrRow, body: &Body, policy: &Policy) -> Value {
    let at = now();
    let link_open = matches!(row.status.as_str(), "ready" | "downloaded")
        && body.export.as_ref().is_some_and(|e| e.expires_at > at);
    json!({
        "id": row.id, "kind": row.kind, "status": row.status,
        "createdAt": row.created_at, "updatedAt": row.updated_at, "dueAt": row.due_at,
        "details": body.details, "requestedBy": if body.requested_by == "self" { "self" } else { "admin" },
        "confirmed": body.confirmed_at.is_some(),
        "needsConfirmation": row.status == "awaiting_confirmation",
        "graceUntil": body.grace_until,
        "canCancel": matches!(row.status.as_str(), "awaiting_confirmation" | "awaiting_verification" | "awaiting_approval" | "queued" | "scheduled" | "in_progress" | "blocked" | "failed"),
        "decisionReason": body.decision_reason,
        "extensionDays": body.extension_days, "extensionReason": body.extension_reason,
        "export": body.export.as_ref().map(|e| json!({"bytes": e.bytes, "expiresAt": e.expires_at, "complete": e.complete, "sections": e.sections})),
        "downloadAvailable": link_open,
        "downloads": body.downloads,
        "completion": body.completion.as_ref().map(|c| json!({"completedAt": c["completedAt"], "retained": c["retained"], "digest": c["digest"]})),
        "dueState": due_state(row, policy, at),
    })
}

fn admin_view(row: &DsrRow, body: &Body, policy: &Policy) -> Value {
    let mut v = user_view(row, body, policy);
    let o = v.as_object_mut().expect("object");
    o.insert("address".into(), json!(row.address));
    o.insert("domain".into(), json!(row.domain));
    o.insert("assignee".into(), json!(row.assignee));
    o.insert("requestedBy".into(), json!(body.requested_by));
    o.insert("verifiedAt".into(), json!(body.verified_at));
    o.insert("verifiedBy".into(), json!(body.verified_by));
    o.insert("approvedAt".into(), json!(body.approved_at));
    o.insert("approvedBy".into(), json!(body.approved_by));
    o.insert("originalDueAt".into(), json!(body.original_due_at.unwrap_or(row.due_at)));
    o.insert("legalHold".into(), policy.hold(body, &row.address).unwrap_or(json!({"active": false})));
    o.insert("error".into(), json!(body.error));
    o.insert("completion".into(), json!(body.completion));
    o.insert("answeredAt".into(), json!(body.answered_at));
    o.insert("requiresVerification".into(), json!(policy.b("dsr.identityVerification.required")));
    o.insert("approvalMode".into(), json!(policy.approval(&row.kind)));
    v
}

// ─── lifecycle ────────────────────────────────────────────────────────────────

fn random_token(bytes: usize) -> String {
    use base64::Engine as _;
    let mut b = vec![0u8; bytes];
    rand::thread_rng().fill_bytes(&mut b);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b)
}

fn new_code() -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut b = [0u8; 8];
    rand::thread_rng().fill_bytes(&mut b);
    b.iter().map(|x| ALPHABET[(*x as usize) % ALPHABET.len()] as char).collect()
}

fn code_hash(id: &str, code: &str) -> String {
    crate::push_relay::hash_token(&format!("{id}:{}", code.trim().to_uppercase()))
}

/// Put a message in the requester's inbox (best effort, in the background).
fn notice(state: &AppState, row: &DsrRow, body: &Body, policy: &Policy, subject: &str, description: &str, force: bool, extra: Option<String>) {
    if !force && !policy.b("dsr.notify.requester") {
        return;
    }
    let (state, account, id) = (state.clone(), row.account_id.clone(), row.id.clone());
    let locale = if body.locale.is_empty() { "en".to_string() } else { body.locale.clone() };
    let (subject, description) = (subject.to_string(), description.to_string());
    let guidance = extra.unwrap_or_default();
    tokio::spawn(async move {
        let result = crate::alerts::deliver_text(&state, &account, &locale, &subject, &description, &guidance).await;
        match result {
            Ok(()) => event(&state, &id, "system", "notice-sent", json!({"subject": subject})).await,
            Err(e) => event(&state, &id, "system", "notice-failed", json!({"reason": e})).await,
        }
    });
}

async fn admin_signal(state: &AppState, policy: &Policy, action: &'static str, row: &DsrRow) {
    if policy.b("dsr.notify.admins") {
        security_event(
            state,
            SecurityEvent::system(action)
                .target(format!("dsr:{}", row.id))
                .detail(json!({"kind": row.kind, "domain": row.domain, "status": row.status})),
        )
        .await;
    }
}

/// Where a confirmed (or admin-created) request goes next.
async fn advance(state: &AppState, row: &mut DsrRow, body: &mut Body, policy: &Policy, actor: &str) -> Result<(), Fail> {
    if policy.b("dsr.identityVerification.required") && body.verified_at.is_none() {
        save(state, row, body, "awaiting_verification").await?;
        event(state, &row.id, actor, "awaiting-verification", json!({})).await;
        return Ok(());
    }
    if policy.approval(&row.kind) == "admin" && body.approved_at.is_none() {
        save(state, row, body, "awaiting_approval").await?;
        event(state, &row.id, actor, "awaiting-approval", json!({})).await;
        return Ok(());
    }
    approve(state, row, body, policy, "system").await
}

/// Approve: export requests are queued, an erasure is scheduled after the grace
/// period (or blocked by a legal hold), other kinds go to an administrator.
async fn approve(state: &AppState, row: &mut DsrRow, body: &mut Body, policy: &Policy, actor: &str) -> Result<(), Fail> {
    body.approved_at = Some(now());
    body.approved_by = Some(actor.to_string());
    match row.kind.as_str() {
        "access" | "portability" => {
            save(state, row, body, "queued").await?;
            event(state, &row.id, actor, "approved", json!({"next": "queued"})).await;
            notice(state, row, body, policy, "Your data export is being prepared", "Your request for a copy of your data was approved. You will find it under Settings > Privacy & data when it is ready.", false, policy.text("dsr.text.confirmation", &body.locale));
        }
        "erasure" => {
            if let Some(h) = policy.hold(body, &row.address) {
                save(state, row, body, "blocked").await?;
                event(state, &row.id, actor, "blocked-by-hold", h).await;
                admin_signal(state, policy, "dsr-erasure-blocked", row).await;
                return Ok(());
            }
            let grace = policy.n("dsr.erasure.graceDays");
            body.grace_until = Some(now() + grace * DAY);
            save(state, row, body, "scheduled").await?;
            event(state, &row.id, actor, "approved", json!({"next": "scheduled", "graceUntil": body.grace_until})).await;
            notice(state, row, body, policy, "Your erasure request is scheduled", &format!("Your account data will be erased after {grace} day(s). You can cancel the request under Settings > Privacy & data until then."), false, None);
        }
        _ => {
            save(state, row, body, "in_progress").await?;
            event(state, &row.id, actor, "approved", json!({"next": "in_progress"})).await;
        }
    }
    Ok(())
}

struct NewRequest<'a> {
    kind: &'a str,
    details: String,
    include_mail: bool,
    locale: String,
    address: String,
    domain: String,
    account_id: String,
    owners: Vec<String>,
    requested_by: String,
}

async fn create(state: &AppState, policy: &Policy, n: NewRequest<'_>) -> Result<(DsrRow, Body, bool), Fail> {
    let at = now();
    let on_behalf = n.requested_by != "self";
    let mut body = Body {
        details: n.details.chars().take(2000).collect(),
        include_mail: n.include_mail,
        requested_by: n.requested_by.clone(),
        locale: n.locale,
        owners: n.owners,
        ..Default::default()
    };
    let id = uuid::Uuid::new_v4().to_string();
    let mut code = None;
    if !on_behalf && policy.b("dsr.confirmation.required") {
        let c = new_code();
        body.confirm_hash = Some(code_hash(&id, &c));
        code = Some(c);
    }
    let mut row = DsrRow {
        id: id.clone(),
        address: n.address,
        domain: n.domain,
        account_id: n.account_id,
        kind: n.kind.to_string(),
        status: "awaiting_confirmation".into(),
        created_at: at,
        updated_at: at,
        due_at: at + policy.n("dsr.sla.days") * DAY,
        assignee: None,
        rev: 1,
        body_json: String::new(),
    };
    row.body_json = serde_json::to_string(&body).unwrap_or_default();
    state.store.dsr_insert(&row).await?;
    let actor = if on_behalf { n.requested_by.as_str() } else { "requester" };
    event(state, &id, actor, "submitted", json!({"kind": row.kind, "dueAt": row.due_at})).await;
    admin_signal(state, policy, "dsr-request-submitted", &row).await;
    let sent = if let Some(c) = &code {
        let text = policy.text("dsr.text.confirmation", &body.locale);
        notice(state, &row, &body, policy, "Confirm your data request", &format!("Your confirmation code for the {} request is {c}. It is valid for {CONFIRM_TTL_DAYS} days. If you did not make this request, ignore this message and it will be cancelled.", row.kind), true, text);
        true
    } else {
        advance(state, &mut row, &mut body, policy, actor).await?;
        false
    };
    Ok((row, body, sent))
}

// ─── subject resolution ───────────────────────────────────────────────────────

async fn account_ids_of(state: &AppState, address: &str, primary: &str) -> Vec<String> {
    let mut ids = vec![primary.to_string()];
    if let Ok(bound) = state.store.lifecycle_account_bindings(address).await {
        ids.extend(bound);
    }
    ids.sort();
    ids.dedup();
    ids.retain(|i| !i.is_empty());
    ids
}

async fn owners_of(state: &AppState, account_ids: &[String], stored: &[String]) -> Vec<String> {
    let mut owners: Vec<String> = stored.to_vec();
    for id in account_ids {
        if let Ok(Some(o)) = state.store.native_personal_subject(id).await {
            owners.push(o);
        }
        if let Ok(sessions) = state.store.sessions_by_account(id).await {
            for s in sessions {
                for o in [
                    state.store.session_personal_subject(&s.id).await,
                    state.store.session_storage_authority(&s.id).await,
                ]
                .into_iter()
                .flatten()
                .flatten()
                {
                    owners.push(o);
                }
            }
        }
    }
    owners.sort();
    owners.dedup();
    owners
}

async fn subj_of(state: &AppState, row: &DsrRow, body: &Body) -> Subj {
    let account_ids = account_ids_of(state, &row.address, &row.account_id).await;
    let owners = owners_of(state, &account_ids, &body.owners).await;
    Subj {
        address: row.address.clone(),
        account_ids,
        owners,
    }
}

// ─── user routes ──────────────────────────────────────────────────────────────

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/privacy/requests/overview", get(overview))
        .route("/api/privacy/requests", get(list_own).post(submit))
        .route("/api/privacy/requests/{id}", get(get_own))
        .route("/api/privacy/requests/{id}/confirm", post(confirm))
        .route("/api/privacy/requests/{id}/cancel", post(cancel))
        .route("/api/privacy/requests/{id}/download-link", post(download_link))
        .route("/api/privacy/requests/{id}/download", get(download))
}

struct Me {
    session: mw_store::Session,
    address: String,
    domain: String,
    account_id: String,
    owners: Vec<String>,
}

async fn me(state: &AppState, headers: &HeaderMap) -> Result<Me, Response> {
    let session = crate::authed(state, headers).await?;
    let subject = crate::settings_routes::UserSubject::resolved(&state.store, &session)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE.into_response())?;
    let mut owners: Vec<String> = [&subject.record_owner, &subject.personal_owner, &subject.proxy_policy_owner]
        .into_iter()
        .flatten()
        .cloned()
        .collect();
    owners.sort();
    owners.dedup();
    Ok(Me {
        address: subject.address.clone(),
        domain: subject.domain.clone(),
        account_id: subject.account_id.clone(),
        owners,
        session,
    })
}

async fn my_policy(state: &AppState, m: &Me) -> Policy {
    Policy::load(state, &m.domain, &m.address, &m.account_id).await
}

async fn mine(state: &AppState, m: &Me, id: &str) -> Result<(DsrRow, Body), Fail> {
    match state.store.dsr_get(id).await? {
        Some(row) if row.address == m.address => {
            let body = body_of(&row);
            Ok((row, body))
        }
        _ => Err(Fail::new(StatusCode::NOT_FOUND, "notFound", "No such request.")),
    }
}

async fn overview(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let m = match me(&state, &headers).await {
        Ok(m) => m,
        Err(r) => return r,
    };
    let policy = my_policy(&state, &m).await;
    no_store(Json(json!({"policy": policy.client_view(), "kinds": KINDS, "sections": SECTIONS})))
}

fn no_store(r: impl IntoResponse) -> Response {
    ([(header::CACHE_CONTROL, "no-store")], r).into_response()
}

async fn list_own(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let m = match me(&state, &headers).await {
        Ok(m) => m,
        Err(r) => return r,
    };
    let policy = my_policy(&state, &m).await;
    match state.store.dsr_list(Some(&m.address)).await {
        Ok(rows) => no_store(Json(json!({
            "requests": rows.iter().map(|r| user_view(r, &body_of(r), &policy)).collect::<Vec<_>>()
        }))),
        Err(e) => Fail::from(e).into_response(),
    }
}

async fn get_own(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<String>) -> Response {
    let m = match me(&state, &headers).await {
        Ok(m) => m,
        Err(r) => return r,
    };
    let policy = my_policy(&state, &m).await;
    let result: Result<Value, Fail> = async {
        let (row, body) = mine(&state, &m, &id).await?;
        let events = state.store.dsr_events(&id).await?;
        let trail: Vec<Value> = events
            .iter()
            .map(|e| json!({"ts": e.ts, "kind": e.kind, "by": if e.actor == "requester" { "you" } else if e.actor == "system" { "system" } else { "administrator" }}))
            .collect();
        Ok(json!({"request": user_view(&row, &body, &policy), "trail": trail}))
    }
    .await;
    match result {
        Ok(v) => no_store(Json(v)),
        Err(f) => f.into_response(),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Submit {
    kind: String,
    #[serde(default)]
    details: String,
    #[serde(default)]
    include_mail: bool,
    #[serde(default)]
    locale: String,
}

fn locale_of(raw: &str) -> String {
    if settings_registry::LOCALES.contains(&raw) { raw.to_string() } else { "en".into() }
}

async fn submit(State(state): State<AppState>, headers: HeaderMap, Json(req): Json<Submit>) -> Response {
    let m = match me(&state, &headers).await {
        Ok(m) => m,
        Err(r) => return r,
    };
    let policy = my_policy(&state, &m).await;
    let result: Result<Response, Fail> = async {
        if !KINDS.contains(&req.kind.as_str()) {
            return Err(Fail::new(StatusCode::BAD_REQUEST, "invalidKind", "Unknown request type."));
        }
        if !policy.kind_enabled(&req.kind) {
            return Err(Fail::new(StatusCode::FORBIDDEN, "requestTypeDisabled", "This request type is not available."));
        }
        if matches!(req.kind.as_str(), "access" | "portability" | "erasure") && policy.b("dsr.reauth.required") {
            step_up::require_user(&state, &m.session, StepUpAction::DataExport)
                .await
                .map_err(Fail::raw)?;
        }
        if state.store.dsr_count_since(&m.address, now() - DAY).await? >= policy.n("dsr.rateLimit.perDay") {
            return Err(Fail::new(StatusCode::TOO_MANY_REQUESTS, "rateLimited", "Too many requests today."));
        }
        let open = state.store.dsr_list(Some(&m.address)).await?;
        if open.iter().any(|r| r.kind == req.kind && is_open(&r.status)) {
            return Err(Fail::new(StatusCode::CONFLICT, "alreadyOpen", "You already have an open request of this type."));
        }
        let include_mail = req.include_mail && policy.b("dsr.export.mailAllowed") && matches!(req.kind.as_str(), "access" | "portability");
        let (row, body, code_sent) = create(
            &state,
            &policy,
            NewRequest {
                kind: &req.kind,
                details: req.details,
                include_mail,
                locale: locale_of(&req.locale),
                address: m.address.clone(),
                domain: m.domain.clone(),
                account_id: m.account_id.clone(),
                owners: m.owners.clone(),
                requested_by: "self".into(),
            },
        )
        .await?;
        let mut out = user_view(&row, &body, &policy);
        out["confirmationSent"] = json!(code_sent);
        Ok((StatusCode::CREATED, no_store(Json(json!({"request": out})))).into_response())
    }
    .await;
    result.unwrap_or_else(IntoResponse::into_response)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfirmBody {
    code: String,
}

async fn confirm(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<String>, Json(req): Json<ConfirmBody>) -> Response {
    let m = match me(&state, &headers).await {
        Ok(m) => m,
        Err(r) => return r,
    };
    let policy = my_policy(&state, &m).await;
    let result: Result<Response, Fail> = async {
        let (mut row, mut body) = mine(&state, &m, &id).await?;
        if row.status != "awaiting_confirmation" {
            return Err(Fail::state("This request is not waiting for a confirmation code."));
        }
        if body.confirm_tries >= MAX_CODE_TRIES {
            return Err(Fail::new(StatusCode::TOO_MANY_REQUESTS, "tooManyTries", "Too many wrong codes; the request was cancelled."));
        }
        if row.created_at + CONFIRM_TTL_DAYS * DAY < now() {
            return Err(Fail::new(StatusCode::GONE, "codeExpired", "The confirmation code has expired."));
        }
        if body.confirm_hash.as_deref() != Some(code_hash(&id, &req.code).as_str()) {
            body.confirm_tries += 1;
            let status = if body.confirm_tries >= MAX_CODE_TRIES { "cancelled" } else { "awaiting_confirmation" };
            save(&state, &mut row, &body, status).await?;
            event(&state, &id, "requester", "confirmation-failed", json!({"tries": body.confirm_tries})).await;
            return Err(Fail::new(StatusCode::FORBIDDEN, "wrongCode", "That code is not right."));
        }
        body.confirmed_at = Some(now());
        body.confirm_hash = None;
        event(&state, &id, "requester", "confirmed", json!({})).await;
        advance(&state, &mut row, &mut body, &policy, "requester").await?;
        Ok(no_store(Json(json!({"request": user_view(&row, &body, &policy)}))))
    }
    .await;
    result.unwrap_or_else(IntoResponse::into_response)
}

async fn cancel(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<String>) -> Response {
    let m = match me(&state, &headers).await {
        Ok(m) => m,
        Err(r) => return r,
    };
    let policy = my_policy(&state, &m).await;
    let result: Result<Response, Fail> = async {
        let (mut row, mut body) = mine(&state, &m, &id).await?;
        if !is_open(&row.status) || row.status == "processing" {
            return Err(Fail::state("This request can no longer be cancelled."));
        }
        let was = row.status.clone();
        body.cancelled_by = Some("requester".into());
        save(&state, &mut row, &body, "cancelled").await?;
        event(&state, &id, "requester", "cancelled", json!({"was": was})).await;
        admin_signal(&state, &policy, "dsr-request-cancelled", &row).await;
        Ok(no_store(Json(json!({"request": user_view(&row, &body, &policy)}))))
    }
    .await;
    result.unwrap_or_else(IntoResponse::into_response)
}

async fn download_link(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<String>) -> Response {
    let m = match me(&state, &headers).await {
        Ok(m) => m,
        Err(r) => return r,
    };
    let policy = my_policy(&state, &m).await;
    let result: Result<Response, Fail> = async {
        let (mut row, mut body) = mine(&state, &m, &id).await?;
        let export_ok = body.export.as_ref().is_some_and(|e| e.expires_at > now());
        if !matches!(row.status.as_str(), "ready" | "downloaded") || !export_ok {
            return Err(Fail::state("There is no export available to download."));
        }
        if policy.b("dsr.reauth.required") {
            step_up::require_user(&state, &m.session, StepUpAction::DataExport)
                .await
                .map_err(Fail::raw)?;
        }
        let token = random_token(32);
        let expires = (now() + policy.n("dsr.download.linkMinutes") * 60).min(body.export.as_ref().map_or(0, |e| e.expires_at));
        body.download = Some(Download { hash: crate::push_relay::hash_token(&token), expires_at: expires, used: false });
        let status = row.status.clone();
        save(&state, &mut row, &body, &status).await?;
        event(&state, &id, "requester", "download-link-created", json!({"expiresAt": expires})).await;
        Ok(no_store(Json(json!({
            "url": format!("/api/privacy/requests/{id}/download?token={token}"),
            "expiresAt": expires, "singleUse": policy.b("dsr.download.singleUse"),
        }))))
    }
    .await;
    result.unwrap_or_else(IntoResponse::into_response)
}

#[derive(Deserialize)]
struct TokenQuery {
    token: String,
}

async fn download(State(state): State<AppState>, headers: HeaderMap, Path(id): Path<String>, Query(q): Query<TokenQuery>) -> Response {
    let m = match me(&state, &headers).await {
        Ok(m) => m,
        Err(r) => return r,
    };
    let policy = my_policy(&state, &m).await;
    let result: Result<Response, Fail> = async {
        let (mut row, mut body) = mine(&state, &m, &id).await?;
        let Some(dl) = body.download.clone() else {
            return Err(Fail::new(StatusCode::FORBIDDEN, "noLink", "Create a download link first."));
        };
        if dl.hash != crate::push_relay::hash_token(&q.token) {
            return Err(Fail::new(StatusCode::FORBIDDEN, "badLink", "This download link is not valid."));
        }
        if dl.expires_at <= now() || !body.export.as_ref().is_some_and(|e| e.expires_at > now()) {
            return Err(Fail::new(StatusCode::GONE, "linkExpired", "This download link has expired."));
        }
        if dl.used && policy.b("dsr.download.singleUse") {
            return Err(Fail::new(StatusCode::GONE, "linkUsed", "This download link was already used."));
        }
        let names = state.store.dsr_part_names(&id).await?;
        if names.is_empty() {
            return Err(Fail::new(StatusCode::GONE, "linkExpired", "The export is no longer stored."));
        }
        body.download = Some(Download { used: true, ..dl });
        body.downloads += 1;
        save(&state, &mut row, &body, "downloaded").await?;
        event(&state, &id, "requester", "downloaded", json!({"downloads": body.downloads})).await;
        let ts = body.answered_at.unwrap_or(row.updated_at);
        let stream = dsr_data::zip_stream(state.clone(), id.clone(), names, ts);
        Ok((
            [
                (header::CONTENT_TYPE, "application/zip".to_string()),
                (header::CONTENT_DISPOSITION, format!("attachment; filename=\"mailwoman-data-{}.zip\"", &id[..8.min(id.len())])),
                (header::CACHE_CONTROL, "no-store".to_string()),
                (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_string()),
            ],
            axum::body::Body::from_stream(stream),
        )
            .into_response())
    }
    .await;
    result.unwrap_or_else(IntoResponse::into_response)
}

// ─── admin routes ─────────────────────────────────────────────────────────────

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, "/dsr", C::DataExportRun, admin_list);
    r.admin_endpoint(Method::POST, "/dsr", C::DataExportRun, admin_create);
    r.admin_endpoint(Method::GET, "/dsr/export", C::DataExportRun, admin_export);
    r.admin_endpoint(Method::GET, "/dsr/{id}", C::DataExportRun, admin_get);
    r.admin_endpoint(Method::POST, "/dsr/{id}/assign", C::DataExportRun, admin_assign);
    r.admin_endpoint(Method::POST, "/dsr/{id}/verify", C::DataExportRun, admin_verify);
    r.admin_endpoint(Method::POST, "/dsr/{id}/approve", C::DataExportRun, admin_approve);
    r.admin_endpoint(Method::POST, "/dsr/{id}/reject", C::DataExportRun, admin_reject);
    r.admin_endpoint(Method::POST, "/dsr/{id}/extend", C::DataExportRun, admin_extend);
    r.admin_endpoint(Method::POST, "/dsr/{id}/complete", C::DataExportRun, admin_complete);
    r.admin_endpoint(Method::POST, "/dsr/{id}/hold", C::DataExportRun, admin_hold);
    r.admin_endpoint(Method::POST, "/dsr/{id}/release-hold", C::DataExportRun, admin_release);
    r.admin_endpoint(Method::POST, "/dsr/{id}/run", C::DataExportRun, admin_run);
}

async fn scoped(state: &AppState, ctx: &AdminCtx, id: &str) -> Result<(DsrRow, Body, Policy), AdminApiError> {
    let row = state.store.dsr_get(id).await?.ok_or_else(|| AdminApiError::not_found("No such request."))?;
    ctx.require_domain(&row.domain)?;
    let body = body_of(&row);
    let policy = Policy::load(state, &row.domain, &row.address, &row.account_id).await;
    Ok((row, body, policy))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ListQuery {
    status: Option<String>,
    kind: Option<String>,
    domain: Option<String>,
    assignee: Option<String>,
    q: Option<String>,
    due: Option<String>,
    limit: Option<usize>,
}

async fn filtered(state: &AppState, ctx: &AdminCtx, q: &ListQuery) -> Result<Vec<DsrRow>, AdminApiError> {
    let all = state.store.dsr_list(None).await?;
    let scope = ctx.domain_filter(C::DataExportRun).cloned();
    let needle = q.q.as_deref().map(str::to_lowercase).filter(|s| !s.is_empty());
    let at = now();
    Ok(all
        .into_iter()
        .filter(|r| scope.as_ref().is_none_or(|s| s.contains(&r.domain)))
        .filter(|r| q.status.as_deref().is_none_or(|s| s.is_empty() || s == "open" && is_open(&r.status) || s == r.status))
        .filter(|r| q.kind.as_deref().is_none_or(|k| k.is_empty() || k == r.kind))
        .filter(|r| q.domain.as_deref().is_none_or(|d| d.is_empty() || d == r.domain))
        .filter(|r| q.assignee.as_deref().is_none_or(|a| a.is_empty() || (a == "none" && r.assignee.is_none()) || r.assignee.as_deref() == Some(a)))
        .filter(|r| match q.due.as_deref() {
            Some("overdue") => is_open(&r.status) && r.due_at < at,
            Some("soon") => is_open(&r.status) && r.due_at >= at && r.due_at - at <= 7 * DAY,
            _ => true,
        })
        .filter(|r| needle.as_ref().is_none_or(|n| r.address.contains(n.as_str()) || r.id.starts_with(n.as_str())))
        .collect())
}

fn stats(rows: &[DsrRow]) -> Value {
    let at = now();
    let mut by_status: HashMap<&str, i64> = HashMap::new();
    let mut by_kind: HashMap<&str, i64> = HashMap::new();
    let (mut open, mut overdue, mut soon, mut answered, mut on_time, mut days_total) = (0i64, 0i64, 0i64, 0i64, 0i64, 0i64);
    for r in rows {
        *by_status.entry(r.status.as_str()).or_insert(0) += 1;
        *by_kind.entry(r.kind.as_str()).or_insert(0) += 1;
        let body = body_of(r);
        if is_open(&r.status) {
            open += 1;
            if r.due_at < at {
                overdue += 1;
            } else if r.due_at - at <= 7 * DAY {
                soon += 1;
            }
        }
        if let Some(a) = body.answered_at {
            answered += 1;
            if a <= r.due_at {
                on_time += 1;
            }
            days_total += (a - r.created_at).max(0);
        }
    }
    let today = at / DAY * DAY;
    let series: Vec<Value> = (0..30)
        .rev()
        .map(|d| {
            let t = today - d * DAY;
            json!({"t": t, "count": rows.iter().filter(|r| r.created_at >= t && r.created_at < t + DAY).count()})
        })
        .collect();
    json!({
        "total": rows.len(), "open": open, "overdue": overdue, "dueSoon": soon,
        "answered": answered, "onTime": on_time,
        "avgAnswerHours": if answered > 0 { Some(days_total as f64 / answered as f64 / 3600.0) } else { None },
        "byStatus": by_status, "byKind": by_kind, "created30d": series,
    })
}

async fn admin_list(State(state): State<AppState>, ctx: AdminCtx, Query(q): Query<ListQuery>) -> Result<Response, AdminApiError> {
    let everything = filtered(&state, &ctx, &ListQuery::default()).await?;
    let rows = filtered(&state, &ctx, &q).await?;
    let limit = q.limit.unwrap_or(200).clamp(1, 1000);
    let mut policies: HashMap<String, Policy> = HashMap::new();
    let mut views = Vec::new();
    for r in rows.iter().take(limit) {
        if !policies.contains_key(&r.domain) {
            policies.insert(r.domain.clone(), Policy::load(&state, &r.domain, &r.address, &r.account_id).await);
        }
        views.push(admin_view(r, &body_of(r), &policies[&r.domain]));
    }
    Ok(no_store(Json(json!({"requests": views, "matched": rows.len(), "stats": stats(&everything), "kinds": KINDS}))))
}

async fn admin_export(State(state): State<AppState>, ctx: AdminCtx, Query(q): Query<ListQuery>) -> Result<Response, AdminApiError> {
    let rows = filtered(&state, &ctx, &q).await?;
    let mut out = Vec::new();
    for r in &rows {
        let b = body_of(r);
        out.push(json!({
            "id": r.id, "address": r.address, "domain": r.domain, "kind": r.kind, "status": r.status,
            "createdAt": r.created_at, "dueAt": r.due_at, "answeredAt": b.answered_at,
            "assignee": r.assignee, "requestedBy": b.requested_by, "decisionReason": b.decision_reason,
        }));
    }
    let csv = dsr_data::csv(&out, &["id", "address", "domain", "kind", "status", "createdAt", "dueAt", "answeredAt", "assignee", "requestedBy", "decisionReason"]);
    Ok((
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"data-requests.csv\""),
            (header::CACHE_CONTROL, "no-store"),
        ],
        csv,
    )
        .into_response())
}

async fn admin_get(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>) -> Result<Response, AdminApiError> {
    let (row, body, policy) = scoped(&state, &ctx, &id).await?;
    let events = state.store.dsr_events(&id).await?;
    Ok(no_store(Json(json!({"request": admin_view(&row, &body, &policy), "trail": events}))))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CreateOnBehalf {
    address: String,
    kind: String,
    #[serde(default)]
    details: String,
    #[serde(default)]
    include_mail: bool,
    #[serde(default)]
    locale: String,
}

async fn admin_create(State(state): State<AppState>, ctx: AdminCtx, Json(req): Json<CreateOnBehalf>) -> Result<Response, AdminApiError> {
    let address = state.store.canonical_policy_address(&req.address).await?;
    let domain = crate::admin_ctx::domain_of(&address);
    ctx.require_domain(&domain)?;
    if !KINDS.contains(&req.kind.as_str()) {
        return Err(Fail::new(StatusCode::BAD_REQUEST, "invalidKind", "Unknown request type.").into());
    }
    let ids = state.store.lifecycle_account_bindings(&address).await?;
    let Some(account_id) = ids.first().cloned() else {
        return Err(AdminApiError::not_found("No account of this address has signed in to this server."));
    };
    let policy = Policy::load(&state, &domain, &address, &account_id).await;
    if !policy.b("dsr.onBehalf.allowed") {
        return Err(Fail::new(StatusCode::FORBIDDEN, "onBehalfDisabled", "Filing requests for users is switched off.").into());
    }
    if !policy.kind_enabled(&req.kind) {
        return Err(Fail::new(StatusCode::FORBIDDEN, "requestTypeDisabled", "This request type is not available.").into());
    }
    let open = state.store.dsr_list(Some(&address)).await?;
    if open.iter().any(|r| r.kind == req.kind && is_open(&r.status)) {
        return Err(Fail::new(StatusCode::CONFLICT, "alreadyOpen", "This person already has an open request of this type.").into());
    }
    let include_mail = req.include_mail && policy.b("dsr.export.mailAllowed") && matches!(req.kind.as_str(), "access" | "portability");
    let owners = owners_of(&state, &ids, &[]).await;
    let (row, body, _) = create(
        &state,
        &policy,
        NewRequest {
            kind: &req.kind,
            details: req.details,
            include_mail,
            locale: locale_of(&req.locale),
            address,
            domain,
            account_id,
            owners,
            requested_by: format!("admin:{}", ctx.username),
        },
    )
    .await?;
    Ok((StatusCode::CREATED, no_store(Json(json!({"request": admin_view(&row, &body, &policy)})))).into_response())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Assign {
    assignee: Option<String>,
}

async fn admin_assign(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>, Json(req): Json<Assign>) -> Result<Response, AdminApiError> {
    let (mut row, body, policy) = scoped(&state, &ctx, &id).await?;
    row.assignee = req.assignee.map(|a| a.trim().chars().take(120).collect::<String>()).filter(|a| !a.is_empty());
    let status = row.status.clone();
    save(&state, &mut row, &body, &status).await?;
    event(&state, &id, &ctx.username, "assigned", json!({"assignee": row.assignee})).await;
    Ok(no_store(Json(json!({"request": admin_view(&row, &body, &policy)}))))
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Note {
    #[serde(default)]
    note: String,
}

async fn admin_verify(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>, Json(req): Json<Note>) -> Result<Response, AdminApiError> {
    let (mut row, mut body, policy) = scoped(&state, &ctx, &id).await?;
    if !matches!(row.status.as_str(), "awaiting_verification" | "awaiting_confirmation") {
        return Err(Fail::state("This request is not waiting for an identity check.").into());
    }
    if req.note.trim().is_empty() {
        return Err(AdminApiError::invalid("invalid", "A note on how the identity was verified is required.", "/note"));
    }
    body.verified_at = Some(now());
    body.verified_by = Some(ctx.username.clone());
    body.confirm_hash = None;
    event(&state, &id, &ctx.username, "identity-verified", json!({"note": req.note.chars().take(500).collect::<String>()})).await;
    advance(&state, &mut row, &mut body, &policy, &ctx.username).await?;
    Ok(no_store(Json(json!({"request": admin_view(&row, &body, &policy)}))))
}

async fn admin_approve(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>, Json(req): Json<Note>) -> Result<Response, AdminApiError> {
    let (mut row, mut body, policy) = scoped(&state, &ctx, &id).await?;
    if !matches!(row.status.as_str(), "awaiting_approval" | "blocked" | "awaiting_verification") {
        return Err(Fail::state("This request is not waiting for approval.").into());
    }
    if row.status == "awaiting_verification" && policy.b("dsr.identityVerification.required") {
        return Err(Fail::state("Record the identity verification first.").into());
    }
    if row.status == "blocked" && policy.hold(&body, &row.address).is_some() {
        return Err(Fail::state("A legal hold still applies; release it first.").into());
    }
    body.decision_reason = Some(req.note.chars().take(500).collect()).filter(|s: &String| !s.is_empty());
    approve(&state, &mut row, &mut body, &policy, &ctx.username).await?;
    Ok(no_store(Json(json!({"request": admin_view(&row, &body, &policy)}))))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reject {
    reason: String,
}

async fn admin_reject(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>, Json(req): Json<Reject>) -> Result<Response, AdminApiError> {
    let (mut row, mut body, policy) = scoped(&state, &ctx, &id).await?;
    if !is_open(&row.status) || row.status == "processing" {
        return Err(Fail::state("This request can no longer be rejected.").into());
    }
    let reason: String = req.reason.trim().chars().take(500).collect();
    if reason.is_empty() {
        return Err(AdminApiError::invalid("invalid", "A reason is required.", "/reason"));
    }
    body.decision_reason = Some(reason.clone());
    body.answered_at = Some(now());
    save(&state, &mut row, &body, "rejected").await?;
    event(&state, &id, &ctx.username, "rejected", json!({"reason": reason})).await;
    notice(&state, &row, &body, &policy, "Your data request was not carried out", &format!("Your {} request was rejected. Reason: {reason}", row.kind), false, None);
    Ok(no_store(Json(json!({"request": admin_view(&row, &body, &policy)}))))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Extend {
    days: i64,
    reason: String,
}

async fn admin_extend(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>, Json(req): Json<Extend>) -> Result<Response, AdminApiError> {
    let (mut row, mut body, policy) = scoped(&state, &ctx, &id).await?;
    if !is_open(&row.status) {
        return Err(Fail::state("Only an open request can be extended.").into());
    }
    let max = policy.n("dsr.sla.maxExtensionDays");
    let reason: String = req.reason.trim().chars().take(500).collect();
    if reason.is_empty() {
        return Err(AdminApiError::invalid("invalid", "A reason is required.", "/reason"));
    }
    if req.days < 1 || body.extension_days + req.days > max {
        return Err(AdminApiError::invalid("invalid", format!("The total extension cannot exceed {max} days."), "/days"));
    }
    body.original_due_at.get_or_insert(row.due_at);
    row.due_at += req.days * DAY;
    body.extension_days += req.days;
    body.extension_reason = Some(reason.clone());
    body.reminded = false;
    body.overdue_flagged = false;
    let status = row.status.clone();
    save(&state, &mut row, &body, &status).await?;
    event(&state, &id, &ctx.username, "extended", json!({"days": req.days, "reason": reason, "dueAt": row.due_at})).await;
    notice(&state, &row, &body, &policy, "Your data request needs more time", &format!("The answer to your {} request will take up to {} more day(s). Reason: {reason}", row.kind, req.days), false, None);
    Ok(no_store(Json(json!({"request": admin_view(&row, &body, &policy)}))))
}

async fn admin_complete(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>, Json(req): Json<Note>) -> Result<Response, AdminApiError> {
    let (mut row, mut body, policy) = scoped(&state, &ctx, &id).await?;
    if row.status != "in_progress" {
        return Err(Fail::state("Only a request in progress can be completed by hand.").into());
    }
    let note: String = req.note.trim().chars().take(500).collect();
    if note.is_empty() {
        return Err(AdminApiError::invalid("invalid", "A note on the outcome is required.", "/note"));
    }
    body.completion = Some(json!({"completedAt": now(), "outcome": note, "retained": [], "digest": null}));
    body.answered_at = Some(now());
    save(&state, &mut row, &body, "completed").await?;
    event(&state, &id, &ctx.username, "completed", json!({"outcome": note})).await;
    notice(&state, &row, &body, &policy, "Your data request is complete", &format!("Your {} request was completed. Outcome: {note}", row.kind), false, policy.text("dsr.text.completion", &body.locale));
    Ok(no_store(Json(json!({"request": admin_view(&row, &body, &policy)}))))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Hold {
    reason: String,
}

async fn admin_hold(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>, Json(req): Json<Hold>) -> Result<Response, AdminApiError> {
    let (mut row, mut body, policy) = scoped(&state, &ctx, &id).await?;
    if !is_open(&row.status) {
        return Err(Fail::state("Only an open request can be put under a legal hold.").into());
    }
    let reason: String = req.reason.trim().chars().take(500).collect();
    if reason.is_empty() {
        return Err(AdminApiError::invalid("invalid", "A reason is required.", "/reason"));
    }
    body.hold = Some(reason.clone());
    let status = if row.kind == "erasure" && matches!(row.status.as_str(), "scheduled" | "queued") { "blocked".to_string() } else { row.status.clone() };
    save(&state, &mut row, &body, &status).await?;
    event(&state, &id, &ctx.username, "hold-set", json!({"reason": reason})).await;
    Ok(no_store(Json(json!({"request": admin_view(&row, &body, &policy)}))))
}

async fn admin_release(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>) -> Result<Response, AdminApiError> {
    let (mut row, mut body, policy) = scoped(&state, &ctx, &id).await?;
    if body.hold.is_none() {
        return Err(Fail::state("This request has no hold of its own to release; a hold from the settings is removed there.").into());
    }
    body.hold = None;
    let status = row.status.clone();
    save(&state, &mut row, &body, &status).await?;
    event(&state, &id, &ctx.username, "hold-released", json!({})).await;
    if row.status == "blocked" {
        reevaluate_blocked(&state, &mut row, &mut body, &policy).await?;
    }
    Ok(no_store(Json(json!({"request": admin_view(&row, &body, &policy)}))))
}

/// Run the pending step of one request now (the worker does the same on its tick).
async fn admin_run(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>) -> Result<Response, AdminApiError> {
    let (mut row, mut body, policy) = scoped(&state, &ctx, &id).await?;
    if row.status == "failed" {
        body.attempts = 0;
        body.error = None;
        let next = if row.kind == "erasure" { "scheduled" } else { "queued" };
        save(&state, &mut row, &body, next).await?;
        event(&state, &id, &ctx.username, "retry", json!({})).await;
    }
    if row.status == "scheduled" && row.kind == "erasure" {
        body.grace_until = Some(now());
        let status = row.status.clone();
        save(&state, &mut row, &body, &status).await?;
        event(&state, &id, &ctx.username, "grace-skipped", json!({})).await;
    }
    work_one(&state, &id).await;
    let (row, body, policy2) = scoped(&state, &ctx, &id).await.unwrap_or((row, body, policy));
    Ok(no_store(Json(json!({"request": admin_view(&row, &body, &policy2)}))))
}

// ─── worker ───────────────────────────────────────────────────────────────────

/// Start the worker loop (from `build_app`).
pub(crate) fn start(state: AppState) {
    tokio::spawn(async move {
        loop {
            if let Err(e) = pass(&state).await {
                tracing::warn!("data request worker pass failed: {e}");
            }
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        }
    });
}

async fn pass(state: &AppState) -> Result<(), mw_store::StoreError> {
    let at = now();
    for row in state.store.dsr_list(None).await? {
        // Finished requests only matter again when their retention ends.
        if !is_open(&row.status) && !matches!(row.status.as_str(), "ready" | "downloaded") && row.updated_at + 30 * DAY > at {
            continue;
        }
        // A request another replica is changing is picked up on the next pass.
        work_one(state, &row.id).await;
    }
    Ok(())
}

async fn reevaluate_blocked(state: &AppState, row: &mut DsrRow, body: &mut Body, policy: &Policy) -> Result<(), Fail> {
    if row.kind == "erasure" && policy.hold(body, &row.address).is_none() {
        body.grace_until = Some(now() + policy.n("dsr.erasure.graceDays") * DAY);
        save(state, row, body, "scheduled").await?;
        event(state, &row.id, "system", "hold-cleared", json!({"graceUntil": body.grace_until})).await;
    }
    Ok(())
}

/// Do whatever is due for one request.
pub(crate) async fn work_one(state: &AppState, id: &str) {
    let Ok(Some(mut row)) = state.store.dsr_get(id).await else { return };
    let mut body = body_of(&row);
    let policy = Policy::load(state, &row.domain, &row.address, &row.account_id).await;
    let at = now();
    let outcome: Result<(), Fail> = async {
        match row.status.as_str() {
            "queued" => run_export(state, &mut row, &mut body, &policy).await?,
            "processing" if body.lease_until < at => {
                save(state, &mut row, &body, "queued").await?;
                event(state, id, "system", "lease-expired", json!({})).await;
                run_export(state, &mut row, &mut body, &policy).await?;
            }
            "scheduled" if body.grace_until.is_some_and(|g| g <= at) => run_erasure(state, &mut row, &mut body, &policy).await?,
            "blocked" => reevaluate_blocked(state, &mut row, &mut body, &policy).await?,
            "awaiting_confirmation" if row.created_at + CONFIRM_TTL_DAYS * DAY < at => {
                body.cancelled_by = Some("system".into());
                save(state, &mut row, &body, "cancelled").await?;
                event(state, id, "system", "confirmation-expired", json!({})).await;
            }
            "ready" | "downloaded" if body.export.as_ref().is_some_and(|e| e.expires_at <= at) => {
                let n = state.store.dsr_parts_delete(id).await?;
                save(state, &mut row, &body, "expired").await?;
                event(state, id, "system", "export-expired", json!({"partsDeleted": n})).await;
            }
            s if !is_open(s) && row.updated_at + policy.n("dsr.retention.completedDays") * DAY < at => {
                state.store.dsr_delete(id).await?;
                return Ok(());
            }
            s if is_open(s) => {
                match due_state(&row, &policy, at) {
                    "dueSoon" if !body.reminded => {
                        body.reminded = true;
                        let status = row.status.clone();
                        save(state, &mut row, &body, &status).await?;
                        event(state, id, "system", "due-soon", json!({"dueAt": row.due_at})).await;
                        admin_signal(state, &policy, "dsr-request-due-soon", &row).await;
                    }
                    "overdue" if !body.overdue_flagged => {
                        body.overdue_flagged = true;
                        let status = row.status.clone();
                        save(state, &mut row, &body, &status).await?;
                        event(state, id, "system", "overdue", json!({"dueAt": row.due_at})).await;
                        admin_signal(state, &policy, "dsr-request-overdue", &row).await;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }
    .await;
    if let Err(f) = outcome {
        if f.code != "changed" {
            tracing::warn!("data request {id}: {}", f.detail);
        }
    }
}

async fn run_export(state: &AppState, row: &mut DsrRow, body: &mut Body, policy: &Policy) -> Result<(), Fail> {
    body.lease_until = now() + 600;
    body.attempts += 1;
    save(state, row, body, "processing").await?;
    let subj = subj_of(state, row, body).await;
    let cap = (policy.n("dsr.export.maxMegabytes").max(1) as usize) * 1024 * 1024;
    let seal = policy.b("dsr.export.encrypt");
    let opts = Options {
        csv: policy.s("dsr.export.format") == "jsonCsv",
        mail: body.include_mail && policy.b("dsr.export.mailAllowed"),
        mail_eml: policy.s("dsr.export.mailFormat") == "eml",
        mail_cap_bytes: cap,
    };
    let requests: Vec<Value> = state
        .store
        .dsr_list(Some(&row.address))
        .await?
        .iter()
        .map(|r| user_view(r, &body_of(r), policy))
        .collect();
    let mut info = body.export.take().unwrap_or_default();
    info.sealed = seal;
    let mut failure: Option<String> = None;
    for name in SECTIONS {
        if info.done.iter().any(|d| d == name) {
            continue;
        }
        let enabled = match *name {
            "profile" | "requests" => true,
            "mail" => opts.mail,
            other => policy.b(&format!("dsr.export.include.{other}")),
        };
        if !enabled {
            info.sections.push(json!({"name": name, "included": false, "reason": "switchedOff"}));
            info.done.push((*name).into());
            continue;
        }
        match dsr_data::build_section(state, name, &subj, &opts, &requests).await {
            Ok(built) => {
                let size: usize = built.files.iter().map(|(_, d)| d.len()).sum();
                if info.bytes as usize + size > cap {
                    info.sections.push(json!({"name": name, "included": false, "reason": "sizeCap", "bytes": size}));
                    info.complete = false;
                } else {
                    let mut files = Vec::new();
                    for (fname, data) in &built.files {
                        state.store.dsr_part_put(&row.id, fname, data, seal).await?;
                        files.push(json!({"name": fname, "bytes": data.len()}));
                    }
                    info.bytes += size as u64;
                    info.sections.push(json!({"name": name, "included": true, "records": built.count, "files": files, "note": built.note}));
                }
                info.done.push((*name).into());
                body.export = Some(info.clone());
                body.lease_until = now() + 600;
                save(state, row, body, "processing").await?;
            }
            Err(e) => {
                failure = Some(e);
                break;
            }
        }
    }
    if let Some(e) = failure {
        body.export = Some(info);
        body.error = Some(e.clone());
        let status = if body.attempts >= 3 { "failed" } else { "queued" };
        save(state, row, body, status).await?;
        event(state, &row.id, "system", "export-error", json!({"error": e, "attempt": body.attempts})).await;
        return Ok(());
    }
    info.complete = !info.sections.iter().any(|s| s["reason"] == "sizeCap");
    let manifest = json!({
        "request": row.id, "generatedAt": now(), "subject": row.address, "format": policy.s("dsr.export.format"),
        "complete": info.complete, "sections": info.sections,
        "notes": ["Mail that stays on the upstream mail server is not part of this export.", "Sections left out for size are listed with reason sizeCap."],
    });
    let manifest = serde_json::to_vec_pretty(&manifest).unwrap_or_default();
    info.bytes += manifest.len() as u64;
    state.store.dsr_part_put(&row.id, "manifest.json", &manifest, seal).await?;
    info.expires_at = now() + policy.n("dataExport.linkHours") * 3600;
    body.export = Some(info);
    body.answered_at = Some(now());
    body.error = None;
    save(state, row, body, "ready").await?;
    event(state, &row.id, "system", "export-ready", json!({"bytes": body.export.as_ref().map(|e| e.bytes)})).await;
    notice(state, row, body, policy, "Your data export is ready", "Your data export is ready. Download it from Settings > Privacy & data. You will be asked for your password.", false, policy.text("dsr.text.completion", &body.locale));
    admin_signal(state, policy, "dsr-export-ready", row).await;
    Ok(())
}

async fn run_erasure(state: &AppState, row: &mut DsrRow, body: &mut Body, policy: &Policy) -> Result<(), Fail> {
    if let Some(h) = policy.hold(body, &row.address) {
        save(state, row, body, "blocked").await?;
        event(state, &row.id, "system", "blocked-by-hold", h).await;
        admin_signal(state, policy, "dsr-erasure-blocked", row).await;
        return Ok(());
    }
    body.lease_until = now() + 600;
    body.attempts += 1;
    save(state, row, body, "processing").await?;
    // Tell the person while their inbox still exists; the message is the last
    // thing put there.
    notice(state, row, body, policy, "Your account data is being erased", "Your erasure request is being carried out. Mail kept on your mail provider's server is not deleted by this server.", false, policy.text("dsr.text.completion", &body.locale));
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let subj = subj_of(state, row, body).await;
    let erased = dsr_data::erase(state, &subj).await;
    // Earlier exports of this person hold the same data; remove them too.
    let mut export_parts = 0u64;
    for other in state.store.dsr_list(Some(&row.address)).await? {
        export_parts += state.store.dsr_parts_delete(&other.id).await?;
    }
    if !erased.errors.is_empty() || erased.remaining > 0 {
        body.error = Some(erased.errors.join("; "));
        let status = if body.attempts >= 3 { "failed" } else { "scheduled" };
        body.grace_until = Some(now() + 60);
        save(state, row, body, status).await?;
        event(state, &row.id, "system", "erasure-error", json!({"errors": erased.errors, "remaining": erased.remaining})).await;
        return Ok(());
    }
    let completed_at = now();
    let retained = json!([
        {"item": "audit log entries", "reason": "kept for the audit retention period set by the administrator"},
        {"item": "directory entry (address, domain, status, timestamps)", "reason": "kept so the address cannot be re-enrolled without an administrator"},
        {"item": "this request and its trail", "reason": "kept for the request retention period as proof of handling"},
        {"item": "mail on the upstream mail server", "reason": "held by the mail provider; this server removed its link to it"},
    ]);
    let mut digest_input = serde_json::to_string(&erased.counts).unwrap_or_default();
    digest_input.push_str(&format!("|{}|{}|{}", row.id, crate::push_relay::hash_token(&row.address), completed_at));
    let digest = crate::push_relay::hash_token(&digest_input);
    body.completion = Some(json!({
        "completedAt": completed_at, "deleted": erased.counts, "exportPartsDeleted": export_parts,
        "retained": retained, "verified": true, "remainingAfterVerification": erased.remaining, "digest": digest,
    }));
    body.answered_at = Some(completed_at);
    body.error = None;
    save(state, row, body, "completed").await?;
    event(state, &row.id, "system", "erasure-completed", json!({"digest": digest})).await;
    admin_signal(state, policy, "dsr-erasure-completed", row).await;
    Ok(())
}
