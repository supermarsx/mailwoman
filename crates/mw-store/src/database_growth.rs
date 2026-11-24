//! Bounded, payload-free process telemetry. It never writes its own samples to SQL.
use crate::backend::Arg;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, VecDeque},
    panic::Location,
    sync::{LazyLock, Mutex},
};

#[derive(Default)]
struct Writes {
    buckets: VecDeque<(i64, BTreeMap<String, (u64, u64, u64)>, [(u64, u64, u64); 4])>,
}
static WRITES: LazyLock<Mutex<Writes>> = LazyLock::new(|| Mutex::new(Writes::default()));

/// Count payload bytes before a backend consumes its owned arguments. As before,
/// optional integers count their fixed-width slot even when SQL NULL is bound.
pub(crate) fn input_bytes(args: &[Arg]) -> u64 {
    args.iter()
        .map(|arg| match arg {
            Arg::Text(v) => v.len() as u64,
            Arg::OptText(v) => v.as_ref().map_or(0, |v| v.len() as u64),
            Arg::Blob(v) => v.len() as u64,
            Arg::OptBlob(v) => v.as_ref().map_or(0, |v| v.len() as u64),
            Arg::Int(_) | Arg::OptInt(_) => 8,
        })
        .sum()
}

pub(crate) fn record_write_bytes(sql: &str, origin: &Location<'_>, bytes: u64, rows: u64) {
    let Some(label) = write_label(sql, origin) else {
        return;
    };
    let minute = chrono::Utc::now().timestamp() / 60;
    WRITES
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .record(label, minute, bytes, rows);
}
impl Writes {
    fn record(&mut self, label: String, minute: i64, bytes: u64, rows: u64) {
        let operation = match label.split_whitespace().next() {
            Some("INSERT") => 0,
            Some("UPDATE") => 1,
            Some("DELETE") => 2,
            Some("REPLACE") => 3,
            _ => return,
        };
        if self.buckets.back().map(|b| b.0) != Some(minute) {
            self.buckets
                .push_back((minute, BTreeMap::new(), [(0, 0, 0); 4]));
        }
        while self.buckets.len() > 15 || self.buckets.front().is_some_and(|b| b.0 < minute - 14) {
            self.buckets.pop_front();
        }
        let current = self.buckets.back_mut().unwrap();
        let totals = &mut current.2[operation];
        totals.0 = totals.0.saturating_add(1);
        totals.1 = totals.1.saturating_add(rows);
        totals.2 = totals.2.saturating_add(bytes);
        let bucket = &mut current.1;
        let label = if bucket.len() >= 128 && !bucket.contains_key(&label) {
            "other".into()
        } else {
            label
        };
        let counters = bucket.entry(label).or_default();
        counters.0 = counters.0.saturating_add(1);
        counters.1 = counters.1.saturating_add(rows);
        counters.2 = counters.2.saturating_add(bytes);
    }
    fn snapshot(&self, now: i64) -> Value {
        let mut totals: BTreeMap<String, (u64, u64, u64)> = BTreeMap::new();
        for (_, bucket, _) in self
            .buckets
            .iter()
            .filter(|b| b.0 >= now / 60 - 14 && b.0 <= now / 60)
        {
            for (key, v) in bucket {
                let t = totals.entry(key.clone()).or_default();
                t.0 += v.0;
                t.1 += v.1;
                t.2 += v.2;
            }
        }
        let mut origins: Vec<_> = totals.into_iter().collect();
        origins.sort_by_key(|v| std::cmp::Reverse(v.1.2));
        json!(origins.into_iter().take(128).map(|(origin,(statements,rows,payload))|json!({"origin":origin,"statements":statements,"affectedRows":rows,"inputBytes":payload})).collect::<Vec<_>>())
    }
}
fn write_label(sql: &str, origin: &Location<'_>) -> Option<String> {
    let mut words = sql.split_whitespace();
    let op = words.next()?.to_ascii_uppercase();
    let table = match op.as_str() {
        "INSERT" | "REPLACE" => {
            let mut token = words.next()?;
            for _ in 0..4 {
                if token.eq_ignore_ascii_case("INTO") {
                    break;
                }
                token = words.next()?;
            }
            words.next()?
        }
        "UPDATE" => words.next()?,
        "DELETE" => {
            if !words.next()?.eq_ignore_ascii_case("FROM") {
                return None;
            }
            words.next()?
        }
        _ => return None,
    };
    let table = table.split('(').next()?.trim_matches(['"', '`', '[', ']']);
    if !table.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return None;
    }
    let source = origin.file().rsplit(['/', '\\']).next().unwrap_or("store");
    Some(format!("{op} {table} · {source}:{}", origin.line()))
}
fn write_snapshot(now: i64) -> Value {
    WRITES
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .snapshot(now)
}
fn operation_snapshot(now: i64) -> Value {
    let writes = WRITES.lock().unwrap_or_else(|e| e.into_inner());
    let mut totals = [(0u64, 0u64, 0u64); 4];
    for (_, _, bucket) in writes
        .buckets
        .iter()
        .filter(|b| b.0 >= now / 60 - 14 && b.0 <= now / 60)
    {
        for (total, next) in totals.iter_mut().zip(bucket) {
            total.0 = total.0.saturating_add(next.0);
            total.1 = total.1.saturating_add(next.1);
            total.2 = total.2.saturating_add(next.2);
        }
    }
    json!(["INSERT","UPDATE","DELETE","REPLACE"].into_iter().zip(totals).map(|(operation,(statements,affected_rows,input_bytes))|
        json!({"operation":operation,"statements":statements,"affectedRows":affected_rows,"inputBytes":input_bytes})).collect::<Vec<_>>())
}
/// Confirmed successful maintenance receipts, not estimated disk reclamation.
#[derive(Clone, Copy, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Cleanup {
    pub removed_bodies: u64,
    pub removed_body_bytes: u64,
    pub cleaned_envelopes: u64,
    pub removed_envelope_bytes: u64,
    pub removed_change_rows: u64,
    pub reclaimed_pages: u64,
    pub retired_file_bytes: u64,
}
impl Cleanup {
    fn add(&mut self, other: Self) {
        self.removed_bodies = self.removed_bodies.saturating_add(other.removed_bodies);
        self.removed_body_bytes = self
            .removed_body_bytes
            .saturating_add(other.removed_body_bytes);
        self.cleaned_envelopes = self
            .cleaned_envelopes
            .saturating_add(other.cleaned_envelopes);
        self.removed_envelope_bytes = self
            .removed_envelope_bytes
            .saturating_add(other.removed_envelope_bytes);
        self.removed_change_rows = self
            .removed_change_rows
            .saturating_add(other.removed_change_rows);
        self.reclaimed_pages = self.reclaimed_pages.saturating_add(other.reclaimed_pages);
        self.retired_file_bytes = self
            .retired_file_bytes
            .saturating_add(other.retired_file_bytes);
    }
    fn since(self, previous: Self) -> Option<Self> {
        Some(Self {
            removed_bodies: self.removed_bodies.checked_sub(previous.removed_bodies)?,
            removed_body_bytes: self
                .removed_body_bytes
                .checked_sub(previous.removed_body_bytes)?,
            cleaned_envelopes: self
                .cleaned_envelopes
                .checked_sub(previous.cleaned_envelopes)?,
            removed_envelope_bytes: self
                .removed_envelope_bytes
                .checked_sub(previous.removed_envelope_bytes)?,
            removed_change_rows: self
                .removed_change_rows
                .checked_sub(previous.removed_change_rows)?,
            reclaimed_pages: self.reclaimed_pages.checked_sub(previous.reclaimed_pages)?,
            retired_file_bytes: self
                .retired_file_bytes
                .checked_sub(previous.retired_file_bytes)?,
        })
    }
}
#[derive(Clone)]
struct Sample {
    at: i64,
    db: Option<u64>,
    wal: Option<u64>,
    occupied: Option<u64>,
    free: Option<u64>,
    cleanup: Cleanup,
    segment: u64,
    break_reason: Option<&'static str>,
}
#[derive(Default)]
struct Growth {
    samples: VecDeque<Sample>,
    generation: String,
    cleanup: Cleanup,
    rapid: u32,
    segment: u64,
}
static GROWTH: LazyLock<Mutex<BTreeMap<uuid::Uuid, Growth>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));
fn bounded_growth(all: &mut BTreeMap<uuid::Uuid, Growth>, id: uuid::Uuid) -> &mut Growth {
    if all.len() >= 32 && !all.contains_key(&id) {
        all.pop_first();
    }
    all.entry(id).or_default()
}
pub(crate) fn record_cleanup(id: uuid::Uuid, receipt: Cleanup) {
    let mut all = GROWTH.lock().unwrap_or_else(|e| e.into_inner());
    bounded_growth(&mut all, id).cleanup.add(receipt);
}
/// File allocation and WAL are sampled separately: checkpointing is not database growth.
pub(crate) fn observe(
    id: uuid::Uuid,
    db: Option<u64>,
    wal: Option<u64>,
    occupied: Option<u64>,
    free: Option<u64>,
    generation: &str,
    threshold: u64,
) -> Value {
    observe_at(
        id,
        db,
        wal,
        occupied,
        free,
        generation,
        threshold,
        chrono::Utc::now().timestamp(),
    )
}
fn observe_at(
    id: uuid::Uuid,
    db: Option<u64>,
    wal: Option<u64>,
    occupied: Option<u64>,
    free: Option<u64>,
    generation: &str,
    threshold: u64,
    now: i64,
) -> Value {
    let mut all = GROWTH.lock().unwrap_or_else(|e| e.into_inner());
    let growth = bounded_growth(&mut all, id);
    // Never join unlike generations/measurement bases or a backwards clock.
    // Missing occupied data stays unknown; file allocation is a separate basis.
    let break_reason = if growth.samples.is_empty() {
        Some("baseline")
    } else if growth.generation != generation {
        Some("generation")
    } else if growth.samples.back().is_some_and(|s| now < s.at) {
        Some("clock")
    } else if growth
        .samples
        .back()
        .is_some_and(|s| s.occupied.is_some() != occupied.is_some())
    {
        Some("basis")
    } else {
        None
    };
    if break_reason.is_some() {
        growth.segment = growth.segment.saturating_add(1);
        growth.rapid = 0;
        growth.generation = generation.to_owned();
    }
    if db.is_some() || occupied.is_some() {
        if break_reason.is_some() || growth.samples.back().is_none_or(|s| now - s.at >= 30) {
            if let Some(previous) = growth
                .samples
                .back()
                .filter(|s| s.segment == growth.segment)
            {
                let rate = delta(occupied.or(db), previous.occupied.or(previous.db))
                    .map(|bytes| bytes * 60.0 / (now - previous.at) as f64);
                growth.rapid = if rate.is_some_and(|rate| rate >= threshold as f64) {
                    growth.rapid.saturating_add(1)
                } else {
                    0
                };
            }
            growth.samples.push_back(Sample {
                at: now,
                db,
                wal,
                occupied,
                free,
                cleanup: growth.cleanup,
                segment: growth.segment,
                break_reason,
            });
            while growth.samples.len() > 720 {
                growth.samples.pop_front();
            }
        }
    }
    let interval = growth
        .samples
        .iter()
        .rev()
        .nth(1)
        .zip(growth.samples.back())
        .filter(|(a, b)| {
            a.segment == b.segment
                && b.segment == growth.segment
                && (db.is_some() || occupied.is_some())
        });
    let changes = interval.map(|(a, b)| {
        json!({"from":a.at,"to":b.at,"seconds":b.at-a.at,
        "occupiedBytes":delta(b.occupied,a.occupied),"dbBytes":delta(b.db,a.db),
        "walBytes":delta(b.wal,a.wal),"freeBytes":delta(b.free,a.free),
        "knownCleanup":b.cleanup.since(a.cleanup)})
    });
    let rate = interval.and_then(|(a, b)| {
        delta(b.occupied.or(b.db), a.occupied.or(a.db))
            .map(|bytes| bytes * 60.0 / (b.at - a.at) as f64)
    });
    if db.is_none() && occupied.is_none() {
        growth.rapid = 0;
    }
    json!({"scope":"process","windowMinutes":15,"thresholdBytesPerMinute":threshold,
        "basis":if occupied.is_some(){"occupied-pages"}else{"database-allocation"},"changes":changes,
        "knownCleanupTotal":growth.cleanup,
        "status":if growth.rapid>=2{"rapid-growth"}else if interval.is_none(){"warming-up"}else{"normal"},
        "bytesPerMinute":rate,"samples":growth.samples.iter().map(|s|json!({"at":s.at,"dbBytes":s.db,"walBytes":s.wal,"occupiedBytes":s.occupied,"freeBytes":s.free,"segment":s.segment,"breakReason":s.break_reason})).collect::<Vec<_>>(),
        "writeOrigins":write_snapshot(now),"writeOperations":operation_snapshot(now)})
}
fn delta(after: Option<u64>, before: Option<u64>) -> Option<f64> {
    after
        .zip(before)
        .map(|(a, b)| (i128::from(a) - i128::from(b)) as f64)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sustained_growth_and_recovery_need_real_samples() {
        let id = uuid::Uuid::new_v4();
        assert_eq!(
            observe_at(id, Some(0), Some(0), None, None, "a", 100, 1000)["status"],
            "warming-up"
        );
        assert_eq!(
            observe_at(id, Some(200), Some(0), None, None, "a", 100, 1060)["status"],
            "normal"
        );
        assert_eq!(
            observe_at(id, Some(400), Some(0), None, None, "a", 100, 1120)["status"],
            "rapid-growth"
        );
        assert_eq!(
            observe_at(id, Some(400), Some(9999), None, None, "a", 100, 1180)["status"],
            "normal"
        );
        assert_eq!(
            observe_at(id, Some(400), Some(0), None, None, "a", 100, 1181)["samples"]
                .as_array()
                .unwrap()
                .len(),
            4
        );
    }
    #[test]
    fn cleanup_and_net_page_reduction_do_not_claim_file_shrink() {
        let id = uuid::Uuid::new_v4();
        observe_at(
            id,
            Some(1000),
            Some(40),
            Some(900),
            Some(100),
            "a",
            100,
            1000,
        );
        record_cleanup(
            id,
            Cleanup {
                removed_bodies: 2,
                removed_body_bytes: 700,
                removed_change_rows: 50,
                ..Default::default()
            },
        );
        let next = observe_at(
            id,
            Some(1000),
            Some(200),
            Some(500),
            Some(500),
            "a",
            100,
            1060,
        );
        assert_eq!(next["bytesPerMinute"], -400.0);
        assert_eq!(next["changes"]["dbBytes"], 0.0);
        assert_eq!(next["changes"]["freeBytes"], 400.0);
        assert_eq!(next["changes"]["knownCleanup"]["removedBodyBytes"], 700);
        assert_eq!(next["changes"]["knownCleanup"]["removedChangeRows"], 50);
        assert_eq!(next["status"], "normal");
        // Inserts consume reusable pages: net growth is200, not submitted input
        // bytes minus receipt bytes; the physical file still need not grow.
        let reused = observe_at(
            id,
            Some(1000),
            Some(0),
            Some(700),
            Some(300),
            "a",
            100,
            1120,
        );
        assert_eq!(reused["bytesPerMinute"], 200.0);
        assert_eq!(reused["changes"]["freeBytes"], -200.0);
        assert_eq!(reused["changes"]["knownCleanup"]["removedBodyBytes"], 0);
    }
    #[test]
    fn allocation_growth_during_cleanup_and_checkpoint_cannot_raise_page_alert() {
        let id = uuid::Uuid::new_v4();
        for (at, db, wal, occupied) in [
            (1000, 1000, 5000, 900),
            (1060, 2000, 3000, 800),
            (1120, 3000, 0, 700),
        ] {
            let out = observe_at(
                id,
                Some(db),
                Some(wal),
                Some(occupied),
                Some(db - occupied),
                "a",
                1,
                at,
            );
            assert_ne!(out["status"], "rapid-growth");
            if at > 1000 {
                assert_eq!(out["bytesPerMinute"], -100.0);
                assert_eq!(out["changes"]["dbBytes"], 1000.0);
            }
        }
    }
    #[test]
    fn generation_basis_and_clock_breaks_keep_history_without_cross_break_rates() {
        let id = uuid::Uuid::new_v4();
        observe_at(
            id,
            Some(1000),
            Some(0),
            Some(900),
            Some(100),
            "old",
            1,
            1000,
        );
        let converted = observe_at(id, Some(200), Some(0), Some(150), Some(50), "new", 1, 1060);
        assert!(converted["bytesPerMinute"].is_null());
        assert!(converted["changes"].is_null());
        assert_eq!(converted["samples"].as_array().unwrap().len(), 2);
        assert_eq!(converted["samples"][1]["breakReason"], "generation");
        let missing = observe_at(id, Some(200), None, None, None, "new", 1, 1120);
        assert!(missing["bytesPerMinute"].is_null());
        assert!(missing["samples"][2]["occupiedBytes"].is_null());
        assert_eq!(missing["samples"][2]["breakReason"], "basis");
        let backwards = observe_at(id, Some(200), None, None, None, "new", 1, 1100);
        assert!(backwards["bytesPerMinute"].is_null());
        assert_eq!(backwards["samples"][3]["breakReason"], "clock");
        for at in 0..800 {
            observe_at(id, Some(200), None, None, None, "new", 1, 2000 + at * 60);
        }
        assert_eq!(GROWTH.lock().unwrap().get(&id).unwrap().samples.len(), 720);
    }
    #[test]
    fn unknowns_counter_reset_and_retired_file_receipts_are_separate() {
        let id = uuid::Uuid::new_v4();
        record_cleanup(
            id,
            Cleanup {
                retired_file_bytes: 9000,
                removed_body_bytes: 70,
                ..Default::default()
            },
        );
        let first = observe_at(id, Some(1000), None, Some(900), Some(100), "a", 1, 1000);
        assert!(first["changes"].is_null());
        record_cleanup(
            id,
            Cleanup {
                retired_file_bytes: 5000,
                ..Default::default()
            },
        );
        let retired = observe_at(id, Some(1000), None, Some(900), Some(100), "a", 1, 1060);
        assert_eq!(retired["changes"]["knownCleanup"]["retiredFileBytes"], 5000);
        assert_eq!(retired["changes"]["occupiedBytes"], 0.0);
        assert!(retired["changes"]["walBytes"].is_null());
        GROWTH.lock().unwrap().get_mut(&id).unwrap().cleanup = Cleanup::default();
        let reset = observe_at(id, Some(1000), None, Some(800), Some(200), "a", 1, 1120);
        assert!(reset["changes"]["knownCleanup"].is_null());
        assert_eq!(reset["bytesPerMinute"], -100.0);
        let unavailable = observe_at(id, None, None, None, None, "a", 1, 1180);
        assert!(unavailable["bytesPerMinute"].is_null());
    }
    #[test]
    fn delete_counters_track_rows_not_bytes_reclaimed() {
        let before = operation_snapshot(chrono::Utc::now().timestamp());
        record_write_bytes(
            "DELETE FROM cleanup_fixture WHERE at<?1",
            Location::caller(),
            8,
            200,
        );
        let after = operation_snapshot(chrono::Utc::now().timestamp());
        assert!(
            after[2]["statements"].as_u64().unwrap()
                >= before[2]["statements"].as_u64().unwrap() + 1
        );
        assert!(
            after[2]["affectedRows"].as_u64().unwrap()
                >= before[2]["affectedRows"].as_u64().unwrap() + 200
        );
        assert!(
            after[2]["inputBytes"].as_u64().unwrap()
                >= before[2]["inputBytes"].as_u64().unwrap() + 8
        );
        assert!(after[2].get("removedBytes").is_none());
    }
    #[test]
    fn table_removal_delta_is_signed_and_never_compares_unlike_generations() {
        let id = uuid::Uuid::new_v4();
        let detail = |at: &str, bytes: u64| json!({"observedAt":at,"tables":[{"name":"bodies","bytes":bytes}]});
        let mut first = detail("a", 1000);
        observe_tables(id, "old", &mut first);
        let mut cleaned = detail("b", 700);
        observe_tables(id, "old", &mut cleaned);
        assert_eq!(cleaned["tables"][0]["growthBytes"], -300);
        let mut converted = detail("c", 200);
        observe_tables(id, "new", &mut converted);
        assert!(converted["tables"][0].get("growthBytes").is_none());
        let mut next = detail("d", 150);
        observe_tables(id, "new", &mut next);
        assert_eq!(next["tables"][0]["growthBytes"], -50);
    }
    #[test]
    fn losing_allocation_only_measurement_clears_stale_alert_without_inventing_zero() {
        let id = uuid::Uuid::new_v4();
        observe_at(id, Some(0), None, None, None, "a", 1, 1000);
        observe_at(id, Some(200), None, None, None, "a", 1, 1060);
        assert_eq!(
            observe_at(id, Some(400), None, None, None, "a", 1, 1120)["status"],
            "rapid-growth"
        );
        let missing = observe_at(id, None, None, None, None, "a", 1, 1180);
        assert_eq!(missing["status"], "warming-up");
        assert!(missing["bytesPerMinute"].is_null());
        assert_eq!(missing["samples"].as_array().unwrap().len(), 3);
        assert_eq!(
            observe_at(id, Some(500), None, None, None, "a", 1, 1240)["status"],
            "normal"
        );
    }
    #[test]
    fn input_bytes_preserves_utf8_blob_null_and_integer_accounting() {
        let args = vec![
            Arg::Text("é\0".into()),
            Arg::OptText(Some("界".into())),
            Arg::OptText(None),
            Arg::Blob(vec![0, 255]),
            Arg::OptBlob(Some(vec![0, 1, 2, 3])),
            Arg::OptBlob(None),
            Arg::Int(i64::MIN),
            Arg::OptInt(Some(i64::MAX)),
            Arg::OptInt(None),
        ];
        assert_eq!(input_bytes(&args), 3 + 3 + 2 + 4 + 3 * 8);
        assert_eq!(input_bytes(&[]), 0);
    }

    #[test]
    fn precomputed_write_bytes_reach_origin_counters() {
        // The process-wide counters are shared with every other test that
        // writes, and keep only 128 origins per minute and per snapshot, so
        // this test counts into its own table of writes.
        let origin = Location::caller();
        let sql = "INSERT INTO growth_byte_fixture(value) VALUES (?1)";
        let label = write_label(sql, origin).unwrap();
        let args = vec![Arg::Blob(vec![0, 255, 7]), Arg::OptInt(None)];
        let bytes = input_bytes(&args);
        drop(args);
        let now = chrono::Utc::now().timestamp();
        let mut writes = Writes::default();
        writes.record(label.clone(), now / 60, bytes, 2);
        let after = writes.snapshot(now);
        let rows = after.as_array().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["origin"], label);
        assert_eq!(rows[0]["statements"], 1);
        assert_eq!(rows[0]["affectedRows"], 2);
        assert_eq!(rows[0]["inputBytes"], 11);
        // The process-wide entry point accepts the same precomputed bytes.
        record_write_bytes(sql, origin, bytes, 2);
    }

    #[test]
    fn origins_never_include_values_or_sql() {
        let label = write_label(
            "INSERT INTO bodies(blob_ref,sealed_bytes) VALUES (?1,?2)",
            Location::caller(),
        )
        .unwrap();
        assert!(label.starts_with("INSERT bodies · database_growth.rs:"));
        assert!(!label.contains("VALUES"));
        assert!(write_label("SELECT secret FROM bodies", Location::caller()).is_none());
    }
}
#[derive(Default)]
struct Tables {
    generation: String,
    tables: BTreeMap<String, (String, u64)>,
}
static TABLES: LazyLock<Mutex<BTreeMap<uuid::Uuid, Tables>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));
/// Compare explicit inspections, not estimates from SQL parameter lengths.
pub(crate) fn observe_tables(id: uuid::Uuid, generation: &str, details: &mut Value) {
    let at = details["observedAt"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    let mut all = TABLES.lock().unwrap_or_else(|e| e.into_inner());
    if all.len() >= 32 && !all.contains_key(&id) {
        all.pop_first();
    }
    let previous = all.entry(id).or_default();
    if previous.generation != generation {
        previous.tables.clear();
        previous.generation = generation.to_owned();
    }
    if let Some(tables) = details["tables"].as_array_mut() {
        for table in tables {
            if let (Some(name), Some(bytes)) = (table["name"].as_str(), table["bytes"].as_u64()) {
                let name = name.to_owned();
                if let Some((since, before)) = previous.tables.get(&name) {
                    table["growthBytes"] = json!(bytes as i128 - *before as i128);
                    table["growthSince"] = json!(since);
                }
                previous.tables.insert(name, (at.clone(), bytes));
            }
        }
    }
}
