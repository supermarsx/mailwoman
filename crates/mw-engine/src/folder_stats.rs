//! `Mailbox/stats`: what the folder Properties dialog shows about one folder.
//!
//! A Mailwoman extension method on the ordinary JMAP envelope:
//!
//! ```text
//! ["Mailbox/stats", {"accountId", "mailboxId"}, "c"]
//! ```
//!
//! The answer is read from the cache the sync keeps, in a handful of aggregate
//! queries, so it costs the server one scan of the folder's rows and no round
//! trip to the mail server. `approximate` is true while the cache holds fewer
//! messages than the folder reports (a first sync that has not finished, or
//! mail that arrived since): the sizes then cover only what is cached.
//!
//! The server host is deliberately absent. The path, the delimiter and the
//! UIDVALIDITY / UIDNEXT / HIGHESTMODSEQ counters are the folder's own; which
//! of them a user may see is the web client's `properties.remoteLocation.visibility`
//! policy, applied where they are shown.

use serde_json::{Value, json};

use super::server_fail;
use crate::account::AccountRuntime;
use crate::engine::Engine;

impl Engine {
    pub(super) async fn mailbox_stats(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        args: &Value,
    ) -> Value {
        if let Some(requested) = args.get("accountId")
            && requested.as_str() != Some(account_id)
        {
            return json!({"type":"accountNotFound"});
        }
        let Some(mailbox_id) = args.get("mailboxId").and_then(Value::as_str) else {
            return json!({"type":"invalidArguments","arguments":["mailboxId"]});
        };
        let view = match self.mailbox_role_view(account_id).await {
            Ok(view) => view,
            Err(error) => return server_fail(&error),
        };
        let Some(mailbox) = view.mailboxes.iter().find(|m| m.id == mailbox_id) else {
            return json!({"type":"invalidArguments","arguments":["mailboxId"],
                "description":"no such folder in this account"});
        };
        let stats = match self.store().folder_stats(mailbox_id).await {
            Ok(stats) => stats,
            Err(error) => return server_fail(&crate::backend::EngineError::Store(error)),
        };
        let delimiter = self.hierarchy_delimiter(rt).await;
        let total = u64::from(mailbox.total);
        json!({
            "accountId": account_id,
            "mailboxId": mailbox_id,
            "role": mailbox.role,
            "totalEmails": total,
            "unreadEmails": mailbox.unread,
            "cachedEmails": stats.cached,
            "approximate": stats.cached < total,
            "sizeBytes": stats.size_bytes,
            "flaggedEmails": stats.flagged,
            "draftEmails": stats.drafts,
            "oldest": stats.oldest,
            "newest": stats.newest,
            "bodiesCached": stats.bodies_cached,
            "largest": stats.largest.iter()
                .map(|m| json!({"id": m.stable_id, "size": m.size}))
                .collect::<Vec<_>>(),
            "remote": {
                "path": rt.backend.decode_mailbox_name(&mailbox.name),
                "upstreamName": mailbox.name,
                "delimiter": delimiter.map(String::from),
                "uidValidity": mailbox.uidvalidity,
                "uidNext": mailbox.uidnext,
                "highestModSeq": mailbox.highestmodseq,
                "namespace": "personal",
            },
        })
    }
}

impl Engine {
    /// `Email/location`: where each message lives on the server, for the
    /// message Properties dialog. `{"ids": [...]}` (at most 50) answers
    /// `{"list": [{id, mailboxId, mailboxPath, uid, uidValidity}], "notFound": [...]}`.
    /// A message of another account is `notFound`.
    pub(super) async fn email_location(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        args: &Value,
    ) -> Value {
        let Some(ids) = args
            .get("ids")
            .and_then(Value::as_array)
            .filter(|ids| ids.len() <= 50 && ids.iter().all(Value::is_string))
        else {
            return json!({"type":"invalidArguments","arguments":["ids"]});
        };
        let view = match self.mailbox_role_view(account_id).await {
            Ok(view) => view,
            Err(error) => return server_fail(&error),
        };
        let mut list = Vec::new();
        let mut not_found = Vec::new();
        for id in ids.iter().filter_map(Value::as_str) {
            let message = self.store().get_message(id).await.ok().filter(|m| m.account_id == account_id);
            let Some(message) = message else {
                not_found.push(json!(id));
                continue;
            };
            let path = view
                .mailboxes
                .iter()
                .find(|m| m.id == message.mailbox_id)
                .map(|m| rt.backend.decode_mailbox_name(&m.name));
            list.push(json!({
                "id": id,
                "mailboxId": message.mailbox_id,
                "mailboxPath": path,
                "uid": message.uid,
                "uidValidity": message.uidvalidity,
            }));
        }
        json!({"accountId": account_id, "list": list, "notFound": not_found})
    }
}
