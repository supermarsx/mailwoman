//! Folder management (26.20 t28-e17): `Mailbox/set` (RFC 8621 §2.5), the
//! `Email/set` destroy path (D-20), the hierarchy `Mailbox/get` reports, and the
//! session's mail account capability.
//!
//! Every write goes to the mail server first and to the cache only after the
//! server accepted it, so the cache never shows a folder the server does not
//! have. The server's own names are the source of truth for the hierarchy:
//! a mailbox's parent is the mailbox whose name is its name up to the last
//! hierarchy delimiter, which is what an IMAP server means by a child.
//!
//! **Destroy is permanent.** `Email/set` `destroy` removes the message from the
//! server (`\Deleted` + `UID EXPUNGE`), not into Trash; moving to Trash is an
//! `Email/set` update to the Trash mailbox. A caller choosing between the two
//! (t32-e46's after-delete setting) sends one or the other. When the server
//! cannot expunge one message on its own, the message is moved to Trash and the
//! id is reported in `notDestroyed`, with `mailwomanMovedTo` naming Trash —
//! it still exists, so it is not reported as destroyed.

use std::collections::{HashMap, HashSet};

use serde_json::{Map, Value, json};

use super::{SERVER_FAIL_TEXT, server_fail, set_error, tolerant};
use crate::account::AccountRuntime;
use crate::backend::{EngineError, MessageRef, RawMailboxRef};
use crate::change::{ChangeOp, ChangeType};
use crate::engine::Engine;
use crate::mapping::display_name;

/// The longest mailbox name `Mailbox/set` accepts, in octets (advertised as
/// `maxSizeMailboxName`).
const MAX_MAILBOX_NAME: usize = 255;

/// The `Email/query` sort properties the engine implements
/// (`crate::query::SortProperty`), advertised as `emailQuerySortOptions`.
const SORT_OPTIONS: [&str; 4] = ["receivedAt", "size", "from", "subject"];

/// The properties a `Mailbox/set` create or update may name. `role` and
/// `sortOrder` are accepted only as values that change nothing (see
/// [`Engine::mailbox_create`] and [`Engine::mailbox_update`]).
const SETTABLE: [&str; 5] = ["name", "parentId", "isSubscribed", "role", "sortOrder"];

/// The hierarchy of one account's cached mailboxes, read from their names.
pub(super) struct Tree<'a> {
    by_name: HashMap<&'a str, &'a str>,
    delimiter: Option<char>,
}

impl<'a> Tree<'a> {
    pub(super) fn new(mailboxes: &'a [mw_store::Mailbox], delimiter: Option<char>) -> Self {
        Tree {
            by_name: mailboxes
                .iter()
                .map(|m| (m.name.as_str(), m.id.as_str()))
                .collect(),
            delimiter,
        }
    }

    /// The server name of `name`'s parent, when it has one.
    fn parent_name<'n>(&self, name: &'n str) -> Option<&'n str> {
        let (parent, leaf) = name.rsplit_once(self.delimiter?)?;
        (!parent.is_empty() && !leaf.is_empty()).then_some(parent)
    }

    /// The id of `m`'s parent mailbox, when that mailbox is cached.
    fn parent(&self, m: &mw_store::Mailbox) -> Option<&'a str> {
        self.by_name.get(self.parent_name(&m.name)?).copied()
    }

    /// `parentId` for `Mailbox/get`. A stored `parent_id` wins; otherwise the
    /// parent named by the server name.
    pub(super) fn parent_id(&self, m: &mw_store::Mailbox) -> Value {
        match &m.parent_id {
            Some(id) => json!(id),
            None => json!(self.parent(m)),
        }
    }

    /// `name` for `Mailbox/get`: the role's display name for a special-use
    /// mailbox; the leaf after the last delimiter for a child whose parent is
    /// cached; the full server name for anything else. Without a known
    /// delimiter, the older guess in [`display_name`].
    pub(super) fn name(&self, m: &mw_store::Mailbox) -> String {
        let (Some(d), None) = (self.delimiter, m.role.as_deref()) else {
            return display_name(&m.name, m.role.as_deref());
        };
        match self.parent(m) {
            Some(_) => m
                .name
                .rsplit_once(d)
                .map_or(&*m.name, |(_, leaf)| leaf)
                .to_string(),
            None if m.name.eq_ignore_ascii_case("inbox") => "Inbox".to_string(),
            None => m.name.clone(),
        }
    }

    /// Whether `name` is a descendant of `ancestor`.
    fn is_descendant(&self, name: &str, ancestor: &str) -> bool {
        self.delimiter.is_some_and(|d| {
            name.strip_prefix(ancestor)
                .is_some_and(|rest| rest.starts_with(d) && rest.len() > d.len_utf8())
        })
    }

    /// The leaf of a full server name.
    fn leaf<'n>(&self, name: &'n str) -> &'n str {
        match (self.parent_name(name), self.delimiter) {
            (Some(parent), Some(d)) => &name[parent.len() + d.len_utf8()..],
            _ => name,
        }
    }
}

/// Whether a `*/get` call's `properties` list names `prop`.
pub(super) fn wants_property(args: &Value, prop: &str) -> bool {
    args.get("properties")
        .and_then(Value::as_array)
        .is_some_and(|props| props.iter().any(|p| p.as_str() == Some(prop)))
}

/// RFC 8621 §2 `myRights` for a cached mailbox. The folder rights follow what
/// `Mailbox/set` will do: a special-use or engine-local mailbox cannot be
/// renamed or deleted, and nothing can be on an account that does not manage
/// folders.
pub(super) fn my_rights(m: &mw_store::Mailbox, manages: bool) -> Value {
    let ordinary = manages && m.role.is_none() && m.uidvalidity != 0;
    json!({
        "mayReadItems": true,
        "mayAddItems": true,
        "mayRemoveItems": true,
        "maySetSeen": true,
        "maySetKeywords": true,
        "mayCreateChild": manages && m.uidvalidity != 0,
        "mayRename": ordinary,
        "mayDelete": ordinary,
        "maySubmit": true,
    })
}

/// `myRights` for a saved-search virtual folder: it can be read, nothing else.
pub(super) fn read_only_rights() -> Value {
    json!({
        "mayReadItems": true,
        "mayAddItems": false,
        "mayRemoveItems": false,
        "maySetSeen": false,
        "maySetKeywords": false,
        "mayCreateChild": false,
        "mayRename": false,
        "mayDelete": false,
        "maySubmit": false,
    })
}

/// The RFC 8621 §1.3.1 mail account capability. `mailwomanMailboxSet` says
/// whether `Mailbox/set` works on this account; a client hides its folder
/// controls when it is `false` (POP3 and bridge accounts).
fn mail_account_capability(manages: bool) -> Value {
    json!({
        "maxMailboxesPerEmail": 1,
        "maxMailboxDepth": null,
        "maxSizeMailboxName": MAX_MAILBOX_NAME,
        "maxSizeAttachmentsPerEmail": 50_000_000,
        "emailQuerySortOptions": SORT_OPTIONS,
        "mayCreateTopLevelMailbox": manages,
        "mailwomanMailboxSet": manages,
    })
}

/// A SetError of `kind`.
fn set_err(kind: &str, description: impl Into<String>) -> Value {
    json!({ "type": kind, "description": description.into() })
}

/// An `invalidProperties` SetError naming `props`.
fn invalid(props: &[&str], description: impl Into<String>) -> Value {
    json!({
        "type": "invalidProperties",
        "properties": props,
        "description": description.into(),
    })
}

/// A create/update naming a property `Mailbox/set` does not set.
fn check_settable(spec: &Value) -> Result<&Map<String, Value>, Value> {
    let Some(obj) = spec.as_object() else {
        return Err(set_err("invalidPatch", "the mailbox must be an object"));
    };
    let unknown: Vec<&str> = obj
        .keys()
        .map(String::as_str)
        .filter(|k| !SETTABLE.contains(k))
        .collect();
    if !unknown.is_empty() {
        return Err(invalid(&unknown, "Mailbox/set cannot set these properties"));
    }
    Ok(obj)
}

/// Check a requested leaf name. Mailbox names go to the server as IMAP quoted
/// strings with no modified-UTF-7 encoding (none exists in `mw-imap` yet), so
/// only printable ASCII is accepted, without `&` (the modified-UTF-7 shift
/// character), the `*`/`%` LIST wildcards, or the hierarchy delimiter.
fn check_name(value: Option<&Value>, delimiter: Option<char>) -> Result<String, Value> {
    let Some(name) = value.and_then(Value::as_str) else {
        return Err(invalid(&["name"], "a mailbox needs a name"));
    };
    if name.trim().is_empty() {
        return Err(invalid(&["name"], "the name is empty"));
    }
    if name.len() > MAX_MAILBOX_NAME {
        return Err(invalid(
            &["name"],
            format!("the name is longer than {MAX_MAILBOX_NAME} octets"),
        ));
    }
    if !name.chars().all(|c| c.is_ascii() && !c.is_ascii_control()) {
        return Err(invalid(
            &["name"],
            "folder names outside printable ASCII are not supported yet",
        ));
    }
    if let Some(bad) = name
        .chars()
        .find(|c| matches!(c, '&' | '*' | '%') || Some(*c) == delimiter)
    {
        return Err(invalid(
            &["name"],
            format!("the name cannot contain {bad:?}"),
        ));
    }
    Ok(name.to_string())
}

impl Engine {
    /// The account's hierarchy delimiter, or `None` when the backend has none
    /// or cannot say.
    pub(super) async fn hierarchy_delimiter(&self, rt: &AccountRuntime) -> Option<char> {
        match rt.backend.hierarchy_delimiter().await {
            Ok(d) => d,
            Err(EngineError::Unsupported(_)) => None,
            Err(e) => {
                tracing::warn!("reading the hierarchy delimiter failed: {e}");
                None
            }
        }
    }

    /// Whether `Mailbox/set` works on this account: the backend implements the
    /// folder verbs and the account is not bridged through a plugin.
    pub(super) fn manages_folders(&self, account_id: &str, rt: &AccountRuntime) -> bool {
        rt.backend.manages_mailboxes() && !self.is_plugin_backed(account_id)
    }

    /// The subscribed mailbox names, or `None` when the backend cannot list them.
    pub(super) async fn subscribed_names(&self, rt: &AccountRuntime) -> Option<HashSet<String>> {
        match tolerant(rt.backend.subscribed_mailboxes().await) {
            Ok(names) => names.map(|n| n.into_iter().collect()),
            Err(e) => {
                tracing::warn!("listing subscribed mailboxes failed: {e}");
                None
            }
        }
    }

    /// The JMAP session for a connected account: [`super::session_json`] plus
    /// the RFC 8621 mail account capability, whose `mayCreateTopLevelMailbox`
    /// and `mailwomanMailboxSet` say whether this account's folders can be
    /// managed. An account with no runtime gets `false`.
    pub fn session_json_for(&self, account_id: &str, username: &str) -> Value {
        let mut session = super::session_json(account_id, username);
        let manages = self
            .runtime(account_id)
            .is_some_and(|rt| self.manages_folders(account_id, &rt));
        if let Some(caps) = session
            .pointer_mut(&format!("/accounts/{account_id}/accountCapabilities"))
            .and_then(Value::as_object_mut)
        {
            caps.insert(
                "urn:ietf:params:jmap:mail".into(),
                mail_account_capability(manages),
            );
        }
        session
    }

    // ---- Mailbox/set ------------------------------------------------------

    /// `Mailbox/set` (RFC 8621 §2.5): create, rename/move, (un)subscribe and
    /// delete folders on the mail server, then mirror the result in the cache.
    ///
    /// Refused as a whole with `accountNotSupportedByMethod` on an account
    /// whose backend has no folder verbs (POP3, bridges). Per id:
    /// * a special-use or engine-local mailbox is not renamed, moved or
    ///   deleted (`forbidden`), and no mailbox is given a role;
    /// * a sibling with the same name is `alreadyExists`;
    /// * a mailbox with children is `mailboxHasChild`, and one holding
    ///   messages is `mailboxHasEmail` unless `onDestroyRemoveEmails` is true.
    pub(super) async fn mailbox_set(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        args: &Value,
    ) -> Value {
        if !self.manages_folders(account_id, rt) {
            return json!({
                "type": "accountNotSupportedByMethod",
                "description": "this account's mail server does not support managing folders",
            });
        }
        let old_state = self
            .type_state(account_id, ChangeType::Mailbox)
            .await
            .unwrap_or_default();
        if let Some(expected) = args.get("ifInState").and_then(Value::as_str)
            && expected != old_state
        {
            return json!({
                "type": "stateMismatch",
                "description": "ifInState does not match the current Mailbox state",
            });
        }
        let delimiter = self.hierarchy_delimiter(rt).await;

        let mut created = Map::new();
        let mut not_created = Map::new();
        let mut created_ids: HashMap<String, String> = HashMap::new();
        if let Some(creates) = args.get("create").and_then(Value::as_object) {
            for (client_id, spec) in creation_order(creates) {
                match self
                    .mailbox_create(account_id, rt, delimiter, spec, &created_ids)
                    .await
                {
                    Ok((id, entry)) => {
                        created_ids.insert(client_id.clone(), id);
                        created.insert(client_id.clone(), entry);
                    }
                    Err(e) => {
                        not_created.insert(client_id.clone(), e);
                    }
                }
            }
        }

        let mut updated = Map::new();
        let mut not_updated = Map::new();
        if let Some(updates) = args.get("update").and_then(Value::as_object) {
            for (id, patch) in updates {
                let id = resolve_creation_ref(id, &created_ids);
                match self
                    .mailbox_update(account_id, rt, delimiter, &id, patch, &created_ids)
                    .await
                {
                    Ok(()) => {
                        updated.insert(id, Value::Null);
                    }
                    Err(e) => {
                        not_updated.insert(id, e);
                    }
                }
            }
        }

        let mut destroyed = Vec::new();
        let mut not_destroyed = Map::new();
        if let Some(ids) = args.get("destroy").and_then(Value::as_array) {
            let remove_emails = args
                .get("onDestroyRemoveEmails")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            // Children before parents, so a request destroying a whole branch
            // succeeds (RFC 8621 §2.5 lets a server order the destroys).
            let names: HashMap<String, String> = self
                .store()
                .list_mailboxes(account_id)
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|m| (m.id, m.name))
                .collect();
            let mut ids: Vec<String> = ids
                .iter()
                .filter_map(Value::as_str)
                .map(|id| resolve_creation_ref(id, &created_ids))
                .collect();
            ids.sort_by_key(|id| std::cmp::Reverse(names.get(id).map_or(0, String::len)));
            ids.dedup();
            for id in ids {
                match self
                    .mailbox_destroy(account_id, rt, delimiter, &id, remove_emails)
                    .await
                {
                    Ok(()) => destroyed.push(json!(id)),
                    Err(e) => {
                        not_destroyed.insert(id, e);
                    }
                }
            }
        }

        let new_state = self
            .type_state(account_id, ChangeType::Mailbox)
            .await
            .unwrap_or_default();
        let mut resp = json!({
            "accountId": account_id,
            "oldState": old_state,
            "newState": new_state,
            "created": created,
            "updated": updated,
            "destroyed": destroyed,
        });
        for (key, map) in [
            ("notCreated", not_created),
            ("notUpdated", not_updated),
            ("notDestroyed", not_destroyed),
        ] {
            if !map.is_empty() {
                resp[key] = Value::Object(map);
            }
        }
        resp
    }

    /// One account's cached mailboxes, or a `serverFail` SetError.
    async fn mailboxes_or_fail(&self, account_id: &str) -> Result<Vec<mw_store::Mailbox>, Value> {
        self.store()
            .list_mailboxes(account_id)
            .await
            .map_err(|e| server_fail(&e))
    }

    /// Resolve a `parentId` value: `None` for top level, else the cached
    /// mailbox it names (an id, or `#creationId` from this call).
    fn resolve_parent<'m>(
        value: Option<&Value>,
        mailboxes: &'m [mw_store::Mailbox],
        created_ids: &HashMap<String, String>,
    ) -> Result<Option<&'m mw_store::Mailbox>, Value> {
        match value {
            None | Some(Value::Null) => Ok(None),
            Some(Value::String(id)) => {
                let id = resolve_creation_ref(id, created_ids);
                mailboxes
                    .iter()
                    .find(|m| m.id == id && m.uidvalidity != 0)
                    .map(Some)
                    .ok_or_else(|| invalid(&["parentId"], "no such mailbox on the mail server"))
            }
            Some(_) => Err(invalid(&["parentId"], "parentId must be an id or null")),
        }
    }

    /// The full server name for `leaf` under `parent`.
    fn full_name(
        parent: Option<&mw_store::Mailbox>,
        leaf: &str,
        delimiter: Option<char>,
    ) -> Result<String, Value> {
        match (parent, delimiter) {
            (None, _) => Ok(leaf.to_string()),
            (Some(p), Some(d)) => Ok(format!("{}{d}{leaf}", p.name)),
            (Some(_), None) => Err(invalid(
                &["parentId"],
                "the mail server has a flat namespace; folders cannot be nested",
            )),
        }
    }

    /// `alreadyExists` when a cached mailbox other than `except` has `name`.
    fn check_free(
        mailboxes: &[mw_store::Mailbox],
        name: &str,
        except: Option<&str>,
    ) -> Result<(), Value> {
        let taken = mailboxes.iter().find(|m| {
            Some(m.id.as_str()) != except
                && (m.name == name
                    || (m.name.eq_ignore_ascii_case("inbox") && name.eq_ignore_ascii_case("inbox")))
        });
        match taken {
            Some(m) => Err(json!({
                "type": "alreadyExists",
                "existingId": m.id,
                "description": "a mailbox with this name already exists here",
            })),
            None => Ok(()),
        }
    }

    /// Cache the upstream mailbox `name` after a CREATE, with the UIDVALIDITY
    /// the server reports, so the next sync upserts the same row.
    async fn cache_new_mailbox(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        name: &str,
    ) -> Result<String, Value> {
        let listed = rt
            .backend
            .list_mailboxes()
            .await
            .map_err(|e| set_error(&e))?;
        let Some(rm) = listed.iter().find(|m| m.mailbox_ref.name == name) else {
            return Err(set_err(
                "serverFail",
                format!("the mail server created {name:?} but does not list it"),
            ));
        };
        self.store()
            .upsert_mailbox(&mw_store::MailboxUpsert {
                account_id,
                name,
                role: crate::mapping::role_to_store(rm.role),
                uidvalidity: rm.mailbox_ref.uidvalidity,
                uidnext: rm.uidnext,
                highestmodseq: rm.highestmodseq,
                total: rm.total,
                unread: rm.unread,
                parent_id: None,
            })
            .await
            .map_err(|e| server_fail(&e))
    }

    async fn mailbox_create(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        delimiter: Option<char>,
        spec: &Value,
        created_ids: &HashMap<String, String>,
    ) -> Result<(String, Value), Value> {
        let obj = check_settable(spec)?;
        if obj.get("role").is_some_and(|r| !r.is_null()) {
            return Err(invalid(
                &["role"],
                "a new folder cannot take a special-use role",
            ));
        }
        if obj.get("sortOrder").is_some_and(|o| o.as_u64() != Some(0)) {
            return Err(invalid(&["sortOrder"], "folder sort order is not stored"));
        }
        let leaf = check_name(obj.get("name"), delimiter)?;
        let mailboxes = self.mailboxes_or_fail(account_id).await?;
        let parent = Self::resolve_parent(obj.get("parentId"), &mailboxes, created_ids)?;
        let name = Self::full_name(parent, &leaf, delimiter)?;
        Self::check_free(&mailboxes, &name, None)?;

        rt.backend
            .create_mailbox(&name)
            .await
            .map_err(|e| set_error(&e))?;
        // A new folder is subscribed unless the request says otherwise, so a
        // client that lists only subscribed folders shows it too.
        let subscribe = obj
            .get("isSubscribed")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        let mbref = RawMailboxRef {
            name: name.clone(),
            uidvalidity: 0,
        };
        let subscribed = match tolerant(rt.backend.subscribe(&mbref, subscribe).await) {
            Ok(Some(())) => subscribe,
            Ok(None) => false,
            Err(e) => {
                tracing::warn!("subscribing to the new folder {name:?} failed: {e}");
                false
            }
        };
        let id = self.cache_new_mailbox(account_id, rt, &name).await?;
        if let Err(e) = self
            .record_change(account_id, ChangeType::Mailbox, &id, ChangeOp::Created)
            .await
        {
            tracing::warn!("recording the Mailbox create failed: {e}");
        }
        let row = self.store().get_mailbox(&id).await.ok();
        Ok((
            id.clone(),
            json!({
                "id": id,
                "parentId": parent.map(|p| p.id.clone()),
                "role": null,
                "sortOrder": 0,
                "totalEmails": row.as_ref().map_or(0, |r| r.total),
                "unreadEmails": row.as_ref().map_or(0, |r| r.unread),
                "totalThreads": row.as_ref().map_or(0, |r| r.total),
                "unreadThreads": row.as_ref().map_or(0, |r| r.unread),
                "isSubscribed": subscribed,
                "myRights": row.as_ref().map(|r| my_rights(r, true)),
            }),
        ))
    }

    async fn mailbox_update(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        delimiter: Option<char>,
        id: &str,
        patch: &Value,
        created_ids: &HashMap<String, String>,
    ) -> Result<(), Value> {
        let obj = check_settable(patch)?;
        if obj.is_empty() {
            return Err(set_err(
                "invalidPatch",
                "the update names no property to change",
            ));
        }
        let mailboxes = self.mailboxes_or_fail(account_id).await?;
        let Some(m) = mailboxes.iter().find(|m| m.id == id) else {
            return Err(set_err("notFound", "no such mailbox"));
        };
        if let Some(role) = obj.get("role")
            && role.as_str() != m.role.as_deref()
        {
            return Err(invalid(
                &["role"],
                "a folder's special-use role cannot be changed",
            ));
        }
        if obj.get("sortOrder").is_some_and(|o| o.as_u64() != Some(0)) {
            return Err(invalid(&["sortOrder"], "folder sort order is not stored"));
        }
        if obj.contains_key("isSubscribed") && !obj["isSubscribed"].is_boolean() {
            return Err(invalid(
                &["isSubscribed"],
                "isSubscribed must be true or false",
            ));
        }
        let tree = Tree::new(&mailboxes, delimiter);

        // Rename and/or move.
        let mut name_now = m.name.clone();
        if obj.contains_key("name") || obj.contains_key("parentId") {
            let leaf = match obj.get("name") {
                Some(v) => check_name(Some(v), delimiter)?,
                None => tree.leaf(&m.name).to_string(),
            };
            let parent = if obj.contains_key("parentId") {
                Self::resolve_parent(obj.get("parentId"), &mailboxes, created_ids)?
            } else {
                tree.parent_name(&m.name)
                    .and_then(|p| mailboxes.iter().find(|x| x.name == p))
            };
            if let Some(p) = parent
                && (p.id == m.id || tree.is_descendant(&p.name, &m.name))
            {
                return Err(invalid(&["parentId"], "a folder cannot move inside itself"));
            }
            let new_name = Self::full_name(parent, &leaf, delimiter)?;
            if new_name != m.name {
                if m.role.is_some() || m.uidvalidity == 0 {
                    return Err(set_err(
                        "forbidden",
                        "a special-use folder cannot be renamed or moved",
                    ));
                }
                Self::check_free(&mailboxes, &new_name, Some(&m.id))?;
                self.rename_folder(account_id, rt, &tree, &mailboxes, m, &new_name)
                    .await?;
                name_now = new_name;
            }
        }

        if let Some(on) = obj.get("isSubscribed").and_then(Value::as_bool) {
            let mbref = RawMailboxRef {
                name: name_now,
                uidvalidity: m.uidvalidity,
            };
            rt.backend
                .subscribe(&mbref, on)
                .await
                .map_err(|e| set_error(&e))?;
            if let Err(e) = self
                .record_change(account_id, ChangeType::Mailbox, &m.id, ChangeOp::Updated)
                .await
            {
                tracing::warn!("recording the Mailbox update failed: {e}");
            }
        }
        Ok(())
    }

    /// RENAME `m` (and so its children) to `new_name` upstream, then in the
    /// cache, carrying subscriptions across and following any UIDVALIDITY the
    /// server changed.
    async fn rename_folder(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        tree: &Tree<'_>,
        mailboxes: &[mw_store::Mailbox],
        m: &mw_store::Mailbox,
        new_name: &str,
    ) -> Result<(), Value> {
        // Every (old, new) server name the rename produces: the folder, then
        // each descendant with the prefix swapped.
        let pairs: Vec<(String, String)> = mailboxes
            .iter()
            .filter_map(|x| {
                if x.id == m.id {
                    Some((x.name.clone(), new_name.to_string()))
                } else if tree.is_descendant(&x.name, &m.name) {
                    Some((
                        x.name.clone(),
                        format!("{new_name}{}", &x.name[m.name.len()..]),
                    ))
                } else {
                    None
                }
            })
            .collect();
        let subscribed = self.subscribed_names(rt).await;

        let from = RawMailboxRef {
            name: m.name.clone(),
            uidvalidity: m.uidvalidity,
        };
        rt.backend
            .rename_mailbox(&from, new_name)
            .await
            .map_err(|e| set_error(&e))?;
        let renamed = self
            .store()
            .rename_mailbox_tree(account_id, &m.name, new_name, tree.delimiter)
            .await
            .map_err(|e| server_fail(&e))?;

        // RFC 3501 leaves subscriptions where they were on a RENAME, so a
        // subscribed folder would drop out of a subscribed-only client's list.
        if let Some(subscribed) = subscribed {
            for (old, new) in &pairs {
                if subscribed.contains(old) && !subscribed.contains(new) {
                    let at = |name: &str| RawMailboxRef {
                        name: name.to_string(),
                        uidvalidity: 0,
                    };
                    let _ = rt.backend.subscribe(&at(new), true).await;
                    let _ = rt.backend.subscribe(&at(old), false).await;
                }
            }
        }

        // A server may give a renamed mailbox a new UIDVALIDITY; the cursor is
        // then dropped so the next sync re-keys the messages.
        if let Ok(listed) = rt.backend.list_mailboxes().await {
            for (old, new) in &pairs {
                let (Some(row), Some(up)) = (
                    mailboxes.iter().find(|x| &x.name == old),
                    listed.iter().find(|l| &l.mailbox_ref.name == new),
                ) else {
                    continue;
                };
                if up.mailbox_ref.uidvalidity != row.uidvalidity
                    && let Err(e) = self
                        .store()
                        .revalidate_mailbox(&row.id, up.mailbox_ref.uidvalidity)
                        .await
                {
                    tracing::warn!("revalidating renamed mailbox {new:?} failed: {e}");
                }
            }
        }

        for id in &renamed {
            if let Err(e) = self
                .record_change(account_id, ChangeType::Mailbox, id, ChangeOp::Updated)
                .await
            {
                tracing::warn!("recording the Mailbox rename failed: {e}");
            }
        }
        Ok(())
    }

    async fn mailbox_destroy(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        delimiter: Option<char>,
        id: &str,
        remove_emails: bool,
    ) -> Result<(), Value> {
        let mailboxes = self.mailboxes_or_fail(account_id).await?;
        let Some(m) = mailboxes.iter().find(|m| m.id == id) else {
            return Err(set_err("notFound", "no such mailbox"));
        };
        if m.role.is_some() || m.uidvalidity == 0 {
            return Err(set_err(
                "forbidden",
                "a special-use folder cannot be deleted",
            ));
        }
        // The checks read the server, not only the cache: another client may
        // have filed a message or made a subfolder since the last sync, and an
        // IMAP DELETE takes the messages with it.
        let listed = rt
            .backend
            .list_mailboxes()
            .await
            .map_err(|e| set_error(&e))?;
        let tree = Tree::new(&mailboxes, delimiter);
        let has_child = mailboxes
            .iter()
            .map(|x| x.name.as_str())
            .chain(listed.iter().map(|l| l.mailbox_ref.name.as_str()))
            .any(|name| tree.is_descendant(name, &m.name));
        if has_child {
            return Err(set_err(
                "mailboxHasChild",
                "the folder has subfolders; delete or move them first",
            ));
        }
        let cached = self
            .store()
            .count_messages_in_mailbox(id)
            .await
            .map_err(|e| server_fail(&e))?;
        let upstream = listed
            .iter()
            .find(|l| l.mailbox_ref.name == m.name)
            .map_or(0, |l| l.total);
        if !remove_emails && (cached > 0 || m.total > 0 || upstream > 0) {
            return Err(set_err(
                "mailboxHasEmail",
                "the folder holds messages; set onDestroyRemoveEmails to delete them with it",
            ));
        }

        let mbref = RawMailboxRef {
            name: m.name.clone(),
            uidvalidity: m.uidvalidity,
        };
        rt.backend
            .delete_mailbox(&mbref)
            .await
            .map_err(|e| set_error(&e))?;

        let messages: Vec<(String, String)> = self
            .store()
            .list_message_ids(id, i64::MAX, 0)
            .await
            .map_err(|e| server_fail(&e))?
            .into_iter()
            .map(|sid| (sid, id.to_string()))
            .collect();
        let failed = self
            .remove_emails_locally(account_id, &messages, false)
            .await;
        if let Some((_, why)) = failed.into_iter().next() {
            return Err(set_err(
                "serverFail",
                format!(
                    "deleted on the mail server, but the cached messages were not all removed: {why}"
                ),
            ));
        }
        self.store()
            .delete_mailbox(id)
            .await
            .map_err(|e| server_fail(&e))?;
        if let Err(e) = self
            .record_change(account_id, ChangeType::Mailbox, id, ChangeOp::Destroyed)
            .await
        {
            tracing::warn!("recording the Mailbox destroy failed: {e}");
        }
        Ok(())
    }

    // ---- Email/set destroy -------------------------------------------------

    /// The `destroy` half of `Email/set` (26.20 t28-e17, D-20): remove each
    /// message from the mail server, then from the cache and the index.
    ///
    /// Messages the server knows (IMAP coordinates) go in one
    /// `expunge_messages` call; a message with no server coordinates (a local
    /// draft, a POP3 message) is removed from the cache only. When the backend
    /// cannot expunge (`Unsupported`), each message is moved to Trash instead
    /// and reported in `notDestroyed` with `mailwomanMovedTo`; when it is
    /// already in Trash, or there is no Trash on the server, it stays where it
    /// is and is reported as `forbidden`.
    pub(super) async fn destroy_emails(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        ids: &[String],
    ) -> (Vec<Value>, Map<String, Value>) {
        let mut not_destroyed = Map::new();
        let mut local: Vec<(String, String)> = Vec::new();
        let mut upstream: Vec<(String, String, MessageRef)> = Vec::new();
        let mut names: HashMap<String, String> = HashMap::new();
        let mut seen: HashSet<&str> = HashSet::new();
        for id in ids {
            if !seen.insert(id.as_str()) {
                continue;
            }
            let msg = match self.store().get_message(id).await {
                Ok(msg) if msg.account_id == account_id => msg,
                Ok(_) | Err(mw_store::StoreError::NotFound) => {
                    not_destroyed.insert(id.clone(), set_err("notFound", "no such Email"));
                    continue;
                }
                Err(e) => {
                    not_destroyed.insert(id.clone(), set_error(&EngineError::Store(e)));
                    continue;
                }
            };
            match self.imap_ref_from(&msg, &mut names).await {
                Ok(Some(mref)) => upstream.push((id.clone(), msg.mailbox_id, mref)),
                Ok(None) => local.push((id.clone(), msg.mailbox_id)),
                Err(e) => {
                    not_destroyed.insert(id.clone(), set_error(&e));
                }
            }
        }

        if !upstream.is_empty() {
            let refs: Vec<MessageRef> = upstream.iter().map(|(_, _, r)| r.clone()).collect();
            match rt.backend.expunge_messages(&refs).await {
                Ok(()) => local.extend(upstream.into_iter().map(|(id, mb, _)| (id, mb))),
                Err(EngineError::Unsupported(why)) => {
                    for (id, mailbox_id, _) in upstream {
                        let outcome = self
                            .destroy_into_trash(account_id, rt, &id, &mailbox_id, &why)
                            .await;
                        not_destroyed.insert(id, outcome);
                    }
                }
                Err(e) => {
                    for (id, ..) in upstream {
                        not_destroyed.insert(id, set_error(&e));
                    }
                }
            }
        }

        let failed = self.remove_emails_locally(account_id, &local, true).await;
        let mut destroyed = Vec::new();
        for (id, _) in &local {
            match failed.iter().find(|(f, _)| f == id) {
                Some((_, why)) => {
                    not_destroyed.insert(id.clone(), set_err("serverFail", why.clone()));
                }
                None => destroyed.push(json!(id)),
            }
        }
        (destroyed, not_destroyed)
    }

    /// The fallback when the server cannot expunge: move the message to the
    /// server's Trash. Returns the `notDestroyed` entry either way.
    async fn destroy_into_trash(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        id: &str,
        mailbox_id: &str,
        why: &str,
    ) -> Value {
        let trash = self
            .store()
            .list_mailboxes(account_id)
            .await
            .unwrap_or_default()
            .into_iter()
            .find(|m| m.role.as_deref() == Some("trash") && m.uidvalidity != 0);
        match trash {
            Some(t) if t.id != mailbox_id => match self.move_email(rt, id, &t.id).await {
                Ok(()) => json!({
                    "type": "forbidden",
                    "description": format!(
                        "the mail server cannot remove this message ({why}); it was moved to Trash instead"
                    ),
                    "mailwomanMovedTo": t.id,
                }),
                Err(e) => set_error(&e),
            },
            _ => set_err(
                "forbidden",
                format!("the mail server cannot remove this message: {why}"),
            ),
        }
    }

    /// Drop messages from the cache, the per-message caches and the search
    /// index, and record their `Email` destroyed changes at one state (plus a
    /// `Mailbox` update per mailbox touched, when `touch_mailboxes`). Returns
    /// the ids that could not be removed, with the text the client is given.
    async fn remove_emails_locally(
        &self,
        account_id: &str,
        items: &[(String, String)],
        touch_mailboxes: bool,
    ) -> Vec<(String, String)> {
        let mut gone: Vec<String> = Vec::new();
        let mut mailboxes: HashSet<&str> = HashSet::new();
        let mut failed = Vec::new();
        for (id, mailbox_id) in items {
            match self.store().delete_message(id).await {
                Ok(()) => {
                    self.invalidate_message_cache(id).await;
                    gone.push(id.clone());
                    mailboxes.insert(mailbox_id);
                }
                Err(e) => {
                    tracing::warn!("removing message {id} from the cache failed: {e}");
                    failed.push((id.clone(), SERVER_FAIL_TEXT.to_string()));
                }
            }
        }
        if gone.is_empty() {
            return failed;
        }
        if let Err(e) = self.search().delete_batch(&gone) {
            tracing::warn!("removing destroyed messages from the search index failed: {e}");
        }
        if let Err(e) = self
            .store()
            .record_changes(
                account_id,
                ChangeType::Email.as_str(),
                &gone,
                ChangeOp::Destroyed.as_str(),
            )
            .await
        {
            tracing::warn!("recording the Email destroys failed: {e}");
        }
        if touch_mailboxes {
            for mailbox_id in mailboxes {
                if let Err(e) = self
                    .record_change(
                        account_id,
                        ChangeType::Mailbox,
                        mailbox_id,
                        ChangeOp::Updated,
                    )
                    .await
                {
                    tracing::warn!("recording the Mailbox count change failed: {e}");
                }
            }
        }
        failed
    }
}

/// The creates in an order where a `parentId` of `#creationId` names a create
/// already made: parents before children, whatever order the request's object
/// iterates in. A reference to a create that is not in the request (or a
/// cycle) is left for the create itself to refuse.
fn creation_order(creates: &Map<String, Value>) -> Vec<(&String, &Value)> {
    let mut pending: Vec<(&String, &Value)> = creates.iter().collect();
    let mut ordered = Vec::with_capacity(pending.len());
    let mut placed: HashSet<&str> = HashSet::new();
    while !pending.is_empty() {
        let before = pending.len();
        pending.retain(|entry| {
            let (cid, spec) = *entry;
            let waits = spec
                .get("parentId")
                .and_then(Value::as_str)
                .and_then(|p| p.strip_prefix('#'))
                .is_some_and(|p| creates.contains_key(p) && !placed.contains(p));
            if waits {
                return true;
            }
            placed.insert(cid.as_str());
            ordered.push((cid, spec));
            false
        });
        if pending.len() == before {
            ordered.append(&mut pending);
        }
    }
    ordered
}

/// `#creationId` → the id this call created for it; anything else unchanged.
fn resolve_creation_ref(id: &str, created_ids: &HashMap<String, String>) -> String {
    id.strip_prefix('#')
        .and_then(|cid| created_ids.get(cid))
        .cloned()
        .unwrap_or_else(|| id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mbox(id: &str, name: &str, role: Option<&str>) -> mw_store::Mailbox {
        mw_store::Mailbox {
            id: id.into(),
            account_id: "a".into(),
            name: name.into(),
            role: role.map(str::to_string),
            uidvalidity: 1,
            uidnext: 1,
            highestmodseq: 0,
            total: 0,
            unread: 0,
            parent_id: None,
        }
    }

    #[test]
    fn tree_reads_parents_and_leaves_from_the_delimiter() {
        let boxes = vec![
            mbox("i", "INBOX", Some("inbox")),
            mbox("w", "Work", None),
            mbox("q", "Work/Q3.v2", None),
            mbox("o", "Orphan/Child", None),
        ];
        let tree = Tree::new(&boxes, Some('/'));
        assert_eq!(tree.parent_id(&boxes[2]), json!("w"));
        // The `.` in the leaf is not a delimiter on this server.
        assert_eq!(tree.name(&boxes[2]), "Q3.v2");
        assert_eq!(tree.parent_id(&boxes[1]), Value::Null);
        assert_eq!(tree.name(&boxes[1]), "Work");
        // A child whose parent is not cached keeps its full name.
        assert_eq!(tree.parent_id(&boxes[3]), Value::Null);
        assert_eq!(tree.name(&boxes[3]), "Orphan/Child");
        assert_eq!(tree.name(&boxes[0]), "Inbox");
        assert!(tree.is_descendant("Work/Q3.v2", "Work"));
        assert!(!tree.is_descendant("Workshop", "Work"));
        assert!(!tree.is_descendant("Work", "Work"));
        assert_eq!(tree.leaf("Work/Q3.v2"), "Q3.v2");
    }

    #[test]
    fn names_are_checked_before_they_reach_the_server() {
        assert_eq!(
            check_name(Some(&json!("Receipts")), Some('.')),
            Ok("Receipts".into())
        );
        for bad in ["", "  ", "a.b", "50%", "a*", "R&D", "Café", "tab\there"] {
            let err = check_name(Some(&json!(bad)), Some('.')).unwrap_err();
            assert_eq!(err["type"], "invalidProperties", "{bad:?}: {err}");
            assert_eq!(err["properties"], json!(["name"]), "{bad:?}");
        }
        assert!(check_name(Some(&json!("x".repeat(256))), None).is_err());
        assert!(check_name(None, None).is_err());
    }

    #[test]
    fn rights_follow_what_mailbox_set_allows() {
        let plain = mbox("w", "Work", None);
        let trash = mbox("t", "Trash", Some("trash"));
        assert_eq!(my_rights(&plain, true)["mayDelete"], true);
        assert_eq!(my_rights(&plain, false)["mayDelete"], false);
        assert_eq!(my_rights(&trash, true)["mayRename"], false);
        assert_eq!(my_rights(&trash, true)["mayCreateChild"], true);
    }

    /// Internal failures reach the client as a fixed text; the request's own
    /// errors keep theirs (t32-h10's engine item).
    #[test]
    fn internal_error_text_stays_on_the_server() {
        let internal = [
            EngineError::Transport("connect 10.0.0.5:993 refused".into()),
            EngineError::Auth("LOGIN rejected for admin".into()),
            EngineError::Store(mw_store::StoreError::NotFound),
        ];
        for e in internal {
            let v = set_error(&e);
            assert_eq!(v["description"], SERVER_FAIL_TEXT, "{e}");
        }
        let own = set_error(&EngineError::Protocol("unknown submission".into()));
        assert_eq!(
            own["description"],
            "backend protocol error: unknown submission"
        );
        let whole = server_fail(&EngineError::Transport("db at /var/lib/mw".into()));
        assert_eq!(whole["description"], SERVER_FAIL_TEXT);
    }
}
