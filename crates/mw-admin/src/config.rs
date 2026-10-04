//! Admin configuration model (plan §2.5). [`AdminConfig`] round-trips
//! TOML↔struct.
//!
//! **Nothing loads it.** `mw-server` builds `AdminConfig::default()` and reads
//! three variables of its own (`MW_ADMIN_ENABLED`, `MW_ADMIN_USER`,
//! `MW_ADMIN_PASSWORD`). Until 26.20 this module also carried an environment
//! overlay (`apply_env`) for ten more `MW_ADMIN_*` variables; it had no caller, so
//! setting any of them changed nothing and said nothing. The overlay is gone and
//! [`reject_unsupported_env`] names those variables instead, so a deployment that
//! sets one is told at start rather than left believing it took effect.

use serde::{Deserialize, Serialize};

use crate::{AdminError, ObservabilityConfig, SecurityPolicy};

/// Appearance/branding config (§19 appearance section).
///
/// This is the DEPLOYMENT DEFAULT, not a policy. A user with no stored
/// appearance sees these values; the moment they pick a theme, density or accent
/// of their own, their per-account preferences (SPEC §17.3, stored by
/// `crates/mw-server/src/prefs_routes.rs` and served at
/// `GET /api/account/appearance` alongside these defaults) win, and changing the
/// deployment default afterwards does not move them back. Nothing here can force
/// a user's appearance.
///
/// That is deliberate. §17.1 makes the theme a per-user choice, and the theme set
/// includes the high-contrast packs — an operator-enforced theme would be an
/// operator-enforced accessibility regression. An operator who needs the whole
/// deployment on one theme sets the default and leaves it; they do not get a
/// lock, and the admin panel says so rather than implying one.
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
            theme: "grove-light".to_string(),
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
    /// A model field with no reader: the cookie name is the constant
    /// `mw_admin_session` at every site that sets or reads it.
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
            session_cookie: "mw_admin_session".to_string(),
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
/// `MW_ADMIN_PASSWORD` are not here: `mw-server` reads those.
pub const UNSUPPORTED_ENV: &[(&str, &str)] = &[
    (
        "MW_ADMIN_PORT",
        "the admin panel is served on the main listener; there is no separate admin port",
    ),
    (
        "MW_ADMIN_SESSION_COOKIE",
        "the admin session cookie is always named mw_admin_session",
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
        "MW_ADMIN_REQUIRE_2FA",
        "require two-factor from the admin panel's \"Require two-factor\" screen",
    ),
    (
        "MW_ADMIN_MIN_TLS",
        "no minimum TLS version setting exists; the listeners use the rustls defaults",
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

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn the_session_cookie_variable_is_refused_by_name() {
        let AdminError::Config(msg) = rejected(&["MW_ADMIN_SESSION_COOKIE"]).unwrap_err() else {
            panic!("a config error");
        };
        assert!(
            msg.contains("MW_ADMIN_SESSION_COOKIE is set but is not supported"),
            "{msg}"
        );
        assert!(msg.contains("mw_admin_session"), "{msg}");
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

    /// Each of the ten variables the removed overlay used to accept is refused on
    /// its own, so none of them can go back to being silently ignored.
    #[test]
    fn each_former_overlay_variable_is_refused() {
        let former = [
            "MW_ADMIN_SESSION_COOKIE",
            "MW_ADMIN_PORT",
            "MW_ADMIN_LOG_LEVEL",
            "MW_ADMIN_OTLP_DSN",
            "MW_ADMIN_METRICS",
            "MW_ADMIN_REQUIRE_2FA",
            "MW_ADMIN_MIN_TLS",
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
