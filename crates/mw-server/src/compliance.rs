//! Compliance routes: the public privacy and cookie policy pages, the admin
//! policy editor with versions, the records of processing activities and the
//! sub-processor list, the per-user "what we process" view, and the DPIA register.
//!
//! Everything here describes configuration an administrator chose. None of it
//! states that a deployment meets any law.

use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx, AdminJson},
    admin_router::AdminRegistrar,
    capabilities::Capability as C,
    compliance_policy as policy, compliance_dpia as dpia, auth_logs::sweeper as retention_sweeper, settings_registry,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use mw_store::compliance::{ComplianceRecord, MAX_ROWS, PolicyOverride, PolicyVersion};
use serde::Deserialize;
use serde_json::{Value, json};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/privacy", get(privacy_page))
        .route("/cookies-policy", get(cookies_page))
        .route("/api/privacy/processing", get(my_processing))
}

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, "/compliance/summary", C::SettingsRead, summary);
    r.admin_endpoint(Method::GET, "/compliance/policy", C::SettingsRead, policy_view);
    r.admin_endpoint(Method::PUT, "/compliance/policy/overrides", C::SettingsWrite, put_override);
    r.admin_endpoint(Method::POST, "/compliance/policy/publish", C::SettingsWrite, publish);
    r.admin_endpoint(Method::GET, "/compliance/records", C::SettingsRead, records_view);
    r.admin_endpoint(Method::POST, "/compliance/records", C::SettingsWrite, add_record);
    r.admin_endpoint(Method::PUT, "/compliance/records/{id}", C::SettingsWrite, update_record);
    r.admin_endpoint(Method::DELETE, "/compliance/records/{id}", C::SettingsWrite, delete_record);
    r.admin_endpoint(Method::GET, "/compliance/records/export", C::SettingsRead, export_records);
    dpia::register(r);
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

pub(crate) fn date_of(unix: i64) -> String {
    chrono::DateTime::from_timestamp(unix, 0).map_or_else(String::new, |d| d.format("%Y-%m-%d").to_string())
}

fn lang_of(requested: Option<&str>, headers: &HeaderMap) -> &'static str {
    let wanted = requested.map(str::to_owned).or_else(|| {
        headers
            .get(header::ACCEPT_LANGUAGE)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
    });
    match wanted {
        Some(w) if w.to_ascii_lowercase().starts_with("pt") => "pt-PT",
        _ => "en",
    }
}

fn no_store(mut r: Response) -> Response {
    r.headers_mut().insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    r
}

/// `domain` of an admin query: empty means the deployment default.
fn clean_domain(d: Option<&str>) -> Result<String, AdminApiError> {
    let d = d.unwrap_or("").trim().to_ascii_lowercase();
    if d.len() > 253 || d.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err(AdminApiError::invalid("invalidDomain", "Not a domain name.", "/domain"));
    }
    Ok(d)
}

pub(crate) async fn flag(state: &AppState, domain: &str, key: &str) -> bool {
    let subject = mw_store::settings_policy::SettingSubject { personal_owner: None, domain, address: "", account_id: "" };
    matches!(
        settings_registry::effective_for(&state.store, key, &subject).await,
        Ok(Some(v)) if v.value.as_bool() == Some(true)
    )
}

// ---------------------------------------------------------------------------
// Versions
// ---------------------------------------------------------------------------

pub(crate) struct Current {
    pub snapshot: policy::Snapshot,
    pub latest: Option<PolicyVersion>,
    pub pending: Vec<Value>,
}

/// Compare the live snapshot with the newest version. With `record` set (an
/// explicit publish) or `policy.autoVersion` on, a difference becomes a new
/// version. The first version is always recorded.
pub(crate) async fn reconcile(state: &AppState, domain: &str, by: &str, publish: bool) -> Result<Current, AdminApiError> {
    let snapshot = policy::gather(state, domain).await.map_err(AdminApiError::internal)?;
    let latest = state.store.latest_policy_version(domain).await.map_err(AdminApiError::internal)?;
    let old = latest.as_ref().map(|v| policy::from_json(&v.facts_json)).unwrap_or_default();
    let pending = if latest.is_some() { policy::diff(&old, &snapshot) } else { Vec::new() };
    let auto = flag(state, domain, "policy.autoVersion").await;
    let record = latest.is_none() || (!pending.is_empty() && (publish || auto));
    if !record {
        return Ok(Current { snapshot, latest, pending });
    }
    let next = latest.as_ref().map_or(1, |v| v.version + 1);
    let row = PolicyVersion {
        id: uuid::Uuid::new_v4().to_string(),
        domain: domain.to_owned(),
        version: next,
        kind: if publish { "published" } else { "auto" }.into(),
        facts_json: policy::to_json(&snapshot),
        changes_json: Value::Array(pending.clone()).to_string(),
        created_by: by.to_owned(),
        created_at: now(),
    };
    match state.store.add_policy_version(&row).await {
        Ok(()) => {
            if latest.is_some() && flag(state, domain, "policy.reconsentOnChange").await {
                bump_cookie_version(state).await;
            }
            Ok(Current { snapshot, latest: Some(row), pending: Vec::new() })
        }
        // A concurrent request recorded the same version; use theirs.
        Err(_) => {
            let latest = state.store.latest_policy_version(domain).await.map_err(AdminApiError::internal)?;
            Ok(Current { snapshot, latest, pending })
        }
    }
}

/// Raise `cookies.policyVersion` by one so visitors are asked again.
async fn bump_cookie_version(state: &AppState) {
    use mw_store::settings_policy::SettingPolicyRow;
    let key = "cookies.policyVersion";
    let current = retention_sweeper::num(&state.store, key).await.max(1);
    let prior = state.store.get_setting_policy(key).await.ok().flatten();
    let row = SettingPolicyRow {
        key: key.into(),
        global_value: Some((current + 1).to_string()),
        allow_user_override: prior.as_ref().is_some_and(|p| p.allow_user_override),
        global_locked: prior.as_ref().is_some_and(|p| p.global_locked),
        updated_by: "privacy-policy".into(),
        updated_at: now(),
    };
    let _ = state.store.put_setting_policy(&row).await;
}

// ---------------------------------------------------------------------------
// Public pages
// ---------------------------------------------------------------------------

#[derive(Deserialize, Default)]
struct PageQuery {
    lang: Option<String>,
    format: Option<String>,
}

async fn privacy_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: axum::http::Extensions,
    Query(q): Query<PageQuery>,
) -> Response {
    public_page(&state, &headers, &ext, q, "privacy").await
}

async fn cookies_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    ext: axum::http::Extensions,
    Query(q): Query<PageQuery>,
) -> Response {
    public_page(&state, &headers, &ext, q, "cookies").await
}

async fn public_page(state: &AppState, headers: &HeaderMap, ext: &axum::http::Extensions, q: PageQuery, document: &str) -> Response {
    let domain = crate::cookie_consent::request_domain(state, headers, ext).await.unwrap_or_default();
    if !flag(state, &domain, "policy.publicPages").await {
        return StatusCode::NOT_FOUND.into_response();
    }
    let Ok(cur) = reconcile(state, &domain, "system", false).await else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    let Some(version) = cur.latest else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let lang = lang_of(q.lang.as_deref(), headers);
    let snapshot = policy::from_json(&version.facts_json);
    let sections = policy::render(&snapshot, lang, document);
    let heading = match (document, lang) {
        ("cookies", "pt-PT") => "Política de cookies",
        ("cookies", _) => "Cookie policy",
        (_, "pt-PT") => "Política de privacidade",
        _ => "Privacy policy",
    };
    let date = date_of(version.created_at);
    let (content_type, body) = match q.format.as_deref() {
        Some("md") => ("text/markdown; charset=utf-8", policy::to_markdown(&sections, heading, version.version, &date, lang)),
        Some("json") => (
            "application/json",
            json!({"version": version.version, "date": date, "lang": lang, "sections": policy::section_json(&sections)}).to_string(),
        ),
        _ => ("text/html; charset=utf-8", page_html(&sections, heading, version.version, &date, lang)),
    };
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type),
            (header::CACHE_CONTROL, "no-cache"),
            (header::CONTENT_SECURITY_POLICY, "default-src 'none'; style-src 'unsafe-inline'; base-uri 'none'; form-action 'none'"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        body,
    )
        .into_response()
}

const PAGE_CSS: &str = "body{font:16px/1.55 system-ui,sans-serif;margin:0;background:#fff;color:#111}main{max-width:44rem;margin:0 auto;padding:1.5rem 1rem}h1{font-size:1.6rem}h2{font-size:1.15rem;margin-top:2rem}code{font-size:.9em}@media(prefers-color-scheme:dark){body{background:#111;color:#eee}}";

fn page_html(sections: &[policy::Section], heading: &str, version: i64, date: &str, lang: &str) -> String {
    policy::to_html(sections, heading, version, date, lang).replacen("</head>", &format!("<style>{PAGE_CSS}</style></head>"), 1)
}

// ---------------------------------------------------------------------------
// Admin: policy
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PolicyQuery {
    domain: Option<String>,
    lang: Option<String>,
    document: Option<String>,
}

async fn policy_view(State(state): State<AppState>, ctx: AdminCtx, Query(q): Query<PolicyQuery>) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsRead)?;
    let domain = clean_domain(q.domain.as_deref())?;
    let cur = reconcile(&state, &domain, &ctx.username, false).await?;
    let lang = if q.lang.as_deref() == Some("pt-PT") { "pt-PT" } else { "en" };
    let document = if q.document.as_deref() == Some("cookies") { "cookies" } else { "privacy" };
    let sections = policy::render(&cur.snapshot, lang, document);
    let versions = state.store.list_policy_versions(&domain, 50).await.map_err(AdminApiError::internal)?;
    let overrides = state.store.list_policy_overrides(&domain).await.map_err(AdminApiError::internal)?;
    let published_sections = cur.latest.as_ref().map(|v| policy::render(&policy::from_json(&v.facts_json), lang, document));
    Ok(no_store(
        Json(json!({
            "domain": domain,
            "lang": lang,
            "document": document,
            "langs": policy::LANGS,
            "sections": policy::section_json(&sections),
            "published": published_sections.map(|s| policy::section_json(&s)),
            "warnings": policy::warnings(&cur.snapshot),
            "version": cur.latest.as_ref().map(|v| v.version),
            "publishedAt": cur.latest.as_ref().map(|v| v.created_at),
            "pending": cur.pending,
            "autoVersion": flag(&state, &domain, "policy.autoVersion").await,
            "versions": versions.iter().map(|v| json!({
                "version": v.version, "kind": v.kind, "createdBy": v.created_by, "createdAt": v.created_at,
                "changes": serde_json::from_str::<Value>(&v.changes_json).unwrap_or(Value::Null),
            })).collect::<Vec<_>>(),
            "overrides": overrides.iter().map(|o| json!({"lang": o.lang, "section": o.section, "body": o.body})).collect::<Vec<_>>(),
        }))
        .into_response(),
    ))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct OverrideBody {
    #[serde(default)]
    domain: Option<String>,
    lang: String,
    section: String,
    /// `null` or empty removes the override.
    body: Option<String>,
}

async fn put_override(State(state): State<AppState>, ctx: AdminCtx, AdminJson(b): AdminJson<OverrideBody>) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsWrite)?;
    let domain = clean_domain(b.domain.as_deref())?;
    if !policy::LANGS.contains(&b.lang.as_str()) {
        return Err(AdminApiError::invalid("invalidLanguage", "Use en or pt-PT.", "/lang"));
    }
    if !policy::SECTIONS.contains(&b.section.as_str()) && !policy::COOKIE_SECTIONS.contains(&b.section.as_str()) {
        return Err(AdminApiError::invalid("invalidSection", "Unknown section.", "/section"));
    }
    let text = b.body.unwrap_or_default();
    if text.chars().count() > 8000 || text.chars().any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t') {
        return Err(AdminApiError::invalid("invalidBody", "Plain text up to 8000 characters.", "/body"));
    }
    if text.trim().is_empty() {
        state.store.delete_policy_override(&domain, &b.lang, &b.section).await.map_err(AdminApiError::internal)?;
    } else {
        let existing = state.store.list_policy_overrides(&domain).await.map_err(AdminApiError::internal)?;
        if existing.len() >= 100 && !existing.iter().any(|o| o.lang == b.lang && o.section == b.section) {
            return Err(AdminApiError::new(StatusCode::CONFLICT, "tooManyOverrides", "Remove an override first."));
        }
        state
            .store
            .put_policy_override(&PolicyOverride {
                id: uuid::Uuid::new_v4().to_string(),
                domain,
                lang: b.lang.clone(),
                section: b.section.clone(),
                body: text.replace("\r\n", "\n"),
                updated_by: ctx.username.clone(),
                updated_at: now(),
            })
            .await
            .map_err(AdminApiError::internal)?;
    }
    crate::admin_accounts::audit(&state, &ctx, "policy-override-saved", "policy:overrides", None, Some(json!({"lang": b.lang, "section": b.section}))).await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PublishBody {
    #[serde(default)]
    domain: Option<String>,
}

async fn publish(State(state): State<AppState>, ctx: AdminCtx, AdminJson(b): AdminJson<PublishBody>) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsWrite)?;
    let domain = clean_domain(b.domain.as_deref())?;
    let before = state.store.latest_policy_version(&domain).await.map_err(AdminApiError::internal)?.map(|v| v.version);
    let cur = reconcile(&state, &domain, &ctx.username, true).await?;
    let version = cur.latest.as_ref().map(|v| v.version);
    crate::admin_accounts::audit(&state, &ctx, "policy-published", "policy", Some(json!({"version": before})), Some(json!({"version": version}))).await;
    Ok(Json(json!({"version": version, "changed": version != before})).into_response())
}

// ---------------------------------------------------------------------------
// Records of processing
// ---------------------------------------------------------------------------

struct Builtin {
    id: &'static str,
    en: (&'static str, &'static str, &'static str, &'static str),
    pt: (&'static str, &'static str, &'static str, &'static str),
    /// Setting that must be true (or `""` for always).
    when: &'static str,
    retention: &'static [&'static str],
    recipients: &'static str,
}

// (name, purpose, categories of data and people, security measures)
const BUILTINS: &[Builtin] = &[
    Builtin {
        id: "mail-service",
        en: ("Mail service", "Providing mail, contacts, calendars and notes to account holders.", "Account details, mailbox content, contacts, calendar entries and notes of account holders and their correspondents.", "Hashed passwords, sealed stored credentials, access control by role, HTTPS."),
        pt: ("Serviço de correio", "Prestar correio, contactos, calendários e notas aos titulares de contas.", "Dados da conta, conteúdo da caixa de correio, contactos, entradas de calendário e notas dos titulares e dos seus correspondentes.", "Palavras-passe com hash, credenciais guardadas seladas, controlo de acesso por perfil, HTTPS."),
        when: "", retention: &[], recipients: "hosting",
    },
    Builtin {
        id: "security-logging",
        en: ("Authentication and security records", "Detecting and limiting abuse of accounts and the service.", "Sessions, sign-in history, IP addresses, security alerts and the admin audit log.", "Retention limits per store, optional IP reduction, log redaction settings."),
        pt: ("Registos de autenticação e segurança", "Detetar e limitar o abuso das contas e do serviço.", "Sessões, histórico de início de sessão, endereços IP, alertas de segurança e registo de auditoria de administração.", "Limites de retenção por armazenamento, redução opcional de IP, definições de ocultação nos registos."),
        when: "", retention: &["sessions", "authHistory", "authHistoryIp", "auditLogIp", "alertEvents"], recipients: "hosting",
    },
    Builtin {
        id: "data-requests",
        en: ("Data subject requests", "Answering requests to access, export, correct, restrict or erase data.", "Requester address, request type and status, and the exported data parts.", "Export parts sealed with a server key, time-limited downloads, retention of completed requests."),
        pt: ("Pedidos dos titulares dos dados", "Responder a pedidos de acesso, exportação, retificação, limitação ou apagamento de dados.", "Endereço do requerente, tipo e estado do pedido e as partes de dados exportadas.", "Partes de exportação seladas com uma chave do servidor, transferências com prazo, retenção dos pedidos concluídos."),
        when: "dsr.enabled", retention: &["dsrRequests"], recipients: "hosting",
    },
    Builtin {
        id: "cookie-consent",
        en: ("Cookie consent records", "Remembering and evidencing cookie choices.", "Decision, category choices, region code and privacy signal; the account id only when signed in. No IP address or user agent.", "Names and decisions only; purge after the configured period."),
        pt: ("Registos de consentimento de cookies", "Memorizar e comprovar as escolhas de cookies.", "Decisão, categorias escolhidas, código de região e sinal de privacidade; o identificador da conta apenas com sessão iniciada. Sem endereço IP nem agente de utilizador.", "Apenas nomes e decisões; eliminação após o período configurado."),
        when: "cookies.logConsent", retention: &[], recipients: "hosting",
    },
    Builtin {
        id: "remote-content",
        en: ("Remote content proxy", "Loading images and linked content from messages without exposing the reader to the sender.", "Addresses of remote resources requested while reading mail; no content is kept beyond a cache.", "Fetched server-side, cache with time limit, tracker removal where enabled."),
        pt: ("Proxy de conteúdo remoto", "Carregar imagens e conteúdo ligado das mensagens sem expor o leitor ao remetente.", "Endereços de recursos remotos pedidos ao ler correio; nenhum conteúdo é guardado além de uma cache.", "Obtido no servidor, cache com prazo, remoção de rastreadores quando ativa."),
        when: "", retention: &["imageHandles"], recipients: "remote",
    },
    Builtin {
        id: "assistant",
        en: ("AI assistant", "Drafting and summarising text at the user's request.", "Text the user sends to the assistant and the reply; request records.", "Used only when the user asks; request records kept for a limited period."),
        pt: ("Assistente de IA", "Redigir e resumir texto a pedido do utilizador.", "Texto que o utilizador envia ao assistente e a resposta; registos dos pedidos.", "Usado apenas quando o utilizador pede; registos dos pedidos conservados por período limitado."),
        when: "assistant.enabled", retention: &["assistAudit"], recipients: "assistant",
    },
    Builtin {
        id: "open-tracking",
        en: ("Open tracking", "Telling a sender whether a message they chose to track was opened.", "Open events of recipients of messages sent with tracking on.", "Per-message opt-in, limited retention."),
        pt: ("Rastreio de abertura", "Indicar ao remetente se uma mensagem que escolheu rastrear foi aberta.", "Eventos de abertura dos destinatários de mensagens enviadas com rastreio ativo.", "Ativação por mensagem, retenção limitada."),
        when: "mail.openTracking", retention: &["openTracking"], recipients: "hosting",
    },
    Builtin {
        id: "metrics",
        en: ("Service statistics", "Operating and sizing the service.", "Aggregate request and timing counts without personal identifiers.", "Aggregated buckets, rolled up after the configured period."),
        pt: ("Estatísticas do serviço", "Operar e dimensionar o serviço.", "Contagens agregadas de pedidos e tempos, sem identificadores pessoais.", "Intervalos agregados, consolidados após o período configurado."),
        when: "", retention: &["requestMetrics"], recipients: "hosting",
    },
];

async fn builtin_activities(state: &AppState, snapshot: &policy::Snapshot, lang: &str) -> Vec<Value> {
    let pt = lang == "pt-PT";
    let text = |k: &str| snapshot.get(k).and_then(Value::as_str).unwrap_or("").to_owned();
    let basis = text("policy.legalBases");
    let transfers = text("policy.hosting.transfers");
    let hosting = text("policy.hosting.provider");
    let mut out = Vec::new();
    for b in BUILTINS {
        let enabled = b.when.is_empty() || {
            let v = snapshot.get(b.when).and_then(Value::as_bool);
            match v {
                Some(v) => v,
                None => flag(state, "", b.when).await,
            }
        };
        let (name, purpose, categories, security) = if pt { b.pt } else { b.en };
        let retention: Vec<String> = b
            .retention
            .iter()
            .filter_map(|id| {
                let days = snapshot.get(&format!("retention:{id}"))?;
                Some(match days.as_i64() {
                    Some(d) if d > 0 => format!("{}: {} {}", id, d, if pt { "dias" } else { "days" }),
                    _ => format!("{}: {}", id, if pt { "sem limite automático" } else { "no automatic limit" }),
                })
            })
            .collect();
        let recipients = match b.recipients {
            "remote" => if pt { "Servidores que alojam o conteúdo remoto pedido" } else { "Servers hosting the remote content requested" }.to_owned(),
            "assistant" => if pt { "Fornecedor do assistente configurado pelo administrador" } else { "The assistant provider the administrator configured" }.to_owned(),
            _ => if hosting.is_empty() { if pt { "Fornecedor de alojamento (por indicar)" } else { "Hosting provider (not yet filled in)" }.to_owned() } else { hosting.clone() },
        };
        out.push(json!({
            "id": b.id, "builtin": true, "enabled": enabled, "name": name, "purpose": purpose,
            "legalBasis": basis, "categories": categories, "recipients": recipients,
            "transfers": transfers, "retention": retention.join("; "), "security": security,
        }));
    }
    out
}

fn processors(snapshot: &policy::Snapshot, lang: &str) -> Vec<Value> {
    let pt = lang == "pt-PT";
    policy::EGRESS_RECIPIENTS
        .iter()
        .map(|(class, en, ptx, key)| {
            let blocked = !key.is_empty() && snapshot.get(*key).and_then(Value::as_str) == Some("block");
            json!({"id": class, "builtin": true, "name": if pt { ptx } else { en }, "enabled": !blocked, "source": "egress-class"})
        })
        .collect()
}

#[derive(Deserialize)]
struct LangQuery {
    lang: Option<String>,
}

fn record_json(r: &ComplianceRecord) -> Value {
    let mut v = serde_json::from_str::<Value>(&r.body_json).unwrap_or_else(|_| json!({}));
    if let Some(o) = v.as_object_mut() {
        o.insert("id".into(), json!(r.id));
        o.insert("kind".into(), json!(r.kind));
        o.insert("name".into(), json!(r.name));
        o.insert("builtin".into(), json!(false));
        o.insert("updatedAt".into(), json!(r.updated_at));
        o.entry("enabled").or_insert(json!(true));
    }
    v
}

async fn all_records(state: &AppState, lang: &str) -> Result<(Vec<Value>, Vec<Value>), AdminApiError> {
    let snapshot = policy::gather(state, "").await.map_err(AdminApiError::internal)?;
    let mut activities = builtin_activities(state, &snapshot, lang).await;
    let mut procs = processors(&snapshot, lang);
    activities.extend(state.store.list_compliance_records("activity").await.map_err(AdminApiError::internal)?.iter().map(record_json));
    procs.extend(state.store.list_compliance_records("processor").await.map_err(AdminApiError::internal)?.iter().map(record_json));
    Ok((activities, procs))
}

async fn records_view(State(state): State<AppState>, ctx: AdminCtx, Query(q): Query<LangQuery>) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsRead)?;
    let lang = if q.lang.as_deref() == Some("pt-PT") { "pt-PT" } else { "en" };
    let (activities, processors) = all_records(&state, lang).await?;
    Ok(no_store(Json(json!({"activities": activities, "processors": processors})).into_response()))
}

const ACTIVITY_FIELDS: [&str; 8] = ["purpose", "legalBasis", "subjects", "categories", "recipients", "transfers", "retention", "security"];
const PROCESSOR_FIELDS: [&str; 5] = ["purpose", "location", "safeguards", "url", "notes"];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordBody {
    kind: String,
    name: String,
    #[serde(default)]
    fields: std::collections::BTreeMap<String, String>,
    #[serde(default = "yes")]
    enabled: bool,
}
fn yes() -> bool {
    true
}

fn check_record(b: &RecordBody) -> Result<String, AdminApiError> {
    let allowed: &[&str] = match b.kind.as_str() {
        "activity" => &ACTIVITY_FIELDS,
        "processor" => &PROCESSOR_FIELDS,
        _ => return Err(AdminApiError::invalid("invalidKind", "Use activity or processor.", "/kind")),
    };
    let name = b.name.trim();
    if name.is_empty() || name.chars().count() > 200 || name.chars().any(char::is_control) {
        return Err(AdminApiError::invalid("invalidName", "A name of 1 to 200 characters.", "/name"));
    }
    let mut body = serde_json::Map::new();
    for (k, v) in &b.fields {
        if !allowed.contains(&k.as_str()) {
            return Err(AdminApiError::invalid("invalidField", "Unknown field.", format!("/fields/{k}")));
        }
        if v.chars().count() > 2000 || v.chars().any(|c| c.is_control() && c != '\n') {
            return Err(AdminApiError::invalid("invalidField", "Plain text up to 2000 characters.", format!("/fields/{k}")));
        }
        body.insert(k.clone(), json!(v));
    }
    body.insert("enabled".into(), json!(b.enabled));
    Ok(Value::Object(body).to_string())
}

async fn add_record(State(state): State<AppState>, ctx: AdminCtx, AdminJson(b): AdminJson<RecordBody>) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsWrite)?;
    let body_json = check_record(&b)?;
    let existing = state.store.list_compliance_records(&b.kind).await.map_err(AdminApiError::internal)?;
    if existing.len() >= MAX_ROWS {
        return Err(AdminApiError::new(StatusCode::CONFLICT, "tooManyRecords", "Remove a record before adding another."));
    }
    let row = ComplianceRecord { id: uuid::Uuid::new_v4().to_string(), kind: b.kind, name: b.name.trim().to_owned(), body_json, updated_by: ctx.username.clone(), updated_at: now() };
    state.store.put_compliance_record(&row).await.map_err(AdminApiError::internal)?;
    crate::admin_accounts::audit(&state, &ctx, "compliance-record-saved", &format!("compliance:{}", row.kind), None, Some(json!({"name": row.name}))).await;
    Ok((StatusCode::CREATED, Json(record_json(&row))).into_response())
}

async fn update_record(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>, AdminJson(b): AdminJson<RecordBody>) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsWrite)?;
    let Some(current) = state.store.get_compliance_record(&id).await.map_err(AdminApiError::internal)? else {
        return Err(AdminApiError::not_found("No such record."));
    };
    if current.kind != b.kind {
        return Err(AdminApiError::invalid("invalidKind", "The kind cannot change.", "/kind"));
    }
    let body_json = check_record(&b)?;
    let row = ComplianceRecord { id, kind: current.kind, name: b.name.trim().to_owned(), body_json, updated_by: ctx.username.clone(), updated_at: now() };
    state.store.put_compliance_record(&row).await.map_err(AdminApiError::internal)?;
    crate::admin_accounts::audit(&state, &ctx, "compliance-record-saved", &format!("compliance:{}", row.kind), None, Some(json!({"name": row.name}))).await;
    Ok(Json(record_json(&row)).into_response())
}

async fn delete_record(State(state): State<AppState>, ctx: AdminCtx, Path(id): Path<String>) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsWrite)?;
    if !state.store.delete_compliance_record(&id).await.map_err(AdminApiError::internal)? {
        return Err(AdminApiError::not_found("No such record."));
    }
    crate::admin_accounts::audit(&state, &ctx, "compliance-record-deleted", "compliance", None, None).await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[derive(Deserialize)]
struct ExportQuery {
    lang: Option<String>,
    format: Option<String>,
    set: Option<String>,
}

const CSV_COLS: [&str; 10] = ["name", "enabled", "purpose", "legalBasis", "subjects", "categories", "recipients", "transfers", "retention", "security"];
const PROC_COLS: [&str; 6] = ["name", "enabled", "purpose", "location", "safeguards", "url"];

fn cell(v: &Value, key: &str) -> String {
    match v.get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Bool(b)) => b.to_string(),
        Some(Value::Null) | None => String::new(),
        Some(o) => o.to_string(),
    }
}

fn csv_of(rows: &[Value], cols: &[&str]) -> String {
    let mut out = cols.join(",");
    out.push_str("\r\n");
    for r in rows {
        out.push_str(&cols.iter().map(|c| crate::cookie_consent::csv_field(&cell(r, c))).collect::<Vec<_>>().join(","));
        out.push_str("\r\n");
    }
    out
}

fn html_of(title: &str, rows: &[Value], cols: &[&str], lang: &str) -> String {
    let mut out = format!(
        "<!doctype html>\n<html lang=\"{lang}\"><head><meta charset=\"utf-8\"><title>{t}</title><style>body{{font:14px/1.4 system-ui,sans-serif}}table{{border-collapse:collapse;width:100%}}td,th{{border:1px solid #888;padding:4px 6px;vertical-align:top;text-align:start}}@media print{{@page{{size:A4 landscape}}}}</style></head><body><h1>{t}</h1><table><thead><tr>",
        t = policy::html_escape(title),
    );
    for c in cols {
        out.push_str(&format!("<th>{}</th>", policy::html_escape(c)));
    }
    out.push_str("</tr></thead><tbody>");
    for r in rows {
        out.push_str("<tr>");
        for c in cols {
            out.push_str(&format!("<td>{}</td>", policy::html_escape(&cell(r, c))));
        }
        out.push_str("</tr>");
    }
    out.push_str("</tbody></table></body></html>\n");
    out
}

async fn export_records(State(state): State<AppState>, ctx: AdminCtx, Query(q): Query<ExportQuery>) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsRead)?;
    let lang = if q.lang.as_deref() == Some("pt-PT") { "pt-PT" } else { "en" };
    let (activities, procs) = all_records(&state, lang).await?;
    let (rows, cols, title, stem) = if q.set.as_deref() == Some("processors") {
        (procs, &PROC_COLS[..], if lang == "pt-PT" { "Subcontratantes e destinatários" } else { "Sub-processors and recipients" }, "sub-processors")
    } else {
        (activities, &CSV_COLS[..], if lang == "pt-PT" { "Registo das atividades de tratamento" } else { "Records of processing activities" }, "processing-records")
    };
    let (ct, ext, body) = match q.format.as_deref() {
        Some("json") => ("application/json", "json", serde_json::to_string_pretty(&rows).unwrap_or_default()),
        Some("html") => ("text/html; charset=utf-8", "html", html_of(title, &rows, cols, lang)),
        _ => ("text/csv; charset=utf-8", "csv", csv_of(&rows, cols)),
    };
    Ok((
        [
            (header::CONTENT_TYPE, ct.to_owned()),
            (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{stem}.{ext}\"")),
            (header::CACHE_CONTROL, "no-store".to_owned()),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_owned()),
        ],
        body,
    )
        .into_response())
}

// ---------------------------------------------------------------------------
// The signed-in person's view
// ---------------------------------------------------------------------------

/// `GET /api/privacy/processing`: what this service processes about the signed-in
/// person, in plain terms, from the enabled activities and the contact settings.
async fn my_processing(State(state): State<AppState>, headers: HeaderMap, Query(q): Query<LangQuery>) -> Response {
    if let Err(response) = crate::authed(&state, &headers).await {
        return response;
    }
    let lang = lang_of(q.lang.as_deref(), &headers);
    let Ok(snapshot) = policy::gather(&state, "").await else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    let text = |k: &str| snapshot.get(k).and_then(Value::as_str).unwrap_or("").to_owned();
    let mut activities = builtin_activities(&state, &snapshot, lang).await;
    activities.retain(|a| a["enabled"] == json!(true));
    if let Ok(custom) = state.store.list_compliance_records("activity").await {
        activities.extend(custom.iter().map(record_json).filter(|a| a["enabled"] == json!(true)));
    }
    let activities: Vec<Value> = activities
        .iter()
        .map(|a| json!({
            "id": a["id"], "name": a["name"], "purpose": a["purpose"], "categories": a["categories"],
            "recipients": a["recipients"], "retention": a["retention"], "legalBasis": a["legalBasis"],
        }))
        .collect();
    no_store(
        Json(json!({
            "controller": {"name": text("policy.controller.name"), "address": text("policy.controller.address")},
            "contact": {"name": text("dsr.contact.name"), "email": text("dsr.contact.email"), "url": text("dsr.contact.url")},
            "supervisoryAuthority": text("policy.supervisoryAuthority"),
            "activities": activities,
            "recipients": processors(&snapshot, lang),
            "policyUrl": "/privacy",
            "lang": lang,
        }))
        .into_response(),
    )
}

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

async fn summary(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsRead)?;
    let cur = reconcile(&state, "", &ctx.username, false).await?;
    let custom = state.store.list_compliance_records("activity").await.map_err(AdminApiError::internal)?.len();
    let procs = state.store.list_compliance_records("processor").await.map_err(AdminApiError::internal)?.len();
    let d = dpia::summary(&state).await?;
    Ok(no_store(
        Json(json!({
            "policy": {
                "version": cur.latest.as_ref().map(|v| v.version),
                "pending": cur.pending.len(),
                "missing": policy::warnings(&cur.snapshot),
            },
            "records": {"custom": custom, "processors": procs, "builtin": BUILTINS.len()},
            "dpia": d,
        }))
        .into_response(),
    ))
}
