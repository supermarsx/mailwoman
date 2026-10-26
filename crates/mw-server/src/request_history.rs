//! Bounded durable response-ready metrics, isolated per application instance.
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
    request_metrics::{Bucket, Counts},
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
    pub(crate) fn record(&self, status: u16, ms: u64) {
        let time = now();
        let mut memory = self
            .memory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        memory.touch(time);
        for seconds in [60, 3600] {
            if let Some(bucket) = memory.buckets.get_mut(&(seconds, time / seconds * seconds)) {
                bucket.counts.record(status, ms);
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
        let mut grouped: BTreeMap<i64, (Counts, Vec<(i64, i64)>)> = BTreeMap::new();
        for row in rows {
            let t = from + (row.start - from) / width * width;
            let entry = grouped.entry(t).or_default();
            entry.0.merge(&row.counts);
            let interval = (row.observed_from.max(from), row.observed_to.min(to));
            if interval.1 > interval.0 {
                entry.1.push(interval);
                observed.push(interval);
            }
        }
        let mut t = from;
        while t < to {
            let end = (t + width).min(to);
            let (counts, intervals) = grouped.remove(&t).unwrap_or_default();
            let covered = union(&intervals).iter().map(|(a, b)| b - a).sum::<i64>();
            let coverage = if covered == 0 {
                "unavailable"
            } else if covered == end - t {
                "complete"
            } else {
                "partial"
            };
            series.push(json!({"t":t,"requests":(covered>0).then_some(counts.requests),"errors5xx":(covered>0).then_some(counts.errors5xx),"p50Ms":(covered>0).then(||counts.percentile(50)).flatten(),"p95Ms":(covered>0).then(||counts.percentile(95)).flatten(),"coverage":coverage}));
            t = end;
        }
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
        Ok::<_,AdminApiError>(([(header::CACHE_CONTROL,"no-store")],Json(json!({"from":from,"to":to,"bucketSeconds":width,"series":series,"coverage":{"observedSeconds":seconds,"requestedSeconds":to-from,"gaps":gaps},"policy":policy,"scope":"server","timing":"response-ready"}))).into_response())
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
