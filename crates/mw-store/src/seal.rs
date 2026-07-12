//! Authenticated encryption for data at rest (XChaCha20-Poly1305).

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use rand::RngCore;

pub const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 24;

#[derive(Debug, thiserror::Error)]
pub enum SealError {
    #[error("ciphertext too short")]
    Truncated,
    #[error("decryption failed (wrong key or tampered data)")]
    Decrypt,
    #[error("invalid key length")]
    KeyLength,
}

/// Server-held symmetric key used to seal upstream credentials.
#[derive(Clone)]
pub struct ServerKey([u8; KEY_LEN]);

impl std::fmt::Debug for ServerKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ServerKey(<redacted>)")
    }
}

impl ServerKey {
    pub fn generate() -> Self {
        let mut k = [0u8; KEY_LEN];
        rand::thread_rng().fill_bytes(&mut k);
        Self(k)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SealError> {
        if bytes.len() != KEY_LEN {
            return Err(SealError::KeyLength);
        }
        let mut k = [0u8; KEY_LEN];
        k.copy_from_slice(bytes);
        Ok(Self(k))
    }

    /// Parse a hex-encoded 32-byte key (e.g. from `MW_SERVER_KEY`).
    pub fn from_hex(s: &str) -> Result<Self, SealError> {
        let bytes = hex_decode(s.trim()).ok_or(SealError::KeyLength)?;
        Self::from_bytes(&bytes)
    }

    /// Hex-encode the key (e.g. to log a freshly generated `MW_SERVER_KEY`).
    pub fn to_hex(&self) -> String {
        hex_encode(&self.0)
    }

    /// Seal plaintext: output = nonce(24) || ciphertext+tag.
    pub fn seal(&self, plaintext: &[u8]) -> Result<Vec<u8>, SealError> {
        let cipher = XChaCha20Poly1305::new(&self.0.into());
        let mut nonce_bytes = [0u8; NONCE_LEN];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = XNonce::from_slice(&nonce_bytes);
        let ct = cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| SealError::Decrypt)?;
        let mut out = Vec::with_capacity(NONCE_LEN + ct.len());
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ct);
        Ok(out)
    }

    /// Open sealed bytes produced by [`seal`].
    pub fn open(&self, sealed: &[u8]) -> Result<Vec<u8>, SealError> {
        if sealed.len() < NONCE_LEN {
            return Err(SealError::Truncated);
        }
        let (nonce_bytes, ct) = sealed.split_at(NONCE_LEN);
        let cipher = XChaCha20Poly1305::new(&self.0.into());
        let nonce = XNonce::from_slice(nonce_bytes);
        cipher.decrypt(nonce, ct).map_err(|_| SealError::Decrypt)
    }
}

pub fn random_token() -> String {
    let mut b = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut b);
    hex_encode(&b)
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seal_open_round_trip() {
        let key = ServerKey::generate();
        let sealed = key.seal(b"hello world").unwrap();
        assert_eq!(key.open(&sealed).unwrap(), b"hello world");
    }

    #[test]
    fn different_nonces_each_time() {
        let key = ServerKey::generate();
        assert_ne!(key.seal(b"x").unwrap(), key.seal(b"x").unwrap());
    }

    #[test]
    fn wrong_key_fails() {
        let sealed = ServerKey::generate().seal(b"secret").unwrap();
        assert!(matches!(
            ServerKey::generate().open(&sealed),
            Err(SealError::Decrypt)
        ));
    }

    #[test]
    fn tamper_detected() {
        let key = ServerKey::generate();
        let mut sealed = key.seal(b"secret").unwrap();
        let last = sealed.len() - 1;
        sealed[last] ^= 0xff;
        assert!(key.open(&sealed).is_err());
    }

    #[test]
    fn hex_key_round_trip() {
        let key = ServerKey::generate();
        let hex = hex_encode(&key.0);
        let restored = ServerKey::from_hex(&hex).unwrap();
        let sealed = key.seal(b"data").unwrap();
        assert_eq!(restored.open(&sealed).unwrap(), b"data");
    }
}
