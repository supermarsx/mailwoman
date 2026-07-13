//! Small crypto/encoding helpers shared across the crate.

use base64::Engine;
use rand::RngCore;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

/// Fill an `N`-byte array from the OS CSPRNG.
pub(crate) fn random_bytes<const N: usize>() -> [u8; N] {
    let mut b = [0u8; N];
    OsRng.fill_bytes(&mut b);
    b
}

/// URL-safe, unpadded base64 (RFC 4648 §5) — the PKCE / token wire encoding.
pub(crate) fn b64url(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// Hex-encoded SHA-256 of `input` — the at-rest hash for opaque OAuth tokens
/// (auth codes / access / refresh). Tokens are high-entropy 256-bit randoms, so a
/// single SHA-256 (not a slow KDF) is the correct at-rest transform: it is a
/// lookup key, not a low-entropy password.
pub(crate) fn sha256_hex(input: &str) -> String {
    hex::encode(Sha256::digest(input.as_bytes()))
}

/// base64url(SHA-256(bytes)) — the PKCE S256 challenge transform.
pub(crate) fn sha256_b64url(input: &[u8]) -> String {
    b64url(&Sha256::digest(input))
}

/// Constant-time byte-slice equality (no early return on the first mismatch).
/// Length is compared first; unequal lengths short-circuit (length is not secret).
pub(crate) fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
