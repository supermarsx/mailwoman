//! The breach register without I/O: statuses and what each move needs, the notice
//! clock, reminder thresholds, the risk questionnaire, template rendering, the
//! supervisory-authority notice, report output, automation rule matching and the
//! playbook checklists.
//!
//! Time is always passed in, so every rule here is testable at a chosen instant.
//! Nothing here decides whether a deployment meets a law; the questionnaire is a
//! prompt for the person assessing the incident, and its result is a suggestion.
use mw_store::breach::BreachRow;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, VecDeque};

pub(crate) const STATUSES: [&str; 7] = [
    "detected",
    "assessing",
    "contained",
    "notified-authority",
    "notified-subjects",
    "closed",
    "not-notifiable",
];
pub(crate) const SOURCES: [&str; 4] = ["manual", "auto", "dsr", "third_party"];
pub(crate) const SEVERITIES: [&str; 4] = ["unassessed", "none", "risk", "high"];
pub(crate) const TYPES: [&str; 8] = [
    "credential-compromise",
    "mailbox-access",
    "lost-device",
    "misdirected-message",
    "insider",
    "malware",
    "third-party",
    "other",
];
pub(crate) const PHASES: [&str; 3] = ["initial", "update", "final"];
pub(crate) const HOUR: i64 = 3600;

pub(crate) fn is_open(status: &str) -> bool {
    !matches!(status, "closed" | "not-notifiable")
}

/// A short reference people can quote: `BR-1A2B3C4D`.
pub(crate) fn short_ref(id: &str) -> String {
    let head: String = id.chars().filter(|c| c.is_ascii_alphanumeric()).take(8).collect();
    format!("BR-{}", head.to_uppercase())
}

// ─── stored body ──────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct Body {
    pub incident_type: String,
    pub description: String,
    pub reporter: String,
    pub subject_categories: Vec<String>,
    pub data_categories: Vec<String>,
    pub approx_subjects: Option<i64>,
    pub approx_records: Option<i64>,
    pub consequences: String,
    pub measures_taken: String,
    pub measures_proposed: String,
    pub answers: BTreeMap<String, String>,
    pub assessment: Option<Assessment>,
    pub affected: Vec<Affected>,
    pub affected_domains: Vec<String>,
    pub timeline: Vec<Entry>,
    pub evidence: Vec<Evidence>,
    pub not_notifiable_reason: String,
    pub close_reason: String,
    pub delay_reasons: String,
    pub authority_notices: Vec<AuthorityRecord>,
    pub subject_notices: Vec<SubjectNoticeRecord>,
    pub reminded: Vec<i64>,
    pub overdue_reminded_at: Option<i64>,
    pub escalated_at: Option<i64>,
    pub auto: Option<Value>,
    pub playbook: BTreeMap<String, CheckState>,
    pub confirmed_by: Option<String>,
    pub confirmed_at: Option<i64>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct Assessment {
    pub score: i64,
    pub suggested: String,
    pub chosen: String,
    pub rationale: String,
    pub by: String,
    pub at: i64,
    pub complete: bool,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct Affected {
    pub address: String,
    pub account_id: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct Entry {
    pub id: String,
    pub ts: i64,
    pub actor: String,
    /// detection, containment, closure or note.
    pub kind: String,
    pub text: String,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct Evidence {
    pub id: String,
    pub ts: i64,
    pub actor: String,
    /// text or link.
    pub kind: String,
    pub label: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct AuthorityRecord {
    pub id: String,
    pub phase: String,
    pub recorded_at: i64,
    pub recorded_by: String,
    pub submitted_at: i64,
    pub reference: String,
    pub late: bool,
    pub delay_reasons: String,
    pub snapshot: Value,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct SubjectNoticeRecord {
    pub id: String,
    pub ts: i64,
    pub by: String,
    pub audience: String,
    pub locale: String,
    pub recipients: i64,
    pub dry_run: bool,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct CheckState {
    pub done: bool,
    pub by: String,
    pub at: i64,
}

impl Body {
    pub(crate) fn of(row: &BreachRow) -> Self {
        serde_json::from_str(&row.body_json).unwrap_or_default()
    }
    pub(crate) fn authority_notified_at(&self) -> Option<i64> {
        self.authority_notices.iter().map(|n| n.submitted_at).min()
    }
}

// ─── clock ────────────────────────────────────────────────────────────────────

pub(crate) fn deadline(aware_at: i64, hours: i64) -> i64 {
    aware_at.saturating_add(hours.max(1).saturating_mul(HOUR))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Phase {
    /// No awareness time yet (a draft).
    NotStarted,
    Running,
    Overdue,
    /// The authority was told within the window.
    StoppedOnTime,
    /// The authority was told after the window.
    StoppedLate,
    /// Closed or marked not notifiable without a notice.
    NotRequired,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ClockView {
    pub phase: Phase,
    pub aware_at: Option<i64>,
    pub deadline_at: Option<i64>,
    /// Seconds to the deadline; negative once overdue. `None` unless running or overdue.
    pub remaining_secs: Option<i64>,
    pub elapsed_secs: Option<i64>,
    pub notified_at: Option<i64>,
}

pub(crate) fn clock(
    aware_at: Option<i64>,
    deadline_at: Option<i64>,
    notified_at: Option<i64>,
    status: &str,
    now: i64,
) -> ClockView {
    let elapsed = aware_at.map(|a| now - a);
    let mut view = ClockView {
        phase: Phase::NotStarted,
        aware_at,
        deadline_at,
        remaining_secs: None,
        elapsed_secs: elapsed,
        notified_at,
    };
    let (Some(_), Some(dl)) = (aware_at, deadline_at) else {
        return view;
    };
    if let Some(n) = notified_at {
        view.phase = if n <= dl { Phase::StoppedOnTime } else { Phase::StoppedLate };
        return view;
    }
    if !is_open(status) {
        view.phase = Phase::NotRequired;
        return view;
    }
    let remaining = dl - now;
    view.remaining_secs = Some(remaining);
    view.phase = if remaining < 0 { Phase::Overdue } else { Phase::Running };
    view
}

// ─── reminders ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reminder {
    /// This many hours (the configured threshold) before the deadline.
    Before(i64),
    Overdue,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ReminderStep {
    pub announce: Option<Reminder>,
    /// Every threshold now crossed, to add to the incident's fired list.
    pub fired: Vec<i64>,
    pub overdue_at: Option<i64>,
}

/// Which reminder, if any, is due at `now`. Thresholds are hours before the
/// deadline. When several were crossed since the last pass only the tightest is
/// announced, and all are marked fired. Once overdue, the first reminder is sent
/// at once and later ones every `repeat_hours` (0: only the first).
pub(crate) fn due_reminder(
    now: i64,
    deadline_at: i64,
    thresholds: &[i64],
    fired: &[i64],
    last_overdue: Option<i64>,
    repeat_hours: i64,
) -> ReminderStep {
    let remaining = deadline_at - now;
    let mut step = ReminderStep::default();
    if remaining < 0 {
        step.fired = thresholds.iter().copied().filter(|t| !fired.contains(t)).collect();
        match last_overdue {
            None => {
                step.announce = Some(Reminder::Overdue);
                step.overdue_at = Some(now);
            }
            Some(at) if repeat_hours > 0 && now - at >= repeat_hours * HOUR => {
                step.announce = Some(Reminder::Overdue);
                step.overdue_at = Some(now);
            }
            Some(_) => {}
        }
        return step;
    }
    let crossed: Vec<i64> = thresholds
        .iter()
        .copied()
        .filter(|t| remaining <= t * HOUR && !fired.contains(t))
        .collect();
    if let Some(tightest) = crossed.iter().copied().min() {
        step.announce = Some(Reminder::Before(tightest));
    }
    step.fired = crossed;
    step
}

// ─── status moves ─────────────────────────────────────────────────────────────

pub(crate) struct Gate<'a> {
    pub severity: &'a str,
    pub authority_notified: bool,
    pub subjects_notified: bool,
    pub reason: &'a str,
    pub body: &'a Body,
}

/// What a move to `to` needs. `Err` carries a stable code the console translates.
pub(crate) fn check_move(from: &str, to: &str, g: &Gate<'_>) -> Result<(), &'static str> {
    if !STATUSES.contains(&to) {
        return Err("unknownStatus");
    }
    if from == to {
        return Err("sameStatus");
    }
    let reason = g.reason.trim();
    match to {
        "not-notifiable" => {
            if g.severity != "none" {
                return Err("needsNoRiskAssessment");
            }
            if reason.chars().count() < 10 && g.body.not_notifiable_reason.trim().chars().count() < 10 {
                return Err("needsReasoning");
            }
        }
        "notified-authority" => {
            if !g.authority_notified {
                return Err("needsAuthorityRecord");
            }
        }
        "notified-subjects" => {
            if !g.subjects_notified {
                return Err("needsSubjectNotice");
            }
        }
        "closed" => {
            if g.severity == "unassessed" {
                return Err("needsAssessment");
            }
            let must_have_notified = matches!(g.severity, "risk" | "high") && !g.authority_notified;
            if must_have_notified && reason.chars().count() < 10 && g.body.close_reason.trim().chars().count() < 10 {
                return Err("needsCloseReasoning");
            }
        }
        "assessing" | "contained" | "detected" => {
            if matches!(from, "closed" | "not-notifiable") && reason.is_empty() {
                return Err("needsReopenReason");
            }
        }
        _ => {}
    }
    Ok(())
}

// ─── risk questionnaire ───────────────────────────────────────────────────────

/// A question: id, then each answer with its points.
pub(crate) const QUESTIONS: &[(&str, &[(&str, i64)])] = &[
    ("confidentiality", &[("none", 0), ("limited", 1), ("broad", 2)]),
    ("special", &[("no", 0), ("yes", 2)]),
    ("identifiable", &[("no", 0), ("hard", 1), ("easy", 2)]),
    ("volume", &[("few", 0), ("some", 1), ("many", 2)]),
    ("vulnerable", &[("no", 0), ("yes", 1)]),
    ("encrypted", &[("yes", -3), ("partial", -1), ("no", 0)]),
    ("contained", &[("yes", -1), ("partial", 0), ("no", 1)]),
    ("harm", &[("none", 0), ("moderate", 2), ("severe", 4)]),
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Suggestion {
    pub score: i64,
    pub level: &'static str,
    pub complete: bool,
    pub answered: usize,
    pub total: usize,
}

/// Score the answers. An unknown answer counts as unanswered. The result is
/// `unassessed` until every question has an answer.
pub(crate) fn suggest(answers: &BTreeMap<String, String>) -> Suggestion {
    let mut score = 0;
    let mut answered = 0;
    for (q, options) in QUESTIONS {
        if let Some((_, points)) = answers.get(*q).and_then(|a| options.iter().find(|(name, _)| name == a)) {
            score += points;
            answered += 1;
        }
    }
    let complete = answered == QUESTIONS.len();
    let severe = answers.get("harm").is_some_and(|h| h == "severe");
    let level = if !complete {
        "unassessed"
    } else if score >= 5 || (severe && score >= 3) {
        "high"
    } else if score >= 1 {
        "risk"
    } else {
        "none"
    };
    Suggestion { score, level, complete, answered, total: QUESTIONS.len() }
}

// ─── playbooks ────────────────────────────────────────────────────────────────

/// Checklist item ids for an incident type (the console has the wording).
pub(crate) fn playbook(kind: &str) -> Vec<&'static str> {
    let mut items = vec!["preserve-evidence", "identify-data", "count-subjects", "inform-dpo"];
    match kind {
        "credential-compromise" => items.extend(["revoke-sessions", "force-password-reset", "review-forwarding", "block-source"]),
        "mailbox-access" => items.extend(["revoke-sessions", "force-password-reset", "review-forwarding", "check-access-log"]),
        "lost-device" => items.extend(["revoke-sessions", "force-password-reset", "confirm-device-encryption"]),
        "misdirected-message" => items.extend(["recall-message", "ask-recipient-delete", "confirm-deletion"]),
        "insider" => items.extend(["revoke-sessions", "restrict-access", "preserve-logs"]),
        "malware" => items.extend(["isolate-systems", "revoke-sessions", "force-password-reset", "rotate-keys"]),
        "third-party" => items.extend(["contact-processor", "request-processor-report", "rotate-keys"]),
        _ => items.extend(["revoke-sessions"]),
    }
    items.extend(["decide-authority", "record-delay", "decide-subjects", "document-measures", "root-cause", "lessons-learned"]);
    items
}

// ─── templates ────────────────────────────────────────────────────────────────

pub(crate) fn locale_of(raw: &str) -> &'static str {
    let l = raw.trim().to_lowercase();
    if l == "pt" || l.starts_with("pt-pt") || l == "pt_pt" { "pt-PT" } else { "en" }
}

pub(crate) const DEFAULT_SUBJECT_EN: &str = "Notice about a personal data breach: {{incident}}";
pub(crate) const DEFAULT_SUBJECT_PT: &str = "Aviso sobre uma violação de dados pessoais: {{incident}}";
pub(crate) const DEFAULT_BODY_EN: &str = "We are writing to tell you about a personal data breach that may affect you.\n\nWhat happened: {{description}}\n\nData involved: {{categories}}\n\nLikely consequences: {{consequences}}\n\nWhat we have done and what we are doing: {{measures}}\n\nWhat you can do: {{advice}}\n\nContact for questions: {{dpo}}\n\nReference: {{reference}}";
pub(crate) const DEFAULT_BODY_PT: &str = "Escrevemos para o informar de uma violação de dados pessoais que pode afetá-lo.\n\nO que aconteceu: {{description}}\n\nDados envolvidos: {{categories}}\n\nConsequências prováveis: {{consequences}}\n\nO que fizemos e o que estamos a fazer: {{measures}}\n\nO que pode fazer: {{advice}}\n\nContacto para esclarecimentos: {{dpo}}\n\nReferência: {{reference}}";

pub(crate) fn default_template(kind: &str, locale: &str) -> &'static str {
    match (kind, locale_of(locale)) {
        ("subject", "pt-PT") => DEFAULT_SUBJECT_PT,
        ("subject", _) => DEFAULT_SUBJECT_EN,
        (_, "pt-PT") => DEFAULT_BODY_PT,
        _ => DEFAULT_BODY_EN,
    }
}

/// The variables a template may use.
pub(crate) const VARIABLES: &[&str] = &[
    "incident", "reference", "title", "date", "description", "categories", "consequences", "measures",
    "advice", "dpo", "dpoName", "dpoEmail", "dpoPhone", "authority", "domain", "recipient",
];

/// Replace `{{name}}` (spaces inside the braces are allowed). A name that is not
/// in `vars` is left in place so it is visible in the preview.
pub(crate) fn render(template: &str, vars: &BTreeMap<String, String>) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        match after.find("}}") {
            Some(end) => {
                let name = after[..end].trim();
                match vars.get(name) {
                    Some(v) => out.push_str(v),
                    None => {
                        out.push_str("{{");
                        out.push_str(&after[..end]);
                        out.push_str("}}");
                    }
                }
                rest = &after[end + 2..];
            }
            None => {
                out.push_str(&rest[start..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

/// Variable names a template uses that are not in [`VARIABLES`].
pub(crate) fn unknown_variables(template: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else { break };
        let name = after[..end].trim().to_string();
        if !VARIABLES.contains(&name.as_str()) && !found.contains(&name) {
            found.push(name);
        }
        rest = &after[end + 2..];
    }
    found
}

// ─── authority notice (Art. 33(3)) ────────────────────────────────────────────

#[derive(Clone, Debug, Default)]
pub(crate) struct AuthorityCfg {
    pub name: String,
    pub url: String,
    pub contact: String,
    pub dpo_name: String,
    pub dpo_email: String,
    pub dpo_phone: String,
}

fn ts(secs: Option<i64>) -> Value {
    match secs.and_then(|s| chrono::DateTime::from_timestamp(s, 0)) {
        Some(d) => json!(d.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
        None => Value::Null,
    }
}

/// The content of a notification to a supervisory authority, grouped as the
/// items of Art. 33(3) (a) to (d). Mailwoman does not submit it.
pub(crate) fn authority_notice(
    row: &BreachRow,
    body: &Body,
    cfg: &AuthorityCfg,
    phase: &str,
    prior: usize,
    now: i64,
) -> Value {
    let late = row.deadline_at.is_some_and(|d| now > d);
    json!({
        "schema": "mailwoman.breach.authority-notice/1",
        "phase": phase,
        "sequence": prior + 1,
        "generatedAt": ts(Some(now)),
        "note": "Prepared by Mailwoman for the operator to submit through the authority's own channel. It has not been sent.",
        "authority": {"name": cfg.name, "url": cfg.url, "contact": cfg.contact},
        "incident": {
            "reference": short_ref(&row.id), "title": row.title, "domain": row.domain,
            "status": row.status, "detectedAt": ts(Some(row.detected_at)),
            "awareAt": ts(row.aware_at), "deadlineAt": ts(row.deadline_at),
        },
        "dataProtectionOfficer": {"name": cfg.dpo_name, "email": cfg.dpo_email, "phone": cfg.dpo_phone},
        "nature": {
            "description": body.description,
            "categoriesOfDataSubjects": body.subject_categories,
            "approximateDataSubjects": body.approx_subjects,
            "categoriesOfData": body.data_categories,
            "approximateRecords": body.approx_records,
        },
        "likelyConsequences": body.consequences,
        "measures": {"taken": body.measures_taken, "proposed": body.measures_proposed},
        "risk": {"level": row.severity, "rationale": body.assessment.as_ref().map(|a| a.rationale.clone()).unwrap_or_default()},
        "delay": {"afterDeadline": late, "reasons": body.delay_reasons},
        "dataSubjectsInformed": !body.subject_notices.iter().all(|n| n.dry_run) && !body.subject_notices.is_empty(),
    })
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#39;")
}

fn text_of(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Array(a) => a.iter().map(text_of).collect::<Vec<_>>().join(", "),
        other => other.to_string(),
    }
}

/// Labels of the printable documents in en and pt-PT.
fn label(locale: &str, key: &str) -> &'static str {
    let pt = locale_of(locale) == "pt-PT";
    match (key, pt) {
        ("title", false) => "Personal data breach notification",
        ("title", true) => "Notificação de violação de dados pessoais",
        ("draft", false) => "Prepared for the operator to submit; Mailwoman has not sent it.",
        ("draft", true) => "Preparada para o operador submeter; o Mailwoman não a enviou.",
        ("authority", false) => "Supervisory authority",
        ("authority", true) => "Autoridade de controlo",
        ("incident", false) => "Incident",
        ("incident", true) => "Incidente",
        ("a", false) => "(a) Nature of the breach",
        ("a", true) => "(a) Natureza da violação",
        ("b", false) => "(b) Data protection officer or other contact",
        ("b", true) => "(b) Encarregado da proteção de dados ou outro contacto",
        ("c", false) => "(c) Likely consequences",
        ("c", true) => "(c) Consequências prováveis",
        ("d", false) => "(d) Measures taken or proposed",
        ("d", true) => "(d) Medidas adotadas ou propostas",
        ("delay", false) => "Reasons for delay",
        ("delay", true) => "Motivos do atraso",
        ("description", false) => "Description",
        ("description", true) => "Descrição",
        ("subjectCategories", false) => "Categories of data subjects",
        ("subjectCategories", true) => "Categorias de titulares de dados",
        ("subjects", false) => "Approximate number of data subjects",
        ("subjects", true) => "Número aproximado de titulares de dados",
        ("dataCategories", false) => "Categories of personal data",
        ("dataCategories", true) => "Categorias de dados pessoais",
        ("records", false) => "Approximate number of records",
        ("records", true) => "Número aproximado de registos",
        ("taken", false) => "Taken",
        ("taken", true) => "Adotadas",
        ("proposed", false) => "Proposed",
        ("proposed", true) => "Propostas",
        ("phase", false) => "Phase",
        ("phase", true) => "Fase",
        ("aware", false) => "Aware at",
        ("aware", true) => "Tomada de conhecimento",
        ("detected", false) => "Detected at",
        ("detected", true) => "Detetada em",
        ("name", false) => "Name",
        ("name", true) => "Nome",
        ("email", false) => "Email",
        ("email", true) => "Email",
        ("phone", false) => "Phone",
        ("phone", true) => "Telefone",
        ("risk", false) => "Assessed risk",
        ("risk", true) => "Risco avaliado",
        _ => "",
    }
}

/// A self-contained printable page for a notice built by [`authority_notice`].
pub(crate) fn authority_html(n: &Value, locale: &str) -> String {
    let l = |k: &str| esc(label(locale, k));
    let g = |p: &str| esc(&text_of(n.pointer(p).unwrap_or(&Value::Null)));
    let row = |k: &str, v: String| format!("<tr><th scope=\"row\">{}</th><td>{}</td></tr>", l(k), v);
    let delay = if n.pointer("/delay/afterDeadline") == Some(&Value::Bool(true)) {
        format!("<h2>{}</h2><p>{}</p>", l("delay"), g("/delay/reasons"))
    } else {
        String::new()
    };
    format!(
        "<!doctype html><html lang=\"{lang}\"><head><meta charset=\"utf-8\"><title>{title}</title><style>body{{font:14px/1.5 system-ui,sans-serif;max-width:46rem;margin:2rem auto;padding:0 1rem;color:#111}}table{{border-collapse:collapse;width:100%;margin:.5rem 0 1rem}}th,td{{border:1px solid #999;padding:.35rem .5rem;text-align:start;vertical-align:top}}th{{width:14rem;background:#f2f2f2}}p.note{{border:1px solid #999;padding:.5rem}}pre{{white-space:pre-wrap;font:inherit}}</style></head><body>\
<h1>{title}</h1><p class=\"note\">{draft}</p>\
<table>{authority}{reference}{inc_title}{phase}{detected}{aware}{risk}</table>\
<h2>{a}</h2><table>{description}{subject_categories}{subjects}{data_categories}{records}</table>\
<h2>{b}</h2><table>{dpo_name}{dpo_email}{dpo_phone}</table>\
<h2>{c}</h2><pre>{consequences}</pre>\
<h2>{d}</h2><table>{taken}{proposed}</table>{delay}</body></html>",
        lang = locale_of(locale),
        title = l("title"),
        draft = l("draft"),
        authority = row("authority", format!("{} {} {}", g("/authority/name"), g("/authority/url"), g("/authority/contact"))),
        reference = row("incident", g("/incident/reference")),
        inc_title = row("description", g("/incident/title")),
        phase = row("phase", format!("{} #{}", g("/phase"), g("/sequence"))),
        detected = row("detected", g("/incident/detectedAt")),
        aware = row("aware", g("/incident/awareAt")),
        risk = row("risk", format!("{} {}", g("/risk/level"), g("/risk/rationale"))),
        a = l("a"),
        description = row("description", g("/nature/description")),
        subject_categories = row("subjectCategories", g("/nature/categoriesOfDataSubjects")),
        subjects = row("subjects", g("/nature/approximateDataSubjects")),
        data_categories = row("dataCategories", g("/nature/categoriesOfData")),
        records = row("records", g("/nature/approximateRecords")),
        b = l("b"),
        dpo_name = row("name", g("/dataProtectionOfficer/name")),
        dpo_email = row("email", g("/dataProtectionOfficer/email")),
        dpo_phone = row("phone", g("/dataProtectionOfficer/phone")),
        c = l("c"),
        consequences = g("/likelyConsequences"),
        d = l("d"),
        taken = row("taken", g("/measures/taken")),
        proposed = row("proposed", g("/measures/proposed")),
        delay = delay,
    )
}

// ─── register output ──────────────────────────────────────────────────────────

pub(crate) const CSV_COLUMNS: [&str; 18] = [
    "reference", "id", "title", "domain", "status", "severity", "source", "type", "detectedAt", "awareAt",
    "deadlineAt", "authorityNotifiedAt", "late", "approxSubjects", "approxRecords", "dataCategories",
    "delayReasons", "closeReason",
];

pub(crate) fn register_row(row: &BreachRow, body: &Body) -> Value {
    let notified = body.authority_notified_at();
    json!({
        "reference": short_ref(&row.id), "id": row.id, "title": row.title, "domain": row.domain,
        "status": row.status, "severity": row.severity, "source": row.source, "type": body.incident_type,
        "detectedAt": ts(Some(row.detected_at)), "awareAt": ts(row.aware_at), "deadlineAt": ts(row.deadline_at),
        "authorityNotifiedAt": ts(notified),
        "late": match (notified, row.deadline_at) { (Some(n), Some(d)) => n > d, _ => false },
        "approxSubjects": body.approx_subjects, "approxRecords": body.approx_records,
        "dataCategories": body.data_categories.join("; "),
        "delayReasons": body.delay_reasons,
        "closeReason": if body.close_reason.is_empty() { body.not_notifiable_reason.clone() } else { body.close_reason.clone() },
    })
}

/// One incident as a printable page with its trail.
pub(crate) fn report_html(row: &BreachRow, body: &Body, events: &[mw_store::breach::BreachEvent], locale: &str) -> String {
    let mut s = String::new();
    let pt = locale_of(locale) == "pt-PT";
    let h = |en: &'static str, p: &'static str| esc(if pt { p } else { en });
    let line = |k: String, v: String| format!("<tr><th scope=\"row\">{}</th><td>{}</td></tr>", k, v);
    s.push_str(&format!(
        "<!doctype html><html lang=\"{}\"><head><meta charset=\"utf-8\"><title>{} {}</title><style>body{{font:14px/1.5 system-ui,sans-serif;max-width:50rem;margin:2rem auto;padding:0 1rem}}table{{border-collapse:collapse;width:100%;margin:.5rem 0 1rem}}th,td{{border:1px solid #999;padding:.3rem .5rem;text-align:start;vertical-align:top}}th{{width:13rem;background:#f2f2f2}}</style></head><body><h1>{} {}</h1><table>",
        locale_of(locale), esc(&short_ref(&row.id)), esc(&row.title), esc(&short_ref(&row.id)), esc(&row.title)
    ));
    let t = |v: Option<i64>| esc(&text_of(&ts(v)));
    s.push_str(&line(h("Status", "Estado"), esc(&row.status)));
    s.push_str(&line(h("Risk", "Risco"), esc(&row.severity)));
    s.push_str(&line(h("Source", "Origem"), esc(&row.source)));
    s.push_str(&line(h("Detected at", "Detetada em"), t(Some(row.detected_at))));
    s.push_str(&line(h("Aware at", "Tomada de conhecimento"), t(row.aware_at)));
    s.push_str(&line(h("Deadline", "Prazo"), t(row.deadline_at)));
    s.push_str(&line(h("Authority notified at", "Autoridade notificada em"), t(body.authority_notified_at())));
    s.push_str(&line(h("Description", "Descrição"), esc(&body.description)));
    s.push_str(&line(h("Data categories", "Categorias de dados"), esc(&body.data_categories.join(", "))));
    s.push_str(&line(h("Consequences", "Consequências"), esc(&body.consequences)));
    s.push_str(&line(h("Measures taken", "Medidas adotadas"), esc(&body.measures_taken)));
    s.push_str(&line(h("Measures proposed", "Medidas propostas"), esc(&body.measures_proposed)));
    if !body.delay_reasons.is_empty() {
        s.push_str(&line(h("Reasons for delay", "Motivos do atraso"), esc(&body.delay_reasons)));
    }
    s.push_str("</table>");
    s.push_str(&format!("<h2>{}</h2><table>", h("Timeline", "Cronologia")));
    for e in &body.timeline {
        s.push_str(&line(format!("{} {}", t(Some(e.ts)), esc(&e.kind)), esc(&e.text)));
    }
    s.push_str(&format!("</table><h2>{}</h2><table>", h("Evidence", "Provas")));
    for e in &body.evidence {
        s.push_str(&line(esc(&e.label), esc(&e.value)));
    }
    s.push_str(&format!("</table><h2>{}</h2><table>", h("Audit trail", "Registo de auditoria")));
    for e in events {
        s.push_str(&line(format!("{} {}", t(Some(e.ts)), esc(&e.actor)), esc(&e.kind)));
    }
    s.push_str("</table></body></html>");
    s
}

// ─── automation rules ─────────────────────────────────────────────────────────

pub(crate) struct Rule {
    pub id: &'static str,
    /// The setting that holds the threshold (0 is off).
    pub key: &'static str,
    pub incident_type: &'static str,
    pub title: &'static str,
}

pub(crate) const RULES: [Rule; 8] = [
    Rule { id: "theft", key: "breach.auto.theft.threshold", incident_type: "credential-compromise", title: "Repeated session security alerts" },
    Rule { id: "credentialStuffing", key: "breach.auto.credentialStuffing.threshold", incident_type: "credential-compromise", title: "Sign-in spray or spike" },
    Rule { id: "massForward", key: "breach.auto.massForward.threshold", incident_type: "mailbox-access", title: "Forwarding rules created in bulk" },
    Rule { id: "roleEscalation", key: "breach.auto.roleEscalation.threshold", incident_type: "insider", title: "Administrator role changes" },
    Rule { id: "exportVolume", key: "breach.auto.exportVolume.threshold", incident_type: "insider", title: "Unusual number of exports" },
    Rule { id: "auditChain", key: "breach.auto.auditChain.threshold", incident_type: "other", title: "Audit chain verification failed" },
    Rule { id: "backupAnomaly", key: "breach.auto.backupAnomaly.threshold", incident_type: "other", title: "Backup or restore failure" },
    Rule { id: "dsrExport", key: "breach.auto.dsrExport.threshold", incident_type: "insider", title: "Many data request exports" },
];

/// Which rule an audit action counts toward, if any. `detail` is the action's
/// parsed detail object.
pub(crate) fn rule_for(action: &str, detail: &Value) -> Option<&'static str> {
    let a = action;
    if a == "session-security-alert" {
        let kind = detail.get("kind").and_then(Value::as_str).unwrap_or_default();
        return matches!(kind, "impossibleTravel" | "concurrentUse" | "rotationReuse" | "fingerprintMismatch").then_some("theft");
    }
    if matches!(a, "signin-spray-source" | "signin-spike-heightened") {
        return Some("credentialStuffing");
    }
    if a.contains("forward") {
        return Some("massForward");
    }
    if matches!(a, "admin-roles-changed" | "admin-role-updated" | "admin-overrides-changed") {
        return Some("roleEscalation");
    }
    if matches!(a, "audit-export" | "cookie-consent-exported") {
        return Some("exportVolume");
    }
    if a.contains("audit-chain") && (a.contains("fail") || a.contains("broken") || a.contains("mismatch")) {
        return Some("auditChain");
    }
    if (a.contains("backup") || a.contains("restore")) && (a.contains("fail") || a.contains("anomal") || a.contains("error") || a.contains("restore")) {
        return Some("backupAnomaly");
    }
    if a == "dsr-export-ready" {
        return Some("dsrExport");
    }
    None
}

/// A sliding count of events. `hit` records one at `now` and returns whether
/// `threshold` events fall in the last `window_secs`; when it does the counter
/// is emptied so one burst opens one draft.
#[derive(Default, Debug)]
pub(crate) struct Counter {
    times: VecDeque<i64>,
}

impl Counter {
    pub(crate) fn hit(&mut self, now: i64, window_secs: i64, threshold: i64) -> bool {
        self.times.push_back(now);
        while self.times.front().is_some_and(|t| *t < now - window_secs) {
            self.times.pop_front();
        }
        if threshold > 0 && self.times.len() as i64 >= threshold {
            self.times.clear();
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const H: i64 = HOUR;

    #[test]
    fn deadline_is_awareness_plus_the_window() {
        assert_eq!(deadline(1_000, 72), 1_000 + 72 * H);
        assert_eq!(deadline(1_000, 0), 1_000 + H, "a zero window is read as one hour");
    }

    #[test]
    fn clock_phases_follow_the_injected_time() {
        let aware = 10_000;
        let dl = deadline(aware, 72);
        assert_eq!(clock(None, None, None, "detected", aware).phase, Phase::NotStarted);
        let running = clock(Some(aware), Some(dl), None, "assessing", aware + 24 * H);
        assert_eq!(running.phase, Phase::Running);
        assert_eq!(running.remaining_secs, Some(48 * H));
        assert_eq!(clock(Some(aware), Some(dl), None, "assessing", dl).phase, Phase::Running, "the deadline instant is still in time");
        let over = clock(Some(aware), Some(dl), None, "assessing", dl + 1);
        assert_eq!(over.phase, Phase::Overdue);
        assert_eq!(over.remaining_secs, Some(-1));
        assert_eq!(clock(Some(aware), Some(dl), Some(dl - 5), "notified-authority", dl + 99).phase, Phase::StoppedOnTime);
        assert_eq!(clock(Some(aware), Some(dl), Some(dl + 5), "notified-authority", dl + 99).phase, Phase::StoppedLate);
        assert_eq!(clock(Some(aware), Some(dl), None, "not-notifiable", dl + 99).phase, Phase::NotRequired);
    }

    #[test]
    fn a_changed_awareness_time_moves_the_deadline() {
        let first = deadline(1_000, 72);
        let moved = deadline(1_000 + 6 * H, 72);
        assert_eq!(moved - first, 6 * H);
    }

    #[test]
    fn reminders_fire_once_per_threshold_and_announce_the_tightest() {
        let dl = deadline(0, 72);
        let thresholds = [6, 24, 48];
        // 60 h left: nothing crossed.
        assert_eq!(due_reminder(dl - 60 * H, dl, &thresholds, &[], None, 24), ReminderStep::default());
        // 47 h left: the 48 h threshold.
        let s = due_reminder(dl - 47 * H, dl, &thresholds, &[], None, 24);
        assert_eq!(s.announce, Some(Reminder::Before(48)));
        assert_eq!(s.fired, vec![48]);
        // Already fired: quiet.
        assert_eq!(due_reminder(dl - 47 * H, dl, &thresholds, &[48], None, 24).announce, None);
        // The worker was down: 5 h left crosses all three, announce only 6, mark all.
        let s = due_reminder(dl - 5 * H, dl, &thresholds, &[], None, 24);
        assert_eq!(s.announce, Some(Reminder::Before(6)));
        let mut fired = s.fired.clone();
        fired.sort();
        assert_eq!(fired, vec![6, 24, 48]);
    }

    #[test]
    fn overdue_reminds_at_once_then_on_the_repeat_interval() {
        let dl = deadline(0, 72);
        let first = due_reminder(dl + 10, dl, &[6], &[6], None, 24);
        assert_eq!(first.announce, Some(Reminder::Overdue));
        assert_eq!(first.overdue_at, Some(dl + 10));
        assert_eq!(due_reminder(dl + 10 + 23 * H, dl, &[6], &[6], Some(dl + 10), 24).announce, None);
        assert_eq!(due_reminder(dl + 10 + 24 * H, dl, &[6], &[6], Some(dl + 10), 24).announce, Some(Reminder::Overdue));
        assert_eq!(due_reminder(dl + 99 * H, dl, &[6], &[6], Some(dl + 10), 0).announce, None, "0 reminds once");
    }

    fn gate<'a>(severity: &'a str, authority: bool, subjects: bool, reason: &'a str, body: &'a Body) -> Gate<'a> {
        Gate { severity, authority_notified: authority, subjects_notified: subjects, reason, body }
    }

    #[test]
    fn moves_need_what_the_workflow_says() {
        let b = Body::default();
        assert_eq!(check_move("detected", "bogus", &gate("none", false, false, "", &b)), Err("unknownStatus"));
        assert_eq!(check_move("detected", "detected", &gate("none", false, false, "", &b)), Err("sameStatus"));
        assert_eq!(check_move("assessing", "not-notifiable", &gate("risk", false, false, "no personal data was in the file", &b)), Err("needsNoRiskAssessment"));
        assert_eq!(check_move("assessing", "not-notifiable", &gate("none", false, false, "short", &b)), Err("needsReasoning"));
        assert_eq!(check_move("assessing", "not-notifiable", &gate("none", false, false, "the file held no personal data", &b)), Ok(()));
        assert_eq!(check_move("assessing", "notified-authority", &gate("risk", false, false, "", &b)), Err("needsAuthorityRecord"));
        assert_eq!(check_move("assessing", "notified-authority", &gate("risk", true, false, "", &b)), Ok(()));
        assert_eq!(check_move("notified-authority", "notified-subjects", &gate("high", true, false, "", &b)), Err("needsSubjectNotice"));
        assert_eq!(check_move("contained", "closed", &gate("unassessed", false, false, "", &b)), Err("needsAssessment"));
        assert_eq!(check_move("contained", "closed", &gate("risk", false, false, "", &b)), Err("needsCloseReasoning"));
        assert_eq!(check_move("contained", "closed", &gate("risk", true, false, "", &b)), Ok(()));
        assert_eq!(check_move("closed", "assessing", &gate("risk", true, false, "", &b)), Err("needsReopenReason"));
        assert_eq!(check_move("closed", "assessing", &gate("risk", true, false, "new evidence", &b)), Ok(()));
    }

    fn answers(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn questionnaire_is_unassessed_until_complete_and_scores_the_rest() {
        assert_eq!(suggest(&answers(&[("confidentiality", "broad")])).level, "unassessed");
        let low = suggest(&answers(&[
            ("confidentiality", "none"), ("special", "no"), ("identifiable", "no"), ("volume", "few"),
            ("vulnerable", "no"), ("encrypted", "yes"), ("contained", "yes"), ("harm", "none"),
        ]));
        assert!(low.complete);
        assert_eq!(low.level, "none");
        let mid = suggest(&answers(&[
            ("confidentiality", "limited"), ("special", "no"), ("identifiable", "hard"), ("volume", "few"),
            ("vulnerable", "no"), ("encrypted", "no"), ("contained", "partial"), ("harm", "none"),
        ]));
        assert_eq!(mid.level, "risk");
        let high = suggest(&answers(&[
            ("confidentiality", "broad"), ("special", "yes"), ("identifiable", "easy"), ("volume", "many"),
            ("vulnerable", "yes"), ("encrypted", "no"), ("contained", "no"), ("harm", "severe"),
        ]));
        assert_eq!(high.level, "high");
        assert_eq!(suggest(&answers(&[("confidentiality", "nonsense")])).answered, 0);
    }

    #[test]
    fn templates_fill_known_variables_and_leave_unknown_ones_visible() {
        let mut vars = BTreeMap::new();
        vars.insert("incident".to_string(), "Mailbox exposure".to_string());
        vars.insert("dpo".to_string(), "dpo@example.test".to_string());
        let out = render("About {{incident}}; contact {{ dpo }}; {{missing}} and {{open", &vars);
        assert_eq!(out, "About Mailbox exposure; contact dpo@example.test; {{missing}} and {{open");
        assert_eq!(unknown_variables("{{incident}} {{nope}} {{ nope }} {{measures}}"), vec!["nope".to_string()]);
    }

    #[test]
    fn default_templates_use_only_known_variables() {
        for t in [DEFAULT_SUBJECT_EN, DEFAULT_SUBJECT_PT, DEFAULT_BODY_EN, DEFAULT_BODY_PT] {
            assert!(unknown_variables(t).is_empty(), "{t}");
        }
        assert_eq!(locale_of("pt-PT"), "pt-PT");
        assert_eq!(locale_of("fr"), "en");
    }

    fn row() -> BreachRow {
        BreachRow {
            id: "1a2b3c4d-0000".into(), title: "Mailbox exposed".into(), domain: "example.test".into(),
            status: "assessing".into(), severity: "risk".into(), source: "manual".into(),
            detected_at: 1_000, aware_at: Some(1_000), deadline_at: Some(deadline(1_000, 72)),
            assignee: None, created_at: 1_000, updated_at: 1_000, rev: 1, body_json: "{}".into(),
        }
    }

    #[test]
    fn authority_notice_has_the_art_33_3_items_and_escapes_markup() {
        let mut body = Body::default();
        body.description = "A <script>alert(1)</script> leak".into();
        body.data_categories = vec!["email addresses".into()];
        body.approx_subjects = Some(40);
        body.consequences = "phishing".into();
        body.measures_taken = "sessions revoked".into();
        let cfg = AuthorityCfg { name: "Authority".into(), dpo_email: "dpo@example.test".into(), ..Default::default() };
        let r = row();
        let n = authority_notice(&r, &body, &cfg, "initial", 0, 2_000);
        assert_eq!(n["schema"], "mailwoman.breach.authority-notice/1");
        assert_eq!(n["nature"]["approximateDataSubjects"], 40);
        assert_eq!(n["likelyConsequences"], "phishing");
        assert_eq!(n["measures"]["taken"], "sessions revoked");
        assert_eq!(n["dataProtectionOfficer"]["email"], "dpo@example.test");
        assert_eq!(n["delay"]["afterDeadline"], false);
        let html = authority_html(&n, "en");
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>"));
        assert!(html.contains("(a) Nature of the breach"));
        let late = authority_notice(&r, &body, &cfg, "update", 1, deadline(1_000, 72) + 1);
        assert_eq!(late["delay"]["afterDeadline"], true);
        assert_eq!(late["sequence"], 2);
        assert!(authority_html(&late, "pt-PT").contains("Natureza da violação"));
    }

    #[test]
    fn rules_match_the_actions_they_watch() {
        assert_eq!(rule_for("session-security-alert", &json!({"kind": "impossibleTravel"})), Some("theft"));
        assert_eq!(rule_for("session-security-alert", &json!({"kind": "countryChange"})), None);
        assert_eq!(rule_for("signin-spray-source", &json!({})), Some("credentialStuffing"));
        assert_eq!(rule_for("admin-roles-changed", &json!({})), Some("roleEscalation"));
        assert_eq!(rule_for("dsr-export-ready", &json!({})), Some("dsrExport"));
        assert_eq!(rule_for("forwarding-rule-created", &json!({})), Some("massForward"));
        assert_eq!(rule_for("audit-chain-verify-failed", &json!({})), Some("auditChain"));
        assert_eq!(rule_for("login-succeeded", &json!({})), None);
    }

    #[test]
    fn counter_opens_one_draft_per_burst_inside_the_window() {
        let mut c = Counter::default();
        assert!(!c.hit(0, 600, 3));
        assert!(!c.hit(100, 600, 3));
        assert!(c.hit(200, 600, 3));
        assert!(!c.hit(210, 600, 3), "emptied after a hit");
        // Events spread wider than the window never reach the threshold.
        let mut c = Counter::default();
        assert!(!c.hit(0, 100, 3));
        assert!(!c.hit(150, 100, 3));
        assert!(!c.hit(300, 100, 3));
    }

    #[test]
    fn every_playbook_ends_with_the_notification_decisions() {
        for t in TYPES {
            let p = playbook(t);
            assert!(p.contains(&"decide-authority") && p.contains(&"document-measures"), "{t}");
        }
        assert!(playbook("mailbox-access").contains(&"revoke-sessions"));
    }

    #[test]
    fn register_row_flags_late_notice() {
        let mut body = Body::default();
        let r = row();
        body.authority_notices.push(AuthorityRecord { submitted_at: r.deadline_at.unwrap() + 10, ..Default::default() });
        assert_eq!(register_row(&r, &body)["late"], true);
        assert_eq!(short_ref(&r.id), "BR-1A2B3C4D");
    }
}
