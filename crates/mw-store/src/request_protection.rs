//! Atomic, bounded request budgets and adaptive history shared by all replicas.
use crate::{
    Store, StoreError,
    backend::{Tx, q},
    request_protection_model::{Action, Decision, Mode, Policy, Reason, Signal, Snapshot, State},
};
use serde::Serialize;
pub const SUBJECT_CAPACITY: i64 = 20_000;
pub const EVENT_CAPACITY: i64 = 10_000;
pub const EVENT_RETENTION_SECS: i64 = 30 * 86400;
const MAX_STATE_BYTES: usize = 32 * 1024;

#[derive(Clone, Debug)]
pub struct ProtectionOutcome {
    pub subject_key: String,
    pub decision: Decision,
    pub snapshot: Snapshot,
    pub revision: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ServerKey, request_protection_model::WindowPolicy};

    fn policy() -> Policy {
        Policy {
            rate_mode: Mode::Off,
            adaptive_mode: Mode::Off,
            rate_limit: 100,
            rate_window_secs: 60,
            short: WindowPolicy {
                seconds: 60,
                minimum_samples: 10,
                absolute_anomalies: 10,
                ratio_percent: 50,
            },
            long: WindowPolicy {
                seconds: 86400,
                minimum_samples: 100,
                absolute_anomalies: 100,
                ratio_percent: 50,
            },
            normal_cap_per_bucket: 100,
            signals: 7,
            throttle_secs: 3,
            block_secs: 60,
            decay_secs: 3600,
        }
    }

    #[tokio::test]
    async fn off_and_ignored_observations_never_access_counter_tables() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        q("DROP TABLE request_protection_subject")
            .execute(&store.backend)
            .await
            .unwrap();
        q("DROP TABLE request_protection_event")
            .execute(&store.backend)
            .await
            .unwrap();
        let mut policy = policy();
        assert!(
            store
                .request_protection("client", "192.0.2.1", &policy, 100, None)
                .await
                .unwrap()
                .subject_key
                .is_empty()
        );
        policy.adaptive_mode = Mode::Enforce;
        assert!(
            store
                .request_protection("client", "192.0.2.1", &policy, 100, Some(Signal::Ignored))
                .await
                .unwrap()
                .subject_key
                .is_empty()
        );
    }

    #[tokio::test]
    async fn expired_state_is_physically_removed_even_when_new_admission_stops() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let mut policy = policy();
        policy.rate_mode = Mode::Enforce;
        store
            .request_protection("account", "verified", &policy, 100, None)
            .await
            .unwrap();
        assert_eq!(
            q("SELECT COUNT(*) AS n FROM request_protection_subject")
                .fetch_all(&store.backend)
                .await
                .unwrap()[0]
                .get_i64("n"),
            1
        );
        store
            .request_protection_rows(100_000, None, 100)
            .await
            .unwrap();
        assert_eq!(
            q("SELECT COUNT(*) AS n FROM request_protection_subject")
                .fetch_all(&store.backend)
                .await
                .unwrap()[0]
                .get_i64("n"),
            0
        );
    }
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtectionRow {
    pub subject_key: String,
    pub subject_kind: String,
    pub updated_at: i64,
    pub expires_at: i64,
    pub revision: String,
    pub snapshot: Snapshot,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtectionEvent {
    pub id: String,
    pub subject_key: String,
    pub subject_kind: String,
    pub observed_at: i64,
    pub reason: String,
    pub action: String,
    pub enforced: bool,
    pub retry_after_secs: i64,
    pub snapshot: Snapshot,
    pub policy: Option<Policy>,
}
fn corrupt() -> StoreError {
    StoreError::Corrupt("invalid request protection state".into())
}
fn revision() -> String {
    format!("{:032x}", rand::random::<u128>())
}
fn reason(value: Option<Reason>) -> &'static str {
    match value {
        Some(Reason::RequestRate) => "request-rate",
        Some(Reason::ShortAbsolute) => "short-absolute",
        Some(Reason::ShortRatio) => "short-ratio",
        Some(Reason::LongAbsolute) => "long-absolute",
        Some(Reason::LongRatio) => "long-ratio",
        None => "none",
    }
}
fn load(raw: &str, now: i64) -> Result<State, StoreError> {
    if raw.len() > MAX_STATE_BYTES {
        return Err(corrupt());
    }
    let state: State = serde_json::from_str(raw).map_err(|_| corrupt())?;
    if !state.valid(now) {
        return Err(corrupt());
    }
    Ok(state)
}
async fn inventory_lock(tx: &mut Tx) -> Result<(), StoreError> {
    q("INSERT INTO settings(key,value) VALUES('request-protection:inventory-lock','0') ON CONFLICT(key) DO NOTHING").execute_tx(tx).await?;
    q("UPDATE settings SET value=value WHERE key='request-protection:inventory-lock'")
        .execute_tx(tx)
        .await?;
    Ok(())
}
struct AuditDecision<'a> {
    code: &'a str,
    action: &'a str,
    enforced: bool,
    retry: u32,
}
async fn audit(
    tx: &mut Tx,
    key: &str,
    kind: &str,
    now: i64,
    decision: AuditDecision<'_>,
    snapshot: &Snapshot,
    policy: Option<&Policy>,
) -> Result<(), StoreError> {
    // Separate from inventory locking: existing subjects must never acquire the
    // inventory lock after their row lock (expiry cleanup takes the reverse path).
    q("INSERT INTO settings(key,value) VALUES('request-protection:audit-lock','0') ON CONFLICT(key) DO NOTHING").execute_tx(tx).await?;
    q("UPDATE settings SET value=value WHERE key='request-protection:audit-lock'")
        .execute_tx(tx)
        .await?;
    q("DELETE FROM request_protection_event WHERE id IN (SELECT id FROM request_protection_event WHERE observed_at<=?1 ORDER BY observed_at LIMIT 128)").bind(now.saturating_sub(EVENT_RETENTION_SECS)).execute_tx(tx).await?;
    let count = q("SELECT COUNT(*) AS n FROM request_protection_event")
        .fetch_all_tx(tx)
        .await?[0]
        .get_i64("n");
    if count >= EVENT_CAPACITY {
        q("DELETE FROM request_protection_event WHERE id IN (SELECT id FROM request_protection_event ORDER BY observed_at,id LIMIT 128)").execute_tx(tx).await?;
    }
    let counts = serde_json::json!({"snapshot":snapshot,"policy":policy}).to_string();
    q("INSERT INTO request_protection_event(id,subject_key,subject_kind,observed_at,reason,action,enforced,retry_after_secs,counts_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)")
        .bind(revision()).bind(key).bind(kind).bind(now).bind(decision.code).bind(decision.action).bind(i64::from(decision.enforced)).bind(i64::from(decision.retry)).bind(counts).execute_tx(tx).await?;
    Ok(())
}
impl Store {
    /// Bounded cleanup; call from the administrative views and periodic maintenance.
    /// Admission also reclaims expired subjects before allocating a new identity.
    pub async fn prune_request_protection(&self, now: i64) -> Result<(), StoreError> {
        let mut tx = self.backend.begin().await?;
        inventory_lock(&mut tx).await?;
        q("DELETE FROM request_protection_subject WHERE subject_key IN (SELECT subject_key FROM request_protection_subject WHERE expires_at<=?1 ORDER BY expires_at LIMIT 128)").bind(now).execute_tx(&mut tx).await?;
        q("DELETE FROM request_protection_event WHERE id IN (SELECT id FROM request_protection_event WHERE observed_at<=?1 ORDER BY observed_at LIMIT 128)").bind(now.saturating_sub(EVENT_RETENTION_SECS)).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(())
    }
    pub async fn request_protection_events(
        &self,
        now: i64,
        before: Option<(i64, &str)>,
        limit: usize,
    ) -> Result<Vec<ProtectionEvent>, StoreError> {
        self.prune_request_protection(now).await?;
        let (at, id) = before.unwrap_or((i64::MAX, ""));
        let rows=q("SELECT id,subject_key,subject_kind,observed_at,reason,action,enforced,retry_after_secs,counts_json FROM request_protection_event WHERE observed_at>?1 AND observed_at<=?2 AND (observed_at<?3 OR (observed_at=?3 AND id<?4)) ORDER BY observed_at DESC,id DESC LIMIT ?5").bind(now.saturating_sub(EVENT_RETENTION_SECS)).bind(now).bind(at).bind(id).bind(limit.clamp(1,100) as i64).fetch_all(&self.backend).await?;
        rows.iter()
            .map(|row| {
                let raw = row.get_string("counts_json");
                if raw.len() > MAX_STATE_BYTES {
                    return Err(corrupt());
                }
                let value: serde_json::Value = serde_json::from_str(&raw).map_err(|_| corrupt())?;
                let snapshot =
                    serde_json::from_value(value.get("snapshot").unwrap_or(&value).clone())
                        .map_err(|_| corrupt())?;
                let policy = value
                    .get("policy")
                    .filter(|v| !v.is_null())
                    .map(|v| serde_json::from_value(v.clone()).map_err(|_| corrupt()))
                    .transpose()?;
                Ok(ProtectionEvent {
                    id: row.get_string("id"),
                    subject_key: row.get_string("subject_key"),
                    subject_kind: row.get_string("subject_kind"),
                    observed_at: row.get_i64("observed_at"),
                    reason: row.get_string("reason"),
                    action: row.get_string("action"),
                    enforced: row.get_i64("enforced") != 0,
                    retry_after_secs: row.get_i64("retry_after_secs"),
                    snapshot,
                    policy,
                })
            })
            .collect()
    }
    /// Caller supplies a trusted transport IP or a server-authenticated account ID.
    /// Distinct authority kinds cannot collide; identifiers never enter stored rows.
    pub fn request_protection_subject(
        &self,
        kind: &str,
        value: &str,
    ) -> Result<String, StoreError> {
        let canonical;
        let value = match kind {
            "client" => {
                canonical = value
                    .parse::<std::net::IpAddr>()
                    .map_err(|_| corrupt())?
                    .to_canonical()
                    .to_string();
                canonical.as_str()
            }
            "account" | "admin"
                if !value.is_empty()
                    && value.len() <= 512
                    && !value.chars().any(char::is_control) =>
            {
                value
            }
            _ => return Err(corrupt()),
        };
        let message = format!("{kind}\0{value}");
        Ok(crate::seal::hmac_sha256(
            &self.key.derive("mw-request-protection-subject-v1"),
            message.as_bytes(),
        )
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
    }
    /// `None` charges admission once; `Some` records one typed completion signal.
    /// Disabled policies and ignored responses perform no durable reads or writes.
    pub async fn request_protection(
        &self,
        kind: &str,
        value: &str,
        policy: &Policy,
        now: i64,
        signal: Option<Signal>,
    ) -> Result<ProtectionOutcome, StoreError> {
        if !policy.valid() || now < 0 {
            return Err(corrupt());
        }
        if policy.off()
            || signal == Some(Signal::Ignored)
            || (signal.is_some() && policy.adaptive_mode == Mode::Off)
        {
            return Ok(ProtectionOutcome {
                subject_key: String::new(),
                decision: Decision::default(),
                snapshot: State::default().snapshot(),
                revision: String::new(),
            });
        }
        let key = self.request_protection_subject(kind, value)?;
        let mut tx = self.backend.begin().await?;
        // An existing subject uses its own row lock; independent PostgreSQL
        // subjects need not serialize every request through one global lock.
        let locked =
            q("UPDATE request_protection_subject SET revision=revision WHERE subject_key=?1")
                .bind(&key)
                .execute_tx(&mut tx)
                .await?;
        if locked == 0 {
            inventory_lock(&mut tx).await?;
            q("DELETE FROM request_protection_subject WHERE subject_key IN (SELECT subject_key FROM request_protection_subject WHERE expires_at<=?1 ORDER BY expires_at LIMIT 128)").bind(now).execute_tx(&mut tx).await?;
            let present =
                q("SELECT subject_key FROM request_protection_subject WHERE subject_key=?1")
                    .bind(&key)
                    .fetch_all_tx(&mut tx)
                    .await?;
            if present.is_empty() {
                let count = q("SELECT COUNT(*) AS n FROM request_protection_subject")
                    .fetch_all_tx(&mut tx)
                    .await?[0]
                    .get_i64("n");
                if count >= SUBJECT_CAPACITY {
                    return Err(StoreError::Corrupt(
                        "request protection capacity reached".into(),
                    ));
                }
                let empty = serde_json::to_string(&State::default()).map_err(|_| corrupt())?;
                q("INSERT INTO request_protection_subject(subject_key,subject_kind,state_json,updated_at,expires_at,revision) VALUES(?1,?2,?3,?4,?5,?6)").bind(&key).bind(kind).bind(empty).bind(now).bind(now.saturating_add(i64::from(policy.retention_secs()))).bind(revision()).execute_tx(&mut tx).await?;
            }
            // A concurrent first writer may have inserted it while we waited
            // for inventory admission. Lock that row before reading its state.
            q("UPDATE request_protection_subject SET revision=revision WHERE subject_key=?1")
                .bind(&key)
                .execute_tx(&mut tx)
                .await?;
        }
        let rows=q("SELECT state_json,subject_kind,expires_at FROM request_protection_subject WHERE subject_key=?1").bind(&key).fetch_all_tx(&mut tx).await?;
        let row = rows.first().ok_or_else(corrupt)?;
        if row.get_string("subject_kind") != kind {
            return Err(corrupt());
        }
        let mut state = if row.get_i64("expires_at") <= now {
            State::default()
        } else {
            load(&row.get_string("state_json"), now)?
        };
        let previous = state.clone();
        let decision = match signal {
            None => state.admit(policy, now),
            Some(value) => state.observe(policy, now, value),
        };
        state.counts(policy, now);
        let snapshot = state.snapshot();
        let rev = revision();
        let json = serde_json::to_string(&state).map_err(|_| corrupt())?;
        if json.len() > MAX_STATE_BYTES {
            return Err(corrupt());
        }
        let expires = now
            .saturating_add(i64::from(policy.retention_secs()))
            .max(state.active_until());
        q("UPDATE request_protection_subject SET state_json=?1,updated_at=?2,expires_at=?3,revision=?4 WHERE subject_key=?5").bind(json).bind(now).bind(expires).bind(&rev).bind(&key).execute_tx(&mut tx).await?;
        if state.transition_from(&previous, decision, policy) {
            audit(
                &mut tx,
                &key,
                kind,
                now,
                AuditDecision {
                    code: reason(decision.reason),
                    action: if decision.action == Action::Block {
                        "block"
                    } else {
                        "throttle"
                    },
                    enforced: decision.enforced,
                    retry: decision.retry_after_secs,
                },
                &snapshot,
                Some(policy),
            )
            .await?;
        }
        tx.commit().await?;
        Ok(ProtectionOutcome {
            subject_key: key,
            decision,
            snapshot,
            revision: rev,
        })
    }
    pub async fn request_protection_rows(
        &self,
        now: i64,
        after: Option<&str>,
        limit: usize,
    ) -> Result<Vec<ProtectionRow>, StoreError> {
        self.prune_request_protection(now).await?;
        let rows=q("SELECT subject_key,subject_kind,state_json,updated_at,expires_at,revision FROM request_protection_subject WHERE expires_at>?1 AND subject_key>?2 ORDER BY subject_key LIMIT ?3").bind(now).bind(after.unwrap_or("")).bind(limit.clamp(1,100) as i64).fetch_all(&self.backend).await?;
        rows.iter()
            .map(|row| {
                Ok(ProtectionRow {
                    subject_key: row.get_string("subject_key"),
                    subject_kind: row.get_string("subject_kind"),
                    updated_at: row.get_i64("updated_at"),
                    expires_at: row.get_i64("expires_at"),
                    revision: row.get_string("revision"),
                    snapshot: load(&row.get_string("state_json"), now)?.snapshot(),
                })
            })
            .collect()
    }
    pub async fn clear_request_protection(
        &self,
        key: &str,
        expected: &str,
        now: i64,
    ) -> Result<bool, StoreError> {
        let mut tx = self.backend.begin().await?;
        q("UPDATE request_protection_subject SET revision=revision WHERE subject_key=?1")
            .bind(key)
            .execute_tx(&mut tx)
            .await?;
        let rows=q("SELECT subject_kind,state_json,revision FROM request_protection_subject WHERE subject_key=?1").bind(key).fetch_all_tx(&mut tx).await?;
        let Some(row) = rows.first() else {
            return Ok(false);
        };
        if row.get_string("revision") != expected {
            return Ok(false);
        }
        audit(
            &mut tx,
            key,
            &row.get_string("subject_kind"),
            now,
            AuditDecision {
                code: "manual-clear",
                action: "cleared",
                enforced: false,
                retry: 0,
            },
            &load(&row.get_string("state_json"), now)?.snapshot(),
            None,
        )
        .await?;
        q("DELETE FROM request_protection_subject WHERE subject_key=?1")
            .bind(key)
            .execute_tx(&mut tx)
            .await?;
        tx.commit().await?;
        Ok(true)
    }
}
