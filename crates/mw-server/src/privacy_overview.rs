//! `GET /api/privacy/overview`: what is stored about the signed-in person, in
//! plain categories, with counts, approximate sizes, the retention that applies,
//! who processes it, their consents and their restriction status.
//!
//! The category list is the data map of this server for one person. Counts come
//! from the store (`Store::privacy_footprint`); retention comes from the
//! `retention.*` settings and, through `user_retention::effective_for_account`
//! (the document `GET /api/retention/effective` serves), the person's own rules.
use crate::{AppState, consent_ledger, restriction};
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, header},
    response::{IntoResponse, Response},
    routing::get,
};
use serde_json::{Value, json};

pub(crate) fn router() -> Router<AppState> {
    Router::new().route("/api/privacy/overview", get(overview))
}

/// A category of stored data.
struct Category {
    id: &'static str,
    /// The footprint key(s) summed for this category.
    keys: &'static [&'static str],
    /// The `retention.*` setting that bounds it, if any.
    retention_key: Option<&'static str>,
    /// Processors that handle the category when `active` is true.
    processors: &'static [&'static str],
    /// A processor-bearing category is listed as processed only when its consent
    /// purpose allows it.
    purpose: Option<&'static str>,
}

const CATEGORIES: &[Category] = &[
    Category { id: "mail", keys: &["mail"], retention_key: None, processors: &["mailServer"], purpose: None },
    Category { id: "contacts", keys: &["contacts"], retention_key: None, processors: &["mailServer"], purpose: None },
    Category { id: "calendars", keys: &["calendars"], retention_key: None, processors: &[], purpose: None },
    Category { id: "notes", keys: &["notes"], retention_key: None, processors: &[], purpose: None },
    Category { id: "rules", keys: &["rules"], retention_key: None, processors: &[], purpose: None },
    Category { id: "devices", keys: &["sessions", "devices"], retention_key: Some("retention.sessions.idleDays"), processors: &[], purpose: None },
    Category { id: "signInHistory", keys: &["signInHistory"], retention_key: Some("retention.ipAddresses.days"), processors: &[], purpose: None },
    Category { id: "audit", keys: &["audit"], retention_key: Some("retention.passwordChangeAudit.days"), processors: &[], purpose: None },
    Category { id: "assistant", keys: &["searchEmbeddings"], retention_key: Some("retention.assistAudit.days"), processors: &["assistantProvider"], purpose: Some("assistant") },
    Category { id: "push", keys: &["push"], retention_key: None, processors: &["pushService"], purpose: Some("push") },
    Category { id: "tracking", keys: &["tracking"], retention_key: None, processors: &[], purpose: Some("openTracking") },
    Category { id: "apiKeys", keys: &["apiKeys"], retention_key: None, processors: &[], purpose: Some("integrations") },
    Category { id: "consents", keys: &["consents"], retention_key: Some("consent.receipts.retentionDays"), processors: &[], purpose: None },
    Category { id: "requests", keys: &["requests"], retention_key: Some("dsr.retention.completedDays"), processors: &[], purpose: None },
];

/// The retention that applies to a category, from the same document
/// `GET /api/retention/effective` returns (`user_retention::effective_for_account`):
/// the person's own rules and the administrator's mandatory folder retention for
/// mail, and the class-clamped system retention for the stores that hold a
/// person's data. `None` keeps the administrator-setting fallback in `build`.
fn effective_days(effective: &Value, category: &str) -> Option<(&'static str, i64)> {
    if category == "mail" {
        return effective["mail"]
            .as_array()?
            .iter()
            .filter_map(|m| {
                let days = m["days"].as_i64().filter(|d| *d > 0)?;
                Some((if m["source"] == "you" { "userRule" } else { "setting" }, days))
            })
            .min_by_key(|(_, d)| *d);
    }
    let store = match category {
        "devices" => "sessions",
        "signInHistory" => "authHistory",
        "audit" => "passwordChangeAudit",
        "tracking" => "openTracking",
        "requests" => "dsrRequests",
        _ => return None,
    };
    let entry = effective["system"].as_array()?.iter().find(|e| e["id"] == store)?;
    entry["days"].as_i64().filter(|d| *d > 0).map(|d| ("setting", d))
}

async fn overview(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    match build(&state, &session, &headers).await {
        Ok(v) => ([(header::CACHE_CONTROL, "no-store")], Json(v)).into_response(),
        Err(_) => axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}

async fn build(state: &AppState, session: &mw_store::Session, headers: &HeaderMap) -> Result<Value, mw_store::StoreError> {
    let subject = crate::settings_routes::UserSubject::resolved(&state.store, session).await?;
    let account = subject.account_id.clone();
    let footprint = state.store.privacy_footprint(&account).await?;
    let retention = crate::settings_registry::effective_all(&state.store, &subject.as_subject(), |d| {
        d.key.starts_with("retention.") || d.key == "consent.receipts.retentionDays" || d.key == "dsr.retention.completedDays"
    })
    .await
    .unwrap_or_default();
    let effective = crate::user_retention::effective_for_account(state, &account, &subject.address).await;
    let consents = consent_ledger::own_json(state, session, "en").await?;
    let granted = |purpose: &str| {
        consents["purposes"]
            .as_array()
            .and_then(|a| a.iter().find(|p| p["id"] == purpose))
            .is_some_and(|p| p["state"] == "granted" || (p["state"] == "unset" && p["basis"] != "consent"))
    };
    let mut categories = Vec::new();
    for c in CATEGORIES {
        let (rows, bytes) = c.keys.iter().fold((0, 0), |acc, k| {
            let (n, b) = footprint.items.get(*k).copied().unwrap_or((0, 0));
            (acc.0 + n, acc.1 + b)
        });
        let days = effective_days(&effective, c.id).or_else(|| {
            c.retention_key.and_then(|k| {
                retention
                    .iter()
                    .find(|e| e.key == k)
                    .and_then(|e| e.value.as_i64())
                    .filter(|d| *d > 0)
                    .map(|d| ("setting", d))
            })
        });
        let processors: Vec<&str> = if rows > 0 || c.id == "assistant" {
            c.processors.iter().copied().filter(|_| c.purpose.is_none_or(|p| granted(p))).collect()
        } else {
            Vec::new()
        };
        categories.push(json!({
            "id": c.id,
            "rows": rows,
            "bytes": bytes,
            "retention": days.map(|(source, d)| json!({"days": d, "source": source})),
            "processors": processors,
            "purpose": c.purpose,
        }));
    }
    let _ = headers;
    Ok(json!({
        "account": {"address": subject.address, "domain": subject.domain},
        "categories": categories,
        "consents": consents,
        "restriction": restriction::status_json(state, &account).await,
        // Optional pages: another feature publishes them; the client links to
        // them only when they exist.
        "links": {"policy": "/privacy", "processing": "/api/privacy/processing"},
        "shortcuts": {"download": "access", "delete": "erasure"},
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mail_retention_is_the_shortest_positive_period_and_a_rule_wins_the_label() {
        let effective = json!({
            "mail": [
                {"category": "folder:trash", "days": 30, "action": "delete", "source": "administrator"},
                {"category": "rule", "days": 14, "action": "trash", "source": "you"},
                {"category": "rule", "days": null, "action": "keep", "source": "you"},
            ],
            "system": [
                {"id": "sessions", "days": 60, "keptForever": false},
                {"id": "openTracking", "days": 0, "keptForever": true},
            ],
        });
        assert_eq!(effective_days(&effective, "mail"), Some(("userRule", 14)));
        assert_eq!(effective_days(&effective, "devices"), Some(("setting", 60)));
        assert_eq!(effective_days(&effective, "tracking"), None, "kept forever shows no period");
        assert_eq!(effective_days(&effective, "notes"), None);
        assert_eq!(effective_days(&json!({"mail": [{"days": 30, "source": "administrator"}]}), "mail"), Some(("setting", 30)));
    }
}
