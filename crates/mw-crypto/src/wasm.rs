//! The WASM crypto boundary (frozen §2.3) — the browser-side private-key surface
//! called from the dedicated crypto Web Worker (`apps/web/src/crypto/worker.ts`).
//! ALL private material stays in the worker + the passphrase-wrapped client vault
//! and is NEVER posted to the main app state or the server in plaintext (plan §1.2,
//! risk #4). `zeroize` clears the session key cache on lock/timeout.
//!
//! Arguments/returns are the `serde` DTOs below (camelCase to match the frozen TS in
//! `apps/web/src/contracts/crypto.ts`), marshalled across the boundary with
//! `serde-wasm-bindgen`. Errors become thrown JS `Error`s. Private-key ops delegate
//! to [`crate::pgp`] / [`crate::smime`]; the split from the native build is purely
//! which functions each side calls (the crypto is target-agnostic, plan §1.1).

#![allow(clippy::needless_pass_by_value)]

use std::cell::RefCell;
use std::collections::HashMap;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use zeroize::Zeroize;

use crate::types::{CryptoKey, SignatureVerdict};
use crate::{pgp, smime};

thread_local! {
    /// Session cache of unlocked keys (keyRef → wrapped bundle + passphrase), so the
    /// worker need not re-pass the passphrase per op. Cleared by `lockKey`/timeout.
    static SESSION: RefCell<HashMap<String, Cached>> = RefCell::new(HashMap::new());
}

struct Cached {
    bundle: String,
    passphrase: String,
}

impl Drop for Cached {
    fn drop(&mut self) {
        self.bundle.zeroize();
        self.passphrase.zeroize();
    }
}

/// Install the panic→console hook once at module init (wasm-bindgen `start`).
#[wasm_bindgen(start)]
pub fn __init() {
    console_error_panic_hook::set_once();
}

fn to_js<T: Serialize>(v: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(v).map_err(js_err)
}
fn from_js<T: DeserializeOwned>(v: JsValue) -> Result<T, JsValue> {
    serde_wasm_bindgen::from_value(v).map_err(js_err)
}
fn js_err(e: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&e.to_string())
}

// ── generateKey ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GenerateKeyIn {
    #[allow(dead_code)]
    kind: String,
    user_id: String,
    passphrase: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerateKeyOut {
    public_key_armored: String,
    fingerprint: String,
    key_id: String,
    encrypted_private_bundle: String,
}

/// `generateKey({kind:"pgp", userId, passphrase})` → `{ publicKeyArmored,
/// fingerprint, keyId, encryptedPrivateBundle }` — v6 Ed25519/X25519.
#[wasm_bindgen(js_name = generateKey)]
pub fn generate_key(options: JsValue) -> Result<JsValue, JsValue> {
    let i: GenerateKeyIn = from_js(options)?;
    let k = pgp::generate_key(&i.user_id, &i.passphrase).map_err(js_err)?;
    to_js(&GenerateKeyOut {
        public_key_armored: k.public_key_armored,
        fingerprint: k.fingerprint,
        key_id: k.key_id,
        encrypted_private_bundle: k.encrypted_private_bundle,
    })
}

// ── encrypt ──────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EncryptIn {
    kind: String,
    plaintext: String,
    recipient_public_keys: Vec<String>,
    sign_with_key_ref: Option<String>,
    protected_subject: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EncryptOut {
    armored_ciphertext: String,
    encrypted_subject_applied: bool,
}

/// `encrypt({kind, plaintext, recipientPublicKeys, signWithKeyRef?,
/// protectedSubject?})` → `{ armoredCiphertext, encryptedSubjectApplied }`.
#[wasm_bindgen(js_name = encrypt)]
pub fn encrypt(options: JsValue) -> Result<JsValue, JsValue> {
    let i: EncryptIn = from_js(options)?;
    let subject_applied = i.protected_subject.is_some();
    match i.kind.as_str() {
        "pgp" => {
            let body = match &i.protected_subject {
                Some(s) => pgp::wrap_protected_headers(s, i.plaintext.as_bytes()),
                None => i.plaintext.into_bytes(),
            };
            let signer = i.sign_with_key_ref.as_deref().and_then(session_lookup);
            let sign_ref = signer.as_ref().map(|(b, p)| (b.as_str(), p.as_str()));
            let armored =
                pgp::encrypt(&body, &i.recipient_public_keys, sign_ref).map_err(js_err)?;
            to_js(&EncryptOut {
                armored_ciphertext: armored,
                encrypted_subject_applied: subject_applied,
            })
        }
        "smime" => {
            use base64::Engine;
            let der =
                smime::encrypt(i.plaintext.as_bytes(), &i.recipient_public_keys).map_err(js_err)?;
            to_js(&EncryptOut {
                armored_ciphertext: base64::engine::general_purpose::STANDARD.encode(der),
                encrypted_subject_applied: false,
            })
        }
        other => Err(js_err(format!("unknown kind: {other}"))),
    }
}

// ── decrypt ──────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DecryptIn {
    kind: String,
    ciphertext: String,
    encrypted_private_bundle: String,
    passphrase: String,
    signer_public_key: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DecryptOut {
    plaintext_text: String,
    subject: Option<String>,
    signature: SignatureVerdict,
}

/// `decrypt({kind, ciphertext, encryptedPrivateBundle, passphrase})` →
/// `{ plaintextText, subject?, signature }`. (In-worker mw-sanitize wasm sanitizes
/// HTML before it reaches the iframe — plan §1.3, wired by e8.)
#[wasm_bindgen(js_name = decrypt)]
pub fn decrypt(options: JsValue) -> Result<JsValue, JsValue> {
    let i: DecryptIn = from_js(options)?;
    match i.kind.as_str() {
        "pgp" => {
            let out = pgp::decrypt(
                i.ciphertext.as_bytes(),
                &i.encrypted_private_bundle,
                &i.passphrase,
                i.signer_public_key.as_deref(),
            )
            .map_err(js_err)?;
            to_js(&DecryptOut {
                plaintext_text: String::from_utf8_lossy(&out.plaintext).into_owned(),
                subject: out.subject,
                signature: out.signature,
            })
        }
        "smime" => {
            use base64::Engine;
            let der = base64::engine::general_purpose::STANDARD
                .decode(i.ciphertext.as_bytes())
                .map_err(js_err)?;
            let pt =
                smime::decrypt(&der, &i.encrypted_private_bundle, &i.passphrase).map_err(js_err)?;
            to_js(&DecryptOut {
                plaintext_text: String::from_utf8_lossy(&pt).into_owned(),
                subject: None,
                signature: none_verdict("smime"),
            })
        }
        other => Err(js_err(format!("unknown kind: {other}"))),
    }
}

// ── sign ─────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SignIn {
    kind: String,
    data: String,
    encrypted_private_bundle: String,
    passphrase: String,
    /// `false` → an inline cleartext-signed `PGP SIGNED MESSAGE` (the body stays
    /// readable); `true` → a bare detached `PGP SIGNATURE`. PGP only.
    detached: bool,
    /// For S/MIME the signer certificate PEM (the worker holds it beside the key).
    cert_pem: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SignOut {
    signature_armored: String,
}

/// `sign({kind, data, encryptedPrivateBundle, passphrase, detached})` →
/// `{ signatureArmored }`. S/MIME additionally needs the signer `certPem`.
#[wasm_bindgen(js_name = sign)]
pub fn sign(options: JsValue) -> Result<JsValue, JsValue> {
    let i: SignIn = from_js(options)?;
    let armored = match i.kind.as_str() {
        // `detached:false` (clear-sign) keeps the body inline as a `PGP SIGNED
        // MESSAGE`; `detached:true` emits only the `PGP SIGNATURE` armor.
        "pgp" if i.detached => {
            pgp::sign_detached(i.data.as_bytes(), &i.encrypted_private_bundle, &i.passphrase)
                .map_err(js_err)?
        }
        "pgp" => pgp::clear_sign(&i.data, &i.encrypted_private_bundle, &i.passphrase)
            .map_err(js_err)?,
        "smime" => {
            let cert = i
                .cert_pem
                .ok_or_else(|| js_err("smime sign requires certPem"))?;
            smime::sign(
                i.data.as_bytes(),
                &cert,
                &i.encrypted_private_bundle,
                &i.passphrase,
            )
            .map_err(js_err)?
        }
        other => return Err(js_err(format!("unknown kind: {other}"))),
    };
    to_js(&SignOut {
        signature_armored: armored,
    })
}

// ── verify ───────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VerifyIn {
    kind: String,
    data: String,
    signature: String,
    signer_public_key: Option<String>,
}

/// `verify({kind, data, signature, signerPublicKey})` → `SignatureVerdict`.
#[wasm_bindgen(js_name = verify)]
pub fn verify(options: JsValue) -> Result<JsValue, JsValue> {
    let i: VerifyIn = from_js(options)?;
    let verdict = match i.kind.as_str() {
        "pgp" => {
            let key = i
                .signer_public_key
                .ok_or_else(|| js_err("pgp verify requires signerPublicKey"))?;
            pgp::verify_detached(i.data.as_bytes(), &i.signature, &key).map_err(js_err)?
        }
        "smime" => {
            use base64::Engine;
            let der = base64::engine::general_purpose::STANDARD
                .decode(i.signature.as_bytes())
                .map_err(js_err)?;
            smime::verify(&der).map_err(js_err)?
        }
        other => return Err(js_err(format!("unknown kind: {other}"))),
    };
    to_js(&verdict)
}

// ── importPkcs12 ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportPkcs12In {
    p12_bytes: Vec<u8>,
    password: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ImportPkcs12Out {
    cert_pem: String,
    fingerprint: String,
    encrypted_private_bundle: String,
}

/// `importPkcs12({p12Bytes, password})` → `{ certPem, fingerprint,
/// encryptedPrivateBundle }` — S/MIME private-key material, client-side only.
#[wasm_bindgen(js_name = importPkcs12)]
pub fn import_pkcs12(options: JsValue) -> Result<JsValue, JsValue> {
    let i: ImportPkcs12In = from_js(options)?;
    let r = smime::import_pkcs12(&i.p12_bytes, &i.password).map_err(js_err)?;
    to_js(&ImportPkcs12Out {
        cert_pem: r.cert_pem,
        fingerprint: r.fingerprint,
        encrypted_private_bundle: r.encrypted_private_bundle,
    })
}

// ── importArmored ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportArmoredIn {
    armored: String,
    #[serde(default)]
    addresses: Vec<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ImportArmoredOut {
    key: CryptoKey,
    encrypted_private_bundle: Option<String>,
}

/// `importArmored({armored})` → `CryptoKey` (+ `encryptedPrivateBundle` when the
/// armor carried a private key).
#[wasm_bindgen(js_name = importArmored)]
pub fn import_armored(options: JsValue) -> Result<JsValue, JsValue> {
    let i: ImportArmoredIn = from_js(options)?;
    let (key, bundle) = pgp::parse_key(&i.armored, i.addresses).map_err(js_err)?;
    to_js(&ImportArmoredOut {
        key,
        encrypted_private_bundle: bundle,
    })
}

// ── exportPublic ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportPublicIn {
    key_ref: String,
}

/// `exportPublic({keyRef})` → armored public key string. `keyRef` may be a session
/// ref or an armored key/bundle directly.
#[wasm_bindgen(js_name = exportPublic)]
pub fn export_public(options: JsValue) -> Result<JsValue, JsValue> {
    let i: ExportPublicIn = from_js(options)?;
    let bundle = session_lookup(&i.key_ref)
        .map(|(b, _)| b)
        .unwrap_or(i.key_ref);
    let armored = pgp::export_public(&bundle).map_err(js_err)?;
    to_js(&armored)
}

// ── exportBackup ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportBackupIn {
    encrypted_private_bundle: String,
    #[allow(dead_code)]
    kind: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportBackupOut {
    autocrypt_setup_message: String,
}

/// `exportBackup({encryptedPrivateBundle, kind})` → `{ autocryptSetupMessage }`.
#[wasm_bindgen(js_name = exportBackup)]
pub fn export_backup(options: JsValue) -> Result<JsValue, JsValue> {
    let i: ExportBackupIn = from_js(options)?;
    let asm = pgp::autocrypt_setup_message(&i.encrypted_private_bundle).map_err(js_err)?;
    to_js(&ExportBackupOut {
        autocrypt_setup_message: asm,
    })
}

// ── unlockKey / lockKey (session cache) ──────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UnlockIn {
    #[allow(dead_code)]
    kind: String,
    encrypted_private_bundle: String,
    passphrase: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UnlockOut {
    key_ref: String,
}

/// `unlockKey({kind, encryptedPrivateBundle, passphrase})` → `{ keyRef }`. Caches the
/// bundle + passphrase in the worker session so `signWithKeyRef` needs no re-entry.
#[wasm_bindgen(js_name = unlockKey)]
pub fn unlock_key(options: JsValue) -> Result<JsValue, JsValue> {
    let i: UnlockIn = from_js(options)?;
    let mut raw = [0u8; 16];
    crate::rng::fill_random(&mut raw);
    let key_ref = hex::encode(raw);
    SESSION.with(|s| {
        s.borrow_mut().insert(
            key_ref.clone(),
            Cached {
                bundle: i.encrypted_private_bundle,
                passphrase: i.passphrase,
            },
        );
    });
    to_js(&UnlockOut { key_ref })
}

/// `lockKey({keyRef})` — `zeroize` + drop the cached private material for this ref.
#[wasm_bindgen(js_name = lockKey)]
pub fn lock_key(options: JsValue) -> Result<JsValue, JsValue> {
    let i: ExportPublicIn = from_js(options)?; // reuse { keyRef }
    SESSION.with(|s| s.borrow_mut().remove(&i.key_ref));
    Ok(JsValue::UNDEFINED)
}

fn session_lookup(key_ref: &str) -> Option<(String, String)> {
    SESSION.with(|s| {
        s.borrow()
            .get(key_ref)
            .map(|c| (c.bundle.clone(), c.passphrase.clone()))
    })
}

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
