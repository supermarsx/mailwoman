//! Trust tiers: one definition of how much an anonymous request is trusted.
//!
//! * `Authenticated` - the request carries a valid session. Never scaled.
//! * `Known` - anonymous, from a source (IPv4 address, IPv6 /64) with a
//!   successful sign-in inside `security.trust.knownTtlDays`.
//! * `Unknown` - anonymous from any other source.
//!
//! The tier only scales request-rate and bot heuristics (the per-address
//! request budget, `/api/discover`, the sign-in cooldown, the challenge
//! difficulty). It never bypasses a credential check, a second factor, the
//! sign-in lockout, the SSRF gate, an active block or an abuse signal.
//!
//! Privacy: the table holds only a salted hash of the source key, its country,
//! a last-seen time and a small set of salted account hashes (for a count).
//! It is bounded and derived from successful sign-ins only. The hashes are keyed
//! from the server key, so they are stable across restarts; a periodic snapshot
//! (see [`persist`], `security.trust.persist`) is written to the settings store
//! and loaded at startup, merged with a rebuild from the successful rows the
//! authentication log still holds (its own retention applies).
mod persist;
pub use persist::flush_on_shutdown;
pub(crate) use persist::{forget_all, start, status};
use crate::settings_registry;
use std::{
    collections::{BTreeSet, HashMap},
    net::{IpAddr, Ipv6Addr},
    sync::{Arc, Mutex, OnceLock, RwLock},
    time::{Duration, Instant},
};

pub(crate) const PREFIX: &str = "security.trust.";
const CACHE_TTL: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tier {
    Authenticated,
    Known,
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UaAction {
    Allow,
    Limit,
    Block,
}
/// Claimed user-agent classes, in the order of [`Policy::ua`].
pub(crate) const UA_CLASSES: [&str; 8] = [
    "missing",
    "uncommon",
    "automation",
    "search",
    "ai-training",
    "ai-search",
    "ai-user",
    "custom",
];
const UA_KEYS: [&str; 8] = [
    "uaMissing",
    "uaUncommon",
    "uaAutomation",
    "uaSearch",
    "uaAiTraining",
    "uaAiSearch",
    "uaAiUser",
    "uaCustom",
];

#[derive(Clone, Debug)]
pub(crate) struct GeoRules {
    pub enabled: bool,
    pub countries: BTreeSet<String>,
    /// `true`: a source without a country counts as an unknown country.
    pub lookup_failure_unknown: bool,
    pub threshold: u32,
    pub window_secs: u32,
    pub block_on_first: bool,
    pub block_secs: [u32; 3],
    pub repeat_window_secs: u32,
}

#[derive(Clone, Debug)]
pub(crate) struct Policy {
    pub enabled: bool,
    pub known_ttl_secs: i64,
    pub capacity: usize,
    /// Save the table to the settings store and load it at startup.
    pub persist: bool,
    pub persist_interval_secs: u32,
    pub unknown_percent: u32,
    pub known_percent: u32,
    pub ua: [UaAction; 8],
    pub geo: GeoRules,
}

impl Default for Policy {
    /// The registry defaults, for the moments before the first read.
    fn default() -> Self {
        Self {
            enabled: true,
            known_ttl_secs: 30 * 86_400,
            capacity: 10_000,
            persist: true,
            persist_interval_secs: 300,
            unknown_percent: 100,
            known_percent: 200,
            ua: [
                UaAction::Limit,
                UaAction::Limit,
                UaAction::Limit,
                UaAction::Limit,
                UaAction::Block,
                UaAction::Block,
                UaAction::Block,
                UaAction::Limit,
            ],
            geo: GeoRules {
                enabled: true,
                countries: BTreeSet::new(),
                lookup_failure_unknown: false,
                threshold: 3,
                window_secs: 600,
                block_on_first: false,
                block_secs: [3600, 86_400, 604_800],
                repeat_window_secs: 30 * 86_400,
            },
        }
    }
}

impl Policy {
    pub(crate) fn parse(rows: Vec<settings_registry::Effective>) -> Result<Self, ()> {
        let values: HashMap<String, serde_json::Value> = rows
            .into_iter()
            .filter_map(|r| r.key.strip_prefix(PREFIX).map(|k| (k.to_owned(), r.value)))
            .collect();
        let get = |k: &str| values.get(k).ok_or(());
        let num = |k: &str| -> Result<u32, ()> {
            get(k)?
                .as_u64()
                .and_then(|v| u32::try_from(v).ok())
                .ok_or(())
        };
        let flag = |k: &str| get(k)?.as_bool().ok_or(());
        let text = |k: &str| get(k)?.as_str().ok_or(());
        let mut ua = [UaAction::Limit; 8];
        for (slot, key) in ua.iter_mut().zip(UA_KEYS) {
            *slot = match text(key)? {
                "allow" => UaAction::Allow,
                "limit" => UaAction::Limit,
                "block" => UaAction::Block,
                _ => return Err(()),
            };
        }
        let countries = get("adminGeoCountries")?
            .as_array()
            .ok_or(())?
            .iter()
            .map(|v| v.as_str().map(str::to_owned).ok_or(()))
            .collect::<Result<BTreeSet<_>, _>>()?;
        Ok(Self {
            enabled: flag("enabled")?,
            known_ttl_secs: i64::from(num("knownTtlDays")?) * 86_400,
            capacity: num("knownCapacity")? as usize,
            persist: flag("persist")?,
            persist_interval_secs: num("persistIntervalSecs")?,
            unknown_percent: num("unknownPercent")?.clamp(1, 100),
            known_percent: num("knownPercent")?.clamp(100, 1000),
            ua,
            geo: GeoRules {
                enabled: flag("adminGeoEnabled")?,
                countries,
                lookup_failure_unknown: match text("adminGeoLookupFailure")? {
                    "allow" => false,
                    "unknown" => true,
                    _ => return Err(()),
                },
                threshold: num("adminGeoThreshold")?.max(1),
                window_secs: num("adminGeoWindowSecs")?.max(1),
                block_on_first: flag("adminGeoBlockOnFirst")?,
                block_secs: [
                    num("adminGeoBlockSecs")?.max(1),
                    num("adminGeoSecondBlockSecs")?.max(1),
                    num("adminGeoMaxBlockSecs")?.max(1),
                ],
                repeat_window_secs: num("adminGeoRepeatDays")?.max(1).saturating_mul(86_400),
            },
        })
    }

    pub(crate) fn ua_action(&self, class: &str) -> UaAction {
        UA_CLASSES
            .iter()
            .position(|c| *c == class)
            .map_or(UaAction::Limit, |i| self.ua[i])
    }

    /// Percentage of the configured limits a request in `tier` is allowed.
    pub(crate) fn percent(&self, tier: Tier) -> u32 {
        if !self.enabled {
            return 100;
        }
        match tier {
            Tier::Authenticated => 100,
            Tier::Known => self.known_percent,
            Tier::Unknown => self.unknown_percent,
        }
    }
    /// A request allowance (`0` stays `0`: that means "off"). Never below 1.
    pub(crate) fn scale_limit(&self, base: u32, tier: Tier) -> u32 {
        if base == 0 {
            return 0;
        }
        let scaled = (u64::from(base) * u64::from(self.percent(tier))).div_ceil(100);
        u32::try_from(scaled).unwrap_or(u32::MAX).max(1)
    }
    /// A wait or difficulty: known sources wait less, unknown ones (below 100%)
    /// wait longer. Never below 1.
    pub(crate) fn scale_wait(&self, base: u64, tier: Tier) -> u64 {
        if base == 0 {
            return 0;
        }
        (base.saturating_mul(100))
            .div_ceil(u64::from(self.percent(tier)).max(1))
            .max(1)
    }
    /// Tokens to charge a keyed limiter for one request.
    pub(crate) fn cost(&self, tier: Tier) -> f64 {
        100.0 / f64::from(self.percent(tier).max(1))
    }
}

// ─── the known-source table ────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct Known {
    country: Option<String>,
    last_seen: i64,
    accounts: Vec<u64>,
}
/// Bounded table of sources with a recent successful sign-in.
pub(crate) struct KnownStore {
    salt: [u8; 32],
    entries: HashMap<[u8; 16], Known>,
    /// Country -> time of the latest successful sign-in from it.
    countries: HashMap<String, i64>,
    hydrated: bool,
    /// The store this table belongs to. A process has one; a test process with
    /// several servers starts a fresh table when the store changes.
    owner: Option<uuid::Uuid>,
    /// Changed since the last successful save.
    dirty: bool,
    /// Sign-ins at or before this time were forgotten by an administrator.
    cleared_at: i64,
    last_saved: Option<i64>,
    /// Entries taken from the saved snapshot at startup.
    restored: usize,
}
const MAX_ACCOUNTS_PER_SOURCE: usize = 16;

/// The key a source is tracked under: the IPv4 address or the IPv6 /64, with
/// IPv4-mapped IPv6 folded to IPv4.
pub(crate) fn source_key(ip: IpAddr) -> IpAddr {
    match ip.to_canonical() {
        IpAddr::V4(v) => IpAddr::V4(v),
        IpAddr::V6(v) => {
            let s = v.segments();
            IpAddr::V6(Ipv6Addr::new(s[0], s[1], s[2], s[3], 0, 0, 0, 0))
        }
    }
}

impl Default for KnownStore {
    fn default() -> Self {
        Self::new()
    }
}
impl KnownStore {
    pub fn new() -> Self {
        // Random until a store is adopted; then the store's derived key.
        let mut salt = [0u8; 32];
        salt[..16].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
        salt[16..].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
        Self {
            salt,
            entries: HashMap::new(),
            countries: HashMap::new(),
            hydrated: false,
            owner: None,
            dirty: false,
            cleared_at: 0,
            last_saved: None,
            restored: 0,
        }
    }
    fn adopt(&mut self, store: uuid::Uuid, salt: [u8; 32]) {
        if self.owner != Some(store) {
            *self = Self::new();
            self.salt = salt;
            self.owner = Some(store);
        }
    }
    fn digest(&self, value: &str) -> [u8; 16] {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(self.salt);
        h.update(value.as_bytes());
        let mut out = [0u8; 16];
        out.copy_from_slice(&h.finalize()[..16]);
        out
    }
    fn key(&self, ip: IpAddr) -> [u8; 16] {
        self.digest(&source_key(ip).to_string())
    }
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    /// Record a successful sign-in. Only call this for a success.
    pub fn record_success(
        &mut self,
        ip: IpAddr,
        country: Option<&str>,
        account: &str,
        now: i64,
        ttl: i64,
        capacity: usize,
    ) {
        self.dirty = true;
        let key = self.key(ip);
        let fingerprint = u64::from_le_bytes(self.digest(account)[..8].try_into().unwrap());
        if !self.entries.contains_key(&key) && self.entries.len() >= capacity.max(1) {
            self.entries
                .retain(|_, e| now.saturating_sub(e.last_seen) <= ttl);
            while self.entries.len() >= capacity.max(1) {
                let Some(oldest) = self
                    .entries
                    .iter()
                    .min_by_key(|(_, e)| e.last_seen)
                    .map(|(k, _)| *k)
                else {
                    break;
                };
                self.entries.remove(&oldest);
            }
        }
        let entry = self.entries.entry(key).or_insert(Known {
            country: None,
            last_seen: now,
            accounts: Vec::new(),
        });
        entry.last_seen = entry.last_seen.max(now);
        if country.is_some() {
            entry.country = country.map(str::to_owned);
        }
        if !entry.accounts.contains(&fingerprint) && entry.accounts.len() < MAX_ACCOUNTS_PER_SOURCE
        {
            entry.accounts.push(fingerprint);
        }
        if let Some(cc) = country {
            let slot = self.countries.entry(cc.to_owned()).or_insert(now);
            *slot = (*slot).max(now);
        }
    }
    pub fn is_known(&self, ip: IpAddr, now: i64, ttl: i64) -> bool {
        self.entries
            .get(&self.key(ip))
            .is_some_and(|e| now.saturating_sub(e.last_seen) <= ttl)
    }
    /// Whether a successful sign-in from `country` happened inside the window.
    pub fn country_known(&self, country: &str, now: i64, ttl: i64) -> bool {
        self.countries
            .get(country)
            .is_some_and(|t| now.saturating_sub(*t) <= ttl)
    }
    /// Distinct accounts seen from this source (a count only).
    #[cfg(test)]
    pub fn account_count(&self, ip: IpAddr) -> usize {
        self.entries
            .get(&self.key(ip))
            .map_or(0, |e| e.accounts.len())
    }
    pub fn prune(&mut self, now: i64, ttl: i64) {
        let before = self.entries.len() + self.countries.len();
        self.entries
            .retain(|_, e| now.saturating_sub(e.last_seen) <= ttl);
        self.countries.retain(|_, t| now.saturating_sub(*t) <= ttl);
        if self.entries.len() + self.countries.len() != before {
            self.dirty = true;
        }
    }
}

fn known() -> &'static Mutex<KnownStore> {
    static STORE: OnceLock<Mutex<KnownStore>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(KnownStore::new()))
}
fn lock() -> std::sync::MutexGuard<'static, KnownStore> {
    known().lock().unwrap_or_else(|p| p.into_inner())
}

// ─── policy cache (shared, readable without await) ─────────────────────────

struct Cached {
    at: Instant,
    store: uuid::Uuid,
    value: Arc<Policy>,
}
fn cache() -> &'static RwLock<Option<Cached>> {
    static C: OnceLock<RwLock<Option<Cached>>> = OnceLock::new();
    C.get_or_init(|| RwLock::new(None))
}
/// The last policy read, or the registry defaults. Never waits.
pub(crate) fn snapshot() -> Arc<Policy> {
    cache()
        .read()
        .unwrap_or_else(|p| p.into_inner())
        .as_ref()
        .map_or_else(|| Arc::new(Policy::default()), |c| c.value.clone())
}
/// Read (and cache for 30 s) the policy; also starts the one-time rebuild of
/// the known table from the authentication log.
pub(crate) async fn policy(store: &mw_store::Store) -> Arc<Policy> {
    {
        let guard = cache().read().unwrap_or_else(|p| p.into_inner());
        if let Some(c) = guard.as_ref()
            && c.store == store.instance_id()
            && c.at.elapsed() < CACHE_TTL
        {
            return c.value.clone();
        }
    }
    let rows = settings_registry::effective_all(store, &Default::default(), |d| {
        d.key.starts_with(PREFIX)
    })
    .await;
    let parsed = rows.map_err(|_| ()).and_then(Policy::parse);
    let value = match parsed {
        Ok(p) => Arc::new(p),
        // Storage trouble keeps the last policy (or the defaults).
        Err(()) => snapshot(),
    };
    *cache().write().unwrap_or_else(|p| p.into_inner()) = Some(Cached {
        at: Instant::now(),
        store: store.instance_id(),
        value: value.clone(),
    });
    lock().adopt(store.instance_id(), persist::salt(store));
    hydrate_once(store).await;
    value
}

async fn hydrate_once(store: &mw_store::Store) {
    {
        let mut k = lock();
        if k.hydrated {
            return;
        }
        k.hydrated = true;
    }
    let policy = snapshot();
    persist::restore(store, &policy).await;
    let cleared_at = lock().cleared_at;
    let mut before = None;
    for _ in 0..5 {
        let Ok(rows) = store
            .auth_log_events(&mw_store::AuthLogQuery {
                result: Some("success".into()),
                before: before.clone(),
                limit: 100,
                ..Default::default()
            })
            .await
        else {
            return;
        };
        let Some(last) = rows.last() else {
            return;
        };
        before = Some((last.last_seen, last.event.id.clone()));
        for row in &rows {
            if row.last_seen >= cleared_at
                && let Ok(ip) = row.event.ip.parse::<IpAddr>()
            {
                let who = row
                    .event
                    .account_id
                    .as_deref()
                    .unwrap_or(&row.event.username_attempted);
                insert(ip, who, row.last_seen, &policy);
            }
        }
        if rows.len() < 100 {
            return;
        }
    }
}

fn country_of(ip: IpAddr) -> Option<String> {
    match mw_engine::security::country_lookup(Some(ip)) {
        mw_engine::security::CountryLookup::Known(c) => Some(c),
        _ => None,
    }
}
fn insert(ip: IpAddr, account: &str, ts: i64, policy: &Policy) {
    let country = country_of(ip);
    lock().record_success(
        ip,
        country.as_deref(),
        account,
        ts,
        policy.known_ttl_secs,
        policy.capacity,
    );
}

/// A successful sign-in (never a failure, never an anonymous request).
pub(crate) fn record_success(ip: IpAddr, account: &str) {
    let policy = snapshot();
    if !policy.enabled {
        return;
    }
    insert(ip, account, chrono::Utc::now().timestamp(), &policy);
}
pub(crate) fn is_known(ip: IpAddr) -> bool {
    let policy = snapshot();
    policy.enabled
        && lock().is_known(
            ip,
            chrono::Utc::now().timestamp(),
            policy.known_ttl_secs,
        )
}
pub(crate) fn country_known(country: &str) -> bool {
    let policy = snapshot();
    lock().country_known(
        country,
        chrono::Utc::now().timestamp(),
        policy.known_ttl_secs,
    )
}
/// The tier of a request. `authenticated` comes from the caller's own session
/// check; an unknown source address is always `Unknown`.
pub(crate) fn tier_for(ip: Option<IpAddr>, authenticated: bool) -> Tier {
    if authenticated {
        Tier::Authenticated
    } else if ip.is_some_and(is_known) {
        Tier::Known
    } else {
        Tier::Unknown
    }
}
pub(crate) fn prune() {
    let policy = snapshot();
    lock().prune(chrono::Utc::now().timestamp(), policy.known_ttl_secs);
}

// ─── the admin-console country decision ────────────────────────────────────

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum CountryVerdict {
    /// The country has had a successful sign-in or is on the allow list.
    Known,
    /// No such sign-in: a probe from here may be blocked.
    Unknown,
    /// No country could be determined and the policy says do not block.
    Unresolved,
}
pub(crate) fn country_verdict(
    lookup: &mw_engine::security::CountryLookup,
    rules: &GeoRules,
    seen: impl Fn(&str) -> bool,
) -> CountryVerdict {
    use mw_engine::security::CountryLookup::*;
    match lookup {
        Known(cc) if rules.countries.contains(cc) || seen(cc) => CountryVerdict::Known,
        Known(_) => CountryVerdict::Unknown,
        Unknown | DatabaseUnavailable => {
            if rules.lookup_failure_unknown {
                CountryVerdict::Unknown
            } else {
                CountryVerdict::Unresolved
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mw_engine::security::CountryLookup;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }
    const DAY: i64 = 86_400;

    #[test]
    fn a_success_makes_a_source_known_until_the_window_ends() {
        let mut k = KnownStore::new();
        let ttl = 30 * DAY;
        assert!(!k.is_known(ip("203.0.113.5"), 1000, ttl));
        k.record_success(ip("203.0.113.5"), Some("PT"), "a@x", 1000, ttl, 100);
        assert!(k.is_known(ip("203.0.113.5"), 1000 + ttl, ttl));
        assert!(!k.is_known(ip("203.0.113.5"), 1001 + ttl, ttl));
        assert!(!k.is_known(ip("203.0.113.6"), 1000, ttl));
        assert!(k.country_known("PT", 1000, ttl));
        assert!(!k.country_known("PT", 1001 + ttl, ttl));
        assert!(!k.country_known("RU", 1000, ttl));
        k.prune(2000 + ttl, ttl);
        assert_eq!(k.len(), 0);
    }

    #[test]
    fn ipv6_is_tracked_by_64_and_mapped_v4_folds() {
        let mut k = KnownStore::new();
        k.record_success(ip("2001:db8:1:2::1"), None, "a", 10, 100, 10);
        assert!(k.is_known(ip("2001:db8:1:2:ffff::9"), 10, 100));
        assert!(!k.is_known(ip("2001:db8:1:3::1"), 10, 100));
        k.record_success(ip("::ffff:198.51.100.7"), None, "a", 10, 100, 10);
        assert!(k.is_known(ip("198.51.100.7"), 10, 100));
    }

    #[test]
    fn the_table_is_bounded_and_keeps_only_a_count_of_accounts() {
        let mut k = KnownStore::new();
        for i in 0..10u8 {
            k.record_success(ip(&format!("192.0.2.{i}")), None, "a", i64::from(i), 1000, 4);
        }
        assert_eq!(k.len(), 4);
        // The oldest were dropped first.
        assert!(!k.is_known(ip("192.0.2.0"), 10, 1000));
        assert!(k.is_known(ip("192.0.2.9"), 10, 1000));
        for n in 0..40 {
            k.record_success(ip("192.0.2.9"), None, &format!("u{n}"), 11, 1000, 4);
        }
        assert_eq!(k.account_count(ip("192.0.2.9")), MAX_ACCOUNTS_PER_SOURCE);
    }

    #[test]
    fn tier_math_scales_limits_and_never_below_one() {
        let p = Policy::default();
        assert_eq!(p.scale_limit(60, Tier::Unknown), 60);
        assert_eq!(p.scale_limit(60, Tier::Known), 120);
        assert_eq!(p.scale_limit(60, Tier::Authenticated), 60);
        assert_eq!(p.scale_limit(0, Tier::Known), 0);
        let strict = Policy {
            unknown_percent: 50,
            ..Policy::default()
        };
        assert_eq!(strict.scale_limit(5, Tier::Unknown), 3);
        assert_eq!(strict.scale_limit(1, Tier::Unknown), 1);
        assert_eq!(p.scale_wait(4, Tier::Known), 2);
        assert_eq!(p.scale_wait(4, Tier::Unknown), 4);
        assert_eq!(strict.scale_wait(4, Tier::Unknown), 8);
        assert_eq!(p.scale_wait(1, Tier::Known), 1);
        assert!((p.cost(Tier::Known) - 0.5).abs() < 1e-9);
        let off = Policy {
            enabled: false,
            ..Policy::default()
        };
        assert_eq!(off.scale_limit(10, Tier::Known), 10);
    }

    #[test]
    fn ua_actions_default_to_blocking_declared_ai_crawlers_only() {
        let p = Policy::default();
        for c in ["ai-training", "ai-search", "ai-user"] {
            assert_eq!(p.ua_action(c), UaAction::Block, "{c}");
        }
        for c in ["missing", "uncommon", "automation", "search", "custom"] {
            assert_eq!(p.ua_action(c), UaAction::Limit, "{c}");
        }
        assert_eq!(p.ua_action("nonsense"), UaAction::Limit);
    }

    #[test]
    fn country_verdicts_follow_seen_countries_allow_list_and_failure_policy() {
        let mut rules = Policy::default().geo;
        rules.countries.insert("DE".into());
        let seen = |c: &str| c == "PT";
        let v = |l: CountryLookup, r: &GeoRules| country_verdict(&l, r, seen);
        assert_eq!(v(CountryLookup::Known("PT".into()), &rules), CountryVerdict::Known);
        assert_eq!(v(CountryLookup::Known("DE".into()), &rules), CountryVerdict::Known);
        assert_eq!(v(CountryLookup::Known("RU".into()), &rules), CountryVerdict::Unknown);
        assert_eq!(v(CountryLookup::Unknown, &rules), CountryVerdict::Unresolved);
        assert_eq!(
            v(CountryLookup::DatabaseUnavailable, &rules),
            CountryVerdict::Unresolved
        );
        rules.lookup_failure_unknown = true;
        assert_eq!(v(CountryLookup::Unknown, &rules), CountryVerdict::Unknown);
    }

    #[test]
    fn a_spoofed_forwarding_header_cannot_borrow_a_known_address() {
        // A known source exists, but a request that only claims its address in
        // X-Forwarded-For has no peer address: client_ip is None, so it is unknown.
        record_success("198.51.100.200".parse().unwrap(), "someone");
        assert!(is_known("198.51.100.200".parse().unwrap()));
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("x-forwarded-for", "198.51.100.200".parse().unwrap());
        let source = crate::scope_mw::proxy::client_ip(&headers, &axum::http::Extensions::new());
        assert_eq!(source, None);
        assert_eq!(tier_for(source, false), Tier::Unknown);
        assert_eq!(tier_for(source, true), Tier::Authenticated);
    }

    #[tokio::test]
    async fn registry_defaults_parse_into_the_documented_policy() {
        let store = mw_store::Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        let rows = settings_registry::effective_all(&store, &Default::default(), |d| {
            d.key.starts_with(PREFIX)
        })
        .await
        .unwrap();
        let p = Policy::parse(rows).unwrap();
        let d = Policy::default();
        assert_eq!(p.known_ttl_secs, d.known_ttl_secs);
        assert_eq!(p.unknown_percent, 100);
        assert_eq!(p.known_percent, 200);
        assert_eq!(p.ua, d.ua);
        assert_eq!(p.geo.threshold, 3);
        assert_eq!(p.geo.window_secs, 600);
        assert_eq!(p.geo.block_secs, [3600, 86_400, 604_800]);
        assert!(!p.geo.block_on_first);
        assert!(!p.geo.lookup_failure_unknown);
    }
}
