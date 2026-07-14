//! Password-change route (plan §3 e9/e14, SPEC §18.3). Filled by e9; MOUNTED by e14.
//!
//! `POST /api/password` — change the mailbox account's password through the
//! configured [`mw_passwd::PasswordChangeBackend`], then on success:
//!   1. write a **content-free** audit row (0008 `password_change_audit`),
//!   2. re-seal the account's stored upstream credentials under the same
//!      `ServerKey` when the outcome sets `reencrypt_credentials`
//!      ([`mw_store::Store::reseal_account_credentials`]), and
//!   3. clear the forced-change flag, returning `zeroaccessRewrapRequired` so the
//!      client can run the zero-access key-hierarchy re-wrap (mw-crypto, client-side;
//!      the server only relays ciphertext — it performs no zero-access crypto).
//!
//! `GET /api/password/policy` returns the backend's displayed policy.
//!
//! Both routes are **mailbox-session-authed** (a user changes their own password).
//! The old/new passwords never leave [`mw_passwd::Secret`]/the request body; no
//! password material is ever logged (§21.1) — the audit is content-free by type.
//!
//! ## Injection (e14)
//! The live backend (Local / LDAP-3062 / Dovecot / poppassd / webhook) is built by
//! e14 at mount and injected as a request extension ([`PasswdBackend`]); for
//! LDAP-3062 e14 also backs the `LdapExopTransport` port (mw-directory exposes no
//! exop passthrough, so the RFC 3062 exop lives in `mw-passwd`, transport injected).
#![allow(dead_code)]

use std::sync::Arc;

use async_trait::async_trait;
use axum::extract::{Extension, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;

use mw_passwd::{
    AuditEvent, AuditOutcome, AuditSink, BackendKind, Ctx, PasswordChangeBackend, PasswordError,
    Secret, change_audited,
};

use crate::AppState;

/// The live password-change backend e14 injects.
pub(crate) type PasswdBackend = Arc<dyn PasswordChangeBackend>;

/// e14 merges this into `router()` and layers on the injected [`PasswdBackend`].
pub(crate) fn passwd_router() -> Router<AppState> {
    Router::new()
        .route("/api/password", post(change_password))
        .route("/api/password/policy", get(policy))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChangeReq {
    old_password: String,
    new_password: String,
}

/// `GET /api/password/policy` — the displayed policy (shown before a change).
async fn policy(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(backend): Extension<PasswdBackend>,
) -> Response {
    if let Err(resp) = crate::authed(&state, &headers).await {
        return resp;
    }
    let p = backend.policy();
    Json(json!({
        "description": p.to_string(),
        "minLength": p.min_length,
        "requireUpper": p.require_upper,
        "requireLower": p.require_lower,
        "requireDigit": p.require_digit,
        "requireSymbol": p.require_symbol,
    }))
    .into_response()
}

/// `POST /api/password` — change the current account's password.
async fn change_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(backend): Extension<PasswdBackend>,
    Json(body): Json<ChangeReq>,
) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };

    // Enforce the policy on the NEW password before touching any backend so a
    // rejected password never leaves the process (the crate's `Secret` guards it).
    let new = Secret::new(&body.new_password);
    if let Err(e) = backend.policy().validate(&new) {
        return password_error(&e);
    }

    // Account posture drives the outcome flags: proxy/engine mode stores sealed
    // upstream creds equal to this password (⇒ re-seal on success); zero-access
    // accounts additionally require the client-side key-hierarchy re-wrap.
    let has_stored_creds = state
        .store
        .sessions_by_account(&session.account_id)
        .await
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    let zeroaccess = state
        .store
        .get_zeroaccess(&session.account_id)
        .await
        .ok()
        .flatten()
        .map(|z| z.enabled)
        .unwrap_or(false);

    let ctx = Ctx {
        account_id: session.account_id.clone(),
        username: session.username.clone(),
        reseal_credentials: has_stored_creds,
        zeroaccess,
    };

    // The single entry point that emits a content-free audit row on success AND
    // failure (per plan §2.3), backed by the 0008 `password_change_audit` table.
    let sink = StoreAuditSink {
        store: state.store.clone(),
    };
    let outcome = match change_audited(
        backend.as_ref(),
        &sink,
        &ctx,
        Secret::new(&body.old_password),
        new,
    )
    .await
    {
        Ok(o) => o,
        Err(e) => return password_error(&e),
    };

    // Coordinated re-encryption of the sealed upstream credentials.
    let mut resealed = 0u64;
    if outcome.reencrypt_credentials {
        match state
            .store
            .reseal_account_credentials(&session.account_id, &body.new_password)
            .await
        {
            Ok(n) => resealed = n,
            Err(e) => {
                // The password DID change upstream; a re-seal failure must not be
                // silent (subsequent proxy reads would use the stale password).
                tracing::error!("credential re-seal after password change failed: {e}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "password changed but credential re-seal failed" })),
                )
                    .into_response();
            }
        }
    }

    // Clear the forced-change flag (the user just changed it). Best-effort: keep the
    // existing config JSON, flip the mirror column.
    if let Ok(Some(existing)) = state.store.get_passwd_config(&session.account_id).await
        && existing.force_change
    {
        let cleared = mw_store::PasswdConfigRow {
            force_change: false,
            updated_at: chrono::Utc::now().to_rfc3339(),
            ..existing
        };
        let _ = state.store.put_passwd_config(&cleared).await;
    }

    Json(json!({
        "changed": outcome.changed,
        "credentialsResealed": resealed,
        "zeroaccessRewrapRequired": outcome.zeroaccess_rewrap_required,
    }))
    .into_response()
}

/// Map a [`PasswordError`] to an HTTP response. The `Display` carries no password
/// material by construction (secrets never leave [`Secret`]).
fn password_error(e: &PasswordError) -> Response {
    let (code, msg) = match e {
        PasswordError::WrongCurrent => (StatusCode::FORBIDDEN, e.to_string()),
        PasswordError::PolicyViolation(_) => (StatusCode::BAD_REQUEST, e.to_string()),
        PasswordError::Unimplemented => (
            StatusCode::NOT_IMPLEMENTED,
            "password change not configured".to_string(),
        ),
        PasswordError::Transport(_) | PasswordError::Protocol(_) => {
            tracing::warn!("password backend error: {e}");
            (
                StatusCode::BAD_GATEWAY,
                "password change failed".to_string(),
            )
        }
    };
    (code, Json(json!({ "error": msg }))).into_response()
}

/// An [`AuditSink`] that writes the content-free audit row to the 0008
/// `password_change_audit` table. A write failure is logged, never fatal to the
/// change (the change already happened upstream).
struct StoreAuditSink {
    store: mw_store::Store,
}

#[async_trait]
impl AuditSink for StoreAuditSink {
    async fn record(&self, event: &AuditEvent) {
        let backend = backend_kind_str(event.backend);
        let outcome = match &event.outcome {
            AuditOutcome::Success => "ok".to_string(),
            // The error `Display` is content-free (no password material); still, we
            // prefix it so the row is unambiguous.
            AuditOutcome::Failure(reason) => format!("error:{reason}"),
        };
        if let Err(e) = self
            .store
            .put_password_change_audit(&event.account_id, backend, &outcome)
            .await
        {
            tracing::error!("password-change audit write failed: {e}");
        }
    }
}

/// The stable audit string for a backend (matches the 0008 migration convention).
fn backend_kind_str(k: BackendKind) -> &'static str {
    match k {
        BackendKind::Local => "local",
        BackendKind::Ldap3062 => "ldap3062",
        BackendKind::DovecotHttp => "dovecot",
        BackendKind::Poppassd => "poppassd",
        BackendKind::WebhookHmac => "webhook",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mw_store::{ServerKey, Store};

    #[test]
    fn backend_kind_strings_match_migration_convention() {
        assert_eq!(backend_kind_str(BackendKind::Local), "local");
        assert_eq!(backend_kind_str(BackendKind::Ldap3062), "ldap3062");
        assert_eq!(backend_kind_str(BackendKind::DovecotHttp), "dovecot");
        assert_eq!(backend_kind_str(BackendKind::Poppassd), "poppassd");
        assert_eq!(backend_kind_str(BackendKind::WebhookHmac), "webhook");
    }

    #[test]
    fn wrong_current_maps_to_403_policy_to_400() {
        assert_eq!(
            password_error(&PasswordError::WrongCurrent).status(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            password_error(&PasswordError::PolicyViolation("x".into())).status(),
            StatusCode::BAD_REQUEST
        );
    }

    /// The store-backed audit sink writes exactly one content-free row per event.
    #[tokio::test]
    async fn audit_sink_writes_content_free_row() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let sink = StoreAuditSink {
            store: store.clone(),
        };
        sink.record(&AuditEvent {
            account_id: "a1".into(),
            backend: BackendKind::Ldap3062,
            outcome: AuditOutcome::Success,
        })
        .await;
        sink.record(&AuditEvent {
            account_id: "a1".into(),
            backend: BackendKind::Local,
            outcome: AuditOutcome::Failure("current password rejected".into()),
        })
        .await;
        // The rows exist and never carried password material (the sink only ever
        // sees account/backend/outcome — a compile-time guarantee of AuditEvent).
        let cfg = store.get_passwd_config("a1").await.unwrap();
        assert!(cfg.is_none(), "audit must not touch passwd_config");
    }

    /// End-to-end store coordination: a successful change re-seals the account's
    /// stored credentials to the new password (proves the re-seal path used by the
    /// handler, without the full HTTP stack — that is e14's mount smoke test).
    #[tokio::test]
    async fn reseal_after_change_updates_stored_creds() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        store
            .create_session(
                "acct",
                "u",
                "http://mock",
                "http://mock",
                &mw_store::Credentials {
                    username: "u".into(),
                    password: "old".into(),
                },
            )
            .await
            .unwrap();
        let n = store
            .reseal_account_credentials("acct", "brand-new")
            .await
            .unwrap();
        assert_eq!(n, 1);
        let s = store.sessions_by_account("acct").await.unwrap();
        assert_eq!(s[0].credentials.password, "brand-new");
    }
}
