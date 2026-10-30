//! In-process sync and send telemetry for the admin Health panel (t103).
//!
//! Recorded where the work happens ([`crate::engine::Engine::resync_runtime`]
//! for a sync, the submission paths for a send), read by
//! [`crate::engine::Engine::sync_health`]. Memory is bounded: one entry per
//! account that has synced in this process, and a ring of error and send
//! timestamps kept for one hour and capped at [`MAX_EVENTS`].
//!
//! Nothing here touches the store or an upstream, so a read is a lock and a
//! copy.

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// How long error and send events are kept.
pub const EVENT_WINDOW: Duration = Duration::from_secs(3600);
/// The most events of one kind kept inside [`EVENT_WINDOW`].
pub const MAX_EVENTS: usize = 4096;
/// The longest error text kept for one account.
const MAX_ERROR_CHARS: usize = 240;

#[derive(Default)]
pub(crate) struct SyncHealth {
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    accounts: HashMap<String, AccountSync>,
    sync_errors: VecDeque<Instant>,
    sends: VecDeque<(Instant, SendOutcome)>,
    last_send: Option<(Instant, SendOutcome, Option<String>)>,
}

#[derive(Default, Clone)]
struct AccountSync {
    running: u32,
    last_ok: Option<Instant>,
    last_duration: Option<Duration>,
    last_error: Option<(Instant, String)>,
    consecutive_failures: u32,
}

/// What one submission attempt ended in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SendOutcome {
    /// SMTP accepted the message.
    Sent,
    /// Not accepted; it will be tried again later.
    Deferred,
    /// Not accepted and not retried.
    Failed,
    /// The connection dropped after the data was sent: delivery is unknown.
    Uncertain,
}

/// One registered account's sync state as the engine sees it now.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSyncSnapshot {
    pub account_id: String,
    /// A resync is running for this account right now.
    pub syncing: bool,
    /// The change-watch loop (IMAP IDLE / POP3 poll / bridge watch) is held.
    pub watching: bool,
    /// The account is served by a plugin bridge.
    pub plugin_backed: bool,
    /// `syncing`, `ready` or `failed`: the first full sync after registration.
    pub initial_sync: &'static str,
    /// Seconds since the last resync that completed, in this process.
    pub last_ok_age_secs: Option<f64>,
    /// How long that resync took, in milliseconds.
    pub last_duration_ms: Option<f64>,
    /// Seconds since the last failed resync, when it failed after the last success.
    pub last_error_age_secs: Option<f64>,
    pub last_error: Option<String>,
    /// Failed resyncs since the last one that completed.
    pub consecutive_failures: u32,
}

impl AccountSyncSnapshot {
    /// `syncing`, `erroring` or `idle`.
    pub fn state(&self) -> &'static str {
        if self.consecutive_failures > 0 || self.initial_sync == "failed" {
            "erroring"
        } else if self.syncing || self.initial_sync == "syncing" {
            "syncing"
        } else {
            "idle"
        }
    }
}

/// Engine-wide sync and send telemetry at one instant.
#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncHealthSnapshot {
    pub accounts: Vec<AccountSyncSnapshot>,
    /// Resyncs running now, all accounts.
    pub resyncs_running: u64,
    /// Failed resyncs in the last hour, all accounts.
    pub sync_errors_last_hour: u64,
    /// Submission attempts in the last hour by outcome.
    pub sends_last_hour: SendCounts,
    pub last_send: Option<LastSend>,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendCounts {
    pub sent: u64,
    pub deferred: u64,
    pub failed: u64,
    pub uncertain: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LastSend {
    pub outcome: SendOutcome,
    pub age_secs: f64,
    pub error: Option<String>,
}

fn clip(text: &str) -> String {
    text.chars().take(MAX_ERROR_CHARS).collect()
}

fn prune<T>(ring: &mut VecDeque<T>, now: Instant, at: impl Fn(&T) -> Instant) {
    while ring
        .front()
        .is_some_and(|e| now.duration_since(at(e)) > EVENT_WINDOW)
        || ring.len() > MAX_EVENTS
    {
        ring.pop_front();
    }
}

/// Marks one resync as running until it is dropped or finished.
pub(crate) struct SyncGuard<'a> {
    health: &'a SyncHealth,
    account: String,
    started: Instant,
    done: bool,
}

impl SyncGuard<'_> {
    pub(crate) fn finish<T, E: std::fmt::Display>(mut self, result: &Result<T, E>) {
        self.done = true;
        let now = Instant::now();
        let mut inner = self.health.lock();
        let inner = &mut *inner;
        let entry = inner.accounts.entry(self.account.clone()).or_default();
        entry.running = entry.running.saturating_sub(1);
        match result {
            Ok(_) => {
                entry.last_ok = Some(now);
                entry.last_duration = Some(now.duration_since(self.started));
                entry.consecutive_failures = 0;
                entry.last_error = None;
            }
            Err(e) => {
                entry.last_error = Some((now, clip(&e.to_string())));
                entry.consecutive_failures = entry.consecutive_failures.saturating_add(1);
                inner.sync_errors.push_back(now);
                prune(&mut inner.sync_errors, now, |t| *t);
            }
        }
    }
}

impl Drop for SyncGuard<'_> {
    fn drop(&mut self) {
        if !self.done {
            // Cancelled before it finished: no longer running, nothing learned.
            let mut inner = self.health.lock();
            if let Some(entry) = inner.accounts.get_mut(&self.account) {
                entry.running = entry.running.saturating_sub(1);
            }
        }
    }
}

impl SyncHealth {
    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub(crate) fn begin(&self, account: &str) -> SyncGuard<'_> {
        self.lock()
            .accounts
            .entry(account.to_string())
            .or_default()
            .running += 1;
        SyncGuard {
            health: self,
            account: account.to_string(),
            started: Instant::now(),
            done: false,
        }
    }

    pub(crate) fn record_send(&self, outcome: SendOutcome, error: Option<&str>) {
        let now = Instant::now();
        let mut inner = self.lock();
        inner.sends.push_back((now, outcome));
        prune(&mut inner.sends, now, |(t, _)| *t);
        inner.last_send = Some((now, outcome, error.map(clip)));
    }

    /// Forget an account that is no longer registered.
    pub(crate) fn forget(&self, account: &str) {
        self.lock().accounts.remove(account);
    }

    pub(crate) fn snapshot(
        &self,
        registered: &[(String, bool, bool, &'static str)],
    ) -> SyncHealthSnapshot {
        let now = Instant::now();
        let mut inner = self.lock();
        prune(&mut inner.sync_errors, now, |t| *t);
        prune(&mut inner.sends, now, |(t, _)| *t);
        let age = |t: Instant| now.duration_since(t).as_secs_f64();
        let mut running = 0u64;
        let accounts = registered
            .iter()
            .map(|(id, watching, plugin_backed, initial)| {
                let entry = inner.accounts.get(id).cloned().unwrap_or_default();
                running += u64::from(entry.running);
                AccountSyncSnapshot {
                    account_id: id.clone(),
                    syncing: entry.running > 0,
                    watching: *watching,
                    plugin_backed: *plugin_backed,
                    initial_sync: initial,
                    last_ok_age_secs: entry.last_ok.map(age),
                    last_duration_ms: entry.last_duration.map(|d| d.as_secs_f64() * 1000.0),
                    last_error_age_secs: entry.last_error.as_ref().map(|(t, _)| age(*t)),
                    last_error: entry.last_error.map(|(_, e)| e),
                    consecutive_failures: entry.consecutive_failures,
                }
            })
            .collect();
        let mut sends = SendCounts::default();
        for (_, outcome) in &inner.sends {
            match outcome {
                SendOutcome::Sent => sends.sent += 1,
                SendOutcome::Deferred => sends.deferred += 1,
                SendOutcome::Failed => sends.failed += 1,
                SendOutcome::Uncertain => sends.uncertain += 1,
            }
        }
        SyncHealthSnapshot {
            accounts,
            resyncs_running: running,
            sync_errors_last_hour: inner.sync_errors.len() as u64,
            sends_last_hour: sends,
            last_send: inner
                .last_send
                .as_ref()
                .map(|(t, outcome, error)| LastSend {
                    outcome: *outcome,
                    age_secs: age(*t),
                    error: error.clone(),
                }),
        }
    }
}

/// Record the outcome of an immediate `EmailSubmission/set` send. That path
/// does not retry: a refusal is final, or uncertain when the connection
/// dropped after the data was sent.
pub(crate) fn record_inline_send<T>(
    health: &SyncHealth,
    sent: &Result<T, crate::jmap::NotSent>,
) {
    match sent {
        Ok(_) => health.record_send(SendOutcome::Sent, None),
        Err(not_sent) => health.record_send(
            if not_sent.uncertain() {
                SendOutcome::Uncertain
            } else {
                SendOutcome::Failed
            },
            Some(&not_sent.error.to_string()),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_resync_is_running_until_finished_and_failures_count_until_a_success() {
        let health = SyncHealth::default();
        let reg = [("a".to_string(), true, false, "ready")];
        let guard = health.begin("a");
        let snap = health.snapshot(&reg);
        assert!(snap.accounts[0].syncing);
        assert_eq!(snap.resyncs_running, 1);
        assert_eq!(snap.accounts[0].state(), "syncing");
        guard.finish::<(), _>(&Err("connection refused"));
        let snap = health.snapshot(&reg);
        assert_eq!(snap.resyncs_running, 0);
        assert_eq!(snap.sync_errors_last_hour, 1);
        assert_eq!(snap.accounts[0].consecutive_failures, 1);
        assert_eq!(snap.accounts[0].state(), "erroring");
        assert_eq!(
            snap.accounts[0].last_error.as_deref(),
            Some("connection refused")
        );
        health.begin("a").finish::<(), String>(&Ok(()));
        let snap = health.snapshot(&reg);
        assert_eq!(snap.accounts[0].state(), "idle");
        assert!(snap.accounts[0].last_ok_age_secs.is_some());
        assert!(snap.accounts[0].last_error.is_none());
        assert_eq!(snap.sync_errors_last_hour, 1, "the hour still holds the error");
    }

    #[test]
    fn a_dropped_guard_is_no_longer_running_and_records_nothing() {
        let health = SyncHealth::default();
        drop(health.begin("a"));
        let snap = health.snapshot(&[("a".into(), false, false, "ready")]);
        assert!(!snap.accounts[0].syncing);
        assert!(snap.accounts[0].last_ok_age_secs.is_none());
        assert_eq!(snap.sync_errors_last_hour, 0);
    }

    #[test]
    fn sends_are_counted_by_outcome_and_the_last_one_kept() {
        let health = SyncHealth::default();
        health.record_send(SendOutcome::Sent, None);
        health.record_send(SendOutcome::Deferred, Some("451 try later"));
        let snap = health.snapshot(&[]);
        assert_eq!(snap.sends_last_hour.sent, 1);
        assert_eq!(snap.sends_last_hour.deferred, 1);
        let last = snap.last_send.unwrap();
        assert_eq!(last.outcome, SendOutcome::Deferred);
        assert_eq!(last.error.as_deref(), Some("451 try later"));
    }
}
