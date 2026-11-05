//! Two-tier (RAM + disk) cache of already re-encoded remote images, with one
//! in-memory index as the source of truth for both tiers.
//!
//! Only jail output is ever stored. Authentication, the display grant and the SSRF
//! gate run in the handlers BEFORE this module is asked anything, and this module
//! has no way to skip them: a lookup is a function of the key alone and is only
//! reached after the handler has authorised the request.
//!
//! * Lookup order is RAM, then disk, then the network. The index says in O(1), with
//!   no disk IO, whether an entry exists and where it lives.
//! * A disk hit that scores above the promote threshold moves into RAM. A RAM
//!   eviction demotes to disk when the entry is worth a write; the write happens in
//!   a background worker, never on the request path.
//! * File names are 128-bit hashes of (route, URL); the URL is never written.
//! * The lock is never held across IO. Request-path work is O(log n): a hit
//!   rescoring one entry, an eviction popping the lowest score.
//! * Persistence of hit counts and extended lifetimes is lazy: one atomic
//!   index snapshot per interval, not a write per hit.
use std::collections::{BTreeSet, HashMap, VecDeque};
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use sha2::{Digest, Sha256};

pub(crate) type Hash = u128;

const ENTRY_OVERHEAD: usize = 256;
const FILE_OVERHEAD: usize = 64;
const MAGIC: &str = "MWIC1";
const HEADER_MAX: usize = 512;
pub(crate) const SNAPSHOT: &str = "index.snap";
const RECENT_AFTER_MS: u64 = 10 * 60 * 1000;
const DEGRADED_BACKOFF_MS: u64 = 30_000;
const REQUEST_PATH_EVICTIONS: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tier {
    Ram,
    Disk,
}

/// What the request that touched an entry was for. The server knows an image's
/// message only through the grant it just checked, so this is `Open` when the
/// request names a message and `Background` otherwise; an `Open` entry
/// idle for ten minutes is weighed as recent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Context {
    Open,
    Background,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DiskConfig {
    pub enabled: bool,
    /// `None` with `enabled` means no usable path (relative, or `..`).
    pub path: Option<PathBuf>,
    pub auto: bool,
    pub manual_bytes: u64,
    pub floor_bytes: u64,
    pub ceiling_bytes: u64,
    pub entries: usize,
    pub entry_bytes: usize,
    pub min_hits: u32,
    pub write_queue_bytes: usize,
}
impl Default for DiskConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            path: None,
            auto: true,
            manual_bytes: 512 << 20,
            floor_bytes: 64 << 20,
            ceiling_bytes: 4 << 30,
            entries: 16_384,
            entry_bytes: 8 << 20,
            min_hits: 1,
            write_queue_bytes: 8 << 20,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TierConfig {
    pub ram_bytes: usize,
    pub ram_entries: usize,
    pub ram_entry_bytes: usize,
    /// Cold-entry lifetime, seconds. Zero disables the cache.
    pub ttl_base_secs: u64,
    pub ttl_step_secs: u64,
    pub ttl_max_secs: u64,
    pub max_age_secs: u64,
    pub half_life_secs: u64,
    /// Demote below this score, in thousandths.
    pub demote_milli: u32,
    /// Promote at `demote * (1 + hysteresis / 100)`.
    pub hysteresis_pct: u32,
    pub open_pct: u32,
    pub recent_pct: u32,
    pub account_share_pct: u32,
    pub sweep_secs: u64,
    pub snapshot_secs: u64,
    pub sweep_batch: usize,
    pub disk: DiskConfig,
}
impl Default for TierConfig {
    fn default() -> Self {
        Self {
            ram_bytes: 32 << 20,
            ram_entries: 1024,
            ram_entry_bytes: 4 << 20,
            ttl_base_secs: 3600,
            ttl_step_secs: 600,
            ttl_max_secs: 86_400,
            max_age_secs: 604_800,
            half_life_secs: 3600,
            demote_milli: 300,
            hysteresis_pct: 50,
            open_pct: 200,
            recent_pct: 150,
            account_share_pct: 50,
            sweep_secs: 30,
            snapshot_secs: 300,
            sweep_batch: 256,
            disk: DiskConfig::default(),
        }
    }
}

// ── pure functions ─────────────────────────────────────────────────────────────

/// Exponentially decayed hit count after `idle_ms` without a hit.
pub(crate) fn decay(hits: f64, idle_ms: u64, half_life_ms: u64) -> f64 {
    if hits <= 0.0 {
        return 0.0;
    }
    hits * 0.5f64.powf(idle_ms as f64 / half_life_ms.max(1) as f64)
}

/// Context weight in percent. An `Open` entry that has been idle for ten minutes
/// counts as recent.
pub(crate) fn context_percent(ctx: Context, idle_ms: u64, cfg: &TierConfig) -> u32 {
    match ctx {
        Context::Open if idle_ms <= RECENT_AFTER_MS => cfg.open_pct,
        Context::Open => cfg.recent_pct,
        Context::Background => 100,
    }
}

/// Priority score: value per byte of footprint. Value grows with the log of the
/// decayed hit count, shrinks with idle time and scales with usage context; cost
/// is `1 + size / 32 KiB`, so a large image needs proportionally more hits to
/// stay in RAM.
pub(crate) fn score(
    hits: f64,
    size: usize,
    idle_ms: u64,
    context_pct: u32,
    half_life_ms: u64,
) -> f64 {
    let recency = 0.5f64.powf(idle_ms as f64 / half_life_ms.max(1) as f64);
    let value = (0.2 + (1.0 + hits.max(0.0)).ln()) * (0.25 + 0.75 * recency) * (f64::from(context_pct) / 100.0);
    value / (1.0 + size as f64 / 32768.0)
}

/// Where an entry with this score should live. The promote threshold sits above
/// the demote threshold, so a score between them keeps the current tier.
pub(crate) fn place(score: f64, current: Tier, cfg: &TierConfig) -> Tier {
    let demote = f64::from(cfg.demote_milli) / 1000.0;
    let promote = demote * (1.0 + f64::from(cfg.hysteresis_pct) / 100.0);
    match current {
        Tier::Disk if score >= promote => Tier::Ram,
        Tier::Ram if score < demote => Tier::Disk,
        other => other,
    }
}

/// Expiry (unix ms) of a never-hit entry: the base lifetime, capped by max age.
pub(crate) fn initial_expiry(cfg: &TierConfig, created: u64) -> u64 {
    (created + cfg.ttl_base_secs * 1000).min(created + cfg.max_age_secs * 1000)
}

/// Expiry after a hit. The remaining lifetime grows by a step that scales with the
/// decayed hit count (`step * (1 + ln(1 + hits))`), is capped at `ttl_max`, and the
/// result never passes `created + max_age`. A zero step leaves the lifetime alone,
/// so cold entries keep the base TTL.
pub(crate) fn extended_expiry(
    cfg: &TierConfig,
    created: u64,
    now: u64,
    current_expiry: u64,
    hits: f64,
) -> u64 {
    let step = (cfg.ttl_step_secs * 1000) as f64 * (1.0 + (1.0 + hits.max(0.0)).ln());
    let max = cfg.ttl_max_secs.max(cfg.ttl_base_secs) * 1000;
    let remaining = (current_expiry.saturating_sub(now) + step as u64).min(max);
    (now + remaining).min(created + cfg.max_age_secs * 1000)
}

/// The most RAM bytes one account may hold. With one account (or none) there is
/// nobody to protect against, so the whole budget is available.
pub(crate) fn account_cap(budget: usize, share_pct: u32, accounts: usize) -> usize {
    if accounts <= 1 {
        budget
    } else {
        (budget as u64 * u64::from(share_pct.clamp(1, 100)) / 100) as usize
    }
}

/// Disk byte budget. Auto: a tenth of the space this cache could use (free plus
/// what it already holds), clamped to floor..ceiling, but never more than half of
/// that space. Unknown free space gives the floor.
pub(crate) fn disk_budget(cfg: &DiskConfig, free: Option<u64>, held: u64) -> u64 {
    if !cfg.auto {
        return cfg.manual_bytes;
    }
    let (lo, hi) = (cfg.floor_bytes.min(cfg.ceiling_bytes), cfg.floor_bytes.max(cfg.ceiling_bytes));
    match free {
        None => lo,
        Some(free) => {
            let usable = free.saturating_add(held);
            (usable / 10).clamp(lo, hi).min(usable / 2)
        }
    }
}

fn key_of(score: f64) -> i64 {
    (score * 1_000_000.0).clamp(0.0, 9.0e15) as i64
}

pub(crate) fn key_hash(route: &str, url: &str) -> Hash {
    let mut h = Sha256::new();
    h.update(route.as_bytes());
    h.update([0u8]);
    h.update(url.as_bytes());
    let d = h.finalize();
    let mut b = [0u8; 16];
    b.copy_from_slice(&d[..16]);
    u128::from_be_bytes(b)
}

pub(crate) fn account_hash(account: &str) -> u64 {
    let d = Sha256::digest(account.as_bytes());
    let mut b = [0u8; 8];
    b.copy_from_slice(&d[..8]);
    u64::from_be_bytes(b)
}

pub(crate) fn file_name(h: Hash) -> String {
    format!("{h:032x}.img")
}

fn parse_name(name: &str) -> Option<Hash> {
    let stem = name.strip_suffix(".img")?;
    (stem.len() == 32 && stem.bytes().all(|b| b.is_ascii_hexdigit()))
        .then(|| u128::from_str_radix(stem, 16).ok())
        .flatten()
}

fn is_our_temp(name: &str) -> bool {
    name.starts_with("mwtmp-") && name.ends_with(".tmp")
}

fn disk_cost(size: usize, etag: &str) -> usize {
    size + FILE_OVERHEAD + etag.len()
}

// ── file format ────────────────────────────────────────────────────────────────

/// `MWIC1 <len> <created> <expires> <hits_milli> <etag>\n` then the payload.
pub(crate) fn encode_file(payload: &[u8], etag: &str, created: u64, expires: u64, hits: f64) -> Vec<u8> {
    let head = format!(
        "{MAGIC} {} {created} {expires} {} {etag}\n",
        payload.len(),
        (hits * 1000.0) as u64
    );
    let mut out = Vec::with_capacity(head.len() + payload.len());
    out.extend_from_slice(head.as_bytes());
    out.extend_from_slice(payload);
    out
}

#[derive(Debug, PartialEq)]
pub(crate) struct FileHead {
    pub len: usize,
    pub created: u64,
    pub expires: u64,
    pub hits: f64,
    pub etag: String,
    pub header_len: usize,
}

pub(crate) fn parse_head(bytes: &[u8]) -> Option<FileHead> {
    let end = bytes.iter().take(HEADER_MAX).position(|b| *b == b'\n')?;
    let line = std::str::from_utf8(&bytes[..end]).ok()?;
    let mut f = line.split(' ');
    if f.next()? != MAGIC {
        return None;
    }
    let len = f.next()?.parse().ok()?;
    let created = f.next()?.parse().ok()?;
    let expires = f.next()?.parse().ok()?;
    let hits: u64 = f.next()?.parse().ok()?;
    let etag = f.next()?.to_string();
    (f.next().is_none() && etag.len() <= 128).then_some(FileHead {
        len,
        created,
        expires,
        hits: hits as f64 / 1000.0,
        etag,
        header_len: end + 1,
    })
}

/// A complete file to its payload, or `None` when it is truncated, has the wrong
/// length for the index entry, or does not hash to the recorded `ETag`.
pub(crate) fn decode_file(bytes: &[u8], expect_len: usize, etag_of: impl Fn(&[u8]) -> String) -> Option<Bytes> {
    let head = parse_head(bytes)?;
    let body = bytes.get(head.header_len..)?;
    (head.len == expect_len && body.len() == expect_len && etag_of(body) == head.etag)
        .then(|| Bytes::copy_from_slice(body))
}

// ── IO backend ─────────────────────────────────────────────────────────────────

pub(crate) struct FileInfo {
    pub name: String,
    pub size: u64,
    pub modified_ms: u64,
}

/// The disk operations the tier needs, so tests can inject a fake. Names are
/// plain file names inside the cache directory; no operation takes a path.
pub(crate) trait DiskIo: Send + Sync {
    fn ensure_dir(&self) -> io::Result<()>;
    fn list(&self, limit: usize) -> io::Result<Vec<FileInfo>>;
    fn read(&self, name: &str) -> io::Result<Vec<u8>>;
    fn read_head(&self, name: &str, max: usize) -> io::Result<Vec<u8>>;
    fn write_atomic(&self, name: &str, data: &[u8]) -> io::Result<()>;
    fn remove(&self, name: &str) -> io::Result<()>;
    fn free_bytes(&self) -> Option<u64>;
}

fn valid_name(name: &str) -> bool {
    name == SNAPSHOT || parse_name(name).is_some() || is_our_temp(name)
        && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'.')
}

pub(crate) struct FsIo {
    dir: PathBuf,
}
impl FsIo {
    pub(crate) fn new(dir: PathBuf) -> Self {
        Self { dir }
    }
    fn path(&self, name: &str) -> io::Result<PathBuf> {
        if valid_name(name) {
            Ok(self.dir.join(name))
        } else {
            Err(io::Error::new(io::ErrorKind::InvalidInput, "bad cache file name"))
        }
    }
}
impl DiskIo for FsIo {
    fn ensure_dir(&self) -> io::Result<()> {
        let existed = self.dir.exists();
        std::fs::create_dir_all(&self.dir)?;
        let meta = std::fs::symlink_metadata(&self.dir)?;
        if !meta.is_dir() {
            return Err(io::Error::other("cache path is not a directory"));
        }
        #[cfg(unix)]
        if !existed {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&self.dir, std::fs::Permissions::from_mode(0o700))?;
        }
        #[cfg(not(unix))]
        let _ = existed;
        // Writability probe: a read-only mount fails here, not on the first demote.
        let probe = format!("mwtmp-probe{}.tmp", std::process::id());
        self.write_atomic_tmp(&probe, b"")?;
        let _ = std::fs::remove_file(self.dir.join(&probe));
        Ok(())
    }
    fn list(&self, limit: usize) -> io::Result<Vec<FileInfo>> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir(&self.dir)?.take(limit) {
            let entry = entry?;
            let meta = entry.metadata()?;
            if !meta.is_file() {
                continue;
            }
            let Some(name) = entry.file_name().to_str().map(str::to_string) else {
                continue;
            };
            let modified_ms = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map_or(0, |d| d.as_millis() as u64);
            out.push(FileInfo { name, size: meta.len(), modified_ms });
        }
        Ok(out)
    }
    fn read(&self, name: &str) -> io::Result<Vec<u8>> {
        std::fs::read(self.path(name)?)
    }
    fn read_head(&self, name: &str, max: usize) -> io::Result<Vec<u8>> {
        use std::io::Read;
        let mut buf = Vec::new();
        std::fs::File::open(self.path(name)?)?.take(max as u64).read_to_end(&mut buf)?;
        Ok(buf)
    }
    fn write_atomic(&self, name: &str, data: &[u8]) -> io::Result<()> {
        let target = self.path(name)?;
        let tmp = format!("mwtmp-{:032x}.tmp", rand_u128());
        self.write_atomic_tmp(&tmp, data)?;
        std::fs::rename(self.dir.join(&tmp), target).inspect_err(|_| {
            let _ = std::fs::remove_file(self.dir.join(&tmp));
        })
    }
    fn remove(&self, name: &str) -> io::Result<()> {
        match std::fs::remove_file(self.path(name)?) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            other => other,
        }
    }
    fn free_bytes(&self) -> Option<u64> {
        free_space(&self.dir)
    }
}
impl FsIo {
    /// Create-new + write + flush; the caller renames. Mode 0600 on unix; on
    /// Windows the file inherits the directory's ACL.
    fn write_atomic_tmp(&self, tmp: &str, data: &[u8]) -> io::Result<()> {
        use std::io::Write;
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        let mut f = opts.open(self.dir.join(tmp))?;
        let result = f.write_all(data).and_then(|()| f.flush());
        if result.is_err() {
            drop(f);
            let _ = std::fs::remove_file(self.dir.join(tmp));
        }
        result
    }
}

fn rand_u128() -> u128 {
    use std::sync::atomic::AtomicU64;
    static N: AtomicU64 = AtomicU64::new(0);
    let mut h = Sha256::new();
    h.update(now_ms().to_be_bytes());
    h.update(N.fetch_add(1, Ordering::Relaxed).to_be_bytes());
    h.update(std::process::id().to_be_bytes());
    h.update(format!("{:?}", std::thread::current().id()).as_bytes());
    let d = h.finalize();
    let mut b = [0u8; 16];
    b.copy_from_slice(&d[..16]);
    u128::from_be_bytes(b)
}

#[cfg(windows)]
fn free_space(dir: &std::path::Path) -> Option<u64> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetDiskFreeSpaceExW(dir: *const u16, caller: *mut u64, total: *mut u64, free: *mut u64) -> i32;
    }
    let wide: Vec<u16> = dir.as_os_str().encode_wide().chain(Some(0)).collect();
    let (mut caller, mut total, mut free) = (0u64, 0u64, 0u64);
    // SAFETY: NUL-terminated wide string and three valid out-pointers that live
    // through the synchronous call.
    let ok = unsafe { GetDiskFreeSpaceExW(wide.as_ptr(), &mut caller, &mut total, &mut free) } != 0;
    ok.then_some(caller)
}

#[cfg(unix)]
fn free_space(dir: &std::path::Path) -> Option<u64> {
    use std::os::unix::ffi::OsStrExt;
    let c = std::ffi::CString::new(dir.as_os_str().as_bytes()).ok()?;
    // SAFETY: zeroed statvfs is a valid out-buffer; `c` is NUL-terminated.
    let mut s: libc::statvfs = unsafe { std::mem::zeroed() };
    // SAFETY: both pointers are valid for the call.
    (unsafe { libc::statvfs(c.as_ptr(), &mut s) } == 0)
        .then(|| (s.f_bavail as u64).saturating_mul(s.f_frsize as u64))
}

#[cfg(not(any(windows, unix)))]
fn free_space(_dir: &std::path::Path) -> Option<u64> {
    None
}

pub(crate) fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

// ── the index ──────────────────────────────────────────────────────────────────

struct Entry {
    size: usize,
    etag: String,
    ram: Option<Bytes>,
    /// Bytes waiting for the background writer; they serve hits meanwhile.
    pending: Option<Bytes>,
    on_disk: bool,
    created: u64,
    expires: u64,
    last_used: u64,
    hits: f64,
    total_hits: u64,
    ctx: Context,
    account: u64,
    skey: i64,
    /// Tie-break in the ordered sets: the `last_used` the entry was last scored at, so
    /// equal scores evict oldest first.
    kt: u64,
    stamp: u64,
    dirty: bool,
}
impl Entry {
    fn on_disk_or_pending(&self) -> bool {
        self.on_disk || self.pending.is_some()
    }
    fn tier(&self) -> Tier {
        if self.ram.is_some() { Tier::Ram } else { Tier::Disk }
    }
}

#[derive(Default)]
struct Acct {
    bytes: usize,
    set: BTreeSet<(i64, u64, Hash)>,
}

enum Job {
    Write { hash: Hash, stamp: u64 },
    Remove(String),
}

#[derive(Default, Clone, Debug)]
pub(crate) struct Counters {
    pub ram_hits: u64,
    pub disk_hits: u64,
    pub misses: u64,
    pub promotions: u64,
    pub demotions: u64,
    pub drops: u64,
    pub disk_errors: u64,
    pub drift_recoveries: u64,
}

struct State {
    cfg: TierConfig,
    map: HashMap<Hash, Entry>,
    ram_set: BTreeSet<(i64, u64, Hash)>,
    disk_set: BTreeSet<(i64, u64, Hash)>,
    expiry: BTreeSet<(u64, Hash)>,
    accts: HashMap<u64, Acct>,
    ring: VecDeque<Hash>,
    ram_bytes: usize,
    pending_bytes: usize,
    disk_bytes: u64,
    disk_budget: u64,
    io: Option<Arc<dyn DiskIo>>,
    degraded_until: u64,
    last_error: Option<String>,
    jobs: VecDeque<Job>,
    stamp: u64,
    counters: Counters,
    promoted_at: VecDeque<u64>,
    last_snapshot: u64,
    snapshot_dirty: bool,
    last_free_check: u64,
    last_free: Option<u64>,
}

pub(crate) enum Probe {
    Miss,
    Hit {
        etag: String,
        png: Bytes,
        #[cfg_attr(not(test), allow(dead_code))]
        tier: Tier,
    },
    NeedDisk { name: String, size: usize },
}

impl State {
    fn new(cfg: TierConfig) -> Self {
        Self {
            cfg,
            map: HashMap::new(),
            ram_set: BTreeSet::new(),
            disk_set: BTreeSet::new(),
            expiry: BTreeSet::new(),
            accts: HashMap::new(),
            ring: VecDeque::new(),
            ram_bytes: 0,
            pending_bytes: 0,
            disk_bytes: 0,
            disk_budget: 0,
            io: None,
            degraded_until: 0,
            last_error: None,
            jobs: VecDeque::new(),
            stamp: 0,
            counters: Counters::default(),
            promoted_at: VecDeque::new(),
            last_snapshot: 0,
            snapshot_dirty: false,
            last_free_check: 0,
            last_free: None,
        }
    }

    /// RAM the cache itself occupies: payloads, bytes queued for the disk writer,
    /// and a fixed overhead per index entry.
    fn ram_used(&self) -> usize {
        self.ram_bytes + self.pending_bytes + self.map.len() * ENTRY_OVERHEAD
    }

    fn disk_usable(&self, now: u64) -> bool {
        self.cfg.disk.enabled && self.io.is_some() && now >= self.degraded_until && self.disk_budget > 0
    }

    fn rescored_key(&self, e: &Entry, now: u64) -> i64 {
        let idle = now.saturating_sub(e.last_used);
        let h = decay(e.hits, idle, self.cfg.half_life_secs * 1000);
        let pct = context_percent(e.ctx, idle, &self.cfg);
        key_of(score(h, e.size, idle, pct, self.cfg.half_life_secs * 1000))
    }

    fn set_remove(&mut self, h: Hash) {
        let Some(e) = self.map.get(&h) else { return };
        let k = (e.skey, e.kt, h);
        if e.ram.is_some() {
            self.ram_set.remove(&k);
            if let Some(a) = self.accts.get_mut(&e.account) {
                a.set.remove(&k);
            }
        }
        if e.on_disk_or_pending() {
            self.disk_set.remove(&k);
        }
    }

    fn set_insert(&mut self, h: Hash) {
        let Some(e) = self.map.get(&h) else { return };
        let k = (e.skey, e.kt, h);
        if e.ram.is_some() {
            self.ram_set.insert(k);
            self.accts.entry(e.account).or_default().set.insert(k);
        }
        if e.on_disk_or_pending() {
            self.disk_set.insert(k);
        }
    }

    fn rescore(&mut self, h: Hash, now: u64) {
        let Some(e) = self.map.get(&h) else { return };
        let new = self.rescored_key(e, now);
        if new == e.skey && e.kt == e.last_used {
            return;
        }
        self.set_remove(h);
        if let Some(e) = self.map.get_mut(&h) {
            e.skey = new;
            e.kt = e.last_used;
        }
        self.set_insert(h);
    }

    fn queue_remove_file(&mut self, h: Hash) {
        self.jobs.push_back(Job::Remove(file_name(h)));
    }

    /// Drop an entry from every structure and queue deletion of its file.
    fn remove_entry(&mut self, h: Hash) {
        self.set_remove(h);
        let Some(e) = self.map.remove(&h) else { return };
        self.expiry.remove(&(e.expires, h));
        if let Some(b) = &e.ram {
            self.ram_bytes = self.ram_bytes.saturating_sub(b.len());
            if let Some(a) = self.accts.get_mut(&e.account) {
                a.bytes = a.bytes.saturating_sub(b.len());
                if a.bytes == 0 && a.set.is_empty() {
                    self.accts.remove(&e.account);
                }
            }
        }
        if let Some(b) = &e.pending {
            self.pending_bytes = self.pending_bytes.saturating_sub(b.len());
        }
        if e.on_disk_or_pending() {
            self.disk_bytes = self.disk_bytes.saturating_sub(disk_cost(e.size, &e.etag) as u64);
        }
        if e.on_disk {
            self.queue_remove_file(h);
        }
        self.snapshot_dirty = true;
    }

    /// Drop only the RAM copy; the entry stays on disk.
    fn drop_ram_copy(&mut self, h: Hash) {
        self.set_remove(h);
        if let Some(e) = self.map.get_mut(&h)
            && let Some(b) = e.ram.take()
        {
            self.ram_bytes = self.ram_bytes.saturating_sub(b.len());
            if let Some(a) = self.accts.get_mut(&e.account) {
                a.bytes = a.bytes.saturating_sub(b.len());
                if a.bytes == 0 {
                    a.set.clear();
                }
            }
        }
        self.set_insert(h);
    }

    /// Evict one RAM entry: free if already on disk, queue a write when it is worth
    /// one, otherwise forget it.
    fn demote_or_drop(&mut self, h: Hash, now: u64) {
        let Some(e) = self.map.get(&h) else { return };
        let (on_disk, size, hits) = (e.on_disk_or_pending(), e.size, e.hits);
        let score_ok = (e.skey as f64) / 1_000_000.0 >= f64::from(self.cfg.demote_milli) / 1000.0;
        let worth = hits >= f64::from(self.cfg.disk.min_hits)
            && score_ok
            && size <= self.cfg.disk.entry_bytes
            && self.disk_usable(now)
            && self.pending_bytes + size <= self.cfg.disk.write_queue_bytes;
        if on_disk {
            self.drop_ram_copy(h);
            self.counters.demotions += 1;
        } else if worth && self.make_room_disk(disk_cost(size, &self.map[&h].etag) as u64, now, Some(h)) {
            self.set_remove(h);
            let g = self.next_stamp();
            let e = self.map.get_mut(&h).expect("entry exists");
            let bytes = e.ram.take().expect("ram entry");
            self.ram_bytes = self.ram_bytes.saturating_sub(bytes.len());
            if let Some(a) = self.accts.get_mut(&e.account) {
                a.bytes = a.bytes.saturating_sub(bytes.len());
                if a.bytes == 0 {
                    a.set.clear();
                }
            }
            self.pending_bytes += bytes.len();
            self.disk_bytes += disk_cost(e.size, &e.etag) as u64;
            e.pending = Some(bytes);
            e.stamp = g;
            self.set_insert(h);
            self.jobs.push_back(Job::Write { hash: h, stamp: g });
            self.counters.demotions += 1;
        } else {
            self.remove_entry(h);
            self.counters.drops += 1;
        }
    }

    fn next_stamp(&mut self) -> u64 {
        self.stamp += 1;
        self.stamp
    }

    /// Free disk space for `incoming` bytes by evicting the lowest-scoring disk
    /// entries. False when it cannot (budget smaller than the entry).
    fn make_room_disk(&mut self, incoming: u64, _now: u64, keep: Option<Hash>) -> bool {
        if incoming > self.disk_budget {
            return false;
        }
        let mut steps = 0;
        while (self.disk_bytes + incoming > self.disk_budget
            || self.disk_set.len() >= self.cfg.disk.entries)
            && steps < REQUEST_PATH_EVICTIONS * 4
        {
            steps += 1;
            let Some(&(_, _, victim)) = self.disk_set.iter().find(|(_, _, h)| Some(*h) != keep) else {
                return false;
            };
            if self.map.get(&victim).is_some_and(|e| e.ram.is_some()) {
                // Keep the RAM copy, lose only the disk one.
                self.set_remove(victim);
                let e = self.map.get_mut(&victim).expect("entry");
                let cost = disk_cost(e.size, &e.etag) as u64;
                let was_disk = e.on_disk;
                if let Some(p) = e.pending.take() {
                    self.pending_bytes = self.pending_bytes.saturating_sub(p.len());
                }
                e.on_disk = false;
                e.stamp = 0;
                self.disk_bytes = self.disk_bytes.saturating_sub(cost);
                self.set_insert(victim);
                if was_disk {
                    self.queue_remove_file(victim);
                }
            } else {
                self.remove_entry(victim);
                self.counters.drops += 1;
            }
        }
        self.disk_bytes + incoming <= self.disk_budget && self.disk_set.len() < self.cfg.disk.entries
    }

    /// Evict from RAM until `incoming` more bytes and one more entry fit, taking
    /// from `own` (an account over its share) before the global lowest score.
    fn make_room_ram(&mut self, incoming: usize, own: Option<u64>, now: u64, max_steps: usize) {
        let mut steps = 0;
        if let Some(acct) = own {
            let known = self.accts.len() + usize::from(!self.accts.contains_key(&acct));
            let cap = account_cap(self.cfg.ram_bytes, self.cfg.account_share_pct, known);
            while self.accts.get(&acct).is_some_and(|a| a.bytes + incoming > cap) && steps < max_steps {
                steps += 1;
                let Some(&(_, _, victim)) = self.accts.get(&acct).and_then(|a| a.set.iter().next()) else {
                    break;
                };
                self.demote_or_drop(victim, now);
            }
        }
        let (extra, slot) = if incoming > 0 { (incoming + ENTRY_OVERHEAD, 1) } else { (0, 0) };
        while (self.ram_used() + extra > self.cfg.ram_bytes
            || self.ram_set.len() + slot > self.cfg.ram_entries)
            && steps < max_steps
        {
            steps += 1;
            let Some(&(_, _, victim)) = self.ram_set.iter().next() else { break };
            self.demote_or_drop(victim, now);
        }
    }

    fn place_in_ram(&mut self, h: Hash, bytes: Bytes) {
        self.set_remove(h);
        let e = self.map.get_mut(&h).expect("entry");
        self.ram_bytes += bytes.len();
        self.accts.entry(e.account).or_default().bytes += bytes.len();
        e.ram = Some(bytes);
        self.set_insert(h);
    }

    fn set_expiry(&mut self, h: Hash, new: u64) {
        if let Some(e) = self.map.get_mut(&h)
            && e.expires != new
        {
            self.expiry.remove(&(e.expires, h));
            e.expires = new;
            self.expiry.insert((new, h));
            e.dirty = true;
            self.snapshot_dirty = true;
        }
    }

    fn record_hit(&mut self, h: Hash, ctx: Context, now: u64) {
        let Some(e) = self.map.get_mut(&h) else { return };
        let idle = now.saturating_sub(e.last_used);
        e.hits = decay(e.hits, idle, self.cfg.half_life_secs * 1000) + 1.0;
        e.total_hits += 1;
        e.last_used = now;
        e.ctx = ctx;
        e.dirty = true;
        let new_exp = extended_expiry(&self.cfg, e.created, now, e.expires, e.hits);
        self.snapshot_dirty = true;
        self.set_expiry(h, new_exp);
        self.rescore(h, now);
    }

    fn is_stale(&self, e: &Entry, now: u64) -> bool {
        now >= e.expires || now >= e.created + self.cfg.max_age_secs * 1000
    }

    fn probe(&mut self, h: Hash, ctx: Context, now: u64) -> Probe {
        let Some(e) = self.map.get(&h) else {
            self.counters.misses += 1;
            return Probe::Miss;
        };
        if self.cfg.ttl_base_secs == 0 || self.is_stale(e, now) {
            // At max age the egress layer cannot revalidate (the image fetch keeps
            // no upstream validators), so the entry is refetched.
            self.remove_entry(h);
            self.counters.misses += 1;
            return Probe::Miss;
        }
        let tier = e.tier();
        if let Some(bytes) = e.ram.clone().or_else(|| e.pending.clone()) {
            let etag = e.etag.clone();
            self.record_hit(h, ctx, now);
            if tier == Tier::Ram {
                self.counters.ram_hits += 1;
            } else {
                self.counters.disk_hits += 1;
                self.maybe_promote(h, &bytes, now);
            }
            return Probe::Hit { etag, png: bytes, tier };
        }
        Probe::NeedDisk { name: file_name(h), size: e.size }
    }

    fn maybe_promote(&mut self, h: Hash, bytes: &Bytes, now: u64) {
        let Some(e) = self.map.get(&h) else { return };
        if e.ram.is_some()
            || e.size > self.cfg.ram_entry_bytes
            || place((e.skey as f64) / 1_000_000.0, Tier::Disk, &self.cfg) != Tier::Ram
        {
            return;
        }
        let acct = e.account;
        self.make_room_ram(bytes.len(), Some(acct), now, REQUEST_PATH_EVICTIONS);
        if !self.map.contains_key(&h) {
            return;
        }
        if self.ram_used() + bytes.len() + ENTRY_OVERHEAD <= self.cfg.ram_bytes
            && self.ram_set.len() < self.cfg.ram_entries
        {
            self.place_in_ram(h, bytes.clone());
            self.counters.promotions += 1;
            self.promoted_at.push_back(now);
            while self.promoted_at.len() > 1024 {
                self.promoted_at.pop_front();
            }
        }
    }

    fn finish_disk(&mut self, h: Hash, payload: Option<Bytes>, ctx: Context, now: u64) -> Option<(String, Bytes)> {
        let Some(bytes) = payload else {
            // Drift: the index said disk, the file was missing or unreadable.
            if self.map.contains_key(&h) {
                self.counters.drift_recoveries += 1;
                self.remove_entry(h);
            }
            self.counters.misses += 1;
            return None;
        };
        let e = self.map.get(&h)?;
        let etag = e.etag.clone();
        self.counters.disk_hits += 1;
        self.record_hit(h, ctx, now);
        self.maybe_promote(h, &bytes, now);
        Some((etag, bytes))
    }

    fn put(&mut self, h: Hash, account: u64, ctx: Context, etag: String, png: Bytes, now: u64) {
        let size = png.len();
        let cfg_ok = self.cfg.ttl_base_secs > 0;
        if self.map.contains_key(&h) {
            self.remove_entry(h);
        }
        if !cfg_ok || size == 0 {
            return;
        }
        let to_ram = size <= self.cfg.ram_entry_bytes && size + ENTRY_OVERHEAD <= self.cfg.ram_bytes;
        let to_disk_direct = !to_ram
            && size <= self.cfg.disk.entry_bytes
            && self.disk_usable(now)
            && self.pending_bytes + size <= self.cfg.disk.write_queue_bytes;
        if !to_ram && !to_disk_direct {
            return;
        }
        let created = now;
        let mut entry = Entry {
            size,
            etag,
            ram: None,
            pending: None,
            on_disk: false,
            created,
            expires: initial_expiry(&self.cfg, created),
            last_used: now,
            hits: 0.0,
            total_hits: 0,
            ctx,
            account,
            skey: 0,
            kt: 0,
            stamp: 0,
            dirty: true,
        };
        entry.skey = self.rescored_key(&entry, now);
        entry.kt = entry.last_used;
        if to_ram {
            self.make_room_ram(size, Some(account), now, REQUEST_PATH_EVICTIONS);
            if self.ram_used() + size + ENTRY_OVERHEAD > self.cfg.ram_bytes
                || self.ram_set.len() >= self.cfg.ram_entries
            {
                return;
            }
            self.expiry.insert((entry.expires, h));
            self.map.insert(h, entry);
            self.place_in_ram(h, png);
        } else {
            let cost = disk_cost(size, &entry.etag) as u64;
            if !self.make_room_disk(cost, now, None) {
                return;
            }
            let g = self.next_stamp();
            entry.stamp = g;
            self.pending_bytes += size;
            self.disk_bytes += cost;
            entry.pending = Some(png);
            self.expiry.insert((entry.expires, h));
            self.map.insert(h, entry);
            self.set_insert(h);
            self.jobs.push_back(Job::Write { hash: h, stamp: g });
        }
        self.ring.push_back(h);
        self.snapshot_dirty = true;
    }

    /// Enforce current limits (a shrink in the admin console), bounded per call.
    fn trim(&mut self, now: u64, max_steps: usize) {
        self.make_room_ram(0, None, now, max_steps);
        let mut steps = 0;
        while (self.disk_bytes > self.disk_budget || self.disk_set.len() > self.cfg.disk.entries)
            && steps < max_steps
        {
            steps += 1;
            let Some(&(_, _, victim)) = self.disk_set.iter().next() else { break };
            if self.map.get(&victim).is_some_and(|e| e.ram.is_some()) {
                self.disk_only_drop(victim);
            } else {
                self.remove_entry(victim);
                self.counters.drops += 1;
            }
        }
    }

    fn disk_only_drop(&mut self, h: Hash) {
        self.set_remove(h);
        let Some(e) = self.map.get_mut(&h) else { return };
        let cost = disk_cost(e.size, &e.etag) as u64;
        let was = e.on_disk;
        if let Some(p) = e.pending.take() {
            self.pending_bytes = self.pending_bytes.saturating_sub(p.len());
        }
        e.on_disk = false;
        e.stamp = 0;
        self.disk_bytes = self.disk_bytes.saturating_sub(cost);
        self.set_insert(h);
        if was {
            self.queue_remove_file(h);
        }
    }

    /// Disk disabled or detached: forget disk copies (files stay on disk).
    fn forget_disk(&mut self) {
        let hashes: Vec<Hash> = self.disk_set.iter().map(|(_, _, h)| *h).collect();
        for h in hashes {
            let ram = self.map.get(&h).is_some_and(|e| e.ram.is_some());
            if ram {
                self.set_remove(h);
                let e = self.map.get_mut(&h).expect("entry");
                if let Some(p) = e.pending.take() {
                    self.pending_bytes = self.pending_bytes.saturating_sub(p.len());
                }
                e.on_disk = false;
                e.stamp = 0;
                self.set_insert(h);
            } else {
                // No file deletion job: the files are left for a later attach.
                self.set_remove(h);
                if let Some(e) = self.map.get_mut(&h) {
                    e.on_disk = false;
                }
                self.remove_entry(h);
            }
        }
        self.disk_bytes = 0;
        self.jobs.retain(|j| matches!(j, Job::Remove(_)));
    }
}

// ── snapshot ───────────────────────────────────────────────────────────────────

fn snapshot_text(s: &State) -> String {
    let mut out = String::from("MWSNAP1\n");
    for (h, e) in s.map.iter().filter(|(_, e)| e.on_disk) {
        out.push_str(&format!(
            "{h:032x} {} {} {} {} {} {}\n",
            e.size,
            e.created,
            e.expires,
            e.last_used,
            (e.hits * 1000.0) as u64,
            e.etag
        ));
    }
    out
}

struct Snap {
    size: usize,
    created: u64,
    expires: u64,
    last_used: u64,
    hits: f64,
    etag: String,
}

fn parse_snapshot(text: &str) -> HashMap<Hash, Snap> {
    let mut out = HashMap::new();
    let mut lines = text.lines();
    if lines.next() != Some("MWSNAP1") {
        return out;
    }
    for line in lines {
        let f: Vec<&str> = line.split(' ').collect();
        if f.len() != 7 {
            continue;
        }
        let (Ok(h), Ok(size), Ok(created), Ok(expires), Ok(last), Ok(hits)) = (
            u128::from_str_radix(f[0], 16),
            f[1].parse(),
            f[2].parse(),
            f[3].parse(),
            f[4].parse(),
            f[5].parse::<u64>(),
        ) else {
            continue;
        };
        out.insert(h, Snap { size, created, expires, last_used: last, hits: hits as f64 / 1000.0, etag: f[6].to_string() });
    }
    out
}

/// One file found by the startup scan, with the metadata to index it.
pub(crate) struct Scanned {
    hash: Hash,
    size: usize,
    etag: String,
    created: u64,
    expires: u64,
    last_used: u64,
    hits: f64,
}

/// Bounded startup scan, IO only (no lock held). Deletes stray temp files, files
/// past max age by modification time, and malformed files; trusts the snapshot
/// only for files whose size still matches it.
pub(crate) fn scan(io: &dyn DiskIo, cfg: &TierConfig, now: u64) -> io::Result<Vec<Scanned>> {
    io.ensure_dir()?;
    let limit = cfg.disk.entries.saturating_mul(2).max(64);
    let files = io.list(limit)?;
    let snap = io
        .read(SNAPSHOT)
        .ok()
        .and_then(|b| String::from_utf8(b).ok())
        .map(|t| parse_snapshot(&t))
        .unwrap_or_default();
    let max_age = cfg.max_age_secs * 1000;
    let mut out = Vec::new();
    for f in files {
        if is_our_temp(&f.name) {
            let _ = io.remove(&f.name);
            continue;
        }
        let Some(hash) = parse_name(&f.name) else { continue };
        if now.saturating_sub(f.modified_ms) >= max_age && f.modified_ms != 0 {
            let _ = io.remove(&f.name);
            continue;
        }
        let from_snap = snap
            .get(&hash)
            .filter(|s| (f.size as usize) > s.size && (f.size as usize) <= s.size + HEADER_MAX);
        let scanned = match from_snap {
            Some(s) => Some(Scanned {
                hash,
                size: s.size,
                etag: s.etag.clone(),
                created: s.created,
                expires: s.expires,
                last_used: s.last_used,
                hits: s.hits,
            }),
            None => io
                .read_head(&f.name, HEADER_MAX)
                .ok()
                .and_then(|b| parse_head(&b))
                .filter(|hd| hd.header_len as u64 + hd.len as u64 == f.size)
                .map(|hd| Scanned {
                    hash,
                    size: hd.len,
                    etag: hd.etag,
                    created: hd.created,
                    expires: hd.expires,
                    last_used: hd.created,
                    hits: hd.hits,
                }),
        };
        match scanned {
            Some(s) if s.expires > now && now < s.created + max_age => out.push(s),
            _ => {
                let _ = io.remove(&f.name);
            }
        }
    }
    Ok(out)
}

// ── the cache ──────────────────────────────────────────────────────────────────

pub(crate) struct ImageTier {
    state: Mutex<State>,
    notify: tokio::sync::Notify,
    worker: AtomicBool,
    /// Builds the real backend for a configured path; replaced in tests.
    make_io: Box<dyn Fn(PathBuf) -> Arc<dyn DiskIo> + Send + Sync>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct TierStats {
    pub ram_bytes: usize,
    pub ram_used: usize,
    pub ram_entries: usize,
    pub ram_budget: usize,
    pub ram_hot: usize,
    pub ram_cold: usize,
    pub disk_enabled: bool,
    pub disk_active: bool,
    pub disk_bytes: u64,
    pub disk_entries: usize,
    pub disk_budget: u64,
    pub disk_hot: usize,
    pub disk_cold: usize,
    pub index_entries: usize,
    pub pending_bytes: usize,
    pub promotions_per_min: usize,
    pub counters: Counters,
    pub last_error: Option<String>,
    pub path: Option<String>,
}

impl ImageTier {
    pub(crate) fn new(cfg: TierConfig) -> Self {
        Self::with_io_factory(cfg, Box::new(|p| Arc::new(FsIo::new(p))))
    }

    pub(crate) fn with_io_factory(
        cfg: TierConfig,
        make_io: Box<dyn Fn(PathBuf) -> Arc<dyn DiskIo> + Send + Sync>,
    ) -> Self {
        Self {
            state: Mutex::new(State::new(cfg)),
            notify: tokio::sync::Notify::new(),
            worker: AtomicBool::new(false),
            make_io,
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// Apply the current policy. Cheap when nothing changed. Returns whether the
    /// disk backend must be (re)attached; call `attach` when it does.
    pub(crate) fn configure(&self, cfg: &TierConfig, now: u64) -> bool {
        let mut s = self.lock();
        if s.cfg == *cfg {
            return false;
        }
        let disk_changed = s.cfg.disk.enabled != cfg.disk.enabled || s.cfg.disk.path != cfg.disk.path;
        s.cfg = cfg.clone();
        let mut attach = false;
        if disk_changed {
            s.forget_disk();
            s.io = None;
            s.degraded_until = 0;
            s.last_error = None;
            attach = cfg.disk.enabled;
            if cfg.disk.enabled && cfg.disk.path.is_none() {
                s.last_error = Some("no absolute cache directory configured".into());
                attach = false;
            }
        }
        s.disk_budget = disk_budget(&cfg.disk, s.last_free, s.disk_bytes);
        s.trim(now, REQUEST_PATH_EVICTIONS);
        attach
    }

    /// Open the configured directory, scan it and adopt its entries. IO runs
    /// outside the lock. A failure leaves the cache RAM-only and records why.
    pub(crate) fn attach(&self, now: u64) {
        let (path, cfg) = {
            let s = self.lock();
            let Some(p) = s.cfg.disk.path.clone().filter(|_| s.cfg.disk.enabled) else {
                return;
            };
            (p, s.cfg.clone())
        };
        let io = (self.make_io)(path);
        let scanned = scan(io.as_ref(), &cfg, now);
        let free = io.free_bytes();
        let mut s = self.lock();
        if s.cfg.disk != cfg.disk {
            return; // reconfigured while scanning; the new configure() attaches again
        }
        match scanned {
            Err(e) => {
                s.counters.disk_errors += 1;
                s.last_error = Some(format!("disk tier unavailable: {e}"));
                s.degraded_until = now + DEGRADED_BACKOFF_MS;
                s.io = Some(io);
                s.disk_budget = 0;
            }
            Ok(list) => {
                s.io = Some(io);
                s.last_error = None;
                s.last_free = free;
                s.disk_budget = disk_budget(&cfg.disk, free, 0);
                for f in list {
                    if s.map.contains_key(&f.hash) {
                        continue;
                    }
                    let cost = disk_cost(f.size, &f.etag) as u64;
                    if !s.make_room_disk(cost, now, None) {
                        s.jobs.push_back(Job::Remove(file_name(f.hash)));
                        continue;
                    }
                    let mut e = Entry {
                        size: f.size,
                        etag: f.etag,
                        ram: None,
                        pending: None,
                        on_disk: true,
                        created: f.created,
                        expires: f.expires,
                        last_used: f.last_used,
                        hits: f.hits,
                        total_hits: 0,
                        ctx: Context::Background,
                        account: 0,
                        skey: 0,
                        kt: 0,
                        stamp: 0,
                        dirty: false,
                    };
                    e.skey = s.rescored_key(&e, now);
                    e.kt = e.last_used;
                    s.disk_bytes += cost;
                    s.expiry.insert((e.expires, f.hash));
                    s.map.insert(f.hash, e);
                    s.set_insert(f.hash);
                    s.ring.push_back(f.hash);
                }
            }
        }
        drop(s);
        self.notify.notify_one();
    }

    /// RAM-then-disk lookup. A miss costs one hash and one map probe, no IO. The
    /// caller has already authorised the request.
    pub(crate) async fn lookup(
        self: &Arc<Self>,
        route: &str,
        url: &str,
        ctx: Context,
    ) -> Option<(String, Bytes)> {
        // Only an authorised request gets here: it is proxy work worth timing.
        super::latency::engage();
        let found = super::latency::timed(
            super::latency::Phase::Cache,
            self.lookup_tiers(route, url, ctx),
        )
        .await;
        if let Some((_, _, disk)) = &found {
            super::latency::note_hit(*disk);
        }
        found.map(|(etag, png, _)| (etag, png))
    }

    /// The lookup itself; the flag says the hit came from disk.
    async fn lookup_tiers(
        self: &Arc<Self>,
        route: &str,
        url: &str,
        ctx: Context,
    ) -> Option<(String, Bytes, bool)> {
        let h = key_hash(route, url);
        let now = now_ms();
        let (probe, io) = {
            let mut s = self.lock();
            let p = s.probe(h, ctx, now);
            let io = matches!(p, Probe::NeedDisk { .. }).then(|| s.io.clone()).flatten();
            (p, io)
        };
        match probe {
            Probe::Miss => None,
            Probe::Hit { etag, png, .. } => Some((etag, png, false)),
            Probe::NeedDisk { name, size } => {
                let Some(io) = io else {
                    // Disk detached since the index entry was made.
                    self.lock().finish_disk(h, None, ctx, now);
                    return None;
                };
                let read = tokio::task::spawn_blocking(move || {
                    io.read(&name)
                        .ok()
                        .and_then(|b| decode_file(&b, size, |p| super::etag_for(p)))
                })
                .await
                .ok()
                .flatten();
                let result = self.lock().finish_disk(h, read, ctx, now_ms());
                self.notify.notify_one();
                result.map(|(etag, png)| (etag, png, true))
            }
        }
    }

    pub(crate) fn store(&self, route: &str, url: &str, account: &str, ctx: Context, etag: String, png: Vec<u8>) {
        let queued = {
            let mut s = self.lock();
            s.put(key_hash(route, url), account_hash(account), ctx, etag, Bytes::from(png), now_ms());
            !s.jobs.is_empty()
        };
        if queued {
            self.notify.notify_one();
        }
    }

    /// Run queued disk jobs. IO happens with the lock released.
    pub(crate) fn run_jobs(&self, now: u64) {
        loop {
            let (job, io) = {
                let mut s = self.lock();
                let Some(job) = s.jobs.pop_front() else { return };
                (job, s.io.clone())
            };
            let Some(io) = io else { continue };
            match job {
                Job::Remove(name) => {
                    let _ = io.remove(&name);
                }
                Job::Write { hash, stamp } => {
                    let payload = {
                        let s = self.lock();
                        s.map.get(&hash).filter(|e| e.stamp == stamp).and_then(|e| {
                            e.pending.clone().map(|p| (p, e.etag.clone(), e.created, e.expires, e.hits))
                        })
                    };
                    let Some((bytes, etag, created, expires, hits)) = payload else {
                        continue; // entry gone or replaced; nothing was written
                    };
                    let data = encode_file(&bytes, &etag, created, expires, hits);
                    let result = io.write_atomic(&file_name(hash), &data);
                    let mut s = self.lock();
                    match result {
                        Ok(()) => {
                            let alive = s.map.get(&hash).is_some_and(|e| e.stamp == stamp && e.pending.is_some());
                            if alive {
                                let e = s.map.get_mut(&hash).expect("entry");
                                let p = e.pending.take();
                                e.on_disk = true;
                                e.dirty = false;
                                if let Some(p) = p {
                                    s.pending_bytes = s.pending_bytes.saturating_sub(p.len());
                                }
                                s.snapshot_dirty = true;
                            } else {
                                drop(s);
                                let _ = io.remove(&file_name(hash));
                            }
                        }
                        Err(e) => {
                            // Full, read-only or missing: degrade to RAM-only for a
                            // while. Nothing here ever reaches a request.
                            s.counters.disk_errors += 1;
                            s.last_error = Some(format!("disk write failed: {e}"));
                            s.degraded_until = now + DEGRADED_BACKOFF_MS;
                            let still = s.map.get(&hash).is_some_and(|e| e.stamp == stamp && e.pending.is_some());
                            if still {
                                let has_ram = s.map.get(&hash).is_some_and(|e| e.ram.is_some());
                                if has_ram {
                                    s.disk_only_drop(hash);
                                } else {
                                    s.remove_entry(hash);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Periodic maintenance, bounded by `sweep_batch`: expire, rescore a slice of
    /// the ring, proactively demote cold RAM entries under pressure, enforce
    /// limits, refresh the disk budget, and write the snapshot.
    pub(crate) fn sweep(&self, now: u64) {
        let mut snapshot: Option<(Arc<dyn DiskIo>, String)> = None;
        let probe_dir = {
            let mut s = self.lock();
            let batch = s.cfg.sweep_batch.max(1);
            for _ in 0..batch {
                let Some(&(exp, h)) = s.expiry.iter().next() else { break };
                if exp > now {
                    break;
                }
                s.remove_entry(h);
                s.counters.drops += 1;
            }
            for _ in 0..batch.min(s.ring.len()) {
                let Some(h) = s.ring.pop_front() else { break };
                if !s.map.contains_key(&h) {
                    continue;
                }
                s.rescore(h, now);
                s.ring.push_back(h);
                let pressured = s.ram_used() * 5 > s.cfg.ram_bytes * 4;
                let cold = s.map.get(&h).is_some_and(|e| {
                    e.ram.is_some()
                        && place((e.skey as f64) / 1_000_000.0, Tier::Ram, &s.cfg) == Tier::Disk
                });
                if pressured && cold {
                    s.demote_or_drop(h, now);
                }
            }
            while s.ring.len() > s.map.len() * 2 + 16 {
                s.ring.pop_front();
            }
            while s.promoted_at.front().is_some_and(|t| now.saturating_sub(*t) > 60_000) {
                s.promoted_at.pop_front();
            }
            // Over-share accounts after a config change.
            if s.accts.len() > 1 {
                let cap = account_cap(s.cfg.ram_bytes, s.cfg.account_share_pct, s.accts.len());
                let over: Vec<u64> = s.accts.iter().filter(|(_, a)| a.bytes > cap).map(|(k, _)| *k).take(8).collect();
                for acct in over {
                    s.make_room_ram(0, Some(acct), now, batch);
                }
            }
            s.trim(now, batch);
            let mut probe_dir = false;
            if s.cfg.disk.enabled && s.io.is_some() {
                if now >= s.degraded_until && s.degraded_until != 0 {
                    probe_dir = true;
                }
                if now.saturating_sub(s.last_free_check) >= 60_000 {
                    s.last_free_check = now;
                    if let Some(io) = s.io.clone() {
                        let free = io.free_bytes();
                        s.last_free = free;
                        let held = s.disk_bytes;
                        s.disk_budget = disk_budget(&s.cfg.disk, free, held);
                    }
                }
            }
            let due = s.cfg.snapshot_secs > 0
                && now.saturating_sub(s.last_snapshot) >= s.cfg.snapshot_secs * 1000;
            if due && s.snapshot_dirty && s.disk_usable(now) {
                snapshot = s.io.clone().map(|io| (io, snapshot_text(&s)));
                s.last_snapshot = now;
                s.snapshot_dirty = false;
            }
            probe_dir.then(|| s.io.clone()).flatten()
        };
        if let Some(io) = probe_dir {
            let ok = io.ensure_dir();
            let mut s = self.lock();
            match ok {
                Ok(()) => {
                    s.degraded_until = 0;
                    s.last_error = None;
                }
                Err(e) => {
                    s.last_error = Some(format!("disk tier unavailable: {e}"));
                    s.degraded_until = now + DEGRADED_BACKOFF_MS;
                }
            }
        }
        if let Some((io, text)) = snapshot
            && let Err(e) = io.write_atomic(SNAPSHOT, text.as_bytes())
        {
            let mut s = self.lock();
            s.counters.disk_errors += 1;
            s.snapshot_dirty = true;
            s.last_error = Some(format!("snapshot write failed: {e}"));
        }
        self.run_jobs(now);
    }

    pub(crate) fn stats(&self, now: u64) -> TierStats {
        let s = self.lock();
        let demote = key_of(f64::from(s.cfg.demote_milli) / 1000.0);
        let promote = key_of(f64::from(s.cfg.demote_milli) / 1000.0 * (1.0 + f64::from(s.cfg.hysteresis_pct) / 100.0));
        let hot = |set: &BTreeSet<(i64, u64, Hash)>| set.range((promote, 0, 0)..).count();
        let cold = |set: &BTreeSet<(i64, u64, Hash)>| set.range(..(demote, 0, 0)).count();
        TierStats {
            ram_bytes: s.ram_bytes,
            ram_used: s.ram_used(),
            ram_entries: s.ram_set.len(),
            ram_budget: s.cfg.ram_bytes,
            ram_hot: hot(&s.ram_set),
            ram_cold: cold(&s.ram_set),
            disk_enabled: s.cfg.disk.enabled,
            disk_active: s.disk_usable(now),
            disk_bytes: s.disk_bytes,
            disk_entries: s.disk_set.len(),
            disk_budget: s.disk_budget,
            disk_hot: hot(&s.disk_set),
            disk_cold: cold(&s.disk_set),
            index_entries: s.map.len(),
            pending_bytes: s.pending_bytes,
            promotions_per_min: s.promoted_at.iter().filter(|t| now.saturating_sub(**t) <= 60_000).count(),
            counters: s.counters.clone(),
            last_error: s.last_error.clone(),
            path: s.cfg.disk.path.as_ref().map(|p| p.display().to_string()),
        }
    }

    pub(crate) fn ensure_worker(self: &Arc<Self>) {
        if self.worker.swap(true, Ordering::AcqRel) {
            return;
        }
        let Ok(handle) = tokio::runtime::Handle::try_current() else {
            self.worker.store(false, Ordering::Release);
            return;
        };
        let me = Arc::downgrade(self);
        handle.spawn(async move {
            let mut last_sweep = 0u64;
            loop {
                let Some(tier) = me.upgrade() else { return };
                let wait = std::time::Duration::from_secs(tier.lock().cfg.sweep_secs.max(1));
                let _ = tokio::time::timeout(wait, tier.notify.notified()).await;
                let t2 = tier.clone();
                let due = now_ms().saturating_sub(last_sweep) >= wait.as_millis() as u64;
                if due {
                    last_sweep = now_ms();
                }
                let _ = tokio::task::spawn_blocking(move || {
                    if due {
                        t2.sweep(now_ms());
                    } else {
                        t2.run_jobs(now_ms());
                    }
                })
                .await;
            }
        });
    }

    // ── test and diagnostics surface ──────────────────────────────────────────

    #[cfg(test)]
    fn probe(&self, route: &str, url: &str, ctx: Context, now: u64) -> Probe {
        self.lock().probe(key_hash(route, url), ctx, now)
    }
    #[cfg(test)]
    fn put_at(&self, route: &str, url: &str, account: &str, ctx: Context, etag: &str, png: Vec<u8>, now: u64) {
        self.lock().put(key_hash(route, url), account_hash(account), ctx, etag.into(), Bytes::from(png), now);
    }
    #[cfg(test)]
    fn finish(&self, route: &str, url: &str, io_bytes: Option<Vec<u8>>, now: u64) -> Option<(String, Bytes)> {
        let h = key_hash(route, url);
        self.lock().finish_disk(h, io_bytes.map(Bytes::from), Context::Open, now)
    }
}

/// Serve a hit through the global cache, shared by both handlers.
pub(crate) fn global() -> &'static Arc<ImageTier> {
    static TIER: std::sync::OnceLock<Arc<ImageTier>> = std::sync::OnceLock::new();
    TIER.get_or_init(|| Arc::new(ImageTier::new(TierConfig::default())))
}

/// Apply the request's policy: attach or detach the disk backend when its
/// configuration changed, and make sure the maintenance worker runs.
pub(crate) async fn prepare(cfg: &TierConfig) -> &'static Arc<ImageTier> {
    let tier = global();
    tier.ensure_worker();
    if tier.configure(cfg, now_ms()) {
        let t = tier.clone();
        let _ = tokio::task::spawn_blocking(move || t.attach(now_ms())).await;
    }
    tier
}

/// The admin readout: configured against effective values, and per-tier size,
/// entries, hit rate, hot and cold counts and promotions per minute. Hot and cold
/// are entries at or above the promote threshold and below the demote threshold.
pub(crate) fn readout(cfg: &TierConfig, st: &TierStats) -> serde_json::Value {
    let c = &st.counters;
    let rate = |hits: u64, total: u64| (total > 0).then(|| hits * 100 / total);
    let lookups = c.ram_hits + c.disk_hits + c.misses;
    serde_json::json!({
        "configured": {
            "ramBytes": cfg.ram_bytes, "ramEntries": cfg.ram_entries, "ramEntryBytes": cfg.ram_entry_bytes,
            "ttlBaseSecs": cfg.ttl_base_secs, "ttlStepSecs": cfg.ttl_step_secs, "ttlMaxSecs": cfg.ttl_max_secs,
            "maxAgeSecs": cfg.max_age_secs, "halfLifeSecs": cfg.half_life_secs,
            "demoteMilli": cfg.demote_milli, "hysteresisPercent": cfg.hysteresis_pct,
            "openPercent": cfg.open_pct, "recentPercent": cfg.recent_pct,
            "accountSharePercent": cfg.account_share_pct, "sweepSecs": cfg.sweep_secs,
            "snapshotSecs": cfg.snapshot_secs, "sweepBatch": cfg.sweep_batch,
            "disk": {
                "enabled": cfg.disk.enabled, "mode": if cfg.disk.auto { "auto" } else { "manual" },
                "path": cfg.disk.path.as_ref().map(|p| p.display().to_string()),
                "manualBytes": cfg.disk.manual_bytes, "floorBytes": cfg.disk.floor_bytes,
                "ceilingBytes": cfg.disk.ceiling_bytes, "entries": cfg.disk.entries,
                "entryBytes": cfg.disk.entry_bytes, "minHits": cfg.disk.min_hits,
                "writeQueueBytes": cfg.disk.write_queue_bytes,
            },
        },
        "ram": {
            "bytes": st.ram_bytes, "usedBytes": st.ram_used, "budgetBytes": st.ram_budget,
            "entries": st.ram_entries, "hot": st.ram_hot, "cold": st.ram_cold,
            "hits": c.ram_hits, "hitPercent": rate(c.ram_hits, lookups),
        },
        "disk": {
            "enabled": st.disk_enabled, "active": st.disk_active, "path": st.path,
            "bytes": st.disk_bytes, "budgetBytes": st.disk_budget, "entries": st.disk_entries,
            "hot": st.disk_hot, "cold": st.disk_cold, "pendingBytes": st.pending_bytes,
            "hits": c.disk_hits, "hitPercent": rate(c.disk_hits, lookups),
            "errors": c.disk_errors, "lastError": st.last_error,
        },
        "index": { "entries": st.index_entries, "driftRecoveries": c.drift_recoveries },
        "promotionsPerMin": st.promotions_per_min,
        "promotions": c.promotions, "demotions": c.demotions, "drops": c.drops,
        "misses": c.misses, "hitPercent": rate(c.ram_hits + c.disk_hits, lookups),
    })
}

#[cfg(test)]
#[path = "remote_image_tier_tests.rs"]
mod tests;
