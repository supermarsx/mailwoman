//! `mw-mfa` — Mailwoman login second-factor primitives.
//!
//! Pure logic, no I/O: this crate verifies WebAuthn ceremonies, computes/verifies
//! RFC 6238 TOTP codes, and generates/verifies Argon2id-hashed recovery codes. The
//! store and login wiring live in `mw-server` (t16-e3); this crate is deliberately
//! self-contained and heavily unit-tested.
//!
//! ## WebAuthn scope
//! Relying-party (server) verification for **attestation `"none"` only**, which is
//! what Mailwoman requests. Registration decodes and stores the COSE public key;
//! assertion verifies the signature over `authData ‖ SHA256(clientDataJSON)`.
//! Algorithms: **ES256** (ECDSA/P-256/SHA-256, ASN.1 DER signature) and **EdDSA**
//! (Ed25519). RS256 is deferred.
//!
//! The verification is hand-rolled over already-vendored RustCrypto primitives
//! (`p256`, `ed25519-dalek`, `sha2`) with a small in-crate CBOR reader, because the
//! only stable relying-party WebAuthn crate (`webauthn-rs`) pulls `openssl`, which
//! the dependency floor bans. The whole 2FA lane's one net-new crate is `sha1`.

mod cbor;
mod cose;
pub mod recovery;
pub mod totp;
pub mod webauthn;

pub use cose::{CoseAlg, CoseKey};
pub use recovery::{DEFAULT_RECOVERY_CODES, RecoveryCode};
pub use totp::TotpParams;
pub use webauthn::{
    AssertionOutcome, AssertionRequest, RegisteredCredential, RegistrationRequest,
    UserVerification, verify_assertion, verify_registration,
};

/// Errors from any second-factor operation.
#[derive(Debug, thiserror::Error)]
pub enum MfaError {
    /// The CBOR in an attestationObject or COSE_Key was malformed.
    #[error("cbor decode: {0}")]
    Cbor(String),
    /// The COSE_Key was structurally invalid for a supported algorithm.
    #[error("cose key: {0}")]
    Cose(String),
    /// `clientDataJSON` was missing, malformed, or had a wrong `type`.
    #[error("clientDataJSON: {0}")]
    ClientData(String),
    /// The attestationObject was invalid (wrong `fmt`, missing fields, …).
    #[error("attestation: {0}")]
    Attestation(String),
    /// The signature (or the key it verifies under) did not check out.
    #[error("assertion verification failed: {0}")]
    Verify(String),
    /// The client's challenge did not match the server-issued challenge.
    #[error("challenge mismatch")]
    Challenge,
    /// The client's origin did not match the expected origin.
    #[error("origin mismatch")]
    Origin,
    /// `SHA256(rpId)` did not match `authData.rpIdHash`.
    #[error("rpId hash mismatch")]
    RpIdHash,
    /// The User-Present flag was not set.
    #[error("user presence flag not set")]
    UserPresence,
    /// User verification was required but the UV flag was not set.
    #[error("user verification required but flag not set")]
    UserVerification,
    /// The signature counter went backwards (a possible cloned authenticator).
    #[error("signature count regression (possible cloned authenticator)")]
    CounterRegression,
    /// The COSE algorithm is not one this crate verifies (only ES256 / EdDSA).
    #[error("unsupported COSE algorithm")]
    UnsupportedAlg,
    /// A byte buffer was too short or otherwise malformed.
    #[error("malformed input: {0}")]
    Malformed(String),
}

/// Constant-time byte-slice equality (no early return on the first mismatch).
/// Unequal lengths short-circuit — length is not a secret here.
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

#[cfg(test)]
mod tests;
