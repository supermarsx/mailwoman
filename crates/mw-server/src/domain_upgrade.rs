//! Preserve known upstreams once, then refuse accounts without operator routing.
use crate::{SecurityConfig, domain_config, upstream_resolve};
use anyhow::Result;
use mw_store::{AccountKind, DomainConfigRow, Store};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(crate) async fn refresh_session(
    state: &crate::AppState,
    session: &mut mw_store::Session,
) -> Result<(), axum::response::Response> {
    // Header-auth has an explicit operator-owned JMAP override. Keep a session
    // on that exact configured authority; ordinary sessions still use the resolver.
    let header_override = (state.engine.is_none() && crate::env_flag("MW_HEADER_AUTH"))
        .then(|| std::env::var("MW_HEADER_AUTH_JMAP_URL").ok())
        .flatten()
        .filter(|url| !url.is_empty() && url == &session.jmap_url);
    let header_override = if header_override.is_some()
        && state
            .store
            .session_authentication(&session.id)
            .await
            .map_err(|error| crate::account_gate::unavailable("session authentication", error))?
            == "header-auth"
    {
        header_override
    } else {
        None
    };
    if !domain_config::signin_allowed(&state.store, &session.credentials.username)
        .await
        .map_err(|_| crate::unauthorized())?
    {
        return Err(crate::unauthorized());
    }
    let upstream = if let Some(url) = header_override {
        upstream_resolve::Upstream::Jmap(url)
    } else {
        upstream_resolve::resolve_upstream(state, &session.credentials.username)
            .await
            .map_err(|_| crate::unauthorized())?
    };
    if state.engine.is_some() {
        let account = state
            .store
            .get_account(&session.account_id)
            .await
            .map_err(|_| crate::unauthorized())?;
        if !crate::engine_mode::account_matches_upstream(&state.store, &account, upstream.url())
            .await
            .unwrap_or(false)
        {
            crate::account_gate::stop_engine_work(state, &session.account_id);
            return Err(crate::unauthorized());
        }
    } else if session.jmap_url != upstream.url() {
        let document =
            crate::upstream_session(&state.security, upstream.url(), &session.credentials)
                .await
                .map_err(|_| crate::unauthorized())?;
        crate::session_urls_on_origin(upstream.url(), &document)
            .map_err(|_| crate::unauthorized())?;
        let account = document
            .primary_mail_account()
            .ok_or_else(crate::unauthorized)?
            .to_string();
        let api = crate::resolve_api_url(upstream.url(), &document.api_url);
        let changed = state
            .store
            .repoint_session_authority(
                &session.id,
                &session.jmap_url,
                &account,
                upstream.url(),
                &api,
            )
            .await
            .map_err(|error| crate::account_gate::unavailable("session authority", error))?;
        if !changed {
            return Err(crate::unauthorized());
        }
        state.step_up.forget_user(&session.id);
        session.account_id = account;
        session.jmap_url = upstream.url().into();
        session.api_url = api;
    }
    Ok(())
}

pub(crate) async fn prepare(store: &Store, security: &SecurityConfig, engine: bool) -> Result<()> {
    const IMPORTED: &str = "v6:upstream_import:done";
    let progress = store.get_setting(IMPORTED).await?;
    if progress.as_deref() != Some("1") {
        let configured = store.list_domain_configs().await?.iter().any(|row| {
            domain_config::UpstreamConfig::parse(&row.upstream_json)
                .is_ok_and(|c| c.mail().is_some() || c.jmap.is_some())
        });
        let subject = mw_store::settings_policy::SettingSubject {
            domain: "",
            account_id: "",
            address: "",
        };
        let global = crate::settings_registry::effective_for(store, "upstream.default", &subject)
            .await?
            .is_some_and(|value| value.value.as_str().is_some());
        if progress.as_deref() == Some("in-progress")
            || (!configured
                && !global
                && upstream_resolve::resolve_from_config(security, engine).is_err())
        {
            store.set_setting(IMPORTED, "in-progress").await?;
            let mut candidates: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
            if engine {
                for account in store.list_accounts().await? {
                    let Some(domain) = upstream_resolve::domain_of(&account.username)
                        .and_then(|n| domain_config::normalize_domain(&n).ok())
                    else {
                        continue;
                    };
                    let tls = match account.tls.as_str() {
                        "implicit" => "implicit",
                        "starttls" => "starttls",
                        "plaintext" => "none",
                        _ => continue,
                    };
                    let protocol = match account.kind {
                        AccountKind::Imap => "imap",
                        AccountKind::Pop3 => "pop3",
                        AccountKind::Ews => continue,
                    };
                    // Existing targets are operator-reviewable imports; allow private
                    // addressing explicitly so an on-prem install keeps its upstream.
                    let upstream = json!({protocol:{"host":account.host,"port":account.port,"tls":tls,"auth":"password","allowPrivate":true}}).to_string();
                    if domain_config::UpstreamConfig::parse(&upstream).is_ok() {
                        *candidates
                            .entry(domain)
                            .or_default()
                            .entry(upstream)
                            .or_default() += 1;
                    }
                }
            } else {
                for session in store.session_authorities().await? {
                    let Some(domain) = upstream_resolve::domain_of(&session.username)
                        .and_then(|n| domain_config::normalize_domain(&n).ok())
                    else {
                        continue;
                    };
                    let upstream = json!({"jmap":{"sessionUrl":session.jmap_url}}).to_string();
                    if domain_config::UpstreamConfig::parse(&upstream).is_ok() {
                        *candidates
                            .entry(domain)
                            .or_default()
                            .entry(upstream)
                            .or_default() += 1;
                    }
                }
            }
            for (name, choices) in candidates {
                let ambiguous = choices.len() > 1;
                let Some((upstream, _)) = choices.iter().max_by_key(|(_, count)| **count) else {
                    continue;
                };
                let previous = store.get_domain_config(&name).await?;
                // A crash may have imported an earlier domain already; never
                // overwrite a subsequent administrator edit on restart.
                if previous.as_ref().is_some_and(|row| {
                    domain_config::UpstreamConfig::parse(&row.upstream_json)
                        .is_ok_and(|config| config.mail().is_some() || config.jmap.is_some())
                }) {
                    continue;
                }
                let mut row = previous.clone().unwrap_or_else(|| DomainConfigRow {
                    name: name.clone(),
                    upstream_json: "{}".into(),
                    allowlist_json: "[]".into(),
                    blocklist_json: "[]".into(),
                    enabled: true,
                    preferred: false,
                    settings_json: "{}".into(),
                    updated_at: 0,
                });
                let mut settings: Value = serde_json::from_str(&row.settings_json)?;
                anyhow::ensure!(
                    settings.is_object(),
                    "invalid domain settings during upstream import"
                );
                settings["importedAt"] = json!(chrono::Utc::now().to_rfc3339());
                settings["upstreamImportAmbiguous"] = json!(ambiguous);
                row.settings_json = settings.to_string();
                row.upstream_json = upstream.clone();
                row.updated_at = chrono::Utc::now().timestamp();
                if store
                    .compare_and_put_domain_config(previous.as_ref(), &row)
                    .await?
                {
                    audit(
                        store,
                        "domain-upstream-imported",
                        &name,
                        json!({"ambiguous":ambiguous}),
                    )
                    .await?;
                }
            }
        }
        store.set_setting(IMPORTED, "1").await?;
    }
    for session in store.session_authorities().await? {
        // The explicit trusted-header authority remains configured even when
        // there is no generic default. Only sessions issued by that auth path
        // may use it; an ordinary password session cannot borrow the override.
        if !engine
            && crate::env_flag("MW_HEADER_AUTH")
            && std::env::var("MW_HEADER_AUTH_JMAP_URL")
                .ok()
                .is_some_and(|url| !url.is_empty() && url == session.jmap_url)
            && store.session_authentication(&session.id).await? == "header-auth"
            && domain_config::signin_allowed(store, &session.username)
                .await
                .unwrap_or(false)
        {
            continue;
        }
        if upstream_resolve::resolve_for(store, security, engine, &session.username)
            .await
            .is_err()
        {
            store
                .remember_account_names(&session.account_id, &[&session.username])
                .await?;
            store.delete_session(&session.id).await?;
            store
                .delete_native_session(&crate::push_relay::hash_token(&session.id))
                .await?;
            audit(
                store,
                "session-upstream-unconfigured",
                &session.account_id,
                json!({}),
            )
            .await?;
        }
    }
    Ok(())
}

async fn audit(store: &Store, action: &str, target: &str, detail: Value) -> Result<()> {
    store
        .append_audit(&mw_store::AuditRow {
            id: uuid::Uuid::new_v4().to_string(),
            ts: chrono::Utc::now().to_rfc3339(),
            actor: "system".into(),
            actor_kind: "system".into(),
            action: action.into(),
            target: Some(target.into()),
            detail_json: detail.to_string(),
            ip: None,
        })
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mw_store::{Credentials, NewAccount, ServerKey};
    #[tokio::test]
    async fn import_keeps_matching_sessions_and_blocks_unmatched_without_reimport() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let creds = Credentials {
            username: "alice@example.org".into(),
            password: "test".into(),
        };
        let account = store
            .upsert_account_by_identity(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host: "mail.example.org",
                    port: 993,
                    tls: "implicit",
                    username: &creds.username,
                    sync_policy_json: "{}",
                },
                &creds,
            )
            .await
            .unwrap();
        let keep = store
            .create_session(&account, &creds.username, "engine", "engine", &creds)
            .await
            .unwrap();
        let refuse = store
            .create_session("old", "bare-user", "engine", "engine", &creds)
            .await
            .unwrap();
        prepare(&store, &SecurityConfig::default(), true)
            .await
            .unwrap();
        assert!(store.get_session(&keep).await.is_ok());
        assert!(store.get_session(&refuse).await.is_err());
        let imported = store
            .get_domain_config("example.org")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&imported.upstream_json).unwrap()["imap"]["host"],
            "mail.example.org"
        );
        assert!(
            serde_json::from_str::<Value>(&imported.settings_json).unwrap()["importedAt"]
                .is_string()
        );
        store
            .delete_domain_config("example.org", true)
            .await
            .unwrap();
        prepare(&store, &SecurityConfig::default(), true)
            .await
            .unwrap();
        assert!(
            store
                .get_domain_config("example.org")
                .await
                .unwrap()
                .is_none()
        );
        assert!(store.get_session(&keep).await.is_err());
    }

    #[tokio::test]
    async fn proxy_import_uses_most_common_authority_and_marks_ambiguity() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        for (name, url) in [
            ("a@example.org", "https://first.test/jmap"),
            ("b@example.org", "https://first.test/jmap"),
            ("c@example.org", "https://second.test/jmap"),
        ] {
            let credentials = Credentials {
                username: name.into(),
                password: "test".into(),
            };
            store
                .create_session(name, name, url, url, &credentials)
                .await
                .unwrap();
        }
        prepare(&store, &SecurityConfig::default(), false)
            .await
            .unwrap();
        let row = store
            .get_domain_config("example.org")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&row.upstream_json).unwrap()["jmap"]["sessionUrl"],
            "https://first.test/jmap"
        );
        assert_eq!(
            serde_json::from_str::<Value>(&row.settings_json).unwrap()["upstreamImportAmbiguous"],
            true
        );
    }
}
