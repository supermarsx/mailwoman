//! The native (server/engine) crypto surface (plan §1.2) — the PUBLIC-key
//! operations that need no secret: PGP/S/MIME *signature* verification, cert
//! harvesting from received signed mail, and the PQC hybrid wrap of the
//! `mw-store` seal key at rest. `mw-engine`'s `security/` module calls these from
//! `SecurityVerdict/get`, `CryptoKey/lookup`, and the store-seal path.
//!
//! e0 skeleton — the exact seam + `todo!()` bodies. e1 fills them with rPGP /
//! RustCrypto (`cms`/`x509-cert`/`rsa`/`p256`) / `ml-kem`. NO private-key
//! operations live here (those are the wasm/browser surface, plan §1.2).

use crate::types::{CryptoKey, SignatureVerdict};

/// Verify a PGP or S/MIME signature against a public key/cert → the frozen
/// 3-state [`SignatureVerdict`] (§2.1). All public; no secret material.
pub fn verify_signature(
    _kind: &str,
    _data: &[u8],
    _signature: &[u8],
    _signer_public_key: &str,
) -> SignatureVerdict {
    todo!("e1: rPGP / cms signature verify → SignatureVerdict")
}

/// Harvest sender certificates/keys from a received signed message (S/MIME cert
/// harvesting + PGP key/Autocrypt-header extraction), for the keyring.
pub fn harvest_keys(_raw_message: &[u8]) -> Vec<CryptoKey> {
    todo!("e1: extract S/MIME certs + PGP keys / Autocrypt headers from signed mail")
}

/// Hybrid X25519 + ML-KEM-768 key-wrap of the `mw-store` seal master key at rest
/// (plan §1.7 — PQC crypto-agility groundwork; UNAUDITED ml-kem, store-key scope
/// only, NOT a user-facing security claim). Returns the wrapped material.
pub fn wrap_store_key(_seal_key: &[u8], _recipient_public: &[u8]) -> Vec<u8> {
    todo!("e1: hybrid X25519+ML-KEM-768 wrap of the store seal key")
}

/// Unwrap a [`wrap_store_key`] blob back to the seal master key.
pub fn unwrap_store_key(_wrapped: &[u8], _recipient_secret: &[u8]) -> Vec<u8> {
    todo!("e1: hybrid X25519+ML-KEM-768 unwrap of the store seal key")
}
