//! PQC readiness (plan §1.7 / SPEC §8.3): a **hybrid X25519 + ML-KEM-768** key-wrap
//! primitive used to wrap the `mw-store` seal master key at rest (crypto-agility
//! groundwork toward the V6 zero-access hierarchy). NATIVE-only — the browser never
//! wraps the server seal key.
//!
//! Security note (plan §6#8): `ml-kem` is **unaudited**; this is scoped to store-key
//! wrapping, NOT a user-facing security claim. The KEM combiner is the "simple"
//! hash combiner `KEK = SHA-256(suite ‖ x25519_ss ‖ mlkem_ss ‖ x25519_eph_pub ‖
//! mlkem_ct)`; the seal key is then wrapped with AES-256-GCM under `KEK`. The
//! recorded suite tag is [`crate::STORE_KEY_WRAP_SUITE`].

use aes_gcm::aead::{Aead, KeyInit as AeadKeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use ml_kem::{
    Decapsulate, DecapsulationKey, Encapsulate, EncapsulationKey, Kem, KeyExport, KeyInit,
    MlKem768, TryKeyInit,
};
use sha2::{Digest, Sha256};
use x25519_dalek::{PublicKey as XPublicKey, StaticSecret as XStaticSecret};

use crate::error::{CryptoError, Result};
use crate::rng;

// ML-KEM-768 encoded sizes (FIPS 203). The decapsulation key is stored in its
// 64-byte *seed* form (the ml-kem `KeyExport`/`KeyInit` canonical encoding).
const MLKEM_EK_LEN: usize = 1184;
const MLKEM_DK_SEED_LEN: usize = 64;
const MLKEM_CT_LEN: usize = 1088;
const X25519_LEN: usize = 32;

type Ek = EncapsulationKey<MlKem768>;
type Dk = DecapsulationKey<MlKem768>;

/// A recipient's hybrid key pair. `public` = X25519 public (32) ‖ ML-KEM ek (1184);
/// `secret` = X25519 secret (32) ‖ ML-KEM dk seed (64). Both are opaque byte blobs
/// the store persists alongside the wrapped seal material.
pub struct HybridKeyPair {
    pub public: Vec<u8>,
    pub secret: Vec<u8>,
}

/// Generate a hybrid X25519 + ML-KEM-768 recipient key pair.
pub fn generate_recipient() -> HybridKeyPair {
    let x_secret = XStaticSecret::random_from_rng(rng::pgp_rng());
    let x_public = XPublicKey::from(&x_secret);

    let (dk, ek) = MlKem768::generate_keypair_from_rng(&mut rng::rc10());

    let mut public = Vec::with_capacity(X25519_LEN + MLKEM_EK_LEN);
    public.extend_from_slice(x_public.as_bytes());
    public.extend_from_slice(&ek.to_bytes());

    let mut secret = Vec::with_capacity(X25519_LEN + MLKEM_DK_SEED_LEN);
    secret.extend_from_slice(x_secret.as_bytes());
    secret.extend_from_slice(&dk.to_bytes());

    HybridKeyPair { public, secret }
}

/// Wrap `seal_key` for the holder of `recipient_public` (a [`HybridKeyPair::public`]).
/// Output layout: x25519_eph_pub(32) ‖ mlkem_ct(1088) ‖ nonce(12) ‖ aead_ciphertext.
pub fn wrap(seal_key: &[u8], recipient_public: &[u8]) -> Result<Vec<u8>> {
    if recipient_public.len() != X25519_LEN + MLKEM_EK_LEN {
        return Err(CryptoError::KeyWrap("bad recipient public length".into()));
    }
    let x_pub_bytes: [u8; 32] = recipient_public[..X25519_LEN]
        .try_into()
        .map_err(|_| CryptoError::KeyWrap("x25519 public".into()))?;
    let x_pub = XPublicKey::from(x_pub_bytes);
    let ek = Ek::new_from_slice(&recipient_public[X25519_LEN..])
        .map_err(|_| CryptoError::KeyWrap("ml-kem ek".into()))?;

    // X25519: ephemeral-static ECDH.
    let x_eph = XStaticSecret::random_from_rng(rng::pgp_rng());
    let x_eph_pub = XPublicKey::from(&x_eph);
    let x_ss = x_eph.diffie_hellman(&x_pub);

    // ML-KEM: encapsulate to the recipient's ek.
    let (ct, mlkem_ss) = ek.encapsulate_with_rng(&mut rng::rc10());

    let kek = combine(x_ss.as_bytes(), &mlkem_ss, x_eph_pub.as_bytes(), &ct);

    let mut nonce_bytes = [0u8; 12];
    rng::fill_random(&mut nonce_bytes);
    let cipher = Aes256Gcm::new((&kek).into());
    let sealed = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), seal_key)
        .map_err(|_| CryptoError::KeyWrap("aead seal".into()))?;

    let mut out = Vec::with_capacity(X25519_LEN + MLKEM_CT_LEN + 12 + sealed.len());
    out.extend_from_slice(x_eph_pub.as_bytes());
    out.extend_from_slice(&ct);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&sealed);
    Ok(out)
}

/// Unwrap a [`wrap`] blob with `recipient_secret` (a [`HybridKeyPair::secret`]).
pub fn unwrap(wrapped: &[u8], recipient_secret: &[u8]) -> Result<Vec<u8>> {
    if recipient_secret.len() != X25519_LEN + MLKEM_DK_SEED_LEN {
        return Err(CryptoError::KeyWrap("bad recipient secret length".into()));
    }
    let header = X25519_LEN + MLKEM_CT_LEN + 12;
    if wrapped.len() < header {
        return Err(CryptoError::KeyWrap("truncated wrapped blob".into()));
    }
    let x_eph_pub_bytes: [u8; 32] = wrapped[..X25519_LEN]
        .try_into()
        .map_err(|_| CryptoError::KeyWrap("x25519 eph".into()))?;
    let x_eph_pub = XPublicKey::from(x_eph_pub_bytes);
    let ct = &wrapped[X25519_LEN..X25519_LEN + MLKEM_CT_LEN];
    let nonce_bytes = &wrapped[X25519_LEN + MLKEM_CT_LEN..header];
    let sealed = &wrapped[header..];

    let x_secret_bytes: [u8; 32] = recipient_secret[..X25519_LEN]
        .try_into()
        .map_err(|_| CryptoError::KeyWrap("x25519 secret".into()))?;
    let x_secret = XStaticSecret::from(x_secret_bytes);
    let x_ss = x_secret.diffie_hellman(&x_eph_pub);

    let dk = Dk::new_from_slice(&recipient_secret[X25519_LEN..])
        .map_err(|_| CryptoError::KeyWrap("ml-kem dk".into()))?;
    let mlkem_ss = dk
        .decapsulate_slice(ct)
        .map_err(|_| CryptoError::KeyWrap("ml-kem decapsulate".into()))?;

    let kek = combine(x_ss.as_bytes(), &mlkem_ss, x_eph_pub.as_bytes(), ct);
    let cipher = Aes256Gcm::new((&kek).into());
    cipher
        .decrypt(Nonce::from_slice(nonce_bytes), sealed)
        .map_err(|_| CryptoError::KeyWrap("aead open".into()))
}

/// The "simple" hash KEM combiner → a 32-byte AES-256 key.
fn combine(x_ss: &[u8], mlkem_ss: &[u8], x_eph_pub: &[u8], mlkem_ct: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(crate::STORE_KEY_WRAP_SUITE.as_bytes());
    h.update(x_ss);
    h.update(mlkem_ss);
    h.update(x_eph_pub);
    h.update(mlkem_ct);
    h.finalize().into()
}
