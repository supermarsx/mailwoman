//! `AddressBook/*`, `ContactCard/*`, `ContactGroup/*` (frozen §2.2): CardDAV /
//! vCard-backed contacts. `vcard_raw` is the round-trip source of truth (plan
//! risk #13); the projection is `mw_ics::parse_vcard`. Includes merge-duplicates
//! (new card + tombstones, reversible), vCard/CSV import/export, and the
//! Compose recipient `autocomplete`.

use mw_store::{AddressBookRow, ContactGroupRow, ContactRow};
use serde_json::{Value, json};

use crate::account::AccountRuntime;
use crate::backend::{EngineError, Result};
use crate::change::{ChangeOp, ChangeType};
use crate::engine::Engine;

use super::events::resource_href;
use super::{
    SetOutcome, gen_id, gen_token, get_response, query_response, server_fail, set_error, wanted_ids,
};

impl Engine {
    // ── AddressBook/{get,set} ────────────────────────────────────────────────

    pub(crate) async fn address_book_get(&self, account_id: &str, args: &Value) -> Value {
        if let Err(e) = self.ensure_default_address_book(account_id).await {
            return server_fail(e);
        }
        let state = self
            .pim_type_state(account_id, ChangeType::AddressBook)
            .await
            .unwrap_or_default();
        let rows = match self.store().list_address_books(account_id).await {
            Ok(v) => v,
            Err(e) => return server_fail(e),
        };
        let wanted = wanted_ids(args);
        let mut list = Vec::new();
        let mut found = Vec::new();
        for row in &rows {
            if let Some(ids) = &wanted
                && !ids.contains(&row.id)
            {
                continue;
            }
            found.push(row.id.clone());
            list.push(address_book_to_json(row));
        }
        let not_found = match &wanted {
            Some(ids) => ids
                .iter()
                .filter(|id| !found.contains(id))
                .map(|id| json!(id))
                .collect(),
            None => Vec::new(),
        };
        get_response(account_id, &state, list, not_found)
    }

    pub(crate) async fn address_book_set(&self, account_id: &str, args: &Value) -> Value {
        let old_state = self
            .pim_type_state(account_id, ChangeType::AddressBook)
            .await
            .unwrap_or_default();
        let mut out = SetOutcome::default();

        if let Some(creates) = args.get("create").and_then(Value::as_object) {
            for (cid, spec) in creates {
                let id = gen_id("ab");
                let row = AddressBookRow {
                    id: id.clone(),
                    account_id: account_id.to_string(),
                    name: spec
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or("Contacts")
                        .to_string(),
                    is_default: spec
                        .get("isDefault")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                    carddav_url: spec
                        .get("carddavUrl")
                        .and_then(Value::as_str)
                        .map(String::from),
                    sync_token: spec
                        .get("syncToken")
                        .and_then(Value::as_str)
                        .map(String::from),
                    ctag: None,
                };
                match self.store().upsert_address_book(&row).await {
                    Ok(()) => {
                        let _ = self
                            .record_pim_change(
                                account_id,
                                ChangeType::AddressBook,
                                &id,
                                ChangeOp::Created,
                            )
                            .await;
                        out.created.insert(cid.clone(), json!({ "id": id }));
                    }
                    Err(e) => {
                        out.not_created
                            .insert(cid.clone(), set_error("serverFail", e));
                    }
                }
            }
        }
        if let Some(updates) = args.get("update").and_then(Value::as_object) {
            for (id, patch) in updates {
                match self.address_book_update(account_id, id, patch).await {
                    Ok(()) => {
                        out.updated.insert(id.clone(), Value::Null);
                    }
                    Err(e) => {
                        out.not_updated
                            .insert(id.clone(), set_error("serverFail", e));
                    }
                }
            }
        }
        if let Some(destroys) = args.get("destroy").and_then(Value::as_array) {
            for id in destroys.iter().filter_map(Value::as_str) {
                match self.store().delete_address_book(id).await {
                    Ok(()) => {
                        let _ = self
                            .record_pim_change(
                                account_id,
                                ChangeType::AddressBook,
                                id,
                                ChangeOp::Destroyed,
                            )
                            .await;
                        out.destroyed.push(json!(id));
                    }
                    Err(e) => {
                        out.not_destroyed
                            .insert(id.to_string(), set_error("serverFail", e));
                    }
                }
            }
        }

        let new_state = self
            .pim_type_state(account_id, ChangeType::AddressBook)
            .await
            .unwrap_or_default();
        self.broadcast_state(account_id).await;
        out.into_response(account_id, &old_state, &new_state)
    }

    async fn address_book_update(&self, account_id: &str, id: &str, patch: &Value) -> Result<()> {
        let mut row = self
            .store()
            .get_address_book(id)
            .await?
            .ok_or_else(|| EngineError::Protocol(format!("unknown address book {id}")))?;
        if let Some(v) = patch.get("name").and_then(Value::as_str) {
            row.name = v.to_string();
        }
        if let Some(v) = patch.get("isDefault").and_then(Value::as_bool) {
            row.is_default = v;
        }
        if let Some(v) = patch.get("carddavUrl") {
            row.carddav_url = v.as_str().map(String::from);
        }
        self.store().upsert_address_book(&row).await?;
        self.record_pim_change(account_id, ChangeType::AddressBook, id, ChangeOp::Updated)
            .await?;
        Ok(())
    }

    // ── ContactCard/{get,set,query,queryChanges} ─────────────────────────────

    pub(crate) async fn contact_get(&self, account_id: &str, args: &Value) -> Value {
        let state = self
            .pim_type_state(account_id, ChangeType::ContactCard)
            .await
            .unwrap_or_default();
        let ids = match wanted_ids(args) {
            Some(ids) => ids,
            None => match self.all_contact_ids(account_id).await {
                Ok(v) => v,
                Err(e) => return server_fail(e),
            },
        };
        let mut list = Vec::new();
        let mut not_found = Vec::new();
        for id in &ids {
            match self.store().get_contact(id).await {
                Ok(Some(row)) => list.push(contact_row_to_json(&row)),
                Ok(None) => not_found.push(json!(id)),
                Err(e) => return server_fail(e),
            }
        }
        get_response(account_id, &state, list, not_found)
    }

    pub(crate) async fn contact_set(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        args: &Value,
    ) -> Value {
        let old_state = self
            .pim_type_state(account_id, ChangeType::ContactCard)
            .await
            .unwrap_or_default();
        let mut out = SetOutcome::default();

        if let Some(creates) = args.get("create").and_then(Value::as_object) {
            for (cid, spec) in creates {
                match self.contact_create(account_id, rt, spec).await {
                    Ok(id) => {
                        out.created.insert(cid.clone(), json!({ "id": id }));
                    }
                    Err(e) => {
                        out.not_created
                            .insert(cid.clone(), set_error("invalidProperties", e));
                    }
                }
            }
        }
        if let Some(updates) = args.get("update").and_then(Value::as_object) {
            for (id, patch) in updates {
                match self.contact_update(account_id, rt, id, patch).await {
                    Ok(()) => {
                        out.updated.insert(id.clone(), Value::Null);
                    }
                    Err(e) => {
                        out.not_updated
                            .insert(id.clone(), set_error("serverFail", e));
                    }
                }
            }
        }
        if let Some(destroys) = args.get("destroy").and_then(Value::as_array) {
            for id in destroys.iter().filter_map(Value::as_str) {
                match self.contact_destroy(account_id, rt, id).await {
                    Ok(()) => out.destroyed.push(json!(id)),
                    Err(e) => {
                        out.not_destroyed
                            .insert(id.to_string(), set_error("serverFail", e));
                    }
                }
            }
        }

        let new_state = self
            .pim_type_state(account_id, ChangeType::ContactCard)
            .await
            .unwrap_or_default();
        self.broadcast_state(account_id).await;
        out.into_response(account_id, &old_state, &new_state)
    }

    async fn contact_create(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        spec: &Value,
    ) -> Result<String> {
        let book_id = match spec.get("addressBookId").and_then(Value::as_str) {
            Some(b) => b.to_string(),
            None => self.ensure_default_address_book(account_id).await?,
        };
        let id = gen_id("contact");
        let uid = spec
            .get("uid")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(String::from)
            .unwrap_or_else(|| format!("{}@mailwoman.local", gen_token()));
        self.persist_contact(
            account_id,
            &book_id,
            &id,
            &uid,
            spec.clone(),
            None,
            Some(rt),
        )
        .await?;
        self.record_pim_change(account_id, ChangeType::ContactCard, &id, ChangeOp::Created)
            .await?;
        Ok(id)
    }

    async fn contact_update(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        id: &str,
        patch: &Value,
    ) -> Result<()> {
        let row = self
            .store()
            .get_contact(id)
            .await?
            .ok_or_else(|| EngineError::Protocol(format!("unknown contact {id}")))?;
        let mut json = contact_row_to_json(&row);
        if let (Some(t), Some(p)) = (json.as_object_mut(), patch.as_object()) {
            for (k, v) in p {
                t.insert(k.clone(), v.clone());
            }
        }
        self.persist_contact(
            account_id,
            &row.address_book_id,
            id,
            &row.uid,
            json,
            row.etag.clone(),
            Some(rt),
        )
        .await?;
        self.record_pim_change(account_id, ChangeType::ContactCard, id, ChangeOp::Updated)
            .await?;
        Ok(())
    }

    async fn contact_destroy(&self, account_id: &str, rt: &AccountRuntime, id: &str) -> Result<()> {
        let row = self
            .store()
            .get_contact(id)
            .await?
            .ok_or_else(|| EngineError::Protocol(format!("unknown contact {id}")))?;
        if let Some(book) = self.store().get_address_book(&row.address_book_id).await?
            && let (Some(base), Some(url)) = (rt.dav.clone(), book.carddav_url.clone())
        {
            let href = resource_href(&url, &row.uid, "vcf");
            let _ = self.carddav_delete(&base, &href, row.etag.as_deref()).await;
        }
        self.store().delete_contact(id).await?;
        self.record_pim_change(account_id, ChangeType::ContactCard, id, ChangeOp::Destroyed)
            .await?;
        Ok(())
    }

    /// Emit + persist a contact from its projection, pushing to CardDAV when the
    /// book is DAV-backed. Shared by create/update/import/merge + the sync pull.
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn persist_contact(
        &self,
        _account_id: &str,
        book_id: &str,
        id: &str,
        uid: &str,
        mut json: Value,
        prior_etag: Option<String>,
        push_rt: Option<&AccountRuntime>,
    ) -> Result<()> {
        set_str(&mut json, "id", id);
        set_str(&mut json, "addressBookId", book_id);
        set_str(&mut json, "uid", uid);

        let vcard_raw = mw_ics::emit_vcard(&json).map_err(super::events::ics_err)?;
        let mut canonical = mw_ics::parse_vcard(vcard_raw.as_bytes())
            .map_err(super::events::ics_err)?
            .into_iter()
            .next()
            .map(|p| p.json)
            .unwrap_or(json.clone());
        set_str(&mut canonical, "id", id);
        set_str(&mut canonical, "addressBookId", book_id);
        set_str(&mut canonical, "uid", uid);
        // Carry client-only flags the vCard grammar does not round-trip.
        let is_favorite = json
            .get("isFavorite")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        set_json(&mut canonical, "isFavorite", json!(is_favorite));
        if let Some(g) = json.get("groupIds").cloned() {
            set_json(&mut canonical, "groupIds", g);
        }

        let mut etag = prior_etag.clone();
        if let Some(rt) = push_rt
            && let Some(book) = self.store().get_address_book(book_id).await?
            && let (Some(base), Some(url)) = (rt.dav.clone(), book.carddav_url.clone())
        {
            let href = resource_href(&url, uid, "vcf");
            match self
                .carddav_put(&base, &href, &vcard_raw, prior_etag.as_deref())
                .await
            {
                Ok(new_etag) => etag = new_etag,
                Err(e) => tracing::warn!("CardDAV put for contact {id} failed: {e}"),
            }
        }
        set_json(
            &mut canonical,
            "etag",
            etag.clone().map(Value::String).unwrap_or(Value::Null),
        );

        let full_name = full_name_of(&canonical);
        let row = ContactRow {
            id: id.to_string(),
            address_book_id: book_id.to_string(),
            uid: uid.to_string(),
            etag,
            vcard_raw,
            json: serde_json::to_vec(&canonical).ok(),
            full_name,
            is_favorite,
            photo_blob_id: json
                .get("photoBlobId")
                .and_then(Value::as_str)
                .map(String::from),
            pgp_key: json.get("pgpKey").and_then(Value::as_str).map(String::from),
            smime_cert: json
                .get("smimeCert")
                .and_then(Value::as_str)
                .map(String::from),
        };
        self.store().upsert_contact(&row).await?;
        Ok(())
    }

    pub(crate) async fn contact_query(&self, account_id: &str, args: &Value) -> Value {
        let state = self
            .pim_type_state(account_id, ChangeType::ContactCard)
            .await
            .unwrap_or_default();
        match self.contact_query_ids(account_id, args).await {
            Ok(ids) => query_response(account_id, &state, ids),
            Err(e) => server_fail(e),
        }
    }

    async fn contact_query_ids(&self, account_id: &str, args: &Value) -> Result<Vec<String>> {
        let filter = args.get("filter").cloned().unwrap_or(Value::Null);
        let book_id = filter.get("addressBookId").and_then(Value::as_str);
        let want_favorite = filter.get("isFavorite").and_then(Value::as_bool);
        let in_group = filter.get("groupId").and_then(Value::as_str);
        let text = filter
            .get("text")
            .and_then(Value::as_str)
            .map(str::to_lowercase);

        let books = self.store().list_address_books(account_id).await?;
        let mut ids = Vec::new();
        for book in &books {
            if let Some(b) = book_id
                && book.id != b
            {
                continue;
            }
            for c in self.store().list_contacts(&book.id).await? {
                if let Some(fav) = want_favorite
                    && c.is_favorite != fav
                {
                    continue;
                }
                if let Some(q) = &text
                    && !c.full_name.to_lowercase().contains(q)
                    && !c.vcard_raw.to_lowercase().contains(q)
                {
                    continue;
                }
                if let Some(gid) = in_group
                    && !contact_in_group(&c, gid)
                {
                    continue;
                }
                ids.push(c.id);
            }
        }
        Ok(ids)
    }

    pub(crate) async fn contact_query_changes(&self, account_id: &str, args: &Value) -> Value {
        let since = args
            .get("sinceQueryState")
            .and_then(Value::as_str)
            .unwrap_or("0");
        let new_state = self
            .pim_type_state(account_id, ChangeType::ContactCard)
            .await
            .unwrap_or_default();
        let ids = self
            .contact_query_ids(account_id, args)
            .await
            .unwrap_or_default();
        let removed = self
            .build_pim_changes(account_id, ChangeType::ContactCard, since)
            .await
            .map(|c| c.destroyed)
            .unwrap_or_default();
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
            "added": added,
        })
    }

    // ── ContactCard/{import,export,merge,autocomplete} ───────────────────────

    pub(crate) async fn contact_import(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        args: &Value,
    ) -> Value {
        let blob = args.get("blob").and_then(Value::as_str).unwrap_or_default();
        let cards = if blob.contains("BEGIN:VCARD") {
            match mw_ics::parse_vcard(blob.as_bytes()) {
                Ok(v) => v.into_iter().map(|p| p.json).collect(),
                Err(e) => return server_fail(super::events::ics_err(e)),
            }
        } else {
            parse_csv_contacts(blob)
        };
        let book_id = match args.get("addressBookId").and_then(Value::as_str) {
            Some(b) => b.to_string(),
            None => match self.ensure_default_address_book(account_id).await {
                Ok(b) => b,
                Err(e) => return server_fail(e),
            },
        };
        let mut created = Vec::new();
        for card in cards {
            let uid = card
                .get("uid")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .unwrap_or_else(|| format!("{}@mailwoman.local", gen_token()));
            let id = gen_id("contact");
            match self
                .persist_contact(account_id, &book_id, &id, &uid, card, None, Some(rt))
                .await
            {
                Ok(()) => {
                    let _ = self
                        .record_pim_change(
                            account_id,
                            ChangeType::ContactCard,
                            &id,
                            ChangeOp::Created,
                        )
                        .await;
                    created.push(id);
                }
                Err(e) => tracing::warn!("contact import skipped one: {e}"),
            }
        }
        self.broadcast_state(account_id).await;
        json!({ "accountId": account_id, "imported": created.clone(), "count": created.len() })
    }

    pub(crate) async fn contact_export(&self, account_id: &str, args: &Value) -> Value {
        let ids = match wanted_ids(args) {
            Some(ids) => ids,
            None => self.all_contact_ids(account_id).await.unwrap_or_default(),
        };
        let format = args
            .get("format")
            .and_then(Value::as_str)
            .unwrap_or("vcard");
        let mut rows = Vec::new();
        for id in &ids {
            if let Ok(Some(r)) = self.store().get_contact(id).await {
                rows.push(r);
            }
        }
        let blob = if format == "csv" {
            let mut out = String::from("full_name,email\n");
            for r in &rows {
                let email = contact_row_to_json(r)
                    .get("emails")
                    .and_then(Value::as_array)
                    .and_then(|a| a.first())
                    .and_then(|e| e.get("value"))
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                out.push_str(&format!(
                    "{},{}\n",
                    csv_escape(&r.full_name),
                    csv_escape(&email)
                ));
            }
            out
        } else {
            rows.iter().map(|r| r.vcard_raw.clone()).collect::<String>()
        };
        json!({ "accountId": account_id, "blob": blob, "format": format })
    }

    /// Merge duplicate contacts into one card (§2.2, plan risk #9): produce a new
    /// merged card, then tombstone the sources (record `destroyed`) — reversible,
    /// never in-place-destructive.
    pub(crate) async fn contact_merge(&self, account_id: &str, args: &Value) -> Value {
        let ids: Vec<String> = args
            .get("ids")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();
        if ids.len() < 2 {
            return server_fail("ContactCard/merge requires at least two ids");
        }
        let mut sources = Vec::new();
        for id in &ids {
            match self.store().get_contact(id).await {
                Ok(Some(r)) => sources.push(r),
                Ok(None) => return server_fail(format!("unknown contact {id}")),
                Err(e) => return server_fail(e),
            }
        }
        let book_id = sources[0].address_book_id.clone();
        let merged = merge_cards(&sources.iter().map(contact_row_to_json).collect::<Vec<_>>());
        let new_id = gen_id("contact");
        let uid = format!("{}@mailwoman.local", gen_token());
        // No push here (a Mailwoman-native merge); an explicit re-sync propagates.
        if let Err(e) = self
            .persist_contact(account_id, &book_id, &new_id, &uid, merged, None, None)
            .await
        {
            return server_fail(e);
        }
        let _ = self
            .record_pim_change(
                account_id,
                ChangeType::ContactCard,
                &new_id,
                ChangeOp::Created,
            )
            .await;
        // Tombstone the sources.
        for id in &ids {
            let _ = self.store().delete_contact(id).await;
            let _ = self
                .record_pim_change(account_id, ChangeType::ContactCard, id, ChangeOp::Destroyed)
                .await;
        }
        self.broadcast_state(account_id).await;
        json!({ "accountId": account_id, "merged": new_id, "tombstoned": ids })
    }

    /// Prefix/substring autocomplete for Compose recipient completion (§2.2),
    /// ranked favorite-first by the store.
    pub(crate) async fn contact_autocomplete(&self, account_id: &str, args: &Value) -> Value {
        let prefix = args
            .get("prefix")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let limit = args.get("limit").and_then(Value::as_i64).unwrap_or(10);
        let rows = match self
            .store()
            .autocomplete_contacts(account_id, prefix, limit)
            .await
        {
            Ok(v) => v,
            Err(e) => return server_fail(e),
        };
        let list: Vec<Value> = rows
            .iter()
            .map(|r| {
                let email = contact_row_to_json(r)
                    .get("emails")
                    .and_then(Value::as_array)
                    .and_then(|a| a.first())
                    .and_then(|e| e.get("value"))
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                json!({
                    "id": r.id,
                    "name": r.full_name,
                    "email": email,
                    "isFavorite": r.is_favorite,
                })
            })
            .collect();
        json!({ "accountId": account_id, "list": list })
    }

    // ── ContactGroup/{get,set} ───────────────────────────────────────────────

    pub(crate) async fn contact_group_get(&self, account_id: &str, args: &Value) -> Value {
        let state = self
            .pim_type_state(account_id, ChangeType::ContactGroup)
            .await
            .unwrap_or_default();
        let books = match self.store().list_address_books(account_id).await {
            Ok(v) => v,
            Err(e) => return server_fail(e),
        };
        let wanted = wanted_ids(args);
        let mut list = Vec::new();
        let mut found = Vec::new();
        for book in &books {
            let groups = self
                .store()
                .list_contact_groups(&book.id)
                .await
                .unwrap_or_default();
            for g in &groups {
                if let Some(ids) = &wanted
                    && !ids.contains(&g.id)
                {
                    continue;
                }
                found.push(g.id.clone());
                list.push(contact_group_to_json(g));
            }
        }
        let not_found = match &wanted {
            Some(ids) => ids
                .iter()
                .filter(|id| !found.contains(id))
                .map(|id| json!(id))
                .collect(),
            None => Vec::new(),
        };
        get_response(account_id, &state, list, not_found)
    }

    pub(crate) async fn contact_group_set(&self, account_id: &str, args: &Value) -> Value {
        let old_state = self
            .pim_type_state(account_id, ChangeType::ContactGroup)
            .await
            .unwrap_or_default();
        let mut out = SetOutcome::default();

        if let Some(creates) = args.get("create").and_then(Value::as_object) {
            for (cid, spec) in creates {
                let book_id = match spec.get("addressBookId").and_then(Value::as_str) {
                    Some(b) => b.to_string(),
                    None => match self.ensure_default_address_book(account_id).await {
                        Ok(b) => b,
                        Err(e) => {
                            out.not_created
                                .insert(cid.clone(), set_error("serverFail", e));
                            continue;
                        }
                    },
                };
                let id = gen_id("group");
                let row = ContactGroupRow {
                    id: id.clone(),
                    address_book_id: book_id,
                    name: spec
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or("Group")
                        .to_string(),
                    member_ids_json: array_str(spec, "memberIds"),
                };
                match self.store().upsert_contact_group(&row).await {
                    Ok(()) => {
                        let _ = self
                            .record_pim_change(
                                account_id,
                                ChangeType::ContactGroup,
                                &id,
                                ChangeOp::Created,
                            )
                            .await;
                        out.created.insert(cid.clone(), json!({ "id": id }));
                    }
                    Err(e) => {
                        out.not_created
                            .insert(cid.clone(), set_error("serverFail", e));
                    }
                }
            }
        }
        if let Some(updates) = args.get("update").and_then(Value::as_object) {
            for (id, patch) in updates {
                match self.contact_group_update(account_id, id, patch).await {
                    Ok(()) => {
                        out.updated.insert(id.clone(), Value::Null);
                    }
                    Err(e) => {
                        out.not_updated
                            .insert(id.clone(), set_error("serverFail", e));
                    }
                }
            }
        }
        if let Some(destroys) = args.get("destroy").and_then(Value::as_array) {
            for id in destroys.iter().filter_map(Value::as_str) {
                match self.store().delete_contact_group(id).await {
                    Ok(()) => {
                        let _ = self
                            .record_pim_change(
                                account_id,
                                ChangeType::ContactGroup,
                                id,
                                ChangeOp::Destroyed,
                            )
                            .await;
                        out.destroyed.push(json!(id));
                    }
                    Err(e) => {
                        out.not_destroyed
                            .insert(id.to_string(), set_error("serverFail", e));
                    }
                }
            }
        }

        let new_state = self
            .pim_type_state(account_id, ChangeType::ContactGroup)
            .await
            .unwrap_or_default();
        self.broadcast_state(account_id).await;
        out.into_response(account_id, &old_state, &new_state)
    }

    async fn contact_group_update(&self, account_id: &str, id: &str, patch: &Value) -> Result<()> {
        let mut row = self
            .store()
            .get_contact_group(id)
            .await?
            .ok_or_else(|| EngineError::Protocol(format!("unknown contact group {id}")))?;
        if let Some(v) = patch.get("name").and_then(Value::as_str) {
            row.name = v.to_string();
        }
        if patch.get("memberIds").is_some() {
            row.member_ids_json = array_str(patch, "memberIds");
        }
        self.store().upsert_contact_group(&row).await?;
        self.record_pim_change(account_id, ChangeType::ContactGroup, id, ChangeOp::Updated)
            .await?;
        Ok(())
    }

    // ── helpers ──────────────────────────────────────────────────────────────

    pub(crate) async fn all_contact_ids(&self, account_id: &str) -> Result<Vec<String>> {
        let books = self.store().list_address_books(account_id).await?;
        let mut ids = Vec::new();
        for book in &books {
            for c in self.store().list_contacts(&book.id).await? {
                ids.push(c.id);
            }
        }
        Ok(ids)
    }

    /// Ensure a default address book exists, returning its id.
    pub(crate) async fn ensure_default_address_book(&self, account_id: &str) -> Result<String> {
        let books = self.store().list_address_books(account_id).await?;
        if let Some(b) = books
            .iter()
            .find(|b| b.is_default)
            .or_else(|| books.first())
        {
            return Ok(b.id.clone());
        }
        let id = gen_id("ab");
        let row = AddressBookRow {
            id: id.clone(),
            account_id: account_id.to_string(),
            name: "Contacts".to_string(),
            is_default: true,
            carddav_url: None,
            sync_token: None,
            ctag: None,
        };
        self.store().upsert_address_book(&row).await?;
        self.record_pim_change(account_id, ChangeType::AddressBook, &id, ChangeOp::Created)
            .await?;
        Ok(id)
    }
}

// ── free helpers ─────────────────────────────────────────────────────────────

fn set_str(json: &mut Value, key: &str, val: &str) {
    if let Some(obj) = json.as_object_mut() {
        obj.insert(key.to_string(), Value::String(val.to_string()));
    }
}

fn set_json(json: &mut Value, key: &str, val: Value) {
    if let Some(obj) = json.as_object_mut() {
        obj.insert(key.to_string(), val);
    }
}

fn array_str(spec: &Value, key: &str) -> String {
    spec.get(key)
        .filter(|v| v.is_array())
        .map(|v| v.to_string())
        .unwrap_or_else(|| "[]".to_string())
}

/// The display name for a projection: `name.full`, else `given surname`.
fn full_name_of(json: &Value) -> String {
    let name = json.get("name").cloned().unwrap_or(Value::Null);
    let full = name
        .get("full")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if !full.is_empty() {
        return full;
    }
    let given = name.get("given").and_then(Value::as_str).unwrap_or("");
    let surname = name.get("surname").and_then(Value::as_str).unwrap_or("");
    format!("{given} {surname}").trim().to_string()
}

fn contact_in_group(c: &ContactRow, group_id: &str) -> bool {
    contact_row_to_json(c)
        .get("groupIds")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).any(|g| g == group_id))
        .unwrap_or(false)
}

/// The §2.1 `ContactCard` JSON for a row (stored projection, patched with the
/// row identity + the engine-owned favorite/etag/key columns).
fn contact_row_to_json(row: &ContactRow) -> Value {
    let mut json = row
        .json
        .as_ref()
        .and_then(|b| serde_json::from_slice::<Value>(b).ok())
        .or_else(|| {
            mw_ics::parse_vcard(row.vcard_raw.as_bytes())
                .ok()
                .and_then(|v| v.into_iter().next())
                .map(|p| p.json)
        })
        .unwrap_or_else(|| json!({}));
    set_str(&mut json, "id", &row.id);
    set_str(&mut json, "addressBookId", &row.address_book_id);
    set_str(&mut json, "uid", &row.uid);
    set_json(&mut json, "isFavorite", json!(row.is_favorite));
    set_json(
        &mut json,
        "pgpKey",
        row.pgp_key
            .clone()
            .map(Value::String)
            .unwrap_or(Value::Null),
    );
    set_json(
        &mut json,
        "smimeCert",
        row.smime_cert
            .clone()
            .map(Value::String)
            .unwrap_or(Value::Null),
    );
    set_json(
        &mut json,
        "etag",
        row.etag.clone().map(Value::String).unwrap_or(Value::Null),
    );
    json
}

fn address_book_to_json(row: &AddressBookRow) -> Value {
    json!({
        "id": row.id,
        "name": row.name,
        "isDefault": row.is_default,
        "carddavUrl": row.carddav_url,
        "syncToken": row.sync_token,
    })
}

fn contact_group_to_json(row: &ContactGroupRow) -> Value {
    let members: Value = serde_json::from_str(&row.member_ids_json).unwrap_or_else(|_| json!([]));
    json!({
        "id": row.id,
        "addressBookId": row.address_book_id,
        "name": row.name,
        "memberIds": members,
    })
}

/// Merge N contact projections into one (§2.2): first non-empty name wins;
/// emails / phones / organizations / titles are unioned by value; `isFavorite`
/// is true if any source is favourited.
fn merge_cards(cards: &[Value]) -> Value {
    let mut out = cards.first().cloned().unwrap_or_else(|| json!({}));
    let obj = out.as_object_mut().expect("merged card object");
    // Union list fields across all sources.
    for card in cards {
        union_by_value(obj, card, "emails", "value");
        union_by_value(obj, card, "phones", "value");
        union_by_value(obj, card, "onlineServices", "value");
        union_scalars(obj, card, "organizations");
        union_scalars(obj, card, "titles");
        union_scalars(obj, card, "nicknames");
        if card.get("isFavorite").and_then(Value::as_bool) == Some(true) {
            obj.insert("isFavorite".into(), json!(true));
        }
    }
    // A merged card is a fresh identity; drop stale per-source keys.
    obj.remove("etag");
    out
}

fn union_by_value(
    target: &mut serde_json::Map<String, Value>,
    src: &Value,
    key: &str,
    dedupe_on: &str,
) {
    let mut merged: Vec<Value> = target
        .get(key)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut seen: Vec<String> = merged
        .iter()
        .filter_map(|e| e.get(dedupe_on).and_then(Value::as_str).map(String::from))
        .collect();
    for item in src.get(key).and_then(Value::as_array).into_iter().flatten() {
        let v = item
            .get(dedupe_on)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if !v.is_empty() && !seen.contains(&v) {
            seen.push(v);
            merged.push(item.clone());
        }
    }
    target.insert(key.to_string(), Value::Array(merged));
}

fn union_scalars(target: &mut serde_json::Map<String, Value>, src: &Value, key: &str) {
    let mut merged: Vec<Value> = target
        .get(key)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut seen: Vec<String> = merged
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
    for item in src.get(key).and_then(Value::as_array).into_iter().flatten() {
        if let Some(v) = item.as_str()
            && !seen.contains(&v.to_string())
        {
            seen.push(v.to_string());
            merged.push(item.clone());
        }
    }
    target.insert(key.to_string(), Value::Array(merged));
}

/// Parse a minimal CSV (`full_name,email` header-detected) into projections.
fn parse_csv_contacts(csv: &str) -> Vec<Value> {
    let mut lines = csv.lines();
    let Some(header) = lines.next() else {
        return Vec::new();
    };
    let cols: Vec<String> = header.split(',').map(|c| c.trim().to_lowercase()).collect();
    let name_idx = cols
        .iter()
        .position(|c| c == "full_name" || c == "name" || c == "fn");
    let email_idx = cols.iter().position(|c| c == "email" || c == "e-mail");
    let mut out = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split(',').map(str::trim).collect();
        let name = name_idx.and_then(|i| fields.get(i)).copied().unwrap_or("");
        let email = email_idx.and_then(|i| fields.get(i)).copied().unwrap_or("");
        let mut emails = Vec::new();
        if !email.is_empty() {
            emails.push(json!({ "context": "", "value": email, "pref": 0 }));
        }
        out.push(json!({
            "kind": "individual",
            "name": { "full": name, "given": "", "surname": "", "prefix": "", "suffix": "" },
            "emails": emails,
        }));
    }
    out
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
