//! The native (server/engine) crypto surface (plan §1.2) — the PUBLIC-key
//! operations that need no secret: PGP/S/MIME *signature* verification, cert
//! harvesting from received signed mail, and the PQC hybrid wrap of the
//! `mw-store` seal key at rest. `mw-engine`'s `security/` module calls these from
//! `SecurityVerdict/get`, `CryptoKey/lookup`, and the store-seal path.
//!
//! This is a thin facade over [`crate::pgp`], [`crate::smime`], and [`crate::pqc`];
//! the engine may also call those modules directly. NO private-key operations live
//! here (those are the wasm/browser surface, plan §1.2). Non-fatal verification
//! failures are surfaced as a verdict rather than an error, so the engine always
//! gets a renderable 3-state result.

use crate::error::Result;
use crate::types::{CryptoKey, SignatureVerdict};
use crate::{pgp, smime};

fn none_verdict(kind: &str) -> SignatureVerdict {
    SignatureVerdict {
        kind: kind.into(),
        status: "none".into(),
        signer_key_id: None,
        algorithm: None,
        key_created_at: None,
        key_expires_at: None,
        chain_status: None,
        revocation_status: None,
        key_changed: false,
    }
}

/// Verify a PGP or S/MIME signature against a public key/cert → the frozen 3-state
/// [`SignatureVerdict`] (§2.1). All public; no secret material.
///
/// - `kind = "pgp"`: `signature` is an armored detached signature, verified over
///   `data` against the armored `signer_public_key`.
/// - `kind = "smime"`: `signature` is the DER CMS SignedData (self-contained; the
///   signer cert is embedded, so `data`/`signer_public_key` are ignored).
pub fn verify_signature(
    kind: &str,
    data: &[u8],
    signature: &[u8],
    signer_public_key: &str,
) -> SignatureVerdict {
    match kind {
        "pgp" => match std::str::from_utf8(signature) {
            Ok(sig) => pgp::verify_detached(data, sig, signer_public_key)
                .unwrap_or_else(|_| none_verdict("pgp")),
            Err(_) => none_verdict("pgp"),
        },
        "smime" => smime::verify(signature).unwrap_or_else(|_| none_verdict("smime")),
        _ => none_verdict(kind),
    }
}

/// Harvest sender certificates/keys from received signed mail for the keyring.
///
/// - S/MIME: `cms_or_autocrypt` is the DER CMS SignedData (extracted from the
///   `smime.p7s` / multipart-signed part) → embedded certs.
/// - PGP Autocrypt: pass the `Autocrypt:` header *value* and it is parsed as a key
///   (the engine chooses which path from the message's content type / headers).
pub fn harvest_keys(cms_or_autocrypt: &[u8]) -> Vec<CryptoKey> {
    // Try S/MIME CMS first; fall back to an Autocrypt header value.
    if let Ok(certs) = smime::harvest_certs(cms_or_autocrypt)
        && !certs.is_empty()
    {
        return certs;
    }
    if let Ok(header) = std::str::from_utf8(cms_or_autocrypt)
        && let Ok(key) = pgp::parse_autocrypt_header(header)
    {
        return vec![key];
    }
    Vec::new()
}

/// Hybrid X25519 + ML-KEM-768 key-wrap of the `mw-store` seal master key at rest
/// (plan §1.7). `recipient_public` is a [`crate::pqc::HybridKeyPair::public`] blob.
pub fn wrap_store_key(seal_key: &[u8], recipient_public: &[u8]) -> Result<Vec<u8>> {
    crate::pqc::wrap(seal_key, recipient_public)
}

/// Unwrap a [`wrap_store_key`] blob back to the seal master key. `recipient_secret`
/// is a [`crate::pqc::HybridKeyPair::secret`] blob.
pub fn unwrap_store_key(wrapped: &[u8], recipient_secret: &[u8]) -> Result<Vec<u8>> {
    crate::pqc::unwrap(wrapped, recipient_secret)
}

/// Generate a fresh hybrid recipient key pair for store-seal wrapping.
pub fn generate_store_recipient() -> crate::pqc::HybridKeyPair {
    crate::pqc::generate_recipient()
}
