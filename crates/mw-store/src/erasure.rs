//! Erasure, verification and export driven by the data map ([`crate::data_map`]),
//! the erasure tombstones, and the backup register with its restore replay.
//!
//! One person is described by [`PersonIds`]. The purge runs every registry rule
//! that deletes; the verify pass runs the registry's count of every rule, so a
//! table that is mapped but has no DELETE of its own is still checked. A
//! tombstone (a keyed hash of the address, see [`crate::scrub`]) is written for
//! every erasure and scrub; a restore replays the tombstones newer than the
//! backup's watermark.

use crate::data_map::{self, Bind, Export, How};
use crate::{Store, StoreError, q};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;

/// The identifiers one person is found by.
#[derive(Clone, Debug, Default)]
pub struct PersonIds {
    pub address: String,
    pub account_ids: Vec<String>,
    pub owners: Vec<String>,
    pub principals: Vec<String>,
}

impl PersonIds {
    /// The values a rule with `bind` is run with.
    pub fn binds(&self, bind: Bind) -> Vec<&str> {
        let mut out: Vec<&str> = Vec::new();
        fn add<'a>(out: &mut Vec<&'a str>, v: &'a [String]) {
            out.extend(v.iter().map(String::as_str));
        }
        match bind {
            Bind::Account => add(&mut out, &self.account_ids),
            Bind::Owner => add(&mut out, &self.owners),
            Bind::Principal => add(&mut out, &self.principals),
            Bind::Address => {
                if !self.address.is_empty() {
                    out.push(&self.address);
                }
            }
            Bind::Any => {
                add(&mut out, &self.account_ids);
                add(&mut out, &self.owners);
                if !self.address.is_empty() {
                    out.push(&self.address);
                }
            }
            Bind::None => {}
        }
        out.retain(|s| !s.is_empty());
        out.sort_unstable();
        out.dedup();
        out
    }
}

/// One stored tombstone.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tombstone {
    pub seq: i64,
    pub id: String,
    pub subject_hash: String,
    pub key_gen: i64,
    pub scope: String,
    pub request_id: Option<String>,
    pub erased_at: i64,
}

/// One registered backup.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupRow {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub location: String,
    pub created_at: i64,
    pub tombstone_watermark: i64,
    /// Tombstones written after the backup: people erased or scrubbed since, who
    /// are still present in it.
    pub erased_since: i64,
    pub manifest: Value,
}

/// What a restore replay did.
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayReport {
    pub tombstones: usize,
    /// Tombstones whose subject was found in the restored data and erased.
    pub applied: usize,
    /// Tombstones whose subject is not in the restored data (nothing to do).
    pub absent: usize,
    pub errors: Vec<String>,
}

/// Format tag of a backup manifest.
pub const BACKUP_FORMAT: &str = "mailwoman-store-backup-v1";

impl Store {
    // ---- resolving a person -------------------------------------------------

    /// Principal ids reached through the person's identity bindings and sessions.
    pub async fn dsr_resolve_principals(
        &self,
        account_ids: &[String],
        address: &str,
    ) -> Result<Vec<String>, StoreError> {
        let mut out = Vec::new();
        let mut keys: Vec<&str> = account_ids.iter().map(String::as_str).collect();
        if !address.is_empty() {
            keys.push(address);
        }
        for k in keys {
            out.extend(
                q("SELECT principal_id FROM identity_binding WHERE subject=?1")
                    .bind(k)
                    .fetch_all_scalar_string(&self.backend)
                    .await?,
            );
        }
        for id in account_ids {
            out.extend(
                q("SELECT sp.principal_id FROM session_principal sp JOIN sessions s ON s.id=sp.session_id WHERE s.account_id=?1")
                    .bind(id)
                    .fetch_all_scalar_string(&self.backend)
                    .await?,
            );
        }
        out.sort();
        out.dedup();
        Ok(out)
    }

    /// External subjects (IdP subject ids and the like) bound to the principals.
    pub async fn dsr_binding_subjects(&self, principals: &[String]) -> Result<Vec<String>, StoreError> {
        let mut out = Vec::new();
        for p in principals {
            out.extend(
                q("SELECT subject FROM identity_binding WHERE principal_id=?1")
                    .bind(p)
                    .fetch_all_scalar_string(&self.backend)
                    .await?,
            );
        }
        Ok(out)
    }

    // ---- erasure and its verification ---------------------------------------

    /// Run every deleting rule for the person. Returns rows removed per table; a
    /// statement that fails is reported under errors and the rest still run (the
    /// erasure is idempotent and is run again).
    pub async fn dsr_purge_person(
        &self,
        p: &PersonIds,
    ) -> (BTreeMap<String, u64>, Vec<String>) {
        let mut counts = BTreeMap::new();
        let mut errors = Vec::new();
        // Uploaded files live in the upload backend as well as in a row: remove the
        // object first. Without a configured backend only the rows go (below).
        if self.uploads_configured() {
            for id in &p.account_ids {
                let blobs = q("SELECT blob_id FROM uploaded_blobs WHERE account_id=?1")
                    .bind(id)
                    .fetch_all_scalar_string(&self.backend)
                    .await
                    .unwrap_or_default();
                for blob in blobs {
                    match self.delete_upload(id, &blob).await {
                        Ok(()) => *counts.entry("uploaded_blobs".to_string()).or_insert(0) += 1,
                        Err(e) => errors.push(format!("uploaded_blobs: {e}")),
                    }
                }
            }
        }
        for rule in data_map::erase_order() {
            let How::Delete(sql) = rule.how else { continue };
            for id in p.binds(rule.bind) {
                match q(sql).bind(id).execute(&self.backend).await {
                    Ok(n) => {
                        if n > 0 {
                            *counts.entry(rule.table.to_string()).or_insert(0) += n;
                        }
                    }
                    Err(e) => errors.push(format!("{}: {e}", rule.table)),
                }
            }
        }
        (counts, errors)
    }

    /// Rows each rule of the registry still finds for the person (tables with
    /// none are left out). This is the verify pass after an erasure: it reads the
    /// registry, not the delete statements.
    pub async fn dsr_verify_person(&self, p: &PersonIds) -> BTreeMap<String, u64> {
        self.dsr_verify_checked(p).await.0
    }

    /// [`Self::dsr_verify_person`] plus the statements that could not be run (a
    /// count that fails must not read as "nothing left").
    pub async fn dsr_verify_checked(&self, p: &PersonIds) -> (BTreeMap<String, u64>, Vec<String>) {
        let mut out = BTreeMap::new();
        let mut errors = Vec::new();
        for rule in data_map::RULES {
            if matches!(rule.how, How::Keep(_)) || rule.count.is_empty() {
                continue;
            }
            for id in p.binds(rule.bind) {
                match q(rule.count).bind(id).fetch_scalar_i64(&self.backend).await {
                    Ok(n) if n > 0 => *out.entry(rule.table.to_string()).or_insert(0u64) += n as u64,
                    Ok(_) => {}
                    Err(e) => errors.push(format!("{}: {e}", rule.table)),
                }
            }
        }
        (out, errors)
    }

    /// Rows of the person per table that exist now (every rule with a count),
    /// for a dry run and for tests.
    pub async fn dsr_footprint(&self, p: &PersonIds) -> BTreeMap<String, u64> {
        self.dsr_verify_person(p).await
    }

    // ---- export ------------------------------------------------------------

    /// The table-driven part of an export section: rows for [`Export::Rows`]
    /// rules and a presence marker for [`Export::Marker`] rules.
    pub async fn dsr_export_table_section(
        &self,
        section: &str,
        p: &PersonIds,
    ) -> Result<Vec<Value>, StoreError> {
        let mut out = Vec::new();
        for rule in data_map::export_rules(section) {
            match rule.export {
                Export::Rows { select, cols, .. } => {
                    let cols: Vec<(&str, &str)> = cols
                        .split(',')
                        .map(|c| c.split_once(':').unwrap_or((c, "s")))
                        .collect();
                    for id in p.binds(rule.bind) {
                        // The select lists its columns by SQL name; read them by
                        // position of the declared output name via the alias list.
                        for r in q(select).bind(id).fetch_all(&self.backend).await? {
                            let names = sql_column_names(select);
                            let mut obj = serde_json::Map::new();
                            obj.insert("table".into(), json!(rule.table));
                            for (i, (out_name, kind)) in cols.iter().enumerate() {
                                let src = names.get(i).map(String::as_str).unwrap_or(out_name);
                                let mut v = r.get_json(src);
                                if *kind == "j"
                                    && let Value::String(s) = &v
                                {
                                    v = serde_json::from_str(s).unwrap_or(Value::Null);
                                }
                                obj.insert((*out_name).into(), v);
                            }
                            out.push(Value::Object(obj));
                        }
                    }
                }
                Export::Marker { .. } => {
                    let mut n = 0i64;
                    for id in p.binds(rule.bind) {
                        n += q(rule.count)
                            .bind(id)
                            .fetch_scalar_i64(&self.backend)
                            .await
                            .unwrap_or(0);
                    }
                    if n > 0 {
                        out.push(json!({
                            "table": rule.table, "rows": n,
                            "status": "present, not exported",
                        }));
                    }
                }
                _ => {}
            }
        }
        Ok(out)
    }

    // ---- tombstones ----------------------------------------------------------

    /// Highest tombstone sequence (0 when none): the watermark a backup records.
    pub async fn erasure_watermark(&self) -> Result<i64, StoreError> {
        Ok(q("SELECT COALESCE(MAX(seq),0) FROM erasure_tombstone")
            .fetch_scalar_i64(&self.backend)
            .await?)
    }

    /// Record an erasure or scrub of the subject whose normalised address hashes
    /// to `subject_hash` under key generation `key_gen`. Returns the sequence.
    pub async fn erasure_tombstone_add(
        &self,
        subject_hash: &str,
        key_gen: i64,
        scope: &str,
        request_id: Option<&str>,
        now: i64,
    ) -> Result<i64, StoreError> {
        let id = uuid::Uuid::new_v4().to_string();
        let mut last = None;
        for _ in 0..5 {
            let seq = self.erasure_watermark().await? + 1;
            match q("INSERT INTO erasure_tombstone(seq,id,subject_hash,key_gen,scope,request_id,erased_at) VALUES(?1,?2,?3,?4,?5,?6,?7)")
                .bind(seq)
                .bind(&id)
                .bind(subject_hash)
                .bind(key_gen)
                .bind(scope)
                .bind(request_id.map(str::to_string))
                .bind(now)
                .execute(&self.backend)
                .await
            {
                Ok(_) => return Ok(seq),
                Err(e) => last = Some(e),
            }
        }
        Err(last.map(StoreError::from).unwrap_or(StoreError::NotFound))
    }

    /// Tombstones with `seq` above `since`, oldest first.
    pub async fn erasure_tombstones_since(&self, since: i64) -> Result<Vec<Tombstone>, StoreError> {
        Ok(q("SELECT seq,id,subject_hash,key_gen,scope,request_id,erased_at FROM erasure_tombstone WHERE seq>?1 ORDER BY seq LIMIT 100000")
            .bind(since)
            .fetch_all(&self.backend)
            .await?
            .iter()
            .map(|r| Tombstone {
                seq: r.get_i64("seq"),
                id: r.get_string("id"),
                subject_hash: r.get_string("subject_hash"),
                key_gen: r.get_i64("key_gen"),
                scope: r.get_string("scope"),
                request_id: r.get_opt_string("request_id"),
                erased_at: r.get_i64("erased_at"),
            })
            .collect())
    }

    /// Whether a tombstone for this subject hash exists.
    pub async fn erasure_tombstone_exists(&self, subject_hash: &str) -> Result<bool, StoreError> {
        Ok(q("SELECT COUNT(*) FROM erasure_tombstone WHERE subject_hash=?1")
            .bind(subject_hash)
            .fetch_scalar_i64(&self.backend)
            .await?
            > 0)
    }

    /// Delete tombstones older than `days` (0 keeps them forever). Returns the
    /// number removed.
    pub async fn erasure_tombstone_purge(&self, now: i64, days: i64) -> Result<u64, StoreError> {
        if days <= 0 {
            return Ok(0);
        }
        Ok(q("DELETE FROM erasure_tombstone WHERE erased_at<?1")
            .bind(now.saturating_sub(days.saturating_mul(86_400)))
            .execute(&self.backend)
            .await?)
    }

    // ---- backup register ---------------------------------------------------

    pub async fn backup_register_add(
        &self,
        kind: &str,
        label: &str,
        location: &str,
        created_at: i64,
        tombstone_watermark: i64,
        manifest: &Value,
    ) -> Result<String, StoreError> {
        let id = uuid::Uuid::new_v4().to_string();
        q("INSERT INTO backup_register(id,kind,label,location,created_at,tombstone_watermark,manifest_json) VALUES(?1,?2,?3,?4,?5,?6,?7)")
            .bind(&id)
            .bind(kind)
            .bind(label)
            .bind(location)
            .bind(created_at)
            .bind(tombstone_watermark)
            .bind(manifest.to_string())
            .execute(&self.backend)
            .await?;
        Ok(id)
    }

    /// Registered backups, newest first, each with the number of people erased
    /// since it was made.
    pub async fn backup_register_list(&self) -> Result<Vec<BackupRow>, StoreError> {
        let rows = q("SELECT id,kind,label,location,created_at,tombstone_watermark,manifest_json FROM backup_register ORDER BY created_at DESC,id LIMIT 1000")
            .fetch_all(&self.backend)
            .await?;
        let mut out = Vec::new();
        for r in rows {
            let wm = r.get_i64("tombstone_watermark");
            let erased_since = q("SELECT COUNT(*) FROM erasure_tombstone WHERE seq>?1")
                .bind(wm)
                .fetch_scalar_i64(&self.backend)
                .await?;
            out.push(BackupRow {
                id: r.get_string("id"),
                kind: r.get_string("kind"),
                label: r.get_string("label"),
                location: r.get_string("location"),
                created_at: r.get_i64("created_at"),
                tombstone_watermark: wm,
                erased_since,
                manifest: serde_json::from_str(&r.get_string("manifest_json")).unwrap_or(Value::Null),
            });
        }
        Ok(out)
    }

    pub async fn backup_register_delete(&self, id: &str) -> Result<bool, StoreError> {
        Ok(q("DELETE FROM backup_register WHERE id=?1")
            .bind(id)
            .execute(&self.backend)
            .await?
            > 0)
    }

    /// Write a consistent copy of an SQLite store to `dest` (which must not
    /// exist) with a manifest next to it (`<dest>.manifest.json`) recording the
    /// tombstone watermark, and register it. Postgres stores are backed up with
    /// `pg_dump`; register that backup with [`Self::backup_register_add`].
    pub async fn backup_to_file(
        &self,
        dest: &Path,
        label: &str,
        now: i64,
    ) -> Result<Value, StoreError> {
        let crate::backend::Backend::Sqlite(pool) = &self.backend else {
            return Err(StoreError::Corrupt(
                "backup-store writes SQLite files; back up Postgres with pg_dump and register it".into(),
            ));
        };
        if dest.exists() {
            return Err(StoreError::Corrupt(format!("{} already exists", dest.display())));
        }
        let watermark = self.erasure_watermark().await?;
        let literal = dest.to_string_lossy().replace('\'', "''");
        sqlx::query(&format!("VACUUM INTO '{literal}'")).execute(pool).await?;
        let manifest = json!({
            "format": BACKUP_FORMAT,
            "label": label,
            "createdAt": now,
            "tombstoneWatermark": watermark,
            "database": dest.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
        });
        let mut mpath = dest.as_os_str().to_owned();
        mpath.push(".manifest.json");
        std::fs::write(&mpath, serde_json::to_vec_pretty(&manifest).unwrap_or_default())
            .map_err(|e| StoreError::Corrupt(e.to_string()))?;
        self.backup_register_add("store", label, &dest.to_string_lossy(), now, watermark, &manifest)
            .await?;
        Ok(manifest)
    }

    // ---- restore replay ------------------------------------------------------

    /// Apply tombstones to this (restored) store: insert them and the key
    /// generations they were made with, then erase every person whose address
    /// hashes to a tombstone's subject. `tombstones` are those newer than the
    /// backup's watermark; `keys` are the key rows (generation, sealed key bytes as
    /// stored, created_at) of the store they came from.
    pub async fn erasure_replay(
        &self,
        tombstones: &[Tombstone],
        keys: &[crate::scrub::KeyRow],
        now: i64,
    ) -> ReplayReport {
        let mut report = ReplayReport {
            tombstones: tombstones.len(),
            ..ReplayReport::default()
        };
        for k in keys {
            if let Err(e) = self.scrub_key_import(k).await {
                report.errors.push(format!("key {}: {e}", k.generation));
            }
        }
        let candidates = match self.person_address_candidates().await {
            Ok(c) => c,
            Err(e) => {
                report.errors.push(format!("candidates: {e}"));
                return report;
            }
        };
        for t in tombstones {
            let res = q("INSERT INTO erasure_tombstone(seq,id,subject_hash,key_gen,scope,request_id,erased_at) VALUES(?1,?2,?3,?4,?5,?6,?7) ON CONFLICT DO NOTHING")
                .bind(t.seq)
                .bind(&t.id)
                .bind(&t.subject_hash)
                .bind(t.key_gen)
                .bind(&t.scope)
                .bind(t.request_id.clone())
                .bind(t.erased_at)
                .execute(&self.backend)
                .await;
            if let Err(e) = res {
                report.errors.push(format!("tombstone {}: {e}", t.seq));
                continue;
            }
            let key = match self.scrub_key_get(t.key_gen).await {
                Ok(Some(k)) => k,
                Ok(None) => {
                    report
                        .errors
                        .push(format!("tombstone {}: key generation {} is not available", t.seq, t.key_gen));
                    continue;
                }
                Err(e) => {
                    report.errors.push(format!("tombstone {}: {e}", t.seq));
                    continue;
                }
            };
            let mut hit = false;
            for addr in &candidates {
                if crate::scrub::subject_hash(&key, addr) != t.subject_hash {
                    continue;
                }
                hit = true;
                match self.erase_address_everywhere(addr, &t.scope, t.request_id.as_deref(), now).await {
                    Ok(left) if left == 0 => {}
                    Ok(left) => report.errors.push(format!("tombstone {}: {left} rows remain", t.seq)),
                    Err(e) => report.errors.push(format!("tombstone {}: {e}", t.seq)),
                }
            }
            if hit {
                report.applied += 1;
            } else {
                report.absent += 1;
            }
        }
        report
    }

    /// Every address that can name a person in this store.
    async fn person_address_candidates(&self) -> Result<Vec<String>, StoreError> {
        let mut out = Vec::new();
        for sql in [
            "SELECT address FROM mail_user",
            "SELECT address FROM mail_user_account",
            "SELECT address FROM signin_identity",
            "SELECT username FROM accounts",
            "SELECT address FROM dsr_request",
            "SELECT email FROM identities",
        ] {
            out.extend(q(sql).fetch_all_scalar_string(&self.backend).await?);
        }
        out.retain(|a| !a.is_empty());
        out.sort();
        out.dedup();
        Ok(out)
    }

    /// Resolve and erase the person with `address` from the store alone (no
    /// search index or in-memory sessions): used by the restore replay. Returns
    /// the rows the verify pass still finds.
    pub async fn erase_address_everywhere(
        &self,
        address: &str,
        scope: &str,
        _request_id: Option<&str>,
        now: i64,
    ) -> Result<u64, StoreError> {
        let mut account_ids = q("SELECT account_id FROM mail_user_account WHERE address=?1")
            .bind(address)
            .fetch_all_scalar_string(&self.backend)
            .await?;
        account_ids.extend(
            q("SELECT account_id FROM signin_identity WHERE address=?1")
                .bind(address)
                .fetch_all_scalar_string(&self.backend)
                .await?,
        );
        account_ids.extend(
            q("SELECT id FROM accounts WHERE username=?1")
                .bind(address)
                .fetch_all_scalar_string(&self.backend)
                .await?,
        );
        account_ids.sort();
        account_ids.dedup();
        let mut owners = Vec::new();
        for id in &account_ids {
            if let Some(o) = self.native_personal_subject(id).await? {
                owners.push(o);
            }
            for s in self.sessions_by_account(id).await? {
                if let Some(o) = self.session_personal_subject(&s.id).await? {
                    owners.push(o);
                }
                if let Some(o) = self.session_storage_authority(&s.id).await? {
                    owners.push(o);
                }
            }
        }
        owners.sort();
        owners.dedup();
        let principals = self.dsr_resolve_principals(&account_ids, address).await?;
        let p = PersonIds {
            address: address.to_string(),
            account_ids,
            owners,
            principals,
        };
        if scope == "scrub" {
            let opts = crate::scrub::ScrubOptions::default();
            let r = self.scrub_person(&p, &[], &[], &opts, None, now).await?;
            return Ok(r.errors.len() as u64);
        }
        let (_, errors) = self.dsr_purge_person(&p).await;
        if let Some(e) = errors.first() {
            return Err(StoreError::Corrupt(e.clone()));
        }
        if let Ok(Some(l)) = self.user_lifecycle(address).await
            && l.status != "deleted"
        {
            let _ = self.change_user_lifecycle(&l, "deleted", "restore-replay", now).await;
        }
        let r = self.scrub_person(&p, &[], &[], &crate::scrub::ScrubOptions::default(), None, now).await;
        let _ = r;
        Ok(self.dsr_verify_person(&p).await.values().sum())
    }
}

/// The source column names of a SELECT list, with `AS` aliases resolved to the
/// alias (the name a result column carries).
pub(crate) fn sql_column_names(select: &str) -> Vec<String> {
    let lower = select.to_ascii_lowercase();
    let start = lower.find("select ").map(|i| i + 7).unwrap_or(0);
    // The first top-level " from " ends the list.
    let mut depth = 0i32;
    let bytes = lower.as_bytes();
    let mut end = lower.len();
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            b' ' if depth == 0 && lower[i..].starts_with(" from ") => {
                end = i;
                break;
            }
            _ => {}
        }
        i += 1;
    }
    let list = &select[start..end];
    let mut names = Vec::new();
    let mut depth = 0i32;
    let mut cur = String::new();
    for ch in list.chars() {
        match ch {
            '(' => {
                depth += 1;
                cur.push(ch);
            }
            ')' => {
                depth -= 1;
                cur.push(ch);
            }
            ',' if depth == 0 => {
                names.push(std::mem::take(&mut cur));
            }
            _ => cur.push(ch),
        }
    }
    names.push(cur);
    names
        .into_iter()
        .map(|c| {
            let c = c.trim();
            let lc = c.to_ascii_lowercase();
            let name = match lc.rfind(" as ") {
                Some(i) => c[i + 4..].trim(),
                None => c.rsplit('.').next().unwrap_or(c),
            };
            name.trim_matches('"').to_string()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_names_follow_aliases_and_prefixes() {
        assert_eq!(
            sql_column_names("SELECT id,'owner' AS role,t.due_utc,CASE WHEN a IS NULL THEN 0 ELSE 1 END AS has_x FROM t WHERE x=?1"),
            vec!["id", "role", "due_utc", "has_x"]
        );
        assert_eq!(sql_column_names("SELECT \"user\",name FROM tags"), vec!["user", "name"]);
    }
}
