//! Bounded process-local warning/error history for privileged diagnostics.
use crate::{
    admin_ctx::{AdminApiError, AdminCtx},
    admin_router::AdminRegistrar,
    capabilities::Capability,
};
use axum::{Json, extract::Query, http::Method};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, VecDeque},
    fmt::{self, Write},
    sync::{Mutex, OnceLock},
};
use tracing::{
    Event, Subscriber,
    field::{Field, Visit},
};
use tracing_subscriber::{Layer, layer::Context};

#[path = "observability/redact_log.rs"]
mod redact_log;

const CAPACITY: usize = 1000;
const MAX_AGE: i64 = 86_400;
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    id: u64,
    first_seen: i64,
    last_seen: i64,
    repeat_count: u64,
    level: String,
    target: String,
    fields: BTreeMap<String, String>,
}
#[derive(Default)]
struct History {
    entries: VecDeque<Entry>,
    sequence: u64,
    discarded: u64,
}
impl History {
    fn prune(&mut self, now: i64) {
        self.entries
            .retain(|e| now.saturating_sub(e.last_seen) <= MAX_AGE);
    }
    fn append(&mut self, level: &str, target: &str, fields: BTreeMap<String, String>, now: i64) {
        self.prune(now);
        self.sequence = self.sequence.saturating_add(1);
        let same = self.entries.iter().position(|e| {
            now.saturating_sub(e.last_seen) < 60
                && e.level == level
                && e.target == target
                && e.fields == fields
        });
        let mut entry = same.and_then(|i| self.entries.remove(i)).unwrap_or(Entry {
            id: 0,
            first_seen: now,
            last_seen: now,
            repeat_count: 0,
            level: level.into(),
            target: target.into(),
            fields,
        });
        entry.id = self.sequence;
        entry.last_seen = now;
        entry.repeat_count = entry.repeat_count.saturating_add(1);
        if self.entries.len() == CAPACITY {
            self.entries.pop_front();
            self.discarded = self.discarded.saturating_add(1);
        }
        self.entries.push_back(entry);
    }
}
static HISTORY: OnceLock<Mutex<History>> = OnceLock::new();
fn history() -> &'static Mutex<History> {
    HISTORY.get_or_init(Default::default)
}

/// Install alongside the normal console subscriber. Captures warnings/errors only;
/// no span fields, request bodies, or debug/trace protocol dumps are collected.
pub struct CaptureLayer {
    redactor: redact_log::Redactor,
}
impl Default for CaptureLayer {
    fn default() -> Self {
        Self {
            redactor: redact_log::Redactor::from_env(),
        }
    }
}
struct BoundedText {
    text: String,
    overflow: bool,
}
impl Write for BoundedText {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        if self.text.len().saturating_add(text.len()) > 4096 {
            self.overflow = true;
            return Err(fmt::Error);
        }
        self.text.push_str(text);
        Ok(())
    }
}
struct Fields<'a> {
    redactor: &'a redact_log::Redactor,
    fields: BTreeMap<String, String>,
    bytes: usize,
}
impl Visit for Fields<'_> {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if self.fields.len() >= 16 || self.bytes >= 4096 {
            return;
        }
        let mut text = BoundedText {
            text: String::new(),
            overflow: false,
        };
        let _ = write!(text, "{value:?}");
        let safe = if text.overflow {
            "[oversized field omitted]".into()
        } else {
            self.redactor.field(field.name(), &text.text)
        };
        let limit = if field.name() == "message" { 2048 } else { 512 };
        let max_bytes = limit.min(4096 - self.bytes);
        let mut remaining = max_bytes;
        let safe: String = safe
            .chars()
            .filter(|c| !c.is_control())
            .take_while(|c| {
                if c.len_utf8() > remaining {
                    false
                } else {
                    remaining -= c.len_utf8();
                    true
                }
            })
            .collect();
        self.bytes += safe.len();
        self.fields
            .insert(field.name().chars().take(64).collect(), safe);
    }
}
impl<S: Subscriber> Layer<S> for CaptureLayer {
    fn on_event(&self, event: &Event<'_>, _: Context<'_, S>) {
        let meta = event.metadata();
        if !matches!(*meta.level(), tracing::Level::WARN | tracing::Level::ERROR) {
            return;
        }
        let mut fields = Fields {
            redactor: &self.redactor,
            fields: BTreeMap::new(),
            bytes: 0,
        };
        event.record(&mut fields);
        let target: String = meta.target().chars().take(128).collect();
        history().lock().unwrap_or_else(|e| e.into_inner()).append(
            meta.level().as_str(),
            &target,
            fields.fields,
            chrono::Utc::now().timestamp(),
        );
    }
}
pub(crate) fn register(r: &mut impl AdminRegistrar) {
    r.admin_endpoint(
        Method::GET,
        "/logs/application",
        Capability::LogsRead,
        endpoint,
    );
}
async fn endpoint(
    ctx: AdminCtx,
    input: Query<Input>,
) -> Result<impl axum::response::IntoResponse, AdminApiError> {
    Ok((
        [(axum::http::header::CACHE_CONTROL, "private, no-store")],
        read(ctx, input).await?,
    ))
}
#[derive(Default, Deserialize)]
pub(crate) struct Input {
    q: Option<String>,
    level: Option<String>,
    target: Option<String>,
    cursor: Option<u64>,
    limit: Option<usize>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Page {
    events: Vec<Entry>,
    cursor: Option<u64>,
    discarded: u64,
    capacity: usize,
    retention_seconds: i64,
    process_local: bool,
}
async fn read(ctx: AdminCtx, Query(input): Query<Input>) -> Result<Json<Page>, AdminApiError> {
    ctx.require_unscoped(Capability::LogsRead)?;
    if input.q.as_ref().is_some_and(|v| v.len() > 256)
        || input.target.as_ref().is_some_and(|v| v.len() > 128)
        || input
            .level
            .as_deref()
            .is_some_and(|v| !matches!(v, "WARN" | "ERROR"))
    {
        return Err(AdminApiError::invalid(
            "invalid_filter",
            "Use a bounded search and WARN or ERROR severity.",
            "/",
        ));
    }
    let mut buffer = history().lock().unwrap_or_else(|e| e.into_inner());
    buffer.prune(chrono::Utc::now().timestamp());
    let limit = input.limit.unwrap_or(50).clamp(1, 100);
    let query = input.q.as_deref().unwrap_or("").to_lowercase();
    let mut events: Vec<_> = buffer
        .entries
        .iter()
        .rev()
        .filter(|e| {
            input.cursor.is_none_or(|id| e.id < id)
                && input.level.as_ref().is_none_or(|v| &e.level == v)
                && input.target.as_ref().is_none_or(|v| e.target.contains(v))
                && (query.is_empty()
                    || e.target.to_lowercase().contains(&query)
                    || e.fields.values().any(|v| v.to_lowercase().contains(&query)))
        })
        .take(limit + 1)
        .cloned()
        .collect();
    let more = events.len() > limit;
    events.truncate(limit);
    let cursor = if more {
        events.last().map(|e| e.id)
    } else {
        None
    };
    Ok(Json(Page {
        events,
        cursor,
        discarded: buffer.discarded,
        capacity: CAPACITY,
        retention_seconds: MAX_AGE,
        process_local: true,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::prelude::*;
    #[test]
    fn bounded_history_aggregates_prunes_and_preserves_cursor_order() {
        let mut h = History::default();
        h.append("WARN", "test", BTreeMap::new(), 1);
        h.append("WARN", "test", BTreeMap::new(), 2);
        assert_eq!(h.entries.len(), 1);
        assert_eq!(h.entries[0].repeat_count, 2);
        for i in 0..1100 {
            h.append("ERROR", &i.to_string(), BTreeMap::new(), 3);
        }
        assert_eq!(h.entries.len(), CAPACITY);
        assert_eq!(h.discarded, 101);
        assert!(
            h.entries
                .iter()
                .zip(h.entries.iter().skip(1))
                .all(|(a, b)| a.id < b.id)
        );
        h.prune(MAX_AGE + 4);
        assert!(h.entries.is_empty());
    }
    #[test]
    fn capture_redacts_secrets_and_omits_oversized_fields_and_info() {
        let subscriber = tracing_subscriber::registry().with(CaptureLayer::default());
        tracing::subscriber::with_default(subscriber, || {
            tracing::warn!(target:"capture-test", password="short", body="private mail", error="Bearer super-secret", "operation failed");
            tracing::error!(target:"capture-test-large", value=%"x".repeat(5000), "large error");
            tracing::info!(target:"capture-test-info", "omit this");
        });
        let h = history().lock().unwrap();
        let e = h
            .entries
            .iter()
            .find(|e| e.target == "capture-test")
            .unwrap();
        let json = serde_json::to_string(e).unwrap();
        assert!(!json.contains("short"));
        assert!(!json.contains("private mail"));
        assert!(!json.contains("super-secret"));
        assert!(h.entries.iter().any(|e| e.target == "capture-test-large"
            && e.fields["value"] == "[oversized field omitted]"));
        assert!(!h.entries.iter().any(|e| e.target == "capture-test-info"));
    }

    #[tokio::test]
    async fn viewer_requires_global_log_grant_and_filters_without_raw_export() {
        use crate::admin_ctx::DomainScope;
        let scoped = AdminCtx::with_grants(
            "scoped",
            [(
                Capability::LogsRead,
                DomainScope::Domains(["example.org".into()].into_iter().collect()),
            )]
            .into_iter()
            .collect(),
        )
        .for_route(Some(Capability::LogsRead));
        assert!(read(scoped, Query(Input::default())).await.is_err());
        let global = || {
            AdminCtx::with_grants(
                "operator",
                [(Capability::LogsRead, DomainScope::All)]
                    .into_iter()
                    .collect(),
            )
            .for_route(Some(Capability::LogsRead))
        };
        let invalid = Input {
            level: Some("DEBUG".into()),
            ..Default::default()
        };
        assert!(read(global(), Query(invalid)).await.is_err());
        history().lock().unwrap().append(
            "ERROR",
            "viewer-fixture",
            [("message".into(), "backend unavailable".into())]
                .into_iter()
                .collect(),
            chrono::Utc::now().timestamp(),
        );
        let Json(page) = read(
            global(),
            Query(Input {
                target: Some("viewer-fixture".into()),
                level: Some("ERROR".into()),
                q: Some("unavailable".into()),
                ..Default::default()
            }),
        )
        .await
        .unwrap();
        assert_eq!(page.events.len(), 1);
        assert!(page.process_local);
        assert_eq!(page.events[0].level, "ERROR");
    }
}
