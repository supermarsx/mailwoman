//! Account-preferences routes (t16 e18, SPEC §7.4/§19 · W12/W13/W15).
//!
//! The HTTP surface the Settings web UI (t16 e15) drives for the per-account
//! preferences that already persist server-side but had no route yet: signature
//! templates + notification rules (`mw-store` 0017), saved searches surfaced as
//! search folders (the FROZEN 0003 `saved_searches` table, reused — no new table),
//! and sending identities (the 0003 `identities` table).
//!
//! Every route is session-authed through [`crate::authed`] and account-scoped: the
//! account id comes from the authenticated session, never from the request body, so
//! one account can neither read nor mutate another's preferences. The 2FA and
//! session-management routes live in `twofa_routes.rs`; this module is purely the
//! prefs contract. `lib.rs`/e10 mounts [`prefs_router`] into the main router.
//!
//! ## Model notes
//! * Signatures/notification-rules are opaque, non-secret user preferences (no
//!   sealed columns); the notification rule set + quiet-hours window are serialized
//!   into the row's `rule_json` / `quiet_hours_json` blobs this module owns.
//! * Saved searches reuse the frozen 0003 table whose `user` column is the account
//!   id (matching `Mailbox/get`'s `list_saved_searches(account_id)` caller).
//! * Identities map to the 0003 `identities` rows. The web model's optional
//!   `signatureName` (a reference to a named signature TEMPLATE) has no column in the
//!   frozen schema and is NOT persisted here — see the DONE report's backend note;
//!   the JMAP-shaped `signature_html`/`signature_text` columns are left untouched so
//!   `Identity/get` semantics are not corrupted.

use axum::Json;
use axum::Router;
use axum::extract::{Path as UrlPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get};
use serde::{Deserialize, Serialize};
use serde_json::json;

use mw_store::{
    IdentityRow, NotificationRulesRow, SavedSearchRow, SignatureRow, Store, StoreError,
};

use crate::AppState;

// ─────────────────────────────────────────────────────────────────────────────
// Wire shapes (mirror apps/web/src/screens/Settings/types.ts exactly)
// ─────────────────────────────────────────────────────────────────────────────

/// A signature template (`mw-store` 0017 `signatures`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct SignatureDto {
    name: String,
    body: String,
    is_default: bool,
    /// Optional opaque JSON auto-apply rule; omitted when empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    rule: Option<String>,
}

/// A single notification rule (match → notify/mute).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
struct NotificationRuleDto {
    id: String,
    label: String,
    /// `match` is a Rust keyword; the wire/JSON field is `match`.
    r#match: String,
    /// "notify" | "mute" (opaque here).
    action: String,
}

/// A quiet-hours window (local 24h HH:MM).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct QuietHoursDto {
    enabled: bool,
    start: String,
    end: String,
}

impl Default for QuietHoursDto {
    fn default() -> Self {
        QuietHoursDto {
            enabled: false,
            start: "22:00".to_string(),
            end: "07:00".to_string(),
        }
    }
}

/// The account's notification configuration (`GET`/`PUT /api/account/notifications`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
struct NotificationConfigDto {
    enabled: bool,
    #[serde(default)]
    rules: Vec<NotificationRuleDto>,
    #[serde(default)]
    quiet_hours: QuietHoursDto,
}

/// A saved search surfaced as a virtual search folder (frozen 0003 `saved_searches`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct SavedSearchDto {
    /// Empty on create; the server assigns one and echoes it back.
    #[serde(default)]
    id: String,
    name: String,
    query_json: String,
    as_folder: bool,
}

/// A sending identity (0003 `identities`). `signatureName` is accepted for forward
/// compatibility but not persisted (no column in the frozen schema).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct IdentityDto {
    /// Empty on create; the server assigns one and echoes it back.
    #[serde(default)]
    id: String,
    name: String,
    email: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reply_to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    signature_name: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Signatures (W12)
// ─────────────────────────────────────────────────────────────────────────────

async fn signatures_list(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let account = match account_id(&state, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    match do_list_signatures(&state.store, &account).await {
        Ok(signatures) => Json(json!({ "signatures": signatures })).into_response(),
        Err(e) => server_error("list signatures", e),
    }
}

async fn signatures_upsert(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SignatureDto>,
) -> Response {
    let account = match account_id(&state, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    if body.name.trim().is_empty() {
        return bad_request("a signature needs a name");
    }
    match do_upsert_signature(&state.store, &account, &body).await {
        Ok(()) => ok(),
        Err(e) => server_error("upsert signature", e),
    }
}

async fn signatures_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(name): UrlPath<String>,
) -> Response {
    let account = match account_id(&state, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    match state.store.delete_signature(&account, &name).await {
        Ok(()) => ok(),
        Err(e) => server_error("delete signature", e),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Notification rules + quiet hours (W15)
// ─────────────────────────────────────────────────────────────────────────────

async fn notifications_get(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let account = match account_id(&state, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    match do_get_notifications(&state.store, &account).await {
        Ok(cfg) => Json(cfg).into_response(),
        Err(e) => server_error("get notifications", e),
    }
}

async fn notifications_put(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<NotificationConfigDto>,
) -> Response {
    let account = match account_id(&state, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    match do_put_notifications(&state.store, &account, &body).await {
        Ok(()) => ok(),
        Err(e) => server_error("put notifications", e),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Saved searches → search folders (W13, frozen 0003)
// ─────────────────────────────────────────────────────────────────────────────

async fn saved_searches_list(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let account = match account_id(&state, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    match do_list_saved_searches(&state.store, &account).await {
        Ok(searches) => Json(json!({ "savedSearches": searches })).into_response(),
        Err(e) => server_error("list saved searches", e),
    }
}

async fn saved_searches_put(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SavedSearchDto>,
) -> Response {
    let account = match account_id(&state, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    if body.name.trim().is_empty() {
        return bad_request("a saved search needs a name");
    }
    match do_upsert_saved_search(&state.store, &account, &body).await {
        Ok(Some(id)) => Json(json!({ "ok": true, "id": id })).into_response(),
        Ok(None) => not_found("no such saved search"),
        Err(e) => server_error("upsert saved search", e),
    }
}

async fn saved_searches_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
) -> Response {
    let account = match account_id(&state, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    match do_delete_saved_search(&state.store, &account, &id).await {
        Ok(true) => ok(),
        Ok(false) => not_found("no such saved search"),
        Err(e) => server_error("delete saved search", e),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Identities (0003 `identities`)
// ─────────────────────────────────────────────────────────────────────────────

async fn identities_list(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let account = match account_id(&state, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    match do_list_identities(&state.store, &account).await {
        Ok(identities) => Json(json!({ "identities": identities })).into_response(),
        Err(e) => server_error("list identities", e),
    }
}

async fn identities_upsert(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<IdentityDto>,
) -> Response {
    let account = match account_id(&state, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    if body.name.trim().is_empty() || body.email.trim().is_empty() {
        return bad_request("an identity needs a name and an email");
    }
    match do_upsert_identity(&state.store, &account, &body).await {
        Ok(Some(id)) => Json(json!({ "ok": true, "id": id })).into_response(),
        Ok(None) => not_found("no such identity"),
        Err(e) => server_error("upsert identity", e),
    }
}

async fn identities_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
) -> Response {
    let account = match account_id(&state, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    match do_delete_identity(&state.store, &account, &id).await {
        Ok(true) => ok(),
        Ok(false) => not_found("no such identity"),
        Err(e) => server_error("delete identity", e),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Store-scoped operations (account id is authoritative; unit-tested directly)
// ─────────────────────────────────────────────────────────────────────────────

async fn do_list_signatures(store: &Store, account: &str) -> Result<Vec<SignatureDto>, StoreError> {
    Ok(store
        .list_signatures(account)
        .await?
        .iter()
        .map(signature_to_dto)
        .collect())
}

async fn do_upsert_signature(
    store: &Store,
    account: &str,
    dto: &SignatureDto,
) -> Result<(), StoreError> {
    store
        .upsert_signature(&signature_from_dto(account, dto))
        .await
}

async fn do_get_notifications(
    store: &Store,
    account: &str,
) -> Result<NotificationConfigDto, StoreError> {
    Ok(match store.get_notification_rules(account).await? {
        Some(row) => notifications_to_dto(&row),
        // Unset account → sensible defaults (off, no rules, off quiet-hours).
        None => NotificationConfigDto::default(),
    })
}

async fn do_put_notifications(
    store: &Store,
    account: &str,
    cfg: &NotificationConfigDto,
) -> Result<(), StoreError> {
    store
        .put_notification_rules(&notifications_from_dto(account, cfg))
        .await
}

async fn do_list_saved_searches(
    store: &Store,
    account: &str,
) -> Result<Vec<SavedSearchDto>, StoreError> {
    Ok(store
        .list_saved_searches(account)
        .await?
        .iter()
        .map(saved_search_to_dto)
        .collect())
}

/// Upsert a saved search under `account`. A body carrying an id owned by ANOTHER
/// account is refused (`Ok(None)`) so an id cannot be hijacked; an empty/new id gets
/// a fresh uuid. Returns the row id on success.
async fn do_upsert_saved_search(
    store: &Store,
    account: &str,
    dto: &SavedSearchDto,
) -> Result<Option<String>, StoreError> {
    let id = if dto.id.trim().is_empty() {
        new_id()
    } else {
        // Reject an id that already belongs to someone else.
        if let Some(existing) = store.get_saved_search(&dto.id).await?
            && existing.user != account
        {
            return Ok(None);
        }
        dto.id.clone()
    };
    store
        .upsert_saved_search(&SavedSearchRow {
            id: id.clone(),
            user: account.to_string(),
            name: dto.name.clone(),
            query_json: dto.query_json.clone(),
            as_folder: dto.as_folder,
        })
        .await?;
    Ok(Some(id))
}

/// Delete a saved search iff it belongs to `account`. `Ok(false)` = absent or owned
/// by another account (indistinguishable to the caller).
async fn do_delete_saved_search(
    store: &Store,
    account: &str,
    id: &str,
) -> Result<bool, StoreError> {
    match store.get_saved_search(id).await? {
        Some(row) if row.user == account => {
            store.delete_saved_search(id).await?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

async fn do_list_identities(store: &Store, account: &str) -> Result<Vec<IdentityDto>, StoreError> {
    Ok(store
        .list_identities(account)
        .await?
        .iter()
        .map(identity_to_dto)
        .collect())
}

/// Upsert an identity under `account`. A body carrying an id owned by ANOTHER account
/// is refused (`Ok(None)`). New identities are `source = "configured"`; the
/// JMAP-shaped signature columns are left null (see module note on `signatureName`).
async fn do_upsert_identity(
    store: &Store,
    account: &str,
    dto: &IdentityDto,
) -> Result<Option<String>, StoreError> {
    let id = if dto.id.trim().is_empty() {
        new_id()
    } else {
        if let Some(existing) = store.get_identity(&dto.id).await?
            && existing.account_id != account
        {
            return Ok(None);
        }
        dto.id.clone()
    };
    store
        .upsert_identity(&IdentityRow {
            id: id.clone(),
            account_id: account.to_string(),
            name: dto.name.clone(),
            email: dto.email.clone(),
            reply_to: dto.reply_to.clone(),
            signature_html: None,
            signature_text: None,
            sent_mailbox_id: None,
            source: "configured".to_string(),
        })
        .await?;
    Ok(Some(id))
}

/// Delete an identity iff it belongs to `account`. `Ok(false)` = absent or foreign.
async fn do_delete_identity(store: &Store, account: &str, id: &str) -> Result<bool, StoreError> {
    match store.get_identity(id).await? {
        Some(row) if row.account_id == account => {
            store.delete_identity(id).await?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Row ⇄ DTO mapping
// ─────────────────────────────────────────────────────────────────────────────

fn signature_to_dto(row: &SignatureRow) -> SignatureDto {
    SignatureDto {
        name: row.name.clone(),
        body: row.body.clone(),
        is_default: row.is_default,
        rule: if row.rule_json.is_empty() {
            None
        } else {
            Some(row.rule_json.clone())
        },
    }
}

fn signature_from_dto(account: &str, dto: &SignatureDto) -> SignatureRow {
    SignatureRow {
        account_id: account.to_string(),
        name: dto.name.clone(),
        body: dto.body.clone(),
        is_default: dto.is_default,
        rule_json: dto.rule.clone().unwrap_or_default(),
        updated_at: String::new(),
    }
}

fn notifications_to_dto(row: &NotificationRulesRow) -> NotificationConfigDto {
    // The rule set / quiet-hours JSON blobs are ours to shape; junk falls back to
    // an empty set / default window rather than failing the read.
    NotificationConfigDto {
        enabled: row.enabled,
        rules: serde_json::from_str(&row.rule_json).unwrap_or_default(),
        quiet_hours: serde_json::from_str(&row.quiet_hours_json).unwrap_or_default(),
    }
}

fn notifications_from_dto(account: &str, cfg: &NotificationConfigDto) -> NotificationRulesRow {
    NotificationRulesRow {
        account_id: account.to_string(),
        rule_json: serde_json::to_string(&cfg.rules).unwrap_or_else(|_| "[]".to_string()),
        quiet_hours_json: serde_json::to_string(&cfg.quiet_hours).unwrap_or_default(),
        enabled: cfg.enabled,
        updated_at: String::new(),
    }
}

fn saved_search_to_dto(row: &SavedSearchRow) -> SavedSearchDto {
    SavedSearchDto {
        id: row.id.clone(),
        name: row.name.clone(),
        query_json: row.query_json.clone(),
        as_folder: row.as_folder,
    }
}

fn identity_to_dto(row: &IdentityRow) -> IdentityDto {
    IdentityDto {
        id: row.id.clone(),
        name: row.name.clone(),
        email: row.email.clone(),
        reply_to: row.reply_to.clone(),
        // Not persisted in the frozen schema (see module note).
        signature_name: None,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Router + small helpers
// ─────────────────────────────────────────────────────────────────────────────

/// The account-preferences routes. `lib.rs`/e10 merges this into the main router;
/// every route is session-authed and rides the normal CSRF guard like the rest of
/// the `/api/account/*` surface.
pub(crate) fn prefs_router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/account/signatures",
            get(signatures_list).post(signatures_upsert),
        )
        .route("/api/account/signatures/{name}", delete(signatures_delete))
        .route(
            "/api/account/notifications",
            get(notifications_get).put(notifications_put),
        )
        .route(
            "/api/account/saved-searches",
            get(saved_searches_list).put(saved_searches_put),
        )
        .route(
            "/api/account/saved-searches/{id}",
            delete(saved_searches_delete),
        )
        .route(
            "/api/account/identities",
            get(identities_list).post(identities_upsert),
        )
        .route("/api/account/identities/{id}", delete(identities_delete))
}

/// The authenticated caller's account id, or an early auth `Response` to return.
async fn account_id(state: &AppState, headers: &HeaderMap) -> Result<String, Response> {
    crate::authed(state, headers).await.map(|s| s.account_id)
}

/// A fresh opaque row id for a newly created saved search / identity.
fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn ok() -> Response {
    Json(json!({ "ok": true })).into_response()
}

fn bad_request(msg: &str) -> Response {
    (StatusCode::BAD_REQUEST, Json(json!({ "error": msg }))).into_response()
}

fn not_found(msg: &str) -> Response {
    (StatusCode::NOT_FOUND, Json(json!({ "error": msg }))).into_response()
}

/// Log a store error and return an opaque 500 (never leaks the internal error).
fn server_error(what: &str, e: StoreError) -> Response {
    tracing::error!("prefs: {what} failed: {e}");
    (StatusCode::INTERNAL_SERVER_ERROR, "server error").into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mw_store::{AccountKind, Credentials, NewAccount, ServerKey};

    async fn store() -> Store {
        Store::open_in_memory(ServerKey::generate()).await.unwrap()
    }

    /// Seed a real account row and return its id. `identities` has a FK to
    /// `accounts(id)`, so identity round-trips need a live account (unlike the
    /// FK-free signatures / notification-rules / saved-searches tables).
    async fn seed_account(s: &Store, username: &str) -> String {
        s.create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "h",
                port: 993,
                tls: "implicit",
                username,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: username.into(),
                password: "p".into(),
            },
        )
        .await
        .unwrap()
    }

    // ── signatures ────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn signatures_round_trip_and_mapping() {
        let s = store().await;
        assert!(do_list_signatures(&s, "a1").await.unwrap().is_empty());

        do_upsert_signature(
            &s,
            "a1",
            &SignatureDto {
                name: "work".into(),
                body: "Regards".into(),
                is_default: true,
                rule: Some("{\"x\":1}".into()),
            },
        )
        .await
        .unwrap();
        do_upsert_signature(
            &s,
            "a1",
            &SignatureDto {
                name: "personal".into(),
                body: "Cheers".into(),
                is_default: false,
                rule: None,
            },
        )
        .await
        .unwrap();

        let list = do_list_signatures(&s, "a1").await.unwrap();
        assert_eq!(list.len(), 2);
        let work = list.iter().find(|x| x.name == "work").unwrap();
        assert!(work.is_default);
        assert_eq!(work.rule.as_deref(), Some("{\"x\":1}"));
        // An empty rule maps to None (omitted on the wire), not "".
        let personal = list.iter().find(|x| x.name == "personal").unwrap();
        assert_eq!(personal.rule, None);

        s.delete_signature("a1", "personal").await.unwrap();
        assert_eq!(do_list_signatures(&s, "a1").await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn signatures_are_account_scoped() {
        let s = store().await;
        do_upsert_signature(
            &s,
            "a1",
            &SignatureDto {
                name: "work".into(),
                body: "b".into(),
                is_default: false,
                rule: None,
            },
        )
        .await
        .unwrap();
        // Another account sees nothing of a1's signatures.
        assert!(do_list_signatures(&s, "a2").await.unwrap().is_empty());
    }

    // ── notifications ─────────────────────────────────────────────────────────

    #[tokio::test]
    async fn notifications_default_when_unset() {
        let s = store().await;
        let cfg = do_get_notifications(&s, "a1").await.unwrap();
        assert_eq!(cfg, NotificationConfigDto::default());
        assert!(!cfg.enabled);
        assert!(cfg.rules.is_empty());
        assert_eq!(cfg.quiet_hours.start, "22:00");
    }

    #[tokio::test]
    async fn notifications_round_trip_rules_and_quiet_hours() {
        let s = store().await;
        let cfg = NotificationConfigDto {
            enabled: true,
            rules: vec![NotificationRuleDto {
                id: "r1".into(),
                label: "VIP".into(),
                r#match: "boss@example.com".into(),
                action: "notify".into(),
            }],
            quiet_hours: QuietHoursDto {
                enabled: true,
                start: "23:00".into(),
                end: "06:30".into(),
            },
        };
        do_put_notifications(&s, "a1", &cfg).await.unwrap();
        let got = do_get_notifications(&s, "a1").await.unwrap();
        assert_eq!(got, cfg);
        assert_eq!(got.rules[0].r#match, "boss@example.com");
        assert!(got.quiet_hours.enabled);

        // Account isolation: a2 still sees defaults.
        assert_eq!(
            do_get_notifications(&s, "a2").await.unwrap(),
            NotificationConfigDto::default()
        );
    }

    // ── saved searches (frozen 0003, reused) ──────────────────────────────────

    #[tokio::test]
    async fn saved_searches_reuse_0003_round_trip() {
        let s = store().await;
        let id = do_upsert_saved_search(
            &s,
            "a1",
            &SavedSearchDto {
                id: String::new(),
                name: "Unread".into(),
                query_json: "{\"unread\":true}".into(),
                as_folder: true,
            },
        )
        .await
        .unwrap()
        .unwrap();
        assert!(!id.is_empty());

        let list = do_list_saved_searches(&s, "a1").await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, id);
        assert_eq!(list[0].query_json, "{\"unread\":true}");
        assert!(list[0].as_folder);

        // Update in place (same id) keeps the count at one.
        do_upsert_saved_search(
            &s,
            "a1",
            &SavedSearchDto {
                id: id.clone(),
                name: "Unread mail".into(),
                query_json: "{\"unread\":true}".into(),
                as_folder: false,
            },
        )
        .await
        .unwrap();
        let list = do_list_saved_searches(&s, "a1").await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Unread mail");

        assert!(do_delete_saved_search(&s, "a1", &id).await.unwrap());
        assert!(do_list_saved_searches(&s, "a1").await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn saved_searches_cross_account_is_denied() {
        let s = store().await;
        let id = do_upsert_saved_search(
            &s,
            "a1",
            &SavedSearchDto {
                id: String::new(),
                name: "Mine".into(),
                query_json: "{}".into(),
                as_folder: false,
            },
        )
        .await
        .unwrap()
        .unwrap();

        // a2 cannot delete a1's saved search…
        assert!(!do_delete_saved_search(&s, "a2", &id).await.unwrap());
        // …nor overwrite it by presenting its id.
        assert!(
            do_upsert_saved_search(
                &s,
                "a2",
                &SavedSearchDto {
                    id: id.clone(),
                    name: "Hijack".into(),
                    query_json: "{}".into(),
                    as_folder: false,
                },
            )
            .await
            .unwrap()
            .is_none()
        );
        // The original owner still sees it unchanged.
        let list = do_list_saved_searches(&s, "a1").await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Mine");
    }

    // ── identities ────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn identities_round_trip() {
        let s = store().await;
        let a1 = seed_account(&s, "a1@ex").await;
        let id = do_upsert_identity(
            &s,
            &a1,
            &IdentityDto {
                id: String::new(),
                name: "Sales".into(),
                email: "sales@example.com".into(),
                reply_to: Some("help@example.com".into()),
                signature_name: Some("work".into()),
            },
        )
        .await
        .unwrap()
        .unwrap();

        let list = do_list_identities(&s, &a1).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, id);
        assert_eq!(list[0].email, "sales@example.com");
        assert_eq!(list[0].reply_to.as_deref(), Some("help@example.com"));

        assert!(do_delete_identity(&s, &a1, &id).await.unwrap());
        assert!(do_list_identities(&s, &a1).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn identities_cross_account_is_denied() {
        let s = store().await;
        let a1 = seed_account(&s, "a1@ex").await;
        let a2 = seed_account(&s, "a2@ex").await;
        let id = do_upsert_identity(
            &s,
            &a1,
            &IdentityDto {
                id: String::new(),
                name: "Sales".into(),
                email: "sales@example.com".into(),
                reply_to: None,
                signature_name: None,
            },
        )
        .await
        .unwrap()
        .unwrap();

        // a2 cannot delete or hijack a1's identity.
        assert!(!do_delete_identity(&s, &a2, &id).await.unwrap());
        assert!(
            do_upsert_identity(
                &s,
                &a2,
                &IdentityDto {
                    id: id.clone(),
                    name: "Evil".into(),
                    email: "evil@example.com".into(),
                    reply_to: None,
                    signature_name: None,
                },
            )
            .await
            .unwrap()
            .is_none()
        );
        assert!(do_list_identities(&s, &a2).await.unwrap().is_empty());
        assert_eq!(
            do_list_identities(&s, &a1).await.unwrap()[0].email,
            "sales@example.com"
        );
    }
}
