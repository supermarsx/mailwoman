//! Adaptive concurrency for the request pool (`MW_HTTP_ADAPTIVE_CONCURRENCY=on`,
//! off by default).
//!
//! Signal: in-server time of a request, from admission to its response headers
//! (response bodies are not timed, so a slow reader does not count). Requests whose
//! time does not track server load are left out by the caller: downloads, uploads,
//! exports, imports, previews, the image proxy, DAV feeds, mail refresh, WebSocket
//! upgrades, event streams and health probes (see `http_admission::classify`).
//!
//! Rule (AIMD over windows of [`WINDOW`] samples):
//! * baseline = the lowest median of the last [`BASELINE_WINDOWS`] lightly loaded
//!   windows (peak in flight at most half the limit; the first window always
//!   counts): the time of a request that did not queue. It follows a lasting change
//!   once load is light again; under sustained saturation it is not updated, so the
//!   limit cannot ratchet itself up;
//! * congested = window p90 > [`TOLERANCE`] × baseline and more than
//!   [`SLACK`] above it; then limit × [`DECREASE`];
//! * otherwise, when the window's peak in-flight count reached [`DEMAND`] of the
//!   limit, limit + [`INCREASE`] of it (at least 1); with less demand it stays.
//!
//! The limit stays between `floor` and `ceiling`; the ceiling is the configured
//! request limit, and the caller enforces the limit by withholding semaphore
//! permits, so no more than the ceiling are ever admitted.
use std::{collections::VecDeque, time::Duration};

pub const WINDOW: usize = 64;
pub const BASELINE_WINDOWS: usize = 32;
pub const TOLERANCE: f64 = 2.0;
pub const SLACK: Duration = Duration::from_millis(2);
pub const DECREASE: f64 = 0.9;
pub const INCREASE: f64 = 0.05;
pub const DEMAND: f64 = 0.8;

#[derive(Debug)]
pub struct Limiter {
    floor: usize,
    ceiling: usize,
    limit: usize,
    window: Vec<u64>,
    peak_inflight: usize,
    medians: VecDeque<u64>,
    decreases: u64,
    increases: u64,
}

impl Limiter {
    /// Starts at the ceiling: turning the limiter on changes nothing until
    /// latency says so.
    pub fn new(floor: usize, ceiling: usize) -> Self {
        let ceiling = ceiling.max(1);
        let floor = floor.clamp(1, ceiling);
        Self {
            floor,
            ceiling,
            limit: ceiling,
            window: Vec::with_capacity(WINDOW),
            peak_inflight: 0,
            medians: VecDeque::with_capacity(BASELINE_WINDOWS),
            decreases: 0,
            increases: 0,
        }
    }
    pub fn limit(&self) -> usize {
        self.limit
    }
    pub fn floor(&self) -> usize {
        self.floor
    }
    pub fn ceiling(&self) -> usize {
        self.ceiling
    }
    pub fn decreases(&self) -> u64 {
        self.decreases
    }
    pub fn increases(&self) -> u64 {
        self.increases
    }
    pub fn baseline(&self) -> Option<Duration> {
        self.medians
            .iter()
            .min()
            .map(|&us| Duration::from_micros(us))
    }

    /// Record one request: its in-server time and how many requests were in
    /// flight when it was admitted (itself included). Returns the new limit when a
    /// window closed and the limit changed.
    pub fn sample(&mut self, latency: Duration, inflight: usize) -> Option<usize> {
        self.window
            .push(latency.as_micros().min(u128::from(u64::MAX)) as u64);
        self.peak_inflight = self.peak_inflight.max(inflight);
        if self.window.len() < WINDOW {
            return None;
        }
        self.window.sort_unstable();
        let median = self.window[WINDOW / 2];
        let p90 = self.window[WINDOW * 9 / 10];
        let peak = std::mem::take(&mut self.peak_inflight);
        self.window.clear();
        // Only windows with at most half the limit in flight say what an unqueued
        // request costs. Taking every window would let the medians of a saturated
        // server become the baseline, and the limit would ratchet up to the ceiling.
        if self.medians.is_empty() || peak.saturating_mul(2) <= self.limit {
            if self.medians.len() == BASELINE_WINDOWS {
                self.medians.pop_front();
            }
            self.medians.push_back(median);
        }
        let baseline = *self.medians.iter().min().unwrap_or(&median);
        let slack = SLACK.as_micros() as u64;
        let before = self.limit;
        if p90 as f64 > baseline as f64 * TOLERANCE && p90 > baseline + slack {
            self.limit = ((self.limit as f64 * DECREASE) as usize).max(self.floor);
            if self.limit < before {
                self.decreases += 1;
            }
        } else if peak as f64 >= self.limit as f64 * DEMAND {
            let step = ((self.limit as f64 * INCREASE) as usize).max(1);
            self.limit = (self.limit + step).min(self.ceiling);
            if self.limit > before {
                self.increases += 1;
            }
        }
        (self.limit != before).then_some(self.limit)
    }
}

#[cfg(test)]
#[path = "http_adaptive_tests.rs"]
mod tests;
