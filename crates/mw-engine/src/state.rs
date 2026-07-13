//! Real per-account state tokens + `*/changes` diffs + the realtime broadcast
//! (plan §1.2, §2.1/§2.2). Replaces the V1 `SESSION_STATE = "engine-0"`.
//!
//! State is sourced from the store `changes` log: each mutation appends a row
//! and bumps the `(account, type)` monotonic counter. `sessionState` is a
//! composite of the per-type counters, so it advances on any account change.

use std::collections::HashMap;

use crate::backend::Result;
use crate::change::{ChangeOp, ChangeType, Changes, StateChange};
use crate::engine::Engine;

impl Engine {
    /// Append one change and return the new `(account, type)` state. Best-effort
    /// for callers that do not care about the exact number.
    pub(crate) async fn record_change(
        &self,
        account_id: &str,
        kind: ChangeType,
        stable_id: &str,
        op: ChangeOp,
    ) -> Result<u64> {
        Ok(self
            .store()
            .record_change(account_id, kind.as_str(), stable_id, op.as_str())
            .await?)
    }

    /// The current per-type state token as an opaque string.
    pub(crate) async fn type_state(&self, account_id: &str, kind: ChangeType) -> Result<String> {
        Ok(self
            .store()
            .current_state(account_id, kind.as_str())
            .await?
            .to_string())
    }

    /// The composite `sessionState`: it changes whenever any of the account's
    /// datatype states advance (RFC 8620 §2 — `sessionState`). The PIM datatype
    /// counters are folded in so a calendar/task/note/contact change also bumps
    /// `sessionState` (plan §1.8/§2.2).
    pub(crate) async fn session_state(&self, account_id: &str) -> String {
        let e = self.type_num(account_id, ChangeType::Email).await;
        let m = self.type_num(account_id, ChangeType::Mailbox).await;
        let s = self.type_num(account_id, ChangeType::EmailSubmission).await;
        let p = self.pim_state_sum(account_id).await;
        format!("e{e}m{m}s{s}p{p}")
    }

    /// The sum of the account's PIM datatype counters — a cheap composite that
    /// advances on any PIM change (folded into `sessionState`).
    async fn pim_state_sum(&self, account_id: &str) -> u64 {
        let mut sum = 0;
        for kind in [
            ChangeType::Calendar,
            ChangeType::CalendarEvent,
            ChangeType::Task,
            ChangeType::Note,
            ChangeType::AddressBook,
            ChangeType::ContactCard,
            ChangeType::ContactGroup,
        ] {
            sum += self.pim_type_num(account_id, kind).await;
        }
        sum
    }

    async fn type_num(&self, account_id: &str, kind: ChangeType) -> u64 {
        self.store()
            .current_state(account_id, kind.as_str())
            .await
            .unwrap_or(0)
    }

    async fn pim_type_num(&self, account_id: &str, kind: ChangeType) -> u64 {
        self.store()
            .current_pim_state(account_id, kind.as_str())
            .await
            .unwrap_or(0)
    }

    // ── PIM state tokens + `*/changes` (plan §1.8/§2.2) ─────────────────────
    // Sourced from the separate `pim_changes` log so PIM counters are disjoint
    // from the mail `changes` counters — a Calendar and an Email can share the
    // same numeric state without colliding.

    /// Append one PIM change and return the new `(account, type)` PIM state.
    pub(crate) async fn record_pim_change(
        &self,
        account_id: &str,
        kind: ChangeType,
        object_id: &str,
        op: ChangeOp,
    ) -> Result<u64> {
        Ok(self
            .store()
            .record_pim_change(account_id, kind.as_str(), object_id, op.as_str())
            .await?)
    }

    /// The current PIM per-type state token as an opaque string.
    pub(crate) async fn pim_type_state(
        &self,
        account_id: &str,
        kind: ChangeType,
    ) -> Result<String> {
        Ok(self
            .store()
            .current_pim_state(account_id, kind.as_str())
            .await?
            .to_string())
    }

    /// Build the `{oldState,newState,created,updated,destroyed}` diff for a PIM
    /// datatype since `since_state` (frozen §2.1), from the `pim_changes` log.
    pub(crate) async fn build_pim_changes(
        &self,
        account_id: &str,
        kind: ChangeType,
        since_state: &str,
    ) -> Result<Changes> {
        let since: u64 = since_state.parse().unwrap_or(0);
        let current = self
            .store()
            .current_pim_state(account_id, kind.as_str())
            .await?;
        let rows = self
            .store()
            .pim_changes_since(account_id, kind.as_str(), since)
            .await?;
        let (created, updated, destroyed) =
            fold_changes(rows.iter().map(|r| (r.object_id.as_str(), r.op.as_str())));
        Ok(Changes {
            old_state: since.to_string(),
            new_state: current.to_string(),
            created,
            updated,
            destroyed,
            has_more_changes: false,
        })
    }

    /// Build the `{oldState,newState,created,updated,destroyed}` diff for a
    /// datatype since `since_state` (frozen §2.1). `has_more_changes` is always
    /// false — the whole tail is returned.
    pub(crate) async fn build_changes(
        &self,
        account_id: &str,
        kind: ChangeType,
        since_state: &str,
    ) -> Result<Changes> {
        let since: u64 = since_state.parse().unwrap_or(0);
        let current = self
            .store()
            .current_state(account_id, kind.as_str())
            .await?;
        let rows = self
            .store()
            .changes_since(account_id, kind.as_str(), since)
            .await?;

        // Fold to the latest op per id; "created then destroyed in-window" cancels.
        let mut order: Vec<String> = Vec::new();
        let mut folded: HashMap<String, (bool, &'static str)> = HashMap::new();
        for r in &rows {
            let entry = folded.entry(r.stable_id.clone()).or_insert_with(|| {
                order.push(r.stable_id.clone());
                (false, "updated")
            });
            if r.op == "created" {
                entry.0 = true;
            }
            entry.1 = match r.op.as_str() {
                "created" => "created",
                "destroyed" => "destroyed",
                _ => "updated",
            };
        }

        let (mut created, mut updated, mut destroyed) = (Vec::new(), Vec::new(), Vec::new());
        for id in order {
            let (created_in_window, last) = folded[&id];
            match last {
                "destroyed" => {
                    if !created_in_window {
                        destroyed.push(id);
                    }
                }
                "created" => created.push(id),
                _ => updated.push(id),
            }
        }

        Ok(Changes {
            old_state: since.to_string(),
            new_state: current.to_string(),
            created,
            updated,
            destroyed,
            has_more_changes: false,
        })
    }

    /// Fan a [`StateChange`] out to every subscribed WS/SSE session (plan §1.2,
    /// §2.2). A no-op when no session is listening.
    pub(crate) async fn broadcast_state(&self, account_id: &str) {
        let email = self
            .type_num(account_id, ChangeType::Email)
            .await
            .to_string();
        let mailbox = self
            .type_num(account_id, ChangeType::Mailbox)
            .await
            .to_string();
        let submission = self
            .type_num(account_id, ChangeType::EmailSubmission)
            .await
            .to_string();
        let sc = StateChange {
            account_id: account_id.to_string(),
            thread: email.clone(),
            email,
            mailbox,
            submission,
        };
        // Ignore the "no receivers" error — sessions attach lazily.
        let _ = self.changes_tx().send(sc);
    }
}

/// Fold an ordered `(object_id, op)` change stream to the JMAP
/// `created`/`updated`/`destroyed` id sets (frozen §2.1): the latest op per id
/// wins, and a "created then destroyed in-window" cancels out. Shared by the
/// mail and PIM `*/changes` builders.
pub(crate) fn fold_changes<'a>(
    rows: impl Iterator<Item = (&'a str, &'a str)>,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut order: Vec<String> = Vec::new();
    let mut folded: HashMap<String, (bool, &'static str)> = HashMap::new();
    for (id, op) in rows {
        let entry = folded.entry(id.to_string()).or_insert_with(|| {
            order.push(id.to_string());
            (false, "updated")
        });
        if op == "created" {
            entry.0 = true;
        }
        entry.1 = match op {
            "created" => "created",
            "destroyed" => "destroyed",
            _ => "updated",
        };
    }
    let (mut created, mut updated, mut destroyed) = (Vec::new(), Vec::new(), Vec::new());
    for id in order {
        let (created_in_window, last) = folded[&id];
        match last {
            "destroyed" => {
                if !created_in_window {
                    destroyed.push(id);
                }
            }
            "created" => created.push(id),
            _ => updated.push(id),
        }
    }
    (created, updated, destroyed)
}
