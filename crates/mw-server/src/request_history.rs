//! Bounded durable response-ready metrics, isolated per application instance.
//!
//! 404 answers are a series of their own in both tiers ([`not_found`] says which are
//! counted and how a path is redacted); the hourly tier also keeps the top
//! [`NOT_FOUND_PATHS_PER_BUCKET`] redacted `method + path` pairs. Snapshots written
//! before 404s were tracked carry no 404 series and are reported as "not tracked".
mod not_found;
use crate::{
    AppState,
    admin_ctx::{AdminApiError, AdminCtx},
    capabilities::Capability as C,
};
use axum::{
    Json,
    extract::{Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use mw_store::{
    Store,
    request_metrics::{Bucket, Counts, NOT_FOUND_PATHS_PER_BUCKET, NotFound},
};
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Duration,
};

struct Memory {
    touched: i64,
    buckets: BTreeMap<(i64, i64), Bucket>,
}
pub(crate) struct Recorder {
    instance: String,
    memory: Mutex<Memory>,
}
fn now() -> i64 {
    chrono::Utc::now().timestamp()
}
impl Memory {
    fn touch(&mut self, now: i64) {
        let previous = self.touched.min(now);
        for seconds in [60, 3600] {
            let oldest = now.saturating_sub(120 * 60);
            let mut start = previous.max(oldest) / seconds * seconds;
            while start <= now {
                let bucket = self
                    .buckets
                    .entry((seconds, start))
                    .or_insert_with(|| Bucket {
                        seconds,
                        start,
                        observed_from: previous.max(start),
                        observed_to: previous.max(start),
                        counts: Counts::default(),
                    });
                bucket.observed_to = now.min(start + seconds).max(bucket.observed_to);
                start += seconds;
            }
        }
        self.touched = now;
        self.buckets
            .retain(|(_, start), bucket| *start + bucket.seconds > now - 120 * 60);
    }
}
impl Recorder {
    pub(crate) fn start(store: Store) -> Arc<Self> {
        let recorder = Arc::new(Self {
            instance: uuid::Uuid::new_v4().to_string(),
            memory: Mutex::new(Memory {
                touched: now(),
                buckets: BTreeMap::new(),
            }),
        });
        let weak = Arc::downgrade(&recorder);
        tokio::spawn(async move {
            let mut timer = tokio::time::interval(Duration::from_secs(15));
            timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                timer.tick().await;
                let Some(recorder) = weak.upgrade() else {
                    break;
                };
                let (instance, buckets) = (recorder.instance.clone(), recorder.snapshot(now()));
                drop(recorder);
                let work = async {
                    let policy = Policy::load(&store).await?;
                    store
                        .save_request_metrics(&instance, &buckets, now(), policy.retention_days)
                        .await
                };
                match tokio::time::timeout(Duration::from_secs(5), work).await {
                    Ok(Ok(())) => {}
                    _ => tracing::warn!(
                        "request history flush unavailable; bounded snapshots retained for retry"
                    ),
                }
            }
        });
        recorder
    }
    fn snapshot(&self, time: i64) -> Vec<Bucket> {
        let mut memory = self
            .memory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        memory.touch(time);
        memory.buckets.values().cloned().collect()
    }
    /// Record one finished request. `path` is relative to the deploy prefix; it is
    /// read only for a counted 404, and only in its redacted form.
    pub(crate) fn record_response(
        &self,
        status: u16,
        micros: u64,
        method: &axum::http::Method,
        path: &str,
    ) {
        let missing = (status == 404)
            .then(|| not_found::counted(method, path))
            .flatten();
        self.record_at(
            now(),
            status,
            micros,
            missing.as_ref().map(|(m, p)| (*m, p.as_str())),
        );
    }
    fn record_at(&self, time: i64, status: u16, micros: u64, missing: Option<(&str, &str)>) {
        let mut memory = self
            .memory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        memory.touch(time);
        for (seconds, capacity) in [(60, 0), (3600, NOT_FOUND_PATHS_PER_BUCKET)] {
            if let Some(bucket) = memory.buckets.get_mut(&(seconds, time / seconds * seconds)) {
                bucket.counts.record_us(status, micros);
                if let Some((method, path)) = missing {
                    bucket.counts.record_not_found(method, path, capacity);
                }
            }
        }
    }
}
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Policy {
    default_range_seconds: i64,
    max_range_seconds: i64,
    retention_days: i64,
}
impl Policy {
    async fn load(store: &Store) -> Result<Self, mw_store::StoreError> {
        async fn read(
            store: &Store,
            key: &str,
            default: i64,
            min: i64,
            max: i64,
        ) -> Result<i64, mw_store::StoreError> {
            let value = store
                .get_setting_policy(key)
                .await?
                .and_then(|row| row.global_value);
            match value {
                None => Ok(default),
                Some(value) => serde_json::from_str::<i64>(&value)
                    .ok()
                    .filter(|n| (min..=max).contains(n))
                    .ok_or_else(|| {
                        mw_store::StoreError::Corrupt("invalid metrics history policy".into())
                    }),
            }
        }
        let retention_days = read(store, "metrics.rollupDays", 30, 1, 400).await?;
        let max_range_seconds = read(store, "metrics.maxRangeSeconds", 2592000, 3600, 34560000)
            .await?
            .min(retention_days * 86400);
        let default_range_seconds = read(store, "metrics.defaultRangeSeconds", 3600, 60, 34560000)
            .await?
            .min(max_range_seconds);
        Ok(Self {
            default_range_seconds,
            max_range_seconds,
            retention_days,
        })
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Range {
    from: Option<i64>,
    to: Option<i64>,
    max_points: Option<usize>,
}
pub(crate) async fn history(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Query(range): Query<Range>,
) -> Result<Response, AdminApiError> {
    if !ctx.has_unscoped(C::DashboardRead) && !ctx.has_unscoped(C::ObservabilityRead) {
        return Err(AdminApiError::unscoped_required(C::ObservabilityRead));
    }
    let work = async {
        let policy = Policy::load(&state.store).await?;
        let current = now();
        let (from, to, points, source) = validate(&range, &policy, current)?;
        let to = ((to + source - 1) / source * source).min(current);
        let from = from / source * source;
        let per_point = (to - from + points as i64 - 1) / (points as i64);
        let width = (per_point + source - 1) / source * source;
        let rows = state
            .store
            .request_metrics_history(from, to, source)
            .await?;
        let mut series = Vec::new();
        let mut observed = Vec::new();
        let (mut tracked, mut untracked) = (Vec::new(), Vec::new());
        let mut grouped: BTreeMap<i64, Group> = BTreeMap::new();
        for row in rows {
            let t = from + (row.start - from) / width * width;
            let entry = grouped.entry(t).or_default();
            entry.counts.merge(&row.counts);
            let interval = (row.observed_from.max(from), row.observed_to.min(to));
            if let Some(not_found) = &row.counts.not_found {
                entry.not_found = entry.not_found.saturating_add(not_found.count);
            }
            if interval.1 > interval.0 {
                entry.intervals.push(interval);
                observed.push(interval);
                if row.counts.not_found.is_some() {
                    entry.tracked.push(interval);
                    tracked.push(interval);
                } else {
                    entry.untracked = true;
                    untracked.push(interval);
                }
            }
        }
        let mut t = from;
        while t < to {
            let end = (t + width).min(to);
            let group = grouped.remove(&t).unwrap_or_default();
            let counts = &group.counts;
            let covered = union(&group.intervals)
                .iter()
                .map(|(a, b)| b - a)
                .sum::<i64>();
            let coverage = if covered == 0 {
                "unavailable"
            } else if covered == end - t {
                "complete"
            } else {
                "partial"
            };
            let nf_covered = union(&group.tracked)
                .iter()
                .map(|(a, b)| b - a)
                .sum::<i64>();
            // A replica still writing untracked snapshots makes the 404 count partial.
            let nf_coverage = if nf_covered == 0 {
                if covered > 0 {
                    "notTracked"
                } else {
                    "unavailable"
                }
            } else if nf_covered == end - t && !group.untracked {
                "complete"
            } else {
                "partial"
            };
            series.push(json!({"t":t,"requests":(covered>0).then_some(counts.requests),"errors5xx":(covered>0).then_some(counts.errors5xx),"notFound":(nf_covered>0).then_some(group.not_found),"notFoundCoverage":nf_coverage,"p50Ms":(covered>0).then(||counts.precise_percentile_ms(50)).flatten(),"p95Ms":(covered>0).then(||counts.precise_percentile_ms(95)).flatten(),"coverage":coverage,"legacyLatencyEstimate":counts.legacy_estimate(),"p50ResolutionMs":counts.precise_percentile_resolution_ms(50),"p95ResolutionMs":counts.precise_percentile_resolution_ms(95)}));
            t = end;
        }
        let not_found = not_found_coverage(&tracked, &untracked);
        let observed = union(&observed);
        let mut gaps = Vec::new();
        let mut cursor = from;
        for (a, b) in &observed {
            if *a > cursor {
                gaps.push(json!({"from":cursor,"to":a}));
            }
            cursor = cursor.max(*b);
        }
        if cursor < to {
            gaps.push(json!({"from":cursor,"to":to}));
        }
        let seconds = observed.iter().map(|(a, b)| b - a).sum::<i64>();
        Ok::<_,AdminApiError>(([(header::CACHE_CONTROL,"no-store")],Json(json!({"from":from,"to":to,"bucketSeconds":width,"series":series,"coverage":{"observedSeconds":seconds,"requestedSeconds":to-from,"gaps":gaps,"notFound":not_found},"policy":policy,"scope":"server","timing":"response-ready","latencySampleUnit":"microseconds"}))).into_response())
    };
    tokio::time::timeout(Duration::from_secs(5), work)
        .await
        .map_err(|_| {
            AdminApiError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "historyUnavailable",
                "Request history is temporarily unavailable.",
            )
        })?
}
#[derive(Default)]
struct Group {
    counts: Counts,
    intervals: Vec<(i64, i64)>,
    not_found: u64,
    tracked: Vec<(i64, i64)>,
    untracked: bool,
}
type Intervals = Vec<(i64, i64)>;
/// Where the 404 series exists in a range. `notTracked` lists observed periods whose
/// snapshots predate 404 tracking; `trackedSince` is the first tracked moment.
fn not_found_coverage(tracked: &[(i64, i64)], untracked: &[(i64, i64)]) -> serde_json::Value {
    let tracked = union(tracked);
    let untracked = union(untracked);
    json!({
        "observedSeconds": tracked.iter().map(|(a, b)| b - a).sum::<i64>(),
        "trackedSince": tracked.first().map(|(a, _)| *a),
        "notTracked": untracked.iter().map(|(a, b)| json!({"from": a, "to": b})).collect::<Vec<_>>(),
    })
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct PathRange {
    from: Option<i64>,
    to: Option<i64>,
}
/// `GET /admin/metrics/not-found` — the most frequent redacted not-found
/// `method + path` pairs over the hours that overlap the selected window. Needs
/// unscoped Observability read; carries no address, query string or identity.
pub(crate) async fn not_found_paths(
    State(state): State<AppState>,
    ctx: AdminCtx,
    Query(range): Query<PathRange>,
) -> Result<Response, AdminApiError> {
    if !ctx.has_unscoped(C::ObservabilityRead) {
        return Err(AdminApiError::unscoped_required(C::ObservabilityRead));
    }
    let work = async {
        let policy = Policy::load(&state.store).await?;
        let current = now();
        let range = Range {
            from: range.from,
            to: range.to,
            max_points: None,
        };
        let (from, to, _, _) = validate(&range, &policy, current)?;
        let hours_from = from / 3600 * 3600;
        let rows = state
            .store
            .request_metrics_history(hours_from, to, 3600)
            .await?;
        Ok::<_, AdminApiError>(
            (
                [(header::CACHE_CONTROL, "no-store")],
                Json(top_paths_body(&rows, from, hours_from, to)),
            )
                .into_response(),
        )
    };
    tokio::time::timeout(Duration::from_secs(5), work)
        .await
        .map_err(|_| {
            AdminApiError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "historyUnavailable",
                "Request history is temporarily unavailable.",
            )
        })?
}
fn top_paths_body(rows: &[Bucket], from: i64, hours_from: i64, to: i64) -> serde_json::Value {
    let mut summary = NotFound::default();
    let (mut tracked, mut untracked): (Intervals, Intervals) = (Vec::new(), Vec::new());
    for row in rows {
        let interval = (row.observed_from.max(hours_from), row.observed_to.min(to));
        match &row.counts.not_found {
            Some(not_found) => {
                summary.merge(not_found, NOT_FOUND_PATHS_PER_BUCKET);
                if interval.1 > interval.0 {
                    tracked.push(interval);
                }
            }
            None if interval.1 > interval.0 => untracked.push(interval),
            None => {}
        }
    }
    // t102: each listed pair says whether the app's own clients produce it.
    let (mut client, mut external) = (0_u64, 0_u64);
    let paths: Vec<_> = summary
        .paths
        .iter()
        .map(|entry| {
            let source = not_found::source(&entry.path);
            let floor = entry.count - entry.error;
            if source == "client" {
                client = client.saturating_add(floor);
            } else {
                external = external.saturating_add(floor);
            }
            json!({"method":entry.method,"path":entry.path,"count":entry.count,"minCount":floor,"source":source})
        })
        .collect();
    json!({
        "from": from, "to": to, "pathsFrom": hours_from, "pathsTo": to,
        "bucketSeconds": 3600, "limit": NOT_FOUND_PATHS_PER_BUCKET,
        "total": (!tracked.is_empty()).then_some(summary.count),
        "approximate": summary.paths.iter().any(|entry| entry.error > 0),
        "paths": paths, "coverage": not_found_coverage(&tracked, &untracked),
        "listedBySource": {"client": client, "external": external},
        "scope": "server",
    })
}
fn validate(
    range: &Range,
    policy: &Policy,
    current: i64,
) -> Result<(i64, i64, usize, i64), AdminApiError> {
    let (from, to) = match (range.from, range.to) {
        (None, None) => (current - policy.default_range_seconds, current),
        (Some(from), Some(to)) => (from, to.min(current)),
        _ => {
            return Err(AdminApiError::new(
                StatusCode::BAD_REQUEST,
                "invalidRange",
                "Supply both from and to, or neither.",
            ));
        }
    };
    let points = range.max_points.unwrap_or(240);
    if !(1..=1000).contains(&points)
        || from < 0
        || to <= from
        || to - from > policy.max_range_seconds
        || range.to.is_some_and(|to| to > current + 60)
        || from < current - policy.retention_days * 86400
    {
        return Err(AdminApiError::new(
            StatusCode::BAD_REQUEST,
            "invalidRange",
            "The requested range or point count exceeds the configured limits.",
        ));
    }
    Ok((
        from,
        to,
        points,
        if from < current - 48 * 3600 { 3600 } else { 60 },
    ))
}
fn union(intervals: &[(i64, i64)]) -> Vec<(i64, i64)> {
    let mut sorted = intervals.to_vec();
    sorted.sort_unstable();
    let mut merged: Vec<(i64, i64)> = Vec::new();
    for (a, b) in sorted {
        if let Some(last) = merged.last_mut()
            && a <= last.1
        {
            last.1 = last.1.max(b);
        } else {
            merged.push((a, b));
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn policy_is_live_and_retention_caps_requested_ranges() {
        let store = Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        for (key, value) in [
            ("metrics.rollupDays", "1"),
            ("metrics.defaultRangeSeconds", "172800"),
            ("metrics.maxRangeSeconds", "604800"),
        ] {
            store
                .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
                    key: key.into(),
                    global_value: Some(value.into()),
                    allow_user_override: false,
                    global_locked: false,
                    updated_by: "test".into(),
                    updated_at: 1,
                })
                .await
                .unwrap();
        }
        let policy = Policy::load(&store).await.unwrap();
        assert_eq!(policy.retention_days, 1);
        assert_eq!(policy.default_range_seconds, 86400);
        assert_eq!(policy.max_range_seconds, 86400);
    }
    #[test]
    fn coverage_merges_replicas_without_filling_restart_gaps() {
        assert_eq!(
            union(&[(0, 10), (5, 12), (20, 30), (20, 25)]),
            vec![(0, 12), (20, 30)]
        );
    }
    #[test]
    fn ranges_defaults_bounds_and_precision_are_explicit() {
        let policy = Policy {
            default_range_seconds: 3600,
            max_range_seconds: 30 * 86400,
            retention_days: 30,
        };
        let now = 40 * 86400;
        let range = Range {
            from: None,
            to: None,
            max_points: None,
        };
        assert_eq!(
            validate(&range, &policy, now).unwrap(),
            (now - 3600, now, 240, 60)
        );
        assert!(
            validate(
                &Range {
                    from: Some(now - 3600),
                    to: None,
                    max_points: None
                },
                &policy,
                now
            )
            .is_err()
        );
        assert!(
            validate(
                &Range {
                    from: Some(now - 3600),
                    to: Some(now + 61),
                    max_points: None
                },
                &policy,
                now
            )
            .is_err()
        );
        assert!(
            validate(
                &Range {
                    from: None,
                    to: None,
                    max_points: Some(1001)
                },
                &policy,
                now
            )
            .is_err()
        );
        assert_eq!(
            validate(
                &Range {
                    from: Some(now - 7 * 86400),
                    to: Some(now),
                    max_points: Some(100)
                },
                &policy,
                now
            )
            .unwrap()
            .3,
            3600
        );
    }
    #[test]
    fn snapshot_memory_is_bounded_and_idle_minutes_remain_observed() {
        let mut memory = Memory {
            touched: 3600,
            buckets: BTreeMap::new(),
        };
        memory.touch(3600 + 3 * 86400);
        assert!(memory.buckets.len() <= 125);
        assert!(
            memory
                .buckets
                .values()
                .all(|bucket| bucket.counts.requests == 0)
        );
        assert!(
            memory
                .buckets
                .values()
                .any(|bucket| bucket.observed_to > bucket.observed_from)
        );
    }
}

#[cfg(test)]
mod not_found_tests {
    use super::*;
    use axum::http::Method;
    fn recorder(touched: i64) -> Recorder {
        Recorder {
            instance: "test".into(),
            memory: Mutex::new(Memory {
                touched,
                buckets: BTreeMap::new(),
            }),
        }
    }
    fn record(recorder: &Recorder, time: i64, status: u16, method: Method, path: &str) {
        let missing = (status == 404)
            .then(|| not_found::counted(&method, path))
            .flatten();
        recorder.record_at(
            time,
            status,
            900,
            missing.as_ref().map(|(m, p)| (*m, p.as_str())),
        );
    }
    #[test]
    fn not_found_is_counted_in_minute_and_hour_tiers_without_expected_misses() {
        let hour = 3600 * 200_000;
        let recorder = recorder(hour);
        record(&recorder, hour + 5, 200, Method::GET, "/");
        record(
            &recorder,
            hour + 5,
            404,
            Method::GET,
            "/wp-login.php?u=admin",
        );
        record(&recorder, hour + 6, 404, Method::GET, "/favicon.ico");
        record(&recorder, hour + 6, 404, Method::GET, "/api/track/open");
        record(
            &recorder,
            hour + 70,
            404,
            Method::POST,
            "/u/alice@example.com",
        );
        record(&recorder, hour + 70, 403, Method::GET, "/denied");
        let snapshot = recorder.snapshot(hour + 80);
        let minute = |start: i64| {
            snapshot
                .iter()
                .find(|b| b.seconds == 60 && b.start == start)
                .unwrap()
        };
        let first = minute(hour);
        assert_eq!(
            first.counts.requests, 4,
            "expected misses are still requests"
        );
        let nf = first.counts.not_found.as_ref().unwrap();
        assert_eq!(nf.count, 1);
        assert!(nf.paths.is_empty());
        assert_eq!(
            minute(hour + 60).counts.not_found.as_ref().unwrap().count,
            1
        );
        let hourly = snapshot
            .iter()
            .find(|b| b.seconds == 3600 && b.start == hour)
            .unwrap();
        let nf = hourly.counts.not_found.as_ref().unwrap();
        assert_eq!(nf.count, 2);
        let mut keys: Vec<_> = nf
            .paths
            .iter()
            .map(|p| format!("{} {}", p.method, p.path))
            .collect();
        keys.sort();
        assert_eq!(keys, ["GET /wp-login.php", "POST /u/{email}"]);
        assert!(hourly.counts.valid());
    }
    #[test]
    fn hourly_top_paths_stay_bounded_under_ten_thousand_distinct_paths() {
        let hour = 3600 * 200_001;
        let recorder = recorder(hour);
        for index in 0..10_000 {
            record(
                &recorder,
                hour + 10,
                404,
                Method::GET,
                &format!("/probe-{index}/x"),
            );
        }
        let snapshot = recorder.snapshot(hour + 20);
        let hourly = snapshot.iter().find(|b| b.seconds == 3600).unwrap();
        let nf = hourly.counts.not_found.as_ref().unwrap();
        assert_eq!(nf.count, 10_000);
        assert_eq!(nf.paths.len(), NOT_FOUND_PATHS_PER_BUCKET);
        assert!(serde_json::to_string(&hourly.counts).unwrap().len() < 16 * 1024);
    }
    #[test]
    fn untracked_history_is_marked_not_tracked_never_zero() {
        let body = not_found_coverage(&[(500, 600)], &[(0, 400), (300, 500)]);
        assert_eq!(body["trackedSince"], 500);
        assert_eq!(body["observedSeconds"], 100);
        assert_eq!(body["notTracked"], json!([{"from":0,"to":500}]));
        let legacy: Counts = serde_json::from_value(json!({"requests":1,"errors5xx":0,"histogram":[1,0,0,0,0,0,0,0,0,0,0,0,0,0],"max_ms":0})).unwrap();
        let mut tracked = Counts::default();
        tracked.record_us(404, 10);
        tracked.record_not_found("GET", "/x", NOT_FOUND_PATHS_PER_BUCKET);
        let rows = [
            Bucket {
                seconds: 3600,
                start: 0,
                observed_from: 0,
                observed_to: 3600,
                counts: legacy,
            },
            Bucket {
                seconds: 3600,
                start: 3600,
                observed_from: 3600,
                observed_to: 7200,
                counts: tracked,
            },
        ];
        let body = top_paths_body(&rows, 100, 0, 7200);
        assert_eq!(body["total"], 1);
        assert_eq!(
            body["paths"],
            json!([{"method":"GET","path":"/x","count":1,"minCount":1,"source":"external"}])
        );
        assert_eq!(body["coverage"]["trackedSince"], 3600);
        assert_eq!(
            body["coverage"]["notTracked"],
            json!([{"from":0,"to":3600}])
        );
        assert_eq!(body["approximate"], false);
        let body = top_paths_body(&rows[..1], 100, 0, 3600);
        assert!(
            body["total"].is_null(),
            "no tracked hour: unknown, not zero"
        );
    }
}
