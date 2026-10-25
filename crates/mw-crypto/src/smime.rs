//! S/MIME over the RustCrypto stack (`cms`/`x509-cert`/`rsa`/`p256`/`p384`/
//! `ed25519-dalek`, plan §1.1, t32-e41): key and certificate generation (RSA,
//! ECDSA P-256/P-384, Ed25519), certification requests, certificate summaries and
//! harvesting, and the earlier sign/verify/encrypt/decrypt entry points, which now
//! run on [`crate::smime_cms`].
//!
//! - Full verification with chain, trust, revocation and sender state is
//!   [`crate::smime_cms::verify`]; [`verify`] here is the signature-only check the
//!   3-state verdict of 26.19 was, now with the real algorithm and validity times
//!   and with detached content honoured ([`verify_detached`], D-15).
//! - Encryption picks key transport or key agreement per recipient certificate.
//! - Decryption and the private-key paths (PKCS#12, certification requests,
//!   attaching an issued certificate) compile only into the browser guest and this
//!   crate's tests (`client-private-ops`, DA48).
//!
//! The S/MIME `encryptedPrivateBundle` (frozen §2.3) is the private key as a
//! passphrase-encrypted PKCS#8 (PBES2) PEM — never emitted in the clear.

use cms::cert::CertificateChoices;
use cms::content_info::ContentInfo;
use cms::signed_data::SignedData;
use const_oid::ObjectIdentifier;
use der::asn1::{BitString, GeneralizedTime, Ia5String, OctetString, SetOfVec, UtcTime};
use der::{Any, Decode, Encode, Tag};
use sha2::{Digest, Sha256};
use spki::{AlgorithmIdentifierOwned, SubjectPublicKeyInfoOwned};
use x509_cert::Certificate;
use x509_cert::attr::AttributeTypeAndValue;
use x509_cert::ext::Extension;
use x509_cert::ext::pkix::name::GeneralName;
use x509_cert::ext::pkix::{
    BasicConstraints, ExtendedKeyUsage, KeyUsage, KeyUsages, SubjectAltName, SubjectKeyIdentifier,
};
use x509_cert::name::Name;
use x509_cert::serial_number::SerialNumber;
use x509_cert::time::{Time, Validity};

use crate::error::{CryptoError, Result};
use crate::pkix::oid;
use crate::pkix::sig::{self as pkix_sig, Hash, Use};
use crate::rng;
use crate::smime_cms::{self, Cipher, Recipient, SignOptions, VerifyOptions};
use crate::smime_keys::{self, KeyType, PrivateKey};
use crate::smime_trust::TrustConfig;
use crate::types::{CryptoKey, KeyHistoryEntry, SignatureVerdict};
use crate::verdict;

#[cfg(any(target_arch = "wasm32", feature = "client-private-ops"))]
pub use crate::smime_pkcs12::{Pkcs12Import, export_pkcs12, import_pkcs12};

/// What a certificate says about itself — read back from its DER, never from the
/// arguments it was made with.
#[derive(Debug, Clone)]
pub struct CertSummary {
    pub cert_pem: String,
    /// SHA-256 over the certificate DER, upper-case hex.
    pub fingerprint: String,
    pub addresses: Vec<String>,
    /// `rsa-<modulus bits>`, `ecdsa-p256`, `ecdsa-p384`, `ed25519`, `x25519`, or
    /// the public-key algorithm OID.
    pub algorithm: String,
    /// RFC 3339.
    pub not_before: String,
    /// RFC 3339.
    pub not_after: String,
    /// Issuer and subject are the same name: no certificate authority is behind it.
    pub self_issued: bool,
}

/// Result of [`generate`]: a new key (passphrase-wrapped) and a certificate for it
/// signed by that same key.
pub struct GeneratedSmime {
    pub cert: CertSummary,
    pub encrypted_private_bundle: String,
}

fn parse(e: impl std::fmt::Display) -> CryptoError {
    CryptoError::Parse(e.to_string())
}

// ── Sign / verify ────────────────────────────────────────────────────────────

/// Sign `data` with the passphrase-locked `bundle` + `cert_pem`, producing an
/// opaque CMS SignedData (base64 DER) with signed attributes (contentType,
/// signingTime, messageDigest, SMIMECapabilities). The digest follows the key:
/// SHA-256 for RSA and P-256, SHA-384 for P-384, SHA-512 for Ed25519.
pub fn sign(
    data: &[u8],
    cert_pem: &str,
    encrypted_private_bundle: &str,
    passphrase: &str,
) -> Result<String> {
    use base64::Engine;
    let der = smime_cms::sign(
        data,
        cert_pem.as_bytes(),
        encrypted_private_bundle,
        passphrase,
        &SignOptions {
            detached: false,
            ..SignOptions::default()
        },
    )?;
    Ok(base64::engine::general_purpose::STANDARD.encode(der))
}

/// A detached CMS SignedData (DER) over `data`, for `multipart/signed`.
pub fn sign_detached(
    data: &[u8],
    cert_pem: &str,
    encrypted_private_bundle: &str,
    passphrase: &str,
) -> Result<Vec<u8>> {
    smime_cms::sign(
        data,
        cert_pem.as_bytes(),
        encrypted_private_bundle,
        passphrase,
        &SignOptions::default(),
    )
}

/// Verify a CMS SignedData (DER) that carries its content against its embedded
/// certificate, returning the frozen 3-state [`SignatureVerdict`]: `verified` for
/// a good signature, `invalid` for a bad one, `unverified-key` when the signer's
/// certificate is not in the message. No trust decision is made here
/// (`chainStatus` stays `unknown`); [`crate::smime_cms::verify`] makes it.
pub fn verify(cms_der: &[u8]) -> Result<SignatureVerdict> {
    verify_detached(cms_der, None)
}

/// [`verify`] for a detached signature: `content` is the signed part's exact
/// bytes (canonical CRLF). It is ignored when the SignedData carries content.
pub fn verify_detached(cms_der: &[u8], content: Option<&[u8]>) -> Result<SignatureVerdict> {
    let v = smime_cms::verify(
        cms_der,
        &VerifyOptions {
            content: content.map(<[u8]>::to_vec),
            trust: TrustConfig::default(),
            ..VerifyOptions::default()
        },
    )?;
    let mut sig = v.verification.signature;
    sig.status = match v.verification.state.as_str() {
        verdict::BAD_SIGNATURE => "invalid",
        verdict::UNKNOWN_KEY => "unverified-key",
        verdict::NOT_SIGNED => "none",
        // Any other state means the signature itself verified.
        _ => "verified",
    }
    .into();
    sig.chain_status = Some("unknown".into());
    sig.revocation_status = Some("unknown".into());
    Ok(sig)
}

// ── Encrypt / decrypt ────────────────────────────────────────────────────────

fn recipients(pems: &[String]) -> Vec<Recipient> {
    pems.iter()
        .map(|p| Recipient {
            cert: p.as_bytes().to_vec(),
            gcm: None,
        })
        .collect()
}

/// Encrypt `plaintext` to each recipient certificate (PEM) under AES-256-CBC
/// (EnvelopedData): RSA recipients by key transport, P-256/P-384/X25519 ones by
/// key agreement. Returns a DER `ContentInfo`.
pub fn encrypt(plaintext: &[u8], recipient_cert_pems: &[String]) -> Result<Vec<u8>> {
    smime_cms::encrypt(plaintext, &recipients(recipient_cert_pems), Cipher::Cbc).map(|e| e.der)
}

/// [`encrypt`] under AES-256-GCM (AuthEnvelopedData, RFC 5083/5084).
pub fn encrypt_authenveloped(plaintext: &[u8], recipient_cert_pems: &[String]) -> Result<Vec<u8>> {
    smime_cms::encrypt(plaintext, &recipients(recipient_cert_pems), Cipher::Gcm).map(|e| e.der)
}

/// Decrypt a DER EnvelopedData or AuthEnvelopedData with the passphrase-locked
/// `bundle`. Client only (DA48).
#[cfg(any(target_arch = "wasm32", feature = "client-private-ops"))]
pub fn decrypt(
    enveloped_der: &[u8],
    encrypted_private_bundle: &str,
    passphrase: &str,
) -> Result<Vec<u8>> {
    smime_cms::decrypt(enveloped_der, None, encrypted_private_bundle, passphrase)
}

/// [`decrypt`] for an AuthEnvelopedData (kept as its own name for callers of
/// 26.19; both kinds go through the same code now).
#[cfg(any(target_arch = "wasm32", feature = "client-private-ops"))]
pub fn decrypt_authenveloped(
    auth_enveloped_der: &[u8],
    encrypted_private_bundle: &str,
    passphrase: &str,
) -> Result<Vec<u8>> {
    smime_cms::decrypt(
        auth_enveloped_der,
        None,
        encrypted_private_bundle,
        passphrase,
    )
}

// ── Cert harvesting + summaries ──────────────────────────────────────────────

/// Harvest sender certificates from a received CMS SignedData (DER) into keyring
/// [`CryptoKey`]s (`source = "harvested"`).
pub fn harvest_certs(cms_der: &[u8]) -> Result<Vec<CryptoKey>> {
    let ci = ContentInfo::from_der(cms_der).map_err(parse)?;
    let sd = ci
        .content
        .decode_as::<SignedData>()
        .map_err(|_| CryptoError::Parse("not a SignedData".into()))?;
    let mut out = Vec::new();
    if let Some(set) = sd.certificates {
        for choice in set.0.iter() {
            if let CertificateChoices::Certificate(cert) = choice {
                out.push(cert_to_crypto_key(cert)?);
            }
        }
    }
    Ok(out)
}

/// A certificate as a keyring entry (public, `source = "harvested"`).
pub fn cert_to_crypto_key(cert: &Certificate) -> Result<CryptoKey> {
    let summary = summarize(cert)?;
    let fingerprint = summary.fingerprint;
    let now = crate::clock::rfc3339(crate::clock::now_secs());
    Ok(CryptoKey {
        id: format!("smime:{fingerprint}"),
        kind: "smime".into(),
        is_own: false,
        addresses: summary.addresses,
        fingerprint: fingerprint.clone(),
        key_id: fingerprint[..16.min(fingerprint.len())].to_string(),
        algorithm: summary.algorithm,
        created_at: summary.not_before,
        expires_at: Some(summary.not_after),
        public_key_armored: None,
        cert_pem: Some(summary.cert_pem),
        trust: "unverified".into(),
        autocrypt: false,
        source: "harvested".into(),
        has_private: false,
        encrypted_private_backup: None,
        verified_at: None,
        key_history: vec![KeyHistoryEntry {
            fingerprint,
            seen_at: now,
        }],
    })
}

/// Read a certificate's own account of itself (see [`CertSummary`]).
fn summarize(cert: &Certificate) -> Result<CertSummary> {
    let der = cert.to_der().map_err(parse)?;
    let tbs = cert.tbs_certificate();
    let cert_pem =
        der::pem::encode_string("CERTIFICATE", der::pem::LineEnding::LF, &der).map_err(parse)?;
    Ok(CertSummary {
        cert_pem,
        fingerprint: hex::encode(Sha256::digest(&der)).to_uppercase(),
        addresses: cert_email_addresses(cert),
        algorithm: pkix_sig::key_algorithm(tbs.subject_public_key_info()),
        not_before: rfc3339(tbs.validity().not_before)?,
        not_after: rfc3339(tbs.validity().not_after)?,
        self_issued: tbs.issuer() == tbs.subject(),
    })
}

/// Parse one certificate (PEM text or DER bytes) and summarise it.
pub fn describe_certificate(cert: &[u8]) -> Result<CertSummary> {
    summarize(&parse_certificate(cert)?)
}

/// One certificate from PEM text or DER bytes.
fn parse_certificate(bytes: &[u8]) -> Result<Certificate> {
    smime_cms::parse_certificate(bytes)
}

fn rfc3339(time: Time) -> Result<String> {
    let out_of_range = || CryptoError::Parse("certificate time out of range".into());
    let secs = i64::try_from(time.to_unix_duration().as_secs()).map_err(|_| out_of_range())?;
    chrono::DateTime::<chrono::Utc>::from_timestamp(secs, 0)
        .map(|t| t.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
        .ok_or_else(out_of_range)
}

/// Email addresses a certificate names: the subject alternative name's
/// `rfc822Name` entries first (where RFC 8550 §4.4.3 puts them), then any PKCS#9
/// `emailAddress` in the subject DN. Duplicates differing only in case are dropped.
fn cert_email_addresses(cert: &Certificate) -> Vec<String> {
    let found = crate::pkix::name::cert_addresses(cert);
    let mut out: Vec<String> = Vec::new();
    for addr in found.san.iter().chain(found.dn.iter()) {
        if !addr.is_empty() && !out.iter().any(|a| a.eq_ignore_ascii_case(addr)) {
            out.push(addr.clone());
        }
    }
    out
}

// ── Key + certificate generation, certification request ──────────────────────
//
// The certificate and the request are assembled here from `x509-cert`'s component
// types and encoded with `der`: `x509-cert` 0.3 keeps `TbsCertificate`'s fields
// private and builds one only through its `builder` feature, which is off (it adds
// dependency edges this crate does not have). Only the SEQUENCE layout is ours —
// key generation, signatures and every component encoder are the libraries'.
// Every certificate is parsed back with `x509_cert::Certificate` and its
// signature checked before it is returned, and `tests/smime.rs` hands the output
// to `openssl`.

/// Modulus size of a generated RSA key, in bits.
pub const GENERATED_RSA_BITS: usize = 3072;
/// Lifetime of a generated certificate.
pub const GENERATED_VALIDITY_DAYS: u64 = 730;
/// `notBefore` is set this far before the given time, so a recipient whose clock
/// runs slightly behind does not see a certificate that is "not yet valid".
const NOT_BEFORE_SKEW_SECS: u64 = 300;

/// `TBSCertificate` (RFC 5280 §4.1), the v3 fields this module writes.
#[derive(der::Sequence)]
struct TbsCertificateOut {
    #[asn1(context_specific = "0", tag_mode = "EXPLICIT")]
    version: u8,
    serial_number: SerialNumber,
    signature: AlgorithmIdentifierOwned,
    issuer: Name,
    validity: Validity,
    subject: Name,
    subject_public_key_info: SubjectPublicKeyInfoOwned,
    #[asn1(context_specific = "3", tag_mode = "EXPLICIT")]
    extensions: Vec<Extension>,
}

/// `Certificate ::= SEQUENCE { tbsCertificate, signatureAlgorithm, signatureValue }`.
#[derive(der::Sequence)]
struct CertificateOut {
    tbs_certificate: TbsCertificateOut,
    signature_algorithm: AlgorithmIdentifierOwned,
    signature: BitString,
}

/// Generate an RSA key ([`GENERATED_RSA_BITS`]) and a certificate for `email`
/// signed by that key itself, valid for [`GENERATED_VALIDITY_DAYS`] from
/// `now_unix` (seconds since the epoch; the caller supplies the clock because
/// `wasm32-unknown-unknown` has none).
///
/// The certificate is self-signed: no certificate authority vouches for it, so
/// another mail program trusts it only if its user accepts it by hand. Use
/// [`certificate_request`] to ask an authority for one over the same key.
pub fn generate(
    name: &str,
    email: &str,
    passphrase: &str,
    now_unix: u64,
) -> Result<GeneratedSmime> {
    generate_with_bits(name, email, passphrase, now_unix, GENERATED_RSA_BITS)
}

/// [`generate`] with an explicit modulus size (2048, 3072 or 4096 bits).
pub fn generate_with_bits(
    name: &str,
    email: &str,
    passphrase: &str,
    now_unix: u64,
    bits: usize,
) -> Result<GeneratedSmime> {
    let kind = match bits {
        2048 => KeyType::Rsa2048,
        3072 => KeyType::Rsa3072,
        4096 => KeyType::Rsa4096,
        _ => {
            return Err(CryptoError::Input(format!(
                "unsupported RSA key size: {bits}"
            )));
        }
    };
    generate_key(kind, name, email, passphrase, now_unix)
}

/// Generate a key of `kind` and a self-signed certificate for `email`. Key usage
/// follows the key: RSA signs and receives encrypted content keys
/// (digitalSignature, keyEncipherment); P-256/P-384 sign and agree keys
/// (digitalSignature, keyAgreement); Ed25519 only signs (digitalSignature) — a
/// separate encryption certificate is needed for encrypted mail.
pub fn generate_key(
    kind: KeyType,
    name: &str,
    email: &str,
    passphrase: &str,
    now_unix: u64,
) -> Result<GeneratedSmime> {
    if passphrase.is_empty() {
        return Err(CryptoError::Input("a passphrase is required".into()));
    }
    let email = checked_email(email)?;
    let subject = subject_name(name, &email)?;

    let key = PrivateKey::generate(kind)?;
    let spki = key.public_spki()?;

    let mut serial = [0u8; 16];
    rng::fill_random(&mut serial);
    // A serial number is a positive INTEGER (RFC 5280 §4.1.2.2): clear the sign
    // bit, and set the next one so the encoding keeps all 16 octets.
    serial[0] = (serial[0] & 0x7f) | 0x40;

    let not_before = now_unix.saturating_sub(NOT_BEFORE_SKEW_SECS);
    let not_after = now_unix + GENERATED_VALIDITY_DAYS * 86_400;
    let hash = key.default_hash();
    let sig_alg = key.signature_algorithm(hash, false)?;

    let tbs = TbsCertificateOut {
        version: 2, // v3
        serial_number: SerialNumber::new(&serial).map_err(parse)?,
        signature: sig_alg.clone(),
        issuer: subject.clone(),
        validity: Validity::new(x509_time(not_before)?, x509_time(not_after)?),
        subject,
        subject_public_key_info: spki.clone(),
        extensions: vec![
            extension(
                &BasicConstraints {
                    ca: false,
                    path_len_constraint: None,
                },
                true,
            )?,
            extension(&key_usage_for(&key), true)?,
            extension(&ExtendedKeyUsage(vec![oid::KP_EMAIL_PROTECTION]), false)?,
            extension(&subject_alt_name(core::slice::from_ref(&email))?, false)?,
            extension(&subject_key_identifier(&spki)?, false)?,
        ],
    };
    let signature = key.sign(hash, false, &tbs.to_der().map_err(parse)?)?;
    let cert_der = CertificateOut {
        tbs_certificate: tbs,
        signature_algorithm: sig_alg,
        signature: BitString::from_bytes(&signature).map_err(parse)?,
    }
    .to_der()
    .map_err(parse)?;

    // Read our own output back the way a recipient would, and refuse to hand out
    // anything that does not parse, does not verify, or is not for this key.
    let cert = Certificate::from_der(&cert_der)
        .map_err(|e| CryptoError::Sign(format!("generated certificate does not parse: {e}")))?;
    let tbs_der = cert.tbs_certificate().to_der().map_err(parse)?;
    let sig_bytes = cert
        .signature()
        .as_bytes()
        .ok_or_else(|| CryptoError::Sign("generated certificate signature".into()))?;
    pkix_sig::verify(
        &spki,
        cert.signature_algorithm(),
        None,
        &tbs_der,
        sig_bytes,
        Use::Certificate,
    )
    .map_err(|_| CryptoError::Sign("generated certificate does not verify".into()))?;
    if !key.matches(cert.tbs_certificate().subject_public_key_info()) {
        return Err(CryptoError::Sign(
            "generated certificate is not for the key".into(),
        ));
    }

    Ok(GeneratedSmime {
        cert: summarize(&cert)?,
        encrypted_private_bundle: smime_keys::wrap(&key, passphrase)?,
    })
}

fn key_usage_for(key: &PrivateKey) -> KeyUsage {
    match key {
        PrivateKey::Rsa(_) => KeyUsage(KeyUsages::DigitalSignature | KeyUsages::KeyEncipherment),
        PrivateKey::P256(_) | PrivateKey::P384(_) => {
            KeyUsage(KeyUsages::DigitalSignature | KeyUsages::KeyAgreement)
        }
        PrivateKey::Ed25519(_) => KeyUsage(KeyUsages::DigitalSignature.into()),
        PrivateKey::X25519(_) => KeyUsage(KeyUsages::KeyAgreement.into()),
    }
}

/// A PKCS#10 certification request (PEM) for the key in `bundle`, asking for the
/// subject and addresses of `cert_pem` (the certificate currently held for that
/// key). This is the file a certificate authority takes to issue a certificate
/// other mail programs will trust. Refused when the certificate is not for the key.
#[cfg(any(target_arch = "wasm32", feature = "client-private-ops"))]
pub fn certificate_request(
    cert_pem: &str,
    encrypted_private_bundle: &str,
    passphrase: &str,
) -> Result<String> {
    use der::EncodePem;
    use x509_cert::attr::Attribute;
    use x509_cert::request::{CertReq, CertReqInfo, ExtensionReq};
    let cert = parse_certificate(cert_pem.as_bytes())?;
    let key = smime_keys::unwrap(encrypted_private_bundle, passphrase)?;
    ensure_key_matches(&key, &cert)?;

    let mut requested = vec![
        extension(&key_usage_for(&key), true)?,
        extension(&ExtendedKeyUsage(vec![oid::KP_EMAIL_PROTECTION]), false)?,
    ];
    let addresses = cert_email_addresses(&cert);
    if !addresses.is_empty() {
        requested.push(extension(&subject_alt_name(&addresses)?, false)?);
    }
    let info = CertReqInfo {
        version: x509_cert::request::Version::V1,
        subject: cert.tbs_certificate().subject().clone(),
        public_key: key.public_spki()?,
        attributes: SetOfVec::try_from(vec![
            Attribute::try_from(ExtensionReq(requested)).map_err(parse)?,
        ])
        .map_err(parse)?,
    };
    let hash = key.default_hash();
    let signature = key.sign(hash, false, &info.to_der().map_err(parse)?)?;
    CertReq {
        info,
        algorithm: key.signature_algorithm(hash, false)?,
        signature: BitString::from_bytes(&signature).map_err(parse)?,
    }
    .to_pem(der::pem::LineEnding::LF)
    .map_err(parse)
}

/// Check a certificate issued for the key in `bundle` (PEM text or DER bytes, one
/// certificate) and return its summary so the caller can store it in place of the
/// self-signed one. Refused when its public key is not the stored key's — a
/// certificate for some other key cannot be used with this one.
#[cfg(any(target_arch = "wasm32", feature = "client-private-ops"))]
pub fn attach_issued_cert(
    issued_cert: &[u8],
    encrypted_private_bundle: &str,
    passphrase: &str,
) -> Result<CertSummary> {
    let cert = parse_certificate(issued_cert)?;
    let key = smime_keys::unwrap(encrypted_private_bundle, passphrase)?;
    ensure_key_matches(&key, &cert)?;
    summarize(&cert)
}

/// An address this module will put in a certificate: one `local@domain`, ASCII
/// (an `rfc822Name` is an IA5String; an internationalised address needs a
/// different name form, which is not written here), no whitespace or brackets.
fn checked_email(email: &str) -> Result<String> {
    let email = email.trim();
    let ok = email.is_ascii()
        && email.len() <= 254
        && !email
            .bytes()
            .any(|b| b.is_ascii_control() || b.is_ascii_whitespace() || matches!(b, b'<' | b'>'))
        && matches!(email.split_once('@'), Some((l, d)) if !l.is_empty() && !d.is_empty() && !d.contains('@'));
    if ok {
        Ok(email.to_string())
    } else {
        Err(CryptoError::Input(
            "the email address is not one a certificate can carry (ASCII local@domain)".into(),
        ))
    }
}

/// `CN=<name>` (when given) followed by `emailAddress=<email>`, one attribute per
/// RDN. Built from values, never by formatting and re-parsing a DN string, so a
/// name containing `,` `+` or `=` cannot change the structure.
fn subject_name(name: &str, email: &str) -> Result<Name> {
    let name = name.trim();
    // `ub-common-name` (RFC 5280 appendix A.1) is 64 characters.
    if name.chars().count() > 64 || name.chars().any(char::is_control) {
        return Err(CryptoError::Input(
            "the name must be at most 64 characters, with no control characters".into(),
        ));
    }
    let mut rdns: Vec<SetOfVec<AttributeTypeAndValue>> = Vec::new();
    let mut push = |oid: ObjectIdentifier, tag: Tag, value: &str| -> Result<()> {
        let atv = AttributeTypeAndValue {
            oid,
            value: Any::new(tag, value.as_bytes()).map_err(parse)?,
        };
        rdns.push(SetOfVec::try_from(vec![atv]).map_err(parse)?);
        Ok(())
    };
    if !name.is_empty() {
        push(oid::COMMON_NAME, Tag::Utf8String, name)?;
    }
    push(oid::EMAIL_ADDRESS, Tag::Ia5String, email)?;
    Name::from_der(&rdns.to_der().map_err(parse)?).map_err(parse)
}

/// `Time` for a Unix timestamp: UTCTime through 2049, GeneralizedTime from 2050
/// (RFC 5280 §4.1.2.5).
fn x509_time(unix_secs: u64) -> Result<Time> {
    const YEAR_2050: u64 = 2_524_608_000;
    let d = core::time::Duration::from_secs(unix_secs);
    if unix_secs < YEAR_2050 {
        Ok(Time::UtcTime(
            UtcTime::from_unix_duration(d).map_err(parse)?,
        ))
    } else {
        Ok(Time::GeneralTime(
            GeneralizedTime::from_unix_duration(d).map_err(parse)?,
        ))
    }
}

fn extension<T: Encode + const_oid::AssociatedOid>(value: &T, critical: bool) -> Result<Extension> {
    Ok(Extension {
        extn_id: T::OID,
        critical,
        extn_value: OctetString::new(value.to_der().map_err(parse)?).map_err(parse)?,
    })
}

fn subject_alt_name(addresses: &[String]) -> Result<SubjectAltName> {
    let names = addresses
        .iter()
        .map(|a| Ok(GeneralName::Rfc822Name(Ia5String::new(a).map_err(parse)?)))
        .collect::<Result<Vec<_>>>()?;
    Ok(SubjectAltName(names))
}

/// RFC 5280 §4.2.1.2 method 1: SHA-1 over the subjectPublicKey bits. An
/// identifier, not a security property.
fn subject_key_identifier(spki: &SubjectPublicKeyInfoOwned) -> Result<SubjectKeyIdentifier> {
    let digest = sha1::Sha1::digest(spki.subject_public_key.raw_bytes());
    Ok(SubjectKeyIdentifier(
        OctetString::new(digest.as_slice()).map_err(parse)?,
    ))
}

/// The certificate's public key is this private key's.
#[cfg(any(target_arch = "wasm32", feature = "client-private-ops"))]
fn ensure_key_matches(key: &PrivateKey, cert: &Certificate) -> Result<()> {
    if key.matches(cert.tbs_certificate().subject_public_key_info()) {
        Ok(())
    } else {
        Err(CryptoError::Input(
            "the certificate is not for this private key".into(),
        ))
    }
}

// ── private-key bundle helpers ────────────────────────────────────────────────

/// Wrap an RSA private key as a passphrase-encrypted PKCS#8 (PBES2: PBKDF2-SHA256 +
/// AES-256-CBC) PEM bundle.
pub fn wrap_private_key(key: &rsa::RsaPrivateKey, passphrase: &str) -> Result<String> {
    smime_keys::wrap(&PrivateKey::Rsa(Box::new(key.clone())), passphrase)
}

/// A PKCS#8 `PrivateKeyInfo` as a DER `EncryptedPrivateKeyInfo` (PBES2:
/// PBKDF2-HMAC-SHA256 with `rounds` iterations + AES-256-CBC), fresh salt and IV.
/// Salt and IV come from the OS RNG via explicit params, so the wasm build never
/// pulls `getrandom` 0.3+ (plan §1.13).
pub(crate) fn encrypt_pkcs8(pkcs8_der: &[u8], passphrase: &str, rounds: u32) -> Result<Vec<u8>> {
    let mut salt = [0u8; 16];
    let mut iv = [0u8; 16];
    rng::fill_random(&mut salt);
    rng::fill_random(&mut iv);
    let params = pkcs5::pbes2::Parameters::generate_pbkdf2_sha256_aes256cbc(rounds, &salt, iv)
        .map_err(|e| CryptoError::Sign(format!("pbes2 params: {e:?}")))?;
    let scheme = pkcs5::EncryptionScheme::Pbes2(params);
    let ciphertext = scheme
        .encrypt(passphrase, pkcs8_der)
        .map_err(|e| CryptoError::Sign(format!("pbes2 encrypt: {e:?}")))?;
    let epki: pkcs8::EncryptedPrivateKeyInfoOwned = pkcs8::EncryptedPrivateKeyInfo {
        encryption_algorithm: scheme,
        encrypted_data: OctetString::new(ciphertext).map_err(parse)?,
    };
    epki.to_der().map_err(parse)
}

/// The `micalg` for a detached signature by `cert_pem`'s key (the digest
/// [`sign_detached`] uses): `sha-256`, `sha-384` or `sha-512`.
pub fn micalg_for(cert_pem: &str) -> Result<String> {
    let cert = parse_certificate(cert_pem.as_bytes())?;
    let spki = cert.tbs_certificate().subject_public_key_info();
    let hash = match (spki.algorithm.oid, pkix_sig::curve_of(spki)) {
        (oid::EC_PUBLIC_KEY, Some(oid::SECP384R1)) => Hash::Sha384,
        (oid::ED25519, _) => Hash::Sha512,
        _ => Hash::Sha256,
    };
    Ok(crate::mime::smime_micalg(hash.name()))
}
