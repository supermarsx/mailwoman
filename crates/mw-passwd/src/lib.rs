#![forbid(unsafe_code)]
// SCAFFOLD (t7-e0): frozen §2.3 types + method shapes as INERT stubs; the per-
// backend wire logic (LDAP-3062 exop, Dovecot HTTP, poppassd, HMAC webhook) is e3.
#![allow(dead_code)]
//! `mw-passwd` — pluggable in-app password change (plan §2.3, SPEC §18.3).
//!
//! A [`PasswordChangeBackend`] trait with impls [`Local`], [`Ldap3062`],
//! [`DovecotHttp`], [`Poppassd`], [`WebhookHmac`], plus forced-change-on-next-login,
//! password-policy display, and the outcome flags that drive:
//! - **coordinated re-encryption** of sealed upstream credentials (mw-store seal),
//! - the **zero-access key-hierarchy re-wrap** ceremony (client-side; mw-crypto
//!   zero-access; server relays ciphertext only — recovery-key path offered first).
//!
//! The webhook backend is **HMAC-SHA256 signed** (reuse in-tree `hmac`+`sha2`).
//! Every change writes an audit row (0008 `password_change_audit`, plan §2.7).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

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

/// A secret held only transiently in memory (plaintext old/new password).
///
/// A newtype so it is never accidentally logged/serialized; e3 may add `zeroize`.
#[derive(Clone)]
pub struct Secret(String);

impl Secret {
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    /// Expose the plaintext to a backend that must transmit it upstream.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Secret(***)")
    }
}

/// Per-change context (which account, actor, deployment hints).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ctx {
    pub account_id: String,
    pub username: String,
}

/// Password policy the UI displays before a change (plan §2.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PasswordPolicy {
    pub min_length: u32,
    pub require_upper: bool,
    pub require_lower: bool,
    pub require_digit: bool,
    pub require_symbol: bool,
    /// Human-readable description shown to the user.
    pub description: String,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 12,
            require_upper: false,
            require_lower: false,
            require_digit: false,
            require_symbol: false,
            description: "At least 12 characters.".into(),
        }
    }
}

/// The outcome of a successful change — drives post-change coordination (plan §2.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasswordChangeOutcome {
    pub changed: bool,
    /// Sealed upstream credentials must be re-encrypted (mw-server re-seals).
    pub reencrypt_credentials: bool,
    /// A zero-access account needs the client-side key-hierarchy re-wrap ceremony.
    pub zeroaccess_rewrap_required: bool,
}

/// The pluggable password-change seam (plan §2.3).
#[async_trait]
pub trait PasswordChangeBackend: Send + Sync {
    /// Change the password; on success returns the coordination flags.
    async fn change(&self, ctx: &Ctx, old: Secret, new: Secret) -> Result<PasswordChangeOutcome>;
    /// The policy this backend enforces (shown before a change).
    fn policy(&self) -> PasswordPolicy;
}

macro_rules! stub_backend {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Default)]
        pub struct $name;

        #[async_trait]
        impl PasswordChangeBackend for $name {
            async fn change(
                &self,
                _ctx: &Ctx,
                _old: Secret,
                _new: Secret,
            ) -> Result<PasswordChangeOutcome> {
                Err(PasswordError::Unimplemented)
            }
            fn policy(&self) -> PasswordPolicy {
                PasswordPolicy::default()
            }
        }
    };
}

stub_backend!(
    Local,
    "Local store password change (mw-store credential re-seal)."
);
stub_backend!(Ldap3062, "LDAP password-modify extended op (RFC 3062).");
stub_backend!(DovecotHttp, "Dovecot HTTP admin API password change.");
stub_backend!(Poppassd, "poppassd protocol password change.");
stub_backend!(WebhookHmac, "Custom webhook, HMAC-SHA256-signed payload.");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_never_prints_plaintext() {
        let s = Secret::new("hunter2");
        assert_eq!(format!("{s:?}"), "Secret(***)");
        assert_eq!(s.expose(), "hunter2");
    }

    #[tokio::test]
    async fn backends_are_stubs_until_e3() {
        let ctx = Ctx {
            account_id: "a1".into(),
            username: "u".into(),
        };
        let b = Ldap3062;
        assert!(matches!(
            b.change(&ctx, Secret::new("a"), Secret::new("b")).await,
            Err(PasswordError::Unimplemented)
        ));
        assert!(b.policy().min_length >= 8);
    }
}
