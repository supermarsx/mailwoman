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
//! * The read-receipt policy (t29-e9) is one word per account on the same
//!   `settings` store, under the key the engine reads when `MDN/send` is asked
//!   to send without the user's confirmation.
//! * **Settings model C (26.20 t32-e10b, f01).** An administrator can hold
//!   `appearance.theme|accent|density`, `mail.readReceipts` and
//!   `notifications.enabled` (global, domain or per user;
//!   `crate::settings_registry`). These routes stay the user's writers of those
//!   values and apply the hold: the appearance object's held fields are replaced
//!   by the administrator's values on every write and read (one object, so the
//!   rest of it is still saved) and named in `lockedFields`; a read-receipt or
//!   notifications write that changes a held value is refused `403
//!   {code:"setting_locked"}`, and reads report the held value. The deployment
//!   default served with the appearance is the effective value of those keys for
//!   the user's domain. `/api/settings` is merged here
//!   (`crate::settings_routes::user_router`).

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

mod ui_preferences;
pub(crate) mod personal_storage;
mod receipt_preferences;
mod contact_photo;

// ─────────────────────────────────────────────────────────────────────────────
// Wire shapes (mirror apps/web/src/screens/Settings/types.ts exactly)
// ─────────────────────────────────────────────────────────────────────────────

/// A signature template (`mw-store` 0017 `signatures`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct SignatureDto {
    name: String,
    body: String,
    #[serde(default)]
    html: Option<String>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    text_edited: bool,
    is_default: bool,
    /// Optional opaque JSON auto-apply rule; omitted when empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    rule: Option<String>,
}

/// A single notification rule (match → notify/mute).
use crate::notification_policy::NotificationConfigDto;
#[cfg(test)]
use crate::notification_policy::{NotificationRuleDto, QuietHoursDto};

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

async fn personal_record_owner(state: &AppState, headers: &HeaderMap, mutation: bool) -> Result<String, Response> {
    let session = session_of(state, headers).await?;
    if headers.get("x-mailwoman-account").is_some_and(|expected| expected.to_str().ok() != Some(session.account_id.as_str())) {
        return Err((StatusCode::FORBIDDEN, Json(json!({"error":"The authenticated account changed", "code":"accountChanged"}))).into_response());
    }
    personal_storage::owner(state, &session, headers, mutation).await?;
    match state.store.session_personal_subject(&session.id).await {
        Ok(Some(owner)) => Ok(owner),
        _ => Err(StatusCode::SERVICE_UNAVAILABLE.into_response()),
    }
}

async fn signatures_list(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let account = match personal_record_owner(&state, &headers, false).await {
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
    let account = match personal_record_owner(&state, &headers, true).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    if body.name.trim().is_empty() || body.name.len() > 200 || body.name.chars().any(char::is_control) {
        return bad_request("a signature needs a valid name of at most 200 bytes");
    }
    if body.body.len() > 64 * 1024 || body.html.as_ref().is_some_and(|value| value.len() > 32 * 1024) || body.text.as_ref().is_some_and(|value| value.len() > 64 * 1024) {
        return bad_request("a signature must not exceed 32 KiB HTML or 64 KiB plain text");
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
    let account = match personal_record_owner(&state, &headers, true).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    match state.store.delete_personal_signature(&account, &name).await {
        Ok(()) => ok(),
        Err(e) => server_error("delete signature", e),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Notification rules + quiet hours (W15)
// ─────────────────────────────────────────────────────────────────────────────

async fn notifications_get(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match session_of(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    let held = match held(&state, &session, "notifications.enabled").await {
        Ok(h) => h,
        Err(r) => return r,
    };
    let subject = match crate::settings_routes::UserSubject::resolved(&state.store, &session).await
    {
        Ok(subject) => subject,
        Err(error) => return server_error("notification policy", error),
    };
    let push_allowed = match crate::notification_policy::allowed(
        &state.store,
        &subject.as_subject(),
        "notifications.pushAllowed",
    )
    .await
    {
        Ok(value) => value,
        Err(error) => return server_error("notification policy", error),
    };
    let persistent_storage_allowed = match crate::notification_policy::allowed(
        &state.store,
        &subject.as_subject(),
        "offline.persistentStorageAllowed",
    )
    .await
    {
        Ok(value) => value,
        Err(error) => return server_error("storage policy", error),
    };
    match crate::notification_policy::for_subject(&state.store, &subject.as_subject()).await {
        Ok(mut cfg) => {
            let mut locked = Vec::new();
            if let Some(v) = held.as_ref().and_then(|e| e.value.as_bool()) {
                cfg.enabled = v;
                locked.push("enabled");
            }
            let mut body = serde_json::to_value(cfg).unwrap_or_else(|_| json!({}));
            body["lockedFields"] = json!(locked);
            body["pushAllowed"] = json!(push_allowed);
            body["persistentStorageAllowed"] = json!(persistent_storage_allowed);
            Json(body).into_response()
        }
        Err(e) => server_error("get notifications", e),
    }
}

async fn notifications_put(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<NotificationConfigDto>,
) -> Response {
    let session = match session_of(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    if let Err(error) = crate::notification_policy::validate(&body) {
        return bad_request(error);
    }
    let account = session.account_id.clone();
    match held(&state, &session, "notifications.enabled").await {
        Ok(Some(e)) if e.value.as_bool() != Some(body.enabled) => {
            return setting_locked("notifications.enabled");
        }
        Ok(_) => {}
        Err(r) => return r,
    }
    match do_put_notifications(&state.store, &account, &body).await {
        Ok(()) => ok(),
        Err(e) => server_error("put notifications", e),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Saved searches → search folders (W13, frozen 0003)
// ─────────────────────────────────────────────────────────────────────────────

async fn saved_searches_list(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let account = match personal_record_owner(&state, &headers, false).await {
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
    let account = match personal_record_owner(&state, &headers, true).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    if body.name.trim().is_empty()
        || body.name.chars().count() > 256
        || body.name.chars().any(char::is_control)
    {
        return bad_request("a saved search needs a name of at most 256 characters");
    }
    if let Err(error) = mw_engine::EmailFilter::parse_saved_json(&body.query_json) {
        return bad_request(error);
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
    let account = match personal_record_owner(&state, &headers, true).await {
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
    let session = match session_of(&state, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    if body.name.trim().is_empty() || body.email.trim().is_empty() {
        return bad_request("an identity needs a name and an email");
    }
    let account = &session.account_id;
    match identity_address_allowed(&state, &session, &body.email).await {
        Ok(true) => {},
        Ok(false) => return (StatusCode::FORBIDDEN, Json(json!({"error":"This address is not authorized for sending. Choose your account address or an administrator-approved alias.", "code":"forbiddenFrom"}))).into_response(),
        Err(()) => return (StatusCode::BAD_GATEWAY, Json(json!({"error":"Sending identity authority could not be verified. Try again.", "code":"identityAuthorityUnavailable"}))).into_response(),
    }
    match do_upsert_identity(&state.store, account, &body).await {
        Ok(Some(id)) => Json(json!({ "ok": true, "id": id })).into_response(),
        Ok(None) => not_found("no such identity"),
        Err(StoreError::NotFound) => not_found("no such identity"),
        Err(e) => server_error("upsert identity", e),
    }
}

/// Editable profile rows are not permission grants. Proxy identities come from
/// the current authenticated provider; native identities use runtime authority.
async fn identity_address_allowed(state: &AppState, session: &mw_store::Session, address: &str) -> Result<bool, ()> {
    if address.is_empty() || address.trim() != address || address.chars().any(char::is_control) { return Ok(false); }
    if let Some(engine) = &state.engine {
        crate::engine_mode::ensure_account(engine, &session.account_id).await.map_err(|_| ())?;
        return engine.sender_address_allowed(&session.account_id, address).await.map_err(|_| ());
    }
    let client = crate::upstream_client(&state.security, &session.jmap_url, &session.api_url, &session.credentials).await.map_err(|_| ())?;
    let request = json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:submission"],"methodCalls":[["Identity/get",{"accountId":session.account_id,"properties":["id","email"]},"identities"]]});
    let (status, bytes) = tokio::time::timeout(std::time::Duration::from_secs(10), client.request_raw_limited(&session.api_url, axum::body::Bytes::from(request.to_string()), 256*1024)).await.map_err(|_| ())?.map_err(|_| ())?;
    if !status.is_success() { return Err(()); }
    let response: serde_json::Value = serde_json::from_slice(&bytes).map_err(|_| ())?;
    let value = response["methodResponses"].as_array().and_then(|calls|calls.iter().find(|call|call[0]=="Identity/get" && call[2]=="identities")).ok_or(())?;
    if value[1]["accountId"].as_str()!=Some(session.account_id.as_str()) { return Err(()); }
    let identities = value[1]["list"].as_array().ok_or(())?;
    if identities.len()>1024 { return Err(()); }
    Ok(identities.iter().any(|identity|identity["email"].as_str().is_some_and(|email|email.eq_ignore_ascii_case(address))))
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
    personal_storage::key("appearance", account)
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
    /// The administrator's appearance for this user's domain: the effective
    /// values of `appearance.theme`, `appearance.brandName` and
    /// `appearance.accent` without the user's own choice.
    deployment_default: DeploymentAppearanceDto,
    /// Fields of `appearance` an administrator holds; their values above are the
    /// administrator's, and a write cannot change them.
    locked_fields: Vec<&'static str>,
}

/// The deployment-wide appearance default (`mw_admin::Appearance`), in the same
/// camelCase shape the admin panel already receives from `admin.rs`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
struct DeploymentAppearanceDto {
    theme: String,
    font: String,
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
    let session = match session_of(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    let owner = match personal_storage::owner(&state, &session, &headers, false).await { Ok(owner) => owner, Err(response) => return response };
    let locks = match appearance_locks(&state, &session).await {
        Ok(l) => l,
        Err(r) => return r,
    };
    let deployment_default = match deployment_default(&state, &session).await {
        Ok(d) => d,
        Err(e) => return server_error("get appearance default", e),
    };
    match do_get_appearance(&state.store, &owner).await {
        Ok(stored) => {
            let mut appearance = stored.as_ref().and_then(|e| e.prefs.clone());
            let locked_fields = match appearance.as_mut() {
                Some(prefs) => apply_appearance_locks(prefs, &locks),
                None => locked_field_names(&locks),
            };
            Json(AppearanceResponse {
                appearance,
                updated_at: stored.as_ref().map(|e| e.updated_at),
                deployment_default,
                locked_fields,
            })
            .into_response()
        }
        Err(e) => server_error("get appearance", e),
    }
}

async fn appearance_put(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<AppearanceRequest>,
) -> Response {
    let session = match session_of(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    let mut prefs = body.appearance;
    if !prefs.is_object() {
        return bad_request("appearance preferences must be a JSON object");
    }
    if serialized_len(&prefs) > MAX_APPEARANCE_BYTES {
        return payload_too_large("appearance preferences are too large");
    }
    let owner = match personal_storage::owner(&state, &session, &headers, true).await { Ok(owner) => owner, Err(response) => return response };
    let locks = match appearance_locks(&state, &session).await {
        Ok(l) => l,
        Err(r) => return r,
    };
    // Held fields are replaced, not refused: the object is one value and the
    // rest of it is the user's to save (t32-admin §2.4).
    let locked_fields = apply_appearance_locks(&mut prefs, &locks);
    match do_put_appearance(&state.store, &owner, Some(prefs), now_ms()).await {
        Ok(updated_at) => Json(json!({
            "ok": true,
            "updatedAt": updated_at,
            "lockedFields": locked_fields,
        }))
        .into_response(),
        Err(e) => server_error("put appearance", e),
    }
}

/// `DELETE /api/account/appearance` — forget this account's appearance, so the
/// client falls back to the deployment default. Stores a `prefs: null` envelope
/// rather than removing the key (see the section note on `updatedAt`).
async fn appearance_delete(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match session_of(&state, &headers).await {
        Ok(a) => a,
        Err(r) => return r,
    };
    let account = match personal_storage::owner(&state, &session, &headers, true).await { Ok(owner) => owner, Err(response) => return response };
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
// Read-receipt policy (t29-e9)
//
// `ask` (the default), `never` or `always`: what the account wants done when a
// message it opens asks for a read receipt. The reader is the engine
// (`mw_engine::Engine::mdn_policy`, consulted by `MDN/send`): only `always`
// lets a receipt go without the user confirming it, and then only for a
// request that names the message's own return path and is not list mail.
// `never` and `ask` differ for the client alone (whether it shows the request
// as a prompt); neither stops a receipt the user asks for by hand.
//
// The key and the accepted words are the engine's (`Engine::mdn_policy_key`,
// `Engine::MDN_POLICIES`), so the route cannot store a word the reader does not
// know. A proxy-mode server has no engine and no `MDN/send` of this kind:
// there the route answers 404 rather than store a preference nothing reads.
// ─────────────────────────────────────────────────────────────────────────────

/// `GET`/`PUT /api/account/mdn-policy` body.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct MdnPolicyDto {
    policy: String,
}

/// The 404 a server without an engine answers the policy routes with.
fn no_mdn_policy() -> Response {
    not_found("read receipts are not available: this server relays another JMAP server")
}

async fn mdn_policy_get(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match session_of(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    let Some(engine) = &state.engine else {
        return no_mdn_policy();
    };
    let held = match held(&state, &session, "mail.readReceipts").await {
        Ok(h) => h,
        Err(r) => return r,
    };
    if let Some(policy) = held.as_ref().and_then(|e| e.value.as_str()) {
        return Json(json!({ "policy": policy, "locked": true, "allowedModes": mw_engine::Engine::MDN_POLICIES })).into_response();
    }
    match engine.mdn_policy(&session.account_id).await {
        Ok(policy) => Json(json!({ "policy": policy, "locked": false, "allowedModes": mw_engine::Engine::MDN_POLICIES })).into_response(),
        Err(e) => {
            tracing::error!("prefs: get mdn policy failed: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "server error").into_response()
        }
    }
}

async fn mdn_policy_put(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<MdnPolicyDto>,
) -> Response {
    let session = match session_of(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    let account = session.account_id.clone();
    if state.engine.is_none() {
        return no_mdn_policy();
    }
    if !mw_engine::Engine::MDN_POLICIES.contains(&body.policy.as_str()) {
        return bad_request("policy must be one of: ask, never, always, contactsOnly, orgOnly, orgAlwaysAskOthers");
    }
    match held(&state, &session, "mail.readReceipts").await {
        Ok(Some(e)) if e.value.as_str() != Some(body.policy.as_str()) => {
            return setting_locked("mail.readReceipts");
        }
        Ok(_) => {}
        Err(r) => return r,
    }
    match state
        .store
        .set_setting(&mw_engine::Engine::mdn_policy_key(&account), &body.policy)
        .await
    {
        Ok(()) => Json(json!({ "ok": true, "policy": body.policy })).into_response(),
        Err(e) => server_error("put mdn policy", e),
    }
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
        .list_personal_signatures(account)
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
        .upsert_personal_signature(&signature_from_dto(account, dto))
        .await
}

#[cfg(test)]
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
        .list_personal_saved_searches(account)
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
        if let Some(existing) = store.get_personal_saved_search(&dto.id).await?
            && existing.user != account
        {
            return Ok(None);
        }
        dto.id.clone()
    };
    let written = store
        .upsert_personal_saved_search(&SavedSearchRow {
            id: id.clone(),
            user: account.to_string(),
            name: dto.name.clone(),
            query_json: dto.query_json.clone(),
            as_folder: dto.as_folder,
        })
        .await?;
    Ok(written.then_some(id))
}

/// Delete a saved search iff it belongs to `account`. `Ok(false)` = absent or owned
/// by another account (indistinguishable to the caller).
async fn do_delete_saved_search(
    store: &Store,
    account: &str,
    id: &str,
) -> Result<bool, StoreError> {
    store.delete_personal_saved_search(account, id).await
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
/// persists in the 0020 column. Existing provider signature bodies and Sent folder
/// metadata are preserved; profile updates are not authority grants.
async fn do_upsert_identity(
    store: &Store,
    account: &str,
    dto: &IdentityDto,
) -> Result<Option<String>, StoreError> {
    let existing = if dto.id.trim().is_empty() { None } else { store.get_identity(&dto.id).await? };
    if existing.as_ref().is_some_and(|row| row.account_id != account) { return Ok(None); }
    let id = if dto.id.trim().is_empty() {
        new_id()
    } else {
        dto.id.clone()
    };
    store
        .upsert_identity(&IdentityRow {
            id: id.clone(),
            account_id: account.to_string(),
            name: dto.name.clone(),
            email: dto.email.clone(),
            reply_to: dto.reply_to.clone(),
            signature_html: existing.as_ref().and_then(|row| row.signature_html.clone()),
            signature_text: existing.as_ref().and_then(|row| row.signature_text.clone()),
            signature_name: dto.signature_name.clone(),
            sent_mailbox_id: existing.as_ref().and_then(|row| row.sent_mailbox_id.clone()),
            source: "configured".to_string(),
        })
        .await?;
    Ok(Some(id))
}

/// Delete an identity iff it belongs to `account`. `Ok(false)` = absent or foreign.
async fn do_delete_identity(store: &Store, account: &str, id: &str) -> Result<bool, StoreError> {
    store.delete_identity(account, id).await
}

// ─────────────────────────────────────────────────────────────────────────────
// Row ⇄ DTO mapping
// ─────────────────────────────────────────────────────────────────────────────

fn signature_to_dto(row: &SignatureRow) -> SignatureDto {
    SignatureDto {
        html: row.html.clone(), text: row.text.clone(), text_edited: row.text_edited,
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
        html: dto.html.as_deref().map(mw_sanitize::sanitize_email_html),
        text: dto.text.clone(), text_edited: dto.text_edited,
        account_id: account.to_string(),
        name: dto.name.clone(),
        body: dto.body.clone(),
        is_default: dto.is_default,
        rule_json: dto.rule.clone().unwrap_or_default(),
        updated_at: String::new(),
    }
}

#[cfg(test)]
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
            "/api/account/preferences",
            get(ui_preferences::load).put(ui_preferences::save),
        )
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
        // t29-e9: the read-receipt policy `MDN/send` reads (engine mode only).
        .route(
            "/api/account/mdn-policy",
            get(mdn_policy_get).put(mdn_policy_put),
        )
        .merge(receipt_preferences::router())
        .merge(contact_photo::router())
        // t32-e10b (f01): the settings model C, user side.
        .merge(crate::settings_routes::user_router())
}

/// The authenticated caller's account id, or an early auth `Response` to return.
async fn account_id(state: &AppState, headers: &HeaderMap) -> Result<String, Response> {
    crate::authed(state, headers).await.map(|s| s.account_id)
}

/// The authenticated caller's session, or an early auth `Response` to return.
async fn session_of(state: &AppState, headers: &HeaderMap) -> Result<mw_store::Session, Response> {
    crate::authed(state, headers).await
}

/// `403 {code:"setting_locked"}`: an administrator holds `key` for this user.
fn setting_locked(key: &str) -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({
            "code": "setting_locked",
            "error": format!("{key} is set by your administrator"),
            "setting": key,
        })),
    )
        .into_response()
}

/// The effective value of `key` when an administrator holds it for this user,
/// `None` when the user may choose.
async fn held(
    state: &AppState,
    session: &mw_store::Session,
    key: &str,
) -> Result<Option<crate::settings_registry::Effective>, Response> {
    match crate::settings_routes::effective_for_session(state, session, key).await {
        Ok(Some(e)) if e.locked() => Ok(Some(e)),
        Ok(_) => Ok(None),
        Err(e) => Err(server_error("read setting", e)),
    }
}

/// The appearance keys and the fields of the appearance object each one holds.
const APPEARANCE_FIELDS: [(&str, &[&str]); 4] = [
    (
        "appearance.theme",
        &["mode", "theme", "lightTheme", "darkTheme"],
    ),
    ("appearance.accent", &["accentId", "customAccent"]),
    ("appearance.density", &["density"]),
    ("appearance.font", &["font"]),
];

/// The held appearance keys and their values, for this user.
pub(crate) async fn appearance_locks(
    state: &AppState,
    session: &mw_store::Session,
) -> Result<Vec<(&'static str, serde_json::Value)>, Response> {
    let mut out = Vec::new();
    for (key, _) in APPEARANCE_FIELDS {
        if let Some(e) = held(state, session, key).await? {
            out.push((key, e.value));
        }
    }
    Ok(out)
}

fn locked_field_names(locks: &[(&'static str, serde_json::Value)]) -> Vec<&'static str> {
    APPEARANCE_FIELDS
        .iter()
        .filter(|(key, _)| locks.iter().any(|(k, _)| k == key))
        .flat_map(|(_, fields)| fields.iter().copied())
        .collect()
}

/// Replace the held fields of an appearance object with the administrator's
/// values and return their names. A held theme fixes the mode and both pair
/// members to that theme, so no `system`/`schedule` switch can show another one.
fn apply_appearance_locks(
    prefs: &mut serde_json::Value,
    locks: &[(&'static str, serde_json::Value)],
) -> Vec<&'static str> {
    let Some(obj) = prefs.as_object_mut() else {
        return Vec::new();
    };
    for (key, value) in locks {
        match *key {
            "appearance.theme" => {
                obj.insert("mode".into(), json!("fixed"));
                obj.insert("theme".into(), value.clone());
                obj.insert("lightTheme".into(), value.clone());
                obj.insert("darkTheme".into(), value.clone());
                obj.insert("chosen".into(), json!(true));
            }
            "appearance.accent" => {
                obj.insert("accentId".into(), value.clone());
                obj.insert("customAccent".into(), json!(""));
            }
            "appearance.density" => {
                obj.insert("density".into(), value.clone());
            }
            "appearance.font" => {
                obj.insert("font".into(), value.clone());
            }
            _ => {}
        }
    }
    locked_field_names(locks)
}

/// The administrator's appearance for this user: the effective values of the
/// three keys from the settings tables alone (the user's own object is not one
/// of them), falling back to the built-in defaults.
async fn deployment_default(
    state: &AppState,
    session: &mw_store::Session,
) -> Result<DeploymentAppearanceDto, StoreError> {
    let subject = crate::settings_routes::UserSubject::of(session);
    let value = |e: Option<crate::settings_registry::Effective>| {
        e.and_then(|e| e.value.as_str().map(str::to_string))
    };
    let store = &state.store;
    let s = subject.as_subject();
    let theme =
        value(crate::settings_registry::effective_for(store, "appearance.theme", &s).await?);
    let brand =
        value(crate::settings_registry::effective_for(store, "appearance.brandName", &s).await?);
    let accent =
        value(crate::settings_registry::effective_for(store, "appearance.accent", &s).await?);
    let fallback = mw_admin::Appearance::default();
    let font = value(crate::settings_registry::effective_for(store, "appearance.font", &s).await?);
    Ok(DeploymentAppearanceDto {
        theme: theme.unwrap_or(fallback.theme),
        font: font.unwrap_or_else(|| "default".into()),
        brand_name: brand.unwrap_or(fallback.brand_name),
        accent: accent.filter(|a| !a.is_empty()),
    })
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
            &SignatureDto { html: None, text: None, text_edited: false,
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
            &SignatureDto { html: None, text: None, text_edited: false,
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

        s.delete_personal_signature("a1", "personal").await.unwrap();
        assert_eq!(do_list_signatures(&s, "a1").await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn signatures_are_account_scoped() {
        let s = store().await;
        do_upsert_signature(
            &s,
            "a1",
            &SignatureDto { html: None, text: None, text_edited: false,
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
                timezone: None,
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
    async fn signature_formats_sanitize_html_preserve_plain_and_scope() {
        let store=store().await;
        let dto=SignatureDto{name:"work".into(),body:"Legacy plain".into(),is_default:false,rule:None,
            html:Some("<p onclick=evil()>Hello <strong>team</strong><script>evil()</script></p>".into()),text:Some("Independent <plain> & team".into()),text_edited:true};
        do_upsert_signature(&store,"owner",&dto).await.unwrap();
        assert!(do_list_signatures(&store,"other").await.unwrap().is_empty());
        let saved=do_list_signatures(&store,"owner").await.unwrap().remove(0);
        assert_eq!(saved.text.as_deref(),Some("Independent <plain> & team"));assert!(saved.text_edited);
        assert_eq!(saved.html.as_deref(),Some("<p>Hello <strong>team</strong></p>"));
    }

    #[tokio::test]
    async fn identity_profile_edit_preserves_provider_signature_bodies() {
        let s=store().await;
        let account=seed_account(&s,"primary@example.org").await;
        s.upsert_identity(&IdentityRow{id:"provider-profile".into(),account_id:account.clone(),name:"Before".into(),email:"primary@example.org".into(),reply_to:None,signature_html:Some("<p>Provider</p>".into()),signature_text:Some("Provider text".into()),signature_name:None,sent_mailbox_id:Some("sent".into()),source:"server".into()}).await.unwrap();
        do_upsert_identity(&s,&account,&IdentityDto{id:"provider-profile".into(),name:"After".into(),email:"primary@example.org".into(),reply_to:None,signature_name:Some("local-template".into())}).await.unwrap();
        let row=s.get_identity("provider-profile").await.unwrap().unwrap();
        assert_eq!(row.signature_html.as_deref(),Some("<p>Provider</p>"));
        assert_eq!(row.signature_text.as_deref(),Some("Provider text"));
        assert_eq!(row.sent_mailbox_id.as_deref(),Some("sent"));
        assert_eq!(row.signature_name.as_deref(),Some("local-template"));
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
