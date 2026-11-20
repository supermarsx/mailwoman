//! Opt-in, process-wide diagnostic wall timings. No identifiers or file paths.
use serde::Serialize;
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed},
    time::Instant,
};

static ENABLED: AtomicBool = AtomicBool::new(false);
const NAMES: [&str; 8] = [
    "writerLockWait",
    "commit",
    "readerReload",
    "sealedRead",
    "sealedDecode",
    "sealedEncode",
    "sealedWrite",
    "directorySync",
];
#[derive(Clone, Copy)]
pub(crate) enum Stage {
    WriterLockWait,
    Commit,
    ReaderReload,
    SealedRead,
    SealedDecode,
    SealedEncode,
    SealedWrite,
    DirectorySync,
}
struct Stat {
    calls: AtomicU64,
    failures: AtomicU64,
    total_micros: AtomicU64,
    max_micros: AtomicU64,
    bytes: AtomicU64,
}
impl Stat {
    const fn new() -> Self {
        Self {
            calls: AtomicU64::new(0),
            failures: AtomicU64::new(0),
            total_micros: AtomicU64::new(0),
            max_micros: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
        }
    }
}
static STATS: [Stat; 8] = [const { Stat::new() }; 8];
const WRITE_NAMES: [&str; 4] = [
    "managedMetadata",
    "indexMetadata",
    "segmentComponent",
    "other",
];
static WRITE_STATS: [Stat; 4] = [const { Stat::new() }; 4];
fn sample(s: &Stat) -> Sample {
    Sample {
        calls: s.calls.load(Relaxed),
        failures: s.failures.load(Relaxed),
        total_micros: s.total_micros.load(Relaxed),
        max_micros: s.max_micros.load(Relaxed),
        bytes: s.bytes.load(Relaxed),
    }
}
fn write_category(path: &std::path::Path) -> usize {
    if path == std::path::Path::new(".managed.json") {
        return 0;
    }
    if path == std::path::Path::new("meta.json") {
        return 1;
    }
    if path.components().count() == 1
        && let Some(name) = path.to_str()
    {
        let stem = name.split('.').next().unwrap_or("");
        if stem.len() == 32 && stem.bytes().all(|byte| byte.is_ascii_hexdigit()) && name.len() > 33
        {
            return 2;
        }
    }
    3
}
fn add(target: &AtomicU64, value: u64) {
    let _ = target.fetch_update(Relaxed, Relaxed, |old| Some(old.saturating_add(value)));
}

/// Enable only for explicit diagnostics. Counters are cumulative, never reset.
pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Relaxed);
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sample {
    pub calls: u64,
    pub failures: u64,
    pub total_micros: u64,
    pub max_micros: u64,
    pub bytes: u64,
}
#[derive(Clone, Debug, Serialize)]
pub struct Snapshot {
    pub scope: &'static str,
    pub enabled: bool,
    pub phases: BTreeMap<&'static str, Sample>,
    #[serde(rename = "writeCategories")]
    pub write_categories: BTreeMap<&'static str, Sample>,
}
pub fn snapshot() -> Snapshot {
    Snapshot {
        scope: "process",
        enabled: ENABLED.load(Relaxed),
        phases: NAMES.into_iter().zip(STATS.iter().map(sample)).collect(),
        write_categories: WRITE_NAMES
            .into_iter()
            .zip(WRITE_STATS.iter().map(sample))
            .collect(),
    }
}
pub(crate) struct Timer {
    stat: &'static Stat,
    start: Option<Instant>,
    recorded: bool,
}
impl Timer {
    fn new(stage: Stage) -> Self {
        Self::for_stat(&STATS[stage as usize])
    }
    fn for_stat(stat: &'static Stat) -> Self {
        Self {
            stat,
            start: ENABLED.load(Relaxed).then(Instant::now),
            recorded: false,
        }
    }
    fn finish(&mut self, success: bool, bytes: u64) {
        if let Some(start) = self.start {
            let elapsed = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
            let s = self.stat;
            add(&s.calls, 1);
            if !success {
                add(&s.failures, 1);
            }
            add(&s.total_micros, elapsed);
            s.max_micros.fetch_max(elapsed, Relaxed);
            add(&s.bytes, bytes);
        }
        self.recorded = true;
    }
}
impl Drop for Timer {
    fn drop(&mut self) {
        if !self.recorded {
            self.finish(false, 0);
        }
    }
}
pub(crate) fn measure<T, E>(
    stage: Stage,
    bytes: u64,
    f: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    let mut timer = Timer::new(stage);
    let result = f();
    timer.finish(result.is_ok(), bytes);
    result
}
/// Classify only while profiling; no identifiers are retained. Aggregate write
/// timing still wraps the identical underlying operation; categories overlap it.
pub(crate) fn measure_write<T, E>(
    path: &std::path::Path,
    bytes: u64,
    f: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    if !ENABLED.load(Relaxed) {
        return f();
    }
    let mut category = Timer::for_stat(&WRITE_STATS[write_category(path)]);
    let result = measure(Stage::SealedWrite, bytes, f);
    category.finish(result.is_ok(), bytes);
    result
}
pub(crate) fn measure_read<E>(f: impl FnOnce() -> Result<Vec<u8>, E>) -> Result<Vec<u8>, E> {
    let mut timer = Timer::new(Stage::SealedRead);
    let result = f();
    timer.finish(
        result.is_ok(),
        result.as_ref().map_or(0, |bytes| bytes.len() as u64),
    );
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    struct Enabled;
    impl Drop for Enabled {
        fn drop(&mut self) {
            set_enabled(false);
        }
    }
    #[test]
    fn disabled_is_noop_and_failures_preserve_results() {
        let _lock = TEST_LOCK.lock().unwrap();
        set_enabled(false);
        let before = snapshot();
        assert_eq!(measure(Stage::Commit, 7, || Ok::<_, ()>(42)), Ok(42));
        assert_eq!(
            snapshot().phases["commit"].calls,
            before.phases["commit"].calls
        );
        set_enabled(true);
        let _enabled = Enabled;
        assert_eq!(
            measure(Stage::Commit, 7, || Err::<(), _>("unchanged error")),
            Err("unchanged error")
        );
        let after = snapshot();
        set_enabled(false);
        assert_eq!(
            after.phases["commit"].calls,
            before.phases["commit"].calls + 1
        );
        assert_eq!(
            after.phases["commit"].failures,
            before.phases["commit"].failures + 1
        );
        assert_eq!(
            after.phases["commit"].bytes,
            before.phases["commit"].bytes + 7
        );
    }

    #[test]
    fn actual_sealed_publication_and_corrupt_decode_are_observed() {
        let _lock = TEST_LOCK.lock().unwrap();
        set_enabled(true);
        let _enabled = Enabled;
        let before = snapshot();
        let dir = std::env::temp_dir().join(format!(
            "mw-index-timing-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let key = mw_store::ServerKey::generate();
        {
            let index = crate::Index::open_sealed(&dir, key.clone()).unwrap();
            index
                .upsert(&crate::IndexDoc {
                    stable_id: "timing-id".into(),
                    account_id: "timing-account".into(),
                    subject: "timingprobe".into(),
                    ..Default::default()
                })
                .unwrap();
            index.reload().unwrap();
            assert_eq!(
                index
                    .search(&crate::parse_query("timingprobe").unwrap(), 10)
                    .unwrap(),
                vec!["timing-id"]
            );
        }
        let mut bytes = std::fs::read(dir.join("meta.json")).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        std::fs::write(dir.join("meta.json"), bytes).unwrap();
        assert!(crate::Index::open_sealed(&dir, key).is_err());
        let after = snapshot();
        for name in NAMES {
            assert!(
                after.phases[name].calls > before.phases[name].calls,
                "{name}"
            );
        }
        assert!(after.phases["sealedDecode"].failures > before.phases["sealedDecode"].failures);
        assert!(after.phases["sealedRead"].bytes > before.phases["sealedRead"].bytes);
        assert!(after.phases["sealedWrite"].bytes > before.phases["sealedWrite"].bytes);
        assert!(dir.starts_with(std::env::temp_dir()));
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn sealed_write_categories_use_actual_writers_and_preserve_io_failures() {
        use std::{io::Write, path::Path};
        use tantivy::directory::{Directory, TerminatingWrite};
        let _lock = TEST_LOCK.lock().unwrap();
        let dir = std::env::temp_dir().join(format!(
            "mw-write-categories-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&dir).unwrap();
        let directory =
            crate::sealed_directory::SealedDirectory::open(&dir, mw_store::ServerKey::generate())
                .unwrap();
        set_enabled(false);
        let disabled = snapshot();
        directory
            .atomic_write(Path::new(".managed.json"), b"[]")
            .unwrap();
        assert_eq!(
            snapshot().write_categories["managedMetadata"].calls,
            disabled.write_categories["managedMetadata"].calls
        );
        set_enabled(true);
        let _enabled = Enabled;
        let before = snapshot();
        directory
            .atomic_write(Path::new(".managed.json"), b"[]")
            .unwrap();
        directory
            .atomic_write(Path::new("meta.json"), b"{}")
            .unwrap();
        let component = Path::new("0123456789abcdef0123456789abcdef.store");
        let mut writer = directory.open_write(component).unwrap();
        writer.write_all(b"component").unwrap();
        writer.terminate().unwrap();
        assert_eq!(directory.atomic_read(component).unwrap(), b"component");
        directory
            .atomic_write(Path::new("unclassified"), b"other")
            .unwrap();
        // Force an actual same-directory atomic rename failure after encoding.
        std::fs::remove_file(dir.join("meta.json")).unwrap();
        std::fs::create_dir(dir.join("meta.json")).unwrap();
        assert!(
            directory
                .atomic_write(Path::new("meta.json"), b"failure")
                .is_err()
        );
        let after = snapshot();
        for (name, calls, failures, bytes) in [
            ("managedMetadata", 1, 0, 2 + 44),
            ("indexMetadata", 2, 1, 2 + 7 + 88),
            ("segmentComponent", 1, 0, 9 + 44),
            ("other", 1, 0, 5 + 44),
        ] {
            let a = &after.write_categories[name];
            let b = &before.write_categories[name];
            assert_eq!(a.calls - b.calls, calls, "{name}");
            assert_eq!(a.failures - b.failures, failures, "{name}");
            assert_eq!(a.bytes - b.bytes, bytes, "{name}");
            assert!(a.total_micros >= b.total_micros);
        }
        assert_eq!(
            after.phases["sealedWrite"].calls - before.phases["sealedWrite"].calls,
            5
        );
        assert_eq!(
            after.phases["sealedWrite"].failures - before.phases["sealedWrite"].failures,
            1
        );
        let reported = serde_json::to_string(&after).unwrap();
        assert!(!reported.contains("0123456789abcdef"));
        assert!(!reported.contains("unclassified"));
        assert!(!reported.contains("mw-write-categories"));
        assert_eq!(write_category(Path::new("private/meta.json")), 3);
        drop(directory);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
