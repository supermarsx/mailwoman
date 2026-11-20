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
}
pub fn snapshot() -> Snapshot {
    Snapshot {
        scope: "process",
        enabled: ENABLED.load(Relaxed),
        phases: NAMES
            .into_iter()
            .zip(STATS.iter().map(|s| Sample {
                calls: s.calls.load(Relaxed),
                failures: s.failures.load(Relaxed),
                total_micros: s.total_micros.load(Relaxed),
                max_micros: s.max_micros.load(Relaxed),
                bytes: s.bytes.load(Relaxed),
            }))
            .collect(),
    }
}
pub(crate) struct Timer {
    stage: Stage,
    start: Option<Instant>,
    recorded: bool,
}
impl Timer {
    fn new(stage: Stage) -> Self {
        Self {
            stage,
            start: ENABLED.load(Relaxed).then(Instant::now),
            recorded: false,
        }
    }
    fn finish(&mut self, success: bool, bytes: u64) {
        if let Some(start) = self.start {
            let elapsed = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
            let s = &STATS[self.stage as usize];
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
}
