//! The data protection impact assessment (DPIA) register.
//!
//! An assessment is a structured document: a description, necessity and
//! proportionality, risks scored on a likelihood by severity matrix, mitigations,
//! residual risk, the DPO's opinion, consultation notes and links. It moves
//! through draft, review, approved or needs-prior-consultation, and every save is
//! kept as a version. This is a record-keeping tool; the administrator decides
//! whether and when an assessment is needed.

use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson},
    admin_router::AdminRegistrar,
    capabilities::Capability as C,
    compliance_policy::html_escape,
    auth_logs::sweeper as retention_sweeper, settings_registry,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::{Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use mw_store::compliance::{Dpia, MAX_ROWS};
use serde::Deserialize;
use serde_json::{Value, json};

pub(crate) const STATUSES: [&str; 4] = ["draft", "review", "approved", "needs-prior-consultation"];
pub(crate) const TEMPLATES: [&str; 5] = ["generic", "assistant", "email-tracking", "large-scale-monitoring", "new-processor"];

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, "/compliance/dpia", C::SettingsRead, list);
    r.admin_endpoint(Method::POST, "/compliance/dpia", C::SettingsWrite, create);
    r.admin_endpoint(Method::GET, "/compliance/dpia/templates", C::SettingsRead, templates);
    r.admin_endpoint(Method::GET, "/compliance/dpia/suggestions", C::SettingsRead, suggestions);
    r.admin_endpoint(Method::GET, "/compliance/dpia/{id}", C::SettingsRead, get_one);
    r.admin_endpoint(Method::PUT, "/compliance/dpia/{id}", C::SettingsWrite, save);
    r.admin_endpoint(Method::DELETE, "/compliance/dpia/{id}", C::SettingsWrite, remove);
    r.admin_endpoint(Method::GET, "/compliance/dpia/{id}/export", C::SettingsRead, export);
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

/// Likelihood times severity, each 1 to 4.
pub(crate) fn score(likelihood: i64, severity: i64) -> i64 {
    likelihood * severity
}

/// `low` up to 3, `medium` up to 6, `high` up to 9, then `veryHigh`.
pub(crate) fn level(score: i64) -> &'static str {
    match score {
        ..=3 => "low",
        4..=6 => "medium",
        7..=9 => "high",
        _ => "veryHigh",
    }
}

/// Whether `from` may move to `to`.
pub(crate) fn transition_allowed(from: &str, to: &str) -> bool {
    from == to
        || matches!(
            (from, to),
            ("draft", "review")
                | ("review", "draft" | "approved" | "needs-prior-consultation")
                | ("approved", "review")
                | ("needs-prior-consultation", "review" | "approved")
        )
}

fn int(v: &Value, key: &str) -> Option<i64> {
    v.get(key).and_then(Value::as_i64)
}

/// Why `body` cannot be approved, if it cannot: a risk without residual scores,
/// or a residual score in the very high band (which belongs to prior consultation).
pub(crate) fn approval_blocker(body: &Value) -> Option<&'static str> {
    let risks = body.get("risks").and_then(Value::as_array).map_or(&[][..], Vec::as_slice);
    for r in risks {
        let (Some(l), Some(s)) = (int(r, "residualLikelihood"), int(r, "residualSeverity")) else {
            return Some("residualMissing");
        };
        if score(l, s) >= 10 {
            return Some("residualVeryHigh");
        }
    }
    None
}

const TEXT_FIELDS: [&str; 8] = ["description", "purposes", "necessity", "proportionality", "mitigations", "dpoOpinion", "consultation", "notes"];

/// Validate and normalise a body. Unknown members are dropped.
pub(crate) fn clean_body(v: &Value) -> Result<Value, AdminApiError> {
    let Some(obj) = v.as_object() else {
        return Err(AdminApiError::invalid("invalidBody", "Expected an object.", "/body"));
    };
    let mut out = serde_json::Map::new();
    for f in TEXT_FIELDS {
        let t = obj.get(f).and_then(Value::as_str).unwrap_or("");
        if t.chars().count() > 8000 || t.chars().any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t') {
            return Err(AdminApiError::invalid("invalidText", "Plain text up to 8000 characters.", format!("/body/{f}")));
        }
        out.insert(f.into(), json!(t));
    }
    for f in ["feature", "activityId"] {
        let t = obj.get(f).and_then(Value::as_str).unwrap_or("");
        if t.chars().count() > 100 || t.chars().any(char::is_control) {
            return Err(AdminApiError::invalid("invalidText", "Up to 100 characters.", format!("/body/{f}")));
        }
        out.insert(f.into(), json!(t));
    }
    let mut risks = Vec::new();
    for (i, r) in obj.get("risks").and_then(Value::as_array).map_or(&[][..], Vec::as_slice).iter().enumerate() {
        if i >= 50 {
            return Err(AdminApiError::invalid("tooManyRisks", "At most 50 risks.", "/body/risks"));
        }
        let mut risk = serde_json::Map::new();
        for f in ["title", "description", "mitigations"] {
            let t = r.get(f).and_then(Value::as_str).unwrap_or("");
            if t.chars().count() > 4000 || t.chars().any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t') {
                return Err(AdminApiError::invalid("invalidText", "Plain text up to 4000 characters.", format!("/body/risks/{i}/{f}")));
            }
            risk.insert(f.into(), json!(t));
        }
        for f in ["likelihood", "severity", "residualLikelihood", "residualSeverity"] {
            match r.get(f) {
                None | Some(Value::Null) if f.starts_with("residual") => {}
                Some(n) if n.as_i64().is_some_and(|n| (1..=4).contains(&n)) => {
                    risk.insert(f.into(), n.clone());
                }
                _ => return Err(AdminApiError::invalid("invalidScore", "A whole number from 1 to 4.", format!("/body/risks/{i}/{f}"))),
            }
        }
        risks.push(Value::Object(risk));
    }
    out.insert("risks".into(), Value::Array(risks));
    let mut links = Vec::new();
    for (i, l) in obj.get("links").and_then(Value::as_array).map_or(&[][..], Vec::as_slice).iter().enumerate() {
        if i >= 50 {
            return Err(AdminApiError::invalid("tooManyLinks", "At most 50 attachments.", "/body/links"));
        }
        let label = l.get("label").and_then(Value::as_str).unwrap_or("");
        let url = l.get("url").and_then(Value::as_str).unwrap_or("");
        let text = l.get("text").and_then(Value::as_str).unwrap_or("");
        if label.is_empty() || label.chars().count() > 200 || text.chars().count() > 8000 || url.chars().count() > 2000 {
            return Err(AdminApiError::invalid("invalidLink", "A label plus a link or text.", format!("/body/links/{i}")));
        }
        if !url.is_empty() && !url.starts_with("https://") && !url.starts_with("http://") {
            return Err(AdminApiError::invalid("invalidLink", "Links start with https:// or http://.", format!("/body/links/{i}/url")));
        }
        links.push(json!({"label": label, "url": url, "text": text}));
    }
    out.insert("links".into(), Value::Array(links));
    Ok(Value::Object(out))
}

/// The body with computed scores and levels added to each risk.
pub(crate) fn with_scores(body: &Value) -> Value {
    let mut b = body.clone();
    if let Some(risks) = b.get_mut("risks").and_then(Value::as_array_mut) {
        for r in risks {
            let inherent = int(r, "likelihood").zip(int(r, "severity")).map(|(l, s)| score(l, s));
            let residual = int(r, "residualLikelihood").zip(int(r, "residualSeverity")).map(|(l, s)| score(l, s));
            if let Some(o) = r.as_object_mut() {
                o.insert("score".into(), json!(inherent));
                o.insert("level".into(), json!(inherent.map(level)));
                o.insert("residualScore".into(), json!(residual));
                o.insert("residualLevel".into(), json!(residual.map(level)));
            }
        }
    }
    b
}

fn view(d: &Dpia, due_within: i64) -> Value {
    let body = serde_json::from_str::<Value>(&d.body_json).unwrap_or_else(|_| json!({}));
    let due = d.review_at.is_some_and(|r| r - due_within * 86_400 <= now()) && matches!(d.status.as_str(), "approved" | "needs-prior-consultation");
    json!({
        "id": d.id, "title": d.title, "status": d.status, "template": d.template, "version": d.version,
        "reviewAt": d.review_at, "due": due, "overdue": due && d.review_at.is_some_and(|r| r < now()),
        "createdBy": d.created_by, "createdAt": d.created_at, "updatedBy": d.updated_by, "updatedAt": d.updated_at,
        "body": with_scores(&body),
    })
}

async fn int_setting(state: &AppState, key: &str) -> i64 {
    retention_sweeper::num(&state.store, key).await
}

async fn bool_setting(state: &AppState, key: &str) -> bool {
    matches!(
        settings_registry::effective_for(&state.store, key, &Default::default()).await,
        Ok(Some(v)) if v.value.as_bool() == Some(true)
    )
}

// ---------------------------------------------------------------------------
// Templates
// ---------------------------------------------------------------------------

fn risk(title: &str, description: &str, l: i64, s: i64, mitigations: &str) -> Value {
    json!({"title": title, "description": description, "likelihood": l, "severity": s, "mitigations": mitigations})
}

/// The starting body of `template` in `lang`.
pub(crate) fn template_body(template: &str, lang: &str) -> Value {
    let pt = lang == "pt-PT";
    let p = |en: &'static str, ptx: &'static str| if pt { ptx } else { en };
    let (feature, description, purposes, necessity, proportionality, risks): (&str, &str, &str, &str, &str, Vec<Value>) = match template {
        "assistant" => (
            "assistant",
            p("The AI assistant drafts and summarises text that a user sends to it. Describe the provider, what text is sent, and whether it leaves your infrastructure.", "O assistente de IA redige e resume texto que o utilizador lhe envia. Descreva o fornecedor, que texto é enviado e se sai da sua infraestrutura."),
            p("Help users write and summarise mail at their request.", "Ajudar os utilizadores a escrever e resumir correio a seu pedido."),
            p("Explain why the assistant is needed and whether a local or opt-in alternative exists.", "Explique porque é necessário o assistente e se existe uma alternativa local ou por adesão."),
            p("Describe limits: only on request, only the text the user selects, retention of request records.", "Descreva os limites: só a pedido, só o texto que o utilizador seleciona, retenção dos registos de pedidos."),
            vec![
                risk(p("Message text reaches the assistant provider", "O texto das mensagens chega ao fornecedor do assistente"), p("Content of mail, including third parties' data, is sent to the provider.", "O conteúdo do correio, incluindo dados de terceiros, é enviado ao fornecedor."), 3, 3, p("Use only on request; restrict by profile; review provider terms.", "Usar apenas a pedido; restringir por perfil; rever os termos do fornecedor.")),
                risk(p("Inaccurate output is acted on", "Resultado impreciso é seguido"), p("A user relies on a wrong summary or draft.", "Um utilizador confia num resumo ou rascunho errado."), 2, 2, p("Output is a suggestion the user reviews before use.", "O resultado é uma sugestão que o utilizador revê antes de usar.")),
            ],
        ),
        "email-tracking" => (
            "openTracking",
            p("Senders may track whether a recipient opened a message. Describe who can use it, what is recorded and for how long.", "Os remetentes podem rastrear se um destinatário abriu uma mensagem. Descreva quem a pode usar, o que é registado e por quanto tempo."),
            p("Tell a sender whether a message they chose to track was opened.", "Indicar ao remetente se uma mensagem que escolheu rastrear foi aberta."),
            p("Explain why the sender needs this and why it cannot be achieved without tracking.", "Explique porque o remetente precisa disto e porque não se consegue sem rastreio."),
            p("Describe limits: per-message opt-in, retention days, whether recipients can refuse.", "Descreva os limites: ativação por mensagem, dias de retenção, se os destinatários podem recusar."),
            vec![
                risk(p("Recipients are tracked without knowing", "Os destinatários são rastreados sem saber"), p("Opens reveal time and approximate location of a recipient.", "As aberturas revelam a hora e a localização aproximada do destinatário."), 3, 2, p("Per-message opt-in; short retention; honour the recipient's refusal of remote content.", "Ativação por mensagem; retenção curta; respeitar a recusa de conteúdo remoto pelo destinatário.")),
            ],
        ),
        "large-scale-monitoring" => (
            "monitoring",
            p("Systematic monitoring of people at scale, for example security logging that covers all users. Describe what is observed and how.", "Monitorização sistemática de pessoas em grande escala, por exemplo registos de segurança que abrangem todos os utilizadores. Descreva o que é observado e como."),
            p("State the purpose of the monitoring.", "Indique a finalidade da monitorização."),
            p("Explain why less intrusive means are not enough.", "Explique porque meios menos intrusivos não bastam."),
            p("Describe limits: which data, who can view it, retention.", "Descreva os limites: que dados, quem os pode ver, retenção."),
            vec![
                risk(p("Monitoring data is used for another purpose", "Os dados de monitorização são usados para outro fim"), p("Logs collected for security are used to evaluate people.", "Registos recolhidos por segurança são usados para avaliar pessoas."), 2, 3, p("Role-based access to logs; audit of access; retention limits.", "Acesso por perfil aos registos; auditoria do acesso; limites de retenção.")),
            ],
        ),
        "new-processor" => (
            "processor",
            p("A new processor or sub-processor will handle personal data. Name it, what it receives and where it is located.", "Um novo subcontratante vai tratar dados pessoais. Nomeie-o, o que recebe e onde está localizado."),
            p("State what the processor does for you.", "Indique o que o subcontratante faz por si."),
            p("Explain why this processor is needed.", "Explique porque é necessário este subcontratante."),
            p("Describe the data shared and the contract terms in place.", "Descreva os dados partilhados e as condições contratuais em vigor."),
            vec![
                risk(p("Data is handled outside agreed terms", "Os dados são tratados fora das condições acordadas"), p("The processor keeps or reuses data beyond the contract.", "O subcontratante conserva ou reutiliza dados para além do contrato."), 2, 3, p("Written agreement; deletion on exit; list the processor in the records.", "Acordo escrito; eliminação na cessação; incluir o subcontratante nos registos.")),
                risk(p("Transfer outside the hosting country", "Transferência para fora do país de alojamento"), p("Data is stored or accessed from another country.", "Os dados são guardados ou acedidos a partir de outro país."), 2, 3, p("Record the transfer and its safeguards in the policy.", "Registar a transferência e as suas garantias na política.")),
            ],
        ),
        _ => (
            "",
            p("Describe the processing: what data, whose, by what means.", "Descreva o tratamento: que dados, de quem, por que meios."),
            p("State the purposes.", "Indique as finalidades."),
            p("Explain why the processing is necessary for those purposes.", "Explique porque o tratamento é necessário para essas finalidades."),
            p("Describe how the processing is limited to what is needed.", "Descreva como o tratamento se limita ao necessário."),
            vec![risk(p("Describe a risk to people", "Descreva um risco para as pessoas"), "", 2, 2, "")],
        ),
    };
    json!({
        "feature": feature, "activityId": "", "description": description, "purposes": purposes,
        "necessity": necessity, "proportionality": proportionality, "risks": risks,
        "mitigations": "", "dpoOpinion": "", "consultation": "", "notes": "", "links": [],
    })
}

#[derive(Deserialize)]
struct LangQuery {
    lang: Option<String>,
}

async fn templates(State(_): State<AppState>, ctx: AdminCtx, Query(q): Query<LangQuery>) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsRead)?;
    let lang = if q.lang.as_deref() == Some("pt-PT") { "pt-PT" } else { "en" };
    Ok(Json(json!({
        "templates": TEMPLATES.iter().map(|t| json!({"id": t, "body": template_body(t, lang)})).collect::<Vec<_>>(),
        "statuses": STATUSES,
        "matrix": {"scale": [1, 2, 3, 4], "bands": [{"upTo": 3, "level": "low"}, {"upTo": 6, "level": "medium"}, {"upTo": 9, "level": "high"}, {"upTo": 16, "level": "veryHigh"}]},
    }))
    .into_response())
}

// ---------------------------------------------------------------------------
// Register
// ---------------------------------------------------------------------------

async fn list(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsRead)?;
    let window = int_setting(&state, "dpia.reminderDays").await;
    let rows = state.store.list_dpias().await.map_err(AdminApiError::internal)?;
    Ok(Json(json!({"items": rows.iter().map(|d| view(d, window)).collect::<Vec<_>>()})).into_response())
}

async fn get_one(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsRead)?;
    let Some(d) = state.store.get_dpia(&id).await.map_err(AdminApiError::internal)? else {
        return Err(AdminApiError::not_found("No such assessment."));
    };
    let window = int_setting(&state, "dpia.reminderDays").await;
    let versions = state.store.list_dpia_versions(&id).await.map_err(AdminApiError::internal)?;
    let mut v = view(&d, window);
    v["history"] = Value::Array(
        versions
            .iter()
            .map(|x| json!({"version": x.version, "status": x.status, "changedBy": x.changed_by, "at": x.created_at, "body": serde_json::from_str::<Value>(&x.body_json).unwrap_or(Value::Null)}))
            .collect(),
    );
    Ok(Json(v).into_response())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CreateBody {
    title: String,
    #[serde(default = "generic")]
    template: String,
    #[serde(default)]
    lang: Option<String>,
}
fn generic() -> String {
    "generic".into()
}

fn clean_title(t: &str) -> Result<String, AdminApiError> {
    let t = t.trim();
    if t.is_empty() || t.chars().count() > 200 || t.chars().any(char::is_control) {
        return Err(AdminApiError::invalid("invalidTitle", "A title of 1 to 200 characters.", "/title"));
    }
    Ok(t.to_owned())
}

async fn create(State(state): State<AppState>, ctx: AdminCtx, AdminJson(b): AdminJson<CreateBody>) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsWrite)?;
    if !TEMPLATES.contains(&b.template.as_str()) {
        return Err(AdminApiError::invalid("invalidTemplate", "Unknown template.", "/template"));
    }
    if state.store.list_dpias().await.map_err(AdminApiError::internal)?.len() >= MAX_ROWS {
        return Err(AdminApiError::new(StatusCode::CONFLICT, "tooManyAssessments", "Remove an assessment before adding another."));
    }
    let lang = if b.lang.as_deref() == Some("pt-PT") { "pt-PT" } else { "en" };
    let at = now();
    let d = Dpia {
        id: uuid::Uuid::new_v4().to_string(),
        title: clean_title(&b.title)?,
        status: "draft".into(),
        template: b.template.clone(),
        body_json: template_body(&b.template, lang).to_string(),
        version: 1,
        review_at: None,
        created_by: ctx.username.clone(),
        created_at: at,
        updated_by: ctx.username.clone(),
        updated_at: at,
    };
    state.store.save_dpia(&d).await.map_err(AdminApiError::internal)?;
    crate::admin_accounts::audit(&state, &ctx, "dpia-created", &format!("dpia:{}", d.id), None, Some(json!({"title": d.title, "template": d.template}))).await;
    let window = int_setting(&state, "dpia.reminderDays").await;
    Ok((StatusCode::CREATED, Json(view(&d, window))).into_response())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SaveBody {
    /// The version the editor loaded; a different stored version is a conflict.
    version: i64,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    body: Option<Value>,
    #[serde(default)]
    review_at: Option<i64>,
}

async fn save(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>, AdminJson(b): AdminJson<SaveBody>) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsWrite)?;
    let Some(mut d) = state.store.get_dpia(&id).await.map_err(AdminApiError::internal)? else {
        return Err(AdminApiError::not_found("No such assessment."));
    };
    if d.version != b.version {
        return Err(AdminApiError::new(StatusCode::CONFLICT, "stale", "The assessment changed since you opened it."));
    }
    if let Some(t) = &b.title {
        d.title = clean_title(t)?;
    }
    if let Some(body) = &b.body {
        d.body_json = clean_body(body)?.to_string();
    }
    let body: Value = serde_json::from_str(&d.body_json).unwrap_or_else(|_| json!({}));
    if let Some(to) = &b.status {
        if !STATUSES.contains(&to.as_str()) {
            return Err(AdminApiError::invalid("invalidStatus", "Unknown status.", "/status"));
        }
        if !transition_allowed(&d.status, to) {
            return Err(AdminApiError::invalid("invalidTransition", format!("An assessment cannot move from {} to {to}.", d.status), "/status"));
        }
        if to == "approved" && d.status != "approved" {
            if let Some(blocker) = approval_blocker(&body) {
                let detail = match blocker {
                    "residualMissing" => "Record the residual likelihood and severity of every risk before approving.",
                    _ => "A residual risk in the very high band needs prior consultation, not approval.",
                };
                return Err(AdminApiError::invalid(blocker, detail, "/status"));
            }
            let months = int_setting(&state, "dpia.reviewMonths").await.max(1);
            d.review_at = b.review_at.or(Some(now() + months * 30 * 86_400));
        } else if let Some(r) = b.review_at {
            d.review_at = Some(r);
        }
        d.status = to.clone();
    } else if let Some(r) = b.review_at {
        d.review_at = Some(r);
    }
    d.version += 1;
    d.updated_by = ctx.username.clone();
    d.updated_at = now();
    state.store.save_dpia(&d).await.map_err(AdminApiError::internal)?;
    crate::admin_accounts::audit(&state, &ctx, "dpia-saved", &format!("dpia:{}", d.id), None, Some(json!({"status": d.status, "version": d.version}))).await;
    let window = int_setting(&state, "dpia.reminderDays").await;
    Ok(Json(view(&d, window)).into_response())
}

async fn remove(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsWrite)?;
    if !state.store.delete_dpia(&id).await.map_err(AdminApiError::internal)? {
        return Err(AdminApiError::not_found("No such assessment."));
    }
    crate::admin_accounts::audit(&state, &ctx, "dpia-deleted", &format!("dpia:{id}"), None, None).await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

// ---------------------------------------------------------------------------
// Suggestions, summary and export
// ---------------------------------------------------------------------------

/// Features whose settings suggest an assessment: `(feature, template, setting)`.
const RISKY: [(&str, &str, &str); 3] = [
    ("assistant", "assistant", "assistant.enabled"),
    ("openTracking", "email-tracking", "mail.openTracking"),
    ("analytics", "large-scale-monitoring", ""),
];

async fn suggested(state: &AppState) -> Result<Vec<Value>, AdminApiError> {
    if !bool_setting(state, "dpia.suggest").await {
        return Ok(Vec::new());
    }
    let existing = state.store.list_dpias().await.map_err(AdminApiError::internal)?;
    let covered: Vec<String> = existing
        .iter()
        .filter_map(|d| serde_json::from_str::<Value>(&d.body_json).ok())
        .filter_map(|b| b.get("feature").and_then(Value::as_str).map(str::to_owned))
        .collect();
    let mut out = Vec::new();
    for (feature, template, key) in RISKY {
        let on = if key.is_empty() {
            crate::analytics_config::load(&state.store).await.map(|c| c.enabled).unwrap_or(false)
        } else {
            bool_setting(state, key).await
        };
        if on && !covered.iter().any(|c| c == feature) {
            out.push(json!({"feature": feature, "template": template, "setting": if key.is_empty() { "analytics" } else { key }}));
        }
    }
    Ok(out)
}

async fn suggestions(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsRead)?;
    Ok(Json(json!({"items": suggested(&state).await?})).into_response())
}

pub(crate) async fn summary(state: &AppState) -> Result<Value, AdminApiError> {
    let window = int_setting(state, "dpia.reminderDays").await;
    let rows = state.store.list_dpias().await.map_err(AdminApiError::internal)?;
    let mut by = serde_json::Map::new();
    for s in STATUSES {
        by.insert(s.into(), json!(rows.iter().filter(|d| d.status == s).count()));
    }
    let due = rows.iter().filter(|d| view(d, window)["due"] == json!(true)).count();
    Ok(json!({"total": rows.len(), "byStatus": by, "due": due, "suggestions": suggested(state).await?.len()}))
}

#[derive(Deserialize)]
struct ExportQuery {
    format: Option<String>,
}

async fn export(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>, Query(q): Query<ExportQuery>) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsRead)?;
    let Some(d) = state.store.get_dpia(&id).await.map_err(AdminApiError::internal)? else {
        return Err(AdminApiError::not_found("No such assessment."));
    };
    let v = view(&d, 0);
    let (ct, ext, body) = if q.format.as_deref() == Some("json") {
        ("application/json", "json", serde_json::to_string_pretty(&v).unwrap_or_default())
    } else {
        ("text/html; charset=utf-8", "html", html(&v))
    };
    Ok((
        [
            (header::CONTENT_TYPE, ct.to_owned()),
            (header::CONTENT_DISPOSITION, format!("attachment; filename=\"dpia-{}.{ext}\"", &id[..id.len().min(8)])),
            (header::CACHE_CONTROL, "no-store".to_owned()),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_owned()),
        ],
        body,
    )
        .into_response())
}

fn para(label: &str, text: &str) -> String {
    format!("<h2>{}</h2><p>{}</p>", html_escape(label), html_escape(text).replace('\n', "<br>"))
}

pub(crate) fn html(v: &Value) -> String {
    let s = |k: &str| v["body"][k].as_str().unwrap_or("").to_owned();
    let mut out = format!(
        "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><title>{t}</title><style>body{{font:14px/1.45 system-ui,sans-serif;max-width:50rem;margin:1rem auto;padding:0 1rem}}table{{border-collapse:collapse;width:100%}}td,th{{border:1px solid #888;padding:4px 6px;text-align:start;vertical-align:top}}</style></head><body><h1>{t}</h1><p>Status: {st} · Version {ver}</p>",
        t = html_escape(v["title"].as_str().unwrap_or("")),
        st = html_escape(v["status"].as_str().unwrap_or("")),
        ver = v["version"],
    );
    for (label, key) in [("Description", "description"), ("Purposes", "purposes"), ("Necessity", "necessity"), ("Proportionality", "proportionality")] {
        out.push_str(&para(label, &s(key)));
    }
    out.push_str("<h2>Risks</h2><table><thead><tr><th>Risk</th><th>Likelihood</th><th>Severity</th><th>Score</th><th>Mitigations</th><th>Residual</th></tr></thead><tbody>");
    for r in v["body"]["risks"].as_array().map_or(&[][..], Vec::as_slice) {
        out.push_str(&format!(
            "<tr><td>{}<br>{}</td><td>{}</td><td>{}</td><td>{} {}</td><td>{}</td><td>{} {}</td></tr>",
            html_escape(r["title"].as_str().unwrap_or("")),
            html_escape(r["description"].as_str().unwrap_or("")),
            r["likelihood"], r["severity"], r["score"], html_escape(r["level"].as_str().unwrap_or("")),
            html_escape(r["mitigations"].as_str().unwrap_or("")),
            r["residualScore"], html_escape(r["residualLevel"].as_str().unwrap_or("")),
        ));
    }
    out.push_str("</tbody></table>");
    for (label, key) in [("Mitigations", "mitigations"), ("DPO opinion", "dpoOpinion"), ("Consultation", "consultation"), ("Notes", "notes")] {
        out.push_str(&para(label, &s(key)));
    }
    if let Some(rev) = v["reviewAt"].as_i64() {
        out.push_str(&para("Review date", &crate::compliance::date_of(rev)));
    }
    let links = v["body"]["links"].as_array().map_or(&[][..], Vec::as_slice);
    if !links.is_empty() {
        out.push_str("<h2>Attachments</h2><ul>");
        for l in links {
            out.push_str(&format!("<li>{} {} {}</li>", html_escape(l["label"].as_str().unwrap_or("")), html_escape(l["url"].as_str().unwrap_or("")), html_escape(l["text"].as_str().unwrap_or(""))));
        }
        out.push_str("</ul>");
    }
    out.push_str("</body></html>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scores_map_to_bands() {
        assert_eq!(level(score(1, 3)), "low");
        assert_eq!(level(score(2, 3)), "medium");
        assert_eq!(level(score(3, 3)), "high");
        assert_eq!(level(score(3, 4)), "veryHigh");
        assert_eq!(level(score(4, 4)), "veryHigh");
    }

    #[test]
    fn the_workflow_allows_only_listed_moves() {
        assert!(transition_allowed("draft", "review"));
        assert!(!transition_allowed("draft", "approved"));
        assert!(transition_allowed("review", "needs-prior-consultation"));
        assert!(transition_allowed("approved", "review"));
        assert!(!transition_allowed("approved", "draft"));
        assert!(transition_allowed("needs-prior-consultation", "approved"));
    }

    #[test]
    fn approval_needs_residual_scores_below_very_high() {
        let none = json!({"risks": [{"likelihood": 2, "severity": 2}]});
        assert_eq!(approval_blocker(&none), Some("residualMissing"));
        let high = json!({"risks": [{"residualLikelihood": 4, "residualSeverity": 3}]});
        assert_eq!(approval_blocker(&high), Some("residualVeryHigh"));
        let ok = json!({"risks": [{"residualLikelihood": 2, "residualSeverity": 2}]});
        assert_eq!(approval_blocker(&ok), None);
        assert_eq!(approval_blocker(&json!({"risks": []})), None);
    }

    #[test]
    fn bodies_are_validated_and_scored() {
        assert!(clean_body(&json!({"risks": [{"likelihood": 5, "severity": 1}]})).is_err());
        assert!(clean_body(&json!({"links": [{"label": "x", "url": "ftp://a"}]})).is_err());
        assert!(clean_body(&json!([])).is_err());
        let b = clean_body(&json!({"description": "d", "unknown": 1, "risks": [{"title": "t", "likelihood": 3, "severity": 3, "residualLikelihood": 1, "residualSeverity": 2}]})).unwrap();
        assert!(b.get("unknown").is_none());
        let s = with_scores(&b);
        assert_eq!(s["risks"][0]["score"], 9);
        assert_eq!(s["risks"][0]["level"], "high");
        assert_eq!(s["risks"][0]["residualLevel"], "low");
    }

    #[test]
    fn every_template_exists_in_both_languages() {
        for t in TEMPLATES {
            let en = template_body(t, "en");
            let pt = template_body(t, "pt-PT");
            assert!(!en["description"].as_str().unwrap().is_empty());
            assert_ne!(en["description"], pt["description"], "{t}");
            assert!(clean_body(&en).is_ok() && clean_body(&pt).is_ok());
        }
    }

    #[test]
    fn export_escapes_text() {
        let v = json!({"title": "<b>x</b>", "status": "draft", "version": 1, "body": {"description": "<script>", "risks": [], "links": []}});
        let h = html(&v);
        assert!(h.contains("&lt;script&gt;") && !h.contains("<script>"));
    }
}
