//! The `mailwoman admin <noun> <verb>` CLI (plan §2.5 — GitOps-friendly): one
//! noun per §19 panel section, so an operator can drive provisioning, the audit
//! log and the ban list from config management.
//!
//! Like the panel, it offers no flag for a stored value that has no defined
//! meaning (26.20, t28-e8). The flags that store a value nothing applies yet say
//! so in their help text.
//!
//! `mw-admin` owns the clap tree ([`AdminCommand`]) + the [`run`] dispatcher;
//! `mw-server`'s `main.rs` (owned by e11 at mount) embeds [`AdminCommand`] under
//! its top-level `admin` subcommand and calls [`run`] with a constructed
//! [`Admin`]. Keeping the tree here (not in `main.rs`) respects this executor's
//! path ownership (`crates/mw-admin/**` only).
//!
//! The six nouns map 1:1 to the §2.5 endpoint sections:
//! `domains` · `users` · `security-policy` · `integrations` · `observability`
//! (audit viewer/export + login-monitor/ban-list) · `appearance`.

use clap::{Args, Parser, Subcommand};

use crate::{Admin, AdminError, CacheScopeRow, Quota, SecurityPolicy, UserFeatureFlags};

/// Standalone parser wrapping [`AdminCommand`] (tests / a potential
/// `mailwoman-admin` binary). `main.rs` embeds [`AdminCommand`] directly.
#[derive(Debug, Parser)]
#[command(
    name = "admin",
    about = "Mailwoman admin panel CLI (GitOps mirror of /admin)"
)]
pub struct AdminCli {
    #[command(subcommand)]
    pub command: AdminCommand,
}

/// The `admin` noun tree — one variant per §19 panel section.
#[derive(Debug, Subcommand)]
pub enum AdminCommand {
    /// Manage mail domains.
    Domains {
        #[command(subcommand)]
        verb: DomainVerb,
    },
    /// Provision users, quotas, sessions, and feature flags.
    Users {
        #[command(subcommand)]
        verb: UserVerb,
    },
    /// Inspect/update the stored security-policy record.
    #[command(name = "security-policy")]
    SecurityPolicy {
        #[command(subcommand)]
        verb: PolicyVerb,
    },
    /// Integration statuses + API-key oversight.
    Integrations {
        #[command(subcommand)]
        verb: IntegrationVerb,
    },
    /// Audit viewer/export, the ban list, and the stored telemetry record.
    Observability {
        #[command(subcommand)]
        verb: ObsVerb,
    },
    /// The built-in appearance defaults.
    Appearance {
        #[command(subcommand)]
        verb: AppearanceVerb,
    },
    /// Cache scope matrix (mirrors `mw-cache` per-class policy).
    #[command(name = "cache-scope")]
    CacheScope {
        #[command(subcommand)]
        verb: CacheScopeVerb,
    },
}

#[derive(Debug, Subcommand)]
pub enum DomainVerb {
    /// List all domains.
    List,
    /// Show one domain.
    Show { name: String },
    /// Register a domain name. An existing domain is left as it is.
    Create { name: String },
    /// Delete a domain.
    Delete { name: String },
}

#[derive(Debug, Subcommand)]
pub enum UserVerb {
    /// Provision (or update) a user with a quota.
    Provision {
        domain: String,
        username: String,
        #[arg(long, default_value_t = 0)]
        bytes_limit: i64,
        #[arg(long, default_value_t = 0)]
        msg_limit: i64,
    },
    /// Set a user's quota.
    Quota {
        account: String,
        #[arg(long, default_value_t = 0)]
        bytes_limit: i64,
        #[arg(long, default_value_t = 0)]
        msg_limit: i64,
    },
    /// Revoke all of a user's sessions.
    RevokeSessions { account: String },
    /// Set a user's feature flags (each flag is set to the given switch state).
    Flags {
        account: String,
        #[arg(long)]
        zero_access: bool,
        #[arg(long)]
        force_password_change: bool,
        #[arg(long)]
        remote_cache_wipe: bool,
        #[arg(long)]
        disabled: bool,
    },
    /// Request a one-shot remote cache wipe.
    CacheWipe { account: String },
}

#[derive(Debug, Subcommand)]
pub enum PolicyVerb {
    /// Show the two stored values this CLI can set.
    Show,
    /// Patch the stored record (only provided fields change).
    Set(PolicySetArgs),
}

#[derive(Debug, Args)]
pub struct PolicySetArgs {
    /// Stored only: nothing reads this value in this release.
    #[arg(long)]
    pub max_security_floor: Option<bool>,
    /// Stored only: the DLP rules that are applied come from MW_DLP_RULES.
    #[arg(long)]
    pub dlp_rules_json: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum IntegrationVerb {
    /// List integration statuses.
    List,
    /// Revoke an API key (oversight; the key store is `mw-oauth`).
    RevokeApiKey { id: String },
}

#[derive(Debug, Subcommand)]
pub enum ObsVerb {
    /// Show the stored telemetry record.
    Show,
    /// Patch the stored telemetry record. Stored only: the running log level,
    /// OTLP exporter and metrics endpoint are set from the environment at start.
    Set(ObsSetArgs),
    /// List recent audit entries (newest first).
    Audit {
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// Export recent audit entries as JSONL.
    AuditExport {
        #[arg(long, default_value_t = 1000)]
        limit: usize,
    },
    /// List ban-list entries.
    Bans,
    /// Add a ban-list entry. A record: the server does not refuse the address.
    Ban {
        ip: String,
        #[arg(long, default_value = "manual")]
        reason: String,
        #[arg(long)]
        expires_at: Option<String>,
    },
    /// Remove a ban.
    Unban { ip: String },
}

#[derive(Debug, Args)]
pub struct ObsSetArgs {
    #[arg(long)]
    pub log_level: Option<String>,
    #[arg(long)]
    pub otlp_dsn: Option<String>,
    #[arg(long)]
    pub metrics: Option<bool>,
}

#[derive(Debug, Subcommand)]
pub enum AppearanceVerb {
    /// Show the built-in appearance defaults. There is no `set`: the appearance is
    /// not persisted, so a CLI process could not change what a server serves.
    Show,
}

#[derive(Debug, Subcommand)]
pub enum CacheScopeVerb {
    /// List the cache scope matrix.
    List,
    /// Set one class's layers + TTL.
    Set {
        class: String,
        #[arg(long, default_value = "[\"memory\",\"redis\",\"store\"]")]
        layers_json: String,
        #[arg(long, default_value_t = 300)]
        ttl_secs: i64,
    },
}

/// Dispatch a parsed [`AdminCommand`] against `admin`, returning human-readable
/// output. `actor` is the audited principal (e.g. the admin username or `cli`).
pub async fn run(admin: &Admin, command: AdminCommand, actor: &str) -> Result<String, AdminError> {
    match command {
        AdminCommand::Domains { verb } => domains(admin, verb, actor).await,
        AdminCommand::Users { verb } => users(admin, verb, actor).await,
        AdminCommand::SecurityPolicy { verb } => policy(admin, verb, actor).await,
        AdminCommand::Integrations { verb } => integrations(admin, verb, actor).await,
        AdminCommand::Observability { verb } => observability(admin, verb, actor).await,
        AdminCommand::Appearance { verb } => appearance(admin, verb).await,
        AdminCommand::CacheScope { verb } => cache_scope(admin, verb, actor).await,
    }
}

async fn domains(admin: &Admin, verb: DomainVerb, actor: &str) -> Result<String, AdminError> {
    match verb {
        DomainVerb::List => Ok(admin
            .list_domains()
            .await?
            .into_iter()
            .map(|d| d.name)
            .collect::<Vec<_>>()
            .join("\n")),
        DomainVerb::Show { name } => match admin.get_domain(&name).await? {
            Some(d) => Ok(d.name),
            None => Err(AdminError::NotFound),
        },
        DomainVerb::Create { name } => {
            admin.create_domain(actor, &name).await?;
            Ok(format!("domain '{name}' created"))
        }
        DomainVerb::Delete { name } => {
            admin.delete_domain(actor, &name).await?;
            Ok(format!("domain '{name}' deleted"))
        }
    }
}

async fn users(admin: &Admin, verb: UserVerb, actor: &str) -> Result<String, AdminError> {
    match verb {
        UserVerb::Provision {
            domain,
            username,
            bytes_limit,
            msg_limit,
        } => {
            admin
                .provision_user(
                    actor,
                    &domain,
                    &username,
                    Quota {
                        bytes_limit,
                        msg_limit,
                    },
                )
                .await?;
            Ok(format!("user '{username}@{domain}' provisioned"))
        }
        UserVerb::Quota {
            account,
            bytes_limit,
            msg_limit,
        } => {
            admin
                .set_quota(
                    actor,
                    &account,
                    Quota {
                        bytes_limit,
                        msg_limit,
                    },
                )
                .await?;
            Ok(format!("quota set for '{account}'"))
        }
        UserVerb::RevokeSessions { account } => {
            let n = admin.revoke_sessions(actor, &account).await?;
            Ok(format!("revoked {n} session(s) for '{account}'"))
        }
        UserVerb::Flags {
            account,
            zero_access,
            force_password_change,
            remote_cache_wipe,
            disabled,
        } => {
            admin
                .set_feature_flags(
                    actor,
                    &account,
                    UserFeatureFlags {
                        zero_access,
                        force_password_change,
                        remote_cache_wipe,
                        disabled,
                    },
                )
                .await?;
            Ok(format!("flags set for '{account}'"))
        }
        UserVerb::CacheWipe { account } => {
            admin.request_remote_cache_wipe(actor, &account).await?;
            Ok(format!("remote cache wipe requested for '{account}'"))
        }
    }
}

async fn policy(admin: &Admin, verb: PolicyVerb, actor: &str) -> Result<String, AdminError> {
    match verb {
        PolicyVerb::Show => {
            let p = admin.get_security_policy().await?;
            Ok(pretty(&serde_json::json!({
                "max_security_floor": p.max_security_floor,
                "dlp_rules_json": p.dlp_rules_json,
            })))
        }
        PolicyVerb::Set(args) => {
            let mut p: SecurityPolicy = admin.get_security_policy().await?;
            if let Some(v) = args.max_security_floor {
                p.max_security_floor = v;
            }
            if let Some(v) = args.dlp_rules_json {
                p.dlp_rules_json = v;
            }
            admin.set_security_policy(actor, p).await?;
            Ok("security policy record stored (not applied in this release)".to_string())
        }
    }
}

async fn integrations(
    admin: &Admin,
    verb: IntegrationVerb,
    actor: &str,
) -> Result<String, AdminError> {
    match verb {
        IntegrationVerb::List => {
            // This process is not the server, so it cannot see which LDAP rows or
            // Nextcloud variables the running deployment loaded: those two print
            // `unknown`. `GET /admin/integrations` on the server reports them.
            let i = admin.integrations();
            Ok(format!(
                "webhooks={}\napi_key_oversight={}\nldap={}\nnextcloud={}",
                i.webhooks.as_str(),
                i.api_key_oversight.as_str(),
                i.ldap.as_str(),
                i.nextcloud.as_str(),
            ))
        }
        IntegrationVerb::RevokeApiKey { id } => {
            admin.revoke_api_key(actor, &id).await?;
            Ok(format!("api key '{id}' revoked"))
        }
    }
}

async fn observability(admin: &Admin, verb: ObsVerb, actor: &str) -> Result<String, AdminError> {
    match verb {
        ObsVerb::Show => Ok(pretty(&admin.get_observability().await?)),
        ObsVerb::Set(args) => {
            let mut c = admin.get_observability().await?;
            if let Some(v) = args.log_level {
                c.log_level = v;
            }
            if let Some(v) = args.otlp_dsn {
                c.otlp_dsn = if v.is_empty() { None } else { Some(v) };
            }
            if let Some(v) = args.metrics {
                c.metrics_enabled = v;
            }
            admin.set_observability(actor, c).await?;
            Ok("telemetry record stored (not applied in this release)".to_string())
        }
        ObsVerb::Audit { limit } => Ok(pretty(&admin.list_audit(limit).await?)),
        ObsVerb::AuditExport { limit } => admin.export_audit(limit).await,
        ObsVerb::Bans => Ok(pretty(&admin.list_bans().await?)),
        ObsVerb::Ban {
            ip,
            reason,
            expires_at,
        } => {
            admin.ban_ip(actor, &ip, &reason, expires_at).await?;
            Ok(format!("banned '{ip}'"))
        }
        ObsVerb::Unban { ip } => {
            admin.unban_ip(actor, &ip).await?;
            Ok(format!("unbanned '{ip}'"))
        }
    }
}

async fn appearance(admin: &Admin, verb: AppearanceVerb) -> Result<String, AdminError> {
    match verb {
        AppearanceVerb::Show => Ok(pretty(&admin.config().appearance)),
    }
}

async fn cache_scope(
    admin: &Admin,
    verb: CacheScopeVerb,
    actor: &str,
) -> Result<String, AdminError> {
    match verb {
        CacheScopeVerb::List => Ok(pretty(&admin.list_cache_scope().await?)),
        CacheScopeVerb::Set {
            class,
            layers_json,
            ttl_secs,
        } => {
            admin
                .set_cache_scope(
                    actor,
                    CacheScopeRow {
                        class: class.clone(),
                        layers_json,
                        ttl_secs,
                    },
                )
                .await?;
            Ok(format!("cache scope for '{class}' set"))
        }
    }
}

fn pretty<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "null".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account_id;

    /// CLI parity: every §19 endpoint section parses as an `admin` noun.
    #[test]
    fn every_endpoint_section_has_a_cli_noun() {
        let cases: &[&[&str]] = &[
            &["admin", "domains", "list"],
            &["admin", "users", "provision", "example.com", "alice"],
            &["admin", "security-policy", "show"],
            &["admin", "integrations", "list"],
            &["admin", "observability", "show"],
            &["admin", "appearance", "show"],
            &["admin", "cache-scope", "list"],
        ];
        for argv in cases {
            AdminCli::try_parse_from(*argv)
                .unwrap_or_else(|e| panic!("failed to parse {argv:?}: {e}"));
        }
    }

    #[test]
    fn observability_covers_audit_and_ban_verbs() {
        for argv in [
            vec!["admin", "observability", "audit", "--limit", "10"],
            vec!["admin", "observability", "audit-export"],
            vec!["admin", "observability", "bans"],
            vec![
                "admin",
                "observability",
                "ban",
                "1.2.3.4",
                "--reason",
                "abuse",
            ],
            vec!["admin", "observability", "unban", "1.2.3.4"],
        ] {
            AdminCli::try_parse_from(&argv).unwrap_or_else(|e| panic!("{argv:?}: {e}"));
        }
    }

    #[tokio::test]
    async fn dispatch_provision_then_show_quota() {
        let admin = Admin::in_memory();
        let cli = AdminCli::try_parse_from([
            "admin",
            "users",
            "provision",
            "example.com",
            "alice",
            "--bytes-limit",
            "1000",
            "--msg-limit",
            "20",
        ])
        .unwrap();
        let out = run(&admin, cli.command, "root").await.unwrap();
        assert!(out.contains("provisioned"));
        assert_eq!(
            admin
                .get_quota(&account_id("alice", "example.com"))
                .await
                .unwrap(),
            Some(Quota {
                bytes_limit: 1000,
                msg_limit: 20
            })
        );
    }

    /// The flags for stored values with no defined meaning are gone: clap refuses
    /// them rather than accepting and storing them.
    #[test]
    fn removed_flags_are_refused() {
        // Controls: the same commands parse without the removed flags.
        AdminCli::try_parse_from(["admin", "security-policy", "set"]).unwrap();
        AdminCli::try_parse_from(["admin", "domains", "create", "example.com"]).unwrap();
        AdminCli::try_parse_from(["admin", "appearance", "show"]).unwrap();
        for argv in [
            vec!["admin", "security-policy", "set", "--min-tls", "1.3"],
            vec!["admin", "security-policy", "set", "--require-2fa", "true"],
            vec!["admin", "security-policy", "set", "--capture-policy", "off"],
            vec!["admin", "security-policy", "set", "--argon2-m-cost", "1"],
            vec!["admin", "security-policy", "set", "--argon2-t-cost", "1"],
            vec!["admin", "security-policy", "set", "--argon2-p-cost", "1"],
            vec![
                "admin",
                "domains",
                "create",
                "example.com",
                "--upstream",
                "{}",
            ],
            vec!["admin", "domains", "create", "example.com", "--allow", "a"],
            vec!["admin", "domains", "create", "example.com", "--block", "b"],
            vec!["admin", "appearance", "set", "--theme", "x"],
        ] {
            assert!(
                AdminCli::try_parse_from(&argv).is_err(),
                "{argv:?} must not parse"
            );
        }
    }

    #[tokio::test]
    async fn integrations_list_does_not_claim_a_state_for_ldap_or_nextcloud() {
        let admin = Admin::in_memory();
        let cli = AdminCli::try_parse_from(["admin", "integrations", "list"]).unwrap();
        let out = run(&admin, cli.command, "root").await.unwrap();
        assert!(out.contains("ldap=unknown"), "{out}");
        assert!(out.contains("nextcloud=unknown"), "{out}");
        assert!(!out.contains("deferred"), "{out}");
    }

    #[tokio::test]
    async fn dispatch_ban_and_list() {
        let admin = Admin::in_memory();
        let cli = AdminCli::try_parse_from([
            "admin",
            "observability",
            "ban",
            "203.0.113.4",
            "--reason",
            "abuse",
        ])
        .unwrap();
        run(&admin, cli.command, "root").await.unwrap();
        assert!(admin.is_banned("203.0.113.4").await.unwrap());
    }
}
