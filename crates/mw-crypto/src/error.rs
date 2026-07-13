//! The crate-wide error type. Every fallible `mw-crypto` operation returns
//! [`CryptoError`]; the WASM boundary maps it to a JS `Error` string and the
//! native engine surfaces it as a verdict/`notCreated` reason. Kept deliberately
//! coarse — the crypto libraries' internal error detail is not part of the frozen
//! contract and must not leak into the wire shapes (plan §1.5).

use thiserror::Error;

/// A crypto operation failure.
#[derive(Debug, Error)]
pub enum CryptoError {
    /// A key, cert, message, or backup bundle could not be parsed.
    #[error("parse error: {0}")]
    Parse(String),
    /// Key generation failed.
    #[error("key generation failed: {0}")]
    KeyGen(String),
    /// Encryption failed.
    #[error("encryption failed: {0}")]
    Encrypt(String),
    /// Decryption failed (bad passphrase, wrong recipient, corrupt ciphertext).
    #[error("decryption failed: {0}")]
    Decrypt(String),
    /// Signing failed.
    #[error("signing failed: {0}")]
    Sign(String),
    /// A required input was missing or malformed.
    #[error("invalid input: {0}")]
    Input(String),
    /// PKCS#12 import failed.
    #[error("pkcs12 import failed: {0}")]
    Pkcs12(String),
    /// A PQC key-wrap/unwrap failure (native only).
    #[error("key-wrap failed: {0}")]
    KeyWrap(String),
    /// A network/IO failure (WKD lookup; native only).
    #[error("io error: {0}")]
    Io(String),
}

/// The crate result alias.
pub type Result<T> = std::result::Result<T, CryptoError>;
