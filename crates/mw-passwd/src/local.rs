//! [`Local`] — change the password in Mailwoman's own credential store (Argon2id).
//!
//! Verifies the old password against the stored Argon2id PHC hash, enforces the policy
//! on the new one, and writes a fresh Argon2id PHC string via an injected
//! [`LocalCredentialStore`] port (the concrete store — `mw-store` — is wired by e14;
//! the port keeps this crate testable with no database).

use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Algorithm, Argon2, Params, Version};
use async_trait::async_trait;

use crate::{
    BackendKind, Ctx, PasswordChangeBackend, PasswordChangeOutcome, PasswordError, PasswordPolicy,
    Result, Secret,
};

/// The local credential store seam: read the current PHC hash, write a new one.
///
/// Backed by `mw-store` at mount (e14); an in-memory double is used in tests.
#[async_trait]
pub trait LocalCredentialStore: Send + Sync {
    /// The current Argon2id PHC string for the account, or `None` if unset.
    async fn current_hash(&self, account_id: &str) -> Result<Option<String>>;
    /// Persist a new Argon2id PHC string for the account.
    async fn set_hash(&self, account_id: &str, phc: &str) -> Result<()>;
}

/// OWASP-recommended Argon2id parameters (m = 19 MiB, t = 2, p = 1) — the same floor
/// mw-admin uses for admin/local password hashing.
const M_COST: u32 = 19_456;
const T_COST: u32 = 2;
const P_COST: u32 = 1;

fn gen_salt() -> Result<SaltString> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|e| PasswordError::Protocol(format!("csprng: {e}")))?;
    SaltString::encode_b64(&bytes).map_err(|e| PasswordError::Protocol(format!("salt: {e}")))
}

fn hasher() -> Result<Argon2<'static>> {
    let params = Params::new(M_COST, T_COST, P_COST, None)
        .map_err(|e| PasswordError::Protocol(format!("argon2 params: {e}")))?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

/// Hash a password to an Argon2id PHC string (`$argon2id$...`).
fn hash_password(password: &str) -> Result<String> {
    let salt = gen_salt()?;
    Ok(hasher()?
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| PasswordError::Protocol(format!("argon2 hash: {e}")))?
        .to_string())
}

/// Verify a password against a stored Argon2id PHC string.
fn verify_password(password: &str, phc: &str) -> Result<bool> {
    let parsed =
        PasswordHash::new(phc).map_err(|e| PasswordError::Protocol(format!("argon2 phc: {e}")))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

/// Local-store password change (Argon2id via `mw-store`, plan §2.3).
pub struct Local<S: LocalCredentialStore> {
    store: S,
    policy: PasswordPolicy,
}

impl<S: LocalCredentialStore> Local<S> {
    #[must_use]
    pub fn new(store: S, policy: PasswordPolicy) -> Self {
        Self { store, policy }
    }
}

#[async_trait]
impl<S: LocalCredentialStore> PasswordChangeBackend for Local<S> {
    async fn change(&self, ctx: &Ctx, old: Secret, new: Secret) -> Result<PasswordChangeOutcome> {
        self.policy.validate(&new)?;
        let phc = self
            .store
            .current_hash(&ctx.account_id)
            .await?
            .ok_or(PasswordError::WrongCurrent)?;
        if !verify_password(old.expose(), &phc)? {
            return Err(PasswordError::WrongCurrent);
        }
        let new_phc = hash_password(new.expose())?;
        self.store.set_hash(&ctx.account_id, &new_phc).await?;
        Ok(PasswordChangeOutcome::changed_from(ctx))
    }

    fn policy(&self) -> PasswordPolicy {
        self.policy.clone()
    }

    fn kind(&self) -> BackendKind {
        BackendKind::Local
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MemStore {
        hash: Mutex<Option<String>>,
    }
    #[async_trait]
    impl LocalCredentialStore for MemStore {
        async fn current_hash(&self, _account_id: &str) -> Result<Option<String>> {
            Ok(self.hash.lock().unwrap().clone())
        }
        async fn set_hash(&self, _account_id: &str, phc: &str) -> Result<()> {
            *self.hash.lock().unwrap() = Some(phc.to_string());
            Ok(())
        }
    }

    fn seeded(old: &str) -> MemStore {
        MemStore {
            hash: Mutex::new(Some(hash_password(old).unwrap())),
        }
    }

    #[tokio::test]
    async fn happy_path_verifies_old_and_rehashes_new() {
        let store = seeded("old-password-12");
        let backend = Local::new(store, PasswordPolicy::default());
        let ctx = Ctx {
            reseal_credentials: true,
            ..Ctx::new("a1", "u")
        };
        let out = backend
            .change(
                &ctx,
                Secret::new("old-password-12"),
                Secret::new("brand-new-password"),
            )
            .await
            .unwrap();
        assert!(out.changed && out.reencrypt_credentials);
        // The stored hash is the NEW password's PHC (not plaintext), and old no longer verifies.
        let phc = backend.store.current_hash("a1").await.unwrap().unwrap();
        assert!(phc.starts_with("$argon2id$"));
        assert!(!phc.contains("brand-new-password"));
        assert!(verify_password("brand-new-password", &phc).unwrap());
    }

    #[tokio::test]
    async fn deny_path_wrong_current_password() {
        let backend = Local::new(seeded("the-real-old-one"), PasswordPolicy::default());
        let err = backend
            .change(
                &Ctx::new("a1", "u"),
                Secret::new("wrong-guess-12"),
                Secret::new("new-password-12"),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, PasswordError::WrongCurrent));
    }

    #[tokio::test]
    async fn deny_path_new_password_violates_policy() {
        let policy = PasswordPolicy {
            min_length: 20,
            ..PasswordPolicy::default()
        };
        let backend = Local::new(seeded("old-password-12"), policy);
        let err = backend
            .change(
                &Ctx::new("a1", "u"),
                Secret::new("old-password-12"),
                Secret::new("too-short"),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, PasswordError::PolicyViolation(_)));
    }
}
