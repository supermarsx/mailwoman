//! The WASM crypto boundary (frozen §2.3) — the browser-side private-key surface
//! called from the dedicated crypto Web Worker (`apps/web/src/crypto/worker.ts`).
//! ALL private material stays in the worker + the passphrase-wrapped client vault
//! and is NEVER posted to the main app state or the server in plaintext (plan §1.2,
//! risk #4). `zeroize` clears the session key cache on lock/timeout.
//!
//! This is the e0 skeleton: the exact `js_name` surface + `todo!()` bodies. e1
//! implements the rPGP/RustCrypto operations and e8 builds the wasm-pack bundle +
//! wires the worker. Bodies stay `todo!()` so the shape — not the behaviour — is
//! what e0 freezes. Argument/return objects are the `serde` DTOs in [`crate::types`]
//! (`generateKey` → `{publicKeyArmored,fingerprint,keyId,encryptedPrivateBundle}`,
//! `decrypt`/`verify`/`sign` → [`crate::types::SignatureVerdict`], etc., plan §2.3);
//! e1 serializes them across the boundary with `serde-wasm-bindgen`.

#![allow(clippy::needless_pass_by_value)]

use wasm_bindgen::prelude::*;

/// `generateKey({kind:"pgp", userId, passphrase})` → `{ publicKeyArmored,
/// fingerprint, keyId, encryptedPrivateBundle }` — v6 Ed25519/X25519; the private
/// key is wrapped by `passphrase` before it ever leaves the worker.
#[wasm_bindgen(js_name = generateKey)]
pub fn generate_key(_options: JsValue) -> JsValue {
    todo!("e1/e8: rPGP v6 Ed25519/X25519 keygen; passphrase-wrap the private key")
}

/// `encrypt({kind, plaintext, recipientPublicKeys, signWithKeyRef?, passphrase?,
/// protectedSubject?})` → `{ armoredCiphertext, encryptedSubjectApplied }`.
#[wasm_bindgen(js_name = encrypt)]
pub fn encrypt(_options: JsValue) -> JsValue {
    todo!("e1/e8: encrypt-to-recipients (+ optional sign, protected-subject header)")
}

/// `decrypt({kind, ciphertext, encryptedPrivateBundle, passphrase})` →
/// `{ plaintextHtml|plaintextText, subject?, signature }`. The plaintext is
/// sanitized IN-WORKER via the mw-sanitize wasm build before return (plan §1.3).
#[wasm_bindgen(js_name = decrypt)]
pub fn decrypt(_options: JsValue) -> JsValue {
    todo!("e1/e8: decrypt with the passphrase-unwrapped private key; in-worker sanitize")
}

/// `sign({kind, data, encryptedPrivateBundle, passphrase, detached})` →
/// `{ signatureArmored }`.
#[wasm_bindgen(js_name = sign)]
pub fn sign(_options: JsValue) -> JsValue {
    todo!("e1/e8: detached/inline signature with the unwrapped private key")
}

/// `verify({kind, data, signature, signerPublicKey})` → `SignatureVerdict`
/// (mirrors `SecurityVerdict.signature`).
#[wasm_bindgen(js_name = verify)]
pub fn verify(_options: JsValue) -> JsValue {
    todo!("e1/e8: verify a signature against a public key → SignatureVerdict")
}

/// `importPkcs12({p12Bytes, password})` → `{ certPem, fingerprint,
/// encryptedPrivateBundle }` — S/MIME private-key material, client-side only.
#[wasm_bindgen(js_name = importPkcs12)]
pub fn import_pkcs12(_options: JsValue) -> JsValue {
    todo!("e1/e8: PKCS#12 parse → cert PEM + passphrase-wrapped private bundle")
}

/// `importArmored({armored, passphrase?})` → `CryptoKey` (+ `encryptedPrivateBundle`
/// when the armor carried a private key).
#[wasm_bindgen(js_name = importArmored)]
pub fn import_armored(_options: JsValue) -> JsValue {
    todo!("e1/e8: parse armored PGP key → CryptoKey (+ wrapped private bundle)")
}

/// `exportPublic({keyRef})` → armored public key string.
#[wasm_bindgen(js_name = exportPublic)]
pub fn export_public(_options: JsValue) -> JsValue {
    todo!("e1/e8: export the armored public half of a held key")
}

/// `exportBackup({encryptedPrivateBundle, kind})` → `{ autocryptSetupMessage }`.
#[wasm_bindgen(js_name = exportBackup)]
pub fn export_backup(_options: JsValue) -> JsValue {
    todo!("e1/e8: emit an Autocrypt Setup Message from the wrapped private bundle")
}

/// `unlockKey({encryptedPrivateBundle, passphrase})` — decrypt into the worker
/// session cache (returns a key ref/handle). Paired with [`lock_key`].
#[wasm_bindgen(js_name = unlockKey)]
pub fn unlock_key(_options: JsValue) -> JsValue {
    todo!("e1/e8: unwrap into the in-worker session cache; return a key ref")
}

/// `lockKey({keyRef})` — `zeroize` the cached private key (also fired on timeout).
#[wasm_bindgen(js_name = lockKey)]
pub fn lock_key(_options: JsValue) -> JsValue {
    todo!("e1/e8: zeroize + drop the cached private key for this ref")
}
