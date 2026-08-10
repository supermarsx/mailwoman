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
//!   `signatureName` (a reference to a named signature TEMPLATE) persists in the
//!   `signature_name` column added by migration 0020; the JMAP-shaped
//!   `signature_html`/`signature_text` columns are left untouched so `Identity/get`
//!   semantics are not corrupted.
//! * Appearance preferences (t19 e13, SPEC §17.3) are one opaque JSON object per
//!   account on the EXISTING `settings` key/value store — no table, no migration
//!   (t19 DQ-4). See the appearance section below for why the server keeps the
//!   payload opaque.

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

/// A sending identity (0003 `identities`). `signatureName` names the signature
/// TEMPLATE this identity applies; it persists in the 0020 `signature_name` column.
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
// Appearance preferences (t19 e13, SPEC §17.3 — "synced server-side per user")
//
// Theme pack, light/dark pair, mode (fixed/system/schedule), schedule window,
// density, accent, UI font and layout are ONE object. The client
// (`apps/web/src/theme/appearance.ts`) owns its shape and its validation; this
// module stores it as an opaque JSON object and is deliberately blind to the
// field names.
//
// That is a decision, not an omission. The set of valid theme ids lives in the
// TypeScript theme registry, which gains packs every time a pack ships; a Rust
// mirror of that union would silently drift and start rejecting themes the SPA
// can render. Instead the server enforces the properties it CAN own and the
// client re-validates on read (`parseAppearancePrefs` degrades a bad field to
// its default rather than adopting it):
//
//   * account scope — the key is derived from the SESSION's account id, never
//     from the body, so one account cannot read or write another's appearance;
//   * shape — the payload must be a JSON OBJECT (an array/number/string could
//     never round-trip into the client's preference set);
//   * size — [`MAX_APPEARANCE_BYTES`], so the KV cannot be used as free storage;
//   * `updatedAt` — stamped from the SERVER clock on every write. A client clock
//     never decides which value is newer.
//
// Storage is the existing `settings` key/value table (`mw-store` `get_setting`/
// `set_setting`), keyed `appearance:<account-id>` — no new table and no
// migration (t19 DQ-4). `set_setting` has no delete, so a reset writes an
// envelope with `prefs: null`; that keeps `updatedAt` moving forward, which is
// what lets a reset win over a device still holding the older value.
// ─────────────────────────────────────────────────────────────────────────────

/// Cap on one account's serialized appearance object. The real payload is a few
/// hundred bytes; this is the abuse ceiling, not a target.
const MAX_APPEARANCE_BYTES: usize = 8 * 1024;

/// `settings` key holding one account's appearance object.
fn appearance_key(account: &str) -> String {
    format!("appearance:{account}")
}

/// The stored envelope, as READ back. `prefs` is the client's opaque object, or
/// `null` after a reset. `v` lets a future shape change be recognized instead of
/// guessed at. Writes build the same shape with `json!` (see
/// [`do_put_appearance`]) because `Value`'s `Display` cannot fail, which keeps
/// an unreachable serialization error path out of the write path.
#[derive(Debug, Clone, Deserialize, PartialEq)]
struct AppearanceEnvelope {
    #[allow(
        dead_code,
        reason = "read for shape recognition; only v = 1 exists today"
    )]
    v: u32,
    /// Server-stamped milliseconds since the Unix epoch.
    updated_at: i64,
    prefs: Option<serde_json::Value>,
}

/// `GET /api/account/appearance` — the account's stored appearance, plus the
/// deployment default so a client with nothing stored can paint the operator's
/// branding instead of the built-in one.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct AppearanceResponse {
    /// The stored object, or `null` when this account has never saved one.
    appearance: Option<serde_json::Value>,
    /// When it was stored (server clock, ms), or `null`.
    updated_at: Option<i64>,
    /// The admin `[appearance]` section. A DEFAULT, never an enforcement — see
    /// `crates/mw-admin/src/config.rs`.
    deployment_default: DeploymentAppearanceDto,
}

/// The deployment-wide appearance default (`mw_admin::Appearance`), in the same
/// camelCase shape the admin panel already receives from `admin.rs`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
struct DeploymentAppearanceDto {
    theme: String,
    brand_name: String,
    #[serde(default)]
    accent: Option<String>,
}

/// `PUT /api/account/appearance` body.
#[derive(Debug, Clone, Deserialize)]
struct AppearanceRequest {
    appearance: serde_json::Value,
}

async fn appearance_get(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let account = match account_id(&state, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    let cfg = state.v6.admin.config().appearance;
    match do_get_appearance(&state.store, &account).await {
        Ok(stored) => Json(AppearanceResponse {
            appearance: stored.as_ref().and_then(|e| e.prefs.clone()),
            updated_at: stored.as_ref().map(|e| e.updated_at),
            deployment_default: DeploymentAppearanceDto {
                theme: cfg.theme,
                brand_name: cfg.brand_name,
                accent: cfg.accent,
            },
        })
        .into_response(),
        Err(e) => server_error("get appearance", e),
    }
}

async fn appearance_put(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AppearanceRequest>,
) -> Response {
    let account = match account_id(&state, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    if !body.appearance.is_object() {
        return bad_request("appearance preferences must be a JSON object");
    }
    if serialized_len(&body.appearance) > MAX_APPEARANCE_BYTES {
        return payload_too_large("appearance preferences are too large");
    }
    match do_put_appearance(&state.store, &account, Some(body.appearance), now_ms()).await {
        Ok(updated_at) => Json(json!({ "ok": true, "updatedAt": updated_at })).into_response(),
        Err(e) => server_error("put appearance", e),
    }
}

/// `DELETE /api/account/appearance` — forget this account's appearance, so the
/// client falls back to the deployment default. Stores a `prefs: null` envelope
/// rather than removing the key (see the section note on `updatedAt`).
async fn appearance_delete(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let account = match account_id(&state, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    match do_put_appearance(&state.store, &account, None, now_ms()).await {
        Ok(updated_at) => Json(json!({ "ok": true, "updatedAt": updated_at })).into_response(),
        Err(e) => server_error("delete appearance", e),
    }
}

/// Milliseconds since the Unix epoch, from the SERVER clock. A pre-epoch clock
/// (only reachable on a badly misconfigured host) saturates at 0 rather than
/// wrapping negative.
fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

/// Serialized byte length of a JSON value, for the size cap. A value that cannot
/// be serialized is reported as over the cap so it is refused, never accepted.
fn serialized_len(v: &serde_json::Value) -> usize {
    serde_json::to_string(v)
        .map(|s| s.len())
        .unwrap_or(usize::MAX)
}

// ─────────────────────────────────────────────────────────────────────────────
// Store-scoped operations (account id is authoritative; unit-tested directly)
// ─────────────────────────────────────────────────────────────────────────────

/// This account's stored appearance envelope, or `None` when nothing is stored.
/// A corrupt/legacy value reads as `None` rather than failing the request — the
/// next write replaces it.
async fn do_get_appearance(
    store: &Store,
    account: &str,
) -> Result<Option<AppearanceEnvelope>, StoreError> {
    Ok(store
        .get_setting(&appearance_key(account))
        .await?
        .and_then(|raw| serde_json::from_str::<AppearanceEnvelope>(&raw).ok()))
}

/// Replace this account's appearance wholesale (`None` = reset). Returns the
/// stamped `updated_at`. There is no merge: the client sends its complete
/// preference set, so last write wins per account.
async fn do_put_appearance(
    store: &Store,
    account: &str,
    prefs: Option<serde_json::Value>,
    updated_at: i64,
) -> Result<i64, StoreError> {
    let envelope = json!({ "v": 1, "updated_at": updated_at, "prefs": prefs });
    store
        .set_setting(&appearance_key(account), &envelope.to_string())
        .await?;
    Ok(updated_at)
}

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
/// is refused (`Ok(None)`). New identities are `source = "configured"`; `signatureName`
/// persists in the 0020 column while the JMAP-shaped signature columns are left null
/// (see module note on `signatureName`).
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
            signature_name: dto.signature_name.clone(),
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
        signature_name: row.signature_name.clone(),
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
        // t19 e13 (SPEC §17.3): per-user appearance, one opaque object on the
        // existing settings KV. DELETE resets to the deployment default.
        .route(
            "/api/account/appearance",
            get(appearance_get)
                .put(appearance_put)
                .delete(appearance_delete),
        )
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

fn payload_too_large(msg: &str) -> Response {
    (StatusCode::PAYLOAD_TOO_LARGE, Json(json!({ "error": msg }))).into_response()
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
        // signatureName persists (0020 column) and survives the GET round-trip.
        assert_eq!(list[0].signature_name.as_deref(), Some("work"));

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

    // ── appearance (t19 e13, SPEC §17.3) ──────────────────────────────────────

    fn prefs_value() -> serde_json::Value {
        json!({
            "mode": "system",
            "theme": "ocean-dark",
            "lightTheme": "ocean-light",
            "darkTheme": "ocean-dark",
            "schedule": { "darkStart": "20:00", "darkEnd": "07:00" },
            "density": "compact",
            "accent": "",
            "font": "default",
            "layout": "default",
            "ribbonCollapsed": false
        })
    }

    #[tokio::test]
    async fn appearance_round_trips_verbatim() {
        let s = store().await;
        assert_eq!(do_get_appearance(&s, "a1").await.unwrap(), None);

        do_put_appearance(&s, "a1", Some(prefs_value()), 1_700_000_000_000)
            .await
            .unwrap();

        let got = do_get_appearance(&s, "a1").await.unwrap().unwrap();
        assert_eq!(got.updated_at, 1_700_000_000_000);
        // The server is blind to the shape: what went in comes back byte-for-byte,
        // including fields it has no Rust type for.
        assert_eq!(got.prefs.as_ref(), Some(&prefs_value()));
    }

    #[tokio::test]
    async fn appearance_is_account_scoped() {
        let s = store().await;
        do_put_appearance(&s, "a1", Some(prefs_value()), 1)
            .await
            .unwrap();
        // a2 has stored nothing and cannot see a1's object — the key is derived
        // from the session account, so there is no id to present.
        assert_eq!(do_get_appearance(&s, "a2").await.unwrap(), None);
        assert_ne!(appearance_key("a1"), appearance_key("a2"));
    }

    #[tokio::test]
    async fn appearance_write_replaces_rather_than_merges() {
        let s = store().await;
        do_put_appearance(&s, "a1", Some(prefs_value()), 10)
            .await
            .unwrap();
        do_put_appearance(&s, "a1", Some(json!({ "mode": "fixed" })), 20)
            .await
            .unwrap();

        let got = do_get_appearance(&s, "a1").await.unwrap().unwrap();
        assert_eq!(got.updated_at, 20);
        // No merge: the client always sends its COMPLETE set, so the old fields
        // are gone rather than lingering under a new mode.
        assert_eq!(got.prefs, Some(json!({ "mode": "fixed" })));
    }

    #[tokio::test]
    async fn appearance_reset_stores_a_newer_null() {
        let s = store().await;
        do_put_appearance(&s, "a1", Some(prefs_value()), 10)
            .await
            .unwrap();
        do_put_appearance(&s, "a1", None, 20).await.unwrap();

        let got = do_get_appearance(&s, "a1").await.unwrap().unwrap();
        assert_eq!(got.prefs, None);
        // The reset must be NEWER than the value it clears, or a device still
        // holding the old object would win the next reconcile.
        assert_eq!(got.updated_at, 20);
    }

    #[tokio::test]
    async fn appearance_corrupt_value_reads_as_absent() {
        let s = store().await;
        s.set_setting(&appearance_key("a1"), "not json at all")
            .await
            .unwrap();
        // A junk value degrades to "nothing stored" instead of failing the GET;
        // the next write replaces it.
        assert_eq!(do_get_appearance(&s, "a1").await.unwrap(), None);

        do_put_appearance(&s, "a1", Some(prefs_value()), 5)
            .await
            .unwrap();
        assert!(do_get_appearance(&s, "a1").await.unwrap().is_some());
    }

    #[test]
    fn appearance_size_cap_measures_the_serialized_object() {
        let small = json!({ "mode": "fixed" });
        assert!(serialized_len(&small) <= MAX_APPEARANCE_BYTES);

        let fat = json!({ "junk": "x".repeat(MAX_APPEARANCE_BYTES) });
        assert!(serialized_len(&fat) > MAX_APPEARANCE_BYTES);
    }

    #[test]
    fn appearance_now_ms_is_a_plausible_wall_clock() {
        // Guards the saturating conversion: a negative or zero stamp would make
        // every stored value look older than every client's cached copy.
        assert!(now_ms() > 1_700_000_000_000);
    }
}
