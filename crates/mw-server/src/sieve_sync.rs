//! Authenticated Sieve JMAP policy and operator-selected ManageSieve transport.
use crate::{AppState, domain_config, image_proxy, settings_registry};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use mw_sieve::{Credentials, ManageSieveClient, TlsMode};
use serde_json::{Value, json};

// Kept as an empty registration seam; the old upload REST endpoint is removed.
pub(crate) fn sieve_sync_router() -> Router<AppState> {
    Router::new()
}

pub(crate) async fn context(
    state: &AppState,
    session: &mw_store::Session,
    request: Option<&Value>,
) -> Result<mw_engine::sieve_jmap::SieveContext, Response> {
    let account = state
        .store
        .get_account(&session.account_id)
        .await
        .map_err(|_| upstream("could not resolve Sieve account identity"))?;
    let username = &account.username;
    let domain = crate::upstream_resolve::domain_of(username).unwrap_or_default();
    let subject = mw_store::settings_policy::SettingSubject {
        domain: &domain,
        account_id: &session.account_id,
        address: username,
    };
    async fn setting(
        state: &AppState,
        key: &str,
        subject: &mw_store::settings_policy::SettingSubject<'_>,
        fallback: Value,
    ) -> Result<Value, Response> {
        settings_registry::effective_for(&state.store, key, subject)
            .await
            .map(|v| v.map(|e| e.value).unwrap_or(fallback))
            .map_err(|_| upstream("could not resolve Sieve policy"))
    }
    let policy = mw_sieve::policy::Policy {
        max_script_bytes: Some(
            setting(state, "sieve.maxScriptBytes", &subject, json!(65536))
                .await?
                .as_u64()
                .unwrap_or(65536) as usize,
        ),
        max_scripts: Some(
            setting(state, "sieve.maxScripts", &subject, json!(5))
                .await?
                .as_u64()
                .unwrap_or(5) as usize,
        ),
        max_redirects: Some(
            setting(state, "sieve.maxRedirects", &subject, json!(4))
                .await?
                .as_u64()
                .unwrap_or(4) as usize,
        ),
        raw_edit: setting(state, "cap.sieve.raw_edit", &subject, json!(false))
            .await?
            .as_bool()
            .unwrap_or(false),
        forward_external: setting(state, "cap.sieve.forward_external", &subject, json!(false))
            .await?
            .as_bool()
            .unwrap_or(false),
        vacation_allowed: setting(state, "sieve.vacationAllowed", &subject, json!(false))
            .await?
            .as_bool()
            .unwrap_or(false),
        allowed_extensions: setting(state, "sieve.allowedExtensions", &subject, Value::Null)
            .await?
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(|s| s.to_ascii_lowercase())
                    .collect()
            }),
        own_domains: state
            .store
            .policy_domain_family(&domain)
            .await
            .map_err(|_| upstream("could not resolve Sieve domain policy"))?,
        ..Default::default()
    };
    // Any content or activation write with forwarding authority requires fresh
    // identity proof. Validate/get/query remain available without a proof.
    if policy.forward_external
        && request
            .and_then(|r| r["methodCalls"].as_array())
            .is_some_and(|calls| {
                calls.iter().any(|c| {
                    c[0] == "SieveScript/set"
                        || c[0] == "MailRule/set"
                        || c[0] == "VacationResponse/set"
                })
            })
    {
        crate::step_up::require_user(
            state,
            session,
            crate::step_up::StepUpAction::ExternalForwarding,
        )
        .await?;
    }
    let configured = domain_config::upstream_for(&state.store, username)
        .await
        .map_err(|_| upstream("could not load Sieve configuration"))?
        .and_then(|c| c.sieve)
        .is_some();
    let client = if configured {
        Some(connect_account_sieve(state, &session.account_id, username).await?)
    } else {
        None
    };
    Ok(mw_engine::sieve_jmap::SieveContext::new(policy, client))
}
fn upstream(message: &str) -> Response {
    (StatusCode::BAD_GATEWAY, Json(json!({"error":message}))).into_response()
}
/// The only connection constructor used by account Sieve operations.
/// Both identity and account id must come from the authenticated session.
pub(crate) async fn connect_account_sieve(
    state: &AppState,
    account_id: &str,
    username: &str,
) -> Result<ManageSieveClient, Response> {
    let configured = domain_config::upstream_for(&state.store, username)
        .await
        .map_err(|_| upstream("could not load the account's Sieve configuration"))?
        .ok_or_else(|| upstream("the account has no configured upstream"))?;
    let wire_username = configured
        .auth_username(crate::domain_selection::ServiceRole::Rules, username)
        .map_err(|_| upstream("invalid Sieve authentication identity"))?;
    let account = state
        .store
        .get_account(account_id)
        .await
        .map_err(|_| upstream("could not resolve Sieve account"))?;
    if mw_engine::AccountPolicy::from_json(&account.sync_policy_json).domain_binding
        != configured
            .binding()
            .map_err(|_| upstream("invalid domain binding"))?
    {
        return Err(crate::unauthorized());
    }
    let endpoint = configured.sieve.ok_or_else(|| {
        (
            StatusCode::NOT_IMPLEMENTED,
            Json(json!({"error":"this account has no configured ManageSieve endpoint"})),
        )
            .into_response()
    })?;
    let credentials = state
        .store
        .account_credentials(account_id)
        .await
        .map_err(|_| upstream("could not load mail credentials"))?;
    let plan = image_proxy::load_plan(&state.store)
        .await
        .map_err(|_| upstream("configured egress route is unavailable"))?;
    let domain = match crate::upstream_resolve::domain_of(username) {
        Some(domain) => Some(
            state
                .store
                .canonical_policy_domain(&domain)
                .await
                .map_err(|_| upstream("could not resolve Sieve domain route"))?,
        ),
        None => None,
    };
    let resolved = crate::domain_security::resolve_endpoint(
        "sieve",
        &endpoint,
        domain.as_deref().unwrap_or(""),
        &plan,
    )
    .await
    .map_err(|_| upstream("Sieve transport negotiation failed"))?;
    let endpoint = resolved.endpoint;
    let auth_policy = endpoint
        .auth_policy("sieve")
        .map_err(|_| upstream("invalid Sieve authentication policy"))?;
    let mode = match endpoint.tls {
        domain_config::TlsMode::Implicit => TlsMode::Implicit,
        domain_config::TlsMode::Starttls => TlsMode::StartTls,
        domain_config::TlsMode::None => TlsMode::Plaintext,
    };
    let stream = mw_egress::plan::connect_sieve_with(
        &plan,
        domain.as_deref(),
        endpoint.host.trim_matches(['[', ']']),
        endpoint.port,
        endpoint.allow_private,
        mode != TlsMode::Plaintext,
    )
    .await
    .map_err(|_| upstream("could not connect to the configured Sieve endpoint"))?;
    tokio::time::timeout(
        std::time::Duration::from_secs(15),
        ManageSieveClient::from_stream_with_auth(
            Box::new(stream),
            endpoint.host.trim_matches(['[', ']']),
            mode,
            Credentials::Plain {
                username: wire_username,
                password: credentials.password,
            },
            &resolved.tls_options,
            auth_policy.as_ref(),
        ),
    )
    .await
    .map_err(|_| upstream("Sieve handshake timed out"))?
    .map_err(|_| upstream("Sieve authentication or TLS failed"))
}

pub(crate) async fn upload_limit(
    state: &AppState,
    session: &mw_store::Session,
) -> Result<usize, Response> {
    let account = state
        .store
        .get_account(&session.account_id)
        .await
        .map_err(|_| upstream("could not resolve Sieve account"))?;
    let domain = crate::upstream_resolve::domain_of(&account.username).unwrap_or_default();
    let subject = mw_store::settings_policy::SettingSubject {
        domain: &domain,
        account_id: &session.account_id,
        address: &account.username,
    };
    settings_registry::effective_for(&state.store, "sieve.maxScriptBytes", &subject)
        .await
        .map(|v| {
            v.and_then(|v| v.value.as_u64())
                .unwrap_or(65536)
                .min(50_000_000) as usize
        })
        .map_err(|_| upstream("could not resolve Sieve upload policy"))
}
