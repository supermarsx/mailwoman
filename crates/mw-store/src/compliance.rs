//! Compliance documents: policy versions and overrides, operator-written
//! processing records, and the DPIA register with its history.
//!
//! These tables hold operator-maintained records and policy snapshots. Outbound
//! compliance deliveries also use this store, with ID-bound encrypted payloads
//! and explicit export/erasure and retention handling.
use crate::{Store, StoreError, backend::Row, q};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyVersion {
    pub id: String,
    pub domain: String,
    pub version: i64,
    /// `auto` (a setting changed) or `published` (an administrator published).
    pub kind: String,
    pub facts_json: String,
    pub changes_json: String,
    pub created_by: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyOverride {
    pub id: String,
    pub domain: String,
    pub lang: String,
    pub section: String,
    pub body: String,
    pub updated_by: String,
    pub updated_at: i64,
}

/// An operator-written processing activity (`kind` = `activity`) or
/// sub-processor (`kind` = `processor`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComplianceRecord {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub body_json: String,
    pub updated_by: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dpia {
    pub id: String,
    pub title: String,
    pub status: String,
    pub template: String,
    pub body_json: String,
    pub version: i64,
    pub review_at: Option<i64>,
    pub created_by: String,
    pub created_at: i64,
    pub updated_by: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DpiaVersion {
    pub id: String,
    pub dpia_id: String,
    pub version: i64,
    pub status: String,
    pub body_json: String,
    pub changed_by: String,
    pub created_at: i64,
}

/// Most records or assessments an administrator may keep.
pub const MAX_ROWS: usize = 1000;

macro_rules! dpia_cols { () => { "id,title,status,template,body_json,version,review_at,created_by,created_at,updated_by,updated_at" }; }
macro_rules! pver_cols { () => { "id,domain,version,kind,facts_json,changes_json,created_by,created_at" }; }

fn dpia(r: &Row) -> Dpia {
    Dpia {
        id: r.get_string("id"),
        title: r.get_string("title"),
        status: r.get_string("status"),
        template: r.get_string("template"),
        body_json: r.get_string("body_json"),
        version: r.get_i64("version"),
        review_at: r.get_opt_i64("review_at"),
        created_by: r.get_string("created_by"),
        created_at: r.get_i64("created_at"),
        updated_by: r.get_string("updated_by"),
        updated_at: r.get_i64("updated_at"),
    }
}

fn record(r: &Row) -> ComplianceRecord {
    ComplianceRecord {
        id: r.get_string("id"),
        kind: r.get_string("kind"),
        name: r.get_string("name"),
        body_json: r.get_string("body_json"),
        updated_by: r.get_string("updated_by"),
        updated_at: r.get_i64("updated_at"),
    }
}

fn pver(r: &Row) -> PolicyVersion {
    PolicyVersion {
        id: r.get_string("id"),
        domain: r.get_string("domain"),
        version: r.get_i64("version"),
        kind: r.get_string("kind"),
        facts_json: r.get_string("facts_json"),
        changes_json: r.get_string("changes_json"),
        created_by: r.get_string("created_by"),
        created_at: r.get_i64("created_at"),
    }
}

impl Store {
    /// The newest policy version of `domain`, if any.
    pub async fn latest_policy_version(&self, domain: &str) -> Result<Option<PolicyVersion>, StoreError> {
        let row = q(concat!("SELECT ", pver_cols!(), " FROM compliance_policy_version WHERE domain=?1 ORDER BY version DESC LIMIT 1"))
            .bind(domain)
            .fetch_optional(&self.backend)
            .await?;
        Ok(row.as_ref().map(pver))
    }

    /// Newest first, at most `limit`.
    pub async fn list_policy_versions(&self, domain: &str, limit: i64) -> Result<Vec<PolicyVersion>, StoreError> {
        let rows = q(concat!("SELECT ", pver_cols!(), " FROM compliance_policy_version WHERE domain=?1 ORDER BY version DESC LIMIT ?2"))
            .bind(domain)
            .bind(limit)
            .fetch_all(&self.backend)
            .await?;
        Ok(rows.iter().map(pver).collect())
    }

    /// Append a version. `version` is chosen by the caller; a clash with a
    /// concurrent writer returns an error and the caller reads again.
    pub async fn add_policy_version(&self, v: &PolicyVersion) -> Result<(), StoreError> {
        q("INSERT INTO compliance_policy_version(id,domain,version,kind,facts_json,changes_json,created_by,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)")
            .bind(v.id.as_str())
            .bind(v.domain.as_str())
            .bind(v.version)
            .bind(v.kind.as_str())
            .bind(v.facts_json.as_str())
            .bind(v.changes_json.as_str())
            .bind(v.created_by.as_str())
            .bind(v.created_at)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    pub async fn list_policy_overrides(&self, domain: &str) -> Result<Vec<PolicyOverride>, StoreError> {
        let rows = q("SELECT id,domain,lang,section,body,updated_by,updated_at FROM compliance_policy_override WHERE domain=?1 ORDER BY lang,section")
            .bind(domain)
            .fetch_all(&self.backend)
            .await?;
        Ok(rows
            .iter()
            .map(|r| PolicyOverride {
                id: r.get_string("id"),
                domain: r.get_string("domain"),
                lang: r.get_string("lang"),
                section: r.get_string("section"),
                body: r.get_string("body"),
                updated_by: r.get_string("updated_by"),
                updated_at: r.get_i64("updated_at"),
            })
            .collect())
    }

    pub async fn put_policy_override(&self, o: &PolicyOverride) -> Result<(), StoreError> {
        q("INSERT INTO compliance_policy_override(id,domain,lang,section,body,updated_by,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(domain,lang,section) DO UPDATE SET body=excluded.body,updated_by=excluded.updated_by,updated_at=excluded.updated_at")
            .bind(o.id.as_str())
            .bind(o.domain.as_str())
            .bind(o.lang.as_str())
            .bind(o.section.as_str())
            .bind(o.body.as_str())
            .bind(o.updated_by.as_str())
            .bind(o.updated_at)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    pub async fn delete_policy_override(&self, domain: &str, lang: &str, section: &str) -> Result<bool, StoreError> {
        Ok(q("DELETE FROM compliance_policy_override WHERE domain=?1 AND lang=?2 AND section=?3")
            .bind(domain)
            .bind(lang)
            .bind(section)
            .execute(&self.backend)
            .await?
            > 0)
    }

    pub async fn list_compliance_records(&self, kind: &str) -> Result<Vec<ComplianceRecord>, StoreError> {
        let rows = q("SELECT id,kind,name,body_json,updated_by,updated_at FROM compliance_record WHERE kind=?1 ORDER BY name,id")
            .bind(kind)
            .fetch_all(&self.backend)
            .await?;
        Ok(rows.iter().map(record).collect())
    }

    pub async fn get_compliance_record(&self, id: &str) -> Result<Option<ComplianceRecord>, StoreError> {
        let row = q("SELECT id,kind,name,body_json,updated_by,updated_at FROM compliance_record WHERE id=?1")
            .bind(id)
            .fetch_optional(&self.backend)
            .await?;
        Ok(row.as_ref().map(record))
    }

    pub async fn put_compliance_record(&self, r: &ComplianceRecord) -> Result<(), StoreError> {
        q("INSERT INTO compliance_record(id,kind,name,body_json,updated_by,updated_at) VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(id) DO UPDATE SET name=excluded.name,body_json=excluded.body_json,updated_by=excluded.updated_by,updated_at=excluded.updated_at")
            .bind(r.id.as_str())
            .bind(r.kind.as_str())
            .bind(r.name.as_str())
            .bind(r.body_json.as_str())
            .bind(r.updated_by.as_str())
            .bind(r.updated_at)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    pub async fn purge_compliance_delivery_history(&self,cutoff:i64)->Result<u64,StoreError>{
        Ok(q("DELETE FROM compliance_record WHERE id IN (SELECT id FROM compliance_record WHERE kind IN ('compliance-mail:sent','compliance-mail:failed','compliance-mail:dismissed') AND updated_at<?1 LIMIT 1000)").bind(cutoff).execute(&self.backend).await?)
    }

    /// Insert a durable delivery only once, including across server processes.
    pub async fn insert_compliance_record_once(&self,r:&ComplianceRecord)->Result<bool,StoreError>{
        Ok(q("INSERT INTO compliance_record(id,kind,name,body_json,updated_by,updated_at) VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(id) DO NOTHING")
            .bind(r.id.as_str()).bind(r.kind.as_str()).bind(r.name.as_str()).bind(r.body_json.as_str()).bind(r.updated_by.as_str()).bind(r.updated_at).execute(&self.backend).await?>0)
    }
    pub async fn due_compliance_deliveries(&self,kind:&str,at:i64)->Result<Vec<ComplianceRecord>,StoreError>{
        let rows=q("SELECT id,kind,name,body_json,updated_by,updated_at FROM compliance_record WHERE kind=?1 AND updated_at<=?2 ORDER BY updated_at,id LIMIT 100").bind(kind).bind(at).fetch_all(&self.backend).await?;
        Ok(rows.iter().map(record).collect())
    }
    /// Compare-and-swap claim/transition: two workers cannot send the same row.
    pub async fn transition_compliance_record(&self,old:&ComplianceRecord,new:&ComplianceRecord)->Result<bool,StoreError>{
        Ok(q("UPDATE compliance_record SET kind=?1,body_json=?2,updated_at=?3 WHERE id=?4 AND kind=?5 AND body_json=?6 AND updated_at=?7")
            .bind(new.kind.as_str()).bind(new.body_json.as_str()).bind(new.updated_at).bind(old.id.as_str()).bind(old.kind.as_str()).bind(old.body_json.as_str()).bind(old.updated_at).execute(&self.backend).await?>0)
    }
    pub fn seal_compliance_delivery(&self,id:&str,plain:&[u8])->Result<Vec<u8>,StoreError>{Ok(self.key.seal_bound(plain,"compliance-delivery-v1",&[id])?)}
    pub fn open_compliance_delivery(&self,id:&str,sealed:&[u8])->Result<Vec<u8>,StoreError>{Ok(self.key.open_bound(sealed,"compliance-delivery-v1",&[id])?)}

    pub async fn delete_compliance_record(&self, id: &str) -> Result<bool, StoreError> {
        Ok(q("DELETE FROM compliance_record WHERE id=?1").bind(id).execute(&self.backend).await? > 0)
    }

    pub async fn list_dpias(&self) -> Result<Vec<Dpia>, StoreError> {
        let rows = q(concat!("SELECT ", dpia_cols!(), " FROM dpia ORDER BY updated_at DESC, id"))
            .fetch_all(&self.backend)
            .await?;
        Ok(rows.iter().map(dpia).collect())
    }

    pub async fn get_dpia(&self, id: &str) -> Result<Option<Dpia>, StoreError> {
        let row = q(concat!("SELECT ", dpia_cols!(), " FROM dpia WHERE id=?1"))
            .bind(id)
            .fetch_optional(&self.backend)
            .await?;
        Ok(row.as_ref().map(dpia))
    }

    /// Insert or replace the assessment and append `version` to its history in one transaction.
    pub async fn save_dpia(&self, d: &Dpia) -> Result<(), StoreError> {
        let mut tx = self.backend.begin().await?;
        q("INSERT INTO dpia(id,title,status,template,body_json,version,review_at,created_by,created_at,updated_by,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11) ON CONFLICT(id) DO UPDATE SET title=excluded.title,status=excluded.status,body_json=excluded.body_json,version=excluded.version,review_at=excluded.review_at,updated_by=excluded.updated_by,updated_at=excluded.updated_at")
            .bind(d.id.as_str())
            .bind(d.title.as_str())
            .bind(d.status.as_str())
            .bind(d.template.as_str())
            .bind(d.body_json.as_str())
            .bind(d.version)
            .bind(d.review_at)
            .bind(d.created_by.as_str())
            .bind(d.created_at)
            .bind(d.updated_by.as_str())
            .bind(d.updated_at)
            .execute_tx(&mut tx)
            .await?;
        q("INSERT INTO dpia_version(id,dpia_id,version,status,body_json,changed_by,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7)")
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(d.id.as_str())
            .bind(d.version)
            .bind(d.status.as_str())
            .bind(d.body_json.as_str())
            .bind(d.updated_by.as_str())
            .bind(d.updated_at)
            .execute_tx(&mut tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn delete_dpia(&self, id: &str) -> Result<bool, StoreError> {
        let mut tx = self.backend.begin().await?;
        q("DELETE FROM dpia_version WHERE dpia_id=?1").bind(id).execute_tx(&mut tx).await?;
        let n = q("DELETE FROM dpia WHERE id=?1").bind(id).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(n > 0)
    }

    pub async fn list_dpia_versions(&self, dpia_id: &str) -> Result<Vec<DpiaVersion>, StoreError> {
        let rows = q("SELECT id,dpia_id,version,status,body_json,changed_by,created_at FROM dpia_version WHERE dpia_id=?1 ORDER BY version DESC")
            .bind(dpia_id)
            .fetch_all(&self.backend)
            .await?;
        Ok(rows
            .iter()
            .map(|r| DpiaVersion {
                id: r.get_string("id"),
                dpia_id: r.get_string("dpia_id"),
                version: r.get_i64("version"),
                status: r.get_string("status"),
                body_json: r.get_string("body_json"),
                changed_by: r.get_string("changed_by"),
                created_at: r.get_i64("created_at"),
            })
            .collect())
    }
}
