//! The knob catalogue (26.20 t32-e70a, worker f10; t32-cfg §1, §2, §4.1, §8 A1/A4/A5).
//!
//! One row per operator knob that is not a settings-registry key: every `MW_*`
//! environment variable the code reads, the build and deployment inputs, the
//! built-in constants an operator could reasonably want, the settings keys and
//! config tables that have their own screens, the CLI actions and the config
//! files. Registry keys ([`crate::settings_registry::REGISTRY`]) are merged in at
//! run time; an `MW_*` variable that pins or stands in for a registry key is
//! listed there (its `env`), not here, so there is one list per fact.
//!
//! Mini-slot `t32-knobs`: a commit that adds an `MW_*` variable or a registry key
//! adds its row in the same commit, or `tests/t32_cfg_knob_coverage.rs` fails
//! (R-CFG 5).
//!
//! # Secrets (t32-cfg DC3, A1)
//! Every row has a [`ValueKind`]. A [`ValueKind::Secret`] value is never shown: the
//! configuration view says only whether it is set. A [`ValueKind::Url`] or
//! [`ValueKind::Dsn`] value is shown with its user name, password, query and
//! fragment removed ([`strip_url`]). This holds for every caller, the read-only
//! Auditor role included, and for every output: effective configuration, export,
//! search and the generated reference (`docs/admin/configuration.md`).

use std::fmt::Write as _;

use serde_json::{Value, json};

use crate::settings_registry::{self, Kind, Precedence, REGISTRY, SettingDef, Text};

/// What a knob is, as far as the console is concerned (t32-cfg §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// Configurable in the console (`configured_by` names where); the variable,
    /// when there is one, pins or stands in for the stored value.
    Ui,
    /// Shown read-only with its reason (DC1).
    ReadOnly,
    /// Refused at start (`mw_admin::config::UNSUPPORTED_ENV`).
    Refused,
    /// An input of one CLI run.
    CliOnly,
    /// Read when a client or shell is built.
    Build,
    /// Read by tests and test harnesses only.
    TestOnly,
    /// Read by the deployment tooling (deploy.sh, compose), not by the server.
    Deploy,
    /// A marker in text the server writes or reads, not a variable.
    Marker,
}

impl Class {
    pub fn as_str(self) -> &'static str {
        match self {
            Class::Ui => "ui",
            Class::ReadOnly => "readOnly",
            Class::Refused => "refused",
            Class::CliOnly => "cliOnly",
            Class::Build => "build",
            Class::TestOnly => "testOnly",
            Class::Deploy => "deploy",
            Class::Marker => "marker",
        }
    }

    /// Whether the configuration view lists rows of this class.
    pub fn shown(self) -> bool {
        !matches!(self, Class::TestOnly | Class::Deploy | Class::Marker)
    }
}

/// How a value may be shown (t32-cfg A1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    /// Never shown; only "set" or "not set".
    Secret,
    /// A URL or a list of URLs: shown without user name, password, query, fragment.
    Url,
    /// A database location: a path, or a URL / key-value DSN shown like [`Self::Url`].
    Dsn,
    /// A host path.
    Path,
    Plain,
}

impl ValueKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ValueKind::Secret => "secret",
            ValueKind::Url => "url",
            ValueKind::Dsn => "dsn",
            ValueKind::Path => "path",
            ValueKind::Plain => "plain",
        }
    }
}

/// Why a knob stays out of the console (t32-cfg DC1 1-4, A19).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// Needed before the database opens or to reach the console (DC1 1).
    Boot,
    /// A root of trust, or a protection a stolen admin session could switch off
    /// to attack users (DC1 2).
    Trust,
    /// A wrong value leaves state in clients that cannot be taken back (DC1 3).
    ClientState,
    /// A CLI, build or test input, or a protocol or safety constant (DC1 4).
    Input,
    /// Refused at start: nothing applies it.
    Refused,
}

impl Reason {
    /// The `reasonId` the console translates (`admin-config.ftl`).
    pub fn id(self) -> &'static str {
        match self {
            Reason::Boot => "dc1-boot",
            Reason::Trust => "dc1-trust",
            Reason::ClientState => "dc1-client-state",
            Reason::Input => "dc1-input",
            Reason::Refused => "refused",
        }
    }

    /// The English sentence the generated reference prints.
    pub fn text(self) -> &'static str {
        match self {
            Reason::Boot => "needed before the database opens or to reach the console",
            Reason::Trust => {
                "a root of trust, or a protection a stolen admin session could switch off"
            }
            Reason::ClientState => "a wrong value leaves state in clients that cannot be undone",
            Reason::Input => "a CLI, build or test input, or a protocol or safety constant",
            Reason::Refused => "refused at start because nothing applies it",
        }
    }
}

/// Catalogue groups, in the order the console and the reference list them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Group {
    Listener,
    Storage,
    Admin,
    Signin,
    Integrations,
    Mail,
    Observability,
    Constants,
    StoredConfig,
    Cli,
    Build,
    Deploy,
    Test,
}

impl Group {
    pub const ALL: &'static [Group] = &[
        Group::Listener,
        Group::Storage,
        Group::Admin,
        Group::Signin,
        Group::Integrations,
        Group::Mail,
        Group::Observability,
        Group::Constants,
        Group::StoredConfig,
        Group::Cli,
        Group::Build,
        Group::Deploy,
        Group::Test,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Group::Listener => "listener",
            Group::Storage => "storage",
            Group::Admin => "admin",
            Group::Signin => "signin",
            Group::Integrations => "integrations",
            Group::Mail => "mail",
            Group::Observability => "observability",
            Group::Constants => "constants",
            Group::StoredConfig => "stored",
            Group::Cli => "cli",
            Group::Build => "build",
            Group::Deploy => "deploy",
            Group::Test => "test",
        }
    }

    fn heading(self) -> &'static str {
        match self {
            Group::Listener => "Listener, TLS and HTTP headers",
            Group::Storage => "Storage and host paths",
            Group::Admin => "Admin console bootstrap",
            Group::Signin => "Sign-in, identity providers and sessions",
            Group::Integrations => "Integrations",
            Group::Mail => "Mail, upstreams and content",
            Group::Observability => "Logs, traces and metrics",
            Group::Constants => "Built-in constants",
            Group::StoredConfig => "Stored settings keys, tables, CLI actions and files",
            Group::Cli => "CLI inputs",
            Group::Build => "Build inputs",
            Group::Deploy => "Deployment tooling",
            Group::Test => "Test inputs and markers",
        }
    }
}

/// One catalogue row.
#[derive(Debug, Clone, Copy)]
pub struct Knob {
    /// The environment variable (`MW_DB_PATH`, `RUST_LOG`), or `const:<name>`,
    /// `key:<settings key>`, `table:<name>`, `cli:<action>`, `file:<name>`.
    pub id: &'static str,
    pub group: Group,
    pub class: Class,
    pub kind: ValueKind,
    /// How the variable relates to a stored value (DC2); `None` for rows that are
    /// not variables.
    pub precedence: Option<Precedence>,
    /// `g` global, `d` domain, `u` user, `dev` device.
    pub scope: &'static str,
    /// Whether a change applies while the server runs.
    pub hot: bool,
    /// The default as shown (empty = unset).
    pub default: &'static str,
    /// What it does (English; the console labels it by `labelId`).
    pub what: &'static str,
    pub reason: Option<Reason>,
    /// For [`Class::Ui`] rows: where it is set. For CLI rows: the command.
    pub configured_by: &'static str,
}

impl Knob {
    /// The environment variable this row is, if it is one.
    pub fn env_var(&self) -> Option<&'static str> {
        (!self.id.contains(':')).then_some(self.id)
    }

    /// The anchor in `docs/admin/configuration.md` and the console field id.
    pub fn anchor(&self) -> String {
        anchor_of(self.id)
    }

    /// The translation id of the row's label.
    pub fn label_id(&self) -> String {
        format!("cfg-knob-{}", self.anchor())
    }
}

/// `MW_DB_PATH` → `mw-db-path`, `const:MAX_UPLOAD_BYTES` → `const-max-upload-bytes`.
pub fn anchor_of(id: &str) -> String {
    let mut out = String::with_capacity(id.len());
    for c in id.chars() {
        if c.is_ascii_alphanumeric() || c == '.' {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

// ─── row constructors ───────────────────────────────────────────────────────────

const fn row(
    id: &'static str,
    group: Group,
    class: Class,
    kind: ValueKind,
    default: &'static str,
    what: &'static str,
) -> Knob {
    Knob {
        id,
        group,
        class,
        kind,
        precedence: None,
        scope: "g",
        hot: false,
        default,
        what,
        reason: None,
        configured_by: "",
    }
}

/// A variable read once at start and shown read-only.
const fn boot(
    id: &'static str,
    group: Group,
    kind: ValueKind,
    reason: Reason,
    default: &'static str,
    what: &'static str,
) -> Knob {
    let mut k = row(id, group, Class::ReadOnly, kind, default, what);
    k.precedence = Some(Precedence::Boot);
    k.reason = Some(reason);
    k
}

/// A variable that pins (or stands in for) a value configured in the console.
const fn ui(
    id: &'static str,
    group: Group,
    kind: ValueKind,
    precedence: Precedence,
    default: &'static str,
    what: &'static str,
    configured_by: &'static str,
) -> Knob {
    let mut k = row(id, group, Class::Ui, kind, default, what);
    k.precedence = Some(precedence);
    k.configured_by = configured_by;
    k
}

/// Retained for deployment inspection only; these inputs grant no account access.
const fn ignored_nextcloud_input(id: &'static str, kind: ValueKind, what: &'static str) -> Knob {
    let mut k = row(id, Group::Integrations, Class::ReadOnly, kind, "", what);
    k.reason = Some(Reason::Trust);
    k.configured_by = "Admin → Nextcloud; User settings → Nextcloud";
    k
}

const fn refused(id: &'static str) -> Knob {
    let mut k = row(
        id,
        Group::Admin,
        Class::Refused,
        ValueKind::Plain,
        "",
        "refused at start (see the reason)",
    );
    k.precedence = Some(Precedence::Boot);
    k.reason = Some(Reason::Refused);
    k
}

const fn cli(id: &'static str, kind: ValueKind, what: &'static str, command: &'static str) -> Knob {
    let mut k = row(id, Group::Cli, Class::CliOnly, kind, "", what);
    k.reason = Some(Reason::Input);
    k.configured_by = command;
    k
}

const fn build(id: &'static str, kind: ValueKind, what: &'static str) -> Knob {
    let mut k = row(id, Group::Build, Class::Build, kind, "", what);
    k.reason = Some(Reason::Input);
    k
}

const fn test(id: &'static str, kind: ValueKind, what: &'static str) -> Knob {
    let mut k = row(id, Group::Test, Class::TestOnly, kind, "", what);
    k.reason = Some(Reason::Input);
    k
}

const fn deploy(id: &'static str, what: &'static str) -> Knob {
    let mut k = row(id, Group::Deploy, Class::Deploy, ValueKind::Plain, "", what);
    k.reason = Some(Reason::Input);
    k
}

/// A built-in constant shown read-only.
const fn constant(id: &'static str, value: &'static str, what: &'static str) -> Knob {
    let mut k = row(
        id,
        Group::Constants,
        Class::ReadOnly,
        ValueKind::Plain,
        value,
        what,
    );
    k.reason = Some(Reason::Input);
    k
}

/// A stored setting, table, CLI action or file configured somewhere else.
const fn stored(
    id: &'static str,
    scope: &'static str,
    what: &'static str,
    configured_by: &'static str,
) -> Knob {
    let mut k = row(
        id,
        Group::StoredConfig,
        Class::Ui,
        ValueKind::Plain,
        "",
        what,
    );
    k.scope = scope;
    k.hot = true;
    k.configured_by = configured_by;
    k
}

const fn readonly_item(id: &'static str, what: &'static str, reason: Reason) -> Knob {
    let mut k = row(
        id,
        Group::StoredConfig,
        Class::ReadOnly,
        ValueKind::Plain,
        "",
        what,
    );
    k.reason = Some(reason);
    k
}

const fn hot(mut k: Knob) -> Knob {
    k.hot = true;
    k
}

use Group as Gr;
use Precedence::{Boot as PBoot, Fallback, Pin};
use Reason as R;
use ValueKind::{Dsn, Path, Plain, Secret, Url};

/// Every catalogue row (registry keys are merged in at run time).
pub static CATALOGUE: &[Knob] = &[
    // ── Listener, TLS and HTTP headers ─────────────────────────────────────────
    boot(
        "MW_BIND",
        Gr::Listener,
        Plain,
        R::Boot,
        "0.0.0.0:8080",
        "listen address",
    ),
    boot(
        "MW_BASE_PATH",
        Gr::Listener,
        Plain,
        R::Boot,
        "",
        "URL prefix of every route, the console included",
    ),
    boot(
        "MW_PUBLIC_URL",
        Gr::Listener,
        Url,
        R::Boot,
        "the request's Host header",
        "canonical external origin (cookies, WebAuthn, OAuth)",
    ),
    boot(
        "MW_COOKIE_SECURE",
        Gr::Listener,
        Plain,
        R::Boot,
        "derived from the scheme",
        "the Secure attribute on cookies",
    ),
    boot(
        "MW_MODE",
        Gr::Listener,
        Plain,
        R::Boot,
        "proxy",
        "proxy or engine mode, for the whole process",
    ),
    boot(
        "MW_ACME",
        Gr::Listener,
        Plain,
        R::Boot,
        "false",
        "built-in ACME certificates",
    ),
    boot(
        "MW_ACME_CONTACT",
        Gr::Listener,
        Plain,
        R::Boot,
        "",
        "ACME account contact",
    ),
    boot(
        "MW_ACME_CACHE",
        Gr::Listener,
        Path,
        R::Boot,
        "acme-cache",
        "ACME account and certificate cache directory",
    ),
    boot(
        "MW_ACME_STAGING",
        Gr::Listener,
        Plain,
        R::Boot,
        "false",
        "use the ACME staging directory",
    ),
    boot(
        "MW_TLS_CERT",
        Gr::Listener,
        Path,
        R::Boot,
        "",
        "certificate file (reloaded on SIGHUP)",
    ),
    boot(
        "MW_TLS_KEY",
        Gr::Listener,
        Secret,
        R::Boot,
        "",
        "private key file (reloaded on SIGHUP)",
    ),
    boot(
        "MW_TLS_HANDSHAKE_TIMEOUT_MS",
        Gr::Listener,
        Plain,
        R::Boot,
        "10000",
        "complete inbound TLS handshake deadline in milliseconds (1000..60000; restart required)",
    ),
    boot(
        "MW_TLS_MAX_HANDSHAKES",
        Gr::Listener,
        Plain,
        R::Boot,
        "128",
        "maximum simultaneous inbound TLS handshakes (1..1024; restart required)",
    ),
    boot(
        "MW_PROXY_HEADER_TIMEOUT_MS",
        Gr::Listener,
        Plain,
        R::Boot,
        "5000",
        "complete PROXY header negotiation deadline in milliseconds (1000..30000; restart required)",
    ),
    boot(
        "MW_PROXY_MAX_HANDSHAKES",
        Gr::Listener,
        Plain,
        R::Boot,
        "128",
        "maximum simultaneous PROXY header negotiations (1..1024; restart required)",
    ),
    boot(
        "MW_PROXY_PROTOCOL",
        Gr::Listener,
        Plain,
        R::Trust,
        "off",
        "accept the PROXY protocol from trusted peers",
    ),
    boot(
        "MW_TRUSTED_PROXIES",
        Gr::Listener,
        Plain,
        R::Trust,
        "",
        "peers that may set the client address and scheme",
    ),
    boot(
        "MW_FORWARDED_MODE",
        Gr::Listener,
        Plain,
        R::Trust,
        "off",
        "which forwarding headers are read from trusted peers",
    ),
    boot(
        "MW_HSTS",
        Gr::Listener,
        Plain,
        R::ClientState,
        "false",
        "send Strict-Transport-Security",
    ),
    boot(
        "MW_HSTS_MAX_AGE",
        Gr::Listener,
        Plain,
        R::ClientState,
        "",
        "HSTS max-age in seconds",
    ),
    boot(
        "MW_HSTS_INCLUDE_SUBDOMAINS",
        Gr::Listener,
        Plain,
        R::ClientState,
        "false",
        "HSTS includeSubDomains",
    ),
    boot(
        "MW_HSTS_PRELOAD",
        Gr::Listener,
        Plain,
        R::ClientState,
        "false",
        "HSTS preload",
    ),
    boot(
        "MW_COEP",
        Gr::Listener,
        Plain,
        R::Trust,
        "true",
        "Cross-Origin-Embedder-Policy require-corp",
    ),
    boot(
        "MW_CSRF_STRICT",
        Gr::Listener,
        Plain,
        R::Trust,
        "false (deploy.sh: true)",
        "double-submit CSRF enforcement",
    ),
    boot(
        "MW_NATIVE_ORIGINS",
        Gr::Listener,
        Url,
        R::Trust,
        "",
        "CORS origins of the desktop and mobile shells",
    ),
    // ── Storage and host paths ─────────────────────────────────────────────────
    boot(
        "MW_SERVER_KEY",
        Gr::Storage,
        Secret,
        R::Boot,
        "generated at first start",
        "32-byte key that seals every stored secret",
    ),
    boot(
        "MW_DB_PATH",
        Gr::Storage,
        Dsn,
        R::Boot,
        "mailwoman.db",
        "SQLite path or PostgreSQL URL",
    ),
    boot(
        "MW_WEB_DIR",
        Gr::Storage,
        Path,
        R::Boot,
        "embedded",
        "serve the web client from this directory",
    ),
    boot(
        "MW_REDIS_URL",
        Gr::Storage,
        Url,
        R::Boot,
        "",
        "shared cache",
    ),
    boot(
        "MW_UPLOAD_DIR",
        Gr::Storage,
        Path,
        R::Boot,
        "the database",
        "sealed upload directory",
    ),
    boot(
        "MW_RENDER_BIN",
        Gr::Storage,
        Path,
        R::Boot,
        "",
        "render sandbox binary",
    ),
    boot(
        "MW_RENDER_JAIL",
        Gr::Storage,
        Plain,
        R::Trust,
        "",
        "require the render sandbox jail",
    ),
    boot(
        "MW_PLUGIN_DIR",
        Gr::Storage,
        Path,
        R::Boot,
        "",
        "plugin directory",
    ),
    boot(
        "MW_THIRDPARTY_PLUGIN_DIR",
        Gr::Storage,
        Path,
        R::Boot,
        "",
        "third-party plugin directory",
    ),
    boot(
        "MW_SEARCH_DIR",
        Gr::Storage,
        Path,
        R::Boot,
        "next to the database",
        "search index location (memory or a path)",
    ),
    boot(
        "MW_WKD_DIR",
        Gr::Storage,
        Path,
        R::Boot,
        "",
        "WKD files directory",
    ),
    boot(
        "MW_ABUSE_SPOOL",
        Gr::Storage,
        Path,
        R::Boot,
        "",
        "abuse report (ARF) spool directory",
    ),
    boot(
        "MW_GEOIP_DB",
        Gr::Storage,
        Path,
        R::Boot,
        "",
        "local GeoIP database file",
    ),
    // ── Admin console bootstrap ────────────────────────────────────────────────
    boot(
        "MW_ADMIN_ENABLED",
        Gr::Admin,
        Plain,
        R::Boot,
        "true when user and password are set",
        "admin console on or off",
    ),
    boot(
        "MW_ADMIN_USER",
        Gr::Admin,
        Plain,
        R::Boot,
        "",
        "environment admin user name (break-glass)",
    ),
    boot(
        "MW_ADMIN_PASSWORD",
        Gr::Admin,
        Secret,
        R::Boot,
        "",
        "environment admin password (break-glass)",
    ),
    boot(
        "MW_ADMIN_PATH",
        Gr::Admin,
        Plain,
        R::Boot,
        mw_admin::config::DEFAULT_ADMIN_PATH,
        "console and admin API path",
    ),
    boot(
        "MW_ADMIN_SESSION_COOKIE",
        Gr::Admin,
        Plain,
        R::Boot,
        "mw_admin_session",
        "admin session cookie name",
    ),
    boot(
        "MW_ADMIN_ENV_LOGIN",
        Gr::Admin,
        Plain,
        R::Trust,
        "on",
        "whether the environment admin may sign in (off turns it off)",
    ),
    ui(
        "MW_ADMIN_MIN_TLS",
        Gr::Admin,
        Plain,
        Pin,
        "",
        "pins the stored admin TLS floor",
        "Admin → Security policy",
    ),
    ui(
        "MW_ADMIN_REQUIRE_2FA",
        Gr::Admin,
        Plain,
        Pin,
        "",
        "pins the admin two-factor requirement",
        "Admin → Security policy",
    ),
    refused("MW_ADMIN_PORT"),
    refused("MW_ADMIN_LOG_LEVEL"),
    refused("MW_ADMIN_OTLP_DSN"),
    refused("MW_ADMIN_METRICS"),
    refused("MW_ADMIN_THEME"),
    refused("MW_ADMIN_BRAND"),
    refused("MW_ADMIN_ACCENT"),
    // ── Sign-in, identity providers and sessions ───────────────────────────────
    boot(
        "MW_HEADER_AUTH",
        Gr::Signin,
        Plain,
        R::Trust,
        "false",
        "trust an identity header from listed addresses",
    ),
    boot(
        "MW_HEADER_AUTH_HEADER",
        Gr::Signin,
        Plain,
        R::Trust,
        "",
        "the identity header name",
    ),
    boot(
        "MW_HEADER_AUTH_PASSWORD",
        Gr::Signin,
        Secret,
        R::Trust,
        "",
        "service password for header-authenticated mail access",
    ),
    boot(
        "MW_HEADER_AUTH_JMAP_URL",
        Gr::Signin,
        Url,
        R::Trust,
        "",
        "JMAP upstream for header-authenticated sessions",
    ),
    boot(
        "MW_HEADER_AUTH_TRUSTED_IPS",
        Gr::Signin,
        Plain,
        R::Trust,
        "",
        "addresses allowed to send the identity header",
    ),
    ui(
        "MW_DIRECTORY_BIND_DN",
        Gr::Signin,
        Plain,
        Pin,
        "",
        "directory (GAL/LDAP) service bind DN",
        "Admin → Integrations → Directory",
    ),
    ui(
        "MW_DIRECTORY_BIND_PW",
        Gr::Signin,
        Secret,
        Pin,
        "",
        "directory (GAL/LDAP) service bind password",
        "Admin → Integrations → Directory",
    ),
    boot(
        "MW_WEBAUTHN_ORIGIN",
        Gr::Signin,
        Url,
        R::Trust,
        "derived from the public URL",
        "WebAuthn relying-party origin",
    ),
    boot(
        "MW_WEBAUTHN_RP_ID",
        Gr::Signin,
        Plain,
        R::Trust,
        "derived from the public URL",
        "WebAuthn relying-party id",
    ),
    ui(
        "MW_SESSION_IDLE_SECS",
        Gr::Signin,
        Plain,
        PBoot,
        "1800",
        "mailbox session idle limit (upper bound for the stored policy)",
        "Admin → Sessions",
    ),
    ui(
        "MW_SESSION_ABSOLUTE_SECS",
        Gr::Signin,
        Plain,
        PBoot,
        "1296000",
        "mailbox session absolute limit (upper bound for the stored policy)",
        "Admin → Sessions",
    ),
    // ── Integrations ────────────────────────────────────────────────────────────
    ui(
        "MW_PASSWD_BACKEND",
        Gr::Integrations,
        Plain,
        Pin,
        "",
        "password-change backend; when set the environment integration is in force",
        "Admin → Integrations → Password change",
    ),
    ui(
        "MW_PASSWD_LDAP_URL",
        Gr::Integrations,
        Url,
        Pin,
        "",
        "LDAP password-modify server",
        "Admin → Integrations → Password change",
    ),
    ui(
        "MW_PASSWD_LDAP_STARTTLS",
        Gr::Integrations,
        Plain,
        Pin,
        "",
        "LDAP password-modify STARTTLS",
        "Admin → Integrations → Password change",
    ),
    ui(
        "MW_PASSWD_LDAP_BIND_DN",
        Gr::Integrations,
        Plain,
        Pin,
        "",
        "LDAP password-modify bind DN",
        "Admin → Integrations → Password change",
    ),
    ui(
        "MW_PASSWD_LDAP_BIND_PW",
        Gr::Integrations,
        Secret,
        Pin,
        "",
        "LDAP password-modify bind password",
        "Admin → Integrations → Password change",
    ),
    ui(
        "MW_PASSWD_DOVECOT_URL",
        Gr::Integrations,
        Url,
        Pin,
        "",
        "doveadm HTTP API",
        "Admin → Integrations → Password change",
    ),
    ui(
        "MW_PASSWD_DOVECOT_API_KEY",
        Gr::Integrations,
        Secret,
        Pin,
        "",
        "doveadm HTTP API key",
        "Admin → Integrations → Password change",
    ),
    ui(
        "MW_PASSWD_DOVECOT_COMMAND",
        Gr::Integrations,
        Plain,
        Pin,
        "pw",
        "doveadm API command name sent in the request",
        "Admin → Integrations → Password change",
    ),
    ui(
        "MW_PASSWD_POPPASSD_HOST",
        Gr::Integrations,
        Plain,
        Pin,
        "",
        "poppassd host",
        "Admin → Integrations → Password change",
    ),
    ui(
        "MW_PASSWD_POPPASSD_PORT",
        Gr::Integrations,
        Plain,
        Pin,
        "106",
        "poppassd port",
        "Admin → Integrations → Password change",
    ),
    ui(
        "MW_PASSWD_WEBHOOK_URL",
        Gr::Integrations,
        Url,
        Pin,
        "",
        "password-change webhook",
        "Admin → Integrations → Password change",
    ),
    ui(
        "MW_PASSWD_WEBHOOK_SECRET",
        Gr::Integrations,
        Secret,
        Pin,
        "",
        "password-change webhook signing secret",
        "Admin → Integrations → Password change",
    ),
    ignored_nextcloud_input(
        "MW_NEXTCLOUD_URL",
        Url,
        "deprecated shared Nextcloud URL; ignored by account-owned file operations",
    ),
    ignored_nextcloud_input(
        "MW_NEXTCLOUD_USER",
        Plain,
        "deprecated shared Nextcloud user; ignored, each mailbox must verify its own link",
    ),
    ignored_nextcloud_input(
        "MW_NEXTCLOUD_APP_PASSWORD",
        Secret,
        "deprecated shared Nextcloud password; ignored, never grants mailbox access",
    ),
    ui(
        "MW_BRIDGE_OAUTH_CLIENT_SECRET",
        Gr::Integrations,
        Secret,
        Pin,
        "",
        "bridge OAuth client secret",
        "Admin → OAuth providers",
    ),
    ui(
        "MW_DCR_INITIAL_ACCESS_TOKEN",
        Gr::Integrations,
        Secret,
        Pin,
        "",
        "OAuth dynamic client registration initial access token",
        "Admin → Integrations → Client registration",
    ),
    ui(
        "MW_WEBHOOK_INBOUND_SECRET",
        Gr::Integrations,
        Secret,
        Pin,
        "",
        "inbound webhook HMAC secret",
        "Admin → Integrations → Inbound webhooks",
    ),
    boot(
        "MW_UI_PLUGIN_TRUST_KEYS",
        Gr::Integrations,
        Plain,
        R::Trust,
        "",
        "public keys UI plugin signatures are checked against",
    ),
    boot(
        "MW_ASSIST_LOCAL_PROGRAMS",
        Gr::Integrations,
        Plain,
        R::Trust,
        "",
        "programs a local AI adapter may run on the host",
    ),
    boot(
        "MW_MCP_RESOURCE",
        Gr::Integrations,
        Url,
        R::ClientState,
        "derived from the public origin",
        "MCP OAuth resource identifier; changing it invalidates issued tokens",
    ),
    // ── Mail, upstreams and content ────────────────────────────────────────────
    ui(
        "MW_JMAP_UPSTREAMS",
        Gr::Mail,
        Url,
        Fallback,
        "",
        "allowed JMAP proxy origins (seeds the default upstream)",
        "Admin → Domains → Upstreams",
    ),
    ui(
        "MW_SMTP_HOST",
        Gr::Mail,
        Plain,
        Fallback,
        "the IMAP host",
        "engine submission host when a domain has no SMTP upstream",
        "Admin → Domains → Upstreams",
    ),
    ui(
        "MW_SMTP_PORT",
        Gr::Mail,
        Plain,
        Fallback,
        "587",
        "engine submission port",
        "Admin → Domains → Upstreams",
    ),
    ui(
        "MW_SMTP_SECURITY",
        Gr::Mail,
        Plain,
        Fallback,
        "starttls",
        "engine submission security",
        "Admin → Domains → Upstreams",
    ),
    boot(
        "MW_ENGINE_TLS",
        Gr::Mail,
        Plain,
        R::Input,
        "",
        "forces the engine's TLS mode (test override; the dashboard warns while it is set)",
    ),
    boot(
        "MW_AUTOCONFIG_ALLOW_PRIVATE",
        Gr::Mail,
        Plain,
        R::Trust,
        "false",
        "allow private addresses in autoconfig lookups",
    ),
    ui(
        "MW_ALLOWED_FROMS",
        Gr::Mail,
        Plain,
        PBoot,
        "",
        "server-advertised sender identities",
        "Admin → Identities",
    ),
    ui(
        "MW_DLP_RULES",
        Gr::Mail,
        Plain,
        PBoot,
        "",
        "outbound DLP rules from the environment (applied beside the stored rules)",
        "Admin → Security policy",
    ),
    // ── Logs, traces and metrics ───────────────────────────────────────────────
    boot(
        "MW_LOG",
        Gr::Observability,
        Plain,
        R::Boot,
        "info",
        "log filter at start (the live level is set in the console)",
    ),
    boot(
        "MW_OTLP_ENDPOINT",
        Gr::Observability,
        Url,
        R::Boot,
        "",
        "OTLP trace collector",
    ),
    boot(
        "MW_OTEL_SERVICE",
        Gr::Observability,
        Plain,
        R::Boot,
        "mailwoman",
        "OpenTelemetry service name",
    ),
    ui(
        "MW_METRICS_TOKEN",
        Gr::Observability,
        Secret,
        Pin,
        "",
        "bearer token for /metrics",
        "Admin → Observability",
    ),
    boot(
        "MW_SENTRY_DSN",
        Gr::Observability,
        Secret,
        R::Boot,
        "",
        "Sentry error export",
    ),
    boot(
        "RUST_LOG",
        Gr::Observability,
        Plain,
        R::Boot,
        "",
        "log filter; wins over MW_LOG",
    ),
    // ── Built-in constants ─────────────────────────────────────────────────────
    constant(
        "const:REFRESH_EVERY",
        "30 s",
        "how often each process re-reads admin changes",
    ),
    constant(
        "const:BOOT_REGISTER_CONCURRENCY",
        "4",
        "engine accounts registered in parallel at start",
    ),
    constant(
        "const:BOOT_REGISTER_TIMEOUT",
        "600 s",
        "time limit for registering engine accounts at start",
    ),
    constant(
        "const:MATERIALIZE_BACK_DAYS",
        "2 years",
        "recurring events expanded into the past",
    ),
    constant(
        "const:MATERIALIZE_FWD_DAYS",
        "5 years",
        "recurring events expanded into the future",
    ),
    constant(
        "const:RERANK_TOP_N",
        "built-in",
        "semantic search rerank depth",
    ),
    constant("const:REST_DEFAULT_LIMIT", "50", "REST API page size"),
    constant("const:REST_MAX_LIMIT", "200", "REST API largest page"),
    constant(
        "const:PLUGIN_MAX_MEMORY_MB",
        "1024",
        "ceiling for a plugin's memory limit",
    ),
    constant(
        "const:PLUGIN_MAX_DEADLINE_MS",
        "120000",
        "ceiling for a plugin's time limit",
    ),
    constant(
        "const:OPEN_TRACKING_LIMITS",
        "built-in",
        "open-tracking abuse ceilings",
    ),
    constant("const:PUSH_WAKE_TTL", "built-in", "push wake time to live"),
    constant(
        "const:REALTIME_HEARTBEAT",
        "built-in",
        "event stream heartbeat",
    ),
    constant(
        "const:DISCOVER_LIMITS",
        "built-in",
        "discovery route limits (the route answers 410)",
    ),
    constant(
        "const:DEFAULT_MAX_QUERY_CHANGES",
        "500",
        "JMAP queryChanges limit",
    ),
    constant(
        "const:MAX_APPEARANCE_BYTES",
        "8 KiB",
        "largest stored appearance record",
    ),
    constant(
        "const:SSO_PENDING_TTL",
        "600 s",
        "single sign-on round-trip life",
    ),
    hot(stored(
        "const:MAX_UPLOAD_BYTES",
        "g",
        "largest attachment upload (the limit.* keys set it per user)",
        "Admin → Limits",
    )),
    hot(stored(
        "const:IMAGE_PROXY_LIMITS",
        "g",
        "remote-content cache and rate limits",
        "Admin → Privacy → Remote content",
    )),
    hot(stored(
        "const:TOTP_ISSUER",
        "g",
        "issuer label in authenticator apps (the product name)",
        "Admin → Branding",
    )),
    // ── Stored settings keys, tables, CLI actions and files ────────────────────
    stored(
        "key:v6:admin:security_policy",
        "g",
        "security policy record",
        "Admin → Security policy",
    ),
    stored(
        "key:v6:admin:observability",
        "g",
        "metrics toggle and live log level",
        "Admin → Observability",
    ),
    stored(
        "key:v6:admin:bans",
        "g",
        "IP bans written by the admin API, the CLI and failed admin sign-ins",
        "Admin → Access",
    ),
    stored(
        "key:v6:admin:flags",
        "u",
        "per-account flags (disabled, features)",
        "Admin → Users",
    ),
    stored(
        "key:appearance",
        "u",
        "a user's appearance record",
        "Settings → Appearance; Admin → Settings",
    ),
    stored(
        "key:mdn_policy",
        "u",
        "a user's read-receipt policy",
        "Settings → Mail; Admin → Settings",
    ),
    stored(
        "key:push.quiet_hours",
        "g",
        "old quiet-hours key, read once as the fallback of push.quietHours",
        "Admin → Notifications",
    ),
    stored(
        "table:domains",
        "d",
        "domains, upstreams and allow/block lists",
        "Admin → Domains",
    ),
    stored("table:quotas", "u", "per-user quotas", "Admin → Users"),
    stored("table:admin_users", "g", "admin accounts", "Admin → Admins"),
    stored(
        "table:twofa_policy",
        "d",
        "two-factor policy",
        "Admin → Two-factor",
    ),
    stored(
        "table:assist_config",
        "g",
        "AI assistant configuration",
        "Admin → AI assistant",
    ),
    stored(
        "table:sso_config",
        "g",
        "single sign-on providers",
        "Admin → Single sign-on",
    ),
    stored(
        "table:egress_proxy",
        "g",
        "outbound proxies",
        "Admin → Outbound",
    ),
    stored(
        "table:plugins",
        "g",
        "server plugins and their limits",
        "Admin → Plugins",
    ),
    stored("table:ui_plugins", "g", "UI plugins", "Admin → UI plugins"),
    stored(
        "table:oauth_dcr",
        "g",
        "dynamically registered OAuth clients",
        "Admin → OAuth clients",
    ),
    stored(
        "table:webhooks",
        "u",
        "outbound webhooks",
        "Admin → Integrations",
    ),
    stored("table:api_keys", "u", "API keys", "Admin → Integrations"),
    stored(
        "table:cache_scope",
        "g",
        "cache layers and lifetimes per data class",
        "Admin → Cache scope",
    ),
    stored(
        "table:directory_config",
        "g",
        "directory (LDAP/GAL) endpoints",
        "Admin → Integrations → Directory",
    ),
    stored(
        "table:push_config",
        "g",
        "VAPID key pair",
        "Admin → Notifications",
    ),
    stored(
        "table:integration_config",
        "g",
        "password change, Nextcloud, client registration and inbound webhook integrations",
        "Admin → Integrations",
    ),
    stored(
        "cli:admin",
        "g",
        "mailwoman admin … (every console write keeps CLI parity where it exists)",
        "mailwoman admin",
    ),
    stored(
        "cli:plugin",
        "g",
        "mailwoman plugin list|approve",
        "Admin → Plugins",
    ),
    stored(
        "cli:maintenance rethread",
        "g",
        "re-thread mailboxes",
        "Admin → Maintenance",
    ),
    stored(
        "cli:doctor",
        "g",
        "cache and sandbox posture",
        "Admin → Diagnostics",
    ),
    stored(
        "cli:maintenance gc-uploads",
        "g",
        "remove unreferenced uploads",
        "Admin → Scheduled tasks",
    ),
    stored(
        "cli:maintenance vacuum",
        "g",
        "compact the database",
        "Admin → Scheduled tasks",
    ),
    readonly_item(
        "cli:fonts pull",
        "a build step that writes the self-hosted font files into the web build",
        R::Input,
    ),
    readonly_item("cli:healthcheck", "container health probe", R::Input),
    readonly_item(
        "cli:migrate-store",
        "offline copy between SQLite and PostgreSQL (server stopped)",
        R::Input,
    ),
    readonly_item("cli:mcp-stdio", "MCP client proxy", R::Input),
    readonly_item(
        "cli:password",
        "change one account's password from the CLI",
        R::Input,
    ),
    readonly_item(
        "file:admin.toml",
        "AdminConfig TOML; nothing loads it (the configuration export is the file form)",
        R::Input,
    ),
    readonly_item(
        "file:server.json",
        "desktop provisioning file, written by the OS administrator",
        R::Input,
    ),
    // ── CLI inputs ─────────────────────────────────────────────────────────────
    cli(
        "MW_ACCOUNT_ID",
        Plain,
        "account whose password the CLI changes",
        "mailwoman password",
    ),
    cli(
        "MW_OLD_PASSWORD",
        Secret,
        "current password for the CLI change",
        "mailwoman password",
    ),
    cli(
        "MW_NEW_PASSWORD",
        Secret,
        "new password for the CLI change",
        "mailwoman password",
    ),
    cli(
        "MW_MIGRATE_FROM",
        Dsn,
        "source store of an offline copy",
        "mailwoman migrate-store",
    ),
    cli(
        "MW_MIGRATE_TO",
        Dsn,
        "target store of an offline copy",
        "mailwoman migrate-store",
    ),
    cli(
        "MW_MCP_SERVER",
        Url,
        "server the MCP client proxy talks to",
        "mailwoman mcp-stdio",
    ),
    cli(
        "MW_MCP_TOKEN",
        Secret,
        "token of the MCP client proxy",
        "mailwoman mcp-stdio",
    ),
    // ── Build inputs ───────────────────────────────────────────────────────────
    build(
        "MW_BUILD_SHA",
        Plain,
        "optional source revision embedded in the dashboard server metadata",
    ),
    build(
        "MW_SHELL_SERVER_URL",
        Url,
        "server URL baked into a shell build",
    ),
    build(
        "MW_SHELL_PRODUCT_NAME",
        Plain,
        "product name of a shell build",
    ),
    build(
        "MW_SHELL_IDENTIFIER",
        Plain,
        "bundle identifier of a shell build",
    ),
    build(
        "MW_SHELL_UPDATER_URL",
        Url,
        "desktop update feed of a shell build",
    ),
    build(
        "MW_SHELL_UPDATER_PUBKEY",
        Secret,
        "desktop update signing public key of a shell build",
    ),
    build(
        "MW_I18N_PSEUDO",
        Plain,
        "build the web client with the pseudo-locale",
    ),
    // ── Deployment tooling ─────────────────────────────────────────────────────
    deploy(
        "MW_IMAGE_DIGEST",
        "image digest deploy.sh writes into the app environment",
    ),
    deploy(
        "MW_CADDYFILE",
        "Caddyfile deploy.sh passes to the proxy container",
    ),
    deploy(
        "MW_ACME_CA_ROOT",
        "private ACME CA root deploy.sh passes to the proxy container",
    ),
    deploy(
        "MW_PROXY_APP_IMAGE",
        "app image of the proxy-conformance compose file",
    ),
    deploy(
        "MW_PROXY_MOCK_IMAGE",
        "mock image of the proxy-conformance compose file",
    ),
    // ── Test inputs and markers ────────────────────────────────────────────────
    test("MW_TEST_PG", Dsn, "PostgreSQL for the store tests"),
    test("MW_TEST_REDIS", Plain, "run the Redis tests"),
    test("MW_TEST_REDIS_URL", Url, "Redis for the cache tests"),
    test("MW_TEST_SKIP_LOG", Plain, "quiet test logging"),
    test(
        "MW_REQUIRE_LIVE",
        Plain,
        "fail instead of skipping when a live test service is missing",
    ),
    test("MW_E14_PG_DSN", Secret, "PostgreSQL for the e14 tests"),
    test("MW_MOCK_PORT", Plain, "port of the mock mail server"),
    test(
        "MW_E2E_ADMIN_PATH",
        Plain,
        "console path the e2e suites use",
    ),
    test("MW_IMAP_LIVE_HOST", Plain, "live IMAP test server"),
    test("MW_POP3_LIVE_HOST", Plain, "live POP3 test server"),
    test("MW_SSO_LIVE", Plain, "run the live single sign-on tests"),
    test("MW_T14_LIVE", Plain, "run the t14 live tests"),
    test("MW_T14_TLS_LIVE", Plain, "run the t14 TLS live tests"),
    test("MW_T20_PROXY", Plain, "t20 forwarding test proxy"),
    test("MW_T20_PROXY_BASE", Plain, "t20 forwarding test base path"),
    test("MW_T20_PROXY_KIND", Plain, "t20 forwarding test proxy kind"),
    test("MW_T22_SQUID", Plain, "t22 egress test proxy"),
    {
        let mut k = test(
            "MW_ACTIONS",
            Plain,
            "a marker in the AI assistant's tool prompt, not a variable",
        );
        k.class = Class::Marker;
        k
    },
];

/// The catalogue row for `id`.
pub fn lookup(id: &str) -> Option<&'static Knob> {
    CATALOGUE.iter().find(|k| k.id == id)
}

/// Every environment variable the server knows: catalogue rows and the
/// variables of registry keys.
pub fn known_env_vars() -> impl Iterator<Item = &'static str> {
    CATALOGUE
        .iter()
        .filter_map(Knob::env_var)
        .chain(REGISTRY.iter().filter_map(|d| d.env.map(|(v, _)| v)))
}

/// Why a refused variable is refused (`UNSUPPORTED_ENV`).
pub fn refused_instead(var: &str) -> Option<&'static str> {
    mw_admin::config::UNSUPPORTED_ENV
        .iter()
        .find(|(n, _)| *n == var)
        .map(|(_, instead)| *instead)
}

/// How a registry key's value may be shown.
pub fn registry_value_kind(def: &SettingDef) -> ValueKind {
    match def.kind {
        Kind::Text {
            shape: Text::HttpsUrl | Text::HttpUrl | Text::UpstreamUrl | Text::Contact,
            ..
        } => ValueKind::Url,
        Kind::Links { .. } => ValueKind::Url,
        _ => ValueKind::Plain,
    }
}

/// Redact typed link targets without interpreting their human labels as URLs.
pub fn redact_setting(def: &SettingDef, value: &Value) -> Value {
    if matches!(def.kind, Kind::Links { .. }) {
        return value
            .as_array()
            .map(|links| {
                Value::Array(
                    links
                        .iter()
                        .map(|link| {
                            json!({
                                "label": link.get("label").and_then(Value::as_str),
                                "url": link.get("url").and_then(Value::as_str).map(strip_url),
                            })
                        })
                        .collect(),
                )
            })
            .unwrap_or(Value::Null);
    }
    redact_json(registry_value_kind(def), value)
}

// ─── redaction ──────────────────────────────────────────────────────────────────

/// Whether a variable name says it holds a secret (`…TOKEN`, `…PASSWORD`, `…PW`,
/// `…SECRET`, `…KEY`, `…DSN`). Rows with such a name must be [`ValueKind::Secret`]
/// (`t32_cfg_knob_coverage` checks it with its own copy of the rule).
pub fn secret_name(name: &str) -> bool {
    ["TOKEN", "PASSWORD", "PW", "SECRET", "KEY", "DSN"]
        .iter()
        .any(|s| name.ends_with(s))
}

/// A URL, a list of URLs or a DSN without user name, password, query and
/// fragment. A key-value DSN keeps only `host`, `port`, `dbname` and `sslmode`.
/// Anything else is returned unchanged (a path).
pub fn strip_url(raw: &str) -> String {
    if raw.chars().any(char::is_control) {
        return "[redacted]".into();
    }
    if raw.contains(',') && raw.contains("://") {
        // A comma may belong to a password. Never echo a fragment that is not
        // itself a URL when treating an input as a URL list.
        if raw.split(',').any(|p| !p.trim().contains("://")) {
            return "[redacted]".into();
        }
        return raw
            .split(',')
            .map(|p| strip_one(p.trim()))
            .collect::<Vec<_>>()
            .join(",");
    }
    strip_one(raw.trim())
}

fn strip_one(raw: &str) -> String {
    if raw.starts_with("mailto:") {
        return raw.split(['?', '#']).next().unwrap_or_default().to_string();
    }
    if raw.contains('=') && (raw.contains('\'') || raw.contains('"')) {
        // libpq quoting/escaping is not whitespace tokenisation.
        return "[redacted]".into();
    }
    if let Some((scheme, rest)) = raw.split_once("://") {
        if reqwest::Url::parse(raw).is_err() || scheme.is_empty() {
            return "[redacted]".into();
        }
        // Cut at the first of `/?#` after the authority.
        let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
        let authority = &rest[..end];
        let host = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
        let tail = &rest[end..];
        let path = &tail[..tail.find(['?', '#']).unwrap_or(tail.len())];
        return format!("{scheme}://{host}{path}");
    }
    if raw.contains('=') && raw.split_whitespace().all(|p| p.contains('=')) {
        return raw
            .split_whitespace()
            .filter(|p| {
                let k = p.split('=').next().unwrap_or("").to_ascii_lowercase();
                matches!(k.as_str(), "host" | "port" | "dbname" | "sslmode")
            })
            .collect::<Vec<_>>()
            .join(" ");
    }
    if raw.contains('=') {
        return "[redacted]".into();
    }
    // `user:password@host/db` without a scheme.
    if let Some((_, host)) = raw.rsplit_once('@') {
        return host
            .split(['?', '#'])
            .next()
            .unwrap_or_default()
            .to_string();
    }
    raw.split(['?', '#']).next().unwrap_or_default().to_string()
}

/// The value as any caller may see it.
pub fn redact(kind: ValueKind, raw: Option<&str>) -> Value {
    match (kind, raw) {
        (ValueKind::Secret, r) => json!({ "set": r.is_some() }),
        (_, None) => Value::Null,
        (ValueKind::Url | ValueKind::Dsn, Some(r)) => Value::String(strip_url(r)),
        (ValueKind::Path | ValueKind::Plain, Some(r)) => Value::String(r.to_string()),
    }
}

/// A stored JSON value as any caller may see it: strings in URL-shaped keys are
/// stripped like [`strip_url`].
pub fn redact_json(kind: ValueKind, value: &Value) -> Value {
    match (kind, value) {
        (ValueKind::Secret, v) => json!({ "set": !v.is_null() }),
        (ValueKind::Url | ValueKind::Dsn, Value::String(s)) => Value::String(strip_url(s)),
        (ValueKind::Url | ValueKind::Dsn, Value::Array(values)) => {
            Value::Array(values.iter().map(|v| redact_json(kind, v)).collect())
        }
        (ValueKind::Url | ValueKind::Dsn, Value::Object(values)) => Value::Object(
            values
                .iter()
                .map(|(k, v)| (k.clone(), redact_json(kind, v)))
                .collect(),
        ),
        (_, v) => v.clone(),
    }
}

// ─── effective rows ─────────────────────────────────────────────────────────────

/// The effective configuration rows of the catalogue (the registry part is added
/// by `config_routes`), read through `env` (the server passes the process
/// environment). Rows of classes the view does not show are left out.
pub fn catalogue_rows(env: &dyn Fn(&str) -> Option<String>) -> Vec<Value> {
    CATALOGUE
        .iter()
        .filter(|k| k.class.shown())
        .map(|k| catalogue_row(k, env))
        .collect()
}

fn catalogue_row(k: &Knob, env: &dyn Fn(&str) -> Option<String>) -> Value {
    let raw = k.env_var().and_then(env);
    let set = raw.is_some();
    let (value, source) = match (k.class, k.env_var()) {
        (Class::Refused, _) => (Value::Null, if set { "refused" } else { "default" }),
        (Class::Build, _) => (redact(k.kind, None), "build"),
        (Class::CliOnly, _) => (redact(k.kind, None), "default"),
        (_, Some(_)) => {
            let source = match (set, k.precedence) {
                (false, _) => "default",
                (true, Some(Precedence::Pin)) => "env-pin",
                (true, Some(Precedence::Fallback)) => "env-fallback",
                (true, _) => "boot",
            };
            let value = if set || k.kind == ValueKind::Secret {
                redact(k.kind, raw.as_deref())
            } else if k.default.is_empty() {
                Value::Null
            } else {
                Value::String(k.default.to_string())
            };
            (value, source)
        }
        (_, None) if k.id.starts_with("const:") && k.class == Class::ReadOnly => {
            (Value::String(k.default.to_string()), "build")
        }
        (_, None) => (
            Value::Null,
            if k.class == Class::Ui {
                "stored"
            } else {
                "default"
            },
        ),
    };
    let mut row = json!({
        "id": k.id,
        "group": k.group.as_str(),
        "class": k.class.as_str(),
        "kind": k.kind.as_str(),
        "value": value,
        "source": source,
        "default": k.default,
        "scope": k.scope,
        "hot": k.hot,
        "restartNeeded": !k.hot && k.env_var().is_some() && matches!(k.class, Class::ReadOnly | Class::Ui),
        "labelId": k.label_id(),
        "what": k.what,
        "docs": docs_href(k.id),
    });
    if let Some(var) = k.env_var() {
        row["envVar"] = json!(var);
    }
    if let Some(p) = k.precedence {
        row["precedence"] = json!(p.as_str());
    }
    if let Some(r) = k.reason {
        row["reasonId"] = json!(r.id());
    }
    if !k.configured_by.is_empty() {
        row["configuredBy"] = json!(k.configured_by);
    }
    if k.class == Class::Refused
        && let Some(instead) = refused_instead(k.id)
    {
        row["instead"] = json!(instead);
    }
    row
}

/// The reference link of a row or registry key.
pub fn docs_href(id: &str) -> String {
    format!("docs/admin/configuration.md#{}", anchor_of(id))
}

// ─── generated reference ────────────────────────────────────────────────────────

/// `docs/admin/configuration.md`, generated from the catalogue and the registry.
/// `tests/t32_cfg_knob_coverage.rs` fails when the committed file differs.
pub fn render_docs() -> String {
    let mut out = String::new();
    out.push_str(
        "# Server configuration reference\n\n\
         <!-- Generated from crates/mw-server/src/knobs.rs and settings_registry.rs.\n     \
         Do not edit by hand: run `BLESS=1 cargo test -p mw-server --test t32_cfg_knob_coverage`. -->\n\n\
         Catalogue of declared server, deployment and build inputs and their configuration locations.\n\
         The console's **Server configuration** page shows the same list with each value and\n\
         its source. Values marked unavailable are not introspectable from this running process;\n\
         configuredInput is an environment input, not an assertion about effective runtime state.\n\
         Registry values describe resolved policy, not proof that every subsystem consumes that key.\n\
         Secret values are never shown, only whether they are set. URLs are\n\
         shown without user name, password and query.\n\n\
         Precedence: **pin** = the variable wins and a console write is refused\n\
         (`409 pinnedByEnv`); **fallback** = the stored value wins, the variable is used when\n\
         nothing is stored; **boot** = read once at start, a change needs a restart.\n\n",
    );
    for group in Group::ALL {
        let rows: Vec<&Knob> = CATALOGUE.iter().filter(|k| k.group == *group).collect();
        if rows.is_empty() {
            continue;
        }
        let _ = writeln!(out, "## {}\n", group.heading());
        out.push_str("| Name | Class | Value | Default | Precedence | What | Set in / reason |\n");
        out.push_str("|---|---|---|---|---|---|---|\n");
        for k in rows {
            let set_in = match (k.class, k.reason) {
                (Class::Refused, _) => refused_instead(k.id).unwrap_or("").to_string(),
                (_, Some(r)) if k.configured_by.is_empty() => r.text().to_string(),
                (_, Some(r)) => format!("{}; {}", k.configured_by, r.text()),
                (_, None) => k.configured_by.to_string(),
            };
            let _ = writeln!(
                out,
                "| <a id=\"{}\"></a>`{}` | {} | {} | {} | {} | {} | {} |",
                k.anchor(),
                k.id,
                k.class.as_str(),
                k.kind.as_str(),
                cell(k.default),
                k.precedence.map_or("", Precedence::as_str),
                cell(k.what),
                cell(&set_in),
            );
        }
        out.push('\n');
    }
    out.push_str(
        "## Settings registry keys\n\n\
         Set under **Settings** in the console or through\n\
         the routes under `/settings`. Each key's details are in [settings.md](settings.md).\n\n\
         | Key | Category | Scopes | Default | Value | Environment | Hot | Read by |\n\
         |---|---|---|---|---|---|---|---|\n",
    );
    for def in REGISTRY {
        let env = def
            .env
            .map(|(v, p)| format!("`{v}` ({})", p.as_str()))
            .unwrap_or_default();
        let _ = writeln!(
            out,
            "| <a id=\"{}\"></a>`{}` | {} | {} | {} | {} | {} | {} | {} |",
            anchor_of(def.key),
            def.key,
            def.category,
            def.scopes.as_str(),
            cell(def.default),
            registry_value_kind(def).as_str(),
            env,
            if def.hot() { "yes" } else { "no" },
            cell(def.maps_to),
        );
    }
    out
}

fn cell(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}

/// The registry entry described for the catalogue view (used by the search feed).
pub fn registry_label(def: &SettingDef) -> String {
    format!("{} ({})", def.key, def.category)
}

/// Every registry key, for callers that do not want to import the registry.
pub fn registry_keys() -> impl Iterator<Item = &'static str> {
    REGISTRY.iter().filter(|d| !d.is_family()).map(|d| d.key)
}

/// The registry entry for `key`.
pub fn registry_def(key: &str) -> Option<&'static SettingDef> {
    settings_registry::lookup(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn ids_are_unique_and_not_registry_vars() {
        let mut seen = BTreeSet::new();
        for k in CATALOGUE {
            assert!(seen.insert(k.id), "{} listed twice", k.id);
        }
        for d in REGISTRY {
            if let Some((var, _)) = d.env {
                assert!(
                    lookup(var).is_none(),
                    "{var} belongs to registry key {}; do not list it in knobs.rs",
                    d.key
                );
            }
        }
    }

    #[test]
    fn read_only_rows_have_a_reason() {
        for k in CATALOGUE {
            if k.class != Class::Ui {
                assert!(k.reason.is_some(), "{} needs a reason", k.id);
            }
            if k.class == Class::Ui {
                assert!(!k.configured_by.is_empty(), "{} needs configured_by", k.id);
            }
            if k.class == Class::Refused {
                assert!(
                    refused_instead(k.id).is_some(),
                    "{} is not in UNSUPPORTED_ENV",
                    k.id
                );
            }
        }
        for (var, _) in mw_admin::config::UNSUPPORTED_ENV {
            assert_eq!(lookup(var).map(|k| k.class), Some(Class::Refused), "{var}");
        }
    }

    #[test]
    fn strip_url_removes_credentials_and_query() {
        assert_eq!(
            strip_url("postgres://mw:hunter2@db.internal:5432/mail?sslpassword=x"),
            "postgres://db.internal:5432/mail"
        );
        assert_eq!(
            strip_url("redis://:pw@cache:6379/0"),
            "redis://cache:6379/0"
        );
        assert_eq!(
            strip_url("https://a.example/p?token=t#f, https://u:p@b.example"),
            "https://a.example/p,https://b.example"
        );
        assert_eq!(
            strip_url("host=db port=5432 user=mw password=hunter2 dbname=mail"),
            "host=db port=5432 dbname=mail"
        );
        assert_eq!(strip_url("mw:hunter2@db/mail"), "db/mail");
        assert_eq!(
            strip_url("/var/lib/mailwoman/mail.db"),
            "/var/lib/mailwoman/mail.db"
        );
    }

    #[test]
    fn secret_rows_show_only_set() {
        let env = |v: &str| (v == "MW_SERVER_KEY").then(|| "s3cr3t-value".to_string());
        let rows = catalogue_rows(&env);
        let row = rows.iter().find(|r| r["id"] == "MW_SERVER_KEY").unwrap();
        assert_eq!(row["value"], json!({ "set": true }));
        assert_eq!(row["source"], "boot");
        assert!(!serde_json::to_string(&rows).unwrap().contains("s3cr3t"));
    }

    #[test]
    fn nested_urls_and_ambiguous_dsn_never_echo_credentials() {
        let nested = json!({"urls":["https://user:secret@example.org/path?token=secret#secret"]});
        assert_eq!(
            redact_json(ValueKind::Url, &nested),
            json!({"urls":["https://example.org/path"]})
        );
        for value in [
            "https://user:sec,ret@example.org",
            "host=db password='secret with spaces' dbname=mail",
            "host=db password=secret with spaces",
            "user:secret@db/mail?password=secret#secret",
            "https://example.org\nsecret",
        ] {
            assert!(!strip_url(value).contains("secret"), "{value}");
        }
        let def = registry_def("about.projectLinks").unwrap();
        assert_eq!(
            redact_setting(
                def,
                &json!([{"label":"a=b@example.org","url":"https://user:secret@example.org?token=secret"}])
            ),
            json!([{"label":"a=b@example.org","url":"https://example.org"}])
        );
    }

    #[test]
    fn anchors() {
        assert_eq!(anchor_of("MW_DB_PATH"), "mw-db-path");
        assert_eq!(
            anchor_of("cli:maintenance vacuum"),
            "cli-maintenance-vacuum"
        );
        assert_eq!(
            anchor_of("privacy.watermark.enabled"),
            "privacy.watermark.enabled"
        );
    }
}
