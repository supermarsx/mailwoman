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
const MAX_SQL_BYTES: usize = 16384;
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
    pub(super) returned: Option<u64>,
    pub(super) affected: Option<u64>,
}
pub(super) fn capture(sql: &'static str, class: Class) -> Option<Capture> {
    let scope = CURRENT.try_with(Arc::clone).ok()?;
    if scope._permit.is_none() {
        return None;
    }
    {
        let mut data = scope.data.lock().unwrap_or_else(|p| p.into_inner());
        data.started = data.started.saturating_add(1);
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
        returned: None,
        affected: None,
    })
}
impl Capture {
    pub(super) fn finish(self, success: bool, micros: u64) {
        let mut data = self.scope.data.lock().unwrap_or_else(|p| p.into_inner());
        data.completed = data.completed.saturating_add(1);
        if let Some(shape) = data.shapes.get_mut(self.sql) {
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
    let (output, queries) = super::measure_queries(CURRENT.scope(scope.clone(), future)).await;
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
