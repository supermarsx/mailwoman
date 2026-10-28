//! `Mailbox/empty` (t45): empty one folder, one bounded batch per call.
//!
//! A Mailwoman extension method on the ordinary JMAP envelope:
//!
//! ```text
//! ["Mailbox/empty", {"accountId", "mailboxId", "outcome": "trash"|"destroy",
//!                    "expectedTotal": n, "limit": 0..=200}, "c"]
//! ```
//!
//! **The outcome is the server's to decide, and the client must name it.** An
//! ordinary folder's messages move to Trash; Trash and Junk are emptied for
//! good; a folder in an account with no Trash is emptied for good too. The
//! request carries the outcome the user confirmed, and a request whose outcome
//! is not the one the server would apply (a role changed, a Trash appeared or
//! went away) is refused with `stateMismatch` before anything is touched — so a
//! dialog that said "moved to Trash" can never delete permanently.
//!
//! **A stale view is refused.** `expectedTotal` is the count the client last
//! showed. When the folder now holds a different number of messages, beyond a
//! small allowance for mail arriving while the user reads the dialog, the call
//! is refused with `stateMismatch` and the current `total`, and the client
//! previews again.
//!
//! **Bounded.** One call touches at most `limit` messages (default 100, at most
//! [`MAX_BATCH`]): one backend `move_messages` (IMAP `UID MOVE`, or `UID COPY` +
//! `\Deleted` + `UID EXPUNGE` on a server without MOVE) or one
//! `expunge_messages` (`\Deleted` + `UID EXPUNGE` of exactly those UIDs). The
//! client calls again until `remaining` is 0, which is what lets it show
//! progress and stop between batches. `limit: 0` is the preview: it answers
//! the outcome and the count and changes nothing.
//!
//! **Only this folder.** The messages are the ones whose mailbox is this one;
//! child folders and their messages are not touched.
//!
//! **Authority.** The mailbox must be one of this account's (the account the
//! request was authenticated for, which for a linked mailbox is the linked
//! child the server selected); an id from anywhere else is `invalidArguments`.
//! A backend without folder verbs (POP3) gets `accountNotSupportedByMethod`, a
//! bridge cannot destroy, and an IMAP server that reports (MYRIGHTS) that the
//! user may not delete messages here is `forbidden`.

use std::collections::HashMap;

use serde_json::{Map, Value, json};

use super::{pseudo_uid, server_fail, set_error, tolerant};
use crate::account::AccountRuntime;
use crate::backend::{EngineError, MessageRef, MoveOutcome, RawMailboxRef};
use crate::change::{ChangeOp, ChangeType};
use crate::engine::Engine;

/// The most messages one `Mailbox/empty` call touches.
pub const MAX_BATCH: u64 = 200;
const DEFAULT_BATCH: u64 = 100;

/// What emptying a folder does to its messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Outcome {
    /// Every message moves to the account's Trash.
    Trash,
    /// Every message is removed from the mail server for good.
    Destroy,
}

impl Outcome {
    fn as_str(self) -> &'static str {
        match self {
            Outcome::Trash => "trash",
            Outcome::Destroy => "destroy",
        }
    }
}

/// What this account's backend can do when emptying a folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Ability {
    pub move_to_trash: bool,
    pub destroy: bool,
}

/// How far the folder's count may drift from `expectedTotal` (mail arriving
/// while the dialog is open) before the view counts as stale.
fn tolerance(expected: u64) -> u64 {
    (expected / 20).max(2)
}

fn refuse(kind: &str, description: &str) -> Value {
    json!({"type": kind, "description": description})
}

impl Engine {
    /// POP3 has no folders, so nothing can be emptied there; a bridge can move
    /// messages but has no expunge, so it cannot empty for good.
    pub(super) fn empty_ability(&self, account_id: &str, rt: &AccountRuntime) -> Ability {
        let manages = rt.backend.manages_mailboxes();
        let bridge = self.is_plugin_backed(account_id);
        Ability {
            move_to_trash: manages || bridge,
            destroy: manages && !bridge,
        }
    }

    /// `Mailbox/empty`; see the module documentation.
    pub(super) async fn mailbox_empty(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        args: &Value,
    ) -> Value {
        if let Some(requested) = args.get("accountId")
            && requested.as_str() != Some(account_id)
        {
            return refuse("accountNotFound", "no such account");
        }
        let Some(mailbox_id) = args.get("mailboxId").and_then(Value::as_str) else {
            return json!({"type":"invalidArguments","arguments":["mailboxId"]});
        };
        let requested = match args.get("outcome").and_then(Value::as_str) {
            Some("trash") => Outcome::Trash,
            Some("destroy") => Outcome::Destroy,
            _ => return json!({"type":"invalidArguments","arguments":["outcome"]}),
        };
        let limit = match args.get("limit") {
            None | Some(Value::Null) => DEFAULT_BATCH,
            Some(value) => match value.as_u64() {
                Some(n) if n <= MAX_BATCH => n,
                _ => return json!({"type":"invalidArguments","arguments":["limit"]}),
            },
        };
        let expected = match args.get("expectedTotal") {
            Some(value) => match value.as_u64() {
                Some(n) => n,
                None => return json!({"type":"invalidArguments","arguments":["expectedTotal"]}),
            },
            None => return json!({"type":"invalidArguments","arguments":["expectedTotal"]}),
        };

        let ability = self.empty_ability(account_id, rt);
        if !ability.move_to_trash && !ability.destroy {
            return refuse(
                "accountNotSupportedByMethod",
                "this account's mail server has no folders to empty",
            );
        }
        // Only this account's own mailboxes: the role view lists nothing else,
        // and a saved search is not in it either.
        let view = match self.mailbox_role_view(account_id).await {
            Ok(view) => view,
            Err(error) => return server_fail(&error),
        };
        let Some(mailbox) = view
            .mailboxes
            .iter()
            .find(|m| m.id == mailbox_id && m.account_id == account_id)
            .cloned()
        else {
            return json!({"type":"invalidArguments","arguments":["mailboxId"],"description":"no such mailbox in this account"});
        };
        let trashes: Vec<&mw_store::Mailbox> = view
            .mailboxes
            .iter()
            .filter(|m| m.role.as_deref() == Some("trash") && m.uidvalidity != 0)
            .collect();
        let permanent_role = matches!(mailbox.role.as_deref(), Some("trash" | "junk"));
        let (outcome, trash) = if permanent_role {
            (Outcome::Destroy, None)
        } else {
            match trashes.as_slice() {
                [] => (Outcome::Destroy, None),
                [only] if only.id != mailbox.id => (Outcome::Trash, Some((*only).clone())),
                [_] => (Outcome::Destroy, None),
                _ => {
                    return refuse(
                        "forbidden",
                        "more than one folder is mapped as Trash; fix the folder roles first",
                    );
                }
            }
        };
        if outcome != requested {
            return json!({
                "type": "stateMismatch",
                "description": "emptying this folder now has a different outcome; preview again",
                "mailwomanOutcome": outcome.as_str(),
            });
        }
        if outcome == Outcome::Destroy && !ability.destroy {
            return refuse(
                "accountNotSupportedByMethod",
                "this account's mail server cannot delete messages permanently",
            );
        }
        if let Some(refusal) = self.empty_rights(rt, &mailbox).await {
            return refusal;
        }

        let total = match self.store().count_messages_in_mailbox(&mailbox.id).await {
            Ok(total) => total,
            Err(error) => return server_fail(&error),
        };
        if total.abs_diff(expected) > tolerance(expected) {
            return json!({
                "type": "stateMismatch",
                "description": "the folder changed since it was counted; preview again",
                "mailwomanTotal": total,
            });
        }

        let mut response = json!({
            "accountId": account_id,
            "mailboxId": mailbox.id,
            "outcome": outcome.as_str(),
            "trashId": trash.as_ref().map(|t| t.id.clone()),
            "total": total,
            "processed": 0,
            "remaining": total,
        });
        if limit == 0 || total == 0 {
            return response;
        }

        let ids = match self
            .store()
            .list_message_ids(&mailbox.id, limit as i64, 0)
            .await
        {
            Ok(ids) => ids,
            Err(error) => return server_fail(&error),
        };
        let (processed, not_processed) = match &trash {
            None => {
                let (destroyed, not_destroyed) = self.destroy_emails(account_id, rt, &ids).await;
                (destroyed.len() as u64, not_destroyed)
            }
            Some(trash) => {
                self.move_batch_to_trash(account_id, rt, &mailbox, trash, &ids)
                    .await
            }
        };
        let remaining = self
            .store()
            .count_messages_in_mailbox(&mailbox.id)
            .await
            .unwrap_or_else(|_| total.saturating_sub(processed));
        tracing::info!(
            account = account_id,
            mailbox = %mailbox.id,
            outcome = outcome.as_str(),
            processed,
            refused = not_processed.len(),
            remaining,
            "Mailbox/empty batch"
        );
        response["processed"] = json!(processed);
        response["remaining"] = json!(remaining);
        if !not_processed.is_empty() {
            response["notProcessed"] = Value::Object(not_processed);
        }
        response
    }

    /// `forbidden` when the mail server says this user may not remove messages
    /// from `mailbox` (RFC 4314 `t` delete and `e` expunge). A server without
    /// ACL support answers nothing, and the server stays the enforcement point.
    async fn empty_rights(
        &self,
        rt: &AccountRuntime,
        mailbox: &mw_store::Mailbox,
    ) -> Option<Value> {
        if mailbox.uidvalidity == 0 {
            return None;
        }
        let target = RawMailboxRef {
            name: mailbox.name.clone(),
            uidvalidity: mailbox.uidvalidity,
        };
        match rt.backend.my_rights(&target).await {
            Ok(rights) if !(rights.contains('t') && rights.contains('e')) => Some(refuse(
                "forbidden",
                "you may not delete messages in this folder",
            )),
            Ok(_) => None,
            Err(EngineError::Unsupported(_)) => None,
            Err(error) => {
                tracing::debug!("MYRIGHTS before Mailbox/empty failed: {error}");
                None
            }
        }
    }

    /// Move `ids` (all in `source`) to `trash` with one backend move, then
    /// mirror the move in the cache and the search index. Returns how many
    /// moved and a SetError per id that did not.
    async fn move_batch_to_trash(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        source: &mw_store::Mailbox,
        trash: &mw_store::Mailbox,
        ids: &[String],
    ) -> (u64, Map<String, Value>) {
        let mut not_processed = Map::new();
        let rows = match self.store().get_messages(ids).await {
            Ok(rows) => rows,
            Err(error) => {
                let why = set_error(&EngineError::Store(error));
                for id in ids {
                    not_processed.insert(id.clone(), why.clone());
                }
                return (0, not_processed);
            }
        };
        let mut names: HashMap<String, String> = HashMap::new();
        // (stable id, message-id seed, upstream uid) — sorted by uid so the
        // destination UIDs of COPYUID pair with them (RFC 4315 orders both sets
        // the same way).
        let mut upstream: Vec<(String, String, MessageRef, u32)> = Vec::new();
        let mut local: Vec<(String, String)> = Vec::new();
        for (id, row) in ids.iter().zip(rows) {
            let Some(msg) = row.filter(|m| m.account_id == account_id && m.mailbox_id == source.id)
            else {
                not_processed.insert(
                    id.clone(),
                    json!({"type":"notFound","description":"the message left this folder"}),
                );
                continue;
            };
            let seed = msg.message_id.clone().unwrap_or_else(|| id.clone());
            match self.imap_ref_from(&msg, &mut names).await {
                Ok(Some(mref)) => {
                    let uid = match &mref {
                        MessageRef::Imap { uid, .. } => *uid,
                        _ => 0,
                    };
                    upstream.push((id.clone(), seed, mref, uid));
                }
                Ok(None) => local.push((id.clone(), seed)),
                Err(error) => {
                    not_processed.insert(id.clone(), set_error(&error));
                }
            }
        }
        upstream.sort_by_key(|(_, _, _, uid)| *uid);

        let to = RawMailboxRef {
            name: trash.name.clone(),
            uidvalidity: trash.uidvalidity,
        };
        // (stable id, destination uid, destination uidvalidity)
        let mut moved: Vec<(String, u32, u32)> = local
            .into_iter()
            .map(|(id, seed)| (id, pseudo_uid(&seed), trash.uidvalidity))
            .collect();
        if !upstream.is_empty() {
            let refs: Vec<MessageRef> = upstream.iter().map(|(_, _, r, _)| r.clone()).collect();
            match tolerant(rt.backend.move_messages(&refs, &to).await) {
                Ok(outcome) => {
                    let paired = match outcome {
                        Some(MoveOutcome::Uidplus { uidvalidity, uids })
                            if uids.len() == upstream.len() =>
                        {
                            Some((uidvalidity, uids))
                        }
                        _ => None,
                    };
                    for (i, (id, seed, _, _)) in upstream.into_iter().enumerate() {
                        let (uid, validity) = match &paired {
                            Some((validity, uids)) => (uids[i], *validity),
                            None => (pseudo_uid(&seed), trash.uidvalidity),
                        };
                        moved.push((id, uid, validity));
                    }
                }
                Err(error) => {
                    let why = set_error(&error);
                    for (id, ..) in upstream {
                        not_processed.insert(id, why.clone());
                    }
                }
            }
        }

        let mut done: Vec<String> = Vec::with_capacity(moved.len());
        for (id, uid, validity) in moved {
            match self
                .store()
                .relocate_message(&id, &trash.id, uid, validity)
                .await
            {
                Ok(()) => {
                    let _ = self.search().relocate(&id, &trash.id);
                    done.push(id);
                }
                Err(error) => {
                    // The server moved it; the next sync of both folders puts
                    // the cache right. Reported, so the client stops here.
                    tracing::warn!(
                        "mirroring an emptied message {id} in the cache failed: {error}"
                    );
                    not_processed.insert(id, set_error(&EngineError::Store(error)));
                }
            }
        }
        if !done.is_empty() {
            if let Err(error) = self
                .store()
                .record_changes(
                    account_id,
                    ChangeType::Email.as_str(),
                    &done,
                    ChangeOp::Updated.as_str(),
                )
                .await
            {
                tracing::warn!("recording the Mailbox/empty moves failed: {error}");
            }
            for mailbox_id in [&source.id, &trash.id] {
                if let Err(error) = self
                    .record_change(
                        account_id,
                        ChangeType::Mailbox,
                        mailbox_id,
                        ChangeOp::Updated,
                    )
                    .await
                {
                    tracing::warn!("recording the Mailbox count change failed: {error}");
                }
            }
            self.emit_event(crate::change::EventKind::MessageMoved, account_id, &done);
        }
        (done.len() as u64, not_processed)
    }
}

#[cfg(test)]
mod tests {
    use super::tolerance;

    #[test]
    fn a_small_drift_is_allowed_and_a_large_one_is_not() {
        assert_eq!(tolerance(0), 2);
        assert_eq!(tolerance(10), 2);
        assert_eq!(tolerance(1000), 50);
    }
}
