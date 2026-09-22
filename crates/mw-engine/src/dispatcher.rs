//! The delayed dispatcher (plan §1.3, §3 e9): one background task that fires
//! due submissions (undo-send / send-later) and resurfaces due snoozes.
//!
//! Undo-send is a persisted queue, not a synchronous send: `EmailSubmission/set`
//! with a hold window (or a future `sendAt`) enqueues a `pending` row; this task
//! re-reads each row and checks `undoStatus` **atomically before dialing SMTP**
//! (plan risk #5), so a cancel that lands inside the window wins the race. A
//! submission with no hold and no future `sendAt` is fired inline at enqueue
//! time (see `jmap.rs::submission_set`) so the V1 synchronous send shape is
//! preserved. The same loop folds in the snooze resurface scheduler (§1.5).
//!
//! ## Submission states (26.20 t24-e7, B3)
//!
//! ```text
//!             SMTP accepted (recorded BEFORE the Sent copy is filed)
//!   pending ───────────────────────────────────────────────────────▶ final
//!    │  ▲                                                               (terminal)
//!    │  └── nothing accepted, transient, attempts < MAX: back off
//!    ├───── nothing accepted, permanent, or attempts reach MAX ──────▶ failed
//!    └───── user cancel (compare-and-set from pending) ───────────────▶ canceled
//! ```
//!
//! Only `pending` rows are ever handed to SMTP, and a row leaves `pending` for
//! `final` as soon as SMTP accepts its message. Filing the Sent copy happens
//! after that and cannot move the row back, so a filing failure is not a reason
//! to send again. `failed` is surfaced to JMAP clients as `canceled` plus
//! `mailwomanFailed: true`.
//!
//! Retries of a submission SMTP did not accept are spaced by [`retry_delay`],
//! which is unrelated to the scan interval [`TICK`], and end at
//! [`MAX_SUBMISSION_ATTEMPTS`].

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use chrono::{DateTime, Utc};
use mw_store::SubmissionRow;

use crate::backend::Result;
use crate::change::{ChangeOp, ChangeType};
use crate::engine::Engine;
use crate::jmap::{SUBMISSION_FAILED, SUBMISSION_PENDING, accepted_unrecorded};

/// How often the dispatcher wakes to scan for due work. This is a scan
/// interval, not a retry interval: see [`retry_delay`].
const TICK: Duration = Duration::from_millis(500);

/// Dispatch attempts in which SMTP accepted nothing before a submission is
/// marked `failed`. With [`retry_delay`] the last attempt happens about an hour
/// after the first.
pub const MAX_SUBMISSION_ATTEMPTS: u32 = 8;

/// The first retry delay; each later one doubles, up to [`RETRY_DELAY_CAP`].
const RETRY_DELAY_BASE: chrono::Duration = chrono::Duration::seconds(30);
/// The longest wait between two attempts.
const RETRY_DELAY_CAP: chrono::Duration = chrono::Duration::minutes(30);

/// How long to wait after the `attempts`-th failed attempt (1-based) before the
/// next one: 30 s, 1 min, 2, 4, 8, 16, then 30 min.
pub fn retry_delay(attempts: u32) -> chrono::Duration {
    let doublings = attempts.saturating_sub(1).min(16);
    RETRY_DELAY_BASE
        .checked_mul(1 << doublings)
        .map_or(RETRY_DELAY_CAP, |d| d.min(RETRY_DELAY_CAP))
}

impl Engine {
    /// Start the single dispatcher task (idempotent). Called from `start_watch`
    /// for the real server path, and directly by tests that exercise the queue.
    pub fn start_dispatcher(self: &Arc<Self>) {
        if self.dispatcher_started().swap(true, Ordering::SeqCst) {
            return; // already running
        }
        let engine = Arc::clone(self);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(TICK).await;
                if let Err(e) = engine.dispatch_tick().await {
                    tracing::warn!("dispatcher tick failed: {e}");
                }
            }
        });
    }

    /// One dispatcher pass: fire due submissions, then resurface due snoozes.
    /// Public so a test can drive the queue deterministically without waiting on
    /// the timer.
    pub async fn dispatch_tick(&self) -> Result<()> {
        self.dispatch_tick_at(Utc::now()).await
    }

    /// [`Engine::dispatch_tick`] as if the clock read `now`, so a test can step
    /// past a retry delay without sleeping through it.
    pub async fn dispatch_tick_at(&self, now: DateTime<Utc>) -> Result<()> {
        self.fire_due_submissions(now).await?;
        self.resurface_due_snoozes(now).await?;
        Ok(())
    }

    async fn fire_due_submissions(&self, now: DateTime<Utc>) -> Result<()> {
        let pending = self.store().pending_submissions().await?;
        for row in pending {
            if !submission_is_due(&row, now) {
                continue;
            }
            // SMTP accepted this one earlier in this process but the `final`
            // write failed. Retry the write, never the send.
            if accepted_unrecorded()
                .lock()
                .expect("accepted-unrecorded lock")
                .contains(&row.id)
            {
                if self.store().mark_submission_sent(&row.id).await.is_ok() {
                    accepted_unrecorded()
                        .lock()
                        .expect("accepted-unrecorded lock")
                        .remove(&row.id);
                    self.record_change(
                        &row.account_id,
                        ChangeType::EmailSubmission,
                        &row.id,
                        ChangeOp::Updated,
                    )
                    .await?;
                    self.broadcast_state(&row.account_id).await;
                }
                continue;
            }
            let Some(rt) = self.runtime(&row.account_id) else {
                // Account not connected right now — retry on a later tick.
                continue;
            };
            // Re-read + re-check atomically: a cancel that landed since the scan
            // must win (source of truth is the row, plan risk #5).
            match self.store().get_submission(&row.id).await? {
                Some(fresh) if fresh.undo_status == SUBMISSION_PENDING => {}
                _ => continue,
            }
            // Honour the backoff from an earlier failed attempt.
            let attempts = self
                .store()
                .get_submission_attempts(&row.id)
                .await?
                .unwrap_or_default();
            if let Some(next) = attempts
                .next_attempt_at
                .as_deref()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                && now < next.with_timezone(&Utc)
            {
                continue;
            }

            match self
                .send_submission(&row.id, &row.account_id, &rt, &row.email_id)
                .await
            {
                // Delivered: `send_submission` has recorded `final` and the change.
                Ok(()) => {}
                Err(not_sent) => {
                    let attempt = attempts.attempts.saturating_add(1);
                    let give_up = not_sent.permanent || attempt >= MAX_SUBMISSION_ATTEMPTS;
                    let (status, next_at) = if give_up {
                        (SUBMISSION_FAILED, None)
                    } else {
                        (
                            SUBMISSION_PENDING,
                            Some((now + retry_delay(attempt)).to_rfc3339()),
                        )
                    };
                    tracing::warn!(
                        "submission {} not sent (attempt {attempt} of {MAX_SUBMISSION_ATTEMPTS}{}): {}",
                        row.id,
                        if give_up { ", giving up" } else { "" },
                        not_sent.error
                    );
                    let recorded = self
                        .store()
                        .record_submission_failure(
                            &row.id,
                            &not_sent.error.to_string(),
                            next_at.as_deref(),
                            status,
                        )
                        .await?;
                    if recorded {
                        self.record_change(
                            &row.account_id,
                            ChangeType::EmailSubmission,
                            &row.id,
                            ChangeOp::Updated,
                        )
                        .await?;
                    }
                }
            }
            self.broadcast_state(&row.account_id).await;
        }
        Ok(())
    }

    async fn resurface_due_snoozes(&self, now: DateTime<Utc>) -> Result<()> {
        let now = now.to_rfc3339();
        let due = self.store().due_snoozed(&now).await?;
        for d in due {
            // Clear the snooze; the message re-enters the normal inbox view. The
            // Email state advances so the next `Email/changes` surfaces it and a
            // push nudges the client.
            let meta = self.store().get_message_meta(&d.stable_id).await?;
            let mut meta = meta.unwrap_or_default();
            meta.snoozed_until = None;
            self.store()
                .upsert_message_meta(&d.stable_id, &meta)
                .await?;
            self.record_change(
                &d.account_id,
                ChangeType::Email,
                &d.stable_id,
                ChangeOp::Updated,
            )
            .await?;
            self.search().reload().ok();
            self.broadcast_state(&d.account_id).await;
        }
        Ok(())
    }
}

/// Whether a pending submission's fire time has arrived: `sendAt` when set,
/// else `createdAt + holdSeconds` (the undo-send window).
fn submission_is_due(row: &SubmissionRow, now: DateTime<Utc>) -> bool {
    if let Some(send_at) = &row.send_at
        && let Ok(dt) = DateTime::parse_from_rfc3339(send_at)
    {
        return now >= dt.with_timezone(&Utc);
    }
    // Hold window measured from creation.
    if let Ok(created) = DateTime::parse_from_rfc3339(&row.created_at) {
        let fire =
            created.with_timezone(&Utc) + chrono::Duration::seconds(i64::from(row.hold_seconds));
        return now >= fire;
    }
    // Unparseable timestamps: fire now rather than wedge the row forever.
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_delay_doubles_then_caps() {
        let secs: Vec<i64> = (1..=MAX_SUBMISSION_ATTEMPTS)
            .map(|n| retry_delay(n).num_seconds())
            .collect();
        assert_eq!(secs, [30, 60, 120, 240, 480, 960, 1800, 1800]);
        // Far beyond the cap, and at the degenerate 0, it stays bounded.
        assert_eq!(retry_delay(u32::MAX), RETRY_DELAY_CAP);
        assert_eq!(retry_delay(0), RETRY_DELAY_BASE);
        // Every retry waits much longer than one scan.
        assert!(retry_delay(1).to_std().unwrap() > TICK * 10);
    }
}
