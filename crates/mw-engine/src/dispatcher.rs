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

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use chrono::Utc;
use mw_store::SubmissionRow;

use crate::backend::{EngineError, Result};
use crate::change::{ChangeOp, ChangeType};
use crate::engine::Engine;

/// How often the dispatcher wakes to scan for due work.
const TICK: Duration = Duration::from_millis(500);

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
        self.fire_due_submissions().await?;
        self.resurface_due_snoozes().await?;
        Ok(())
    }

    async fn fire_due_submissions(&self) -> Result<()> {
        let now = Utc::now();
        let pending = self.store().pending_submissions().await?;
        for row in pending {
            if !submission_is_due(&row, now) {
                continue;
            }
            let Some(rt) = self.runtime(&row.account_id) else {
                // Account not connected right now — retry on a later tick.
                continue;
            };
            // Re-read + re-check atomically: a cancel that landed since the scan
            // must win (source of truth is the row, plan risk #5).
            match self.store().get_submission(&row.id).await? {
                Some(fresh) if fresh.undo_status == "pending" => {}
                _ => continue,
            }
            match self.submit_email(&row.account_id, &rt, &row.email_id).await {
                Ok(()) => {
                    self.store().set_submission_status(&row.id, "final").await?;
                    self.record_change(
                        &row.account_id,
                        ChangeType::EmailSubmission,
                        &row.id,
                        ChangeOp::Updated,
                    )
                    .await?;
                }
                Err(EngineError::Unsupported(_)) => {
                    // Nothing to send for this backend — mark final, don't loop.
                    self.store().set_submission_status(&row.id, "final").await?;
                }
                Err(e) => {
                    tracing::warn!("submission {} dispatch failed: {e}", row.id);
                    // Leave pending; a transient failure retries next tick.
                }
            }
            self.broadcast_state(&row.account_id).await;
        }
        Ok(())
    }

    async fn resurface_due_snoozes(&self) -> Result<()> {
        let now = Utc::now().to_rfc3339();
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
fn submission_is_due(row: &SubmissionRow, now: chrono::DateTime<Utc>) -> bool {
    if let Some(send_at) = &row.send_at
        && let Ok(dt) = chrono::DateTime::parse_from_rfc3339(send_at)
    {
        return now >= dt.with_timezone(&Utc);
    }
    // Hold window measured from creation.
    if let Ok(created) = chrono::DateTime::parse_from_rfc3339(&row.created_at) {
        let fire =
            created.with_timezone(&Utc) + chrono::Duration::seconds(i64::from(row.hold_seconds));
        return now >= fire;
    }
    // Unparseable timestamps: fire now rather than wedge the row forever.
    true
}
