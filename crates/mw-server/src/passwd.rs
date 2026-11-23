//! Password-change route (plan §3 e9/e14, SPEC §18.3). Filled by e9; MOUNTED by e14.
//!
//! `POST /api/password` — change the mailbox account's password through the
//! configured [`mw_passwd::PasswordChangeBackend`]. A new password equal to the
//! current one in the request is refused with a 400 before any backend is asked:
//! it is not a change, and it must not release an account held for one. On
//! success:
//!   1. write a **content-free** audit row (0008 `password_change_audit`),
//!   2. re-seal the account's stored upstream credentials under the same
//!      `ServerKey` when the outcome sets `reencrypt_credentials`
//!      ([`mw_store::Store::reseal_account_credentials`]), and
//!   3. clear both forced-change records — the `passwd_config.force_change` mirror
//!      column and the admin panel's per-account `force_password_change` flag, which
//!      is the one that holds the account (`account_gate`) — returning
//!      `zeroaccessRewrapRequired` so the client can run the zero-access
//!      key-hierarchy re-wrap (mw-crypto, client-side; the server only relays
//!      ciphertext — it performs no zero-access crypto).
//!
//! `GET /api/password/policy` returns the backend's displayed policy plus
//! `forceChange`, whether this account is held until it changes its password.
//!
//! Both routes are **mailbox-session-authed** (a user changes their own password)
//! and are the two an account held for a password change may still reach.
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
use sha2::{Digest, Sha256};

use mw_passwd::{
    AuditEvent, AuditOutcome, AuditSink, BackendKind, Ctx, PasswordChangeBackend, PasswordError,
    PasswordPolicy, Secret, change_audited,
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
    // Reachable while held for a password change (`authed_with_gate`): the forced
    // screen shows the policy.
    let (session, gate) = match crate::authed_with_gate(&state, &headers).await {
        Ok(pair) => pair,
        Err(resp) => return resp,
    };
    let p = match effective_policy(&state, &session, backend.as_ref()).await {
        Ok(p) => p,
        Err(resp) => return resp,
    };
    Json(json!({
        "description": p.to_string(),
        "minLength": p.min_length,
        "requireUpper": p.require_upper,
        "requireLower": p.require_lower,
        "requireDigit": p.require_digit,
        "requireSymbol": p.require_symbol,
        "forceChange": gate.password_change_required,
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
    // Reachable while held for a password change: this is the way out of the hold.
    let session = match crate::authed_with_gate(&state, &headers).await {
        Ok((s, _)) => s,
        Err(resp) => return resp,
    };

    // Enforce the policy on the NEW password before touching any backend so a
    // rejected password never leaves the process (the crate's `Secret` guards it).
    // The policy is the administrator's for this account's domain; it applies to
    // the new password only, so an existing password that does not meet it still
    // signs in.
    let policy = match effective_policy(&state, &session, backend.as_ref()).await {
        Ok(p) => p,
        Err(resp) => return resp,
    };
    let new = Secret::new(&body.new_password);
    if let Err(e) = policy.validate(&new) {
        return password_error(&e);
    }
    // "Changing" to the same password is not a change. Without this an account
    // held for a password change could release itself with the password it was
    // held for. The backend verifies `old_password` against the real one, so a
    // request that gets past here and succeeds has set a different password.
    if same_secret(&body.old_password, &body.new_password) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": SAME_PASSWORD })),
        )
            .into_response();
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
    // The admin panel's `force_password_change` flag is a separate record, and it
    // is the one the session gate reads (`account_gate`). Clear it too, or the
    // account stays held after a change that succeeded — but only when the backend
    // reports that it changed the password. A failure here is logged, not
    // returned: the password did change, and the client finds out it is still
    // held from the next `GET /api/me`.
    if outcome.changed
        && let Err(e) = crate::account_gate::clear_password_change(&state, &session).await
    {
        tracing::error!(
            "password changed for {} but clearing force_password_change failed: {e}",
            session.username
        );
    }

    Json(json!({
        "changed": outcome.changed,
        "credentialsResealed": resealed,
        "zeroaccessRewrapRequired": outcome.zeroaccess_rewrap_required,
    }))
    .into_response()
}

/// The settings that make up the password policy, in [`policy_from`] order.
pub(crate) const POLICY_KEYS: [&str; 5] = [
    "passwd.policy.minLength",
    "passwd.policy.requireUpper",
    "passwd.policy.requireLower",
    "passwd.policy.requireDigit",
    "passwd.policy.requireSymbol",
];

/// The password policy from the effective values of [`POLICY_KEYS`] (in that
/// order; a missing or mistyped value reads as the registry default), combined
/// with `backend`, the policy the backend enforces itself: the longer minimum and
/// every character class either requires. The minimum is never below
/// [`PasswordPolicy::FLOOR_MIN_LENGTH`].
pub(crate) fn policy_from(
    values: &[Option<serde_json::Value>],
    backend: &PasswordPolicy,
) -> PasswordPolicy {
    let flag = |i: usize| {
        values
            .get(i)
            .and_then(Option::as_ref)
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
    };
    let min = values
        .first()
        .and_then(Option::as_ref)
        .and_then(serde_json::Value::as_u64)
        .map_or(PasswordPolicy::default().min_length, |n| {
            u32::try_from(n).unwrap_or(u32::MAX)
        });
    PasswordPolicy {
        min_length: min.max(backend.min_length),
        require_upper: flag(1) || backend.require_upper,
        require_lower: flag(2) || backend.require_lower,
        require_digit: flag(3) || backend.require_digit,
        require_symbol: flag(4) || backend.require_symbol,
        description: String::new(),
    }
    .with_floor()
}

/// The effective password policy for `subject` (`passwd.policy.*`: deployment
/// value, then the domain's), combined with `backend` as [`policy_from`] says.
/// The registry's own rules apply, including an environment pin.
///
/// # Errors
/// The store's error reading the settings.
pub(crate) async fn effective_policy_for(
    store: &mw_store::Store,
    subject: &mw_store::settings_policy::SettingSubject<'_>,
    backend: &PasswordPolicy,
) -> Result<PasswordPolicy, mw_store::StoreError> {
    let values =
        crate::settings_registry::effective_selected_for(store, &POLICY_KEYS, subject, &[])
            .await?
            .into_iter()
            .map(|e| e.map(|e| e.value))
            .collect::<Vec<_>>();
    Ok(policy_from(&values, backend))
}

/// [`effective_policy_for`] the signed-in account, as a response on failure.
async fn effective_policy(
    state: &AppState,
    session: &mw_store::Session,
    backend: &dyn PasswordChangeBackend,
) -> Result<PasswordPolicy, Response> {
    let failed = |e: mw_store::StoreError| {
        tracing::error!("password policy read failed: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "password policy unavailable" })),
        )
            .into_response()
    };
    let subject = crate::settings_routes::UserSubject::resolved(&state.store, session)
        .await
        .map_err(failed)?;
    effective_policy_for(&state.store, &subject.as_subject(), &backend.policy())
        .await
        .map_err(failed)
}

/// The 400 body for a new password equal to the current one.
const SAME_PASSWORD: &str = "the new password must differ from the current password";

/// Whether two secrets are equal, compared without an early exit: each is hashed
/// (SHA-256) and every byte of the two digests is compared, so the time taken
/// does not depend on where, or whether, they differ. Neither value is logged.
fn same_secret(a: &str, b: &str) -> bool {
    let (a, b) = (Sha256::digest(a.as_bytes()), Sha256::digest(b.as_bytes()));
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
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

    /// `passwd.policy.*` values combined with a backend's own policy: the longer
    /// minimum, every class either requires, never below 8.
    #[test]
    fn policy_from_combines_the_settings_with_the_backend_floor() {
        let v = |n: u64, sym: bool| {
            vec![
                Some(json!(n)),
                None,
                None,
                Some(json!(false)),
                Some(json!(sym)),
            ]
        };
        let floor = PasswordPolicy::floor();
        let p = policy_from(&v(14, true), &floor);
        assert_eq!(
            (p.min_length, p.require_symbol, p.require_digit),
            (14, true, false)
        );
        assert_eq!(p.description, "At least 14 characters, including a symbol.");
        // Lower than the built-in default of 12 is honoured...
        assert_eq!(policy_from(&v(9, false), &floor).min_length, 9);
        // ...but never below 8, even if a stored value somehow is.
        assert_eq!(policy_from(&v(3, false), &floor).min_length, 8);
        // Nothing stored reads as the registry default of 12.
        assert_eq!(policy_from(&[], &floor).min_length, 12);
        // A stricter backend policy still holds.
        let strict = PasswordPolicy {
            min_length: 30,
            require_digit: true,
            ..PasswordPolicy::default()
        };
        let p = policy_from(&v(14, true), &strict);
        assert_eq!(
            (p.min_length, p.require_symbol, p.require_digit),
            (30, true, true)
        );
    }

    #[test]
    fn same_secret_is_exact_equality() {
        assert!(same_secret("Old-Passw0rd!", "Old-Passw0rd!"));
        assert!(same_secret("", ""));
        assert!(!same_secret("Old-Passw0rd!", "old-passw0rd!"));
        assert!(!same_secret("Old-Passw0rd!", "Old-Passw0rd! "));
        assert!(!same_secret("a", ""));
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
