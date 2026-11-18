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
    buckets: VecDeque<(i64, BTreeMap<String, (u64, u64, u64)>)>,
}
static WRITES: LazyLock<Mutex<Writes>> = LazyLock::new(|| Mutex::new(Writes::default()));

pub(crate) fn record_write(sql: &str, origin: &Location<'_>, args: &[Arg], rows: u64) {
    let Some(label) = write_label(sql, origin) else {
        return;
    };
    let bytes: u64 = args
        .iter()
        .map(|arg| match arg {
            Arg::Text(v) => v.len() as u64,
            Arg::OptText(v) => v.as_ref().map_or(0, |v| v.len() as u64),
            Arg::Blob(v) => v.len() as u64,
            Arg::OptBlob(v) => v.as_ref().map_or(0, |v| v.len() as u64),
            Arg::Int(_) | Arg::OptInt(_) => 8,
        })
        .sum();
    let minute = chrono::Utc::now().timestamp() / 60;
    let mut writes = WRITES.lock().unwrap_or_else(|e| e.into_inner());
    if writes.buckets.back().map(|b| b.0) != Some(minute) {
        writes.buckets.push_back((minute, BTreeMap::new()));
    }
    while writes.buckets.front().is_some_and(|b| b.0 < minute - 14) {
        writes.buckets.pop_front();
    }
    let bucket = &mut writes.buckets.back_mut().unwrap().1;
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
    let writes = WRITES.lock().unwrap_or_else(|e| e.into_inner());
    let mut totals: BTreeMap<String, (u64, u64, u64)> = BTreeMap::new();
    for (_, bucket) in writes.buckets.iter().filter(|b| b.0 >= now / 60 - 14) {
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
#[derive(Default)]
struct Growth {
    samples: VecDeque<(i64, u64, u64, u64)>,
    rapid: u32,
}
static GROWTH: LazyLock<Mutex<BTreeMap<uuid::Uuid, Growth>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));
/// File allocation and WAL are sampled separately: checkpointing is not database growth.
pub(crate) fn observe(
    id: uuid::Uuid,
    db: Option<u64>,
    wal: Option<u64>,
    occupied: Option<u64>,
    threshold: u64,
) -> Value {
    observe_at(
        id,
        db,
        wal,
        occupied,
        threshold,
        chrono::Utc::now().timestamp(),
    )
}
fn observe_at(
    id: uuid::Uuid,
    db: Option<u64>,
    wal: Option<u64>,
    occupied: Option<u64>,
    threshold: u64,
    now: i64,
) -> Value {
    let mut all = GROWTH.lock().unwrap_or_else(|e| e.into_inner());
    if all.len() >= 32 && !all.contains_key(&id) {
        all.pop_first();
    }
    let growth = all.entry(id).or_default();
    if let Some(db) = db {
        if growth.samples.back().is_none_or(|s| now - s.0 >= 30) {
            if let Some(previous) = growth.samples.back() {
                let rate = db
                    .saturating_sub(previous.1)
                    .max(occupied.unwrap_or(db).saturating_sub(previous.3))
                    as f64
                    * 60.0
                    / (now - previous.0).max(1) as f64;
                growth.rapid = if rate >= threshold as f64 {
                    growth.rapid.saturating_add(1)
                } else {
                    0
                };
            }
            growth
                .samples
                .push_back((now, db, wal.unwrap_or(0), occupied.unwrap_or(db)));
            while growth.samples.len() > 720 {
                growth.samples.pop_front();
            }
        }
    }
    let rate = growth.samples.iter().rev().nth(1).and_then(|a| {
        growth
            .samples
            .back()
            .map(|b| (b.3 as f64 - a.3 as f64) * 60.0 / (b.0 - a.0).max(1) as f64)
    });
    json!({"scope":"process","windowMinutes":15,"thresholdBytesPerMinute":threshold,
        "status":if growth.rapid>=2{"rapid-growth"}else if growth.samples.len()<2{"warming-up"}else{"normal"},
        "bytesPerMinute":rate,"samples":growth.samples.iter().map(|(at,db,wal,occupied)|json!({"at":at,"dbBytes":db,"walBytes":wal,"occupiedBytes":occupied})).collect::<Vec<_>>(),
        "writeOrigins":write_snapshot(now)})
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sustained_growth_and_recovery_need_real_samples() {
        let id = uuid::Uuid::new_v4();
        assert_eq!(
            observe_at(id, Some(0), Some(0), None, 100, 1000)["status"],
            "warming-up"
        );
        assert_eq!(
            observe_at(id, Some(200), Some(0), None, 100, 1060)["status"],
            "normal"
        );
        assert_eq!(
            observe_at(id, Some(400), Some(0), None, 100, 1120)["status"],
            "rapid-growth"
        );
        assert_eq!(
            observe_at(id, Some(400), Some(9999), None, 100, 1180)["status"],
            "normal"
        );
        assert_eq!(
            observe_at(id, Some(400), Some(0), None, 100, 1181)["samples"]
                .as_array()
                .unwrap()
                .len(),
            4
        );
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
static TABLES: LazyLock<Mutex<BTreeMap<uuid::Uuid, BTreeMap<String, (String, u64)>>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));
/// Compare explicit inspections, not estimates from SQL parameter lengths.
pub(crate) fn observe_tables(id: uuid::Uuid, details: &mut Value) {
    let at = details["observedAt"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    let mut all = TABLES.lock().unwrap_or_else(|e| e.into_inner());
    if all.len() >= 32 && !all.contains_key(&id) {
        all.pop_first();
    }
    let previous = all.entry(id).or_default();
    if let Some(tables) = details["tables"].as_array_mut() {
        for table in tables {
            if let (Some(name), Some(bytes)) = (table["name"].as_str(), table["bytes"].as_u64()) {
                let name = name.to_owned();
                if let Some((since, before)) = previous.get(&name) {
                    table["growthBytes"] = json!(bytes as i128 - *before as i128);
                    table["growthSince"] = json!(since);
                }
                previous.insert(name, (at.clone(), bytes));
            }
        }
    }
}
