//! t34 server modes for the native load fixture (all off by default).
//!
//! The fixture validates every selector before any database creation or
//! seeding, builds the engine with the requested index publication options
//! (the fixture builds its own `Engine`, so the server's environment parser is
//! not on its path), and reports what was requested together with a
//! per-snapshot witness. The witness never controls application authority.
//!
//! Ranges mirror the server parsers (`guard_snapshot::parse`,
//! `engine_mode::publication_options_from`, `write_behind::AccountingConfig`).
//! An unset or empty variable takes the server default.
//!
//! Guard snapshot and write-behind counters are `pub(crate)` in `mw-server`
//! and reachable only through `AppState`. Until production exposes a hook
//! (see `scripts/perf/t34-modes.mjs`, "required production exposure"), the
//! witness reports them as `supported:false`, and the harness refuses to
//! qualify an arm that claims either mode.
use serde_json::{Value, json};
use std::time::Duration;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct T34Selection {
    pub guard_mode: Option<String>,
    pub guard_staleness_ms: Option<u64>,
    pub ack: Option<String>,
    pub window_ms: Option<u64>,
    pub max_batch_jobs: Option<u64>,
    pub fence_timeout_ms: Option<u64>,
    pub drain_secs: Option<u64>,
    pub accounting: Option<String>,
    pub flush_ms: Option<u64>,
    pub max_pending: Option<u64>,
}

fn unset(raw: Option<&str>) -> Option<&str> {
    raw.map(str::trim).filter(|value| !value.is_empty())
}
fn word(name: &str, raw: Option<&str>, allowed: &[&str]) -> Result<Option<String>, String> {
    match unset(raw) {
        None => Ok(None),
        Some(value) if allowed.contains(&value) => Ok(Some(value.to_owned())),
        Some(value) => Err(format!("{name}={value:?} is invalid; use one of {}", allowed.join(", "))),
    }
}
fn ranged(name: &str, raw: Option<&str>, low: u64, high: u64) -> Result<Option<u64>, String> {
    match unset(raw) {
        None => Ok(None),
        Some(value) => value
            .parse::<u64>()
            .ok()
            .filter(|n| (low..=high).contains(n) && value.bytes().all(|b| b.is_ascii_digit()))
            .map(Some)
            .ok_or_else(|| format!("{name}={value:?} is invalid; use an integer from {low} to {high}")),
    }
}

impl T34Selection {
    pub fn parse(var: impl Fn(&str) -> Option<String>) -> Result<Self, String> {
        let get = |name: &str| var(name);
        let selection = Self {
            guard_mode: word("MW_GUARD_SNAPSHOT", get("MW_GUARD_SNAPSHOT").as_deref(), &["off", "bounded"])?,
            guard_staleness_ms: ranged("MW_GUARD_SNAPSHOT_MAX_STALENESS_MS", get("MW_GUARD_SNAPSHOT_MAX_STALENESS_MS").as_deref(), 100, 5000)?,
            ack: word("MW_INDEX_PUBLICATION_ACK", get("MW_INDEX_PUBLICATION_ACK").as_deref(), &["commit", "journal"])?,
            window_ms: ranged("MW_INDEX_PUBLICATION_WINDOW_MS", get("MW_INDEX_PUBLICATION_WINDOW_MS").as_deref(), 0, 10)?,
            max_batch_jobs: ranged("MW_INDEX_PUBLICATION_MAX_BATCH_JOBS", get("MW_INDEX_PUBLICATION_MAX_BATCH_JOBS").as_deref(), 1, 128)?,
            fence_timeout_ms: ranged("MW_INDEX_READ_FENCE_TIMEOUT_MS", get("MW_INDEX_READ_FENCE_TIMEOUT_MS").as_deref(), 100, 30_000)?,
            drain_secs: ranged("MW_INDEX_PUBLICATION_DRAIN_SECS", get("MW_INDEX_PUBLICATION_DRAIN_SECS").as_deref(), 0, 60)?,
            accounting: word("MW_REQUEST_PROTECTION_ACCOUNTING", get("MW_REQUEST_PROTECTION_ACCOUNTING").as_deref(), &["synchronous", "write-behind"])?,
            flush_ms: ranged("MW_REQUEST_PROTECTION_FLUSH_MS", get("MW_REQUEST_PROTECTION_FLUSH_MS").as_deref(), 50, 5000)?,
            max_pending: ranged("MW_REQUEST_PROTECTION_MAX_PENDING", get("MW_REQUEST_PROTECTION_MAX_PENDING").as_deref(), 64, 65_536)?,
        };
        Ok(selection)
    }

    pub fn from_environment() -> Result<Self, String> {
        Self::parse(|name| std::env::var(name).ok())
    }

    /// Index publication options. The fixture database is always SQLite.
    pub fn publication_options(&self) -> Result<mw_engine::PublicationOptions, String> {
        let mut options = mw_engine::PublicationOptions::default();
        if let Some(ack) = &self.ack {
            options.ack = mw_engine::PublicationAck::parse(ack).ok_or("invalid acknowledgement mode")?;
        }
        if let Some(ms) = self.window_ms {
            options.collection_window = Duration::from_millis(ms);
        }
        if let Some(jobs) = self.max_batch_jobs {
            options.max_batch_jobs = jobs as usize;
        }
        if let Some(ms) = self.fence_timeout_ms {
            options.read_fence_timeout = Duration::from_millis(ms);
        }
        if let Some(secs) = self.drain_secs {
            options.drain_timeout = Duration::from_secs(secs);
        }
        options.validate().map_err(|error| error.to_string())?;
        Ok(options)
    }

    /// What the operator asked for, with explicit defaults for unset values.
    pub fn requested(&self) -> Value {
        json!({
            "guardSnapshot": {
                "mode": self.guard_mode.as_deref().unwrap_or("off"),
                "maxStalenessMs": self.guard_staleness_ms,
            },
            "indexPublication": {
                "ack": self.ack.as_deref().unwrap_or("commit"),
                "windowMs": self.window_ms,
                "maxBatchJobs": self.max_batch_jobs,
                "readFenceTimeoutMs": self.fence_timeout_ms,
                "drainSecs": self.drain_secs,
            },
            "requestProtection": {
                "accounting": self.accounting.as_deref().unwrap_or("synchronous"),
                "flushMs": self.flush_ms,
                "maxPending": self.max_pending,
            },
        })
    }
}

/// Handoff block: the request and the first witness. `witness` is the same
/// object every diagnostics snapshot carries under `t34Modes.witness`.
pub fn handoff(selection: &T34Selection, witness: Value) -> Value {
    json!({"schema":1,"requested":selection.requested(),"startup":witness})
}

/// Per-snapshot witness. Index publication evidence is the engine's own
/// `PublicationStatus` (already under `searchIndex.publication`); the harness
/// reads journal counters from there. Guard snapshot and write-behind are
/// reported unsupported until production exposes a public witness.
pub fn witness(engine: &mw_engine::Engine) -> Value {
    let publication = engine.index_publication_status();
    json!({
        "schema": 1,
        "guardSnapshot": {"supported": false, "reason": "no public fixture witness in mw-server"},
        "indexPublication": {
            "supported": true,
            "ackMode": publication.ack_mode.as_str(),
            "earlyAck": publication.early_ack,
        },
        "requestProtection": {"supported": false, "reason": "no public fixture witness in mw-server"},
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(pairs: &[(&str, &str)]) -> Result<T34Selection, String> {
        T34Selection::parse(|name| pairs.iter().find(|(k, _)| *k == name).map(|(_, v)| (*v).to_owned()))
    }
    #[test]
    fn unset_is_all_off_and_options_are_defaults() {
        let selection = parse(&[]).unwrap();
        assert_eq!(selection.requested()["indexPublication"]["ack"], "commit");
        assert_eq!(selection.publication_options().unwrap().ack, mw_engine::PublicationAck::Commit);
    }
    #[test]
    fn invalid_values_are_refused_with_the_variable_name() {
        for (name, value) in [
            ("MW_GUARD_SNAPSHOT", "on"),
            ("MW_GUARD_SNAPSHOT_MAX_STALENESS_MS", "99"),
            ("MW_INDEX_PUBLICATION_ACK", "Journal"),
            ("MW_INDEX_PUBLICATION_WINDOW_MS", "11"),
            ("MW_INDEX_READ_FENCE_TIMEOUT_MS", "30001"),
            ("MW_REQUEST_PROTECTION_ACCOUNTING", "async"),
            ("MW_REQUEST_PROTECTION_FLUSH_MS", "49"),
            ("MW_REQUEST_PROTECTION_MAX_PENDING", "63"),
        ] {
            let error = parse(&[(name, value)]).unwrap_err();
            assert!(error.contains(name), "{error}");
        }
    }
    #[test]
    fn journal_selection_reaches_the_publication_options() {
        let selection = parse(&[("MW_INDEX_PUBLICATION_ACK", "journal"), ("MW_INDEX_READ_FENCE_TIMEOUT_MS", "250")]).unwrap();
        let options = selection.publication_options().unwrap();
        assert_eq!(options.ack, mw_engine::PublicationAck::Journal);
        assert_eq!(options.read_fence_timeout, Duration::from_millis(250));
    }
}
