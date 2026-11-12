//! Archive tier of the long retention class.
//!
//! Aged rows of a few security-related stores move from the hot table into
//! sealed, compressed segment files. A segment is gzip-compressed JSON lines: one
//! header line, then one object per row. The database keeps one metadata row per
//! segment and an index from row key to segment, so a lookup opens one file.
//!
//! * **Append only.** A segment file is written once, under a temporary name,
//!   and renamed. Nothing rewrites it.
//! * **Sealed and chained.** `seal = HMAC-SHA256(key, prev_seal | content_hash |
//!   counters)`, where `content_hash` is the SHA-256 of the file and `prev_seal`
//!   is the previous segment's seal for the same store (`genesis` for the first).
//!   Editing a file, removing a segment's metadata or reordering segments is
//!   found by [`Store::archive_verify_all`]. The key is derived from the server
//!   key and the segment's `key_gen`, so rotating the generation changes the key
//!   for new segments while old segments still verify.
//! * **Safe order.** Rows leave the hot table only after the file is renamed and
//!   its metadata and index are committed. A crash between the two repeats the
//!   delete on the next run; rows already in the index are not archived twice.
//! * **Restore, search, export.** A segment can be restored into its table
//!   (rows that are present are skipped), searched by row key or account, and
//!   exported as plain JSON lines.
//! * **Purging** a segment removes its file and index rows and keeps the
//!   metadata row (state `purged`), so the chain stays checkable.

use crate::backend::q;
use crate::jobs::retention::{Age, stmt, text_cutoff};
use crate::seal::hmac_sha256;
use crate::{Store, StoreError};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// The `prev_seal` of a store's first segment.
pub const GENESIS: &str = "genesis";

fn segment_path(dir:&Path,location:&str)->Result<PathBuf,StoreError>{
    let mut components=Path::new(location).components();
    if !matches!(components.next(),Some(std::path::Component::Normal(_)))||components.next().is_some(){return Err(StoreError::Invalid("archive filename must stay in its directory".into()));}
    let path=dir.join(location);
    if let Ok(metadata)=std::fs::symlink_metadata(&path){if !metadata.file_type().is_file(){return Err(StoreError::Invalid("archive segment must be a regular file".into()));}}
    Ok(path)
}

/// One column of an archived table.
#[derive(Debug, Clone, Copy)]
pub struct Col {
    pub name: &'static str,
    pub int: bool,
}

const fn t(name: &'static str) -> Col {
    Col { name, int: false }
}
const fn n(name: &'static str) -> Col {
    Col { name, int: true }
}

/// A store whose aged rows can be archived.
#[derive(Debug, Clone, Copy)]
pub struct ArchiveTable {
    pub id: &'static str,
    pub table: &'static str,
    pub pk: &'static str,
    pub age: Age,
    pub cols: &'static [Col],
    pub account_col: Option<&'static str>,
}

/// Stores of the long class that can be archived. Hash-chained logs (the audit
/// log) keep their own checkpoint mechanism and are not listed.
pub const ARCHIVE_TABLES: &[ArchiveTable] = &[
    ArchiveTable {
        id: "passwordChangeAudit",
        table: "password_change_audit",
        pk: "id",
        age: Age::Text("ts"),
        cols: &[t("id"), t("ts"), t("account_id"), t("backend"), t("outcome")],
        account_col: Some("account_id"),
    },
    ArchiveTable {
        id: "ssoLoginAudit",
        table: "sso_login_audit",
        pk: "id",
        age: Age::Text("ts"),
        cols: &[t("id"), t("ts"), t("provider_id"), t("kind"), t("subject_hash"), t("outcome")],
        account_col: None,
    },
    ArchiveTable {
        id: "dlpAudit",
        table: "dlp_audit",
        pk: "id",
        age: Age::Text("at"),
        cols: &[
            t("id"),
            t("account_id"),
            t("at"),
            t("rule_id"),
            t("rule_name"),
            t("action"),
            t("matched_detectors_json"),
            n("blocked"),
        ],
        account_col: Some("account_id"),
    },
    ArchiveTable {
        id: "assistAudit",
        table: "assist_audit",
        pk: "id",
        age: Age::Text("ts"),
        cols: &[t("id"), t("ts"), t("actor"), t("capability"), t("scope_summary"), t("endpoint_host")],
        account_col: None,
    },
];

/// The archivable table with `id`.
pub fn archive_table(id: &str) -> Option<&'static ArchiveTable> {
    ARCHIVE_TABLES.iter().find(|a| a.id == id)
}

impl ArchiveTable {
    fn age_col(&self) -> &'static str {
        match self.age {
            Age::Text(c) | Age::Unix(c) => c,
            Age::RcHandle => "key",
        }
    }
    fn age_cond(&self) -> String {
        match self.age {
            Age::Text(c) => format!("substr({c}, 1, 19) < ?1"),
            Age::Unix(c) => format!("{c} < ?1"),
            Age::RcHandle => "1 = 0".into(),
        }
    }
    fn text_age(&self) -> bool {
        matches!(self.age, Age::Text(_))
    }
}

/// One segment's metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub id: String,
    pub store_id: String,
    pub seq: i64,
    pub row_count: i64,
    pub first_ts: i64,
    pub last_ts: i64,
    pub bytes: i64,
    pub content_hash: String,
    pub prev_seal: String,
    pub seal: String,
    pub key_gen: i64,
    pub location: String,
    pub state: String,
    pub created_at: i64,
    pub verified_at: i64,
    pub verify_status: String,
}

fn segment(r: &crate::backend::Row) -> Segment {
    Segment {
        id: r.get_string("id"),
        store_id: r.get_string("store_id"),
        seq: r.get_i64("seq"),
        row_count: r.get_i64("row_count"),
        first_ts: r.get_i64("first_ts"),
        last_ts: r.get_i64("last_ts"),
        bytes: r.get_i64("bytes"),
        content_hash: r.get_string("content_hash"),
        prev_seal: r.get_string("prev_seal"),
        seal: r.get_string("seal"),
        key_gen: r.get_i64("key_gen"),
        location: r.get_string("location"),
        state: r.get_string("state"),
        created_at: r.get_i64("created_at"),
        verified_at: r.get_i64("verified_at"),
        verify_status: r.get_string("verify_status"),
    }
}

const SEG_COLS: &str = "id,store_id,seq,row_count,first_ts,last_ts,bytes,content_hash,prev_seal,seal,key_gen,location,state,created_at,verified_at,verify_status";

/// What one archive pass did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Archived {
    /// Rows moved (or, for a dry run, rows that would be).
    pub rows: u64,
    pub segments: u64,
    /// The pass stopped at its batch limit with rows left.
    pub more: bool,
}

/// How verification ended for one segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Verdict {
    Ok,
    /// The file is gone or unreadable.
    Missing,
    /// The file's hash differs from the recorded one.
    ContentChanged,
    /// The recorded seal does not match the metadata (wrong key or edited row).
    SealMismatch,
    /// `prev_seal` does not equal the previous segment's seal.
    ChainBroken,
    /// Purged on purpose; metadata only.
    Purged,
}

impl Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Ok => "ok",
            Verdict::Missing => "missing",
            Verdict::ContentChanged => "contentChanged",
            Verdict::SealMismatch => "sealMismatch",
            Verdict::ChainBroken => "chainBroken",
            Verdict::Purged => "purged",
        }
    }
}

/// Result of [`Store::archive_verify_all`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyReport {
    pub checked: u64,
    pub ok: u64,
    /// Segment ids that failed, with the verdict.
    pub failed: Vec<(String, String)>,
}

/// A search hit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Hit {
    pub segment_id: String,
    pub row: Value,
}

fn sha_hex(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn io(e: std::io::Error) -> StoreError {
    StoreError::Corrupt(format!("archive file: {e}"))
}

fn ts_of(v: &Value, t: &ArchiveTable) -> i64 {
    let c = t.age_col();
    match &v[c] {
        Value::Number(n) => n.as_i64().unwrap_or(0),
        Value::String(s) => chrono::DateTime::parse_from_rfc3339(s)
            .map(|d| d.timestamp())
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&s[..s.len().min(19)], "%Y-%m-%dT%H:%M:%S")
                    .map(|d| d.and_utc().timestamp())
            })
            .unwrap_or(0),
        _ => 0,
    }
}

fn compress(plain: &[u8]) -> Result<Vec<u8>, StoreError> {
    let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    enc.write_all(plain).map_err(io)?;
    enc.finish().map_err(io)
}

fn decompress(bytes: &[u8]) -> Result<Vec<u8>, StoreError> {
    let mut out = Vec::new();
    flate2::read::GzDecoder::new(bytes).read_to_end(&mut out).map_err(io)?;
    Ok(out)
}

fn seal_message(s: &Segment) -> Vec<u8> {
    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        s.prev_seal, s.content_hash, s.store_id, s.seq, s.row_count, s.first_ts, s.last_ts, s.key_gen
    )
    .into_bytes()
}

impl Store {
    fn archive_seal(&self, s: &Segment) -> String {
        let key = self.key.derive(&format!("mw-retention-archive-seal-v1:{}", s.key_gen));
        hex(&hmac_sha256(&key, &seal_message(s)))
    }

    /// Segment metadata of one store, oldest first (all stores when `None`).
    pub async fn archive_segments(&self, store_id: Option<&str>) -> Result<Vec<Segment>, StoreError> {
        let rows = match store_id {
            Some(id) => {
                q(stmt(format!(
                    "SELECT {SEG_COLS} FROM retention_archive_segment WHERE store_id=?1 ORDER BY seq"
                )))
                .bind(id)
                .fetch_all(&self.backend)
                .await?
            }
            None => {
                q(stmt(format!(
                    "SELECT {SEG_COLS} FROM retention_archive_segment ORDER BY store_id, seq"
                )))
                .fetch_all(&self.backend)
                .await?
            }
        };
        Ok(rows.iter().map(segment).collect())
    }

    async fn archive_segment_by_id(&self, id: &str) -> Result<Option<Segment>, StoreError> {
        let r = q(stmt(format!("SELECT {SEG_COLS} FROM retention_archive_segment WHERE id=?1")))
            .bind(id)
            .fetch_optional(&self.backend)
            .await?;
        Ok(r.as_ref().map(segment))
    }

    async fn archive_indexed(&self, store_id: &str, key: &str) -> Result<bool, StoreError> {
        let r = q("SELECT 1 AS x FROM retention_archive_index WHERE store_id=?1 AND row_key=?2 LIMIT 1")
            .bind(store_id)
            .bind(key)
            .fetch_optional(&self.backend)
            .await?;
        Ok(r.is_some())
    }

    /// Move rows of `t` older than `cutoff` into sealed segments under `dir`,
    /// at most `batch` rows per segment and `max_segments` segments. `held` are
    /// account ids whose rows stay. With `dry_run` only the rows are counted.
    #[allow(clippy::too_many_arguments)]
    pub async fn archive_move(
        &self,
        t: &ArchiveTable,
        cutoff: i64,
        batch: i64,
        max_segments: u32,
        dir: &Path,
        key_gen: i64,
        now: i64,
        held: &[String],
        dry_run: bool,
    ) -> Result<Archived, StoreError> {
        let hold = held_clause(t, held);
        let cond = format!("{}{}", t.age_cond(), hold);
        let bind_cut = |s: crate::backend::Sql| {
            if t.text_age() { s.bind(text_cutoff(cutoff)) } else { s.bind(cutoff) }
        };
        if dry_run {
            let row = bind_cut(q(stmt(format!("SELECT COUNT(*) AS n FROM {} WHERE {cond}", t.table))))
                .fetch_one(&self.backend)
                .await?;
            return Ok(Archived { rows: row.get_i64("n").max(0) as u64, segments: 0, more: false });
        }
        let cols: Vec<&str> = t.cols.iter().map(|c| c.name).collect();
        let select = format!(
            "SELECT {} FROM {} WHERE {cond} ORDER BY {}, {} LIMIT ?2",
            cols.join(","),
            t.table,
            t.age_col(),
            t.pk
        );
        std::fs::create_dir_all(dir).map_err(io)?;
        let mut out = Archived::default();
        for i in 0..max_segments.max(1) {
            let rows = bind_cut(q(stmt(select.clone())))
                .bind(batch)
                .fetch_all(&self.backend)
                .await?;
            if rows.is_empty() {
                break;
            }
            let mut objs: Vec<Value> = Vec::with_capacity(rows.len());
            for r in &rows {
                let mut m = Map::new();
                for c in t.cols {
                    m.insert(
                        c.name.to_string(),
                        if c.int { json!(r.get_i64(c.name)) } else { json!(r.get_string(c.name)) },
                    );
                }
                objs.push(Value::Object(m));
            }
            let keys: Vec<String> = objs.iter().map(|o| o[t.pk].as_str().unwrap_or_default().to_string()).collect();
            // Rows an earlier interrupted pass already archived are only deleted.
            let mut fresh: Vec<&Value> = Vec::new();
            for (o, k) in objs.iter().zip(&keys) {
                if !self.archive_indexed(t.id, k).await? {
                    fresh.push(o);
                }
            }
            if !fresh.is_empty() {
                self.archive_write_segment(t, &fresh, dir, key_gen, now).await?;
                out.segments += 1;
            }
            for k in &keys {
                q(stmt(format!("DELETE FROM {} WHERE {} = ?1", t.table, t.pk)))
                    .bind(k.as_str())
                    .execute(&self.backend)
                    .await?;
            }
            out.rows += keys.len() as u64;
            if (rows.len() as i64) < batch {
                break;
            }
            if i + 1 == max_segments.max(1) {
                out.more = true;
            }
        }
        Ok(out)
    }

    async fn archive_write_segment(
        &self,
        t: &ArchiveTable,
        rows: &[&Value],
        dir: &Path,
        key_gen: i64,
        now: i64,
    ) -> Result<Segment, StoreError> {
        let prev = self.archive_segments(Some(t.id)).await?;
        let (seq, prev_seal) = prev
            .last()
            .map(|s| (s.seq + 1, s.seal.clone()))
            .unwrap_or((1, GENESIS.to_string()));
        let times: Vec<i64> = rows.iter().map(|r| ts_of(r, t)).collect();
        let first_ts = times.iter().copied().min().unwrap_or(0);
        let last_ts = times.iter().copied().max().unwrap_or(0);
        let mut plain = Vec::new();
        let header = json!({"v": 1, "store": t.id, "table": t.table, "seq": seq, "rows": rows.len(), "firstTs": first_ts, "lastTs": last_ts});
        plain.extend_from_slice(header.to_string().as_bytes());
        plain.push(b'\n');
        for r in rows {
            plain.extend_from_slice(r.to_string().as_bytes());
            plain.push(b'\n');
        }
        let id = uuid::Uuid::new_v4().to_string();
        let file = self.key.seal_bound(&compress(&plain)?, "retention-archive-file-v1", &[t.id, &id, &key_gen.to_string()])?;
        let name = format!("{}-{:08}-{}.mwarc", t.id, seq, &id[..8]);
        let path = dir.join(&name);
        let tmp = dir.join(format!("{name}.tmp"));
        std::fs::write(&tmp, &file).map_err(io)?;
        std::fs::rename(&tmp, &path).map_err(io)?;
        let mut seg = Segment {
            id: id.clone(),
            store_id: t.id.to_string(),
            seq,
            row_count: rows.len() as i64,
            first_ts,
            last_ts,
            bytes: file.len() as i64,
            content_hash: sha_hex(&file),
            prev_seal,
            seal: String::new(),
            key_gen,
            location: name,
            state: "sealed".into(),
            created_at: now,
            verified_at: now,
            verify_status: "ok".into(),
        };
        seg.seal = self.archive_seal(&seg);
        q("INSERT INTO retention_archive_segment(id,store_id,seq,row_count,first_ts,last_ts,bytes,content_hash,prev_seal,seal,key_gen,location,state,created_at,verified_at,verify_status) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)")
            .bind(seg.id.as_str())
            .bind(seg.store_id.as_str())
            .bind(seg.seq)
            .bind(seg.row_count)
            .bind(seg.first_ts)
            .bind(seg.last_ts)
            .bind(seg.bytes)
            .bind(seg.content_hash.as_str())
            .bind(seg.prev_seal.as_str())
            .bind(seg.seal.as_str())
            .bind(seg.key_gen)
            .bind(seg.location.as_str())
            .bind(seg.state.as_str())
            .bind(seg.created_at)
            .bind(seg.verified_at)
            .bind(seg.verify_status.as_str())
            .execute(&self.backend)
            .await?;
        for (r, ts) in rows.iter().zip(&times) {
            let key = r[t.pk].as_str().unwrap_or_default();
            let account = t.account_col.and_then(|c| r[c].as_str()).map(str::to_string);
            q("INSERT INTO retention_archive_index(id,segment_id,store_id,row_key,ts,account_id) VALUES(?1,?2,?3,?4,?5,?6)")
                .bind(format!("{}:{}", seg.id, key))
                .bind(seg.id.as_str())
                .bind(t.id)
                .bind(key)
                .bind(*ts)
                .bind(account)
                .execute(&self.backend)
                .await?;
        }
        Ok(seg)
    }

    /// Encrypt legacy compressed files without changing the logical content
    /// hash or integrity chain. Metadata switches only after ciphertext is synced.
    pub async fn archive_encrypt_legacy(&self,dir:&Path)->Result<u64,StoreError>{
        let segments=self.archive_segments(None).await?;let mut changed=0;
        for seg in segments{
            if seg.state=="purged"{continue;}
            let old=segment_path(dir,&seg.location)?;let bytes=std::fs::read(&old).map_err(io)?;
            if bytes.starts_with(b"MWB1"){continue;}
            if sha_hex(&bytes)!=seg.content_hash||self.archive_seal(&seg)!=seg.seal{return Err(StoreError::Corrupt("archive verification failed; migration refused".into()));}
            decompress(&bytes)?;
            let sealed=self.key.seal_bound(&bytes,"retention-archive-file-v1",&[&seg.store_id,&seg.id,&seg.key_gen.to_string()])?;
            let name=format!("{}.{}.enc",seg.location,uuid::Uuid::new_v4());
            let path=dir.join(&name);let mut file=std::fs::OpenOptions::new().create_new(true).write(true).open(&path).map_err(io)?;
            file.write_all(&sealed).map_err(io)?;file.sync_all().map_err(io)?;drop(file);
            let updated=q("UPDATE retention_archive_segment SET location=?1,bytes=?2 WHERE id=?3 AND location=?4 AND content_hash=?5")
                .bind(name).bind(sealed.len() as i64).bind(seg.id.as_str()).bind(seg.location.as_str()).bind(seg.content_hash.as_str()).execute(&self.backend).await?;
            if updated>0{std::fs::remove_file(old).map_err(io)?;changed+=1;}else{let _=std::fs::remove_file(path);}
        }
        Ok(changed)
    }
    fn archive_decode_file(&self,seg:&Segment,bytes:&[u8])->Result<Vec<u8>,StoreError>{
        let compressed=if bytes.starts_with(b"MWB1"){
            self.key.open_bound(bytes,"retention-archive-file-v1",&[&seg.store_id,&seg.id,&seg.key_gen.to_string()])?
        }else{bytes.to_vec()};
        // New segments hash ciphertext; migrated legacy segments retain their
        // original compressed-content hash so successors' seals remain valid.
        if sha_hex(bytes)!=seg.content_hash&&sha_hex(&compressed)!=seg.content_hash{return Err(StoreError::Corrupt("archive content hash mismatch".into()));}
        Ok(compressed)
    }

    /// Check one segment against its file and the segment before it.
    pub async fn archive_verify(&self, seg: &Segment, dir: &Path, prev: Option<&Segment>) -> Verdict {
        if seg.state == "purged" {
            return Verdict::Purged;
        }
        let expected_prev = prev.map(|p| p.seal.as_str()).unwrap_or(GENESIS);
        if seg.prev_seal != expected_prev {
            return Verdict::ChainBroken;
        }
        let Ok(path)=segment_path(dir,&seg.location) else{return Verdict::ContentChanged;};
        let Ok(bytes) = std::fs::read(path) else {
            return Verdict::Missing;
        };
        if self.archive_decode_file(seg,&bytes).is_err() {
            return Verdict::ContentChanged;
        }
        if self.archive_seal(seg) != seg.seal {
            return Verdict::SealMismatch;
        }
        if bytes.starts_with(b"MWB1") && self.key.open_bound(&bytes,"retention-archive-file-v1",&[&seg.store_id,&seg.id,&seg.key_gen.to_string()]).is_err() {
            return Verdict::ContentChanged;
        }
        Verdict::Ok
    }

    /// Verify every segment of every store (or one) and record the outcome.
    pub async fn archive_verify_all(
        &self,
        store_id: Option<&str>,
        dir: &Path,
        now: i64,
    ) -> Result<VerifyReport, StoreError> {
        let mut report = VerifyReport::default();
        let mut by_store: std::collections::BTreeMap<String, Vec<Segment>> = Default::default();
        for s in self.archive_segments(store_id).await? {
            by_store.entry(s.store_id.clone()).or_default().push(s);
        }
        for segs in by_store.values() {
            for (i, s) in segs.iter().enumerate() {
                let v = self.archive_verify(s, dir, i.checked_sub(1).map(|j| &segs[j])).await;
                if v == Verdict::Purged {
                    continue;
                }
                report.checked += 1;
                if v == Verdict::Ok {
                    report.ok += 1;
                } else {
                    report.failed.push((s.id.clone(), v.as_str().to_string()));
                }
                q("UPDATE retention_archive_segment SET verified_at=?1, verify_status=?2 WHERE id=?3")
                    .bind(now)
                    .bind(v.as_str())
                    .bind(s.id.as_str())
                    .execute(&self.backend)
                    .await?;
            }
        }
        Ok(report)
    }

    /// Rows of one segment, after its seal checks out.
    pub async fn archive_read(&self, segment_id: &str, dir: &Path) -> Result<Vec<Value>, StoreError> {
        let seg = self
            .archive_segment_by_id(segment_id)
            .await?
            .ok_or_else(|| StoreError::Invalid("no such segment".into()))?;
        if seg.state == "purged" {
            return Err(StoreError::Invalid("segment was purged".into()));
        }
        let bytes = std::fs::read(segment_path(dir,&seg.location)?).map_err(io)?;
        if self.archive_seal(&seg)!=seg.seal{return Err(StoreError::Corrupt("archive seal mismatch".into()));}
        let compressed=self.archive_decode_file(&seg,&bytes)?;
        let plain = decompress(&compressed)?;
        let text = String::from_utf8(plain).map_err(|_| StoreError::Invalid("segment is not UTF-8".into()))?;
        let mut rows = Vec::new();
        for (i, line) in text.lines().enumerate() {
            if i == 0 || line.is_empty() {
                continue;
            }
            rows.push(serde_json::from_str(line).map_err(|e| StoreError::Corrupt(e.to_string()))?);
        }
        Ok(rows)
    }

    /// Rows of one segment as plain JSON lines, for export.
    pub async fn archive_export(&self, segment_id: &str, dir: &Path) -> Result<String, StoreError> {
        let rows = self.archive_read(segment_id, dir).await?;
        Ok(rows.iter().map(|r| r.to_string() + "\n").collect())
    }

    /// Put a segment's rows back into the hot table; rows that exist are skipped.
    /// Returns the rows inserted.
    pub async fn archive_restore(&self, segment_id: &str, dir: &Path) -> Result<u64, StoreError> {
        let seg = self
            .archive_segment_by_id(segment_id)
            .await?
            .ok_or_else(|| StoreError::Invalid("no such segment".into()))?;
        let t = archive_table(&seg.store_id).ok_or_else(|| StoreError::Invalid("unknown store".into()))?;
        let rows = self.archive_read(segment_id, dir).await?;
        let names: Vec<&str> = t.cols.iter().map(|c| c.name).collect();
        let marks: Vec<String> = (1..=names.len()).map(|i| format!("?{i}")).collect();
        let insert = stmt(format!("INSERT INTO {}({}) VALUES({})", t.table, names.join(","), marks.join(",")));
        let exists = stmt(format!("SELECT 1 AS x FROM {} WHERE {} = ?1 LIMIT 1", t.table, t.pk));
        let mut n = 0;
        for r in &rows {
            let key = r[t.pk].as_str().unwrap_or_default();
            if q(exists).bind(key).fetch_optional(&self.backend).await?.is_some() {
                continue;
            }
            let mut s = q(insert);
            for c in t.cols {
                s = if c.int {
                    s.bind(r[c.name].as_i64().unwrap_or(0))
                } else {
                    s.bind(r[c.name].as_str().unwrap_or_default().to_string())
                };
            }
            s.execute(&self.backend).await?;
            n += 1;
        }
        Ok(n)
    }

    /// Rows of the archive that match a row key or an account id.
    pub async fn archive_search(
        &self,
        store_id: &str,
        row_key: Option<&str>,
        account_id: Option<&str>,
        limit: i64,
        dir: &Path,
    ) -> Result<Vec<Hit>, StoreError> {
        let idx = match (row_key, account_id) {
            (Some(k), _) => {
                q("SELECT segment_id,row_key FROM retention_archive_index WHERE store_id=?1 AND row_key=?2 ORDER BY ts LIMIT ?3")
                    .bind(store_id).bind(k).bind(limit.clamp(1, 500))
                    .fetch_all(&self.backend).await?
            }
            (None, Some(a)) => {
                q("SELECT segment_id,row_key FROM retention_archive_index WHERE store_id=?1 AND account_id=?2 ORDER BY ts LIMIT ?3")
                    .bind(store_id).bind(a).bind(limit.clamp(1, 500))
                    .fetch_all(&self.backend).await?
            }
            _ => return Ok(Vec::new()),
        };
        let t = archive_table(store_id).ok_or_else(|| StoreError::Invalid("unknown store".into()))?;
        let mut cache: std::collections::HashMap<String, Vec<Value>> = Default::default();
        let mut hits = Vec::new();
        for r in &idx {
            let seg = r.get_string("segment_id");
            let key = r.get_string("row_key");
            if !cache.contains_key(&seg) {
                cache.insert(seg.clone(), self.archive_read(&seg, dir).await?);
            }
            if let Some(row) = cache[&seg].iter().find(|v| v[t.pk].as_str() == Some(key.as_str())) {
                hits.push(Hit { segment_id: seg, row: row.clone() });
            }
        }
        Ok(hits)
    }

    /// Remove the files and index rows of segments whose newest row is older than
    /// `cutoff`; the metadata stays with state `purged`. With `dry_run` only the
    /// segments are counted. Returns segments purged.
    pub async fn archive_purge(&self, cutoff: i64, dir: &Path, dry_run: bool) -> Result<u64, StoreError> {
        let rows = q(stmt(format!(
            "SELECT {SEG_COLS} FROM retention_archive_segment WHERE state <> 'purged' AND last_ts < ?1 ORDER BY store_id, seq"
        )))
        .bind(cutoff)
        .fetch_all(&self.backend)
        .await?;
        if dry_run {
            return Ok(rows.len() as u64);
        }
        let mut n = 0;
        for r in &rows {
            let seg = segment(r);
            match std::fs::remove_file(segment_path(dir,&seg.location)?) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(io(e)),
            }
            q("DELETE FROM retention_archive_index WHERE segment_id=?1")
                .bind(seg.id.as_str())
                .execute(&self.backend)
                .await?;
            q("UPDATE retention_archive_segment SET state='purged' WHERE id=?1")
                .bind(seg.id.as_str())
                .execute(&self.backend)
                .await?;
            n += 1;
        }
        Ok(n)
    }

    /// Totals for the console: segments, rows, bytes of live segments, and the
    /// time of the oldest verification.
    pub async fn archive_totals(&self) -> Result<ArchiveTotals, StoreError> {
        let r = q("SELECT COUNT(*) AS segs, COALESCE(SUM(row_count),0) AS rows_n, COALESCE(SUM(bytes),0) AS bytes_n, COALESCE(MIN(verified_at),0) AS oldest, COALESCE(MIN(first_ts),0) AS first_ts FROM retention_archive_segment WHERE state <> 'purged'")
            .fetch_one(&self.backend)
            .await?;
        let bad = q("SELECT COUNT(*) AS n FROM retention_archive_segment WHERE state <> 'purged' AND verify_status <> 'ok'")
            .fetch_one(&self.backend)
            .await?;
        Ok(ArchiveTotals {
            segments: r.get_i64("segs"),
            rows: r.get_i64("rows_n"),
            bytes: r.get_i64("bytes_n"),
            oldest_verification: r.get_i64("oldest"),
            oldest_row: r.get_i64("first_ts"),
            failing: bad.get_i64("n"),
        })
    }

    /// Row counts of a store by age, for the age-distribution cards. `edges` are
    /// day boundaries in ascending order; the result has `edges.len() + 1`
    /// buckets (younger than the first edge, ..., older than the last).
    pub async fn retention_age_buckets(
        &self,
        table: &str,
        age: Age,
        now: i64,
        edges: &[i64],
    ) -> Result<Vec<i64>, StoreError> {
        let (col, text) = match age {
            Age::Text(c) => (c, true),
            Age::Unix(c) => (c, false),
            Age::RcHandle => return Ok(vec![0; edges.len() + 1]),
        };
        let mut counts = Vec::with_capacity(edges.len() + 1);
        let mut cumulative = Vec::new();
        // Rows older than each edge, then differences.
        for e in edges {
            let cut = now - e * 86_400;
            let sql = if text {
                format!("SELECT COUNT(*) AS n FROM {table} WHERE substr({col}, 1, 19) < ?1")
            } else {
                format!("SELECT COUNT(*) AS n FROM {table} WHERE {col} < ?1")
            };
            let s = q(stmt(sql));
            let r = if text { s.bind(text_cutoff(cut)) } else { s.bind(cut) }
                .fetch_one(&self.backend)
                .await?;
            cumulative.push(r.get_i64("n"));
        }
        let total = q(stmt(format!("SELECT COUNT(*) AS n FROM {table}")))
            .fetch_one(&self.backend)
            .await?
            .get_i64("n");
        // Bucket 0: younger than edge 0 = total - older(edge0).
        let mut older_prev = total;
        for older in &cumulative {
            counts.push((older_prev - older).max(0));
            older_prev = *older;
        }
        counts.push(older_prev.max(0));
        Ok(counts)
    }
}

/// Totals of the live archive.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveTotals {
    pub segments: i64,
    pub rows: i64,
    pub bytes: i64,
    /// Unix time of the least recently verified live segment, 0 with none.
    pub oldest_verification: i64,
    pub oldest_row: i64,
    pub failing: i64,
}

fn held_clause(t: &ArchiveTable, held: &[String]) -> String {
    let (Some(col), false) = (t.account_col, held.is_empty()) else {
        return String::new();
    };
    let ids: Vec<String> = held
        .iter()
        .filter(|id| {
            !id.is_empty()
                && id.len() <= 128
                && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        })
        .map(|id| format!("'{id}'"))
        .collect();
    if ids.is_empty() {
        return String::new();
    }
    format!(" AND {col} NOT IN ({})", ids.join(","))
}

/// A path the archive may use: relative or absolute, no parent segments.
pub fn directory_is_valid(dir: &str) -> bool {
    !dir.trim().is_empty() && !PathBuf::from(dir).components().any(|c| matches!(c, std::path::Component::ParentDir))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ServerKey, Store};

    async fn store() -> (Store, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("mw-arch-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let s = Store::open(dir.join("s.sqlite").to_str().unwrap(), ServerKey::from_bytes(&[5u8; 32]).unwrap())
            .await
            .unwrap();
        (s, dir)
    }

    async fn put(s: &Store, id: &str, ts: &str) {
        q("INSERT INTO assist_audit(id,ts,actor,capability,scope_summary,endpoint_host) VALUES(?1,?2,'a','c','s','h')")
            .bind(id)
            .bind(ts)
            .execute(&s.backend)
            .await
            .unwrap();
    }

    async fn count(s: &Store) -> i64 {
        q("SELECT COUNT(*) AS n FROM assist_audit").fetch_one(&s.backend).await.unwrap().get_i64("n")
    }

    #[tokio::test]
    async fn legacy_archive_encryption_preserves_chain_and_restore(){
        let (s,dir)=store().await;let arch=dir.join("archive");let table=archive_table("assistAudit").unwrap();
        put(&s,"legacy","2020-01-01T00:00:00Z").await;
        s.archive_move(table,1_700_000_000,10,10,&arch,1,1000,&[],false).await.unwrap();
        let mut segment=s.archive_segments(None).await.unwrap().remove(0);
        let path=arch.join(&segment.location);let ciphertext=std::fs::read(&path).unwrap();
        let plain=s.key.open_bound(&ciphertext,"retention-archive-file-v1",&[&segment.store_id,&segment.id,&segment.key_gen.to_string()]).unwrap();
        std::fs::write(&path,&plain).unwrap();segment.content_hash=sha_hex(&plain);segment.seal=s.archive_seal(&segment);
        q("UPDATE retention_archive_segment SET content_hash=?1,seal=?2 WHERE id=?3").bind(&segment.content_hash).bind(&segment.seal).bind(&segment.id).execute(&s.backend).await.unwrap();
        assert_eq!(s.archive_encrypt_legacy(&arch).await.unwrap(),1);assert!(!path.exists());
        assert_eq!(s.archive_encrypt_legacy(&arch).await.unwrap(),0);
        let migrated=s.archive_segments(None).await.unwrap().remove(0);assert_eq!(migrated.seal,segment.seal);assert_eq!(migrated.content_hash,segment.content_hash);
        assert_eq!(s.archive_verify_all(None,&arch,2000).await.unwrap().ok,1);
        assert_eq!(s.archive_restore(&migrated.id,&arch).await.unwrap(),1);
        assert!(dir.starts_with(std::env::temp_dir()));std::fs::remove_dir_all(dir).ok();
    }

    #[tokio::test]
    async fn archive_write_verify_tamper_restore() {
        let (s, dir) = store().await;
        let arch = dir.join("arch");
        let t = archive_table("assistAudit").unwrap();
        for i in 0..5 {
            put(&s, &format!("old{i}"), &format!("2020-01-0{}T00:00:00Z", i + 1)).await;
        }
        put(&s, "new", "2026-01-01T00:00:00Z").await;
        let cutoff = chrono::DateTime::parse_from_rfc3339("2021-01-01T00:00:00Z").unwrap().timestamp();

        let dry = s.archive_move(t, cutoff, 3, 10, &arch, 1, 1000, &[], true).await.unwrap();
        assert_eq!((dry.rows, count(&s).await), (5, 6), "dry run changes nothing");

        let done = s.archive_move(t, cutoff, 3, 10, &arch, 1, 1000, &[], false).await.unwrap();
        assert_eq!((done.rows, done.segments), (5, 2));
        assert_eq!(count(&s).await, 1);
        let again = s.archive_move(t, cutoff, 3, 10, &arch, 1, 1000, &[], false).await.unwrap();
        assert_eq!(again.rows, 0, "idempotent");

        let segs = s.archive_segments(Some("assistAudit")).await.unwrap();
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].prev_seal, GENESIS);
        assert_eq!(segs[1].prev_seal, segs[0].seal);
        let ok = s.archive_verify_all(None, &arch, 2000).await.unwrap();
        assert_eq!((ok.checked, ok.ok), (2, 2));

        // Ciphertext is the persistent format; gzip only exists in memory.
        for segment in &segs{let bytes=std::fs::read(arch.join(&segment.location)).unwrap();assert!(bytes.starts_with(b"MWB1"));assert!(decompress(&bytes).is_err());}

        // Search by key and export.
        let hits = s.archive_search("assistAudit", Some("old1"), None, 10, &arch).await.unwrap();
        assert_eq!(hits.len(), 1);
        let export = s.archive_export(&segs[0].id, &arch).await.unwrap();
        assert_eq!(export.lines().count(), 3);

        // Restore puts the rows back and skips any that exist.
        assert_eq!(s.archive_restore(&segs[0].id, &arch).await.unwrap(), 3);
        assert_eq!(s.archive_restore(&segs[0].id, &arch).await.unwrap(), 0);
        assert_eq!(count(&s).await, 4);

        // Tamper with the file: the hash no longer matches.
        let f = arch.join(&segs[1].location);
        let mut bytes = std::fs::read(&f).unwrap();
        let last = bytes.len() / 2;
        bytes[last] ^= 0xff;
        std::fs::write(&f, &bytes).unwrap();
        let bad = s.archive_verify_all(None, &arch, 3000).await.unwrap();
        assert_eq!(bad.failed, vec![(segs[1].id.clone(), "contentChanged".to_string())]);
        assert!(s.archive_read(&segs[1].id, &arch).await.is_err());

        // Editing the metadata is a seal mismatch; breaking the chain is found.
        q("UPDATE retention_archive_segment SET row_count=99 WHERE id=?1").bind(segs[0].id.as_str()).execute(&s.backend).await.unwrap();
        let bad = s.archive_verify_all(None, &arch, 4000).await.unwrap();
        assert!(bad.failed.iter().any(|(id, v)| id == &segs[0].id && v == "sealMismatch"));
        q("UPDATE retention_archive_segment SET row_count=3 WHERE id=?1").bind(segs[0].id.as_str()).execute(&s.backend).await.unwrap();
        q("UPDATE retention_archive_segment SET prev_seal='x' WHERE id=?1").bind(segs[1].id.as_str()).execute(&s.backend).await.unwrap();
        let bad = s.archive_verify_all(None, &arch, 5000).await.unwrap();
        assert!(bad.failed.iter().any(|(id, v)| id == &segs[1].id && v == "chainBroken"));

        // Purge keeps the metadata, removes the file.
        assert_eq!(s.archive_purge(i64::MAX, &arch, true).await.unwrap(), 2);
        assert_eq!(s.archive_purge(i64::MAX, &arch, false).await.unwrap(), 2);
        assert!(!arch.join(&segs[0].location).exists());
        assert_eq!(s.archive_purge(i64::MAX, &arch, false).await.unwrap(), 0);
        assert_eq!(s.archive_totals().await.unwrap().segments, 0);
        std::fs::remove_dir_all(dir).ok();
    }

    #[tokio::test]
    async fn key_generations_verify_independently() {
        let (s, dir) = store().await;
        let arch = dir.join("arch");
        let t = archive_table("assistAudit").unwrap();
        put(&s, "a", "2020-01-01T00:00:00Z").await;
        s.archive_move(t, 4_000_000_000, 10, 1, &arch, 1, 1, &[], false).await.unwrap();
        put(&s, "b", "2020-01-02T00:00:00Z").await;
        s.archive_move(t, 4_000_000_000, 10, 1, &arch, 2, 2, &[], false).await.unwrap();
        let segs = s.archive_segments(None).await.unwrap();
        assert_eq!((segs[0].key_gen, segs[1].key_gen), (1, 2));
        assert_ne!(s.archive_seal(&segs[0]), s.archive_seal(&Segment { key_gen: 2, ..segs[0].clone() }));
        assert_eq!(s.archive_verify_all(None, &arch, 3).await.unwrap().ok, 2);
        std::fs::remove_dir_all(dir).ok();
    }

    #[tokio::test]
    async fn age_buckets_split_by_edges() {
        let (s, dir) = store().await;
        let now = chrono::DateTime::parse_from_rfc3339("2026-06-01T00:00:00Z").unwrap().timestamp();
        put(&s, "1", "2026-05-30T00:00:00Z").await;
        put(&s, "2", "2026-04-01T00:00:00Z").await;
        put(&s, "3", "2020-01-01T00:00:00Z").await;
        let b = s.retention_age_buckets("assist_audit", Age::Text("ts"), now, &[30, 365]).await.unwrap();
        assert_eq!(b, vec![1, 1, 1]);
        std::fs::remove_dir_all(dir).ok();
    }
}
