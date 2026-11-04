//! One reading of "what is the country of this address" for every feature that
//! uses the local country database, and one vocabulary for "it could not be
//! told".
//!
//! `mw_engine::security::CountryLookup` says *database unavailable*, *address not
//! in the database* or *known*. This module adds the third cause that must never
//! be mistaken for either failure: a **local network** address (loopback,
//! private, link-local, shared address space, unique-local IPv6). Those have no
//! country by nature, so no feature may block them for lacking one.
//!
//! The two failure causes are separate on purpose:
//!
//! * [`Outcome::NoDatabase`]: no database is loaded or readable. Every
//!   country feature is effectively inert; this is a setup problem the
//!   administrator should fix (the Security centre says so).
//! * [`Outcome::NotFound`]: a database is loaded and has no entry for the
//!   address (unallocated or reserved ranges, new allocations).
//!
//! What each feature does about them is an administrator setting; see
//! [`Failure`] and the table in `docs/admin/settings.md` (section "When a
//! country cannot be determined").
use crate::AppState;
use mw_engine::security::{CountryLookup, country_lookup};
use serde_json::{Value, json};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Shared default for "no database is loaded".
pub(crate) const NO_DATABASE_KEY: &str = "security.geo.noDatabase";
/// Shared default for "a database is loaded and has no entry for the address".
pub(crate) const NOT_FOUND_KEY: &str = "security.geo.notFound";
/// Session-alert behaviour when the country cannot be determined.
pub(crate) const SESSION_KEY: &str = "security.geo.sessionLookupFailure";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    Known(String),
    /// Loopback, private, link-local, shared (CGNAT) or unique-local address.
    Local,
    NoDatabase,
    NotFound,
}
impl Outcome {
    /// A stable id for logs and audit details.
    pub(crate) fn cause(&self) -> &'static str {
        match self {
            Outcome::Known(_) => "known",
            Outcome::Local => "local-network",
            Outcome::NoDatabase => "no-database",
            Outcome::NotFound => "not-found",
        }
    }
}

/// Loopback, RFC 1918, link-local, 100.64/10 shared space, unique-local IPv6,
/// unspecified. Documentation ranges are not local: they are just unallocated.
pub(crate) fn is_local(ip: IpAddr) -> bool {
    match ip.to_canonical() {
        IpAddr::V4(v) => local_v4(v),
        IpAddr::V6(v) => local_v6(v),
    }
}
fn local_v4(v: Ipv4Addr) -> bool {
    let o = v.octets();
    v.is_loopback()
        || v.is_private()
        || v.is_link_local()
        || v.is_unspecified()
        || (o[0] == 100 && (o[1] & 0xc0) == 64)
}
fn local_v6(v: Ipv6Addr) -> bool {
    let s = v.segments();
    v.is_loopback() || v.is_unspecified() || (s[0] & 0xfe00) == 0xfc00 || (s[0] & 0xffc0) == 0xfe80
}

pub(crate) fn classify(ip: Option<IpAddr>) -> Outcome {
    classify_with(ip, country_lookup(ip))
}
/// [`classify`] for a lookup already made (and for tests).
pub(crate) fn classify_with(ip: Option<IpAddr>, lookup: CountryLookup) -> Outcome {
    if ip.is_some_and(is_local) {
        return Outcome::Local;
    }
    match lookup {
        CountryLookup::Known(c) => Outcome::Known(c),
        CountryLookup::DatabaseUnavailable => Outcome::NoDatabase,
        CountryLookup::Unknown => Outcome::NotFound,
    }
}

/// What a feature does when the country cannot be told. Ordered by severity so
/// a shared default and a per-feature key combine as "the stricter one wins".
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Failure {
    /// Carry on as if the feature did not apply; log it.
    Allow,
    /// Treat the source as a country that is not on the feature's list.
    Unknown,
    /// Refuse (sign-in only; other features cap this at `Unknown`).
    Block,
}
impl Failure {
    pub(crate) fn parse(raw: &str) -> Self {
        match raw {
            "treat-as-unknown" | "unknown" => Failure::Unknown,
            "block" | "deny" => Failure::Block,
            _ => Failure::Allow,
        }
    }
    /// For features where refusing is not safe (the admin-discovery block works
    /// on requests, never on a sign-in): `Block` acts as `Unknown`.
    pub(crate) fn capped(self) -> Self {
        self.min(Failure::Unknown)
    }
}

/// The shared defaults.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Shared {
    pub no_database: Failure,
    pub not_found: Failure,
}
impl Default for Shared {
    fn default() -> Self {
        Shared { no_database: Failure::Allow, not_found: Failure::Allow }
    }
}
impl Shared {
    /// The shared setting for a failure cause (`Allow` for anything else).
    pub(crate) fn for_outcome(&self, outcome: &Outcome) -> Failure {
        match outcome {
            Outcome::NoDatabase => self.no_database,
            Outcome::NotFound => self.not_found,
            _ => Failure::Allow,
        }
    }
}

pub(crate) async fn shared(state: &AppState) -> Shared {
    async fn read(state: &AppState, key: &str) -> Failure {
        crate::settings_registry::effective_for(&state.store, key, &Default::default())
            .await
            .ok()
            .flatten()
            .and_then(|e| e.value.as_str().map(Failure::parse))
            .unwrap_or(Failure::Allow)
    }
    Shared {
        no_database: read(state, NO_DATABASE_KEY).await,
        not_found: read(state, NOT_FOUND_KEY).await,
    }
}

/// Whether a country change should alert when the country is not known.
pub(crate) async fn session_alerts_on_failure(state: &AppState) -> bool {
    crate::settings_registry::effective_for(&state.store, SESSION_KEY, &Default::default())
        .await
        .ok()
        .flatten()
        .and_then(|e| e.value.as_str().map(|v| v == "alert"))
        .unwrap_or(false)
}

/// The country features that need a database, and whether each is switched on.
/// `(id, enabled)`; ids are the web client's translation keys.
pub(crate) async fn feature_states(state: &AppState) -> Vec<(&'static str, bool)> {
    async fn value(state: &AppState, key: &str) -> Option<Value> {
        crate::settings_registry::effective_for(&state.store, key, &Default::default())
            .await
            .ok()
            .flatten()
            .map(|e| e.value)
    }
    let text = |v: Option<Value>| v.and_then(|v| v.as_str().map(str::to_owned));
    let flag = |v: Option<Value>| v.and_then(|v| v.as_bool()).unwrap_or(false);
    let scope = text(value(state, "security.signin.geoScope").await).unwrap_or_else(|| "off".into());
    let account_mode =
        text(value(state, "security.signin.accountGeoMode").await).unwrap_or_else(|| "off".into());
    vec![
        ("signinGeography", scope != "off"),
        ("accountGeography", account_mode != "off"),
        ("sessionGeography", flag(value(state, "security.signin.geoEnforceSessions").await)),
        ("adminDiscovery", flag(value(state, "security.trust.adminGeoEnabled").await)),
        ("sessionCountryAlerts", flag(value(state, "security.alerts.countryChangeEnabled").await)),
    ]
}

/// The Security centre banner: a country feature is switched on and no database is loaded.
pub(crate) fn needs_warning(database_loaded: bool, enabled_features: &[&str]) -> bool {
    !database_loaded && !enabled_features.is_empty()
}

/// The status block for the Security centre: database state, build date and age,
/// the features that depend on it, and the warning condition.
pub(crate) async fn status(state: &AppState) -> Value {
    let (db_state, error, db) = crate::geoip::database_state();
    let features = feature_states(state).await;
    let enabled: Vec<&str> = features.iter().filter(|(_, on)| *on).map(|(id, _)| *id).collect();
    let build = db.as_ref().map(|d| d.info().build_epoch).filter(|e| *e > 0);
    let age_days = build.map(|e| ((chrono::Utc::now().timestamp() - e as i64) / 86_400).max(0));
    json!({
        "state": db_state,
        "error": error,
        "loaded": db.is_some(),
        "buildEpoch": build,
        "ageDays": age_days,
        "stale": build.is_some_and(crate::geoip::is_stale),
        "features": features.iter().map(|(id, on)| json!({"id": id, "enabled": on})).collect::<Vec<_>>(),
        "warning": needs_warning(db.is_some(), &enabled),
        "dependentFeatures": enabled,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ip(s: &str) -> Option<IpAddr> {
        Some(s.parse().unwrap())
    }

    #[test]
    fn private_loopback_and_shared_addresses_are_local_networks() {
        for a in [
            "127.0.0.1", "10.1.2.3", "172.16.0.9", "172.31.255.1", "192.168.1.5", "169.254.9.9",
            "100.64.0.1", "100.127.255.254", "0.0.0.0", "::1", "fd12:3456::1", "fe80::1",
            "::ffff:192.168.0.2",
        ] {
            assert!(is_local(a.parse().unwrap()), "{a}");
        }
        for a in [
            "8.8.8.8", "203.0.113.5", "172.32.0.1", "100.128.0.1", "2001:db8::1", "2a00:1450::1",
        ] {
            assert!(!is_local(a.parse().unwrap()), "{a}");
        }
    }

    #[test]
    fn local_networks_are_never_a_lookup_failure_but_the_two_causes_stay_apart() {
        for l in [CountryLookup::DatabaseUnavailable, CountryLookup::Unknown] {
            assert_eq!(classify_with(ip("192.168.1.9"), l), Outcome::Local);
        }
        assert_eq!(
            classify_with(ip("203.0.113.1"), CountryLookup::DatabaseUnavailable),
            Outcome::NoDatabase
        );
        assert_eq!(
            classify_with(ip("203.0.113.1"), CountryLookup::Unknown),
            Outcome::NotFound
        );
        assert_eq!(
            classify_with(None, CountryLookup::Unknown),
            Outcome::NotFound,
            "no address at all is a failed lookup, not a local one"
        );
        assert_eq!(
            classify_with(ip("203.0.113.1"), CountryLookup::Known("PT".into())),
            Outcome::Known("PT".into())
        );
    }

    #[test]
    fn failure_values_parse_order_and_cap() {
        assert_eq!(Failure::parse("allow"), Failure::Allow);
        assert_eq!(Failure::parse("treat-as-unknown"), Failure::Unknown);
        assert_eq!(Failure::parse("unknown"), Failure::Unknown);
        assert_eq!(Failure::parse("deny"), Failure::Block);
        assert_eq!(Failure::parse("block"), Failure::Block);
        assert_eq!(Failure::parse("garbage"), Failure::Allow);
        assert!(Failure::Allow < Failure::Unknown && Failure::Unknown < Failure::Block);
        assert_eq!(Failure::Block.capped(), Failure::Unknown);
        assert_eq!(Failure::Allow.capped(), Failure::Allow);
    }

    #[test]
    fn the_banner_shows_only_when_a_feature_is_on_and_no_database_is_loaded() {
        assert!(needs_warning(false, &["accountGeography"]));
        assert!(!needs_warning(true, &["accountGeography"]));
        assert!(!needs_warning(false, &[]));
        assert!(!needs_warning(true, &[]));
    }

    #[test]
    fn the_shared_default_applies_per_cause_and_admin_features_cap_block() {
        let s = Shared { no_database: Failure::Block, not_found: Failure::Unknown };
        assert_eq!(s.for_outcome(&Outcome::NoDatabase), Failure::Block);
        assert_eq!(s.for_outcome(&Outcome::NotFound), Failure::Unknown);
        assert_eq!(s.for_outcome(&Outcome::Local), Failure::Allow);
        assert_eq!(s.for_outcome(&Outcome::Known("PT".into())), Failure::Allow);
        assert_eq!(s.for_outcome(&Outcome::NoDatabase).capped(), Failure::Unknown);
    }

    #[test]
    fn registry_defaults_keep_todays_behaviour() {
        let get = |k: &str| crate::settings_registry::lookup(k).unwrap().default_value();
        assert_eq!(get(NO_DATABASE_KEY), json!("allow"));
        assert_eq!(get(NOT_FOUND_KEY), json!("allow"));
        assert_eq!(get(SESSION_KEY), json!("ignore"));
        assert_eq!(get("security.signin.geoNoDatabase"), json!("block"));
    }
}
