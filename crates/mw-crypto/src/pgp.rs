//! OpenPGP (RFC 9580) over rPGP — the `pgp` crate (MIT/Apache, plan §1.1/§1.11).
//!
//! - Key generation: v6 (Ed25519 primary / X25519 encryption subkey, the default) or
//!   the "compatible" v4 profile (EdDSA + ECDH Curve25519, what GnuPG 2.2–2.4 read).
//! - Encryption is recipient-aware (t32-e37, DA32): SEIPDv2 (AES-256/OCB) when every
//!   recipient is a v6 key or advertises SEIPDv2, else SEIPDv1 (AES-256 + MDC) so a
//!   v4-only reader such as GnuPG 2.4 can decrypt.
//! - Keys are used only through [`cert`]: verified self-signatures decide flags,
//!   expiry, revocation and subkey binding; verdicts carry the real algorithm and
//!   creation / expiry times ([`crate::verdict`]).
//! - Key management lives in [`manage`], Autocrypt in [`autocrypt`], PGP/MIME
//!   structures in [`crate::mime`].
//!
//! The PGP `encryptedPrivateBundle` (frozen §2.3) is the rPGP secret key armored
//! **under S2K passphrase protection** — the private material is never emitted in
//! the clear. Public verify/harvest run native; keygen/decrypt/private-sign run in
//! the browser wasm build (plan §1.2) — but the code is target-agnostic; the
//! feature split is purely which functions each side calls.

pub mod autocrypt;
pub mod cert;
pub mod manage;
pub mod pgpmime;

use pgp::composed::{
    ArmorOptions, CleartextSignedMessage, Deserializable, DetachedSignature, EncryptionCaps,
    KeyType, Message, MessageBuilder, SecretKeyParamsBuilder, SignedPublicKey, SignedSecretKey,
    SubkeyParamsBuilder,
};
use pgp::crypto::aead::AeadAlgorithm;
use pgp::crypto::ecc_curve::ECCCurve;
use pgp::crypto::hash::HashAlgorithm;
use pgp::crypto::sym::SymmetricKeyAlgorithm;
use pgp::packet::Signature;
use pgp::ser::Serialize as _;
use pgp::types::{CompressionAlgorithm, KeyDetails, KeyVersion, Password, SigningKey, Timestamp};
use serde::{Deserialize, Serialize};
use smallvec::smallvec;

use crate::clock;
use crate::error::{CryptoError, Result};
use crate::rng;
use crate::types::{CryptoKey, KeyHistoryEntry, SignatureVerdict};
use crate::verdict::{self, Verification};

pub use autocrypt::{autocrypt_header, parse_autocrypt_header};
pub use cert::{PgpKeyInfo, PgpSubkeyInfo, PgpUserId, Revocation};

/// Result of [`generate_key`] — the browser stores the wrapped bundle in its vault
/// and uploads only `public_key_armored` + the opaque bundle (never plaintext).
pub struct GeneratedKey {
    pub public_key_armored: String,
    pub fingerprint: String,
    pub key_id: String,
    pub encrypted_private_bundle: String,
}

/// Which kind of key [`generate_key_with`] makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum KeyProfile {
    /// v6: Ed25519 primary (certify + sign), X25519 encryption subkey (RFC 9580).
    #[default]
    V6,
    /// v4: EdDSA (Ed25519) primary, ECDH Curve25519 encryption subkey — the key type
    /// GnuPG 2.2 to 2.4 generate and read. Advertises SEIPDv1 only.
    Compatible,
}

/// Options for [`generate_key_with`].
#[derive(Debug, Clone, Default)]
pub struct GenerateOptions {
    pub profile: KeyProfile,
    /// Seconds after creation when the key expires; `None` = no expiry.
    pub expires_in_secs: Option<u64>,
}

/// Result of [`decrypt`].
pub struct Decrypted {
    pub plaintext: Vec<u8>,
    /// The protected-headers `Subject:` if the message carried one (§2.3).
    pub subject: Option<String>,
    /// The frozen verdict (same as `verification.signature`).
    pub signature: SignatureVerdict,
    /// The full signature state.
    pub verification: Verification,
    /// The message used SEIPDv2 (AEAD) rather than SEIPDv1.
    pub aead: bool,
}

/// Result of [`encrypt_with`].
pub struct Encrypted {
    pub armored: String,
    /// 1 (SEIPDv1, AES-256 + MDC) or 2 (SEIPDv2, AES-256/OCB).
    pub seipd_version: u8,
}

fn parse(e: impl std::fmt::Display) -> CryptoError {
    CryptoError::Parse(e.to_string())
}

// ── key generation ───────────────────────────────────────────────────────────

/// Generate a v6 OpenPGP key: Ed25519 primary (certify + sign), X25519 encryption
/// subkey. The returned bundle is the secret key armored under S2K protection with
/// `passphrase`.
pub fn generate_key(user_id: &str, passphrase: &str) -> Result<GeneratedKey> {
    generate_key_with(user_id, passphrase, &GenerateOptions::default())
}

/// Generate a key of `opts.profile`, optionally expiring `opts.expires_in_secs`
/// after creation. See [`KeyProfile`].
pub fn generate_key_with(
    user_id: &str,
    passphrase: &str,
    opts: &GenerateOptions,
) -> Result<GeneratedKey> {
    // rPGP's builder defaults read `std::time::SystemTime`, which has no clock on
    // `wasm32-unknown-unknown`, so the creation time is passed explicitly.
    let created = now_timestamp();
    let keygen = |e: pgp::errors::Error| CryptoError::KeyGen(e.to_string());
    let build = |e: pgp::composed::SubkeyParamsBuilderError| CryptoError::KeyGen(e.to_string());

    let (version, primary_type, sub_type, seipd_v2) = match opts.profile {
        KeyProfile::V6 => (KeyVersion::V6, KeyType::Ed25519, KeyType::X25519, true),
        KeyProfile::Compatible => (
            KeyVersion::V4,
            KeyType::Ed25519Legacy,
            KeyType::ECDH(ECCCurve::Curve25519Legacy),
            false,
        ),
    };

    let mut enc_subkey = SubkeyParamsBuilder::default();
    enc_subkey
        .version(version)
        .key_type(sub_type)
        .created_at(created)
        .can_encrypt(EncryptionCaps::All)
        .can_sign(false)
        .can_authenticate(false);

    let mut params = SecretKeyParamsBuilder::default();
    params
        .version(version)
        .key_type(primary_type)
        .created_at(created)
        .can_certify(true)
        .can_sign(true)
        .can_encrypt(EncryptionCaps::None)
        .feature_seipd_v1(true)
        .feature_seipd_v2(seipd_v2)
        .preferred_symmetric_algorithms(smallvec![
            SymmetricKeyAlgorithm::AES256,
            SymmetricKeyAlgorithm::AES128,
        ])
        .preferred_hash_algorithms(smallvec![HashAlgorithm::Sha512, HashAlgorithm::Sha256])
        .preferred_compression_algorithms(smallvec![
            CompressionAlgorithm::ZLIB,
            CompressionAlgorithm::Uncompressed,
        ])
        .primary_user_id(user_id.to_string())
        .subkey(enc_subkey.build().map_err(build)?);
    if seipd_v2 {
        params.preferred_aead_algorithms(smallvec![
            (SymmetricKeyAlgorithm::AES256, AeadAlgorithm::Ocb),
            (SymmetricKeyAlgorithm::AES128, AeadAlgorithm::Ocb),
        ]);
    }

    let secret_params = params
        .build()
        .map_err(|e| CryptoError::KeyGen(e.to_string()))?;
    let mut signed = secret_params.generate(rng::pgp_rng()).map_err(keygen)?;

    // Expiry is a property of the self-signatures; the builder has no field for it,
    // so the fresh (still unlocked) key is re-signed with one.
    if let Some(secs) = opts.expires_in_secs {
        let at = u64::from(created.as_secs()).saturating_add(secs);
        signed = manage::with_expiry(signed, &Password::empty(), Some(at), true)?;
    }

    let public = signed.to_public_key();
    let fingerprint = cert::fingerprint_hex(&public);
    let key_id = public.legacy_key_id().to_string();
    let public_key_armored = public
        .to_armored_string(ArmorOptions::default())
        .map_err(keygen)?;

    // Lock every secret component (primary + subkeys) under the passphrase (S2K)
    // before the key can be serialized — the decryption secret lives in the
    // encryption subkey, so it must be locked too, not just the primary.
    manage::lock_all(&mut signed, passphrase)?;
    let encrypted_private_bundle = signed
        .to_armored_string(ArmorOptions::default())
        .map_err(keygen)?;

    Ok(GeneratedKey {
        public_key_armored,
        fingerprint,
        key_id,
        encrypted_private_bundle,
    })
}

// ── encryption ───────────────────────────────────────────────────────────────

/// Encrypt `plaintext` to every recipient in `recipient_public_keys` (armored),
/// optionally signing with the passphrase-locked `sign_with` bundle. Returns armored
/// ciphertext. SEIPDv2 when every recipient supports it, else SEIPDv1 (see the
/// module docs).
pub fn encrypt(
    plaintext: &[u8],
    recipient_public_keys: &[String],
    sign_with: Option<(&str, &str)>,
) -> Result<String> {
    encrypt_with(plaintext, recipient_public_keys, sign_with).map(|e| e.armored)
}

/// [`encrypt`], also reporting which SEIPD version was chosen.
pub fn encrypt_with(
    plaintext: &[u8],
    recipient_public_keys: &[String],
    sign_with: Option<(&str, &str)>,
) -> Result<Encrypted> {
    encrypt_at(
        plaintext,
        recipient_public_keys,
        sign_with,
        clock::now_secs(),
    )
}

/// [`encrypt_with`] evaluating the recipients' keys at `now` (Unix seconds).
pub fn encrypt_at(
    plaintext: &[u8],
    recipient_public_keys: &[String],
    sign_with: Option<(&str, &str)>,
    now: u64,
) -> Result<Encrypted> {
    if recipient_public_keys.is_empty() {
        return Err(CryptoError::Input("no recipients".into()));
    }
    let recipients: Vec<SignedPublicKey> = recipient_public_keys
        .iter()
        .map(|a| parse_public(a))
        .collect::<Result<_>>()?;
    let evals: Vec<cert::CertEval<'_>> =
        recipients.iter().map(|c| cert::evaluate(c, now)).collect();
    for e in &evals {
        if e.encryption_targets().is_empty() {
            return Err(CryptoError::Input(format!(
                "recipient key {} has no usable encryption key ({})",
                cert::fingerprint_hex(&e.cert.primary_key),
                e.state.token()
            )));
        }
    }
    let use_v2 = evals.iter().all(|e| e.seipd_v2);

    // The optional signer must outlive the builder borrow.
    let signer = match sign_with {
        Some((bundle, passphrase)) => Some((parse_secret(bundle)?, passphrase.to_string())),
        None => None,
    };
    let signer_public = signer.as_ref().map(|(ssk, _)| ssk.to_public_key());
    let signing_key = match (&signer, &signer_public) {
        (Some((ssk, _)), Some(public)) => Some(select_signing_key(ssk, public, now)?),
        _ => None,
    };

    let armored = if use_v2 {
        let mut builder = MessageBuilder::from_bytes("", plaintext.to_vec()).seipd_v2(
            rng::pgp_rng(),
            SymmetricKeyAlgorithm::AES256,
            AeadAlgorithm::Ocb,
            Default::default(),
        );
        if let (Some(key), Some((_, passphrase))) = (&signing_key, &signer) {
            builder.sign(key.as_signing(), pw(passphrase), HashAlgorithm::Sha512);
        }
        for e in &evals {
            for target in e.encryption_targets() {
                match target {
                    cert::EncTarget::Primary(k) => builder.encrypt_to_key(rng::pgp_rng(), k),
                    cert::EncTarget::Sub(k) => builder.encrypt_to_key(rng::pgp_rng(), k),
                }
                .map_err(|e| CryptoError::Encrypt(e.to_string()))?;
            }
        }
        builder.to_armored_string(rng::pgp_rng(), ArmorOptions::default())
    } else {
        let mut builder = MessageBuilder::from_bytes("", plaintext.to_vec())
            .seipd_v1(rng::pgp_rng(), SymmetricKeyAlgorithm::AES256);
        if let (Some(key), Some((_, passphrase))) = (&signing_key, &signer) {
            builder.sign(key.as_signing(), pw(passphrase), HashAlgorithm::Sha512);
        }
        for e in &evals {
            for target in e.encryption_targets() {
                match target {
                    cert::EncTarget::Primary(k) => builder.encrypt_to_key(rng::pgp_rng(), k),
                    cert::EncTarget::Sub(k) => builder.encrypt_to_key(rng::pgp_rng(), k),
                }
                .map_err(|e| CryptoError::Encrypt(e.to_string()))?;
            }
        }
        builder.to_armored_string(rng::pgp_rng(), ArmorOptions::default())
    }
    .map_err(|e| CryptoError::Encrypt(e.to_string()))?;

    Ok(Encrypted {
        armored,
        seipd_version: if use_v2 { 2 } else { 1 },
    })
}

// ── decryption ───────────────────────────────────────────────────────────────

/// Decrypt an armored/binary PGP message with the passphrase-locked `bundle`.
/// Verifies an inline signature against `signer_public_key` when provided and
/// extracts a protected-headers `Subject:` when present.
pub fn decrypt(
    ciphertext: &[u8],
    encrypted_private_bundle: &str,
    passphrase: &str,
    signer_public_key: Option<&str>,
) -> Result<Decrypted> {
    let signers: Vec<&str> = signer_public_key.into_iter().collect();
    decrypt_with(ciphertext, encrypted_private_bundle, passphrase, &signers)
}

/// [`decrypt`] checking an inline signature against any of `signer_public_keys`.
pub fn decrypt_with(
    ciphertext: &[u8],
    encrypted_private_bundle: &str,
    passphrase: &str,
    signer_public_keys: &[&str],
) -> Result<Decrypted> {
    decrypt_at(
        ciphertext,
        encrypted_private_bundle,
        passphrase,
        signer_public_keys,
        clock::now_secs(),
    )
}

/// [`decrypt_with`] judging the signer's key at `now` (Unix seconds).
pub fn decrypt_at(
    ciphertext: &[u8],
    encrypted_private_bundle: &str,
    passphrase: &str,
    signer_public_keys: &[&str],
    now: u64,
) -> Result<Decrypted> {
    let ssk = parse_secret(encrypted_private_bundle)?;
    let message = parse_message(ciphertext)?;
    let aead = message_uses_aead(ciphertext);

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

    let certs: Vec<SignedPublicKey> = signer_public_keys
        .iter()
        .filter_map(|a| parse_public(a).ok())
        .collect();
    let verification = verify_message_signatures(&decrypted, &certs, now);

    let (subject, body) = unwrap_protected_headers(&data);
    Ok(Decrypted {
        plaintext: body,
        subject,
        signature: verification.signature.clone(),
        verification,
        aead,
    })
}

/// Does the (armored or binary) message contain a SEIPDv2 packet? Read from the
/// packet stream without decrypting.
fn message_uses_aead(ciphertext: &[u8]) -> bool {
    use pgp::packet::{Packet, PacketParser};
    let bytes = match dearmor(ciphertext) {
        Some(b) => b,
        None => ciphertext.to_vec(),
    };
    PacketParser::new(&bytes[..]).flatten().any(|p| {
        matches!(p, Packet::SymEncryptedProtectedData(ref d)
            if d.version() == 2)
    })
}

fn dearmor(input: &[u8]) -> Option<Vec<u8>> {
    use std::io::Read;
    if !input.starts_with(b"-----BEGIN") {
        return None;
    }
    let mut d = pgp::armor::Dearmor::new(input);
    let mut out = Vec::new();
    d.read_to_end(&mut out).ok()?;
    Some(out)
}

fn parse_message(ciphertext: &[u8]) -> Result<Message<'_>> {
    if ciphertext.starts_with(b"-----BEGIN") {
        let text = std::str::from_utf8(ciphertext).map_err(parse)?;
        Ok(Message::from_string(text).map_err(parse)?.0)
    } else {
        Message::from_bytes(ciphertext).map_err(parse)
    }
}

/// Judge the signatures of a fully read, decrypted message against `certs`.
fn verify_message_signatures(
    msg: &Message<'_>,
    certs: &[SignedPublicKey],
    now: u64,
) -> Verification {
    let Message::Signed { reader, .. } = msg else {
        return Verification::not_signed("pgp");
    };
    let mut best: Option<Verification> = None;
    for i in 0..reader.num_signatures() {
        let Some(sig) = reader.signature(i) else {
            continue;
        };
        let v = judge_signature(sig, certs, now, |component| {
            msg.verify_nested_explicit(i, component.as_verifying())
                .is_ok()
        });
        best = Some(match best {
            None => v,
            Some(b) => better(b, v),
        });
    }
    best.unwrap_or_else(|| Verification::not_signed("pgp"))
}

/// Of two results for different signatures on the same data, keep the stronger.
fn better(a: Verification, b: Verification) -> Verification {
    fn rank(s: &str) -> u8 {
        match s {
            verdict::VALID => 5,
            verdict::KEY_EXPIRED | verdict::NOT_YET_VALID => 3,
            verdict::UNKNOWN_KEY => 2,
            verdict::REVOKED | verdict::BAD_SIGNATURE => 1,
            _ => 0,
        }
    }
    if rank(&b.state) > rank(&a.state) {
        b
    } else {
        a
    }
}

// ── signing ──────────────────────────────────────────────────────────────────

/// Produce a detached, armored signature over `data` with the locked `bundle`.
pub fn sign_detached(
    data: &[u8],
    encrypted_private_bundle: &str,
    passphrase: &str,
) -> Result<String> {
    let ssk = parse_secret(encrypted_private_bundle)?;
    let public = ssk.to_public_key();
    let key = select_signing_key(&ssk, &public, clock::now_secs())?;
    let sig = match key {
        SignKey::Primary(k) => DetachedSignature::sign_binary_data(
            rng::pgp_rng(),
            k,
            &pw(passphrase),
            HashAlgorithm::Sha512,
            data,
        ),
        SignKey::Sub(k) => DetachedSignature::sign_binary_data(
            rng::pgp_rng(),
            k,
            &pw(passphrase),
            HashAlgorithm::Sha512,
            data,
        ),
    }
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
    let public = ssk.to_public_key();
    let key = select_signing_key(&ssk, &public, clock::now_secs())?;
    let signed = match key {
        SignKey::Primary(k) => {
            CleartextSignedMessage::sign(rng::pgp_rng(), data, k, &pw(passphrase))
        }
        SignKey::Sub(k) => CleartextSignedMessage::sign(rng::pgp_rng(), data, k, &pw(passphrase)),
    }
    .map_err(|e| CryptoError::Sign(e.to_string()))?;
    signed
        .to_armored_string(ArmorOptions::default())
        .map_err(|e| CryptoError::Sign(e.to_string()))
}

/// The secret component that makes data signatures.
pub(crate) enum SignKey<'a> {
    Primary(&'a pgp::packet::SecretKey),
    Sub(&'a pgp::packet::SecretSubkey),
}

impl SignKey<'_> {
    fn as_signing(&self) -> &dyn SigningKey {
        match self {
            SignKey::Primary(k) => *k,
            SignKey::Sub(k) => *k,
        }
    }
}

/// Pick the signing key of a secret key at `now`: the primary when it is flagged
/// for signing and usable, else the newest usable signing subkey.
pub(crate) fn select_signing_key<'a>(
    ssk: &'a SignedSecretKey,
    public: &SignedPublicKey,
    now: u64,
) -> Result<SignKey<'a>> {
    let eval = cert::evaluate(public, now);
    let usable = eval.signing_fingerprints();
    if usable.is_empty() {
        return Err(CryptoError::Sign(format!(
            "key {} has no usable signing key ({})",
            cert::fingerprint_hex(&public.primary_key),
            eval.state.token()
        )));
    }
    if usable.contains(&cert::fingerprint_hex(&ssk.primary_key)) {
        return Ok(SignKey::Primary(&ssk.primary_key));
    }
    ssk.secret_subkeys
        .iter()
        .filter(|s| usable.contains(&cert::fingerprint_hex(&s.key)))
        .max_by_key(|s| s.key.created_at().as_secs())
        .map(|s| SignKey::Sub(&s.key))
        .ok_or_else(|| CryptoError::Sign("no secret material for the signing key".into()))
}

// ── verification ─────────────────────────────────────────────────────────────

/// Verify an inline cleartext-signed message (`PGP SIGNED MESSAGE`, produced by
/// [`clear_sign`]) against `signer_public_key`, returning `(verdict, cleartext)`.
/// The cleartext is the recovered, un-escaped message body.
pub fn verify_clear_signed(
    armored: &str,
    signer_public_key: &str,
) -> Result<(SignatureVerdict, String)> {
    let (v, text) = verify_cleartext(armored, &[signer_public_key])?;
    Ok((v.signature, text))
}

/// Verify a cleartext-signed message against any of `signer_public_keys` →
/// `(verification, signed text)`.
pub fn verify_cleartext(
    armored: &str,
    signer_public_keys: &[&str],
) -> Result<(Verification, String)> {
    verify_cleartext_at(armored, signer_public_keys, clock::now_secs())
}

/// [`verify_cleartext`] judging keys at `now`.
pub fn verify_cleartext_at(
    armored: &str,
    signer_public_keys: &[&str],
    now: u64,
) -> Result<(Verification, String)> {
    let certs = parse_publics(signer_public_keys)?;
    let (msg, _) = CleartextSignedMessage::from_string(armored).map_err(parse)?;
    let text = msg.signed_text();
    let mut best: Option<Verification> = None;
    for sig in msg.signatures() {
        let v = judge_signature(sig, &certs, now, |component| {
            component.verify_data(sig, text.as_bytes())
        });
        best = Some(match best {
            None => v,
            Some(b) => better(b, v),
        });
    }
    // `signed_text` is the de-dash-escaped, CRLF-normalized content the signature
    // actually covers (`text()` would return the dash-escaped transport form).
    Ok((
        best.unwrap_or_else(|| Verification::not_signed("pgp")),
        text,
    ))
}

/// Verify a detached armored signature over `data` against `signer_public_key`.
pub fn verify_detached(
    data: &[u8],
    signature_armored: &str,
    signer_public_key: &str,
) -> Result<SignatureVerdict> {
    verify_detached_with(data, signature_armored, &[signer_public_key]).map(|v| v.signature)
}

/// Verify a detached signature (armored, or binary as in a PGP/MIME
/// `application/pgp-signature` part) over `data` against any of
/// `signer_public_keys`.
pub fn verify_detached_with(
    data: &[u8],
    signature: &str,
    signer_public_keys: &[&str],
) -> Result<Verification> {
    verify_detached_at(
        data,
        signature.as_bytes(),
        signer_public_keys,
        clock::now_secs(),
    )
}

/// [`verify_detached_with`] on raw signature bytes, judging keys at `now`.
pub fn verify_detached_at(
    data: &[u8],
    signature: &[u8],
    signer_public_keys: &[&str],
    now: u64,
) -> Result<Verification> {
    let certs = parse_publics(signer_public_keys)?;
    let sigs = parse_signatures(signature)?;
    if sigs.is_empty() {
        return Ok(Verification::not_signed("pgp"));
    }
    let mut best: Option<Verification> = None;
    for sig in &sigs {
        let v = judge_signature(sig, &certs, now, |component| {
            component.verify_data(sig, data)
        });
        best = Some(match best {
            None => v,
            Some(b) => better(b, v),
        });
    }
    Ok(best.unwrap_or_else(|| Verification::not_signed("pgp")))
}

/// Every signature packet in an armored or binary signature block.
fn parse_signatures(input: &[u8]) -> Result<Vec<Signature>> {
    use pgp::packet::{Packet, PacketParser};
    let bytes = match dearmor(input) {
        Some(b) => b,
        None if input.starts_with(b"-----BEGIN") => {
            return Err(CryptoError::Parse("signature armor did not decode".into()));
        }
        None => input.to_vec(),
    };
    let mut out = Vec::new();
    for p in PacketParser::new(&bytes[..]) {
        match p.map_err(parse)? {
            Packet::Signature(s) => out.push(s),
            Packet::Marker(_) | Packet::Padding(_) => {}
            other => {
                return Err(CryptoError::Parse(format!(
                    "unexpected {:?} packet in a signature",
                    pgp::packet::PacketTrait::tag(&other)
                )));
            }
        }
    }
    Ok(out)
}

/// A public key component a signature may come from.
pub(crate) enum Component<'a> {
    Primary(&'a pgp::packet::PublicKey),
    Sub(&'a pgp::packet::PublicSubkey),
}

impl Component<'_> {
    fn as_verifying(&self) -> &dyn pgp::types::VerifyingKey {
        match self {
            Component::Primary(k) => *k,
            Component::Sub(k) => *k,
        }
    }
    fn verify_data(&self, sig: &Signature, data: &[u8]) -> bool {
        match self {
            Component::Primary(k) => sig.verify(*k, data).is_ok(),
            Component::Sub(k) => sig.verify(*k, data).is_ok(),
        }
    }
    fn fingerprint(&self) -> pgp::types::Fingerprint {
        match self {
            Component::Primary(k) => k.fingerprint(),
            Component::Sub(k) => k.fingerprint(),
        }
    }
    fn key_id(&self) -> pgp::types::KeyId {
        match self {
            Component::Primary(k) => k.legacy_key_id(),
            Component::Sub(k) => k.legacy_key_id(),
        }
    }
    fn matches(&self, sig: &Signature) -> bool {
        let fp = self.fingerprint();
        let id = self.key_id();
        sig.issuer_fingerprint().iter().any(|f| **f == fp)
            || sig.issuer_key_id().iter().any(|i| **i == id)
    }
}

/// Decide the state of one signature against candidate certificates. `check` runs
/// the cryptographic verification for one key component.
fn judge_signature(
    sig: &Signature,
    certs: &[SignedPublicKey],
    now: u64,
    check: impl Fn(&Component<'_>) -> bool,
) -> Verification {
    let signed_at = cert::sig_created(sig);
    let mut v = Verification::new("pgp", verdict::UNKNOWN_KEY);
    v.signed_at = signed_at.map(clock::rfc3339);
    v.signature.signer_key_id = sig
        .issuer_key_id()
        .first()
        .map(|k| k.to_string())
        .or_else(|| {
            sig.issuer_fingerprint()
                .first()
                .map(|f| hex::encode(f.as_bytes()).to_uppercase())
        });

    for cert in certs {
        let mut components = vec![Component::Primary(&cert.primary_key)];
        components.extend(cert.public_subkeys.iter().map(|s| Component::Sub(&s.key)));
        let has_issuer = !sig.issuer_fingerprint().is_empty() || !sig.issuer_key_id().is_empty();
        for component in components {
            if has_issuer && !component.matches(sig) {
                continue;
            }
            let good = check(&component);
            if !good && !has_issuer {
                continue;
            }
            return describe(sig, cert, &component, good, signed_at, now);
        }
    }
    v
}

/// The verdict for `sig` made by `component` of `cert`; `good` says whether the
/// cryptographic check passed.
fn describe(
    sig: &Signature,
    cert: &SignedPublicKey,
    component: &Component<'_>,
    good: bool,
    signed_at: Option<u64>,
    now: u64,
) -> Verification {
    let fp = hex::encode(component.fingerprint().as_bytes()).to_uppercase();
    let at_signing = cert::evaluate(cert, signed_at.unwrap_or(now));
    let current = cert::evaluate(cert, now);

    let mut v = Verification::new("pgp", verdict::VALID);
    v.signer_fingerprint = Some(cert::fingerprint_hex(&cert.primary_key));
    v.signer_addresses = current.addresses();
    v.signed_at = signed_at.map(clock::rfc3339);
    v.signature.signer_key_id = Some(cert.primary_key.legacy_key_id().to_string());

    // Component metadata as of signing time.
    let (state_then, flags_sign, expires, created, alg) =
        if fp == cert::fingerprint_hex(&cert.primary_key) {
            (
                at_signing.state,
                at_signing.flags.sign(),
                at_signing.expires,
                u64::from(cert.primary_key.created_at().as_secs()),
                cert::algorithm_name(cert.primary_key.public_params()),
            )
        } else {
            let sub = at_signing
                .subkeys
                .iter()
                .find(|s| cert::fingerprint_hex(&s.key.key) == fp);
            match sub {
                Some(s) => (
                    s.state,
                    s.flags.sign(),
                    match (s.expires, at_signing.expires) {
                        (Some(a), Some(b)) => Some(a.min(b)),
                        (a, b) => a.or(b),
                    },
                    u64::from(s.key.key.created_at().as_secs()),
                    cert::algorithm_name(s.key.key.public_params()),
                ),
                None => (cert::State::Invalid, false, None, 0, String::new()),
            }
        };
    v.signature.algorithm = Some(alg);
    v.signature.key_created_at = Some(clock::rfc3339(created));
    v.signature.key_expires_at = expires.map(clock::rfc3339);

    let revoked_now = current
        .revocation
        .as_ref()
        .is_some_and(|r| r.applies_at(signed_at.unwrap_or(now)));
    v.signature.revocation_status = Some(if revoked_now { "revoked" } else { "good" }.into());

    if !good {
        return v.with(verdict::BAD_SIGNATURE, Some("tampered"));
    }
    if cert::weak_data_hash(sig) {
        return v.with(verdict::BAD_SIGNATURE, Some("weak-hash"));
    }
    if revoked_now {
        return v.with(verdict::REVOKED, None);
    }
    match state_then {
        cert::State::Valid => {}
        cert::State::Expired => return v.with(verdict::KEY_EXPIRED, None),
        cert::State::NotYetValid => return v.with(verdict::NOT_YET_VALID, None),
        cert::State::Revoked => return v.with(verdict::REVOKED, None),
        cert::State::Invalid => return v.with(verdict::BAD_SIGNATURE, Some("unbound-key")),
    }
    if !flags_sign {
        return v.with(verdict::BAD_SIGNATURE, Some("not-a-signing-key"));
    }
    v
}

// ── keys: import, inspect, export ────────────────────────────────────────────

/// Parse an armored PGP key (public or secret) into a [`CryptoKey`] for the keyring.
/// If the armor carries a secret key, `has_private` is set and the (still
/// passphrase-locked, if it was) armored secret is returned as the bundle.
/// `addresses` empty → the key's own valid User ID addresses.
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

/// Every public key in an armored or binary block (a keyring file may hold more
/// than one), each as a [`CryptoKey`] with `source`.
pub fn parse_keys(input: &[u8], source: &str) -> Result<Vec<CryptoKey>> {
    let certs: Vec<SignedPublicKey> = if input.starts_with(b"-----BEGIN") {
        let text = std::str::from_utf8(input).map_err(parse)?;
        let (iter, _) = SignedPublicKey::from_string_many(text).map_err(parse)?;
        iter.collect::<std::result::Result<_, _>>().map_err(parse)?
    } else {
        SignedPublicKey::from_bytes_many(input)
            .map_err(parse)?
            .collect::<std::result::Result<_, _>>()
            .map_err(parse)?
    };
    certs
        .iter()
        .map(|c| {
            let armored = c
                .to_armored_string(ArmorOptions::default())
                .map_err(parse)?;
            Ok(build_crypto_key(c, &armored, Vec::new(), false, source))
        })
        .collect()
}

/// Describe a key (public or secret armor) and its parts as of now.
pub fn inspect_key(armored: &str) -> Result<PgpKeyInfo> {
    inspect_key_at(armored, clock::now_secs())
}

/// [`inspect_key`] at `now` (Unix seconds).
pub fn inspect_key_at(armored: &str, now: u64) -> Result<PgpKeyInfo> {
    if let Ok((ssk, _)) = SignedSecretKey::from_string(armored) {
        let public = ssk.to_public_key();
        return Ok(cert::evaluate(&public, now).info(true));
    }
    let public = parse_public(armored)?;
    Ok(cert::evaluate(&public, now).info(false))
}

/// Export the armored public key from a locked secret bundle (or pass through an
/// armored public key).
pub fn export_public(bundle_or_public: &str) -> Result<String> {
    public_of(bundle_or_public)?
        .to_armored_string(ArmorOptions::default())
        .map_err(parse)
}

/// The public key from a secret bundle or public armor.
pub(crate) fn public_of(bundle_or_public: &str) -> Result<SignedPublicKey> {
    if let Ok((ssk, _)) = SignedSecretKey::from_string(bundle_or_public) {
        return Ok(ssk.to_public_key());
    }
    parse_public(bundle_or_public)
}

/// Wrap the passphrase-locked secret bundle as an Autocrypt Setup Message backup.
/// The bundle is already S2K-protected, so the ASM payload is exactly that armored,
/// passphrase-protected key (Autocrypt §4.2 transport container).
pub fn autocrypt_setup_message(encrypted_private_bundle: &str) -> Result<String> {
    let ssk = parse_secret(encrypted_private_bundle)?;
    ssk.to_armored_string(ArmorOptions::default())
        .map_err(parse)
}

// ── Autocrypt Setup Message — full wire format (Autocrypt Level 1 §4.2) ───────
//
// The full ASM is a symmetrically-encrypted OpenPGP MESSAGE guarded by a numeric
// **Setup Code** (9 groups of 4 digits), armored with the `Passphrase-Format:
// numeric9x4` / `Passphrase-Begin: <first two digits>` headers, and delivered as an
// `application/autocrypt-setup` attachment in an `Autocrypt-Setup-Message: v1`
// email. Unlike the simple backup above (still-S2K-locked armor), this container is
// what another Autocrypt client consumes: it hands the user the setup code out of
// band and decrypts the attachment with it. The inner payload we wrap is the
// passphrase-locked secret bundle, so recovery restores the same at-rest-locked key
// (defense in depth — the importing device unlocks with the user passphrase).

/// The armor header naming the setup-code format (Autocrypt §4.2).
const ASM_PASSPHRASE_FORMAT: &str = "Passphrase-Format: numeric9x4";
/// The `application/autocrypt-setup` MIME type of the ASM attachment.
pub const AUTOCRYPT_SETUP_CONTENT_TYPE: &str = "application/autocrypt-setup";

/// Generate an Autocrypt Setup Code: 9 groups of 4 decimal digits (`numeric9x4`,
/// Autocrypt §4.3), e.g. `"1234-5678-9012-3456-7890-1234-5678-9012-3456"`. Sourced
/// from the same portable CSPRNG the key/nonce paths use.
pub fn generate_setup_code() -> String {
    use rand::Rng;
    let mut rng = rng::pgp_rng();
    let groups: Vec<String> = (0..9)
        .map(|_| format!("{:04}", rng.gen_range(0..10_000u32)))
        .collect();
    groups.join("-")
}

/// Produce the `application/autocrypt-setup` payload: the secret `bundle`,
/// symmetrically encrypted under `setup_code` (SEIPDv2 / AES-256 / OCB), armored
/// with the `Passphrase-Format` / `Passphrase-Begin` headers so an Autocrypt client
/// prompts for the code. `setup_code` should be a [`generate_setup_code`] value; its
/// first two digits are advertised in `Passphrase-Begin` as an entry hint.
pub fn autocrypt_setup_message_full(
    encrypted_private_bundle: &str,
    setup_code: &str,
) -> Result<String> {
    // Validate the bundle parses (fail fast on a non-key input).
    let _ = parse_secret(encrypted_private_bundle)?;

    let mut builder = MessageBuilder::from_bytes("", encrypted_private_bundle.as_bytes().to_vec())
        .seipd_v2(
            rng::pgp_rng(),
            SymmetricKeyAlgorithm::AES256,
            AeadAlgorithm::Ocb,
            Default::default(),
        );
    let s2k = pgp::types::StringToKey::new_default(rng::pgp_rng());
    builder
        .encrypt_with_password(rng::pgp_rng(), s2k, &pw(setup_code))
        .map_err(|e| CryptoError::Encrypt(e.to_string()))?;
    let armored = builder
        .to_armored_string(rng::pgp_rng(), ArmorOptions::default())
        .map_err(|e| CryptoError::Encrypt(e.to_string()))?;

    // Inject the Autocrypt armor headers right after the BEGIN line (RFC 9580 armor
    // permits arbitrary headers; a decoder that does not recognize them ignores them).
    let begin = "-----BEGIN PGP MESSAGE-----";
    let hint = setup_code
        .chars()
        .filter(|c| c.is_ascii_digit())
        .take(2)
        .collect::<String>();
    let headers = format!("{ASM_PASSPHRASE_FORMAT}\nPassphrase-Begin: {hint}");
    let out = match armored.split_once('\n') {
        Some((first, rest)) if first.trim_end() == begin => format!("{first}\n{headers}\n{rest}"),
        _ => format!("{begin}\n{headers}\n{armored}"),
    };
    Ok(out)
}

/// Recover the inner secret bundle from an [`autocrypt_setup_message_full`] payload
/// using its `setup_code`. The returned armor is the passphrase-locked secret key
/// (unlock with the user passphrase to use it).
pub fn decrypt_autocrypt_setup_message(payload: &str, setup_code: &str) -> Result<String> {
    let (message, _) = Message::from_string(payload).map_err(parse)?;
    let mut decrypted = message
        .decrypt_with_password(&pw(setup_code))
        .map_err(|e| CryptoError::Decrypt(e.to_string()))?;
    if decrypted.is_compressed() {
        decrypted = decrypted
            .decompress()
            .map_err(|e| CryptoError::Decrypt(e.to_string()))?;
    }
    let data = decrypted
        .as_data_vec()
        .map_err(|e| CryptoError::Decrypt(e.to_string()))?;
    String::from_utf8(data).map_err(|e| CryptoError::Parse(format!("ASM payload not UTF-8: {e}")))
}

/// Frame an ASM `payload` (from [`autocrypt_setup_message_full`]) as a complete
/// `Autocrypt-Setup-Message: v1` email (RFC5322 + `multipart/mixed`): a human-
/// readable part plus the `application/autocrypt-setup` attachment. `from` seeds the
/// `From:`/`To:` (the message is sent to oneself, Autocrypt §4.2).
pub fn autocrypt_setup_message_mime(from: &str, payload: &str) -> String {
    let boundary = format!("mw-asm-{:x}", Sha1Hint::of(payload));
    format!(
        "From: {from}\r\n\
         To: {from}\r\n\
         Subject: Autocrypt Setup Message\r\n\
         Autocrypt-Setup-Message: v1\r\n\
         MIME-Version: 1.0\r\n\
         Content-Type: multipart/mixed; boundary=\"{boundary}\"\r\n\
         \r\n\
         --{boundary}\r\n\
         Content-Type: text/plain\r\n\
         \r\n\
         This is the Autocrypt Setup Message used to transfer your key between \
         clients. To decrypt it you need the Setup Code shown on the sending device.\r\n\
         --{boundary}\r\n\
         Content-Type: {AUTOCRYPT_SETUP_CONTENT_TYPE}\r\n\
         Content-Disposition: attachment; filename=\"autocrypt-setup-message.txt\"\r\n\
         \r\n\
         {payload}\r\n\
         --{boundary}--\r\n"
    )
}

/// A tiny non-cryptographic tag used only to derive a unique MIME boundary from the
/// payload (never a security primitive).
struct Sha1Hint;
impl Sha1Hint {
    fn of(s: &str) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        for b in s.bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x1000_0000_0000_01b3);
        }
        h
    }
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

// ── Protected headers, Mailwoman's first format (kept for reading old mail) ──
//
// New messages use RFC 9788 header protection on the MIME entity
// (`crate::mime::protect_headers`); this stanza is what 26.19 and earlier wrote.

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

// ── Attached keys ────────────────────────────────────────────────────────────

/// Keys carried by an attachment: `application/pgp-keys`, or a `.asc`/`.gpg`/
/// `.pgp`/`.key` file (or any text) holding a `PGP PUBLIC KEY BLOCK`. Returns
/// `None` when the part is not a key attachment, `Some(keys)` (possibly empty when
/// it claims to be one but holds no readable key) otherwise. Secret keys are never
/// returned from here: a key attachment is someone else's public key.
pub fn detect_attached_keys(
    content_type: &str,
    filename: Option<&str>,
    bytes: &[u8],
) -> Option<Vec<CryptoKey>> {
    let ct = content_type
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    let name = filename.unwrap_or("").to_ascii_lowercase();
    let armored_key = find_subslice(bytes, b"-----BEGIN PGP PUBLIC KEY BLOCK-----");
    let claims = ct == "application/pgp-keys"
        || (armored_key.is_some()
            && [".asc", ".gpg", ".pgp", ".key", ".txt"]
                .iter()
                .any(|ext| name.ends_with(ext) || name.is_empty()));
    if !claims {
        return None;
    }
    let slice = match armored_key {
        Some(start) => &bytes[start..],
        None => bytes,
    };
    Some(parse_keys(slice, "attached").unwrap_or_default())
}

// ── WKD (native — HTTPS GET; pure URL derivation is testable offline) ────────

/// The WKD hash of an address's local part: `zbase32(SHA-1(lowercase local part))`
/// (draft-koch-openpgp-webkey-service §3.1). Also the file name a server publishes
/// the key under.
pub fn wkd_hash(local_part: &str) -> String {
    use sha1::{Digest, Sha1};
    let mut h = Sha1::new();
    h.update(local_part.to_lowercase().as_bytes());
    zbase32_encode(&h.finalize())
}

/// Derive the WKD (Web Key Directory) URL for an email address: the advanced method
/// hosts under `openpgpkey.<domain>`, the direct method under `<domain>`; the local
/// part is `zbase32(SHA-1(lowercase local part))`. Pure — no network.
pub fn wkd_url(email: &str, advanced: bool) -> Result<String> {
    let (local, domain) = email
        .rsplit_once('@')
        .ok_or_else(|| CryptoError::Input("invalid email".into()))?;
    if local.is_empty() || domain.is_empty() {
        return Err(CryptoError::Input("invalid email".into()));
    }
    // The hash uses the lower-cased local part; the `l=` parameter carries it as
    // written, so a server that keys on the exact local part can still find it.
    let encoded = wkd_hash(local);
    let domain_lower = domain.to_lowercase();
    let local_enc = urlencode(local);
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

/// Both WKD URLs for an address, advanced method first (the order a client tries).
pub fn wkd_urls(email: &str) -> Result<[String; 2]> {
    Ok([wkd_url(email, true)?, wkd_url(email, false)?])
}

/// The `User-Agent` WKD lookups announce. Named for what it is rather than
/// inheriting `mw-egress`'s image-proxy default, which would tell a key server it was
/// talking to an image proxy.
#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
const WKD_UA: &str = "Mailwoman-WKD";

/// Map an egress refusal onto a crypto error, keeping `404` distinct.
///
/// A WKD `404` is the ordinary case — most addresses publish no key — and saying so
/// plainly is the difference between "this person has no published key" and "the
/// lookup broke". Before this routed through the gate the status was never checked at
/// all and a 404 error page was fed to the key parser, surfacing as a parse failure.
#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
fn wkd_refusal(r: mw_egress::Refusal) -> CryptoError {
    match r.status() {
        Some(404) => CryptoError::Input("no key published for that address".into()),
        Some(code) => CryptoError::Io(format!("WKD lookup failed: HTTP {code}")),
        None => CryptoError::Io(format!("WKD lookup failed: {r:?}")),
    }
}

/// Fetch a WKD key (native only — HTTPS GET, no keyserver fallback). The advanced
/// method is tried first; when it fails (most often because the `openpgpkey.`
/// subdomain does not exist) the direct method is tried, as the WKD draft
/// specifies. The key must carry a User ID with the looked-up address; a server
/// handing out someone else's key is refused.
#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub async fn wkd_fetch(email: &str) -> Result<CryptoKey> {
    let [advanced, direct] = wkd_urls(email)?;
    // Routed through `mw-egress` (t22-e9): the full SSRF gate, not just the ambient
    // proxy refusal this used to carry. The URL's domain comes from a correspondent's
    // address — which arrives in an incoming message — so it is attacker-influenceable,
    // and refusing proxies did nothing about it resolving to an internal host. The
    // strict `ip_allowed` policy applies: WKD is served from a public domain's
    // `.well-known`, so there is no legitimate private target here (unlike autoconfig,
    // which has an opt-in for exactly that reason).
    let fetch = |url: String| async move {
        mw_egress::fetch_url_hardened_with(
            &url,
            "application/octet-stream",
            WKD_UA,
            mw_egress::ip_allowed,
        )
        .await
        .map_err(wkd_refusal)
    };
    let bytes = match fetch(advanced).await {
        Ok(b) => b,
        Err(first) => fetch(direct).await.map_err(|second| match second {
            // Report the direct method's 404 (no key) over the advanced method's
            // failure, which is usually just a missing subdomain.
            CryptoError::Input(_) => second,
            _ => first,
        })?,
    };
    wkd_key_from_bytes(email, &bytes)
}

/// Parse a WKD response body and check it is a key for `email`.
pub fn wkd_key_from_bytes(email: &str, bytes: &[u8]) -> Result<CryptoKey> {
    let wanted = email.to_lowercase();
    let certs: Vec<SignedPublicKey> = SignedPublicKey::from_bytes_many(bytes)
        .map_err(parse)?
        .collect::<std::result::Result<_, _>>()
        .map_err(parse)?;
    let now = clock::now_secs();
    let cert = certs
        .iter()
        .find(|c| cert::evaluate(c, now).addresses().contains(&wanted))
        .ok_or_else(|| {
            CryptoError::Input("the published key has no User ID with that address".into())
        })?;
    let armored = cert
        .to_armored_string(ArmorOptions::default())
        .map_err(parse)?;
    Ok(build_crypto_key(
        cert,
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

pub(crate) fn pw(passphrase: &str) -> Password {
    passphrase.into()
}

/// The current time as an rPGP [`Timestamp`], from [`clock::now_secs`] (the JS
/// clock on wasm), never `std::time::SystemTime`, which is unimplemented on
/// `wasm32-unknown-unknown`.
pub(crate) fn now_timestamp() -> Timestamp {
    Timestamp::from_secs(u32::try_from(clock::now_secs()).unwrap_or(u32::MAX))
}

pub(crate) fn parse_public(armored: &str) -> Result<SignedPublicKey> {
    let (cert, _) = SignedPublicKey::from_string(armored).map_err(parse)?;
    Ok(cert)
}

fn parse_publics(armored: &[&str]) -> Result<Vec<SignedPublicKey>> {
    armored.iter().map(|a| parse_public(a)).collect()
}

pub(crate) fn parse_secret(armored: &str) -> Result<SignedSecretKey> {
    let (ssk, _) = SignedSecretKey::from_string(armored).map_err(parse)?;
    Ok(ssk)
}

pub(crate) fn build_crypto_key(
    cert: &SignedPublicKey,
    public_key_armored: &str,
    addresses: Vec<String>,
    has_private: bool,
    source: &str,
) -> CryptoKey {
    let now = clock::now_secs();
    let eval = cert::evaluate(cert, now);
    let fingerprint = cert::fingerprint_hex(&cert.primary_key);
    let addresses = if addresses.is_empty() {
        eval.addresses()
    } else {
        addresses
    };
    let trust = if eval.state == cert::State::Revoked {
        "revoked"
    } else if has_private {
        "verified"
    } else {
        "unverified"
    };
    CryptoKey {
        id: format!("pgp:{fingerprint}"),
        kind: "pgp".into(),
        is_own: has_private,
        addresses,
        fingerprint: fingerprint.clone(),
        key_id: cert.primary_key.legacy_key_id().to_string(),
        algorithm: cert::algorithm_name(cert.primary_key.public_params()),
        created_at: clock::rfc3339(u64::from(cert.primary_key.created_at().as_secs())),
        expires_at: eval.expires.map(clock::rfc3339),
        public_key_armored: Some(public_key_armored.to_string()),
        cert_pem: None,
        trust: trust.into(),
        autocrypt: false,
        source: source.into(),
        has_private,
        encrypted_private_backup: None,
        verified_at: None,
        key_history: vec![KeyHistoryEntry {
            fingerprint,
            seen_at: clock::rfc3339(now),
        }],
    }
}

/// Serialize a public key to binary (Autocrypt `keydata`, WKD files).
pub(crate) fn to_binary(cert: &SignedPublicKey) -> Result<Vec<u8>> {
    cert.to_bytes().map_err(parse)
}
