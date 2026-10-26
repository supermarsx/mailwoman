//! Authenticated encryption for data at rest (XChaCha20-Poly1305).

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
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

    /// Hex-encode the key for explicit secure persistence. Never log this value.
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

    /// Versioned encryption bound to an exact purpose and ordered authority fields.
    /// Length prefixes prevent ambiguous concatenations. This format deliberately
    /// cannot be opened by the legacy, unbound credential decoder.
    pub fn seal_bound(
        &self,
        plaintext: &[u8],
        purpose: &str,
        authority: &[&str],
    ) -> Result<Vec<u8>, SealError> {
        let aad = bound_context(purpose, authority);
        let mut nonce = [0u8; NONCE_LEN];
        rand::thread_rng().fill_bytes(&mut nonce);
        let ciphertext = XChaCha20Poly1305::new(&self.0.into())
            .encrypt(
                XNonce::from_slice(&nonce),
                Payload {
                    msg: plaintext,
                    aad: &aad,
                },
            )
            .map_err(|_| SealError::Decrypt)?;
        let mut output = b"MWB1".to_vec();
        output.extend_from_slice(&nonce);
        output.extend_from_slice(&ciphertext);
        Ok(output)
    }

    pub fn open_bound(
        &self,
        sealed: &[u8],
        purpose: &str,
        authority: &[&str],
    ) -> Result<Vec<u8>, SealError> {
        if sealed.len() < 4 + NONCE_LEN + 16 {
            return Err(SealError::Truncated);
        }
        if &sealed[..4] != b"MWB1" {
            return Err(SealError::Decrypt);
        }
        let aad = bound_context(purpose, authority);
        XChaCha20Poly1305::new(&self.0.into())
            .decrypt(
                XNonce::from_slice(&sealed[4..4 + NONCE_LEN]),
                Payload {
                    msg: &sealed[4 + NONCE_LEN..],
                    aad: &aad,
                },
            )
            .map_err(|_| SealError::Decrypt)
    }
}

fn bound_context(purpose: &str, authority: &[&str]) -> Vec<u8> {
    let mut output = b"mailwoman-bound-secret-v1".to_vec();
    for field in std::iter::once(purpose).chain(authority.iter().copied()) {
        output.extend_from_slice(&(field.len() as u64).to_be_bytes());
        output.extend_from_slice(field.as_bytes());
    }
    output
}

impl ServerKey {
    /// A 32-byte key for one purpose, derived from this key:
    /// `HMAC-SHA256(server key, "mailwoman-subkey:" ‖ label)` (t32 e31a).
    ///
    /// For keys the server needs besides the sealing key — the remote-content URL
    /// signing key is `derive("mw-remote-content-v1")`. Different labels give
    /// independent keys, and none reveals the server key or another label's key.
    /// The label is a constant chosen in code, never request data.
    pub fn derive(&self, label: &str) -> [u8; 32] {
        let mut msg = b"mailwoman-subkey:".to_vec();
        msg.extend_from_slice(label.as_bytes());
        hmac_sha256(&self.0, &msg)
    }
}

/// HMAC-SHA256 (RFC 2104) over `sha2`, which this crate already links; `mw-store`
/// has no `hmac` dependency and this is the only MAC it needs. Checked against the
/// RFC 4231 vectors below.
pub(crate) fn hmac_sha256(key: &[u8], msg: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    const BLOCK: usize = 64;
    let mut k = [0u8; BLOCK];
    if key.len() > BLOCK {
        k[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        k[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0x36u8; BLOCK];
    let mut opad = [0x5cu8; BLOCK];
    for i in 0..BLOCK {
        ipad[i] ^= k[i];
        opad[i] ^= k[i];
    }
    let inner = Sha256::new()
        .chain_update(ipad)
        .chain_update(msg)
        .finalize();
    Sha256::new()
        .chain_update(opad)
        .chain_update(inner)
        .finalize()
        .into()
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
    fn bound_secrets_refuse_owner_link_purpose_and_format_swaps() {
        let key = ServerKey::generate();
        let context = ["owner", "link", "authority"];
        let sealed = key
            .seal_bound(b"credential", "child-account", &context)
            .unwrap();
        assert_eq!(
            key.open_bound(&sealed, "child-account", &context).unwrap(),
            b"credential"
        );
        for context in [
            ["other", "link", "authority"],
            ["owner", "other", "authority"],
            ["owner", "link", "other"],
        ] {
            assert!(key.open_bound(&sealed, "child-account", &context).is_err());
        }
        assert!(key.open_bound(&sealed, "oidc-key", &context).is_err());
        assert!(key.open(&sealed).is_err());
        assert!(
            key.open_bound(&key.seal(b"credential").unwrap(), "child-account", &context)
                .is_err()
        );
        assert_ne!(bound_context("a", &["bc"]), bound_context("ab", &["c"]));
    }

    #[test]
    fn hmac_matches_rfc_4231() {
        // Test case 2: key "Jefe", data "what do ya want for nothing?".
        assert_eq!(
            hex_encode(&hmac_sha256(b"Jefe", b"what do ya want for nothing?")),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
        // Test case 6: a 131-byte key, longer than the block, is hashed first.
        assert_eq!(
            hex_encode(&hmac_sha256(
                &[0xaa; 131],
                b"Test Using Larger Than Block-Size Key - Hash Key First"
            )),
            "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54"
        );
    }

    #[test]
    fn derived_keys_are_stable_per_label_and_independent_across_labels_and_keys() {
        let key = ServerKey::from_hex(&"11".repeat(32)).unwrap();
        let a = key.derive("mw-remote-content-v1");
        assert_eq!(a, key.derive("mw-remote-content-v1"));
        assert_ne!(a, key.derive("mw-remote-content-v2"));
        assert_ne!(a, ServerKey::generate().derive("mw-remote-content-v1"));
        assert_ne!(&a[..], &key.0[..], "a derived key is not the server key");
    }

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
