//! The transport-generic Gmail account-backend (plan §3 e12).
//!
//! [`GmailBackend`] orchestrates the Gmail REST API through an injected
//! [`Transport`] (the only I/O seam) and produces the neutral [`crate::model`]
//! types. On `wasm32-wasip2` the transport is the host-mediated `http-fetch` /
//! `oauth-token` imports ([`crate::component`]); in tests it replays recorded
//! fixtures. All Gmail quirks (labels, history-ID delta) are isolated here.

use base64::Engine as _;
use base64::engine::general_purpose::{STANDARD, URL_SAFE, URL_SAFE_NO_PAD};

use crate::gmail;
use crate::labels;
use crate::model::{
    BackendCaps, BridgeError, ChangeEvent, Flag, Mailbox, MailboxDelta, MailboxRef, MessageRef,
    RawMessage, Result, SyncCursor,
};

/// The bound-account handle passed to `oauth-token`. One plugin instance backs one
/// account; the host resolves this handle to the actual Gmail user and mints/refreshes
/// the token host-side (it never enters the guest).
pub const ACCOUNT: &str = "self";

/// A host-mediated HTTP response.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

/// The single I/O seam. The guest never opens a socket: `http` is the host's
/// `http-fetch` (allowlist-enforced), `oauth_token` the host's token acquisition.
pub trait Transport {
    fn oauth_token(&self, account: &str) -> Result<String>;
    fn http(
        &self,
        method: &str,
        url: &str,
        headers: Vec<(String, String)>,
        body: Option<Vec<u8>>,
    ) -> Result<HttpResponse>;
    fn log(&self, _msg: &str) {}
}

/// The Gmail bridge backend, generic over its [`Transport`].
pub struct GmailBackend<T: Transport> {
    tp: T,
}

impl<T: Transport> GmailBackend<T> {
    #[must_use]
    pub fn new(tp: T) -> Self {
        Self { tp }
    }

    // ── AccountBackend surface ──────────────────────────────────────────────────

    pub fn capabilities(&self) -> Result<BackendCaps> {
        // Gmail is HTTP-poll (no IDLE) and supports label re-tagging (move); it has
        // none of the Outlook-native reaction/voting/recall/focused caps.
        Ok(BackendCaps {
            idle: false,
            move_cap: true,
            reactions: false,
            voting: false,
            recall: false,
            focused_sync: false,
        })
    }

    pub fn list_mailboxes(&self) -> Result<Vec<Mailbox>> {
        let token = self.token()?;
        let labels = self.fetch_labels(&token)?;

        let mut out = Vec::new();
        for l in &labels {
            let is_mailbox = l.is_user() || labels::system_label_is_mailbox(&l.id);
            if !is_mailbox || labels::is_hidden_label(&l.id) {
                continue;
            }
            // Address by a stable, human-facing name where possible; system labels
            // have id==name. `sync_mailbox` resolves the name back to a label id.
            let name = if l.name.is_empty() {
                l.id.clone()
            } else {
                l.name.clone()
            };
            out.push(Mailbox {
                mailbox_ref: MailboxRef {
                    name,
                    uidvalidity: 1,
                },
                role: labels::label_to_role(&l.id).to_string(),
                parent: None,
                total: l.messages_total,
                unread: l.messages_unread,
            });
        }
        // The synthetic "All Mail" view (Gmail exposes no ALL label id).
        out.push(Mailbox {
            mailbox_ref: MailboxRef {
                name: "All Mail".to_string(),
                uidvalidity: 1,
            },
            role: "all".to_string(),
            parent: None,
            total: 0,
            unread: 0,
        });
        Ok(out)
    }

    pub fn sync_mailbox(&self, mbox: &MailboxRef, cursor: &SyncCursor) -> Result<MailboxDelta> {
        let token = self.token()?;
        let label_id = self.resolve_label_id(&token, &mbox.name)?;

        match cursor.history_id() {
            // Incremental: history-ID delta (the frozen `SyncCursor::Plugin` path).
            Some(hist) => self.sync_delta(&token, &label_id, &hist),
            // Baseline: enumerate the label + snapshot the current historyId.
            None => self.sync_full(&token, &label_id),
        }
    }

    pub fn fetch_raw(&self, refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
        let token = self.token()?;
        let mut out = Vec::with_capacity(refs.len());
        for r in refs {
            let (_label, id) = r
                .decode_gmail()
                .ok_or_else(|| BridgeError::Protocol("un-decodable message ref".into()))?;
            let msg: gmail::Message = self.get_json(&token, &gmail::url_message_raw(&id))?;
            let raw_b64 = msg
                .raw
                .ok_or_else(|| BridgeError::Protocol("messages.get returned no raw body".into()))?;
            let raw = decode_b64url(&raw_b64)?;
            out.push(RawMessage {
                message_ref: r.clone(),
                raw,
                msg_flags: labels::labels_to_flags(&msg.label_ids),
                // Gmail's `internalDate` is epoch millis, not RFC3339; the engine
                // re-derives the date from the parsed MIME Date header. Don't emit a
                // mislabelled value.
                internaldate: None,
            });
        }
        Ok(out)
    }

    pub fn store_flags(&self, refs: &[MessageRef], add: &[Flag], remove: &[Flag]) -> Result<()> {
        let token = self.token()?;
        let (add_ids, rem_ids) = labels::flags_to_label_ops(add, remove);
        if add_ids.is_empty() && rem_ids.is_empty() {
            return Ok(());
        }
        for r in refs {
            let (_label, id) = r
                .decode_gmail()
                .ok_or_else(|| BridgeError::Protocol("un-decodable message ref".into()))?;
            let body = serde_json::json!({
                "addLabelIds": add_ids,
                "removeLabelIds": rem_ids,
            });
            self.post_json(&token, &gmail::url_message_modify(&id), &body)?;
        }
        Ok(())
    }

    pub fn move_messages(&self, refs: &[MessageRef], to: &MailboxRef) -> Result<()> {
        let token = self.token()?;
        let dest = self.resolve_label_id(&token, &to.name)?;
        for r in refs {
            let (src_label, id) = r
                .decode_gmail()
                .ok_or_else(|| BridgeError::Protocol("un-decodable message ref".into()))?;
            // A Gmail "move" is a re-tag: add the destination label, drop the source.
            let mut remove: Vec<String> = Vec::new();
            if !src_label.is_empty() && src_label != dest {
                remove.push(src_label);
            }
            let body = serde_json::json!({
                "addLabelIds": [dest],
                "removeLabelIds": remove,
            });
            self.post_json(&token, &gmail::url_message_modify(&id), &body)?;
        }
        Ok(())
    }

    pub fn submit(&self, mbox: &MailboxRef, raw: &[u8], _flags: &[Flag]) -> Result<MessageRef> {
        let token = self.token()?;
        let b64 = URL_SAFE_NO_PAD.encode(raw);
        let is_draft = mbox.name.eq_ignore_ascii_case("draft")
            || mbox.name.eq_ignore_ascii_case("drafts")
            || mbox.name == labels::SYS_DRAFT;
        let (url, body) = if is_draft {
            (
                gmail::url_drafts_create(),
                serde_json::json!({ "message": { "raw": b64 } }),
            )
        } else {
            (
                gmail::url_messages_send(),
                serde_json::json!({ "raw": b64 }),
            )
        };
        let resp = self.post_json(&token, &url, &body)?;
        // `drafts.create` wraps the message; `messages.send` returns it directly.
        let val: serde_json::Value = serde_json::from_slice(&resp.body)
            .map_err(|e| BridgeError::Protocol(format!("submit response: {e}")))?;
        let id = val
            .get("message")
            .and_then(|m| m.get("id"))
            .or_else(|| val.get("id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| BridgeError::Protocol("submit response missing id".into()))?;
        let label = if is_draft {
            labels::SYS_DRAFT
        } else {
            labels::SYS_SENT
        };
        Ok(MessageRef::for_gmail(label, id))
    }

    pub fn poll_changes(&self) -> Result<Vec<ChangeEvent>> {
        // Gmail has no server push; the host drives sync on its own cadence via the
        // history-ID cursor. Nothing to drain here.
        Ok(Vec::new())
    }

    // ── sync strategies ─────────────────────────────────────────────────────────

    fn sync_full(&self, token: &str, label_id: &str) -> Result<MailboxDelta> {
        let url = if label_id == labels::ALL_MAIL_ID {
            gmail::url_messages_list_all()
        } else {
            gmail::url_messages_list(label_id)
        };
        let list: gmail::MessagesList = self.get_json(token, &url)?;
        let added = list
            .messages
            .iter()
            .map(|m| MessageRef::for_gmail(label_id, &m.id))
            .collect();
        // Baseline historyId to resume from — the profile's current head.
        let profile: gmail::Profile = self.get_json(token, &gmail::url_profile())?;
        Ok(MailboxDelta {
            added,
            removed: Vec::new(),
            flag_changes: Vec::new(),
            next_cursor: SyncCursor::from_history_id(&profile.history_id),
        })
    }

    fn sync_delta(&self, token: &str, label_id: &str, start_history: &str) -> Result<MailboxDelta> {
        let url = if label_id == labels::ALL_MAIL_ID {
            gmail::url_history_all(start_history)
        } else {
            gmail::url_history(start_history, label_id)
        };
        let hist: gmail::HistoryList = self.get_json(token, &url)?;

        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut flag_changes = Vec::new();
        for rec in &hist.history {
            for hm in &rec.messages_added {
                added.push(MessageRef::for_gmail(label_id, &hm.message.id));
            }
            for hm in &rec.messages_deleted {
                removed.push(MessageRef::for_gmail(label_id, &hm.message.id));
            }
            // A label add/remove is a flag change — recompute the full flag set from
            // the message's (post-change) labelIds Gmail includes in the record.
            for hm in rec.labels_added.iter().chain(rec.labels_removed.iter()) {
                let r = MessageRef::for_gmail(label_id, &hm.message.id);
                let flags = labels::labels_to_flags(&hm.message.label_ids);
                flag_changes.push((r, flags));
            }
        }

        // Resume from the response head; if empty (no records) keep the old cursor
        // so we never lose our place.
        let next = if hist.history_id.is_empty() {
            SyncCursor::from_history_id(start_history)
        } else {
            SyncCursor::from_history_id(&hist.history_id)
        };
        Ok(MailboxDelta {
            added,
            removed,
            flag_changes,
            next_cursor: next,
        })
    }

    // ── helpers ─────────────────────────────────────────────────────────────────

    fn token(&self) -> Result<String> {
        self.tp.oauth_token(ACCOUNT)
    }

    fn fetch_labels(&self, token: &str) -> Result<Vec<gmail::Label>> {
        let list: gmail::LabelsList = self.get_json(token, &gmail::url_labels())?;
        Ok(list.labels)
    }

    /// Resolve a mailbox name (display name or label id) back to a Gmail label id.
    /// Matches by id first, then by display name; "All Mail" ⇒ the synthetic id.
    fn resolve_label_id(&self, token: &str, name: &str) -> Result<String> {
        if name.eq_ignore_ascii_case("all mail") || name == labels::ALL_MAIL_ID {
            return Ok(labels::ALL_MAIL_ID.to_string());
        }
        let labels = self.fetch_labels(token)?;
        if let Some(l) = labels.iter().find(|l| l.id == name || l.name == name) {
            return Ok(l.id.clone());
        }
        // Fall back to using the name verbatim (system ids are their own name).
        Ok(name.to_string())
    }

    fn get_json<D: serde::de::DeserializeOwned>(&self, token: &str, url: &str) -> Result<D> {
        let resp = self.tp.http("GET", url, auth_headers(token), None)?;
        check_status(resp.status, url)?;
        serde_json::from_slice(&resp.body)
            .map_err(|e| BridgeError::Protocol(format!("decode {url}: {e}")))
    }

    fn post_json(&self, token: &str, url: &str, body: &serde_json::Value) -> Result<HttpResponse> {
        let bytes = serde_json::to_vec(body)
            .map_err(|e| BridgeError::Protocol(format!("encode body: {e}")))?;
        let mut headers = auth_headers(token);
        headers.push(("content-type".into(), "application/json".into()));
        let resp = self.tp.http("POST", url, headers, Some(bytes))?;
        check_status(resp.status, url)?;
        Ok(resp)
    }
}

fn auth_headers(token: &str) -> Vec<(String, String)> {
    vec![("authorization".into(), format!("Bearer {token}"))]
}

fn check_status(status: u16, url: &str) -> Result<()> {
    match status {
        200..=299 => Ok(()),
        401 | 403 => Err(BridgeError::Auth(format!("{status} for {url}"))),
        404 => Err(BridgeError::MailboxNotFound(url.to_string())),
        s => Err(BridgeError::Transport(format!("HTTP {s} for {url}"))),
    }
}

/// Decode a Gmail `format=raw` body, tolerating base64url with/without padding and
/// standard base64 (Gmail uses url-safe, but be defensive).
fn decode_b64url(s: &str) -> Result<Vec<u8>> {
    let trimmed: String = s.split_whitespace().collect();
    URL_SAFE_NO_PAD
        .decode(trimmed.trim_end_matches('='))
        .or_else(|_| URL_SAFE.decode(&trimmed))
        .or_else(|_| STANDARD.decode(&trimmed))
        .map_err(|e| BridgeError::Protocol(format!("base64 raw body: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// A fixture-replaying transport: exact-substring URL routing to canned bodies.
    struct MockTp {
        routes: Vec<(&'static str, String)>,
        posts: std::cell::RefCell<Vec<(String, Vec<u8>)>>,
    }

    impl MockTp {
        fn new(routes: Vec<(&'static str, &str)>) -> Self {
            Self {
                routes: routes
                    .into_iter()
                    .map(|(k, v)| (k, v.to_string()))
                    .collect(),
                posts: std::cell::RefCell::new(Vec::new()),
            }
        }
        fn body_for(&self, url: &str) -> Option<Vec<u8>> {
            self.routes
                .iter()
                .find(|(needle, _)| url.contains(needle))
                .map(|(_, body)| body.clone().into_bytes())
        }
    }

    impl Transport for MockTp {
        fn oauth_token(&self, _account: &str) -> Result<String> {
            Ok("fake-token".into())
        }
        fn http(
            &self,
            method: &str,
            url: &str,
            _headers: Vec<(String, String)>,
            body: Option<Vec<u8>>,
        ) -> Result<HttpResponse> {
            if method == "POST" {
                self.posts
                    .borrow_mut()
                    .push((url.to_string(), body.unwrap_or_default()));
                return Ok(HttpResponse {
                    status: 200,
                    body: br#"{"id":"sent-1"}"#.to_vec(),
                });
            }
            match self.body_for(url) {
                Some(b) => Ok(HttpResponse {
                    status: 200,
                    body: b,
                }),
                None => Ok(HttpResponse {
                    status: 404,
                    body: Vec::new(),
                }),
            }
        }
    }

    const LABELS: &str = r#"{"labels":[
        {"id":"INBOX","name":"INBOX","type":"system","messagesTotal":2,"messagesUnread":1},
        {"id":"SENT","name":"SENT","type":"system"},
        {"id":"TRASH","name":"TRASH","type":"system"},
        {"id":"Label_9","name":"Receipts","type":"user","messagesTotal":5},
        {"id":"STARRED","name":"STARRED","type":"system"}
    ]}"#;

    fn raw_msg_json() -> String {
        // "Subject: hi\r\n\r\nbody" base64url-encoded, with labels.
        let raw = URL_SAFE_NO_PAD.encode(b"Subject: hi\r\n\r\nbody");
        format!(r#"{{"id":"m1","labelIds":["INBOX","UNREAD","STARRED"],"raw":"{raw}"}}"#)
    }

    #[test]
    fn list_mailboxes_maps_labels_to_roles() {
        let tp = MockTp::new(vec![("/labels", LABELS)]);
        let be = GmailBackend::new(tp);
        let boxes = be.list_mailboxes().unwrap();
        let by_role: HashMap<_, _> = boxes
            .iter()
            .map(|m| (m.role.clone(), m.mailbox_ref.name.clone()))
            .collect();
        assert_eq!(by_role.get("inbox").map(String::as_str), Some("INBOX"));
        assert_eq!(by_role.get("sent").map(String::as_str), Some("SENT"));
        assert_eq!(by_role.get("trash").map(String::as_str), Some("TRASH"));
        assert_eq!(by_role.get("all").map(String::as_str), Some("All Mail"));
        // The user label is a folder (role none) shown by its display name.
        assert!(boxes.iter().any(|m| m.mailbox_ref.name == "Receipts"));
        // STARRED is a flag, never a mailbox.
        assert!(boxes.iter().all(|m| m.mailbox_ref.name != "STARRED"));
    }

    #[test]
    fn history_id_delta_round_trips() {
        let list = r#"{"messages":[{"id":"m1"},{"id":"m2"}]}"#;
        let profile = r#"{"historyId":"1000"}"#;
        let history = r#"{"history":[
            {"messagesAdded":[{"message":{"id":"m3","labelIds":["INBOX","UNREAD"]}}]},
            {"messagesDeleted":[{"message":{"id":"m1","labelIds":[]}}]},
            {"labelsAdded":[{"message":{"id":"m2","labelIds":["INBOX","STARRED"]},"labelIds":["STARRED"]}]}
        ],"historyId":"1005"}"#;
        let tp = MockTp::new(vec![
            ("/labels", LABELS),
            ("/history", history),
            ("/messages?labelIds=INBOX", list),
            ("/profile", profile),
        ]);
        let be = GmailBackend::new(tp);
        let mbox = MailboxRef {
            name: "INBOX".into(),
            uidvalidity: 1,
        };

        // Baseline sync: empty cursor ⇒ full list + profile head.
        let full = be
            .sync_mailbox(&mbox, &SyncCursor { opaque: vec![] })
            .unwrap();
        assert_eq!(full.added.len(), 2);
        assert_eq!(full.next_cursor.history_id().as_deref(), Some("1000"));

        // Feed the returned cursor back ⇒ history-ID delta path.
        let delta = be.sync_mailbox(&mbox, &full.next_cursor).unwrap();
        assert_eq!(delta.added.len(), 1, "m3 added");
        assert_eq!(delta.removed.len(), 1, "m1 removed");
        assert_eq!(delta.flag_changes.len(), 1, "m2 starred");
        // m2's recomputed flags contain Flagged (STARRED) and Seen (no UNREAD).
        let (_, flags) = &delta.flag_changes[0];
        assert!(flags.contains(&Flag::Flagged));
        assert!(flags.contains(&Flag::Seen));
        // Cursor advanced to the response head.
        assert_eq!(delta.next_cursor.history_id().as_deref(), Some("1005"));
    }

    #[test]
    fn fetch_raw_decodes_body_and_flags() {
        let raw = raw_msg_json();
        let tp = MockTp::new(vec![("/messages/m1?format=raw", &raw)]);
        let be = GmailBackend::new(tp);
        let r = MessageRef::for_gmail("INBOX", "m1");
        let msgs = be.fetch_raw(std::slice::from_ref(&r)).unwrap();
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].raw.starts_with(b"Subject: hi"));
        assert!(msgs[0].msg_flags.contains(&Flag::Flagged));
        assert!(
            !msgs[0].msg_flags.contains(&Flag::Seen),
            "UNREAD ⇒ not Seen"
        );
    }

    #[test]
    fn store_flags_posts_correct_label_ops() {
        let tp = MockTp::new(vec![]);
        let be = GmailBackend::new(tp);
        let r = MessageRef::for_gmail("INBOX", "m1");
        be.store_flags(std::slice::from_ref(&r), &[Flag::Seen], &[])
            .unwrap();
        // Inspect the POST body: marking Seen removes UNREAD.
        let posts = be.tp.posts.borrow();
        assert_eq!(posts.len(), 1);
        assert!(posts[0].0.contains("/messages/m1/modify"));
        let body = String::from_utf8(posts[0].1.clone()).unwrap();
        assert!(body.contains("UNREAD"));
        assert!(body.contains("removeLabelIds"));
    }
}
