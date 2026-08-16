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
use crate::engine::{Engine, IndexPatch};
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
    ///
    /// **`maxChanges` is honoured, and `hasMoreChanges` is true when it bites**
    /// (26.20 t22-e2, finding V6). Before this, `maxChanges` was ignored here
    /// exactly as it was on `Email/queryChanges` — measured, a 550-change tail
    /// came back in full with `hasMoreChanges: false`, so a client asking for 50
    /// received 550 and was told that was all of them. Fixing only
    /// `queryChanges` would have left the identical bug one method over, which
    /// is why it is fixed in the shared handler that serves `Email/changes`,
    /// `Mailbox/changes` and `EmailSubmission/changes` alike.
    ///
    /// Unlike `queryChanges`, truncating here is **legal and safe**: RFC 8620
    /// §5.2 gives `Foo/changes` a `hasMoreChanges` flag, and
    /// [`Engine::build_changes_limited`] sets `newState` to the last row
    /// actually returned, so the client's next call resumes exactly where this
    /// one stopped instead of skipping the remainder.
    ///
    /// A `sinceState` this account never reached is `cannotCalculateChanges`,
    /// per the spec — previously it produced an empty diff, which tells a client
    /// with a stale or invented state that it is up to date.
    async fn type_changes(&self, account_id: &str, kind: ChangeType, args: &Value) -> Value {
        let since = args
            .get("sinceState")
            .and_then(Value::as_str)
            .unwrap_or("0");
        let max_changes = args
            .get("maxChanges")
            .and_then(Value::as_u64)
            .map(|n| n.min(i64::MAX as u64) as i64)
            .unwrap_or(DEFAULT_MAX_QUERY_CHANGES);

        let current = match self.store().current_state(account_id, kind.as_str()).await {
            Ok(n) => n,
            Err(e) => return server_fail(&EngineError::Store(e)),
        };
        if since.parse::<u64>().map(|n| n > current).unwrap_or(true) {
            return cannot_calculate_changes("sinceState is not a known state");
        }

        match self
            .build_changes_limited(account_id, kind, since, Some(max_changes))
            .await
        {
            Ok(Some(changes)) => {
                let mut v = serde_json::to_value(&changes).unwrap_or_else(|_| json!({}));
                if let Some(obj) = v.as_object_mut() {
                    obj.insert("accountId".into(), json!(account_id));
                }
                v
            }
            // A single state holds more rows than `maxChanges` — a batched
            // `Email/set` over a large selection writes N rows at one state —
            // so there is no page that is both within the cap and lossless.
            Ok(None) => cannot_calculate_changes(
                "one state holds more changes than maxChanges; raise it or refetch",
            ),
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
        let page = match self.query_page(account_id, args).await {
            Ok(p) => p,
            Err(QueryFail::AnchorNotFound) => return anchor_not_found(),
            Err(QueryFail::Engine(e)) => return server_fail(&e),
        };

        let mut out = json!({
            "accountId": account_id,
            "queryState": self.type_state(account_id, ChangeType::Email).await.unwrap_or_default(),
            "ids": page.ids,
            "position": page.position,
            "canCalculateChanges": true
        });
        page.publish_total(&mut out);
        out
    }

    /// Resolve an `Email/query` to the **page the client asked for**, pushing
    /// `position`/`limit` into SQL where the filter allows it (26.20 t22-e2).
    ///
    /// Routing is the frozen rule (§2.1): `mw-search` for any
    /// full-text/attachment/custom-sort condition, the SQL fast path for a pure
    /// `inMailbox` newest-first listing, and saved-search folders run their
    /// stored filter.
    ///
    /// # Three things this does that the unpaged predecessor did not
    ///
    /// **`limit`/`position` reach SQL.** They used to be a `.skip().take()` over
    /// a fully materialised folder, so a 50-row page of a 20 000-message mailbox
    /// read 20 000 ids. `Store::list_message_ids` now receives the requested
    /// numbers. A request that sends **no** `limit` is still unbounded, exactly
    /// as before: inventing a server cap would silently truncate an existing
    /// caller's list, which is the failure this lane condemns elsewhere (V9).
    ///
    /// **`anchor`/`anchorOffset` (RFC 8620 §5.5).** The window can start at an
    /// id rather than an index, which is how a client pages a folder that is
    /// being written to without rows sliding under it. On the SQL path the
    /// anchor's index comes from [`Store::message_position_in_mailbox`] — two
    /// statements, no folder scan — so anchor paging does not cost what it was
    /// introduced to avoid. An anchor that is not in the result is
    /// `anchorNotFound`, per the spec, rather than a silent page 1.
    ///
    /// **No unconditional `get_saved_search` (V7).** That lookup ran on *every*
    /// `Email/query`, including the pure `inMailbox` fast path where it can only
    /// ever miss. It is now deferred, and the deferral rests on a real
    /// invariant: **a saved-search folder id is never a `messages.mailbox_id`**,
    /// so any id that yields rows is a real mailbox and needs no lookup at all.
    /// The probe happens only when the fast path comes back empty — an empty
    /// mailbox, or a page past the end, pays one extra statement it did not pay
    /// before, and the overwhelmingly common case pays one fewer. Filters that
    /// do *not* take the fast path keep the eager expansion unchanged, including
    /// its existing behaviour of replacing the whole filter.
    async fn query_page(
        &self,
        account_id: &str,
        args: &Value,
    ) -> std::result::Result<QueryPage, QueryFail> {
        let want_total = args
            .get("calculateTotal")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let anchor = args.get("anchor").and_then(Value::as_str);
        let anchor_offset = args
            .get("anchorOffset")
            .and_then(Value::as_i64)
            .unwrap_or(0);
        let position = args.get("position").and_then(Value::as_u64).unwrap_or(0);
        // Absent `limit` stays unbounded, as it has always been.
        let sql_limit = args
            .get("limit")
            .and_then(Value::as_u64)
            .map(|n| n.min(i64::MAX as u64) as i64)
            .unwrap_or(i64::MAX);

        let raw_filter = args.get("filter").cloned().unwrap_or(Value::Null);
        let mut filter: EmailFilter = serde_json::from_value(raw_filter).unwrap_or_default();
        // A8: captured before the saved-search expansion below can replace
        // `filter` wholesale, so opting a saved-search folder into semantic
        // re-ranking works too. The stored filter's own flag wins if it sets one.
        let semantic = filter.semantic;
        let comparator = first_comparator(args);
        let sort = search_index::sort_from_comparator(comparator.as_ref());
        let custom_sort = sort != mw_search::Sort::received_desc();

        let route = query_route(&filter, custom_sort);
        if matches!(route, QueryRoute::Empty) {
            // No `inMailbox`, nothing for the search index to do: empty, as it
            // has always been. `total` is a real zero when asked for.
            return Ok(QueryPage {
                total: want_total.then_some(0),
                ..QueryPage::default()
            });
        }
        if let QueryRoute::Sql(mb) = route {
            // Where the window starts. `None` means an anchor was requested and
            // this mailbox does not contain it — which is either `anchorNotFound`
            // or a saved-search folder, decided after the probe below.
            let anchor_pos = match anchor {
                Some(a) => self.store().message_position_in_mailbox(&mb, a).await?,
                None => None,
            };
            let start = match anchor {
                Some(_) => anchor_pos.map(|p| p.saturating_add_signed(anchor_offset)),
                None => Some(position),
            };

            if let Some(start) = start {
                let ids = self
                    .store()
                    .list_message_ids(&mb, sql_limit, start as i64)
                    .await?;
                // Rows came back, or the anchor was found here: `mb` is a real
                // mailbox and the saved-search probe is provably pointless.
                if !ids.is_empty() || anchor_pos.is_some() {
                    return Ok(QueryPage {
                        ids,
                        position: start,
                        total: self.exact_mailbox_total(&mb, want_total).await?,
                        total_truncated: false,
                    });
                }
            }

            match self.store().get_saved_search(&mb).await? {
                // It was a saved-search folder after all: run its stored filter
                // through the search path below.
                Some(ss) => filter = serde_json::from_str(&ss.query_json).unwrap_or_default(),
                None if anchor.is_some() => return Err(QueryFail::AnchorNotFound),
                // A real mailbox with nothing at this offset.
                None => {
                    return Ok(QueryPage {
                        ids: Vec::new(),
                        position: start.unwrap_or(0),
                        total: self.exact_mailbox_total(&mb, want_total).await?,
                        total_truncated: false,
                    });
                }
            }
        } else if let Some(mb) = filter.in_mailbox.clone()
            && let Some(ss) = self.store().get_saved_search(&mb).await?
        {
            // Unchanged from 26.19 for the non-fast-path filters: a saved-search
            // folder id in `inMailbox` replaces the filter wholesale.
            filter = serde_json::from_str(&ss.query_json).unwrap_or_default();
        }

        let hits = self.search_ids(account_id, &filter, sort, semantic).await?;
        let start = match anchor {
            Some(a) => match hits.ids.iter().position(|id| id == a) {
                Some(p) => (p as u64).saturating_add_signed(anchor_offset),
                None => return Err(QueryFail::AnchorNotFound),
            },
            None => position,
        };
        let total = if want_total && !hits.truncated {
            Some(hits.ids.len() as u64)
        } else {
            None
        };
        let ids: Vec<String> = hits
            .ids
            .into_iter()
            .skip(start as usize)
            .take(usize::try_from(sql_limit).unwrap_or(usize::MAX))
            .collect();
        Ok(QueryPage {
            ids,
            position: start,
            total,
            total_truncated: want_total && hits.truncated,
        })
    }

    /// `COUNT(*)` for the fast path's `calculateTotal`, or `None` when the client
    /// did not ask — the point of asking being that a client which does not want
    /// a total does not pay for one (t22 OQ-5).
    async fn exact_mailbox_total(&self, mailbox_id: &str, want: bool) -> Result<Option<u64>> {
        if !want {
            return Ok(None);
        }
        Ok(Some(
            self.store().count_messages_in_mailbox(mailbox_id).await?,
        ))
    }

    /// Resolve an `Email/query` to the full ordered id list (before paging).
    ///
    /// `pub(crate)` so the A8 re-rank tests can drive the real filter → search →
    /// re-rank path without standing up a mock account backend; the JSON envelope
    /// around it is already covered by the V2 integration suite.
    ///
    /// Paging arguments are **stripped** rather than honoured: this is the
    /// "whole result" entry point, and a caller that passes a `limit` through it
    /// would get a page back under a name that promises everything.
    pub(crate) async fn query_ids(&self, account_id: &str, args: &Value) -> Result<Vec<String>> {
        let mut unpaged = args.clone();
        if let Some(obj) = unpaged.as_object_mut() {
            for k in ["position", "limit", "anchor", "anchorOffset"] {
                obj.remove(k);
            }
        }
        match self.query_page(account_id, &unpaged).await {
            Ok(p) => Ok(p.ids),
            Err(QueryFail::Engine(e)) => Err(e),
            // Unreachable: `anchor` was just removed.
            Err(QueryFail::AnchorNotFound) => Err(EngineError::Protocol(
                "anchorNotFound from an unpaged query".into(),
            )),
        }
    }

    /// Run the `mw-search` half of a query: account scoping, the compiled filter,
    /// and the A8 re-rank, reporting whether the index cap truncated the result
    /// (V9).
    async fn search_ids(
        &self,
        account_id: &str,
        filter: &EmailFilter,
        sort: mw_search::Sort,
        semantic: Option<bool>,
    ) -> Result<mw_search::SearchHits> {
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
        let sq = search_index::build_search_query(filter, sort, &mailbox_ids, scope);
        // `search_hits`, not `search`: the latter cannot say whether `MAX_HITS`
        // cut the result, and a `total` derived from a cut result is a wrong
        // number that looks exactly like a right one (V9).
        let mut hits = self
            .search()
            .search_hits(&sq, 0)
            .map_err(|e| EngineError::Protocol(format!("search: {e}")))?;

        // A8 (26.19, SPEC §10.4/§14.3): opt-in semantic re-rank. Reached ONLY when
        // the request asked for it AND the deployment attached an embedding
        // provider; otherwise `ids` is returned exactly as the lexical index
        // produced it, unchanged from 26.18. The pass cannot fail the query — every
        // degradation path leaves the order untouched (see `search_semantic`).
        //
        // Note this DOES override the requested `sort` for the re-ranked window,
        // and that is the point: the web always sends `receivedAt desc`, so asking
        // for `semantic` is asking for relevance order instead. Paging is applied
        // by the caller AFTER this, so page 1 shows the most similar hits.
        if filter.semantic.or(semantic) == Some(true)
            && let Some(provider) = self.embedding_provider()
        {
            // The web pairs `semantic` with `text`; `subject`/`body` are accepted
            // as the query source too so an operator-style filter opts in usefully
            // instead of silently degrading for want of a `text` key.
            let query_text = filter
                .text
                .as_deref()
                .or(filter.subject.as_deref())
                .or(filter.body.as_deref())
                .unwrap_or_default();
            let report = crate::search_semantic::rerank_hits(
                &provider,
                self.store(),
                self.search_handle(),
                account_id,
                query_text,
                &mut hits.ids,
            )
            .await;
            tracing::debug!(
                considered = report.considered,
                scored = report.scored,
                filled = report.filled,
                skipped_mismatch = report.skipped_mismatch,
                degraded = report.degraded,
                "semantic re-rank"
            );
        }
        Ok(hits)
    }

    /// `Email/queryChanges` (RFC 8620 §5.6): what changed in this query since
    /// `sinceQueryState`, and **only** what changed (26.20 t22-e2).
    ///
    /// # What this replaced, and why it was worth replacing
    ///
    /// It returned the **entire current query** as `added`, at every index, on
    /// every call. Measured on a 20 000-message folder: **1 749 125 bytes**
    /// against the **3 580-byte** page it exists to spare the client — 489× the
    /// bytes, and 1.5–1.7× the *time*, of simply refetching. A delta method that
    /// costs more than the refetch is not an optimisation with a bug in it; it
    /// is a method that has never once been worth calling. The `oldQueryState`
    /// it reported was the client's own argument echoed back, so a client could
    /// not even tell it had been given a full resync.
    ///
    /// # The shape now
    ///
    /// The change log is the source, not the query: the ids that changed since
    /// `since` go into `removed`, and those that still match go back into
    /// `added` with their current index — which is what an id whose *position*
    /// moved needs, and is the reason an id appears in both lists.
    ///
    /// **`maxChanges` refuses rather than truncates.** `Foo/queryChanges` has no
    /// `hasMoreChanges`; RFC 8620 §5.6 says a server that cannot answer within
    /// `maxChanges` MUST return `cannotCalculateChanges`, and the client resyncs
    /// with a plain `Email/query`. That is the honest answer — a truncated delta
    /// silently corrupts the client's list, because the client applies it and
    /// believes it is up to date. The cap reaches SQL
    /// ([`Store::changes_since_limited`]), so a refusal costs one bounded read
    /// rather than materialising a tail in order to measure it.
    ///
    /// A client that sends no `maxChanges` gets [`DEFAULT_MAX_QUERY_CHANGES`].
    /// JMAP permits an unbounded delta, but the `changes` table is append-only —
    /// nothing in `crates/` prunes it — so "unbounded" means "the whole history
    /// of this deployment" for a client returning from a long absence, and the
    /// spec-defined fallback for refusing is exactly the full refetch such a
    /// client should be doing anyway.
    ///
    /// **`upToId` bounds the query read.** It is the client's statement of how
    /// far its own list reaches; nothing past it can affect what the client
    /// shows, so only that prefix is materialised — see [`Engine::query_prefix`].
    ///
    /// **`total` is emitted only for `calculateTotal: true`**, and only when it
    /// is exact (V9), matching `Email/query`. It used to be `ids.len()` of the
    /// whole materialised query, unconditionally.
    async fn email_query_changes(&self, account_id: &str, args: &Value) -> Value {
        let since = args
            .get("sinceQueryState")
            .and_then(Value::as_str)
            .unwrap_or("0");
        let max_changes = args
            .get("maxChanges")
            .and_then(Value::as_u64)
            .map(|n| n.min(i64::MAX as u64) as i64)
            .unwrap_or(DEFAULT_MAX_QUERY_CHANGES);

        // A state we cannot diff from is a refusal, not an empty delta: a client
        // told "nothing changed" stops asking.
        match self.resumable_from(account_id, since).await {
            Ok(true) => {}
            Ok(false) => return cannot_calculate_changes("sinceQueryState is not a known state"),
            Err(e) => return server_fail(&e),
        }

        let changes = match self
            .build_changes_limited(account_id, ChangeType::Email, since, Some(max_changes))
            .await
        {
            Ok(Some(c)) => c,
            // The cap cannot be honoured without losing changes — a single
            // state holds more rows than `maxChanges`, which is what a batched
            // `Email/set` over a large selection produces.
            Ok(None) => {
                return cannot_calculate_changes(
                    "one state holds more changes than maxChanges; refetch the query instead",
                );
            }
            Err(e) => return server_fail(&e),
        };
        if changes.has_more_changes {
            return cannot_calculate_changes(
                "more changes than maxChanges; refetch the query instead",
            );
        }

        // Every id that changed leaves the client's list; those still matching
        // re-enter it at their current index.
        let touched: Vec<String> = changes
            .created
            .iter()
            .chain(changes.updated.iter())
            .cloned()
            .collect();
        let removed: Vec<String> = changes
            .destroyed
            .iter()
            .cloned()
            .chain(touched.iter().cloned())
            .collect();

        let added: Vec<Value> = if touched.is_empty() {
            // Nothing can be added, so the query is not read at all — the common
            // case for a push tick that only saw deletions.
            Vec::new()
        } else {
            let up_to_id = args.get("upToId").and_then(Value::as_str);
            let ids = match self.query_prefix(account_id, args, up_to_id).await {
                Ok(v) => v,
                Err(e) => return server_fail(&e),
            };
            let mut added: Vec<Value> = touched
                .iter()
                .filter_map(|id| {
                    ids.iter()
                        .position(|q| q == id)
                        .map(|i| json!({ "id": id, "index": i }))
                })
                .collect();
            // Ascending index is what a client applying them in order wants.
            added.sort_by_key(|v| v.get("index").and_then(Value::as_u64).unwrap_or(0));
            added
        };

        let mut out = json!({
            "accountId": account_id,
            "oldQueryState": changes.old_state,
            "newQueryState": changes.new_state,
            "removed": removed,
            "added": added
        });
        match self.query_total(account_id, args).await {
            Ok(total) => total.publish_total(&mut out),
            Err(e) => return server_fail(&e),
        }
        out
    }

    /// Can a delta be computed from `since`? `false` for a state this account
    /// never reached — a client that invented one, or one from a database that
    /// has been replaced underneath it.
    ///
    /// State `0` is always resumable: it is the "I have nothing" state every
    /// client starts from, and it is below `current` by construction.
    async fn resumable_from(&self, account_id: &str, since: &str) -> Result<bool> {
        let Ok(n) = since.parse::<u64>() else {
            return Ok(false);
        };
        let current = self
            .store()
            .current_state(account_id, ChangeType::Email.as_str())
            .await?;
        Ok(n <= current)
    }

    /// The prefix of a query the client can actually be holding: everything up
    /// to and including `up_to_id`, or the whole query when the client did not
    /// say (26.20 t22-e2).
    ///
    /// On the SQL fast path the prefix is read as a prefix — one positional
    /// lookup plus a `LIMIT`ed read — rather than materialised and then cut. A
    /// client holding 50 rows of a 20 000-message folder therefore causes a
    /// 50-row read, which is the difference between `upToId` being a real bound
    /// and being decoration.
    async fn query_prefix(
        &self,
        account_id: &str,
        args: &Value,
        up_to_id: Option<&str>,
    ) -> Result<Vec<String>> {
        if let Some(id) = up_to_id {
            let filter: EmailFilter =
                serde_json::from_value(args.get("filter").cloned().unwrap_or(Value::Null))
                    .unwrap_or_default();
            let comparator = first_comparator(args);
            let sort = search_index::sort_from_comparator(comparator.as_ref());
            let custom_sort = sort != mw_search::Sort::received_desc();
            if let QueryRoute::Sql(mb) = query_route(&filter, custom_sort)
                && let Some(p) = self.store().message_position_in_mailbox(&mb, id).await?
            {
                return Ok(self
                    .store()
                    .list_message_ids(&mb, p.saturating_add(1).min(i64::MAX as u64) as i64, 0)
                    .await?);
            }
        }
        let mut ids = self.query_ids(account_id, args).await?;
        if let Some(id) = up_to_id
            && let Some(p) = ids.iter().position(|q| q == id)
        {
            ids.truncate(p + 1);
        }
        Ok(ids)
    }

    /// The `calculateTotal` half of a query, without its ids — for the paths
    /// that need the number but not the list. Returns a [`QueryPage`] carrying
    /// only the total so both callers publish it through the same rule (V9).
    async fn query_total(&self, account_id: &str, args: &Value) -> Result<QueryPage> {
        let want_total = args
            .get("calculateTotal")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if !want_total {
            return Ok(QueryPage::default());
        }
        let filter: EmailFilter =
            serde_json::from_value(args.get("filter").cloned().unwrap_or(Value::Null))
                .unwrap_or_default();
        let semantic = filter.semantic;
        let comparator = first_comparator(args);
        let sort = search_index::sort_from_comparator(comparator.as_ref());
        let custom_sort = sort != mw_search::Sort::received_desc();

        let (total, truncated) = match query_route(&filter, custom_sort) {
            QueryRoute::Sql(mb) => (
                Some(self.store().count_messages_in_mailbox(&mb).await?),
                false,
            ),
            QueryRoute::Empty => (Some(0), false),
            QueryRoute::Search => {
                let hits = self.search_ids(account_id, &filter, sort, semantic).await?;
                if hits.truncated {
                    (None, true)
                } else {
                    (Some(hits.ids.len() as u64), false)
                }
            }
        };
        Ok(QueryPage {
            total,
            total_truncated: truncated,
            ..QueryPage::default()
        })
    }

    // ---- Email/get ------------------------------------------------------

    /// `Email/get` — **one store batch per page**, not one per message
    /// (26.20 t22-e3g, plan row S5).
    ///
    /// # What this cost, and what the number is made of
    ///
    /// A 50-id page issued **151 statements** here (`build_email` in a loop:
    /// `get_message` + `get_envelope` + `get_message_meta` per id, plus one
    /// `type_state`) — the plan's 163 for the whole request, before `t22-e0`
    /// folded `sessionState`'s twelve counter SELECTs to three. Invisible on
    /// SQLite at ~34 ms; **425 ms on Postgres**, where each of those awaits is a
    /// network round trip and the page is what a user is waiting for.
    ///
    /// It is **four** now, whatever the page size: the message rows, the
    /// envelopes, the engine-local metadata, and the state token.
    ///
    /// # Why the envelope read is still per-id when a cache is attached
    ///
    /// This is the part that cannot be batched away without breaking something,
    /// so it is deliberate rather than overlooked. `get_message` and
    /// `get_message_meta` were always direct store reads and batch
    /// unconditionally. The **envelope** is different: it goes through
    /// [`Engine::cached_envelope`], which is what populates the header-window
    /// cache and what routes a zero-access account away from every shared tier.
    /// Reading it straight from the store would silently stop both, and
    /// `crates/mw-engine/tests/v6.rs` asserts each
    /// (`standard_account_populates_the_header_window_cache`,
    /// `zero_access_account_never_materializes_plaintext_in_the_engine_cache`).
    /// Those tests are the reason this branch exists rather than an obstacle
    /// to it.
    ///
    /// So: **no cache attached** — the default, and what the statement
    /// assertions measure — one `get_envelopes` for the page. **Cache
    /// attached** — per-id cache-aside as before, which costs SQL only on a miss
    /// and nothing on a hit, so it remains strictly cheaper than what it
    /// replaced.
    ///
    /// # The one read that stays per-message, stated rather than glossed
    ///
    /// A message with **no stored envelope** falls back to re-parsing its sealed
    /// body, one read each. That is a per-message read of *different bytes*
    /// rather than a repeated read of the same table, and it is the rare case: a
    /// synced mailbox stores envelopes. A page of POP3-ingested mail (which
    /// stores none) therefore still costs one body read per message. Not batched
    /// here because one statement returning fifty message bodies trades a
    /// round-trip problem for a memory one, and nothing has measured that trade.
    async fn email_get(&self, account_id: &str, args: &Value) -> Value {
        let empty = Vec::new();
        let requested: Vec<&str> = args
            .get("ids")
            .and_then(Value::as_array)
            .unwrap_or(&empty)
            .iter()
            .filter_map(Value::as_str)
            .collect();

        let assembled = match self.build_emails(&requested).await {
            Ok(v) => v,
            Err(e) => return server_fail(&e),
        };

        // `list` is positional against the request: a repeated id is answered
        // twice, because de-duplicating it drops an entry the client asked for.
        let mut list = Vec::with_capacity(assembled.len());
        let mut not_found = Vec::new();
        for (id, email) in requested.iter().zip(assembled) {
            match email {
                Some(e) => list.push(e),
                None => not_found.push(json!(id)),
            }
        }
        json!({
            "accountId": account_id,
            "state": self.type_state(account_id, ChangeType::Email).await.unwrap_or_default(),
            "list": list,
            "notFound": not_found
        })
    }

    /// Assemble a whole page of `mw_jmap::Email` JSON in a bounded number of
    /// store reads, positionally aligned with `stable_ids` (26.20 t22-e3g).
    ///
    /// `None` where the id names no stored message — the batch equivalent of
    /// [`Engine::build_email`]s `Ok(None)`, and what fills JMAP notFound.
    ///
    /// **Duplicate ids cost one read, not two.** The store reads are issued over
    /// the de-duplicated set and projected back positionally, so `["a", "a"]`
    /// reads `a` once and answers twice. Left un-deduplicated, a client could
    /// turn a 50-id page into fifty copies of one id and pay for all of them.
    async fn build_emails(&self, stable_ids: &[&str]) -> Result<Vec<Option<Value>>> {
        if stable_ids.is_empty() {
            return Ok(Vec::new());
        }
        let mut unique: Vec<String> = Vec::with_capacity(stable_ids.len());
        let mut slot: HashMap<&str, usize> = HashMap::with_capacity(stable_ids.len());
        for id in stable_ids {
            if !slot.contains_key(id) {
                slot.insert(id, unique.len());
                unique.push((*id).to_string());
            }
        }

        // Statements 1 and 2: everything that was never cache-aside.
        let msgs = self.store().get_messages(&unique).await?;
        let metas = self.store().get_message_metas(&unique).await?;

        // Statement 3, when no cache is attached — see `email_get` for why an
        // attached cache keeps the per-id path.
        let envelopes: Vec<Option<Vec<u8>>> = if self.v6_hooks().has_cache() {
            let mut out = Vec::with_capacity(unique.len());
            for (id, msg) in unique.iter().zip(&msgs) {
                out.push(match msg {
                    Some(m) => self.cached_envelope(&m.account_id, id).await?,
                    None => None,
                });
            }
            out
        } else {
            self.store().get_envelopes(&unique).await?
        };

        let mut built: Vec<Option<Value>> = Vec::with_capacity(unique.len());
        for ((id, msg), (envelope, meta)) in unique
            .iter()
            .zip(msgs)
            .zip(envelopes.into_iter().zip(metas))
        {
            let Some(msg) = msg else {
                built.push(None);
                continue;
            };
            // Only an envelope-less message reaches the body, one read each.
            let email = match envelope {
                Some(bytes) => serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({})),
                None => match &msg.blob_ref {
                    Some(blob) => match self.cached_body(&msg.account_id, id, blob).await? {
                        Some(raw) => mw_mime::parse(&raw)
                            .ok()
                            .and_then(|p| serde_json::to_value(p.email).ok())
                            .unwrap_or_else(|| json!({})),
                        None => json!({}),
                    },
                    None => json!({}),
                },
            };
            built.push(Some(patch_engine_fields(
                email,
                id,
                &msg,
                &meta.unwrap_or_default(),
            )));
        }

        // Project back onto the request, duplicates included.
        Ok(stable_ids
            .iter()
            .map(|id| built[slot[id]].clone())
            .collect())
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

        // The multi-id `update` is a BATCH (26.20 t22-e3s): every id's store
        // write happens in the loop, then the whole set costs exactly **one**
        // search-index commit, one batched flag write and one state increment
        // — with a change row per id, all sharing that one state.
        //
        // Before 26.20 each id paid its own Tantivy `commit()` + `reader
        // .reload()` inside `reindex_message`, which is where essentially the
        // entire cost of "mark this folder read" lived: 9.5 s on SQLite and
        // 26.5 s on Postgres for 500 ids, ~6 minutes for 20 000. The commit is
        // the expensive part, not the document write, so batching the documents
        // and committing once turns that into one commit's worth of work.
        //
        // Per-id failures still go to `notUpdated` and do not stop the rest
        // (RFC 8620 §5.3), and a failed id contributes no patch — so the index
        // can never publish an update the store rejected.
        let mut patches: Vec<IndexPatch> = Vec::new();
        // Mailbox rows resolved once per batch rather than once per id: 500 ids
        // in one folder used to mean 500 identical `get_mailbox` reads.
        let mut mailbox_names: HashMap<String, String> = HashMap::new();
        // Flag writes are collected here and applied in ONE `set_flags_batch`
        // after the loop (3 statements for 500 ids, against 1 500 through
        // per-id `set_flags`).
        let mut pending_flags: Vec<(String, String)> = Vec::new();
        if let Some(updates) = args.get("update").and_then(Value::as_object) {
            // One read for every id in the request. An id the batch does not
            // return falls back to `get_message` inside `update_email`, which
            // preserves the exact per-id error (`NotFound` → `notUpdated`) and
            // costs a statement only for ids that are missing anyway.
            let update_ids: Vec<String> = updates.keys().cloned().collect();
            let mut rows: HashMap<String, mw_store::Message> = HashMap::new();
            match self.store().get_messages(&update_ids).await {
                Ok(found) => {
                    for (id, msg) in update_ids.iter().zip(found) {
                        if let Some(msg) = msg {
                            rows.insert(id.clone(), msg);
                        }
                    }
                }
                // Not fatal: every id simply takes the per-id read below, which
                // is what this method did before 26.20.
                Err(e) => tracing::warn!("batched read for Email/set failed, per-id fallback: {e}"),
            }

            for (id, patch) in updates {
                match self
                    .update_email(
                        rt,
                        id,
                        patch,
                        rows.remove(id),
                        &mut mailbox_names,
                        &mut pending_flags,
                    )
                    .await
                {
                    Ok(indexed) => {
                        updated.insert(id.clone(), Value::Null);
                        patches.extend(indexed);
                    }
                    Err(e) => {
                        not_updated.insert(id.clone(), set_error(&e));
                    }
                }
            }
        }

        // The batched flag write, before anything is published to the index —
        // OQ-11's ordering: the store commits first, so a rejected id cannot
        // reach the index. `false` is the batch form of `NotFound`; those ids
        // move from `updated` to `notUpdated` and lose their index patch, which
        // is what keeps RFC 8620 §5.3's per-id semantics intact.
        if !pending_flags.is_empty() {
            let (failed, why): (Vec<&String>, Value) =
                match self.store().set_flags_batch(&pending_flags).await {
                    Ok(written) => (
                        pending_flags
                            .iter()
                            .zip(written)
                            .filter(|(_, ok)| !ok)
                            .map(|((id, _), _)| id)
                            .collect(),
                        set_error(&EngineError::Store(mw_store::StoreError::NotFound)),
                    ),
                    // Nothing was written, so nothing may be indexed or reported
                    // as updated.
                    Err(e) => (
                        pending_flags.iter().map(|(id, _)| id).collect(),
                        set_error(&EngineError::Protocol(format!(
                            "batched flag write failed: {e}"
                        ))),
                    ),
                };
            if !failed.is_empty() {
                let failed: std::collections::HashSet<&str> =
                    failed.into_iter().map(String::as_str).collect();
                for id in &failed {
                    updated.remove(*id);
                    not_updated.insert((*id).to_string(), why.clone());
                }
                patches.retain(|p| !failed.contains(p.stable_id.as_str()));
            }
        }

        if !patches.is_empty() {
            self.reindex_messages(&patches).await;
        }
        // Only non-move ids contribute the batch's change rows; a moved id was
        // already recorded by `move_email`.
        let changed: Vec<String> = patches
            .iter()
            .filter(|p| !p.moved)
            .map(|p| p.stable_id.clone())
            .collect();
        if !changed.is_empty() {
            // **One state increment, but a row per id.** `Email/changes` returns
            // rows with `state > since`, so N rows all sharing state `S+1` give
            // the single state bump this batching is for *and* a truthful change
            // list — a single row naming one id would leave every other device
            // showing stale flags for the remaining N-1 until something else
            // touched them.
            if let Err(e) = self
                .store()
                .record_changes(
                    account_id,
                    ChangeType::Email.as_str(),
                    &changed,
                    ChangeOp::Updated.as_str(),
                )
                .await
            {
                tracing::warn!("recording the batched Email/set changes failed: {e}");
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
    /// (pin/snooze/follow-up), and/or a mailbox move (plan §1.2, §1.5).
    ///
    /// Returns the [`IndexPatch`] the caller must fold into the batch, or `None`
    /// when there is nothing for the batch to do — either the patch changed
    /// nothing, or it was a move, which records its own change and re-index.
    ///
    /// **This method no longer commits the index, records a change, or writes
    /// flags to the store itself** (26.20 t22-e3s): all three are the batch's
    /// job in [`Engine::email_set`], which is what makes a 500-id update cost
    /// one index commit and three flag statements instead of 500 and 1 500.
    ///
    /// `row` is the message this id's entry of the caller's batched
    /// [`mw_store::Store::get_messages`] returned; `None` falls back to the
    /// per-id read so a missing id still produces its own `NotFound`.
    /// `mailbox_names` memoizes mailbox-row reads across the batch, and
    /// `pending_flags` collects `(stable_id, flags_json)` for the single
    /// [`mw_store::Store::set_flags_batch`] the caller runs afterwards.
    ///
    /// The **flag delta and the upstream `store_flags` call remain per-id** —
    /// each id can want a different add/remove set, and the backend seam takes
    /// one `MessageRef` slice per delta. Batching those is a backend-protocol
    /// change, not part of this adoption; the win taken here is the store and
    /// index round trips.
    async fn update_email(
        &self,
        rt: &AccountRuntime,
        id: &str,
        patch: &Value,
        row: Option<mw_store::Message>,
        mailbox_names: &mut HashMap<String, String>,
        pending_flags: &mut Vec<(String, String)>,
    ) -> Result<Option<IndexPatch>> {
        let msg = match row {
            Some(msg) => msg,
            None => self
                .store()
                .get_message(id)
                .await
                .map_err(EngineError::Store)?,
        };
        let mut touched = false;
        let mut new_flags: Option<Vec<Flag>> = None;
        let mut new_pinned: Option<bool> = None;

        if let Some(kw) = patch.get("keywords").and_then(Value::as_object) {
            let kw_map: HashMap<String, bool> = kw
                .iter()
                .filter_map(|(k, v)| v.as_bool().map(|b| (k.clone(), b)))
                .collect();
            let desired = keywords_to_flags(&kw_map);
            let current = flags_from_json(&msg.flags_json);
            let (add, remove) = flag_delta(&current, &desired);

            // The message row is already in hand, and it carries the same
            // `mailbox_id`/`uidvalidity`/`uid` that `imap_ref_for` would re-read
            // — so resolve the ref from it and only pay for the mailbox name,
            // memoized across the batch.
            if let Some(mref) = self.imap_ref_from(&msg, mailbox_names).await? {
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
            let flags_json = flags_to_json(&stored);
            if moves_mailbox(patch, &msg.mailbox_id) {
                // A patch that also moves must have its flags on disk *before*
                // `move_email` runs, because `set_flags` adjusts the `unread`
                // counter of whichever mailbox the message is in **at the time
                // of the write** (t22-e1's V8 work). Deferring it into the
                // post-loop batch would decrement the destination's counter
                // instead of the source's, quietly corrupting both. Rare, and
                // not the path this batching exists for — write it through and
                // keep the ordering identical to pre-26.20.
                self.store().set_flags(id, &flags_json).await?;
            } else {
                pending_flags.push((id.to_string(), flags_json));
            }
            new_flags = Some(stored);
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
            new_pinned = Some(meta.pinned);
            touched = true;
        }

        let mut moved = false;
        if let Some(target) = move_target(patch, &msg.mailbox_id) {
            self.move_email(rt, id, &target).await?;
            moved = true;
        }

        // A moved id joins the re-index batch too. `move_email` re-keys the
        // index entry with `Index::relocate`, which rebuilds from the stored
        // `doc_json` and so keeps the keywords the document already had — a
        // patch that archives AND marks read would otherwise leave the index
        // asserting the message is still unread. It does not contribute the
        // batch's change row, because `move_email` recorded its own.
        Ok(touched.then(|| IndexPatch {
            stable_id: id.to_string(),
            flags: new_flags,
            pinned: new_pinned,
            moved,
        }))
    }

    /// The IMAP [`MessageRef`] for a message row already in hand, or `None` for
    /// a POP3/local message the backend cannot address.
    ///
    /// Same result as [`Engine::imap_ref_for`] without its `message_location`
    /// read — the caller's [`mw_store::Message`] already carries those columns.
    /// `names` memoizes the mailbox-name lookup, which for a batch confined to
    /// one folder turns N reads into one.
    async fn imap_ref_from(
        &self,
        msg: &mw_store::Message,
        names: &mut HashMap<String, String>,
    ) -> Result<Option<MessageRef>> {
        // uidvalidity 0 marks an engine-local message (draft/sent) with no server
        // coordinates; there is nothing upstream to address.
        if msg.uidvalidity == 0 {
            return Ok(None);
        }
        let name = match names.get(&msg.mailbox_id) {
            Some(cached) => cached.clone(),
            None => {
                let name = self
                    .store()
                    .get_mailbox(&msg.mailbox_id)
                    .await
                    .map_err(EngineError::Store)?
                    .name;
                names.insert(msg.mailbox_id.clone(), name.clone());
                name
            }
        };
        Ok(Some(MessageRef::Imap {
            mailbox: RawMailboxRef {
                name,
                uidvalidity: msg.uidvalidity,
            },
            uidvalidity: msg.uidvalidity,
            uid: msg.uid,
        }))
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

/// The default `maxChanges` for `*/changes` and `Email/queryChanges` when the
/// client sends none (26.20 t22-e2).
///
/// JMAP permits an unbounded delta, and this is deliberately not that. The
/// `changes` table is append-only — **no code in `crates/` ever deletes from
/// it** — so "every change since state 0" is the entire history of a
/// deployment, and the two methods that read it are the ones a reconnecting
/// client calls first. Both have a defined answer for exceeding the bound:
/// `*/changes` truncates and says so via `hasMoreChanges`, `queryChanges`
/// refuses via `cannotCalculateChanges` and the client refetches. Neither loses
/// data; both bound the response.
///
/// 500 is chosen against the measurement this lane started from: a 20 000-row
/// folder produced 550 changes for 551 flag writes, so a cap at 500 is reached
/// by a bulk operation on a large selection and by very little else. It is not
/// tuned to a byte budget, because the response size depends on id width; it is
/// tuned to "a client that has been away long enough for a refetch to be the
/// cheaper answer".
const DEFAULT_MAX_QUERY_CHANGES: i64 = 500;

/// One resolved page of an `Email/query` (26.20 t22-e2).
#[derive(Default)]
struct QueryPage {
    /// The window the client asked for, already paged — not the whole query.
    ids: Vec<String>,
    /// The absolute index of `ids[0]` in the full query order.
    position: u64,
    /// The exact number of matching messages. `None` unless the client asked
    /// (`calculateTotal`) **and** an exact answer was available.
    total: Option<u64>,
    /// The client asked for a total and the search index cap made an exact one
    /// impossible (V9).
    total_truncated: bool,
}

impl QueryPage {
    /// Write `total` into a response, or say plainly that there is not one.
    ///
    /// Three distinct outcomes, which is one more than a bare `total` field can
    /// express and the reason this is a method rather than an inline `insert`:
    ///
    /// * the client did not ask → **no `total` key**, and no `COUNT(*)` was
    ///   issued to produce one;
    /// * the client asked and the number is exact → `total`;
    /// * the client asked and the number would be a **truncated** one → still no
    ///   `total`, plus `mailwomanCannotCalculateTotal: true` (V9).
    ///
    /// The third case is the point. `mw_search` caps an unbounded search at
    /// 100 000 documents, so a 200 000-document index used to answer
    /// `total: 100000` — a wrong number, indistinguishable from a right one, and
    /// the number a client sizes its scrollbar and its paging against. An absent
    /// total is visibly absent; a truncated one is not.
    fn publish_total(&self, out: &mut Value) {
        let Some(obj) = out.as_object_mut() else {
            return;
        };
        match self.total {
            Some(n) => {
                obj.insert("total".into(), json!(n));
            }
            None if self.total_truncated => {
                obj.insert("mailwomanCannotCalculateTotal".into(), json!(true));
            }
            None => {}
        }
    }
}

/// Why an `Email/query` could not be answered as asked.
enum QueryFail {
    /// RFC 8620 §5.5: the `anchor` id is not in the query result.
    AnchorNotFound,
    Engine(EngineError),
}

impl From<EngineError> for QueryFail {
    fn from(e: EngineError) -> Self {
        QueryFail::Engine(e)
    }
}

impl From<mw_store::StoreError> for QueryFail {
    fn from(e: mw_store::StoreError) -> Self {
        QueryFail::Engine(EngineError::Store(e))
    }
}

/// Which of the three ways an `Email/query` gets answered (frozen routing rule
/// §2.1).
enum QueryRoute {
    /// Straight from SQL, newest-first, in this mailbox.
    Sql(String),
    /// Fast-path shape but **no `inMailbox`**: there is no listing to run, and
    /// the answer is empty rather than "every message in the account".
    ///
    /// This case is a deliberate carry-over, not an oversight. `master`'s
    /// `query_ids` returned `Vec::new()` here (`let Some(mb) = … else { return
    /// Ok(Vec::new()) }`), and two `mw-server` suites say so in prose —
    /// `t13_jwz.rs` and `t13_geoip.rs` both note "an unfiltered `Email/query`
    /// returns nothing". Routing it to the search index instead would quietly
    /// turn a request that matched nothing into one that matches the account's
    /// entire mailbox set, which is not a paging change.
    Empty,
    /// Needs the search index.
    Search,
}

/// Decide the route once, so the pager, the `upToId` prefix read and the
/// `calculateTotal` counter cannot disagree about which path a given query
/// takes. A disagreement would surface as a `total` counted over one order
/// beside a page read from another.
fn query_route(filter: &EmailFilter, custom_sort: bool) -> QueryRoute {
    if custom_sort || filter.needs_search() {
        return QueryRoute::Search;
    }
    match filter.in_mailbox.clone() {
        Some(mb) => QueryRoute::Sql(mb),
        None => QueryRoute::Empty,
    }
}

/// Patch the engine-owned fields onto a parsed/unsealed `Email` document.
///
/// The single place `Email/get`s per-id path and its batched page agree on what
/// an `Email` looks like (26.20 t22-e3g). It was inline in `build_email`; a
/// batch that re-implemented it would drift from the per-id path field by field,
/// and the drift would show as a client rendering one page differently from
/// another rather than as a failure.
fn patch_engine_fields(
    mut email: Value,
    stable_id: &str,
    msg: &mw_store::Message,
    meta: &StoredMeta,
) -> Value {
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
    email
}

/// RFC 8620 §5.5 `anchorNotFound`: the `anchor` id is not in the query result.
fn anchor_not_found() -> Value {
    json!({
        "type": "anchorNotFound",
        "description": "the anchor id is not in this query's result"
    })
}

/// RFC 8620 §5.2/§5.6 `cannotCalculateChanges`: the client must refetch.
fn cannot_calculate_changes(why: &str) -> Value {
    json!({ "type": "cannotCalculateChanges", "description": why })
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

/// The mailbox an `Email/set` update patch moves a message to, if it moves at
/// all: the first `mailboxIds` entry set to `true` that is not where the message
/// already is.
///
/// Extracted so the two places that need the answer cannot drift apart — the
/// move itself, and the flag write, which must go through to the store *before*
/// a move rather than into the batch (26.20 t22-e3s).
fn move_target(patch: &Value, current_mailbox_id: &str) -> Option<String> {
    patch
        .get("mailboxIds")
        .and_then(Value::as_object)?
        .iter()
        .find(|(_, v)| v.as_bool() == Some(true))
        .map(|(k, _)| k.clone())
        .filter(|target| target != current_mailbox_id)
}

/// Whether this patch relocates the message — see [`move_target`].
fn moves_mailbox(patch: &Value, current_mailbox_id: &str) -> bool {
    move_target(patch, current_mailbox_id).is_some()
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

#[cfg(test)]
mod query_paging_tests {
    //! `Email/query` paging, `Email/queryChanges`, `Email/changes` (26.20 t22-e2).
    //!
    //! # The instruments, and why each one rather than the obvious one
    //!
    //! * **`rows_returned`, not statement count, for pushdown.** Slicing a fully
    //!   materialised folder in Rust and pushing `LIMIT`/`OFFSET` into SQL both
    //!   issue *one* statement against *one* table, and the recorded SQL text
    //!   carries placeholders rather than bound values — so neither the count nor
    //!   the text can tell them apart. What differs is how many rows the database
    //!   handed back: 50 against 20 000. That number is sqlx's own
    //!   `rows_returned`, captured by the shared recorder in
    //!   [`crate::state::session_state_tests`].
    //! * **Response bytes, not elapsed time, for `queryChanges`.** The failure
    //!   was 489x the bytes of the refetch it replaces, and bytes are what a push
    //!   tick pays for. The "before" number is not quoted from a document —
    //!   `master_shape_query_changes` reproduces the pre-fix response *verbatim*
    //!   and is measured in the same run, so the ratio is derived here rather
    //!   than asserted from memory.
    //! * **The absence of a *named* statement, not a smaller total, for V7.**
    //!   Dropping `get_saved_search` is one statement in fifteen; a total-count
    //!   assertion rounds it away, and a later regression would not move the
    //!   total either.
    //! * **A refusal that is actually reached.** A `maxChanges` implementation
    //!   that never refuses has not been tested, so every cap assertion here has
    //!   a leg that trips it and a leg that does not.
    //!
    //! Every scale assertion states the value it produces on `master` in its
    //! failure message, so a future reader can tell a fix from a tautology.

    use std::collections::HashSet;
    use std::thread::ThreadId;

    use mw_store::{
        AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, SavedSearchRow,
        ServerKey, Store,
    };
    use serde_json::json;

    use super::*;
    use crate::state::session_state_tests::{Stmts, counted, db_threads};

    /// The folder size the scale assertions run against.
    ///
    /// Big enough that the pre-fix `queryChanges` response is genuinely large,
    /// and small enough to seed row by row through the public store API in a
    /// unit test. The measured headline is a 20 000-row folder at 1 749 125
    /// bytes; the property under test is that the new response does not grow
    /// with the folder **at all**, and that is asserted directly by comparing
    /// two folder sizes rather than extrapolated from one.
    const FOLDER: usize = 2_000;

    struct Fixture {
        engine: Engine,
        threads: HashSet<ThreadId>,
        account: String,
        mailbox: String,
        /// Every id in `mailbox`, in the order `Email/query` returns them.
        ordered: Vec<String>,
    }

    impl Fixture {
        async fn with(n: usize) -> Self {
            let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
            let threads = db_threads(&store).await;
            let account = store
                .create_account(
                    &NewAccount {
                        kind: AccountKind::Imap,
                        host: "h",
                        port: 993,
                        tls: "implicit",
                        username: "u",
                        sync_policy_json: "{}",
                    },
                    &Credentials {
                        username: "u".into(),
                        password: "p".into(),
                    },
                )
                .await
                .unwrap();
            let mailbox = store
                .upsert_mailbox(&MailboxUpsert {
                    account_id: &account,
                    name: "INBOX",
                    role: Some("inbox"),
                    uidvalidity: 100,
                    uidnext: 1,
                    highestmodseq: 0,
                    total: 0,
                    unread: 0,
                    parent_id: None,
                })
                .await
                .unwrap();
            for uid in 1..=n as u32 {
                // Distinct ascending dates, so "newest first" is a real order
                // rather than a tie broken by the id.
                let date = format!("2026-07-01T00:00:{uid:08}Z");
                let message_id = format!("<m{uid}@x>");
                store
                    .upsert_message(&MessageUpsert {
                        account_id: &account,
                        mailbox_id: &mailbox,
                        uid,
                        uidvalidity: 100,
                        message_id: Some(&message_id),
                        thread_id: None,
                        internaldate: Some(&date),
                        size: 1024,
                        flags_json: "[]",
                        envelope: None,
                        blob_ref: None,
                    })
                    .await
                    .unwrap();
            }
            let ordered = store.list_message_ids(&mailbox, i64::MAX, 0).await.unwrap();
            assert_eq!(ordered.len(), n);
            Self {
                engine: Engine::new(store),
                threads,
                account,
                mailbox,
                ordered,
            }
        }

        fn store(&self) -> &Store {
            self.engine.store()
        }

        /// The filter every fast-path assertion uses.
        fn in_mailbox(&self) -> Value {
            json!({ "inMailbox": self.mailbox })
        }

        async fn query(&self, args: Value) -> Value {
            self.engine.email_query(&self.account, &args).await
        }

        async fn counted_query(&self, args: Value) -> (Value, Stmts) {
            counted(&self.threads, self.engine.email_query(&self.account, &args)).await
        }

        async fn query_changes(&self, args: Value) -> Value {
            self.engine.email_query_changes(&self.account, &args).await
        }

        async fn changes(&self, args: Value) -> Value {
            self.engine
                .type_changes(&self.account, ChangeType::Email, &args)
                .await
        }

        /// Mark `n` messages read the way `Email/set` does, so the change log
        /// holds real rows at real states.
        async fn touch(&self, n: usize) {
            for id in self.ordered.iter().take(n) {
                self.store().set_flags(id, "[\"Seen\"]").await.unwrap();
                self.engine
                    .record_change(&self.account, ChangeType::Email, id, ChangeOp::Updated)
                    .await
                    .unwrap();
            }
        }

        async fn email_state(&self) -> String {
            self.engine
                .type_state(&self.account, ChangeType::Email)
                .await
                .unwrap()
        }
    }

    fn ids_of(resp: &Value) -> Vec<String> {
        resp["ids"]
            .as_array()
            .expect("ids")
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect()
    }

    fn bytes(v: &Value) -> usize {
        serde_json::to_vec(v).expect("serialize").len()
    }

    /// `Email/queryChanges` **as `master` answered it**, reproduced here so the
    /// "before" number is measured in the same run as the "after" rather than
    /// quoted from a document.
    ///
    /// The shape, verbatim: the whole current query, every id, at every index,
    /// in `added`, plus `total` and the caller's own `sinceQueryState` echoed
    /// back as `oldQueryState`.
    fn master_shape_query_changes(
        account_id: &str,
        since: &str,
        new_state: &str,
        ids: &[String],
    ) -> Value {
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
            "removed": Vec::<String>::new(),
            "added": added
        })
    }

    // -- Email/query: paging reaches SQL --------------------------------------

    /// The pushdown, on the instrument that can see it.
    ///
    /// On `master` the fast path called `list_message_ids(mb, i64::MAX, 0)` and
    /// sliced in Rust, so a 50-row page of a 2 000-message folder read **2 000**
    /// rows. The statement count was 1 then and is 1 now; only `rows_returned`
    /// separates the two.
    #[tokio::test]
    async fn a_page_reads_the_page_not_the_folder() {
        let f = Fixture::with(FOLDER).await;

        let (resp, stmts) = f
            .counted_query(json!({ "filter": f.in_mailbox(), "position": 0, "limit": 50 }))
            .await;
        assert_eq!(ids_of(&resp), f.ordered[..50]);

        let paging = stmts.matching("FROM messages");
        assert_eq!(
            paging.len(),
            1,
            "one statement answers the page; got {stmts:#?}"
        );
        assert_eq!(
            paging[0].rows, 50,
            "the LIMIT must reach the database: {} rows came back for a 50-row \
             page of a {FOLDER}-message folder. On master this reads {FOLDER}, \
             because the limit was applied by .take() after a full read",
            paging[0].rows
        );

        // A deep page costs the same read, which is the property a client pages
        // 20 000 rows on.
        let (deep, stmts) = f
            .counted_query(json!({
                "filter": f.in_mailbox(), "position": FOLDER - 50, "limit": 50
            }))
            .await;
        assert_eq!(ids_of(&deep), f.ordered[FOLDER - 50..]);
        assert_eq!(deep["position"], json!(FOLDER - 50));
        assert_eq!(stmts.matching("FROM messages")[0].rows, 50);
    }

    /// `calculateTotal` is opt-in, and opting out costs nothing.
    ///
    /// `master` returned `total` unconditionally, computed as the length of the
    /// fully materialised query — so "no `COUNT(*)` was issued" was true there
    /// only because the whole folder had already been read.
    #[tokio::test]
    async fn total_is_computed_only_when_asked_for() {
        let f = Fixture::with(FOLDER).await;

        let (resp, stmts) = f
            .counted_query(json!({ "filter": f.in_mailbox(), "limit": 50 }))
            .await;
        assert!(
            resp.get("total").is_none(),
            "no `calculateTotal`, no total: {resp}"
        );
        assert_eq!(
            stmts.matching("COUNT(*)").len(),
            0,
            "a client that did not ask for a total must not pay for one; got {stmts:#?}"
        );

        let (resp, stmts) = f
            .counted_query(json!({
                "filter": f.in_mailbox(), "limit": 50, "calculateTotal": true
            }))
            .await;
        assert_eq!(resp["total"], json!(FOLDER));
        assert_eq!(
            stmts.matching("COUNT(*)").len(),
            1,
            "exactly one count, once per query rather than once per page; got {stmts:#?}"
        );
        // And the total describes the folder, not the page.
        assert_eq!(ids_of(&resp).len(), 50);
    }

    /// V7: the pure `inMailbox` fast path issues **no `saved_searches`
    /// statement**.
    ///
    /// Asserted on the absence of that specific statement rather than on a
    /// smaller total: the drop is one statement in fifteen and rounds away in a
    /// total, so a total-based assertion would neither fail on `master` nor
    /// catch the regression.
    #[tokio::test]
    async fn the_fast_path_does_not_look_up_a_saved_search() {
        let f = Fixture::with(200).await;

        let (resp, stmts) = f
            .counted_query(json!({ "filter": f.in_mailbox(), "limit": 10 }))
            .await;
        assert_eq!(ids_of(&resp).len(), 10);
        assert_eq!(
            stmts.matching("saved_searches").len(),
            0,
            "master runs get_saved_search on every query, including this one, \
             where it can only miss; got {stmts:#?}"
        );

        // The behaviour it paid for is intact: a saved-search folder id still
        // expands to its stored filter, and there it costs the lookup, because
        // there the lookup is the only thing that can resolve the id.
        f.store()
            .upsert_saved_search(&SavedSearchRow {
                id: "ss-folder".into(),
                user: f.account.clone(),
                name: "Seen".into(),
                query_json: json!({ "hasKeyword": "$seen" }).to_string(),
                as_folder: true,
            })
            .await
            .unwrap();
        let (_, stmts) = f
            .counted_query(json!({ "filter": { "inMailbox": "ss-folder" }, "limit": 10 }))
            .await;
        assert_eq!(
            stmts.matching("saved_searches").len(),
            1,
            "an id with no messages is exactly the case the lookup exists for; \
             got {stmts:#?}"
        );
    }

    /// `anchor`/`anchorOffset`, and the refusal that makes them safe.
    #[tokio::test]
    async fn anchor_starts_the_window_at_an_id() {
        let f = Fixture::with(FOLDER).await;
        let anchor = f.ordered[1_234].clone();

        let (resp, stmts) = f
            .counted_query(json!({
                "filter": f.in_mailbox(), "anchor": anchor, "limit": 5
            }))
            .await;
        assert_eq!(ids_of(&resp), f.ordered[1_234..1_239]);
        assert_eq!(resp["position"], json!(1_234));

        // The property, stated as a constant rather than a bound: anchoring at
        // row 1 234 reads exactly what anchoring at row 3 reads. Three
        // statements — the anchor's own sort key, the `COUNT(*)` of the rows
        // before it, and the page — for 7 rows total, at any depth. A bound
        // ("fewer than N rows") would be satisfied by an implementation that
        // walks the offset in a small folder; depth-independence would not.
        let deep: u64 = stmts.matching("FROM messages").iter().map(|s| s.rows).sum();
        let (_, shallow_stmts) = f
            .counted_query(json!({
                "filter": f.in_mailbox(), "anchor": f.ordered[3], "limit": 5
            }))
            .await;
        let shallow: u64 = shallow_stmts
            .matching("FROM messages")
            .iter()
            .map(|s| s.rows)
            .sum();
        assert_eq!(
            deep, shallow,
            "an anchor 1 234 rows deep read {deep} rows against {shallow} for an \
             anchor 3 rows deep; the whole point of anchor paging is that it does \
             not walk the offset. {stmts:#?}"
        );
        assert_eq!(
            deep, 7,
            "1 anchor row + 1 count + the 5-row page: {stmts:#?}"
        );

        // A negative offset opens the window before the anchor.
        let back = f
            .query(json!({
                "filter": f.in_mailbox(), "anchor": anchor, "anchorOffset": -2, "limit": 3
            }))
            .await;
        assert_eq!(ids_of(&back), f.ordered[1_232..1_235]);
        assert_eq!(back["position"], json!(1_232));

        // An offset that would run off the front clamps at 0 rather than wrapping.
        let clamped = f
            .query(json!({
                "filter": f.in_mailbox(), "anchor": f.ordered[1], "anchorOffset": -50, "limit": 2
            }))
            .await;
        assert_eq!(clamped["position"], json!(0));
        assert_eq!(ids_of(&clamped), f.ordered[..2]);

        // RFC 8620 5.5: an anchor that is not in the result is an error, not
        // page 1. Returning page 1 is the failure that silently resets a
        // client's scroll position.
        let missing = f
            .query(json!({
                "filter": f.in_mailbox(), "anchor": "not-a-message", "limit": 5
            }))
            .await;
        assert_eq!(missing["type"], json!("anchorNotFound"), "{missing}");
        assert!(missing.get("ids").is_none());
    }

    /// V9: a truncated search publishes **no** total rather than a truncated one.
    ///
    /// `mw_search`'s cap is 100 000, which no unit test is going to index, so the
    /// mechanism is driven at the seam it really runs through — a search whose
    /// `SearchHits::truncated` is set — using an explicit cap. What is asserted
    /// is the engine's *rule*: `truncated` means the `total` field is absent and
    /// flagged, never a number. On master the field was `ids.len()`
    /// unconditionally, so a 200 000-document index answered `total: 100000`.
    #[tokio::test]
    async fn a_truncated_search_publishes_no_total() {
        let f = Fixture::with(20).await;
        let sq = mw_search::SearchQuery {
            raw: String::new(),
            expr: mw_search::Expr::All,
            sort: mw_search::Sort::received_desc(),
        };
        for (i, id) in f.ordered.iter().enumerate() {
            f.engine
                .search()
                .upsert(&mw_search::IndexDoc {
                    stable_id: id.clone(),
                    account_id: f.account.clone(),
                    mailbox_id: f.mailbox.clone(),
                    subject: format!("needle {i}"),
                    body: "needle".into(),
                    ..mw_search::IndexDoc::default()
                })
                .unwrap();
        }

        // The seam: the cap is binding, and the index says so.
        let capped = f.engine.search().search_hits(&sq, 5).unwrap();
        assert!(capped.truncated, "5 of 20 must report truncation");
        assert_eq!(capped.ids.len(), 5);
        // ...and is not binding when it is not.
        assert!(!f.engine.search().search_hits(&sq, 0).unwrap().truncated);

        // The rule, through the response builder both query paths publish
        // through: a truncated total is never emitted as a number.
        let mut out = json!({});
        QueryPage {
            total: None,
            total_truncated: true,
            ..QueryPage::default()
        }
        .publish_total(&mut out);
        assert!(
            out.get("total").is_none(),
            "master answers `total: 100000` for a 200 000-document index — a wrong \
             number that looks exactly like a right one: {out}"
        );
        assert_eq!(out["mailwomanCannotCalculateTotal"], json!(true));

        // An untruncated search still gets a real total, so the flag is not just
        // "totals are gone".
        let full = f
            .query(json!({
                "filter": { "text": "needle" }, "calculateTotal": true
            }))
            .await;
        assert_eq!(full["total"], json!(20));
        assert!(full.get("mailwomanCannotCalculateTotal").is_none());
    }

    // -- Email/queryChanges ---------------------------------------------------

    /// The headline: what a delta costs on the wire.
    ///
    /// Measured against `master`'s response shape, reproduced in this same run
    /// by `master_shape_query_changes`, so the ratio is derived rather than
    /// quoted.
    #[tokio::test]
    async fn query_changes_sends_the_delta_not_the_query() {
        let f = Fixture::with(FOLDER).await;
        let before_state = f.email_state().await;
        f.touch(3).await;

        let resp = f
            .query_changes(json!({
                "filter": f.in_mailbox(), "sinceQueryState": before_state, "maxChanges": 50
            }))
            .await;

        let master = master_shape_query_changes(
            &f.account,
            &before_state,
            &f.email_state().await,
            &f.ordered,
        );
        let (new_bytes, old_bytes) = (bytes(&resp), bytes(&master));
        eprintln!(
            "[t22-e2] Email/queryChanges over a {FOLDER}-message folder, 3 changed: \
             master {old_bytes} bytes -> {new_bytes} bytes ({}x)",
            old_bytes / new_bytes.max(1)
        );
        assert!(
            new_bytes < 16 * 1024,
            "a 3-change delta must fit in a page's worth of bytes; got {new_bytes} \
             against master's {old_bytes}"
        );
        assert!(
            old_bytes / new_bytes.max(1) >= 100,
            "master {old_bytes} vs {new_bytes}: if this ratio is small the corpus \
             is too small for the byte assertion to mean anything"
        );

        // The delta is the three ids, at their real indices, and nothing else.
        let added = resp["added"].as_array().unwrap().clone();
        assert_eq!(added.len(), 3, "{resp}");
        for entry in &added {
            let id = entry["id"].as_str().unwrap();
            let index = entry["index"].as_u64().unwrap() as usize;
            assert_eq!(&f.ordered[index], id, "index must name the id: {entry}");
        }
        // An id whose position may have moved leaves the client's list first.
        let removed: Vec<&str> = resp["removed"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        for entry in &added {
            assert!(removed.contains(&entry["id"].as_str().unwrap()), "{resp}");
        }
        assert_eq!(resp["oldQueryState"], json!(before_state));
        assert_ne!(resp["newQueryState"], json!(before_state));
    }

    /// The same measurement at the folder size the plan quotes: **20 000**.
    ///
    /// `#[ignore]`d because seeding 20 000 rows one at a time through the public
    /// store API takes about a minute and the property is already proved by the
    /// two tests either side of this one. It exists so the headline number is
    /// reproducible by anyone (`cargo test -p mw-engine --lib -- --ignored
    /// query_changes_at_the_measured_folder_size --nocapture`) rather than
    /// carried in a commit message. Recorded on this host:
    ///
    /// ```text
    /// master 1 749 047 bytes -> 595 bytes (2939x)
    /// ```
    ///
    /// The plan's independently-measured figure for the same folder is
    /// **1 749 125 bytes**; the 78-byte gap is the width of this fixture's
    /// account id and state token against the verifier's. That the two agree to
    /// four significant figures is the calibration — a "before" reproduced from
    /// the old source shape should land on the number the old source produced.
    #[tokio::test]
    #[ignore = "seeds 20 000 rows; the scale headline, reproducible on demand"]
    async fn query_changes_at_the_measured_folder_size() {
        let f = Fixture::with(20_000).await;
        let since = f.email_state().await;
        f.touch(3).await;
        let resp = f
            .query_changes(json!({
                "filter": f.in_mailbox(), "sinceQueryState": since, "maxChanges": 50
            }))
            .await;
        let master =
            master_shape_query_changes(&f.account, &since, &f.email_state().await, &f.ordered);
        let (new_bytes, old_bytes) = (bytes(&resp), bytes(&master));
        eprintln!(
            "[t22-e2] Email/queryChanges over a 20 000-message folder, 3 changed: \
             master {old_bytes} bytes -> {new_bytes} bytes ({}x)",
            old_bytes / new_bytes.max(1)
        );
        assert!(
            old_bytes > 1_500_000,
            "master's response: {old_bytes} bytes"
        );
        assert!(new_bytes < 16 * 1024, "the delta: {new_bytes} bytes");
    }

    /// The response does not grow with the folder — the load-independent form of
    /// the byte assertion above, and the one a corpus size cannot flatter.
    #[tokio::test]
    async fn the_delta_size_does_not_depend_on_the_folder_size() {
        let mut sizes = Vec::new();
        for n in [500usize, 2_000] {
            let f = Fixture::with(n).await;
            let since = f.email_state().await;
            f.touch(3).await;
            let resp = f
                .query_changes(json!({
                    "filter": f.in_mailbox(), "sinceQueryState": since, "maxChanges": 50
                }))
                .await;
            assert_eq!(resp["added"].as_array().unwrap().len(), 3);
            sizes.push(bytes(&resp));
        }
        assert_eq!(
            sizes[0], sizes[1],
            "the same three changes in a 500-message folder and a 2 000-message \
             folder must serialise to the same bytes; master's grow 4x with the \
             folder, because there the folder *is* the response"
        );
    }

    /// The refusal path, exercised where it fires **and** where it does not.
    ///
    /// A `maxChanges` implementation that never refuses has not been tested; one
    /// that always refuses is not an implementation. `Foo/queryChanges` has no
    /// `hasMoreChanges`, so RFC 8620 5.6 requires refusal rather than a partial
    /// answer — a truncated delta is applied by the client, which then believes
    /// it is up to date.
    #[tokio::test]
    async fn query_changes_refuses_rather_than_truncating() {
        let f = Fixture::with(200).await;
        let since = f.email_state().await;
        f.touch(60).await;

        // Under the cap: a real delta.
        let ok = f
            .query_changes(json!({
                "filter": f.in_mailbox(), "sinceQueryState": since, "maxChanges": 100
            }))
            .await;
        assert_eq!(ok["added"].as_array().unwrap().len(), 60, "{ok}");
        assert!(ok.get("type").is_none(), "not a refusal: {ok}");

        // Over the cap: a refusal, and specifically this one.
        let refused = f
            .query_changes(json!({
                "filter": f.in_mailbox(), "sinceQueryState": since, "maxChanges": 50
            }))
            .await;
        assert_eq!(
            refused["type"],
            json!("cannotCalculateChanges"),
            "60 changes with maxChanges: 50 must refuse, not answer 50 of them: {refused}"
        );
        assert!(
            refused.get("added").is_none() && refused.get("removed").is_none(),
            "a refusal carries no delta for a client to half-apply: {refused}"
        );

        // Exactly at the cap is not over it — the boundary a lookahead exists to
        // get right.
        let exact = f
            .query_changes(json!({
                "filter": f.in_mailbox(), "sinceQueryState": since, "maxChanges": 60
            }))
            .await;
        assert_eq!(exact["added"].as_array().unwrap().len(), 60, "{exact}");

        // A state this account never reached is also a refusal, not an empty
        // delta: "nothing changed" makes a client with a stale state stop asking.
        let bogus = f
            .query_changes(json!({
                "filter": f.in_mailbox(), "sinceQueryState": "99999", "maxChanges": 50
            }))
            .await;
        assert_eq!(bogus["type"], json!("cannotCalculateChanges"), "{bogus}");
        let garbage = f
            .query_changes(json!({
                "filter": f.in_mailbox(), "sinceQueryState": "not-a-state"
            }))
            .await;
        assert_eq!(
            garbage["type"],
            json!("cannotCalculateChanges"),
            "{garbage}"
        );
    }

    /// `upToId` bounds the read, not just the answer.
    #[tokio::test]
    async fn up_to_id_bounds_the_query_read() {
        let f = Fixture::with(FOLDER).await;
        let since = f.email_state().await;
        f.touch(3).await;

        let (resp, stmts) = counted(
            &f.threads,
            f.engine.email_query_changes(
                &f.account,
                &json!({
                    "filter": f.in_mailbox(),
                    "sinceQueryState": since,
                    "maxChanges": 50,
                    "upToId": f.ordered[49],
                }),
            ),
        )
        .await;
        assert_eq!(resp["added"].as_array().unwrap().len(), 3, "{resp}");

        let listing: u64 = stmts
            .matching("ORDER BY internaldate")
            .iter()
            .map(|s| s.rows)
            .sum();
        assert_eq!(
            listing, 50,
            "a client holding 50 rows must cause a 50-row read, not a \
             {FOLDER}-row one: {stmts:#?}"
        );
    }

    // -- Email/changes (V6) ---------------------------------------------------

    /// V6: `maxChanges` is honoured here too, and truncation is **reported**.
    ///
    /// Fixing only `queryChanges` would leave the identical bug one method over.
    /// On `master` this returned all 60 with `hasMoreChanges: false` — a client
    /// asking for 20 got 60 and was told that was all of them.
    #[tokio::test]
    async fn email_changes_honours_max_changes_and_says_when_it_truncated() {
        let f = Fixture::with(100).await;
        let since = f.email_state().await;
        f.touch(60).await;

        let capped = f
            .changes(json!({ "sinceState": since, "maxChanges": 20 }))
            .await;
        let updated = capped["updated"].as_array().unwrap().clone();
        // 19, not 20: the page ends at the last COMPLETE state, and the store
        // cannot say whether the row after the cap shares the 20th state, so the
        // safe answer drops it. Under-delivering by one state is legal (a server
        // may always return fewer) and costs one re-read; over-delivering by one
        // state would strand the rest of a batch forever. See
        // `a_capped_page_never_stops_inside_one_state`.
        assert_eq!(updated.len(), 19, "the cap must bind: {capped}");
        assert_eq!(
            capped["hasMoreChanges"],
            json!(true),
            "master answers false here, with all 60 rows attached: {capped}"
        );

        // The resume point is the last row RETURNED, not the current state.
        // Reporting the current state alongside a partial list tells the client
        // it is up to date about changes it was never sent.
        let resumed = f
            .changes(json!({
                "sinceState": capped["newState"].as_str().unwrap(), "maxChanges": 100
            }))
            .await;
        assert_eq!(
            resumed["updated"].as_array().unwrap().len(),
            41,
            "{resumed}"
        );
        assert_eq!(resumed["hasMoreChanges"], json!(false));

        // The two pages together are every change, exactly once.
        let mut all: Vec<String> = updated
            .iter()
            .chain(resumed["updated"].as_array().unwrap())
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        all.sort();
        all.dedup();
        assert_eq!(all.len(), 60, "no change may be skipped between the pages");

        // Uncapped (under the default) is unchanged behaviour.
        let whole = f.changes(json!({ "sinceState": since })).await;
        assert_eq!(whole["updated"].as_array().unwrap().len(), 60);
        assert_eq!(whole["hasMoreChanges"], json!(false));

        // An unknown state refuses rather than reporting an empty diff.
        let bogus = f.changes(json!({ "sinceState": "99999" })).await;
        assert_eq!(bogus["type"], json!("cannotCalculateChanges"), "{bogus}");
    }

    /// A filter with neither an `inMailbox` nor a search condition stays
    /// **empty** — it does not become "every message in the account".
    ///
    /// `master` returned `Vec::new()` for this, and two `mw-server` suites say
    /// so in prose (`t13_jwz.rs`, `t13_geoip.rs`: "an unfiltered `Email/query`
    /// returns nothing") without asserting it. Restructuring the router around
    /// the fast path made it fall through to the search index instead, which
    /// turned a request matching nothing into one matching the whole account —
    /// caught by reading the prose, not by a red test, which is why it gets one
    /// now.
    #[tokio::test]
    async fn an_unfiltered_query_stays_empty() {
        let f = Fixture::with(30).await;
        for (i, id) in f.ordered.iter().enumerate() {
            f.engine
                .search()
                .upsert(&mw_search::IndexDoc {
                    stable_id: id.clone(),
                    account_id: f.account.clone(),
                    mailbox_id: f.mailbox.clone(),
                    subject: format!("indexed {i}"),
                    ..mw_search::IndexDoc::default()
                })
                .unwrap();
        }

        for args in [
            json!({}),
            json!({ "filter": Value::Null }),
            json!({ "filter": {} }),
            json!({ "limit": 10, "calculateTotal": true }),
        ] {
            let resp = f.query(args.clone()).await;
            assert!(
                ids_of(&resp).is_empty(),
                "an unfiltered query must match nothing, not everything                  indexed: {args} -> {resp}"
            );
        }
        // And when a total is asked for it is a real zero, not an absent field.
        let counted = f
            .query(json!({ "limit": 10, "calculateTotal": true }))
            .await;
        assert_eq!(counted["total"], json!(0), "{counted}");

        // The contrast that makes the above meaningful: the same corpus DOES
        // answer a query that names a mailbox, and one that names a text term.
        assert_eq!(
            f.query(json!({ "filter": f.in_mailbox() })).await["ids"]
                .as_array()
                .unwrap()
                .len(),
            30
        );
        assert_eq!(
            f.query(json!({ "filter": { "text": "indexed" } })).await["ids"]
                .as_array()
                .unwrap()
                .len(),
            30
        );
    }

    /// The cap must land on a **state boundary**, because change rows do not
    /// carry distinct states.
    ///
    /// `Store::record_changes` writes **N rows at ONE state** — that is what a
    /// batched `Email/set` over a large selection produces (26.20 `t22-e1`
    /// `6f35e49`, consumed by `t22-e3s`). So a cap that lands mid-batch and then
    /// reports "you are up to date through this row" makes the client resume at
    /// `state > it` and **silently lose the rest of the batch**.
    ///
    /// This test exists because the first version of this lane had that bug: the
    /// other `maxChanges` test above passes with it, because `touch()` records
    /// one change at a time and every row there has its own state. A cap
    /// assertion built only on singly-recorded changes cannot see this at all.
    #[tokio::test]
    async fn a_capped_page_never_stops_inside_one_state() {
        let f = Fixture::with(60).await;
        let since = f.email_state().await;

        // Three batches at three states: 10, then 30, then 10.
        for batch in [&f.ordered[0..10], &f.ordered[10..40], &f.ordered[40..50]] {
            f.store()
                .record_changes(&f.account, ChangeType::Email.as_str(), batch, "updated")
                .await
                .unwrap();
        }

        // A cap of 20 can only be honoured by stopping after the FIRST batch —
        // 10 rows, one whole state. Stopping at row 20 would sit inside the
        // 30-row batch and strand its other 20 ids forever.
        let page = f
            .changes(json!({ "sinceState": since, "maxChanges": 20 }))
            .await;
        assert_eq!(
            page["updated"].as_array().unwrap().len(),
            10,
            "the page must end at the state boundary, not at the cap: {page}"
        );
        assert_eq!(page["hasMoreChanges"], json!(true));

        // Resuming from the reported state yields the other two batches whole.
        let rest = f
            .changes(json!({
                "sinceState": page["newState"].as_str().unwrap(), "maxChanges": 100
            }))
            .await;
        assert_eq!(rest["updated"].as_array().unwrap().len(), 40, "{rest}");
        let mut all: Vec<&str> = page["updated"]
            .as_array()
            .unwrap()
            .iter()
            .chain(rest["updated"].as_array().unwrap())
            .map(|v| v.as_str().unwrap())
            .collect();
        all.sort();
        all.dedup();
        assert_eq!(
            all.len(),
            50,
            "every id in every batch must survive the paging, exactly once"
        );

        // A cap smaller than a single batch has no lossless answer at all, and
        // says so rather than returning a page that drops 20 of 30 ids.
        let impossible = f
            .changes(json!({
                "sinceState": page["newState"].as_str().unwrap(), "maxChanges": 5
            }))
            .await;
        assert_eq!(
            impossible["type"],
            json!("cannotCalculateChanges"),
            "a 30-row state under a 5-row cap has no page that is both within \
             the cap and lossless: {impossible}"
        );

        // `Email/queryChanges` refuses on the same condition, for the same
        // reason — it has no `hasMoreChanges` to fall back on either.
        let qc = f
            .query_changes(json!({
                "filter": f.in_mailbox(),
                "sinceQueryState": page["newState"].as_str().unwrap(),
                "maxChanges": 5
            }))
            .await;
        assert_eq!(qc["type"], json!("cannotCalculateChanges"), "{qc}");
    }
}

#[cfg(test)]
mod email_get_batch_tests {
    //! `Email/get` — one store batch per page (26.20 t22-e3g, plan row S5).
    //!
    //! # The instrument, and the calibration that has to come before it
    //!
    //! The acceptance is `stmts(50 ids) == stmts(5 ids)` **and** `<= 6`, against
    //! **163 today**. The equality is the load-bearing half and the bound is the
    //! sanity check, not the reverse: a loop hidden behind a batch signature
    //! satisfies `<= 6` only by accident on a small page, and fails the equality
    //! on the first page that is not small. `t22-e3s` established the discipline
    //! — it broke `upsert_batch` on purpose to make it commit per document and
    //! confirmed its counter read 500 before believing any number from it — and
    //! [`the_counter_reads_the_per_id_loop_it_is_meant_to_catch`] is this lane's
    //! version of that: it drives the **unbatched** path through the same counter
    //! and asserts the count scales with the id count. If that test ever stops
    //! failing on a per-id implementation, every other number here is worthless.
    //!
    //! The counter itself is the shared recorder in
    //! [`crate::state::session_state_tests`], which counts sqlx's own
    //! `sqlx::query` events — one per statement actually executed, at the driver
    //! layer. It is not a count of store-method calls, so a "batch" method that
    //! loops internally is caught by it rather than hidden by it.

    use std::collections::HashSet;
    use std::thread::ThreadId;

    use mw_store::{
        AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store,
    };
    use serde_json::json;

    use super::*;
    use crate::state::session_state_tests::{Stmts, counted, db_threads};

    /// A fixture whose messages carry **envelopes**, which is what a synced
    /// mailbox looks like.
    ///
    /// This matters to the measurement, not just to realism: `build_email` reads
    /// a message's body **only when its envelope is absent**, so a fixture
    /// without envelopes measures a fallback path instead of the read path, and
    /// one with them measures what `Email/get` actually costs a user opening a
    /// page of mail.
    struct Fixture {
        engine: Engine,
        threads: HashSet<ThreadId>,
        account: String,
        ids: Vec<String>,
    }

    impl Fixture {
        async fn with(n: usize) -> Self {
            let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
            let threads = db_threads(&store).await;
            let account = store
                .create_account(
                    &NewAccount {
                        kind: AccountKind::Imap,
                        host: "h",
                        port: 993,
                        tls: "implicit",
                        username: "u",
                        sync_policy_json: "{}",
                    },
                    &Credentials {
                        username: "u".into(),
                        password: "p".into(),
                    },
                )
                .await
                .unwrap();
            let mailbox = store
                .upsert_mailbox(&MailboxUpsert {
                    account_id: &account,
                    name: "INBOX",
                    role: Some("inbox"),
                    uidvalidity: 100,
                    uidnext: 1,
                    highestmodseq: 0,
                    total: 0,
                    unread: 0,
                    parent_id: None,
                })
                .await
                .unwrap();

            let mut ids = Vec::new();
            for uid in 1..=n as u32 {
                let envelope = serde_json::to_vec(&json!({
                    "subject": format!("Subject {uid}"),
                    "from": [{ "name": "Alice", "email": "alice@example.org" }],
                    "to": [{ "name": null, "email": "me@example.org" }],
                    "receivedAt": format!("2026-07-01T00:00:{uid:08}Z"),
                    "size": 1024,
                    "hasAttachment": false,
                }))
                .unwrap();
                let date = format!("2026-07-01T00:00:{uid:08}Z");
                let message_id = format!("<m{uid}@x>");
                ids.push(
                    store
                        .upsert_message(&MessageUpsert {
                            account_id: &account,
                            mailbox_id: &mailbox,
                            uid,
                            uidvalidity: 100,
                            message_id: Some(&message_id),
                            thread_id: None,
                            internaldate: Some(&date),
                            size: 1024,
                            flags_json: r#"["Seen"]"#,
                            envelope: Some(&envelope),
                            blob_ref: None,
                        })
                        .await
                        .unwrap(),
                );
            }
            Self {
                engine: Engine::new(store),
                threads,
                account,
                ids,
            }
        }

        fn store(&self) -> &Store {
            self.engine.store()
        }

        /// `Email/get` over the first `n` seeded ids, with the statements it cost.
        async fn get(&self, n: usize) -> (Value, Stmts) {
            let ids: Vec<&str> = self.ids[..n].iter().map(String::as_str).collect();
            counted(
                &self.threads,
                self.engine.email_get(&self.account, &json!({ "ids": ids })),
            )
            .await
        }
    }

    /// **`master`'s per-id composition, retained here as the oracle** — the same
    /// move `t22-e0` made with `session_state_unfolded`.
    ///
    /// This is what `Engine::build_email` was before this lane: `get_message`,
    /// then the cache-aside envelope with a body re-parse behind it, then
    /// `get_message_meta`, then the shared field patch. It has no production
    /// caller now — `email_get` goes through `build_emails` — so leaving it in
    /// `impl Engine` would be dead code that reads like a live second path.
    ///
    /// It lives here because the equivalence assertion needs something to be
    /// equivalent *to*, and a hand-written expected document would encode what I
    /// believe `Email/get` produces rather than what it produced yesterday. Note
    /// it deliberately still calls `patch_engine_fields`: the ordering, holes and
    /// per-field content are what is under test, not the patcher.
    async fn build_email_per_id(engine: &Engine, stable_id: &str) -> Option<Value> {
        let msg = match engine.store().get_message(stable_id).await {
            Ok(m) => m,
            Err(mw_store::StoreError::NotFound) => return None,
            Err(e) => panic!("store: {e}"),
        };
        let email: Value = match engine
            .cached_envelope(&msg.account_id, stable_id)
            .await
            .unwrap()
        {
            Some(bytes) => serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({})),
            None => match &msg.blob_ref {
                Some(blob) => match engine
                    .cached_body(&msg.account_id, stable_id, blob)
                    .await
                    .unwrap()
                {
                    Some(raw) => mw_mime::parse(&raw)
                        .ok()
                        .and_then(|p| serde_json::to_value(p.email).ok())
                        .unwrap_or_else(|| json!({})),
                    None => json!({}),
                },
                None => json!({}),
            },
        };
        let meta = engine
            .store()
            .get_message_meta(stable_id)
            .await
            .unwrap()
            .unwrap_or_default();
        Some(patch_engine_fields(email, stable_id, &msg, &meta))
    }

    fn list_of(resp: &Value) -> &Vec<Value> {
        resp["list"].as_array().expect("list")
    }

    /// **Calibration, and nothing here is believed until this passes.**
    ///
    /// It drives the *pre-batch* composition — `build_email` once per id, which
    /// is exactly what `master`'s `email_get` loop did — through the same counter
    /// the acceptance test uses, and asserts the count **scales with the id
    /// count**. Two properties at once:
    ///
    /// * the counter sees store statements at all (a recorder wired to the wrong
    ///   thread reads zero and makes every batch look perfect);
    /// * it distinguishes 5 ids from 50, which is the entire question. A counter
    ///   that reported a constant for the per-id path would make the acceptance
    ///   test pass against an unchanged implementation.
    ///
    /// The numbers are recorded rather than bounded loosely: this is `master`'s
    /// cost, measured, and it is what the batched path is compared against.
    #[tokio::test]
    async fn the_counter_reads_the_per_id_loop_it_is_meant_to_catch() {
        let f = Fixture::with(50).await;

        async fn unbatched(f: &Fixture, n: usize) -> Stmts {
            let ids: Vec<String> = f.ids[..n].to_vec();
            counted(&f.threads, async {
                for id in &ids {
                    // The three per-id reads `build_email` made, verbatim.
                    let _ = f.store().get_message(id).await;
                    let _ = f.store().get_envelope(id).await;
                    let _ = f.store().get_message_meta(id).await;
                }
            })
            .await
            .1
        }

        let five = unbatched(&f, 5).await;
        let fifty = unbatched(&f, 50).await;
        assert_eq!(five.len(), 15, "3 statements per id, 5 ids: {five:#?}");
        assert_eq!(fifty.len(), 150, "3 statements per id, 50 ids");
        assert!(
            fifty.len() > five.len(),
            "the counter cannot tell a 50-id loop from a 5-id one, so no number \
             it produces below means anything"
        );
    }

    /// The acceptance: **constant in the page size, and small**.
    #[tokio::test]
    async fn email_get_costs_the_same_for_fifty_ids_as_for_five() {
        let f = Fixture::with(50).await;

        let (five, stmts_5) = f.get(5).await;
        let (fifty, stmts_50) = f.get(50).await;
        assert_eq!(list_of(&five).len(), 5);
        assert_eq!(list_of(&fifty).len(), 50);

        assert_eq!(
            stmts_50.len(),
            stmts_5.len(),
            "the SQL cost of Email/get must not depend on the page size: \
             {} statements for 5 ids vs {} for 50.\n5: {stmts_5:#?}\n50: {stmts_50:#?}",
            stmts_5.len(),
            stmts_50.len()
        );
        assert!(
            stmts_50.len() <= 6,
            "master issues 151 here (163 for the whole request); the ceiling is 6,              got {}: {stmts_50:#?}",
            stmts_50.len()
        );
        // Pinned exactly, so a future change that adds a read has to say so:
        // the message rows, the envelopes, the engine-local metadata, and the
        // state token.
        assert_eq!(
            stmts_50.len(),
            4,
            "the page costs four statements: {stmts_50:#?}"
        );
        eprintln!(
            "[t22-e3g] Email/get: master 16 stmts for 5 ids / 151 for 50 -> {} / {}",
            stmts_5.len(),
            stmts_50.len()
        );

        // Named individually, so a regression says WHICH read came back per-id
        // rather than only that the total moved.
        for (table, what) in [
            ("FROM messages", "the message rows + envelopes"),
            ("FROM message_meta", "the engine-local metadata"),
        ] {
            let n = stmts_50.matching(table).len();
            assert!(
                n <= 2,
                "{what}: {n} statements against `{table}` for one page — a batch \
                 that loops internally reads exactly like this: {stmts_50:#?}"
            );
        }
    }

    /// Batching must not change a single byte of the answer.
    ///
    /// The equality is against the **per-id path**, evaluated in the same run,
    /// rather than against a hand-written expected document — a hand-written one
    /// would encode what I believe `build_email` produces, which is the thing
    /// under test.
    #[tokio::test]
    async fn the_batched_page_is_byte_identical_to_the_per_id_page() {
        let f = Fixture::with(12).await;

        let (batched, _) = f.get(12).await;
        let mut per_id = Vec::new();
        for id in &f.ids {
            per_id.push(build_email_per_id(&f.engine, id).await.unwrap());
        }
        assert_eq!(
            list_of(&batched),
            &per_id,
            "the batched page must be what the per-id path produced, in order"
        );
    }

    /// Order, holes, duplicates and the degenerate page.
    ///
    /// A batch keyed by a `HashMap` returns rows in whatever order the database
    /// felt like, and a `notFound` id silently shifts every entry after it. Both
    /// are invisible to a test that only counts `list.len()`.
    #[tokio::test]
    async fn missing_ids_land_in_not_found_without_shifting_the_rest() {
        let f = Fixture::with(6).await;

        let mixed = json!({
            "ids": [f.ids[3], "ghost-a", f.ids[0], "ghost-b", f.ids[5]]
        });
        let resp = f.engine.email_get(&f.account, &mixed).await;

        let got: Vec<&str> = list_of(&resp)
            .iter()
            .map(|e| e["id"].as_str().unwrap())
            .collect();
        assert_eq!(
            got,
            vec![f.ids[3].as_str(), f.ids[0].as_str(), f.ids[5].as_str()],
            "requested order must survive the batch, holes removed: {resp}"
        );
        let not_found: Vec<&str> = resp["notFound"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(not_found, vec!["ghost-a", "ghost-b"]);

        // A repeated id is answered twice — JMAP's `list` is positional against
        // the request, and de-duplicating it silently drops an entry the client
        // asked for.
        let dup = f
            .engine
            .email_get(&f.account, &json!({ "ids": [f.ids[1], f.ids[1]] }))
            .await;
        assert_eq!(list_of(&dup).len(), 2, "{dup}");
        assert_eq!(list_of(&dup)[0], list_of(&dup)[1]);

        // No ids: an answer, and no statement against `messages` to produce it.
        let (empty, stmts) = counted(
            &f.threads,
            f.engine.email_get(&f.account, &json!({ "ids": [] })),
        )
        .await;
        assert!(list_of(&empty).is_empty());
        assert_eq!(
            stmts.matching("FROM messages").len(),
            0,
            "an empty page must not read the table: {stmts:#?}"
        );
    }

    /// **The two envelope branches must produce the same page.**
    ///
    /// `build_emails` reads envelopes one of two ways — batched from the store
    /// when no cache is attached, per-id through `cached_envelope` when one is —
    /// and a two-branch design is only safe if something asserts the branches
    /// agree. Nothing did: `tests/v6.rs` attaches a cache but drives `Email/get`
    /// with a **single** id, so the batched page under a cache was reachable in
    /// production and unreachable in the suite.
    ///
    /// This is the assertion that makes the branch honest. It also covers a
    /// second-read hit, because a cache that returned something different warm
    /// than cold would be invisible to a one-shot comparison.
    #[tokio::test]
    async fn a_cached_page_and_an_uncached_page_are_the_same_page() {
        let cold = Fixture::with(12).await;
        let uncached = cold
            .engine
            .email_get(
                &cold.account,
                &json!({ "ids": cold.ids.iter().collect::<Vec<_>>() }),
            )
            .await;

        let warm = Fixture::with(12).await;
        warm.engine.attach_v6(
            crate::v6::V6Hooks::new().with_cache(mw_cache::Cache::in_memory(
                mw_cache::ScopeMatrix::spec_defaults(),
            )),
        );
        let args = json!({ "ids": warm.ids.iter().collect::<Vec<_>>() });
        let first = warm.engine.email_get(&warm.account, &args).await;
        // Second read: now served from the warmed header-window cache.
        let second = warm.engine.email_get(&warm.account, &args).await;

        // Ids differ between fixtures (they are minted per store), so compare
        // the part that must not depend on the branch: everything else.
        fn shape(resp: &Value) -> Vec<Value> {
            resp["list"]
                .as_array()
                .expect("list")
                .iter()
                .map(|e| {
                    let mut e = e.clone();
                    let o = e.as_object_mut().unwrap();
                    for k in ["id", "blobId", "threadId", "mailboxIds"] {
                        o.remove(k);
                    }
                    e
                })
                .collect()
        }
        assert_eq!(shape(&first).len(), 12);
        assert_eq!(
            shape(&uncached),
            shape(&first),
            "the batched (no-cache) page and the cache-aside page must agree"
        );
        assert_eq!(
            shape(&first),
            shape(&second),
            "a warm cache must not change the page it serves"
        );

        // And the branch really was taken: the cache is populated afterwards,
        // which is the behaviour batching straight from the store would have
        // silently removed.
        assert!(
            warm.engine.account_posture(&warm.account) == mw_cache::AccountPosture::Standard,
            "fixture precondition: a standard account is the one that caches"
        );
    }

    /// The envelope-absent fallback still works, and still re-parses the body.
    ///
    /// `build_email` reads a body only when the envelope is missing. That path is
    /// rarer than the one above but it is the one that produces a *wrong* answer
    /// rather than a slow one if the batch mis-associates a body with an id, so
    /// it gets its own assertion on content rather than on cost.
    #[tokio::test]
    async fn a_message_without_an_envelope_still_renders_from_its_body() {
        let f = Fixture::with(2).await;
        let raw = b"Message-ID: <nobody@x>\r\n\
                    From: Bob <bob@example.net>\r\n\
                    To: me@example.org\r\n\
                    Subject: Parsed from the body\r\n\
                    Date: Wed, 01 Jul 2026 09:00:00 +0000\r\n\
                    \r\n\
                    body text\r\n";
        let blob = f.store().put_body(&f.account, raw).await.unwrap();
        let date = "2026-07-01T09:00:00Z";
        let mid = "<nobody@x>";
        let id = f
            .store()
            .upsert_message(&MessageUpsert {
                account_id: &f.account,
                mailbox_id: &f.store().list_mailboxes(&f.account).await.unwrap()[0].id,
                uid: 9_000,
                uidvalidity: 100,
                message_id: Some(mid),
                thread_id: None,
                internaldate: Some(date),
                size: raw.len() as u64,
                flags_json: "[]",
                envelope: None,
                blob_ref: Some(&blob),
            })
            .await
            .unwrap();

        // Mixed with two enveloped messages, so a batch that assumed every id
        // takes the same path shows up here.
        let resp = f
            .engine
            .email_get(&f.account, &json!({ "ids": [f.ids[0], id, f.ids[1]] }))
            .await;
        let list = list_of(&resp);
        assert_eq!(list.len(), 3, "{resp}");
        assert_eq!(list[1]["id"].as_str(), Some(id.as_str()));
        assert_eq!(
            list[1]["subject"].as_str(),
            Some("Parsed from the body"),
            "the envelope-less message must be re-parsed from its body, and the \
             result must land on ITS entry rather than a neighbour's: {resp}"
        );
        // The enveloped neighbours are untouched by it.
        assert_eq!(list[0]["subject"].as_str(), Some("Subject 1"));
        assert_eq!(list[2]["subject"].as_str(), Some("Subject 2"));
    }
}
