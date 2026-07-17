//! IMAP ACL (RFC 4314) + METADATA (RFC 5464) on the engine's JMAP-style method
//! surface (t13 plan §6 E7). These ride the existing `handle_jmap` envelope
//! exactly like `SecurityVerdict/get` — no new admin endpoint, no new auth
//! surface, and **no persistence**: every call reads through to the account's
//! [`AccountBackend`](crate::backend::AccountBackend), which delegates to the
//! upstream IMAP server. The server is the real enforcement point (RFC 4314):
//! `MailboxRights/set` (SETACL/DELETEACL) succeeds or fails per the authenticated
//! user's own rights server-side — the engine does not double-enforce a policy of
//! its own. `MailboxRights/get` surfaces `myRights` honestly so the web UI (E8)
//! can gate the SETACL affordance on the caller holding the `a` (admin) right.
//!
//! The frozen [`AclEntry`](crate::backend::AclEntry) /
//! [`MetadataEntry`](crate::backend::MetadataEntry) structs (E0) serialize
//! directly onto the surface — no separate JMAP type is required.
//!
//! ## Method contract (the E8 web-client / E9 mount bind against these shapes)
//!
//! - **`MailboxRights/get`** `{ mailboxId }` →
//!   `{ accountId, state, mailboxId, myRights: "<rfc4314 rights>", acl: [{identifier, rights}] }`
//! - **`MailboxRights/set`** `{ mailboxId, identifier, rights? }` — `rights` a
//!   non-null string GRANTS (SETACL); `rights` null/absent REVOKES (DELETEACL) →
//!   `{ accountId, state, mailboxId, updated: { "<identifier>": null } }` on
//!   success, or `{ …, notUpdated: { "<identifier>": {type, description} } }`.
//! - **`ServerMetadata/get`** `{ mailboxId?, entries: [".."] }` — `mailboxId`
//!   null/absent = server-level annotations (RFC 5464 empty-mailbox scope),
//!   `Some` = a mailbox's annotations →
//!   `{ accountId, state, mailboxId, list: [{entry, value}] }` (`value` null = NIL).
//! - **`ServerMetadata/set`** `{ mailboxId?, entry, value? }` — `value` a non-null
//!   string SETS it; null/absent REMOVES it (RFC 5464 NIL) →
//!   `{ accountId, state, mailboxId, updated: { "<entry>": null } }` on success,
//!   or `{ …, notUpdated: { "<entry>": {type, description} } }`.
//!
//! A backend that does not advertise ACL/METADATA (POP3, plugin/bridge, or any
//! backend inheriting the E0 default trait impls) returns a clean
//! `{ "type": "unsupported", … }` method-level error — never a panic.

use serde_json::{Value, json};

use crate::account::AccountRuntime;
use crate::backend::{EngineError, RawMailboxRef, Result};
use crate::engine::Engine;

impl Engine {
    /// `MailboxRights/get {mailboxId}` — the authenticated user's own rights
    /// (`MYRIGHTS`) plus the full ACL list (`GETACL`) for one mailbox, read
    /// through to the upstream server.
    pub(crate) async fn mailbox_rights_get(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        args: &Value,
    ) -> Value {
        let Some(mailbox_id) = args.get("mailboxId").and_then(Value::as_str) else {
            return invalid_args("mailboxId is required");
        };
        let mbox = match self.resolve_mbox_ref(mailbox_id).await {
            Ok(m) => m,
            Err(e) => return method_error(&e),
        };
        let my_rights = match rt.backend.my_rights(&mbox).await {
            Ok(r) => r,
            Err(e) => return method_error(&e),
        };
        let acl = match rt.backend.get_acl(&mbox).await {
            Ok(a) => a,
            Err(e) => return method_error(&e),
        };
        json!({
            "accountId": account_id,
            "state": self.session_state(account_id).await,
            "mailboxId": mailbox_id,
            "myRights": my_rights,
            "acl": acl,
        })
    }

    /// `MailboxRights/set {mailboxId, identifier, rights?}` — grant (SETACL) when
    /// `rights` is a non-null string, or revoke (DELETEACL) when it is null/absent.
    /// The upstream server authorizes the change per the caller's own rights.
    pub(crate) async fn mailbox_rights_set(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        args: &Value,
    ) -> Value {
        let Some(mailbox_id) = args.get("mailboxId").and_then(Value::as_str) else {
            return invalid_args("mailboxId is required");
        };
        let Some(identifier) = args.get("identifier").and_then(Value::as_str) else {
            return invalid_args("identifier is required");
        };
        // `rights` present (non-null) → SETACL grant; null/absent → DELETEACL revoke.
        let rights = args.get("rights").and_then(Value::as_str);
        let mbox = match self.resolve_mbox_ref(mailbox_id).await {
            Ok(m) => m,
            Err(e) => return method_error(&e),
        };
        let outcome = match rights {
            Some(r) => rt.backend.set_acl(&mbox, identifier, r).await,
            None => rt.backend.delete_acl(&mbox, identifier).await,
        };
        let state = self.session_state(account_id).await;
        match outcome {
            Ok(()) => json!({
                "accountId": account_id,
                "state": state,
                "mailboxId": mailbox_id,
                "updated": { identifier: Value::Null },
            }),
            Err(e) => json!({
                "accountId": account_id,
                "state": state,
                "mailboxId": mailbox_id,
                "notUpdated": { identifier: method_error(&e) },
            }),
        }
    }

    /// `ServerMetadata/get {mailboxId?, entries}` — read annotation `entries`
    /// (`GETMETADATA`). `mailboxId` null/absent targets server-level metadata; a
    /// value targets that mailbox's annotations.
    pub(crate) async fn server_metadata_get(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        args: &Value,
    ) -> Value {
        let entries: Vec<String> = args
            .get("entries")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();
        let mailbox_id = args.get("mailboxId").and_then(Value::as_str);
        let mbox = match self.resolve_opt_mbox_ref(mailbox_id).await {
            Ok(m) => m,
            Err(e) => return method_error(&e),
        };
        let list = match rt.backend.get_metadata(mbox.as_ref(), &entries).await {
            Ok(v) => v,
            Err(e) => return method_error(&e),
        };
        json!({
            "accountId": account_id,
            "state": self.session_state(account_id).await,
            "mailboxId": mailbox_id,
            "list": list,
        })
    }

    /// `ServerMetadata/set {mailboxId?, entry, value?}` — set one annotation
    /// (`SETMETADATA`) when `value` is a non-null string, or remove it (RFC 5464
    /// NIL) when null/absent. Scope as in `server_metadata_get`.
    pub(crate) async fn server_metadata_set(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        args: &Value,
    ) -> Value {
        let Some(entry) = args.get("entry").and_then(Value::as_str) else {
            return invalid_args("entry is required");
        };
        // `value` present (non-null) → set; null/absent → remove (RFC 5464 NIL).
        let value = args.get("value").and_then(Value::as_str);
        let mailbox_id = args.get("mailboxId").and_then(Value::as_str);
        let mbox = match self.resolve_opt_mbox_ref(mailbox_id).await {
            Ok(m) => m,
            Err(e) => return method_error(&e),
        };
        let state = self.session_state(account_id).await;
        match rt.backend.set_metadata(mbox.as_ref(), entry, value).await {
            Ok(()) => json!({
                "accountId": account_id,
                "state": state,
                "mailboxId": mailbox_id,
                "updated": { entry: Value::Null },
            }),
            Err(e) => json!({
                "accountId": account_id,
                "state": state,
                "mailboxId": mailbox_id,
                "notUpdated": { entry: method_error(&e) },
            }),
        }
    }

    /// Resolve a JMAP mailbox id to the backend's [`RawMailboxRef`] (name +
    /// uidvalidity), so ACL/METADATA commands address the upstream folder.
    async fn resolve_mbox_ref(&self, mailbox_id: &str) -> Result<RawMailboxRef> {
        let m = self.store().get_mailbox(mailbox_id).await?;
        Ok(RawMailboxRef {
            name: m.name,
            uidvalidity: m.uidvalidity,
        })
    }

    /// Resolve an optional mailbox id: `None` (server-level METADATA scope) stays
    /// `None`; `Some(id)` resolves via [`Self::resolve_mbox_ref`].
    async fn resolve_opt_mbox_ref(
        &self,
        mailbox_id: Option<&str>,
    ) -> Result<Option<RawMailboxRef>> {
        match mailbox_id {
            Some(id) => Ok(Some(self.resolve_mbox_ref(id).await?)),
            None => Ok(None),
        }
    }
}

/// A JMAP method-level error object. An `Unsupported` backend (ACL/METADATA not
/// advertised) surfaces a clean `"unsupported"` result, never a panic; anything
/// else is a `"serverFail"`.
fn method_error(e: &EngineError) -> Value {
    let ty = match e {
        EngineError::Unsupported(_) => "unsupported",
        _ => "serverFail",
    };
    json!({ "type": ty, "description": e.to_string() })
}

/// A JMAP `invalidArguments` method-level error (a required arg was missing).
fn invalid_args(msg: &str) -> Value {
    json!({ "type": "invalidArguments", "description": msg })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use serde_json::{Value, json};

    use crate::account::AccountRuntime;
    use crate::backend::{
        AccountBackend, AclEntry, BackendCaps, ChangeSink, EngineError, Flag, MailboxDelta,
        MessageRef, MetadataEntry, MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, Result,
        SyncCursor, WatchHandle,
    };
    use crate::{Engine, MailSubmitter};
    use mw_smtp::{Outgoing, SubmissionResult};
    use mw_store::{AccountKind, Credentials, MailboxUpsert, NewAccount, ServerKey, Store};

    // A backend that speaks ACL/METADATA, returning known fixtures so the tests
    // pin the request→response envelope shape.
    struct AclBackend;

    #[async_trait]
    impl AccountBackend for AclBackend {
        async fn capabilities(&self) -> Result<BackendCaps> {
            Ok(BackendCaps {
                acl: true,
                metadata: true,
                ..BackendCaps::default()
            })
        }
        async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
            Ok(Vec::new())
        }
        async fn sync_mailbox(&self, _m: &RawMailboxRef, c: &SyncCursor) -> Result<MailboxDelta> {
            Ok(MailboxDelta {
                added: Vec::new(),
                flag_changes: Vec::new(),
                removed: Vec::new(),
                next_cursor: c.clone(),
            })
        }
        async fn fetch_raw(&self, _r: &[MessageRef]) -> Result<Vec<RawMessage>> {
            Ok(Vec::new())
        }
        async fn store_flags(&self, _r: &[MessageRef], _a: &[Flag], _d: &[Flag]) -> Result<()> {
            Ok(())
        }
        async fn move_messages(
            &self,
            _r: &[MessageRef],
            _to: &RawMailboxRef,
        ) -> Result<MoveOutcome> {
            Err(EngineError::Unsupported("noop".into()))
        }
        async fn append(&self, _m: &RawMailboxRef, _raw: &[u8], _f: &[Flag]) -> Result<MessageRef> {
            Err(EngineError::Unsupported("noop".into()))
        }
        async fn watch(&self, _s: ChangeSink) -> Result<WatchHandle> {
            Err(EngineError::Unsupported("noop".into()))
        }

        // ── ACL / METADATA overrides returning known fixtures ──
        async fn get_acl(&self, _mbox: &RawMailboxRef) -> Result<Vec<AclEntry>> {
            Ok(vec![
                AclEntry {
                    identifier: "alice".into(),
                    rights: "lrswipkxtea".into(),
                },
                AclEntry {
                    identifier: "anyone".into(),
                    rights: "lr".into(),
                },
            ])
        }
        async fn set_acl(&self, _mbox: &RawMailboxRef, _id: &str, _rights: &str) -> Result<()> {
            Ok(())
        }
        async fn delete_acl(&self, _mbox: &RawMailboxRef, _id: &str) -> Result<()> {
            Ok(())
        }
        async fn my_rights(&self, _mbox: &RawMailboxRef) -> Result<String> {
            Ok("lrswipkxtea".into())
        }
        async fn get_metadata(
            &self,
            _mbox: Option<&RawMailboxRef>,
            _entries: &[String],
        ) -> Result<Vec<MetadataEntry>> {
            Ok(vec![MetadataEntry {
                entry: "/shared/comment".into(),
                value: Some("hello".into()),
            }])
        }
        async fn set_metadata(
            &self,
            _mbox: Option<&RawMailboxRef>,
            _entry: &str,
            _value: Option<&str>,
        ) -> Result<()> {
            Ok(())
        }
    }

    // A backend that inherits the E0 default (Unsupported) ACL/METADATA impls.
    struct UnsupportedBackend;

    #[async_trait]
    impl AccountBackend for UnsupportedBackend {
        async fn capabilities(&self) -> Result<BackendCaps> {
            Ok(BackendCaps::default())
        }
        async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
            Ok(Vec::new())
        }
        async fn sync_mailbox(&self, _m: &RawMailboxRef, c: &SyncCursor) -> Result<MailboxDelta> {
            Ok(MailboxDelta {
                added: Vec::new(),
                flag_changes: Vec::new(),
                removed: Vec::new(),
                next_cursor: c.clone(),
            })
        }
        async fn fetch_raw(&self, _r: &[MessageRef]) -> Result<Vec<RawMessage>> {
            Ok(Vec::new())
        }
        async fn store_flags(&self, _r: &[MessageRef], _a: &[Flag], _d: &[Flag]) -> Result<()> {
            Ok(())
        }
        async fn move_messages(
            &self,
            _r: &[MessageRef],
            _to: &RawMailboxRef,
        ) -> Result<MoveOutcome> {
            Err(EngineError::Unsupported("noop".into()))
        }
        async fn append(&self, _m: &RawMailboxRef, _raw: &[u8], _f: &[Flag]) -> Result<MessageRef> {
            Err(EngineError::Unsupported("noop".into()))
        }
        async fn watch(&self, _s: ChangeSink) -> Result<WatchHandle> {
            Err(EngineError::Unsupported("noop".into()))
        }
        // ACL/METADATA methods intentionally NOT overridden → E0 defaults apply.
    }

    struct NoopSubmitter;

    #[async_trait]
    impl MailSubmitter for NoopSubmitter {
        async fn submit(&self, _msg: Outgoing) -> Result<SubmissionResult> {
            Ok(SubmissionResult {
                accepted: Vec::new(),
                rejected: Vec::new(),
            })
        }
    }

    struct Harness {
        engine: Arc<Engine>,
        account_id: String,
        mailbox_id: String,
    }

    async fn setup(backend: Arc<dyn AccountBackend>) -> Harness {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let account_id = store
            .create_account(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host: "imap.example.org",
                    port: 993,
                    tls: "implicit",
                    username: "me@example.org",
                    sync_policy_json: "{}",
                },
                &Credentials {
                    username: "me@example.org".into(),
                    password: "pw".into(),
                },
            )
            .await
            .unwrap();
        let mailbox_id = store
            .upsert_mailbox(&MailboxUpsert {
                account_id: &account_id,
                name: "Shared/Team",
                role: None,
                uidvalidity: 42,
                uidnext: 1,
                highestmodseq: 0,
                total: 0,
                unread: 0,
                parent_id: None,
            })
            .await
            .unwrap();
        let engine = Arc::new(Engine::new(store));
        let runtime = AccountRuntime::new(
            backend,
            Arc::new(NoopSubmitter) as Arc<dyn MailSubmitter>,
            "me@example.org",
        );
        engine.register_backend(account_id.clone(), runtime);
        Harness {
            engine,
            account_id,
            mailbox_id,
        }
    }

    impl Harness {
        async fn call(&self, method: &str, args: Value) -> Value {
            let req = json!({ "methodCalls": [[method, args, "c0"]] });
            let resp = self.engine.handle_jmap(&self.account_id, &req).await;
            // Confirm the invocation name + call-id are echoed (dispatch reached).
            assert_eq!(resp["methodResponses"][0][0], json!(method));
            assert_eq!(resp["methodResponses"][0][2], json!("c0"));
            resp["methodResponses"][0][1].clone()
        }
    }

    #[tokio::test]
    async fn mailbox_rights_get_returns_my_rights_and_acl() {
        let h = setup(Arc::new(AclBackend)).await;
        let r = h
            .call("MailboxRights/get", json!({ "mailboxId": h.mailbox_id }))
            .await;
        assert_eq!(r["accountId"], json!(h.account_id));
        assert_eq!(r["mailboxId"], json!(h.mailbox_id));
        assert_eq!(r["myRights"], json!("lrswipkxtea"));
        assert!(r["state"].is_string());
        // AclEntry serializes directly: {identifier, rights}.
        assert_eq!(r["acl"][0]["identifier"], json!("alice"));
        assert_eq!(r["acl"][0]["rights"], json!("lrswipkxtea"));
        assert_eq!(r["acl"][1]["identifier"], json!("anyone"));
        assert_eq!(r["acl"][1]["rights"], json!("lr"));
    }

    #[tokio::test]
    async fn mailbox_rights_set_grant_and_revoke() {
        let h = setup(Arc::new(AclBackend)).await;
        // Grant (SETACL): rights present.
        let grant = h
            .call(
                "MailboxRights/set",
                json!({ "mailboxId": h.mailbox_id, "identifier": "bob", "rights": "lrs" }),
            )
            .await;
        assert_eq!(grant["mailboxId"], json!(h.mailbox_id));
        assert_eq!(grant["updated"]["bob"], Value::Null);
        assert!(grant.get("notUpdated").is_none());
        // Revoke (DELETEACL): rights absent.
        let revoke = h
            .call(
                "MailboxRights/set",
                json!({ "mailboxId": h.mailbox_id, "identifier": "bob" }),
            )
            .await;
        assert_eq!(revoke["updated"]["bob"], Value::Null);
    }

    #[tokio::test]
    async fn server_metadata_get_server_level_and_mailbox_scope() {
        let h = setup(Arc::new(AclBackend)).await;
        // Server-level scope: no mailboxId.
        let srv = h
            .call(
                "ServerMetadata/get",
                json!({ "entries": ["/shared/comment"] }),
            )
            .await;
        assert_eq!(srv["accountId"], json!(h.account_id));
        assert_eq!(srv["mailboxId"], Value::Null);
        assert_eq!(srv["list"][0]["entry"], json!("/shared/comment"));
        assert_eq!(srv["list"][0]["value"], json!("hello"));
        // Mailbox scope: mailboxId present, echoed back.
        let mbx = h
            .call(
                "ServerMetadata/get",
                json!({ "mailboxId": h.mailbox_id, "entries": ["/shared/comment"] }),
            )
            .await;
        assert_eq!(mbx["mailboxId"], json!(h.mailbox_id));
    }

    #[tokio::test]
    async fn server_metadata_set_and_remove() {
        let h = setup(Arc::new(AclBackend)).await;
        let set = h
            .call(
                "ServerMetadata/set",
                json!({ "entry": "/shared/comment", "value": "note" }),
            )
            .await;
        assert_eq!(set["updated"]["/shared/comment"], Value::Null);
        assert!(set.get("notUpdated").is_none());
        // Remove (NIL): value absent.
        let del = h
            .call(
                "ServerMetadata/set",
                json!({ "mailboxId": h.mailbox_id, "entry": "/shared/comment" }),
            )
            .await;
        assert_eq!(del["updated"]["/shared/comment"], Value::Null);
        assert_eq!(del["mailboxId"], json!(h.mailbox_id));
    }

    #[tokio::test]
    async fn unsupported_backend_returns_clean_error_not_panic() {
        let h = setup(Arc::new(UnsupportedBackend)).await;
        // get: the Unsupported surfaces as a method-level error object.
        let get = h
            .call("MailboxRights/get", json!({ "mailboxId": h.mailbox_id }))
            .await;
        assert_eq!(get["type"], json!("unsupported"));
        assert!(get["description"].is_string());
        // set: the failure lands in notUpdated as an "unsupported" error.
        let set = h
            .call(
                "MailboxRights/set",
                json!({ "mailboxId": h.mailbox_id, "identifier": "bob", "rights": "lrs" }),
            )
            .await;
        assert_eq!(set["notUpdated"]["bob"]["type"], json!("unsupported"));
        // metadata get + set likewise degrade cleanly.
        let mget = h
            .call(
                "ServerMetadata/get",
                json!({ "entries": ["/shared/comment"] }),
            )
            .await;
        assert_eq!(mget["type"], json!("unsupported"));
        let mset = h
            .call(
                "ServerMetadata/set",
                json!({ "entry": "/shared/comment", "value": "x" }),
            )
            .await;
        assert_eq!(
            mset["notUpdated"]["/shared/comment"]["type"],
            json!("unsupported")
        );
    }

    #[tokio::test]
    async fn missing_required_args_are_invalid_arguments() {
        let h = setup(Arc::new(AclBackend)).await;
        let no_mbox = h.call("MailboxRights/get", json!({})).await;
        assert_eq!(no_mbox["type"], json!("invalidArguments"));
        let no_id = h
            .call("MailboxRights/set", json!({ "mailboxId": h.mailbox_id }))
            .await;
        assert_eq!(no_id["type"], json!("invalidArguments"));
        let no_entry = h.call("ServerMetadata/set", json!({})).await;
        assert_eq!(no_entry["type"], json!("invalidArguments"));
    }
}
