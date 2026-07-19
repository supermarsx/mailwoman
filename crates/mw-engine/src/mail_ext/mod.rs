//! The mail-family JMAP completeness surface (t16 J1–J5): the RFC 8621 / 8620 /
//! 9425 mail methods the core `jmap.rs` dispatch did not yet answer —
//! `Thread/get` + `Thread/changes`, `SearchSnippet/get`,
//! `VacationResponse/get|set`, `Quota/get`, and `Email/copy|import|parse`.
//!
//! Every method rides the existing `handle_jmap` envelope;
//! [`Engine::dispatch_mail_ext`] is reached from the core dispatch (`jmap.rs`)
//! for any mail-ext method (mirrors [`crate::pim::dispatch`]). The set of method
//! names in [`dispatch::is_mail_ext_method`] is the contract the web client and
//! the mock compile against — do not add/rename without a coordinator
//! re-broadcast.
//!
//! These reuse data the engine already holds and do not duplicate store state or
//! re-derive threading:
//! - `Thread/*` groups by the real JWZ `thread_id` assigned at ingest
//!   ([`crate::thread`]); the store keys threads off the `messages`/`threads`
//!   tables already.
//! - `Quota/get` reports the admin-configured `quotas` row (0007) against live
//!   usage summed from the sealed message cache.
//! - `VacationResponse/*` persists the per-account singleton in the existing
//!   `settings` table (key-namespaced by account).
//! - `Email/copy|import|parse` reuse [`Engine::fetch_blob`] + [`Engine::ingest`].

use serde_json::{Value, json};

use crate::backend::{EngineError, Result};
use crate::engine::Engine;

pub mod dispatch;
pub mod email_ext;
pub mod quota;
pub mod snippet;
pub mod threads;
pub mod vacation;

#[cfg(test)]
mod tests;

impl Engine {
    /// Every cached message for an account, walked mailbox-by-mailbox over the
    /// existing `list_mailboxes` + `list_message_ids` + `get_message` store API
    /// (no new store method): the shared source for thread grouping
    /// (`Thread/get`) and usage accounting (`Quota/get`). A message row that
    /// vanishes mid-walk is skipped rather than failing the whole call.
    pub(crate) async fn account_messages(
        &self,
        account_id: &str,
    ) -> Result<Vec<mw_store::Message>> {
        let mut out = Vec::new();
        for mb in self.store().list_mailboxes(account_id).await? {
            let ids = self.store().list_message_ids(&mb.id, i64::MAX, 0).await?;
            for id in ids {
                match self.store().get_message(&id).await {
                    Ok(m) => out.push(m),
                    Err(mw_store::StoreError::NotFound) => {}
                    Err(e) => return Err(EngineError::Store(e)),
                }
            }
        }
        Ok(out)
    }
}

/// A JMAP method-level failure result (`serverFail`), matching `jmap.rs`.
pub(crate) fn server_fail(e: impl std::fmt::Display) -> Value {
    json!({ "type": "serverFail", "description": e.to_string() })
}

/// Restrict a `*/get` to the caller's `ids` array (returns `None` for "all", the
/// RFC 8620 §5.1 convention).
pub(crate) fn wanted_ids(args: &Value) -> Option<Vec<String>> {
    args.get("ids").and_then(Value::as_array).map(|a| {
        a.iter()
            .filter_map(Value::as_str)
            .map(String::from)
            .collect()
    })
}
