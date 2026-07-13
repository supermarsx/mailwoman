//! Admin configuration: TOML/env binding (plan §2.5 — GitOps-friendly). The
//! whole panel is described by [`AdminConfig`]; `enabled = false` unmounts the
//! panel (the actual route unmount is e11's — this models the flag). Every field
//! round-trips TOML↔struct and can be overlaid from the environment.

use serde::{Deserialize, Serialize};

use crate::{AdminError, ObservabilityConfig, SecurityPolicy};

/// Appearance/branding config (§19 appearance section).
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

/// The full admin-panel configuration (plan §2.5). Scalar fields precede the
/// `[security]`/`[observability]`/`[appearance]` sub-tables so TOML
/// serialization (which requires values-before-tables) round-trips cleanly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AdminConfig {
    /// `admin.enabled` — `false` unmounts the panel (route unmount is e11's).
    pub enabled: bool,
    /// The separate admin session cookie name (§2.5 separate session domain).
    pub session_cookie: String,
    /// Optional dedicated admin port (separate session domain).
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

    /// Overlay environment variables onto the config (env wins over TOML, plan
    /// §2.5). Recognized:
    /// `MW_ADMIN_ENABLED`, `MW_ADMIN_SESSION_COOKIE`, `MW_ADMIN_PORT`,
    /// `MW_ADMIN_LOG_LEVEL`, `MW_ADMIN_OTLP_DSN`, `MW_ADMIN_METRICS`,
    /// `MW_ADMIN_REQUIRE_2FA`, `MW_ADMIN_MIN_TLS`, `MW_ADMIN_THEME`,
    /// `MW_ADMIN_BRAND`.
    pub fn apply_env(&mut self) {
        apply_env_with(self, |k| std::env::var(k).ok());
    }

    /// Load from an optional TOML source, then overlay the environment.
    pub fn load(toml_src: Option<&str>) -> Result<Self, AdminError> {
        let mut cfg = match toml_src {
            Some(s) => Self::from_toml(s)?,
            None => Self::default(),
        };
        cfg.apply_env();
        Ok(cfg)
    }
}

/// Env overlay with an injectable lookup (keeps the logic testable without
/// touching the process environment).
fn apply_env_with(cfg: &mut AdminConfig, get: impl Fn(&str) -> Option<String>) {
    if let Some(v) = get("MW_ADMIN_ENABLED") {
        cfg.enabled = parse_bool(&v).unwrap_or(cfg.enabled);
    }
    if let Some(v) = get("MW_ADMIN_SESSION_COOKIE") {
        cfg.session_cookie = v;
    }
    if let Some(v) = get("MW_ADMIN_PORT")
        && let Ok(p) = v.parse::<u16>()
    {
        cfg.separate_port = Some(p);
    }
    if let Some(v) = get("MW_ADMIN_LOG_LEVEL") {
        cfg.observability.log_level = v;
    }
    if let Some(v) = get("MW_ADMIN_OTLP_DSN") {
        cfg.observability.otlp_dsn = if v.is_empty() { None } else { Some(v) };
    }
    if let Some(v) = get("MW_ADMIN_METRICS") {
        cfg.observability.metrics_enabled =
            parse_bool(&v).unwrap_or(cfg.observability.metrics_enabled);
    }
    if let Some(v) = get("MW_ADMIN_REQUIRE_2FA") {
        cfg.security.require_2fa = parse_bool(&v).unwrap_or(cfg.security.require_2fa);
    }
    if let Some(v) = get("MW_ADMIN_MIN_TLS") {
        cfg.security.min_tls = v;
    }
    if let Some(v) = get("MW_ADMIN_THEME") {
        cfg.appearance.theme = v;
    }
    if let Some(v) = get("MW_ADMIN_BRAND") {
        cfg.appearance.brand_name = v;
    }
}

fn parse_bool(v: &str) -> Option<bool> {
    match v.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

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

    #[test]
    fn env_overlay_wins() {
        let mut cfg = AdminConfig::default();
        let env: HashMap<&str, &str> = [
            ("MW_ADMIN_ENABLED", "false"),
            ("MW_ADMIN_LOG_LEVEL", "debug"),
            ("MW_ADMIN_OTLP_DSN", "http://c:4317"),
            ("MW_ADMIN_REQUIRE_2FA", "true"),
            ("MW_ADMIN_PORT", "9443"),
        ]
        .into_iter()
        .collect();
        apply_env_with(&mut cfg, |k| env.get(k).map(|s| s.to_string()));
        assert!(!cfg.enabled);
        assert_eq!(cfg.observability.log_level, "debug");
        assert_eq!(cfg.observability.otlp_dsn.as_deref(), Some("http://c:4317"));
        assert!(cfg.security.require_2fa);
        assert_eq!(cfg.separate_port, Some(9443));
    }
}
