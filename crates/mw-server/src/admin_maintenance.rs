//! Admin-session-gated maintenance surface (t14 26.14, plan §Wave-B E-mount).
//!
//! Two admin-gated server capabilities, both fail-closed behind the same
//! `mw_admin_session` gate the rest of `/admin/*` uses (cookie + `admin.enabled`):
//!
//! 1. **JWZ historical backfill** — `POST /admin/maintenance/rethread` invokes the
//!    engine's idempotent one-shot [`mw_engine::Engine::rethread_account`] (E5) for
//!    an admin-selected account and returns its [`RethreadSummary`] as JSON. This is
//!    the EXACT contract the web "re-thread" button (t14-E-web-backfill) is built
//!    against: path `POST /admin/maintenance/rethread`, body `{ "accountId": "…" }`,
//!    response `{ accounts, messages, threads, reassigned }`. An admin audit line is
//!    emitted per run. There is NO automatic trigger anywhere — it is one-shot and
//!    explicit (the CLI `mailwoman maintenance rethread <account>` is the other, both
//!    admin/operator-driven).
//!
//! 2. **Server-metadata admin account passthrough** (E4's HUMAN-flag-3 requirement) —
//!    an authenticated admin session may drive `ServerMetadata/get|set` +
//!    `MailboxRights/get|set` against a SELECTED provisioned account's backend at
//!    `/jmap/api`, honoring the `accountId` carried in each JMAP method call. The
//!    `/jmap/api` handler ([`crate::jmap_api`]) calls [`try_admin_jmap_passthrough`]
//!    ONLY when the normal (mailbox-cookie) JMAP auth fails, so the non-admin path is
//!    byte-unchanged and never weakened. The passthrough is restricted to exactly the
//!    four metadata/ACL methods (any other method ⇒ fall through to the normal 401)
//!    and to a single account per request.

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::AppState;

const ADMIN_COOKIE: &str = "mw_admin_session";

/// The four JMAP methods the admin account passthrough authorizes (RFC 5464
/// METADATA + RFC 4314 ACL, the E4 editor's surface). Anything else in the request
/// makes the passthrough decline (→ the normal 401 stands).
const ADMIN_JMAP_METHODS: &[&str] = &[
    "ServerMetadata/get",
    "ServerMetadata/set",
    "MailboxRights/get",
    "MailboxRights/set",
];

/// The `/admin/maintenance/*` router (merged + ridden by `lib.rs`'s middleware).
pub(crate) fn admin_maintenance_router() -> Router<AppState> {
    Router::new().route("/admin/maintenance/rethread", post(rethread))
}

// ── admin session gate (mirrors admin.rs / admin_sso.rs) ─────────────────────

fn unauthorized() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "admin authentication required" })),
    )
        .into_response()
}

fn admin_cookie(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(header::COOKIE)?.to_str().ok()?;
    for part in raw.split(';') {
        if let Some(v) = part.trim().strip_prefix(&format!("{ADMIN_COOKIE}="))
            && !v.is_empty()
        {
            return Some(v.to_string());
        }
    }
    None
}

/// Resolve the authenticated admin id, or `None`. Enforces the `admin.enabled`
/// gate (disabled ⇒ no admin is ever authenticated).
async fn admin_id(state: &AppState, headers: &HeaderMap) -> Option<String> {
    if !state.v6.admin_enabled {
        return None;
    }
    let token = admin_cookie(headers)?;
    let hash = crate::push_relay::hash_token(&token);
    state.store.get_admin_session(&hash).await.ok().flatten()
}

/// Append an admin audit line via the shared append-only mechanism. `action` is a
/// stable kebab-case verb; `detail` MUST NOT carry any secret / mail content (it is
/// still run through [`mw_admin::redact_detail`] as a backstop). Non-fatal.
async fn audit(state: &AppState, actor: &str, action: &str, target: &str, detail: Value) {
    let entry = mw_admin::AuditLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        ts: chrono::Utc::now().to_rfc3339(),
        actor: actor.to_string(),
        actor_kind: mw_admin::ActorKind::Admin,
        action: action.to_string(),
        target: Some(target.to_string()),
        detail_json: mw_admin::redact_detail(&detail),
        ip: None,
    };
    if let Err(e) = state.v6.admin.audit(entry).await {
        tracing::warn!("admin maintenance audit append failed: {e}");
    }
}

// ── 1. JWZ backfill endpoint ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RethreadReq {
    account_id: String,
}

/// `POST /admin/maintenance/rethread` `{ "accountId": "<id>" }` — run the engine's
/// idempotent one-shot JWZ backfill (E5) for the selected account and return the
/// summary. Admin-session-gated + engine-mode only; fail-closed.
async fn rethread(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<RethreadReq>,
) -> Response {
    let Some(actor) = admin_id(&state, &headers).await else {
        return unauthorized();
    };
    let Some(engine) = &state.engine else {
        return (
            StatusCode::NOT_IMPLEMENTED,
            Json(json!({ "error": "rethread requires engine mode" })),
        )
            .into_response();
    };
    match engine.rethread_account(&body.account_id).await {
        Ok(summary) => {
            audit(
                &state,
                &actor,
                "maintenance-rethread",
                &body.account_id,
                json!({
                    "accounts": summary.accounts,
                    "messages": summary.messages,
                    "threads": summary.threads,
                    "reassigned": summary.reassigned,
                }),
            )
            .await;
            Json(json!({
                "accounts": summary.accounts,
                "messages": summary.messages,
                "threads": summary.threads,
                "reassigned": summary.reassigned,
            }))
            .into_response()
        }
        Err(e) => {
            tracing::warn!("rethread failed for {}: {e}", body.account_id);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "rethread failed" })),
            )
                .into_response()
        }
    }
}

// ── 2. Server-metadata / ACL admin account passthrough ───────────────────────

/// If `request` is a pure metadata/ACL request (every `methodCall` is one of
/// [`ADMIN_JMAP_METHODS`]) that names a SINGLE `accountId`, return that id; else
/// `None` (so the caller declines the passthrough and the normal 401 stands).
fn metadata_passthrough_account(request: &Value) -> Option<String> {
    let calls = request.get("methodCalls")?.as_array()?;
    if calls.is_empty() {
        return None;
    }
    let mut account: Option<String> = None;
    for call in calls {
        let arr = call.as_array()?;
        let name = arr.first()?.as_str()?;
        if !ADMIN_JMAP_METHODS.contains(&name) {
            return None;
        }
        let acct = arr.get(1)?.get("accountId")?.as_str()?;
        match &account {
            None => account = Some(acct.to_string()),
            Some(existing) if existing == acct => {}
            // A request mixing accounts is refused — one account per admin request.
            Some(_) => return None,
        }
    }
    account
}

/// Admin-gated `/jmap/api` passthrough for the E4 server-metadata editor. Called by
/// [`crate::jmap_api`] ONLY after the normal mailbox-cookie auth has FAILED, so it
/// is purely additive and never weakens the standard path. Returns:
///   * `Some(response)` — the request was an admin-authorized metadata/ACL request
///     (dispatched to the selected account's backend, or a clean error).
///   * `None` — decline: not engine mode, not a pure metadata/ACL request, mixed
///     accounts, or no valid admin session. The caller then returns its normal 401.
pub(crate) async fn try_admin_jmap_passthrough(
    state: &AppState,
    headers: &HeaderMap,
    body: &[u8],
) -> Option<Response> {
    // Only engine mode has local account backends to drive.
    let engine = state.engine.as_ref()?;
    let request: Value = serde_json::from_slice(body).ok()?;
    let account_id = metadata_passthrough_account(&request)?;
    // Fail-closed: a valid admin session is required. No session ⇒ decline so the
    // caller emits the identical normal 401 (never reveals the passthrough exists).
    let actor = admin_id(state, headers).await?;

    // Connect the selected account's backend (metadata/ACL ride live IMAP calls).
    if let Err(e) = crate::engine_mode::ensure_account(engine, &account_id).await {
        tracing::warn!("admin metadata passthrough: account {account_id} unavailable: {e}");
        return Some(
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": "selected account is not available" })),
            )
                .into_response(),
        );
    }

    // Record which methods the admin drove against which account (NO values — the
    // metadata/ACL payloads are not logged).
    let methods: Vec<&str> = request
        .get("methodCalls")
        .and_then(Value::as_array)
        .map(|calls| {
            calls
                .iter()
                .filter_map(|c| c.as_array()?.first()?.as_str())
                .collect()
        })
        .unwrap_or_default();
    audit(
        state,
        &actor,
        "admin-metadata-passthrough",
        &account_id,
        json!({ "methods": methods }),
    )
    .await;

    // Dispatch against the SELECTED account (its accountId is the context; each call
    // already carries the same id, so routing lands on that account's backend).
    let response = engine.handle_jmap(&account_id, &request).await;
    Some(Json(response).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── pure logic: the metadata/ACL method-allowlist + single-account gate ──────

    #[test]
    fn accepts_pure_single_account_metadata_request() {
        let req = json!({
            "using": ["urn:ietf:params:jmap:core"],
            "methodCalls": [
                ["ServerMetadata/get", { "accountId": "acct-1", "mailboxId": null }, "sm"],
                ["ServerMetadata/set", { "accountId": "acct-1", "mailboxId": null, "entry": "/private/comment", "value": "x" }, "sms"],
            ],
        });
        assert_eq!(
            metadata_passthrough_account(&req).as_deref(),
            Some("acct-1")
        );
    }

    #[test]
    fn accepts_mailbox_rights_methods() {
        let req = json!({
            "methodCalls": [
                ["MailboxRights/get", { "accountId": "a", "mailboxId": "mb" }, "mr"],
                ["MailboxRights/set", { "accountId": "a", "mailboxId": "mb", "identifier": "u", "rights": "lr" }, "mrs"],
            ],
        });
        assert_eq!(metadata_passthrough_account(&req).as_deref(), Some("a"));
    }

    #[test]
    fn declines_non_allowlisted_method() {
        // A single non-metadata method taints the whole request → decline.
        let req = json!({
            "methodCalls": [
                ["ServerMetadata/get", { "accountId": "a" }, "sm"],
                ["Email/get", { "accountId": "a", "ids": [] }, "e"],
            ],
        });
        assert_eq!(metadata_passthrough_account(&req), None);
    }

    #[test]
    fn declines_mixed_accounts() {
        let req = json!({
            "methodCalls": [
                ["ServerMetadata/get", { "accountId": "a" }, "sm"],
                ["ServerMetadata/get", { "accountId": "b" }, "sm2"],
            ],
        });
        assert_eq!(metadata_passthrough_account(&req), None);
    }

    #[test]
    fn declines_empty_or_missing_account() {
        assert_eq!(metadata_passthrough_account(&json!({})), None);
        assert_eq!(
            metadata_passthrough_account(&json!({ "methodCalls": [] })),
            None
        );
        let no_acct = json!({ "methodCalls": [["ServerMetadata/get", {}, "sm"]] });
        assert_eq!(metadata_passthrough_account(&no_acct), None);
    }

    // ── HTTP gate: the admin-session gate + wiring, over a real spawned server ────
    //
    // These drive the two Wave-B server capabilities end-to-end (minus a live IMAP
    // backend, which is E-e2e's job). Housed here rather than in a `tests/t14_*.rs`
    // file so they stay inside E-mount's owned surface (E-e2e owns `tests/t14_*.rs`).

    use crate::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
    use std::net::SocketAddr;
    use std::path::PathBuf;

    const SERVER_KEY_HEX: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
    const ADMIN_USER: &str = "root";
    const ADMIN_PASS: &str = "hunter2";

    fn unique() -> String {
        static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        format!(
            "{}_{}_{}",
            std::process::id(),
            SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )
    }

    fn web_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("mw-t14-mnt-web-{}", unique()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("index.html"),
            "<!doctype html><title>MW</title><div id=app>MW</div>",
        )
        .unwrap();
        dir
    }

    async fn spawn(mode: ServerMode) -> String {
        let db = std::env::temp_dir().join(format!("mw-t14-mnt-{}.db", unique()));
        let config = AppConfig {
            db_path: db.to_string_lossy().into_owned(),
            server_key_hex: Some(SERVER_KEY_HEX.into()),
            web_dir: Some(web_dir()),
            cookie_secure: false,
            mode,
            hardening: HardeningConfig::default(),
            security: SecurityConfig::default(),
        };
        let v6 = V6Config {
            admin_enabled: true,
            admin_username: Some(ADMIN_USER.into()),
            admin_password: Some(ADMIN_PASS.into()),
            redis_url: None,
        };
        let app = build_app_full(config, v6).await.expect("server boots").0;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr: SocketAddr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        format!("http://{addr}")
    }

    /// Log in through the REAL `/admin/login` route → the `mw_admin_session` cookie
    /// value as a ready-to-send `Cookie` header.
    async fn admin_login(c: &reqwest::Client, base: &str) -> String {
        let resp = c
            .post(format!("{base}/admin/login"))
            .json(&json!({ "username": ADMIN_USER, "password": ADMIN_PASS }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200, "admin login succeeds");
        let set_cookie = resp
            .headers()
            .get(reqwest::header::SET_COOKIE)
            .expect("login sets a cookie")
            .to_str()
            .unwrap();
        let pair = set_cookie.split(';').next().unwrap().to_string();
        assert!(
            pair.starts_with(&format!("{ADMIN_COOKIE}=")),
            "cookie: {pair}"
        );
        pair
    }

    #[tokio::test]
    async fn rethread_endpoint_admin_gated_and_returns_summary() {
        let base = spawn(ServerMode::Engine).await;
        let c = reqwest::Client::new();

        // Unauth → 401 (fail-closed; never runs the backfill).
        let no_cookie = c
            .post(format!("{base}/admin/maintenance/rethread"))
            .json(&json!({ "accountId": "acct-unknown" }))
            .send()
            .await
            .unwrap();
        assert_eq!(
            no_cookie.status(),
            401,
            "rethread requires an admin session"
        );

        // Bogus admin cookie → 401.
        let bogus = c
            .post(format!("{base}/admin/maintenance/rethread"))
            .header(reqwest::header::COOKIE, format!("{ADMIN_COOKIE}=nope"))
            .json(&json!({ "accountId": "acct-unknown" }))
            .send()
            .await
            .unwrap();
        assert_eq!(bogus.status(), 401, "an unknown admin token is rejected");

        // Admin logs in → 200 + the summary. An account with no stored mail is a
        // clean no-op backfill: accounts=1, messages=0, threads=0, reassigned=0.
        let cookie = admin_login(&c, &base).await;
        let ok = c
            .post(format!("{base}/admin/maintenance/rethread"))
            .header(reqwest::header::COOKIE, &cookie)
            .json(&json!({ "accountId": "acct-unknown" }))
            .send()
            .await
            .unwrap();
        assert_eq!(ok.status(), 200, "admin drives the backfill");
        let body: Value = ok.json().await.unwrap();
        assert_eq!(body["accounts"], 1, "one account: {body}");
        assert_eq!(body["messages"], 0, "no stored mail: {body}");
        assert_eq!(body["threads"], 0, "no threads: {body}");
        assert_eq!(body["reassigned"], 0, "idempotent no-op: {body}");
    }

    #[tokio::test]
    async fn rethread_requires_engine_mode() {
        let base = spawn(ServerMode::Proxy).await;
        let c = reqwest::Client::new();
        let cookie = admin_login(&c, &base).await;
        let resp = c
            .post(format!("{base}/admin/maintenance/rethread"))
            .header(reqwest::header::COOKIE, &cookie)
            .json(&json!({ "accountId": "acct-unknown" }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 501, "proxy mode has no engine");
    }

    #[tokio::test]
    async fn metadata_passthrough_is_admin_gated_and_method_scoped() {
        let base = spawn(ServerMode::Engine).await;
        let c = reqwest::Client::new();
        let cookie = admin_login(&c, &base).await;

        let metadata_req = json!({
            "using": ["urn:ietf:params:jmap:core"],
            "methodCalls": [["ServerMetadata/get", { "accountId": "acct-unknown", "mailboxId": null }, "sm"]],
        });

        // (a) metadata request, NO admin session → the normal JMAP 401 (declined).
        let unauth = c
            .post(format!("{base}/jmap/api"))
            .json(&metadata_req)
            .send()
            .await
            .unwrap();
        assert_eq!(unauth.status(), 401, "no session → normal 401");

        // (b) NON-metadata JMAP + valid admin session → still 401. The admin cookie
        //     authorizes ONLY the four metadata/ACL methods, never arbitrary JMAP.
        let arbitrary = c
            .post(format!("{base}/jmap/api"))
            .header(reqwest::header::COOKIE, &cookie)
            .json(&json!({
                "using": ["urn:ietf:params:jmap:core"],
                "methodCalls": [["Email/get", { "accountId": "acct-unknown", "ids": [] }, "e"]],
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(
            arbitrary.status(),
            401,
            "admin session does not authorize non-metadata JMAP"
        );

        // (c) metadata request + valid admin session → the passthrough engages and
        //     reaches account-backend connection. 502 (NOT 401) proves the admin gate
        //     passed + the allowlist admitted the request (no live backend in-test).
        let engaged = c
            .post(format!("{base}/jmap/api"))
            .header(reqwest::header::COOKIE, &cookie)
            .json(&metadata_req)
            .send()
            .await
            .unwrap();
        assert_eq!(
            engaged.status(),
            502,
            "admin metadata passthrough engaged (gate passed, backend unavailable)"
        );
    }
}
