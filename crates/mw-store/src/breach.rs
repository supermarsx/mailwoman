//! Personal-data breach register: incidents, their trail and the delivery log of
//! data-subject notices.
//!
//! An incident row keeps the filterable columns and the clock; everything else
//! is in `body_json` and is changed with a compare-and-set on `rev`.
use crate::{Store, StoreError, q};
use serde::Serialize;
use serde_json::Value;

/// One incident as stored.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BreachRow {
    pub id: String,
    pub title: String,
    /// Empty for an incident that is not tied to one domain.
    pub domain: String,
    pub status: String,
    pub severity: String,
    pub source: String,
    pub detected_at: i64,
    pub aware_at: Option<i64>,
    pub deadline_at: Option<i64>,
    pub assignee: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub rev: i64,
    pub body_json: String,
}

/// One entry of an incident's trail.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BreachEvent {
    pub id: String,
    pub incident_id: String,
    pub ts: i64,
    pub actor: String,
    pub kind: String,
    pub detail: Value,
}

/// One delivery of a notice to one recipient on one channel.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BreachDelivery {
    pub id: String,
    pub incident_id: String,
    pub notice_id: String,
    pub recipient: String,
    pub channel: String,
    pub status: String,
    pub detail: String,
    pub ts: i64,
}

fn row_of(r: &crate::backend::Row) -> BreachRow {
    BreachRow {
        id: r.get_string("id"),
        title: r.get_string("title"),
        domain: r.get_string("domain"),
        status: r.get_string("status"),
        severity: r.get_string("severity"),
        source: r.get_string("source"),
        detected_at: r.get_i64("detected_at"),
        aware_at: r.get_opt_i64("aware_at"),
        deadline_at: r.get_opt_i64("deadline_at"),
        assignee: r.get_opt_string("assignee"),
        created_at: r.get_i64("created_at"),
        updated_at: r.get_i64("updated_at"),
        rev: r.get_i64("rev"),
        body_json: r.get_string("body_json"),
    }
}

impl Store {
    pub async fn breach_insert(&self, row: &BreachRow) -> Result<(), StoreError> {
        q("INSERT INTO breach_incident(id,title,domain,status,severity,source,detected_at,aware_at,deadline_at,assignee,created_at,updated_at,rev,body_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)")
            .bind(&row.id).bind(&row.title).bind(&row.domain).bind(&row.status)
            .bind(&row.severity).bind(&row.source).bind(row.detected_at)
            .bind(row.aware_at).bind(row.deadline_at).bind(row.assignee.clone())
            .bind(row.created_at).bind(row.updated_at).bind(row.rev).bind(&row.body_json)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    pub async fn breach_get(&self, id: &str) -> Result<Option<BreachRow>, StoreError> {
        Ok(q("SELECT id,title,domain,status,severity,source,detected_at,aware_at,deadline_at,assignee,created_at,updated_at,rev,body_json FROM breach_incident WHERE id=?1")
            .bind(id)
            .fetch_optional(&self.backend)
            .await?
            .as_ref()
            .map(row_of))
    }

    /// Write `row` when the stored `rev` is still `row.rev`; the stored rev
    /// becomes `row.rev + 1`. `false` when another writer got there first.
    pub async fn breach_update(&self, row: &BreachRow, now: i64) -> Result<bool, StoreError> {
        let n = q("UPDATE breach_incident SET title=?3,domain=?4,status=?5,severity=?6,source=?7,detected_at=?8,aware_at=?9,deadline_at=?10,assignee=?11,updated_at=?12,rev=?2+1,body_json=?13 WHERE id=?1 AND rev=?2")
            .bind(&row.id).bind(row.rev).bind(&row.title).bind(&row.domain)
            .bind(&row.status).bind(&row.severity).bind(&row.source).bind(row.detected_at)
            .bind(row.aware_at).bind(row.deadline_at).bind(row.assignee.clone())
            .bind(now).bind(&row.body_json)
            .execute(&self.backend)
            .await?;
        Ok(n == 1)
    }

    /// Newest detection first, at most 5000.
    pub async fn breach_list(&self) -> Result<Vec<BreachRow>, StoreError> {
        Ok(q("SELECT id,title,domain,status,severity,source,detected_at,aware_at,deadline_at,assignee,created_at,updated_at,rev,body_json FROM breach_incident ORDER BY detected_at DESC,id LIMIT 5000")
            .fetch_all(&self.backend)
            .await?
            .iter()
            .map(row_of)
            .collect())
    }

    pub async fn breach_event_add(
        &self,
        incident_id: &str,
        actor: &str,
        kind: &str,
        detail: &Value,
        now: i64,
    ) -> Result<(), StoreError> {
        q("INSERT INTO breach_event(id,incident_id,ts,actor,kind,detail_json) VALUES(?1,?2,?3,?4,?5,?6)")
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(incident_id)
            .bind(now)
            .bind(actor)
            .bind(kind)
            .bind(detail.to_string())
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    pub async fn breach_events(&self, incident_id: &str) -> Result<Vec<BreachEvent>, StoreError> {
        Ok(q("SELECT id,incident_id,ts,actor,kind,detail_json FROM breach_event WHERE incident_id=?1 ORDER BY ts,id LIMIT 5000")
            .bind(incident_id)
            .fetch_all(&self.backend)
            .await?
            .iter()
            .map(|r| BreachEvent {
                id: r.get_string("id"),
                incident_id: r.get_string("incident_id"),
                ts: r.get_i64("ts"),
                actor: r.get_string("actor"),
                kind: r.get_string("kind"),
                detail: serde_json::from_str(&r.get_string("detail_json")).unwrap_or(Value::Null),
            })
            .collect())
    }

    pub async fn breach_delivery_add(&self, d: &BreachDelivery) -> Result<(), StoreError> {
        q("INSERT INTO breach_delivery(id,incident_id,notice_id,recipient,channel,status,detail,ts) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)")
            .bind(&d.id).bind(&d.incident_id).bind(&d.notice_id).bind(&d.recipient)
            .bind(&d.channel).bind(&d.status).bind(&d.detail).bind(d.ts)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    pub async fn breach_deliveries(&self, incident_id: &str) -> Result<Vec<BreachDelivery>, StoreError> {
        Ok(q("SELECT id,incident_id,notice_id,recipient,channel,status,detail,ts FROM breach_delivery WHERE incident_id=?1 ORDER BY ts,id LIMIT 20000")
            .bind(incident_id)
            .fetch_all(&self.backend)
            .await?
            .iter()
            .map(|r| BreachDelivery {
                id: r.get_string("id"),
                incident_id: r.get_string("incident_id"),
                notice_id: r.get_string("notice_id"),
                recipient: r.get_string("recipient"),
                channel: r.get_string("channel"),
                status: r.get_string("status"),
                detail: r.get_string("detail"),
                ts: r.get_i64("ts"),
            })
            .collect())
    }

    /// Delete closed incidents last changed before `before`, with their trail and
    /// delivery log. Open incidents are never removed. Returns the incidents removed.
    pub async fn breach_purge_closed_before(&self, before: i64) -> Result<u64, StoreError> {
        let ids: Vec<String> = q("SELECT id FROM breach_incident WHERE status IN ('closed','not-notifiable') AND updated_at<?1")
            .bind(before)
            .fetch_all(&self.backend)
            .await?
            .iter()
            .map(|r| r.get_string("id"))
            .collect();
        for id in &ids {
            q("DELETE FROM breach_delivery WHERE incident_id=?1").bind(id.as_str()).execute(&self.backend).await?;
            q("DELETE FROM breach_event WHERE incident_id=?1").bind(id.as_str()).execute(&self.backend).await?;
            q("DELETE FROM breach_incident WHERE id=?1").bind(id.as_str()).execute(&self.backend).await?;
        }
        Ok(ids.len() as u64)
    }

    /// Composite cursor avoids losing security signals with equal timestamps.
    pub async fn operational_audit_after(&self,after:&str,id:&str,limit:i64)->Result<Vec<crate::AuditRow>,StoreError>{
        Ok(q("SELECT id,ts,actor,actor_kind,action,target,detail_json,ip FROM audit_log WHERE ts>?1 OR (ts=?1 AND id>?2) ORDER BY ts,id LIMIT ?3").bind(after).bind(id).bind(limit.clamp(1,1000)).fetch_all(&self.backend).await?.iter().map(|r|crate::AuditRow{id:r.get_string("id"),ts:r.get_string("ts"),actor:r.get_string("actor"),actor_kind:r.get_string("actor_kind"),action:r.get_string("action"),target:r.get_opt_string("target"),detail_json:r.get_string("detail_json"),ip:r.get_opt_string("ip")}).collect())
    }

    /// Audit rows newer than `after` (RFC 3339, exclusive), oldest first, for the
    /// breach automation's cursor.
    pub async fn breach_audit_after(&self, after: &str, limit: i64) -> Result<Vec<crate::AuditRow>, StoreError> {
        Ok(q("SELECT id, ts, actor, actor_kind, action, target, detail_json, ip FROM audit_log WHERE ts>?1 ORDER BY ts,id LIMIT ?2")
            .bind(after)
            .bind(limit.clamp(1, 2000))
            .fetch_all(&self.backend)
            .await?
            .iter()
            .map(|r| crate::AuditRow {
                id: r.get_string("id"),
                ts: r.get_string("ts"),
                actor: r.get_string("actor"),
                actor_kind: r.get_string("actor_kind"),
                action: r.get_string("action"),
                target: r.get_opt_string("target"),
                detail_json: r.get_string("detail_json"),
                ip: r.get_opt_string("ip"),
            })
            .collect())
    }
}
