//! RFC 9661 script management. The server supplies trusted policy and transport.
use crate::Engine;
use mw_sieve::policy::{EditKind, Policy, ViolationKind};
use mw_sieve::{Checked, ManageSieveClient};
use mw_store::{SieveReplaceResult, SieveScriptRow};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
#[path = "sieve_managed.rs"]
mod managed;
#[path = "sieve_vacation.rs"]
mod vacation;

pub fn is_managed_method(name: &str) -> bool {
    matches!(
        name,
        "MailRule/get" | "MailRule/set" | "VacationResponse/get" | "VacationResponse/set"
    )
}

pub const CAPABILITY: &str = "urn:ietf:params:jmap:sieve";
pub fn is_sieve_method(name: &str) -> bool {
    matches!(
        name,
        "SieveScript/get" | "SieveScript/set" | "SieveScript/query" | "SieveScript/validate"
    )
}
/// Only construct from authenticated account configuration, never request JSON.
pub struct SieveContext {
    /// Set only after a successful script commit adds forwarding authority.
    pub forwarding_added: bool,
    pub policy: Policy,
    pub client: Option<ManageSieveClient>,
    unavailable: bool,
    edit_kind: EditKind,
}
impl SieveContext {
    pub fn new(policy: Policy, client: Option<ManageSieveClient>) -> Self {
        Self {
            forwarding_added: false,
            policy,
            client,
            unavailable: false,
            edit_kind: EditKind::Raw,
        }
    }
    fn extensions(&self) -> std::collections::BTreeSet<String> {
        self.client
            .as_ref()
            .map(|c| c.capabilities().sieve.clone())
            .unwrap_or_else(|| {
                mw_sieve::local::EXTENSIONS
                    .iter()
                    .map(|s| s.to_string())
                    .collect()
            })
            .into_iter()
            .filter(|e| {
                mw_sieve::extensions::KNOWN_CAPABILITIES.contains(&e.as_str())
                    && e != "include"
                    && (self.policy.vacation_allowed
                        || !matches!(e.as_str(), "vacation" | "vacation-seconds"))
                    && self
                        .policy
                        .allowed_extensions
                        .as_ref()
                        .is_none_or(|a| a.contains(e))
            })
            .collect()
    }
    pub fn account_capability(&self) -> Value {
        let extensions = self.extensions();
        json!({"maxSizeScriptName":512,"maxSizeScript":self.policy.max_script_bytes,
            "maxNumberScripts":self.policy.max_scripts,"maxNumberRedirects":self.policy.max_redirects,
            "sieveExtensions":extensions,"notificationMethods":null,"externalLists":null,
            "urn:mailwoman:policy":{"rawEdit":self.policy.raw_edit,"forwardExternal":self.policy.forward_external,
                "ownDomains":self.policy.own_domains,"runsAt":if self.client.is_some(){"server-sieve"}else{"engine"}}})
    }
}
fn err(kind: &str) -> Value {
    json!({"type":kind})
}
fn state(rows: &[SieveScriptRow]) -> String {
    let mut sorted: Vec<_> = rows.iter().collect();
    sorted.sort_by_key(|s| s.id());
    let mut h = Sha256::new();
    for s in sorted {
        for v in [
            s.id(),
            s.name.clone(),
            s.content.clone(),
            s.active.to_string(),
        ] {
            h.update((v.len() as u64).to_be_bytes());
            h.update(v.as_bytes());
        }
    }
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}
fn new_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let mut h = Sha256::new();
    h.update(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
            .to_be_bytes(),
    );
    h.update(SEQ.fetch_add(1, Ordering::Relaxed).to_be_bytes());
    h.update(std::process::id().to_be_bytes());
    format!(
        "s{}",
        h.finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    )
}
fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 512
        && !name
            .chars()
            .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
}
fn account_lock(account: &str) -> std::sync::Arc<tokio::sync::Mutex<()>> {
    use std::sync::{Arc, Mutex, OnceLock, Weak};
    static LOCKS: OnceLock<Mutex<HashMap<String, Weak<tokio::sync::Mutex<()>>>>> = OnceLock::new();
    let mut locks = LOCKS
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    locks.retain(|_, v| v.strong_count() > 0);
    if let Some(lock) = locks.get(account).and_then(Weak::upgrade) {
        return lock;
    }
    let lock = Arc::new(tokio::sync::Mutex::new(()));
    locks.insert(account.into(), Arc::downgrade(&lock));
    lock
}
impl Engine {
    pub async fn handle_sieve_jmap(
        &self,
        account: &str,
        method: &str,
        args: &Value,
        context: Option<&mut SieveContext>,
        created_ids: &mut HashMap<String, String>,
    ) -> Value {
        if args.get("accountId").and_then(Value::as_str) != Some(account) {
            return err("accountNotFound");
        }
        let Some(ctx) = context else {
            return err("unknownMethod");
        };
        if ctx.unavailable {
            return err("serverUnavailable");
        }
        if !is_sieve_method(method) {
            return err("unknownMethod");
        }
        let lock = account_lock(account);
        let _guard = lock.lock().await;
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            self.sieve_call(account, method, args, ctx, created_ids),
        )
        .await;
        let result = result.unwrap_or_else(|_| err("serverUnavailable"));
        if result["type"] == "serverUnavailable" {
            ctx.unavailable = true;
        }
        result
    }
    async fn sieve_source(&self, account: &str, blob: &Value) -> Result<String, Value> {
        let id = blob.as_str().ok_or_else(|| err("invalidProperties"))?;
        let data = self
            .fetch_blob(account, id)
            .await
            .map_err(|_| err("serverFail"))?
            .ok_or_else(|| err("blobNotFound"))?;
        let source = String::from_utf8(data.bytes).map_err(|_| err("invalidSieve"))?;
        if source.is_empty() {
            return Err(err("invalidSieve"));
        }
        Ok(source)
    }
    async fn sieve_validate_source(
        &self,
        source: &str,
        ctx: &mut SieveContext,
    ) -> Result<(), Value> {
        let mut policy = ctx.policy.clone();
        policy.advertised = Some(ctx.extensions());
        if let Err(errors) = mw_sieve::policy::enforce(source, &policy, ctx.edit_kind) {
            let first = errors
                .iter()
                .find(|v| {
                    matches!(
                        v.kind,
                        ViolationKind::RawEditNotAllowed
                            | ViolationKind::ExternalTarget
                            | ViolationKind::VacationNotAllowed
                    )
                })
                .unwrap_or(&errors[0]);
            let kind = match first.kind {
                ViolationKind::TooLarge => "tooLarge",
                ViolationKind::RawEditNotAllowed
                | ViolationKind::ExternalTarget
                | ViolationKind::VacationNotAllowed => "forbidden",
                _ => "invalidSieve",
            };
            // Parser diagnostics can contain script literals. Only expose positions/codes.
            return Err(
                json!({"type":kind,"description":format!("Sieve validation failed at line {}, column {} ({})",first.line,first.column,first.kind.code())}),
            );
        }
        if let Some(client) = ctx.client.as_mut() {
            match client
                .check_script(source)
                .await
                .map_err(|_| err("serverUnavailable"))?
            {
                Checked::Valid { .. } => Ok(()),
                Checked::Invalid { problems, .. } => Err(
                    json!({"type":"invalidSieve","description":format!("Upstream rejected script at line {}, column {}",problems.first().and_then(|p|p.line).unwrap_or(1),problems.first().and_then(|p|p.column).unwrap_or(1))}),
                ),
            }
        } else {
            mw_sieve::local::validate(source).map(|_|()).map_err(|e|json!({"type":"invalidSieve","description":format!("Unsupported local script: {e}")}))
        }
    }
    async fn sieve_snapshot(
        &self,
        account: &str,
        ctx: &mut SieveContext,
    ) -> Result<Vec<SieveScriptRow>, Value> {
        let old = self
            .store()
            .list_sieve_scripts(account)
            .await
            .map_err(|_| err("serverFail"))?;
        let Some(client) = ctx.client.as_mut() else {
            return Ok(old);
        };
        let remote = client
            .list_scripts()
            .await
            .map_err(|_| err("serverUnavailable"))?;
        let mut rows = Vec::new();
        for info in remote {
            let source = client
                .get_script(&info.name)
                .await
                .map_err(|_| err("serverUnavailable"))?;
            let prior = old.iter().find(|s| s.name == info.name);
            rows.push(SieveScriptRow {
                name: info.name,
                content: source.clone(),
                active: info.active,
                state_json: {
                    let mut meta = prior
                        .filter(|s| s.content == source)
                        .and_then(|s| serde_json::from_str::<Value>(&s.state_json).ok())
                        .unwrap_or_else(|| json!({}));
                    meta["_jmapId"] = prior.map(SieveScriptRow::id).unwrap_or_else(new_id).into();
                    meta["_remote"] = true.into();
                    meta.to_string()
                },
                updated_at: 0,
            });
        }
        rows.sort_by(|a, b| a.name.cmp(&b.name));
        match self
            .store()
            .replace_sieve_scripts(account, &old, &rows, None)
            .await
        {
            Ok(SieveReplaceResult::Applied) => Ok(rows),
            _ => Err(err("serverFail")),
        }
    }
    async fn sieve_call(
        &self,
        account: &str,
        method: &str,
        args: &Value,
        ctx: &mut SieveContext,
        created_ids: &mut HashMap<String, String>,
    ) -> Value {
        if method == "SieveScript/validate" {
            let source = match self.sieve_source(account, &args["blobId"]).await {
                Ok(s) => s,
                Err(e) => return e,
            };
            return json!({"accountId":account,"error":self.sieve_validate_source(&source,ctx).await.err()});
        }
        let rows = match self.sieve_snapshot(account, ctx).await {
            Ok(r) => r,
            Err(e) => return e,
        };
        match method {
            "SieveScript/get" => self.sieve_get(account, args, &rows).await,
            "SieveScript/query" => sieve_query(account, args, &rows),
            "SieveScript/set" => self.sieve_set(account, args, rows, ctx, created_ids).await,
            _ => err("unknownMethod"),
        }
    }
    async fn sieve_get(&self, account: &str, args: &Value, rows: &[SieveScriptRow]) -> Value {
        let properties = match args.get("properties").filter(|v| !v.is_null()) {
            None => vec!["id", "name", "blobId", "isActive"],
            Some(Value::Array(a))
                if a.iter()
                    .all(|v| matches!(v.as_str(), Some("id" | "name" | "blobId" | "isActive"))) =>
            {
                a.iter().filter_map(Value::as_str).collect()
            }
            _ => return err("invalidArguments"),
        };
        let ids: Vec<String> = match args.get("ids").filter(|v| !v.is_null()) {
            None => rows.iter().map(SieveScriptRow::id).collect(),
            Some(Value::Array(a)) if a.iter().all(Value::is_string) => a
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect(),
            _ => return err("invalidArguments"),
        };
        let mut cached = rows.to_vec();
        let mut list = Vec::new();
        let mut missing = Vec::new();
        for id in ids {
            let Some(row) = rows.iter().find(|s| s.id() == id) else {
                missing.push(id);
                continue;
            };
            let mut object = Map::new();
            object.insert("id".into(), id.into());
            for property in &properties {
                match *property {
                    "name" => {
                        object.insert("name".into(), row.name.clone().into());
                    }
                    "isActive" => {
                        object.insert("isActive".into(), row.active.into());
                    }
                    "blobId" => {
                        let mut metadata: Value =
                            serde_json::from_str(&row.state_json).unwrap_or_else(|_| json!({}));
                        let existing = metadata["_blobId"].as_str();
                        let blob = if let Some(id) = existing {
                            match self.fetch_blob(account, id).await {
                                Ok(Some(_)) => id.to_string(),
                                _ => match self
                                    .store_upload(
                                        account,
                                        "application/sieve",
                                        row.content.as_bytes(),
                                    )
                                    .await
                                {
                                    Ok(b) => b,
                                    Err(_) => return err("serverFail"),
                                },
                            }
                        } else {
                            match self
                                .store_upload(account, "application/sieve", row.content.as_bytes())
                                .await
                            {
                                Ok(b) => b,
                                Err(_) => return err("serverFail"),
                            }
                        };
                        metadata["_blobId"] = blob.clone().into();
                        if let Some(row) = cached.iter_mut().find(|s| s.id() == row.id()) {
                            row.state_json = metadata.to_string();
                        }
                        object.insert("blobId".into(), blob.into());
                    }
                    _ => {}
                }
            }
            list.push(object);
        }
        if cached != rows
            && !matches!(
                self.store()
                    .replace_sieve_scripts(account, rows, &cached, None)
                    .await,
                Ok(SieveReplaceResult::Applied)
            )
        {
            return err("serverFail");
        }
        json!({"accountId":account,"state":state(rows),"list":list,"notFound":missing})
    }
}
impl Engine {
    async fn sieve_set(
        &self,
        account: &str,
        args: &Value,
        old: Vec<SieveScriptRow>,
        ctx: &mut SieveContext,
        created_ids: &mut HashMap<String, String>,
    ) -> Value {
        let old_state = state(&old);
        let mut pending_ids = created_ids.clone();
        if args
            .get("ifInState")
            .filter(|v| !v.is_null())
            .is_some_and(|v| v.as_str() != Some(&old_state))
        {
            return err("stateMismatch");
        }
        for field in ["create", "update"] {
            if args
                .get(field)
                .filter(|v| !v.is_null())
                .is_some_and(|v| !v.is_object())
            {
                return err("invalidArguments");
            }
        }
        if args
            .get("destroy")
            .filter(|v| !v.is_null())
            .is_some_and(|v| !v.as_array().is_some_and(|a| a.iter().all(Value::is_string)))
            || args
                .get("onSuccessDeactivateScript")
                .is_some_and(|v| !v.is_boolean())
            || args
                .get("onSuccessActivateScript")
                .filter(|v| !v.is_null())
                .is_some_and(|v| !v.is_string())
        {
            return err("invalidArguments");
        }
        // A stored script may predate a stricter policy. Activation is a new
        // execution grant, so check it before applying any mutations in this call.
        if let Some(target) = args.get("onSuccessActivateScript").and_then(Value::as_str)
            && let Some(script) = old.iter().find(|s| s.id() == target)
            // An update below validates its replacement bytes; if it fails,
            // all-success semantics suppress activation automatically.
            && args.get("update").and_then(|updates| updates.get(target)).is_none()
        {
            let mut policy = ctx.policy.clone();
            policy.raw_edit = true; // Activating existing bytes is not a raw edit.
            policy.advertised = Some(ctx.extensions());
            if mw_sieve::policy::enforce(&script.content, &policy, EditKind::Builder).is_err() {
                return err("forbidden");
            }
            if ctx.client.is_none() && mw_sieve::local::validate(&script.content).is_err() {
                return err("invalidSieve");
            }
        }
        let mut rows = old.clone();
        let mut created = Map::new();
        let mut updated = Map::new();
        let mut destroyed = Vec::new();
        let mut not_created = Map::new();
        let mut not_updated = Map::new();
        let mut not_destroyed = Map::new();
        for (create, field) in [(true, "create"), (false, "update")] {
            let Some(items) = args.get(field).and_then(Value::as_object) else {
                continue;
            };
            for (key, patch) in items {
                let outcome: Result<(String, Value), Value> = async {
                    if ctx.unavailable {
                        return Err(err("serverUnavailable"));
                    }
                    let patch = patch.as_object().ok_or_else(|| err("invalidProperties"))?;
                    if patch
                        .keys()
                        .any(|k| !matches!(k.as_str(), "name" | "blobId"))
                    {
                        return Err(err("invalidProperties"));
                    }
                    let prior = if create {
                        None
                    } else {
                        Some(
                            rows.iter()
                                .find(|s| s.id() == *key)
                                .cloned()
                                .ok_or_else(|| err("notFound"))?,
                        )
                    };
                    let id = prior
                        .as_ref()
                        .map(SieveScriptRow::id)
                        .unwrap_or_else(new_id);
                    let name = match patch.get("name") {
                        Some(Value::String(n)) => n.clone(),
                        Some(Value::Null) | None => prior
                            .as_ref()
                            .map(|s| s.name.clone())
                            .unwrap_or_else(|| id.clone()),
                        _ => return Err(err("invalidProperties")),
                    };
                    if !valid_name(&name) {
                        return Err(json!({"type":"invalidProperties","properties":["name"]}));
                    }
                    if let Some(existing) = rows.iter().find(|s| s.name == name && s.id() != id) {
                        return Err(json!({"type":"alreadyExists","existingId":existing.id()}));
                    }
                    if create && ctx.policy.max_scripts.is_some_and(|max| rows.len() >= max) {
                        return Err(err("overQuota"));
                    }
                    let source = if let Some(blob) = patch.get("blobId") {
                        self.sieve_source(account, blob).await?
                    } else {
                        prior.as_ref().map(|s| s.content.clone()).ok_or_else(
                            || json!({"type":"invalidProperties","properties":["blobId"]}),
                        )?
                    };
                    if ctx.edit_kind == EditKind::Raw
                        && let Some(prior) = &prior
                        && vacation::protected(&prior.content)? != vacation::protected(&source)?
                    {
                        return Err(err("forbidden"));
                    }
                    self.sieve_validate_source(&source, ctx).await?;
                    if let Some(client) = ctx.client.as_mut() {
                        client
                            .have_space(&name, source.len() as u64)
                            .await
                            .map_err(|_| err("overQuota"))?;
                        if let Some(prior) = &prior
                            && prior.name != name
                        {
                            client
                                .rename_script(&prior.name, &name)
                                .await
                                .map_err(|_| err("serverUnavailable"))?;
                        }
                        if prior.as_ref().is_none_or(|s| s.content != source)
                            && client.put_script(&name, &source).await.is_err()
                        {
                            // Undo a rename if uploading the changed body failed. Reconciliation on the next call is authoritative.
                            if let Some(prior) = &prior
                                && prior.name != name
                            {
                                let _ = client.rename_script(&name, &prior.name).await;
                            }
                            return Err(err("serverUnavailable"));
                        }
                    }
                    let mut metadata: Value = prior
                        .as_ref()
                        .filter(|s| s.content == source)
                        .and_then(|s| serde_json::from_str(&s.state_json).ok())
                        .unwrap_or_else(|| json!({"_jmapId":id,"_remote":ctx.client.is_some()}));
                    if let Some(blob) = patch.get("blobId") {
                        metadata["_blobId"] = blob.clone();
                    }
                    metadata["_remote"] = ctx.client.is_some().into();
                    let row = SieveScriptRow {
                        name: name.clone(),
                        content: source,
                        active: prior.as_ref().is_some_and(|s| s.active),
                        state_json: metadata.to_string(),
                        updated_at: 0,
                    };
                    rows.retain(|s| s.id() != id);
                    rows.push(row);
                    Ok((
                        id.clone(),
                        if create {
                            json!({"id":id,"name":name,"isActive":false})
                        } else {
                            Value::Null
                        },
                    ))
                }
                .await;
                match outcome {
                    Ok((id, value)) => {
                        if create {
                            pending_ids.insert(key.clone(), id);
                            created.insert(key.clone(), value);
                        } else {
                            updated.insert(key.clone(), value);
                        }
                    }
                    Err(e) => {
                        if e["type"] == "serverUnavailable" {
                            ctx.unavailable = true;
                        }
                        if create {
                            not_created.insert(key.clone(), e);
                        } else {
                            not_updated.insert(key.clone(), e);
                        }
                    }
                }
            }
        }
        if let Some(ids) = args.get("destroy").and_then(Value::as_array) {
            for id in ids {
                let id = id.as_str().unwrap();
                let outcome = if ctx.unavailable {
                    Err(err("serverUnavailable"))
                } else if let Some(row) = rows.iter().find(|s| s.id() == id) {
                    if vacation::protected(&row.content)
                        .is_ok_and(|(meta, body)| meta.is_some() || body.is_some())
                    {
                        Err(err("forbidden"))
                    } else if row.active {
                        Err(err("sieveIsActive"))
                    } else if let Some(client) = ctx.client.as_mut() {
                        client
                            .delete_script(&row.name)
                            .await
                            .map_err(|_| err("serverUnavailable"))
                    } else {
                        Ok(())
                    }
                } else {
                    Err(err("notFound"))
                };
                match outcome {
                    Ok(()) => {
                        rows.retain(|s| s.id() != id);
                        destroyed.push(id.to_string());
                    }
                    Err(e) => {
                        if e["type"] == "serverUnavailable" {
                            ctx.unavailable = true;
                        }
                        not_destroyed.insert(id.into(), e);
                    }
                }
            }
        }
        if not_created.is_empty() && not_updated.is_empty() && not_destroyed.is_empty() {
            let target = args
                .get("onSuccessActivateScript")
                .and_then(Value::as_str)
                .map(|id| {
                    id.strip_prefix('#')
                        .and_then(|key| pending_ids.get(key))
                        .map(String::as_str)
                        .unwrap_or(id)
                });
            let chosen = target
                .and_then(|id| rows.iter().find(|s| s.id() == id))
                .map(|s| (s.id(), s.name.clone()));
            let deactivate = args["onSuccessDeactivateScript"].as_bool().unwrap_or(false);
            if chosen.is_some() || deactivate {
                if let Some(client) = ctx.client.as_mut()
                    && client
                        .set_active(chosen.as_ref().map(|(_, name)| name.as_str()).unwrap_or(""))
                        .await
                        .is_err()
                {
                    return err("serverUnavailable");
                }
                for row in &mut rows {
                    let active = chosen.as_ref().is_some_and(|(id, _)| row.id() == *id);
                    if row.active != active {
                        row.active = active;
                        let mut metadata: Value =
                            serde_json::from_str(&row.state_json).unwrap_or_else(|_| json!({}));
                        metadata["_remote"] = ctx.client.is_some().into();
                        row.state_json = metadata.to_string();
                        if let Some((_, item)) = created
                            .iter_mut()
                            .find(|(_, v)| v["id"].as_str() == Some(&row.id()))
                        {
                            item["isActive"] = active.into();
                        } else {
                            updated.insert(row.id(), json!({"isActive":active}));
                        }
                    }
                }
            }
        }
        rows.sort_by(|a, b| a.name.cmp(&b.name));
        match self
            .store()
            .replace_sieve_scripts(account, &old, &rows, ctx.policy.max_scripts)
            .await
        {
            Ok(SieveReplaceResult::Applied) => {}
            Ok(SieveReplaceResult::StateMismatch) => return err("stateMismatch"),
            Ok(SieveReplaceResult::OverQuota) => return err("overQuota"),
            Err(_) => return err("serverFail"),
        }
        fn targets(source: &str) -> std::collections::BTreeSet<Option<String>> {
            mw_sieve::extensions::check(source)
                .map(|(_, a)| a.redirects.into_iter().map(|t| t.value).collect())
                .unwrap_or_default()
        }
        ctx.forwarding_added |= rows.iter().any(|row| {
            let now = targets(&row.content);
            if now.is_empty() {
                return false;
            }
            match old.iter().find(|prior| prior.id() == row.id()) {
                None => true,
                Some(prior) => {
                    row.active && !prior.active || !now.is_subset(&targets(&prior.content))
                }
            }
        });
        *created_ids = pending_ids;
        json!({"accountId":account,"oldState":old_state,"newState":state(&rows),"created":created,"updated":updated,"destroyed":destroyed,
            "notCreated":not_created,"notUpdated":not_updated,"notDestroyed":not_destroyed})
    }
}
fn filter_matches(filter: &Value, row: &SieveScriptRow, depth: usize) -> Result<bool, Value> {
    if filter.is_null() {
        return Ok(true);
    }
    if depth > 32 {
        return Err(err("unsupportedFilter"));
    }
    let map = filter.as_object().ok_or_else(|| err("unsupportedFilter"))?;
    if let Some(operator) = map.get("operator") {
        if map
            .keys()
            .any(|k| !matches!(k.as_str(), "operator" | "conditions"))
        {
            return Err(err("unsupportedFilter"));
        }
        let conditions = map
            .get("conditions")
            .and_then(Value::as_array)
            .ok_or_else(|| err("unsupportedFilter"))?;
        let values: Vec<bool> = conditions
            .iter()
            .map(|c| filter_matches(c, row, depth + 1))
            .collect::<Result<_, _>>()?;
        return match operator.as_str() {
            Some("AND") => Ok(values.iter().all(|v| *v)),
            Some("OR") => Ok(values.iter().any(|v| *v)),
            Some("NOT") => Ok(!values.iter().any(|v| *v)),
            _ => Err(err("unsupportedFilter")),
        };
    }
    if map
        .keys()
        .any(|k| !matches!(k.as_str(), "name" | "isActive"))
        || map.get("name").is_some_and(|v| !v.is_string())
        || map.get("isActive").is_some_and(|v| !v.is_boolean())
    {
        return Err(err("unsupportedFilter"));
    }
    Ok(map
        .get("name")
        .is_none_or(|v| row.name.contains(v.as_str().unwrap()))
        && map
            .get("isActive")
            .is_none_or(|v| row.active == v.as_bool().unwrap()))
}
fn sieve_query(account: &str, args: &Value, rows: &[SieveScriptRow]) -> Value {
    if ["position", "anchorOffset"]
        .iter()
        .any(|k| args.get(k).is_some_and(|v| !v.is_i64()))
        || args
            .get("limit")
            .filter(|v| !v.is_null())
            .is_some_and(|v| !v.is_u64())
        || args
            .get("anchor")
            .filter(|v| !v.is_null())
            .is_some_and(|v| !v.is_string())
        || args.get("calculateTotal").is_some_and(|v| !v.is_boolean())
    {
        return err("invalidArguments");
    }
    // Validate filters even when the account has no scripts.
    let sentinel = SieveScriptRow {
        name: String::new(),
        content: String::new(),
        active: false,
        state_json: "{}".into(),
        updated_at: 0,
    };
    if let Err(e) = filter_matches(&args["filter"], &sentinel, 0) {
        return e;
    }
    let mut matches: Vec<_> = rows
        .iter()
        .filter(|r| filter_matches(&args["filter"], r, 0).unwrap_or(false))
        .collect();
    let sort = match args.get("sort").filter(|v| !v.is_null()) {
        None => vec![],
        Some(Value::Array(a)) => a.clone(),
        _ => return err("unsupportedSort"),
    };
    for comparator in &sort {
        if !comparator.as_object().is_some_and(|o| {
            o.keys()
                .all(|k| matches!(k.as_str(), "property" | "isAscending"))
        }) || !matches!(comparator["property"].as_str(), Some("name" | "isActive"))
            || comparator
                .get("isAscending")
                .is_some_and(|v| !v.is_boolean())
        {
            return err("unsupportedSort");
        }
    }
    matches.sort_by(|a, b| {
        for c in &sort {
            let order = if c["property"] == "name" {
                a.name.cmp(&b.name)
            } else {
                a.active.cmp(&b.active)
            };
            let order = if c["isAscending"].as_bool().unwrap_or(true) {
                order
            } else {
                order.reverse()
            };
            if !order.is_eq() {
                return order;
            }
        }
        a.id().cmp(&b.id())
    });
    let total = matches.len();
    let position = args.get("position").and_then(Value::as_i64).unwrap_or(0);
    let position = if let Some(anchor) = args.get("anchor").filter(|v| !v.is_null()) {
        let Some(index) = matches
            .iter()
            .position(|r| Some(r.id().as_str()) == anchor.as_str())
        else {
            return err("anchorNotFound");
        };
        (index as i64 + args["anchorOffset"].as_i64().unwrap_or(0)).max(0) as usize
    } else {
        if position < 0 {
            (total as i64 + position).max(0) as usize
        } else {
            position as usize
        }
    };
    let limit = args
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(u64::MAX)
        .min(usize::MAX as u64) as usize;
    let ids: Vec<_> = matches
        .into_iter()
        .skip(position)
        .take(limit)
        .map(SieveScriptRow::id)
        .collect();
    let mut response = json!({"accountId":account,"queryState":state(rows),"canCalculateChanges":false,"position":position,"ids":ids});
    if args["calculateTotal"].as_bool().unwrap_or(false) {
        response["total"] = total.into();
    }
    response
}
