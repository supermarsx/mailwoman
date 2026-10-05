//! S/MIME over the RustCrypto stack (`cms`/`x509-cert`/`rsa`/`p256`, plan §1.1) —
//! CMS SignedData sign/verify and EnvelopedData encrypt/decrypt (RSA key transport +
//! AES-256-CBC content, the most interoperable S/MIME profile; AuthEnvelopedData/
//! AES-GCM is a follow-up), PKCS#12 import (private-key material — the browser wasm
//! side), cert harvesting from signed mail, and trust evaluation.
//!
//! We build the CMS structures from the cms *core* ASN.1 types and drive RSA/AES/SHA
//! ourselves rather than using the cms `builder` feature — that pre-release builder
//! only compiles against exact-rc cipher/elliptic-curve versions since superseded.
//!
//! The S/MIME `encryptedPrivateBundle` (frozen §2.3) is the imported private key
//! re-wrapped as a passphrase-encrypted PKCS#8 (PBES2) — never emitted in the clear.

use cms::cert::{CertificateChoices, IssuerAndSerialNumber};
use cms::content_info::{CmsVersion, ContentInfo};
#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
use cms::enveloped_data::OriginatorInfo;
use cms::enveloped_data::{
    EncryptedContentInfo, EnvelopedData, KeyTransRecipientInfo, RecipientIdentifier, RecipientInfo,
    RecipientInfos,
};
use cms::signed_data::{
    CertificateSet, EncapsulatedContentInfo, SignedData, SignerIdentifier, SignerInfo, SignerInfos,
};
use const_oid::ObjectIdentifier;
use const_oid::db::rfc5911::{ID_DATA, ID_ENCRYPTED_DATA, ID_ENVELOPED_DATA, ID_SIGNED_DATA};
use der::asn1::{BitString, GeneralizedTime, Ia5String, OctetString, SetOfVec, UtcTime};
use der::{Any, Decode, DecodePem, Encode, EncodePem, Tag};
use rsa::pkcs1v15;
use rsa::pkcs8::{DecodePrivateKey, EncodePublicKey};
use rsa::traits::PublicKeyParts;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use sha2::{Digest, Sha256};
use signature::{SignatureEncoding, Signer, Verifier};
use spki::{AlgorithmIdentifierOwned, DecodePublicKey, SubjectPublicKeyInfoOwned};
use x509_cert::Certificate;
use x509_cert::attr::{Attribute, AttributeTypeAndValue};
use x509_cert::ext::Extension;
use x509_cert::ext::pkix::name::GeneralName;
use x509_cert::ext::pkix::{
    BasicConstraints, ExtendedKeyUsage, KeyUsage, KeyUsages, SubjectAltName, SubjectKeyIdentifier,
};
use x509_cert::name::Name;
use x509_cert::request::{CertReq, CertReqInfo, ExtensionReq};
use x509_cert::serial_number::SerialNumber;
use x509_cert::time::{Time, Validity};

use crate::error::{CryptoError, Result};
use crate::rng;
use crate::types::{CryptoKey, KeyHistoryEntry, SignatureVerdict};

const OID_RSA_ENCRYPTION: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.1");
const OID_AES_256_CBC: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.1.42");
const OID_SHA_256: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.1");
const OID_CONTENT_TYPE: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.9.3");
const OID_MESSAGE_DIGEST: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.9.4");
/// `id-ct-authEnvelopedData` (RFC 5083) — the AEAD S/MIME content type.
const OID_AUTH_ENVELOPED_DATA: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("1.2.840.113549.1.9.16.1.23");
/// `id-aes256-GCM` (RFC 5084) — AES-256 in Galois/Counter Mode.
const OID_AES_256_GCM: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.1.46");
/// AES-GCM authentication-tag length in octets (128-bit tag).
#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
const GCM_TAG_LEN: usize = 16;

/// `sha256WithRSAEncryption` (RFC 8017) — the signature on generated certificates
/// and certification requests.
const OID_SHA256_WITH_RSA: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.11");
/// `id-at-commonName` (X.520).
const OID_COMMON_NAME: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.5.4.3");
/// PKCS#9 `emailAddress` — an address carried in a distinguished name.
const OID_EMAIL_ADDRESS: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.9.1");
/// `id-kp-emailProtection` (RFC 5280 §4.2.1.12).
const OID_KP_EMAIL_PROTECTION: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.3.6.1.5.5.7.3.4");
/// `id-ce-subjectAltName` (RFC 5280 §4.2.1.6).
const OID_SUBJECT_ALT_NAME: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.5.29.17");

/// Result of [`import_pkcs12`].
pub struct Pkcs12Import {
    pub cert_pem: String,
    pub fingerprint: String,
    pub encrypted_private_bundle: String,
    /// The addresses the certificate names (subject alternative name, then subject).
    pub addresses: Vec<String>,
    /// The certificate's public-key algorithm, read from the certificate.
    pub algorithm: String,
    /// The certificate's `notAfter`, RFC 3339.
    pub not_after: String,
}

/// What a certificate says about itself — read back from its DER, never from the
/// arguments it was made with.
#[derive(Debug, Clone)]
pub struct CertSummary {
    pub cert_pem: String,
    /// SHA-256 over the certificate DER, upper-case hex.
    pub fingerprint: String,
    pub addresses: Vec<String>,
    /// `rsa-<modulus bits>`, `ecdsa-p256`, or the public-key algorithm OID.
    pub algorithm: String,
    /// RFC 3339.
    pub not_before: String,
    /// RFC 3339.
    pub not_after: String,
    /// Issuer and subject are the same name: no certificate authority is behind it.
    pub self_issued: bool,
}

/// Result of [`generate`]: a new RSA key (passphrase-wrapped) and a certificate for
/// it signed by that same key.
pub struct GeneratedSmime {
    pub cert: CertSummary,
    pub encrypted_private_bundle: String,
}

fn parse(e: impl std::fmt::Display) -> CryptoError {
    CryptoError::Parse(e.to_string())
}

fn alg(oid: ObjectIdentifier, parameters: Option<Any>) -> AlgorithmIdentifierOwned {
    AlgorithmIdentifierOwned { oid, parameters }
}

fn issuer_and_serial(cert: &Certificate) -> IssuerAndSerialNumber {
    IssuerAndSerialNumber {
        issuer: cert.tbs_certificate().issuer().clone(),
        serial_number: cert.tbs_certificate().serial_number().clone(),
    }
}

// ── Sign / verify (CMS SignedData, opaque, signed attributes) ────────────────

/// Sign `data` with the passphrase-locked `bundle` + `cert_pem`, producing an
/// opaque CMS SignedData (base64 DER). RSA-PKCS#1v1.5 + SHA-256 over signed
/// attributes (contentType + messageDigest), the S/MIME norm.
pub fn sign(
    data: &[u8],
    cert_pem: &str,
    encrypted_private_bundle: &str,
    passphrase: &str,
) -> Result<String> {
    let cert = Certificate::from_pem(cert_pem).map_err(parse)?;
    let key = load_rsa(encrypted_private_bundle, Some(passphrase))?;
    let signer = pkcs1v15::SigningKey::<Sha256>::new(key);

    let digest = Sha256::digest(data);
    let signed_attrs: SetOfVec<Attribute> = SetOfVec::try_from(vec![
        attribute(OID_CONTENT_TYPE, Any::encode_from(&ID_DATA).map_err(parse)?)?,
        attribute(
            OID_MESSAGE_DIGEST,
            Any::new(Tag::OctetString, digest.as_slice()).map_err(parse)?,
        )?,
    ])
    .map_err(parse)?;

    // The signature is over the DER of the attributes as a SET (RFC 5652 §5.4).
    let signed_bytes = signed_attrs.to_der().map_err(parse)?;
    let signature = signer
        .try_sign(&signed_bytes)
        .map_err(|e| CryptoError::Sign(e.to_string()))?;

    let si = SignerInfo {
        version: CmsVersion::V1,
        sid: SignerIdentifier::IssuerAndSerialNumber(issuer_and_serial(&cert)),
        digest_alg: alg(OID_SHA_256, None),
        signed_attrs: Some(signed_attrs),
        signature_algorithm: alg(OID_RSA_ENCRYPTION, Some(Any::null())),
        signature: OctetString::new(signature.to_bytes().as_ref()).map_err(parse)?,
        unsigned_attrs: None,
    };

    let content = EncapsulatedContentInfo {
        econtent_type: ID_DATA,
        econtent: Some(Any::new(Tag::OctetString, data).map_err(parse)?),
    };
    let sd = SignedData {
        version: CmsVersion::V1,
        digest_algorithms: SetOfVec::try_from(vec![alg(OID_SHA_256, None)]).map_err(parse)?,
        encap_content_info: content,
        certificates: Some(CertificateSet(
            SetOfVec::try_from(vec![CertificateChoices::Certificate(cert)]).map_err(parse)?,
        )),
        crls: None,
        signer_infos: SignerInfos(SetOfVec::try_from(vec![si]).map_err(parse)?),
    };
    let ci = ContentInfo {
        content_type: ID_SIGNED_DATA,
        content: Any::encode_from(&sd).map_err(parse)?,
    };
    use base64::Engine;
    Ok(base64::engine::general_purpose::STANDARD.encode(ci.to_der().map_err(parse)?))
}

/// Verify a CMS SignedData (DER bytes) against its embedded certificate, returning
/// the frozen 3-state [`SignatureVerdict`]. Handles signed-attributes messages,
/// checking the messageDigest attribute against the content.
pub fn verify(cms_der: &[u8]) -> Result<SignatureVerdict> {
    let ci = ContentInfo::from_der(cms_der).map_err(parse)?;
    let sd = ci
        .content
        .decode_as::<SignedData>()
        .map_err(|_| CryptoError::Parse("not a SignedData".into()))?;

    let si = sd
        .signer_infos
        .0
        .as_slice()
        .first()
        .ok_or_else(|| CryptoError::Parse("no signer info".into()))?;

    let econtent = sd
        .encap_content_info
        .econtent
        .as_ref()
        .map(|a| a.value().to_vec())
        .unwrap_or_default();

    let cert = sd
        .certificates
        .as_ref()
        .and_then(|set| {
            set.0.iter().find_map(|c| match c {
                CertificateChoices::Certificate(cert) => Some(cert.clone()),
                _ => None,
            })
        })
        .ok_or_else(|| CryptoError::Parse("no embedded certificate".into()))?;

    let spki_der = cert
        .tbs_certificate()
        .subject_public_key_info()
        .to_der()
        .map_err(parse)?;

    let signed_message = match &si.signed_attrs {
        Some(attrs) => {
            let want = Sha256::digest(&econtent);
            let got = attrs
                .iter()
                .find(|a| a.oid == OID_MESSAGE_DIGEST)
                .and_then(|a| {
                    a.values
                        .as_slice()
                        .first()
                        .and_then(|v| v.decode_as::<OctetString>().ok())
                });
            match got {
                Some(md) if md.as_bytes() == want.as_slice() => {}
                _ => return Ok(verdict("invalid", None)),
            }
            SetOfVec::from_iter(attrs.iter().cloned())
                .map_err(parse)?
                .to_der()
                .map_err(parse)?
        }
        None => econtent.clone(),
    };

    let status = verify_signature(&spki_der, &signed_message, si.signature.as_bytes());
    let key_id = hex::encode(Sha256::digest(&spki_der))[..16].to_uppercase();
    Ok(verdict(status, Some(key_id)))
}

fn verify_signature(spki_der: &[u8], message: &[u8], signature: &[u8]) -> &'static str {
    if let Ok(rsa_pub) = RsaPublicKey::from_public_key_der(spki_der) {
        let vk = pkcs1v15::VerifyingKey::<Sha256>::new(rsa_pub);
        return match pkcs1v15::Signature::try_from(signature) {
            Ok(sig) if vk.verify(message, &sig).is_ok() => "verified",
            _ => "invalid",
        };
    }
    if let Ok(vk) = p256::ecdsa::VerifyingKey::from_public_key_der(spki_der) {
        return match p256::ecdsa::DerSignature::try_from(signature) {
            Ok(sig) if Verifier::verify(&vk, message, &sig).is_ok() => "verified",
            _ => "invalid",
        };
    }
    "unverified-key"
}

// ── Encrypt / decrypt (CMS EnvelopedData) ────────────────────────────────────

/// Encrypt `plaintext` to each recipient cert (PEM), RSA key transport +
/// AES-256-CBC content. Returns a DER `ContentInfo(EnvelopedData)`.
pub fn encrypt(plaintext: &[u8], recipient_cert_pems: &[String]) -> Result<Vec<u8>> {
    if recipient_cert_pems.is_empty() {
        return Err(CryptoError::Input("no recipients".into()));
    }
    let mut cek = [0u8; 32];
    let mut iv = [0u8; 16];
    rng::fill_random(&mut cek);
    rng::fill_random(&mut iv);
    let mut rng = rng::rc10();

    let ciphertext = aes256_cbc_encrypt(&cek, &iv, plaintext)?;

    let mut recipients = Vec::new();
    for pem in recipient_cert_pems {
        let cert = Certificate::from_pem(pem).map_err(parse)?;
        let spki_der = cert
            .tbs_certificate()
            .subject_public_key_info()
            .to_der()
            .map_err(parse)?;
        let rsa_pub = RsaPublicKey::from_public_key_der(&spki_der)
            .map_err(|e| CryptoError::Encrypt(format!("recipient key not RSA: {e}")))?;
        let enc_cek = rsa_pub
            .encrypt(&mut rng, Pkcs1v15Encrypt, &cek)
            .map_err(|e| CryptoError::Encrypt(e.to_string()))?;
        recipients.push(RecipientInfo::Ktri(KeyTransRecipientInfo {
            version: CmsVersion::V0,
            rid: RecipientIdentifier::IssuerAndSerialNumber(issuer_and_serial(&cert)),
            key_enc_alg: alg(OID_RSA_ENCRYPTION, Some(Any::null())),
            enc_key: OctetString::new(enc_cek).map_err(parse)?,
        }));
    }

    let eci = EncryptedContentInfo {
        content_type: ID_DATA,
        content_enc_alg: alg(
            OID_AES_256_CBC,
            Some(Any::new(Tag::OctetString, &iv[..]).map_err(parse)?),
        ),
        encrypted_content: Some(OctetString::new(ciphertext).map_err(parse)?),
    };
    let ed = EnvelopedData {
        version: CmsVersion::V0,
        originator_info: None,
        recip_infos: RecipientInfos(SetOfVec::try_from(recipients).map_err(parse)?),
        encrypted_content: eci,
        unprotected_attrs: None,
    };
    let ci = ContentInfo {
        content_type: ID_ENVELOPED_DATA,
        content: Any::encode_from(&ed).map_err(parse)?,
    };
    ci.to_der().map_err(parse)
}

/// Decrypt a DER `ContentInfo(EnvelopedData)` with the passphrase-locked `bundle`.
/// Handles RSA key-transport recipients + AES-CBC content.
pub fn decrypt(
    enveloped_der: &[u8],
    encrypted_private_bundle: &str,
    passphrase: &str,
) -> Result<Vec<u8>> {
    let ci = ContentInfo::from_der(enveloped_der).map_err(parse)?;
    let ed = ci
        .content
        .decode_as::<EnvelopedData>()
        .map_err(|_| CryptoError::Parse("not an EnvelopedData".into()))?;
    let key = load_rsa(encrypted_private_bundle, Some(passphrase))?;

    let cek = ed
        .recip_infos
        .0
        .iter()
        .find_map(|ri| match ri {
            RecipientInfo::Ktri(ktri) => key.decrypt(Pkcs1v15Encrypt, ktri.enc_key.as_bytes()).ok(),
            _ => None,
        })
        .ok_or_else(|| CryptoError::Decrypt("no usable RSA recipient".into()))?;

    let eci = &ed.encrypted_content;
    let iv = eci
        .content_enc_alg
        .parameters
        .as_ref()
        .ok_or_else(|| CryptoError::Decrypt("no content IV".into()))?
        .decode_as::<OctetString>()
        .map_err(parse)?;
    let ct = eci
        .encrypted_content
        .as_ref()
        .ok_or_else(|| CryptoError::Decrypt("no encrypted content".into()))?;

    aes256_cbc_decrypt(&cek, iv.as_bytes(), ct.as_bytes())
}

// ── Authenticated encryption (CMS AuthEnvelopedData, AES-256-GCM, RFC 5083/5084) ─
//
// The AEAD S/MIME profile (`id-ct-authEnvelopedData`): RSA key transport of the
// content-encryption key + AES-256-GCM content, the GCM authentication tag carried
// in the `mac` field (NOT appended to `encryptedContent`, RFC 5083 §2.1). Native-
// scoped — `aes-gcm` is a native-only dependency, and the GCM AEAD path is the
// server/interop side; the CBC profile above stays available on both targets.

/// Encrypt `plaintext` to each recipient cert (PEM), RSA key transport +
/// AES-256-GCM content (CMS AuthEnvelopedData, RFC 5083). Returns a DER
/// `ContentInfo(AuthEnvelopedData)`. The 16-byte GCM tag rides the `mac` field.
#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub fn encrypt_authenveloped(plaintext: &[u8], recipient_cert_pems: &[String]) -> Result<Vec<u8>> {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Nonce};

    if recipient_cert_pems.is_empty() {
        return Err(CryptoError::Input("no recipients".into()));
    }
    let mut cek = [0u8; 32];
    let mut nonce_bytes = [0u8; 12];
    rng::fill_random(&mut cek);
    rng::fill_random(&mut nonce_bytes);
    let mut rng = rng::rc10();

    let cipher = Aes256Gcm::new((&cek).into());
    let mut sealed = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), plaintext)
        .map_err(|_| CryptoError::Encrypt("aes-gcm seal".into()))?;
    // aes-gcm appends the 16-byte tag; CMS carries it in `mac`, so split it off.
    if sealed.len() < GCM_TAG_LEN {
        return Err(CryptoError::Encrypt("aes-gcm output too short".into()));
    }
    let tag = sealed.split_off(sealed.len() - GCM_TAG_LEN);
    let ciphertext = sealed;

    let mut recipients = Vec::new();
    for pem in recipient_cert_pems {
        let cert = Certificate::from_pem(pem).map_err(parse)?;
        let spki_der = cert
            .tbs_certificate()
            .subject_public_key_info()
            .to_der()
            .map_err(parse)?;
        let rsa_pub = RsaPublicKey::from_public_key_der(&spki_der)
            .map_err(|e| CryptoError::Encrypt(format!("recipient key not RSA: {e}")))?;
        let enc_cek = rsa_pub
            .encrypt(&mut rng, Pkcs1v15Encrypt, &cek)
            .map_err(|e| CryptoError::Encrypt(e.to_string()))?;
        recipients.push(RecipientInfo::Ktri(KeyTransRecipientInfo {
            version: CmsVersion::V0,
            rid: RecipientIdentifier::IssuerAndSerialNumber(issuer_and_serial(&cert)),
            key_enc_alg: alg(OID_RSA_ENCRYPTION, Some(Any::null())),
            enc_key: OctetString::new(enc_cek).map_err(parse)?,
        }));
    }

    let gcm_params = GcmParameters {
        nonce: OctetString::new(&nonce_bytes[..]).map_err(parse)?,
        icv_len: GCM_TAG_LEN as u32,
    };
    let eci = EncryptedContentInfo {
        content_type: ID_DATA,
        content_enc_alg: alg(
            OID_AES_256_GCM,
            Some(Any::encode_from(&gcm_params).map_err(parse)?),
        ),
        encrypted_content: Some(OctetString::new(ciphertext).map_err(parse)?),
    };
    let aed = AuthEnvelopedData {
        version: CmsVersion::V0,
        originator_info: None,
        recip_infos: RecipientInfos(SetOfVec::try_from(recipients).map_err(parse)?),
        auth_encrypted_content_info: eci,
        auth_attrs: None,
        mac: OctetString::new(tag).map_err(parse)?,
        unauth_attrs: None,
    };
    let ci = ContentInfo {
        content_type: OID_AUTH_ENVELOPED_DATA,
        content: Any::encode_from(&aed).map_err(parse)?,
    };
    ci.to_der().map_err(parse)
}

/// Decrypt a DER `ContentInfo(AuthEnvelopedData)` (AES-256-GCM content, RSA
/// key-transport recipients) with the passphrase-locked `bundle`. The GCM tag is
/// read from the `mac` field and verified as part of the AEAD open, so a tampered
/// ciphertext or tag fails (returns a decrypt error).
#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub fn decrypt_authenveloped(
    auth_enveloped_der: &[u8],
    encrypted_private_bundle: &str,
    passphrase: &str,
) -> Result<Vec<u8>> {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes256Gcm, Nonce};

    let ci = ContentInfo::from_der(auth_enveloped_der).map_err(parse)?;
    let aed = ci
        .content
        .decode_as::<AuthEnvelopedData>()
        .map_err(|_| CryptoError::Parse("not an AuthEnvelopedData".into()))?;
    let key = load_rsa(encrypted_private_bundle, Some(passphrase))?;

    let cek = aed
        .recip_infos
        .0
        .iter()
        .find_map(|ri| match ri {
            RecipientInfo::Ktri(ktri) => key.decrypt(Pkcs1v15Encrypt, ktri.enc_key.as_bytes()).ok(),
            _ => None,
        })
        .ok_or_else(|| CryptoError::Decrypt("no usable RSA recipient".into()))?;
    if cek.len() != 32 {
        return Err(CryptoError::Decrypt("bad AES-256-GCM key length".into()));
    }

    let eci = &aed.auth_encrypted_content_info;
    if eci.content_enc_alg.oid != OID_AES_256_GCM {
        return Err(CryptoError::Decrypt("content not AES-256-GCM".into()));
    }
    let params: GcmParameters = eci
        .content_enc_alg
        .parameters
        .as_ref()
        .ok_or_else(|| CryptoError::Decrypt("no GCM parameters".into()))?
        .decode_as()
        .map_err(parse)?;
    let ct = eci
        .encrypted_content
        .as_ref()
        .ok_or_else(|| CryptoError::Decrypt("no encrypted content".into()))?;

    // Reassemble ciphertext‖tag for the AEAD open (aes-gcm expects the tag appended).
    let mut sealed = ct.as_bytes().to_vec();
    sealed.extend_from_slice(aed.mac.as_bytes());

    let cipher = Aes256Gcm::new(cek.as_slice().into());
    cipher
        .decrypt(
            Nonce::from_slice(params.nonce.as_bytes()),
            sealed.as_slice(),
        )
        .map_err(|_| CryptoError::Decrypt("aes-gcm open (bad key/tag/ciphertext)".into()))
}

fn aes256_cbc_encrypt(key: &[u8], iv: &[u8], pt: &[u8]) -> Result<Vec<u8>> {
    use aes::Aes256;
    use cbc::cipher::{BlockModeEncrypt, KeyIvInit, block_padding::Pkcs7};
    let enc = cbc::Encryptor::<Aes256>::new_from_slices(key, iv)
        .map_err(|e| CryptoError::Encrypt(e.to_string()))?;
    Ok(enc.encrypt_padded_vec::<Pkcs7>(pt))
}

fn aes256_cbc_decrypt(key: &[u8], iv: &[u8], ct: &[u8]) -> Result<Vec<u8>> {
    use aes::Aes256;
    use cbc::cipher::{BlockModeDecrypt, KeyIvInit, block_padding::Pkcs7};
    if key.len() != 32 || iv.len() != 16 {
        return Err(CryptoError::Decrypt("bad AES-256-CBC key/iv length".into()));
    }
    let dec = cbc::Decryptor::<Aes256>::new_from_slices(key, iv)
        .map_err(|e| CryptoError::Decrypt(e.to_string()))?;
    dec.decrypt_padded_vec::<Pkcs7>(ct)
        .map_err(|e| CryptoError::Decrypt(e.to_string()))
}

// ── PKCS#12 import ───────────────────────────────────────────────────────────

/// Import a PKCS#12 (.p12/.pfx) bundle, returning the certificate PEM + fingerprint
/// and the private key re-wrapped as a passphrase-encrypted PKCS#8 bundle (the
/// browser vault stores that; the plaintext key never leaves the worker, §2.3).
pub fn import_pkcs12(p12_bytes: &[u8], password: &str) -> Result<Pkcs12Import> {
    let pfx = Pfx::from_der(p12_bytes).map_err(|e| CryptoError::Pkcs12(e.to_string()))?;

    let auth_safe_bytes = pfx
        .auth_safe
        .content
        .decode_as::<OctetString>()
        .map_err(|e| CryptoError::Pkcs12(format!("auth_safe: {e}")))?;
    let safes = SafesSeq::from_der(auth_safe_bytes.as_bytes())
        .map_err(|e| CryptoError::Pkcs12(format!("authenticated safe: {e}")))?;

    let mut cert_der: Option<Vec<u8>> = None;
    let mut key_pkcs8: Option<Vec<u8>> = None;

    for ci in safes.0 {
        let safe_contents_der: Vec<u8> = if ci.content_type == ID_DATA {
            ci.content
                .decode_as::<OctetString>()
                .map_err(|e| CryptoError::Pkcs12(format!("safe data: {e}")))?
                .as_bytes()
                .to_vec()
        } else if ci.content_type == ID_ENCRYPTED_DATA {
            decrypt_encrypted_data(&ci, password)?
        } else {
            continue;
        };

        let bags = SafeContentsSeq::from_der(&safe_contents_der)
            .map_err(|e| CryptoError::Pkcs12(format!("safe contents: {e}")))?;
        for bag in bags.0 {
            let bag_value_der = bag.bag_value.to_der().map_err(parse)?;
            if bag.bag_id == OID_CERT_BAG {
                let cert_bag = CertBag::from_der(&bag_value_der)
                    .map_err(|e| CryptoError::Pkcs12(format!("cert bag: {e}")))?;
                cert_der.get_or_insert(cert_bag.cert_value.as_bytes().to_vec());
            } else if bag.bag_id == OID_PKCS8_SHROUDED_KEY_BAG {
                let epki = pkcs8::EncryptedPrivateKeyInfoOwned::from_der(&bag_value_der)
                    .map_err(|e| CryptoError::Pkcs12(format!("shrouded key: {e}")))?;
                let doc = epki
                    .decrypt(password.as_bytes())
                    .map_err(|e| CryptoError::Pkcs12(format!("key decrypt: {e}")))?;
                key_pkcs8.get_or_insert(doc.as_bytes().to_vec());
            } else if bag.bag_id == OID_KEY_BAG {
                key_pkcs8.get_or_insert(bag_value_der);
            }
        }
    }

    let cert_der =
        cert_der.ok_or_else(|| CryptoError::Pkcs12("no certificate in bundle".into()))?;
    let key_pkcs8 =
        key_pkcs8.ok_or_else(|| CryptoError::Pkcs12("no private key in bundle".into()))?;

    let cert = Certificate::from_der(&cert_der)
        .map_err(|e| CryptoError::Pkcs12(format!("certificate: {e}")))?;
    let summary = summarize(&cert)?;

    let rsa = RsaPrivateKey::from_pkcs8_der(&key_pkcs8)
        .map_err(|e| CryptoError::Pkcs12(format!("key parse: {e}")))?;
    let encrypted_private_bundle = wrap_private_key(&rsa, password)?;

    Ok(Pkcs12Import {
        cert_pem: summary.cert_pem,
        fingerprint: summary.fingerprint,
        encrypted_private_bundle,
        addresses: summary.addresses,
        algorithm: summary.algorithm,
        not_after: summary.not_after,
    })
}

/// Decrypt an id-encryptedData ContentInfo's SafeContents via PBES2 with `password`.
fn decrypt_encrypted_data(ci: &ContentInfo, password: &str) -> Result<Vec<u8>> {
    use cms::encrypted_data::EncryptedData;
    let ed = ci
        .content
        .decode_as::<EncryptedData>()
        .map_err(|e| CryptoError::Pkcs12(format!("encrypted data: {e}")))?;
    let eci = ed.enc_content_info;
    let alg_der = eci.content_enc_alg.to_der().map_err(parse)?;
    let alg_ref = spki::AlgorithmIdentifierRef::from_der(&alg_der).map_err(parse)?;
    let scheme = pkcs5::EncryptionScheme::try_from(alg_ref)
        .map_err(|e| CryptoError::Pkcs12(format!("unsupported PBE: {e:?}")))?;
    let ct = eci
        .encrypted_content
        .ok_or_else(|| CryptoError::Pkcs12("no encrypted safe content".into()))?;
    scheme
        .decrypt(password, ct.as_bytes())
        .map_err(|e| CryptoError::Pkcs12(format!("safe decrypt: {e}")))
}

// ── Cert harvesting + trust ──────────────────────────────────────────────────

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

fn cert_to_crypto_key(cert: &Certificate) -> Result<CryptoKey> {
    let summary = summarize(cert)?;
    let fingerprint = summary.fingerprint;
    Ok(CryptoKey {
        id: format!("smime:{fingerprint}"),
        kind: "smime".into(),
        is_own: false,
        addresses: summary.addresses,
        fingerprint: fingerprint.clone(),
        key_id: fingerprint[..16.min(fingerprint.len())].to_string(),
        algorithm: summary.algorithm,
        created_at: chrono::Utc::now().to_rfc3339(),
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
            seen_at: chrono::Utc::now().to_rfc3339(),
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
        algorithm: cert_algorithm(cert)?,
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
    let trimmed = bytes.trim_ascii_start();
    if trimmed.starts_with(b"-----BEGIN") {
        Certificate::from_pem(trimmed).map_err(parse)
    } else {
        Certificate::from_der(bytes).map_err(parse)
    }
}

fn rfc3339(time: Time) -> Result<String> {
    let out_of_range = || CryptoError::Parse("certificate time out of range".into());
    let secs = i64::try_from(time.to_unix_duration().as_secs()).map_err(|_| out_of_range())?;
    chrono::DateTime::<chrono::Utc>::from_timestamp(secs, 0)
        .map(|t| t.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
        .ok_or_else(out_of_range)
}

/// The public-key algorithm of a certificate, from its SubjectPublicKeyInfo:
/// `rsa-<modulus bits>`, `ecdsa-p256`, or the algorithm OID for anything else.
fn cert_algorithm(cert: &Certificate) -> Result<String> {
    let spki = cert.tbs_certificate().subject_public_key_info();
    let spki_der = spki.to_der().map_err(parse)?;
    if let Ok(rsa_pub) = RsaPublicKey::from_public_key_der(&spki_der) {
        return Ok(format!("rsa-{}", rsa_pub.n().bits()));
    }
    if p256::ecdsa::VerifyingKey::from_public_key_der(&spki_der).is_ok() {
        return Ok("ecdsa-p256".into());
    }
    Ok(spki.algorithm.oid.to_string())
}

/// Email addresses a certificate names: the subject alternative name's
/// `rfc822Name` entries first (where RFC 8550 §4.4.3 puts them), then any PKCS#9
/// `emailAddress` in the subject DN. Duplicates differing only in case are dropped.
fn cert_email_addresses(cert: &Certificate) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut push = |addr: &str| {
        let addr = addr.trim();
        if !addr.is_empty() && !out.iter().any(|a| a.eq_ignore_ascii_case(addr)) {
            out.push(addr.to_string());
        }
    };
    for ext in cert.tbs_certificate().extensions().into_iter().flatten() {
        if ext.extn_id != OID_SUBJECT_ALT_NAME {
            continue;
        }
        if let Ok(san) = SubjectAltName::from_der(ext.extn_value.as_bytes()) {
            for name in &san.0 {
                if let GeneralName::Rfc822Name(addr) = name {
                    push(addr.as_str());
                }
            }
        }
    }
    for rdn in cert.tbs_certificate().subject().iter_rdn() {
        for atv in rdn.iter() {
            if atv.oid != OID_EMAIL_ADDRESS {
                continue;
            }
            if let Ok(addr) = core::str::from_utf8(atv.value.value()) {
                push(addr);
            }
        }
    }
    out
}

// ── Key + certificate generation, certification request, PKCS#12 export ───────
//
// The certificate and the request are assembled here from `x509-cert`'s component
// types and encoded with `der`: `x509-cert` 0.3 keeps `TbsCertificate`'s fields
// private and builds one only through its `builder` feature, which is off (it adds
// dependency edges this crate does not have). Only the SEQUENCE layout is ours —
// the RSA key generation, the SHA-256/PKCS#1 v1.5 signature and every component
// encoder are the libraries'. Every certificate is parsed back with
// `x509_cert::Certificate` and its signature checked before it is returned, and
// `tests/smime.rs` hands the output to `openssl`.

/// Modulus size of a generated key, in bits.
pub const GENERATED_RSA_BITS: usize = 3072;
/// Lifetime of a generated certificate.
pub const GENERATED_VALIDITY_DAYS: u64 = 730;
/// `notBefore` is set this far before the given time, so a recipient whose clock
/// runs slightly behind does not see a certificate that is "not yet valid".
const NOT_BEFORE_SKEW_SECS: u64 = 300;
/// PBKDF2 rounds for the key inside an exported PKCS#12. Lower than the vault's
/// own wrap ([`wrap_private_key`]): Windows refuses a PKCS#12 whose iteration
/// counts sum past 600 000, and other importers apply similar limits.
const PKCS12_PBKDF2_ROUNDS: u32 = 210_000;

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
    if !matches!(bits, 2048 | 3072 | 4096) {
        return Err(CryptoError::Input(format!(
            "unsupported RSA key size: {bits}"
        )));
    }
    if passphrase.is_empty() {
        return Err(CryptoError::Input("a passphrase is required".into()));
    }
    let email = checked_email(email)?;
    let subject = subject_name(name, &email)?;

    let key = RsaPrivateKey::new(&mut rng::rc10(), bits)
        .map_err(|e| CryptoError::Sign(format!("key generation: {e}")))?;
    let spki = public_key_info(&key)?;

    let mut serial = [0u8; 16];
    rng::fill_random(&mut serial);
    // A serial number is a positive INTEGER (RFC 5280 §4.1.2.2): clear the sign
    // bit, and set the next one so the encoding keeps all 16 octets.
    serial[0] = (serial[0] & 0x7f) | 0x40;

    let not_before = now_unix.saturating_sub(NOT_BEFORE_SKEW_SECS);
    let not_after = now_unix + GENERATED_VALIDITY_DAYS * 86_400;

    let tbs = TbsCertificateOut {
        version: 2, // v3
        serial_number: SerialNumber::new(&serial).map_err(parse)?,
        signature: alg(OID_SHA256_WITH_RSA, Some(Any::null())),
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
            extension(&email_key_usage(), true)?,
            extension(&ExtendedKeyUsage(vec![OID_KP_EMAIL_PROTECTION]), false)?,
            extension(&subject_alt_name(core::slice::from_ref(&email))?, false)?,
            extension(&subject_key_identifier(&spki)?, false)?,
        ],
    };
    let signature = rsa_sha256_sign(&key, &tbs.to_der().map_err(parse)?)?;
    let cert_der = CertificateOut {
        tbs_certificate: tbs,
        signature_algorithm: alg(OID_SHA256_WITH_RSA, Some(Any::null())),
        signature: BitString::from_bytes(&signature).map_err(parse)?,
    }
    .to_der()
    .map_err(parse)?;

    // Read our own output back the way a recipient would, and refuse to hand out
    // anything that does not parse, does not verify, or is not for this key.
    let cert = Certificate::from_der(&cert_der)
        .map_err(|e| CryptoError::Sign(format!("generated certificate does not parse: {e}")))?;
    let tbs_der = cert.tbs_certificate().to_der().map_err(parse)?;
    let spki_der = spki.to_der().map_err(parse)?;
    if verify_signature(&spki_der, &tbs_der, cert.signature().raw_bytes()) != "verified" {
        return Err(CryptoError::Sign(
            "generated certificate does not verify".into(),
        ));
    }
    ensure_key_matches(&key, &cert)?;

    Ok(GeneratedSmime {
        cert: summarize(&cert)?,
        encrypted_private_bundle: wrap_private_key(&key, passphrase)?,
    })
}

/// A PKCS#10 certification request (PEM) for the key in `bundle`, asking for the
/// subject and addresses of `cert_pem` (the certificate currently held for that
/// key). This is the file a certificate authority takes to issue a certificate
/// other mail programs will trust. Refused when the certificate is not for the key.
pub fn certificate_request(
    cert_pem: &str,
    encrypted_private_bundle: &str,
    passphrase: &str,
) -> Result<String> {
    let cert = Certificate::from_pem(cert_pem).map_err(parse)?;
    let key = load_rsa(encrypted_private_bundle, Some(passphrase))?;
    ensure_key_matches(&key, &cert)?;

    let mut requested = vec![
        extension(&email_key_usage(), true)?,
        extension(&ExtendedKeyUsage(vec![OID_KP_EMAIL_PROTECTION]), false)?,
    ];
    let addresses = cert_email_addresses(&cert);
    if !addresses.is_empty() {
        requested.push(extension(&subject_alt_name(&addresses)?, false)?);
    }
    let info = CertReqInfo {
        version: x509_cert::request::Version::V1,
        subject: cert.tbs_certificate().subject().clone(),
        public_key: public_key_info(&key)?,
        attributes: SetOfVec::try_from(vec![
            Attribute::try_from(ExtensionReq(requested)).map_err(parse)?,
        ])
        .map_err(parse)?,
    };
    let signature = rsa_sha256_sign(&key, &info.to_der().map_err(parse)?)?;
    CertReq {
        info,
        algorithm: alg(OID_SHA256_WITH_RSA, Some(Any::null())),
        signature: BitString::from_bytes(&signature).map_err(parse)?,
    }
    .to_pem(der::pem::LineEnding::LF)
    .map_err(parse)
}

/// Check a certificate issued for the key in `bundle` (PEM text or DER bytes, one
/// certificate) and return its summary so the caller can store it in place of the
/// self-signed one. Refused when its public key is not the stored key's — a
/// certificate for some other key cannot be used with this one.
pub fn attach_issued_cert(
    issued_cert: &[u8],
    encrypted_private_bundle: &str,
    passphrase: &str,
) -> Result<CertSummary> {
    let cert = parse_certificate(issued_cert)?;
    let key = load_rsa(encrypted_private_bundle, Some(passphrase))?;
    ensure_key_matches(&key, &cert)?;
    summarize(&cert)
}

/// Write `cert_pem` and the key in `bundle` as a PKCS#12 (`.p12`) file protected by
/// `passphrase` — the reverse of [`import_pkcs12`], and the form other mail
/// programs import. Refused when the certificate is not for the key.
///
/// Layout: an unencrypted certificate bag and a PKCS#8-shrouded key bag (PBES2:
/// PBKDF2-HMAC-SHA256 + AES-256-CBC), each in its own `data` content. **No
/// `macData` is written**: the integrity MAC needs HMAC, which this crate cannot
/// reach without a new dependency edge. The private key is still encrypted, but a
/// program that insists on the MAC will refuse the file.
pub fn export_pkcs12(
    cert_pem: &str,
    encrypted_private_bundle: &str,
    passphrase: &str,
) -> Result<Vec<u8>> {
    use rsa::pkcs8::EncodePrivateKey;

    if passphrase.is_empty() {
        return Err(CryptoError::Input("a passphrase is required".into()));
    }
    let cert = Certificate::from_pem(cert_pem).map_err(parse)?;
    let key = load_rsa(encrypted_private_bundle, Some(passphrase))?;
    ensure_key_matches(&key, &cert)?;
    let cert_der = cert.to_der().map_err(parse)?;

    // `localKeyId` ties the key bag to its certificate bag for importers that
    // pair them by attribute (the SHA-1 of the certificate, as OpenSSL writes it).
    let local_key_id = attribute(
        OID_LOCAL_KEY_ID,
        Any::new(Tag::OctetString, sha1::Sha1::digest(&cert_der).as_slice()).map_err(parse)?,
    )?;
    let bag_attributes = || -> Result<Option<SetOfVec<Any>>> {
        let any = Any::encode_from(&local_key_id).map_err(parse)?;
        Ok(Some(SetOfVec::try_from(vec![any]).map_err(parse)?))
    };

    let cert_bag = SafeBag {
        bag_id: OID_CERT_BAG,
        bag_value: Any::encode_from(&CertBag {
            cert_id: OID_X509_CERTIFICATE,
            cert_value: OctetString::new(cert_der).map_err(parse)?,
        })
        .map_err(parse)?,
        bag_attributes: bag_attributes()?,
    };

    let pkcs8 = key
        .to_pkcs8_der()
        .map_err(|e| CryptoError::Pkcs12(e.to_string()))?;
    let shrouded = encrypt_pkcs8(pkcs8.as_bytes(), passphrase, PKCS12_PBKDF2_ROUNDS)?;
    let key_bag = SafeBag {
        bag_id: OID_PKCS8_SHROUDED_KEY_BAG,
        bag_value: Any::from_der(&shrouded).map_err(parse)?,
        bag_attributes: bag_attributes()?,
    };

    let data_content = |bags: Vec<SafeBag>| -> Result<ContentInfo> {
        let safe_contents = bags.to_der().map_err(parse)?;
        Ok(ContentInfo {
            content_type: ID_DATA,
            content: Any::new(Tag::OctetString, safe_contents).map_err(parse)?,
        })
    };
    let authenticated_safe = vec![data_content(vec![cert_bag])?, data_content(vec![key_bag])?]
        .to_der()
        .map_err(parse)?;

    Pfx {
        version: 3,
        auth_safe: ContentInfo {
            content_type: ID_DATA,
            content: Any::new(Tag::OctetString, authenticated_safe).map_err(parse)?,
        },
        mac_data: None,
    }
    .to_der()
    .map_err(parse)
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
        push(OID_COMMON_NAME, Tag::Utf8String, name)?;
    }
    push(OID_EMAIL_ADDRESS, Tag::Ia5String, email)?;
    Name::from_der(&rdns.to_der().map_err(parse)?).map_err(parse)
}

fn public_key_info(key: &RsaPrivateKey) -> Result<SubjectPublicKeyInfoOwned> {
    let der = key
        .to_public_key()
        .to_public_key_der()
        .map_err(|e| CryptoError::Sign(e.to_string()))?;
    SubjectPublicKeyInfoOwned::from_der(der.as_bytes()).map_err(parse)
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

/// What an RSA S/MIME key does: sign (digitalSignature) and receive an encrypted
/// content key (keyEncipherment).
fn email_key_usage() -> KeyUsage {
    KeyUsage(KeyUsages::DigitalSignature | KeyUsages::KeyEncipherment)
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

fn rsa_sha256_sign(key: &RsaPrivateKey, message: &[u8]) -> Result<Vec<u8>> {
    let signature = pkcs1v15::SigningKey::<Sha256>::new(key.clone())
        .try_sign(message)
        .map_err(|e| CryptoError::Sign(e.to_string()))?;
    Ok(signature.to_bytes().as_ref().to_vec())
}

/// The certificate's public key is this private key's.
fn ensure_key_matches(key: &RsaPrivateKey, cert: &Certificate) -> Result<()> {
    let spki_der = cert
        .tbs_certificate()
        .subject_public_key_info()
        .to_der()
        .map_err(parse)?;
    match RsaPublicKey::from_public_key_der(&spki_der) {
        Ok(public) if public.n() == key.n() && public.e() == key.e() => Ok(()),
        _ => Err(CryptoError::Input(
            "the certificate is not for this private key".into(),
        )),
    }
}

// ── private-key bundle helpers ────────────────────────────────────────────────

/// Wrap an RSA private key as a passphrase-encrypted PKCS#8 (PBES2: PBKDF2-SHA256 +
/// AES-256-CBC) PEM bundle. Salt/IV come from the OS RNG via explicit params, so the
/// wasm build never pulls `getrandom` 0.3+ (plan §1.13).
pub fn wrap_private_key(key: &RsaPrivateKey, passphrase: &str) -> Result<String> {
    use rsa::pkcs8::EncodePrivateKey;
    let pkcs8 = key
        .to_pkcs8_der()
        .map_err(|e| CryptoError::Sign(e.to_string()))?;
    let der = encrypt_pkcs8(pkcs8.as_bytes(), passphrase, 600_000)?;
    der::pem::encode_string("ENCRYPTED PRIVATE KEY", der::pem::LineEnding::LF, &der).map_err(parse)
}

/// A PKCS#8 `PrivateKeyInfo` as a DER `EncryptedPrivateKeyInfo` (PBES2:
/// PBKDF2-HMAC-SHA256 with `rounds` iterations + AES-256-CBC), fresh salt and IV.
fn encrypt_pkcs8(pkcs8_der: &[u8], passphrase: &str, rounds: u32) -> Result<Vec<u8>> {
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

/// Load an RSA private key from an encrypted (PBES2) or cleartext PKCS#8 PEM bundle.
fn load_rsa(bundle_pem: &str, passphrase: Option<&str>) -> Result<RsaPrivateKey> {
    if bundle_pem.contains("ENCRYPTED PRIVATE KEY") {
        let pw = passphrase.ok_or_else(|| CryptoError::Input("passphrase required".into()))?;
        RsaPrivateKey::from_pkcs8_encrypted_pem(bundle_pem, pw.as_bytes())
            .map_err(|e| CryptoError::Decrypt(format!("bundle unlock: {e}")))
    } else {
        RsaPrivateKey::from_pkcs8_pem(bundle_pem)
            .map_err(|e| CryptoError::Parse(format!("key parse: {e}")))
    }
}

fn attribute(oid: ObjectIdentifier, value: Any) -> Result<Attribute> {
    Ok(Attribute {
        oid,
        values: SetOfVec::try_from(vec![value]).map_err(parse)?,
    })
}

fn verdict(status: &str, key_id: Option<String>) -> SignatureVerdict {
    SignatureVerdict {
        kind: "smime".into(),
        status: status.into(),
        signer_key_id: key_id,
        algorithm: Some("rsa-sha256".into()),
        key_created_at: None,
        key_expires_at: None,
        chain_status: Some("unknown".into()),
        revocation_status: Some("unknown".into()),
        key_changed: false,
    }
}

// ── CMS AuthEnvelopedData ASN.1 (RFC 5083/5084) — self-defined; cms 0.2 ships no
//    AuthEnvelopedData/GCMParameters type. Reuses the enveloped_data recipient +
//    content types. Optional context-specific fields are decoded when present but
//    omitted on emit (we never write originatorInfo/authAttrs/unauthAttrs). ───────

/// `GCMParameters ::= SEQUENCE { aes-nonce OCTET STRING, aes-ICVlen INTEGER }`
/// (RFC 5084 §3.2). We always emit an explicit 16-octet ICV length.
#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
#[derive(der::Sequence)]
struct GcmParameters {
    nonce: OctetString,
    icv_len: u32,
}

/// `AuthEnvelopedData ::= SEQUENCE { version, originatorInfo [0] OPTIONAL,
/// recipientInfos, authEncryptedContentInfo, authAttrs [1] OPTIONAL, mac,
/// unauthAttrs [2] OPTIONAL }` (RFC 5083 §2.1).
#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
#[derive(der::Sequence)]
struct AuthEnvelopedData {
    version: CmsVersion,
    #[asn1(context_specific = "0", optional = "true", tag_mode = "IMPLICIT")]
    originator_info: Option<OriginatorInfo>,
    recip_infos: RecipientInfos,
    auth_encrypted_content_info: EncryptedContentInfo,
    #[asn1(context_specific = "1", optional = "true", tag_mode = "IMPLICIT")]
    auth_attrs: Option<SetOfVec<Attribute>>,
    mac: OctetString,
    #[asn1(context_specific = "2", optional = "true", tag_mode = "IMPLICIT")]
    unauth_attrs: Option<SetOfVec<Attribute>>,
}

// ── minimal PKCS#12 ASN.1 (RFC 7292) — self-defined to avoid a `pkcs12` crate
//    that pins a conflicting `cms` pre-release. Only the fields we read. ────────

const OID_KEY_BAG: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.12.10.1.1");
const OID_PKCS8_SHROUDED_KEY_BAG: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("1.2.840.113549.1.12.10.1.2");
const OID_CERT_BAG: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.12.10.1.3");
/// PKCS#9 `x509Certificate` — the certificate type inside a `CertBag`.
const OID_X509_CERTIFICATE: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("1.2.840.113549.1.9.22.1");
/// PKCS#9 `localKeyId` — the bag attribute pairing a key with its certificate.
const OID_LOCAL_KEY_ID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.113549.1.9.21");

/// `PFX ::= SEQUENCE { version INTEGER, authSafe ContentInfo, macData MacData OPTIONAL }`.
#[derive(der::Sequence)]
struct Pfx {
    #[allow(dead_code)]
    version: u8,
    auth_safe: ContentInfo,
    #[asn1(optional = "true")]
    #[allow(dead_code)]
    mac_data: Option<Any>,
}

/// `SafeBag ::= SEQUENCE { bagId OID, bagValue [0] EXPLICIT ANY, bagAttributes SET OPTIONAL }`.
#[derive(der::Sequence)]
struct SafeBag {
    bag_id: ObjectIdentifier,
    #[asn1(context_specific = "0", tag_mode = "EXPLICIT")]
    bag_value: Any,
    #[asn1(optional = "true")]
    #[allow(dead_code)]
    bag_attributes: Option<SetOfVec<Any>>,
}

/// `CertBag ::= SEQUENCE { certId OID, certValue [0] EXPLICIT OCTET STRING }`.
#[derive(der::Sequence)]
struct CertBag {
    #[allow(dead_code)]
    cert_id: ObjectIdentifier,
    #[asn1(context_specific = "0", tag_mode = "EXPLICIT")]
    cert_value: OctetString,
}

/// `AuthenticatedSafe ::= SEQUENCE OF ContentInfo`.
struct SafesSeq(Vec<ContentInfo>);
impl<'a> der::DecodeValue<'a> for SafesSeq {
    type Error = der::Error;
    fn decode_value<R: der::Reader<'a>>(reader: &mut R, header: der::Header) -> der::Result<Self> {
        Ok(Self(<Vec<ContentInfo> as der::DecodeValue>::decode_value(
            reader, header,
        )?))
    }
}
impl der::FixedTag for SafesSeq {
    const TAG: Tag = Tag::Sequence;
}

/// `SafeContents ::= SEQUENCE OF SafeBag`.
struct SafeContentsSeq(Vec<SafeBag>);
impl<'a> der::DecodeValue<'a> for SafeContentsSeq {
    type Error = der::Error;
    fn decode_value<R: der::Reader<'a>>(reader: &mut R, header: der::Header) -> der::Result<Self> {
        Ok(Self(<Vec<SafeBag> as der::DecodeValue>::decode_value(
            reader, header,
        )?))
    }
}
impl der::FixedTag for SafeContentsSeq {
    const TAG: Tag = Tag::Sequence;
}
