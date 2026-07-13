//! Admin/local password hashing (plan §2.5, §5): Argon2id with OWASP-recommended
//! parameters. Admin panel default auth is passkey-capable; when a password is
//! set (`admin_users.password_hash`) it is stored as an Argon2id PHC string —
//! never plaintext.

use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Algorithm, Argon2, Params, Version};

use crate::{AdminError, SecurityPolicy};

/// Generate a fresh 16-byte Argon2id salt from the OS CSPRNG.
fn gen_salt() -> Result<SaltString, AdminError> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|e| AdminError::Config(format!("csprng: {e}")))?;
    SaltString::encode_b64(&bytes).map_err(|e| AdminError::Config(format!("salt encode: {e}")))
}

/// Build an Argon2id hasher from the policy's Argon2 parameters (m/t/p cost).
fn hasher(policy: &SecurityPolicy) -> Result<Argon2<'static>, AdminError> {
    let params = Params::new(
        policy.argon2_m_cost,
        policy.argon2_t_cost,
        policy.argon2_p_cost,
        None,
    )
    .map_err(|e| AdminError::Config(format!("argon2 params: {e}")))?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

/// Hash a password with Argon2id using the policy's parameters. Returns a PHC
/// string (`$argon2id$...`) suitable for `admin_users.password_hash`.
pub fn hash_password(password: &str, policy: &SecurityPolicy) -> Result<String, AdminError> {
    let salt = gen_salt()?;
    let hash = hasher(policy)?
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AdminError::Config(format!("argon2 hash: {e}")))?;
    Ok(hash.to_string())
}

/// Verify a password against a stored Argon2id PHC string.
pub fn verify_password(password: &str, phc: &str) -> Result<bool, AdminError> {
    let parsed =
        PasswordHash::new(phc).map_err(|e| AdminError::Config(format!("argon2 phc: {e}")))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_verify_round_trip() {
        let policy = SecurityPolicy::default();
        let phc = hash_password("correct horse battery staple", &policy).unwrap();
        assert!(phc.starts_with("$argon2id$"));
        assert!(verify_password("correct horse battery staple", &phc).unwrap());
        assert!(!verify_password("wrong", &phc).unwrap());
    }

    #[test]
    fn hash_is_not_plaintext() {
        let phc = hash_password("s3cr3t-value", &SecurityPolicy::default()).unwrap();
        assert!(!phc.contains("s3cr3t-value"));
    }
}
