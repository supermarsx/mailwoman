//! Admin configuration model (plan §2.5). [`AdminConfig`] round-trips
//! TOML↔struct.
//!
//! **Nothing loads the TOML.** `mw-server` builds `AdminConfig::default()` and
//! reads three variables of its own (`MW_ADMIN_ENABLED`, `MW_ADMIN_USER`,
//! `MW_ADMIN_PASSWORD`).
//!
//! **Environment overlay.** [`EnvOverlay`] reads the three `MW_ADMIN_*` variables
//! that are applied: `MW_ADMIN_MIN_TLS` and `MW_ADMIN_REQUIRE_2FA` pin a field of
//! the security policy (a set variable wins over the stored value, and the admin
//! API and CLI refuse to change a pinned field), `MW_ADMIN_SESSION_COOKIE`
//! names the admin session cookie ([`session_cookie_name`]), and `MW_ADMIN_PATH`
//! sets the URL path of the admin console and its API ([`admin_path`]). An invalid
//! value is an error naming the variable, and the server does not start.
//!
//! [`reject_unsupported_env`] names the `MW_ADMIN_*` variables nothing applies, so
//! a deployment that sets one is refused at start rather than left believing it
//! took effect.

use std::sync::RwLock;

use serde::{Deserialize, Serialize};

use crate::policy::{MinTls, PinnedField, SecurityPolicy};
use crate::{AdminError, ObservabilityConfig};

/// Appearance/branding config (§19 appearance section).
///
/// This is the DEPLOYMENT DEFAULT. A user with no stored appearance sees these
/// values; the moment they pick a theme, density or accent of their own, their
/// per-account preferences (SPEC §17.3, stored by
/// `crates/mw-server/src/prefs_routes.rs` and served at
/// `GET /api/account/appearance` alongside these defaults) win, and changing the
/// deployment default afterwards does not move them back.
///
/// Since 26.20 (t32 settings model C) the three values are the global values of
/// the settings keys `appearance.theme`, `appearance.brandName` and
/// `appearance.accent`, stored in the database ([`crate::store::AdminBackend::
/// get_deployment_appearance`]); `GET/PUT /admin/appearance` and
/// `mailwoman admin appearance show|set` read and write them. An administrator
/// who must hold users to one theme locks `appearance.theme` in the settings
/// model, which is a separate and explicit decision; this struct only carries the
/// default. `Appearance::default()` is the built-in default when nothing is
/// stored: the `dark` pack, as the web client's own default.
///
/// `theme` is stored as a free string and is NOT validated against the theme
/// registry, which lives in TypeScript (`apps/web/src/theme/registry.ts`) and
/// grows a pack per release. A Rust mirror of that union would drift and start
/// refusing themes the SPA can render; instead the client resolves an unknown id
/// back to its own default (`parseAppearancePrefs`). The cost is that a typo here
/// shows up as "the default did not apply" rather than as a rejected save.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Appearance {
    pub theme: String,
    pub brand_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accent: Option<String>,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            brand_name: "Mailwoman".to_string(),
            accent: None,
        }
    }
}

/// The admin-panel configuration model (plan §2.5). Scalar fields precede the
/// `[security]`/`[observability]`/`[appearance]` sub-tables so TOML
/// serialization (which requires values-before-tables) round-trips cleanly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AdminConfig {
    /// `admin.enabled`. A model field: `mw-server` gates `/admin/*` on its own
    /// `V6Config::admin_enabled` (`MW_ADMIN_ENABLED`).
    pub enabled: bool,
    /// A model field with no reader: the cookie name in force is
    /// [`session_cookie_name`], set from `MW_ADMIN_SESSION_COOKIE` at start.
    pub session_cookie: String,
    /// A model field with no reader: nothing binds a separate admin port.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub separate_port: Option<u16>,
    pub security: SecurityPolicy,
    pub observability: ObservabilityConfig,
    pub appearance: Appearance,
}

impl Default for AdminConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            session_cookie: DEFAULT_SESSION_COOKIE.to_string(),
            separate_port: None,
            security: SecurityPolicy::default(),
            observability: ObservabilityConfig::default(),
            appearance: Appearance::default(),
        }
    }
}

impl AdminConfig {
    /// Parse from a TOML document.
    pub fn from_toml(s: &str) -> Result<Self, AdminError> {
        toml::from_str(s).map_err(|e| AdminError::Config(e.to_string()))
    }

    /// Serialize to a TOML document.
    pub fn to_toml(&self) -> Result<String, AdminError> {
        toml::to_string_pretty(self).map_err(|e| AdminError::Config(e.to_string()))
    }
}

/// The `MW_ADMIN_*` variables that name a setting nothing in the workspace applies,
/// each with what to do instead. `MW_ADMIN_ENABLED`, `MW_ADMIN_USER` and
/// `MW_ADMIN_PASSWORD` are read by `mw-server`; `MW_ADMIN_MIN_TLS`,
/// `MW_ADMIN_REQUIRE_2FA`, `MW_ADMIN_SESSION_COOKIE` and `MW_ADMIN_PATH` by
/// [`EnvOverlay`].
pub const UNSUPPORTED_ENV: &[(&str, &str)] = &[
    (
        "MW_ADMIN_PORT",
        "the admin panel is served on the main listener; there is no separate admin port",
    ),
    (
        "MW_ADMIN_LOG_LEVEL",
        "the log filter is set with MW_LOG at start",
    ),
    (
        "MW_ADMIN_OTLP_DSN",
        "the OTLP collector is set with MW_OTLP_ENDPOINT at start",
    ),
    (
        "MW_ADMIN_METRICS",
        "/metrics is served when MW_METRICS_TOKEN is set at start",
    ),
    (
        "MW_ADMIN_THEME",
        "no deployment-default theme is read from the environment",
    ),
    (
        "MW_ADMIN_BRAND",
        "no brand name is read from the environment",
    ),
    (
        "MW_ADMIN_ACCENT",
        "no accent colour is read from the environment",
    ),
];

/// Refuse to continue when the process environment sets a `MW_ADMIN_*` variable
/// that nothing applies ([`UNSUPPORTED_ENV`]). The error names every such variable
/// that is set, with what to use instead.
///
/// A variable set to the empty string counts as set: an operator who exported it
/// meant something by it.
///
/// Intended to be called once at server start, before anything is bound.
pub fn reject_unsupported_env() -> Result<(), AdminError> {
    reject_unsupported_env_with(|k| std::env::var_os(k).is_some())
}

/// [`reject_unsupported_env`] with an injectable lookup (keeps the logic testable
/// without touching the process environment).
fn reject_unsupported_env_with(is_set: impl Fn(&str) -> bool) -> Result<(), AdminError> {
    let set: Vec<String> = UNSUPPORTED_ENV
        .iter()
        .filter(|(name, _)| is_set(name))
        .map(|(name, instead)| format!("{name} is set but is not supported: {instead}"))
        .collect();
    if set.is_empty() {
        Ok(())
    } else {
        Err(AdminError::Config(format!(
            "{}. Unset {} to start.",
            set.join("; "),
            if set.len() == 1 { "it" } else { "them" }
        )))
    }
}

/// The admin session cookie name when `MW_ADMIN_SESSION_COOKIE` is not set.
pub const DEFAULT_SESSION_COOKIE: &str = "mw_admin_session";

/// Cookie names the admin session must not take: the mailbox session and the CSRF
/// token cookie. Sharing a name with either would let one cookie overwrite the
/// other.
const RESERVED_COOKIES: &[&str] = &["mw_session", "mw_csrf"];

static SESSION_COOKIE: RwLock<String> = RwLock::new(String::new());

/// The name of the admin session cookie in force: `MW_ADMIN_SESSION_COOKIE` as
/// applied at start ([`set_session_cookie_name`]), or [`DEFAULT_SESSION_COOKIE`].
/// Every site in `mw-server` that sets, reads or clears the admin cookie, and the
/// CSRF guard's list of authorising cookies, calls this.
pub fn session_cookie_name() -> String {
    let g = SESSION_COOKIE
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if g.is_empty() {
        DEFAULT_SESSION_COOKIE.to_string()
    } else {
        g.clone()
    }
}

/// Set the process-wide admin cookie name; `None` restores the default. Called by
/// `mw-server` at start with the overlay's value. The name is a deployment
/// constant: nothing changes it while the server runs.
pub fn set_session_cookie_name(name: Option<&str>) {
    *SESSION_COOKIE
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = name.unwrap_or("").to_string();
}

/// The admin console's path when `MW_ADMIN_PATH` is not set.
pub const DEFAULT_ADMIN_PATH: &str = "/admin";

/// First path segments the admin console must not take, compared without regard to
/// case: every prefix `mw-server` mounts a route under, the files the web build
/// puts at the root, and the paths the server generates itself. An admin path under
/// one of them would shadow, or be shadowed by, what is already there.
///
/// `tests/t32_h15_admin_path.rs` in `mw-server` checks that every route prefix the
/// router mounts is listed here.
pub const RESERVED_ADMIN_SEGMENTS: &[&str] = &[
    // Routes (`mw-server/src/**`).
    "api",
    "jmap",
    "oauth",
    "oidc",
    ".well-known",
    "dav",
    "caldav",
    "carddav",
    "mcp",
    "metrics",
    "healthz",
    "errors",
    "t",
    // Generated by the server.
    "__mw_base.js",
    "__mw_error.css",
    // The web build's root (`apps/web/dist`) and what is installed next to it.
    "assets",
    "fonts",
    "icons",
    "themes",
    "index.html",
    "sw.js",
    "manifest.webmanifest",
    "favicon.ico",
    "favicon.svg",
    "apple-touch-icon.png",
    "pdf.worker.mjs",
    "robots.txt",
    "font-licenses",
    "workspace-popup.html",
];

static ADMIN_PATH: RwLock<String> = RwLock::new(String::new());

/// The admin console's path in force, relative to the deploy prefix
/// (`MW_BASE_PATH`): `MW_ADMIN_PATH` as applied at start ([`set_admin_path`]), or
/// [`DEFAULT_ADMIN_PATH`]. It starts with `/` and has no trailing slash.
///
/// This is the one place the admin prefix comes from. The console page, every
/// admin API route ([`admin_route`]), the CSRF exemption of the admin sign-in and
/// the error page's link to the console are all built from it; no `"/admin"`
/// literal is used for any of them (`mw-server`'s `tests/t32_h15_admin_path.rs`
/// fails on one).
pub fn admin_path() -> String {
    let g = ADMIN_PATH
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if g.is_empty() {
        DEFAULT_ADMIN_PATH.to_string()
    } else {
        g.clone()
    }
}

/// An admin route: `suffix` (starting with `/`, e.g. `/login`) under
/// [`admin_path`].
pub fn admin_route(suffix: &str) -> String {
    format!("{}{suffix}", admin_path())
}

/// Set the process-wide admin path; `None` restores the default. Called by
/// `mw-server` at start with the overlay's value, before the router is built. The
/// path is a deployment constant: nothing changes it while the server runs.
pub fn set_admin_path(path: Option<&str>) {
    *ADMIN_PATH
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = path.unwrap_or("").to_string();
}

/// Check an `MW_ADMIN_PATH` value: `/` followed by one to four segments of
/// `A-Z a-z 0-9 . _ ~ -`, no `.` or `..` segment, no empty segment, no trailing
/// slash, and a first segment that is not in [`RESERVED_ADMIN_SEGMENTS`].
/// Returns the path unchanged, or why it is refused.
pub fn validate_admin_path(raw: &str) -> Result<String, String> {
    let Some(body) = raw.strip_prefix('/') else {
        return Err("it must start with /".into());
    };
    let segments: Vec<&str> = body.split('/').collect();
    if body.is_empty() || segments.len() > 4 {
        return Err("it must have one to four segments".into());
    }
    for seg in &segments {
        if seg.is_empty() {
            return Err("it must not have an empty segment or a trailing /".into());
        }
        if *seg == "." || *seg == ".." {
            return Err("it must not have a . or .. segment".into());
        }
        if !seg
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._~-".contains(&b))
        {
            return Err("each segment may use only A-Z a-z 0-9 . _ ~ -".into());
        }
    }
    let first = segments[0];
    if RESERVED_ADMIN_SEGMENTS
        .iter()
        .any(|r| r.eq_ignore_ascii_case(first))
    {
        return Err(format!("/{first} is already used by the server"));
    }
    Ok(raw.to_string())
}

/// The `MW_ADMIN_*` variables that are applied, parsed and checked.
///
/// `min_tls` and `require_2fa` pin the policy field of the same meaning: while the
/// variable is set its value is the one in force whatever is stored, and the admin
/// API and CLI refuse to change the field. Unsetting the variable brings the stored
/// value back at the next start. `session_cookie` names the admin cookie, and
/// `admin_path` is where the console and its API are served.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EnvOverlay {
    pub min_tls: Option<MinTls>,
    pub require_2fa: Option<bool>,
    pub session_cookie: Option<String>,
    pub admin_path: Option<String>,
}

impl EnvOverlay {
    pub const MIN_TLS_VAR: &'static str = "MW_ADMIN_MIN_TLS";
    pub const REQUIRE_2FA_VAR: &'static str = "MW_ADMIN_REQUIRE_2FA";
    pub const SESSION_COOKIE_VAR: &'static str = "MW_ADMIN_SESSION_COOKIE";
    pub const ADMIN_PATH_VAR: &'static str = "MW_ADMIN_PATH";

    /// Read the overlay from the process environment.
    ///
    /// # Errors
    /// [`AdminError::Config`] naming the variable whose value is invalid.
    pub fn from_env() -> Result<Self, AdminError> {
        Self::from_lookup(|k| std::env::var(k).ok())
    }

    /// [`Self::from_env`] with an injectable lookup. A variable set to the empty
    /// string is invalid, as for [`reject_unsupported_env`]: an operator who
    /// exported it meant something by it.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, AdminError> {
        let bad = |var: &str, value: &str, want: &str| {
            AdminError::Config(format!("{var}={value:?} is not valid: expected {want}"))
        };
        let min_tls = match lookup(Self::MIN_TLS_VAR) {
            None => None,
            Some(v) => Some(
                MinTls::parse(v.trim()).ok_or_else(|| bad(Self::MIN_TLS_VAR, &v, "1.2 or 1.3"))?,
            ),
        };
        let require_2fa = match lookup(Self::REQUIRE_2FA_VAR) {
            None => None,
            Some(v) => Some(match v.trim().to_ascii_lowercase().as_str() {
                "1" | "true" | "yes" | "on" => true,
                "0" | "false" | "no" | "off" => false,
                _ => return Err(bad(Self::REQUIRE_2FA_VAR, &v, "true or false")),
            }),
        };
        let session_cookie = match lookup(Self::SESSION_COOKIE_VAR) {
            None => None,
            Some(v) => {
                let name = v.trim();
                let token = !name.is_empty()
                    && name.len() <= 64
                    && name
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"_-.".contains(&b));
                if !token {
                    return Err(bad(
                        Self::SESSION_COOKIE_VAR,
                        &v,
                        "1 to 64 characters from A-Z a-z 0-9 _ - .",
                    ));
                }
                if RESERVED_COOKIES.contains(&name) {
                    return Err(AdminError::Config(format!(
                        "{}={v:?} is not valid: another Mailwoman cookie has that name",
                        Self::SESSION_COOKIE_VAR
                    )));
                }
                Some(name.to_string())
            }
        };
        // Not trimmed: a path with surrounding whitespace is a mistake to report,
        // not one to guess at.
        let admin_path = match lookup(Self::ADMIN_PATH_VAR) {
            None => None,
            Some(v) => Some(validate_admin_path(&v).map_err(|why| {
                AdminError::Config(format!(
                    "{}={v:?} is not valid: {why}",
                    Self::ADMIN_PATH_VAR
                ))
            })?),
        };
        Ok(Self {
            min_tls,
            require_2fa,
            session_cookie,
            admin_path,
        })
    }

    /// The policy fields this overlay pins, by wire name.
    pub fn pinned(&self) -> Vec<PinnedField> {
        let mut out = Vec::new();
        if self.min_tls.is_some() {
            out.push(PinnedField {
                field: "minTls",
                var: Self::MIN_TLS_VAR,
            });
        }
        if self.require_2fa.is_some() {
            out.push(PinnedField {
                field: "require2fa",
                var: Self::REQUIRE_2FA_VAR,
            });
        }
        out
    }

    /// Put the pinned values over a stored record.
    pub fn apply(&self, policy: &mut SecurityPolicy, require_2fa: &mut bool) {
        if let Some(v) = self.min_tls {
            policy.min_tls = v;
        }
        if let Some(v) = self.require_2fa {
            *require_2fa = v;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn overlay(set: &[(&str, &str)]) -> Result<EnvOverlay, AdminError> {
        EnvOverlay::from_lookup(|k| {
            set.iter()
                .find(|(n, _)| *n == k)
                .map(|(_, v)| (*v).to_string())
        })
    }

    #[test]
    fn an_empty_environment_pins_nothing() {
        let o = overlay(&[]).unwrap();
        assert_eq!(o, EnvOverlay::default());
        assert!(o.pinned().is_empty());
    }

    #[test]
    fn the_overlay_parses_and_pins_what_is_set() {
        let o = overlay(&[
            ("MW_ADMIN_MIN_TLS", "1.3"),
            ("MW_ADMIN_REQUIRE_2FA", "TRUE"),
            ("MW_ADMIN_SESSION_COOKIE", "corp_admin"),
        ])
        .unwrap();
        assert_eq!(o.min_tls, Some(MinTls::V13));
        assert_eq!(o.require_2fa, Some(true));
        assert_eq!(o.session_cookie.as_deref(), Some("corp_admin"));
        let pinned: Vec<_> = o.pinned().iter().map(|p| (p.field, p.var)).collect();
        assert_eq!(
            pinned,
            vec![
                ("minTls", "MW_ADMIN_MIN_TLS"),
                ("require2fa", "MW_ADMIN_REQUIRE_2FA")
            ]
        );
        let mut p = SecurityPolicy::default();
        let mut r = false;
        o.apply(&mut p, &mut r);
        assert_eq!(p.min_tls, MinTls::V13);
        assert!(r);
    }

    #[test]
    fn an_invalid_value_is_an_error_naming_the_variable() {
        for (var, value) in [
            ("MW_ADMIN_MIN_TLS", "banana"),
            ("MW_ADMIN_MIN_TLS", "1.1"),
            ("MW_ADMIN_MIN_TLS", ""),
            ("MW_ADMIN_REQUIRE_2FA", "maybe"),
            ("MW_ADMIN_SESSION_COOKIE", ""),
            ("MW_ADMIN_SESSION_COOKIE", "a b"),
            ("MW_ADMIN_SESSION_COOKIE", "x;y"),
            ("MW_ADMIN_SESSION_COOKIE", "mw_session"),
            ("MW_ADMIN_SESSION_COOKIE", "mw_csrf"),
        ] {
            let AdminError::Config(msg) = overlay(&[(var, value)]).unwrap_err() else {
                panic!("a config error for {var}={value}");
            };
            assert!(msg.starts_with(&format!("{var}=")), "{msg}");
        }
    }

    #[test]
    fn the_cookie_name_defaults_and_can_be_set_and_restored() {
        assert_eq!(session_cookie_name(), DEFAULT_SESSION_COOKIE);
        set_session_cookie_name(Some("corp_admin"));
        assert_eq!(session_cookie_name(), "corp_admin");
        set_session_cookie_name(None);
        assert_eq!(session_cookie_name(), DEFAULT_SESSION_COOKIE);
    }

    #[test]
    fn the_admin_path_defaults_and_can_be_set_and_restored() {
        assert_eq!(admin_path(), DEFAULT_ADMIN_PATH);
        assert_eq!(admin_route("/login"), "/admin/login");
        set_admin_path(Some("/ops-7f3k"));
        assert_eq!(admin_path(), "/ops-7f3k");
        assert_eq!(admin_route("/users/a1/flags"), "/ops-7f3k/users/a1/flags");
        set_admin_path(None);
        assert_eq!(admin_path(), DEFAULT_ADMIN_PATH);
    }

    #[test]
    fn admin_paths_are_validated() {
        for ok in [
            "/admin",
            "/ops-7f3k",
            "/a.b_c~d-e",
            "/x/y/z/w",
            "/Console9",
            "/apis",
        ] {
            assert_eq!(validate_admin_path(ok).as_deref(), Ok(ok), "{ok}");
        }
        for bad in [
            "",
            "/",
            "admin",
            "/ops/",
            "//ops",
            "/ops//x",
            "/ops/../x",
            "/./ops",
            "/a/b/c/d/e",
            "/ops?x",
            "/ops#x",
            "/ops x",
            "/ops%2f",
            " /ops",
            "/api",
            "/API",
            "/api/x",
            "/Assets",
            "/.well-known",
            "/sw.js",
            "/__mw_error.css",
            "/t",
            "/dav",
            "/caldav",
        ] {
            assert!(validate_admin_path(bad).is_err(), "{bad:?} accepted");
        }
    }

    #[test]
    fn the_overlay_reads_the_admin_path() {
        assert_eq!(overlay(&[]).unwrap().admin_path, None);
        assert_eq!(
            overlay(&[("MW_ADMIN_PATH", "/ops-7f3k")])
                .unwrap()
                .admin_path
                .as_deref(),
            Some("/ops-7f3k")
        );
        let err = overlay(&[("MW_ADMIN_PATH", "/api")])
            .unwrap_err()
            .to_string();
        assert!(err.contains("MW_ADMIN_PATH"), "{err}");
        assert!(err.contains("already used"), "{err}");
    }

    #[test]
    fn toml_round_trips() {
        let cfg = AdminConfig::default();
        let s = cfg.to_toml().unwrap();
        let back = AdminConfig::from_toml(&s).unwrap();
        assert_eq!(cfg, back);
    }

    #[test]
    fn toml_round_trips_with_options_set() {
        let mut cfg = AdminConfig {
            enabled: false,
            separate_port: Some(9443),
            ..Default::default()
        };
        cfg.observability.otlp_dsn = Some("http://collector:4317".to_string());
        cfg.observability.sentry_dsn = Some("https://key@sentry.example/1".to_string());
        cfg.appearance.accent = Some("#3355ff".to_string());
        let s = cfg.to_toml().unwrap();
        let back = AdminConfig::from_toml(&s).unwrap();
        assert_eq!(cfg, back);
    }

    #[test]
    fn partial_toml_uses_defaults() {
        // Only `enabled` supplied; every other field falls back to Default.
        let cfg = AdminConfig::from_toml("enabled = false\n").unwrap();
        assert!(!cfg.enabled);
        assert_eq!(cfg.session_cookie, "mw_admin_session");
        assert_eq!(cfg.appearance.brand_name, "Mailwoman");
    }

    fn rejected(set: &[&str]) -> Result<(), AdminError> {
        reject_unsupported_env_with(|k| set.contains(&k))
    }

    #[test]
    fn an_environment_without_unsupported_variables_passes() {
        assert!(rejected(&[]).is_ok());
        // The three variables `mw-server` reads are not refused.
        assert!(rejected(&["MW_ADMIN_ENABLED", "MW_ADMIN_USER", "MW_ADMIN_PASSWORD"]).is_ok());
    }

    #[test]
    fn the_admin_port_is_refused_by_name() {
        let err = rejected(&["MW_ADMIN_PORT"]).unwrap_err();
        let AdminError::Config(msg) = &err else {
            panic!("a config error, got {err:?}");
        };
        assert!(
            msg.contains("MW_ADMIN_PORT is set but is not supported"),
            "{msg}"
        );
        assert!(msg.contains("no separate admin port"), "{msg}");
        assert!(msg.ends_with("Unset it to start."), "{msg}");
        assert!(!msg.contains("MW_ADMIN_SESSION_COOKIE"), "{msg}");
    }

    /// The three variables the overlay applies are not refused.
    #[test]
    fn the_overlay_variables_are_not_refused() {
        assert!(
            rejected(&[
                "MW_ADMIN_SESSION_COOKIE",
                "MW_ADMIN_MIN_TLS",
                "MW_ADMIN_REQUIRE_2FA"
            ])
            .is_ok()
        );
    }

    #[test]
    fn every_variable_that_is_set_is_named_in_one_error() {
        let AdminError::Config(msg) =
            rejected(&["MW_ADMIN_PORT", "MW_ADMIN_LOG_LEVEL", "MW_ADMIN_BRAND"]).unwrap_err()
        else {
            panic!("a config error");
        };
        for name in ["MW_ADMIN_PORT", "MW_ADMIN_LOG_LEVEL", "MW_ADMIN_BRAND"] {
            assert!(
                msg.contains(&format!("{name} is set but is not supported")),
                "{msg}"
            );
        }
        assert!(msg.ends_with("Unset them to start."), "{msg}");
    }

    /// Each of the seven variables nothing applies is refused on its own, so none
    /// of them can go back to being silently ignored.
    #[test]
    fn each_unapplied_variable_is_refused() {
        let former = [
            "MW_ADMIN_PORT",
            "MW_ADMIN_LOG_LEVEL",
            "MW_ADMIN_OTLP_DSN",
            "MW_ADMIN_METRICS",
            "MW_ADMIN_THEME",
            "MW_ADMIN_BRAND",
            "MW_ADMIN_ACCENT",
        ];
        assert_eq!(UNSUPPORTED_ENV.len(), former.len());
        for name in former {
            let AdminError::Config(msg) = rejected(&[name]).unwrap_err() else {
                panic!("a config error for {name}");
            };
            assert!(msg.starts_with(&format!("{name} is set")), "{msg}");
        }
    }
}
