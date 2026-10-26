//! Domain-selected provider authentication. All destinations are operator-owned.
pub(crate) mod config;
mod probe;

use config::EwsConfig;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SelectedEws {
    pub config: EwsConfig,
    pub policy_domain: String,
    pub login_address: String,
    pub wire_username: String,
    pub binding: String,
}

impl SelectedEws {
    pub async fn for_login(store: &mw_store::Store, address: &str) -> Result<Option<Self>, String> {
        let identity = crate::domain_config::canonical(store, address, None).await?;
        let Some(row) = crate::domain_config::configured_row(store, &identity).await? else {
            return Ok(None);
        };
        if !row.enabled {
            return Err("provider domain disabled".into());
        }
        let upstream = crate::domain_config::UpstreamConfig::parse(&row.upstream_json)?;
        let Some(config) = upstream.ews.clone() else {
            return Ok(None);
        };
        config.validate()?;
        let binding = upstream
            .binding()?
            .ok_or("provider authority requires explicit selection")?;
        // The configured domain decides the endpoint. The authenticated login
        // spelling is retained independently from its canonical policy domain.
        let login_address = identity.login_name;
        let wire_username =
            upstream.auth_username(crate::domain_selection::ServiceRole::Mail, &login_address)?;
        Ok(Some(Self {
            config,
            policy_domain: identity.domain,
            login_address,
            wire_username,
            binding,
        }))
    }

    pub async fn admit(
        &self,
        store: &mw_store::Store,
        account: Option<&str>,
    ) -> Result<(), String> {
        let current = Self::for_login(store, &self.login_address)
            .await?
            .ok_or("provider selection removed")?;
        if current != *self {
            return Err("provider selection changed".into());
        }
        let policy_address = store
            .canonical_policy_address(&self.login_address)
            .await
            .map_err(|_| "provider identity unavailable")?;
        let subject = mw_store::settings_policy::SettingSubject {
            account_id: account.unwrap_or(""),
            domain: &self.policy_domain,
            address: &policy_address,
        };
        for key in [
            "cap.plugins".to_owned(),
            crate::features::plugin_policy_key("bridge-ews"),
        ] {
            let enabled = crate::settings_registry::effective_for(store, &key, &subject)
                .await
                .map_err(|_| "provider policy unavailable")?;
            if !enabled.is_some_and(|value| value.value.as_bool() == Some(true)) {
                return Err("provider feature disabled".into());
            }
        }
        if let Some(account) = account
            && crate::account_gate::for_account(store, account)
                .await
                .map_err(|_| "provider account unavailable")?
                .disabled
        {
            return Err("provider account disabled".into());
        }
        Ok(())
    }
}

/// Revalidate durable provider provenance before opening credentials or network.
pub(crate) async fn stored_selection(
    store: &mw_store::Store,
    account: &str,
) -> Result<Option<SelectedEws>, String> {
    let Some(saved) = store
        .provider_account(account)
        .await
        .map_err(|_| "provider account unavailable")?
    else {
        match store.get_account(account).await {
            Ok(row) if row.kind == mw_store::AccountKind::Ews => {
                return Err("provider binding unavailable".into());
            }
            Ok(_) | Err(mw_store::StoreError::NotFound) => {}
            Err(_) => return Err("provider account unavailable".into()),
        }
        return Ok(None);
    };
    let native = store
        .get_account(account)
        .await
        .map_err(|_| "provider account unavailable")?;
    if native.kind != mw_store::AccountKind::Ews {
        return Err("provider kind mismatch".into());
    }
    let policy: serde_json::Value = serde_json::from_str(&native.sync_policy_json)
        .map_err(|_| "provider account policy unavailable")?;
    if !policy.is_object()
        || policy
            .get("credential_rejected")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
    {
        return Err("provider credentials require fresh sign-in".into());
    }
    let selected = SelectedEws::for_login(store, &saved.login_address)
        .await?
        .ok_or("provider selection removed")?;
    if selected.binding != saved.domain_binding
        || selected.config.endpoint()?.as_str() != saved.endpoint_url
        || selected.wire_username.to_ascii_lowercase() != saved.backend_username
        || selected.config.ca_pem != saved.ca_pem
    {
        return Err("provider authority changed; fresh sign-in required".into());
    }
    selected.admit(store, Some(account)).await?;
    Ok(Some(selected))
}

pub(crate) async fn login(
    engine: &std::sync::Arc<mw_engine::Engine>,
    registry: &crate::plugins::PluginRegistry,
    selected: SelectedEws,
    password: &str,
) -> Result<(String, String), crate::login_failure::LoginFailure> {
    use crate::login_failure::{LoginFailure, LoginFailureReason as Reason};
    static LOGINS: std::sync::OnceLock<tokio::sync::Semaphore> = std::sync::OnceLock::new();
    let _admission = LOGINS
        .get_or_init(|| tokio::sync::Semaphore::new(4))
        .try_acquire()
        .map_err(|_| LoginFailure::new(Reason::Policy))?;
    let store = engine.store();
    let endpoint = selected
        .config
        .endpoint()
        .map_err(|_| LoginFailure::new(Reason::NotConfigured))?;
    let key = format!(
        "ews-login:{}:{}",
        mw_store::provider_accounts::ews_authority(endpoint.as_str()),
        selected.wire_username.to_ascii_lowercase()
    );
    let lock = crate::engine_mode::connect_lock(&key);
    let _connecting = lock.lock().await;
    selected
        .admit(store, None)
        .await
        .map_err(|_| LoginFailure::new(Reason::Policy))?;
    let existing = store
        .ews_account_for_login(endpoint.as_str(), &selected.wire_username)
        .await
        .map_err(|_| LoginFailure::new(Reason::Storage))?;
    if let Some(existing) = &existing {
        let original = crate::domain_config::canonical(store, &existing.login_address, None)
            .await
            .map_err(|_| LoginFailure::new(Reason::Policy))?;
        if original.domain != selected.policy_domain {
            return Err(LoginFailure::new(Reason::Policy));
        }
    }
    let prior_id = existing
        .as_ref()
        .map(|row| row.account_id.as_str())
        .unwrap_or("");
    let gate = crate::account_gate::for_login(
        store,
        prior_id,
        &selected.login_address,
        &selected.login_address,
    )
    .await
    .map_err(|_| LoginFailure::new(Reason::Storage))?;
    if gate.disabled {
        return Err(LoginFailure::new(Reason::Disabled));
    }
    selected
        .admit(store, existing.as_ref().map(|row| row.account_id.as_str()))
        .await
        .map_err(|_| LoginFailure::new(Reason::Policy))?;
    let mailboxes = probe::authenticate(
        store,
        &selected,
        &selected.wire_username,
        password,
        existing.as_ref().map(|row| row.account_id.as_str()),
    )
    .await?;
    if !crate::domain_config::signin_allowed(store, &selected.login_address)
        .await
        .map_err(|_| LoginFailure::new(Reason::Policy))?
    {
        return Err(LoginFailure::new(Reason::Policy));
    }
    selected
        .admit(store, existing.as_ref().map(|row| row.account_id.as_str()))
        .await
        .map_err(|_| LoginFailure::new(Reason::Policy))?;
    let account = store
        .enroll_verified_ews(mw_store::provider_accounts::VerifiedEwsEnrollment {
            endpoint_url: endpoint.as_str(),
            backend_username: &selected.wire_username,
            login_address: &selected.login_address,
            domain_binding: &selected.binding,
            ca_pem: selected.config.ca_pem.as_deref(),
            password,
        })
        .await
        .map_err(|_| LoginFailure::new(Reason::Storage))?;
    let account_lock = crate::engine_mode::connect_lock(&account.account_id);
    let _activating = account_lock.lock().await;
    crate::v7_mount::activate_ews(engine, registry, &account.account_id, mailboxes)
        .await
        .map_err(|_| LoginFailure::new(Reason::Policy))?;
    Ok((account.account_id, account.login_address))
}
