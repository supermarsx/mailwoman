//! The admin capability catalogue (t32-ext §M, t32-cfg §4.20, t32-q §5.3).
//!
//! Every admin route names one [`Capability`] when it is registered
//! (`admin_router::admin_endpoint`), and the request is refused with 403 unless the
//! signed-in admin holds it ([`crate::admin_ctx::AdminCtx`]). The ids are the wire
//! form: `GET /admin/capabilities` serves them, `GET /admin/me` lists the ones the
//! admin holds, `admin_role_capability.capability` stores them, and the audit log
//! records the one a write used.
//!
//! **Scope.** A role is granted for all domains or for a list of domains
//! (`admin_account_role.domains_json`). A capability marked `unscoped` acts on
//! objects that belong to no domain (security policy, plugins, egress, logs,
//! roles, updates, …): a grant limited to some domains does not carry it. The
//! other capabilities act on users, domains, domain settings or domain branding,
//! and a handler checks the target's domain against the grant.
//!
//! **Built-in roles** ([`BUILTIN_ROLES`]) are defined here, not stored: a
//! capability added to the catalogue is part of Super admin at once, and the
//! read-only Auditor gains every new `*.read` id.

use serde::Serialize;

macro_rules! catalogue {
    ($( $(#[$doc:meta])* $variant:ident = $id:literal, $scope:ident; )*) => {
        /// One admin capability. See the module docs.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum Capability {
            $( $(#[$doc])* $variant, )*
        }

        impl Capability {
            /// Every capability, in catalogue order.
            pub const ALL: &'static [Capability] = &[ $( Capability::$variant, )* ];

            /// The wire id (`users.read`).
            pub const fn as_str(self) -> &'static str {
                match self { $( Capability::$variant => $id, )* }
            }

            /// Whether only a grant for all domains carries this capability.
            pub const fn unscoped_only(self) -> bool {
                match self { $( Capability::$variant => catalogue!(@scope $scope), )* }
            }
        }
    };
    (@scope unscoped) => { true };
    (@scope domain) => { false };
}

catalogue! {
    // ── §M ────────────────────────────────────────────────────────────────────
    DashboardRead = "dashboard.read", domain;
    UsersRead = "users.read", domain;
    UsersWrite = "users.write", domain;
    UsersRevokeSessions = "users.revoke_sessions", domain;
    /// Force password change, zero-access, cache wipe (t32-q §5.3: `disabled`
    /// moved to [`Capability::UsersDisable`]).
    UsersFlags = "users.flags", domain;
    UsersQuota = "users.quota", domain;
    UsersCapabilities = "users.capabilities", domain;
    DomainsRead = "domains.read", domain;
    DomainsWrite = "domains.write", domain;
    DomainsDelete = "domains.delete", domain;
    TunnelsRead = "tunnels.read", domain;
    TunnelsWrite = "tunnels.write", unscoped;
    TunnelsTest = "tunnels.test", domain;
    /// Access control, lockout, allowlists.
    SecurityRead = "security.read", unscoped;
    SecurityWrite = "security.write", unscoped;
    SecurityPolicyWrite = "security_policy.write", unscoped;
    TwofaPolicyWrite = "twofa_policy.write", unscoped;
    SsoRead = "sso.read", unscoped;
    SsoWrite = "sso.write", unscoped;
    SettingsRead = "settings.read", domain;
    SettingsWrite = "settings.write", domain;
    SettingsUserValues = "settings.user_values", domain;
    BrandingRead = "branding.read", domain;
    BrandingManage = "branding.manage", domain;
    UserCssReview = "user_css.review", domain;
    AiAssistantRead = "ai_assistant.read", unscoped;
    AiAssistantWrite = "ai_assistant.write", unscoped;
    AiAssistantKill = "ai_assistant.kill", unscoped;
    IntegrationsRead = "integrations.read", unscoped;
    ApiKeysRevoke = "api_keys.revoke", unscoped;
    OauthClientsWrite = "oauth_clients.write", unscoped;
    PluginsRead = "plugins.read", unscoped;
    PluginsWrite = "plugins.write", unscoped;
    UiPluginsWrite = "ui_plugins.write", unscoped;
    EgressWrite = "egress.write", unscoped;
    MaintenanceRun = "maintenance.run", unscoped;
    ServerMetadataWrite = "server_metadata.write", unscoped;
    DiagnosticsRead = "diagnostics.read", unscoped;
    DiagnosticsRun = "diagnostics.run", unscoped;
    DiagnosticsBundle = "diagnostics.bundle", unscoped;
    LogsRead = "logs.read", unscoped;
    LogsDownload = "logs.download", unscoped;
    LogsLevel = "logs.level", unscoped;
    AuditRead = "audit.read", domain;
    AuditExport = "audit.export", domain;
    AdminsRead = "admins.read", unscoped;
    AdminsManage = "admins.manage", unscoped;
    RolesManage = "roles.manage", unscoped;
    // ── t32-ext §3.3 (R–V) ────────────────────────────────────────────────────
    PrivacyRead = "privacy.read", unscoped;
    PrivacyWrite = "privacy.write", unscoped;
    EgressRead = "egress.read", unscoped;
    SievePolicy = "sieve.policy", domain;
    PgpPolicy = "pgp.policy", domain;
    ImapExtensions = "imap.extensions", domain;
    AppearanceManage = "appearance.manage", domain;
    ThemesEdit = "themes.edit", unscoped;
    // ── schedule A2a (e24a) ───────────────────────────────────────────────────
    /// A domain's upstream servers (where its mail is).
    DomainsUpstreamWrite = "domains.upstream_write", unscoped;
    /// Binding an SSH tunnel to a domain or upstream.
    TunnelsBind = "tunnels.bind", unscoped;
    // ── t32-ext §10-§14 (per-area admin policy) ───────────────────────────────
    SmimePolicy = "smime.policy", domain;
    SignaturesManage = "signatures.manage", domain;
    NotificationsManage = "notifications.manage", domain;
    ReaderAnalysis = "reader.analysis", domain;
    ProvidersManage = "providers.manage", unscoped;
    DelegationPolicy = "delegation.policy", domain;
    SharedMailboxesManage = "shared_mailboxes.manage", domain;
    KeyboardPolicy = "keyboard.policy", domain;
    ViewsOrg = "views.org", domain;
    IdentitiesPolicy = "identities.policy", domain;
    TemplatesManage = "templates.manage", domain;
    // ── t32-cfg §4.20 ─────────────────────────────────────────────────────────
    ConfigRead = "config.read", unscoped;
    ConfigExport = "config.export", unscoped;
    ConfigImport = "config.import", unscoped;
    IntegrationsWrite = "integrations.write", unscoped;
    CachePurge = "cache.purge", unscoped;
    ServerTextWrite = "server_text.write", domain;
    MaintenanceRead = "maintenance.read", unscoped;
    MaintenanceWrite = "maintenance.write", unscoped;
    QueueManage = "queue.manage", unscoped;
    JobsRead = "jobs.read", unscoped;
    /// Live metrics, series, drill-down.
    ObservabilityRead = "observability.read", unscoped;
    SessionsRead = "sessions.read", domain;
    SessionsRevoke = "sessions.revoke", domain;
    /// Bans of identity kinds are domain-scoped; IP and geo kinds need an
    /// unscoped grant (the bans handlers check the kind).
    BansRead = "bans.read", domain;
    BansWrite = "bans.write", domain;
    UpdatesRead = "updates.read", unscoped;
    UpdatesCheck = "updates.check", unscoped;
    UpdatesApply = "updates.apply", unscoped;
    BackupsRead = "backups.read", unscoped;
    BackupsRun = "backups.run", unscoped;
    AboutManage = "about.manage", unscoped;
    DiagnosticsSchedule = "diagnostics.schedule", unscoped;
    DiagnosticsMailpath = "diagnostics.mailpath", domain;
    DiagnosticsSimulate = "diagnostics.simulate", domain;
    DiagnosticsJmapSession = "diagnostics.jmap_session", unscoped;
    DataExportRun = "data_export.run", domain;
    // ── t32-q §5.3 ────────────────────────────────────────────────────────────
    UsersCreate = "users.create", domain;
    UsersDisable = "users.disable", domain;
    UsersDelete = "users.delete", domain;
    /// The global new-user policy and each domain's mode.
    SigninPolicy = "signin.policy", unscoped;
    /// One domain's new-user policy and allow patterns.
    DomainsSigninPolicy = "domains.signin_policy", domain;
    UsageRead = "usage.read", domain;
    UsageExport = "usage.export", domain;
}

impl Capability {
    /// The capability with wire id `id`.
    pub fn parse(id: &str) -> Option<Capability> {
        Self::ALL.iter().copied().find(|c| c.as_str() == id)
    }
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for Capability {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

/// One built-in role: read-only, cloneable, seeded into `admin_role` with
/// `builtin = 1` so accounts can be assigned to it.
pub struct BuiltinRole {
    /// `admin_role.id` and `.name`.
    pub id: &'static str,
    pub name: &'static str,
    /// The capabilities the role grants.
    pub capabilities: fn() -> Vec<Capability>,
}

/// The role every account created from the environment credential holds.
pub const SUPER_ADMIN: &str = "super_admin";

/// The five built-in roles (t32-ext §M, t32-cfg §4.20, t32-q §5.3).
pub const BUILTIN_ROLES: &[BuiltinRole] = &[
    BuiltinRole {
        id: SUPER_ADMIN,
        name: "Super admin",
        capabilities: || Capability::ALL.to_vec(),
    },
    BuiltinRole {
        id: "domain_admin",
        name: "Domain admin",
        capabilities: domain_admin,
    },
    BuiltinRole {
        id: "helpdesk",
        name: "Helpdesk",
        capabilities: helpdesk,
    },
    BuiltinRole {
        id: "auditor",
        name: "Auditor",
        capabilities: auditor,
    },
    BuiltinRole {
        id: "security_officer",
        name: "Security officer",
        capabilities: security_officer,
    },
];

/// The built-in role with `id`.
pub fn builtin_role(id: &str) -> Option<&'static BuiltinRole> {
    BUILTIN_ROLES.iter().find(|r| r.id == id)
}

fn domain_admin() -> Vec<Capability> {
    use Capability::*;
    vec![
        DashboardRead,
        UsersRead,
        UsersWrite,
        UsersRevokeSessions,
        UsersFlags,
        UsersQuota,
        UsersCapabilities,
        UsersCreate,
        UsersDisable,
        UsersDelete,
        DomainsRead,
        DomainsWrite,
        DomainsSigninPolicy,
        TunnelsRead,
        TunnelsTest,
        SettingsRead,
        SettingsWrite,
        SettingsUserValues,
        BrandingRead,
        BrandingManage,
        AuditRead,
        SessionsRead,
        SessionsRevoke,
        BansRead,
        BansWrite,
        DiagnosticsMailpath,
        DiagnosticsSimulate,
        DataExportRun,
        UsageRead,
        UsageExport,
    ]
}

fn helpdesk() -> Vec<Capability> {
    use Capability::*;
    vec![
        DashboardRead,
        UsersRead,
        UsersRevokeSessions,
        UsersFlags,
        UsersDisable,
        AuditRead,
        SessionsRead,
        SessionsRevoke,
        BansRead,
        DiagnosticsSimulate,
        UsageRead,
    ]
}

fn auditor() -> Vec<Capability> {
    use Capability::*;
    let mut caps: Vec<Capability> = Capability::ALL
        .iter()
        .copied()
        .filter(|c| c.as_str().ends_with(".read"))
        .collect();
    caps.extend([AuditExport, UsageExport]);
    caps
}

fn security_officer() -> Vec<Capability> {
    use Capability::*;
    vec![
        SecurityRead,
        SecurityWrite,
        SecurityPolicyWrite,
        TwofaPolicyWrite,
        SsoRead,
        UsersRead,
        UsersRevokeSessions,
        UsersDisable,
        SigninPolicy,
        ApiKeysRevoke,
        AuditRead,
        AuditExport,
        LogsRead,
        DiagnosticsRead,
        DiagnosticsRun,
        DiagnosticsSimulate,
        ConfigRead,
        ObservabilityRead,
        SessionsRead,
        SessionsRevoke,
        BansRead,
        BansWrite,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn ids_are_unique_and_round_trip() {
        let mut seen = BTreeSet::new();
        for c in Capability::ALL {
            assert!(seen.insert(c.as_str()), "duplicate id {c}");
            assert_eq!(Capability::parse(c.as_str()), Some(*c));
            let (area, action) = c.as_str().split_once('.').expect("area.action");
            assert!(!area.is_empty() && !action.is_empty(), "{c}");
        }
        assert_eq!(Capability::parse("users.everything"), None);
    }

    #[test]
    fn the_plan_ids_are_all_present() {
        // t32-cfg §4.20 and t32-q §5.3, checked one by one so a rename is caught.
        for id in [
            "config.read",
            "config.export",
            "config.import",
            "integrations.write",
            "cache.purge",
            "server_text.write",
            "maintenance.read",
            "maintenance.write",
            "maintenance.run",
            "queue.manage",
            "jobs.read",
            "observability.read",
            "sessions.read",
            "sessions.revoke",
            "bans.read",
            "bans.write",
            "updates.read",
            "updates.check",
            "updates.apply",
            "backups.read",
            "backups.run",
            "about.manage",
            "diagnostics.schedule",
            "diagnostics.mailpath",
            "diagnostics.simulate",
            "diagnostics.jmap_session",
            "data_export.run",
            "users.create",
            "users.disable",
            "users.delete",
            "signin.policy",
            "domains.signin_policy",
            "usage.read",
            "usage.export",
            "domains.upstream_write",
            "tunnels.bind",
        ] {
            assert!(Capability::parse(id).is_some(), "{id} missing");
        }
    }

    #[test]
    fn builtin_roles_follow_the_plan() {
        let caps = |id: &str| -> BTreeSet<Capability> {
            (builtin_role(id).unwrap().capabilities)()
                .into_iter()
                .collect()
        };
        assert_eq!(caps(SUPER_ADMIN).len(), Capability::ALL.len());
        let auditor = caps("auditor");
        assert!(
            auditor
                .iter()
                .all(|c| c.as_str().ends_with(".read") || c.as_str().ends_with(".export")),
            "the auditor writes nothing"
        );
        assert!(auditor.contains(&Capability::ConfigRead));
        assert!(!auditor.contains(&Capability::ConfigExport));
        let helpdesk = caps("helpdesk");
        assert!(helpdesk.contains(&Capability::UsersDisable));
        assert!(!helpdesk.contains(&Capability::UsersDelete));
        let domain = caps("domain_admin");
        assert!(
            domain.iter().all(|c| !c.unscoped_only()),
            "a domain admin's grant is scoped, so it holds nothing unscoped"
        );
        let officer = caps("security_officer");
        assert!(officer.contains(&Capability::SigninPolicy));
        assert!(!officer.contains(&Capability::AdminsManage));
    }
}
