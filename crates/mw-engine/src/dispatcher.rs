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
//! ## Submission states (26.20 t24-e7, B3; manual hold t28-e12)
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
//! A `pending` row is durably claimed as `sending:<unique token>` before SMTP.
//! Only that claim owner records its result. An abandoned claim or an uncertain
//! final reply is never automatically retried. A confirmed acceptance becomes
//! `final` immediately. Filing the Sent copy happens
//! after that and cannot move the row back, so a filing failure is not a reason
//! to send again. `failed` is surfaced to JMAP clients as `canceled` plus
//! `mailwomanFailed: true`.
//!
//! A `pending` row is **due** at a time (`sendAt`, else `createdAt +
//! holdSeconds`) unless it is **held** (`hold` column not NULL, 0031): a held row
//! is never due, at any time. This task does not send it and nothing here can
//! clear the hold; `jmap.rs::release_submission` does, on an
//! `EmailSubmission/set` update, and then sends through
//! [`Engine::attempt_submission`] like this task. A held row can still be
//! canceled.
//!
//! ```text
//!   pending+held ── release (hold, sendAt, holdSeconds cleared) ──▶ pending, due now
//!        └───────── cancel ──────────────────────────────────────▶ canceled
//! ```
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
use crate::jmap::{Filed, SUBMISSION_FAILED, SUBMISSION_PENDING, claim_send};

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
        // Acceptance is already known in this process: retry only persistence,
        // even though the durable claim correctly excludes this row from pending.
        let accepted: Vec<String> = self.accepted_submissions.lock()
            .expect("accepted submissions lock").keys().cloned().collect();
        for id in accepted { self.attempt_submission(&id, now).await?; }
        let pending = self.store().pending_submissions().await?;
        for row in pending {
            // The scan's copy of the row carries no hold, so this is the time
            // half of the question only; `attempt_submission` asks the whole
            // of it against the row as it is then.
            if !submission_is_due(&row, None, now) {
                continue;
            }
            self.attempt_submission(&row.id, now).await?;
        }
        Ok(())
    }

    /// Make one attempt to send submission `id` as if the clock read `now`,
    /// if — read fresh from the store — it is `pending`, not held, due, not
    /// backing off, its account is connected, and no other task is sending it.
    /// Otherwise do nothing. `Ok(Some(_))` means SMTP accepted the message in
    /// this call and says what filing did; `Ok(None)` covers both "nothing was
    /// attempted" and "attempted, not accepted" — the row says which.
    ///
    /// An attempt SMTP does not accept is recorded here: the row backs off and
    /// stays `pending`, or becomes `failed` when retrying cannot help or the
    /// attempts are used up.
    ///
    /// The dispatcher scan calls this for every row whose time has come, and
    /// `jmap.rs::release_submission` calls it for the row it has just released.
    pub(crate) async fn attempt_submission(
        &self,
        id: &str,
        now: DateTime<Utc>,
    ) -> Result<Option<Filed>> {
        // One sender per row: an inline send or a release may be in flight for
        // this id right now, with the row still `pending`.
        let Some(_claim) = claim_send(id) else {
            return Ok(None);
        };
        // Read after the claim, so the row is the one this attempt acts on: a
        // cancel that landed since the scan must win (source of truth is the
        // row, plan risk #5), and so must a send that has just finished.
        let Some(row) = self
            .store()
            .get_submission(id)
            .await?
        else {
            self.accepted_submissions.lock().expect("accepted submissions lock").remove(id);
            return Ok(None);
        };
        // SMTP accepted this one earlier in this process but the `final`
        // write failed. Retry the write, never the send.
        let accepted_claim = self.accepted_submissions.lock()
            .expect("accepted submissions lock").get(id).cloned();
        if let Some(claim) = accepted_claim {
            if let Ok(recorded) = self.store().accept_submission_attempt(id, &claim).await {
                self.accepted_submissions.lock().expect("accepted submissions lock").remove(id);
                if recorded {
                    self.record_change(&row.account_id, ChangeType::EmailSubmission, id, ChangeOp::Updated).await?;
                    self.broadcast_state(&row.account_id).await;
                }
            }
            return Ok(None);
        }
        if row.undo_status != SUBMISSION_PENDING { return Ok(None); }
        // A hold that cannot be read is a hold: the row is not sent.
        let hold = self
            .store()
            .get_submission_hold(id)
            .await?
            .and_then(|h| h.hold);
        if !submission_is_due(&row, hold.as_deref(), now) {
            return Ok(None);
        }
        let Some(rt) = self.runtime(&row.account_id) else {
            // Account not connected right now — retry on a later tick.
            return Ok(None);
        };
        // Honour the backoff from an earlier failed attempt.
        let attempts = self
            .store()
            .get_submission_attempts(id)
            .await?
            .unwrap_or_default();
        if let Some(next) = attempts
            .next_attempt_at
            .as_deref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            && now < next.with_timezone(&Utc)
        {
            return Ok(None);
        }

        let filed = match self
            .send_submission(&row, &rt, attempts.next_attempt_at.as_deref())
            .await
        {
            // Delivered: `send_submission` has recorded `final` and the change.
            Ok(filed) => Some(filed),
            Err(not_sent) => {
                let Some(claim) = &not_sent.claim else { return Ok(None); };
                let attempt = attempts.attempts.saturating_add(1);
                let give_up = not_sent.uncertain() || not_sent.permanent || attempt >= MAX_SUBMISSION_ATTEMPTS;
                let (status, next_at) = if not_sent.uncertain() {
                    ("uncertain", None)
                } else if give_up {
                    (SUBMISSION_FAILED, None)
                } else {
                    (
                        SUBMISSION_PENDING,
                        Some((now + retry_delay(attempt)).to_rfc3339()),
                    )
                };
                tracing::warn!(
                    "submission {id} attempt ended (attempt {attempt} of {MAX_SUBMISSION_ATTEMPTS}{}): {}",
                    if give_up { ", giving up" } else { "" },
                    not_sent.error
                );
                let recorded = self
                    .store()
                    .finish_submission_attempt(
                        id,
                        claim,
                        &not_sent.error.to_string(),
                        next_at.as_deref(),
                        status,
                    )
                    .await?;
                if recorded {
                    self.record_change(
                        &row.account_id,
                        ChangeType::EmailSubmission,
                        id,
                        ChangeOp::Updated,
                    )
                    .await?;
                    if give_up && !not_sent.uncertain() {
                        self.emit_event(
                            crate::change::EventKind::SubmissionFailed,
                            &row.account_id,
                            &[id.to_string()],
                        );
                    }
                }
                None
            }
        };
        self.broadcast_state(&row.account_id).await;
        Ok(filed)
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

/// Whether a pending submission may be sent at `now`. A row with a `hold`
/// (0031) is never due, whatever its times say and whatever the hold is called:
/// an unrecognised hold must not read as "no hold". Otherwise its fire time
/// must have arrived: `sendAt` when set, else `createdAt + holdSeconds` (the
/// undo-send window).
fn submission_is_due(row: &SubmissionRow, hold: Option<&str>, now: DateTime<Utc>) -> bool {
    if hold.is_some() {
        return false;
    }
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

    #[tokio::test]
    async fn accepted_recovery_is_engine_scoped_and_claim_bound() {
        async fn fixture() -> Engine {
            let store = mw_store::Store::open_in_memory(mw_store::ServerKey::generate()).await.unwrap();
            let account = store.create_account(&mw_store::NewAccount {
                kind: mw_store::AccountKind::Imap, host: "imap.test", port: 993,
                tls: "implicit", username: "me@test.example", sync_policy_json: "{}",
            }, &mw_store::Credentials { username: "me@test.example".into(), password: "test".into() }).await.unwrap();
            store.insert_submission(&mw_store::SubmissionRow {
                id: "same-id".into(), account_id: account, email_id: "draft".into(),
                identity_id: None, send_at: None, undo_status: "sending:same-claim".into(),
                hold_seconds: 0, created_at: "2000-01-01T00:00:00Z".into(),
            }).await.unwrap();
            Engine::new(store)
        }
        let owner = fixture().await;
        let other = fixture().await;
        owner.accepted_submissions.lock().unwrap().insert("same-id".into(), "sending:same-claim".into());
        other.dispatch_tick().await.unwrap();
        assert_eq!(other.store().get_submission("same-id").await.unwrap().unwrap().undo_status,"sending:same-claim");
        assert_eq!(owner.accepted_submissions.lock().unwrap().len(),1);
        owner.dispatch_tick().await.unwrap();
        assert_eq!(owner.store().get_submission("same-id").await.unwrap().unwrap().undo_status,"final");
        assert!(owner.accepted_submissions.lock().unwrap().is_empty());
        owner.store().transition_submission("same-id","final","sending:replacement").await.unwrap();
        owner.accepted_submissions.lock().unwrap().insert("same-id".into(),"sending:same-claim".into());
        owner.dispatch_tick().await.unwrap();
        assert_eq!(owner.store().get_submission("same-id").await.unwrap().unwrap().undo_status,"sending:replacement");
        assert!(owner.accepted_submissions.lock().unwrap().is_empty());
    }

    fn row(send_at: Option<&str>, hold_seconds: u32) -> SubmissionRow {
        SubmissionRow {
            id: "sub".into(),
            account_id: "acct".into(),
            email_id: "email".into(),
            identity_id: None,
            send_at: send_at.map(String::from),
            undo_status: SUBMISSION_PENDING.into(),
            hold_seconds,
            created_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn a_held_submission_is_never_due() {
        let long_after: DateTime<Utc> = "2036-01-01T00:00:00Z".parse().unwrap();
        // Controls: without a hold, each of these rows is due by then.
        for r in [
            row(None, 0),
            row(None, 30),
            row(Some("2026-06-01T00:00:00Z"), 0),
        ] {
            assert!(submission_is_due(&r, None, long_after));
            assert!(!submission_is_due(&r, Some("manual"), long_after));
            // A hold this build does not know is still a hold.
            assert!(!submission_is_due(&r, Some("legal"), long_after));
            assert!(!submission_is_due(&r, Some(""), long_after));
        }
        // And the time rules are unchanged.
        let just_before: DateTime<Utc> = "2026-01-01T00:00:29Z".parse().unwrap();
        assert!(!submission_is_due(&row(None, 30), None, just_before));
        assert!(!submission_is_due(
            &row(Some("2026-06-01T00:00:00Z"), 0),
            None,
            just_before
        ));
    }

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
