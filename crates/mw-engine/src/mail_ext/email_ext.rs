//! `Email/copy` + `Email/import` + `Email/parse` (RFC 8621 §4.7–§4.9).
//!
//! All three reuse the engine's existing blob + ingest machinery — there is no
//! new persistence path:
//! - `parse` fetches each blob ([`Engine::fetch_blob`], stored-message or
//!   uploaded-`U` namespaces) and parses it with `mw-mime`, returning the `Email`
//!   without importing it.
//! - `import` appends to the connected backend with the received date, then
//!   caches acknowledged server coordinates. Historical imports do not trigger
//!   arrival rules, vacation replies or new-mail alerts.
//! - `copy` fetches a source email's raw bytes from `fromAccountId`'s sealed
//!   cache and ingests them into this account, optionally destroying the source.

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use serde_json::{Map, Value, json};

use crate::backend::{EngineError, MessageRef, RawMailboxRef, RawMessage, Result};
use crate::change::{ChangeOp, ChangeType};
use crate::engine::Engine;
use crate::mapping::keywords_to_flags;

use super::server_fail;

impl Engine {
    /// Whether this connected backend can preserve imported dates and return
    /// durable message coordinates. POP3 and unsupported bridges stay read-only.
    pub async fn import_available(&self, account_id: &str) -> Result<bool> {
        let Some(runtime) = self.runtime(account_id) else {
            return Ok(false);
        };
        if !runtime.backend.supports_dated_import() {
            return Ok(false);
        }
        let caps = runtime.backend.capabilities().await?;
        Ok(caps.uidplus || caps.imap4rev2)
    }
    /// `Email/parse` (RFC 8621 §4.9): parse each blob as a message and return its
    /// `Email` object without importing it.
    pub(crate) async fn email_parse(&self, account_id: &str, args: &Value) -> Value {
        let empty = Vec::new();
        let blob_ids = args
            .get("blobIds")
            .and_then(Value::as_array)
            .unwrap_or(&empty);

        let mut parsed = Map::new();
        let mut not_parsable = Vec::new();
        let mut not_found = Vec::new();
        for bid in blob_ids.iter().filter_map(Value::as_str) {
            match self.fetch_blob(account_id, bid).await {
                Ok(Some(blob)) => match mw_mime::parse(&blob.bytes) {
                    Ok(p) => {
                        let mut email =
                            serde_json::to_value(&p.email).unwrap_or_else(|_| json!({}));
                        if let Some(o) = email.as_object_mut() {
                            o.insert("blobId".into(), json!(bid));
                            o.insert("size".into(), json!(blob.bytes.len() as u64));
                        }
                        parsed.insert(bid.to_string(), email);
                    }
                    Err(_) => not_parsable.push(json!(bid)),
                },
                Ok(None) => not_found.push(json!(bid)),
                Err(e) => return server_fail(e),
            }
        }
        json!({
            "accountId": account_id,
            "parsed": Value::Object(parsed),
            "notParsable": not_parsable,
            "notFound": not_found
        })
    }

    /// `Email/import` (RFC 8621 §4.8): ingest each referenced blob into its target
    /// mailbox as a durable upstream message.
    pub(crate) async fn email_import(&self, account_id: &str, args: &Value) -> Value {
        let old_state = self
            .type_state(account_id, ChangeType::Email)
            .await
            .unwrap_or_default();
        match args.get("ifInState") {
            Some(Value::String(state)) if state != &old_state => {
                return json!({"type":"stateMismatch"});
            }
            None | Some(Value::Null) | Some(Value::String(_)) => {}
            _ => {
                return json!({"type":"invalidArguments","description":"ifInState must be a string or null"});
            }
        }
        if !args.get("emails").is_some_and(Value::is_object) {
            return json!({"type":"invalidArguments","description":"emails must be an object"});
        }
        let mut created = Map::new();
        let mut not_created = Map::new();

        if let Some(emails) = args.get("emails").and_then(Value::as_object) {
            for (cid, spec) in emails {
                match self.import_one(account_id, spec).await {
                    Ok(v) => {
                        created.insert(cid.clone(), v);
                    }
                    Err(e) => {
                        not_created.insert(cid.clone(), import_error(&e));
                    }
                }
            }
        }
        if !created.is_empty() && self.resync(account_id).await.is_err() {
            // APPEND was acknowledged: a later refresh failure cannot undo success.
            tracing::warn!("import completed; mailbox refresh deferred");
        }
        let new_state = self
            .type_state(account_id, ChangeType::Email)
            .await
            .unwrap_or_default();
        let mut resp = json!({
            "accountId": account_id,
            "oldState": old_state,
            "newState": new_state,
            "created": Value::Object(created),
        });
        if !not_created.is_empty() {
            resp["notCreated"] = Value::Object(not_created);
        }
        resp
    }

    /// `Email/copy` (RFC 8621 §4.7): copy messages from `fromAccountId` into this
    /// account's target mailbox, optionally destroying the originals.
    pub(crate) async fn email_copy(&self, account_id: &str, args: &Value) -> Value {
        let from_account = args
            .get("fromAccountId")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if from_account.is_empty() {
            return json!({
                "type": "invalidArguments",
                "description": "Email/copy requires fromAccountId"
            });
        }
        // This engine entrypoint has exactly one authenticated account authority.
        // A request-supplied account id is not a delegation or linked-account grant.
        if from_account != account_id {
            return json!({"type":"fromAccountNotFound"});
        }
        let old_state = self
            .type_state(account_id, ChangeType::Email)
            .await
            .unwrap_or_default();
        let destroy_original = args
            .get("onSuccessDestroyOriginal")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let mut created = Map::new();
        let mut not_created = Map::new();
        if let Some(creates) = args.get("create").and_then(Value::as_object) {
            for (cid, spec) in creates {
                match self
                    .copy_one(from_account, account_id, spec, destroy_original)
                    .await
                {
                    Ok(v) => {
                        created.insert(cid.clone(), v);
                    }
                    Err(e) => {
                        not_created.insert(cid.clone(), import_error(&e));
                    }
                }
            }
        }
        let new_state = self
            .type_state(account_id, ChangeType::Email)
            .await
            .unwrap_or_default();
        let mut resp = json!({
            "fromAccountId": from_account,
            "accountId": account_id,
            "oldState": old_state,
            "newState": new_state,
            "created": Value::Object(created),
        });
        if !not_created.is_empty() {
            resp["notCreated"] = Value::Object(not_created);
        }
        resp
    }

    // ---- shared create paths -------------------------------------------

    async fn import_one(&self, account_id: &str, spec: &Value) -> Result<Value> {
        let blob_id = spec
            .get("blobId")
            .and_then(Value::as_str)
            .ok_or_else(|| EngineError::Protocol("import entry missing blobId".into()))?;
        let mailbox_id = target_mailbox(spec)?;
        let mailbox = self.owned_import_mailbox(account_id, &mailbox_id).await?;
        let blob = self
            .fetch_blob(account_id, blob_id)
            .await?
            .ok_or(EngineError::Store(mw_store::StoreError::NotFound))?;
        let sid = self
            .append_import(account_id, &mailbox, blob.bytes, spec)
            .await?;
        self.created_entry(&sid)
            .await
            .map_err(|_| EngineError::Transport("import cache confirmation failed".into()))
    }

    async fn copy_one(
        &self,
        from_account: &str,
        account_id: &str,
        spec: &Value,
        destroy_original: bool,
    ) -> Result<Value> {
        let source_id = spec
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| EngineError::Protocol("copy entry missing source email id".into()))?;
        let mailbox_id = target_mailbox(spec)?;
        let mailbox = self.owned_import_mailbox(account_id, &mailbox_id).await?;
        // Source raw bytes from the other account's sealed cache (whole message).
        let blob = self
            .fetch_blob(from_account, source_id)
            .await?
            .ok_or_else(|| {
                EngineError::Protocol(format!(
                    "source email {source_id:?} not found in account {from_account:?}"
                ))
            })?;
        let sid = self
            .ingest_blob_into(account_id, &mailbox, blob.bytes, spec)
            .await?;
        if destroy_original {
            self.store()
                .delete_message(source_id)
                .await
                .map_err(EngineError::Store)?;
            let _ = self.search().delete(source_id);
            self.record_change(
                from_account,
                ChangeType::Email,
                source_id,
                ChangeOp::Destroyed,
            )
            .await?;
            self.emit_event(
                crate::change::EventKind::MessageDeleted,
                from_account,
                &[source_id.to_string()],
            );
        }
        self.created_entry(&sid).await
    }

    async fn owned_import_mailbox(
        &self,
        account_id: &str,
        mailbox_id: &str,
    ) -> Result<mw_store::Mailbox> {
        let mailbox = self.store().get_mailbox(mailbox_id).await?;
        if mailbox.account_id != account_id {
            return Err(EngineError::Store(mw_store::StoreError::NotFound));
        }
        Ok(mailbox)
    }

    async fn append_import(
        &self,
        account_id: &str,
        mailbox: &mw_store::Mailbox,
        raw: Vec<u8>,
        spec: &Value,
    ) -> Result<String> {
        let parsed = mw_mime::parse(&raw)
            .map_err(|_| EngineError::Protocol("invalid import message".into()))?;
        let mut keywords = HashMap::new();
        match spec.get("keywords") {
            None => {}
            Some(Value::Object(values)) => {
                for (key, value) in values {
                    if value != &Value::Bool(true) || !valid_import_keyword(key) {
                        return Err(EngineError::Protocol("invalid import keywords".into()));
                    }
                    keywords.insert(key.clone(), true);
                }
            }
            _ => return Err(EngineError::Protocol("keywords must be an object".into())),
        }
        let date = match spec.get("receivedAt") {
            Some(Value::String(date)) => date.clone(),
            None => parsed
                .email
                .received_at
                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
            _ => {
                return Err(EngineError::Protocol(
                    "receivedAt must be a UTC timestamp".into(),
                ));
            }
        };
        let date = chrono::DateTime::parse_from_rfc3339(&date)
            .map_err(|_| EngineError::Protocol("invalid receivedAt".into()))?;
        if date.offset().local_minus_utc() != 0 {
            return Err(EngineError::Protocol("receivedAt must be UTC".into()));
        }
        let received_at = date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let runtime = self
            .runtime(account_id)
            .ok_or_else(|| EngineError::Unsupported("account is not connected".into()))?;
        if !runtime.backend.supports_dated_import() {
            return Err(EngineError::Unsupported(
                "backend does not support durable import".into(),
            ));
        }
        let caps = runtime.backend.capabilities().await?;
        if !caps.uidplus && !caps.imap4rev2 {
            return Err(EngineError::Unsupported(
                "import requires UIDPLUS or IMAP4rev2".into(),
            ));
        }
        let flags = keywords_to_flags(&keywords);
        let reference = runtime
            .backend
            .append_with_date(
                &RawMailboxRef {
                    name: mailbox.name.clone(),
                    uidvalidity: mailbox.uidvalidity,
                },
                &raw,
                &flags,
                &received_at,
            )
            .await?;
        if !matches!(
            reference,
            MessageRef::Imap {
                uid: 1..,
                uidvalidity: 1..,
                ..
            }
        ) {
            return Err(EngineError::Protocol(
                "import completed without durable coordinates; refresh before retrying".into(),
            ));
        }
        self.ingest_import(
            account_id,
            &mailbox.id,
            &RawMessage {
                message_ref: reference,
                raw,
                flags,
                internaldate: Some(received_at),
            },
        )
        .await
        .map_err(|_| EngineError::Transport("import cache confirmation failed".into()))
    }

    /// Ingest raw message bytes into a target mailbox as a new engine-local
    /// message (uidvalidity 0), honoring the create spec's `keywords`/`receivedAt`.
    async fn ingest_blob_into(
        &self,
        account_id: &str,
        mb: &mw_store::Mailbox,
        raw: Vec<u8>,
        spec: &Value,
    ) -> Result<String> {
        let keywords: HashMap<String, bool> = spec
            .get("keywords")
            .and_then(Value::as_object)
            .map(|kw| {
                kw.iter()
                    .filter_map(|(k, v)| v.as_bool().map(|b| (k.clone(), b)))
                    .collect()
            })
            .unwrap_or_default();
        let flags = keywords_to_flags(&keywords);
        let received_at = spec
            .get("receivedAt")
            .and_then(Value::as_str)
            .map(String::from)
            .or_else(|| Some(chrono::Utc::now().to_rfc3339()));
        let msg = RawMessage {
            message_ref: MessageRef::Imap {
                mailbox: RawMailboxRef {
                    name: mb.name.clone(),
                    uidvalidity: 0,
                },
                uidvalidity: 0,
                uid: content_pseudo_uid(&raw),
            },
            raw,
            flags,
            internaldate: received_at,
        };
        self.ingest(account_id, &mb.id, &msg).await
    }

    /// The `created` entry (`{id, blobId, threadId, size}`) for a freshly ingested
    /// message.
    async fn created_entry(&self, sid: &str) -> Result<Value> {
        let msg = self
            .store()
            .get_message(sid)
            .await
            .map_err(EngineError::Store)?;
        Ok(json!({
            "id": sid,
            "blobId": sid,
            "threadId": msg.thread_id,
            "size": msg.size,
        }))
    }
}

/// The single `mailboxIds` target of a copy/import create spec.
fn target_mailbox(spec: &Value) -> Result<String> {
    let ids = spec
        .get("mailboxIds")
        .and_then(Value::as_object)
        .ok_or_else(|| EngineError::Protocol("entry missing a target mailboxId".into()))?;
    if ids.len() != 1 || ids.values().any(|value| value != &Value::Bool(true)) {
        return Err(EngineError::Protocol(
            "exactly one true target mailboxId is supported".into(),
        ));
    }
    Ok(ids.keys().next().expect("one target").clone())
}

fn valid_import_keyword(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 255
        && !key.eq_ignore_ascii_case("$recent")
        && !key.eq_ignore_ascii_case("$deleted")
        && key
            .bytes()
            .all(|byte| (b'!'..=b'~').contains(&byte) && !b"(){]%*\"\\".contains(&byte))
}

/// A structured `notCreated` error (RFC 8621 uses `invalidProperties` for a
/// malformed create; a missing blob is surfaced as `notFound`).
fn import_error(e: &EngineError) -> Value {
    match e {
        EngineError::Unsupported(_) => {
            json!({"type":"forbidden","description":"This backend does not support durable import."})
        }
        EngineError::Store(mw_store::StoreError::NotFound) | EngineError::MailboxNotFound(_) => {
            json!({"type":"notFound","description":"The requested item is unavailable in this account."})
        }
        EngineError::Protocol(message)
            if matches!(
                message.as_str(),
                "import entry missing blobId"
                    | "copy entry missing source email id"
                    | "entry missing a target mailboxId"
                    | "exactly one true target mailboxId is supported"
                    | "invalid import message"
                    | "invalid import keywords"
                    | "keywords must be an object"
                    | "receivedAt must be a UTC timestamp"
                    | "invalid receivedAt"
                    | "receivedAt must be UTC"
            ) =>
        {
            json!({"type":"invalidProperties","description":message})
        }
        _ => {
            json!({"type":"serverFail","description":"The import could not be confirmed. Refresh the destination before retrying; some messages may already be saved."})
        }
    }
}

/// A deterministic non-zero pseudo-UID from the message bytes, so the engine's
/// `(mailbox, uidvalidity, uid)` uniqueness index is satisfied for a local
/// import while identical bytes import idempotently.
fn content_pseudo_uid(raw: &[u8]) -> u32 {
    let mut h = DefaultHasher::new();
    raw.hash(&mut h);
    (h.finish() as u32) | 1
}

#[cfg(test)]
mod import_error_tests {
    use super::*;
    #[test]
    fn unexpected_backend_errors_are_uncertain_and_redacted() {
        for error in [
            EngineError::Protocol("secret upstream reply".into()),
            EngineError::Transport("private host".into()),
        ] {
            let result = import_error(&error);
            assert_eq!(result["type"], "serverFail");
            assert!(!result.to_string().contains(&error.to_string()));
            assert!(
                result["description"]
                    .as_str()
                    .unwrap()
                    .contains("before retrying")
            );
        }
        assert_eq!(
            import_error(&EngineError::Protocol("invalid import keywords".into()))["type"],
            "invalidProperties"
        );
    }
}
