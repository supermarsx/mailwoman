//! Masked-email routes (t10 plan §3 e7, SPEC §28.4). The user-scoped alias service
//! over the 0010 `masked_email` table (via the e0 `mw-store` repo). Generate a stable,
//! collision-checked masked alias bound to a forwarding target, then drive its
//! lifecycle (`enabled` → `disabled` → `deleted`).
//!
//! The alias LIFECYCLE lives here (server-side); the `masked-email` wasm component only
//! performs the on-send envelope rewrite in the jail. The shape mirrors the JMAP
//! `MaskedEmail` object (id / email / state / forDomain / description / createdAt /
//! lastUsedAt) so the SPA composer can surface it directly.
//!
//! ## Routes (frozen shape from the e0 stub; MOUNTED by e13 into `router()`)
//! All are **mailbox-session-authed** (the same opaque `mw_session` cookie / native
//! bearer the JMAP surface uses, via [`crate::authed`]) and **per-user scoped** — a
//! session only ever sees or mutates aliases whose `account_id` is its own; a
//! cross-account id is a `404` (never leaks another account's alias existence).
//!   * `GET    /api/masked`            — list the session account's (non-deleted) aliases.
//!   * `POST   /api/masked`            — generate a new alias (optional target/description).
//!   * `POST   /api/masked/{id}/state` — enable/disable an alias.
//!   * `DELETE /api/masked/{id}`       — delete an alias (soft-delete → `deleted` state).
//!
//! ## Alias generation (deterministic + collision-checked)
//! An alias local-part is the leading hex of `SHA-256(seed || ":" || counter)`. The
//! `seed` is a fresh random per request (so aliases are unguessable), but the
//! collision-avoidance LOOP is fully deterministic: given a seed + domain + the set of
//! addresses already in use, it always yields the same address, incrementing the
//! counter past any collision. That keeps generation testable (no `Math.random`-style
//! nondeterminism) while global uniqueness is backstopped by the `alias_addr` UNIQUE
//! index in 0010.
#![allow(dead_code)]

use std::sync::Arc;

use async_trait::async_trait;
use axum::Router;
use axum::extract::{Path as UrlPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use mw_engine::account::MailSubmitter;
use mw_engine::backend::{EngineError, Result as EngineResult};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{MaskedEmailRow, Store, StoreError};

use crate::AppState;

/// Lifecycle states (mirror the 0010 `masked_email.state` domain + JMAP `MaskedEmail`).
const STATE_ENABLED: &str = "enabled";
const STATE_DISABLED: &str = "disabled";
const STATE_DELETED: &str = "deleted";

/// Alias local-part length in hex chars (48 bits of the digest — collision-negligible,
/// still short enough to be a readable address).
const TOKEN_LEN: usize = 12;

/// e13 merges this into `router()` once mounted. `router()` is byte-unchanged today.
pub(crate) fn masked_router() -> Router<AppState> {
    Router::new()
        .route("/api/masked", get(list).post(generate))
        .route("/api/masked/{id}/state", post(set_state))
        .route("/api/masked/{id}", axum::routing::delete(delete_alias))
}

// ── request/response models ─────────────────────────────────────────────────────

/// `POST /api/masked` body: all optional. `target` defaults to the session account's
/// own address (the alias forwards to the real mailbox); `forDomain`/`description` are
/// free-form metadata (the originating site + a human note), JMAP `MaskedEmail`-style.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateReq {
    #[serde(default)]
    target: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    for_domain: Option<String>,
}

/// `POST /api/masked/{id}/state` body: the requested lifecycle state (`enabled` or
/// `disabled`; deletion is the `DELETE` verb, not a state write).
#[derive(Debug, Deserialize)]
struct StateReq {
    state: String,
}

/// The structured metadata packed into the single `masked_email.target_desc` column
/// (kept opaque by the repo). `target` is the forwarding address; the rest is
/// JMAP-`MaskedEmail`-style annotation.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AliasMeta {
    #[serde(default)]
    target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    for_domain: Option<String>,
}

/// The outcome of a scoped mutation: `NotFound` covers both "no such id" and
/// "belongs to another account" (uniform, non-enumerable).
enum Outcome {
    NotFound,
    Ok(MaskedEmailRow),
}

// ── alias generation (pure, deterministic, collision-checked) ────────────────────

/// The alias local-part for `(seed, counter)`: the leading [`TOKEN_LEN`] hex chars of
/// `SHA-256(seed || ":" || counter)`.
fn alias_token(seed: &str, counter: u32) -> String {
    let mut h = Sha256::new();
    h.update(seed.as_bytes());
    h.update(b":");
    h.update(counter.to_be_bytes());
    let digest = h.finalize();
    let mut s = String::with_capacity(TOKEN_LEN);
    for b in digest.iter().take(TOKEN_LEN / 2) {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// A masked alias address `<token>@<domain>` that does not collide with any address in
/// `existing`. Deterministic for a fixed `seed`/`domain`/`existing`: it increments the
/// counter past any collision and returns the first free candidate.
fn generate_alias_addr(seed: &str, domain: &str, existing: &[String]) -> String {
    let mut counter = 0u32;
    loop {
        let addr = format!("{}@{}", alias_token(seed, counter), domain);
        if !existing.iter().any(|e| e.eq_ignore_ascii_case(&addr)) {
            return addr;
        }
        counter = counter.wrapping_add(1);
    }
}

/// The masking domain for a session: `MW_MASKED_EMAIL_DOMAIN` when configured, else the
/// account's own mail domain, else a safe fallback.
fn masked_domain(username: &str) -> String {
    if let Ok(d) = std::env::var("MW_MASKED_EMAIL_DOMAIN") {
        let d = d.trim();
        if !d.is_empty() {
            return d.to_ascii_lowercase();
        }
    }
    username
        .rsplit_once('@')
        .map(|(_, d)| d.to_ascii_lowercase())
        .filter(|d| !d.is_empty())
        .unwrap_or_else(|| "masked.local".to_string())
}

/// Decode the `target_desc` column. Falls back to treating a non-JSON value as a bare
/// forwarding target (robust against externally-written rows).
fn parse_meta(s: &str) -> AliasMeta {
    serde_json::from_str::<AliasMeta>(s).unwrap_or_else(|_| AliasMeta {
        target: s.to_string(),
        description: None,
        for_domain: None,
    })
}

/// Render a row as the JMAP-`MaskedEmail`-style JSON the SPA consumes.
fn alias_json(row: &MaskedEmailRow) -> Value {
    let meta = parse_meta(&row.target_desc);
    json!({
        "id": row.id,
        "email": row.alias_addr,
        "state": row.state,
        "target": meta.target,
        "description": meta.description,
        "forDomain": meta.for_domain,
        "createdAt": row.created_at,
        "lastUsedAt": row.last_used_at,
    })
}

// ── service layer (takes `&Store`; unit-tested) ─────────────────────────────────

/// Generate + persist a new alias for `account_id` (default forwarding target =
/// `username`). Enabled on creation.
async fn create_alias(
    store: &Store,
    account_id: &str,
    username: &str,
    req: &CreateReq,
) -> Result<MaskedEmailRow, StoreError> {
    let existing: Vec<String> = store
        .list_masked_email(account_id)
        .await?
        .into_iter()
        .map(|r| r.alias_addr)
        .collect();
    let domain = masked_domain(username);
    // Fresh random seed ⇒ the alias is unguessable; the collision loop stays deterministic.
    let seed = uuid::Uuid::new_v4().simple().to_string();
    let alias = generate_alias_addr(&seed, &domain, &existing);

    let target = req
        .target
        .clone()
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| username.to_string());
    let meta = AliasMeta {
        target,
        description: req.description.clone().filter(|d| !d.is_empty()),
        for_domain: req.for_domain.clone().filter(|d| !d.is_empty()),
    };
    let row = MaskedEmailRow {
        id: uuid::Uuid::new_v4().to_string(),
        account_id: account_id.to_string(),
        alias_addr: alias,
        target_desc: serde_json::to_string(&meta).unwrap_or_default(),
        state: STATE_ENABLED.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        last_used_at: None,
    };
    store.put_masked_email(&row).await?;
    Ok(row)
}

/// Every non-deleted alias for `account_id`, newest-first (the repo orders it).
async fn list_active(store: &Store, account_id: &str) -> Result<Vec<MaskedEmailRow>, StoreError> {
    Ok(store
        .list_masked_email(account_id)
        .await?
        .into_iter()
        .filter(|r| r.state != STATE_DELETED)
        .collect())
}

/// Fetch an alias only if it belongs to `account_id` (per-user scoping). `None` for
/// unknown OR another account's alias.
async fn owned_alias(
    store: &Store,
    account_id: &str,
    id: &str,
) -> Result<Option<MaskedEmailRow>, StoreError> {
    Ok(store
        .get_masked_email(id)
        .await?
        .filter(|r| r.account_id == account_id))
}

/// Transition an owned alias to `new_state` (`enabled`/`disabled`). Scoped: another
/// account's id is `NotFound`.
async fn change_state(
    store: &Store,
    account_id: &str,
    id: &str,
    new_state: &str,
) -> Result<Outcome, StoreError> {
    if owned_alias(store, account_id, id).await?.is_none() {
        return Ok(Outcome::NotFound);
    }
    store.set_masked_email_state(id, new_state).await?;
    Ok(store
        .get_masked_email(id)
        .await?
        .map_or(Outcome::NotFound, Outcome::Ok))
}

/// Soft-delete an owned alias (→ `deleted` state; the row is a tombstone so the alias is
/// never re-minted / re-activated). Scoped. Returns `false` for unknown/other-account.
async fn soft_delete(store: &Store, account_id: &str, id: &str) -> Result<bool, StoreError> {
    if owned_alias(store, account_id, id).await?.is_none() {
        return Ok(false);
    }
    store.set_masked_email_state(id, STATE_DELETED).await?;
    Ok(true)
}

// ── on-send From-rewrite (26.10 follow-up a) ──────────────────────────────────────
//
// When a user submits a message whose envelope `From` is one of THEIR masked
// aliases, the alias must be what the recipient sees (the real address stays
// hidden in the Return-Path), and an alias they may NOT send as must be refused
// fail-closed. The store-layer alias service (above) already holds the mapping;
// this is the send-time seam the 26.10 mount deferred (there was "no host plumbing
// to feed the per-send alias→target map"). It lives server-side as a
// [`MailSubmitter`] decorator — `mw-server` builds the per-account submitter, so it
// can wrap it with masked-alias enforcement without any change to `mw-engine`'s
// frozen submission path. A non-alias `From` is passed through byte-unchanged.

/// The outcome of resolving a candidate envelope `From` against the masked-alias
/// store, for one sending account.
enum FromDecision {
    /// Not a masked alias (of any account) — send the envelope byte-unchanged.
    Pass,
    /// An `enabled` alias OWNED BY the sending account — present it as `From`.
    Allow(MaskedEmailRow),
    /// Fail-closed: a masked alias this account must not send as — another
    /// account's alias, or an owned-but-disabled/deleted one.
    Reject(&'static str),
}

/// Resolve a candidate envelope `From` for `account_id` (global, per-address).
/// Security model (fail-closed):
///   * unknown address ⇒ [`FromDecision::Pass`] (an ordinary sender, unchanged);
///   * a masked alias owned by ANOTHER account ⇒ [`FromDecision::Reject`]
///     (never let account A send as account B's alias);
///   * an owned alias that is not `enabled` (disabled/deleted) ⇒
///     [`FromDecision::Reject`] (a disabled/deleted alias is not usable as `From`);
///   * an owned + `enabled` alias ⇒ [`FromDecision::Allow`].
async fn resolve_from(
    store: &Store,
    account_id: &str,
    from_addr: &str,
) -> Result<FromDecision, StoreError> {
    let addr = from_addr.trim();
    if addr.is_empty() {
        return Ok(FromDecision::Pass);
    }
    match store.get_masked_email_by_addr(addr).await? {
        None => Ok(FromDecision::Pass),
        Some(row) if row.account_id != account_id => Ok(FromDecision::Reject(
            "From is another account's masked alias",
        )),
        Some(row) if row.state != STATE_ENABLED => Ok(FromDecision::Reject(
            "From is a disabled or deleted masked alias",
        )),
        Some(row) => Ok(FromDecision::Allow(row)),
    }
}

/// A [`MailSubmitter`] decorator that enforces masked-email `From` semantics at
/// send time. `mw-server` wraps each standards-account submitter with this (see
/// `engine_mode::register`); a submission whose envelope `From` is a masked alias
/// is validated against the owning account before it reaches the wire.
pub(crate) struct MaskedSubmitter {
    store: Store,
    account_id: String,
    inner: Arc<dyn MailSubmitter>,
}

impl MaskedSubmitter {
    /// Wrap `inner` with masked-alias `From` enforcement for `account_id`.
    pub(crate) fn new(store: Store, account_id: String, inner: Arc<dyn MailSubmitter>) -> Self {
        Self {
            store,
            account_id,
            inner,
        }
    }
}

#[async_trait]
impl MailSubmitter for MaskedSubmitter {
    async fn submit(&self, mut msg: Outgoing) -> EngineResult<SubmissionResult> {
        match resolve_from(&self.store, &self.account_id, &msg.mail_from).await {
            // Ordinary sender: envelope untouched, forward as-is.
            Ok(FromDecision::Pass) => {}
            // Owned + enabled alias: present the canonical alias as the envelope
            // sender (so bounces/replies route through the masked domain to the
            // alias's forwarding target, keeping the real address out of the
            // Return-Path) and record best-effort usage. The message `From:` header
            // the recipient sees is already the alias the composer set.
            Ok(FromDecision::Allow(row)) => {
                msg.mail_from = row.alias_addr.clone();
                let now = chrono::Utc::now().to_rfc3339();
                let _ = self.store.touch_masked_email(&row.id, &now).await;
            }
            // Fail-closed: never send as an alias this account may not use.
            Ok(FromDecision::Reject(why)) => {
                return Err(EngineError::Protocol(format!(
                    "masked-email: refusing to send as '{}' ({why})",
                    msg.mail_from
                )));
            }
            // A store error must not fall through to an unvalidated masked send;
            // surface it as a (retryable) protocol error instead.
            Err(e) => {
                return Err(EngineError::Protocol(format!(
                    "masked-email: alias resolution failed: {e}"
                )));
            }
        }
        self.inner.submit(msg).await
    }
}

// ── handlers ─────────────────────────────────────────────────────────────────────

/// `GET /api/masked` — the session account's non-deleted aliases.
async fn list(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    match list_active(&state.store, &session.account_id).await {
        Ok(rows) => {
            let aliases: Vec<Value> = rows.iter().map(alias_json).collect();
            axum::Json(json!({ "aliases": aliases })).into_response()
        }
        Err(e) => store_error(&e),
    }
}

/// `POST /api/masked` — generate a new alias. Accepts an optional JSON body (an empty
/// body / `{}` is valid: the alias forwards to the account's own address).
async fn generate(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Option<axum::Json<CreateReq>>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let req = body.map(|axum::Json(b)| b).unwrap_or_default();
    match create_alias(&state.store, &session.account_id, &session.username, &req).await {
        Ok(row) => (StatusCode::CREATED, axum::Json(alias_json(&row))).into_response(),
        Err(e) => store_error(&e),
    }
}

/// `POST /api/masked/{id}/state` — enable/disable an alias (scoped to the session).
async fn set_state(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
    axum::Json(body): axum::Json<StateReq>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    let requested = body.state.as_str();
    if requested != STATE_ENABLED && requested != STATE_DISABLED {
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(json!({ "error": "state must be 'enabled' or 'disabled'" })),
        )
            .into_response();
    }
    match change_state(&state.store, &session.account_id, &id, requested).await {
        Ok(Outcome::Ok(row)) => axum::Json(alias_json(&row)).into_response(),
        Ok(Outcome::NotFound) => not_found(&id),
        Err(e) => store_error(&e),
    }
}

/// `DELETE /api/masked/{id}` — soft-delete an alias (scoped to the session).
async fn delete_alias(
    State(state): State<AppState>,
    headers: HeaderMap,
    UrlPath(id): UrlPath<String>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    match soft_delete(&state.store, &session.account_id, &id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => not_found(&id),
        Err(e) => store_error(&e),
    }
}

// ── response helpers ─────────────────────────────────────────────────────────────

fn not_found(id: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        axum::Json(json!({ "error": format!("unknown masked alias '{id}'") })),
    )
        .into_response()
}

fn store_error(e: &StoreError) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        axum::Json(json!({ "error": format!("masked-email store error: {e}") })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mw_store::ServerKey;

    async fn store() -> Store {
        Store::open_in_memory(ServerKey::generate()).await.unwrap()
    }

    // ── alias generation: deterministic + collision-checked ──────────────────────

    #[test]
    fn generation_is_deterministic_and_skips_collisions() {
        let first = generate_alias_addr("seed-1", "masked.test", &[]);
        // Deterministic: same inputs ⇒ same address.
        assert_eq!(first, generate_alias_addr("seed-1", "masked.test", &[]));
        assert!(first.ends_with("@masked.test"));
        assert_eq!(first.split('@').next().unwrap().len(), TOKEN_LEN);

        // Colliding on the first candidate ⇒ a distinct, non-colliding address.
        let second = generate_alias_addr("seed-1", "masked.test", std::slice::from_ref(&first));
        assert_ne!(second, first);
        assert!(!second.is_empty());

        // Different seeds ⇒ different aliases (unguessability).
        assert_ne!(first, generate_alias_addr("seed-2", "masked.test", &[]));
    }

    #[test]
    fn masked_domain_prefers_env_then_account_domain() {
        // Not asserting the env branch (process-global); the account-domain fallback:
        unsafe {
            std::env::remove_var("MW_MASKED_EMAIL_DOMAIN");
        }
        assert_eq!(masked_domain("Alice@Corp.Example"), "corp.example");
        assert_eq!(masked_domain("no-domain"), "masked.local");
    }

    #[test]
    fn meta_round_trips_and_tolerates_plain_text() {
        let meta = AliasMeta {
            target: "real@corp.example".into(),
            description: Some("shopping".into()),
            for_domain: Some("shop.example".into()),
        };
        let encoded = serde_json::to_string(&meta).unwrap();
        let back = parse_meta(&encoded);
        assert_eq!(back.target, "real@corp.example");
        assert_eq!(back.description.as_deref(), Some("shopping"));
        assert_eq!(back.for_domain.as_deref(), Some("shop.example"));
        // A non-JSON legacy value is treated as a bare target.
        assert_eq!(parse_meta("just-a-target").target, "just-a-target");
    }

    // ── create → list round-trip ─────────────────────────────────────────────────

    #[tokio::test]
    async fn create_then_list_round_trips_with_distinct_aliases() {
        let store = store().await;
        let req = CreateReq {
            target: None,
            description: Some("newsletter".into()),
            for_domain: Some("news.example".into()),
        };
        let a = create_alias(&store, "acct-a", "alice@corp.example", &req)
            .await
            .unwrap();
        let b = create_alias(
            &store,
            "acct-a",
            "alice@corp.example",
            &CreateReq::default(),
        )
        .await
        .unwrap();

        assert_eq!(a.state, STATE_ENABLED);
        assert!(a.alias_addr.ends_with("@corp.example"));
        assert_ne!(a.alias_addr, b.alias_addr, "aliases are unique per account");
        // Default forwarding target = the account's own address.
        assert_eq!(parse_meta(&b.target_desc).target, "alice@corp.example");
        assert_eq!(
            parse_meta(&a.target_desc).description.as_deref(),
            Some("newsletter")
        );

        let listed = list_active(&store, "acct-a").await.unwrap();
        assert_eq!(listed.len(), 2);
    }

    // ── enable / disable / delete lifecycle ──────────────────────────────────────

    #[tokio::test]
    async fn lifecycle_enable_disable_delete() {
        let store = store().await;
        let row = create_alias(
            &store,
            "acct-a",
            "alice@corp.example",
            &CreateReq::default(),
        )
        .await
        .unwrap();
        let id = row.id.clone();

        // Disable → reflected in the row + still listed (not deleted).
        assert!(matches!(
            change_state(&store, "acct-a", &id, STATE_DISABLED).await.unwrap(),
            Outcome::Ok(r) if r.state == STATE_DISABLED
        ));
        assert_eq!(list_active(&store, "acct-a").await.unwrap().len(), 1);

        // Re-enable.
        assert!(matches!(
            change_state(&store, "acct-a", &id, STATE_ENABLED).await.unwrap(),
            Outcome::Ok(r) if r.state == STATE_ENABLED
        ));

        // Soft-delete → gone from the active list, tombstone persists as `deleted`.
        assert!(soft_delete(&store, "acct-a", &id).await.unwrap());
        assert!(list_active(&store, "acct-a").await.unwrap().is_empty());
        assert_eq!(
            store.get_masked_email(&id).await.unwrap().unwrap().state,
            STATE_DELETED
        );

        // A missing id is a clean NotFound / false (idempotent).
        assert!(matches!(
            change_state(&store, "acct-a", "nope", STATE_DISABLED)
                .await
                .unwrap(),
            Outcome::NotFound
        ));
        assert!(!soft_delete(&store, "acct-a", "nope").await.unwrap());
    }

    // ── per-user scoping: A cannot touch B's alias ───────────────────────────────

    #[tokio::test]
    async fn per_user_scoping_isolates_accounts() {
        let store = store().await;
        let a = create_alias(
            &store,
            "acct-a",
            "alice@corp.example",
            &CreateReq::default(),
        )
        .await
        .unwrap();

        // Account B does not see A's alias.
        assert!(list_active(&store, "acct-b").await.unwrap().is_empty());
        assert!(
            owned_alias(&store, "acct-b", &a.id)
                .await
                .unwrap()
                .is_none()
        );

        // Account B cannot disable A's alias (NotFound) and it stays enabled.
        assert!(matches!(
            change_state(&store, "acct-b", &a.id, STATE_DISABLED)
                .await
                .unwrap(),
            Outcome::NotFound
        ));
        assert!(!soft_delete(&store, "acct-b", &a.id).await.unwrap());
        assert_eq!(
            store.get_masked_email(&a.id).await.unwrap().unwrap().state,
            STATE_ENABLED,
            "B's failed mutation must not affect A's alias"
        );

        // The owner still can.
        assert!(matches!(
            change_state(&store, "acct-a", &a.id, STATE_DISABLED)
                .await
                .unwrap(),
            Outcome::Ok(_)
        ));
    }

    // ── on-send From-rewrite (26.10 follow-up a) ─────────────────────────────────

    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A fake inner submitter: records the last `Outgoing` it was handed and how
    /// many times it fired (so a fail-closed rejection can assert it was NEVER
    /// reached, i.e. no bytes hit the wire).
    #[derive(Default)]
    struct FakeInner {
        last: Mutex<Option<Outgoing>>,
        calls: AtomicUsize,
    }

    #[async_trait]
    impl MailSubmitter for FakeInner {
        async fn submit(&self, msg: Outgoing) -> EngineResult<SubmissionResult> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let accepted = msg.rcpt_to.clone();
            *self.last.lock().unwrap() = Some(msg);
            Ok(SubmissionResult {
                accepted,
                rejected: Vec::new(),
            })
        }
    }

    fn outgoing(from: &str) -> Outgoing {
        Outgoing {
            mail_from: from.to_string(),
            rcpt_to: vec!["dest@partner.example".into()],
            raw: b"From: whoever\r\n\r\nhi".to_vec(),
        }
    }

    /// Sending FROM an enabled alias the account owns rewrites the envelope to the
    /// canonical alias and reaches the wire.
    #[tokio::test]
    async fn send_from_owned_enabled_alias_rewrites_and_delivers() {
        let store = store().await;
        let row = create_alias(
            &store,
            "acct-a",
            "alice@corp.example",
            &CreateReq::default(),
        )
        .await
        .unwrap();
        let inner = Arc::new(FakeInner::default());
        let sub = MaskedSubmitter::new(store.clone(), "acct-a".into(), inner.clone());

        // Submit with the alias in mixed case — the canonical (stored) form is presented.
        let res = sub
            .submit(outgoing(&row.alias_addr.to_ascii_uppercase()))
            .await
            .unwrap();
        assert_eq!(res.accepted, vec!["dest@partner.example".to_string()]);
        assert_eq!(inner.calls.load(Ordering::SeqCst), 1, "reached the wire");
        assert_eq!(
            inner.last.lock().unwrap().as_ref().unwrap().mail_from,
            row.alias_addr,
            "envelope From is the canonical alias"
        );
        // Usage was recorded.
        assert!(
            store
                .get_masked_email(&row.id)
                .await
                .unwrap()
                .unwrap()
                .last_used_at
                .is_some()
        );
    }

    /// Sending FROM another account's alias is refused fail-closed — nothing sends.
    #[tokio::test]
    async fn send_from_non_owned_alias_fails_closed() {
        let store = store().await;
        // acct-b owns the alias; acct-a tries to send as it.
        let b = create_alias(&store, "acct-b", "bob@corp.example", &CreateReq::default())
            .await
            .unwrap();
        let inner = Arc::new(FakeInner::default());
        let sub = MaskedSubmitter::new(store.clone(), "acct-a".into(), inner.clone());

        let err = sub.submit(outgoing(&b.alias_addr)).await.unwrap_err();
        assert!(matches!(err, EngineError::Protocol(_)));
        assert_eq!(
            inner.calls.load(Ordering::SeqCst),
            0,
            "cross-account alias must never reach the wire"
        );
    }

    /// A disabled or deleted owned alias is not usable as `From` (fail-closed).
    #[tokio::test]
    async fn send_from_disabled_or_deleted_alias_fails_closed() {
        let store = store().await;
        let inner = Arc::new(FakeInner::default());
        let sub = MaskedSubmitter::new(store.clone(), "acct-a".into(), inner.clone());

        // Disabled.
        let disabled = create_alias(
            &store,
            "acct-a",
            "alice@corp.example",
            &CreateReq::default(),
        )
        .await
        .unwrap();
        change_state(&store, "acct-a", &disabled.id, STATE_DISABLED)
            .await
            .unwrap();
        let err = sub
            .submit(outgoing(&disabled.alias_addr))
            .await
            .unwrap_err();
        assert!(matches!(err, EngineError::Protocol(_)));

        // Deleted (soft-delete tombstone still occupies the address).
        let deleted = create_alias(
            &store,
            "acct-a",
            "alice@corp.example",
            &CreateReq::default(),
        )
        .await
        .unwrap();
        soft_delete(&store, "acct-a", &deleted.id).await.unwrap();
        let err = sub.submit(outgoing(&deleted.alias_addr)).await.unwrap_err();
        assert!(matches!(err, EngineError::Protocol(_)));

        assert_eq!(
            inner.calls.load(Ordering::SeqCst),
            0,
            "a disabled/deleted alias must never reach the wire"
        );
    }

    /// An ordinary (non-alias) `From` is forwarded byte-for-byte unchanged.
    #[tokio::test]
    async fn normal_from_is_byte_unchanged() {
        let store = store().await;
        // An alias exists for the account, but the send is from the real address.
        let _ = create_alias(
            &store,
            "acct-a",
            "alice@corp.example",
            &CreateReq::default(),
        )
        .await
        .unwrap();
        let inner = Arc::new(FakeInner::default());
        let sub = MaskedSubmitter::new(store.clone(), "acct-a".into(), inner.clone());

        let original = outgoing("alice@corp.example");
        sub.submit(original.clone()).await.unwrap();
        let seen = inner.last.lock().unwrap().clone().unwrap();
        assert_eq!(seen.mail_from, original.mail_from);
        assert_eq!(seen.rcpt_to, original.rcpt_to);
        assert_eq!(seen.raw, original.raw, "message bytes untouched");
    }
}
