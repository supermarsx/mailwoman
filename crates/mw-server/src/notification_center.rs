//! Durable, principal-scoped notifications. Dismissing/archiving presentation
//! never acknowledges a security warning or revokes its underlying evidence.
use crate::{AppState, security_alerts::evidence::Evidence};
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

const PREFIX: &str = "notification-center:v1:";
const LIMIT: usize = 500;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Notice {
    pub id: String,
    pub ts: i64,
    pub kind: String,
    pub event: String,
    pub severity: String,
    pub state: String,
    pub read: bool,
    pub context: Option<Evidence>,
}
fn prefix(_owner: &str) -> String {
    PREFIX.into()
}
fn key(owner: &str, id: &str) -> String {
    format!("{}{}", prefix(owner), crate::push_relay::hash_token(id))
}
pub(crate) async fn account_owner(state: &AppState, account: &str) -> Option<String> {
    let sessions = state.store.sessions_by_account(account).await.ok()?;
    let mut owner = None;
    for session in sessions {
        let next = state
            .store
            .session_personal_subject(&session.id)
            .await
            .ok()??;
        if owner.as_ref().is_some_and(|old| old != &next) {
            return None;
        }
        owner = Some(next);
    }
    if owner.is_some() {
        owner
    } else {
        state
            .store
            .native_personal_subject(account)
            .await
            .ok()
            .flatten()
    }
}
pub(crate) async fn emit(
    state: &AppState,
    owner: &str,
    kind: &str,
    event: &str,
    severity: &str,
    context: Option<Evidence>,
) {
    let notice = Notice {
        id: uuid::Uuid::new_v4().to_string(),
        ts: chrono::Utc::now().timestamp(),
        kind: kind.into(),
        event: event.into(),
        severity: severity.into(),
        state: "active".into(),
        read: false,
        context,
    };
    if let Err(error) = save(&state.store, owner, &notice).await {
        tracing::warn!("notification centre write failed: {error}");
    }
}
async fn save(
    store: &mw_store::Store,
    owner: &str,
    notice: &Notice,
) -> Result<(), mw_store::StoreError> {
    store
        .put_personal_setting(
            owner,
            &key(owner, &notice.id),
            &serde_json::to_string(notice).expect("notification serializes"),
        )
        .await?;
    let mut rows = store.personal_records(owner, &prefix(owner)).await?;
    if rows.len() > LIMIT {
        rows.sort_by_key(|(_, value)| {
            serde_json::from_str::<Notice>(value)
                .map(|n| n.ts)
                .unwrap_or(0)
        });
        let count = rows.len() - LIMIT;
        for (key, _) in rows.into_iter().take(count) {
            store.delete_personal_setting(owner, &key).await?;
        }
    }
    Ok(())
}
pub(crate) fn router() -> Router<AppState> {
    Router::new().route("/api/account/notification-center", get(list).post(change))
}
fn private(response: impl IntoResponse) -> Response {
    let mut response = response.into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, "private, no-store".parse().unwrap());
    response
}
async fn owner(state: &AppState, headers: &HeaderMap, mutation: bool) -> Result<String, Response> {
    let session = crate::authed(state, headers).await?;
    crate::prefs_routes::personal_storage::record_owner(state, &session, headers, mutation).await
}
async fn list(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let owner = match owner(&state, &headers, false).await {
        Ok(owner) => owner,
        Err(response) => return response,
    };
    let rows = match state.store.personal_records(&owner, &prefix(&owner)).await {
        Ok(rows) => rows,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let mut notices: Vec<Notice> = rows
        .into_iter()
        .filter_map(|(_, value)| serde_json::from_str(&value).ok())
        .collect();
    notices.sort_by(|a, b| b.ts.cmp(&a.ts).then_with(|| b.id.cmp(&a.id)));
    private(Json(json!({"notices":notices})))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Change {
    id: String,
    action: String,
}
fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 256 && !id.chars().any(char::is_control)
}
async fn change(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(change): Json<Change>,
) -> Response {
    let owner = match owner(&state, &headers, true).await {
        Ok(owner) => owner,
        Err(response) => return response,
    };
    if !valid_id(&change.id)
        || !matches!(
            change.action.as_str(),
            "dismiss" | "archive" | "restore" | "read" | "publish-update"
        )
    {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let raw = match state
        .store
        .personal_record(&owner, &key(&owner, &change.id))
        .await
    {
        Ok(raw) => raw,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let mut notice = match raw.and_then(|raw| serde_json::from_str::<Notice>(&raw).ok()) {
        Some(notice) => notice,
        None if change.action == "publish-update" && change.id.starts_with("update:") => Notice {
            id: change.id.clone(),
            ts: chrono::Utc::now().timestamp(),
            kind: "update".into(),
            event: "newVersion".into(),
            severity: "info".into(),
            state: "active".into(),
            read: false,
            context: None,
        },
        None => return StatusCode::NOT_FOUND.into_response(),
    };
    match change.action.as_str() {
        "dismiss" => {
            notice.state = "dismissed".into();
            notice.read = true;
        }
        "archive" => {
            notice.state = "archived".into();
            notice.read = true;
        }
        "restore" => notice.state = "active".into(),
        "read" => notice.read = true,
        _ => {}
    }
    match save(&state.store, &owner, &notice).await {
        Ok(()) => private(Json(json!({"notice":notice}))),
        Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn notices_and_states_are_separate_for_different_verified_owners() {
        let store = mw_store::Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        let mut notice = Notice {
            id: "same".into(),
            ts: 1,
            kind: "security".into(),
            event: "countryChange".into(),
            severity: "warning".into(),
            state: "active".into(),
            read: false,
            context: None,
        };
        save(&store, "owner-a", &notice).await.unwrap();
        notice.state = "archived".into();
        save(&store, "owner-b", &notice).await.unwrap();
        let a: Notice = serde_json::from_str(
            &store
                .personal_record("owner-a", &key("owner-a", "same"))
                .await
                .unwrap()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(a.state, "active");
        let b: Notice = serde_json::from_str(
            &store
                .personal_record("owner-b", &key("owner-b", "same"))
                .await
                .unwrap()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(b.state, "archived");
        assert!(valid_id("update:26.19+build"));
        assert!(!valid_id("bad\nidentifier"));
    }
}
