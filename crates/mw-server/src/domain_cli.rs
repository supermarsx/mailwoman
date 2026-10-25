//! Offline administrator domain commands use the same schema as the HTTP editor.
use anyhow::{Result, anyhow, bail};
use mw_admin::cli::DomainVerb;
use mw_store::{DomainConfigRow, Store};
use serde_json::json;

pub async fn run(store: &Store, verb: DomainVerb) -> Result<String> {
    let name = |raw: &str| crate::domain_config::normalize_domain(raw).map_err(|e| anyhow!(e));
    let now = chrono::Utc::now().timestamp();
    let enable = matches!(&verb, DomainVerb::Enable { .. });
    let (action, target) = match verb {
        DomainVerb::List => {
            return Ok(store
                .list_domain_configs()
                .await?
                .iter()
                .map(|r| r.name.clone())
                .collect::<Vec<_>>()
                .join("\n"));
        }
        DomainVerb::Show { name: raw } => {
            let n = name(&raw)?;
            let row = store
                .get_domain_config(&n)
                .await?
                .ok_or_else(|| anyhow!("domain not found"))?;
            return Ok(serde_json::to_string_pretty(
                &json!({"name":row.name,"enabled":row.enabled,"preferred":row.preferred,
                "upstream":serde_json::from_str::<serde_json::Value>(&row.upstream_json)?,
                "allowlist":serde_json::from_str::<serde_json::Value>(&row.allowlist_json)?,
                "blocklist":serde_json::from_str::<serde_json::Value>(&row.blocklist_json)?,
                "aliases":store.domain_aliases(&n).await?,"hosts":store.domain_hosts(&n).await?}),
            )?);
        }
        DomainVerb::Create { name: raw } => {
            let n = name(&raw)?;
            if store.domain_alias_target(&n).await?.is_some() {
                bail!("name is already an alias");
            }
            if store.get_domain_config(&n).await?.is_none() {
                let row = DomainConfigRow {
                    name: n.clone(),
                    upstream_json: "{}".into(),
                    allowlist_json: "[]".into(),
                    blocklist_json: "[]".into(),
                    enabled: true,
                    preferred: false,
                    settings_json: "{}".into(),
                    updated_at: now,
                };
                if !store.compare_and_put_domain_config(None, &row).await? {
                    bail!("domain changed concurrently");
                }
            }
            ("domain-created", n)
        }
        DomainVerb::Delete { name: raw } => {
            let n = name(&raw)?;
            if !store.delete_domain_config(&n, false).await? {
                bail!("domain is missing, preferred, or has signed-in accounts");
            }
            ("domain-deleted", n)
        }
        DomainVerb::SetUpstream { name: raw, json } => {
            let n = name(&raw)?;
            let config =
                crate::domain_config::UpstreamConfig::parse(&json).map_err(|e| anyhow!(e))?;
            let engine = std::env::var("MW_MODE").is_ok_and(|mode| mode == "engine");
            if (engine && config.jmap.is_some()) || (!engine && config.mail().is_some()) {
                bail!("upstream must match MW_MODE");
            }
            let previous = store
                .get_domain_config(&n)
                .await?
                .ok_or_else(|| anyhow!("domain not found"))?;
            let mut row = previous.clone();
            row.upstream_json = json;
            row.updated_at = now;
            if !store
                .compare_and_put_domain_config(Some(&previous), &row)
                .await?
            {
                bail!("domain changed concurrently");
            }
            ("domain-updated", n)
        }
        DomainVerb::Enable { name: raw } | DomainVerb::Disable { name: raw } => {
            let n = name(&raw)?;
            let previous = store
                .get_domain_config(&n)
                .await?
                .ok_or_else(|| anyhow!("domain not found"))?;
            let mut row = previous.clone();
            row.enabled = enable;
            row.updated_at = now;
            if !store
                .compare_and_put_domain_config(Some(&previous), &row)
                .await?
            {
                bail!("domain changed concurrently");
            }
            (
                if enable {
                    "domain-enabled"
                } else {
                    "domain-disabled"
                },
                n,
            )
        }
        DomainVerb::Prefer { name: raw } => {
            let n = name(&raw)?;
            store.set_preferred_domain(Some(&n)).await?;
            ("domain-preferred", n)
        }
        DomainVerb::Unprefer => {
            store.set_preferred_domain(None).await?;
            ("domain-preference-cleared", "domains".into())
        }
        DomainVerb::Alias {
            name: raw,
            alias,
            remove,
        } => {
            let n = name(&raw)?;
            let alias = name(&alias)?;
            if remove {
                store.delete_domain_alias(&alias, &n).await?;
            } else {
                store.put_domain_alias(&alias, &n).await?;
            }
            ("domain-alias-updated", n)
        }
        DomainVerb::Host {
            name: raw,
            host,
            remove,
        } => {
            let n = name(&raw)?;
            let host = name(&host)?;
            if remove {
                store.delete_domain_host(&host, &n).await?;
            } else {
                store.put_domain_host(&host, &n).await?;
            }
            ("domain-host-updated", n)
        }
    };
    store
        .append_audit(&mw_store::AuditRow {
            id: uuid::Uuid::new_v4().to_string(),
            ts: chrono::Utc::now().to_rfc3339(),
            actor: "cli".into(),
            actor_kind: "admin".into(),
            action: action.into(),
            target: Some(format!("domain:{target}")),
            detail_json: "{}".into(),
            ip: None,
        })
        .await?;
    Ok(format!("{action}: {target}"))
}
