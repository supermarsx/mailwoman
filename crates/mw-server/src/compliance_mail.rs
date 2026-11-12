//! Durable, sealed outbound compliance notifications. Only explicitly enabled
//! server events may enqueue mail; administrative readers never receive bodies.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx},
    admin_router::AdminRegistrar,
    capabilities::Capability as C,
    settings_registry,
};
use axum::{
    Json,
    extract::{Path, State},
    http::Method,
    response::{IntoResponse, Response},
};
use base64::Engine as _;
use mw_store::{compliance::ComplianceRecord, settings_policy::SettingSubject};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const PENDING: &str = "compliance-mail:pending";
const SENDING: &str = "compliance-mail:sending";
#[derive(Serialize, Deserialize, Clone)]
struct Mail {
    recipient: String,
    subject: String,
    text: String,
    attempts: u32,
    error: String,
}
fn now() -> i64 {
    chrono::Utc::now().timestamp()
}
pub(crate) async fn config(
    state: &AppState,
) -> Result<std::collections::BTreeMap<String, Value>, String> {
    let subject = SettingSubject {
        personal_owner: None,
        domain: "",
        address: "",
        account_id: "",
    };
    let rows = settings_registry::effective_all(&state.store, &subject, |d| {
        d.key.starts_with("compliance.mail.")
            || d.key.starts_with("incidents.")
            || d.key.starts_with("dsr.notify.")
            || d.key == "breach.notify.email"
            || d.key == "breach.subjects.email"
            || d.key == "retention.complianceMail.days"
            || d.key == "dsr.legalHold.subjects"
    })
    .await
    .map_err(|_| "Notification settings unavailable")?;
    Ok(rows.into_iter().map(|r| (r.key, r.value)).collect())
}
fn encode(state: &AppState, id: &str, mail: &Mail) -> Result<String, String> {
    let plain = serde_json::to_vec(mail).map_err(|_| "Could not encode notification")?;
    Ok(base64::engine::general_purpose::STANDARD.encode(
        state
            .store
            .seal_compliance_delivery(id, &plain)
            .map_err(|_| "Could not encrypt notification")?,
    ))
}
fn decode(state: &AppState, row: &ComplianceRecord) -> Result<Mail, String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&row.body_json)
        .map_err(|_| "Invalid notification")?;
    let plain = state
        .store
        .open_compliance_delivery(&row.id, &bytes)
        .map_err(|_| "Notification authentication failed")?;
    serde_json::from_slice(&plain).map_err(|_| "Invalid notification payload".into())
}
pub(crate) async fn enqueue(
    state: &AppState,
    event: &str,
    recipient: &str,
    subject: &str,
    text: &str,
) -> Result<String, String> {
    mw_smtp::validate_mailbox(recipient).map_err(|_| "Invalid notification recipient")?;
    if subject.chars().any(char::is_control) || subject.len() > 512 || text.len() > 32768 {
        return Err("Notification content exceeds limits".into());
    }
    let cfg = config(state).await?;
    if cfg.get("compliance.mail.enabled") != Some(&json!(true)) {
        return Err("Outbound compliance email is disabled".into());
    }
    let id = format!(
        "compliance-mail:{}",
        crate::push_relay::hash_token(&format!(
            "{}:{}:{}:{}",
            event,
            recipient.to_lowercase(),
            subject,
            text
        ))
    );
    let mail = Mail {
        recipient: recipient.into(),
        subject: subject.into(),
        text: text.into(),
        attempts: 0,
        error: String::new(),
    };
    let row = ComplianceRecord {
        id: id.clone(),
        kind: PENDING.into(),
        name: event.into(),
        body_json: encode(state, &id, &mail)?,
        updated_by: "system".into(),
        updated_at: now(),
    };
    state
        .store
        .insert_compliance_record_once(&row)
        .await
        .map_err(|_| "Could not queue notification")?;
    Ok(id)
}
#[derive(Clone, Debug)]
struct Dial {
    plan: mw_egress::plan::EgressPlan,
    host: String,
    port: u16,
    private: bool,
}
impl mw_net::Dialer for Dial {
    fn dial<'a>(
        &'a self,
        host: &'a str,
        port: u16,
    ) -> mw_net::BoxFuture<'a, std::io::Result<mw_net::BoxStream>> {
        Box::pin(async move {
            if host != self.host || port != self.port {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "Notification relay target mismatch",
                ));
            }
            let pipe =
                mw_egress::plan::connect_system_mail_with(&self.plan, host, port, self.private)
                    .await
                    .map_err(|_| std::io::Error::other("Notification relay route unavailable"))?;
            Ok(Box::new(pipe) as mw_net::BoxStream)
        })
    }
}
enum Delivery {
    Sent,
    Retry,
    Uncertain,
    Suppressed,
}
fn mime(from: &str, mail: &Mail, id: &str) -> Vec<u8> {
    let mut words = Vec::new();
    let mut chunk = String::new();
    for ch in mail.subject.chars() {
        if chunk.len() + ch.len_utf8() > 42 {
            words.push(format!(
                "=?UTF-8?B?{}?=",
                base64::engine::general_purpose::STANDARD.encode(chunk.as_bytes())
            ));
            chunk.clear();
        }
        chunk.push(ch);
    }
    if !chunk.is_empty() {
        words.push(format!(
            "=?UTF-8?B?{}?=",
            base64::engine::general_purpose::STANDARD.encode(chunk.as_bytes())
        ));
    }
    let subject = words.join("\r\n ");
    let body = base64::engine::general_purpose::STANDARD.encode(mail.text.as_bytes());
    let mut raw = format!(
        "From: {from}\r\nTo: {}\r\nSubject: {subject}\r\nDate: {}\r\nMessage-ID: <{}@mailwoman.invalid>\r\nAuto-Submitted: auto-generated\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Transfer-Encoding: base64\r\n\r\n",
        mail.recipient,
        chrono::Utc::now().to_rfc2822(),
        id.replace(':', "-")
    );
    for chunk in body.as_bytes().chunks(76) {
        raw.push_str(std::str::from_utf8(chunk).unwrap_or(""));
        raw.push_str("\r\n");
    }
    raw.into_bytes()
}
async fn deliver(state: &AppState, mail: &Mail, id: &str, event: &str) -> Result<Delivery, String> {
    let cfg = config(state).await?;
    let value = |key: &str| cfg.get(key).and_then(Value::as_str).unwrap_or("");
    if cfg.get("compliance.mail.enabled") != Some(&json!(true)) {
        return Err("Outbound compliance email is disabled".into());
    }
    let gate = if event.starts_with("dsr-requester:") {
        Some("dsr.notify.emailRequester")
    } else if event.starts_with("dsr-admin:") {
        Some("dsr.notify.emailAdmins")
    } else if event.starts_with("breach-operator:") {
        Some("breach.notify.email")
    } else if event.starts_with("operational:") {
        Some("incidents.notify.email")
    } else if event.starts_with("breach-subject:") {
        Some("breach.subjects.email")
    } else {
        None
    };
    if gate.is_some_and(|key| cfg.get(key) != Some(&json!(true))) {
        return Ok(Delivery::Suppressed);
    }
    let host = value("compliance.mail.host").trim();
    let from = value("compliance.mail.from").trim();
    if host.is_empty() || host.contains(['/', ':', '@']) {
        return Err("Configure a relay hostname".into());
    }
    mw_smtp::validate_mailbox(from).map_err(|_| "Configure a valid sender address")?;
    let port = cfg
        .get("compliance.mail.port")
        .and_then(Value::as_u64)
        .unwrap_or(587) as u16;
    let username = value("compliance.mail.username");
    let credentials = if username.is_empty() {
        mw_smtp::Credentials::None
    } else {
        let password = if let Some(file) = std::env::var_os("MW_COMPLIANCE_SMTP_PASSWORD_FILE") {
            use std::io::Read;
            let mut bytes = Vec::new();
            std::fs::File::open(file)
                .map_err(|_| "SMTP password file unavailable")?
                .take(4097)
                .read_to_end(&mut bytes)
                .map_err(|_| "SMTP password file unavailable")?;
            if bytes.len() > 4096 {
                return Err("SMTP password file exceeds limits".into());
            }
            String::from_utf8(bytes)
                .map_err(|_| "SMTP password file must be UTF-8")?
                .trim_end_matches(['\r', '\n'])
                .to_owned()
        } else {
            std::env::var("MW_COMPLIANCE_SMTP_PASSWORD")
                .map_err(|_| "SMTP password is not configured")?
        };
        mw_smtp::Credentials::Plain {
            user: username.into(),
            pass: password,
        }
    };
    let plan = crate::image_proxy::load_plan(&state.store)
        .await
        .map_err(|_| "Outbound route configuration unavailable")?;
    if cfg.get("compliance.mail.requireProxy") != Some(&json!(false))
        && plan
            .selected_route(mw_egress::TrafficClass::SystemMail, None)
            .is_none()
    {
        return Err("Configure a system_mail proxy route before sending".into());
    }
    let smtp = mw_smtp::Submitter::new(mw_smtp::SubmitConfig {
        host: host.into(),
        port,
        security: if value("compliance.mail.security") == "implicit" {
            mw_smtp::Security::ImplicitTls
        } else {
            mw_smtp::Security::StartTls
        },
        credentials,
        ehlo_name: "mailwoman".into(),
    })
    .with_dialer(Some(std::sync::Arc::new(Dial {
        plan,
        host: host.into(),
        port,
        private: cfg.get("compliance.mail.allowPrivate") == Some(&json!(true)),
    })));
    let message = mw_smtp::Outgoing {
        mail_from: from.into(),
        rcpt_to: vec![mail.recipient.clone()],
        raw: mime(from, mail, id),
    };
    match tokio::time::timeout(std::time::Duration::from_secs(30), smtp.submit(message)).await {
        Ok(Ok(result)) if result.accepted.iter().any(|r| r == &mail.recipient) => {
            Ok(Delivery::Sent)
        }
        Ok(Err(mw_smtp::SmtpError::DeliveryUncertain(_))) | Err(_) => Ok(Delivery::Uncertain),
        _ => Ok(Delivery::Retry),
    }
}
async fn pass(state: &AppState) {
    let Ok(cfg) = config(state).await else {
        return;
    };
    let configured = cfg
        .get("retention.complianceMail.days")
        .and_then(Value::as_i64)
        .unwrap_or(30)
        .max(1);
    let (classes, enforce) = crate::retention_tiers::class_days(&state.store).await;
    let days = crate::retention_tiers::clamp(
        crate::auth_logs::sweeper::Unit::Age,
        Some(crate::retention_tiers::Class::Short),
        configured,
        &classes,
        enforce,
    )
    .max(1);
    // Preserve the queue conservatively when legal holds exist; encrypted recipient
    // fields cannot be filtered by SQL without revealing their identifiers.
    if cfg
        .get("dsr.legalHold.subjects")
        .and_then(Value::as_array)
        .is_none_or(|holds| holds.is_empty())
    {
        let _ = state
            .store
            .purge_compliance_delivery_history(now() - days * 86400)
            .await;
    }
    if cfg.get("compliance.mail.enabled") != Some(&json!(true)) {
        return;
    }
    // A process may die after SMTP accepted DATA. Never automatically resend a
    // delivery abandoned in that window; keep it visible for operator review.
    for old in state
        .store
        .due_compliance_deliveries(SENDING, now() - 300)
        .await
        .unwrap_or_default()
    {
        let mut row = old.clone();
        row.kind = "compliance-mail:uncertain".into();
        row.updated_at = now();
        let _ = state.store.transition_compliance_record(&old, &row).await;
    }
    for old in state
        .store
        .due_compliance_deliveries(PENDING, now())
        .await
        .unwrap_or_default()
    {
        let Ok(mut mail) = decode(state, &old) else {
            let mut failed = old.clone();
            failed.kind = "compliance-mail:failed".into();
            failed.updated_at = now();
            let _ = state
                .store
                .transition_compliance_record(&old, &failed)
                .await;
            continue;
        };
        let mut claimed = old.clone();
        claimed.kind = SENDING.into();
        claimed.updated_at = now();
        if !matches!(
            state
                .store
                .transition_compliance_record(&old, &claimed)
                .await,
            Ok(true)
        ) {
            continue;
        }
        mail.attempts += 1;
        let result = deliver(state, &mail, &old.id, &old.name).await;
        let outcome = match &result {
            Ok(Delivery::Sent) => "accepted",
            Ok(Delivery::Suppressed) => "skipped",
            Ok(Delivery::Uncertain) => "uncertain",
            Ok(Delivery::Retry) => "retrying",
            Err(_) => "failed",
        };
        crate::breach::record_email_delivery(state, &old.name, &mail.recipient, outcome).await;
        let mut next = claimed.clone();
        next.kind = match &result {
            Ok(Delivery::Sent) => "compliance-mail:sent",
            Ok(Delivery::Suppressed) => "compliance-mail:dismissed",
            Ok(Delivery::Uncertain) => "compliance-mail:uncertain",
            _ if mail.attempts >= 5 => "compliance-mail:failed",
            _ => PENDING,
        }
        .into();
        mail.error = match result {
            Ok(Delivery::Suppressed) => "Notification channel disabled before delivery".into(),
            Err(reason) => reason,
            Ok(Delivery::Retry) => "Relay refused or could not accept the notification".into(),
            Ok(Delivery::Uncertain) => "Acceptance unknown; review before sending again".into(),
            _ => String::new(),
        };
        next.updated_at = now()
            + if next.kind == PENDING {
                60 * (1_i64 << mail.attempts)
            } else {
                0
            };
        if let Ok(body) = encode(state, &old.id, &mail) {
            next.body_json = body;
            let _ = state
                .store
                .transition_compliance_record(&claimed, &next)
                .await;
        }
        crate::audit::security_event(
            state,
            crate::audit::SecurityEvent::system("compliance-mail-delivery")
                .target(&old.id)
                .detail(json!({"status":next.kind,"attempts":mail.attempts})),
        )
        .await;
    }
}
pub(crate) fn start(state: AppState) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            pass(&state).await;
        }
    });
}
pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::POST,
        "/compliance/mail/{id}/resolve",
        C::SettingsWrite,
        resolve,
    );
    r.admin_endpoint(Method::GET, "/compliance/mail", C::SettingsRead, status);
    r.admin_endpoint(
        Method::POST,
        "/compliance/mail/test",
        C::SettingsWrite,
        test_mail,
    );
}
async fn status(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsRead)?;
    let mut rows = Vec::new();
    for kind in [
        PENDING,
        SENDING,
        "compliance-mail:sent",
        "compliance-mail:failed",
        "compliance-mail:uncertain",
        "compliance-mail:dismissed",
    ] {
        for row in state
            .store
            .due_compliance_deliveries(kind, i64::MAX)
            .await
            .map_err(AdminApiError::internal)?
        {
            let mail = decode(&state, &row).ok();
            rows.push(json!({"id":row.id,"event":row.name,"status":kind.rsplit(':').next(),"at":row.updated_at,"attempts":mail.as_ref().map(|m|m.attempts),"error":mail.map(|m|m.error)}));
        }
    }
    Ok(([(axum::http::header::CACHE_CONTROL,"no-store")],Json(json!({"items":rows,"passwordConfigured":std::env::var_os("MW_COMPLIANCE_SMTP_PASSWORD").is_some()||std::env::var_os("MW_COMPLIANCE_SMTP_PASSWORD_FILE").is_some()}))).into_response())
}
async fn test_mail(
    State(state): State<AppState>,
    ctx: AdminCtx,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsWrite)?;
    let cfg = config(&state).await.map_err(|_| {
        AdminApiError::new(
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "unavailable",
            "Notification settings unavailable",
        )
    })?;
    let recipients = cfg
        .get("dsr.notify.operatorAddresses")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if recipients.is_empty() {
        return Err(AdminApiError::invalid(
            "noRecipients",
            "Configure operator recipients first.",
            "/recipients",
        ));
    }
    for recipient in recipients.iter().filter_map(Value::as_str) {
        enqueue(
            &state,
            &format!("test:{}", uuid::Uuid::new_v4()),
            recipient,
            "Mailwoman compliance notification test",
            "This is a test of the configured compliance notification relay.",
        )
        .await
        .map_err(|_| {
            AdminApiError::invalid(
                "notificationDisabled",
                "Enable and configure compliance email first.",
                "/settings",
            )
        })?;
    }
    Ok(Json(json!({"queued":recipients.len()})).into_response())
}

/// The requester export/erasure also covers the encrypted delivery queue. Bodies
/// and verification codes are deliberately excluded from the metadata export.
pub(crate) async fn subject_records(
    state: &AppState,
    address: &str,
    purge: bool,
) -> Result<Vec<Value>, String> {
    let mut found = Vec::new();
    for kind in [
        PENDING,
        SENDING,
        "compliance-mail:sent",
        "compliance-mail:failed",
        "compliance-mail:uncertain",
        "compliance-mail:dismissed",
    ] {
        for row in state
            .store
            .list_compliance_records(kind)
            .await
            .map_err(|_| "Notification records unavailable")?
        {
            let mail = decode(state, &row)?;
            if mail.recipient.eq_ignore_ascii_case(address) {
                found.push(json!({"id":row.id,"event":row.name,"status":kind.rsplit(':').next(),"at":row.updated_at,"attempts":mail.attempts}));
                if purge {
                    state
                        .store
                        .delete_compliance_record(&row.id)
                        .await
                        .map_err(|_| "Could not erase notification")?;
                }
            }
        }
    }
    Ok(found)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Resolution {
    action: String,
    #[serde(default)]
    accept_duplicate_risk: bool,
}
async fn resolve(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path(id): Path<String>,
    crate::admin_ctx::AdminJson(body): crate::admin_ctx::AdminJson<Resolution>,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(C::SettingsWrite)?;
    let old = state
        .store
        .get_compliance_record(&id)
        .await
        .map_err(AdminApiError::internal)?
        .ok_or_else(|| AdminApiError::not_found("Notification not found"))?;
    if !matches!(
        old.kind.as_str(),
        "compliance-mail:failed" | "compliance-mail:uncertain"
    ) {
        return Err(AdminApiError::conflict(
            "notReviewable",
            "Only failed or uncertain deliveries can be resolved.",
        ));
    }
    if !matches!(body.action.as_str(), "retry" | "dismiss") {
        return Err(AdminApiError::invalid(
            "invalidAction",
            "Choose retry or dismiss.",
            "/action",
        ));
    }
    if body.action == "retry"
        && old.kind == "compliance-mail:uncertain"
        && !body.accept_duplicate_risk
    {
        return Err(AdminApiError::invalid(
            "confirmationRequired",
            "Retry can deliver a duplicate; explicit confirmation is required.",
            "/acceptDuplicateRisk",
        ));
    }
    let mut mail = decode(&state, &old).map_err(|_| {
        AdminApiError::conflict(
            "unreadable",
            "Restore the notification encryption key before resolving.",
        )
    })?;
    let mut next = old.clone();
    next.updated_at = now();
    if body.action == "retry" {
        next.kind = PENDING.into();
        mail.attempts = 0;
        mail.error.clear();
    } else {
        next.kind = "compliance-mail:dismissed".into();
    }
    next.body_json = encode(&state, &id, &mail).map_err(|_| {
        AdminApiError::conflict("unreadable", "Notification encryption unavailable.")
    })?;
    if !state
        .store
        .transition_compliance_record(&old, &next)
        .await
        .map_err(AdminApiError::internal)?
    {
        return Err(AdminApiError::conflict(
            "stale",
            "Delivery changed; reload before resolving.",
        ));
    }
    crate::admin_accounts::audit(
        &state,
        &ctx,
        "compliance-mail-resolved",
        &id,
        None,
        Some(json!({"action":body.action,"duplicateRiskAccepted":body.accept_duplicate_risk})),
    )
    .await;
    Ok(Json(json!({"status":next.kind.rsplit(':').next()})).into_response())
}
