//! The `mailwoman admin <noun> <verb>` CLI (plan §2.5 — GitOps-friendly): one
//! noun per §19 panel section, so an operator can drive provisioning, the audit
//! log and the ban list from config management.
//!
//! Like the panel, it offers no flag for a stored value that has no defined
//! meaning (26.20, t28-e8). The `security-policy set` flags write the policy a
//! running server applies (it re-reads the stored policy every 30 seconds) and are
//! checked as the admin API checks them.
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

use crate::policy::{CapturePolicy, MinTls, SecurityFloor, SecurityPolicyWrite};
use crate::{Admin, AdminError, CacheScopeRow, Quota, UserFeatureFlags};

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
    /// Manage console administrator accounts (not mailbox users).
    Admins {
        #[command(subcommand)]
        verb: AdminAccountVerb,
    },

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
    /// Inspect/update the security policy.
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
    /// The deployment-default appearance (theme, brand name, accent).
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
pub enum AdminAccountVerb {
    List,
    /// Read the new password from standard input, never a command-line argument.
    Create {
        username: String,
        #[arg(long)]
        display_name: Option<String>,
    },
    ResetPassword {
        username: String,
    },
    Disable {
        username: String,
    },
    GrantRole {
        username: String,
        role: String,
        #[arg(long)]
        domain: Vec<String>,
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
    /// Replace upstream configuration with a validated JSON object.
    SetUpstream { name: String, json: String },
    Enable { name: String },
    Disable { name: String },
    Prefer { name: String },
    Unprefer,
    /// Map an alias, or remove it with --remove.
    Alias { name: String, alias: String, #[arg(long)] remove: bool },
    /// Map a web host, or remove it with --remove.
    Host { name: String, host: String, #[arg(long)] remove: bool },
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
    /// Show the policy in force, with the fields an environment variable pins.
    Show,
    /// Change the given fields; the others keep their value.
    Set(PolicySetArgs),
}

#[derive(Debug, Args)]
pub struct PolicySetArgs {
    /// Lowest TLS version for the HTTPS listener and for outbound IMAP, SMTP,
    /// POP3, ManageSieve and proxy-tunnel connections: 1.2 or 1.3. New
    /// connections only. Not LDAP, Redis, Postgres or HTTP fetches.
    #[arg(long, value_parser = parse_min_tls)]
    pub min_tls: Option<MinTls>,
    /// Require a second factor for every account (the global rule of the
    /// "Require two-factor" screen).
    #[arg(long)]
    pub require_2fa: Option<bool>,
    /// Argon2id memory in KiB for new hashes of locally stored account passwords
    /// and recovery codes (19456..=1048576). Existing password hashes are
    /// upgraded at the next successful sign-in.
    #[arg(long)]
    pub argon2_m_cost: Option<u32>,
    /// Argon2id passes (1..=10; 1 needs at least 47104 KiB).
    #[arg(long)]
    pub argon2_t_cost: Option<u32>,
    /// Argon2id lanes (1..=16).
    #[arg(long)]
    pub argon2_p_cost: Option<u32>,
    /// off or protect. protect asks the desktop and Android apps to keep their
    /// window out of screenshots; a browser cannot be asked.
    #[arg(long, value_parser = parse_capture)]
    pub capture_policy: Option<CapturePolicy>,
    /// The least restrictive rendering mode the Mailwoman clients allow: none,
    /// full-sanitized, sanitized-no-media or plain-text.
    #[arg(long, value_parser = parse_floor)]
    pub max_security_floor: Option<FloorArg>,
    /// A JSON array of DLP rules, applied beside MW_DLP_RULES to mail sent
    /// through engine-mode accounts.
    #[arg(long)]
    pub dlp_rules_json: Option<String>,
}

/// `--max-security-floor`: `none` or a rendering mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FloorArg(pub Option<SecurityFloor>);

fn parse_min_tls(s: &str) -> Result<MinTls, String> {
    MinTls::parse(s).ok_or_else(|| "expected 1.2 or 1.3".to_string())
}

fn parse_capture(s: &str) -> Result<CapturePolicy, String> {
    CapturePolicy::parse(s).ok_or_else(|| "expected off or protect".to_string())
}

fn parse_floor(s: &str) -> Result<FloorArg, String> {
    if s == "none" {
        return Ok(FloorArg(None));
    }
    SecurityFloor::parse(s)
        .map(|f| FloorArg(Some(f)))
        .ok_or_else(|| {
            "expected none, full-sanitized, sanitized-no-media or plain-text".to_string()
        })
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
    /// Show the deployment-default appearance (stored values, or the built-in
    /// defaults when nothing is stored).
    Show,
    /// Store the deployment-default appearance. Omitted options keep their
    /// current value. A running server serves it within 30 seconds.
    Set {
        /// Theme id, for example `dark` or `grove-light`.
        #[arg(long)]
        theme: Option<String>,
        /// The product name shown to users.
        #[arg(long)]
        brand_name: Option<String>,
        /// Accent id, for example `blue`; an empty value means the theme's own.
        #[arg(long)]
        accent: Option<String>,
    },
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
        AdminCommand::Admins { .. } => Err(AdminError::Invalid(
            "admin accounts require the mailwoman store dispatcher".into(),
        )),
        AdminCommand::Domains { verb } => domains(admin, verb, actor).await,
        AdminCommand::Users { verb } => users(admin, verb, actor).await,
        AdminCommand::SecurityPolicy { verb } => policy(admin, verb, actor).await,
        AdminCommand::Integrations { verb } => integrations(admin, verb, actor).await,
        AdminCommand::Observability { verb } => observability(admin, verb, actor).await,
        AdminCommand::Appearance { verb } => appearance(admin, verb, actor).await,
        AdminCommand::CacheScope { verb } => cache_scope(admin, verb, actor).await,
    }
}

async fn domains(admin: &Admin, verb: DomainVerb, actor: &str) -> Result<String, AdminError> {
    match verb {
        DomainVerb::SetUpstream { .. } | DomainVerb::Enable { .. } | DomainVerb::Disable { .. }
        | DomainVerb::Prefer { .. } | DomainVerb::Unprefer | DomainVerb::Alias { .. } | DomainVerb::Host { .. } =>
            Err(AdminError::Invalid("This domain command requires the server store adapter.".into())),
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
            let e = admin.effective_security_policy().await?;
            let p = &e.policy;
            Ok(pretty(&serde_json::json!({
                "min_tls": p.min_tls.as_str(),
                "require_2fa": e.require_2fa,
                "argon2_m_cost": p.argon2_m_cost,
                "argon2_t_cost": p.argon2_t_cost,
                "argon2_p_cost": p.argon2_p_cost,
                "capture_policy": p.capture_policy.as_str(),
                "max_security_floor": p.max_security_floor.map(SecurityFloor::as_str),
                "dlp_rules_json": p.dlp_rules_json,
                "pinned_by_env": e.pinned_by_env.iter().map(|f| f.var).collect::<Vec<_>>(),
            })))
        }
        PolicyVerb::Set(args) => {
            let e = admin.effective_security_policy().await?;
            let mut write = SecurityPolicyWrite {
                policy: e.policy,
                require_2fa: e.require_2fa,
            };
            let p = &mut write.policy;
            if let Some(v) = args.min_tls {
                p.min_tls = v;
            }
            if let Some(v) = args.argon2_m_cost {
                p.argon2_m_cost = v;
            }
            if let Some(v) = args.argon2_t_cost {
                p.argon2_t_cost = v;
            }
            if let Some(v) = args.argon2_p_cost {
                p.argon2_p_cost = v;
            }
            if let Some(v) = args.capture_policy {
                p.capture_policy = v;
            }
            if let Some(FloorArg(v)) = args.max_security_floor {
                p.max_security_floor = v;
            }
            if let Some(v) = args.dlp_rules_json {
                p.dlp_rules_json = v;
            }
            if let Some(v) = args.require_2fa {
                write.require_2fa = v;
            }
            admin.write_security_policy(actor, write).await?;
            Ok("security policy stored; a running server applies it within 30 seconds".to_string())
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

async fn appearance(
    admin: &Admin,
    verb: AppearanceVerb,
    actor: &str,
) -> Result<String, AdminError> {
    match verb {
        AppearanceVerb::Show => Ok(pretty(&admin.appearance().await?)),
        AppearanceVerb::Set {
            theme,
            brand_name,
            accent,
        } => {
            let mut next = admin.appearance().await?;
            if let Some(theme) = theme {
                next.theme = theme;
            }
            if let Some(brand_name) = brand_name {
                next.brand_name = brand_name;
            }
            if let Some(accent) = accent {
                next.accent = (!accent.is_empty()).then_some(accent);
            }
            validate_appearance(&next)?;
            admin.set_appearance(actor, next.clone()).await?;
            Ok(pretty(&next))
        }
    }
}

/// The checks the settings registry applies to the three keys
/// (`mw-server/src/settings_registry.rs`): theme and accent ids are lower-case
/// letters, digits and `-` (at most 64 and 32 characters), the brand name is 1 to
/// 64 characters without control characters.
pub fn validate_appearance(a: &crate::Appearance) -> Result<(), AdminError> {
    let slug = |s: &str| {
        s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    };
    if a.theme.is_empty() || a.theme.len() > 64 || !slug(&a.theme) {
        return Err(AdminError::Invalid(
            "theme: lower-case letters, digits and - (1 to 64)".into(),
        ));
    }
    if a.accent
        .as_deref()
        .is_some_and(|x| x.len() > 32 || !slug(x))
    {
        return Err(AdminError::Invalid(
            "accent: lower-case letters, digits and - (at most 32)".into(),
        ));
    }
    if a.brand_name.trim().is_empty()
        || a.brand_name.chars().count() > 64
        || a.brand_name.chars().any(char::is_control)
    {
        return Err(AdminError::Invalid(
            "brand name: 1 to 64 characters, no control characters".into(),
        ));
    }
    Ok(())
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

    /// `security-policy set` writes every field the server applies, refuses an
    /// out-of-range value without storing anything, and `show` reads it back.
    #[tokio::test]
    async fn security_policy_set_writes_the_applied_fields_and_refuses_bad_values() {
        let admin = Admin::in_memory();
        let set = |argv: &[&str]| {
            let mut v = vec!["admin", "security-policy", "set"];
            v.extend_from_slice(argv);
            AdminCli::try_parse_from(v).map(|c| c.command)
        };
        run(
            &admin,
            set(&[
                "--min-tls",
                "1.3",
                "--require-2fa",
                "true",
                "--argon2-m-cost",
                "32768",
                "--argon2-t-cost",
                "3",
                "--capture-policy",
                "protect",
                "--max-security-floor",
                "plain-text",
            ])
            .unwrap(),
            "root",
        )
        .await
        .unwrap();
        let e = admin.effective_security_policy().await.unwrap();
        assert_eq!(e.policy.min_tls, MinTls::V13);
        assert!(e.require_2fa);
        assert_eq!(
            (e.policy.argon2_m_cost, e.policy.argon2_t_cost),
            (32_768, 3)
        );
        assert_eq!(e.policy.capture_policy, CapturePolicy::Protect);
        assert_eq!(e.policy.max_security_floor, Some(SecurityFloor::PlainText));

        // Out of range: refused, nothing changes.
        let err = run(&admin, set(&["--argon2-m-cost", "1024"]).unwrap(), "root")
            .await
            .unwrap_err();
        assert!(matches!(err, AdminError::Invalid(_)), "{err:?}");
        assert_eq!(admin.effective_security_policy().await.unwrap(), e);
        // Values outside a field's set do not parse.
        assert!(set(&["--min-tls", "1.1"]).is_err());
        assert!(set(&["--capture-policy", "block"]).is_err());
        assert!(set(&["--max-security-floor", "true"]).is_err());
        // `none` clears the floor.
        run(
            &admin,
            set(&["--max-security-floor", "none"]).unwrap(),
            "root",
        )
        .await
        .unwrap();
        let show = AdminCli::try_parse_from(["admin", "security-policy", "show"]).unwrap();
        let out = run(&admin, show.command, "root").await.unwrap();
        assert!(out.contains(r#""max_security_floor": null"#), "{out}");
        assert!(out.contains(r#""min_tls": "1.3""#), "{out}");
    }

    /// A field pinned by the environment cannot be changed from the CLI.
    #[tokio::test]
    async fn a_pinned_field_is_refused_by_the_cli() {
        let admin = Admin::in_memory().with_overlay(crate::EnvOverlay {
            min_tls: Some(MinTls::V13),
            ..Default::default()
        });
        let cli = AdminCli::try_parse_from(["admin", "security-policy", "set", "--min-tls", "1.2"])
            .unwrap();
        let err = run(&admin, cli.command, "root").await.unwrap_err();
        assert!(
            matches!(
                err,
                AdminError::Pinned {
                    var: "MW_ADMIN_MIN_TLS",
                    ..
                }
            ),
            "{err:?}"
        );
    }

    /// The flags for stored values with no defined meaning are gone: clap refuses
    /// them rather than accepting and storing them.
    #[test]
    fn removed_flags_are_refused() {
        // Controls: the same commands parse without the removed flags.
        AdminCli::try_parse_from(["admin", "domains", "create", "example.com"]).unwrap();
        AdminCli::try_parse_from(["admin", "appearance", "show"]).unwrap();
        for argv in [
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
            vec!["admin", "appearance", "set", "--upstream", "x"],
        ] {
            assert!(
                AdminCli::try_parse_from(&argv).is_err(),
                "{argv:?} must not parse"
            );
        }
    }

    #[tokio::test]
    async fn appearance_set_is_stored_validated_and_shown() {
        let admin = Admin::in_memory();
        async fn show(admin: &Admin) -> String {
            let cli = AdminCli::try_parse_from(["admin", "appearance", "show"]).unwrap();
            run(admin, cli.command, "root").await.unwrap()
        }
        let out = show(&admin).await;
        assert!(out.contains("\"dark\""), "built-in default is dark: {out}");

        let cli = AdminCli::try_parse_from([
            "admin",
            "appearance",
            "set",
            "--theme",
            "grove-light",
            "--accent",
            "blue",
        ])
        .unwrap();
        run(&admin, cli.command, "root").await.unwrap();
        let out = show(&admin).await;
        assert!(out.contains("grove-light") && out.contains("blue"), "{out}");
        assert!(out.contains("Mailwoman"), "the brand name was kept: {out}");

        let cli = AdminCli::try_parse_from(["admin", "appearance", "set", "--accent", ""]).unwrap();
        run(&admin, cli.command, "root").await.unwrap();
        assert!(!show(&admin).await.contains("blue"));

        for bad in [
            vec!["admin", "appearance", "set", "--theme", "Not A Theme"],
            vec!["admin", "appearance", "set", "--brand-name", ""],
            vec!["admin", "appearance", "set", "--accent", "BLUE"],
        ] {
            let cli = AdminCli::try_parse_from(&bad).unwrap();
            assert!(
                matches!(
                    run(&admin, cli.command, "root").await,
                    Err(AdminError::Invalid(_))
                ),
                "{bad:?}"
            );
        }
        let audit = admin.list_audit(10).await.unwrap();
        assert_eq!(
            audit
                .iter()
                .filter(|e| e.action == "appearance-changed")
                .count(),
            2,
            "one audit row per stored change"
        );
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
