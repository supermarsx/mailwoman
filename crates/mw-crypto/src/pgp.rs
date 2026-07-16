//! OpenPGP (RFC 9580) over rPGP — the `pgp` crate (MIT/Apache, plan §1.1/§1.11).
//! v6 key generation (Ed25519 primary / X25519 encryption subkey), encrypt/sign/
//! decrypt/verify with SEIPDv2 AEAD (OCB) + AES-256 + SHA-256 floor, armored +
//! Autocrypt import/export, Autocrypt L1 header parse/gen, Autocrypt Setup Message
//! backup, and TOFU trust evaluation.
//!
//! The PGP `encryptedPrivateBundle` (frozen §2.3) is the rPGP secret key armored
//! **under S2K passphrase protection** — the private material is never emitted in
//! the clear. Public verify/harvest run native; keygen/decrypt/private-sign run in
//! the browser wasm build (plan §1.2) — but the code is target-agnostic; the
//! feature split is purely which functions each side calls.

use pgp::composed::{
    ArmorOptions, CleartextSignedMessage, Deserializable, DetachedSignature, EncryptionCaps,
    KeyType, Message, MessageBuilder, SecretKeyParamsBuilder, SignedPublicKey, SignedPublicSubKey,
    SignedSecretKey, SubkeyParamsBuilder,
};
use pgp::crypto::aead::AeadAlgorithm;
use pgp::crypto::hash::HashAlgorithm;
use pgp::crypto::sym::SymmetricKeyAlgorithm;
use pgp::ser::Serialize;
use pgp::types::{KeyDetails, KeyVersion, Password, Timestamp};

use crate::error::{CryptoError, Result};
use crate::rng;
use crate::types::{CryptoKey, KeyHistoryEntry, SignatureVerdict};

/// Result of [`generate_key`] — the browser stores the wrapped bundle in its vault
/// and uploads only `public_key_armored` + the opaque bundle (never plaintext).
pub struct GeneratedKey {
    pub public_key_armored: String,
    pub fingerprint: String,
    pub key_id: String,
    pub encrypted_private_bundle: String,
}

/// Result of [`decrypt`].
pub struct Decrypted {
    pub plaintext: Vec<u8>,
    /// The protected-headers `Subject:` if the message carried one (§2.3).
    pub subject: Option<String>,
    pub signature: SignatureVerdict,
}

fn parse(e: impl std::fmt::Display) -> CryptoError {
    CryptoError::Parse(e.to_string())
}

/// Generate a v6 OpenPGP key: Ed25519 primary (certify + sign), X25519 encryption
/// subkey. The returned bundle is the secret key armored under S2K protection with
/// `passphrase`.
pub fn generate_key(user_id: &str, passphrase: &str) -> Result<GeneratedKey> {
    // rPGP reads the wall clock via `std::time::SystemTime`, which panics on
    // `wasm32-unknown-unknown` ("time not implemented"). chrono's `wasmbind` reads
    // the JS clock on wasm, so we source the creation time from it and pass it
    // explicitly — this also fixes the signature-creation timestamp (see `sign_now`).
    let created = now_timestamp();

    let mut enc_subkey = SubkeyParamsBuilder::default();
    enc_subkey
        .version(KeyVersion::V6)
        .key_type(KeyType::X25519)
        .created_at(created)
        .can_encrypt(EncryptionCaps::All)
        .can_sign(false)
        .can_authenticate(false);

    let mut params = SecretKeyParamsBuilder::default();
    params
        .version(KeyVersion::V6)
        .key_type(KeyType::Ed25519)
        .created_at(created)
        .can_certify(true)
        .can_sign(true)
        .can_encrypt(EncryptionCaps::None)
        .primary_user_id(user_id.to_string())
        .subkey(
            enc_subkey
                .build()
                .map_err(|e| CryptoError::KeyGen(e.to_string()))?,
        );

    let secret_params = params
        .build()
        .map_err(|e| CryptoError::KeyGen(e.to_string()))?;

    let mut signed = secret_params
        .generate(rng::pgp_rng())
        .map_err(|e| CryptoError::KeyGen(e.to_string()))?;

    // Fingerprint / key id / armored public half come from the public key (which
    // implements `KeyDetails`); `SignedSecretKey` itself does not.
    let public = signed.to_public_key();
    let fingerprint = fingerprint_hex(&public);
    let key_id = public.legacy_key_id().to_string();
    let public_key_armored = public
        .to_armored_string(ArmorOptions::default())
        .map_err(|e| CryptoError::KeyGen(e.to_string()))?;

    // Lock every secret component (primary + subkeys) under the passphrase (S2K)
    // before the key can be serialized — the decryption secret lives in the
    // encryption subkey, so it must be locked too, not just the primary.
    signed
        .primary_key
        .set_password(rng::pgp_rng(), &pw(passphrase))
        .map_err(|e| CryptoError::KeyGen(e.to_string()))?;
    for sub in &mut signed.secret_subkeys {
        sub.key
            .set_password(rng::pgp_rng(), &pw(passphrase))
            .map_err(|e| CryptoError::KeyGen(e.to_string()))?;
    }
    let encrypted_private_bundle = signed
        .to_armored_string(ArmorOptions::default())
        .map_err(|e| CryptoError::KeyGen(e.to_string()))?;

    Ok(GeneratedKey {
        public_key_armored,
        fingerprint,
        key_id,
        encrypted_private_bundle,
    })
}

/// Encrypt `plaintext` (SEIPDv2 / OCB / AES-256) to every recipient in
/// `recipient_public_keys` (armored), optionally signing with the passphrase-locked
/// `sign_with` bundle. Returns armored ciphertext.
pub fn encrypt(
    plaintext: &[u8],
    recipient_public_keys: &[String],
    sign_with: Option<(&str, &str)>,
) -> Result<String> {
    if recipient_public_keys.is_empty() {
        return Err(CryptoError::Input("no recipients".into()));
    }
    let recipients: Vec<SignedPublicKey> = recipient_public_keys
        .iter()
        .map(|a| parse_public(a))
        .collect::<Result<_>>()?;

    let mut builder = MessageBuilder::from_bytes("", plaintext.to_vec()).seipd_v2(
        rng::pgp_rng(),
        SymmetricKeyAlgorithm::AES256,
        AeadAlgorithm::Ocb,
        Default::default(),
    );

    // Optional signing key must outlive the builder borrow.
    let signer = match sign_with {
        Some((bundle, passphrase)) => Some((parse_secret(bundle)?, passphrase.to_string())),
        None => None,
    };
    if let Some((ref ssk, ref passphrase)) = signer {
        builder.sign(&ssk.primary_key, pw(passphrase), HashAlgorithm::Sha256);
    }

    let subkeys: Vec<SignedPublicSubKey> = recipients
        .iter()
        .map(encryption_subkey)
        .collect::<Result<_>>()?;
    for enc in &subkeys {
        builder
            .encrypt_to_key(rng::pgp_rng(), enc)
            .map_err(|e| CryptoError::Encrypt(e.to_string()))?;
    }

    builder
        .to_armored_string(rng::pgp_rng(), ArmorOptions::default())
        .map_err(|e| CryptoError::Encrypt(e.to_string()))
}

/// Decrypt an armored/binary PGP message with the passphrase-locked `bundle`.
/// Verifies an inline signature against `signer_public_key` when provided and
/// extracts a protected-headers `Subject:` when present.
pub fn decrypt(
    ciphertext: &[u8],
    encrypted_private_bundle: &str,
    passphrase: &str,
    signer_public_key: Option<&str>,
) -> Result<Decrypted> {
    let ssk = parse_secret(encrypted_private_bundle)?;
    // Parse armored or binary; both borrow `ciphertext`, which outlives this fn.
    let armored_str;
    let message = if ciphertext.starts_with(b"-----BEGIN") {
        armored_str = std::str::from_utf8(ciphertext).map_err(parse)?;
        Message::from_string(armored_str).map_err(parse)?.0
    } else {
        Message::from_bytes(ciphertext).map_err(parse)?
    };

    let mut decrypted = message
        .decrypt(&pw(passphrase), &ssk)
        .map_err(|e| CryptoError::Decrypt(e.to_string()))?;
    if decrypted.is_compressed() {
        decrypted = decrypted
            .decompress()
            .map_err(|e| CryptoError::Decrypt(e.to_string()))?;
    }

    let data = decrypted
        .as_data_vec()
        .map_err(|e| CryptoError::Decrypt(e.to_string()))?;

    let signature = match signer_public_key {
        Some(pubkey) => {
            let cert = parse_public(pubkey)?;
            match decrypted.verify(&cert) {
                Ok(_) => verdict_verified(&cert),
                Err(_) => verdict("invalid", None),
            }
        }
        None => verdict("none", None),
    };

    let (subject, body) = unwrap_protected_headers(&data);
    Ok(Decrypted {
        plaintext: body,
        subject,
        signature,
    })
}

/// Produce a detached, armored signature over `data` with the locked `bundle`.
pub fn sign_detached(
    data: &[u8],
    encrypted_private_bundle: &str,
    passphrase: &str,
) -> Result<String> {
    let ssk = parse_secret(encrypted_private_bundle)?;
    let sig = DetachedSignature::sign_binary_data(
        rng::pgp_rng(),
        &ssk.primary_key,
        &pw(passphrase),
        HashAlgorithm::Sha256,
        data,
    )
    .map_err(|e| CryptoError::Sign(e.to_string()))?;
    sig.to_armored_string(ArmorOptions::default())
        .map_err(|e| CryptoError::Sign(e.to_string()))
}

/// Produce an inline cleartext-signed message (RFC 9580 Cleartext Signature
/// Framework) over `data` with the locked `bundle`. Unlike [`sign_detached`], the
/// returned armor is a complete `-----BEGIN PGP SIGNED MESSAGE-----` block: the
/// original text (dash-escaped where required) stays readable inline and carries
/// the `Hash:` armor header + signature that an OpenPGP verifier accepts. rPGP owns
/// the canonicalization (CRLF normalization, trailing-whitespace, dash-escaping),
/// so the body is never discarded (the sign-only wire hole).
pub fn clear_sign(data: &str, encrypted_private_bundle: &str, passphrase: &str) -> Result<String> {
    let ssk = parse_secret(encrypted_private_bundle)?;
    let signed = CleartextSignedMessage::sign(
        rng::pgp_rng(),
        data,
        &ssk.primary_key,
        &pw(passphrase),
    )
    .map_err(|e| CryptoError::Sign(e.to_string()))?;
    signed
        .to_armored_string(ArmorOptions::default())
        .map_err(|e| CryptoError::Sign(e.to_string()))
}

/// Verify an inline cleartext-signed message (`PGP SIGNED MESSAGE`, produced by
/// [`clear_sign`]) against `signer_public_key`, returning `(verdict, cleartext)`.
/// The cleartext is the recovered, un-escaped message body.
pub fn verify_clear_signed(
    armored: &str,
    signer_public_key: &str,
) -> Result<(SignatureVerdict, String)> {
    let cert = parse_public(signer_public_key)?;
    let (msg, _) = CleartextSignedMessage::from_string(armored).map_err(parse)?;
    let verdict = match msg.verify(&cert) {
        Ok(_) => verdict_verified(&cert),
        Err(_) => verdict("invalid", None),
    };
    // `signed_text` is the de-dash-escaped, CRLF-normalized content the signature
    // actually covers (`text()` would return the dash-escaped transport form).
    Ok((verdict, msg.signed_text()))
}

/// Verify a detached armored signature over `data` against `signer_public_key`.
pub fn verify_detached(
    data: &[u8],
    signature_armored: &str,
    signer_public_key: &str,
) -> Result<SignatureVerdict> {
    let cert = parse_public(signer_public_key)?;
    let (sig, _) = DetachedSignature::from_string(signature_armored).map_err(parse)?;
    match sig.verify(&cert, data) {
        Ok(_) => Ok(verdict_verified(&cert)),
        Err(_) => Ok(verdict("invalid", None)),
    }
}

/// Parse an armored PGP key (public or secret) into a [`CryptoKey`] for the keyring.
/// If the armor carries a secret key, `has_private` is set and the (still
/// passphrase-locked, if it was) armored secret is returned as the bundle.
pub fn parse_key(armored: &str, addresses: Vec<String>) -> Result<(CryptoKey, Option<String>)> {
    // Try secret first (a TSK contains the public half too).
    if let Ok((ssk, _)) = SignedSecretKey::from_string(armored) {
        let public = ssk.to_public_key();
        let public_key_armored = public
            .to_armored_string(ArmorOptions::default())
            .map_err(parse)?;
        let key = build_crypto_key(&public, &public_key_armored, addresses, true, "imported");
        return Ok((key, Some(armored.to_string())));
    }
    let cert = parse_public(armored)?;
    let key = build_crypto_key(&cert, armored, addresses, false, "imported");
    Ok((key, None))
}

/// Export the armored public key from a locked secret bundle (or pass through an
/// armored public key).
pub fn export_public(bundle_or_public: &str) -> Result<String> {
    if let Ok((ssk, _)) = SignedSecretKey::from_string(bundle_or_public) {
        return ssk
            .to_public_key()
            .to_armored_string(ArmorOptions::default())
            .map_err(parse);
    }
    let cert = parse_public(bundle_or_public)?;
    cert.to_armored_string(ArmorOptions::default())
        .map_err(parse)
}

/// Generate an Autocrypt Level 1 header value (`addr=...; keydata=<base64>`) for a
/// public key. `prefer_encrypt` toggles the `prefer-encrypt=mutual` attribute.
pub fn autocrypt_header(
    public_key_armored: &str,
    addr: &str,
    prefer_encrypt: bool,
) -> Result<String> {
    use base64::Engine;
    let cert = parse_public(public_key_armored)?;
    let bytes = cert.to_bytes().map_err(parse)?;
    let keydata = base64::engine::general_purpose::STANDARD.encode(bytes);
    let prefer = if prefer_encrypt {
        "prefer-encrypt=mutual; "
    } else {
        ""
    };
    Ok(format!("addr={addr}; {prefer}keydata={keydata}"))
}

/// Parse an Autocrypt Level 1 header value into a [`CryptoKey`] (`source =
/// "autocrypt-header"`).
pub fn parse_autocrypt_header(header_value: &str) -> Result<CryptoKey> {
    use base64::Engine;
    let mut addr = None;
    let mut keydata = None;
    for part in header_value.split(';') {
        let part = part.trim();
        if let Some(v) = part.strip_prefix("addr=") {
            addr = Some(v.trim().to_string());
        } else if let Some(v) = part.strip_prefix("keydata=") {
            keydata = Some(v.split_whitespace().collect::<String>());
        }
    }
    let keydata = keydata.ok_or_else(|| CryptoError::Parse("autocrypt: no keydata".into()))?;
    let der = base64::engine::general_purpose::STANDARD
        .decode(keydata.as_bytes())
        .map_err(parse)?;
    let cert = SignedPublicKey::from_bytes(std::io::Cursor::new(der)).map_err(parse)?;
    let armored = cert
        .to_armored_string(ArmorOptions::default())
        .map_err(parse)?;
    let addresses = addr.into_iter().collect();
    let mut key = build_crypto_key(&cert, &armored, addresses, false, "autocrypt-header");
    key.autocrypt = true;
    Ok(key)
}

/// Wrap the passphrase-locked secret bundle as an Autocrypt Setup Message backup.
/// The bundle is already S2K-protected, so the ASM payload is exactly that armored,
/// passphrase-protected key (Autocrypt §4.2 transport container).
pub fn autocrypt_setup_message(encrypted_private_bundle: &str) -> Result<String> {
    let ssk = parse_secret(encrypted_private_bundle)?;
    ssk.to_armored_string(ArmorOptions::default())
        .map_err(parse)
}

/// Evaluate a TOFU trust transition given the currently-recorded fingerprint (if
/// any) and a freshly-seen one. Returns the new trust token + whether the key
/// changed (a key-change alert, §2.1 `signature.keyChanged`).
pub fn tofu_eval(recorded_fingerprint: Option<&str>, seen_fingerprint: &str) -> (String, bool) {
    match recorded_fingerprint {
        None => ("tofu".to_string(), false),
        Some(prev) if prev.eq_ignore_ascii_case(seen_fingerprint) => ("tofu".to_string(), false),
        Some(_) => ("unverified".to_string(), true),
    }
}

// ── Protected headers (§2.3 protected/encrypted subject) ─────────────────────

const PROTECTED_HEADERS_CT: &str = "Content-Type: text/plain; protected-headers=\"v1\"";

/// Frame `body` with a protected-headers stanza carrying `subject` (memoryhole /
/// protected-headers). The whole stanza becomes the encrypted plaintext so the
/// subject travels inside the ciphertext.
pub fn wrap_protected_headers(subject: &str, body: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(PROTECTED_HEADERS_CT.as_bytes());
    out.extend_from_slice(b"\r\nSubject: ");
    out.extend_from_slice(subject.as_bytes());
    out.extend_from_slice(b"\r\n\r\n");
    out.extend_from_slice(body);
    out
}

/// Recover `(subject, body)` from a protected-headers stanza produced by
/// [`wrap_protected_headers`]; returns `(None, data)` when no stanza is present.
pub fn unwrap_protected_headers(data: &[u8]) -> (Option<String>, Vec<u8>) {
    if !data.starts_with(PROTECTED_HEADERS_CT.as_bytes()) {
        return (None, data.to_vec());
    }
    if let Some(pos) = find_subslice(data, b"\r\n\r\n") {
        let headers = &data[..pos];
        let body = &data[pos + 4..];
        let subject = std::str::from_utf8(headers)
            .ok()
            .and_then(|h| h.lines().find_map(|l| l.strip_prefix("Subject: ")))
            .map(str::to_string);
        return (subject, body.to_vec());
    }
    (None, data.to_vec())
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

// ── WKD (native — HTTPS GET; pure URL derivation is testable offline) ────────

/// Derive the WKD (Web Key Directory) URL for an email address: the advanced method
/// hosts under `openpgpkey.<domain>`, the direct method under `<domain>`; the local
/// part is `zbase32(SHA-1(lowercase local part))`. Pure — no network.
pub fn wkd_url(email: &str, advanced: bool) -> Result<String> {
    let (local, domain) = email
        .rsplit_once('@')
        .ok_or_else(|| CryptoError::Input("invalid email".into()))?;
    let local_lower = local.to_lowercase();
    let domain_lower = domain.to_lowercase();
    let hash = {
        use sha1::{Digest, Sha1};
        let mut h = Sha1::new();
        h.update(local_lower.as_bytes());
        h.finalize()
    };
    let encoded = zbase32_encode(&hash);
    let local_enc = urlencode(&local_lower);
    if advanced {
        Ok(format!(
            "https://openpgpkey.{domain_lower}/.well-known/openpgpkey/{domain_lower}/hu/{encoded}?l={local_enc}"
        ))
    } else {
        Ok(format!(
            "https://{domain_lower}/.well-known/openpgpkey/hu/{encoded}?l={local_enc}"
        ))
    }
}

/// Fetch a WKD key (native only — HTTPS GET, no keyserver fallback).
#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub async fn wkd_fetch(email: &str) -> Result<CryptoKey> {
    let url = wkd_url(email, true)?;
    let resp = reqwest::get(&url)
        .await
        .map_err(|e| CryptoError::Io(e.to_string()))?;
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| CryptoError::Io(e.to_string()))?;
    let cert = SignedPublicKey::from_bytes(std::io::Cursor::new(bytes.to_vec())).map_err(parse)?;
    let armored = cert
        .to_armored_string(ArmorOptions::default())
        .map_err(parse)?;
    Ok(build_crypto_key(
        &cert,
        &armored,
        vec![email.to_string()],
        false,
        "wkd",
    ))
}

/// z-base-32 encode (WKD alphabet). Encodes 5-bit groups big-endian.
fn zbase32_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ybndrfg8ejkmcpqxot1uwisza345h769";
    let mut out = String::new();
    let mut buffer: u32 = 0;
    let mut bits = 0u32;
    for &byte in data {
        buffer = (buffer << 8) | u32::from(byte);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            let idx = ((buffer >> bits) & 0x1f) as usize;
            out.push(ALPHABET[idx] as char);
        }
    }
    if bits > 0 {
        let idx = ((buffer << (5 - bits)) & 0x1f) as usize;
        out.push(ALPHABET[idx] as char);
    }
    out
}

/// Minimal percent-encoding for the WKD `l=` query parameter.
fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn pw(passphrase: &str) -> Password {
    passphrase.into()
}

/// The current time as an rPGP [`Timestamp`], sourced from chrono (whose `wasmbind`
/// reads the JS clock) so it never touches `std::time::SystemTime`, which is
/// unimplemented on `wasm32-unknown-unknown`.
fn now_timestamp() -> Timestamp {
    let secs = chrono::Utc::now().timestamp().clamp(0, i64::from(u32::MAX)) as u32;
    Timestamp::from_secs(secs)
}

fn parse_public(armored: &str) -> Result<SignedPublicKey> {
    let (cert, _) = SignedPublicKey::from_string(armored).map_err(parse)?;
    Ok(cert)
}

fn parse_secret(armored: &str) -> Result<SignedSecretKey> {
    let (ssk, _) = SignedSecretKey::from_string(armored).map_err(parse)?;
    Ok(ssk)
}

fn encryption_subkey(cert: &SignedPublicKey) -> Result<SignedPublicSubKey> {
    cert.public_subkeys
        .iter()
        .find(|sk| sk.algorithm().can_encrypt())
        .cloned()
        .ok_or_else(|| CryptoError::Input("recipient key has no encryption subkey".into()))
}

fn fingerprint_hex(key: &impl KeyDetails) -> String {
    hex::encode(key.fingerprint().as_bytes()).to_uppercase()
}

fn build_crypto_key(
    key: &impl KeyDetails,
    public_key_armored: &str,
    addresses: Vec<String>,
    has_private: bool,
    source: &str,
) -> CryptoKey {
    let fingerprint = fingerprint_hex(key);
    CryptoKey {
        id: format!("pgp:{fingerprint}"),
        kind: "pgp".into(),
        is_own: has_private,
        addresses,
        fingerprint: fingerprint.clone(),
        key_id: key.legacy_key_id().to_string(),
        algorithm: format!("{:?}", key.algorithm()).to_lowercase(),
        created_at: chrono::Utc::now().to_rfc3339(),
        expires_at: None,
        public_key_armored: Some(public_key_armored.to_string()),
        cert_pem: None,
        trust: if has_private {
            "verified"
        } else {
            "unverified"
        }
        .into(),
        autocrypt: false,
        source: source.into(),
        has_private,
        encrypted_private_backup: None,
        verified_at: None,
        key_history: vec![KeyHistoryEntry {
            fingerprint,
            seen_at: chrono::Utc::now().to_rfc3339(),
        }],
    }
}

fn verdict(status: &str, signer_key_id: Option<String>) -> SignatureVerdict {
    SignatureVerdict {
        kind: "pgp".into(),
        status: status.into(),
        signer_key_id,
        algorithm: Some("ed25519".into()),
        key_created_at: None,
        key_expires_at: None,
        chain_status: None,
        revocation_status: None,
        key_changed: false,
    }
}

fn verdict_verified(cert: &SignedPublicKey) -> SignatureVerdict {
    verdict("verified", Some(cert.legacy_key_id().to_string()))
}
