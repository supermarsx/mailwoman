//! [`Local`] — change the password in Mailwoman's own credential store (Argon2id).
//!
//! Verifies the old password against the stored Argon2id PHC hash, enforces the policy
//! on the new one, and writes a fresh Argon2id PHC string via an injected
//! [`LocalCredentialStore`] port (the concrete store — `mw-store` — is wired by e14;
//! the port keeps this crate testable with no database).
//!
//! The cost of a new hash is an [`ArgonCost`]: the process-wide one ([`set_cost`] /
//! [`cost`], default [`ArgonCost::DEFAULT`]) or a per-backend override
//! ([`Local::with_cost`]). Verification never reads it: a stored hash is always checked
//! with the parameters written in its own PHC string, so changing the cost cannot lock
//! out an existing password. [`Local::verify`] re-hashes a password it has just verified
//! when the stored hash was made with a different cost.

use std::sync::{PoisonError, RwLock};

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
    /// Replace the stored hash with `phc` only if it is still `expected`; `Ok(false)`
    /// means it was not, and nothing was written. [`Local::verify`] writes an upgraded
    /// hash through this, so that a password change landing between its read and its
    /// write is not overwritten with a hash of the old password.
    ///
    /// This default is a read followed by a write, which narrows that window but does
    /// not close it. A store that can compare and write in one statement should
    /// override it.
    async fn replace_hash(&self, account_id: &str, expected: &str, phc: &str) -> Result<bool> {
        if self.current_hash(account_id).await?.as_deref() != Some(expected) {
            return Ok(false);
        }
        self.set_hash(account_id, phc).await?;
        Ok(true)
    }
}

/// An Argon2id cost setting that failed [`ArgonCost::new`]'s bounds.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ArgonCostError {
    #[error("argon2 memory must be {min}..={max} KiB, got {got}")]
    Memory { got: u32, min: u32, max: u32 },
    #[error("argon2 passes must be 1..={max}, got {got}")]
    Passes { got: u32, max: u32 },
    #[error("argon2 with a single pass needs at least {min} KiB of memory, got {got}")]
    SinglePassMemory { got: u32, min: u32 },
    #[error("argon2 parallelism must be 1..={max}, got {got}")]
    Parallelism { got: u32, max: u32 },
}

/// Argon2id cost parameters: memory in KiB, passes, lanes. A value of this type is
/// always within the bounds [`ArgonCost::new`] checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArgonCost {
    m_kib: u32,
    t: u32,
    p: u32,
}

impl ArgonCost {
    /// OWASP's minimum Argon2id configuration (m = 19 MiB, t = 2, p = 1) — what this
    /// backend used before the cost became a setting.
    pub const DEFAULT: Self = Self {
        m_kib: 19_456,
        t: 2,
        p: 1,
    };
    /// Lowest memory accepted: the 19 MiB of OWASP's minimum configuration.
    pub const MIN_M_KIB: u32 = 19_456;
    /// Highest memory accepted: 1 GiB. Every hash or verify in flight holds this much,
    /// so a larger value lets one mistyped setting exhaust the server.
    pub const MAX_M_KIB: u32 = 1_048_576;
    /// Lowest memory accepted with a single pass: OWASP lists t = 1 only at 46 MiB.
    pub const MIN_M_KIB_SINGLE_PASS: u32 = 47_104;
    /// Highest pass count accepted; time grows linearly with it.
    pub const MAX_T: u32 = 10;
    /// Highest lane count accepted.
    pub const MAX_P: u32 = 16;

    /// Check the three parameters against the bounds above. Out of range is an error;
    /// nothing is clamped.
    ///
    /// # Errors
    /// [`ArgonCostError`] naming the parameter that is out of range.
    pub fn new(m_kib: u32, t: u32, p: u32) -> std::result::Result<Self, ArgonCostError> {
        if !(Self::MIN_M_KIB..=Self::MAX_M_KIB).contains(&m_kib) {
            return Err(ArgonCostError::Memory {
                got: m_kib,
                min: Self::MIN_M_KIB,
                max: Self::MAX_M_KIB,
            });
        }
        if !(1..=Self::MAX_T).contains(&t) {
            return Err(ArgonCostError::Passes {
                got: t,
                max: Self::MAX_T,
            });
        }
        if t == 1 && m_kib < Self::MIN_M_KIB_SINGLE_PASS {
            return Err(ArgonCostError::SinglePassMemory {
                got: m_kib,
                min: Self::MIN_M_KIB_SINGLE_PASS,
            });
        }
        if !(1..=Self::MAX_P).contains(&p) {
            return Err(ArgonCostError::Parallelism {
                got: p,
                max: Self::MAX_P,
            });
        }
        Ok(Self { m_kib, t, p })
    }

    /// Memory in KiB.
    #[must_use]
    pub fn m_kib(self) -> u32 {
        self.m_kib
    }

    /// Passes.
    #[must_use]
    pub fn t(self) -> u32 {
        self.t
    }

    /// Lanes.
    #[must_use]
    pub fn p(self) -> u32 {
        self.p
    }
}

impl Default for ArgonCost {
    fn default() -> Self {
        Self::DEFAULT
    }
}

static COST: RwLock<ArgonCost> = RwLock::new(ArgonCost::DEFAULT);

/// Set the process-wide cost used for new hashes by every [`Local`] without its own
/// ([`Local::with_cost`]). Existing hashes are untouched and keep verifying.
pub fn set_cost(cost: ArgonCost) {
    *COST.write().unwrap_or_else(PoisonError::into_inner) = cost;
}

/// The process-wide cost ([`ArgonCost::DEFAULT`] until [`set_cost`] is called).
#[must_use]
pub fn cost() -> ArgonCost {
    *COST.read().unwrap_or_else(PoisonError::into_inner)
}

fn gen_salt() -> Result<SaltString> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|e| PasswordError::Protocol(format!("csprng: {e}")))?;
    SaltString::encode_b64(&bytes).map_err(|e| PasswordError::Protocol(format!("salt: {e}")))
}

fn hasher(cost: ArgonCost) -> Result<Argon2<'static>> {
    let params = Params::new(cost.m_kib, cost.t, cost.p, None)
        .map_err(|e| PasswordError::Protocol(format!("argon2 params: {e}")))?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

/// Hash a password to an Argon2id PHC string (`$argon2id$...`) at `cost`.
///
/// # Errors
/// [`PasswordError::Protocol`] if the CSPRNG or the hasher fails.
pub fn hash_password_with(password: &str, cost: ArgonCost) -> Result<String> {
    let salt = gen_salt()?;
    Ok(hasher(cost)?
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| PasswordError::Protocol(format!("argon2 hash: {e}")))?
        .to_string())
}

/// Verify a password against a stored Argon2id PHC string, with the parameters in
/// that string.
fn verify_password(password: &str, phc: &str) -> Result<bool> {
    let parsed =
        PasswordHash::new(phc).map_err(|e| PasswordError::Protocol(format!("argon2 phc: {e}")))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

/// The `(m_kib, t, p)` written in a PHC string, or `None` if it does not parse or
/// lacks one of the three.
#[must_use]
pub fn phc_cost(phc: &str) -> Option<(u32, u32, u32)> {
    let parsed = PasswordHash::new(phc).ok()?;
    Some((
        parsed.params.get_decimal("m")?,
        parsed.params.get_decimal("t")?,
        parsed.params.get_decimal("p")?,
    ))
}

/// Whether a stored hash is anything other than Argon2id v19 at exactly `cost`.
fn needs_rehash(phc: &str, cost: ArgonCost) -> bool {
    let Ok(parsed) = PasswordHash::new(phc) else {
        return true;
    };
    parsed.algorithm != argon2::ARGON2ID_IDENT
        || parsed.version != Some(Version::V0x13.into())
        || phc_cost(phc) != Some((cost.m_kib, cost.t, cost.p))
}

/// What [`verify_and_upgrade`] found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyOutcome {
    /// The password is not the one the stored hash was made from.
    Mismatch,
    /// The password is correct. `upgraded` is a new PHC string of the same password at
    /// the requested cost when the stored hash was made with a different one; the
    /// caller persists it. `None` means the stored hash is already at that cost (or the
    /// re-hash itself failed, in which case the next successful verify tries again).
    Match { upgraded: Option<String> },
}

/// Verify `password` against `phc` with the parameters in `phc`, and on success — only
/// on success — re-hash it at `cost` if the stored hash differs from `cost` in either
/// direction (a lowered setting is followed too; [`ArgonCost::new`] is the floor).
///
/// A wrong password does one verify and returns [`VerifyOutcome::Mismatch`] whatever
/// cost the stored hash has, so a caller who does not know the password cannot tell a
/// hash that is due an upgrade from one that is not.
///
/// # Errors
/// [`PasswordError::Protocol`] if `phc` is not a PHC string.
pub fn verify_and_upgrade(password: &str, phc: &str, cost: ArgonCost) -> Result<VerifyOutcome> {
    if !verify_password(password, phc)? {
        return Ok(VerifyOutcome::Mismatch);
    }
    let upgraded = if needs_rehash(phc, cost) {
        hash_password_with(password, cost).ok()
    } else {
        None
    };
    Ok(VerifyOutcome::Match { upgraded })
}

/// What [`Local::verify`] found and did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalVerify {
    verified: bool,
    rehashed: bool,
    rehash_error: Option<String>,
}

impl LocalVerify {
    /// Whether the password is the account's current one.
    #[must_use]
    pub fn verified(&self) -> bool {
        self.verified
    }

    /// Whether the stored hash was replaced with one at the current cost.
    #[must_use]
    pub fn rehashed(&self) -> bool {
        self.rehashed
    }

    /// Why an upgraded hash could not be stored, for the caller to log. The password
    /// still verified; the old hash is still in place.
    #[must_use]
    pub fn rehash_error(&self) -> Option<&str> {
        self.rehash_error.as_deref()
    }
}

/// Local-store password change (Argon2id via `mw-store`, plan §2.3).
pub struct Local<S: LocalCredentialStore> {
    store: S,
    policy: PasswordPolicy,
    cost: Option<ArgonCost>,
}

impl<S: LocalCredentialStore> Local<S> {
    #[must_use]
    pub fn new(store: S, policy: PasswordPolicy) -> Self {
        Self {
            store,
            policy,
            cost: None,
        }
    }

    /// Hash with `cost` instead of the process-wide one.
    #[must_use]
    pub fn with_cost(mut self, cost: ArgonCost) -> Self {
        self.cost = Some(cost);
        self
    }

    /// [`ArgonCost::new`] followed by [`set_cost`], for callers that hold the three
    /// numbers.
    ///
    /// # Errors
    /// [`ArgonCostError`] if a parameter is out of range; the process-wide cost is then
    /// left as it was.
    pub fn set_argon_cost(m_kib: u32, t: u32, p: u32) -> std::result::Result<(), ArgonCostError> {
        set_cost(ArgonCost::new(m_kib, t, p)?);
        Ok(())
    }

    /// The process-wide cost as `(m_kib, t, p)`.
    #[must_use]
    pub fn argon_cost() -> (u32, u32, u32) {
        let c = cost();
        (c.m_kib, c.t, c.p)
    }

    /// The `(m_kib, t, p)` written in a stored PHC string — see [`phc_cost`].
    #[must_use]
    pub fn stored_cost(phc: &str) -> Option<(u32, u32, u32)> {
        phc_cost(phc)
    }

    fn current_cost(&self) -> ArgonCost {
        self.cost.unwrap_or_else(cost)
    }

    /// Check `password` against the account's stored hash and, if it is correct and the
    /// hash was made with a different cost, replace the hash with one at the current
    /// cost ([`LocalCredentialStore::replace_hash`]: one write, or none).
    ///
    /// A failed or skipped replacement does not change the answer: the result still says
    /// the password verified, and carries a store error in [`LocalVerify::rehash_error`].
    /// An account with no stored hash does not verify; a throwaway hash is computed
    /// first so that it does not answer faster than a wrong password does.
    ///
    /// # Errors
    /// The store's read error, or [`PasswordError::Protocol`] if the stored value is not
    /// a PHC string.
    pub async fn verify(&self, account_id: &str, password: &Secret) -> Result<LocalVerify> {
        let cost = self.current_cost();
        let mut out = LocalVerify {
            verified: false,
            rehashed: false,
            rehash_error: None,
        };
        let Some(phc) = self.store.current_hash(account_id).await? else {
            let _ = hash_password_with(password.expose(), cost);
            return Ok(out);
        };
        let VerifyOutcome::Match { upgraded } = verify_and_upgrade(password.expose(), &phc, cost)?
        else {
            return Ok(out);
        };
        out.verified = true;
        if let Some(new_phc) = upgraded {
            match self.store.replace_hash(account_id, &phc, &new_phc).await {
                Ok(true) => out.rehashed = true,
                // The hash changed underneath us (a concurrent password change or
                // upgrade); whatever is there now is newer than ours.
                Ok(false) => {}
                Err(e) => out.rehash_error = Some(e.to_string()),
            }
        }
        Ok(out)
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
        let new_phc = hash_password_with(new.expose(), self.current_cost())?;
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

    /// Hash at the process-wide cost, as `change` does for a backend without its own.
    fn hash_password(password: &str) -> Result<String> {
        hash_password_with(password, cost())
    }

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

    // ── t29-e4: configurable cost, rehash on verify ─────────────────────────────

    /// What this backend hashed with before the cost became a setting.
    const OLD: (u32, u32, u32) = (19_456, 2, 1);

    fn raised() -> ArgonCost {
        ArgonCost::new(32_768, 3, 1).unwrap()
    }

    /// A store whose writes can be made to fail, counting the ones that land.
    #[derive(Default)]
    struct CountingStore {
        hash: Mutex<Option<String>>,
        writes: Mutex<u32>,
        fail_writes: bool,
    }
    impl CountingStore {
        fn seeded(phc: String) -> Self {
            Self {
                hash: Mutex::new(Some(phc)),
                ..Self::default()
            }
        }
        fn stored(&self) -> Option<String> {
            self.hash.lock().unwrap().clone()
        }
        fn writes(&self) -> u32 {
            *self.writes.lock().unwrap()
        }
    }
    #[async_trait]
    impl LocalCredentialStore for CountingStore {
        async fn current_hash(&self, _account_id: &str) -> Result<Option<String>> {
            Ok(self.stored())
        }
        async fn set_hash(&self, _account_id: &str, phc: &str) -> Result<()> {
            if self.fail_writes {
                return Err(PasswordError::Transport("disk full".into()));
            }
            *self.hash.lock().unwrap() = Some(phc.to_string());
            *self.writes.lock().unwrap() += 1;
            Ok(())
        }
    }

    #[test]
    fn default_cost_is_the_previous_constants() {
        let d = ArgonCost::DEFAULT;
        assert_eq!((d.m_kib(), d.t(), d.p()), OLD);
        assert_eq!(ArgonCost::default(), d);
        let phc = hash_password_with("pw-123456789", d).unwrap();
        assert_eq!(phc_cost(&phc), Some(OLD));
    }

    #[test]
    fn raising_the_cost_keeps_old_hashes_verifying_and_returns_an_upgrade() {
        let old_phc = hash_password_with("correct horse", ArgonCost::DEFAULT).unwrap();
        // Precondition: at the cost it was made with, it verifies and needs nothing.
        assert_eq!(
            verify_and_upgrade("correct horse", &old_phc, ArgonCost::DEFAULT).unwrap(),
            VerifyOutcome::Match { upgraded: None }
        );

        let out = verify_and_upgrade("correct horse", &old_phc, raised()).unwrap();
        let VerifyOutcome::Match {
            upgraded: Some(new_phc),
        } = out
        else {
            panic!("expected a match with an upgraded hash, got {out:?}");
        };
        assert_eq!(phc_cost(&new_phc), Some((32_768, 3, 1)));
        assert!(new_phc.starts_with("$argon2id$v=19$m=32768,t=3,p=1$"));
        assert_ne!(new_phc, old_phc);
        // The upgraded hash is of the same password, and is itself up to date.
        assert_eq!(
            verify_and_upgrade("correct horse", &new_phc, raised()).unwrap(),
            VerifyOutcome::Match { upgraded: None }
        );
        assert!(!verify_password("wrong horse", &new_phc).unwrap());
    }

    #[test]
    fn wrong_password_yields_no_upgrade() {
        let old_phc = hash_password_with("correct horse", ArgonCost::DEFAULT).unwrap();
        assert_eq!(
            verify_and_upgrade("wrong horse", &old_phc, raised()).unwrap(),
            VerifyOutcome::Mismatch
        );
        // Same answer when the stored hash is already at the requested cost.
        assert_eq!(
            verify_and_upgrade("wrong horse", &old_phc, ArgonCost::DEFAULT).unwrap(),
            VerifyOutcome::Mismatch
        );
    }

    #[test]
    fn lowering_the_cost_rehashes_down_to_the_setting() {
        let strong = hash_password_with("correct horse", raised()).unwrap();
        let out = verify_and_upgrade("correct horse", &strong, ArgonCost::DEFAULT).unwrap();
        let VerifyOutcome::Match {
            upgraded: Some(new_phc),
        } = out
        else {
            panic!("expected a re-hash at the lowered cost, got {out:?}");
        };
        assert_eq!(phc_cost(&new_phc), Some(OLD));
        assert!(verify_password("correct horse", &new_phc).unwrap());
    }

    #[test]
    fn a_hash_of_another_argon2_variant_is_rehashed_as_argon2id() {
        let salt = gen_salt().unwrap();
        let params = Params::new(OLD.0, OLD.1, OLD.2, None).unwrap();
        let argon2i = Argon2::new(Algorithm::Argon2i, Version::V0x13, params)
            .hash_password(b"correct horse", &salt)
            .unwrap()
            .to_string();
        assert!(argon2i.starts_with("$argon2i$"));
        let out = verify_and_upgrade("correct horse", &argon2i, ArgonCost::DEFAULT).unwrap();
        let VerifyOutcome::Match {
            upgraded: Some(new_phc),
        } = out
        else {
            panic!("expected a re-hash as argon2id, got {out:?}");
        };
        assert!(new_phc.starts_with("$argon2id$v=19$"));
    }

    #[test]
    fn out_of_bounds_cost_is_rejected() {
        // Precondition: the edges themselves are accepted.
        assert!(ArgonCost::new(19_456, 2, 1).is_ok());
        assert!(ArgonCost::new(1_048_576, 10, 16).is_ok());
        assert!(ArgonCost::new(47_104, 1, 1).is_ok());

        assert!(matches!(
            ArgonCost::new(19_455, 2, 1),
            Err(ArgonCostError::Memory { got: 19_455, .. })
        ));
        assert!(matches!(
            ArgonCost::new(8, 2, 1),
            Err(ArgonCostError::Memory { .. })
        ));
        assert!(matches!(
            ArgonCost::new(1_048_577, 2, 1),
            Err(ArgonCostError::Memory { .. })
        ));
        assert!(matches!(
            ArgonCost::new(u32::MAX, 2, 1),
            Err(ArgonCostError::Memory { .. })
        ));
        assert!(matches!(
            ArgonCost::new(19_456, 0, 1),
            Err(ArgonCostError::Passes { got: 0, .. })
        ));
        assert!(matches!(
            ArgonCost::new(19_456, 11, 1),
            Err(ArgonCostError::Passes { got: 11, .. })
        ));
        assert!(matches!(
            ArgonCost::new(19_456, 1, 1),
            Err(ArgonCostError::SinglePassMemory { .. })
        ));
        assert!(matches!(
            ArgonCost::new(47_103, 1, 1),
            Err(ArgonCostError::SinglePassMemory { .. })
        ));
        assert!(matches!(
            ArgonCost::new(19_456, 2, 0),
            Err(ArgonCostError::Parallelism { got: 0, .. })
        ));
        assert!(matches!(
            ArgonCost::new(19_456, 2, 17),
            Err(ArgonCostError::Parallelism { got: 17, .. })
        ));
    }

    #[tokio::test]
    async fn local_verify_upgrades_the_stored_hash_once() {
        let old_phc = hash_password_with("old-password-12", ArgonCost::DEFAULT).unwrap();
        let backend = Local::new(
            CountingStore::seeded(old_phc.clone()),
            PasswordPolicy::default(),
        )
        .with_cost(raised());

        // A wrong password neither verifies nor writes.
        let out = backend
            .verify("a1", &Secret::new("wrong-guess-12"))
            .await
            .unwrap();
        assert!(!out.verified() && !out.rehashed() && out.rehash_error().is_none());
        assert_eq!(backend.store.writes(), 0);
        assert_eq!(backend.store.stored(), Some(old_phc));

        // The right one verifies and the stored hash now carries the raised cost.
        let out = backend
            .verify("a1", &Secret::new("old-password-12"))
            .await
            .unwrap();
        assert!(out.verified() && out.rehashed());
        let stored = backend.store.stored().unwrap();
        assert_eq!(phc_cost(&stored), Some((32_768, 3, 1)));
        assert!(verify_password("old-password-12", &stored).unwrap());
        assert_eq!(backend.store.writes(), 1);

        // A second sign-in finds it up to date: no further write.
        let out = backend
            .verify("a1", &Secret::new("old-password-12"))
            .await
            .unwrap();
        assert!(out.verified() && !out.rehashed());
        assert_eq!(backend.store.writes(), 1);
    }

    #[tokio::test]
    async fn local_verify_survives_a_failed_upgrade_write() {
        let old_phc = hash_password_with("old-password-12", ArgonCost::DEFAULT).unwrap();
        let store = CountingStore {
            fail_writes: true,
            ..CountingStore::seeded(old_phc.clone())
        };
        let backend = Local::new(store, PasswordPolicy::default()).with_cost(raised());
        let out = backend
            .verify("a1", &Secret::new("old-password-12"))
            .await
            .unwrap();
        assert!(out.verified());
        assert!(!out.rehashed());
        assert!(out.rehash_error().unwrap().contains("disk full"));
        // Not at all rather than partly: the old hash is intact.
        assert_eq!(backend.store.stored(), Some(old_phc));
    }

    #[tokio::test]
    async fn replace_hash_does_not_overwrite_a_hash_that_changed() {
        let store = CountingStore::seeded("current".into());
        assert!(!store.replace_hash("a1", "stale", "new").await.unwrap());
        assert_eq!(store.stored().as_deref(), Some("current"));
        assert!(store.replace_hash("a1", "current", "new").await.unwrap());
        assert_eq!(store.stored().as_deref(), Some("new"));
    }

    #[tokio::test]
    async fn local_verify_of_an_account_with_no_hash_is_false() {
        let backend = Local::new(CountingStore::default(), PasswordPolicy::default());
        let out = backend
            .verify("a1", &Secret::new("anything-12"))
            .await
            .unwrap();
        assert!(!out.verified() && !out.rehashed());
        assert_eq!(backend.store.writes(), 0);
    }

    #[tokio::test]
    async fn change_hashes_the_new_password_at_the_backend_cost() {
        let old_phc = hash_password_with("old-password-12", ArgonCost::DEFAULT).unwrap();
        let backend = Local::new(CountingStore::seeded(old_phc), PasswordPolicy::default())
            .with_cost(raised());
        backend
            .change(
                &Ctx::new("a1", "u"),
                Secret::new("old-password-12"),
                Secret::new("brand-new-password"),
            )
            .await
            .unwrap();
        let stored = backend.store.stored().unwrap();
        assert_eq!(phc_cost(&stored), Some((32_768, 3, 1)));
        assert!(verify_password("brand-new-password", &stored).unwrap());
    }

    /// The only test that writes the process-wide cost; it puts the default back. The
    /// other tests pass an explicit cost or only verify, so the order does not matter.
    #[tokio::test]
    async fn process_wide_cost_drives_a_backend_without_its_own() {
        type L = Local<CountingStore>;
        assert_eq!(L::argon_cost(), OLD);
        // An out-of-range setting is refused and leaves the cost as it was.
        assert!(L::set_argon_cost(1024, 2, 1).is_err());
        assert_eq!(L::argon_cost(), OLD);

        let old_phc = hash_password("old-password-12").unwrap();
        assert_eq!(L::stored_cost(&old_phc), Some(OLD));
        let backend = Local::new(CountingStore::seeded(old_phc), PasswordPolicy::default());

        L::set_argon_cost(32_768, 3, 1).unwrap();
        let out = backend.verify("a1", &Secret::new("old-password-12")).await;
        set_cost(ArgonCost::DEFAULT);

        let out = out.unwrap();
        assert!(out.verified() && out.rehashed());
        let stored = backend.store.stored().unwrap();
        assert_eq!(L::stored_cost(&stored), Some((32_768, 3, 1)));
        assert_eq!(L::argon_cost(), OLD);
    }
}
