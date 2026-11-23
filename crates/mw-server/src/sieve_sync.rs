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

/// The Sieve policy keys [`context`] resolves.
const POLICY_KEYS: [&str; 7] = [
    "sieve.maxScriptBytes",
    "sieve.maxScripts",
    "sieve.maxRedirects",
    "cap.sieve.raw_edit",
    "cap.sieve.forward_external",
    "sieve.vacationAllowed",
    "sieve.allowedExtensions",
];

/// t34-e11, `MW_GUARD_SNAPSHOT=bounded` only: this session's published request
/// inputs (the verified frame `request_inputs` admitted for this request) and
/// that frame's settings revision. A caller that uses them seals the frame
/// before it responds (`request_inputs::seal`), so a source that changed in
/// the meantime is refused rather than served. Off mode, or no frame for this
/// exact session: `None`, and every read below stays the original fresh one.
async fn published(
    state: &AppState,
    session: &mw_store::Session,
) -> Option<(
    std::sync::Arc<crate::request_inputs::Current>,
    mw_store::settings_policy::SettingsReadRevision,
)> {
    if crate::request_inputs::snapshot::mode() == crate::request_inputs::snapshot::Mode::Off {
        return None;
    }
    let inputs = crate::request_inputs::current(&state.store).await?;
    if !inputs.matches_session(session) || inputs.prepared().is_err() {
        return None;
    }
    let revision = crate::request_inputs::revision(&state.store, false).await?;
    Some((inputs, revision))
}

pub(crate) async fn context(
    state: &AppState,
    session: &mw_store::Session,
    request: Option<&Value>,
) -> Result<mw_engine::sieve_jmap::SieveContext, Response> {
    // Only the session document (no JMAP request) uses the frame: its result
    // only advertises the capability. JMAP Sieve methods, whose writes this
    // policy authorizes, keep the original fresh per-key reads.
    let frame = match request {
        None => published(state, session).await,
        Some(_) => None,
    };
    let prepared = match &frame {
        Some((inputs, _)) => Some(
            inputs
                .prepared()
                .map_err(|_| upstream("could not resolve Sieve account identity"))?,
        ),
        None => None,
    };
    let account = match prepared {
        Some(prepared) => std::borrow::Cow::Borrowed(&prepared.account),
        None => std::borrow::Cow::Owned(
            state
                .store
                .get_account(&session.account_id)
                .await
                .map_err(|_| upstream("could not resolve Sieve account identity"))?,
        ),
    };
    let username = &account.username;
    let domain = crate::upstream_resolve::domain_of(username).unwrap_or_default();
    let subject = mw_store::settings_policy::SettingSubject {
        personal_owner: None,
        domain: &domain,
        account_id: &session.account_id,
        address: username,
    };
    // One resolution for all seven keys at the frame's revision (bounded mode
    // on published inputs); otherwise each key resolves fresh, as before.
    let resolved = match &frame {
        Some((_, revision)) => Some(
            settings_registry::effective_all_at_revision(
                &state.store,
                &subject,
                |def| POLICY_KEYS.contains(&def.key),
                revision,
            )
            .await
            .map_err(|_| upstream("could not resolve Sieve policy"))?,
        ),
        None => None,
    };
    let resolved = resolved.as_deref();
    async fn setting(
        state: &AppState,
        key: &str,
        subject: &mw_store::settings_policy::SettingSubject<'_>,
        fallback: Value,
        resolved: Option<&[settings_registry::Effective]>,
    ) -> Result<Value, Response> {
        debug_assert!(POLICY_KEYS.contains(&key));
        if let Some(values) = resolved {
            return Ok(values
                .iter()
                .find(|value| value.key == key)
                .map(|value| value.value.clone())
                .unwrap_or(fallback));
        }
        settings_registry::effective_for(&state.store, key, subject)
            .await
            .map(|v| v.map(|e| e.value).unwrap_or(fallback))
            .map_err(|_| upstream("could not resolve Sieve policy"))
    }
    let policy = mw_sieve::policy::Policy {
        max_script_bytes: Some(
            setting(state, "sieve.maxScriptBytes", &subject, json!(65536), resolved)
                .await?
                .as_u64()
                .unwrap_or(65536) as usize,
        ),
        max_scripts: Some(
            setting(state, "sieve.maxScripts", &subject, json!(5), resolved)
                .await?
                .as_u64()
                .unwrap_or(5) as usize,
        ),
        max_redirects: Some(
            setting(state, "sieve.maxRedirects", &subject, json!(4), resolved)
                .await?
                .as_u64()
                .unwrap_or(4) as usize,
        ),
        raw_edit: setting(state, "cap.sieve.raw_edit", &subject, json!(false), resolved)
            .await?
            .as_bool()
            .unwrap_or(false),
        forward_external: setting(state, "cap.sieve.forward_external", &subject, json!(false), resolved)
            .await?
            .as_bool()
            .unwrap_or(false),
        vacation_allowed: setting(state, "sieve.vacationAllowed", &subject, json!(false), resolved)
            .await?
            .as_bool()
            .unwrap_or(false),
        allowed_extensions: setting(state, "sieve.allowedExtensions", &subject, Value::Null, resolved)
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
    // The frame's parsed domain configuration is the one `upstream_for` gives
    // this account (engine_mode::ensure_account relies on the same identity).
    let configured = match prepared {
        Some(prepared) => prepared.configuration.sieve.is_some(),
        None => domain_config::upstream_for(&state.store, username)
            .await
            .map_err(|_| upstream("could not load Sieve configuration"))?
            .and_then(|c| c.sieve)
            .is_some(),
    };
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
        personal_owner: None,
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

#[cfg(test)]
mod tests {
    use super::*;
    use mw_store::settings_policy::{SettingPolicyRow, SettingScope, SettingSubject, SettingValueRow};

    /// t34-e11: the seven Sieve keys resolved together at one revision equal
    /// the seven resolved one by one on the same settings state, with a global
    /// policy, domain-level values and an administrator-set user-level value in
    /// play (plus a user's own row, which neither resolver applies). This is
    /// what lets the bounded-mode gate in [`context`] be removed later.
    #[tokio::test]
    async fn sieve_keys_resolved_together_equal_keys_resolved_separately() {
        let store = mw_store::Store::open_in_memory(mw_store::ServerKey::generate()).await.unwrap();
        let user = "person@sieve-eq.test";
        let account = store
            .create_account(
                &mw_store::NewAccount {
                    kind: mw_store::AccountKind::Imap, host: "127.0.0.1", port: 9, tls: "starttls",
                    username: user, sync_policy_json: "{}",
                },
                &mw_store::Credentials { username: user.into(), password: "x".into() },
            )
            .await
            .unwrap();
        let policy = |key: &str, value: &str, allow_user_override: bool| SettingPolicyRow {
            key: key.into(), global_value: Some(value.into()), allow_user_override,
            global_locked: false, updated_by: "t34-e11".into(), updated_at: 1,
        };
        let value = |scope: SettingScope, scope_id: &str, key: &str, value: &str| SettingValueRow {
            scope, scope_id: scope_id.into(), key: key.into(), value: value.into(),
            locked: false, updated_by: "t34-e11".into(), updated_at: 1,
        };
        store.put_setting_policy(&policy("sieve.maxScripts", "7", false)).await.unwrap();
        store.put_setting_policy(&policy("sieve.maxScriptBytes", "4096", true)).await.unwrap();
        store.put_setting_value(&value(SettingScope::Domain, "sieve-eq.test", "sieve.maxRedirects", "2")).await.unwrap();
        store.put_setting_value(&value(SettingScope::Domain, "sieve-eq.test", "cap.sieve.raw_edit", "true")).await.unwrap();
        store.put_setting_value(&value(SettingScope::AdminUser, &account, "sieve.vacationAllowed", "true")).await.unwrap();
        store.put_setting_value(&value(SettingScope::User, &account, "sieve.maxScriptBytes", "1000")).await.unwrap();
        let subject = SettingSubject { personal_owner: None, domain: "sieve-eq.test", account_id: &account, address: user };
        let revision = store.settings_read_revision().await.unwrap();
        let together = settings_registry::effective_all_at_revision(
            &store, &subject, |def| POLICY_KEYS.contains(&def.key), &revision).await.unwrap();
        assert_eq!(together.len(), POLICY_KEYS.len(), "every Sieve key is a registry key");
        for key in POLICY_KEYS {
            let alone = settings_registry::effective_for(&store, key, &subject).await.unwrap()
                .unwrap_or_else(|| panic!("{key} resolves alone"));
            let joint = together.iter().find(|v| v.key == key).unwrap_or_else(|| panic!("{key} resolves together"));
            assert_eq!(joint.value, alone.value, "{key}");
        }
        let get = |key: &str| together.iter().find(|v| v.key == key).unwrap().value.clone();
        // The overrides took effect, so the equality above is not default-vs-default.
        assert_eq!(get("sieve.maxScripts"), json!(7), "global policy");
        assert_eq!(get("sieve.maxRedirects"), json!(2), "domain value");
        assert_eq!(get("cap.sieve.raw_edit"), json!(true), "domain value");
        assert_eq!(get("sieve.vacationAllowed"), json!(true), "admin-set user value");
        // These are administrator policy keys: a row the user wrote for one is
        // not applied, by either resolver (both give the global policy value).
        assert_eq!(get("sieve.maxScriptBytes"), json!(4096), "a user's own row does not override");
        store.close().await;
    }
}
