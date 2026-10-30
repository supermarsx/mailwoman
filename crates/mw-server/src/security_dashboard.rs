//! The security centre Overview (t97): one read-only, bounded summary of what the
//! server already records, for the admin console's mini dashboard.
//!
//! `GET {console}/security/dashboard` is open to any signed-in administrator, but
//! each tile is computed only when the caller holds the capability its detail
//! page needs, for every domain (the Overview is a deployment-wide view):
//!
//! | tile         | capability          | detail page                 |
//! |--------------|---------------------|-----------------------------|
//! | `factors`    | `security.read`     | Requirements tab            |
//! | `signins`    | `logs.read` or `bans.read` | Authentication logs  |
//! | `protection` | `bans.read`         | bans, locks, protection     |
//! | `admins`     | `admins.read`       | Admin accounts              |
//! | `audit`      | `audit.read`        | Audit log                   |
//! | `posture`    | `diagnostics.read`  | Deployment checks           |
//!
//! A tile the caller may not see is left out of the response (not sent empty),
//! and a caller who may see none gets `403`. Source addresses are shown in full
//! only to holders of `logs.read`, who can already read them in the
//! authentication history; anyone else gets the network prefix (IPv4 /24,
//! IPv6 /48). No credential, token or session identifier is returned.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx},
    admin_router::{Access, AdminRegistrar},
    capabilities::Capability,
};
use axum::{
    Json,
    extract::State,
    http::{Extensions, HeaderMap, Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::net::IpAddr;

pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/security/dashboard",
        Access::SignedIn,
        dashboard,
    );
}

/// Which tiles a caller may see.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Tiles {
    pub factors: bool,
    pub signins: bool,
    pub protection: bool,
    pub admins: bool,
    pub audit: bool,
    pub posture: bool,
    pub console: bool,
    pub full_ips: bool,
}
impl Tiles {
    pub(crate) fn of(ctx: &AdminCtx) -> Self {
        let has = |cap| ctx.has_unscoped(cap);
        Self {
            factors: has(Capability::SecurityRead),
            signins: has(Capability::LogsRead) || has(Capability::BansRead),
            protection: has(Capability::BansRead),
            admins: has(Capability::AdminsRead),
            audit: has(Capability::AuditRead),
            posture: has(Capability::DiagnosticsRead),
            console: has(Capability::SecurityRead),
            full_ips: has(Capability::LogsRead),
        }
    }
    fn any(self) -> bool {
        self.factors || self.signins || self.protection || self.admins || self.audit || self.posture
    }
}

/// The network prefix of an address: `203.0.113.0/24`, `2001:db8:1::/48`.
/// Anything that does not parse is withheld.
pub(crate) fn mask_ip(raw: &str) -> String {
    match raw.trim().parse::<IpAddr>() {
        Ok(IpAddr::V4(v4)) => {
            let o = v4.octets();
            format!("{}.{}.{}.0/24", o[0], o[1], o[2])
        }
        Ok(IpAddr::V6(v6)) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return mask_ip(&v4.to_string());
            }
            let s = v6.segments();
            format!("{:x}:{:x}:{:x}::/48", s[0], s[1], s[2])
        }
        Err(_) => String::new(),
    }
}
fn shown_ip(tiles: Tiles, raw: &str) -> String {
    if tiles.full_ips {
        raw.to_string()
    } else {
        mask_ip(raw)
    }
}
fn domain_of(address: &str) -> Option<String> {
    address
        .rsplit_once('@')
        .map(|(_, d)| d.trim().to_ascii_lowercase())
        .filter(|d| !d.is_empty())
}

async fn dashboard(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    ext: Extensions,
) -> Result<Response, AdminApiError> {
    let tiles = Tiles::of(&ctx);
    if !tiles.any() {
        return Err(AdminApiError::new(
            StatusCode::FORBIDDEN,
            "forbidden",
            "No security overview tile is available to this administrator.",
        ));
    }
    let now = chrono::Utc::now().timestamp();
    let mut body = json!({ "generatedAt": now });
    if tiles.factors {
        body["factors"] = factors(&state).await?;
    }
    if tiles.signins {
        body["signins"] = signins(&state, tiles, now).await?;
    }
    if tiles.protection {
        body["protection"] = protection(&state, now).await?;
    }
    if tiles.admins {
        body["admins"] = admins(&state, tiles, now).await?;
    }
    if tiles.audit {
        body["audit"] = audit(&state).await?;
    }
    if tiles.posture {
        body["posture"] = posture(&state, tiles, &headers, &ext).await?;
    }
    Ok(([(header::CACHE_CONTROL, "private, no-store")], Json(body)).into_response())
}

/// 2FA adoption: who has a factor, by kind, and who lacks one while a policy
/// requires it (the global or domain row of the Requirements tab, or the
/// `twofa.required` setting at the global or domain level).
async fn factors(state: &AppState) -> Result<Value, AdminApiError> {
    let accounts = state.store.security_factor_accounts().await?;
    let policies = state.store.list_twofa_policies().await?;
    let global_row = policies
        .iter()
        .any(|p| p.scope_kind == "global" && p.require_2fa);
    let domain_rows: BTreeSet<String> = policies
        .iter()
        .filter(|p| p.scope_kind == "domain" && p.require_2fa)
        .map(|p| p.scope_value.to_ascii_lowercase())
        .collect();
    // `twofa.required` resolved once per domain (bounded), not per account.
    let mut by_domain: BTreeMap<String, bool> = BTreeMap::new();
    let setting = |domain: String| {
        let store = state.store.clone();
        async move {
            let subject = mw_store::settings_policy::SettingSubject {
                personal_owner: None,
                domain: &domain,
                account_id: "",
                address: "",
            };
            crate::settings_registry::effective_for(&store, "twofa.required", &subject)
                .await
                .ok()
                .flatten()
                .is_some_and(|v| v.value.as_bool() == Some(true))
        }
    };
    let global_setting = setting(String::new()).await;
    let (mut totp, mut passkey, mut both, mut any, mut unprotected_required) = (0, 0, 0, 0, 0);
    for account in &accounts {
        if account.totp {
            totp += 1;
        }
        if account.passkey {
            passkey += 1;
        }
        if account.totp && account.passkey {
            both += 1;
        }
        if account.totp || account.passkey {
            any += 1;
            continue;
        }
        let domain = account.address.as_deref().and_then(domain_of);
        let required = global_row
            || global_setting
            || match &domain {
                Some(d) if domain_rows.contains(d) => true,
                Some(d) => match by_domain.get(d) {
                    Some(v) => *v,
                    None if by_domain.len() < 2000 => {
                        let v = setting(d.clone()).await;
                        by_domain.insert(d.clone(), v);
                        v
                    }
                    None => false,
                },
                None => false,
            };
        if required {
            unprotected_required += 1;
        }
    }
    let cutoff = (chrono::Utc::now() - chrono::Duration::days(90)).to_rfc3339();
    let posture = state.store.security_posture_counts(&cutoff).await?;
    Ok(json!({
        "adminsWithoutFactor": posture["adminsWithoutFactor"],
        "staleApiKeys": posture["staleApiKeys"],
        "environmentAdminLoginEnabled": state.v6.admin_username.is_some()
            && std::env::var("MW_ADMIN_ENV_LOGIN").as_deref() != Ok("off"),
        "knownAccounts": accounts.len(),
        "withFactor": any,
        "totp": totp,
        "passkey": passkey,
        "both": both,
        "withoutFactorRequired": unprotected_required,
        "requiredGlobally": global_row || global_setting,
        "requiredDomains": domain_rows.len(),
    }))
}

/// Failed and blocked sign-ins, 24 h (hourly) and 7 d (daily), with the top
/// sources and countries of the last 7 days.
async fn signins(state: &AppState, tiles: Tiles, now: i64) -> Result<Value, AdminApiError> {
    let day = now - 86_400;
    let week = now - 7 * 86_400;
    let hourly = fill(&buckets(state, day, 3600).await?, day, 3600, 24);
    let daily = fill(&buckets(state, week, 86_400).await?, week, 86_400, 7);
    let total = |rows: &[Value], key: &str| {
        rows.iter()
            .map(|r| r[key].as_i64().unwrap_or(0))
            .sum::<i64>()
    };
    let sources: Vec<Value> = state
        .store
        .security_signin_sources(week, 5)
        .await?
        .into_iter()
        .map(|s| json!({"source": shown_ip(tiles, &s.ip), "country": s.country, "count": s.count}))
        .collect();
    let countries: Vec<Value> = state
        .store
        .security_signin_countries(week, 5)
        .await?
        .into_iter()
        .map(|(country, count)| json!({"country": country, "count": count}))
        .collect();
    let geo_ready = !matches!(
        mw_engine::security::country_lookup(None),
        mw_engine::security::CountryLookup::DatabaseUnavailable
    );
    Ok(json!({
        "failed24h": total(&hourly, "failed"),
        "blocked24h": total(&hourly, "blocked"),
        "failed7d": total(&daily, "failed"),
        "blocked7d": total(&daily, "blocked"),
        "hourly": hourly,
        "daily": daily,
        "topSources": sources,
        "topCountries": countries,
        "sourcesMasked": !tiles.full_ips,
        "geoipReady": geo_ready,
    }))
}

async fn buckets(
    state: &AppState,
    since: i64,
    width: i64,
) -> Result<Vec<(i64, i64, i64)>, AdminApiError> {
    Ok(state
        .store
        .security_signin_buckets(since, width, None)
        .await?
        .into_iter()
        .map(|b| (b.t, b.failed, b.blocked))
        .collect())
}
/// `count` buckets of `width` seconds ending now, zero-filled, oldest first.
fn fill(rows: &[(i64, i64, i64)], since: i64, width: i64, count: i64) -> Vec<Value> {
    let start = since.div_euclid(width) * width + width;
    (0..count)
        .map(|i| {
            let t = start + i * width;
            let found = rows.iter().find(|r| r.0 == t);
            json!({
                "t": t,
                "failed": found.map_or(0, |r| r.1),
                "blocked": found.map_or(0, |r| r.2),
            })
        })
        .collect()
}
/// Request protection actions, manual bans and automatic sign-in locks.
async fn protection(state: &AppState, now: i64) -> Result<Value, AdminApiError> {
    let (enforced, observed) = state
        .store
        .security_protection_event_counts(now - 86_400)
        .await?;
    let locks = state.store.list_auth_source_locks(now).await?;
    let admin_locks = locks.iter().filter(|l| l.surface == "admin").count();
    let bans = state
        .v6
        .admin
        .list_bans()
        .await
        .map_err(AdminApiError::internal)?;
    let now_rfc = chrono::Utc::now();
    let active = bans
        .iter()
        .filter(|b| {
            b.expires_at.as_deref().is_none_or(|e| {
                chrono::DateTime::parse_from_rfc3339(e).map_or(true, |at| at > now_rfc)
            })
        })
        .count();
    Ok(json!({
        "protectionEnforced24h": enforced,
        "protectionObserved24h": observed,
        "activeLocks": locks.len(),
        "activeAdminLocks": admin_locks,
        "activeBans": active,
    }))
}

/// Live administrator sessions and the latest administrator sign-ins.
async fn admins(state: &AppState, tiles: Tiles, now: i64) -> Result<Value, AdminApiError> {
    let sessions = state.store.list_admin_sessions(now).await?;
    let signed_in: BTreeSet<&str> = sessions.iter().map(|s| s.admin_id.as_str()).collect();
    let recent: Vec<Value> = state
        .store
        .security_recent_admin_signins(5)
        .await?
        .into_iter()
        .map(|(user, at, ip)| json!({"username": user, "at": at, "source": shown_ip(tiles, &ip)}))
        .collect();
    Ok(json!({
        "activeSessions": sessions.len(),
        "adminsSignedIn": signed_in.len(),
        "recentSignins": recent,
        "sourcesMasked": !tiles.full_ips,
    }))
}

/// The latest security audit rows (policy changes, refused step-ups, refused
/// passkeys, console-path changes, security settings).
async fn audit(state: &AppState) -> Result<Value, AdminApiError> {
    let rows: Vec<Value> = state
        .store
        .security_recent_audit(8)
        .await?
        .into_iter()
        .map(|r| json!({"id": r.id, "at": r.ts, "actor": r.actor, "action": r.action, "target": r.target}))
        .collect();
    Ok(json!({ "recent": rows }))
}

/// The deployment checks the Deployment tab and diagnostics run, plus GeoIP, the
/// analytics tag and whether the console still sits at the default path.
async fn posture(
    state: &AppState,
    tiles: Tiles,
    headers: &HeaderMap,
    ext: &Extensions,
) -> Result<Value, AdminApiError> {
    let checks = crate::deployment_security::checks(state, headers, ext).await?;
    let warnings = checks.iter().filter(|c| c.status == "warning").count();
    let pick = |id: &str| {
        checks
            .iter()
            .find(|c| c.id == id)
            .map_or(Value::Null, |c| json!(c.status))
    };
    let analytics = crate::analytics_tag::load(&state.store)
        .await
        .map(|c| c.enabled)
        .ok();
    let geo_ready = !matches!(
        mw_engine::security::country_lookup(None),
        mw_engine::security::CountryLookup::DatabaseUnavailable
    );
    let mut out = json!({
        "checks": checks.len(),
        "warnings": warnings,
        "https": pick("https"),
        "hsts": pick("hsts"),
        "secureCookies": pick("secureCookies"),
        "csrf": pick("csrf"),
        "adminFactors": pick("adminFactors"),
        "geoipReady": geo_ready,
        "analyticsTag": analytics,
    });
    if tiles.console {
        out["consoleDefaultPath"] = json!(
            mw_admin::config::admin_path()
                .eq_ignore_ascii_case(mw_admin::config::DEFAULT_ADMIN_PATH)
        );
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn masks_keep_only_the_network_prefix() {
        assert_eq!(mask_ip("203.0.113.77"), "203.0.113.0/24");
        assert_eq!(mask_ip("2001:db8:1:2::5"), "2001:db8:1::/48");
        assert_eq!(mask_ip("::ffff:198.51.100.9"), "198.51.100.0/24");
        assert_eq!(mask_ip("not an address"), "");
    }
}
