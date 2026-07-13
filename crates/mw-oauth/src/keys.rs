//! Opaque scoped API keys: `mwk_<prefix>.<secret>` (SPEC §20.1, plan §2.3).
//!
//! - `prefix` is a public, indexable handle (stored plaintext for O(1) lookup).
//! - `secret` is a 256-bit random shown exactly once; only its **Argon2id** hash
//!   is persisted. Verification is Argon2id's constant-time compare.

use argon2::Argon2;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use chrono::Utc;
use rand::rngs::OsRng;

use crate::util::{b64url, ct_eq, random_bytes};
use crate::{ApiKey, MintedApiKey, Scope};

/// Wire scheme prefix.
pub const KEY_SCHEME: &str = "mwk_";
/// Bytes of randomness in the public prefix (→ 16 hex chars).
const PREFIX_BYTES: usize = 8;
/// Bytes of randomness in the secret half (256-bit).
const SECRET_BYTES: usize = 32;

/// Mint a fresh `mwk_<prefix>.<secret>` key for `account_id` under `scope`.
///
/// Returns the shown-once display token plus the storable [`ApiKey`] record whose
/// `hash` is the Argon2id digest of the secret half. The plaintext secret is never
/// retained by the record.
pub fn mint(account_id: &str, scope: Scope) -> MintedApiKey {
    let prefix = hex::encode(random_bytes::<PREFIX_BYTES>());
    let secret = b64url(&random_bytes::<SECRET_BYTES>());
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(secret.as_bytes(), &salt)
        .expect("argon2id hashing with default (static) params cannot fail")
        .to_string();
    let display_token = format!("{KEY_SCHEME}{prefix}.{secret}");
    let record = ApiKey {
        prefix,
        hash,
        account_id: account_id.to_string(),
        scope,
        created_at: Utc::now().to_rfc3339(),
        last_used_at: None,
        revoked_at: None,
    };
    MintedApiKey {
        display_token,
        record,
    }
}

/// Split a presented `mwk_<prefix>.<secret>` into `(prefix, secret)`.
///
/// Returns `None` on any malformed token (wrong scheme, missing separator, empty
/// half) — callers treat that as an auth failure.
pub fn parse(token: &str) -> Option<(String, String)> {
    let rest = token.strip_prefix(KEY_SCHEME)?;
    let (prefix, secret) = rest.split_once('.')?;
    if prefix.is_empty() || secret.is_empty() {
        return None;
    }
    Some((prefix.to_string(), secret.to_string()))
}

/// Verify a presented `mwk_...` token against a stored [`ApiKey`].
///
/// Rejects revoked keys, prefix mismatches, and bad secrets. The secret check is
/// Argon2id's constant-time verification; the prefix check is also constant-time.
pub fn verify(presented: &str, stored: &ApiKey) -> bool {
    if stored.revoked_at.is_some() {
        return false;
    }
    let Some((prefix, secret)) = parse(presented) else {
        return false;
    };
    if !ct_eq(prefix.as_bytes(), stored.prefix.as_bytes()) {
        return false;
    }
    let Ok(parsed) = PasswordHash::new(&stored.hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(secret.as_bytes(), &parsed)
        .is_ok()
}
