//! Session security warnings: what the user and the administrator are told.
//!
//! One [`raise`] call per finding (country change, impossible travel, concurrent
//! use, client mismatch, rotated-id reuse):
//!
//! * the **audit log** always gets a `session-security-alert` security event
//!   (reason, country pair, the session's short id, action taken; never the
//!   cookie);
//! * the **administrator readout** keeps the last 50 alerts instance-wide;
//! * for a webmail account the **in-app notice** is stored (at most 20 per
//!   account, kept `security.alerts.noticeRetentionDays`) and shown on the next
//!   load and under Settings, Security, Sessions;
//! * the **email** is a message delivered into the user's own mailbox by the
//!   same path as the other security alerts ([`crate::alerts`]): the engine
//!   APPEND or the guarded JMAP import. There is no outbound SMTP for these; if
//!   the account has no delivery session the warning stays in-app and in the
//!   audit log.
//!
//! The user chooses the channel (`security.alerts.channels`: app, email, both)
//! and, only if the administrator allows (`countryChangeUserMute`), may mute the
//! country-change alert. The administrator can lock any of these user settings,
//! turn email off for everyone (`emailEnabled`), or disable the country-change
//! alert globally (`countryChangeEnabled`).
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx},
    admin_router::AdminRegistrar,
    audit::{SecurityEvent, security_event},
    capabilities::Capability,
    sec_cfg,
    session_theft::Sensitivity,
};
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use mw_store::settings_policy::{SettingSource, SettingSubject};
use serde::Deserialize;
use serde_json::{Value, json};
pub(crate) mod evidence;

pub(crate) const CAVEAT: &str =
    "Country detection may be inaccurate; VPNs and mobile networks can cause false changes.";
const NOTICE_PREFIX: &str = "sec-notice:v1:";
const RECENT_KEY: &str = "sec-alerts:v1:recent";
const MAX_NOTICES: usize = 20;
const MAX_RECENT: usize = 50;

#[derive(Clone, Debug)]
pub(crate) struct Alert {
    pub kind: &'static str,
    pub reason: String,
    pub from: Option<String>,
    pub to: Option<String>,
    /// "alerted", "step-up" or "revoked".
    pub action: &'static str,
    /// The session's short id (`mw_store::session_handle`), never the cookie.
    pub session: String,
    pub ts: i64,
    pub admin: bool,
    pub account: String,
    pub context: Option<evidence::Evidence>,
}
impl Alert {
    pub(crate) fn geo(&self) -> bool {
        matches!(
            self.kind,
            "countryChange" | "impossibleTravel" | "concurrentUse" | "countryUnavailable"
        )
    }
    fn json(&self) -> Value {
        json!({
            "ts": self.ts, "kind": self.kind, "reason": self.reason,
            "fromCountry": self.from, "toCountry": self.to, "action": self.action,
            "session": self.session, "surface": if self.admin {"admin"} else {"webmail"},
            "context": self.context,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Channels {
    App,
    Email,
    Both,
}
#[derive(Clone, Debug)]
pub(crate) struct UserPrefs {
    pub country_alert: bool,
    pub channels: Channels,
    pub sensitivity: Sensitivity,
    pub email_allowed: bool,
}
impl Default for UserPrefs {
    fn default() -> Self {
        Self {
            country_alert: true,
            channels: Channels::Both,
            sensitivity: Sensitivity::Normal,
            email_allowed: true,
        }
    }
}

/// The effective alert preferences of an account: the administrator's global
/// switches, the user's own choices where allowed, locks respected.
pub(crate) async fn user_prefs(state: &AppState, account: &str) -> UserPrefs {
    let globals = sec_cfg::values(&state.store, "security.alerts.").await;
    let address = state
        .store
        .security_account_address(account)
        .await
        .ok()
        .flatten()
        .unwrap_or_default();
    let domain = address.rsplit_once('@').map_or("", |(_, d)| d);
    let subject = SettingSubject {
        personal_owner: None,
        domain,
        account_id: account,
        address: &address,
    };
    let get = |key: &'static str| {
        let subject = &subject;
        async move {
            crate::settings_registry::effective_for(&state.store, key, subject)
                .await
                .ok()
                .flatten()
        }
    };
    let mut prefs = UserPrefs {
        email_allowed: globals.flag("security.alerts.emailEnabled"),
        ..UserPrefs::default()
    };
    if let Some(e) = get("security.alerts.countryChange").await {
        let value = e.value.as_bool().unwrap_or(true);
        // A user's own "off" counts only where the administrator allows muting.
        prefs.country_alert = value
            || !(matches!(e.source, SettingSource::User)
                && !globals.flag("security.alerts.countryChangeUserMute"));
    }
    if let Some(e) = get("security.alerts.channels").await {
        prefs.channels = match e.value.as_str() {
            Some("app") => Channels::App,
            Some("email") => Channels::Email,
            _ => Channels::Both,
        };
    }
    if let Some(e) = get("security.alerts.sensitivity").await {
        prefs.sensitivity = Sensitivity::parse(e.value.as_str().unwrap_or("normal"));
    }
    prefs
}

pub(crate) async fn raise(state: &AppState, alert: Alert, prefs: &UserPrefs) {
    let detail = alert.json();
    let event = if alert.admin {
        SecurityEvent::admin(&alert.account, "session-security-alert")
    } else {
        SecurityEvent::user(&alert.account, "session-security-alert")
    };
    security_event(state, event.detail(detail.clone())).await;
    tracing::warn!(
        "session security: {} ({}) {:?}->{:?} session {} action {}",
        alert.kind,
        alert.reason,
        alert.from,
        alert.to,
        alert.session,
        alert.action
    );
    push_recent(state, &alert).await;
    if alert.admin {
        return;
    }
    let owner = match alert
        .context
        .as_ref()
        .and_then(|context| context.subject.clone())
    {
        Some(owner) => Some(owner),
        None => crate::notification_center::account_owner(state, &alert.account).await,
    };
    if let Some(owner) = &owner {
        crate::notification_center::emit(
            state,
            owner,
            "session",
            alert.kind,
            if alert.action == "revoked" {
                "critical"
            } else {
                "warning"
            },
            alert.context.clone(),
        )
        .await;
    }
    if matches!(prefs.channels, Channels::App | Channels::Both) {
        if let Some(owner) = &owner {
            add_notice(state, &alert, owner).await;
        }
    }
    if matches!(prefs.channels, Channels::Email | Channels::Both) && prefs.email_allowed {
        let state = state.clone();
        tokio::spawn(async move {
            let (subject, body, guidance) = email_text(&state, &alert).await;
            if let Err(error) = crate::alerts::deliver_text(
                &state,
                &alert.account,
                if subject.starts_with("Aviso") {
                    "pt-PT"
                } else {
                    "en"
                },
                &subject,
                &body,
                &guidance,
            )
            .await
            {
                tracing::warn!("session security email not delivered; in-app notice kept: {error}");
            }
        });
    }
}

async fn push_recent(state: &AppState, alert: &Alert) {
    let mut ring: Vec<Value> = state
        .store
        .get_setting(RECENT_KEY)
        .await
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default();
    let mut row = alert.json();
    row["account"] = json!(alert.account);
    ring.push(row);
    if ring.len() > MAX_RECENT {
        let drop = ring.len() - MAX_RECENT;
        ring.drain(..drop);
    }
    if let Ok(raw) = serde_json::to_string(&ring) {
        let _ = state.store.set_setting(RECENT_KEY, &raw).await;
    }
}

fn account_key(account: &str) -> String {
    crate::push_relay::hash_token(account)[..24].to_owned()
}

async fn add_notice(state: &AppState, alert: &Alert, owner: &str) {
    let prefix = format!("{NOTICE_PREFIX}{}:", account_key(owner));
    let id = format!("{:012}-{:04x}", alert.ts, rand::random::<u16>());
    let mut row = alert.json();
    row["id"] = json!(id);
    row["acknowledged"] = json!(false);
    row["caveat"] = json!(alert.geo());
    let Ok(raw) = serde_json::to_string(&row) else {
        return;
    };
    if state
        .store
        .put_personal_setting(owner, &format!("{prefix}{id}"), &raw)
        .await
        .is_err()
    {
        return;
    }
    let retention = sec_cfg::values(&state.store, "security.alerts.")
        .await
        .num("security.alerts.noticeRetentionDays")
        .clamp(1, 365) as i64;
    let oldest_kept = alert.ts - retention * 86_400;
    if let Ok(rows) = state.store.personal_records(&owner, &prefix).await {
        let excess = rows.len().saturating_sub(MAX_NOTICES);
        for (index, (key, value)) in rows.iter().enumerate() {
            let ts = serde_json::from_str::<Value>(value)
                .ok()
                .and_then(|v| v["ts"].as_i64())
                .unwrap_or(0);
            if index < excess || ts < oldest_kept {
                let _ = state.store.delete_personal_setting(owner, key).await;
            }
        }
    }
}

async fn email_text(state: &AppState, alert: &Alert) -> (String, String, String) {
    let language = state
        .store
        .security_account_address(&alert.account)
        .await
        .ok()
        .flatten()
        .map(|address| {
            let domain = address.rsplit_once('@').map_or("", |(_, d)| d).to_owned();
            (address, domain)
        });
    let ui = match &language {
        Some((address, domain)) => {
            let subject = SettingSubject {
                personal_owner: None,
                domain,
                account_id: &alert.account,
                address,
            };
            crate::settings_registry::effective_for(&state.store, "ui.language", &subject)
                .await
                .ok()
                .flatten()
                .and_then(|e| e.value.as_str().map(str::to_owned))
        }
        None => None,
    };
    compose(
        ui.as_deref()
            .is_some_and(|l| l.eq_ignore_ascii_case("pt-PT")),
        alert,
    )
}

/// The email text. English, and European Portuguese for `pt-PT` accounts.
pub(crate) fn compose(pt: bool, a: &Alert) -> (String, String, String) {
    let (subject, mut body, guidance) = compose_summary(pt, a);
    body.push_str(&format!(
        "\n\n{}: {}\n",
        if pt {
            "Data e hora (UTC)"
        } else {
            "Date and time (UTC)"
        },
        chrono::DateTime::from_timestamp(a.ts, 0)
            .map(|date| date.to_rfc3339())
            .unwrap_or_default()
    ));
    if let Some(context) = &a.context {
        body.push_str(&context.text(pt));
    }
    (subject, body, guidance)
}
fn compose_summary(pt: bool, a: &Alert) -> (String, String, String) {
    let from = a.from.as_deref().unwrap_or("?");
    let to = a.to.as_deref().unwrap_or("?");
    let caveat_en = if a.geo() {
        format!(" {CAVEAT}")
    } else {
        String::new()
    };
    if pt {
        let what = match a.kind {
            "countryChange" => format!("A sua sessão foi usada a partir de um país novo ({to}), depois de {from}."),
            "impossibleTravel" => format!("A sua sessão foi usada a partir de {to} pouco depois de {from}."),
            "concurrentUse" => format!("A sua sessão foi usada em simultâneo a partir de países diferentes ({from} e {to})."),
            "countryUnavailable" => "Não foi possível determinar o país de um pedido da sua sessão.".to_owned(),
            "fingerprintMismatch" => "A sua sessão foi usada a partir de um navegador ou dispositivo diferente daquele com que foi iniciada.".to_owned(),
            _ => "Uma credencial de sessão anterior foi apresentada outra vez, por isso todas as sessões desse início de sessão foram terminadas.".to_owned(),
        };
        let did = match a.action {
            "step-up" => "Foi pedida a confirmação da palavra-passe e do segundo fator.",
            "revoked" => "A sessão foi terminada.",
            _ => "Não foi tomada nenhuma ação.",
        };
        let caveat = if a.geo() {
            " A deteção do país pode ser imprecisa; VPN e redes móveis podem causar mudanças falsas."
        } else {
            ""
        };
        return (
            "Aviso de segurança do Mailwoman".into(),
            format!("{what} {did}"),
            format!(
                "Se não foi você, abra Definições, Segurança, e use «Não fui eu» para terminar todas as sessões; depois altere a palavra-passe. Sessão {}.{caveat}",
                a.session
            ),
        );
    }
    let what = match a.kind {
        "countryChange" => format!("Your signed-in session was used from a new country ({to}), after {from}."),
        "impossibleTravel" => format!("Your session was used from {to} shortly after {from}."),
        "concurrentUse" => format!("Your session was used at the same time from different countries ({from} and {to})."),
        "countryUnavailable" => "The country of a request on your session could not be determined.".to_owned(),
        "fingerprintMismatch" => "Your session was used from a different browser or device than the one it was started on.".to_owned(),
        _ => "A previous session credential was presented again, so every session from that sign-in was ended.".to_owned(),
    };
    let did = match a.action {
        "step-up" => "The session was asked to confirm your password and second factor.",
        "revoked" => "The session was ended.",
        _ => "No action was taken.",
    };
    (
        "Mailwoman security notice".into(),
        format!("{what} {did}"),
        format!(
            "If this was not you, open Settings, Security, and use \"This wasn't me\" to end all sessions, then change your password. Session {}.{caveat_en}",
            a.session
        ),
    )
}

// ─── user routes ────────────────────────────────────────────────────────────

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/account/security/session-alerts", get(list))
        .route("/api/account/security/session-alerts/ack", post(ack))
}

fn no_store(mut r: Response) -> Response {
    r.headers_mut()
        .insert(header::CACHE_CONTROL, "private, no-store".parse().unwrap());
    r
}

async fn list(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    let owner = match crate::prefs_routes::personal_storage::record_owner(
        &state, &session, &headers, false,
    )
    .await
    {
        Ok(owner) => owner,
        Err(response) => return response,
    };
    let prefix = format!("{NOTICE_PREFIX}{}:", account_key(&owner));
    let Ok(rows) = state.store.personal_records(&owner, &prefix).await else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    let notices: Vec<Value> = rows
        .iter()
        .rev()
        .filter_map(|(_, v)| serde_json::from_str(v).ok())
        .collect();
    let unread = notices
        .iter()
        .filter(|n| n["acknowledged"] != json!(true))
        .count();
    no_store(
        Json(json!({
            "notices": notices,
            "unread": unread,
            "currentSession": mw_store::session_handle(&session.id),
            "caveat": CAVEAT,
        }))
        .into_response(),
    )
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Ack {
    id: Option<String>,
    #[serde(default)]
    all: bool,
}
async fn ack(State(state): State<AppState>, headers: HeaderMap, Json(body): Json<Ack>) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    let owner =
        match crate::prefs_routes::personal_storage::record_owner(&state, &session, &headers, true)
            .await
        {
            Ok(owner) => owner,
            Err(response) => return response,
        };
    let prefix = format!("{NOTICE_PREFIX}{}:", account_key(&owner));
    let Ok(rows) = state.store.personal_records(&owner, &prefix).await else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    let mut changed = 0;
    for (key, value) in rows {
        let wanted = body.all
            || body
                .id
                .as_deref()
                .is_some_and(|id| key == format!("{prefix}{id}"));
        if !wanted {
            continue;
        }
        if let Ok(mut row) = serde_json::from_str::<Value>(&value) {
            row["acknowledged"] = json!(true);
            if let Ok(raw) = serde_json::to_string(&row)
                && state
                    .store
                    .put_personal_setting(&owner, &key, &raw)
                    .await
                    .is_ok()
            {
                changed += 1;
            }
        }
    }
    no_store(Json(json!({"updated": changed})).into_response())
}

// ─── admin readout ──────────────────────────────────────────────────────────

pub(crate) fn register_admin(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/security/session-watch",
        Capability::SecurityRead,
        admin_status,
    );
}

async fn admin_status(
    State(state): State<AppState>,
    ctx: AdminCtx,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::SecurityRead)?;
    let spray_cfg = crate::signin_spray::config(&state).await;
    let binding = sec_cfg::values(&state.store, crate::session_binding::PREFIX).await;
    let theft = sec_cfg::values(&state.store, "security.theft.").await;
    let alerts = sec_cfg::values(&state.store, "security.alerts.").await;
    let recent: Vec<Value> = state
        .store
        .get_setting(RECENT_KEY)
        .await
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default();
    Ok((
        [(header::CACHE_CONTROL, "private, no-store")],
        Json(json!({
            "spray": crate::signin_spray::status(&spray_cfg),
            "spike": crate::signin_spike::status(&state).await,
            "sessionBinding": {
                "enabled": binding.flag("security.sessionBinding.enabled"),
                "attributes": binding.list("security.sessionBinding.attributes"),
                "strictness": binding.text("security.sessionBinding.strictness"),
                "webmailAction": binding.text("security.sessionBinding.webmailAction"),
                "adminAction": binding.text("security.sessionBinding.adminAction"),
            },
            "theft": {
                "enabled": theft.flag("security.theft.enabled"),
                "minTravelSecs": theft.num("security.theft.minTravelSecs"),
                "concurrentUse": theft.text("security.theft.concurrentUse"),
                "webmailAction": theft.text("security.theft.webmailAction"),
                "adminAction": theft.text("security.theft.adminAction"),
            },
            "countryChange": {
                "enabled": alerts.flag("security.alerts.countryChangeEnabled"),
                "userMuteAllowed": alerts.flag("security.alerts.countryChangeUserMute"),
                "cooldownSecs": alerts.num("security.alerts.countryChangeCooldownSecs"),
                "emailEnabled": alerts.flag("security.alerts.emailEnabled"),
            },
            "recentAlerts": recent.iter().rev().take(25).collect::<Vec<_>>(),
            "caveat": CAVEAT,
        })),
    )
        .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alert(kind: &'static str) -> Alert {
        Alert {
            kind,
            reason: "x".into(),
            from: Some("PT".into()),
            to: Some("JP".into()),
            action: "step-up",
            session: "0123456789abcdef".into(),
            ts: 1000,
            admin: false,
            account: "acct".into(),
            context: None,
        }
    }

    #[test]
    fn email_text_names_the_countries_the_session_and_the_caveat_but_never_a_cookie() {
        let (subject, body, guidance) = compose(false, &alert("countryChange"));
        assert!(subject.contains("security notice"));
        assert!(body.contains("JP") && body.contains("PT"));
        assert!(guidance.contains("0123456789abcdef"));
        assert!(guidance.contains("inaccurate") && guidance.contains("VPNs"));
        let (_, _, g) = compose(false, &alert("rotationReuse"));
        assert!(
            !g.contains("inaccurate"),
            "the geo caveat belongs to location alerts only"
        );
        let (ps, pb, pg) = compose(true, &alert("impossibleTravel"));
        assert!(ps.starts_with("Aviso") && pb.contains("JP") && pg.contains("imprecisa"));
    }

    #[test]
    fn notices_are_bounded_and_acknowledgeable_in_a_real_store() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let store = mw_store::Store::open_in_memory(mw_store::ServerKey::generate())
                .await
                .unwrap();
            let state_less_prefix = format!("{NOTICE_PREFIX}{}:", account_key("acct"));
            for i in 0..30 {
                store
                    .set_setting(
                        &format!("{state_less_prefix}{:012}-0000", 1000 + i),
                        &json!({"ts":1000+i}).to_string(),
                    )
                    .await
                    .unwrap();
            }
            let rows = store
                .settings_with_prefix(&state_less_prefix)
                .await
                .unwrap();
            assert_eq!(rows.len(), 30);
            // The delete API refuses keys outside the session-security families.
            assert!(store.delete_sec_state("signin.antiBotMode").await.is_err());
            assert!(store.delete_sec_state(&rows[0].0).await.unwrap());
        });
    }
}
