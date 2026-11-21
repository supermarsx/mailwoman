//! Explicit fixture-only q() attribution. Never retain bind values or report SQL.
//! Static-template hashes and fixed token families identify code/query shapes,
//! not a SQL query plan or physical pager reads. Detached tasks/direct SQLx are excluded.
use super::{Class, QuerySnapshot};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fmt::Write as _,
    future::Future,
    sync::{Arc, Mutex, OnceLock},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
pub const MAX_OPERATIONS: usize = 256;
pub const MAX_TEMPLATES: usize = 128;
pub const MAX_REPORT_BYTES: usize = 65536;
// Bounded native templates are 63,902 SQLite / 39,662 PostgreSQL bytes.
// Encoded reports retain their independent 64 KiB limit.
pub(crate) const MAX_SQL_BYTES: usize = 65_536;
// Fixed compile-time phases only. No client/path/SQL/bind-derived label can be
// supplied. Unknown is explicit so untagged awaited q() work is never hidden.
macro_rules! phases {
    ($($variant:ident => $label:literal),+ $(,)?) => {
        #[derive(Clone, Copy)]
        pub enum Phase { $($variant),+ }
        const PHASE_LABELS: &[&str] = &[$($label),+];
    };
}
phases! {
 Unknown=>"unknown", Request=>"request", HealthAccess=>"guard.health-access", Protocol=>"guard.protocol", Probe=>"guard.probe",
 Protection=>"guard.protection", SourceBans=>"guard.source-bans", InboundAllowlist=>"guard.inbound-allowlist",
 SessionWatch=>"guard.session-watch", SessionGeo=>"guard.session-geography", ClientAccess=>"guard.client-access",
 LinkedScope=>"guard.linked-scope", Maintenance=>"guard.maintenance", Features=>"guard.features", Auth=>"auth.total",
 SessionRead=>"auth.session-read", NativeRead=>"auth.native-read", SessionClock=>"auth.session-clock",
 SessionTouch=>"auth.session-touch", SessionTouchSkipped=>"auth.session-touch-skipped", LinkedLogin=>"auth.linked-login",
 DomainUpgrade=>"auth.domain-upgrade", AccountGate=>"auth.account-gate", LinkedContext=>"auth.linked-context",
 FeaturePresence=>"feature.key-presence", FeatureResolution=>"feature.resolution", SessionWatchConfig=>"session-watch.config",
 AfterBody=>"jmap.after-body", Handler=>"jmap.handler", Engine=>"jmap.engine"
}
#[derive(Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseTotals {
    pub phase: &'static str,
    pub started: u64,
    pub completed: u64,
    pub failures: u64,
    pub total_micros: u64,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapePhase {
    pub phase: &'static str,
    pub calls: u64,
    pub failures: u64,
    pub total_micros: u64,
}
tokio::task_local! { static ACTIVE_PHASE: Phase; }
/// Does not create a query scope or propagate it into detached/spawned tasks.
pub fn active() -> bool {
    CURRENT
        .try_with(|scope| scope._permit.is_some())
        .unwrap_or(false)
}
pub async fn in_phase<F: Future>(phase: Phase, future: F) -> F::Output {
    if active() {
        ACTIVE_PHASE.scope(phase, future).await
    } else {
        future.await
    }
}

const TOKENS: &[&str] = &[
    "sessions",
    "session_principal",
    "accounts",
    "mail_user",
    "mail_user_account",
    "identity_principal",
    "identity_binding",
    "linked_account",
    "linked_context",
    "linked_primary",
    "linked_credential",
    "setting_policy",
    "setting_value",
    "personal_setting_value",
    "settings_revision",
    "settings",
    "domains",
    "domain_alias",
    "account_ban",
    "account_restriction",
    "signin_event",
    "messages",
    "message_meta",
    "message_list_envelope",
    "bodies",
    "message_body_cache_lease",
    "mailboxes",
    "changes",
    "change_log_checkpoint",
    "change_log_retention_cursor",
    "security_verdicts",
    "contacts",
    "address_books",
    "sync_state",
    "submissions",
    "request_protection_subject",
    "request_protection_event",
    "ip_allow",
    "ip_block",
    "user_activity",
    "user_retention_account",
    "feature_profile",
    "feature_profile_assignment",
    "admin_account",
    "session_clock",
    "session_retirement",
];
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Shape {
    pub phase_calls: Vec<ShapePhase>,
    pub template_sha256: String,
    pub class: &'static str,
    pub static_token_families: Vec<&'static str>,
    pub calls: u64,
    pub failures: u64,
    pub returned_rows: u64,
    pub returned_rows_known_calls: u64,
    pub affected_rows: u64,
    pub affected_rows_known_calls: u64,
    pub total_micros: u64,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub valid: bool,
    pub invalid_reason: Option<&'static str>,
    pub boundary: &'static str,
    pub operations_started: u64,
    pub operations_completed: u64,
    pub query_counts: QuerySnapshot,
    pub shapes: Vec<Shape>,
    pub phases: Vec<PhaseTotals>,
}
impl Report {
    pub fn require_valid(&self) -> Result<(), &'static str> {
        if self.valid {
            Ok(())
        } else {
            Err(self.invalid_reason.unwrap_or("invalid attribution"))
        }
    }
}
#[derive(Default)]
struct Data {
    started: u64,
    completed: u64,
    invalid: Option<&'static str>,
    shapes: BTreeMap<&'static str, Shape>,
    phases: [PhaseTotals; PHASE_LABELS.len()],
}
struct Scope {
    data: Mutex<Data>,
    _permit: Option<OwnedSemaphorePermit>,
}
tokio::task_local! {static CURRENT:Arc<Scope>;}
fn class_name(class: Class) -> &'static str {
    match class {
        Class::Read => "read",
        Class::Write => "write",
        Class::Other => "other",
        Class::Acquire => "acquire",
    }
}
pub(super) struct Capture {
    scope: Arc<Scope>,
    sql: &'static str,
    phase: Phase,
    pub(super) returned: Option<u64>,
    pub(super) affected: Option<u64>,
}
pub(super) fn capture(sql: &'static str, class: Class) -> Option<Capture> {
    let scope = CURRENT.try_with(Arc::clone).ok()?;
    if scope._permit.is_none() {
        return None;
    }
    let phase = ACTIVE_PHASE
        .try_with(|phase| *phase)
        .unwrap_or(Phase::Unknown);
    {
        let mut data = scope.data.lock().unwrap_or_else(|p| p.into_inner());
        data.started = data.started.saturating_add(1);
        data.phases[phase as usize].started = data.phases[phase as usize].started.saturating_add(1);
        if data.started > MAX_OPERATIONS as u64 {
            data.invalid = Some("operation budget exceeded");
            return None;
        }
        if sql.len() > MAX_SQL_BYTES {
            data.invalid = Some("static SQL budget exceeded");
            return None;
        }
        if !data.shapes.contains_key(sql) {
            if data.shapes.len() >= MAX_TEMPLATES {
                data.invalid = Some("template budget exceeded");
                return None;
            }
            let tokens = TOKENS
                .iter()
                .copied()
                .filter(|token| {
                    sql.split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                        .any(|word| word.eq_ignore_ascii_case(token))
                })
                .collect();
            let mut template_sha256 = String::with_capacity(64);
            for byte in Sha256::digest(sql.as_bytes()).iter() {
                write!(&mut template_sha256, "{byte:02x}")
                    .expect("writing a hash into a String cannot fail");
            }
            data.shapes.insert(
                sql,
                Shape {
                    phase_calls: Vec::new(),
                    template_sha256,
                    class: class_name(class),
                    static_token_families: tokens,
                    calls: 0,
                    failures: 0,
                    returned_rows: 0,
                    returned_rows_known_calls: 0,
                    affected_rows: 0,
                    affected_rows_known_calls: 0,
                    total_micros: 0,
                },
            );
        }
    }
    Some(Capture {
        scope,
        sql,
        phase,
        returned: None,
        affected: None,
    })
}
impl Capture {
    pub(super) fn finish(self, success: bool, micros: u64) {
        let mut data = self.scope.data.lock().unwrap_or_else(|p| p.into_inner());
        data.completed = data.completed.saturating_add(1);
        let phase = &mut data.phases[self.phase as usize];
        phase.completed = phase.completed.saturating_add(1);
        phase.failures = phase.failures.saturating_add(u64::from(!success));
        phase.total_micros = phase.total_micros.saturating_add(micros);
        if let Some(shape) = data.shapes.get_mut(self.sql) {
            let label = PHASE_LABELS[self.phase as usize];
            let at = shape
                .phase_calls
                .iter()
                .position(|row| row.phase == label)
                .unwrap_or_else(|| {
                    shape.phase_calls.push(ShapePhase {
                        phase: label,
                        calls: 0,
                        failures: 0,
                        total_micros: 0,
                    });
                    shape.phase_calls.len() - 1
                });
            let phase = &mut shape.phase_calls[at];
            phase.calls += 1;
            phase.failures += u64::from(!success);
            phase.total_micros = phase.total_micros.saturating_add(micros);
            shape.calls += 1;
            shape.failures += u64::from(!success);
            shape.total_micros = shape.total_micros.saturating_add(micros);
            if let Some(rows) = self.returned {
                shape.returned_rows = shape.returned_rows.saturating_add(rows);
                shape.returned_rows_known_calls += 1;
            }
            if let Some(rows) = self.affected {
                shape.affected_rows = shape.affected_rows.saturating_add(rows);
                shape.affected_rows_known_calls += 1;
            }
        } else {
            data.invalid = Some("missing static query shape");
        }
    }
}
/// Measures only this task's awaited q() operations. At most four live scopes;
/// accepted timers retain scope admission until actual wrapper completion.
/// Requests still execute on diagnostic overflow, but their report is invalid.
pub async fn measure<F: Future>(future: F) -> (F::Output, Report) {
    static SCOPES: OnceLock<Arc<Semaphore>> = OnceLock::new();
    let permit = SCOPES
        .get_or_init(|| Arc::new(Semaphore::new(4)))
        .clone()
        .try_acquire_owned()
        .ok();
    let invalid = if permit.is_none() {
        Some("scope budget exceeded")
    } else {
        None
    };
    let scope = Arc::new(Scope {
        data: Mutex::new(Data {
            invalid,
            ..Default::default()
        }),
        _permit: permit,
    });
    let (output, queries) = super::measure_queries(
        CURRENT.scope(scope.clone(), ACTIVE_PHASE.scope(Phase::Unknown, future)),
    )
    .await;
    let mut report = {
        let data = scope.data.lock().unwrap_or_else(|p| p.into_inner());
        Report {
            valid: false,
            invalid_reason: data.invalid,
            boundary: "awaited q() wrappers including BEGIN; direct SQLx, COMMIT/ROLLBACK and detached tasks excluded; acquisition timings overlap; token families are static SQL tokens, not physical I/O",
            operations_started: data.started,
            operations_completed: data.completed,
            query_counts: queries,
            shapes: data.shapes.values().cloned().collect(),
            phases: data
                .phases
                .iter()
                .zip(PHASE_LABELS)
                .map(|(totals, label)| PhaseTotals {
                    phase: label,
                    ..*totals
                })
                .collect(),
        }
    };
    if report.operations_started != report.operations_completed {
        report.invalid_reason.get_or_insert("incomplete operations");
    }
    for (class, counts) in [
        ("read", queries.read),
        ("write", queries.write),
        ("other", queries.other),
    ] {
        let (calls, failures) = report
            .shapes
            .iter()
            .filter(|shape| shape.class == class)
            .fold((0, 0), |(calls, failures), s| {
                (calls + s.calls, failures + s.failures)
            });
        if calls != counts.calls || failures != counts.failures {
            report.invalid_reason.get_or_insert("query count mismatch");
        }
    }
    if report.phases.iter().map(|phase| phase.started).sum::<u64>() != report.operations_started
        || report
            .phases
            .iter()
            .map(|phase| phase.completed)
            .sum::<u64>()
            != report.operations_completed
        || report
            .phases
            .iter()
            .map(|phase| phase.failures)
            .sum::<u64>()
            != queries.read.failures + queries.write.failures + queries.other.failures
    {
        report.invalid_reason.get_or_insert("phase count mismatch");
    }
    for phase in &report.phases {
        let (completed, failures) = report
            .shapes
            .iter()
            .flat_map(|shape| shape.phase_calls.iter())
            .filter(|row| row.phase == phase.phase)
            .fold((0, 0), |(calls, failures), row| {
                (calls + row.calls, failures + row.failures)
            });
        if completed != phase.completed || failures != phase.failures {
            report
                .invalid_reason
                .get_or_insert("phase shape count mismatch");
        }
    }
    for shape in &report.shapes {
        if shape
            .phase_calls
            .iter()
            .map(|phase| phase.calls)
            .sum::<u64>()
            != shape.calls
            || shape
                .phase_calls
                .iter()
                .map(|phase| phase.failures)
                .sum::<u64>()
                != shape.failures
        {
            report
                .invalid_reason
                .get_or_insert("shape phase count mismatch");
        }
    }
    report.valid = report.invalid_reason.is_none();
    if serde_json::to_vec(&report).map_or(true, |bytes| bytes.len() > MAX_REPORT_BYTES) {
        report.valid = false;
        report.invalid_reason = Some("report byte budget exceeded");
        report.shapes.clear();
    }
    (output, report)
}

#[cfg(test)]
#[path = "query_shapes_tests.rs"]
mod tests;
