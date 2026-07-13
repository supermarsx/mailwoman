//! Keyring family (`CryptoKey/*`, frozen §2.2) — server-side, PUBLIC keys +
//! opaque client-encrypted backup ONLY. `set` uploads the PUBLIC key/cert plus an
//! opaque `encryptedPrivateBackup` (the server NEVER decrypts it, plan §1.2 /
//! risk #4). `lookup` orchestrates WKD/VKS (with consent) + cert-harvest from
//! stored signed mail; `setTrust` is the TOFU verify/revoke.
//!
//! Also hosts the `MailRule/*` surface (frozen §2.2) — the block/silence/ignore
//! materialization over the existing engine `rules.rs` (`get_rules`/`set_rules`).

use serde_json::{Map, Value, json};

use crate::account::AccountRuntime;
use crate::change::{ChangeOp, ChangeType};
use crate::engine::Engine;
use crate::security::convert::{
    initial_history, key_dto_to_row, key_row_to_dto, mail_rule_to_rule, rule_to_mail_rule,
};
use crate::security::types::{CryptoKey, MailRule};

use super::{gen_id, server_fail};

impl Engine {
    /// `CryptoKey/get {ids?}` → `{accountId,state,list:[CryptoKey],notFound}`.
    pub(crate) async fn crypto_key_get(&self, account_id: &str, args: &Value) -> Value {
        let wanted: Option<Vec<String>> = args.get("ids").and_then(Value::as_array).map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(String::from)
                .collect()
        });
        let rows = match self.store().list_crypto_keys(account_id).await {
            Ok(v) => v,
            Err(e) => return server_fail(e),
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
            list.push(serde_json::to_value(key_row_to_dto(row)).unwrap_or(Value::Null));
        }
        let not_found: Vec<Value> = match &wanted {
            Some(ids) => ids
                .iter()
                .filter(|id| !found.contains(id))
                .map(|id| json!(id))
                .collect(),
            None => Vec::new(),
        };
        let state = self
            .crypto_type_state(account_id, ChangeType::CryptoKey)
            .await
            .unwrap_or_default();
        super::get_response(account_id, &state, list, not_found)
    }

    /// `CryptoKey/set {create,update,destroy}` — persist the PUBLIC key/cert +
    /// opaque `encryptedPrivateBackup` (never plaintext private material, §1.2).
    pub(crate) async fn crypto_key_set(&self, account_id: &str, args: &Value) -> Value {
        let old_state = self
            .crypto_type_state(account_id, ChangeType::CryptoKey)
            .await
            .unwrap_or_default();
        let mut created = Map::new();
        let mut not_created = Map::new();
        let mut updated = Map::new();
        let mut not_updated = Map::new();
        let mut destroyed = Vec::new();
        let mut changed = false;

        if let Some(creates) = args.get("create").and_then(Value::as_object) {
            for (client_id, spec) in creates {
                match self.crypto_key_create_one(account_id, spec).await {
                    Ok(id) => {
                        created.insert(client_id.clone(), json!({ "id": id }));
                        changed = true;
                    }
                    Err(e) => {
                        not_created.insert(client_id.clone(), server_fail(e));
                    }
                }
            }
        }

        if let Some(updates) = args.get("update").and_then(Value::as_object) {
            for (id, patch) in updates {
                match self.crypto_key_update_one(account_id, id, patch).await {
                    Ok(()) => {
                        updated.insert(id.clone(), Value::Null);
                        changed = true;
                    }
                    Err(e) => {
                        not_updated.insert(id.clone(), server_fail(e));
                    }
                }
            }
        }

        if let Some(destroys) = args.get("destroy").and_then(Value::as_array) {
            for id in destroys.iter().filter_map(Value::as_str) {
                if self.store().delete_crypto_key(account_id, id).await.is_ok() {
                    let _ = self
                        .record_crypto_change(
                            account_id,
                            ChangeType::CryptoKey,
                            id,
                            ChangeOp::Destroyed,
                        )
                        .await;
                    destroyed.push(json!(id));
                    changed = true;
                }
            }
        }

        if changed {
            self.broadcast_state(account_id).await;
        }
        let new_state = self
            .crypto_type_state(account_id, ChangeType::CryptoKey)
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
        if !not_created.is_empty() {
            resp["notCreated"] = Value::Object(not_created);
        }
        if !not_updated.is_empty() {
            resp["notUpdated"] = Value::Object(not_updated);
        }
        resp
    }

    /// Persist one uploaded key (server-assigned id, TOFU association recorded).
    async fn crypto_key_create_one(
        &self,
        account_id: &str,
        spec: &Value,
    ) -> Result<String, String> {
        let mut dto: CryptoKey =
            serde_json::from_value(spec.clone()).map_err(|e| format!("invalid CryptoKey: {e}"))?;
        if dto.id.is_empty() {
            dto.id = gen_id("key");
        }
        let now = chrono::Utc::now().to_rfc3339();
        if dto.key_history.is_empty() && !dto.fingerprint.is_empty() {
            dto.key_history = initial_history(&dto.fingerprint, &now);
        }
        let row = key_dto_to_row(account_id, &dto);
        self.store()
            .upsert_crypto_key(&row)
            .await
            .map_err(|e| e.to_string())?;
        for addr in &dto.addresses {
            let _ = self
                .store()
                .add_key_association(&mw_store::KeyAssociationRow {
                    account_id: account_id.to_string(),
                    address: addr.clone(),
                    crypto_key_id: dto.id.clone(),
                    seen_at: now.clone(),
                })
                .await;
        }
        let _ = self
            .record_crypto_change(
                account_id,
                ChangeType::CryptoKey,
                &dto.id,
                ChangeOp::Created,
            )
            .await;
        Ok(dto.id)
    }

    /// Apply a partial patch to a stored key (trust / addresses / backup).
    async fn crypto_key_update_one(
        &self,
        account_id: &str,
        id: &str,
        patch: &Value,
    ) -> Result<(), String> {
        let Some(row) = self
            .store()
            .get_crypto_key(account_id, id)
            .await
            .map_err(|e| e.to_string())?
        else {
            return Err(format!("unknown key {id}"));
        };
        let mut dto = key_row_to_dto(&row);
        if let Some(t) = patch.get("trust").and_then(Value::as_str) {
            dto.trust = t.to_string();
        }
        if let Some(b) = patch.get("encryptedPrivateBackup").and_then(Value::as_str) {
            dto.encrypted_private_backup = Some(b.to_string());
        }
        if let Some(addrs) = patch.get("addresses").and_then(Value::as_array) {
            dto.addresses = addrs
                .iter()
                .filter_map(Value::as_str)
                .map(String::from)
                .collect();
        }
        self.store()
            .upsert_crypto_key(&key_dto_to_row(account_id, &dto))
            .await
            .map_err(|e| e.to_string())?;
        let _ = self
            .record_crypto_change(account_id, ChangeType::CryptoKey, id, ChangeOp::Updated)
            .await;
        Ok(())
    }

    /// `CryptoKey/query` → the account's key ids (own + contact), newest first.
    pub(crate) async fn crypto_key_query(&self, account_id: &str, _args: &Value) -> Value {
        let rows = match self.store().list_crypto_keys(account_id).await {
            Ok(v) => v,
            Err(e) => return server_fail(e),
        };
        let ids: Vec<Value> = rows.iter().map(|r| json!(r.id)).collect();
        let state = self
            .crypto_type_state(account_id, ChangeType::CryptoKey)
            .await
            .unwrap_or_default();
        json!({
            "accountId": account_id,
            "queryState": state,
            "ids": ids,
            "total": rows.len(),
            "position": 0,
            "canCalculateChanges": false,
        })
    }

    /// `CryptoKey/lookup {address, sources}` → `{accountId, list, notFound}`.
    /// Orchestrates (in order, best-effort): locally-stored/harvested keys for the
    /// address, then WKD fetch when `"wkd"` is requested and the host is reachable.
    /// VKS/autocrypt sources fall through to the stored set (harvest is recorded at
    /// ingest). Never surfaces private material.
    pub(crate) async fn crypto_key_lookup(
        &self,
        account_id: &str,
        _rt: &AccountRuntime,
        args: &Value,
    ) -> Value {
        let address = args
            .get("address")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_lowercase();
        let sources: Vec<String> = args
            .get("sources")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_else(|| vec!["harvested".into(), "wkd".into()]);

        let mut list: Vec<Value> = Vec::new();
        // 1. Locally stored (harvested / imported / previously looked-up) keys.
        if let Ok(rows) = self.store().list_crypto_keys(account_id).await {
            for row in &rows {
                let addrs: Vec<String> =
                    serde_json::from_str(&row.addresses_json).unwrap_or_default();
                if addrs.iter().any(|a| a.to_lowercase() == address) {
                    list.push(serde_json::to_value(key_row_to_dto(row)).unwrap_or(Value::Null));
                }
            }
        }
        // 2. WKD fetch (network, best-effort; ignored on any failure / offline CI).
        if list.is_empty()
            && sources.iter().any(|s| s == "wkd")
            && !address.is_empty()
            && let Ok(mut key) = mw_crypto::pgp::wkd_fetch(&address).await
        {
            key.id = gen_id("key");
            key.source = "wkd".into();
            let row = key_dto_to_row(account_id, &key);
            let _ = self.store().upsert_crypto_key(&row).await;
            let _ = self
                .record_crypto_change(
                    account_id,
                    ChangeType::CryptoKey,
                    &key.id,
                    ChangeOp::Created,
                )
                .await;
            list.push(serde_json::to_value(&key).unwrap_or(Value::Null));
        }
        let not_found = if list.is_empty() {
            vec![json!(address)]
        } else {
            Vec::new()
        };
        json!({ "accountId": account_id, "list": list, "notFound": not_found })
    }

    /// `CryptoKey/setTrust {id, trust}` — TOFU verify/revoke. Sets `trust` and,
    /// when moving to `"verified"`, stamps `verifiedAt`.
    pub(crate) async fn crypto_key_set_trust(&self, account_id: &str, args: &Value) -> Value {
        let id = args.get("id").and_then(Value::as_str).unwrap_or_default();
        let trust = args
            .get("trust")
            .and_then(Value::as_str)
            .unwrap_or("unverified");
        let mut updated = Map::new();
        match self.store().get_crypto_key(account_id, id).await {
            Ok(Some(row)) => {
                let mut dto = key_row_to_dto(&row);
                dto.trust = trust.to_string();
                if trust == "verified" {
                    dto.verified_at = Some(chrono::Utc::now().to_rfc3339());
                }
                if self
                    .store()
                    .upsert_crypto_key(&key_dto_to_row(account_id, &dto))
                    .await
                    .is_ok()
                {
                    let _ = self
                        .record_crypto_change(
                            account_id,
                            ChangeType::CryptoKey,
                            id,
                            ChangeOp::Updated,
                        )
                        .await;
                    self.broadcast_state(account_id).await;
                    updated.insert(id.to_string(), Value::Null);
                }
            }
            Ok(None) => {}
            Err(e) => return server_fail(e),
        }
        json!({ "accountId": account_id, "updated": updated })
    }

    // ── MailRule/* (surface over rules.rs, frozen §2.2) ─────────────────────

    /// `MailRule/get {ids?}` → the block/silence/ignore surface over `rules.rs`.
    pub(crate) async fn mail_rule_get(&self, account_id: &str, args: &Value) -> Value {
        let wanted: Option<Vec<String>> = args.get("ids").and_then(Value::as_array).map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(String::from)
                .collect()
        });
        let rules = self.get_rules(account_id).await.unwrap_or_default();
        let mut list = Vec::new();
        let mut found = Vec::new();
        for rule in &rules {
            if let Some(ids) = &wanted
                && !ids.contains(&rule.id)
            {
                continue;
            }
            found.push(rule.id.clone());
            list.push(serde_json::to_value(rule_to_mail_rule(rule)).unwrap_or(Value::Null));
        }
        let not_found: Vec<Value> = match &wanted {
            Some(ids) => ids
                .iter()
                .filter(|id| !found.contains(id))
                .map(|id| json!(id))
                .collect(),
            None => Vec::new(),
        };
        let state = self
            .crypto_type_state(account_id, ChangeType::MailRule)
            .await
            .unwrap_or_default();
        super::get_response(account_id, &state, list, not_found)
    }

    /// `MailRule/set {create,update,destroy}` — mutate the stored Sieve rules.
    /// Engine-applied by default; e7 uploads via ManageSieve where advertised.
    pub(crate) async fn mail_rule_set(
        &self,
        account_id: &str,
        rt: &AccountRuntime,
        args: &Value,
    ) -> Value {
        let old_state = self
            .crypto_type_state(account_id, ChangeType::MailRule)
            .await
            .unwrap_or_default();
        let mut rules = self.get_rules(account_id).await.unwrap_or_default();
        let mut created = Map::new();
        let mut updated = Map::new();
        let mut destroyed = Vec::new();
        let mut changed_ids: Vec<(String, ChangeOp)> = Vec::new();

        if let Some(creates) = args.get("create").and_then(Value::as_object) {
            for (client_id, spec) in creates {
                if let Ok(mut mr) = serde_json::from_value::<MailRule>(inject_defaults(spec)) {
                    if mr.id.is_empty() {
                        mr.id = gen_id("mr");
                    }
                    let id = mr.id.clone();
                    rules.push(mail_rule_to_rule(&mr));
                    created.insert(client_id.clone(), json!({ "id": id }));
                    changed_ids.push((id, ChangeOp::Created));
                }
            }
        }
        if let Some(updates) = args.get("update").and_then(Value::as_object) {
            for (id, patch) in updates {
                if let Some(pos) = rules.iter().position(|r| &r.id == id) {
                    let mut mr = rule_to_mail_rule(&rules[pos]);
                    apply_mail_rule_patch(&mut mr, patch);
                    rules[pos] = mail_rule_to_rule(&mr);
                    updated.insert(id.clone(), Value::Null);
                    changed_ids.push((id.clone(), ChangeOp::Updated));
                }
            }
        }
        if let Some(destroys) = args.get("destroy").and_then(Value::as_array) {
            for id in destroys.iter().filter_map(Value::as_str) {
                if let Some(pos) = rules.iter().position(|r| r.id == id) {
                    rules.remove(pos);
                    destroyed.push(json!(id));
                    changed_ids.push((id.to_string(), ChangeOp::Destroyed));
                }
            }
        }

        if !changed_ids.is_empty() {
            if let Err(e) = self.set_rules(account_id, &rules).await {
                return server_fail(e);
            }
            // Best-effort ManageSieve upload where advertised (e7 owns the wiring).
            let _ = self.upload_sieve_if_supported(rt, &rules).await;
            for (id, op) in &changed_ids {
                let _ = self
                    .record_crypto_change(account_id, ChangeType::MailRule, id, *op)
                    .await;
            }
            self.broadcast_state(account_id).await;
        }
        let new_state = self
            .crypto_type_state(account_id, ChangeType::MailRule)
            .await
            .unwrap_or_default();
        json!({
            "accountId": account_id,
            "oldState": old_state,
            "newState": new_state,
            "created": created,
            "updated": updated,
            "destroyed": destroyed,
        })
    }

    /// Upload the rule set as Sieve where the backend advertises ManageSieve.
    /// A no-op stub in e6 (engine-applied is the always-green path, plan §1.9);
    /// e7 fills the ManageSieve transport. Returns `Ok(false)` when not uploaded.
    pub(crate) async fn upload_sieve_if_supported(
        &self,
        _rt: &AccountRuntime,
        _rules: &[mw_sieve::Rule],
    ) -> Result<bool, mw_store::StoreError> {
        Ok(false)
    }
}

/// Merge frozen defaults into a partial `MailRule` create spec so a minimal
/// `{conditions,actions}` payload deserializes (matchAll/enabled/runsAt/name).
fn inject_defaults(spec: &Value) -> Value {
    let mut obj = spec.as_object().cloned().unwrap_or_default();
    obj.entry("id").or_insert(json!(""));
    obj.entry("name").or_insert(json!("Rule"));
    obj.entry("matchAll").or_insert(json!(false));
    obj.entry("conditions").or_insert(json!([]));
    obj.entry("actions").or_insert(json!([]));
    obj.entry("enabled").or_insert(json!(true));
    obj.entry("runsAt").or_insert(json!("engine"));
    Value::Object(obj)
}

/// Apply a JMAP patch object to a `MailRule` (name/enabled/conditions/actions).
fn apply_mail_rule_patch(mr: &mut MailRule, patch: &Value) {
    if let Some(name) = patch.get("name").and_then(Value::as_str) {
        mr.name = name.to_string();
    }
    if let Some(enabled) = patch.get("enabled").and_then(Value::as_bool) {
        mr.enabled = enabled;
    }
    if let Some(conds) = patch.get("conditions")
        && let Ok(c) = serde_json::from_value(conds.clone())
    {
        mr.conditions = c;
    }
    if let Some(acts) = patch.get("actions")
        && let Ok(a) = serde_json::from_value(acts.clone())
    {
        mr.actions = a;
    }
}
