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

use mw_mime::{Attachment, ComposeRequest, EmailAddress};
use mw_store::{IdentityRow, StoredMeta, SubmissionRow};
use serde_json::{Map, Value, json};

use crate::account::AccountRuntime;
use crate::backend::{EngineError, Flag, MessageRef, RawMailboxRef, RawMessage, Result};
use crate::change::{ChangeOp, ChangeType};
use crate::engine::Engine;
use crate::mapping::{
    display_name, flag_delta, flags_from_json, flags_to_json, flags_to_keywords, keywords_to_flags,
    role_sort_order,
};
use crate::query::{Comparator, EmailFilter};
use crate::search_index;

// The mail-family completeness surface (t16 J1–J5): `Thread/*`, `SearchSnippet/get`,
// `VacationResponse/get|set`, `Quota/get`, and `Email/copy|import|parse`. Declared
// here (not in `lib.rs`) so the dispatch owner also owns the mod line; the files
// live under `src/mail_ext/` (mirrors `pim/`). Reached from `dispatch` below via
// the `is_mail_ext_method` guard.
#[path = "mail_ext/mod.rs"]
pub(crate) mod mail_ext;

/// Build the JMAP [`Session`](mw_jmap::Session) resource for a connected account,
/// advertising core + mail + submission and pointing every URL back at us.
pub fn session_json(account_id: &str, username: &str) -> Value {
    json!({
        "capabilities": {
            "urn:ietf:params:jmap:core": { "maxSizeUpload": 50_000_000, "maxConcurrentRequests": 4 },
            // The core mail capability covers `Thread/*`, `SearchSnippet/get`, and
            // `Email/copy|import|parse` (RFC 8621) — all answered by the `mail_ext`
            // dispatch alongside the `Email/*`/`Mailbox/*` handlers.
            "urn:ietf:params:jmap:mail": {},
            "urn:ietf:params:jmap:submission": {},
            // t16 J4/J3: VacationResponse (RFC 8621 §8) + Quota (RFC 9425).
            "urn:ietf:params:jmap:vacationresponse": {},
            "urn:ietf:params:jmap:quota": {},
            // Mailwoman-native PIM capabilities (frozen §1.1/§2.2). The web
            // transport/offline/push layers reuse the mail machinery verbatim;
            // these URNs advertise the PIM method families under our own types.
            "urn:mailwoman:calendars": {},
            "urn:mailwoman:tasks": {},
            "urn:mailwoman:notes": {},
            "urn:mailwoman:contacts": {},
            // V4 crypto/security capabilities (frozen §1.4/§2.2). The keyring +
            // verdict + DLP + sender-control + mail-rule families ride the same
            // envelope; private-key ops run client-side (WASM), never here.
            "urn:mailwoman:crypto": {},
            "urn:mailwoman:security": {}
        },
        "accounts": {
            account_id: { "name": username, "isPersonal": true, "isReadOnly": false, "accountCapabilities": {
                "urn:ietf:params:jmap:vacationresponse": {},
                "urn:ietf:params:jmap:quota": {},
                "urn:mailwoman:calendars": {},
                "urn:mailwoman:tasks": {},
                "urn:mailwoman:notes": {},
                "urn:mailwoman:contacts": {},
                "urn:mailwoman:crypto": {},
                "urn:mailwoman:security": {}
            } }
        },
        "primaryAccounts": {
            "urn:ietf:params:jmap:mail": account_id,
            "urn:ietf:params:jmap:submission": account_id,
            "urn:ietf:params:jmap:vacationresponse": account_id,
            "urn:ietf:params:jmap:quota": account_id,
            "urn:mailwoman:calendars": account_id,
            "urn:mailwoman:tasks": account_id,
            "urn:mailwoman:notes": account_id,
            "urn:mailwoman:contacts": account_id,
            "urn:mailwoman:crypto": account_id,
            "urn:mailwoman:security": account_id
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

        json!({
            "methodResponses": responses,
            "sessionState": self.session_state(account_id).await
        })
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
            "Mailbox/changes" => {
                self.type_changes(account_id, ChangeType::Mailbox, args)
                    .await
            }
            "Email/query" => self.email_query(account_id, args).await,
            "Email/queryChanges" => self.email_query_changes(account_id, args).await,
            "Email/get" => self.email_get(account_id, args).await,
            "Email/changes" => self.type_changes(account_id, ChangeType::Email, args).await,
            "Email/set" => self.email_set(account_id, rt, args, created_ids).await,
            "EmailSubmission/set" => self.submission_set(account_id, rt, args, created_ids).await,
            "EmailSubmission/get" => self.submission_get(account_id, args).await,
            "EmailSubmission/query" => self.submission_query(account_id, args).await,
            "EmailSubmission/changes" => {
                self.type_changes(account_id, ChangeType::EmailSubmission, args)
                    .await
            }
            "Identity/get" => self.identity_get(account_id, rt, args).await,
            "Identity/query" => self.identity_query(account_id, rt).await,
            // ACL (RFC 4314) + METADATA (RFC 5464) read-through seam (t13 §6 E7):
            // ride the same envelope; the backend handle is the account's
            // `AccountBackend`, delegating to the upstream server. Handlers live
            // in `acl.rs`.
            "MailboxRights/get" => self.mailbox_rights_get(account_id, rt, args).await,
            "MailboxRights/set" => self.mailbox_rights_set(account_id, rt, args).await,
            "ServerMetadata/get" => self.server_metadata_get(account_id, rt, args).await,
            "ServerMetadata/set" => self.server_metadata_set(account_id, rt, args).await,
            // Mail-family completeness (t16 J1–J5): `Thread/*`, `SearchSnippet/get`,
            // `VacationResponse/get|set`, `Quota/get`, `Email/copy|import|parse` ride
            // the same envelope behind `dispatch_mail_ext`. Ordered after the explicit
            // `Email/*` arms above so `Email/get`/`set`/`query` still win.
            other if mail_ext::dispatch::is_mail_ext_method(other) => {
                self.dispatch_mail_ext(account_id, other, args).await
            }
            // Mailwoman-native PIM families (§2.2) ride the same envelope; e8
            // fills the handlers behind `dispatch_pim`.
            other if crate::pim::dispatch::is_pim_method(other) => {
                self.dispatch_pim(account_id, rt, other, args).await
            }
            // Mailwoman-native crypto/security families (§2.2) ride the same
            // envelope; e6 fills the handlers behind `dispatch_security`.
            other if crate::security::dispatch::is_security_method(other) => {
                self.dispatch_security(account_id, rt, other, args).await
            }
            other => json!({
                "type": "unknownMethod",
                "description": format!("engine does not implement {other}")
            }),
        }
    }

    /// The generic `*/changes` handler (frozen §2.1): `{oldState,newState,
    /// created,updated,destroyed,hasMoreChanges}` for a datatype since a state.
    async fn type_changes(&self, account_id: &str, kind: ChangeType, args: &Value) -> Value {
        let since = args
            .get("sinceState")
            .and_then(Value::as_str)
            .unwrap_or("0");
        match self.build_changes(account_id, kind, since).await {
            Ok(changes) => {
                let mut v = serde_json::to_value(&changes).unwrap_or_else(|_| json!({}));
                if let Some(obj) = v.as_object_mut() {
                    obj.insert("accountId".into(), json!(account_id));
                }
                v
            }
            Err(e) => server_fail(&e),
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
        // Saved searches surface as virtual folders (role:null +
        // mailwomanSearchQuery). Querying one runs its stored filter (§2.1).
        let saved = self
            .store()
            .list_saved_searches(account_id)
            .await
            .unwrap_or_default();
        for s in &saved {
            if !s.as_folder {
                continue;
            }
            if let Some(ids) = &wanted
                && !ids.contains(&s.id.as_str())
            {
                continue;
            }
            list.push(json!({
                "id": s.id,
                "name": s.name,
                "parentId": Value::Null,
                "role": Value::Null,
                "sortOrder": 20,
                "totalEmails": 0,
                "unreadEmails": 0,
                "totalThreads": 0,
                "unreadThreads": 0,
                "mailwomanSearchQuery": s.query_json,
            }));
        }

        let not_found: Vec<Value> = match &wanted {
            Some(ids) => ids
                .iter()
                .filter(|id| {
                    !mailboxes.iter().any(|m| m.id == **id) && !saved.iter().any(|s| s.id == **id)
                })
                .map(|id| json!(id))
                .collect(),
            None => Vec::new(),
        };
        json!({
            "accountId": account_id,
            "state": self.type_state(account_id, ChangeType::Mailbox).await.unwrap_or_default(),
            "list": list,
            "notFound": not_found
        })
    }

    // ---- Email/query ----------------------------------------------------

    async fn email_query(&self, account_id: &str, args: &Value) -> Value {
        let all = match self.query_ids(account_id, args).await {
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
            "queryState": self.type_state(account_id, ChangeType::Email).await.unwrap_or_default(),
            "ids": ids,
            "total": total,
            "position": position,
            "canCalculateChanges": true
        })
    }

    /// Resolve an `Email/query` to the full ordered id list (before paging),
    /// routing to `mw-search` for any full-text/attachment/custom-sort condition
    /// and to the SQL fast path for a pure `inMailbox` newest-first listing
    /// (frozen routing rule §2.1). Saved-search folders run their stored filter.
    async fn query_ids(&self, account_id: &str, args: &Value) -> Result<Vec<String>> {
        let raw_filter = args.get("filter").cloned().unwrap_or(Value::Null);
        let mut filter: EmailFilter = serde_json::from_value(raw_filter).unwrap_or_default();
        let comparator = first_comparator(args);
        let sort = search_index::sort_from_comparator(comparator.as_ref());
        let custom_sort = sort != mw_search::Sort::received_desc();

        // A saved-search folder id in `inMailbox` expands to its stored filter.
        let mut saved_folder = false;
        if let Some(mb) = filter.in_mailbox.clone()
            && let Some(ss) = self.store().get_saved_search(&mb).await?
        {
            saved_folder = true;
            filter = serde_json::from_str(&ss.query_json).unwrap_or_default();
        }

        let use_search = saved_folder || custom_sort || filter.needs_search();
        if !use_search {
            // SQL fast path: pure `inMailbox`, newest-first.
            let Some(mb) = filter.in_mailbox.as_deref() else {
                return Ok(Vec::new());
            };
            return Ok(self.store().list_message_ids(mb, i64::MAX, 0).await?);
        }

        let mailbox_ids: Vec<String> = self
            .store()
            .list_mailboxes(account_id)
            .await?
            .into_iter()
            .map(|m| m.id)
            .collect();
        // Scope to a single mailbox only when the filter pins a real one.
        let scope = filter
            .in_mailbox
            .as_deref()
            .filter(|mb| mailbox_ids.iter().any(|m| m == mb));
        let sq = search_index::build_search_query(&filter, sort, &mailbox_ids, scope);
        self.search()
            .search(&sq, 0)
            .map_err(|e| EngineError::Protocol(format!("search: {e}")))
    }

    /// `Email/queryChanges` (frozen §2.1): a best-effort delta. Recomputes the
    /// current query and diffs it against the caller's `sinceQueryState` using
    /// the change log so `added`/`removed` are cheap for the client to apply.
    async fn email_query_changes(&self, account_id: &str, args: &Value) -> Value {
        let since = args
            .get("sinceQueryState")
            .and_then(Value::as_str)
            .unwrap_or("0");
        let new_state = self
            .type_state(account_id, ChangeType::Email)
            .await
            .unwrap_or_default();
        let ids = match self.query_ids(account_id, args).await {
            Ok(v) => v,
            Err(e) => return server_fail(&e),
        };
        // Destroyed ids since `since` that the client should drop.
        let removed: Vec<String> = match self
            .build_changes(account_id, ChangeType::Email, since)
            .await
        {
            Ok(c) => c.destroyed,
            Err(_) => Vec::new(),
        };
        let added: Vec<Value> = ids
            .iter()
            .enumerate()
            .map(|(i, id)| json!({ "id": id, "index": i }))
            .collect();
        json!({
            "accountId": account_id,
            "oldQueryState": since,
            "newQueryState": new_state,
            "total": ids.len(),
            "removed": removed,
            "added": added
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
            "state": self.type_state(account_id, ChangeType::Email).await.unwrap_or_default(),
            "list": list,
            "notFound": not_found
        })
    }

    /// Assemble the `mw_jmap::Email` JSON for one stable id from the sealed
    /// envelope (or a re-parse of the sealed raw body), patched with the
    /// engine-owned id / mailboxIds / keywords / threadId / blobId and the
    /// engine-local `pinned`/`snoozedUntil`/`followUpAt` extras (§2.1).
    async fn build_email(&self, stable_id: &str) -> Result<Option<Value>> {
        let msg = match self.store().get_message(stable_id).await {
            Ok(m) => m,
            Err(mw_store::StoreError::NotFound) => return Ok(None),
            Err(e) => return Err(EngineError::Store(e)),
        };

        // Cache-aside on the header-window (envelope) + message-body read paths
        // (plan §3 e10). Inert without an attached cache; zero-access accounts
        // bypass every shared tier via `get_derived` (the store already holds
        // ciphertext the engine treats as opaque).
        let mut email: Value = match self.cached_envelope(&msg.account_id, stable_id).await? {
            Some(bytes) => serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({})),
            None => match &msg.blob_ref {
                Some(blob) => match self.cached_body(&msg.account_id, stable_id, blob).await? {
                    Some(raw) => mw_mime::parse(&raw)
                        .ok()
                        .and_then(|p| serde_json::to_value(p.email).ok())
                        .unwrap_or_else(|| json!({})),
                    None => json!({}),
                },
                None => json!({}),
            },
        };

        let meta = self
            .store()
            .get_message_meta(stable_id)
            .await?
            .unwrap_or_default();
        let obj = email.as_object_mut().expect("email is an object");
        obj.insert("id".into(), json!(stable_id));
        // blobId scheme (e14): the whole message is `<stableId>`, each attachment
        // part is `<stableId>.<partId>` — both resolved by `Engine::fetch_blob`
        // behind `/jmap/download`. Patch the message blobId + every attachment's.
        obj.insert("blobId".into(), json!(stable_id));
        if let Some(atts) = obj.get_mut("attachments").and_then(Value::as_array_mut) {
            for att in atts {
                if let Some(pid) = att.get("partId").and_then(Value::as_str) {
                    att["blobId"] = json!(format!("{stable_id}.{pid}"));
                }
            }
        }
        obj.insert("threadId".into(), json!(msg.thread_id));
        obj.insert("mailboxIds".into(), json!({ msg.mailbox_id.clone(): true }));
        let keywords = flags_to_keywords(&flags_from_json(&msg.flags_json));
        obj.insert("keywords".into(), json!(keywords));
        obj.insert("pinned".into(), json!(meta.pinned));
        obj.insert("snoozedUntil".into(), json!(meta.snoozed_until));
        obj.insert("followUpAt".into(), json!(meta.follow_up_at));
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
        let old_state = self
            .type_state(account_id, ChangeType::Email)
            .await
            .unwrap_or_default();
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
                match self.update_email(account_id, rt, id, patch).await {
                    Ok(()) => {
                        updated.insert(id.clone(), Value::Null);
                    }
                    Err(e) => {
                        not_updated.insert(id.clone(), set_error(&e));
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
        let req = self
            .compose_from_spec(account_id, spec, &rt.identity, &message_id)
            .await?;
        let raw = mw_mime::build(&req).map_err(|e| EngineError::Protocol(e.to_string()))?;

        let (mailbox_id, imap_name) = self
            .ensure_role_mailbox(account_id, "drafts", "Drafts")
            .await?;
        let mbref = RawMailboxRef {
            name: imap_name,
            uidvalidity: 0,
        };
        // Persist upstream where supported (IMAP); POP3 has no Drafts. A
        // plugin/bridge backend has NO append-to-folder semantics — its frozen
        // account-backend `submit` export *transmits* (Graph `sendMail` / Gmail
        // `messages/send` / EWS `SendItem`), so appending a locally-composed draft
        // there would SEND it. For a plugin-backed account the draft therefore lives
        // only in the local cache (still immediately queryable via `ingest_local`
        // below); outbound send happens exclusively through the submitter at
        // `EmailSubmission` time. Standards IMAP keeps the best-effort upstream APPEND
        // (byte-unchanged).
        if !self.is_plugin_backed(account_id) {
            tolerant(rt.backend.append(&mbref, &raw, &[Flag::Draft]).await)?;
        }

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
        // The draft's download blobId is the whole-message form `<stableId>`
        // (see `Engine::fetch_blob`), so a just-composed draft is exportable.
        Ok((sid.clone(), Some(sid)))
    }

    /// Build a [`ComposeRequest`] from an `Email/set` create spec, resolving any
    /// `attachments` whose `blobId` names an existing stored message/part or a
    /// new-file upload (the reserved `U` namespace) via [`Engine::fetch_blob`]
    /// — forward / attach-from-mail as well as attach-uploaded-file. A blobId
    /// that resolves to nothing is a clean error (→ `notCreated`), never a
    /// panic.
    async fn compose_from_spec(
        &self,
        account_id: &str,
        spec: &Value,
        identity: &str,
        message_id: &str,
    ) -> Result<ComposeRequest> {
        let mut req = compose_base_from_spec(spec, identity, message_id);
        if let Some(atts) = spec.get("attachments").and_then(Value::as_array) {
            for att in atts {
                // An attachment entry without a blobId (e.g. an inline body-part
                // reference) is not a stored-blob attachment; skip it.
                let Some(blob_id) = att.get("blobId").and_then(Value::as_str) else {
                    continue;
                };
                let blob = self.fetch_blob(account_id, blob_id).await?.ok_or_else(|| {
                    EngineError::Protocol(format!(
                        "attachment blobId {blob_id:?} does not resolve to a stored message part or uploaded blob"
                    ))
                })?;
                // Prefer the client-declared type/name; fall back to what the
                // stored part reports (fetch_blob derives both from the MIME part).
                let content_type = att
                    .get("type")
                    .and_then(Value::as_str)
                    .map(String::from)
                    .unwrap_or(blob.content_type);
                let filename = att
                    .get("name")
                    .and_then(Value::as_str)
                    .map(String::from)
                    .unwrap_or(blob.filename);
                req.attachments.push(Attachment {
                    filename,
                    content_type,
                    bytes: blob.bytes,
                });
            }
        }
        Ok(req)
    }

    /// Apply an Email/set update: keyword changes, engine-local meta
    /// (pin/snooze/follow-up), and/or a mailbox move. Records the `Email` change
    /// and re-indexes so state + search stay consistent (plan §1.2, §1.5).
    async fn update_email(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        id: &str,
        patch: &Value,
    ) -> Result<()> {
        let msg = self
            .store()
            .get_message(id)
            .await
            .map_err(EngineError::Store)?;
        let mut touched = false;

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
            touched = true;
        }

        // Engine-local metadata (§2.1): pinned / snoozedUntil / followUpAt. A
        // present key sets it; JSON null clears it.
        if patch.get("pinned").is_some()
            || patch.get("snoozedUntil").is_some()
            || patch.get("followUpAt").is_some()
        {
            let mut meta = self.store().get_message_meta(id).await?.unwrap_or_default();
            if let Some(v) = patch.get("pinned") {
                meta.pinned = v.as_bool().unwrap_or(meta.pinned);
            }
            if let Some(v) = patch.get("snoozedUntil") {
                meta.snoozed_until = v.as_str().map(str::to_string);
            }
            if let Some(v) = patch.get("followUpAt") {
                meta.follow_up_at = v.as_str().map(str::to_string);
            }
            self.store()
                .upsert_message_meta(
                    id,
                    &StoredMeta {
                        pinned: meta.pinned,
                        snoozed_until: meta.snoozed_until,
                        follow_up_at: meta.follow_up_at,
                    },
                )
                .await?;
            touched = true;
        }

        let mut moved = false;
        if let Some(mids) = patch.get("mailboxIds").and_then(Value::as_object)
            && let Some(target) = mids
                .iter()
                .find(|(_, v)| v.as_bool() == Some(true))
                .map(|(k, _)| k.clone())
            && target != msg.mailbox_id
        {
            self.move_email(rt, id, &target).await?;
            moved = true;
        }

        // `move_email` already records its own change + re-index; only record a
        // plain update when a non-move field changed.
        if touched && !moved {
            self.reindex_message(id).await;
            self.record_change(account_id, ChangeType::Email, id, ChangeOp::Updated)
                .await?;
        }
        Ok(())
    }

    /// Move one message to `target_mailbox_id`: `MOVE` it upstream (idempotent by
    /// stable id), then relocate the cached row **in place preserving its
    /// `stable_id`** (plan §1.4). Tags, `message_meta`, and the search index all
    /// key on that id, so they follow the move without re-keying; only the
    /// index's stored `mailboxId` is refreshed. This is the single move path in
    /// V2 — it pays the tracked V1 debt where a move minted a new id.
    pub(crate) async fn move_email(
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
        let source_mailbox_id = msg.mailbox_id.clone();
        let dest = self
            .store()
            .get_mailbox(target_mailbox_id)
            .await
            .map_err(EngineError::Store)?;

        // Upstream move; UIDPLUS gives the destination coordinates, else derive a
        // deterministic pseudo-UID (engine-local / non-UIDPLUS servers).
        let mut new_uid = pseudo_uid(msg.message_id.as_deref().unwrap_or(id));
        let mut new_uidvalidity = dest.uidvalidity;
        if let Some(mref) = self.imap_ref_for(id).await? {
            let to = RawMailboxRef {
                name: dest.name.clone(),
                uidvalidity: dest.uidvalidity,
            };
            if let Some(crate::backend::MoveOutcome::Uidplus { uidvalidity, uids }) =
                tolerant(rt.backend.move_messages(&[mref], &to).await)?
                && let Some(u) = uids.first()
            {
                new_uid = *u;
                new_uidvalidity = uidvalidity;
            }
        }

        self.store()
            .relocate_message(id, target_mailbox_id, new_uid, new_uidvalidity)
            .await?;
        // Re-key the search index onto the destination mailbox (preserves every
        // other indexed field via the stored doc).
        let _ = self.search().relocate(id, target_mailbox_id);

        self.record_change(&msg.account_id, ChangeType::Email, id, ChangeOp::Updated)
            .await?;
        self.record_change(
            &msg.account_id,
            ChangeType::Mailbox,
            &source_mailbox_id,
            ChangeOp::Updated,
        )
        .await?;
        self.record_change(
            &msg.account_id,
            ChangeType::Mailbox,
            target_mailbox_id,
            ChangeOp::Updated,
        )
        .await?;
        Ok(())
    }

    // ---- EmailSubmission (queue: undo-send / send-later / Outbox) -------

    /// `EmailSubmission/set` (plan §1.3): create **enqueues** a submission
    /// (`undoStatus:pending`) with an optional hold window / `sendAt`; a
    /// submission with no hold and no future `sendAt` fires inline (the V1
    /// synchronous send shape). update `{undoStatus:"canceled"}` cancels a
    /// still-pending submission before its window elapses.
    async fn submission_set(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        args: &Value,
        created_ids: &HashMap<String, String>,
    ) -> Value {
        let old_state = self
            .type_state(account_id, ChangeType::EmailSubmission)
            .await
            .unwrap_or_default();
        let mut created = Map::new();
        let mut not_created = Map::new();
        let mut updated = Map::new();
        let mut not_updated = Map::new();

        if let Some(creates) = args.get("create").and_then(Value::as_object) {
            for (client_id, spec) in creates {
                let email_ref = spec.get("emailId").and_then(Value::as_str).unwrap_or("");
                let real_id = resolve_email_id(email_ref, created_ids);
                let identity_id = spec
                    .get("identityId")
                    .and_then(Value::as_str)
                    .map(String::from);
                let send_at = spec.get("sendAt").and_then(Value::as_str).map(String::from);
                let hold = spec
                    .get("mailwomanHoldSeconds")
                    .and_then(Value::as_u64)
                    .unwrap_or(0) as u32;
                // V4 DLP gate (plan §1.8): evaluate outbound rules at create time
                // (covers both the inline and the deferred send paths) BEFORE the
                // submission is enqueued. A `block` verdict fails this create with
                // a structured `dlpBlocked` error and the message is never queued;
                // the redacted audit row is written by `evaluate`.
                let dlp_verdicts = crate::security::dlp::evaluate(self, account_id, &real_id).await;
                if let Some(err) = dlp_block_error(&dlp_verdicts) {
                    not_created.insert(client_id.clone(), err);
                    continue;
                }
                match self
                    .enqueue_submission(account_id, rt, &real_id, identity_id, send_at, hold)
                    .await
                {
                    Ok((sub_id, undo_status)) => {
                        created.insert(
                            client_id.clone(),
                            json!({ "id": sub_id, "undoStatus": undo_status }),
                        );
                    }
                    Err(e) => {
                        not_created.insert(client_id.clone(), set_error(&e));
                    }
                }
            }
        }

        if let Some(updates) = args.get("update").and_then(Value::as_object) {
            for (id, patch) in updates {
                let cancel = patch.get("undoStatus").and_then(Value::as_str) == Some("canceled");
                match self.cancel_submission(account_id, id, cancel).await {
                    Ok(()) => {
                        updated.insert(id.clone(), Value::Null);
                    }
                    Err(e) => {
                        not_updated.insert(id.clone(), set_error(&e));
                    }
                }
            }
        }

        let new_state = self
            .type_state(account_id, ChangeType::EmailSubmission)
            .await
            .unwrap_or_default();
        let mut resp = json!({
            "accountId": account_id,
            "oldState": old_state,
            "newState": new_state,
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

    /// Persist a submission row, then fire it inline when it is due immediately
    /// (no hold, no future `sendAt`); otherwise leave it for the dispatcher.
    /// Returns `(submissionId, undoStatus)`.
    async fn enqueue_submission(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        email_id: &str,
        identity_id: Option<String>,
        send_at: Option<String>,
        hold_seconds: u32,
    ) -> Result<(String, &'static str)> {
        let sub_id = format!("sub-{}", gen_token());
        let created_at = now_rfc3339();
        let row = SubmissionRow {
            id: sub_id.clone(),
            account_id: account_id.to_string(),
            email_id: email_id.to_string(),
            identity_id,
            send_at: send_at.clone(),
            undo_status: "pending".to_string(),
            hold_seconds,
            created_at,
        };
        self.store().insert_submission(&row).await?;
        self.record_change(
            account_id,
            ChangeType::EmailSubmission,
            &sub_id,
            ChangeOp::Created,
        )
        .await?;

        let now = chrono::Utc::now();
        let future_send = send_at
            .as_deref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .is_some_and(|dt| dt.with_timezone(&chrono::Utc) > now);
        if hold_seconds == 0 && !future_send {
            // Fire now (preserves the V1 synchronous send shape).
            match self.submit_email(account_id, rt, email_id).await {
                Ok(()) => {
                    self.store().set_submission_status(&sub_id, "final").await?;
                    self.record_change(
                        account_id,
                        ChangeType::EmailSubmission,
                        &sub_id,
                        ChangeOp::Updated,
                    )
                    .await?;
                    Ok((sub_id, "final"))
                }
                Err(e) => {
                    self.store()
                        .set_submission_status(&sub_id, "canceled")
                        .await?;
                    Err(e)
                }
            }
        } else {
            // Deferred: the dispatcher fires it when the window elapses.
            Ok((sub_id, "pending"))
        }
    }

    /// Cancel a still-pending submission (the undo-send action). Errors if the
    /// submission is unknown or already `final`/`canceled`.
    async fn cancel_submission(&self, account_id: &str, id: &str, cancel: bool) -> Result<()> {
        if !cancel {
            return Ok(()); // update touched nothing we act on
        }
        let row = self
            .store()
            .get_submission(id)
            .await?
            .ok_or_else(|| EngineError::Protocol(format!("unknown submission {id}")))?;
        if row.undo_status != "pending" {
            return Err(EngineError::Protocol(format!(
                "submission {id} is {} and cannot be canceled",
                row.undo_status
            )));
        }
        self.store().set_submission_status(id, "canceled").await?;
        self.record_change(
            account_id,
            ChangeType::EmailSubmission,
            id,
            ChangeOp::Updated,
        )
        .await?;
        // Audit/webhook feed off the recall (plan §3 e10). Metadata only.
        self.emit_audit(crate::v6::AuditEvent {
            account_id: account_id.to_string(),
            action: "submission.recalled".into(),
            target: Some(id.to_string()),
            detail: serde_json::json!({ "emailId": row.email_id }),
        });
        Ok(())
    }

    /// `EmailSubmission/get` — fetch submissions by id (Outbox item detail).
    async fn submission_get(&self, account_id: &str, args: &Value) -> Value {
        let wanted: Option<Vec<String>> = args.get("ids").and_then(Value::as_array).map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(String::from)
                .collect()
        });
        let rows = match self.store().list_submissions(account_id).await {
            Ok(v) => v,
            Err(e) => return server_fail(&e),
        };
        let mut list = Vec::new();
        let mut found = Vec::new();
        for row in &rows {
            if let Some(ids) = &wanted
                && !ids.contains(&row.id)
            {
                continue;
            }
            found.push(row.id.clone());
            list.push(submission_json(row));
        }
        let not_found: Vec<Value> = match &wanted {
            Some(ids) => ids
                .iter()
                .filter(|id| !found.contains(id))
                .map(|id| json!(id))
                .collect(),
            None => Vec::new(),
        };
        json!({
            "accountId": account_id,
            "state": self.type_state(account_id, ChangeType::EmailSubmission).await.unwrap_or_default(),
            "list": list,
            "notFound": not_found
        })
    }

    /// `EmailSubmission/query` — **the Outbox** (all submissions, newest-first).
    async fn submission_query(&self, account_id: &str, args: &Value) -> Value {
        let rows = match self.store().list_submissions(account_id).await {
            Ok(v) => v,
            Err(e) => return server_fail(&e),
        };
        // Optional undoStatus filter (e.g. only pending = the live Outbox).
        let want_status = args
            .get("filter")
            .and_then(|f| f.get("undoStatus"))
            .and_then(Value::as_str);
        let ids: Vec<String> = rows
            .iter()
            .filter(|r| want_status.is_none_or(|s| r.undo_status == s))
            .map(|r| r.id.clone())
            .collect();
        json!({
            "accountId": account_id,
            "queryState": self.type_state(account_id, ChangeType::EmailSubmission).await.unwrap_or_default(),
            "ids": ids.clone(),
            "total": ids.len(),
            "position": 0,
            "canCalculateChanges": true
        })
    }

    // ---- Identity ------------------------------------------------------

    /// `Identity/get` — configured + server-pulled allowed-froms (§2.1). Seeds a
    /// default identity from the account's own address on first access.
    async fn identity_get(&self, account_id: &str, rt: &AccountRuntime, args: &Value) -> Value {
        self.ensure_default_identity(account_id, rt).await;
        self.ensure_server_identities(account_id).await;
        let rows = match self.store().list_identities(account_id).await {
            Ok(v) => v,
            Err(e) => return server_fail(&e),
        };
        let wanted: Option<Vec<String>> = args.get("ids").and_then(Value::as_array).map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(String::from)
                .collect()
        });
        let list: Vec<Value> = rows
            .iter()
            .filter(|r| wanted.as_ref().is_none_or(|ids| ids.contains(&r.id)))
            .map(identity_json)
            .collect();
        json!({
            "accountId": account_id,
            "state": "identity-0",
            "list": list,
            "notFound": []
        })
    }

    /// `Identity/query` — the ids of the account's identities.
    async fn identity_query(&self, account_id: &str, rt: &AccountRuntime) -> Value {
        self.ensure_default_identity(account_id, rt).await;
        self.ensure_server_identities(account_id).await;
        let ids: Vec<String> = self
            .store()
            .list_identities(account_id)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|r| r.id)
            .collect();
        json!({
            "accountId": account_id,
            "queryState": "identity-0",
            "ids": ids.clone(),
            "total": ids.len(),
            "position": 0,
            "canCalculateChanges": false
        })
    }

    /// Seed a `configured` identity from the account's own address if none
    /// exist yet, so `Identity/get` always returns at least the primary from.
    async fn ensure_default_identity(&self, account_id: &str, rt: &AccountRuntime) {
        let existing = self
            .store()
            .list_identities(account_id)
            .await
            .unwrap_or_default();
        if !existing.is_empty() || rt.identity.is_empty() {
            return;
        }
        let sent = self
            .store()
            .list_mailboxes(account_id)
            .await
            .unwrap_or_default()
            .into_iter()
            .find(|m| m.role.as_deref() == Some("sent"))
            .map(|m| m.id);
        let _ = self
            .store()
            .upsert_identity(&IdentityRow {
                id: format!("identity-{account_id}"),
                account_id: account_id.to_string(),
                name: String::new(),
                email: rt.identity.clone(),
                reply_to: None,
                signature_html: None,
                signature_text: None,
                signature_name: None,
                sent_mailbox_id: sent,
                source: "configured".to_string(),
            })
            .await;
    }

    /// Pull the deployment's server-advertised allowed-froms (`MW_ALLOWED_FROMS`,
    /// source `"server"`) into the identity store, beyond the single configured
    /// seed. Deduped by email against existing rows (case-insensitive) so repeated
    /// access is idempotent; the row id is derived from the address so the same
    /// allowed-from never duplicates. Best-effort — a store error leaves the
    /// configured identities intact.
    async fn ensure_server_identities(&self, account_id: &str) {
        let advertised = crate::identity::load_server_identities();
        if advertised.is_empty() {
            return;
        }
        let existing = self
            .store()
            .list_identities(account_id)
            .await
            .unwrap_or_default();
        // Reuse the Sent mailbox the configured identity resolved (if any).
        let sent = existing.iter().find_map(|r| r.sent_mailbox_id.clone());
        for si in advertised {
            let email = si.email.trim();
            if email.is_empty() {
                continue;
            }
            if existing.iter().any(|r| r.email.eq_ignore_ascii_case(email)) {
                continue;
            }
            let _ = self
                .store()
                .upsert_identity(&IdentityRow {
                    id: format!("identity-server-{account_id}-{}", identity_slug(email)),
                    account_id: account_id.to_string(),
                    name: si.name.clone(),
                    email: email.to_string(),
                    reply_to: si.reply_to.clone(),
                    signature_html: si.signature_html.clone(),
                    signature_text: si.signature_text.clone(),
                    signature_name: None,
                    sent_mailbox_id: sent.clone(),
                    source: "server".to_string(),
                })
                .await;
        }
    }

    /// Submit a draft's MIME through the account submitter, then file the sent
    /// copy into `Sent` (both upstream, best-effort, and in the local cache).
    pub(crate) async fn submit_email(
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

        // V4 DLP enforcement runs at `EmailSubmission/set` create time (see
        // `submission_set` → `dlp_block_error`), which gates BOTH the inline and
        // the deferred send paths before a submission is ever enqueued. By the
        // time we reach the actual dispatch here the draft has already cleared
        // DLP, so no second evaluation (and no duplicate audit) is needed.
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

        // File into Sent: upstream APPEND (best-effort) + local re-file. A
        // plugin/bridge backend's `submit` export *transmits* rather than appends
        // (the send already fired through `rt.submitter` above), and the provider
        // files the message into its own Sent folder on send — so a second upstream
        // append here would RE-SEND. Skip it for plugin-backed accounts; the local
        // re-file below still surfaces the sent copy on the JMAP Sent mailbox.
        // Standards IMAP keeps the best-effort upstream APPEND (byte-unchanged).
        let (sent_id, sent_name) = self.ensure_role_mailbox(account_id, "sent", "Sent").await?;
        let sent_ref = RawMailboxRef {
            name: sent_name,
            uidvalidity: 0,
        };
        if !self.is_plugin_backed(account_id) {
            tolerant(rt.backend.append(&sent_ref, &raw, &[Flag::Seen]).await)?;
        }

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
        // Remove the original draft now that it has been sent + filed: drop it
        // from the cache + index and record the Email destroyed change.
        let draft_mailbox = msg.mailbox_id.clone();
        self.store().delete_message(email_id).await?;
        let _ = self.search().delete(email_id);
        self.record_change(account_id, ChangeType::Email, email_id, ChangeOp::Destroyed)
            .await?;
        self.record_change(
            account_id,
            ChangeType::Mailbox,
            &draft_mailbox,
            ChangeOp::Updated,
        )
        .await?;
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
    pub(crate) async fn imap_ref_for(&self, stable_id: &str) -> Result<Option<MessageRef>> {
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

/// The first `sort` comparator of a query, if any (frozen §2.1 sort set).
fn first_comparator(args: &Value) -> Option<Comparator> {
    args.get("sort")
        .and_then(Value::as_array)
        .and_then(|a| a.first())
        .and_then(|c| serde_json::from_value(c.clone()).ok())
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

/// Build the base [`ComposeRequest`] (headers + bodies) from an `Email/set`
/// create spec. Attachment resolution is layered on by
/// [`Engine::compose_from_spec`], which needs an async blob lookup.
fn compose_base_from_spec(spec: &Value, identity: &str, message_id: &str) -> ComposeRequest {
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
        attachments: Vec::new(),
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
pub(crate) fn recipients(email: &mw_jmap::Email) -> Vec<String> {
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

/// Build the structured `dlpBlocked` `notCreated` error (frozen §2.2) when any
/// DLP verdict is a block, else `None`. The `verdicts` are the redacted DLP
/// verdicts (detector tokens only — never matched content).
fn dlp_block_error(verdicts: &[crate::security::types::DlpVerdict]) -> Option<Value> {
    let blocking: Vec<&crate::security::types::DlpVerdict> =
        verdicts.iter().filter(|v| v.blocked).collect();
    if blocking.is_empty() {
        return None;
    }
    let description = blocking
        .iter()
        .map(|v| v.rule_name.clone())
        .collect::<Vec<_>>()
        .join("; ");
    Some(json!({
        "type": "dlpBlocked",
        "description": description,
        "verdicts": blocking,
    }))
}

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

/// The JMAP `EmailSubmission` object for a stored row (frozen §2.1).
fn submission_json(row: &SubmissionRow) -> Value {
    json!({
        "id": row.id,
        "emailId": row.email_id,
        "identityId": row.identity_id,
        "sendAt": row.send_at,
        "undoStatus": row.undo_status,
        "mailwomanHoldSeconds": row.hold_seconds,
    })
}

/// The JMAP `Identity` object for a stored row (frozen §2.1). `source`
/// (`"configured"` | `"server"`) is surfaced additively so a client can tell a
/// server-advertised allowed-from from the account's own configured identity.
fn identity_json(row: &IdentityRow) -> Value {
    json!({
        "id": row.id,
        "name": row.name,
        "email": row.email,
        "replyTo": row.reply_to,
        "signatureHtml": row.signature_html,
        "signatureText": row.signature_text,
        "sentMailboxId": row.sent_mailbox_id,
        "source": row.source,
    })
}

/// A filesystem/id-safe slug of an email address for a server-identity row id
/// (non-alphanumerics collapse to `-`), so the same allowed-from maps to a stable
/// row on every pull.
fn identity_slug(email: &str) -> String {
    email
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}
