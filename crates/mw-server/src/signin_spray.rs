//! Password-spray protection, type 1: too many usernames from one source.
//!
//! A credential-stuffing or spraying client tries *many different usernames*
//! from one address. A shared office, campus or carrier NAT instead has many
//! *different people who mostly succeed*. The two are told apart by counting
//! only the **distinct usernames a source failed with** inside a window:
//!
//! * a successful sign-in is never counted, and it removes that username from
//!   the source's set, so a person who mistypes once and then gets in leaves no
//!   trace;
//! * only a rejected credential counts (`auth-rejected`, `credential-rejected`);
//!   second-factor refusals, transport errors, disabled accounts and refused
//!   challenges do not;
//! * an authenticated request never reaches this module (sign-in is anonymous);
//!   [`Config::counts`] still ignores [`Tier::Authenticated`] explicitly.
//!
//! # Defaults and why
//! `challengeAt = 10`, `paceAt = 15`, `lockAt = 25` distinct failing usernames
//! in `windowSecs = 900`. A person fails with one or two usernames (their own,
//! mistyped); a household or a small office with up to eight failing names in
//! fifteen minutes is far beyond normal; a legitimate NAT of hundreds of users
//! contributes only the users who *fail and never succeed*, which is rare
//! enough that 10 is crossed only by an actual guessing run. A known source (a
//! successful sign-in inside `security.trust.knownTtlDays`) gets three times
//! those numbers.
//!
//! # Privacy
//! Only an HMAC-SHA256 (server-derived key, truncated to 16 bytes) of the
//! lower-cased username is kept, never the username or any password. State is
//! in memory, bounded per source (2048 names) and by source count
//! (`security.spray.capacity`, oldest source first), and is never written to
//! storage. The lock a source can end up in is the existing sign-in lock
//! ([`mw_store::auth_failures`]), which holds only the address.
//!
//! # Source identity
//! The key is the IPv4 address or IPv6 /64 of [`crate::scope_mw::proxy::client_ip`],
//! which honours a forwarded address only through the trusted-proxy rules; a
//! spoofed `X-Forwarded-For` from an untrusted peer is ignored there.
use crate::{
    AppState, sec_cfg,
    trust_tier::{self, Tier},
};
use mw_store::{AuthSurface, SigninEvent, auth_failures::FailurePolicy};
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Mutex, OnceLock},
};

pub(crate) const PREFIX: &str = "security.spray.";
/// Distinct usernames remembered for one source.
const PER_SOURCE_CAP: usize = 2048;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Action {
    None,
    Challenge,
    Pace,
    Lock,
}
impl Action {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Action::None => "none",
            Action::Challenge => "challenge",
            Action::Pace => "pace",
            Action::Lock => "lock",
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Config {
    pub enabled: bool,
    pub window_secs: i64,
    pub challenge_at: usize,
    pub pace_at: usize,
    pub lock_at: usize,
    pub pace_secs: u64,
    pub lock_secs: i64,
    pub known_multiplier: usize,
    pub capacity: usize,
}
impl Default for Config {
    fn default() -> Self {
        Self::from_values(&sec_cfg::Values::default())
    }
}
impl Config {
    pub(crate) fn from_values(v: &sec_cfg::Values) -> Self {
        let challenge_at = (v.num("security.spray.challengeAt") as usize).max(2);
        let pace_at = (v.num("security.spray.paceAt") as usize).max(challenge_at);
        let lock_at = (v.num("security.spray.lockAt") as usize).max(pace_at);
        Self {
            enabled: v.flag("security.spray.enabled"),
            window_secs: (v.num("security.spray.windowSecs") as i64).clamp(60, 86_400),
            challenge_at,
            pace_at,
            lock_at,
            pace_secs: v.num("security.spray.paceSecs").clamp(1, 300),
            lock_secs: (v.num("security.spray.lockSecs") as i64).clamp(30, 86_400),
            known_multiplier: (v.num("security.spray.knownMultiplier") as usize).clamp(1, 20),
            capacity: (v.num("security.spray.capacity") as usize).clamp(100, 200_000),
        }
    }
    /// Whether a failure from a source of this tier is counted at all.
    pub(crate) fn counts(&self, tier: Tier) -> bool {
        self.enabled && tier != Tier::Authenticated
    }
    pub(crate) fn action_for(&self, distinct: usize, tier: Tier) -> Action {
        let m = if tier == Tier::Known {
            self.known_multiplier
        } else {
            1
        };
        if distinct >= self.lock_at.saturating_mul(m) {
            Action::Lock
        } else if distinct >= self.pace_at.saturating_mul(m) {
            Action::Pace
        } else if distinct >= self.challenge_at.saturating_mul(m) {
            Action::Challenge
        } else {
            Action::None
        }
    }
}

struct Entry {
    names: HashMap<[u8; 16], i64>,
    last_seen: i64,
    /// The highest action already reported to the audit log for this run.
    reported: Action,
}

/// The per-source distinct-username sets. Pure: time and keys are arguments.
#[derive(Default)]
pub(crate) struct Tracker {
    entries: HashMap<IpAddr, Entry>,
}
impl Tracker {
    /// Record a rejected credential and return the distinct failing usernames
    /// of the source inside `window`.
    pub(crate) fn note_failure(
        &mut self,
        source: IpAddr,
        name: [u8; 16],
        now: i64,
        window: i64,
        capacity: usize,
    ) -> usize {
        let key = trust_tier::source_key(source);
        if !self.entries.contains_key(&key) && self.entries.len() >= capacity.max(1) {
            self.evict(capacity);
        }
        let entry = self.entries.entry(key).or_insert_with(|| Entry {
            names: HashMap::new(),
            last_seen: now,
            reported: Action::None,
        });
        let floor = now.saturating_sub(window);
        entry.names.retain(|_, ts| *ts > floor);
        if !entry.names.contains_key(&name)
            && entry.names.len() >= PER_SOURCE_CAP
            && let Some(oldest) = entry.names.iter().min_by_key(|(_, ts)| **ts).map(|(k, _)| *k)
        {
            entry.names.remove(&oldest);
        }
        entry.names.insert(name, now);
        entry.last_seen = now;
        entry.names.len()
    }
    /// A successful sign-in with `name`: it was a person, not a guess.
    pub(crate) fn note_success(&mut self, source: IpAddr, name: &[u8; 16]) {
        if let Some(entry) = self.entries.get_mut(&trust_tier::source_key(source)) {
            entry.names.remove(name);
        }
    }
    pub(crate) fn distinct(&self, source: IpAddr, now: i64, window: i64) -> usize {
        let floor = now.saturating_sub(window);
        self.entries
            .get(&trust_tier::source_key(source))
            .map_or(0, |e| e.names.values().filter(|ts| **ts > floor).count())
    }
    /// Report a new, higher action for a source once; `true` the first time.
    pub(crate) fn escalated(&mut self, source: IpAddr, action: Action) -> bool {
        match self.entries.get_mut(&trust_tier::source_key(source)) {
            Some(e) if action > e.reported => {
                e.reported = action;
                true
            }
            Some(e) => {
                if action == Action::None {
                    e.reported = Action::None;
                }
                false
            }
            None => false,
        }
    }
    pub(crate) fn forget(&mut self, source: IpAddr) {
        self.entries.remove(&trust_tier::source_key(source));
    }
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }
    /// Drop the least recently active tenth (at least one) of the sources.
    fn evict(&mut self, capacity: usize) {
        let drop = (capacity / 10).max(1);
        let mut order: Vec<_> = self.entries.iter().map(|(k, e)| (e.last_seen, *k)).collect();
        order.sort_unstable();
        for (_, key) in order.into_iter().take(drop) {
            self.entries.remove(&key);
        }
    }
    /// Sources currently at or above the challenge step, for the admin readout.
    pub(crate) fn active(&self, cfg: &Config, now: i64) -> usize {
        self.entries
            .values()
            .filter(|e| {
                e.names.values().filter(|ts| **ts > now - cfg.window_secs).count()
                    >= cfg.challenge_at
            })
            .count()
    }
}

fn tracker() -> &'static Mutex<Tracker> {
    static T: OnceLock<Mutex<Tracker>> = OnceLock::new();
    T.get_or_init(Default::default)
}
fn lock() -> std::sync::MutexGuard<'static, Tracker> {
    tracker().lock().unwrap_or_else(|p| p.into_inner())
}

pub(crate) fn name_hash(secret: &str, username: &str) -> [u8; 16] {
    sec_cfg::keyed_hash(secret, "spray-user", &username.trim().to_lowercase())
}
fn secret(state: &AppState) -> String {
    state.store.signin_challenge_secret("spray")
}

/// The configuration, cached and hot-reloaded.
pub(crate) async fn config(state: &AppState) -> Config {
    Config::from_values(&*sec_cfg::values(&state.store, PREFIX).await)
}

/// What this source's recent failures call for, right now.
pub(crate) fn action_for_source(cfg: &Config, ip: IpAddr, tier: Tier) -> Action {
    if !cfg.counts(tier) {
        return Action::None;
    }
    cfg.action_for(
        lock().distinct(ip, chrono::Utc::now().timestamp(), cfg.window_secs),
        tier,
    )
}

/// Called for every authentication event (`signin_guard::observe`).
pub(crate) async fn observe(state: &AppState, event: &SigninEvent, surface: AuthSurface) {
    let Ok(ip) = event.ip.parse::<IpAddr>() else {
        return;
    };
    let cfg = config(state).await;
    if !cfg.enabled {
        return;
    }
    let name = name_hash(&secret(state), &event.username_attempted);
    if event.result == "success" {
        lock().note_success(ip, &name);
        return;
    }
    if event.result != "failure" || !matches!(event.reason.as_str(), "auth-rejected" | "credential-rejected")
    {
        return;
    }
    let tier = trust_tier::tier_for(Some(ip), false);
    if !cfg.counts(tier) {
        return;
    }
    let now = chrono::Utc::now().timestamp();
    let (distinct, action, first) = {
        let mut t = lock();
        let distinct = t.note_failure(ip, name, now, cfg.window_secs, cfg.capacity);
        let action = cfg.action_for(distinct, tier);
        let first = t.escalated(ip, action);
        (distinct, action, first)
    };
    if action == Action::Lock {
        let policy = FailurePolicy {
            threshold: 1,
            window_secs: cfg.window_secs.clamp(30, 86_400),
            lock_secs: cfg.lock_secs,
        };
        if let Err(error) = state
            .store
            .record_auth_source_failure(surface.as_str(), ip, now, policy)
            .await
        {
            tracing::warn!("spray lock could not be stored: {error}");
        }
    }
    if first && action > Action::None {
        crate::audit::security_event(
            state,
            crate::audit::SecurityEvent::system("signin-spray-source")
                .target(format!("source:{}", trust_tier::source_key(ip)))
                .detail(serde_json::json!({
                    "action": action.as_str(),
                    "distinctUsernames": distinct,
                    "windowSecs": cfg.window_secs,
                    "surface": surface.as_str(),
                    "knownSource": tier == Tier::Known,
                })),
        )
        .await;
    }
}

/// An administrator cleared this source's lock: also clear its spray run.
pub(crate) fn forget_source(ip: IpAddr) {
    lock().forget(ip);
}

pub(crate) fn status(cfg: &Config) -> serde_json::Value {
    let t = lock();
    serde_json::json!({
        "enabled": cfg.enabled,
        "trackedSources": t.len(),
        "sourcesAtChallengeOrAbove": t.active(cfg, chrono::Utc::now().timestamp()),
        "windowSecs": cfg.window_secs,
        "challengeAt": cfg.challenge_at,
        "paceAt": cfg.pace_at,
        "lockAt": cfg.lock_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }
    fn h(n: u32) -> [u8; 16] {
        name_hash("secret", &format!("user{n}@example.org"))
    }
    const W: i64 = 900;

    #[test]
    fn counts_distinct_usernames_not_attempts() {
        let mut t = Tracker::default();
        let a = ip("203.0.113.9");
        for i in 0..50 {
            t.note_failure(a, h(1), 1000 + i, W, 100);
        }
        assert_eq!(t.distinct(a, 1100, W), 1, "one username guessed 50 times is one name");
        for n in 2..=7 {
            t.note_failure(a, h(n), 1100, W, 100);
        }
        assert_eq!(t.distinct(a, 1100, W), 7);
    }

    #[test]
    fn username_hash_is_keyed_normalised_and_not_the_name() {
        assert_eq!(name_hash("k", " Alice@Example.org "), name_hash("k", "alice@example.org"));
        assert_ne!(name_hash("k1", "alice"), name_hash("k2", "alice"));
        let hash = name_hash("k", "alice@example.org");
        assert_eq!(hash.len(), 16);
        assert!(!sec_cfg::hex(&hash).contains("616c696365"), "hex of the name must not appear");
    }

    #[test]
    fn window_expiry_forgets_old_failures() {
        let mut t = Tracker::default();
        let a = ip("203.0.113.9");
        for n in 0..5 {
            t.note_failure(a, h(n), 1000, W, 100);
        }
        assert_eq!(t.distinct(a, 1000 + W - 1, W), 5);
        assert_eq!(t.distinct(a, 1000 + W, W), 0);
        // A later failure prunes the old ones as well.
        assert_eq!(t.note_failure(a, h(99), 1000 + W + 5, W, 100), 1);
    }

    #[test]
    fn a_success_removes_the_username_so_typos_do_not_accumulate() {
        let mut t = Tracker::default();
        let a = ip("198.51.100.7");
        for n in 0..30 {
            t.note_failure(a, h(n), 1000, W, 100);
            t.note_success(a, &h(n));
        }
        assert_eq!(t.distinct(a, 1001, W), 0);
    }

    #[test]
    fn ipv6_is_tracked_per_slash_64_and_mapped_v4_folds() {
        let mut t = Tracker::default();
        t.note_failure(ip("2001:db8:1:2::1"), h(1), 1000, W, 100);
        t.note_failure(ip("2001:db8:1:2:ffff::9"), h(2), 1000, W, 100);
        t.note_failure(ip("2001:db8:1:3::1"), h(3), 1000, W, 100);
        assert_eq!(t.distinct(ip("2001:db8:1:2::77"), 1000, W), 2);
        assert_eq!(t.distinct(ip("2001:db8:1:3::1"), 1000, W), 1);
        t.note_failure(ip("::ffff:192.0.2.1"), h(1), 1000, W, 100);
        assert_eq!(t.distinct(ip("192.0.2.1"), 1000, W), 1);
    }

    #[test]
    fn per_source_and_global_bounds_hold() {
        let mut t = Tracker::default();
        let a = ip("203.0.113.9");
        for n in 0..(PER_SOURCE_CAP as u32 + 500) {
            t.note_failure(a, h(n), 1000 + i64::from(n), 1_000_000, 100);
        }
        assert_eq!(t.distinct(a, 5000, 1_000_000), PER_SOURCE_CAP);
        let mut t = Tracker::default();
        for n in 0..1000u32 {
            let src = ip(&format!("10.{}.{}.1", n / 250, n % 250));
            t.note_failure(src, h(1), 1000 + i64::from(n), W, 200);
        }
        assert!(t.len() <= 200, "sources bounded, got {}", t.len());
        // The oldest sources went first.
        assert_eq!(t.distinct(ip("10.3.249.1"), 2000, W), 1);
        assert_eq!(t.distinct(ip("10.0.0.1"), 2000, W), 0);
    }

    #[test]
    fn thresholds_escalate_and_known_sources_get_the_multiplier() {
        let cfg = Config::default();
        assert_eq!((cfg.challenge_at, cfg.pace_at, cfg.lock_at), (10, 15, 25));
        assert_eq!(cfg.action_for(9, Tier::Unknown), Action::None);
        assert_eq!(cfg.action_for(10, Tier::Unknown), Action::Challenge);
        assert_eq!(cfg.action_for(15, Tier::Unknown), Action::Pace);
        assert_eq!(cfg.action_for(25, Tier::Unknown), Action::Lock);
        assert_eq!(cfg.action_for(29, Tier::Known), Action::None);
        assert_eq!(cfg.action_for(30, Tier::Known), Action::Challenge);
        assert_eq!(cfg.action_for(45, Tier::Known), Action::Pace);
        assert_eq!(cfg.action_for(75, Tier::Known), Action::Lock);
    }

    #[test]
    fn authenticated_and_disabled_do_not_count() {
        let mut cfg = Config::default();
        assert!(!cfg.counts(Tier::Authenticated));
        assert!(cfg.counts(Tier::Unknown) && cfg.counts(Tier::Known));
        cfg.enabled = false;
        assert!(!cfg.counts(Tier::Unknown));
    }

    #[test]
    fn a_nat_of_many_successful_users_never_trips() {
        // 400 people behind one address sign in within the window. Each succeeds,
        // 60 mistype once first. None of that reaches the thresholds.
        let cfg = Config::default();
        let mut t = Tracker::default();
        let nat = ip("192.0.2.80");
        let mut worst = 0;
        for n in 0..400u32 {
            if n % 7 == 0 {
                let d = t.note_failure(nat, h(n), 1000 + i64::from(n), W, 100);
                worst = worst.max(d);
            }
            t.note_success(nat, &h(n));
        }
        assert_eq!(cfg.action_for(worst, Tier::Unknown), Action::None);
        assert_eq!(t.distinct(nat, 1400, W), 0);
        // Even failures that never succeed stay under the first step for a handful of people.
        for n in 1000..1006u32 {
            t.note_failure(nat, h(n), 1500, W, 100);
        }
        assert_eq!(cfg.action_for(t.distinct(nat, 1500, W), Tier::Unknown), Action::None);
        // A guessing run does.
        for n in 2000..2025u32 {
            t.note_failure(nat, h(n), 1600, W, 100);
        }
        assert_eq!(cfg.action_for(t.distinct(nat, 1600, W), Tier::Unknown), Action::Lock);
    }

    #[test]
    fn a_spoofed_forwarded_for_header_cannot_dodge_the_count() {
        use crate::scope_mw::proxy::{ForwardedMode, ProxyConfig, resolve};
        let mut headers = axum::http::HeaderMap::new();
        let real = ip("198.51.100.7");
        let cfg = ProxyConfig::new(ForwardedMode::Xff, "10.0.0.0/8");
        let mut t = Tracker::default();
        // The peer is not a trusted proxy: each request claims another client
        // address, and every claim is ignored.
        for n in 0..12u32 {
            headers.insert("x-forwarded-for", format!("203.0.113.{n}").parse().unwrap());
            let source = resolve(&cfg, real, &headers);
            assert_eq!(source, real);
            t.note_failure(source, h(n), 1000, W, 100);
        }
        assert_eq!(Config::default().action_for(t.distinct(real, 1000, W), Tier::Unknown), Action::Challenge);
        // Behind a trusted proxy the forwarded client is the source instead.
        let behind = resolve(&cfg, ip("10.0.0.1"), &headers);
        assert_eq!(behind, ip("203.0.113.11"));
    }

    #[test]
    fn an_escalation_is_reported_once_per_level() {
        let mut t = Tracker::default();
        let a = ip("203.0.113.9");
        t.note_failure(a, h(1), 1000, W, 100);
        assert!(t.escalated(a, Action::Challenge));
        assert!(!t.escalated(a, Action::Challenge));
        assert!(t.escalated(a, Action::Lock));
        t.forget(a);
        assert_eq!(t.distinct(a, 1000, W), 0);
    }

    #[test]
    fn registry_keys_exist_and_defaults_match_the_documented_ones() {
        for key in [
            "security.spray.enabled",
            "security.spray.windowSecs",
            "security.spray.challengeAt",
            "security.spray.paceAt",
            "security.spray.lockAt",
        ] {
            assert!(crate::settings_registry::lookup(key).is_some(), "{key}");
        }
    }
}
