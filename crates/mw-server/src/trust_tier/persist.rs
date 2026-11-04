//! Warm restarts for the known-source table.
//!
//! One row of the existing `settings` table (`trust-tier:v1`, no migration)
//! holds a versioned, size-bounded snapshot: `mw-tt1.<hex hmac>.<json>`.
//!
//! * The source hashes are keyed with `Store::derive_key("mw-trust-tier-v1")`, so
//!   the same address hashes to the same value after a restart. The snapshot is
//!   authenticated with HMAC-SHA256 under `derive_key("mw-trust-tier-mac-v1")`: a
//!   snapshot from another server key, an edited one or a truncated one fails
//!   the check and is ignored (the table then starts from the auth-log rebuild).
//! * What is stored: the keyed hash of the source, its country code, the last-seen
//!   time and up to 16 keyed account hashes (a count). No address, no username.
//! * Entries older than `security.trust.knownTtlDays` are dropped when the
//!   snapshot is written and when it is loaded; at most `knownCapacity` entries
//!   are kept, the oldest last-seen dropped first.
//! * Replicas that share one store share the row: a write first merges the row
//!   already there (max last-seen, union of account hashes), a load merges into
//!   the table. A replica that writes just after another may drop that
//!   replica's newest entries from the row; the next interval restores them.
//! * `forget_all` writes an empty snapshot carrying a `cleared_at` time: entries
//!   and auth-log rows not newer than it are ignored by every replica.
use super::*;
use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

pub(super) const SETTING_KEY: &str = "trust-tier:v1";
const PREFIX_V1: &str = "mw-tt1.";
const VERSION: u32 = 1;
const HASH_LABEL: &str = "mw-trust-tier-v1";
const MAC_LABEL: &str = "mw-trust-tier-mac-v1";
/// A stored value larger than this is not parsed at all.
const MAX_RAW: usize = 24 * 1024 * 1024;
/// The largest `security.trust.knownCapacity`.
const MAX_ENTRIES: usize = 100_000;
const MAX_COUNTRIES: usize = 300;
pub(crate) const MIN_INTERVAL_SECS: u64 = 30;
pub(crate) const MAX_INTERVAL_SECS: u64 = 3600;

/// The write interval, bounded to 30 s .. 1 h.
pub(crate) fn clamp_interval(secs: u64) -> u64 {
    secs.clamp(MIN_INTERVAL_SECS, MAX_INTERVAL_SECS)
}

pub(super) fn salt(store: &mw_store::Store) -> [u8; 32] {
    store.derive_key(HASH_LABEL)
}
fn mac_key(store: &mw_store::Store) -> [u8; 32] {
    store.derive_key(MAC_LABEL)
}

#[derive(Serialize, Deserialize)]
struct Snap {
    v: u32,
    saved_at: i64,
    #[serde(default)]
    cleared_at: i64,
    entries: Vec<SnapEntry>,
    countries: std::collections::BTreeMap<String, i64>,
}
#[derive(Serialize, Deserialize)]
struct SnapEntry {
    k: String,
    c: Option<String>,
    t: i64,
    a: Vec<u64>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Reject {
    Empty,
    TooLarge,
    Version,
    Mac,
    Corrupt,
}
impl Reject {
    fn text(&self) -> &'static str {
        match self {
            Reject::Empty => "empty",
            Reject::TooLarge => "too large",
            Reject::Version => "unsupported version",
            Reject::Mac => "failed the integrity check",
            Reject::Corrupt => "malformed",
        }
    }
}

pub(super) struct Restored {
    entries: Vec<([u8; 16], Known)>,
    countries: HashMap<String, i64>,
    cleared_at: i64,
}

fn hex16(s: &str) -> Option<[u8; 16]> {
    if s.len() != 32 || !s.is_ascii() {
        return None;
    }
    let mut out = [0u8; 16];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(out)
}
fn tag(key: &[u8; 32], body: &[u8]) -> Hmac<sha2::Sha256> {
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(key).expect("any key length");
    mac.update(body);
    mac
}
fn country_ok(c: &str) -> bool {
    c.len() == 2 && c.bytes().all(|b| b.is_ascii_uppercase())
}

impl KnownStore {
    pub(super) fn from_parts(
        salt: [u8; 32],
        entries: HashMap<[u8; 16], Known>,
        countries: HashMap<String, i64>,
        cleared_at: i64,
    ) -> Self {
        Self {
            salt,
            entries,
            countries,
            cleared_at,
            ..Self::new()
        }
    }
    fn snap(&self, now: i64, ttl: i64, capacity: usize) -> Snap {
        let mut rows: Vec<(&[u8; 16], &Known)> = self
            .entries
            .iter()
            .filter(|(_, e)| now.saturating_sub(e.last_seen) <= ttl && e.last_seen >= self.cleared_at)
            .collect();
        rows.sort_by_key(|(k, e)| (std::cmp::Reverse(e.last_seen), **k));
        rows.truncate(capacity.clamp(1, MAX_ENTRIES));
        let mut countries: Vec<(&String, &i64)> = self
            .countries
            .iter()
            .filter(|(_, t)| now.saturating_sub(**t) <= ttl && **t >= self.cleared_at)
            .collect();
        countries.sort_by_key(|(c, t)| (std::cmp::Reverse(**t), (*c).clone()));
        countries.truncate(MAX_COUNTRIES);
        Snap {
            v: VERSION,
            saved_at: now,
            cleared_at: self.cleared_at,
            entries: rows
                .into_iter()
                .map(|(k, e)| SnapEntry {
                    k: crate::sec_cfg::hex(k),
                    c: e.country.clone(),
                    t: e.last_seen,
                    a: e.accounts.clone(),
                })
                .collect(),
            countries: countries.into_iter().map(|(c, t)| (c.clone(), *t)).collect(),
        }
    }
    /// Merge a restored snapshot in: max last-seen, union of the capped account
    /// hashes, the newer side's country. Entries past the window or not newer than
    /// `cleared_at` are dropped; the table is capped to `capacity`. Returns the
    /// number of restored entries that were taken.
    pub(super) fn merge_restored(
        &mut self,
        r: Restored,
        now: i64,
        ttl: i64,
        capacity: usize,
    ) -> usize {
        self.cleared_at = self.cleared_at.max(r.cleared_at);
        let floor = self.cleared_at;
        self.entries.retain(|_, e| e.last_seen >= floor);
        let mut taken = 0;
        for (key, row) in r.entries {
            if now.saturating_sub(row.last_seen) > ttl || row.last_seen < floor {
                continue;
            }
            taken += 1;
            match self.entries.get_mut(&key) {
                Some(e) => {
                    if row.last_seen > e.last_seen {
                        e.last_seen = row.last_seen;
                        if row.country.is_some() {
                            e.country = row.country;
                        }
                    }
                    for a in row.accounts {
                        if e.accounts.len() < MAX_ACCOUNTS_PER_SOURCE && !e.accounts.contains(&a) {
                            e.accounts.push(a);
                        }
                    }
                }
                None => {
                    self.entries.insert(key, row);
                }
            }
        }
        for (cc, t) in r.countries {
            if now.saturating_sub(t) <= ttl && t >= floor {
                let slot = self.countries.entry(cc).or_insert(t);
                *slot = (*slot).max(t);
            }
        }
        let cap = capacity.clamp(1, MAX_ENTRIES);
        if self.entries.len() > cap {
            let mut by_age: Vec<([u8; 16], i64)> =
                self.entries.iter().map(|(k, e)| (*k, e.last_seen)).collect();
            by_age.sort_by_key(|(k, t)| (std::cmp::Reverse(*t), *k));
            for (k, _) in by_age.into_iter().skip(cap) {
                self.entries.remove(&k);
            }
        }
        taken
    }
}

fn seal(snap: &Snap, key: &[u8; 32]) -> String {
    let json = serde_json::to_string(snap).unwrap_or_default();
    let mac = tag(key, json.as_bytes()).finalize().into_bytes();
    format!("{PREFIX_V1}{}.{json}", crate::sec_cfg::hex(&mac))
}

/// Validate a stored value. Nothing is trusted before the MAC is checked.
pub(super) fn open(raw: &str, key: &[u8; 32]) -> Result<Restored, Reject> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(Reject::Empty);
    }
    if raw.len() > MAX_RAW {
        return Err(Reject::TooLarge);
    }
    let Some(rest) = raw.strip_prefix(PREFIX_V1) else {
        return Err(if raw.starts_with("mw-tt") {
            Reject::Version
        } else {
            Reject::Corrupt
        });
    };
    let (mac_hex, json) = rest.split_once('.').ok_or(Reject::Corrupt)?;
    if mac_hex.len() != 64 || !mac_hex.is_ascii() {
        return Err(Reject::Corrupt);
    }
    let mut mac_bytes = [0u8; 32];
    for (i, b) in mac_bytes.iter_mut().enumerate() {
        *b = u8::from_str_radix(&mac_hex[i * 2..i * 2 + 2], 16).map_err(|_| Reject::Corrupt)?;
    }
    tag(key, json.as_bytes())
        .verify_slice(&mac_bytes)
        .map_err(|_| Reject::Mac)?;
    let snap: Snap = serde_json::from_str(json).map_err(|_| Reject::Corrupt)?;
    if snap.v != VERSION {
        return Err(Reject::Version);
    }
    if snap.entries.len() > MAX_ENTRIES || snap.countries.len() > MAX_COUNTRIES {
        return Err(Reject::TooLarge);
    }
    let mut entries = Vec::with_capacity(snap.entries.len());
    for e in snap.entries {
        let Some(k) = hex16(&e.k) else {
            return Err(Reject::Corrupt);
        };
        let country = match e.c {
            Some(c) if country_ok(&c) => Some(c),
            Some(_) => return Err(Reject::Corrupt),
            None => None,
        };
        let mut accounts = e.a;
        accounts.truncate(MAX_ACCOUNTS_PER_SOURCE);
        entries.push((
            k,
            Known {
                country,
                last_seen: e.t,
                accounts,
            },
        ));
    }
    let mut countries = HashMap::new();
    for (c, t) in snap.countries {
        if !country_ok(&c) {
            return Err(Reject::Corrupt);
        }
        countries.insert(c, t);
    }
    Ok(Restored {
        entries,
        countries,
        cleared_at: snap.cleared_at,
    })
}

/// Load the stored snapshot into the table. Never fails startup.
pub(super) async fn restore(store: &mw_store::Store, policy: &Policy) {
    if !policy.persist {
        return;
    }
    let raw = match store.get_setting(SETTING_KEY).await {
        Ok(Some(raw)) => raw,
        Ok(None) => return,
        Err(e) => {
            tracing::warn!(target: "trust_tier", "saved known sources not read: {e}");
            return;
        }
    };
    let now = chrono::Utc::now().timestamp();
    match open(&raw, &mac_key(store)) {
        Ok(r) => {
            let mut k = lock();
            if k.owner != Some(store.instance_id()) {
                return;
            }
            let n = k.merge_restored(r, now, policy.known_ttl_secs, policy.capacity);
            k.restored = n;
            drop(k);
            tracing::info!(target: "trust_tier", "restored {n} known source(s) from the saved table");
        }
        Err(Reject::Empty) => {}
        Err(why) => {
            tracing::warn!(target: "trust_tier", "saved known sources ignored: {}", why.text());
        }
    }
}

static IN_FLIGHT: AtomicBool = AtomicBool::new(false);

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Saved {
    Skipped,
    Written(usize),
}

/// Write the table if it changed (or `force`). Skips when persistence is off,
/// when another write is running or when the table belongs to another store.
pub(super) async fn save(store: &mw_store::Store, force: bool) -> Result<Saved, String> {
    let policy = snapshot();
    if !policy.persist {
        return Ok(Saved::Skipped);
    }
    if IN_FLIGHT.swap(true, Ordering::AcqRel) {
        return Ok(Saved::Skipped);
    }
    let out = write(store, &policy, force, true).await;
    IN_FLIGHT.store(false, Ordering::Release);
    out
}

async fn write(
    store: &mw_store::Store,
    policy: &Policy,
    force: bool,
    merge_existing: bool,
) -> Result<Saved, String> {
    let now = chrono::Utc::now().timestamp();
    let (salt, entries, countries, cleared_at) = {
        let mut k = lock();
        if k.owner != Some(store.instance_id()) || (!k.dirty && !force) {
            return Ok(Saved::Skipped);
        }
        k.dirty = false;
        (k.salt, k.entries.clone(), k.countries.clone(), k.cleared_at)
    };
    let key = mac_key(store);
    let mut view = KnownStore::from_parts(salt, entries, countries, cleared_at);
    if merge_existing && let Ok(Some(raw)) = store.get_setting(SETTING_KEY).await {
        // A foreign, stale or broken row is simply replaced.
        if let Ok(r) = open(&raw, &key) {
            view.merge_restored(r, now, policy.known_ttl_secs, policy.capacity);
        }
    }
    let snap = view.snap(now, policy.known_ttl_secs, policy.capacity);
    let n = snap.entries.len();
    match store.set_setting(SETTING_KEY, &seal(&snap, &key)).await {
        Ok(()) => {
            let mut k = lock();
            if k.owner == Some(store.instance_id()) {
                k.last_saved = Some(now);
            }
            Ok(Saved::Written(n))
        }
        Err(e) => {
            let mut k = lock();
            if k.owner == Some(store.instance_id()) {
                k.dirty = true;
            }
            Err(e.to_string())
        }
    }
}

/// Forget every known source: the table, the saved copy and, through
/// `cleared_at`, the auth-log rebuild of earlier sign-ins.
pub(crate) async fn forget_all(store: &mw_store::Store) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp();
    {
        let mut k = lock();
        if k.owner == Some(store.instance_id()) {
            let (salt, owner) = (k.salt, k.owner);
            *k = KnownStore::from_parts(salt, HashMap::new(), HashMap::new(), now);
            k.owner = owner;
            k.hydrated = true;
        }
    }
    let key = mac_key(store);
    let cleared = KnownStore::from_parts(salt(store), HashMap::new(), HashMap::new(), now);
    let snap = cleared.snap(now, 0, 1);
    // Written even with persistence off: the point is to remove what was saved.
    let had_row = matches!(store.get_setting(SETTING_KEY).await, Ok(Some(_)));
    if had_row || snapshot().persist {
        store
            .set_setting(SETTING_KEY, &seal(&snap, &key))
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Readout for the admin console.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Status {
    pub persist: bool,
    pub interval_secs: u64,
    pub entries: usize,
    pub restored_at_startup: usize,
    pub last_saved: Option<i64>,
}

pub(crate) fn status() -> Status {
    let policy = snapshot();
    let k = lock();
    Status {
        persist: policy.persist,
        interval_secs: clamp_interval(u64::from(policy.persist_interval_secs)),
        entries: k.entries.len(),
        restored_at_startup: k.restored,
        last_saved: k.last_saved,
    }
}

static STARTED: std::sync::Mutex<Option<mw_store::Store>> = std::sync::Mutex::new(None);

/// Load the saved table now (before serving), then keep it saved. The task saves
/// only when the table changed and the interval has passed; a failure is logged
/// once per attempt and never affects a lookup.
pub(crate) async fn start(store: mw_store::Store) {
    let _ = tokio::time::timeout(Duration::from_secs(10), policy(&store)).await;
    *STARTED.lock().unwrap_or_else(|p| p.into_inner()) = Some(store.clone());
    tokio::spawn(async move {
        let mut last = Instant::now();
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            let secs = clamp_interval(u64::from(snapshot().persist_interval_secs));
            if last.elapsed() < Duration::from_secs(secs) {
                continue;
            }
            last = Instant::now();
            if let Err(e) = save(&store, false).await {
                tracing::warn!(target: "trust_tier", "known sources not saved (retrying in {secs}s): {e}");
            }
        }
    });
}

/// Best-effort final save on graceful shutdown (at most 3 s).
pub async fn flush_on_shutdown() {
    let store = STARTED.lock().unwrap_or_else(|p| p.into_inner()).clone();
    let Some(store) = store else {
        return;
    };
    match tokio::time::timeout(Duration::from_secs(3), async {
        // Wait out a write already running rather than skipping the final one.
        for _ in 0..30 {
            if !IN_FLIGHT.load(Ordering::Acquire) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        save(&store, false).await
    })
    .await
    {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => tracing::warn!(target: "trust_tier", "known sources not saved at shutdown: {e}"),
        Err(_) => tracing::warn!(target: "trust_tier", "known sources not saved at shutdown: timed out"),
    }
}

#[cfg(test)]
mod tests;
