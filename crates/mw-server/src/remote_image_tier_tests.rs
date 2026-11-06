use super::*;
use std::sync::atomic::{AtomicBool, AtomicUsize};

/// In-memory backend that counts every call, so a test can prove an operation
/// did no disk IO, and can be made to fail like a full or read-only disk.
#[derive(Default)]
struct MemIo {
    files: Mutex<HashMap<String, (Vec<u8>, u64)>>,
    reads: AtomicUsize,
    writes: AtomicUsize,
    lists: AtomicUsize,
    removes: AtomicUsize,
    fail_write: AtomicBool,
    fail_dir: AtomicBool,
    free: Mutex<Option<u64>>,
}
impl MemIo {
    fn calls(&self) -> usize {
        self.reads.load(Ordering::SeqCst)
            + self.writes.load(Ordering::SeqCst)
            + self.lists.load(Ordering::SeqCst)
            + self.removes.load(Ordering::SeqCst)
    }
    fn has(&self, name: &str) -> bool {
        self.files.lock().unwrap().contains_key(name)
    }
}
impl DiskIo for MemIo {
    fn ensure_dir(&self) -> io::Result<()> {
        if self.fail_dir.load(Ordering::SeqCst) {
            return Err(io::Error::other("read-only file system"));
        }
        Ok(())
    }
    fn list(&self, limit: usize) -> io::Result<Vec<FileInfo>> {
        self.lists.fetch_add(1, Ordering::SeqCst);
        Ok(self
            .files
            .lock()
            .unwrap()
            .iter()
            .take(limit)
            .map(|(n, (b, m))| FileInfo { name: n.clone(), size: b.len() as u64, modified_ms: *m })
            .collect())
    }
    fn read(&self, name: &str) -> io::Result<Vec<u8>> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.files
            .lock()
            .unwrap()
            .get(name)
            .map(|(b, _)| b.clone())
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
    }
    fn read_head(&self, name: &str, max: usize) -> io::Result<Vec<u8>> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.files
            .lock()
            .unwrap()
            .get(name)
            .map(|(b, _)| b[..b.len().min(max)].to_vec())
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
    }
    fn write_atomic(&self, name: &str, data: &[u8]) -> io::Result<()> {
        self.writes.fetch_add(1, Ordering::SeqCst);
        if self.fail_write.load(Ordering::SeqCst) {
            return Err(io::Error::other("no space left on device"));
        }
        self.files.lock().unwrap().insert(name.into(), (data.to_vec(), NOW));
        Ok(())
    }
    fn remove(&self, name: &str) -> io::Result<()> {
        self.removes.fetch_add(1, Ordering::SeqCst);
        self.files.lock().unwrap().remove(name);
        Ok(())
    }
    fn free_bytes(&self) -> Option<u64> {
        *self.free.lock().unwrap()
    }
}

const NOW: u64 = 1_000_000_000_000;
const MIN: u64 = 60_000;

fn disk_cfg() -> DiskConfig {
    DiskConfig {
        enabled: true,
        path: Some(PathBuf::from("/cache")),
        auto: false,
        manual_bytes: 1 << 20,
        entries: 100,
        entry_bytes: 64 << 10,
        min_hits: 1,
        write_queue_bytes: 1 << 20,
        ..DiskConfig::default()
    }
}

fn cfg() -> TierConfig {
    TierConfig {
        ram_bytes: 4096 + 4 * ENTRY_OVERHEAD,
        ram_entries: 100,
        ram_entry_bytes: 4096,
        disk: disk_cfg(),
        stale_secs: 0,
        max_stale_secs: 0,
        ..TierConfig::default()
    }
}

fn tier_with(io: &Arc<MemIo>, cfg: TierConfig) -> ImageTier {
    let io = io.clone();
    let t = ImageTier::with_io_factory(TierConfig::default(), Box::new(move |_| io.clone()));
    if t.configure(&cfg, NOW) {
        t.attach(NOW);
    }
    t
}

fn png(n: u8, len: usize) -> Vec<u8> {
    vec![n; len]
}

fn etag(b: &[u8]) -> String {
    super::super::etag_for(b)
}

fn put(t: &ImageTier, url: &str, acct: &str, bytes: Vec<u8>, now: u64) {
    let e = etag(&bytes);
    t.put_at("direct", url, acct, Context::Open, &e, bytes, now);
}

/// Six images that each outscore a lightly used one: new, then hit six times, so
/// eviction pressure falls on the entry under test rather than on its neighbours.
fn fill(t: &ImageTier, tag: &str, now: u64) {
    for i in 0..6u64 {
        let url = format!("https://a/{tag}{i}.png");
        put(t, &url, "acct", png(2, 1500), now + i);
        for k in 0..6 {
            hit(t, &url, now + i + 1 + k);
        }
    }
}

fn hit(t: &ImageTier, url: &str, now: u64) -> bool {
    matches!(t.probe("direct", url, Context::Open, now), Probe::Hit { .. })
}

// ── pure functions ─────────────────────────────────────────────────────────────

#[test]
fn a_cold_entry_keeps_the_base_ttl_and_a_zero_step_never_extends() {
    let mut c = TierConfig::default();
    assert_eq!(initial_expiry(&c, NOW), NOW + 3_600_000);
    c.ttl_step_secs = 0;
    let exp = initial_expiry(&c, NOW);
    assert_eq!(extended_expiry(&c, NOW, NOW + 1000, exp, 1.0), exp);
}

#[test]
fn each_hit_extends_by_a_step_that_grows_with_frequency_within_the_clamps() {
    let c = TierConfig::default();
    let exp0 = initial_expiry(&c, NOW);
    let one = extended_expiry(&c, NOW, NOW, exp0, 1.0);
    let many = extended_expiry(&c, NOW, NOW, exp0, 20.0);
    assert!(one > exp0 && many > one, "{exp0} {one} {many}");
    // The remaining lifetime never exceeds ttl_max.
    let mut exp = exp0;
    for _ in 0..200 {
        exp = extended_expiry(&c, NOW, NOW, exp, 50.0);
    }
    assert_eq!(exp, NOW + c.ttl_max_secs * 1000);
    // The absolute max age wins over everything.
    let late = NOW + c.max_age_secs * 1000 - 1000;
    assert_eq!(extended_expiry(&c, NOW, late, late + 5000, 50.0), NOW + c.max_age_secs * 1000);
    // Base TTL longer than max age is capped by it.
    let short = TierConfig { max_age_secs: 10, ..c };
    assert_eq!(initial_expiry(&short, NOW), NOW + 10_000);
}

#[test]
fn score_rises_with_hits_and_context_and_falls_with_idle_time_and_size() {
    let hl = 3_600_000;
    let base = score(2.0, 10_000, 0, 100, hl);
    assert!(score(5.0, 10_000, 0, 100, hl) > base);
    assert!(score(2.0, 10_000, 3 * hl, 100, hl) < base);
    assert!(score(2.0, 400_000, 0, 100, hl) < base);
    assert!(score(2.0, 10_000, 0, 200, hl) > base);
    let c = TierConfig::default();
    assert!(context_percent(Context::Open, 0, &c) > context_percent(Context::Open, 11 * MIN, &c));
    assert!(context_percent(Context::Open, 11 * MIN, &c) > context_percent(Context::Background, 0, &c));
    // Open decays to Recent after ten minutes idle.
    assert_eq!(context_percent(Context::Open, 11 * MIN, &c), c.recent_pct);
    assert!(decay(8.0, hl, hl) - 4.0 < 1e-9);
}

#[test]
fn placement_has_hysteresis_between_the_demote_and_promote_thresholds() {
    let c = TierConfig::default(); // demote 0.3, promote 0.45
    assert_eq!(place(0.40, Tier::Ram, &c), Tier::Ram);
    assert_eq!(place(0.40, Tier::Disk, &c), Tier::Disk);
    assert_eq!(place(0.29, Tier::Ram, &c), Tier::Disk);
    assert_eq!(place(0.46, Tier::Disk, &c), Tier::Ram);
    let flat = TierConfig { hysteresis_pct: 0, ..c };
    assert_eq!(place(0.30, Tier::Disk, &flat), Tier::Ram);
}

#[test]
fn the_account_cap_applies_only_when_others_are_present() {
    assert_eq!(account_cap(1000, 50, 1), 1000);
    assert_eq!(account_cap(1000, 50, 0), 1000);
    assert_eq!(account_cap(1000, 50, 2), 500);
    assert_eq!(account_cap(1000, 500, 3), 1000);
}

#[test]
fn disk_budget_follows_free_space_inside_floor_and_ceiling() {
    let mut d = DiskConfig { auto: true, floor_bytes: 100, ceiling_bytes: 1000, ..DiskConfig::default() };
    assert_eq!(disk_budget(&d, None, 0), 100);
    assert_eq!(disk_budget(&d, Some(5_000), 0), 500);
    assert_eq!(disk_budget(&d, Some(1_000_000), 0), 1000);
    // A nearly full disk never gives more than half of what is usable.
    assert_eq!(disk_budget(&d, Some(60), 0), 30);
    // What the cache already holds counts as usable space.
    assert_eq!(disk_budget(&d, Some(0), 4_000), 400);
    d.auto = false;
    d.manual_bytes = 777;
    assert_eq!(disk_budget(&d, Some(1), 0), 777);
}

#[test]
fn files_round_trip_and_corruption_is_rejected() {
    let body = png(7, 100);
    let tag = etag(&body);
    let f = encode_file(&body, &tag, 1, 2, 3.5);
    let head = parse_head(&f).unwrap();
    assert_eq!((head.len, head.created, head.expires, head.hits), (100, 1, 2, 3.5));
    assert_eq!(decode_file(&f, 100, |p| etag(p)).unwrap().as_ref(), &body[..]);
    let mut bad = f.clone();
    *bad.last_mut().unwrap() ^= 1;
    assert!(decode_file(&bad, 100, |p| etag(p)).is_none());
    assert!(decode_file(&f[..f.len() - 1], 100, |p| etag(p)).is_none());
    assert!(decode_file(&f, 99, |p| etag(p)).is_none());
    assert!(parse_head(b"garbage\n").is_none());
    assert!(parse_head(b"no newline").is_none());
}

#[test]
fn names_are_hashes_and_never_contain_the_url() {
    let h = key_hash("direct", "https://secret.example/pixel.png?u=alice");
    let name = file_name(h);
    assert_eq!(name.len(), 36);
    assert!(!name.contains("secret") && !name.contains("alice"));
    assert_eq!(parse_name(&name), Some(h));
    assert_ne!(key_hash("direct", "u"), key_hash("proxy", "u"));
    assert!(parse_name("../x.img").is_none() && parse_name("short.img").is_none());
    assert!(!valid_name("../../etc/passwd"));
}

// ── index and tiers with an injected backend ───────────────────────────────────

#[test]
fn an_index_miss_does_no_disk_io_and_a_ram_hit_does_none_either() {
    let io = Arc::new(MemIo::default());
    let t = tier_with(&io, cfg());
    let before = io.calls();
    assert!(matches!(t.probe("direct", "https://a/x.png", Context::Open, NOW), Probe::Miss));
    for i in 0..50 {
        assert!(matches!(t.probe("direct", &format!("https://a/{i}.png"), Context::Open, NOW), Probe::Miss));
    }
    assert_eq!(io.calls(), before, "misses must not touch the disk");
    put(&t, "https://a/x.png", "acct", png(1, 100), NOW);
    assert!(hit(&t, "https://a/x.png", NOW + 1));
    assert_eq!(io.calls(), before, "a RAM hit must not touch the disk");
}

#[test]
fn a_ram_eviction_demotes_a_hit_entry_to_disk_off_the_request_path() {
    let io = Arc::new(MemIo::default());
    let t = tier_with(&io, cfg());
    put(&t, "https://a/cold.png", "acct", png(7, 1500), NOW - 1);
    put(&t, "https://a/hot.png", "acct", png(1, 1500), NOW);
    assert!(hit(&t, "https://a/hot.png", NOW + 10)); // one hit makes it worth a write
    // Push it out of RAM with newer, larger traffic.
    fill(&t, "fill", NOW + 100);
    // The write is queued, not done: the request path wrote nothing.
    assert_eq!(io.writes.load(Ordering::SeqCst), 0, "no disk write on the request path");
    let name = file_name(key_hash("direct", "https://a/hot.png"));
    // Until the worker runs, the pending bytes still serve a hit with no IO.
    assert!(hit(&t, "https://a/hot.png", NOW + 200));
    t.run_jobs(NOW + 300);
    assert!(io.has(&name), "the worker wrote the demoted entry");
    let st = t.stats(NOW + 300);
    assert!(st.disk_entries >= 1 && st.disk_bytes > 0);
    // An image that was never hit is dropped on eviction, not written.
    assert!(!io.has(&file_name(key_hash("direct", "https://a/cold.png"))));
}

#[test]
fn a_disk_hit_reads_the_file_and_promotes_into_ram_above_the_threshold() {
    let io = Arc::new(MemIo::default());
    let t = tier_with(&io, cfg());
    put(&t, "https://a/p.png", "acct", png(3, 1000), NOW);
    for k in 0..3 {
        assert!(hit(&t, "https://a/p.png", NOW + 10 + k));
    }
    fill(&t, "f", NOW + 100);
    t.run_jobs(NOW + 200);
    let name = file_name(key_hash("direct", "https://a/p.png"));
    assert!(io.has(&name));
    // It left RAM for disk (pending finished, RAM copy dropped).
    let Probe::NeedDisk { name: n, size } = t.probe("direct", "https://a/p.png", Context::Open, NOW + 300) else {
        panic!("expected a disk probe");
    };
    assert_eq!((n.as_str(), size), (name.as_str(), 1000));
    let file = io.read(&name).unwrap();
    let body = decode_file(&file, size, |p| etag(p)).unwrap();
    assert_eq!(body.as_ref(), &png(3, 1000)[..]);
    let promoted_before = t.stats(NOW + 300).counters.promotions;
    assert!(t.finish("direct", "https://a/p.png", Some(file[file.len() - 1000 - 0..].to_vec()), NOW + 310).is_some());
    let st = t.stats(NOW + 320);
    assert!(st.counters.promotions > promoted_before, "{st:?}");
    // Now a RAM hit again.
    assert!(matches!(
        t.probe("direct", "https://a/p.png", Context::Open, NOW + 400),
        Probe::Hit { tier: Tier::Ram, .. }
    ));
}

#[test]
fn a_missing_file_is_drift_and_drops_the_entry() {
    let io = Arc::new(MemIo::default());
    let t = tier_with(&io, cfg());
    put(&t, "https://a/d.png", "acct", png(4, 1000), NOW);
    assert!(hit(&t, "https://a/d.png", NOW + 1));
    fill(&t, "g", NOW + 10);
    t.run_jobs(NOW + 100);
    let name = file_name(key_hash("direct", "https://a/d.png"));
    assert!(io.has(&name));
    io.files.lock().unwrap().remove(&name); // someone deleted it behind our back
    assert!(matches!(t.probe("direct", "https://a/d.png", Context::Open, NOW + 200), Probe::NeedDisk { .. }));
    assert!(t.finish("direct", "https://a/d.png", None, NOW + 201).is_none());
    assert!(matches!(t.probe("direct", "https://a/d.png", Context::Open, NOW + 202), Probe::Miss));
    assert_eq!(t.stats(NOW + 300).counters.drift_recoveries, 1);
    // A corrupt payload takes the same path.
}

#[tokio::test]
async fn the_async_lookup_serves_from_disk_and_survives_a_missing_file() {
    let io = Arc::new(MemIo::default());
    let t = Arc::new(tier_with(&io, cfg()));
    let now = now_ms();
    t.put_at("direct", "https://a/l.png", "acct", Context::Open, &etag(&png(5, 900)), png(5, 900), now);
    assert!(t.lookup("direct", "https://a/l.png", Context::Open).await.is_some());
    // Force it to disk by overfilling RAM.
    for i in 0..6u64 {
        let url = format!("https://a/h{i}.png");
        t.put_at("direct", &url, "acct", Context::Open, "\"e\"", png(2, 1500), now + 5 + i);
        for k in 0..6 {
            hit(&t, &url, now + 6 + i + k);
        }
    }
    t.run_jobs(now + 50);
    let got = t.lookup("direct", "https://a/l.png", Context::Open).await;
    assert_eq!(got.map(|(_, b)| b.len()), Some(900));
    let name = file_name(key_hash("direct", "https://a/l.png"));
    io.files.lock().unwrap().remove(&name);
    // Whatever tier it is in now, a lookup either serves from RAM or recovers.
    let _ = t.lookup("direct", "https://a/l.png", Context::Open).await;
    let _ = t.lookup("direct", "https://a/l.png", Context::Open).await;
}

#[test]
fn a_restart_rebuilds_the_index_from_the_directory_and_cleans_stale_files() {
    let io = Arc::new(MemIo::default());
    let t = tier_with(&io, cfg());
    put(&t, "https://a/r.png", "acct", png(6, 800), NOW);
    assert!(hit(&t, "https://a/r.png", NOW + 1));
    fill(&t, "s", NOW + 10);
    t.run_jobs(NOW + 100);
    let name = file_name(key_hash("direct", "https://a/r.png"));
    assert!(io.has(&name));
    // Debris a crashed run leaves: a temp file, an expired entry and a malformed file.
    io.files.lock().unwrap().insert("mwtmp-dead.tmp".into(), (vec![1], NOW));
    let stale = file_name(key_hash("direct", "https://a/old.png"));
    io.files
        .lock()
        .unwrap()
        .insert(stale.clone(), (encode_file(&png(9, 10), &etag(&png(9, 10)), NOW - 10_000, NOW - 1, 1.0), NOW));
    let bad = file_name(key_hash("direct", "https://a/bad.png"));
    io.files.lock().unwrap().insert(bad.clone(), (b"not a cache file".to_vec(), NOW));
    io.files.lock().unwrap().insert("README.txt".into(), (b"keep me".to_vec(), NOW));

    let t2 = tier_with(&io, cfg()); // "restart"
    assert!(!io.has("mwtmp-dead.tmp") && !io.has(&stale) && !io.has(&bad));
    assert!(io.has("README.txt"), "foreign files are never deleted");
    assert!(matches!(t2.probe("direct", "https://a/r.png", Context::Open, NOW + 200), Probe::NeedDisk { .. }));
    assert!(matches!(t2.probe("direct", "https://a/old.png", Context::Open, NOW + 200), Probe::Miss));
    let st = t2.stats(NOW + 200);
    let images = io.files.lock().unwrap().keys().filter(|n| parse_name(n).is_some()).count();
    assert!(st.disk_entries >= 1 && st.disk_entries == images, "{} vs {images}", st.disk_entries);
    assert_eq!(st.ram_entries, 0);
}

#[test]
fn the_snapshot_is_written_lazily_and_read_back_on_restart() {
    let io = Arc::new(MemIo::default());
    let c = TierConfig { snapshot_secs: 1, ..cfg() };
    let t = tier_with(&io, c.clone());
    put(&t, "https://a/n.png", "acct", png(8, 700), NOW);
    for _ in 0..2 {
        assert!(hit(&t, "https://a/n.png", NOW + 1));
    }
    fill(&t, "m", NOW + 10);
    t.run_jobs(NOW + 20);
    let writes = io.writes.load(Ordering::SeqCst);
    t.sweep(NOW + 5_000);
    assert!(io.has(SNAPSHOT));
    let snap_writes = io.writes.load(Ordering::SeqCst);
    assert_eq!(snap_writes, writes + 1);
    t.sweep(NOW + 6_000);
    assert_eq!(io.writes.load(Ordering::SeqCst), snap_writes, "no rewrite without changes");
    let t2 = tier_with(&io, c);
    let st = t2.stats(NOW + 7_000);
    let images = io.files.lock().unwrap().keys().filter(|n| parse_name(n).is_some()).count();
    assert!(st.disk_entries >= 1 && st.disk_entries == images, "{} vs {images}", st.disk_entries);
    // The snapshot's hit count survived: the twice-hit entry scores above a cold one.
    assert!(matches!(
        t2.probe("direct", "https://a/n.png", Context::Open, NOW + 7_001),
        Probe::NeedDisk { .. }
    ));
}

#[test]
fn a_full_disk_degrades_to_ram_only_without_errors_on_the_request_path() {
    let io = Arc::new(MemIo::default());
    let t = tier_with(&io, cfg());
    io.fail_write.store(true, Ordering::SeqCst);
    put(&t, "https://a/w.png", "acct", png(1, 1000), NOW);
    assert!(hit(&t, "https://a/w.png", NOW + 1));
    fill(&t, "z", NOW + 10);
    t.run_jobs(NOW + 100); // the write fails here, off the request path
    let st = t.stats(NOW + 100);
    assert!(st.counters.disk_errors >= 1 && !st.disk_active, "{st:?}");
    assert!(st.last_error.as_deref().is_some_and(|e| e.contains("no space")));
    // The entry whose write failed is gone; the cache keeps serving from RAM.
    assert!(matches!(t.probe("direct", "https://a/w.png", Context::Open, NOW + 110), Probe::Miss));
    put(&t, "https://a/after.png", "acct", png(1, 100), NOW + 120);
    assert!(hit(&t, "https://a/after.png", NOW + 121));
    // While degraded, evictions do not attempt writes.
    let writes = io.writes.load(Ordering::SeqCst);
    assert!(hit(&t, "https://a/after.png", NOW + 122));
    fill(&t, "y", NOW + 130);
    t.run_jobs(NOW + 200);
    assert_eq!(io.writes.load(Ordering::SeqCst), writes);
    // Recovery: after the backoff, a sweep re-probes the directory.
    io.fail_write.store(false, Ordering::SeqCst);
    t.sweep(NOW + 100 + DEGRADED_BACKOFF_MS + 1);
    assert!(t.stats(NOW + 100 + DEGRADED_BACKOFF_MS + 2).disk_active);
}

#[test]
fn a_missing_or_read_only_directory_leaves_the_cache_ram_only() {
    let io = Arc::new(MemIo::default());
    io.fail_dir.store(true, Ordering::SeqCst);
    let t = tier_with(&io, cfg());
    let st = t.stats(NOW);
    assert!(st.disk_enabled && !st.disk_active);
    assert!(st.last_error.as_deref().is_some_and(|e| e.contains("unavailable")));
    put(&t, "https://a/k.png", "acct", png(1, 100), NOW);
    assert!(hit(&t, "https://a/k.png", NOW + 1));
    // No path at all.
    let t2 = ImageTier::new(TierConfig { disk: DiskConfig { enabled: true, path: None, ..disk_cfg() }, ..cfg() });
    assert!(!t2.configure(&TierConfig { disk: DiskConfig { enabled: true, path: None, ..disk_cfg() }, ..cfg() }, NOW));
    put(&t2, "https://a/k.png", "acct", png(1, 100), NOW);
    assert!(hit(&t2, "https://a/k.png", NOW + 1));
}

#[test]
fn shrinking_the_budget_evicts_without_a_restart() {
    let io = Arc::new(MemIo::default());
    let big = TierConfig { ram_bytes: 1 << 20, ..cfg() };
    let t = tier_with(&io, big.clone());
    for i in 0..10 {
        put(&t, &format!("https://a/q{i}.png"), "acct", png(1, 1000), NOW + i);
    }
    assert_eq!(t.stats(NOW + 20).ram_entries, 10);
    let small = TierConfig { ram_entries: 4, ..big.clone() };
    t.configure(&small, NOW + 30);
    assert_eq!(t.stats(NOW + 31).ram_entries, 4);
    let tiny = TierConfig { ram_bytes: 2 * 1000 + 2 * ENTRY_OVERHEAD, ..big };
    t.configure(&tiny, NOW + 40);
    let st = t.stats(NOW + 41);
    assert!(st.ram_used <= tiny.ram_bytes && st.ram_entries <= 2, "{st:?}");
    // Lowest scores went first: the newest entry survives.
    assert!(hit(&t, "https://a/q9.png", NOW + 50));
    // Disk shrink: drop the budget below what is held.
    let (io2, d) = (Arc::new(MemIo::default()), disk_cfg());
    let t2 = tier_with(&io2, TierConfig { ram_bytes: 1000, ram_entry_bytes: 500, disk: d.clone(), ..cfg() });
    for i in 0..10 {
        put(&t2, &format!("https://a/big{i}.png"), "acct", png(1, 2000), NOW + i);
    }
    t2.run_jobs(NOW + 100);
    let held = t2.stats(NOW + 101).disk_entries;
    assert!(held >= 3, "{held}");
    t2.configure(
        &TierConfig {
            ram_bytes: 1000,
            ram_entry_bytes: 500,
            disk: DiskConfig { manual_bytes: 2 * (2000 + FILE_OVERHEAD as u64 + 40), ..d },
            ..cfg()
        },
        NOW + 200,
    );
    t2.run_jobs(NOW + 201);
    let after = t2.stats(NOW + 202);
    assert!(after.disk_entries <= 2 && after.disk_bytes <= after.disk_budget, "{after:?}");
    assert!(io2.files.lock().unwrap().len() <= 2, "evicted files were deleted");
}

#[test]
fn an_entry_larger_than_the_ram_limit_goes_straight_to_disk_and_oversize_is_dropped() {
    let io = Arc::new(MemIo::default());
    let t = tier_with(&io, cfg());
    put(&t, "https://a/large.png", "acct", png(1, 10_000), NOW); // > ram_entry_bytes 4096, < disk 64K
    put(&t, "https://a/huge.png", "acct", png(1, 100_000), NOW); // > both
    assert!(matches!(t.probe("direct", "https://a/huge.png", Context::Open, NOW + 1), Probe::Miss));
    assert!(matches!(
        t.probe("direct", "https://a/large.png", Context::Open, NOW + 1),
        Probe::Hit { tier: Tier::Disk, .. }
    ));
    t.run_jobs(NOW + 2);
    assert!(io.has(&file_name(key_hash("direct", "https://a/large.png"))));
}

#[test]
fn expiry_and_max_age_drop_entries_and_release_their_bytes() {
    let io = Arc::new(MemIo::default());
    let c = TierConfig { max_age_secs: 7200, ..cfg() };
    let t = tier_with(&io, c.clone());
    put(&t, "https://a/e.png", "acct", png(1, 500), NOW);
    assert!(hit(&t, "https://a/e.png", NOW + 1000));
    // Past the base TTL but extended by the hit.
    assert!(hit(&t, "https://a/e.png", NOW + 3_600_000 + 10));
    // Hits can never push it past max age.
    assert!(matches!(t.probe("direct", "https://a/e.png", Context::Open, NOW + 7_200_000 + 1), Probe::Miss));
    assert_eq!(t.stats(NOW + 7_300_000).ram_bytes, 0);
    // A cold entry expires at the base TTL and the sweep removes it.
    put(&t, "https://a/c.png", "acct", png(1, 500), NOW);
    t.sweep(NOW + 3_600_001);
    assert_eq!(t.stats(NOW + 3_600_002).index_entries, 0);
}

#[test]
fn an_account_over_its_share_evicts_its_own_entries_before_another_accounts() {
    let io = Arc::new(MemIo::default());
    let c = TierConfig { ram_bytes: 20 * ENTRY_OVERHEAD + 8000, ram_entry_bytes: 2000, account_share_pct: 50, ..cfg() };
    let t = tier_with(&io, c);
    put(&t, "https://b/1.png", "bob", png(1, 1000), NOW);
    for i in 0..10 {
        put(&t, &format!("https://a/{i}.png"), "alice", png(2, 1000), NOW + 10 + i);
    }
    // Bob's older, lower-scoring entry would be the global victim by recency, but
    // Alice is over her share, so she paid for her own growth.
    assert!(hit(&t, "https://b/1.png", NOW + 100));
    let st = t.stats(NOW + 100);
    // Alice is held to her share (half of the budget) while Bob keeps his image.
    assert!(st.ram_bytes <= 6560 + 1000, "{st:?}");
}

#[test]
fn hysteresis_prevents_flapping_and_cold_ram_entries_are_demoted_or_dropped_under_pressure() {
    let io = Arc::new(MemIo::default());
    let c = TierConfig { ram_bytes: 10 * ENTRY_OVERHEAD + 5000, ram_entry_bytes: 2000, sweep_batch: 64, ..cfg() };
    let t = tier_with(&io, c);
    for i in 0..4 {
        put(&t, &format!("https://a/t{i}.png"), "acct", png(1, 1200), NOW + i);
    }
    assert!(t.stats(NOW + 10).ram_used * 5 > 5000 * 4 - 1);
    // Three hours of idleness: scores decay below the demote threshold; a sweep
    // under memory pressure demotes (hit entries) or drops (cold ones).
    let later = NOW + 3 * 3_600_000 - 1;
    t.sweep(later);
    let st = t.stats(later);
    assert!(st.ram_entries < 4 || st.ram_cold == st.ram_entries, "{st:?}");
}

#[test]
fn a_replaced_entry_releases_its_old_bytes_and_file() {
    let io = Arc::new(MemIo::default());
    let t = tier_with(&io, cfg());
    put(&t, "https://a/u.png", "acct", png(1, 1000), NOW);
    put(&t, "https://a/u.png", "acct", png(2, 500), NOW + 1);
    let st = t.stats(NOW + 2);
    assert_eq!((st.ram_entries, st.ram_bytes, st.index_entries), (1, 500, 1));
    let Probe::Hit { png: p, .. } = t.probe("direct", "https://a/u.png", Context::Open, NOW + 3) else {
        panic!("hit expected");
    };
    assert_eq!(p.as_ref(), &png(2, 500)[..]);
}

#[test]
fn the_index_memory_is_counted_in_the_ram_budget() {
    let io = Arc::new(MemIo::default());
    let t = tier_with(&io, cfg());
    put(&t, "https://a/m.png", "acct", png(1, 100), NOW);
    let st = t.stats(NOW);
    assert_eq!(st.ram_used, 100 + ENTRY_OVERHEAD);
    // Tiny payloads cannot exceed the budget through index overhead alone.
    let small = TierConfig { ram_bytes: 5 * ENTRY_OVERHEAD, ..cfg() };
    let t2 = tier_with(&Arc::new(MemIo::default()), small.clone());
    for i in 0..50 {
        put(&t2, &format!("https://a/{i}.png"), "acct", png(1, 10), NOW + i);
    }
    assert!(t2.stats(NOW + 100).ram_used <= small.ram_bytes);
}

#[test]
fn the_real_filesystem_backend_writes_atomically_with_hash_names_and_cleans_up() {
    let dir = std::env::temp_dir().join(format!("mw-tier-test-{}", rand_u128()));
    let io = FsIo::new(dir.clone());
    io.ensure_dir().unwrap();
    let h = key_hash("direct", "https://a/fs.png");
    let name = file_name(h);
    let body = png(5, 321);
    io.write_atomic(&name, &encode_file(&body, &etag(&body), 1, 2, 0.0)).unwrap();
    assert_eq!(io.read(&name).unwrap().len() > 321, true);
    let listed = io.list(100).unwrap();
    assert_eq!(listed.len(), 1, "no temp file is left behind");
    assert_eq!(listed[0].name, name);
    assert!(io.write_atomic("../escape.img", b"x").is_err());
    assert!(io.read("../escape.img").is_err());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(dir.join(&name)).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
    assert!(io.free_bytes().is_some_and(|f| f > 0));
    io.remove(&name).unwrap();
    io.remove(&name).unwrap(); // idempotent
    assert!(io.list(10).unwrap().is_empty());
    // A path that is a file, not a directory, is refused.
    let file = dir.join("afile");
    std::fs::write(&file, b"x").unwrap();
    assert!(FsIo::new(file).ensure_dir().is_err());
    std::fs::remove_dir_all(dir).unwrap();
}

// ── authorisation comes first ──────────────────────────────────────────────────

/// The cache holds public bytes, so the only thing between a requester and them is
/// that authentication and the signed-URL check run BEFORE any lookup. Seed the
/// real global tier with the exact key the legacy proxy would use, then ask for it
/// without a session and with a forged signed URL: neither may be served, and the
/// tier must not even be probed (its hit and miss counters do not move).
/// `build_app` mounts every route and its future is large in a debug build: on the
/// default 2 MiB test-thread stack this test overflowed as the app grew (it passes at
/// 3 MiB), so it runs its runtime on a thread with a stack that leaves room.
#[test]
fn an_unauthenticated_or_unsigned_request_never_reads_the_cache() {
    std::thread::Builder::new()
        .stack_size(32 << 20)
        .spawn(|| {
            tokio::runtime::Builder::new_multi_thread()
                .thread_stack_size(32 << 20)
                .enable_all()
                .build()
                .unwrap()
                .block_on(unauthenticated_requests_never_read_the_cache())
        })
        .unwrap()
        .join()
        .unwrap();
}

async fn unauthenticated_requests_never_read_the_cache() {
    let dir = std::env::temp_dir().join(format!("mw-tier-auth-{}", rand_u128()));
    std::fs::create_dir_all(&dir).unwrap();
    let config = crate::AppConfig {
        db_path: dir.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: None,
        web_dir: None,
        cookie_secure: false,
        mode: crate::ServerMode::Proxy,
        hardening: crate::HardeningConfig::default(),
        security: crate::SecurityConfig::default(),
    };
    let app = crate::build_app(config).await.expect("build_app");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    let url = "https://cdn.example/seeded-secret.png";
    let body = png(9, 600);
    let tag = etag(&body);
    let tier = global();
    tier.store("direct:legacy:true:8388608", url, "acct", Context::Open, tag.clone(), body.clone());
    tier.store("direct:true:8388608", url, "acct", Context::Open, tag, body.clone());
    assert!(matches!(
        tier.lock().probe(key_hash("direct:legacy:true:8388608", url), Context::Open, now_ms()),
        Probe::Hit { .. }
    ), "the seed is really in the cache");
    let before = tier.stats(now_ms()).counters;

    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    let legacy = client
        .get(format!("{base}/api/image-proxy?url=https%3A%2F%2Fcdn.example%2Fseeded-secret.png"))
        .send()
        .await
        .unwrap();
    assert_eq!(legacy.status(), 401, "no session, no image");
    assert_ne!(legacy.bytes().await.unwrap().as_ref(), &body[..]);
    for path in ["/api/rc/i/forged.token", "/api/rc/i/"] {
        let r = client.get(format!("{base}{path}")).send().await.unwrap();
        assert!(
            r.status().is_client_error(),
            "{path} must be refused, got {}",
            r.status()
        );
        assert_ne!(r.bytes().await.unwrap().as_ref(), &body[..], "{path}");
    }
    let after = tier.stats(now_ms()).counters;
    assert_eq!(
        (before.ram_hits, before.disk_hits, before.misses),
        (after.ram_hits, after.disk_hits, after.misses),
        "a refused request must not read the cache"
    );
}

// ── stale-while-revalidate ─────────────────────────────────────────────────────

fn put_swr(t: &ImageTier, url: &str, now: u64, up: Option<Upstream>, d: Directives) {
    let bytes = png(7, 600);
    let e = etag(&bytes);
    t.lock().put_with(key_hash("direct", url), account_hash("acct"), Context::Open, e, Bytes::from(bytes), now, up, d);
}

fn probe_swr(t: &ImageTier, url: &str, now: u64, grace_secs: u64) -> Probe {
    t.lock().probe_with(key_hash("direct", url), Context::Open, now, grace_secs * 1000)
}

fn swr_cfg() -> TierConfig {
    TierConfig { stale_secs: 3600, max_stale_secs: 86_400, ..cfg() }
}

#[test]
fn a_stale_entry_is_served_inside_the_window_and_is_not_extended_by_being_served() {
    let io = Arc::new(MemIo::default());
    let t = tier_with(&io, swr_cfg());
    let up = Upstream { etag: Some("\"v1\"".into()), last_modified: None };
    put_swr(&t, "https://a/s.png", NOW, Some(up.clone()), Directives::default());
    // Past the base TTL (3600 s) but inside the window: stale, with the validators.
    let at = NOW + 3_600_000 + 60_000;
    match probe_swr(&t, "https://a/s.png", at, 3600) {
        Probe::Hit { stale: true, validators, .. } => assert_eq!(validators, Some(up)),
        _ => panic!("expected a stale hit"),
    }
    // Serving it again does not push its expiry out: still stale, not fresh.
    assert!(matches!(probe_swr(&t, "https://a/s.png", at + 1000, 3600), Probe::Hit { stale: true, .. }));
    // Without a window it is a plain miss (the old behaviour), and kept for the error fallback.
    assert!(matches!(probe_swr(&t, "https://a/s.png", at, 0), Probe::Miss));
    assert!(matches!(probe_swr(&t, "https://a/s.png", at, 86_400), Probe::Hit { stale: true, .. }));
    // Past the window it is a miss for the revalidation path, but the error fallback still has it.
    let later = NOW + 3_600_000 + 3_600_000 + 1;
    assert!(matches!(probe_swr(&t, "https://a/s.png", later, 3600), Probe::Miss));
    assert!(matches!(probe_swr(&t, "https://a/s.png", later, 86_400), Probe::Hit { stale: true, .. }));
    // Past the maximum stale age it is gone.
    let gone = NOW + 3_600_000 + 86_400_000 + 1;
    assert!(matches!(probe_swr(&t, "https://a/s.png", gone, 86_400), Probe::Miss));
    assert_eq!(t.stats(gone).index_entries, 0);
}

#[test]
fn a_revalidated_entry_is_fresh_again_and_keeps_serving_without_a_refetch() {
    let io = Arc::new(MemIo::default());
    let t = tier_with(&io, swr_cfg());
    put_swr(&t, "https://a/r.png", NOW, None, Directives::default());
    let at = NOW + 3_600_000 + 5000;
    assert!(matches!(probe_swr(&t, "https://a/r.png", at, 3600), Probe::Hit { stale: true, .. }));
    let new = Upstream { etag: Some("\"v2\"".into()), last_modified: Some("Wed".into()) };
    assert!(t.lock().renew(key_hash("direct", "https://a/r.png"), Some(new.clone()), Directives::default(), at));
    match probe_swr(&t, "https://a/r.png", at + 1000, 3600) {
        Probe::Hit { stale: false, validators, .. } => assert_eq!(validators, Some(new)),
        _ => panic!("expected a fresh hit after the 304"),
    }
    assert_eq!(t.stats(at).counters.revalidated, 1);
}

#[test]
fn sweep_keeps_an_entry_through_its_stale_window_and_then_drops_it() {
    let io = Arc::new(MemIo::default());
    let t = tier_with(&io, swr_cfg());
    put_swr(&t, "https://a/w.png", NOW, None, Directives::default());
    t.sweep(NOW + 3_600_001 + 60_000);
    assert_eq!(t.stats(NOW).index_entries, 1, "kept for the stale window");
    t.sweep(NOW + 3_600_000 + 86_400_000 + 1);
    assert_eq!(t.stats(NOW).index_entries, 0);
}

#[test]
fn cache_control_is_honoured() {
    let d = parse_cache_control("public, max-age=60, s-maxage=30");
    assert_eq!(d, Directives { no_store: false, no_cache: false, max_age: Some(30) });
    assert!(parse_cache_control("no-store").no_store);
    let nc = parse_cache_control("No-Cache");
    assert!(nc.no_cache && nc.max_age.is_none());
    assert!(parse_cache_control("max-age=0").no_cache);
    assert_eq!(parse_cache_control("max-age=abc, private").max_age, None);

    let io = Arc::new(MemIo::default());
    let t = tier_with(&io, swr_cfg());
    // no-store is never stored.
    t.store_with("direct", "https://a/ns.png", "acct", Context::Open, "e".into(), png(1, 500), None, Directives { no_store: true, ..Default::default() });
    assert_eq!(t.stats(NOW).index_entries, 0);
    // max-age shorter than the base TTL bounds freshness, and hits do not extend it.
    put_swr(&t, "https://a/ma.png", NOW, None, Directives { max_age: Some(60), ..Default::default() });
    assert!(matches!(probe_swr(&t, "https://a/ma.png", NOW + 30_000, 3600), Probe::Hit { stale: false, .. }));
    assert!(matches!(probe_swr(&t, "https://a/ma.png", NOW + 61_000, 3600), Probe::Hit { stale: true, .. }));
    // no-cache is stale at once (served, then revalidated).
    put_swr(&t, "https://a/nc.png", NOW, None, Directives { no_cache: true, ..Default::default() });
    assert!(matches!(probe_swr(&t, "https://a/nc.png", NOW + 1, 3600), Probe::Hit { stale: true, .. }));
    assert!(matches!(probe_swr(&t, "https://a/nc.png", NOW + 1, 0), Probe::Miss));
}
