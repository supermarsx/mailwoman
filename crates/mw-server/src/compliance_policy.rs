//! The privacy policy and cookie policy generator.
//!
//! The text is made from the effective settings of a domain (the *facts*), the
//! static cookie inventory, the retention table and the egress classes, in
//! English and European Portuguese. An administrator may replace the text of any
//! section per language; that override is part of the same flat snapshot, so a
//! published version freezes both. The generator describes what is configured.
//! It does not state that a configuration meets any law.

use crate::{
    AppState, auth_logs::sweeper as retention_sweeper, cookie_inventory::INVENTORY, settings_registry,
};
use mw_store::settings_policy::SettingSubject;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

/// A flat snapshot: setting values (`cookies.mode`), derived facts (`retention:outbox`)
/// and section overrides (`override:en:contact`).
pub(crate) type Snapshot = BTreeMap<String, Value>;

pub(crate) const LANGS: [&str; 2] = ["en", "pt-PT"];
/// Section ids in display order. The cookie policy uses [`COOKIE_SECTIONS`].
pub(crate) const SECTIONS: [&str; 11] = [
    "controller", "data", "purposes", "cookies", "retention", "rights", "remote", "recipients",
    "transfers", "security", "contact",
];
pub(crate) const COOKIE_SECTIONS: [&str; 3] = ["cookie-what", "cookies", "cookie-choices"];

/// Setting keys whose value the generator reads.
const PREFIXES: [&str; 8] = [
    "cookies.", "dsr.", "privacy.", "logging.redaction.", "assistant.", "policy.", "mail.openTracking", "mail.readReceipts",
];
/// Keys that are read but do not describe the processing, so a change to them is not a policy change.
const NOT_POLICY: [&str; 3] = ["cookies.policyVersion", "policy.autoVersion", "policy.reconsentOnChange"];

/// The fields an operator must fill in before the policy is complete.
pub(crate) const REQUIRED: [&str; 5] = [
    "policy.controller.name", "policy.controller.address", "policy.legalBases",
    "policy.hosting.provider", "policy.supervisoryAuthority",
];

fn is_policy_key(key: &str) -> bool {
    PREFIXES.iter().any(|p| key.starts_with(p)) && !NOT_POLICY.contains(&key)
        && !key.starts_with("privacy.image")
        && !matches!(key, "privacy.maxBytes" | "privacy.cacheBytes" | "privacy.ratePerMin" | "privacy.rateBurst")
        && !key.starts_with("cookies.text.")
        && !key.starts_with("dsr.notify.")
        && !key.starts_with("dsr.reauth.")
}

/// Gather the snapshot for `domain` (empty = deployment defaults) with the stored overrides.
pub(crate) async fn gather(state: &AppState, domain: &str) -> Result<Snapshot, mw_store::StoreError> {
    let subject = SettingSubject { personal_owner: None, domain, address: "", account_id: "" };
    let mut out = Snapshot::new();
    for v in settings_registry::effective_all(&state.store, &subject, |d| is_policy_key(d.key)).await? {
        out.insert(v.key, v.value);
    }
    for e in retention_sweeper::ENTRIES {
        let days = match (e.key, e.fixed_days) {
            (Some(k), _) => Some(retention_sweeper::num(&state.store, k).await),
            (None, d) => d,
        };
        out.insert(format!("retention:{}", e.id), days.map_or(Value::Null, Value::from));
    }
    for r in state.store.list_cookie_rules().await? {
        out.insert(format!("cookierule:{}:{}", r.kind, r.pattern), json!(r.category));
    }
    for o in state.store.list_policy_overrides(domain).await? {
        out.insert(format!("override:{}:{}", o.lang, o.section), json!(o.body));
    }
    Ok(out)
}

/// One difference between two snapshots.
pub(crate) fn diff(old: &Snapshot, new: &Snapshot) -> Vec<Value> {
    let mut keys: Vec<&String> = old.keys().chain(new.keys()).collect();
    keys.sort();
    keys.dedup();
    keys.into_iter()
        .filter(|k| old.get(*k) != new.get(*k))
        .map(|k| json!({"key": k, "old": old.get(k), "new": new.get(k)}))
        .collect()
}

fn s<'a>(f: &'a Snapshot, key: &str) -> &'a str {
    f.get(key).and_then(Value::as_str).map_or("", str::trim)
}
fn b(f: &Snapshot, key: &str) -> bool {
    f.get(key).and_then(Value::as_bool).unwrap_or(false)
}
fn n(f: &Snapshot, key: &str) -> i64 {
    f.get(key).and_then(Value::as_i64).unwrap_or(0)
}
fn l<'a>(lang: &str, en: &'a str, pt: &'a str) -> &'a str {
    if lang == "pt-PT" { pt } else { en }
}
fn onoff(lang: &str, on: bool) -> &'static str {
    match (lang == "pt-PT", on) {
        (false, true) => "on",
        (false, false) => "off",
        (true, true) => "ativo",
        (true, false) => "desativado",
    }
}

/// A rendered section.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Section {
    pub id: &'static str,
    pub title: String,
    /// Markdown (headings are added by the page, not here).
    pub body: String,
    pub overridden: bool,
    /// Operator settings this section needs and does not have.
    pub missing: Vec<&'static str>,
}

fn title(lang: &str, id: &str) -> &'static str {
    let pt = lang == "pt-PT";
    match (id, pt) {
        ("controller", false) => "Who is responsible",
        ("controller", true) => "Quem é o responsável",
        ("data", false) => "What we process",
        ("data", true) => "Que dados tratamos",
        ("purposes", false) => "Why we process it",
        ("purposes", true) => "Porque os tratamos",
        ("cookies", false) | ("cookie-what", false) => if id == "cookies" { "Cookies and browser storage" } else { "What this page covers" },
        ("cookies", true) | ("cookie-what", true) => if id == "cookies" { "Cookies e armazenamento do navegador" } else { "O que esta página abrange" },
        ("cookie-choices", false) => "Your choices",
        ("cookie-choices", true) => "As suas escolhas",
        ("retention", false) => "How long we keep it",
        ("retention", true) => "Durante quanto tempo conservamos",
        ("rights", false) => "Your rights",
        ("rights", true) => "Os seus direitos",
        ("remote", false) => "Remote content in messages",
        ("remote", true) => "Conteúdo remoto nas mensagens",
        ("recipients", false) => "Who receives data",
        ("recipients", true) => "Quem recebe dados",
        ("transfers", false) => "Transfers outside the hosting country",
        ("transfers", true) => "Transferências para fora do país de alojamento",
        ("security", false) => "How data is protected",
        ("security", true) => "Como os dados são protegidos",
        ("contact", false) => "Contact and complaints",
        ("contact", true) => "Contacto e reclamações",
        _ => "",
    }
}

/// The third-party traffic classes, for the recipients section and the sub-processor list.
/// `(class, en, pt, setting that enables it or "")`.
pub(crate) const EGRESS_RECIPIENTS: [(&str, &str, &str, &str); 8] = [
    ("push", "Push notification services chosen by your browser or device", "Serviços de notificações push escolhidos pelo seu navegador ou dispositivo", ""),
    ("remote_content", "The servers that host remote images in messages (fetched through this server)", "Os servidores que alojam imagens remotas nas mensagens (obtidas através deste servidor)", "privacy.remoteContent"),
    ("keyservers", "OpenPGP key servers, when keys are looked up", "Servidores de chaves OpenPGP, quando são procuradas chaves", ""),
    ("pki", "Certificate revocation services, when S/MIME is used", "Serviços de revogação de certificados, quando é usado S/MIME", ""),
    ("oauth", "Mail providers' sign-in services, when a provider account is linked", "Serviços de início de sessão de fornecedores de correio, quando uma conta de fornecedor é associada", ""),
    ("analytics", "The analytics collector, when the administrator enables one", "O coletor de análise, quando o administrador ativa um", ""),
    ("update_check", "The release feed, when the update check is enabled", "O feed de versões, quando a verificação de atualizações está ativa", ""),
    ("package_tracking", "Parcel tracking providers, when the administrator enables them", "Fornecedores de seguimento de encomendas, quando o administrador os ativa", ""),
];

fn list(items: &[String]) -> String {
    items.iter().map(|i| format!("- {i}")).collect::<Vec<_>>().join("\n")
}

fn or_blank(lang: &str, v: &str) -> String {
    if v.is_empty() { l(lang, "(not yet filled in by the operator)", "(ainda não preenchido pelo operador)").to_owned() } else { v.to_owned() }
}

fn retention_label(lang: &str, id: &str) -> String {
    let pt = lang == "pt-PT";
    let (en, ptt) = match id {
        "passwordChangeAudit" => ("Password change records", "Registos de alteração de palavra-passe"),
        "ssoLoginAudit" => ("Single sign-on sign-in records", "Registos de início de sessão único"),
        "dlpAudit" => ("Outgoing content check records", "Registos de verificação de conteúdo enviado"),
        "assistAudit" => ("Assistant request records", "Registos de pedidos ao assistente"),
        "alertEvents" => ("Security alerts", "Alertas de segurança"),
        "jobRuns" => ("Background job history", "Histórico de tarefas em segundo plano"),
        "outbox" => ("Delivered outgoing mail queue entries", "Entradas entregues da fila de envio"),
        "sessions" => ("Web sessions", "Sessões web"),
        "nativeSessions" => ("App sessions", "Sessões de aplicações"),
        "rememberedDevices" => ("Remembered devices", "Dispositivos memorizados"),
        "authHistoryIp" => ("IP addresses in sign-in history", "Endereços IP no histórico de início de sessão"),
        "auditLogIp" => ("IP addresses in the admin audit log", "Endereços IP no registo de auditoria"),
        "authHistory" => ("Sign-in history", "Histórico de início de sessão"),
        "requestMetrics" => ("Request statistics", "Estatísticas de pedidos"),
        "dsrRequests" => ("Completed data requests", "Pedidos de dados concluídos"),
        "openTracking" => ("Open tracking events", "Eventos de rastreio de abertura"),
        "diagnostics" => ("Diagnostic runs", "Execuções de diagnóstico"),
        "logFiles" => ("Server log files", "Ficheiros de registo do servidor"),
        "accountGeo" => ("Sign-in locations", "Localizações de início de sessão"),
        "uploads" => ("Temporary uploads", "Carregamentos temporários"),
        other => return other.to_owned(),
    };
    if pt { ptt.to_owned() } else { en.to_owned() }
}

/// Stores the retention section lists (others are internal bookkeeping).
const RETENTION_LISTED: [&str; 20] = [
    "sessions", "nativeSessions", "rememberedDevices", "authHistory", "authHistoryIp", "auditLogIp", "passwordChangeAudit",
    "ssoLoginAudit", "dlpAudit", "assistAudit", "alertEvents", "jobRuns", "outbox", "requestMetrics", "dsrRequests",
    "openTracking", "diagnostics", "logFiles", "accountGeo", "uploads",
];

fn body(f: &Snapshot, lang: &str, id: &str) -> (String, Vec<&'static str>) {
    let mut missing = Vec::new();
    let mut need = |key: &'static str| {
        if s(f, key).is_empty() {
            missing.push(key);
        }
    };
    let out = match id {
        "controller" => {
            need("policy.controller.name");
            need("policy.controller.address");
            format!(
                "{}: {}\n\n{}: {}",
                l(lang, "Controller", "Responsável pelo tratamento"),
                or_blank(lang, s(f, "policy.controller.name")),
                l(lang, "Address", "Morada"),
                or_blank(lang, s(f, "policy.controller.address")),
            )
        }
        "data" => {
            let mut items = vec![
                l(lang, "Account details: your address, display name and sign-in credentials (passwords are stored hashed).", "Dados da conta: o seu endereço, nome e credenciais de início de sessão (as palavras-passe são guardadas com hash).").to_owned(),
                l(lang, "Mail content and the settings, contacts, calendars and notes you create, stored so the service can show them to you.", "Conteúdo do correio e as definições, contactos, calendários e notas que cria, guardados para o serviço os poder mostrar.").to_owned(),
                l(lang, "Security records: sessions, sign-in history and the audit log, which can include IP addresses.", "Registos de segurança: sessões, histórico de início de sessão e registo de auditoria, que podem incluir endereços IP.").to_owned(),
            ];
            if b(f, "cookies.logConsent") {
                items.push(l(lang, "Your cookie choices, kept as a consent record.", "As suas escolhas de cookies, guardadas como registo de consentimento.").to_owned());
            }
            if b(f, "assistant.enabled") {
                items.push(l(lang, "Text you send to the assistant, when you use it.", "O texto que envia ao assistente, quando o usa.").to_owned());
            }
            if b(f, "mail.openTracking") {
                items.push(l(lang, "Whether a recipient opened a message you sent, when open tracking is used on that message.", "Se um destinatário abriu uma mensagem que enviou, quando o rastreio de abertura é usado nessa mensagem.").to_owned());
            }
            list(&items)
        }
        "purposes" => {
            need("policy.legalBases");
            let mut items = vec![
                l(lang, "Providing the mail service and keeping it secure.", "Prestar o serviço de correio e mantê-lo seguro.").to_owned(),
                l(lang, "Answering requests about your data.", "Responder a pedidos sobre os seus dados.").to_owned(),
            ];
            if b(f, "cookies.enabled") {
                items.push(l(lang, "Remembering your cookie choices.", "Memorizar as suas escolhas de cookies.").to_owned());
            }
            format!(
                "{}\n\n{}: {}",
                list(&items),
                l(lang, "Legal bases", "Fundamentos jurídicos"),
                or_blank(lang, s(f, "policy.legalBases")),
            )
        }
        "cookie-what" => l(
            lang,
            "This page describes the cookies and browser storage this service sets, the category of each, and how to change your choice.",
            "Esta página descreve os cookies e o armazenamento do navegador que este serviço define, a categoria de cada um e como alterar a sua escolha.",
        )
        .to_owned(),
        "cookies" => {
            let mut rows: Vec<String> = Vec::new();
            for cat in ["necessary", "functional", "analytics", "marketing"] {
                let entries: Vec<String> = INVENTORY
                    .iter()
                    .filter(|e| e.category == cat)
                    .map(|e| format!("`{}` ({}, {}): {}", e.pattern, e.kind, e.duration, e.purpose))
                    .chain(f.iter().filter_map(|(k, v)| {
                        let rest = k.strip_prefix("cookierule:")?;
                        (v.as_str() == Some(cat)).then(|| format!("`{}` ({})", rest.split_once(':').map_or(rest, |x| x.1), rest.split_once(':').map_or("", |x| x.0)))
                    }))
                    .collect();
                if !entries.is_empty() {
                    rows.push(format!("**{}**\n{}", category_label(lang, cat), list(&entries)));
                }
            }
            let mut out = rows.join("\n\n");
            out.push_str(&format!(
                "\n\n{}: {}.",
                l(lang, "Cookie consent banner", "Aviso de consentimento de cookies"),
                onoff(lang, b(f, "cookies.enabled")),
            ));
            if b(f, "cookies.enabled") {
                out.push_str(&format!(
                    " {} {} {}.",
                    l(lang, "A consent decision stays valid for", "Uma decisão de consentimento é válida durante"),
                    n(f, "cookies.lifetimeDays"),
                    l(lang, "days.", "dias."),
                ));
                if b(f, "cookies.honourGpc") {
                    out.push_str(l(lang, " A Global Privacy Control signal is treated as a refusal of analytics and marketing.", " Um sinal Global Privacy Control é tratado como recusa de análise e marketing."));
                }
            }
            out
        }
        "cookie-choices" => l(
            lang,
            "You can change your choice at any time from the cookie settings link of the app. Necessary storage cannot be switched off because the service does not work without it. You can also clear cookies and site data in your browser.",
            "Pode alterar a sua escolha em qualquer altura na ligação de definições de cookies da aplicação. O armazenamento necessário não pode ser desativado porque o serviço não funciona sem ele. Pode também apagar cookies e dados do site no seu navegador.",
        )
        .to_owned(),
        "retention" => {
            let items: Vec<String> = RETENTION_LISTED
                .iter()
                .filter_map(|id| {
                    let days = f.get(&format!("retention:{id}"))?;
                    let text = match days.as_i64() {
                        Some(d) if d > 0 => format!("{d} {}", l(lang, "days", "dias")),
                        Some(_) => l(lang, "no automatic limit (kept until removed)", "sem limite automático (mantido até ser removido)").to_owned(),
                        None => l(lang, "until the record is removed", "até o registo ser removido").to_owned(),
                    };
                    Some(format!("{}: {}", retention_label(lang, id), text))
                })
                .collect();
            let mut out = list(&items);
            if b(f, "cookies.logConsent") {
                out.push_str(&format!(
                    "\n{}",
                    l(lang, "- Cookie consent records: kept for the period the administrator set for the consent log.", "- Registos de consentimento de cookies: conservados durante o período definido pelo administrador para o registo de consentimento."),
                ));
            }
            out
        }
        "rights" => {
            let mut types = Vec::new();
            for (k, en, pt) in [
                ("dsr.type.access", "access your data", "aceder aos seus dados"),
                ("dsr.type.portability", "receive a copy in a portable format", "receber uma cópia num formato portátil"),
                ("dsr.type.rectification", "have data corrected", "retificar dados"),
                ("dsr.type.erasure", "have your data erased", "apagar os seus dados"),
                ("dsr.type.restriction", "restrict how your data is used", "limitar a utilização dos seus dados"),
                ("dsr.type.objection", "object to processing", "opor-se ao tratamento"),
            ] {
                if b(f, k) {
                    types.push(l(lang, en, pt).to_owned());
                }
            }
            if !b(f, "dsr.enabled") || types.is_empty() {
                l(lang, "Requests about your data can be sent to the contact below.", "Os pedidos sobre os seus dados podem ser enviados para o contacto abaixo.").to_owned()
            } else {
                format!(
                    "{}\n{}\n\n{} {} {}.",
                    l(lang, "You can ask, from the data requests page of the app, to:", "Pode pedir, na página de pedidos de dados da aplicação, para:"),
                    list(&types),
                    l(lang, "We aim to respond within", "Procuramos responder no prazo de"),
                    n(f, "dsr.sla.days"),
                    l(lang, "days.", "dias"),
                )
            }
        }
        "remote" => {
            let mode = s(f, "privacy.remoteContent");
            let text = match (mode, lang == "pt-PT") {
                ("block", false) => "Remote images and linked content in messages are not loaded.",
                ("block", true) => "As imagens remotas e o conteúdo ligado nas mensagens não são carregados.",
                ("direct", false) => "Remote images and linked content in messages are loaded directly by your browser from the sender's servers, which can then see your address and device.",
                ("direct", true) => "As imagens remotas e o conteúdo ligado nas mensagens são carregados diretamente pelo seu navegador a partir dos servidores do remetente, que podem assim ver o seu endereço e dispositivo.",
                (_, false) => "Remote images and linked content in messages are fetched by this server on your behalf, so the sender's server does not see your address or device.",
                (_, true) => "As imagens remotas e o conteúdo ligado nas mensagens são obtidos por este servidor em seu nome, pelo que o servidor do remetente não vê o seu endereço nem o seu dispositivo.",
            };
            let mut out = text.to_owned();
            if b(f, "privacy.trackers") {
                out.push_str(l(lang, " Known tracking pixels are removed.", " Os pixels de rastreio conhecidos são removidos."));
            }
            if b(f, "privacy.cleanLinks") {
                out.push_str(l(lang, " Tracking parameters are removed from links.", " Os parâmetros de rastreio são removidos das ligações."));
            }
            out
        }
        "recipients" => {
            let mut items = vec![l(lang, "The hosting provider named under hosting, which stores the data for us.", "O fornecedor de alojamento indicado em alojamento, que guarda os dados por nossa conta.").to_owned()];
            for (_, en, pt, key) in EGRESS_RECIPIENTS {
                if key.is_empty() || s(f, key) != "block" {
                    items.push(l(lang, en, pt).to_owned());
                }
            }
            need("policy.hosting.provider");
            format!(
                "{}\n\n{}: {}",
                list(&items),
                l(lang, "Hosting provider and location", "Fornecedor de alojamento e localização"),
                or_blank(lang, s(f, "policy.hosting.provider")),
            )
        }
        "transfers" => {
            let t = s(f, "policy.hosting.transfers");
            if t.is_empty() {
                l(lang, "The operator has not listed transfers outside the hosting country.", "O operador não indicou transferências para fora do país de alojamento.").to_owned()
            } else {
                t.to_owned()
            }
        }
        "security" => {
            let mut items = vec![
                l(lang, "Passwords are stored hashed; stored credentials and export files are sealed with a server key.", "As palavras-passe são guardadas com hash; as credenciais guardadas e os ficheiros de exportação são selados com uma chave do servidor.").to_owned(),
                l(lang, "Connections to the service use HTTPS.", "As ligações ao serviço usam HTTPS.").to_owned(),
            ];
            if b(f, "logging.redaction.addresses") || b(f, "logging.redaction.ips") || b(f, "logging.redaction.tokens") {
                let mut what = Vec::new();
                if b(f, "logging.redaction.addresses") { what.push(l(lang, "addresses", "endereços")); }
                if b(f, "logging.redaction.ips") { what.push(l(lang, "IP addresses", "endereços IP")); }
                if b(f, "logging.redaction.tokens") { what.push(l(lang, "tokens", "tokens")); }
                items.push(format!("{} {}.", l(lang, "Server logs redact:", "Os registos do servidor ocultam:"), what.join(", ")));
            }
            list(&items)
        }
        "contact" => {
            need("policy.supervisoryAuthority");
            let mut out = String::new();
            let (name, email, url) = (s(f, "dsr.contact.name"), s(f, "dsr.contact.email"), s(f, "dsr.contact.url"));
            if name.is_empty() && email.is_empty() && url.is_empty() {
                out.push_str(l(lang, "Data protection contact: not yet set by the operator.", "Contacto de proteção de dados: ainda não definido pelo operador."));
            } else {
                out.push_str(&format!("{}: {}", l(lang, "Data protection contact", "Contacto de proteção de dados"), [name, email, url].iter().filter(|x| !x.is_empty()).copied().collect::<Vec<_>>().join(", ")));
            }
            out.push_str(&format!(
                "\n\n{}: {}",
                l(lang, "Supervisory authority", "Autoridade de controlo"),
                or_blank(lang, s(f, "policy.supervisoryAuthority")),
            ));
            out
        }
        _ => String::new(),
    };
    (out, missing)
}

fn category_label(lang: &str, cat: &str) -> &'static str {
    match (cat, lang == "pt-PT") {
        ("necessary", false) => "Necessary",
        ("necessary", true) => "Necessários",
        ("functional", false) => "Functional",
        ("functional", true) => "Funcionais",
        ("analytics", false) => "Analytics",
        ("analytics", true) => "Análise",
        (_, false) => "Marketing",
        (_, true) => "Marketing",
    }
}

/// The sections of one document (`privacy` or `cookies`) in `lang`.
pub(crate) fn render(f: &Snapshot, lang: &str, document: &str) -> Vec<Section> {
    let lang = if LANGS.contains(&lang) { lang } else { "en" };
    let ids: &[&'static str] = if document == "cookies" { &COOKIE_SECTIONS } else { &SECTIONS };
    ids.iter()
        .map(|&id| {
            let (generated, missing) = body(f, lang, id);
            let over = f.get(&format!("override:{lang}:{id}")).and_then(Value::as_str).filter(|v| !v.trim().is_empty());
            Section {
                id,
                title: title(lang, id).to_owned(),
                body: over.map_or(generated, str::to_owned),
                overridden: over.is_some(),
                missing: if over.is_some() { Vec::new() } else { missing },
            }
        })
        .collect()
}

/// The unfilled required operator fields.
pub(crate) fn warnings(f: &Snapshot) -> Vec<&'static str> {
    REQUIRED.iter().copied().filter(|k| s(f, k).is_empty()).collect()
}

pub(crate) fn section_json(sections: &[Section]) -> Value {
    Value::Array(
        sections
            .iter()
            .map(|x| json!({"id": x.id, "title": x.title, "body": x.body, "overridden": x.overridden, "missing": x.missing}))
            .collect(),
    )
}

// ---------------------------------------------------------------------------
// Markdown and HTML
// ---------------------------------------------------------------------------

pub(crate) fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn inline(text: &str) -> String {
    let escaped = html_escape(text);
    let mut out = String::new();
    for (i, part) in escaped.split("**").enumerate() {
        if i % 2 == 1 {
            out.push_str(&format!("<strong>{part}</strong>"));
        } else {
            for (j, code) in part.split('`').enumerate() {
                if j % 2 == 1 {
                    out.push_str(&format!("<code>{code}</code>"));
                } else {
                    out.push_str(code);
                }
            }
        }
    }
    out
}

/// Markdown subset (paragraphs, `- ` lists, bold, code) to HTML. Everything else is escaped text.
pub(crate) fn md_to_html(md: &str) -> String {
    let mut out = String::new();
    let mut in_list = false;
    for block in md.split("\n\n") {
        let mut para: Vec<String> = Vec::new();
        for line in block.lines() {
            if let Some(item) = line.strip_prefix("- ") {
                if !para.is_empty() {
                    out.push_str(&format!("<p>{}</p>\n", para.join("<br>")));
                    para.clear();
                }
                if !in_list {
                    out.push_str("<ul>\n");
                    in_list = true;
                }
                out.push_str(&format!("<li>{}</li>\n", inline(item)));
            } else {
                if in_list {
                    out.push_str("</ul>\n");
                    in_list = false;
                }
                para.push(inline(line));
            }
        }
        if in_list {
            out.push_str("</ul>\n");
            in_list = false;
        }
        if !para.is_empty() {
            out.push_str(&format!("<p>{}</p>\n", para.join("<br>")));
        }
    }
    out
}

pub(crate) fn to_markdown(sections: &[Section], heading: &str, version: i64, date: &str, lang: &str) -> String {
    let mut out = format!("# {heading}\n\n{} {version} · {date}\n\n", l(lang, "Version", "Versão"));
    for x in sections {
        out.push_str(&format!("## {}\n\n{}\n\n", x.title, x.body));
    }
    out
}

pub(crate) fn to_html(sections: &[Section], heading: &str, version: i64, date: &str, lang: &str) -> String {
    let mut out = format!(
        "<!doctype html>\n<html lang=\"{}\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{}</title></head>\n<body>\n<main>\n<h1>{}</h1>\n<p>{} {version} · {}</p>\n",
        html_escape(lang), html_escape(heading), html_escape(heading), l(lang, "Version", "Versão"), html_escape(date),
    );
    for x in sections {
        out.push_str(&format!("<section id=\"{}\"><h2>{}</h2>\n{}</section>\n", x.id, html_escape(&x.title), md_to_html(&x.body)));
    }
    out.push_str("</main>\n</body></html>\n");
    out
}

/// A JSON object of the snapshot, for storing.
pub(crate) fn to_json(f: &Snapshot) -> String {
    Value::Object(f.iter().map(|(k, v)| (k.clone(), v.clone())).collect::<Map<_, _>>()).to_string()
}

pub(crate) fn from_json(text: &str) -> Snapshot {
    serde_json::from_str::<Map<String, Value>>(text).map(|m| m.into_iter().collect()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(pairs: &[(&str, Value)]) -> Snapshot {
        pairs.iter().map(|(k, v)| ((*k).to_owned(), v.clone())).collect()
    }

    #[test]
    fn blank_operator_fields_are_warned_and_flagged_per_section() {
        let f = snap(&[]);
        assert_eq!(warnings(&f).len(), REQUIRED.len());
        let sections = render(&f, "en", "privacy");
        assert_eq!(sections.len(), SECTIONS.len());
        let controller = sections.iter().find(|x| x.id == "controller").unwrap();
        assert_eq!(controller.missing, vec!["policy.controller.name", "policy.controller.address"]);
        let f = snap(&[("policy.controller.name", json!("Example Ltd")), ("policy.controller.address", json!("1 Road"))]);
        assert!(render(&f, "en", "privacy").iter().find(|x| x.id == "controller").unwrap().missing.is_empty());
    }

    #[test]
    fn text_follows_the_configuration_and_the_language() {
        let f = snap(&[("dsr.enabled", json!(true)), ("dsr.type.erasure", json!(true)), ("dsr.sla.days", json!(30))]);
        let en = render(&f, "en", "privacy");
        let pt = render(&f, "pt-PT", "privacy");
        let rights = |s: &[Section]| s.iter().find(|x| x.id == "rights").unwrap().body.clone();
        assert!(rights(&en).contains("have your data erased") && rights(&en).contains("30"));
        assert!(rights(&pt).contains("apagar os seus dados"));
        assert!(!rights(&en).contains("portable format"));
        let off = snap(&[("dsr.enabled", json!(false))]);
        assert!(rights(&render(&off, "en", "privacy")).contains("contact below"));
    }

    #[test]
    fn optional_features_appear_only_when_enabled() {
        let on = snap(&[("assistant.enabled", json!(true)), ("mail.openTracking", json!(true))]);
        let data = |f: &Snapshot| render(f, "en", "privacy").iter().find(|x| x.id == "data").unwrap().body.clone();
        assert!(data(&on).contains("assistant") && data(&on).contains("open tracking"));
        assert!(!data(&snap(&[])).contains("assistant"));
    }

    #[test]
    fn retention_lists_days_and_unlimited_stores() {
        let f = snap(&[("retention:sessions", json!(30)), ("retention:outbox", json!(0)), ("retention:auditLog", Value::Null)]);
        let body = render(&f, "en", "privacy").iter().find(|x| x.id == "retention").unwrap().body.clone();
        assert!(body.contains("Web sessions: 30 days"));
        assert!(body.contains("Delivered outgoing mail queue entries: no automatic limit"));
    }

    #[test]
    fn cookie_policy_lists_the_inventory_by_category_and_admin_rules() {
        let f = snap(&[("cookies.enabled", json!(true)), ("cookies.lifetimeDays", json!(180)), ("cookierule:cookie:vendor_*", json!("analytics"))]);
        let sections = render(&f, "en", "cookies");
        let c = sections.iter().find(|x| x.id == "cookies").unwrap();
        assert!(c.body.contains("`mw_session`") && c.body.contains("**Necessary**"));
        assert!(c.body.contains("`vendor_*`") && c.body.contains("180"));
    }

    #[test]
    fn an_override_replaces_the_generated_text_and_clears_its_warning() {
        let f = snap(&[("override:en:contact", json!("Write to us."))]);
        let contact = render(&f, "en", "privacy").into_iter().find(|x| x.id == "contact").unwrap();
        assert!(contact.overridden && contact.body == "Write to us." && contact.missing.is_empty());
        let pt = render(&f, "pt-PT", "privacy").into_iter().find(|x| x.id == "contact").unwrap();
        assert!(!pt.overridden);
    }

    #[test]
    fn diff_reports_added_removed_and_changed_keys() {
        let old = snap(&[("a", json!(1)), ("b", json!(2))]);
        let new = snap(&[("a", json!(1)), ("b", json!(3)), ("c", json!(true))]);
        let d = diff(&old, &new);
        assert_eq!(d.len(), 2);
        assert_eq!(d[0]["key"], "b");
        assert_eq!(d[1]["old"], Value::Null);
    }

    #[test]
    fn html_escapes_operator_text_and_renders_lists() {
        let html = md_to_html("<script>x</script> **bold**\n\n- one\n- `two`");
        assert!(html.contains("&lt;script&gt;") && !html.contains("<script>"));
        assert!(html.contains("<strong>bold</strong>") && html.contains("<li><code>two</code></li>"));
    }

    #[test]
    fn policy_keys_exclude_version_and_cosmetic_settings() {
        assert!(is_policy_key("cookies.mode"));
        assert!(!is_policy_key("cookies.policyVersion"));
        assert!(!is_policy_key("privacy.imageRateBurstFloor"));
        assert!(!is_policy_key("cookies.text.title"));
        assert!(is_policy_key("policy.legalBases"));
    }
}
