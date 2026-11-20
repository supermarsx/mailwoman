//! Monotonic conversion telemetry; never used for copy or cutover decisions.
use crate::online_copy::CopyProgress;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ConversionTiming {
    pub finished_at: Option<i64>,
    pub elapsed_ms: Option<u64>,
    pub copy_elapsed_ms: Option<u64>,
    pub replay_elapsed_ms: Option<u64>,
    pub average_copy_bytes_per_second: Option<f64>,
    pub average_replay_keys_per_second: Option<f64>,
}
#[derive(Clone, Copy)]
pub(super) enum Stage {
    Idle,
    Copy,
    Replay,
}
pub(super) struct JobTiming {
    started: Instant,
    observed: Instant,
    stage: Stage,
    copy: Duration,
    replay: Duration,
    stopped: Option<(Duration, i64)>,
}
fn millis(value: Duration) -> u64 {
    value.as_millis().min(u64::MAX as u128) as u64
}
impl JobTiming {
    pub(super) fn new(at: Instant) -> Self {
        Self {
            started: at,
            observed: at,
            stage: Stage::Idle,
            copy: Duration::ZERO,
            replay: Duration::ZERO,
            stopped: None,
        }
    }
    pub(super) fn enter(&mut self, stage: Stage, at: Instant) {
        if self.stopped.is_some() {
            return;
        }
        let elapsed = at.saturating_duration_since(self.observed);
        match self.stage {
            Stage::Copy => self.copy += elapsed,
            Stage::Replay => self.replay += elapsed,
            Stage::Idle => {}
        }
        self.stage = stage;
        self.observed = at;
    }
    pub(super) fn finish(&mut self, at: Instant, wall_time: i64) {
        if self.stopped.is_some() {
            return;
        }
        self.enter(Stage::Idle, at);
        self.stopped = Some((at.saturating_duration_since(self.started), wall_time));
    }
    pub(super) fn snapshot(
        &self,
        at: Instant,
        progress: Option<&CopyProgress>,
    ) -> ConversionTiming {
        let active = if self.stopped.is_none() {
            at.saturating_duration_since(self.observed)
        } else {
            Duration::ZERO
        };
        let copy = self.copy
            + if matches!(self.stage, Stage::Copy) {
                active
            } else {
                Duration::ZERO
            };
        let replay = self.replay
            + if matches!(self.stage, Stage::Replay) {
                active
            } else {
                Duration::ZERO
            };
        let elapsed = self
            .stopped
            .map(|value| value.0)
            .unwrap_or_else(|| at.saturating_duration_since(self.started));
        ConversionTiming {
            finished_at: self.stopped.map(|value| value.1),
            elapsed_ms: Some(millis(elapsed)),
            copy_elapsed_ms: Some(millis(copy)),
            replay_elapsed_ms: Some(millis(replay)),
            average_copy_bytes_per_second: progress
                .filter(|_| !copy.is_zero())
                .map(|value| value.copied_bytes as f64 / copy.as_secs_f64()),
            average_replay_keys_per_second: progress
                .filter(|_| !replay.is_zero())
                .map(|value| value.replayed_keys as f64 / replay.as_secs_f64()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn progress() -> CopyProgress {
        CopyProgress {
            table_index: 0,
            table_name: None,
            table_count: 1,
            copied_rows: 1,
            copied_bytes: 8_000_000,
            replayed_keys: 30,
            pending: 4,
            seed_complete: true,
            valid: true,
        }
    }
    #[test]
    fn measured_stage_rates_exclude_preparation_and_verification_and_freeze() {
        let start = Instant::now();
        let mut clock = JobTiming::new(start);
        assert!(
            clock
                .snapshot(start, None)
                .average_copy_bytes_per_second
                .is_none()
        );
        clock.enter(Stage::Copy, start + Duration::from_secs(10));
        clock.enter(Stage::Idle, start + Duration::from_secs(14));
        clock.enter(Stage::Replay, start + Duration::from_secs(30));
        clock.enter(Stage::Idle, start + Duration::from_secs(32));
        clock.enter(Stage::Replay, start + Duration::from_secs(40));
        clock.finish(start + Duration::from_secs(41), 1000);
        let finished = clock.snapshot(start + Duration::from_secs(400), Some(&progress()));
        assert_eq!(finished.elapsed_ms, Some(41000));
        assert_eq!(finished.finished_at, Some(1000));
        assert_eq!(finished.copy_elapsed_ms, Some(4000));
        assert_eq!(finished.replay_elapsed_ms, Some(3000));
        assert_eq!(finished.average_copy_bytes_per_second, Some(2_000_000.0));
        assert_eq!(finished.average_replay_keys_per_second, Some(10.0));
        clock.finish(start + Duration::from_secs(500), 5);
        clock.enter(Stage::Copy, start + Duration::from_secs(600));
        assert_eq!(
            serde_json::to_value(finished).unwrap(),
            serde_json::to_value(
                clock.snapshot(start + Duration::from_secs(700), Some(&progress()))
            )
            .unwrap()
        );
    }
    #[test]
    fn active_snapshots_use_monotonic_time_and_zero_denominators_are_unknown() {
        let start = Instant::now();
        let mut clock = JobTiming::new(start);
        let initial = clock.snapshot(start, Some(&progress()));
        assert_eq!(initial.elapsed_ms, Some(0));
        assert_eq!(initial.finished_at, None);
        assert_eq!(initial.average_copy_bytes_per_second, None);
        assert_eq!(initial.average_replay_keys_per_second, None);
        clock.enter(Stage::Copy, start + Duration::from_secs(2));
        assert_eq!(
            clock
                .snapshot(start + Duration::from_secs(4), Some(&progress()))
                .average_copy_bytes_per_second,
            Some(4_000_000.0)
        );
        assert_eq!(
            clock
                .snapshot(start + Duration::from_secs(5), None)
                .elapsed_ms,
            Some(5000)
        );
    }
    #[test]
    fn legacy_report_has_unknown_metrics_without_wall_clock_estimates() {
        let legacy = r#"{"id":"legacy","phase":"completed","startedAt":100,"updatedAt":200,"progress":null,"error":null,"retainedPrevious":true}"#;
        let report: crate::online_maintenance::ConversionReport =
            serde_json::from_str(legacy).unwrap();
        assert_eq!(report.timing.finished_at, None);
        assert_eq!(report.timing.elapsed_ms, None);
        assert_eq!(report.timing.average_copy_bytes_per_second, None);
        assert_eq!(report.timing.average_replay_keys_per_second, None);
        let encoded = serde_json::to_value(report).unwrap();
        assert!(encoded["finishedAt"].is_null());
        assert!(encoded["elapsedMs"].is_null());
        assert!(
            encoded.get("timing").is_none(),
            "metrics must be flattened for existing status clients"
        );
    }
}
