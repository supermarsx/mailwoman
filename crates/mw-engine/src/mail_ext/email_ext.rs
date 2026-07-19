//! `Email/copy` + `Email/import` + `Email/parse` (RFC 8621 §4.7–§4.9).
//!
//! All three reuse the engine's existing blob + ingest machinery — there is no
//! new persistence path:
//! - `parse` fetches each blob ([`Engine::fetch_blob`], stored-message or
//!   uploaded-`U` namespaces) and parses it with `mw-mime`, returning the `Email`
//!   without importing it.
//! - `import` re-ingests a blob into a target mailbox via [`Engine::ingest`] —
//!   the same path sync uses, so threading, sealing, and indexing all happen.
//!   Imports land in the local cache (queryable immediately); no upstream APPEND
//!   is attempted, mirroring the draft/sent local-ingest path.
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
    /// mailbox as a new local message.
    pub(crate) async fn email_import(&self, account_id: &str, args: &Value) -> Value {
        let old_state = self
            .type_state(account_id, ChangeType::Email)
            .await
            .unwrap_or_default();
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
        let blob = self.fetch_blob(account_id, blob_id).await?.ok_or_else(|| {
            EngineError::Protocol(format!("blobId {blob_id:?} does not resolve to a blob"))
        })?;
        let sid = self
            .ingest_blob_into(account_id, &mailbox_id, blob.bytes, spec)
            .await?;
        self.created_entry(&sid).await
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
            .ingest_blob_into(account_id, &mailbox_id, blob.bytes, spec)
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
        }
        self.created_entry(&sid).await
    }

    /// Ingest raw message bytes into a target mailbox as a new engine-local
    /// message (uidvalidity 0), honoring the create spec's `keywords`/`receivedAt`.
    async fn ingest_blob_into(
        &self,
        account_id: &str,
        mailbox_id: &str,
        raw: Vec<u8>,
        spec: &Value,
    ) -> Result<String> {
        // Confirm the mailbox exists (a clean error, never a panic, otherwise).
        let mb = self
            .store()
            .get_mailbox(mailbox_id)
            .await
            .map_err(EngineError::Store)?;
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
        self.ingest(account_id, mailbox_id, &msg).await
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
    spec.get("mailboxIds")
        .and_then(Value::as_object)
        .and_then(|m| {
            m.iter()
                .find(|(_, v)| v.as_bool() == Some(true))
                .map(|(k, _)| k.clone())
        })
        .ok_or_else(|| EngineError::Protocol("entry missing a target mailboxId".into()))
}

/// A structured `notCreated` error (RFC 8621 uses `invalidProperties` for a
/// malformed create; a missing blob is surfaced as `notFound`).
fn import_error(e: &EngineError) -> Value {
    let msg = e.to_string();
    let kind = if msg.contains("does not resolve") || msg.contains("not found") {
        "notFound"
    } else {
        "invalidProperties"
    };
    json!({ "type": kind, "description": msg })
}

/// A deterministic non-zero pseudo-UID from the message bytes, so the engine's
/// `(mailbox, uidvalidity, uid)` uniqueness index is satisfied for a local
/// import while identical bytes import idempotently.
fn content_pseudo_uid(raw: &[u8]) -> u32 {
    let mut h = DefaultHasher::new();
    raw.hash(&mut h);
    (h.finish() as u32) | 1
}
