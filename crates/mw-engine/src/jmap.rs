//! The engine's JMAP surface (plan §2.2): answers the exact `Mailbox/get` /
//! `Email/query` / `Email/get` / `Email/set` / `EmailSubmission/set` shapes the
//! `apps/web` client already speaks, byte-compatibly with `mw-mock-jmap` and the
//! `mw-jmap` types, including result-reference resolution (RFC 8620 §3.7).
//!
//! The web app cannot tell engine mode from proxy mode: same routes, same
//! request/response JSON. Everything here is served from the local `mw-store`
//! cache the sync engine keeps fresh; sends fan out through the account's
//! `MailSubmitter` and are filed back into `Sent`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use mw_mime::{ComposeRequest, EmailAddress};
use serde_json::{Map, Value, json};

use crate::account::AccountRuntime;
use crate::backend::{EngineError, Flag, MessageRef, RawMailboxRef, RawMessage, Result};
use crate::engine::{Engine, SESSION_STATE};
use crate::mapping::{
    display_name, flag_delta, flags_from_json, flags_to_json, flags_to_keywords, keywords_to_flags,
    role_sort_order,
};

/// Build the JMAP [`Session`](mw_jmap::Session) resource for a connected account,
/// advertising core + mail + submission and pointing every URL back at us.
pub fn session_json(account_id: &str, username: &str) -> Value {
    json!({
        "capabilities": {
            "urn:ietf:params:jmap:core": { "maxSizeUpload": 50_000_000, "maxConcurrentRequests": 4 },
            "urn:ietf:params:jmap:mail": {},
            "urn:ietf:params:jmap:submission": {}
        },
        "accounts": {
            account_id: { "name": username, "isPersonal": true, "isReadOnly": false, "accountCapabilities": {} }
        },
        "primaryAccounts": {
            "urn:ietf:params:jmap:mail": account_id,
            "urn:ietf:params:jmap:submission": account_id
        },
        "username": username,
        "apiUrl": "/jmap/api",
        "downloadUrl": "/jmap/download/{accountId}/{blobId}/{name}",
        "uploadUrl": "/jmap/upload/{accountId}",
        "eventSourceUrl": "/jmap/eventsource",
        "state": "session-0"
    })
}

impl Engine {
    /// Handle one JMAP `Request` (RFC 8620 §3.3) and produce the `Response`,
    /// resolving `#`-prefixed result references before dispatching each call.
    pub async fn handle_jmap(&self, account_id: &str, request: &Value) -> Value {
        let rt = self.runtime(account_id);
        let empty = Vec::new();
        let calls = request
            .get("methodCalls")
            .and_then(Value::as_array)
            .unwrap_or(&empty);

        let mut responses: Vec<Value> = Vec::new();
        // Tracks Email/set creation-id → stable id so EmailSubmission/set can
        // reference a just-created draft as `#clientId`.
        let mut created_ids: HashMap<String, String> = HashMap::new();

        for call in calls {
            let Some(arr) = call.as_array() else { continue };
            if arr.len() < 3 {
                continue;
            }
            let name = arr[0].as_str().unwrap_or_default();
            let call_id = arr[2].as_str().unwrap_or("c0");
            let mut args = arr[1].clone();
            resolve_references(&mut args, &responses);

            let resp = match &rt {
                Some(rt) => {
                    self.dispatch(account_id, rt, name, &args, &mut created_ids)
                        .await
                }
                None => json!({
                    "type": "accountNotFound",
                    "description": "account is not connected in engine mode"
                }),
            };
            responses.push(json!([name, resp, call_id]));
        }

        json!({ "methodResponses": responses, "sessionState": SESSION_STATE })
    }

    /// Dispatch a single resolved method call to its handler.
    async fn dispatch(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        name: &str,
        args: &Value,
        created_ids: &mut HashMap<String, String>,
    ) -> Value {
        match name {
            "Mailbox/get" => self.mailbox_get(account_id, args).await,
            "Email/query" => self.email_query(account_id, args).await,
            "Email/get" => self.email_get(account_id, args).await,
            "Email/set" => self.email_set(account_id, rt, args, created_ids).await,
            "EmailSubmission/set" => self.submission_set(account_id, rt, args, created_ids).await,
            other => json!({
                "type": "unknownMethod",
                "description": format!("engine does not implement {other}")
            }),
        }
    }

    // ---- Mailbox/get ----------------------------------------------------

    async fn mailbox_get(&self, account_id: &str, args: &Value) -> Value {
        let mailboxes = match self.store().list_mailboxes(account_id).await {
            Ok(m) => m,
            Err(e) => return server_fail(&e),
        };
        // Optional id filter (the web client usually omits it to list all).
        let wanted: Option<Vec<&str>> = args
            .get("ids")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_str).collect());

        let mut list = Vec::new();
        for m in &mailboxes {
            if let Some(ids) = &wanted
                && !ids.contains(&m.id.as_str())
            {
                continue;
            }
            let role = m.role.as_deref();
            list.push(json!({
                "id": m.id,
                "name": display_name(&m.name, role),
                "parentId": m.parent_id,
                "role": role,
                "sortOrder": role_sort_order(role),
                "totalEmails": m.total,
                "unreadEmails": m.unread,
                "totalThreads": m.total,
                "unreadThreads": m.unread,
            }));
        }
        let not_found: Vec<Value> = match &wanted {
            Some(ids) => ids
                .iter()
                .filter(|id| !mailboxes.iter().any(|m| m.id == **id))
                .map(|id| json!(id))
                .collect(),
            None => Vec::new(),
        };
        json!({
            "accountId": account_id,
            "state": SESSION_STATE,
            "list": list,
            "notFound": not_found
        })
    }

    // ---- Email/query ----------------------------------------------------

    async fn email_query(&self, account_id: &str, args: &Value) -> Value {
        let mailbox = args
            .get("filter")
            .and_then(|f| f.get("inMailbox"))
            .and_then(Value::as_str);
        let Some(mailbox) = mailbox else {
            return json!({
                "accountId": account_id, "queryState": SESSION_STATE, "ids": [],
                "total": 0, "position": 0, "canCalculateChanges": false
            });
        };

        let all = match self.store().list_message_ids(mailbox, i64::MAX, 0).await {
            Ok(v) => v,
            Err(e) => return server_fail(&e),
        };
        let total = all.len();
        let position = args.get("position").and_then(Value::as_u64).unwrap_or(0) as usize;
        let limit = args
            .get("limit")
            .and_then(Value::as_u64)
            .map(|n| n as usize);

        let ids: Vec<String> = all
            .into_iter()
            .skip(position)
            .take(limit.unwrap_or(usize::MAX))
            .collect();

        json!({
            "accountId": account_id,
            "queryState": SESSION_STATE,
            "ids": ids,
            "total": total,
            "position": position,
            "canCalculateChanges": false
        })
    }

    // ---- Email/get ------------------------------------------------------

    async fn email_get(&self, account_id: &str, args: &Value) -> Value {
        let empty = Vec::new();
        let ids = args.get("ids").and_then(Value::as_array).unwrap_or(&empty);

        let mut list = Vec::new();
        let mut not_found = Vec::new();
        for id in ids.iter().filter_map(Value::as_str) {
            match self.build_email(id).await {
                Ok(Some(email)) => list.push(email),
                Ok(None) => not_found.push(json!(id)),
                Err(e) => return server_fail(&e),
            }
        }
        json!({
            "accountId": account_id,
            "state": SESSION_STATE,
            "list": list,
            "notFound": not_found
        })
    }

    /// Assemble the `mw_jmap::Email` JSON for one stable id from the sealed
    /// envelope (or a re-parse of the sealed raw body), patched with the
    /// engine-owned id / mailboxIds / keywords / threadId / blobId.
    async fn build_email(&self, stable_id: &str) -> Result<Option<Value>> {
        let msg = match self.store().get_message(stable_id).await {
            Ok(m) => m,
            Err(mw_store::StoreError::NotFound) => return Ok(None),
            Err(e) => return Err(EngineError::Store(e)),
        };

        let mut email: Value = match self.store().get_envelope(stable_id).await? {
            Some(bytes) => serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({})),
            None => match &msg.blob_ref {
                Some(blob) => match self.store().get_body(blob).await? {
                    Some(raw) => mw_mime::parse(&raw)
                        .ok()
                        .and_then(|p| serde_json::to_value(p.email).ok())
                        .unwrap_or_else(|| json!({})),
                    None => json!({}),
                },
                None => json!({}),
            },
        };

        let obj = email.as_object_mut().expect("email is an object");
        obj.insert("id".into(), json!(stable_id));
        obj.insert("blobId".into(), json!(msg.blob_ref));
        obj.insert("threadId".into(), json!(msg.thread_id));
        obj.insert("mailboxIds".into(), json!({ msg.mailbox_id.clone(): true }));
        let keywords = flags_to_keywords(&flags_from_json(&msg.flags_json));
        obj.insert("keywords".into(), json!(keywords));
        Ok(Some(email))
    }

    // ---- Email/set ------------------------------------------------------

    async fn email_set(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        args: &Value,
        created_ids: &mut HashMap<String, String>,
    ) -> Value {
        let mut created = Map::new();
        let mut not_created = Map::new();
        let mut updated = Map::new();
        let mut not_updated = Map::new();

        if let Some(creates) = args.get("create").and_then(Value::as_object) {
            for (client_id, spec) in creates {
                match self.create_draft(account_id, rt, spec).await {
                    Ok((sid, blob)) => {
                        created_ids.insert(client_id.clone(), sid.clone());
                        created.insert(client_id.clone(), json!({ "id": sid, "blobId": blob }));
                    }
                    Err(e) => {
                        not_created.insert(client_id.clone(), set_error(&e));
                    }
                }
            }
        }

        if let Some(updates) = args.get("update").and_then(Value::as_object) {
            for (id, patch) in updates {
                match self.update_email(rt, id, patch).await {
                    Ok(()) => {
                        updated.insert(id.clone(), Value::Null);
                    }
                    Err(e) => {
                        not_updated.insert(id.clone(), set_error(&e));
                    }
                }
            }
        }

        let mut resp = json!({
            "accountId": account_id,
            "oldState": SESSION_STATE,
            "newState": SESSION_STATE,
            "created": created,
            "updated": updated,
            "destroyed": []
        });
        if !not_created.is_empty() {
            resp["notCreated"] = Value::Object(not_created);
        }
        if !not_updated.is_empty() {
            resp["notUpdated"] = Value::Object(not_updated);
        }
        resp
    }

    /// Create a draft: compose MIME, best-effort `APPEND` it to the upstream
    /// Drafts folder, and ingest it locally so it is immediately queryable.
    async fn create_draft(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        spec: &Value,
    ) -> Result<(String, Option<String>)> {
        let message_id = gen_message_id();
        let req = compose_from_spec(spec, &rt.identity, &message_id);
        let raw = mw_mime::build(&req).map_err(|e| EngineError::Protocol(e.to_string()))?;

        let (mailbox_id, imap_name) = self
            .ensure_role_mailbox(account_id, "drafts", "Drafts")
            .await?;
        let mbref = RawMailboxRef {
            name: imap_name,
            uidvalidity: 0,
        };
        // Persist upstream where supported (IMAP); POP3 has no Drafts.
        tolerant(rt.backend.append(&mbref, &raw, &[Flag::Draft]).await)?;

        let sid = self
            .ingest_local(
                account_id,
                &mailbox_id,
                &mbref,
                &message_id,
                raw,
                &[Flag::Draft],
            )
            .await?;
        let blob = self
            .store()
            .get_message(&sid)
            .await
            .ok()
            .and_then(|m| m.blob_ref);
        Ok((sid, blob))
    }

    /// Apply an Email/set update: keyword changes and/or a mailbox move.
    async fn update_email(&self, rt: &AccountRuntime, id: &str, patch: &Value) -> Result<()> {
        let msg = self
            .store()
            .get_message(id)
            .await
            .map_err(EngineError::Store)?;

        if let Some(kw) = patch.get("keywords").and_then(Value::as_object) {
            let kw_map: HashMap<String, bool> = kw
                .iter()
                .filter_map(|(k, v)| v.as_bool().map(|b| (k.clone(), b)))
                .collect();
            let desired = keywords_to_flags(&kw_map);
            let current = flags_from_json(&msg.flags_json);
            let (add, remove) = flag_delta(&current, &desired);

            if let Some(mref) = self.imap_ref_for(id).await? {
                // POP3 keeps flags engine-local, so an Unsupported here is fine.
                tolerant(rt.backend.store_flags(&[mref], &add, &remove).await)?;
            }
            // Preserve IMAP-internal flags the keyword set cannot express.
            let mut stored: Vec<Flag> = current
                .iter()
                .filter(|f| matches!(f, Flag::Deleted | Flag::Recent))
                .cloned()
                .collect();
            stored.extend(desired);
            self.store().set_flags(id, &flags_to_json(&stored)).await?;
        }

        if let Some(mids) = patch.get("mailboxIds").and_then(Value::as_object)
            && let Some(target) = mids
                .iter()
                .find(|(_, v)| v.as_bool() == Some(true))
                .map(|(k, _)| k.clone())
            && target != msg.mailbox_id
        {
            self.move_email(rt, id, &target).await?;
        }
        Ok(())
    }

    /// Move one message to `target_mailbox_id`: `MOVE` it upstream (idempotent by
    /// stable id), then re-file the cached row into the destination mailbox.
    async fn move_email(
        &self,
        rt: &AccountRuntime,
        id: &str,
        target_mailbox_id: &str,
    ) -> Result<()> {
        let msg = self
            .store()
            .get_message(id)
            .await
            .map_err(EngineError::Store)?;
        let dest = self
            .store()
            .get_mailbox(target_mailbox_id)
            .await
            .map_err(EngineError::Store)?;
        let envelope = self.store().get_envelope(id).await?;

        if let Some(mref) = self.imap_ref_for(id).await? {
            let to = RawMailboxRef {
                name: dest.name.clone(),
                uidvalidity: dest.uidvalidity,
            };
            tolerant(rt.backend.move_messages(&[mref], &to).await)?;
        }

        // Re-file locally under the destination mailbox (a fresh stable id — the
        // store's identity match is scoped per-mailbox, so a cross-folder move
        // cannot preserve the id through the public API; V1 poll UI re-queries).
        self.store().delete_message(id).await?;
        let uid = pseudo_uid(msg.message_id.as_deref().unwrap_or(id));
        let owned = MessageUpsertOwned::from_row(
            &msg,
            target_mailbox_id,
            uid,
            dest.uidvalidity,
            envelope.as_deref(),
        );
        self.store().upsert_message(&owned.as_ref()).await?;
        Ok(())
    }

    // ---- EmailSubmission/set -------------------------------------------

    async fn submission_set(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        args: &Value,
        created_ids: &HashMap<String, String>,
    ) -> Value {
        let mut created = Map::new();
        let mut not_created = Map::new();

        if let Some(creates) = args.get("create").and_then(Value::as_object) {
            for (client_id, spec) in creates {
                let email_ref = spec.get("emailId").and_then(Value::as_str).unwrap_or("");
                let real_id = resolve_email_id(email_ref, created_ids);
                match self.submit_email(account_id, rt, &real_id).await {
                    Ok(()) => {
                        created.insert(
                            client_id.clone(),
                            json!({ "id": format!("sub-{}", gen_token()) }),
                        );
                    }
                    Err(e) => {
                        not_created.insert(client_id.clone(), set_error(&e));
                    }
                }
            }
        }

        let mut resp = json!({
            "accountId": account_id,
            "oldState": SESSION_STATE,
            "newState": SESSION_STATE,
            "created": created,
            "updated": {},
            "destroyed": []
        });
        if !not_created.is_empty() {
            resp["notCreated"] = Value::Object(not_created);
        }
        resp
    }

    /// Submit a draft's MIME through the account submitter, then file the sent
    /// copy into `Sent` (both upstream, best-effort, and in the local cache).
    async fn submit_email(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        email_id: &str,
    ) -> Result<()> {
        let msg = self
            .store()
            .get_message(email_id)
            .await
            .map_err(EngineError::Store)?;
        let blob = msg
            .blob_ref
            .as_ref()
            .ok_or_else(|| EngineError::Protocol("draft has no stored body".into()))?;
        let raw = self
            .store()
            .get_body(blob)
            .await?
            .ok_or_else(|| EngineError::Protocol("draft body missing".into()))?;

        // Envelope addresses drive MAIL FROM / RCPT TO.
        let email: mw_jmap::Email = match self.store().get_envelope(email_id).await? {
            Some(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
            None => mw_mime::parse(&raw).map(|p| p.email).unwrap_or_default(),
        };
        let mail_from = email
            .from
            .as_ref()
            .and_then(|f| f.first())
            .map(|a| a.email.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| rt.identity.clone());
        let rcpt_to = recipients(&email);
        if rcpt_to.is_empty() {
            return Err(EngineError::Protocol("no recipients".into()));
        }

        let result = rt
            .submitter
            .submit(mw_smtp::Outgoing {
                mail_from,
                rcpt_to,
                raw: raw.clone(),
            })
            .await?;
        if result.accepted.is_empty() {
            return Err(EngineError::Protocol(format!(
                "all recipients rejected: {:?}",
                result.rejected
            )));
        }

        // File into Sent: upstream APPEND (best-effort) + local re-file.
        let (sent_id, sent_name) = self.ensure_role_mailbox(account_id, "sent", "Sent").await?;
        let sent_ref = RawMailboxRef {
            name: sent_name,
            uidvalidity: 0,
        };
        tolerant(rt.backend.append(&sent_ref, &raw, &[Flag::Seen]).await)?;

        let message_id = msg.message_id.clone().unwrap_or_else(gen_message_id);
        self.ingest_local(
            account_id,
            &sent_id,
            &sent_ref,
            &message_id,
            raw,
            &[Flag::Seen],
        )
        .await?;
        // Remove the original draft now that it has been sent + filed.
        self.store().delete_message(email_id).await?;
        Ok(())
    }

    // ---- shared helpers -------------------------------------------------

    /// Find (or create) the mailbox with a given role, returning `(id, imap_name)`.
    async fn ensure_role_mailbox(
        &self,
        account_id: &str,
        role: &str,
        fallback_name: &str,
    ) -> Result<(String, String)> {
        let mailboxes = self.store().list_mailboxes(account_id).await?;
        if let Some(m) = mailboxes.iter().find(|m| m.role.as_deref() == Some(role)) {
            return Ok((m.id.clone(), m.name.clone()));
        }
        // No special-use folder upstream: create a local one so drafts/sent are
        // still queryable (uidvalidity 0 marks it engine-local).
        let id = self
            .store()
            .upsert_mailbox(&mw_store::MailboxUpsert {
                account_id,
                name: fallback_name,
                role: Some(role),
                uidvalidity: 0,
                uidnext: 0,
                highestmodseq: 0,
                total: 0,
                unread: 0,
                parent_id: None,
            })
            .await?;
        Ok((id, fallback_name.to_string()))
    }

    /// Ingest a locally-composed message (draft / sent copy) into a mailbox,
    /// returning its stable id. Parses the just-built bytes so the stored
    /// envelope matches what `Email/get` will return.
    async fn ingest_local(
        &self,
        account_id: &str,
        mailbox_id: &str,
        mbref: &RawMailboxRef,
        message_id: &str,
        raw: Vec<u8>,
        flags: &[Flag],
    ) -> Result<String> {
        let msg = RawMessage {
            message_ref: MessageRef::Imap {
                mailbox: mbref.clone(),
                uidvalidity: 0,
                uid: pseudo_uid(message_id),
            },
            raw,
            flags: flags.to_vec(),
            internaldate: Some(now_rfc3339()),
        };
        self.ingest(account_id, mailbox_id, &msg).await
    }

    /// Build the IMAP [`MessageRef`] for a stable id from its stored location,
    /// or `None` for a POP3/local message (which the backend cannot address).
    async fn imap_ref_for(&self, stable_id: &str) -> Result<Option<MessageRef>> {
        let Some(loc) = self.store().message_location(stable_id).await? else {
            return Ok(None);
        };
        // uidvalidity 0 marks an engine-local message (draft/sent) with no server
        // coordinates; there is nothing upstream to address.
        if loc.uidvalidity == 0 {
            return Ok(None);
        }
        let mailbox = self
            .store()
            .get_mailbox(&loc.mailbox_id)
            .await
            .map_err(EngineError::Store)?;
        Ok(Some(MessageRef::Imap {
            mailbox: RawMailboxRef {
                name: mailbox.name,
                uidvalidity: loc.uidvalidity,
            },
            uidvalidity: loc.uidvalidity,
            uid: loc.uid,
        }))
    }
}

// ---- free helpers ----------------------------------------------------------

/// Resolve JMAP result references (RFC 8620 §3.7) in a method's arguments, in
/// place. A `"#ids"` key whose value is `{resultOf, name, path}` is replaced by
/// the value at `path` inside the referenced prior response, stored under the
/// de-`#`'d key. Ported from the proven `mw-mock-jmap` logic so the byte shape
/// the web client chains against is identical.
pub fn resolve_references(args: &mut Value, responses: &[Value]) {
    let Some(obj) = args.as_object() else {
        return;
    };
    let ref_keys: Vec<String> = obj.keys().filter(|k| k.starts_with('#')).cloned().collect();
    for key in ref_keys {
        let spec = args[key.as_str()].clone();
        if let (Some(result_of), Some(path)) = (
            spec.get("resultOf").and_then(Value::as_str),
            spec.get("path").and_then(Value::as_str),
        ) {
            let resolved = responses
                .iter()
                .find(|r| r.get(2).and_then(Value::as_str) == Some(result_of))
                .and_then(|r| r.get(1))
                .and_then(|a| a.pointer(path))
                .cloned();
            if let Some(value) = resolved {
                let target = key.trim_start_matches('#');
                args[target] = value;
            }
        }
        if let Some(map) = args.as_object_mut() {
            map.remove(&key);
        }
    }
}

/// Resolve an `emailId` that may be a `#creationId` reference to a stable id.
fn resolve_email_id(email_ref: &str, created_ids: &HashMap<String, String>) -> String {
    if let Some(client_id) = email_ref.strip_prefix('#') {
        created_ids
            .get(client_id)
            .cloned()
            .unwrap_or_else(|| email_ref.to_string())
    } else {
        email_ref.to_string()
    }
}

/// Build a [`ComposeRequest`] from an `Email/set` create spec.
fn compose_from_spec(spec: &Value, identity: &str, message_id: &str) -> ComposeRequest {
    let from = parse_addrs(spec.get("from"))
        .into_iter()
        .next()
        .or_else(|| {
            (!identity.is_empty()).then(|| EmailAddress {
                name: None,
                email: identity.to_string(),
            })
        });
    let (text_body, html_body) = extract_bodies(spec);
    ComposeRequest {
        from,
        to: parse_addrs(spec.get("to")),
        cc: parse_addrs(spec.get("cc")),
        bcc: parse_addrs(spec.get("bcc")),
        reply_to: parse_addrs(spec.get("replyTo")),
        subject: spec
            .get("subject")
            .and_then(Value::as_str)
            .map(String::from),
        text_body,
        html_body,
        message_id: Some(message_id.to_string()),
        in_reply_to: spec
            .get("inReplyTo")
            .and_then(Value::as_str)
            .map(String::from),
        references: spec
            .get("references")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default(),
        headers: Vec::new(),
    }
}

/// Parse a JMAP address list (`[{name?, email}]`) into [`EmailAddress`]es.
fn parse_addrs(v: Option<&Value>) -> Vec<EmailAddress> {
    v.and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|x| {
                    let email = x.get("email").and_then(Value::as_str)?;
                    Some(EmailAddress {
                        name: x.get("name").and_then(Value::as_str).map(String::from),
                        email: email.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Pull the text/html body strings out of a create spec's `bodyValues` +
/// `textBody`/`htmlBody` part lists (falling back to bodyValues["1"]).
fn extract_bodies(spec: &Value) -> (Option<String>, Option<String>) {
    let body_values = spec.get("bodyValues").and_then(Value::as_object);
    let body_for = |parts_key: &str| -> Option<String> {
        let part_id = spec
            .get(parts_key)
            .and_then(Value::as_array)
            .and_then(|a| a.first())
            .and_then(|p| p.get("partId"))
            .and_then(Value::as_str)?;
        body_values?
            .get(part_id)
            .and_then(|v| v.get("value"))
            .and_then(Value::as_str)
            .map(String::from)
    };
    let mut text = body_for("textBody");
    let html = body_for("htmlBody");
    if text.is_none()
        && html.is_none()
        && let Some(v) = body_values
            .and_then(|bv| bv.get("1"))
            .and_then(|v| v.get("value"))
            .and_then(Value::as_str)
    {
        // Ambiguous single body: treat as text (safe default for a draft).
        text = Some(v.to_string());
    }
    (text, html)
}

/// Envelope recipients: `to` + `cc` + `bcc`.
fn recipients(email: &mw_jmap::Email) -> Vec<String> {
    let mut out = Vec::new();
    for addrs in [&email.to, &email.cc, &email.bcc].into_iter().flatten() {
        out.extend(addrs.iter().map(|a| a.email.clone()));
    }
    out.retain(|e| !e.is_empty());
    out
}

/// A JMAP method-level `SetError` object for a failed create/update.
fn set_error(e: &EngineError) -> Value {
    json!({ "type": "serverFail", "description": e.to_string() })
}

/// A whole-method server failure result.
fn server_fail(e: &dyn std::error::Error) -> Value {
    json!({ "type": "serverFail", "description": e.to_string() })
}

/// Swallow an `Unsupported` backend result (a POP3/local no-op) while
/// propagating real failures.
fn tolerant<T>(res: Result<T>) -> Result<Option<T>> {
    match res {
        Ok(v) => Ok(Some(v)),
        Err(EngineError::Unsupported(_)) => Ok(None),
        Err(e) => Err(e),
    }
}

/// A deterministic non-zero pseudo-UID from an id string (local messages).
fn pseudo_uid(seed: &str) -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    seed.hash(&mut h);
    (h.finish() as u32) | 1
}

/// Monotonic-ish unique token source for generated ids.
static COUNTER: AtomicU64 = AtomicU64::new(1);

fn gen_token() -> String {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{t:x}{n:x}")
}

fn gen_message_id() -> String {
    format!("<{}@mailwoman.local>", gen_token())
}

fn now_rfc3339() -> String {
    // A real RFC3339 stamp so locally-composed messages sort correctly among
    // upstream messages (the store's `list_message_ids` orders by internaldate).
    chrono::Utc::now().to_rfc3339()
}

use mw_store::MessageUpsert;

/// Owned mirror of [`MessageUpsert`] so `move_email` can borrow from a `Message`
/// row without lifetime gymnastics.
struct MessageUpsertOwned {
    account_id: String,
    mailbox_id: String,
    uid: u32,
    uidvalidity: u32,
    message_id: Option<String>,
    thread_id: Option<String>,
    internaldate: Option<String>,
    size: u64,
    flags_json: String,
    envelope: Option<Vec<u8>>,
    blob_ref: Option<String>,
}

impl MessageUpsertOwned {
    fn from_row(
        row: &mw_store::Message,
        mailbox_id: &str,
        uid: u32,
        uidvalidity: u32,
        envelope: Option<&[u8]>,
    ) -> Self {
        Self {
            account_id: row.account_id.clone(),
            mailbox_id: mailbox_id.to_string(),
            uid,
            uidvalidity,
            message_id: row.message_id.clone(),
            thread_id: row.thread_id.clone(),
            internaldate: row.internaldate.clone(),
            size: row.size,
            flags_json: row.flags_json.clone(),
            envelope: envelope.map(<[u8]>::to_vec),
            blob_ref: row.blob_ref.clone(),
        }
    }

    fn as_ref(&self) -> MessageUpsert<'_> {
        MessageUpsert {
            account_id: &self.account_id,
            mailbox_id: &self.mailbox_id,
            uid: self.uid,
            uidvalidity: self.uidvalidity,
            message_id: self.message_id.as_deref(),
            thread_id: self.thread_id.as_deref(),
            internaldate: self.internaldate.as_deref(),
            size: self.size,
            flags_json: &self.flags_json,
            envelope: self.envelope.as_deref(),
            blob_ref: self.blob_ref.as_deref(),
        }
    }
}
