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
            max_stage: 2,
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
        assert_eq!(
            store
                .request_protection_probe_decision("client", "192.0.2.1", &policy, 100)
                .await
                .unwrap(),
            Decision::default()
        );
        assert!(
            store
                .request_protection("client", "192.0.2.1", &policy, 100, Some(Signal::ProbePath))
                .await
                .unwrap()
                .subject_key
                .is_empty()
        );
        policy.rate_mode = Mode::Enforce;
        // A rate-only policy must not read the adaptive source-block table.
        assert_eq!(
            store
                .request_protection_probe_decision("client", "192.0.2.1", &policy, 100)
                .await
                .unwrap(),
            Decision::default()
        );
        policy.rate_mode = Mode::Off;
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

    async fn subject_row(store: &Store, key: &str) -> (String, String, i64, i64) {
        let row = q("SELECT state_json,revision,updated_at,expires_at FROM request_protection_subject WHERE subject_key=?1").bind(key).fetch_all(&store.backend).await.unwrap().remove(0);
        (
            row.get_string("state_json"),
            row.get_string("revision"),
            row.get_i64("updated_at"),
            row.get_i64("expires_at"),
        )
    }

    async fn table_counts(store: &Store) -> (i64, i64, i64) {
        let subjects = q("SELECT COUNT(*) AS n FROM request_protection_subject")
            .fetch_all(&store.backend)
            .await
            .unwrap()[0]
            .get_i64("n");
        let events = q("SELECT COUNT(*) AS n FROM request_protection_event")
            .fetch_all(&store.backend)
            .await
            .unwrap()[0]
            .get_i64("n");
        let settings = q("SELECT COUNT(*) AS n FROM settings")
            .fetch_all(&store.backend)
            .await
            .unwrap()[0]
            .get_i64("n");
        (subjects, events, settings)
    }

    #[tokio::test]
    async fn source_country_round_trips_atomically_in_rows_and_audit_without_clearing_known_data() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let mut policy = policy();
        policy.adaptive_mode = Mode::Enforce;
        policy.signals = 0;
        for (kind, country) in [("client", "PT"), ("ua-automation", "GB")] {
            let outcome = store
                .request_protection_with_country(
                    kind,
                    "192.0.2.1",
                    &policy,
                    100,
                    Some(Signal::ProbePath),
                    Some(country),
                )
                .await
                .unwrap();
            assert_eq!(outcome.snapshot.country_code.as_deref(), Some(country));
            let before = subject_row(&store, &outcome.subject_key).await;
            assert!(!before.0.contains("192.0.2.1"));
            let decision = store
                .request_protection_probe_decision(kind, "192.0.2.1", &policy, 101)
                .await
                .unwrap();
            assert_eq!(decision.reason, Some(Reason::ProbePath));
            assert_eq!(subject_row(&store, &outcome.subject_key).await, before);
            // Existing callers without metadata keep the last trusted lookup.
            let normal = store
                .request_protection(kind, "192.0.2.1", &policy, 102, Some(Signal::Normal))
                .await
                .unwrap();
            assert_eq!(normal.snapshot.country_code.as_deref(), Some(country));
            let invalid = store
                .request_protection_with_country(
                    kind,
                    "192.0.2.1",
                    &policy,
                    103,
                    Some(Signal::Normal),
                    Some("pT"),
                )
                .await
                .unwrap();
            assert_eq!(invalid.snapshot.country_code.as_deref(), Some(country));
            let rows = store.request_protection_rows(103, None, 100).await.unwrap();
            assert_eq!(
                rows.iter()
                    .find(|row| row.subject_key == outcome.subject_key)
                    .unwrap()
                    .snapshot
                    .country_code
                    .as_deref(),
                Some(country)
            );
            let events = store
                .request_protection_events(103, None, 100)
                .await
                .unwrap();
            let event = events
                .iter()
                .find(|event| event.subject_key == outcome.subject_key)
                .unwrap();
            assert_eq!(event.reason, "probe-path");
            assert_eq!(event.snapshot.country_code.as_deref(), Some(country));
        }
    }

    #[tokio::test]
    async fn country_is_never_assigned_to_aggregate_account_or_admin_subjects() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let mut policy = policy();
        policy.adaptive_mode = Mode::Enforce;
        for kind in ["account", "admin"] {
            let outcome = store
                .request_protection_with_country(
                    kind,
                    "verified-account",
                    &policy,
                    100,
                    Some(Signal::ProbePath),
                    Some("PT"),
                )
                .await
                .unwrap();
            assert!(outcome.snapshot.country_code.is_none());
            assert!(
                !subject_row(&store, &outcome.subject_key)
                    .await
                    .0
                    .contains("country")
            );
        }
        assert!(
            store
                .request_protection_rows(101, None, 100)
                .await
                .unwrap()
                .iter()
                .all(|row| row.snapshot.country_code.is_none())
        );
        assert!(
            store
                .request_protection_events(101, None, 100)
                .await
                .unwrap()
                .iter()
                .all(|event| event.snapshot.country_code.is_none())
        );
    }

    #[tokio::test]
    async fn probe_lookup_never_creates_an_identity_or_inventory_rows() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let mut policy = policy();
        policy.adaptive_mode = Mode::Enforce;
        let before = table_counts(&store).await;
        assert_eq!(
            store
                .request_protection_probe_decision("client", "192.0.2.1", &policy, 100)
                .await
                .unwrap(),
            Decision::default()
        );
        assert_eq!(
            store
                .request_protection_probe_decision("account", "verified", &policy, 100)
                .await
                .unwrap(),
            Decision::default()
        );
        assert_eq!(table_counts(&store).await, before);
    }

    #[tokio::test]
    async fn probe_source_lookup_preserves_state_rate_count_revision_and_audit() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let mut policy = policy();
        policy.adaptive_mode = Mode::Enforce;
        policy.rate_mode = Mode::Enforce;
        policy.signals = 0;
        policy.block_secs = 37;
        let admitted = store
            .request_protection("client", "192.0.2.1", &policy, 100, None)
            .await
            .unwrap();
        let probe = store
            .request_protection("client", "192.0.2.1", &policy, 100, Some(Signal::ProbePath))
            .await
            .unwrap();
        assert_eq!(
            probe.decision,
            Decision {
                action: Action::Block,
                reason: Some(Reason::ProbePath),
                enforced: true,
                retry_after_secs: 37
            }
        );
        assert_eq!(probe.snapshot.stage, 2);
        assert_eq!(probe.snapshot.rate_count, admitted.snapshot.rate_count);
        assert_eq!(probe.snapshot.short.anomalous, 1);
        let before = subject_row(&store, &probe.subject_key).await;
        let counts = table_counts(&store).await;
        for value in ["192.0.2.1", "::ffff:192.0.2.1"] {
            let decision = store
                .request_protection_probe_decision("client", value, &policy, 101)
                .await
                .unwrap();
            assert_eq!(decision.reason, Some(Reason::ProbePath));
            assert_eq!(decision.action, Action::Block);
            assert!(decision.enforced);
            assert_eq!(decision.retry_after_secs, 36);
        }
        assert_eq!(
            store
                .request_protection_probe_decision("account", "192.0.2.1", &policy, 101)
                .await
                .unwrap(),
            Decision::default()
        );
        let mut observe = policy.clone();
        observe.adaptive_mode = Mode::Observe;
        assert!(
            !store
                .request_protection_probe_decision("client", "192.0.2.1", &observe, 101)
                .await
                .unwrap()
                .enforced
        );
        let mut capped = policy.clone();
        capped.max_stage = 1;
        let limited = store
            .request_protection_probe_decision("client", "192.0.2.1", &capped, 101)
            .await
            .unwrap();
        assert_eq!(limited.action, Action::Throttle);
        assert_eq!(limited.retry_after_secs, capped.throttle_secs);
        assert_eq!(
            store
                .request_protection_probe_decision("client", "192.0.2.1", &policy, 137)
                .await
                .unwrap(),
            Decision::default()
        );
        assert_eq!(subject_row(&store, &probe.subject_key).await, before);
        assert_eq!(table_counts(&store).await, counts);
        let events = store
            .request_protection_events(101, None, 100)
            .await
            .unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].reason, "probe-path");
        assert_eq!(events[0].action, "block");
        assert!(events[0].enforced);
        // Expired inventory is ignored, not physically pruned by this read API.
        assert_eq!(
            store
                .request_protection_probe_decision("client", "192.0.2.1", &policy, before.3)
                .await
                .unwrap(),
            Decision::default()
        );
        assert_eq!(subject_row(&store, &probe.subject_key).await, before);
        assert_eq!(table_counts(&store).await, counts);
    }

    #[tokio::test]
    async fn first_observed_probe_only_proposes_and_stage_one_caps_use_throttle_duration() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let mut policy = policy();
        policy.adaptive_mode = Mode::Observe;
        policy.signals = 0;
        policy.block_secs = 43;
        let outcome = store
            .request_protection("client", "192.0.2.2", &policy, 100, Some(Signal::ProbePath))
            .await
            .unwrap();
        assert_eq!(outcome.decision.action, Action::Block);
        assert!(!outcome.decision.enforced);
        assert_eq!(outcome.decision.retry_after_secs, 43);
        let active = store
            .request_protection_probe_decision("client", "192.0.2.2", &policy, 101)
            .await
            .unwrap();
        assert_eq!(active.action, Action::Block);
        assert!(!active.enforced);
        policy.max_stage = 1;
        policy.throttle_secs = 7;
        policy.adaptive_mode = Mode::Enforce;
        let outcome = store
            .request_protection("client", "192.0.2.3", &policy, 100, Some(Signal::ProbePath))
            .await
            .unwrap();
        assert_eq!(outcome.decision.action, Action::Throttle);
        assert_eq!(outcome.snapshot.stage, 1);
        assert!(outcome.decision.enforced);
        assert_eq!(outcome.decision.retry_after_secs, 7);
    }

    #[tokio::test]
    async fn probe_lookup_ignores_weak_actions_and_refuses_corrupt_state() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let mut policy = policy();
        policy.adaptive_mode = Mode::Enforce;
        policy.short.absolute_anomalies = 1;
        let weak = store
            .request_protection("client", "192.0.2.1", &policy, 100, Some(Signal::Malformed))
            .await
            .unwrap();
        assert_eq!(weak.decision.action, Action::Throttle);
        assert_eq!(
            store
                .request_protection_probe_decision("client", "192.0.2.1", &policy, 101)
                .await
                .unwrap(),
            Decision::default()
        );
        q("UPDATE request_protection_subject SET state_json='{' WHERE subject_key=?1")
            .bind(&weak.subject_key)
            .execute(&store.backend)
            .await
            .unwrap();
        assert!(matches!(
            store
                .request_protection_probe_decision("client", "192.0.2.1", &policy, 101)
                .await,
            Err(StoreError::Corrupt(_))
        ));
        let before = subject_row(&store, &weak.subject_key).await;
        policy.adaptive_mode = Mode::Off;
        assert_eq!(
            store
                .request_protection_probe_decision("client", "192.0.2.1", &policy, 101)
                .await
                .unwrap(),
            Decision::default()
        );
        assert_eq!(subject_row(&store, &weak.subject_key).await, before);
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
        Some(Reason::ProbePath) => "probe-path",
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
        // OFF deployments with no expired data perform no maintenance writes.
        let expired = q("SELECT CAST(CASE WHEN EXISTS(SELECT 1 FROM request_protection_subject WHERE expires_at<=?1) OR EXISTS(SELECT 1 FROM request_protection_event WHERE observed_at<=?2) THEN 1 ELSE 0 END AS BIGINT) AS n")
            .bind(now).bind(now.saturating_sub(EVENT_RETENTION_SECS))
            .fetch_one(&self.backend).await?.get_i64("n");
        if expired == 0 {
            return Ok(());
        }
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
            "client" | "ua-missing" | "ua-uncommon" | "ua-automation" | "ua-search"
            | "ua-ai-training" | "ua-ai-search" | "ua-ai-user" | "ua-custom" => {
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
    /// Read only an existing high-confidence source action before authentication.
    /// This performs one indexed lookup, never admission, expiry cleanup, audit,
    /// state revision, inventory allocation or budget accounting.
    pub async fn request_protection_probe_decision(
        &self,
        kind: &str,
        value: &str,
        policy: &Policy,
        now: i64,
    ) -> Result<Decision, StoreError> {
        if !policy.valid() || now < 0 {
            return Err(corrupt());
        }
        if policy.adaptive_mode == Mode::Off {
            return Ok(Decision::default());
        }
        let key = self.request_protection_subject(kind, value)?;
        let rows = q("SELECT state_json,subject_kind,expires_at FROM request_protection_subject WHERE subject_key=?1")
            .bind(&key)
            .fetch_all(&self.backend)
            .await?;
        let Some(row) = rows.first() else {
            return Ok(Decision::default());
        };
        if row.get_string("subject_kind") != kind {
            return Err(corrupt());
        }
        if row.get_i64("expires_at") <= now {
            return Ok(Decision::default());
        }
        Ok(load(&row.get_string("state_json"), now)?.probe_decision(policy, now))
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
        self.request_protection_with_country(kind, value, policy, now, signal, None)
            .await
    }
    /// Source country is trusted lookup metadata, not an identity. Aggregated
    /// account/admin subjects ignore it; unavailable lookups preserve prior data.
    pub async fn request_protection_with_country(
        &self,
        kind: &str,
        value: &str,
        policy: &Policy,
        now: i64,
        signal: Option<Signal>,
        country: Option<&str>,
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
        if (kind == "client" || kind.starts_with("ua-"))
            && let Some(country) = country
        {
            state.set_country_code(country);
        }
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
