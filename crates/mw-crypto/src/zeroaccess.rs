//! Zero-access storage key hierarchy + device pairing (SPEC §9, plan §1.3/§1.4,
//! t6-e6). This module COMPOSES the primitives the rest of `mw-crypto`/`mw-store`
//! already ship — Argon2id, XChaCha20-Poly1305 (the same cipher `mw-store::seal`
//! uses at rest), P-256 ECDH (already pulled by the S/MIME stack), and SHA-256 —
//! into the client-side hierarchy of SPEC §9.1:
//!
//! ```text
//! passphrase | WebAuthn-PRF secret
//!         │  Argon2id (client-side, in-WASM)
//!         ▼
//!     Root Key  (never leaves the client)
//!         ├─► KEK ──wraps──► per-account Data Keys
//!         │                     ├─► message-cache key   (derive_subkey)
//!         │                     ├─► search-index key
//!         │                     ├─► notes key
//!         │                     └─► attachment-cache key
//!         └─► Recovery phrase (printable, optional)
//! ```
//!
//! It introduces **no new cipher** (plan §1.3 hard constraint): the row cipher is
//! the same `XChaCha20-Poly1305` construction as `mw-store::seal`, here bound to
//! `AAD = table ‖ 0x1F ‖ row_id ‖ 0x1F ‖ schema_version` (SPEC §9.3). The at-rest
//! ciphertext layout is `nonce(24) ‖ ciphertext+tag`, byte-identical to `seal.rs`
//! so `mw-store` (e1) and the web worker (e8) produce/consume the same blob.
//!
//! **Boundary invariant (plan §1.2 / DoD):** the `#[wasm_bindgen]` surface at the
//! bottom of this file NEVER returns a plaintext root/KEK/data key to JS. Derived
//! keys live in a `zeroize`-on-drop worker session and are addressed by opaque
//! refs; only ciphertext, wrapped-key blobs, public pairing material, and the
//! (explicitly user-exported) recovery phrase cross the boundary. The
//! `no_plaintext_key_escapes*` tests assert this for the byte outputs.

#![allow(clippy::needless_pass_by_value)]

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{Key as XKey, XChaCha20Poly1305, XNonce};
use sha2::{Digest, Sha256};

use crate::error::{CryptoError, Result};

/// Symmetric key length across the whole hierarchy (root/KEK/data/subkeys).
pub const KEY_LEN: usize = 32;
/// XChaCha20-Poly1305 nonce length — matches `mw-store::seal` exactly.
const NONCE_LEN: usize = 24;
/// P-256 compressed SEC1 public point length.
const P256_PUB_LEN: usize = 33;

// ── Domain-separation labels (frozen — changing one rotates every derived key) ──
const DERIVE_DOMAIN: &[u8] = b"mailwoman/zero-access/derive/v1";
const LABEL_KEK: &str = "kek";
/// AAD tag bound into wrapped-key blobs so a wrap can't be replayed as a row.
const WRAP_AAD: &[u8] = b"mailwoman/zero-access/wrap/v1";
/// Transcript domain for the pairing SAS + envelope seal.
const PAIR_SEAL_AAD: &[u8] = b"mailwoman/zero-access/pair/v1";
const PAIR_SAS_DOMAIN: &[u8] = b"mailwoman/zero-access/sas/v1";
/// Number of SAS words users compare out-of-band during pairing.
const SAS_WORDS: usize = 6;

// ─────────────────────────────────────────────────────────────────────────────
// Root-key derivation (Argon2id) + subkey KDF
// ─────────────────────────────────────────────────────────────────────────────

/// Argon2id cost parameters recorded alongside the wrapped root key
/// (`zeroaccess_accounts.kdf_params`, §2.1) so any device can re-derive it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArgonParams {
    /// Memory cost in KiB.
    pub m_cost: u32,
    /// Iterations (time cost).
    pub t_cost: u32,
    /// Degree of parallelism.
    pub p_cost: u32,
}

impl ArgonParams {
    /// OWASP-ish interactive defaults for a client deriving on a login (19 MiB,
    /// 2 passes, 1 lane). Tunable per deployment; recorded so re-derivation matches.
    pub const fn interactive() -> Self {
        Self {
            m_cost: 19_456,
            t_cost: 2,
            p_cost: 1,
        }
    }
}

impl Default for ArgonParams {
    fn default() -> Self {
        Self::interactive()
    }
}

/// Derive the **Root Key** from a passphrase OR a WebAuthn-PRF secret via Argon2id
/// (SPEC §9.1). `secret` is the passphrase bytes or the raw PRF output; `salt` is
/// the account's stored per-user salt (>= 16 bytes). Runs client-side / in-WASM.
pub fn derive_root_key(secret: &[u8], salt: &[u8], params: &ArgonParams) -> Result<[u8; KEY_LEN]> {
    if salt.len() < 16 {
        return Err(CryptoError::Input(
            "zero-access salt must be >= 16 bytes".into(),
        ));
    }
    let p = Params::new(params.m_cost, params.t_cost, params.p_cost, Some(KEY_LEN))
        .map_err(|e| CryptoError::Input(format!("argon2 params: {e}")))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, p);
    let mut out = [0u8; KEY_LEN];
    argon
        .hash_password_into(secret, salt, &mut out)
        .map_err(|e| CryptoError::KeyGen(format!("argon2id: {e}")))?;
    Ok(out)
}

/// Hash-based KDF over a high-entropy parent key: `SHA-256(DOMAIN ‖ 0x00 ‖ label ‖
/// 0x00 ‖ parent)`. Because `parent` is a uniform 32-byte key, this is a secure
/// pseudo-random-function-based subkey derivation with explicit domain separation
/// (no HMAC/HKDF dependency needed, and no new cipher — plan §1.3).
pub fn derive_subkey(parent: &[u8; KEY_LEN], label: &str) -> [u8; KEY_LEN] {
    let mut h = Sha256::new();
    h.update(DERIVE_DOMAIN);
    h.update([0x00]);
    h.update(label.as_bytes());
    h.update([0x00]);
    h.update(parent);
    h.finalize().into()
}

/// The Key-Encryption-Key derived from the root key (§9.1). The KEK wraps every
/// per-account data key; the root key itself is only ever used to derive it.
pub fn derive_kek(root: &[u8; KEY_LEN]) -> [u8; KEY_LEN] {
    derive_subkey(root, LABEL_KEK)
}

/// Generate a fresh random 32-byte data key (per-account, or a per-class subkey
/// source). Uses the crate's OS CSPRNG (`js` backend on wasm).
pub fn generate_data_key() -> [u8; KEY_LEN] {
    let mut k = [0u8; KEY_LEN];
    crate::rng::fill_random(&mut k);
    k
}

// ─────────────────────────────────────────────────────────────────────────────
// Symmetric seal / open (the ONE cipher — XChaCha20-Poly1305, as in mw-store)
// ─────────────────────────────────────────────────────────────────────────────

fn xchacha(key: &[u8]) -> Result<XChaCha20Poly1305> {
    if key.len() != KEY_LEN {
        return Err(CryptoError::Input("data key must be 32 bytes".into()));
    }
    Ok(XChaCha20Poly1305::new(XKey::from_slice(key)))
}

/// Low-level AEAD seal: `nonce(24) ‖ ciphertext+tag`, bound to `aad`. Identical
/// framing to `mw-store::seal` so the store row round-trips.
fn seal_aead(key: &[u8], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>> {
    let cipher = xchacha(key)?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    crate::rng::fill_random(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);
    let ct = cipher
        .encrypt(
            nonce,
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|_| CryptoError::Encrypt("xchacha seal".into()))?;
    let mut out = Vec::with_capacity(NONCE_LEN + ct.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ct);
    Ok(out)
}

/// Low-level AEAD open of a [`seal_aead`] blob. Wrong key OR wrong `aad` fails.
fn open_aead(key: &[u8], sealed: &[u8], aad: &[u8]) -> Result<Vec<u8>> {
    if sealed.len() < NONCE_LEN {
        return Err(CryptoError::Decrypt("truncated ciphertext".into()));
    }
    let (nonce_bytes, ct) = sealed.split_at(NONCE_LEN);
    let cipher = xchacha(key)?;
    let nonce = XNonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, Payload { msg: ct, aad })
        .map_err(|_| CryptoError::Decrypt("xchacha open (bad key or AAD)".into()))
}

/// Wrap a `data_key` under the `kek` (SPEC §9.1 `KEK ──wraps──► data key`). The
/// blob is bound to [`WRAP_AAD`] so it can never be misread as a row ciphertext.
pub fn wrap_key(kek: &[u8; KEY_LEN], data_key: &[u8]) -> Result<Vec<u8>> {
    seal_aead(kek, data_key, WRAP_AAD)
}

/// Unwrap a [`wrap_key`] blob under the `kek`.
pub fn unwrap_key(kek: &[u8; KEY_LEN], wrapped: &[u8]) -> Result<Vec<u8>> {
    open_aead(kek, wrapped, WRAP_AAD)
}

/// Build the frozen row AAD (SPEC §9.3): `table ‖ 0x1F ‖ row_id ‖ 0x1F ‖
/// ascii-decimal(schema_version)`. `0x1F` (ASCII unit separator) is the `‖`
/// delimiter — it cannot appear in an SQL table name or an opaque row id, so the
/// binding is unambiguous. **e1 (`mw-store`) and e8 (web worker) MUST reproduce
/// this exact byte sequence** or `open_row` rejects the row.
pub fn row_aad(table: &str, row_id: &str, schema_version: u32) -> Vec<u8> {
    let mut aad = Vec::with_capacity(table.len() + row_id.len() + 12);
    aad.extend_from_slice(table.as_bytes());
    aad.push(0x1F);
    aad.extend_from_slice(row_id.as_bytes());
    aad.push(0x1F);
    aad.extend_from_slice(schema_version.to_string().as_bytes());
    aad
}

/// Seal a plaintext row under a per-account/per-class `data_key`, bound to `aad`
/// (build it with [`row_aad`]). Output = `nonce(24) ‖ ciphertext+tag`.
pub fn seal_row(data_key: &[u8; KEY_LEN], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>> {
    seal_aead(data_key, plaintext, aad)
}

/// Open a [`seal_row`] blob. Fails on wrong key OR mismatched `aad` (i.e. a row
/// moved to a different table/id/schema — tamper detection, SPEC §9.3).
pub fn open_row(data_key: &[u8; KEY_LEN], ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>> {
    open_aead(data_key, ciphertext, aad)
}

// ─────────────────────────────────────────────────────────────────────────────
// Recovery phrase (printable root-key backup, SPEC §9.1)
// ─────────────────────────────────────────────────────────────────────────────
//
// A 33-word phrase = 32 key-byte words + 1 checksum word. Words come from a
// self-contained 256-entry list built from disjoint 2-char syllables (16 × 16),
// so each byte maps to exactly one 4-char pronounceable token and back — no giant
// embedded wordlist, no duplicate-collision risk (split point is fixed at 2).

const SYL_HI: [&str; 16] = [
    "ba", "de", "fi", "go", "ku", "lo", "ma", "ne", "pi", "ru", "sa", "to", "vi", "ze", "ja", "he",
];
const SYL_LO: [&str; 16] = [
    "bo", "da", "fe", "gi", "ko", "lu", "mi", "na", "pe", "ro", "su", "ta", "vo", "zi", "je", "ha",
];

fn byte_to_word(b: u8) -> String {
    format!(
        "{}{}",
        SYL_HI[(b >> 4) as usize],
        SYL_LO[(b & 0x0F) as usize]
    )
}

fn word_to_byte(w: &str) -> Option<u8> {
    if w.len() != 4 {
        return None;
    }
    let (hi, lo) = w.split_at(2);
    let h = SYL_HI.iter().position(|&s| s == hi)?;
    let l = SYL_LO.iter().position(|&s| s == lo)?;
    Some(((h << 4) | l) as u8)
}

fn checksum_byte(root: &[u8; KEY_LEN]) -> u8 {
    let mut h = Sha256::new();
    h.update(b"mailwoman/zero-access/recovery/v1");
    h.update(root);
    h.finalize()[0]
}

/// Encode a root key as a printable recovery phrase (SPEC §9.1). This is an
/// **explicit, user-initiated export** of the root key for offline backup — the
/// only path by which key material is meant to leave the device, and only when the
/// user asks. Store it offline; anyone with it can derive every account key.
pub fn recovery_phrase(root: &[u8; KEY_LEN]) -> String {
    let mut words: Vec<String> = root.iter().map(|&b| byte_to_word(b)).collect();
    words.push(byte_to_word(checksum_byte(root)));
    words.join(" ")
}

/// Restore a root key from a [`recovery_phrase`]. Rejects a wrong word count,
/// unknown words, or a checksum mismatch (mistyped phrase).
pub fn root_key_from_phrase(phrase: &str) -> Result<[u8; KEY_LEN]> {
    let words: Vec<&str> = phrase.split_whitespace().collect();
    if words.len() != KEY_LEN + 1 {
        return Err(CryptoError::Input(format!(
            "recovery phrase must be {} words",
            KEY_LEN + 1
        )));
    }
    let mut root = [0u8; KEY_LEN];
    for (i, w) in words[..KEY_LEN].iter().enumerate() {
        root[i] = word_to_byte(w)
            .ok_or_else(|| CryptoError::Input(format!("unknown recovery word: {w}")))?;
    }
    let want = word_to_byte(words[KEY_LEN])
        .ok_or_else(|| CryptoError::Input("unknown checksum word".into()))?;
    if want != checksum_byte(&root) {
        return Err(CryptoError::Input(
            "recovery phrase checksum mismatch (mistyped?)".into(),
        ));
    }
    Ok(root)
}

// ─────────────────────────────────────────────────────────────────────────────
// Device pairing — SAS-verified, client-to-client, server relays ciphertext only
// ─────────────────────────────────────────────────────────────────────────────
//
// Ceremony (SPEC §9.1, plan §1.3):
//   1. NEW device: `pair_generate()` → ephemeral P-256 keypair; shows its public
//      point in a QR. Its secret stays on the new device.
//   2. EXISTING device (holds the root key): scans the QR, calls
//      `pair_seal(root, new_public)` → its own ephemeral P-256 key + ECDH →
//      wrap key; seals the root key → `envelope = eph_pub(33) ‖ nonce ‖ ct`.
//      Both sides derive the same 6-word SAS from the transcript.
//   3. The opaque `envelope` is relayed THROUGH THE SERVER (ciphertext only) to
//      the new device, which calls `pair_open(envelope, secret)` → same ECDH →
//      recovers the root key + the SAS.
//   4. The user compares the SAS words on both screens; a match authenticates the
//      channel (defeats a MITM relay). The server never sees a plaintext key.

use p256::elliptic_curve::sec1::ToSec1Point;
use p256::{PublicKey, SecretKey};

/// An ephemeral pairing keypair. `public` is the 33-byte compressed SEC1 point
/// carried in the QR; `secret` is the 32-byte scalar the new device retains until
/// it opens the envelope. Neither is a hierarchy key.
pub struct PairingKeypair {
    pub public: Vec<u8>,
    pub secret: Vec<u8>,
}

fn gen_p256_secret() -> SecretKey {
    loop {
        let mut b = [0u8; 32];
        crate::rng::fill_random(&mut b);
        if let Ok(sk) = SecretKey::from_slice(&b) {
            return sk;
        }
    }
}

/// Step 1 (new device): generate the ephemeral pairing keypair for the QR.
pub fn pair_generate() -> PairingKeypair {
    let sk = gen_p256_secret();
    let pk = sk.public_key();
    PairingKeypair {
        public: pk.to_sec1_point(true).as_bytes().to_vec(),
        secret: sk.to_bytes().to_vec(),
    }
}

/// The output of [`pair_seal`]: the SAS words to display + the opaque envelope to
/// relay through the server.
pub struct SealedPairing {
    pub sas_words: Vec<String>,
    pub envelope: Vec<u8>,
}

/// The output of [`pair_open`]: the recovered root key + the SAS to compare.
pub struct OpenedPairing {
    pub sas_words: Vec<String>,
    pub root_key: [u8; KEY_LEN],
}

fn ecdh(secret: &SecretKey, peer: &PublicKey) -> [u8; KEY_LEN] {
    let shared = p256::ecdh::diffie_hellman(secret.to_nonzero_scalar(), peer.as_affine());
    // Derive a symmetric wrap key from the raw shared secret (domain-separated),
    // rather than using the coordinate directly.
    let mut h = Sha256::new();
    h.update(PAIR_SEAL_AAD);
    h.update(shared.raw_secret_bytes());
    h.finalize().into()
}

/// Derive the SAS words both devices show, from the full transcript so a swapped
/// public point changes them (MITM detection).
fn sas_words(new_pub: &[u8], eph_pub: &[u8], ct: &[u8]) -> Vec<String> {
    let mut h = Sha256::new();
    h.update(PAIR_SAS_DOMAIN);
    h.update(new_pub);
    h.update(eph_pub);
    h.update(ct);
    let digest = h.finalize();
    digest[..SAS_WORDS]
        .iter()
        .map(|&b| byte_to_word(b))
        .collect()
}

/// Step 2 (existing device): seal `root` to the new device's `peer_public`.
pub fn pair_seal(root: &[u8; KEY_LEN], peer_public: &[u8]) -> Result<SealedPairing> {
    if peer_public.len() != P256_PUB_LEN {
        return Err(CryptoError::Input("pairing public must be 33 bytes".into()));
    }
    let peer = PublicKey::from_sec1_bytes(peer_public)
        .map_err(|_| CryptoError::Input("invalid pairing public".into()))?;
    let eph = gen_p256_secret();
    let eph_pub = eph.public_key().to_sec1_point(true).as_bytes().to_vec();
    let wrap = ecdh(&eph, &peer);
    let sealed = seal_aead(&wrap, root, PAIR_SEAL_AAD)?;

    let mut envelope = Vec::with_capacity(P256_PUB_LEN + sealed.len());
    envelope.extend_from_slice(&eph_pub);
    envelope.extend_from_slice(&sealed);
    let ct = &envelope[P256_PUB_LEN..];
    let sas = sas_words(peer_public, &eph_pub, ct);
    Ok(SealedPairing {
        sas_words: sas,
        envelope,
    })
}

/// Step 3 (new device): open the relayed `envelope` with the retained `own_secret`
/// (from [`pair_generate`]). Returns the root key + the SAS to compare on-screen.
pub fn pair_open(envelope: &[u8], own_secret: &[u8]) -> Result<OpenedPairing> {
    if envelope.len() < P256_PUB_LEN + NONCE_LEN {
        return Err(CryptoError::Input("truncated pairing envelope".into()));
    }
    let (eph_pub, sealed) = envelope.split_at(P256_PUB_LEN);
    let sk = SecretKey::from_slice(own_secret)
        .map_err(|_| CryptoError::Input("invalid pairing secret".into()))?;
    let eph = PublicKey::from_sec1_bytes(eph_pub)
        .map_err(|_| CryptoError::Input("invalid envelope public".into()))?;
    let wrap = ecdh(&sk, &eph);
    let root_vec = open_aead(&wrap, sealed, PAIR_SEAL_AAD)?;
    if root_vec.len() != KEY_LEN {
        return Err(CryptoError::Decrypt(
            "pairing payload not a root key".into(),
        ));
    }
    let mut root_key = [0u8; KEY_LEN];
    root_key.copy_from_slice(&root_vec);
    // Own public point re-derived from the retained secret == what the QR carried.
    let own_pub = sk.public_key().to_sec1_point(true).as_bytes().to_vec();
    let sas = sas_words(&own_pub, eph_pub, sealed);
    Ok(OpenedPairing {
        sas_words: sas,
        root_key,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// WASM boundary (frozen — key material stays in the worker session as opaque refs)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
mod wasm_exports {
    //! The browser-side surface, called from the crypto Web Worker (e8). Derived
    //! keys are cached in a `zeroize`-on-drop session keyed by an opaque hex ref;
    //! **no root/KEK/data key is ever serialized back to JS** — only refs,
    //! ciphertext, wrapped blobs, public pairing material, and the explicitly
    //! user-exported recovery phrase leave this module.

    use std::cell::RefCell;
    use std::collections::HashMap;

    use base64::Engine as _;
    use serde::de::DeserializeOwned;
    use serde::{Deserialize, Serialize};
    use wasm_bindgen::prelude::*;
    use zeroize::Zeroizing;

    use super::*;

    thread_local! {
        /// ref → 32-byte hierarchy key (root/KEK/data). Zeroized on removal/drop.
        static KEYS: RefCell<HashMap<String, Zeroizing<[u8; KEY_LEN]>>> =
            RefCell::new(HashMap::new());
        /// ref → retained P-256 pairing secret (new-device side).
        static PAIR_SECRETS: RefCell<HashMap<String, Zeroizing<Vec<u8>>>> =
            RefCell::new(HashMap::new());
    }

    fn b64() -> base64::engine::general_purpose::GeneralPurpose {
        base64::engine::general_purpose::STANDARD
    }
    fn to_js<T: Serialize>(v: &T) -> std::result::Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(v).map_err(js_err)
    }
    fn from_js<T: DeserializeOwned>(v: JsValue) -> std::result::Result<T, JsValue> {
        serde_wasm_bindgen::from_value(v).map_err(js_err)
    }
    fn js_err(e: impl std::fmt::Display) -> JsValue {
        JsValue::from_str(&e.to_string())
    }
    fn err<T>(e: impl std::fmt::Display) -> std::result::Result<T, JsValue> {
        Err(js_err(e))
    }

    fn new_ref() -> String {
        let mut raw = [0u8; 16];
        crate::rng::fill_random(&mut raw);
        hex::encode(raw)
    }
    fn put_key(k: [u8; KEY_LEN]) -> String {
        let r = new_ref();
        KEYS.with(|m| m.borrow_mut().insert(r.clone(), Zeroizing::new(k)));
        r
    }
    fn get_key(r: &str) -> std::result::Result<[u8; KEY_LEN], JsValue> {
        KEYS.with(|m| m.borrow().get(r).map(|z| **z))
            .ok_or_else(|| js_err("unknown key ref (locked or expired)"))
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct DeriveRootIn {
        /// base64 passphrase bytes OR the WebAuthn-PRF secret (exactly one).
        secret_b64: String,
        salt_b64: String,
        m_cost: u32,
        t_cost: u32,
        p_cost: u32,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct RefOut {
        key_ref: String,
    }

    /// `deriveRootKey({secretB64, saltB64, mCost, tCost, pCost})` → `{ keyRef }`.
    /// The root key stays in-worker; only its ref is returned.
    #[wasm_bindgen(js_name = zaDeriveRootKey)]
    pub fn za_derive_root_key(options: JsValue) -> std::result::Result<JsValue, JsValue> {
        let i: DeriveRootIn = from_js(options)?;
        let secret = b64().decode(i.secret_b64).map_err(js_err)?;
        let salt = b64().decode(i.salt_b64).map_err(js_err)?;
        let params = ArgonParams {
            m_cost: i.m_cost,
            t_cost: i.t_cost,
            p_cost: i.p_cost,
        };
        let root = derive_root_key(&secret, &salt, &params).map_err(js_err)?;
        to_js(&RefOut {
            key_ref: put_key(root),
        })
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct RefIn {
        key_ref: String,
    }

    /// `zaDeriveKek({keyRef})` → `{ keyRef }` (the KEK, as a new in-worker ref).
    #[wasm_bindgen(js_name = zaDeriveKek)]
    pub fn za_derive_kek(options: JsValue) -> std::result::Result<JsValue, JsValue> {
        let i: RefIn = from_js(options)?;
        let root = get_key(&i.key_ref)?;
        to_js(&RefOut {
            key_ref: put_key(derive_kek(&root)),
        })
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct SubkeyIn {
        key_ref: String,
        label: String,
    }

    /// `zaDeriveSubkey({keyRef, label})` → `{ keyRef }` — per-class keys
    /// (`"message-cache"`, `"search"`, `"notes"`, `"attachment"`).
    #[wasm_bindgen(js_name = zaDeriveSubkey)]
    pub fn za_derive_subkey(options: JsValue) -> std::result::Result<JsValue, JsValue> {
        let i: SubkeyIn = from_js(options)?;
        let parent = get_key(&i.key_ref)?;
        to_js(&RefOut {
            key_ref: put_key(derive_subkey(&parent, &i.label)),
        })
    }

    /// `zaGenerateDataKey()` → `{ keyRef }` — a fresh random per-account data key.
    #[wasm_bindgen(js_name = zaGenerateDataKey)]
    pub fn za_generate_data_key() -> std::result::Result<JsValue, JsValue> {
        to_js(&RefOut {
            key_ref: put_key(generate_data_key()),
        })
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct WrapIn {
        kek_ref: String,
        data_key_ref: String,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct BlobOut {
        blob_b64: String,
    }

    /// `zaWrapKey({kekRef, dataKeyRef})` → `{ blobB64 }` (the wrapped data key,
    /// safe to persist server-side). Raw keys never cross the boundary.
    #[wasm_bindgen(js_name = zaWrapKey)]
    pub fn za_wrap_key(options: JsValue) -> std::result::Result<JsValue, JsValue> {
        let i: WrapIn = from_js(options)?;
        let kek = get_key(&i.kek_ref)?;
        let dk = get_key(&i.data_key_ref)?;
        let blob = wrap_key(&kek, &dk).map_err(js_err)?;
        to_js(&BlobOut {
            blob_b64: b64().encode(blob),
        })
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct UnwrapIn {
        kek_ref: String,
        blob_b64: String,
    }

    /// `zaUnwrapKey({kekRef, blobB64})` → `{ keyRef }` (the data key, in-worker).
    #[wasm_bindgen(js_name = zaUnwrapKey)]
    pub fn za_unwrap_key(options: JsValue) -> std::result::Result<JsValue, JsValue> {
        let i: UnwrapIn = from_js(options)?;
        let kek = get_key(&i.kek_ref)?;
        let blob = b64().decode(i.blob_b64).map_err(js_err)?;
        let dk = unwrap_key(&kek, &blob).map_err(js_err)?;
        if dk.len() != KEY_LEN {
            return err("unwrapped key wrong length");
        }
        let mut k = [0u8; KEY_LEN];
        k.copy_from_slice(&dk);
        to_js(&RefOut {
            key_ref: put_key(k),
        })
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct SealRowIn {
        key_ref: String,
        plaintext_b64: String,
        table: String,
        row_id: String,
        schema_version: u32,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct CiphertextOut {
        ciphertext_b64: String,
    }

    /// `zaSealRow({keyRef, plaintextB64, table, rowId, schemaVersion})` →
    /// `{ ciphertextB64 }`. AAD is bound per [`row_aad`] (§9.3).
    #[wasm_bindgen(js_name = zaSealRow)]
    pub fn za_seal_row(options: JsValue) -> std::result::Result<JsValue, JsValue> {
        let i: SealRowIn = from_js(options)?;
        let key = get_key(&i.key_ref)?;
        let pt = b64().decode(i.plaintext_b64).map_err(js_err)?;
        let aad = row_aad(&i.table, &i.row_id, i.schema_version);
        let ct = seal_row(&key, &pt, &aad).map_err(js_err)?;
        to_js(&CiphertextOut {
            ciphertext_b64: b64().encode(ct),
        })
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct OpenRowIn {
        key_ref: String,
        ciphertext_b64: String,
        table: String,
        row_id: String,
        schema_version: u32,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct PlaintextOut {
        plaintext_b64: String,
    }

    /// `zaOpenRow({keyRef, ciphertextB64, table, rowId, schemaVersion})` →
    /// `{ plaintextB64 }`. Fails on a wrong key or a moved row (AAD mismatch).
    #[wasm_bindgen(js_name = zaOpenRow)]
    pub fn za_open_row(options: JsValue) -> std::result::Result<JsValue, JsValue> {
        let i: OpenRowIn = from_js(options)?;
        let key = get_key(&i.key_ref)?;
        let ct = b64().decode(i.ciphertext_b64).map_err(js_err)?;
        let aad = row_aad(&i.table, &i.row_id, i.schema_version);
        let pt = open_row(&key, &ct, &aad).map_err(js_err)?;
        to_js(&PlaintextOut {
            plaintext_b64: b64().encode(pt),
        })
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct PhraseOut {
        phrase: String,
    }

    /// `zaRecoveryPhrase({keyRef})` → `{ phrase }`. EXPLICIT user export of the
    /// root key for offline backup — the sole intentional key-egress path.
    #[wasm_bindgen(js_name = zaRecoveryPhrase)]
    pub fn za_recovery_phrase(options: JsValue) -> std::result::Result<JsValue, JsValue> {
        let i: RefIn = from_js(options)?;
        let root = get_key(&i.key_ref)?;
        to_js(&PhraseOut {
            phrase: recovery_phrase(&root),
        })
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct RestoreIn {
        phrase: String,
    }

    /// `zaRestoreFromPhrase({phrase})` → `{ keyRef }` — re-imports the root key
    /// into the worker session (checksum-verified).
    #[wasm_bindgen(js_name = zaRestoreFromPhrase)]
    pub fn za_restore_from_phrase(options: JsValue) -> std::result::Result<JsValue, JsValue> {
        let i: RestoreIn = from_js(options)?;
        let root = root_key_from_phrase(&i.phrase).map_err(js_err)?;
        to_js(&RefOut {
            key_ref: put_key(root),
        })
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct PairGenOut {
        public_b64: String,
        secret_ref: String,
    }

    /// `zaPairGenerate()` → `{ publicB64, secretRef }` (new device). `publicB64`
    /// goes in the QR; the secret stays in-worker under `secretRef`.
    #[wasm_bindgen(js_name = zaPairGenerate)]
    pub fn za_pair_generate() -> std::result::Result<JsValue, JsValue> {
        let kp = pair_generate();
        let secret_ref = new_ref();
        PAIR_SECRETS.with(|m| {
            m.borrow_mut()
                .insert(secret_ref.clone(), Zeroizing::new(kp.secret));
        });
        to_js(&PairGenOut {
            public_b64: b64().encode(kp.public),
            secret_ref,
        })
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct PairSealIn {
        root_ref: String,
        peer_public_b64: String,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct PairSealOut {
        sas_words: Vec<String>,
        envelope_b64: String,
    }

    /// `zaPairSeal({rootRef, peerPublicB64})` → `{ sasWords, envelopeB64 }`
    /// (existing device). Seals the root key to the scanned public; the envelope
    /// is opaque to the relaying server.
    #[wasm_bindgen(js_name = zaPairSeal)]
    pub fn za_pair_seal(options: JsValue) -> std::result::Result<JsValue, JsValue> {
        let i: PairSealIn = from_js(options)?;
        let root = get_key(&i.root_ref)?;
        let peer = b64().decode(i.peer_public_b64).map_err(js_err)?;
        let sealed = pair_seal(&root, &peer).map_err(js_err)?;
        to_js(&PairSealOut {
            sas_words: sealed.sas_words,
            envelope_b64: b64().encode(sealed.envelope),
        })
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct PairOpenIn {
        envelope_b64: String,
        secret_ref: String,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct PairOpenOut {
        sas_words: Vec<String>,
        key_ref: String,
    }

    /// `zaPairComplete({envelopeB64, secretRef})` → `{ sasWords, keyRef }` (new
    /// device). Recovers the root key into the session; `sasWords` is shown for
    /// the user to compare against the other device before trusting.
    #[wasm_bindgen(js_name = zaPairComplete)]
    pub fn za_pair_complete(options: JsValue) -> std::result::Result<JsValue, JsValue> {
        let i: PairOpenIn = from_js(options)?;
        let envelope = b64().decode(i.envelope_b64).map_err(js_err)?;
        let secret = PAIR_SECRETS
            .with(|m| m.borrow().get(&i.secret_ref).map(|z| z.to_vec()))
            .ok_or_else(|| js_err("unknown pairing secret ref"))?;
        let opened = pair_open(&envelope, &secret).map_err(js_err)?;
        PAIR_SECRETS.with(|m| m.borrow_mut().remove(&i.secret_ref));
        to_js(&PairOpenOut {
            sas_words: opened.sas_words,
            key_ref: put_key(opened.root_key),
        })
    }

    /// `zaLock({keyRef})` — zeroize + drop one cached hierarchy key.
    #[wasm_bindgen(js_name = zaLock)]
    pub fn za_lock(options: JsValue) -> std::result::Result<JsValue, JsValue> {
        let i: RefIn = from_js(options)?;
        KEYS.with(|m| m.borrow_mut().remove(&i.key_ref));
        Ok(JsValue::UNDEFINED)
    }

    /// `zaLockAll()` — clear the entire zero-access session (logout/timeout).
    #[wasm_bindgen(js_name = zaLockAll)]
    pub fn za_lock_all() -> std::result::Result<JsValue, JsValue> {
        KEYS.with(|m| m.borrow_mut().clear());
        PAIR_SECRETS.with(|m| m.borrow_mut().clear());
        Ok(JsValue::UNDEFINED)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests (native — the wasm boundary wraps these same pure functions)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SALT: &[u8] = b"0123456789abcdef-zero-access-salt";

    fn test_params() -> ArgonParams {
        // Cheap params keep the test suite fast; production uses `interactive()`.
        ArgonParams {
            m_cost: 256,
            t_cost: 1,
            p_cost: 1,
        }
    }

    fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
        haystack.windows(needle.len()).any(|w| w == needle)
    }

    #[test]
    fn passphrase_to_root_to_data_key_to_row_round_trip() {
        // passphrase → root → KEK → data key → subkey → seal row → open row.
        let root = derive_root_key(b"correct horse battery staple", SALT, &test_params()).unwrap();
        let kek = derive_kek(&root);
        let data_key = generate_data_key();
        let wrapped = wrap_key(&kek, &data_key).unwrap();
        let unwrapped = unwrap_key(&kek, &wrapped).unwrap();
        assert_eq!(unwrapped, data_key);

        let mut msg_key = [0u8; KEY_LEN];
        msg_key.copy_from_slice(&unwrapped);
        let msg_key = derive_subkey(&msg_key, "message-cache");

        let aad = row_aad("messages", "msg-42", 7);
        let ct = seal_row(&msg_key, b"Subject: hi\r\n\r\nbody", &aad).unwrap();
        let pt = open_row(&msg_key, &ct, &aad).unwrap();
        assert_eq!(pt, b"Subject: hi\r\n\r\nbody");
    }

    #[test]
    fn derivation_is_deterministic() {
        let a = derive_root_key(b"pw", SALT, &test_params()).unwrap();
        let b = derive_root_key(b"pw", SALT, &test_params()).unwrap();
        assert_eq!(a, b);
        assert_eq!(derive_kek(&a), derive_kek(&b));
        assert_ne!(derive_subkey(&a, "search"), derive_subkey(&a, "notes"));
    }

    #[test]
    fn aad_mismatch_is_rejected() {
        let key = generate_data_key();
        let ct = seal_row(&key, b"secret row", &row_aad("messages", "id-1", 3)).unwrap();
        // Same key, but the row moved table / id / schema → open MUST fail.
        assert!(open_row(&key, &ct, &row_aad("notes", "id-1", 3)).is_err());
        assert!(open_row(&key, &ct, &row_aad("messages", "id-2", 3)).is_err());
        assert!(open_row(&key, &ct, &row_aad("messages", "id-1", 4)).is_err());
        // Correct AAD still opens.
        assert_eq!(
            open_row(&key, &ct, &row_aad("messages", "id-1", 3)).unwrap(),
            b"secret row"
        );
    }

    #[test]
    fn wrong_key_rejected() {
        let ct = seal_row(&generate_data_key(), b"x", &row_aad("t", "r", 1)).unwrap();
        assert!(open_row(&generate_data_key(), &ct, &row_aad("t", "r", 1)).is_err());
    }

    #[test]
    fn row_aad_is_the_frozen_format() {
        // table ‖ 0x1F ‖ row_id ‖ 0x1F ‖ ascii(schema_version) — e1/e8 must match.
        assert_eq!(row_aad("messages", "abc", 7), b"messages\x1fabc\x1f7");
    }

    #[test]
    fn recovery_phrase_restores_root_key() {
        let root = derive_root_key(b"pw", SALT, &test_params()).unwrap();
        let phrase = recovery_phrase(&root);
        assert_eq!(phrase.split_whitespace().count(), KEY_LEN + 1);
        assert_eq!(root_key_from_phrase(&phrase).unwrap(), root);
    }

    #[test]
    fn recovery_phrase_rejects_corruption() {
        let root = generate_data_key();
        let phrase = recovery_phrase(&root);
        let mut words: Vec<&str> = phrase.split_whitespace().collect();
        // Flip one data word to a different valid word → checksum must catch it.
        let replacement = if words[0] == "baba" { "bada" } else { "baba" };
        words[0] = replacement;
        assert!(root_key_from_phrase(&words.join(" ")).is_err());
        assert!(root_key_from_phrase("too few words").is_err());
    }

    #[test]
    fn wordlist_maps_every_byte_uniquely() {
        let mut seen = std::collections::HashSet::new();
        for b in 0u8..=255 {
            let w = byte_to_word(b);
            assert!(seen.insert(w.clone()), "duplicate word {w}");
            assert_eq!(word_to_byte(&w), Some(b));
        }
        assert_eq!(seen.len(), 256);
    }

    #[test]
    fn two_device_sas_pairing_agrees_a_key() {
        // New device generates its ephemeral keypair (QR).
        let new_dev = pair_generate();
        // Existing device holds the root key, seals to the scanned public.
        let root = derive_root_key(b"master", SALT, &test_params()).unwrap();
        let sealed = pair_seal(&root, &new_dev.public).unwrap();
        // Server relays only `sealed.envelope` (opaque). New device opens it.
        let opened = pair_open(&sealed.envelope, &new_dev.secret).unwrap();

        assert_eq!(opened.root_key, root, "paired root key must agree");
        assert_eq!(opened.sas_words, sealed.sas_words, "SAS must match");
        assert_eq!(opened.sas_words.len(), SAS_WORDS);
    }

    #[test]
    fn pairing_mitm_swapped_public_fails_or_diverges_sas() {
        let new_dev = pair_generate();
        let attacker = pair_generate();
        let root = derive_root_key(b"master", SALT, &test_params()).unwrap();
        // Existing device unknowingly seals to the ATTACKER's public.
        let sealed = pair_seal(&root, &attacker.public).unwrap();
        // The real new device cannot open an envelope sealed to the attacker.
        assert!(pair_open(&sealed.envelope, &new_dev.secret).is_err());
    }

    #[test]
    fn no_plaintext_key_escapes_into_at_rest_bytes() {
        // Root/KEK/data keys must never appear verbatim in anything the server
        // stores (wrapped blob, row ciphertext) or relays (pairing envelope).
        let root = derive_root_key(b"pw", SALT, &test_params()).unwrap();
        let kek = derive_kek(&root);
        let data_key = generate_data_key();

        let wrapped = wrap_key(&kek, &data_key).unwrap();
        assert!(!contains_subslice(&wrapped, &data_key));
        assert!(!contains_subslice(&wrapped, &kek));
        assert!(!contains_subslice(&wrapped, &root));

        let ct = seal_row(&data_key, b"body", &row_aad("messages", "1", 1)).unwrap();
        assert!(!contains_subslice(&ct, &data_key));

        let new_dev = pair_generate();
        let sealed = pair_seal(&root, &new_dev.public).unwrap();
        assert!(!contains_subslice(&sealed.envelope, &root));
    }
}
