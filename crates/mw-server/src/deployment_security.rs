//! Read-only deployment assessment from current runtime and policy authority.
//! No port scan, upstream request, credential material or environment dump.
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx},
    admin_router::AdminRegistrar,
    capabilities::Capability,
};
use axum::{
    Json,
    extract::State,
    http::{Extensions, HeaderMap, Method, header},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::{Value, json};
#[derive(Serialize)]
pub(crate) struct Check {
    pub id: &'static str,
    pub status: &'static str,
    pub value: Value,
    pub settings: Vec<&'static str>,
}
fn row(id: &'static str, status: &'static str, value: Value, settings: &[&'static str]) -> Check {
    Check {
        id,
        status,
        value,
        settings: settings.to_vec(),
    }
}
pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/security/deployment",
        Capability::DiagnosticsRead,
        assessment,
    );
}
async fn value(state: &AppState, key: &str) -> Result<Value, AdminApiError> {
    crate::settings_registry::effective_for(&state.store, key, &Default::default())
        .await?
        .map(|v| v.value)
        .ok_or_else(|| AdminApiError::internal("missing deployment policy"))
}
async fn assessment(
    State(state): State<AppState>,
    ctx: AdminCtx,
    headers: HeaderMap,
    ext: Extensions,
) -> Result<Response, AdminApiError> {
    ctx.require_unscoped(Capability::DiagnosticsRead)?;
    let checks = checks(&state, &headers, &ext).await?;
    Ok(([(header::CACHE_CONTROL,"private, no-store")],Json(json!({"generatedAt":chrono::Utc::now().to_rfc3339(),"scope":"current-process-and-request","checks":checks,"limitations":["no-external-tls-or-firewall-probe","no-upstream-authentication","configured-worker-not-executed","not-a-security-certification"]}))).into_response())
}

/// The deployment checks, also run by the server diagnostics suite.
pub(crate) async fn checks(
    state: &AppState,
    headers: &HeaderMap,
    ext: &Extensions,
) -> Result<Vec<Check>, AdminApiError> {
    let https = state.external.is_https(headers, ext);

    let secure = state
        .external
        .cookie_secure(state.cookie_secure, headers, ext);
    let (idle, absolute) = crate::admin_accounts::session_timeouts(state).await?;
    let cutoff = (chrono::Utc::now() - chrono::Duration::days(90)).to_rfc3339();
    let posture = state.store.security_posture_counts(&cutoff).await?;
    let admin = crate::signin_guard::policy(state, mw_store::AuthSurface::Admin).await?;
    let mailbox = crate::signin_guard::policy(state, mw_store::AuthSurface::Webmail).await?;
    let cooldown = value(state, "signin.retryCooldownSecs").await?;
    let geo_scope = value(state, "security.signin.geoScope").await?;
    let geo_sessions = value(state, "security.signin.geoEnforceSessions").await?;
    let geo_ready = !matches!(
        mw_engine::security::country_lookup(None),
        mw_engine::security::CountryLookup::DatabaseUnavailable
    );
    let limits = crate::tls::inbound_limits::InboundLimits::from_env()
        .map_err(|_| AdminApiError::internal("invalid startup connection limits"))?;
    let http = crate::inbound_http::HttpLimits::from_env()
        .map_err(|_| AdminApiError::internal("invalid startup HTTP limits"))?;
    Ok(vec![
        row(
            "httpAdmission",
            "info",
            json!({
                "connections":http.connections,"requests":http.requests,
                "headerBytes":http.header_bytes,"headers":http.headers,
                "headerTimeoutMs":http.header_timeout.as_millis(),"bodyTimeoutMs":http.body_timeout.as_millis(),
                "http2Streams":http.http2_streams,"shutdownTimeoutMs":http.shutdown_timeout.as_millis(),
                "requiresRestart":true,"adaptive":false,
                "source":"built-in-listener-startup-configuration",
            }),
            &[],
        ),
        row(
            "connectionNegotiation",
            "info",
            json!({
                "tlsTimeoutMs": limits.tls_timeout.as_millis(),
                "tlsMaximum": limits.tls_handshakes,
                "proxyTimeoutMs": limits.proxy_timeout.as_millis(),
                "proxyMaximum": limits.proxy_handshakes,
                "requiresRestart": true,
                "source": "built-in-listener-startup-configuration",
            }),
            &[],
        ),
        row(
            "https",
            if https { "pass" } else { "warning" },
            json!({"effectiveRequestHttps":https,"configuredHttps":state.external.declares_https()}),
            &[],
        ),
        row(
            "secureCookies",
            if secure { "pass" } else { "warning" },
            json!(secure),
            &[],
        ),
        row(
            "hsts",
            if state.external.hsts(headers, ext).is_some() {
                "pass"
            } else {
                "info"
            },
            json!(state.external.hsts(headers, ext).is_some()),
            &[],
        ),
        row(
            "csrf",
            if state.hardening.csrf_strict {
                "pass"
            } else {
                "warning"
            },
            json!(state.hardening.csrf_strict),
            &[],
        ),
        row(
            "adminSessions",
            "info",
            json!({"idleSeconds":idle,"absoluteSeconds":absolute}),
            &["admin.session.idleSecs", "admin.session.absoluteSecs"],
        ),
        row(
            "adminFactors",
            if posture["adminsWithoutFactor"].as_i64().unwrap_or(1) == 0 {
                "pass"
            } else {
                "warning"
            },
            posture["adminsWithoutFactor"].clone(),
            &[],
        ),
        row(
            "mailboxFactors",
            "info",
            json!({"enrolled":posture["accountsWithFactor"],"knownAccounts":posture["knownAccounts"]}),
            &["twofa.required"],
        ),
        row(
            "signinPacing",
            if cooldown.as_u64().unwrap_or(0) > 0 {
                "pass"
            } else {
                "warning"
            },
            cooldown,
            &["signin.retryCooldownSecs"],
        ),
        row(
            "credentialLocks",
            "pass",
            json!({"administratorLimit":admin.threshold,"mailboxLimit":mailbox.threshold,"windowSeconds":admin.window_secs,"lockSeconds":admin.lock_secs}),
            &[
                "security.signin.adminFailureLimit",
                "security.signin.mailboxFailureLimit",
                "security.signin.failureWindowSecs",
                "security.signin.lockSecs",
            ],
        ),
        row(
            "geography",
            if geo_scope != "off" && !geo_ready {
                "warning"
            } else {
                "info"
            },
            json!({"scope":geo_scope,"existingRequests":geo_sessions,"localDatabaseReady":geo_ready}),
            &[
                "security.signin.geoScope",
                "security.signin.geoEnforceSessions",
            ],
        ),
        row(
            "renderIsolation",
            if state.render_jail_required && state.render_bin.is_none() {
                "warning"
            } else {
                "info"
            },
            json!({"jailRequired":state.render_jail_required,"workerConfigured":state.render_bin.is_some()}),
            &[],
        ),
    ])
}
