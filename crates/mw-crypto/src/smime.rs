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
use der::asn1::{OctetString, SetOfVec};
use der::{Any, Decode, DecodePem, Encode, Tag};
use rsa::pkcs1v15;
use rsa::pkcs8::DecodePrivateKey;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use sha2::{Digest, Sha256};
use signature::{SignatureEncoding, Signer, Verifier};
use spki::{AlgorithmIdentifierOwned, DecodePublicKey};
use x509_cert::Certificate;
use x509_cert::attr::Attribute;

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

/// Result of [`import_pkcs12`].
pub struct Pkcs12Import {
    pub cert_pem: String,
    pub fingerprint: String,
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

    let cert_pem = der::pem::encode_string("CERTIFICATE", der::pem::LineEnding::LF, &cert_der)
        .map_err(parse)?;
    let fingerprint = hex::encode(Sha256::digest(&cert_der)).to_uppercase();

    let rsa = RsaPrivateKey::from_pkcs8_der(&key_pkcs8)
        .map_err(|e| CryptoError::Pkcs12(format!("key parse: {e}")))?;
    let encrypted_private_bundle = wrap_private_key(&rsa, password)?;

    Ok(Pkcs12Import {
        cert_pem,
        fingerprint,
        encrypted_private_bundle,
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
    let der = cert.to_der().map_err(parse)?;
    let fingerprint = hex::encode(Sha256::digest(&der)).to_uppercase();
    let addresses = cert_email_addresses(cert);
    let cert_pem =
        der::pem::encode_string("CERTIFICATE", der::pem::LineEnding::LF, &der).map_err(parse)?;
    Ok(CryptoKey {
        id: format!("smime:{fingerprint}"),
        kind: "smime".into(),
        is_own: false,
        addresses,
        fingerprint: fingerprint.clone(),
        key_id: fingerprint[..16.min(fingerprint.len())].to_string(),
        algorithm: "rsa".into(),
        created_at: chrono::Utc::now().to_rfc3339(),
        expires_at: None,
        public_key_armored: None,
        cert_pem: Some(cert_pem),
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

/// Extract email addresses from a certificate's subject DN (`emailAddress=` / `E=`).
fn cert_email_addresses(cert: &Certificate) -> Vec<String> {
    let mut out = Vec::new();
    let subject = cert.tbs_certificate().subject().to_string();
    for part in subject.split([',', '+']) {
        let part = part.trim();
        if let Some(v) = part
            .strip_prefix("emailAddress=")
            .or_else(|| part.strip_prefix("E="))
            // RFC 4514 has no short name for pkcs-9 emailAddress, so it renders as the OID.
            .or_else(|| part.strip_prefix("1.2.840.113549.1.9.1="))
        {
            out.push(v.trim().to_string());
        }
    }
    out
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
    let mut salt = [0u8; 16];
    let mut iv = [0u8; 16];
    rng::fill_random(&mut salt);
    rng::fill_random(&mut iv);
    let params = pkcs5::pbes2::Parameters::generate_pbkdf2_sha256_aes256cbc(600_000, &salt, iv)
        .map_err(|e| CryptoError::Sign(format!("pbes2 params: {e:?}")))?;
    let scheme = pkcs5::EncryptionScheme::Pbes2(params);
    let ciphertext = scheme
        .encrypt(passphrase, pkcs8.as_bytes())
        .map_err(|e| CryptoError::Sign(format!("pbes2 encrypt: {e:?}")))?;
    let epki: pkcs8::EncryptedPrivateKeyInfoOwned = pkcs8::EncryptedPrivateKeyInfo {
        encryption_algorithm: scheme,
        encrypted_data: OctetString::new(ciphertext).map_err(parse)?,
    };
    let der = epki.to_der().map_err(parse)?;
    der::pem::encode_string("ENCRYPTED PRIVATE KEY", der::pem::LineEnding::LF, &der).map_err(parse)
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
