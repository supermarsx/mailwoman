//! Per-person scrubbing: replace one person's identifiers in audit and log
//! history with a stable keyed pseudonym, without breaking the audit hash chain.
//!
//! * The pseudonym of a person is `HMAC-SHA256(key, "person" || normalised
//!   address)` truncated and prefixed with its key generation. The same person
//!   always maps to the same pseudonym under one generation, so history can still
//!   be correlated, but the address cannot be recovered without the key.
//! * Keys are random 32-byte secrets, one per generation, sealed with the server
//!   key in `scrub_key`. Rotating adds a generation. A retired generation is kept
//!   until `destroy_after`; once deleted, pseudonyms made with it are permanently
//!   unlinkable (and tombstones made with it can no longer be replayed).
//! * `audit_log` is hash chained. Rewriting a row changes its hash and every hash
//!   after it, so the scrub re-seals the chain from the first changed row and
//!   writes an `audit_rechain` anchor holding the old and the new head hash.
//!   [`Store::audit_chain_verify`] passes after a scrub; a copy of the old head
//!   held outside the database can be mapped to the new one through the anchor.

use crate::erasure::PersonIds;
use crate::{Store, StoreError, q};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Pseudonym algorithms an administrator can choose.
pub const ALGORITHMS: [&str; 2] = ["hmac-sha256-16", "hmac-sha256-32"];
/// Algorithm name recorded in anchors.
pub const DEFAULT_ALGORITHM: &str = "hmac-sha256-16";
/// Version tag of the chain hash input.
pub const CHAIN_VERSION: &str = "mw-audit-chain-v1";

/// A key generation as stored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyRow {
    pub generation: i64,
    pub sealed_key: Vec<u8>,
    pub created_at: i64,
    pub retired_at: Option<i64>,
    pub destroy_after: Option<i64>,
}

/// Public view of a key generation (never the secret).
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyInfo {
    pub generation: i64,
    pub created_at: i64,
    pub retired_at: Option<i64>,
    pub destroy_after: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct ScrubOptions {
    pub dry_run: bool,
    pub algorithm: String,
    /// Write a tombstone of scope `scrub` (not for the scrub that follows an
    /// erasure, which already has an `erasure` tombstone).
    pub tombstone: bool,
}

impl Default for ScrubOptions {
    fn default() -> Self {
        Self {
            dry_run: false,
            algorithm: DEFAULT_ALGORITHM.into(),
            tombstone: false,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrubReport {
    pub dry_run: bool,
    /// Rows changed (or that would change) per table.
    pub counts: BTreeMap<String, u64>,
    pub errors: Vec<String>,
    pub key_gen: i64,
    pub pseudonym: String,
    /// The chain anchor written, when the audit log was re-sealed.
    pub rechain_id: Option<String>,
    pub tombstone_seq: Option<i64>,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainReport {
    pub rows: u64,
    pub unsealed: u64,
    pub gaps: u64,
    pub ok: bool,
    pub first_bad_seq: Option<i64>,
    pub head_matches: bool,
    pub rechains: u64,
}

/// Lower-case and trim an address for hashing.
pub fn normalise(address: &str) -> String {
    address.trim().to_ascii_lowercase()
}

fn mac(key: &[u8; 32], label: &str, value: &str) -> [u8; 32] {
    let mut msg = Vec::new();
    msg.extend_from_slice(label.as_bytes());
    msg.push(0);
    msg.extend_from_slice(value.as_bytes());
    crate::seal::hmac_sha256(key, &msg)
}

/// Full hex keyed hash of an address: what a tombstone stores.
pub fn subject_hash(key: &[u8; 32], address: &str) -> String {
    hex(&mac(key, "subject", &normalise(address)))
}

/// The stable pseudonym of a value under one key generation.
pub fn pseudonym(key: &[u8; 32], generation: i64, kind: &str, value: &str, algorithm: &str) -> String {
    let len = if algorithm == "hmac-sha256-32" { 32 } else { 16 };
    let norm = if kind == "person" { normalise(value) } else { value.to_string() };
    let h = hex(&mac(key, kind, &norm));
    format!("scrubbed-{generation}-{}", &h[..len])
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn like_escape(s: &str) -> String {
    let mut out = String::from("%");
    for c in s.chars() {
        if matches!(c, '%' | '_' | '!') {
            out.push('!');
        }
        out.push(c);
    }
    out.push('%');
    out
}

/// ASCII case-insensitive replacement of every needle in `text`.
pub fn replace_needles(text: &str, needles: &[(String, String)]) -> String {
    let mut out = text.to_string();
    for (needle, to) in needles {
        if needle.is_empty() {
            continue;
        }
        let nl = needle.to_ascii_lowercase();
        let mut res = String::with_capacity(out.len());
        let lower = out.to_ascii_lowercase();
        let mut at = 0;
        while let Some(i) = lower[at..].find(&nl) {
            res.push_str(&out[at..at + i]);
            res.push_str(to);
            at += i + nl.len();
        }
        res.push_str(&out[at..]);
        out = res;
    }
    out
}

struct Target {
    table: &'static str,
    select: &'static str,
    update: &'static str,
    /// Text columns the SELECT returns after `id`, in order.
    cols: &'static [&'static str],
    ip_col: Option<&'static str>,
}

const TARGETS: &[Target] = &[
    Target {
        table: "assist_audit",
        select: "SELECT id,actor FROM assist_audit WHERE actor LIKE ?1 ESCAPE '!'",
        update: "UPDATE assist_audit SET actor=?2 WHERE id=?1",
        cols: &["actor"],
        ip_col: None,
    },
    Target {
        table: "alert_event",
        select: "SELECT id,summary,delivered_json FROM alert_event WHERE summary LIKE ?1 ESCAPE '!' OR delivered_json LIKE ?1 ESCAPE '!'",
        update: "UPDATE alert_event SET summary=?2,delivered_json=?3 WHERE id=?1",
        cols: &["summary", "delivered_json"],
        ip_col: None,
    },
    Target {
        table: "breach_delivery",
        select: "SELECT id,recipient,detail FROM breach_delivery WHERE recipient LIKE ?1 ESCAPE '!' OR detail LIKE ?1 ESCAPE '!'",
        update: "UPDATE breach_delivery SET recipient=?2,detail=?3 WHERE id=?1",
        cols: &["recipient", "detail"],
        ip_col: None,
    },
    Target {
        table: "breach_event",
        select: "SELECT id,actor,detail_json FROM breach_event WHERE actor LIKE ?1 ESCAPE '!' OR detail_json LIKE ?1 ESCAPE '!'",
        update: "UPDATE breach_event SET actor=?2,detail_json=?3 WHERE id=?1",
        cols: &["actor", "detail_json"],
        ip_col: None,
    },
    Target {
        table: "signin_event",
        select: "SELECT id,username_attempted,ip FROM signin_event WHERE username_attempted LIKE ?1 ESCAPE '!'",
        update: "UPDATE signin_event SET username_attempted=?2,ip=?3 WHERE id=?1",
        cols: &["username_attempted", "ip"],
        ip_col: Some("ip"),
    },
];

impl Store {
    // ---- keys ----------------------------------------------------------------

    /// The newest key generation, created on first use.
    pub async fn scrub_key_current(&self, now: i64) -> Result<(i64, [u8; 32]), StoreError> {
        if let Some(generation) = q("SELECT COALESCE(MAX(gen),0) FROM scrub_key WHERE retired_at IS NULL")
            .fetch_scalar_i64(&self.backend)
            .await
            .ok()
            .filter(|g| *g > 0)
            && let Some(k) = self.scrub_key_get(generation).await?
        {
            return Ok((generation, k));
        }
        let generation = self.scrub_rotate(now, None).await?;
        let k = self
            .scrub_key_get(generation)
            .await?
            .ok_or_else(|| StoreError::Corrupt("scrub key missing after creation".into()))?;
        Ok((generation, k))
    }

    pub async fn scrub_key_get(&self, generation: i64) -> Result<Option<[u8; 32]>, StoreError> {
        let Some(r) = q("SELECT sealed_key FROM scrub_key WHERE gen=?1")
            .bind(generation)
            .fetch_optional(&self.backend)
            .await?
        else {
            return Ok(None);
        };
        let plain = self.key.open(&r.get_blob("sealed_key"))?;
        let arr: [u8; 32] = plain
            .try_into()
            .map_err(|_| StoreError::Corrupt("scrub key has the wrong length".into()))?;
        Ok(Some(arr))
    }

    /// Add a new generation and retire the previous ones. A retired generation is
    /// destroyed `retain_days` later (`None` keeps it).
    pub async fn scrub_rotate(&self, now: i64, retain_days: Option<i64>) -> Result<i64, StoreError> {
        let next = q("SELECT COALESCE(MAX(gen),0) FROM scrub_key")
            .fetch_scalar_i64(&self.backend)
            .await?
            + 1;
        let secret: [u8; 32] = rand::random();
        q("UPDATE scrub_key SET retired_at=?1, destroy_after=?2 WHERE retired_at IS NULL")
            .bind(now)
            .bind(retain_days.filter(|d| *d > 0).map(|d| now + d * 86_400))
            .execute(&self.backend)
            .await?;
        q("INSERT INTO scrub_key(gen,sealed_key,created_at,retired_at,destroy_after) VALUES(?1,?2,?3,NULL,NULL)")
            .bind(next)
            .bind(self.key.seal(&secret)?)
            .bind(now)
            .execute(&self.backend)
            .await?;
        Ok(next)
    }

    /// Delete retired generations whose `destroy_after` has passed.
    pub async fn scrub_destroy_expired_keys(&self, now: i64) -> Result<u64, StoreError> {
        Ok(q("DELETE FROM scrub_key WHERE retired_at IS NOT NULL AND destroy_after IS NOT NULL AND destroy_after<=?1")
            .bind(now)
            .execute(&self.backend)
            .await?)
    }

    pub async fn scrub_key_infos(&self) -> Result<Vec<KeyInfo>, StoreError> {
        Ok(q("SELECT gen,created_at,retired_at,destroy_after FROM scrub_key ORDER BY gen")
            .fetch_all(&self.backend)
            .await?
            .iter()
            .map(|r| KeyInfo {
                generation: r.get_i64("gen"),
                created_at: r.get_i64("created_at"),
                retired_at: r.get_opt_i64("retired_at"),
                destroy_after: r.get_opt_i64("destroy_after"),
            })
            .collect())
    }

    /// Key rows as stored, for carrying to a restored store.
    pub async fn scrub_key_rows(&self) -> Result<Vec<KeyRow>, StoreError> {
        Ok(q("SELECT gen,sealed_key,created_at,retired_at,destroy_after FROM scrub_key ORDER BY gen")
            .fetch_all(&self.backend)
            .await?
            .iter()
            .map(|r| KeyRow {
                generation: r.get_i64("gen"),
                sealed_key: r.get_blob("sealed_key"),
                created_at: r.get_i64("created_at"),
                retired_at: r.get_opt_i64("retired_at"),
                destroy_after: r.get_opt_i64("destroy_after"),
            })
            .collect())
    }

    /// Insert a key generation taken from another store sealed with the same
    /// server key; an existing generation is left alone.
    pub async fn scrub_key_import(&self, k: &KeyRow) -> Result<(), StoreError> {
        q("INSERT INTO scrub_key(gen,sealed_key,created_at,retired_at,destroy_after) VALUES(?1,?2,?3,?4,?5) ON CONFLICT DO NOTHING")
            .bind(k.generation)
            .bind(&k.sealed_key)
            .bind(k.created_at)
            .bind(k.retired_at)
            .bind(k.destroy_after)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Write the tombstone of an erasure or scrub of `address`.
    pub async fn erasure_record(
        &self,
        address: &str,
        scope: &str,
        request_id: Option<&str>,
        now: i64,
    ) -> Result<i64, StoreError> {
        let (generation, key) = self.scrub_key_current(now).await?;
        self.erasure_tombstone_add(&subject_hash(&key, address), generation, scope, request_id, now)
            .await
    }

    // ---- the scrub -------------------------------------------------------------

    /// Replace the person's identifiers in audit and log history.
    ///
    /// `extra_needles` are further plain identifiers (a username, an IdP
    /// subject); `hashed_needles` are hashes of identifiers that history stores
    /// hashed (the SSO subject hash). With `dry_run` nothing is written and the
    /// counts are the rows that would change.
    pub async fn scrub_person(
        &self,
        p: &PersonIds,
        extra_needles: &[String],
        hashed_needles: &[String],
        opts: &ScrubOptions,
        request_id: Option<&str>,
        now: i64,
    ) -> Result<ScrubReport, StoreError> {
        let (generation, key) = self.scrub_key_current(now).await?;
        let person = pseudonym(&key, generation, "person", &p.address, &opts.algorithm);
        let mut report = ScrubReport {
            dry_run: opts.dry_run,
            key_gen: generation,
            pseudonym: person.clone(),
            ..ScrubReport::default()
        };
        let mut raw: Vec<String> = Vec::new();
        raw.push(p.address.clone());
        raw.extend(p.account_ids.iter().cloned());
        raw.extend(extra_needles.iter().cloned());
        for id in &p.account_ids {
            if let Ok(users) = q("SELECT username FROM accounts WHERE id=?1")
                .bind(id)
                .fetch_all_scalar_string(&self.backend)
                .await
            {
                raw.extend(users);
            }
        }
        raw.retain(|s| s.trim().len() >= 4);
        raw.sort();
        raw.dedup();
        let needles: Vec<(String, String)> = raw.iter().map(|n| (n.clone(), person.clone())).collect();

        for t in TARGETS {
            for (needle, _) in &needles {
                let pat = like_escape(needle);
                let rows = match q(t.select).bind(&pat).fetch_all(&self.backend).await {
                    Ok(r) => r,
                    Err(e) => {
                        report.errors.push(format!("{}: {e}", t.table));
                        continue;
                    }
                };
                for r in rows {
                    let id = r.get_string("id");
                    let old: Vec<Option<String>> = t.cols.iter().map(|c| r.get_opt_string(c)).collect();
                    let mut new: Vec<Option<String>> = old
                        .iter()
                        .map(|v| v.as_ref().map(|s| replace_needles(s, &needles)))
                        .collect();
                    if let Some(ip) = t.ip_col
                        && let Some(i) = t.cols.iter().position(|c| *c == ip)
                        && let Some(v) = &old[i]
                        && !v.is_empty()
                        && !v.starts_with("scrubbed-")
                    {
                        new[i] = Some(pseudonym(&key, generation, "ip", v, &opts.algorithm));
                    }
                    if new == old {
                        continue;
                    }
                    *report.counts.entry(t.table.to_string()).or_insert(0) += 1;
                    if opts.dry_run {
                        continue;
                    }
                    let mut qq = q(t.update).bind(&id);
                    for v in &new {
                        qq = qq.bind(v.clone());
                    }
                    if let Err(e) = qq.execute(&self.backend).await {
                        report.errors.push(format!("{}: {e}", t.table));
                    }
                }
            }
        }
        // SSO subject hashes.
        for h in hashed_needles {
            let n = if opts.dry_run {
                q("SELECT COUNT(*) FROM sso_login_audit WHERE subject_hash=?1")
                    .bind(h)
                    .fetch_scalar_i64(&self.backend)
                    .await
                    .map(|n| n as u64)
            } else {
                q("UPDATE sso_login_audit SET subject_hash=?2 WHERE subject_hash=?1")
                    .bind(h)
                    .bind(&person)
                    .execute(&self.backend)
                    .await
            };
            match n {
                Ok(n) if n > 0 => *report.counts.entry("sso_login_audit".into()).or_insert(0) += n,
                Ok(_) => {}
                Err(e) => report.errors.push(format!("sso_login_audit: {e}")),
            }
        }
        // The hash-chained audit log.
        match self
            .scrub_audit_log(&needles, &key, generation, &person, opts, now)
            .await
        {
            Ok((n, anchor)) => {
                if n > 0 {
                    report.counts.insert("audit_log".into(), n);
                }
                report.rechain_id = anchor;
            }
            Err(e) => report.errors.push(format!("audit_log: {e}")),
        }
        if !opts.dry_run && opts.tombstone && report.errors.is_empty() {
            report.tombstone_seq = Some(
                self.erasure_tombstone_add(&subject_hash(&key, &p.address), generation, "scrub", request_id, now)
                    .await?,
            );
        }
        Ok(report)
    }

    // ---- the audit hash chain ----------------------------------------------------

    /// Give every audit row without a position a `seq` and chain hash, in time
    /// order. Returns the rows sealed.
    pub async fn audit_chain_seal(&self) -> Result<u64, StoreError> {
        let mut total = 0u64;
        loop {
            let head = q("SELECT last_id,hash FROM audit_chain_head WHERE id=1")
                .fetch_optional(&self.backend)
                .await?;
            let (mut last, mut prev) = match &head {
                Some(r) => (r.get_i64("last_id"), r.get_opt_string("hash")),
                None => (0, None),
            };
            let max_seq = q("SELECT COALESCE(MAX(seq),0) FROM audit_log")
                .fetch_scalar_i64(&self.backend)
                .await?;
            if max_seq > last {
                last = max_seq;
                prev = q("SELECT hash FROM audit_log WHERE seq=?1")
                    .bind(max_seq)
                    .fetch_opt_scalar_string(&self.backend)
                    .await?;
            }
            let rows = q(CHAIN_SELECT_UNSEALED).fetch_all(&self.backend).await?;
            if rows.is_empty() {
                return Ok(total);
            }
            for r in rows {
                last += 1;
                let mut row = chain_row(&r);
                row.seq = last;
                row.prev_hash = prev.clone();
                let hash = chain_hash(&row);
                q("UPDATE audit_log SET seq=?2,prev_hash=?3,hash=?4 WHERE id=?1 AND seq IS NULL")
                    .bind(&row.id)
                    .bind(last)
                    .bind(row.prev_hash.clone())
                    .bind(&hash)
                    .execute(&self.backend)
                    .await?;
                prev = Some(hash);
                total += 1;
            }
            q("INSERT INTO audit_chain_head(id,last_id,hash) VALUES(1,?1,?2) ON CONFLICT(id) DO UPDATE SET last_id=excluded.last_id, hash=excluded.hash")
                .bind(last)
                .bind(prev.clone())
                .execute(&self.backend)
                .await?;
        }
    }

    /// Check the chain: every sealed row's hash recomputes from its stored
    /// fields, consecutive rows link, and the last row is the stored head.
    pub async fn audit_chain_verify(&self) -> Result<ChainReport, StoreError> {
        let mut rep = ChainReport {
            unsealed: q("SELECT COUNT(*) FROM audit_log WHERE seq IS NULL")
                .fetch_scalar_i64(&self.backend)
                .await? as u64,
            rechains: q("SELECT COUNT(*) FROM audit_rechain")
                .fetch_scalar_i64(&self.backend)
                .await? as u64,
            ..ChainReport::default()
        };
        let rows = q(CHAIN_SELECT_SEALED).fetch_all(&self.backend).await?;
        let mut prev: Option<(i64, String)> = None;
        let mut bad = None;
        for r in &rows {
            let row = chain_row(r);
            rep.rows += 1;
            if chain_hash(&row) != row.hash.clone().unwrap_or_default() {
                bad.get_or_insert(row.seq);
            }
            match &prev {
                Some((s, h)) if *s + 1 == row.seq => {
                    if row.prev_hash.as_deref() != Some(h.as_str()) {
                        bad.get_or_insert(row.seq);
                    }
                }
                Some(_) => rep.gaps += 1,
                None => {}
            }
            prev = Some((row.seq, row.hash.clone().unwrap_or_default()));
        }
        let head = q("SELECT last_id,hash FROM audit_chain_head WHERE id=1")
            .fetch_optional(&self.backend)
            .await?;
        rep.head_matches = match (&head, &prev) {
            (Some(h), Some((seq, hash))) => {
                h.get_i64("last_id") == *seq && h.get_opt_string("hash").as_deref() == Some(hash.as_str())
            }
            (None, None) => true,
            // Rows exist but were never sealed into a head: nothing to compare.
            (None, Some(_)) => false,
            (Some(h), None) => h.get_i64("last_id") == 0,
        };
        rep.first_bad_seq = bad;
        rep.ok = bad.is_none() && rep.head_matches;
        Ok(rep)
    }

    /// Pseudonymise audit rows that name the person and re-seal the chain.
    async fn scrub_audit_log(
        &self,
        needles: &[(String, String)],
        key: &[u8; 32],
        generation: i64,
        scrub_id: &str,
        opts: &ScrubOptions,
        now: i64,
    ) -> Result<(u64, Option<String>), StoreError> {
        if !opts.dry_run {
            self.audit_chain_seal().await?;
        }
        let mut changed: BTreeMap<i64, ChainRow> = BTreeMap::new();
        let mut seen: std::collections::BTreeSet<String> = Default::default();
        for (needle, _) in needles {
            let pat = like_escape(needle);
            let rows = q(CHAIN_SELECT_MATCHING).bind(&pat).fetch_all(&self.backend).await?;
            for r in rows {
                let mut row = chain_row(&r);
                if !seen.insert(row.id.clone()) {
                    continue;
                }
                let before = row.clone();
                let actor_hit = needles.iter().any(|(n, _)| row.actor.to_ascii_lowercase().contains(&n.to_ascii_lowercase()));
                row.actor = replace_needles(&row.actor, needles);
                row.target = row.target.map(|t| replace_needles(&t, needles));
                row.detail_json = replace_needles(&row.detail_json, needles);
                row.before_json = row.before_json.map(|t| replace_needles(&t, needles));
                row.after_json = row.after_json.map(|t| replace_needles(&t, needles));
                if actor_hit
                    && let Some(ip) = &row.ip
                    && !ip.is_empty()
                    && !ip.starts_with("scrubbed-")
                {
                    row.ip = Some(pseudonym(key, generation, "ip", ip, &opts.algorithm));
                }
                if row != before {
                    changed.insert(row.seq, row);
                }
            }
        }
        let n = changed.len() as u64;
        if n == 0 || opts.dry_run {
            return Ok((n, None));
        }
        for row in changed.values() {
            q("UPDATE audit_log SET actor=?2,target=?3,detail_json=?4,ip=?5,before_json=?6,after_json=?7 WHERE id=?1")
                .bind(&row.id)
                .bind(&row.actor)
                .bind(row.target.clone())
                .bind(&row.detail_json)
                .bind(row.ip.clone())
                .bind(row.before_json.clone())
                .bind(row.after_json.clone())
                .execute(&self.backend)
                .await?;
        }
        let from = *changed.keys().next().unwrap_or(&0);
        let old_head = q("SELECT hash FROM audit_chain_head WHERE id=1")
            .fetch_opt_scalar_string(&self.backend)
            .await?;
        // Re-seal from the first changed row to the end.
        let rows = q(CHAIN_SELECT_FROM).bind(from).fetch_all(&self.backend).await?;
        let mut prev: Option<(i64, String)> = None;
        let mut last_seq = from;
        let mut new_head = None;
        for r in &rows {
            let mut row = chain_row(r);
            if let Some((s, h)) = &prev
                && *s + 1 == row.seq
            {
                row.prev_hash = Some(h.clone());
            }
            let hash = chain_hash(&row);
            q("UPDATE audit_log SET prev_hash=?2,hash=?3 WHERE id=?1")
                .bind(&row.id)
                .bind(row.prev_hash.clone())
                .bind(&hash)
                .execute(&self.backend)
                .await?;
            last_seq = row.seq;
            prev = Some((row.seq, hash.clone()));
            new_head = Some(hash);
        }
        q("INSERT INTO audit_chain_head(id,last_id,hash) VALUES(1,?1,?2) ON CONFLICT(id) DO UPDATE SET last_id=excluded.last_id, hash=excluded.hash")
            .bind(last_seq)
            .bind(new_head.clone())
            .execute(&self.backend)
            .await?;
        let anchor = uuid::Uuid::new_v4().to_string();
        q("INSERT INTO audit_rechain(id,ts,scrub_id,from_seq,to_seq,old_head_hash,new_head_hash,rows_changed,algorithm,key_gen) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)")
            .bind(&anchor)
            .bind(now)
            .bind(scrub_id)
            .bind(from)
            .bind(last_seq)
            .bind(old_head)
            .bind(new_head)
            .bind(n as i64)
            .bind(opts.algorithm.as_str())
            .bind(generation)
            .execute(&self.backend)
            .await?;
        Ok((n, Some(anchor)))
    }

    /// Chain anchors written by scrubs, newest first.
    pub async fn audit_rechains(&self, limit: i64) -> Result<Vec<Value>, StoreError> {
        Ok(q("SELECT id,ts,scrub_id,from_seq,to_seq,old_head_hash,new_head_hash,rows_changed,algorithm,key_gen FROM audit_rechain ORDER BY ts DESC,id LIMIT ?1")
            .bind(limit.clamp(1, 1000))
            .fetch_all(&self.backend)
            .await?
            .iter()
            .map(|r| {
                json!({
                    "id": r.get_string("id"), "ts": r.get_i64("ts"), "scrubId": r.get_string("scrub_id"),
                    "fromSeq": r.get_i64("from_seq"), "toSeq": r.get_i64("to_seq"),
                    "oldHeadHash": r.get_opt_string("old_head_hash"), "newHeadHash": r.get_opt_string("new_head_hash"),
                    "rowsChanged": r.get_i64("rows_changed"), "algorithm": r.get_string("algorithm"),
                    "keyGen": r.get_i64("key_gen"),
                })
            })
            .collect())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ChainRow {
    seq: i64,
    id: String,
    ts: String,
    actor: String,
    actor_kind: String,
    action: String,
    target: Option<String>,
    detail_json: String,
    ip: Option<String>,
    admin_role: Option<String>,
    capability: Option<String>,
    category: Option<String>,
    request_id: Option<String>,
    before_json: Option<String>,
    after_json: Option<String>,
    prev_hash: Option<String>,
    hash: Option<String>,
}

const CHAIN_COLS: &str = "id,ts,actor,actor_kind,action,target,detail_json,ip,admin_role,capability,category,request_id,before_json,after_json,prev_hash,hash,seq";
const CHAIN_SELECT_UNSEALED: &str = "SELECT id,ts,actor,actor_kind,action,target,detail_json,ip,admin_role,capability,category,request_id,before_json,after_json,prev_hash,hash,seq FROM audit_log WHERE seq IS NULL ORDER BY ts,id LIMIT 5000";
const CHAIN_SELECT_SEALED: &str = "SELECT id,ts,actor,actor_kind,action,target,detail_json,ip,admin_role,capability,category,request_id,before_json,after_json,prev_hash,hash,seq FROM audit_log WHERE seq IS NOT NULL ORDER BY seq";
const CHAIN_SELECT_FROM: &str = "SELECT id,ts,actor,actor_kind,action,target,detail_json,ip,admin_role,capability,category,request_id,before_json,after_json,prev_hash,hash,seq FROM audit_log WHERE seq>=?1 ORDER BY seq";
const CHAIN_SELECT_MATCHING: &str = "SELECT id,ts,actor,actor_kind,action,target,detail_json,ip,admin_role,capability,category,request_id,before_json,after_json,prev_hash,hash,seq FROM audit_log WHERE seq IS NOT NULL AND (actor LIKE ?1 ESCAPE '!' OR target LIKE ?1 ESCAPE '!' OR detail_json LIKE ?1 ESCAPE '!' OR before_json LIKE ?1 ESCAPE '!' OR after_json LIKE ?1 ESCAPE '!') ORDER BY seq";

fn chain_row(r: &crate::backend::Row) -> ChainRow {
    let _ = CHAIN_COLS;
    ChainRow {
        seq: r.get_opt_i64("seq").unwrap_or(0),
        id: r.get_string("id"),
        ts: r.get_string("ts"),
        actor: r.get_string("actor"),
        actor_kind: r.get_string("actor_kind"),
        action: r.get_string("action"),
        target: r.get_opt_string("target"),
        detail_json: r.get_string("detail_json"),
        ip: r.get_opt_string("ip"),
        admin_role: r.get_opt_string("admin_role"),
        capability: r.get_opt_string("capability"),
        category: r.get_opt_string("category"),
        request_id: r.get_opt_string("request_id"),
        before_json: r.get_opt_string("before_json"),
        after_json: r.get_opt_string("after_json"),
        prev_hash: r.get_opt_string("prev_hash"),
        hash: r.get_opt_string("hash"),
    }
}

/// SHA-256 over the version tag, the previous hash, the position and every
/// content field, each length-prefixed (an absent value differs from an empty one).
fn chain_hash(r: &ChainRow) -> String {
    let mut h = Sha256::new();
    let mut put = |v: Option<&str>| match v {
        Some(s) => {
            h.update([1u8]);
            h.update((s.len() as u64).to_be_bytes());
            h.update(s.as_bytes());
        }
        None => h.update([0u8]),
    };
    put(Some(CHAIN_VERSION));
    put(r.prev_hash.as_deref());
    put(Some(&r.seq.to_string()));
    put(Some(&r.id));
    put(Some(&r.ts));
    put(Some(&r.actor));
    put(Some(&r.actor_kind));
    put(Some(&r.action));
    put(r.target.as_deref());
    put(Some(&r.detail_json));
    put(r.ip.as_deref());
    put(r.admin_role.as_deref());
    put(r.capability.as_deref());
    put(r.category.as_deref());
    put(r.request_id.as_deref());
    put(r.before_json.as_deref());
    put(r.after_json.as_deref());
    hex(&h.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pseudonym_is_stable_keyed_and_case_blind() {
        let k1 = [1u8; 32];
        let k2 = [2u8; 32];
        let a = pseudonym(&k1, 1, "person", "Ana@Example.org", DEFAULT_ALGORITHM);
        assert_eq!(a, pseudonym(&k1, 1, "person", " ana@example.org ", DEFAULT_ALGORITHM));
        assert_ne!(a, pseudonym(&k2, 1, "person", "ana@example.org", DEFAULT_ALGORITHM));
        assert!(a.starts_with("scrubbed-1-") && a.len() == "scrubbed-1-".len() + 16);
        assert!(!a.contains("ana"));
        assert_eq!(pseudonym(&k1, 1, "person", "x@y.z", "hmac-sha256-32").len(), "scrubbed-1-".len() + 32);
    }

    #[test]
    fn needles_replace_case_insensitively() {
        let n = vec![("ana@example.org".to_string(), "P".to_string())];
        assert_eq!(replace_needles("a ANA@example.org b ana@example.org", &n), "a P b P");
        assert_eq!(like_escape("a_b%c"), "%a!_b!%c%");
    }
}
