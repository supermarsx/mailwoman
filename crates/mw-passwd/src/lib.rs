#![forbid(unsafe_code)]
//! `mw-passwd` — pluggable in-app password change (plan §2.3, SPEC §18.3).
//!
//! A [`PasswordChangeBackend`] trait with five impls — [`Local`] (Argon2id via the
//! local credential store), [`Ldap3062`] (LDAP password-modify extended op, RFC 3062),
//! [`DovecotHttp`] (Dovecot doveadm HTTP admin API), [`Poppassd`] (poppassd line
//! protocol over TCP), [`WebhookHmac`] (HMAC-SHA256-signed webhook) — plus
//! [`PasswordPolicy`] display, forced-change-on-next-login state ([`PasswdConfig`]),
//! and a [`PasswordChangeOutcome`] whose flags drive post-change coordination:
//! - **coordinated re-encryption** of sealed upstream credentials (`mw-store` seal —
//!   the SERVER re-seals; this crate only signals `reencrypt_credentials`),
//! - the **zero-access key-hierarchy re-wrap** ceremony (`zeroaccess_rewrap_required`
//!   — the CLIENT runs the crypto via `mw-crypto`; this crate only SIGNALS it, and
//!   deliberately performs no zero-access crypto itself).
//!
//! Every change is audited: run a backend through [`change_audited`] with an
//! [`AuditSink`] and it records a content-free [`AuditEvent`] on success *and*
//! failure (0008 `password_change_audit`, plan §2.7).
//!
//! ## Boundaries (what this crate does NOT own)
//! - No zero-access crypto: it only sets `zeroaccess_rewrap_required` (SPEC §9.1
//!   re-wrap is a client-side `mw-crypto` ceremony, server relays ciphertext only).
//! - No credential re-seal: it only sets `reencrypt_credentials`; `mw-server` re-seals
//!   via `mw_store::seal` (`e9`/`e14`).
//! - No transport ownership for LDAP: the RFC 3062 exop is encoded here, but the
//!   connection is an injected [`ldap3062::LdapExopTransport`] port (mw-directory's
//!   frozen `DirectorySource` exposes no exop passthrough — see that module).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

mod audit;
mod dovecot;
mod ldap3062;
mod local;
mod policy;
mod poppassd;
mod secret;
mod webhook;

pub use audit::{AuditEvent, AuditOutcome, AuditSink, BackendKind};
pub use dovecot::{DovecotConfig, DovecotHttp};
pub use ldap3062::{
    Ldap3062, LdapExopTransport, RFC3062_PASSWD_MODIFY_OID, encode_passwd_modify_request,
    parse_passwd_modify_response,
};
pub use local::{Local, LocalCredentialStore};
pub use policy::{PasswdConfig, PasswordPolicy};
pub use poppassd::{LineTransport, Poppassd, PoppassdConfig, TcpLineTransport};
pub use secret::Secret;
pub use webhook::{WebhookConfig, WebhookHmac, verify_signature};

/// Errors from a password-change attempt (plan §2.3).
#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    #[error("current password rejected")]
    WrongCurrent,
    #[error("new password violates policy: {0}")]
    PolicyViolation(String),
    #[error("backend transport error: {0}")]
    Transport(String),
    #[error("backend protocol error: {0}")]
    Protocol(String),
    #[error("not implemented")]
    Unimplemented,
}

pub type Result<T> = std::result::Result<T, PasswordError>;

/// Per-change context: which account/actor, plus the account posture flags that the
/// backend folds into the [`PasswordChangeOutcome`].
///
/// The posture flags default to `false` (`#[serde(default)]`) so existing callers and
/// serialized rows stay compatible; `mw-server` (e9/e14) sets them per account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ctx {
    pub account_id: String,
    pub username: String,
    /// This account stores sealed *upstream* credentials (IMAP/SMTP/etc.) that equal
    /// the password being changed ⇒ the server must re-seal them on success.
    #[serde(default)]
    pub reseal_credentials: bool,
    /// This is a zero-access account ⇒ a client-side key-hierarchy re-wrap is required
    /// (this crate only signals it; the crypto is `mw-crypto`, client-side).
    #[serde(default)]
    pub zeroaccess: bool,
}

impl Ctx {
    /// A minimal context (no posture flags set).
    #[must_use]
    pub fn new(account_id: impl Into<String>, username: impl Into<String>) -> Self {
        Self {
            account_id: account_id.into(),
            username: username.into(),
            reseal_credentials: false,
            zeroaccess: false,
        }
    }
}

/// The outcome of a successful change — drives post-change coordination (plan §2.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasswordChangeOutcome {
    pub changed: bool,
    /// Sealed upstream credentials must be re-encrypted (mw-server re-seals; not here).
    pub reencrypt_credentials: bool,
    /// A zero-access account needs the client-side key-hierarchy re-wrap ceremony
    /// (mw-crypto, client-side; this crate only signals — performs no crypto).
    pub zeroaccess_rewrap_required: bool,
}

impl PasswordChangeOutcome {
    /// Build the success outcome from the account posture in `ctx`. Every backend routes
    /// its success through this so the two coordination signals are computed uniformly.
    #[must_use]
    pub fn changed_from(ctx: &Ctx) -> Self {
        Self {
            changed: true,
            reencrypt_credentials: ctx.reseal_credentials,
            zeroaccess_rewrap_required: ctx.zeroaccess,
        }
    }
}

/// The pluggable password-change seam (plan §2.3).
#[async_trait]
pub trait PasswordChangeBackend: Send + Sync {
    /// Change the password; on success returns the coordination flags.
    async fn change(&self, ctx: &Ctx, old: Secret, new: Secret) -> Result<PasswordChangeOutcome>;
    /// The policy this backend enforces (shown before a change).
    fn policy(&self) -> PasswordPolicy;
    /// Which backend this is — recorded in the audit event.
    fn kind(&self) -> BackendKind;
}

/// Run a backend and emit a **content-free** audit event (success *and* failure) to
/// `sink`, then return the result. This is the entry point `mw-server` uses so that
/// "every change emits an audit event" holds by construction (plan §2.3, §2.7).
///
/// The audit event carries the account id, backend kind, and success/failure only —
/// never the old/new password (which never leave [`Secret`]).
pub async fn change_audited<B, S>(
    backend: &B,
    sink: &S,
    ctx: &Ctx,
    old: Secret,
    new: Secret,
) -> Result<PasswordChangeOutcome>
where
    B: PasswordChangeBackend + ?Sized,
    S: AuditSink + ?Sized,
{
    let result = backend.change(ctx, old, new).await;
    let outcome = match &result {
        Ok(_) => AuditOutcome::Success,
        Err(e) => AuditOutcome::Failure(e.to_string()),
    };
    sink.record(&AuditEvent {
        account_id: ctx.account_id.clone(),
        backend: backend.kind(),
        outcome,
    })
    .await;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct FixedBackend(std::result::Result<(), PasswordError>);
    #[async_trait]
    impl PasswordChangeBackend for FixedBackend {
        async fn change(
            &self,
            ctx: &Ctx,
            _old: Secret,
            _new: Secret,
        ) -> Result<PasswordChangeOutcome> {
            match &self.0 {
                Ok(()) => Ok(PasswordChangeOutcome::changed_from(ctx)),
                Err(_) => Err(PasswordError::WrongCurrent),
            }
        }
        fn policy(&self) -> PasswordPolicy {
            PasswordPolicy::default()
        }
        fn kind(&self) -> BackendKind {
            BackendKind::Local
        }
    }

    #[derive(Default)]
    struct RecordingSink(Mutex<Vec<AuditEvent>>);
    #[async_trait]
    impl AuditSink for RecordingSink {
        async fn record(&self, event: &AuditEvent) {
            self.0.lock().unwrap().push(event.clone());
        }
    }

    #[tokio::test]
    async fn change_audited_emits_success_event() {
        let sink = RecordingSink::default();
        let ctx = Ctx::new("a1", "u");
        let out = change_audited(
            &FixedBackend(Ok(())),
            &sink,
            &ctx,
            Secret::new("old"),
            Secret::new("new"),
        )
        .await
        .unwrap();
        assert!(out.changed);
        let events = sink.0.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].account_id, "a1");
        assert_eq!(events[0].backend, BackendKind::Local);
        assert_eq!(events[0].outcome, AuditOutcome::Success);
    }

    #[tokio::test]
    async fn change_audited_emits_failure_event() {
        let sink = RecordingSink::default();
        let err = change_audited(
            &FixedBackend(Err(PasswordError::WrongCurrent)),
            &sink,
            &Ctx::new("a2", "u"),
            Secret::new("old"),
            Secret::new("new"),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, PasswordError::WrongCurrent));
        let events = sink.0.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].outcome, AuditOutcome::Failure(_)));
    }

    #[test]
    fn outcome_flags_follow_ctx_posture() {
        let plain = Ctx::new("a1", "u");
        let o = PasswordChangeOutcome::changed_from(&plain);
        assert!(o.changed && !o.reencrypt_credentials && !o.zeroaccess_rewrap_required);

        let za = Ctx {
            reseal_credentials: true,
            zeroaccess: true,
            ..Ctx::new("a2", "u2")
        };
        let o = PasswordChangeOutcome::changed_from(&za);
        assert!(o.changed && o.reencrypt_credentials && o.zeroaccess_rewrap_required);
    }

    #[test]
    fn ctx_round_trips_with_defaulted_posture() {
        // A row serialized before the posture fields existed still deserializes.
        let ctx: Ctx = serde_json::from_str(r#"{"account_id":"a","username":"u"}"#).unwrap();
        assert!(!ctx.reseal_credentials && !ctx.zeroaccess);
        let back: Ctx = serde_json::from_str(&serde_json::to_string(&ctx).unwrap()).unwrap();
        assert_eq!(ctx, back);
    }
}
