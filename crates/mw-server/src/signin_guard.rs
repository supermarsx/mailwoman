//! Deployment-wide temporary credential-failure locks, separate from retry pacing.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx},
    admin_router::AdminRegistrar,
    capabilities::Capability,
};
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
};
use mw_store::{AuthSurface, SigninEvent, auth_failures::FailurePolicy};
use serde_json::json;
use std::net::IpAddr;

pub(crate) fn credential_rejection(reason: &str) -> bool {
    matches!(
        reason,
        "auth-rejected" | "credential-rejected" | "factor_refused"
    )
}
fn unavailable() -> AdminApiError {
    AdminApiError::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "signinGuardUnavailable",
        "Sign-in protection is temporarily unavailable",
    )
}
async fn policy(state: &AppState, surface: AuthSurface) -> Result<FailurePolicy, AdminApiError> {
    async fn number(state: &AppState, key: &str, default: u64) -> Result<u64, AdminApiError> {
        let value = crate::settings_registry::effective_for(&state.store, key, &Default::default())
            .await
            .map_err(|_| unavailable())?;
        match value {
            Some(v) => v.value.as_u64().ok_or_else(unavailable),
            None => Ok(default),
        }
    }
    let (key, default) = match surface {
        AuthSurface::Admin => ("security.signin.adminFailureLimit", 5),
        AuthSurface::Webmail => ("security.signin.mailboxFailureLimit", 10),
    };
    Ok(FailurePolicy {
        threshold: number(state, key, default).await?.clamp(1, 100) as usize,
        window_secs: number(state, "security.signin.failureWindowSecs", 900)
            .await?
            .clamp(30, 86400) as i64,
        lock_secs: number(state, "security.signin.lockSecs", 900)
            .await?
            .clamp(30, 86400) as i64,
    })
}
pub(crate) async fn check(
    state: &AppState,
    surface: AuthSurface,
    source: Option<IpAddr>,
) -> Result<(), AdminApiError> {
    let Some(ip) = source else { return Ok(()) };
    // Resolve configured policy before upstream authentication, even when no lock exists.
    let _ = policy(state, surface).await?;
    let now = chrono::Utc::now().timestamp();
    if let Some(lock) = state
        .store
        .auth_source_lock(surface.as_str(), ip, now)
        .await
        .map_err(|_| unavailable())?
    {
        return Err(AdminApiError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "signinLocked",
            "Too many rejected credentials; wait before trying again",
        )
        .with("retryAfterSecs", (lock.locked_until - now).clamp(1, 86400)));
    }
    Ok(())
}
pub(crate) async fn observe(state: &AppState, event: &SigninEvent, surface: AuthSurface) {
    let Ok(ip) = event.ip.parse::<IpAddr>() else {
        return;
    };
    let result = if event.result == "success" {
        state
            .store
            .clear_auth_source_failures(surface.as_str(), ip, None)
            .await
            .map(|_| ())
    } else if event.result == "failure" && credential_rejection(&event.reason) {
        let Ok(policy) = policy(state, surface).await else {
            warn_unavailable();
            return;
        };
        state
            .store
            .record_auth_source_failure(surface.as_str(), ip, event.ts, policy)
            .await
            .map(|_| ())
    } else {
        return;
    };
    if result.is_err() {
        warn_unavailable()
    }
}
fn warn_unavailable() {
    use std::sync::atomic::{AtomicI64, Ordering};
    static LAST: AtomicI64 = AtomicI64::new(0);
    let now = chrono::Utc::now().timestamp();
    let last = LAST.load(Ordering::Relaxed);
    if now.saturating_sub(last) >= 60
        && LAST
            .compare_exchange(last, now, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
    {
        tracing::warn!("temporary sign-in lock state unavailable");
    }
}
pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(Method::GET, "/bans/automatic", Capability::BansRead, list);
    r.admin_endpoint(
        Method::DELETE,
        "/bans/automatic/{surface}/{ip}",
        Capability::BansWrite,
        unlock,
    );
}
async fn list(State(state): State<AppState>, ctx: AdminCtx) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::BansRead)?;
    let locks = state
        .store
        .list_auth_source_locks(chrono::Utc::now().timestamp())
        .await
        .map_err(|_| unavailable())?;
    Ok((
        [(axum::http::header::CACHE_CONTROL, "private, no-store")],
        Json(json!({"locks":locks})),
    )
        .into_response())
}
async fn unlock(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Path((surface, ip)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<StatusCode, AdminApiError> {
    ctx.require_unscoped(Capability::BansWrite)?;
    crate::step_up::require_admin(&state, &ctx, crate::step_up::StepUpAction::BanManage).await?;
    if !matches!(surface.as_str(), "admin" | "webmail") {
        return Err(AdminApiError::invalid(
            "invalidSurface",
            "Invalid sign-in surface",
            "/surface",
        ));
    }
    let ip: IpAddr = ip
        .parse()
        .map_err(|_| AdminApiError::invalid("invalidIp", "Invalid IP address", "/ip"))?;
    let revision = headers
        .get(axum::http::header::IF_MATCH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix('"'))
        .and_then(|v| v.strip_suffix('"'))
        .filter(|v| !v.is_empty())
        .ok_or_else(|| {
            AdminApiError::new(
                StatusCode::PRECONDITION_REQUIRED,
                "ifMatchRequired",
                "Refresh the lock before unlocking",
            )
        })?;
    if !state
        .store
        .clear_auth_source_failures(&surface, ip, Some(revision))
        .await
        .map_err(|_| unavailable())?
    {
        return Err(AdminApiError::new(
            StatusCode::PRECONDITION_FAILED,
            "conflict",
            "The lock changed; refresh before unlocking",
        ));
    }
    crate::audit::security_event(
        &state,
        crate::audit::SecurityEvent::admin(&ctx.username, "automatic-signin-lock-cleared")
            .detail(json!({"surface":surface,"ip":ip.to_canonical().to_string()})),
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_known_credential_rejections_count() {
        for reason in ["auth-rejected", "credential-rejected", "factor_refused"] {
            assert!(credential_rejection(reason));
        }
        for reason in [
            "challenge-rejected",
            "source-blocked",
            "storage",
            "tls",
            "dns",
            "connect",
            "transport",
            "timeout",
            "account-disabled",
            "session-refused",
            "upstream-policy",
        ] {
            assert!(!credential_rejection(reason));
        }
    }
}
