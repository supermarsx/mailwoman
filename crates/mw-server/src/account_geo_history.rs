//! Durable per-account history of the countries seen in successful sign-ins,
//! for `security.signin.accountGeoMode = block-unusual-countries`.
//!
//! "Usual" used to be judged from the authentication log's retained rows, so
//! it forgot as the log aged out (`logs.auth.retentionDays`, 7 by default).
//! This history is separate from the log and kept for
//! `security.signin.accountGeoHistoryDays` (default 365).
//!
//! What is stored, exactly: one settings row per account, key
//! `acct-geo:v1:<account id>`, value a JSON object
//! `{"v":1,"seeded":bool,"cleared":unix-seconds,"c":[{"cc":"PT","first":..,"last":..,"n":..}]}`.
//! Each entry is an ISO 3166 country code, the unix time of the first and last
//! successful sign-in from it and the number of such sign-ins. No IP addresses,
//! devices, user agents or other data. At most [`MAX_COUNTRIES`] entries; when
//! full, the entry with the oldest last-seen time is dropped.
//!
//! Written only after a sign-in has completed (the geography decision allowed
//! it and the session was kept) and only while the account's policy is
//! `block-unusual-countries`. The first time the policy needs an account that has
//! no row, the row is seeded once from the retained successful rows of the
//! authentication log. The user can forget the history; the forgotten
//! countries are not re-seeded from the log. Entries older than the retention
//! are ignored and swept; rows are removed when the identity is retired.
use mw_store::Store;
use mw_engine::security::{CountryLookup, country_lookup};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::IpAddr};

pub(crate) const PREFIX: &str = mw_store::sec_state::ACCOUNT_GEO_PREFIX;
pub(crate) const MAX_COUNTRIES: usize = 64;
const SWEEP_EVERY_SECS: i64 = 3600;
pub(crate) const DAYS_KEY: &str = "security.signin.accountGeoHistoryDays";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Entry {
    pub cc: String,
    pub first: i64,
    pub last: i64,
    pub n: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct History {
    #[serde(default)]
    pub v: u8,
    /// The log has been consulted once for this account.
    #[serde(default)]
    pub seeded: bool,
    /// When the user last forgot the history; log rows before it are not reseeded.
    #[serde(default)]
    pub cleared: i64,
    #[serde(default)]
    pub c: Vec<Entry>,
}

impl History {
    /// Note a successful sign-in from `cc` at `now`.
    pub(crate) fn record(&mut self, cc: &str, now: i64) {
        self.v = 1;
        self.seeded = true;
        if let Some(e) = self.c.iter_mut().find(|e| e.cc == cc) {
            e.last = e.last.max(now);
            e.first = e.first.min(now);
            e.n = e.n.saturating_add(1);
            return;
        }
        if self.c.len() >= MAX_COUNTRIES
            && let Some(oldest) = self
                .c
                .iter()
                .enumerate()
                .min_by_key(|(_, e)| e.last)
                .map(|(i, _)| i)
        {
            self.c.remove(oldest);
        }
        self.c.push(Entry { cc: cc.to_owned(), first: now, last: now, n: 1 });
    }
    /// Countries last seen within `days` of `now`.
    pub(crate) fn active(&self, now: i64, days: i64) -> BTreeSet<String> {
        let floor = now - days * 86_400;
        self.c.iter().filter(|e| e.last >= floor).map(|e| e.cc.clone()).collect()
    }
    /// Drop entries older than `days`; true when anything was removed.
    pub(crate) fn prune(&mut self, now: i64, days: i64) -> bool {
        let floor = now - days * 86_400;
        let before = self.c.len();
        self.c.retain(|e| e.last >= floor);
        self.c.len() != before
    }
    /// The user forgets the history (the log is not consulted again for what
    /// happened before `now`).
    pub(crate) fn clear(&mut self, now: i64) {
        self.v = 1;
        self.seeded = true;
        self.cleared = now;
        self.c.clear();
    }
}

fn key(account: &str) -> String {
    format!("{PREFIX}{account}")
}

pub(crate) async fn retention_days(store: &Store) -> i64 {
    crate::settings_registry::effective_for(&store, DAYS_KEY, &Default::default())
        .await
        .ok()
        .flatten()
        .and_then(|e| e.value.as_i64())
        .unwrap_or(365)
        .clamp(30, 3650)
}

pub(crate) async fn load(store: &Store, account: &str) -> Option<History> {
    let raw = store.get_setting(&key(account)).await.ok().flatten()?;
    serde_json::from_str(&raw).ok()
}
async fn save(store: &Store, account: &str, history: &History) -> bool {
    match serde_json::to_string(history) {
        Ok(raw) => store.set_setting(&key(account), &raw).await.is_ok(),
        Err(_) => false,
    }
}

/// Build the entries a never-seeded account gets from the authentication log.
async fn seed(store: &Store, account: &str, cleared: i64) -> History {
    let events = store
        .signin_history(account, i64::MAX, Some("success"), 200)
        .await
        .unwrap_or_default();
    let mut history = History { v: 1, seeded: true, cleared, c: Vec::new() };
    // Oldest first so first/last come out right.
    for event in events.iter().rev().filter(|e| e.ts > cleared) {
        let Some(ip) = event.ip.parse::<IpAddr>().ok() else { continue };
        if let CountryLookup::Known(cc) = country_lookup(Some(ip)) {
            history.record(&cc, event.ts);
        }
    }
    history
}

/// The countries the account has signed in from within the retention. An
/// account with no history row is seeded once from the log (and the result
/// saved); an empty set means "first sign-in".
pub(crate) async fn usual_countries(store: &Store, account: &str) -> BTreeSet<String> {
    let now = chrono::Utc::now().timestamp();
    let days = retention_days(store).await;
    let history = match load(store, account).await {
        Some(h) if h.seeded => h,
        other => {
            let h = seed(store, account, other.map_or(0, |h| h.cleared)).await;
            save(store, account, &h).await;
            h
        }
    };
    history.active(now, days)
}

/// A sign-in from `country` completed.
pub(crate) async fn record_success(store: &Store, account: &str, country: &str) {
    let now = chrono::Utc::now().timestamp();
    let mut history = match load(store, account).await {
        Some(h) if h.seeded => h,
        other => seed(store, account, other.map_or(0, |h| h.cleared)).await,
    };
    history.record(country, now);
    save(store, account, &history).await;
    sweep_if_due(store, now);
}

/// The user (or an administrator for that user) forgets the history.
pub(crate) async fn forget(store: &Store, account: &str) -> bool {
    let mut history = load(store, account).await.unwrap_or_default();
    history.clear(chrono::Utc::now().timestamp());
    save(store, account, &history).await
}

/// What the Settings page shows: the countries within the retention.
pub(crate) async fn view(store: &Store, account: &str) -> serde_json::Value {
    let now = chrono::Utc::now().timestamp();
    let days = retention_days(store).await;
    let mut entries: Vec<Entry> = load(store, account)
        .await
        .map(|h| h.c)
        .unwrap_or_default()
        .into_iter()
        .filter(|e| e.last >= now - days * 86_400)
        .collect();
    entries.sort_by_key(|e| std::cmp::Reverse(e.last));
    serde_json::json!({ "retentionDays": days, "maxCountries": MAX_COUNTRIES, "countries": entries })
}

fn sweep_if_due(store: &Store, now: i64) {
    use std::sync::atomic::{AtomicI64, Ordering};
    static LAST: AtomicI64 = AtomicI64::new(0);
    let last = LAST.load(Ordering::Relaxed);
    if now - last < SWEEP_EVERY_SECS
        || LAST.compare_exchange(last, now, Ordering::Relaxed, Ordering::Relaxed).is_err()
    {
        return;
    }
    let store = store.clone();
    tokio::spawn(async move {
        let days = retention_days(&store).await;
        sweep(&store, now, days).await;
    });
}

/// Remove entries older than `days`, and rows left with nothing to remember.
pub(crate) async fn sweep(store: &Store, now: i64, days: i64) -> usize {
    let mut removed = 0;
    let Ok(rows) = store.settings_with_prefix(PREFIX).await else {
        return 0;
    };
    for (row_key, value) in rows {
        let Ok(mut history) = serde_json::from_str::<History>(&value) else {
            continue;
        };
        let changed = history.prune(now, days);
        // A forgotten history keeps its marker for the retention, so the log is
        // not consulted for what the user removed.
        let marker = history.cleared > now - days * 86_400;
        if history.c.is_empty() && !marker {
            if store.delete_sec_state(&row_key).await.unwrap_or(false) {
                removed += 1;
            }
        } else if changed
            && let Ok(raw) = serde_json::to_string(&history)
            && store.set_setting(&row_key, &raw).await.is_ok()
        {
            removed += 1;
        }
    }
    removed
}

#[cfg(test)]
mod tests {
    use super::*;
    const DAY: i64 = 86_400;

    #[test]
    fn record_counts_and_tracks_first_and_last() {
        let mut h = History::default();
        h.record("PT", 100);
        h.record("PT", 500);
        h.record("DE", 300);
        let pt = h.c.iter().find(|e| e.cc == "PT").unwrap();
        assert_eq!((pt.first, pt.last, pt.n), (100, 500, 2));
        assert_eq!(h.c.len(), 2);
    }

    #[test]
    fn the_list_is_bounded_and_evicts_the_oldest_last_seen() {
        let mut h = History::default();
        for i in 0..MAX_COUNTRIES {
            let cc = format!("{}{}", (b'A' + (i / 26) as u8) as char, (b'A' + (i % 26) as u8) as char);
            h.record(&cc, 1000 + i as i64);
        }
        assert_eq!(h.c.len(), MAX_COUNTRIES);
        h.record("ZZ", 5000);
        assert_eq!(h.c.len(), MAX_COUNTRIES, "still bounded");
        assert!(h.c.iter().any(|e| e.cc == "ZZ"));
        assert!(!h.c.iter().any(|e| e.cc == "AA"), "oldest last-seen went first");
        // Refreshing an old entry protects it.
        h.record("AB", 6000);
        h.record("YY", 7000);
        assert!(h.c.iter().any(|e| e.cc == "AB"));
        assert!(!h.c.iter().any(|e| e.cc == "AC"));
    }

    #[test]
    fn retention_ignores_and_prunes_old_entries() {
        let mut h = History::default();
        h.record("PT", 100 * DAY);
        h.record("DE", 300 * DAY);
        let now = 400 * DAY;
        assert_eq!(h.active(now, 365), ["DE", "PT"].iter().map(|s| s.to_string()).collect());
        assert_eq!(h.active(now, 100), ["DE"].iter().map(|s| s.to_string()).collect());
        assert!(h.prune(now, 100));
        assert_eq!(h.c.len(), 1);
        assert!(!h.prune(now, 100));
    }

    #[test]
    fn the_stored_form_is_small_and_has_no_addresses() {
        let mut h = History::default();
        h.record("PT", 10);
        let raw = serde_json::to_string(&h).unwrap();
        assert_eq!(raw, r#"{"v":1,"seeded":true,"cleared":0,"c":[{"cc":"PT","first":10,"last":10,"n":1}]}"#);
    }

    #[test]
    fn registry_default_and_bounds() {
        let d = crate::settings_registry::lookup(DAYS_KEY).unwrap();
        assert_eq!(d.default_value(), serde_json::json!(365));
        assert!(crate::settings_registry::validate(d, &serde_json::json!(29), "/value").is_err());
        assert!(crate::settings_registry::validate(d, &serde_json::json!(3651), "/value").is_err());
        assert!(crate::settings_registry::validate(d, &serde_json::json!(30), "/value").is_ok());
    }

    async fn state() -> Store {
        Store::open_in_memory(mw_store::ServerKey::generate()).await.unwrap()
    }

    #[tokio::test]
    async fn history_survives_a_restart_sweeps_and_is_forgotten() {
        let store = &state().await;
        let acct = "acct-hist-1";
        let now = chrono::Utc::now().timestamp();
        record_success(store, acct, "PT").await;
        record_success(store, acct, "DE").await;
        // "Restart": nothing in memory is relied on, a fresh load sees the rows.
        let loaded = load(store, acct).await.unwrap();
        assert_eq!(loaded.c.len(), 2);
        assert_eq!(usual_countries(store, acct).await.len(), 2);
        // Sweep with a window that excludes everything but keeps nothing to remember.
        assert_eq!(sweep(store, now + 400 * DAY, 365).await >= 1, true);
        assert!(load(store, acct).await.is_none(), "an empty, never-forgotten row is removed");
        // Forget keeps a marker and does not reseed from the log.
        record_success(store, acct, "PT").await;
        assert!(forget(store, acct).await);
        assert!(usual_countries(store, acct).await.is_empty());
        let marker = load(store, acct).await.unwrap();
        assert!(marker.seeded && marker.c.is_empty() && marker.cleared > 0);
        // Retiring the identity removes the row through the store.
        assert!(store.delete_sec_state(&key(acct)).await.unwrap());
    }
}
