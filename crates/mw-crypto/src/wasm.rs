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
use crate::verdict::Verification;
use crate::{mime, pgp, smime};

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

/// Marshal a DTO to JS. `None` becomes `null`, as `contracts/crypto.ts` declares
/// (`string | null`); a field that is absent rather than null says so with
/// `skip_serializing_if`.
fn to_js<T: Serialize>(v: &T) -> Result<JsValue, JsValue> {
    v.serialize(&serde_wasm_bindgen::Serializer::new().serialize_missing_as_null(true))
        .map_err(js_err)
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
    kind: String,
    user_id: String,
    passphrase: String,
    /// S/MIME only: RSA modulus size (2048, 3072 or 4096). Absent → the crate
    /// default, [`smime::GENERATED_RSA_BITS`].
    rsa_bits: Option<usize>,
    key_type: Option<crate::smime_keys::KeyType>,
    /// PGP only: `"v6"` (default) or `"compatible"` (v4, readable by GnuPG 2.4).
    #[serde(default)]
    profile: pgp::KeyProfile,
    /// PGP only: seconds after creation when the key expires.
    expires_in_secs: Option<u64>,
}
/// A field that does not apply to the generated kind is absent, not empty: a PGP
/// key has no `certPem`, an S/MIME key has no `publicKeyArmored`.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerateKeyOut {
    #[serde(skip_serializing_if = "Option::is_none")]
    public_key_armored: Option<String>,
    fingerprint: String,
    key_id: String,
    encrypted_private_bundle: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    cert_pem: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    addresses: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    algorithm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_at: Option<String>,
}

/// `generateKey({kind, userId, passphrase})`.
///
/// - `kind:"pgp"` → `{ publicKeyArmored, fingerprint, keyId,
///   encryptedPrivateBundle }` — v6 Ed25519/X25519.
/// - `kind:"smime"` → `{ certPem, fingerprint, keyId, encryptedPrivateBundle,
///   addresses, algorithm, expiresAt }` — an RSA key and an X.509 certificate for
///   the address in `userId`, signed by that key itself (see [`smime::generate`]).
///   `addresses`, `algorithm` and `expiresAt` are read back from the certificate.
#[wasm_bindgen(js_name = generateKey)]
pub fn generate_key(options: JsValue) -> Result<JsValue, JsValue> {
    let i: GenerateKeyIn = from_js(options)?;
    match i.kind.as_str() {
        "pgp" => {
            let opts = pgp::GenerateOptions {
                profile: i.profile,
                expires_in_secs: i.expires_in_secs,
            };
            let k = pgp::generate_key_with(&i.user_id, &i.passphrase, &opts).map_err(js_err)?;
            to_js(&GenerateKeyOut {
                public_key_armored: Some(k.public_key_armored),
                fingerprint: k.fingerprint,
                key_id: k.key_id,
                encrypted_private_bundle: k.encrypted_private_bundle,
                cert_pem: None,
                addresses: None,
                algorithm: None,
                expires_at: None,
            })
        }
        "smime" => {
            let (name, email) = split_user_id(&i.user_id);
            let bits = i.rsa_bits.unwrap_or(smime::GENERATED_RSA_BITS);
            let g = match i.key_type {
                Some(kind) => smime::generate_key(kind, name, email, &i.passphrase, unix_now()),
                None => smime::generate_with_bits(name, email, &i.passphrase, unix_now(), bits),
            }
            .map_err(js_err)?;
            to_js(&GenerateKeyOut {
                public_key_armored: None,
                key_id: g.cert.fingerprint[..16.min(g.cert.fingerprint.len())].to_string(),
                fingerprint: g.cert.fingerprint,
                encrypted_private_bundle: g.encrypted_private_bundle,
                cert_pem: Some(g.cert.cert_pem),
                addresses: Some(g.cert.addresses),
                algorithm: Some(g.cert.algorithm),
                expires_at: Some(g.cert.not_after),
            })
        }
        other => Err(js_err(format!("unknown kind: {other}"))),
    }
}

/// `Name <email>` → `(name, email)`; a bare address → `("", address)`.
fn split_user_id(user_id: &str) -> (&str, &str) {
    match (user_id.rfind('<'), user_id.rfind('>')) {
        (Some(open), Some(close)) if open < close => {
            (user_id[..open].trim(), user_id[open + 1..close].trim())
        }
        _ => ("", user_id.trim()),
    }
}

/// Seconds since the Unix epoch from the JS clock (`std::time` has no clock on
/// `wasm32-unknown-unknown`).
fn unix_now() -> u64 {
    (js_sys::Date::now() / 1000.0) as u64
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
    #[serde(default)]
    cipher: crate::smime_cms::Cipher,
    #[serde(default)]
    recipient_gcm: Vec<Option<bool>>,
    sign_with_cert_pem: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EncryptOut {
    armored_ciphertext: String,
    encrypted_subject_applied: bool,
    /// PGP: 1 (SEIPDv1, some recipient lacks AEAD support) or 2 (SEIPDv2).
    #[serde(skip_serializing_if = "Option::is_none")]
    seipd_version: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    smime_type: Option<String>,
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
            let out =
                pgp::encrypt_with(&body, &i.recipient_public_keys, sign_ref).map_err(js_err)?;
            to_js(&EncryptOut {
                armored_ciphertext: out.armored,
                encrypted_subject_applied: subject_applied,
                seipd_version: Some(out.seipd_version),
                smime_type: None,
            })
        }
        "smime" => {
            use base64::Engine;
            let mut body = i.plaintext.into_bytes();
            if let Some(key_ref) = &i.sign_with_key_ref {
                let (bundle, passphrase) =
                    session_lookup(key_ref).ok_or_else(|| js_err("signing key is locked"))?;
                let cert = i
                    .sign_with_cert_pem
                    .as_deref()
                    .ok_or_else(|| js_err("S/MIME signing needs signWithCertPem"))?;
                body = crate::smime_cms::sign(
                    &body,
                    cert.as_bytes(),
                    &bundle,
                    &passphrase,
                    &crate::smime_cms::SignOptions {
                        detached: false,
                        ..Default::default()
                    },
                )
                .map_err(js_err)?;
            }
            let recipients: Vec<_> = i
                .recipient_public_keys
                .iter()
                .enumerate()
                .map(|(n, cert)| crate::smime_cms::Recipient {
                    cert: cert.as_bytes().to_vec(),
                    gcm: i.recipient_gcm.get(n).copied().flatten(),
                })
                .collect();
            let out = crate::smime_cms::encrypt(&body, &recipients, i.cipher).map_err(js_err)?;
            to_js(&EncryptOut {
                armored_ciphertext: base64::engine::general_purpose::STANDARD.encode(out.der),
                encrypted_subject_applied: false,
                seipd_version: None,
                smime_type: Some(out.smime_type.into()),
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
    /// PGP: every candidate signer key (the sender's keys from the keyring).
    #[serde(default)]
    signer_public_keys: Vec<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DecryptOut {
    plaintext_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    subject: Option<String>,
    signature: SignatureVerdict,
    /// The full signature state (see `crate::verdict`).
    verification: Verification,
    /// PGP: the message used SEIPDv2.
    #[serde(skip_serializing_if = "Option::is_none")]
    aead: Option<bool>,
}

/// `decrypt({kind, ciphertext, encryptedPrivateBundle, passphrase})` →
/// `{ plaintextText, subject?, signature }`. (In-worker mw-sanitize wasm sanitizes
/// HTML before it reaches the iframe — plan §1.3, wired by e8.)
#[wasm_bindgen(js_name = decrypt)]
pub fn decrypt(options: JsValue) -> Result<JsValue, JsValue> {
    let i: DecryptIn = from_js(options)?;
    match i.kind.as_str() {
        "pgp" => {
            let signers: Vec<&str> = i
                .signer_public_key
                .iter()
                .chain(i.signer_public_keys.iter())
                .map(String::as_str)
                .collect();
            let out = pgp::decrypt_with(
                i.ciphertext.as_bytes(),
                &i.encrypted_private_bundle,
                &i.passphrase,
                &signers,
            )
            .map_err(js_err)?;
            to_js(&DecryptOut {
                plaintext_text: String::from_utf8_lossy(&out.plaintext).into_owned(),
                subject: out.subject,
                signature: out.signature,
                verification: out.verification,
                aead: Some(out.aead),
            })
        }
        "smime" => {
            use base64::Engine;
            let der = base64::engine::general_purpose::STANDARD
                .decode(i.ciphertext.as_bytes())
                .map_err(js_err)?;
            let pt =
                smime::decrypt(&der, &i.encrypted_private_bundle, &i.passphrase).map_err(js_err)?;
            let (pt, verification) = if crate::smime_cms::signed_data_has_content(&pt) == Some(true)
            {
                let verified =
                    crate::smime_cms::verify(&pt, &crate::smime_cms::VerifyOptions::default())
                        .map_err(js_err)?;
                (verified.content.unwrap_or_default(), verified.verification)
            } else {
                (pt, Verification::not_signed("smime"))
            };
            to_js(&DecryptOut {
                plaintext_text: String::from_utf8_lossy(&pt).into_owned(),
                subject: None,
                signature: verification.signature.clone(),
                verification,
                aead: None,
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
    /// readable); `true` → a bare detached signature. Also applies to S/MIME.
    detached: bool,
    /// For S/MIME the signer certificate PEM (the worker holds it beside the key).
    cert_pem: Option<String>,
    #[serde(default)]
    chain: Vec<String>,
    #[serde(default)]
    pss: bool,
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
        "pgp" if i.detached => pgp::sign_detached(
            i.data.as_bytes(),
            &i.encrypted_private_bundle,
            &i.passphrase,
        )
        .map_err(js_err)?,
        "pgp" => {
            pgp::clear_sign(&i.data, &i.encrypted_private_bundle, &i.passphrase).map_err(js_err)?
        }
        "smime" => {
            let cert = i
                .cert_pem
                .ok_or_else(|| js_err("smime sign requires certPem"))?;
            use base64::Engine;
            let der = crate::smime_cms::sign(
                i.data.as_bytes(),
                cert.as_bytes(),
                &i.encrypted_private_bundle,
                &i.passphrase,
                &crate::smime_cms::SignOptions {
                    detached: i.detached,
                    chain: i.chain.into_iter().map(String::into_bytes).collect(),
                    pss: i.pss,
                    ..Default::default()
                },
            )
            .map_err(js_err)?;
            base64::engine::general_purpose::STANDARD.encode(der)
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
    /// PGP: every candidate signer key.
    #[serde(default)]
    signer_public_keys: Vec<String>,
}

/// `verify({kind, data, signature, signerPublicKey})` → `SignatureVerdict`.
#[wasm_bindgen(js_name = verify)]
pub fn verify(options: JsValue) -> Result<JsValue, JsValue> {
    let i: VerifyIn = from_js(options)?;
    let verdict = match i.kind.as_str() {
        "pgp" => {
            let signers: Vec<&str> = i
                .signer_public_key
                .iter()
                .chain(i.signer_public_keys.iter())
                .map(String::as_str)
                .collect();
            if signers.is_empty() {
                return Err(js_err("pgp verify requires signerPublicKey"));
            }
            pgp::verify_detached_with(i.data.as_bytes(), &i.signature, &signers)
                .map_err(js_err)?
                .signature
        }
        "smime" => {
            use base64::Engine;
            let der = base64::engine::general_purpose::STANDARD
                .decode(i.signature.as_bytes())
                .map_err(js_err)?;
            smime::verify_detached(&der, Some(i.data.as_bytes())).map_err(js_err)?
        }
        other => return Err(js_err(format!("unknown kind: {other}"))),
    };
    to_js(&verdict)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VerifySmimeIn {
    cms: Raw,
    content: Option<Raw>,
    sender: Option<String>,
    #[serde(default)]
    extra_certs: Vec<String>,
    #[serde(default)]
    user_trust: Vec<String>,
    #[serde(default)]
    admin_trust: Vec<String>,
    mozilla: Option<bool>,
    #[serde(default)]
    revocation: Vec<crate::pkix::revocation::RevocationResult>,
    #[serde(default)]
    hard_fail: bool,
    local_case_insensitive: Option<bool>,
}

/// Verify exact CMS bytes and return chain, sender and revocation evidence.
#[wasm_bindgen(js_name = verifySmime)]
pub fn verify_smime(options: JsValue) -> Result<JsValue, JsValue> {
    let i: VerifySmimeIn = from_js(options)?;
    let out = crate::smime_cms::verify(
        &i.cms.to_bytes()?,
        &crate::smime_cms::VerifyOptions {
            content: i.content.map(|c| c.to_bytes()).transpose()?,
            sender: i.sender,
            extra_certs: i.extra_certs.into_iter().map(String::into_bytes).collect(),
            trust: crate::smime_trust::TrustConfig {
                user: i.user_trust.into_iter().map(String::into_bytes).collect(),
                admin: i.admin_trust.into_iter().map(String::into_bytes).collect(),
                mozilla: i.mozilla.unwrap_or(true),
            },
            revocation: i.revocation,
            hard_fail: i.hard_fail,
            local_case_insensitive: i.local_case_insensitive.unwrap_or(true),
            now: unix_now(),
            ..Default::default()
        },
    )
    .map_err(js_err)?;
    to_js(&out)
}

#[wasm_bindgen(js_name = smimeRootSetInfo)]
pub fn smime_root_set_info(_options: JsValue) -> Result<JsValue, JsValue> {
    to_js(&crate::smime_trust::root_set_info())
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
    addresses: Vec<String>,
    algorithm: String,
    expires_at: String,
}

/// `importPkcs12({p12Bytes, password})` → `{ certPem, fingerprint,
/// encryptedPrivateBundle, addresses, algorithm, expiresAt }` — S/MIME private-key
/// material, client-side only. The last three are read from the certificate.
#[wasm_bindgen(js_name = importPkcs12)]
pub fn import_pkcs12(options: JsValue) -> Result<JsValue, JsValue> {
    let i: ImportPkcs12In = from_js(options)?;
    let r = smime::import_pkcs12(&i.p12_bytes, &i.password).map_err(js_err)?;
    to_js(&ImportPkcs12Out {
        cert_pem: r.cert_pem,
        fingerprint: r.fingerprint,
        encrypted_private_bundle: r.encrypted_private_bundle,
        addresses: r.addresses,
        algorithm: r.algorithm,
        expires_at: r.not_after,
    })
}

// ── exportPkcs12 / certificateRequest / attachIssuedCert (S/MIME own keys) ───

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SmimeKeyIn {
    cert_pem: String,
    encrypted_private_bundle: String,
    passphrase: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportPkcs12Out {
    p12_base64: String,
}

/// `exportPkcs12({certPem, encryptedPrivateBundle, passphrase})` →
/// `{ p12Base64 }` — the certificate and key as a PKCS#12 file protected by the
/// same passphrase (see [`smime::export_pkcs12`] for what the file does and does
/// not contain). Base64, so the result is a plain string on its way out of the
/// worker.
#[wasm_bindgen(js_name = exportPkcs12)]
pub fn export_pkcs12(options: JsValue) -> Result<JsValue, JsValue> {
    use base64::Engine;
    let i: SmimeKeyIn = from_js(options)?;
    let der = smime::export_pkcs12(&i.cert_pem, &i.encrypted_private_bundle, &i.passphrase)
        .map_err(js_err)?;
    to_js(&ExportPkcs12Out {
        p12_base64: base64::engine::general_purpose::STANDARD.encode(der),
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CertificateRequestOut {
    csr_pem: String,
}

/// `certificateRequest({certPem, encryptedPrivateBundle, passphrase})` →
/// `{ csrPem }` — a PKCS#10 request a certificate authority can issue from.
#[wasm_bindgen(js_name = certificateRequest)]
pub fn certificate_request(options: JsValue) -> Result<JsValue, JsValue> {
    let i: SmimeKeyIn = from_js(options)?;
    let csr_pem =
        smime::certificate_request(&i.cert_pem, &i.encrypted_private_bundle, &i.passphrase)
            .map_err(js_err)?;
    to_js(&CertificateRequestOut { csr_pem })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AttachIssuedCertIn {
    /// One certificate, PEM text or DER.
    cert_bytes: Vec<u8>,
    encrypted_private_bundle: String,
    passphrase: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AttachIssuedCertOut {
    cert_pem: String,
    fingerprint: String,
    addresses: Vec<String>,
    algorithm: String,
    expires_at: String,
    self_issued: bool,
}

/// `attachIssuedCert({certBytes, encryptedPrivateBundle, passphrase})` →
/// `{ certPem, fingerprint, addresses, algorithm, expiresAt, selfIssued }`. Throws
/// when the certificate's public key is not the key in the bundle.
#[wasm_bindgen(js_name = attachIssuedCert)]
pub fn attach_issued_cert(options: JsValue) -> Result<JsValue, JsValue> {
    let i: AttachIssuedCertIn = from_js(options)?;
    let c = smime::attach_issued_cert(&i.cert_bytes, &i.encrypted_private_bundle, &i.passphrase)
        .map_err(js_err)?;
    to_js(&AttachIssuedCertOut {
        cert_pem: c.cert_pem,
        fingerprint: c.fingerprint,
        addresses: c.addresses,
        algorithm: c.algorithm,
        expires_at: c.not_after,
        self_issued: c.self_issued,
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
    #[serde(skip_serializing_if = "Option::is_none")]
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
    kind: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportBackupOut {
    autocrypt_setup_message: String,
}

/// `exportBackup({encryptedPrivateBundle, kind:"pgp"})` → `{ autocryptSetupMessage }`.
/// An Autocrypt Setup Message is an OpenPGP format, so any other kind is refused
/// (an S/MIME key leaves through `exportPkcs12`).
#[wasm_bindgen(js_name = exportBackup)]
pub fn export_backup(options: JsValue) -> Result<JsValue, JsValue> {
    let i: ExportBackupIn = from_js(options)?;
    if i.kind != "pgp" {
        return Err(js_err(format!(
            "exportBackup: an Autocrypt Setup Message holds an OpenPGP key, not a {} key",
            i.kind
        )));
    }
    let asm = pgp::autocrypt_setup_message(&i.encrypted_private_bundle).map_err(js_err)?;
    to_js(&ExportBackupOut {
        autocrypt_setup_message: asm,
    })
}

// ── OpenPGP core (t32-e37) ───────────────────────────────────────────────────

/// Message bytes from JS: a string (taken as UTF-8) or a `Uint8Array`. Kept as the
/// raw `JsValue` and converted by hand, because serde's untagged enums go through
/// `deserialize_any`, which `serde-wasm-bindgen` refuses for a `Uint8Array`.
#[derive(Deserialize)]
#[serde(transparent)]
struct Raw(#[serde(with = "serde_wasm_bindgen::preserve")] JsValue);

impl Raw {
    fn to_bytes(&self) -> Result<Vec<u8>, JsValue> {
        use wasm_bindgen::JsCast;
        if let Some(s) = self.0.as_string() {
            return Ok(s.into_bytes());
        }
        if let Some(a) = self.0.dyn_ref::<js_sys::Uint8Array>() {
            return Ok(a.to_vec());
        }
        // A typed array from another realm (an iframe, or jsdom under test) fails
        // the `instanceof` check; anything array-like with a length is copied.
        let has_length = self.0.is_object()
            && js_sys::Reflect::get(&self.0, &JsValue::from_str("length"))
                .ok()
                .and_then(|l| l.as_f64())
                .is_some();
        if has_length {
            return Ok(js_sys::Uint8Array::new(&self.0).to_vec());
        }
        Err(js_err("expected a string or a Uint8Array"))
    }
}

/// The signing key for a PGP/MIME build: a session key ref, or bundle +
/// passphrase.
fn signer_from(
    key_ref: Option<&str>,
    bundle: Option<String>,
    passphrase: Option<String>,
) -> Option<(String, String)> {
    match key_ref.and_then(session_lookup) {
        Some(found) => Some(found),
        None => match (bundle, passphrase) {
            (Some(b), Some(p)) => Some((b, p)),
            _ => None,
        },
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ArmoredIn {
    armored: String,
}

/// `inspectKey({armored})` → `PgpKeyInfo`: version, algorithm, creation, expiry,
/// state (`valid`/`expired`/`revoked`/`invalid`/`not-yet-valid`), flags, User IDs
/// and subkeys, each judged from verified self-signatures.
#[wasm_bindgen(js_name = inspectKey)]
pub fn inspect_key(options: JsValue) -> Result<JsValue, JsValue> {
    let i: ArmoredIn = from_js(options)?;
    to_js(&pgp::inspect_key(&i.armored).map_err(js_err)?)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VerifyPgpIn {
    /// The signed data (detached), or absent for a cleartext-signed `signature`.
    data: Option<Raw>,
    signature: String,
    signer_public_keys: Vec<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct VerifyPgpOut {
    verification: Verification,
    /// For a cleartext signature: the signed text, un-escaped.
    #[serde(skip_serializing_if = "Option::is_none")]
    signed_text: Option<String>,
}

/// `verifyPgp({data?, signature, signerPublicKeys})` → `{verification,
/// signedText?}`. With `data` the signature is detached (armored or binary);
/// without, `signature` is a `PGP SIGNED MESSAGE`.
#[wasm_bindgen(js_name = verifyPgp)]
pub fn verify_pgp(options: JsValue) -> Result<JsValue, JsValue> {
    let i: VerifyPgpIn = from_js(options)?;
    let signers: Vec<&str> = i.signer_public_keys.iter().map(String::as_str).collect();
    let out = match &i.data {
        Some(data) => VerifyPgpOut {
            verification: pgp::verify_detached_with(&data.to_bytes()?, &i.signature, &signers)
                .map_err(js_err)?,
            signed_text: None,
        },
        None => {
            let (verification, text) =
                pgp::verify_cleartext(&i.signature, &signers).map_err(js_err)?;
            VerifyPgpOut {
                verification,
                signed_text: Some(text),
            }
        }
    };
    to_js(&out)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManageIn {
    encrypted_private_bundle: String,
    passphrase: String,
    #[serde(default)]
    reason: pgp::manage::RevocationReason,
    #[serde(default)]
    message: String,
    /// Unix seconds; `null`/absent = never.
    expires_at: Option<u64>,
    purpose: Option<pgp::manage::SubkeyPurpose>,
    subkey_fingerprint: Option<String>,
}

/// `revokeKey({encryptedPrivateBundle, passphrase, reason, message})` →
/// `{publicKeyArmored, encryptedPrivateBundle}`. `reason`: `compromised` |
/// `superseded` | `retired` | `unspecified`.
#[wasm_bindgen(js_name = revokeKey)]
pub fn revoke_key(options: JsValue) -> Result<JsValue, JsValue> {
    let i: ManageIn = from_js(options)?;
    to_js(
        &pgp::manage::revoke(
            &i.encrypted_private_bundle,
            &i.passphrase,
            i.reason,
            &i.message,
        )
        .map_err(js_err)?,
    )
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CertificateOut {
    certificate: String,
}

/// `revocationCertificate({encryptedPrivateBundle, passphrase, reason, message})`
/// → `{certificate}` (armored; keep it aside to revoke the key later).
#[wasm_bindgen(js_name = revocationCertificate)]
pub fn revocation_certificate(options: JsValue) -> Result<JsValue, JsValue> {
    let i: ManageIn = from_js(options)?;
    let certificate = pgp::manage::revocation_certificate(
        &i.encrypted_private_bundle,
        &i.passphrase,
        i.reason,
        &i.message,
    )
    .map_err(js_err)?;
    to_js(&CertificateOut { certificate })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApplyRevocationIn {
    key_armored: String,
    certificate: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PublicOut {
    public_key_armored: String,
}

/// `applyRevocationCertificate({keyArmored, certificate})` → `{publicKeyArmored}`.
#[wasm_bindgen(js_name = applyRevocationCertificate)]
pub fn apply_revocation_certificate(options: JsValue) -> Result<JsValue, JsValue> {
    let i: ApplyRevocationIn = from_js(options)?;
    let public_key_armored =
        pgp::manage::apply_revocation_certificate(&i.key_armored, &i.certificate)
            .map_err(js_err)?;
    to_js(&PublicOut { public_key_armored })
}

/// `setKeyExpiry({encryptedPrivateBundle, passphrase, expiresAt})` →
/// `{publicKeyArmored, encryptedPrivateBundle}`; applies to the key and its
/// subkeys.
#[wasm_bindgen(js_name = setKeyExpiry)]
pub fn set_key_expiry(options: JsValue) -> Result<JsValue, JsValue> {
    let i: ManageIn = from_js(options)?;
    to_js(
        &pgp::manage::set_expiry(&i.encrypted_private_bundle, &i.passphrase, i.expires_at)
            .map_err(js_err)?,
    )
}

/// `addSubkey({encryptedPrivateBundle, passphrase, purpose, expiresAt?})` →
/// `{publicKeyArmored, encryptedPrivateBundle}`. `purpose`: `encrypt` | `sign`.
#[wasm_bindgen(js_name = addSubkey)]
pub fn add_subkey(options: JsValue) -> Result<JsValue, JsValue> {
    let i: ManageIn = from_js(options)?;
    let purpose = i
        .purpose
        .ok_or_else(|| js_err("addSubkey requires purpose"))?;
    to_js(
        &pgp::manage::add_subkey(
            &i.encrypted_private_bundle,
            &i.passphrase,
            purpose,
            i.expires_at,
        )
        .map_err(js_err)?,
    )
}

/// `revokeSubkey({encryptedPrivateBundle, passphrase, subkeyFingerprint, reason,
/// message})` → `{publicKeyArmored, encryptedPrivateBundle}`.
#[wasm_bindgen(js_name = revokeSubkey)]
pub fn revoke_subkey(options: JsValue) -> Result<JsValue, JsValue> {
    let i: ManageIn = from_js(options)?;
    let fp = i
        .subkey_fingerprint
        .ok_or_else(|| js_err("revokeSubkey requires subkeyFingerprint"))?;
    to_js(
        &pgp::manage::revoke_subkey(
            &i.encrypted_private_bundle,
            &i.passphrase,
            &fp,
            i.reason,
            &i.message,
        )
        .map_err(js_err)?,
    )
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChangePassphraseIn {
    encrypted_private_bundle: String,
    old_passphrase: String,
    new_passphrase: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BundleOut {
    encrypted_private_bundle: String,
}

/// `changePassphrase({encryptedPrivateBundle, oldPassphrase, newPassphrase})` →
/// `{encryptedPrivateBundle}`.
#[wasm_bindgen(js_name = changePassphrase)]
pub fn change_passphrase(options: JsValue) -> Result<JsValue, JsValue> {
    let i: ChangePassphraseIn = from_js(options)?;
    let encrypted_private_bundle = pgp::manage::change_passphrase(
        &i.encrypted_private_bundle,
        &i.old_passphrase,
        &i.new_passphrase,
    )
    .map_err(js_err)?;
    to_js(&BundleOut {
        encrypted_private_bundle,
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupFullIn {
    encrypted_private_bundle: String,
    /// The account address the Autocrypt Setup Message is addressed to.
    from: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BackupFullOut {
    /// Shown to the user once; needed to restore.
    setup_code: String,
    /// The `application/autocrypt-setup` payload.
    payload: String,
    /// A complete `Autocrypt-Setup-Message: v1` email holding the payload.
    mime: String,
}

/// `exportBackupFull({encryptedPrivateBundle, from})` → `{setupCode, payload,
/// mime}`: the still-passphrase-locked key, symmetrically encrypted under a
/// numeric setup code (Autocrypt Level 1 §4.2).
#[wasm_bindgen(js_name = exportBackupFull)]
pub fn export_backup_full(options: JsValue) -> Result<JsValue, JsValue> {
    let i: BackupFullIn = from_js(options)?;
    let setup_code = pgp::generate_setup_code();
    let payload = pgp::autocrypt_setup_message_full(&i.encrypted_private_bundle, &setup_code)
        .map_err(js_err)?;
    let mime = pgp::autocrypt_setup_message_mime(&i.from, &payload);
    to_js(&BackupFullOut {
        setup_code,
        payload,
        mime,
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RestoreBackupIn {
    payload: String,
    setup_code: String,
}

/// `restoreBackup({payload, setupCode})` → `{encryptedPrivateBundle}` (still locked
/// with the key's own passphrase).
#[wasm_bindgen(js_name = restoreBackup)]
pub fn restore_backup(options: JsValue) -> Result<JsValue, JsValue> {
    let i: RestoreBackupIn = from_js(options)?;
    let encrypted_private_bundle =
        pgp::decrypt_autocrypt_setup_message(&i.payload, &i.setup_code).map_err(js_err)?;
    to_js(&BundleOut {
        encrypted_private_bundle,
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PgpMimeBuildIn {
    /// The body entity to protect (headers + body), as composed.
    payload: Raw,
    /// The message's header fields as `[name, value]` pairs.
    headers: Vec<(String, String)>,
    #[serde(default)]
    recipient_public_keys: Vec<String>,
    sign_with_key_ref: Option<String>,
    encrypted_private_bundle: Option<String>,
    passphrase: Option<String>,
    /// `[address, armoredKey]` pairs written as `Autocrypt-Gossip`.
    #[serde(default)]
    gossip: Vec<(String, String)>,
}

/// `pgpMimeEncrypt({payload, headers, recipientPublicKeys, signWithKeyRef? |
/// encryptedPrivateBundle+passphrase, gossip?})` → `{entity, contentType,
/// outerSubject, seipdVersion}`: an RFC 3156 `multipart/encrypted` entity with RFC
/// 9788 header protection inside (`Subject` outside becomes `[...]`).
#[wasm_bindgen(js_name = pgpMimeEncrypt)]
pub fn pgp_mime_encrypt(options: JsValue) -> Result<JsValue, JsValue> {
    let i: PgpMimeBuildIn = from_js(options)?;
    let signer = signer_from(
        i.sign_with_key_ref.as_deref(),
        i.encrypted_private_bundle,
        i.passphrase,
    );
    let sign_ref = signer.as_ref().map(|(b, p)| (b.as_str(), p.as_str()));
    let built = pgp::pgpmime::encrypt_mime(
        &i.payload.to_bytes()?,
        &i.headers,
        &i.recipient_public_keys,
        sign_ref,
        &i.gossip,
    )
    .map_err(js_err)?;
    to_js(&built)
}

/// `pgpMimeSign({payload, headers, signWithKeyRef? |
/// encryptedPrivateBundle+passphrase})` → `{entity, contentType, outerSubject}`:
/// an RFC 3156 `multipart/signed` entity (`hp="clear"` header protection).
#[wasm_bindgen(js_name = pgpMimeSign)]
pub fn pgp_mime_sign(options: JsValue) -> Result<JsValue, JsValue> {
    let i: PgpMimeBuildIn = from_js(options)?;
    let (bundle, passphrase) = signer_from(
        i.sign_with_key_ref.as_deref(),
        i.encrypted_private_bundle,
        i.passphrase,
    )
    .ok_or_else(|| js_err("pgpMimeSign needs signWithKeyRef or encryptedPrivateBundle"))?;
    let built = pgp::pgpmime::sign_mime(&i.payload.to_bytes()?, &i.headers, &bundle, &passphrase)
        .map_err(js_err)?;
    to_js(&built)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PgpMimeOpenIn {
    raw: Raw,
    encrypted_private_bundle: Option<String>,
    passphrase: Option<String>,
    #[serde(default)]
    signer_public_keys: Vec<String>,
}

/// `pgpMimeOpen({raw, encryptedPrivateBundle, passphrase, signerPublicKeys})` →
/// `{entity, protected, verification, encrypted, aead, partial, gossip}`. The
/// `entity` is MIME; its HTML parts must go through the sanitizer before display.
#[wasm_bindgen(js_name = pgpMimeOpen)]
pub fn pgp_mime_open(options: JsValue) -> Result<JsValue, JsValue> {
    let i: PgpMimeOpenIn = from_js(options)?;
    let signers: Vec<&str> = i.signer_public_keys.iter().map(String::as_str).collect();
    let bundle = i
        .encrypted_private_bundle
        .ok_or_else(|| js_err("pgpMimeOpen requires encryptedPrivateBundle"))?;
    let passphrase = i.passphrase.unwrap_or_default();
    to_js(
        &pgp::pgpmime::open_mime(&i.raw.to_bytes()?, &bundle, &passphrase, &signers)
            .map_err(js_err)?,
    )
}

/// `pgpMimeVerify({raw, signerPublicKeys})` → the `pgpMimeOpen` shape for a
/// signed-only message (`multipart/signed` or an inline cleartext signature).
#[wasm_bindgen(js_name = pgpMimeVerify)]
pub fn pgp_mime_verify(options: JsValue) -> Result<JsValue, JsValue> {
    let i: PgpMimeOpenIn = from_js(options)?;
    let signers: Vec<&str> = i.signer_public_keys.iter().map(String::as_str).collect();
    to_js(&pgp::pgpmime::verify_mime(&i.raw.to_bytes()?, &signers).map_err(js_err)?)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AnalyzeIn {
    raw: Raw,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalyzeOut {
    /// `pgp-encrypted` | `pgp-signed` | `smime-signed` | `smime-opaque` |
    /// `inline-pgp` | `none`.
    kind: &'static str,
    partial: bool,
    path: Vec<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    smime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    micalg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    segments: Option<Vec<mime::InlineSegment>>,
}

/// `analyzeMessage({raw})` → `{kind, partial, path, smimeType?, micalg?,
/// segments?}`: where the message's signed or encrypted part is.
#[wasm_bindgen(js_name = analyzeMessage)]
pub fn analyze_message(options: JsValue) -> Result<JsValue, JsValue> {
    let i: AnalyzeIn = from_js(options)?;
    let a = mime::analyze(&i.raw.to_bytes()?);
    let mut out = AnalyzeOut {
        kind: "none",
        partial: a.partial,
        path: a.path,
        smime_type: None,
        micalg: None,
        segments: None,
    };
    match a.part {
        Some(mime::CryptoPart::PgpEncrypted { .. }) => out.kind = "pgp-encrypted",
        Some(mime::CryptoPart::PgpSigned { micalg, .. }) => {
            out.kind = "pgp-signed";
            out.micalg = micalg;
        }
        Some(mime::CryptoPart::SmimeSigned { micalg, .. }) => {
            out.kind = "smime-signed";
            out.micalg = micalg;
        }
        Some(mime::CryptoPart::SmimeOpaque { smime_type, .. }) => {
            out.kind = "smime-opaque";
            out.smime_type = Some(smime_type);
        }
        Some(mime::CryptoPart::InlinePgp { segments }) => {
            out.kind = "inline-pgp";
            out.segments = Some(segments);
        }
        None => {}
    }
    to_js(&out)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AutocryptHeaderIn {
    public_key_armored: String,
    addr: String,
    #[serde(default)]
    prefer_encrypt: bool,
    #[serde(default)]
    gossip: bool,
}

/// `autocryptHeader({publicKeyArmored, addr, preferEncrypt, gossip?})` → the
/// header value (unfolded) for `Autocrypt:`, or `Autocrypt-Gossip:` with `gossip`.
#[wasm_bindgen(js_name = autocryptHeader)]
pub fn autocrypt_header(options: JsValue) -> Result<JsValue, JsValue> {
    let i: AutocryptHeaderIn = from_js(options)?;
    let value = if i.gossip {
        pgp::autocrypt::gossip_header(&i.public_key_armored, &i.addr)
    } else {
        pgp::autocrypt::autocrypt_header(&i.public_key_armored, &i.addr, i.prefer_encrypt)
    }
    .map_err(js_err)?;
    to_js(&value)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ParseAutocryptIn {
    value: String,
    #[serde(default)]
    gossip: bool,
}

/// `parseAutocrypt({value, gossip?})` → `{addr, preferEncrypt, key}`.
#[wasm_bindgen(js_name = parseAutocrypt)]
pub fn parse_autocrypt(options: JsValue) -> Result<JsValue, JsValue> {
    let i: ParseAutocryptIn = from_js(options)?;
    let h = if i.gossip {
        pgp::autocrypt::parse_gossip(&i.value)
    } else {
        pgp::autocrypt::parse_autocrypt(&i.value)
    }
    .map_err(js_err)?;
    to_js(&h)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecommendIn {
    /// One entry per recipient; `null` when nothing is known.
    peers: Vec<Option<pgp::autocrypt::PeerState>>,
    own_prefer_encrypt: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RecommendOut {
    per_recipient: Vec<&'static str>,
    message: &'static str,
}

/// `autocryptRecommend({peers, ownPreferEncrypt})` → `{perRecipient, message}`,
/// each `disable` | `discourage` | `available` | `encrypt` (Autocrypt §2.4).
#[wasm_bindgen(js_name = autocryptRecommend)]
pub fn autocrypt_recommend(options: JsValue) -> Result<JsValue, JsValue> {
    let i: RecommendIn = from_js(options)?;
    let per: Vec<&'static str> = i
        .peers
        .iter()
        .map(|p| pgp::autocrypt::recommend(p.as_ref(), i.own_prefer_encrypt))
        .collect();
    let message = pgp::autocrypt::recommend_message(&per);
    to_js(&RecommendOut {
        per_recipient: per,
        message,
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AttachedKeysIn {
    content_type: String,
    filename: Option<String>,
    bytes: Raw,
}

/// `detectAttachedKeys({contentType, filename?, bytes})` → `CryptoKey[]` or
/// `null` when the attachment is not a key.
#[wasm_bindgen(js_name = detectAttachedKeys)]
pub fn detect_attached_keys(options: JsValue) -> Result<JsValue, JsValue> {
    let i: AttachedKeysIn = from_js(options)?;
    to_js(&pgp::detect_attached_keys(
        &i.content_type,
        i.filename.as_deref(),
        &i.bytes.to_bytes()?,
    ))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WkdIn {
    email: String,
}

/// `wkdUrls({email})` → `[advancedUrl, directUrl]`.
#[wasm_bindgen(js_name = wkdUrls)]
pub fn wkd_urls(options: JsValue) -> Result<JsValue, JsValue> {
    let i: WkdIn = from_js(options)?;
    to_js(&pgp::wkd_urls(&i.email).map_err(js_err)?)
}

// ── unlockKey / lockKey (session cache) ──────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UnlockIn {
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
    match i.kind.as_str() {
        "pgp" => pgp::manage::verify_passphrase(&i.encrypted_private_bundle, &i.passphrase)
            .map_err(js_err)?,
        "smime" => {
            crate::smime_keys::unwrap(&i.encrypted_private_bundle, &i.passphrase)
                .map_err(js_err)?;
        }
        other => return Err(js_err(format!("unknown kind: {other}"))),
    }
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
