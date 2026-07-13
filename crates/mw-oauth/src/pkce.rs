//! PKCE (RFC 7636) — Mailwoman mandates the **S256** method; `plain` is refused.

use crate::util::{ct_eq, sha256_b64url};

/// Compute the S256 challenge for a `code_verifier`:
/// `BASE64URL-NOPAD(SHA256(ASCII(verifier)))`.
pub fn challenge_s256(verifier: &str) -> String {
    sha256_b64url(verifier.as_bytes())
}

/// Verify a `code_verifier` against a stored S256 `challenge` in constant time.
pub fn verify_s256(verifier: &str, challenge: &str) -> bool {
    ct_eq(challenge_s256(verifier).as_bytes(), challenge.as_bytes())
}
