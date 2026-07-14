//! Signed-registry verification (plan §2.1, SPEC §7.5).
//!
//! A plugin component is trusted only if a **detached Ed25519 signature over the
//! exact component bytes** verifies against a key in the configured [`TrustRoot`].
//! Unsigned components load **only** under an explicit `allow_unsigned` admin
//! policy, which flags the loaded handle so the UI/`doctor` can raise a persistent
//! banner and the host can emit an audit signal (`§7.5`: unsigned ⇒ never silent).

use ed25519_dalek::{Signature, Verifier, VerifyingKey};

use crate::{PluginError, Result};

/// The set of public keys a deployment trusts to sign plugin components.
///
/// Empty ⇒ **no** signed component can verify (every signed load fails closed);
/// unsigned components still require `allow_unsigned`. The admin seeds this from a
/// config path (plan §2.1); the registry persists the trust decision (0008).
#[derive(Clone, Default)]
pub struct TrustRoot {
    keys: Vec<VerifyingKey>,
}

impl TrustRoot {
    /// An empty trust root (fails closed for signed components).
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Build a trust root from raw 32-byte Ed25519 public keys.
    pub fn from_public_keys(keys: &[[u8; 32]]) -> Result<Self> {
        let mut out = Vec::with_capacity(keys.len());
        for k in keys {
            let vk = VerifyingKey::from_bytes(k)
                .map_err(|e| PluginError::Manifest(format!("bad trust-root key: {e}")))?;
            out.push(vk);
        }
        Ok(Self { keys: out })
    }

    /// Number of trusted keys.
    #[must_use]
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Whether the trust root holds no keys.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Verify a hex-encoded detached signature over `component_bytes` against any
    /// trusted key. Returns `Ok(())` on the first key that verifies.
    pub fn verify(&self, component_bytes: &[u8], signature_hex: &str) -> Result<()> {
        let raw = decode_hex(signature_hex)
            .ok_or_else(|| PluginError::SignatureInvalid("signature is not valid hex".into()))?;
        let sig_bytes: [u8; 64] = raw.as_slice().try_into().map_err(|_| {
            PluginError::SignatureInvalid(format!("signature must be 64 bytes, got {}", raw.len()))
        })?;
        let sig = Signature::from_bytes(&sig_bytes);
        for vk in &self.keys {
            if vk.verify(component_bytes, &sig).is_ok() {
                return Ok(());
            }
        }
        Err(PluginError::SignatureInvalid(
            "no trusted key verifies the component signature".into(),
        ))
    }
}

/// The trust decision reached for a specific load.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureStatus {
    /// A detached signature verified against a trusted key.
    Verified,
    /// No signature present; loaded under `allow_unsigned` (⇒ banner + audit).
    UnsignedAllowed,
}

/// Decide whether a component may load, given its manifest signature, the trust
/// root, and the admin `allow_unsigned` policy. **Fails closed.**
pub fn decide(
    component_bytes: &[u8],
    manifest_signature: Option<&str>,
    trust: &TrustRoot,
    allow_unsigned: bool,
) -> Result<SignatureStatus> {
    match manifest_signature {
        Some(sig) => {
            trust.verify(component_bytes, sig)?;
            Ok(SignatureStatus::Verified)
        }
        None => {
            if allow_unsigned {
                Ok(SignatureStatus::UnsignedAllowed)
            } else {
                Err(PluginError::SignatureInvalid(
                    "component is unsigned and allow_unsigned is not set".into(),
                ))
            }
        }
    }
}

/// Minimal lowercase/uppercase hex decoder (no dep; signatures are 64 bytes).
fn decode_hex(s: &str) -> Option<Vec<u8>> {
    let s = s.trim();
    if !s.len().is_multiple_of(2) {
        return None;
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let b = s.as_bytes();
    let val = |c: u8| -> Option<u8> {
        match c {
            b'0'..=b'9' => Some(c - b'0'),
            b'a'..=b'f' => Some(c - b'a' + 10),
            b'A'..=b'F' => Some(c - b'A' + 10),
            _ => None,
        }
    };
    for pair in b.chunks(2) {
        out.push((val(pair[0])? << 4) | val(pair[1])?);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn keypair() -> SigningKey {
        // Deterministic test key (NOT for production).
        SigningKey::from_bytes(&[7u8; 32])
    }

    #[test]
    fn signed_round_trip_verifies() {
        let sk = keypair();
        let vk = sk.verifying_key();
        let trust = TrustRoot::from_public_keys(&[vk.to_bytes()]).unwrap();
        let bytes = b"the component bytes";
        let sig = sk.sign(bytes);
        let hex = hex_encode(&sig.to_bytes());
        assert_eq!(
            decide(bytes, Some(&hex), &trust, false).unwrap(),
            SignatureStatus::Verified
        );
    }

    #[test]
    fn tampered_bytes_fail() {
        let sk = keypair();
        let trust = TrustRoot::from_public_keys(&[sk.verifying_key().to_bytes()]).unwrap();
        let sig = sk.sign(b"original");
        let hex = hex_encode(&sig.to_bytes());
        assert!(decide(b"TAMPERED", Some(&hex), &trust, false).is_err());
    }

    #[test]
    fn unsigned_requires_policy() {
        let trust = TrustRoot::empty();
        assert!(matches!(
            decide(b"x", None, &trust, false),
            Err(PluginError::SignatureInvalid(_))
        ));
        assert_eq!(
            decide(b"x", None, &trust, true).unwrap(),
            SignatureStatus::UnsignedAllowed
        );
    }

    #[test]
    fn untrusted_key_is_rejected() {
        let signer = keypair();
        let other = SigningKey::from_bytes(&[9u8; 32]);
        let trust = TrustRoot::from_public_keys(&[other.verifying_key().to_bytes()]).unwrap();
        let sig = signer.sign(b"bytes");
        let hex = hex_encode(&sig.to_bytes());
        assert!(decide(b"bytes", Some(&hex), &trust, false).is_err());
    }

    fn hex_encode(b: &[u8]) -> String {
        let mut s = String::with_capacity(b.len() * 2);
        for byte in b {
            s.push_str(&format!("{byte:02x}"));
        }
        s
    }
}
