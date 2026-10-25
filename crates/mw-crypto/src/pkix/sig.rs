//! Signature checks over the RustCrypto primitives already in the crate: RSA
//! PKCS#1 v1.5 and RSASSA-PSS (`rsa`), ECDSA P-256 and P-384 (`p256`, `p384`) and
//! Ed25519 (`ed25519-dalek`, RFC 8419). Used for certificate and CRL signatures,
//! OCSP responses, and CMS signer infos.

use der::{Decode, Encode};
use sha2::{Digest, Sha256, Sha384, Sha512};
use signature::Verifier;
use spki::{AlgorithmIdentifierOwned, ObjectIdentifier, SubjectPublicKeyInfoOwned};

use super::oid;

/// Why a signature did not verify.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SigError {
    /// The algorithm (or its hash) is not one this module checks.
    Unsupported(String),
    /// A hash this module refuses for signatures (SHA-1 on messages, MD5).
    Weak,
    /// The key or signature could not be decoded.
    Malformed,
    /// The signature does not match.
    Mismatch,
}

/// A digest named by OID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hash {
    Sha1,
    Sha256,
    Sha384,
    Sha512,
}

impl Hash {
    pub fn from_oid(o: &ObjectIdentifier) -> Option<Hash> {
        match *o {
            oid::SHA1 => Some(Hash::Sha1),
            oid::SHA256 => Some(Hash::Sha256),
            oid::SHA384 => Some(Hash::Sha384),
            oid::SHA512 => Some(Hash::Sha512),
            _ => None,
        }
    }

    pub fn oid(self) -> ObjectIdentifier {
        match self {
            Hash::Sha1 => oid::SHA1,
            Hash::Sha256 => oid::SHA256,
            Hash::Sha384 => oid::SHA384,
            Hash::Sha512 => oid::SHA512,
        }
    }

    pub fn digest(self, data: &[u8]) -> Vec<u8> {
        match self {
            Hash::Sha1 => sha1::Sha1::digest(data).to_vec(),
            Hash::Sha256 => Sha256::digest(data).to_vec(),
            Hash::Sha384 => Sha384::digest(data).to_vec(),
            Hash::Sha512 => Sha512::digest(data).to_vec(),
        }
    }

    /// `sha256`, `sha384`, ...
    pub fn name(self) -> &'static str {
        match self {
            Hash::Sha1 => "sha1",
            Hash::Sha256 => "sha256",
            Hash::Sha384 => "sha384",
            Hash::Sha512 => "sha512",
        }
    }
}

/// Where a signature is checked: a certificate chain may still carry SHA-1 on an
/// old root's self-signature (never checked) but nothing else; a message
/// signature never uses SHA-1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Use {
    Certificate,
    Message,
}

/// Verify `signature` over `message` with the key in `spki`.
///
/// `alg` is the signature algorithm identifier. A CMS signer info may name only
/// the key algorithm (`rsaEncryption`, `id-ecPublicKey`) and the digest
/// separately; pass that digest as `digest`.
pub fn verify(
    spki: &SubjectPublicKeyInfoOwned,
    alg: &AlgorithmIdentifierOwned,
    digest: Option<Hash>,
    message: &[u8],
    signature: &[u8],
    usage: Use,
) -> Result<(), SigError> {
    let weak = |h: Hash| h == Hash::Sha1 && usage == Use::Message;
    let spki_der = spki.to_der().map_err(|_| SigError::Malformed)?;
    match alg.oid {
        oid::RSA_ENCRYPTION
        | oid::SHA1_WITH_RSA
        | oid::SHA256_WITH_RSA
        | oid::SHA384_WITH_RSA
        | oid::SHA512_WITH_RSA => {
            let h = match alg.oid {
                oid::SHA1_WITH_RSA => Hash::Sha1,
                oid::SHA256_WITH_RSA => Hash::Sha256,
                oid::SHA384_WITH_RSA => Hash::Sha384,
                oid::SHA512_WITH_RSA => Hash::Sha512,
                _ => digest.ok_or_else(|| SigError::Unsupported("rsa without a digest".into()))?,
            };
            if weak(h) {
                return Err(SigError::Weak);
            }
            rsa_pkcs1(&spki_der, h, message, signature)
        }
        oid::RSASSA_PSS => {
            let p = pss_params(alg)?;
            if weak(p.hash) {
                return Err(SigError::Weak);
            }
            rsa_pss(&spki_der, &p, message, signature)
        }
        oid::ECDSA_SHA256 | oid::ECDSA_SHA384 | oid::ECDSA_SHA512 | oid::EC_PUBLIC_KEY => {
            let h = match alg.oid {
                oid::ECDSA_SHA256 => Hash::Sha256,
                oid::ECDSA_SHA384 => Hash::Sha384,
                oid::ECDSA_SHA512 => Hash::Sha512,
                _ => {
                    digest.ok_or_else(|| SigError::Unsupported("ecdsa without a digest".into()))?
                }
            };
            if weak(h) {
                return Err(SigError::Weak);
            }
            ecdsa(&spki_der, h, message, signature)
        }
        oid::ED25519 => {
            // RFC 8410: Ed25519 key and signature AlgorithmIdentifiers have
            // absent parameters. Never reinterpret X25519 bytes as a signing key.
            if spki.algorithm.oid != oid::ED25519
                || spki.algorithm.parameters.is_some()
                || alg.parameters.is_some()
            {
                return Err(SigError::Malformed);
            }
            ed25519(spki, message, signature)
        }
        oid::MD5_WITH_RSA => Err(SigError::Weak),
        other => Err(SigError::Unsupported(other.to_string())),
    }
}

fn rsa_pkcs1(spki_der: &[u8], h: Hash, message: &[u8], signature: &[u8]) -> Result<(), SigError> {
    use rsa::RsaPublicKey;
    use rsa::pkcs1v15::{Signature, VerifyingKey};
    use spki::DecodePublicKey;
    let key = RsaPublicKey::from_public_key_der(spki_der).map_err(|_| SigError::Malformed)?;
    let sig = Signature::try_from(signature).map_err(|_| SigError::Malformed)?;
    let ok = match h {
        Hash::Sha1 => VerifyingKey::<sha1::Sha1>::new(key).verify(message, &sig),
        Hash::Sha256 => VerifyingKey::<Sha256>::new(key).verify(message, &sig),
        Hash::Sha384 => VerifyingKey::<Sha384>::new(key).verify(message, &sig),
        Hash::Sha512 => VerifyingKey::<Sha512>::new(key).verify(message, &sig),
    };
    ok.map_err(|_| SigError::Mismatch)
}

/// RSASSA-PSS parameters (RFC 4055 §3.1), as far as this module checks them.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PssParams {
    pub hash: Hash,
    pub salt_len: usize,
}

/// `RSASSA-PSS-params ::= SEQUENCE { hashAlgorithm [0] DEFAULT sha1,
/// maskGenAlgorithm [1] DEFAULT mgf1SHA1, saltLength [2] DEFAULT 20,
/// trailerField [3] DEFAULT 1 }`.
#[derive(der::Sequence)]
struct RsaPssParamsAsn1 {
    #[asn1(context_specific = "0", tag_mode = "EXPLICIT", optional = "true")]
    hash: Option<AlgorithmIdentifierOwned>,
    #[asn1(context_specific = "1", tag_mode = "EXPLICIT", optional = "true")]
    mgf: Option<AlgorithmIdentifierOwned>,
    #[asn1(context_specific = "2", tag_mode = "EXPLICIT", optional = "true")]
    salt: Option<u32>,
    #[asn1(context_specific = "3", tag_mode = "EXPLICIT", optional = "true")]
    trailer: Option<u32>,
}

pub(crate) fn pss_params(alg: &AlgorithmIdentifierOwned) -> Result<PssParams, SigError> {
    let der = alg
        .parameters
        .as_ref()
        .ok_or(SigError::Malformed)?
        .to_der()
        .map_err(|_| SigError::Malformed)?;
    let p = RsaPssParamsAsn1::from_der(&der).map_err(|_| SigError::Malformed)?;
    let hash = match &p.hash {
        Some(a) => {
            Hash::from_oid(&a.oid).ok_or_else(|| SigError::Unsupported(a.oid.to_string()))?
        }
        None => Hash::Sha1,
    };
    // MGF1 with the same hash is the only combination in use; refuse others.
    if let Some(m) = &p.mgf {
        let mgf_hash = m
            .parameters
            .as_ref()
            .and_then(|a| a.decode_as::<AlgorithmIdentifierOwned>().ok())
            .and_then(|a| Hash::from_oid(&a.oid));
        if m.oid != oid::MGF1 || mgf_hash != Some(hash) {
            return Err(SigError::Unsupported("PSS mask generation".into()));
        }
    } else if hash != Hash::Sha1 {
        return Err(SigError::Unsupported("PSS mask generation".into()));
    }
    if p.trailer.is_some_and(|t| t != 1) {
        return Err(SigError::Unsupported("PSS trailer field".into()));
    }
    Ok(PssParams {
        hash,
        salt_len: p.salt.unwrap_or(20) as usize,
    })
}

fn rsa_pss(
    spki_der: &[u8],
    p: &PssParams,
    message: &[u8],
    signature: &[u8],
) -> Result<(), SigError> {
    use rsa::RsaPublicKey;
    use rsa::pss::{Signature, VerifyingKey};
    use spki::DecodePublicKey;
    let key = RsaPublicKey::from_public_key_der(spki_der).map_err(|_| SigError::Malformed)?;
    let sig = Signature::try_from(signature).map_err(|_| SigError::Malformed)?;
    let ok = match p.hash {
        Hash::Sha1 => {
            VerifyingKey::<sha1::Sha1>::new_with_salt_len(key, p.salt_len).verify(message, &sig)
        }
        Hash::Sha256 => {
            VerifyingKey::<Sha256>::new_with_salt_len(key, p.salt_len).verify(message, &sig)
        }
        Hash::Sha384 => {
            VerifyingKey::<Sha384>::new_with_salt_len(key, p.salt_len).verify(message, &sig)
        }
        Hash::Sha512 => {
            VerifyingKey::<Sha512>::new_with_salt_len(key, p.salt_len).verify(message, &sig)
        }
    };
    ok.map_err(|_| SigError::Mismatch)
}

/// ECDSA over P-256 or P-384 with an explicit hash (the curve comes from the key).
/// The signature is the DER `ECDSA-Sig-Value` that X.509 and CMS carry.
fn ecdsa(spki_der: &[u8], h: Hash, message: &[u8], signature: &[u8]) -> Result<(), SigError> {
    use signature::hazmat::PrehashVerifier;
    use spki::DecodePublicKey;
    let prehash = h.digest(message);
    if let Ok(vk) = p256::ecdsa::VerifyingKey::from_public_key_der(spki_der) {
        let sig = p256::ecdsa::Signature::from_der(signature).map_err(|_| SigError::Malformed)?;
        return vk
            .verify_prehash(&prehash, &sig)
            .map_err(|_| SigError::Mismatch);
    }
    if let Ok(vk) = p384::ecdsa::VerifyingKey::from_public_key_der(spki_der) {
        let sig = p384::ecdsa::Signature::from_der(signature).map_err(|_| SigError::Malformed)?;
        return vk
            .verify_prehash(&prehash, &sig)
            .map_err(|_| SigError::Mismatch);
    }
    Err(SigError::Unsupported("elliptic curve".into()))
}

fn ed25519(
    spki: &SubjectPublicKeyInfoOwned,
    message: &[u8],
    signature: &[u8],
) -> Result<(), SigError> {
    let key: [u8; 32] = spki
        .subject_public_key
        .as_bytes()
        .and_then(|b| b.try_into().ok())
        .ok_or(SigError::Malformed)?;
    let vk = ed25519_dalek::VerifyingKey::from_bytes(&key).map_err(|_| SigError::Malformed)?;
    let sig: [u8; 64] = signature.try_into().map_err(|_| SigError::Malformed)?;
    vk.verify_strict(message, &ed25519_dalek::Signature::from_bytes(&sig))
        .map_err(|_| SigError::Mismatch)
}

/// The algorithm name of a public key, for display and policy: `rsa-<bits>`,
/// `ecdsa-p256`, `ecdsa-p384`, `ed25519`, `x25519`, or the OID.
pub fn key_algorithm(spki: &SubjectPublicKeyInfoOwned) -> String {
    use rsa::traits::PublicKeyParts;
    use spki::DecodePublicKey;
    match spki.algorithm.oid {
        oid::RSA_ENCRYPTION => spki
            .to_der()
            .ok()
            .and_then(|d| rsa::RsaPublicKey::from_public_key_der(&d).ok())
            .map(|k| format!("rsa-{}", k.n().bits()))
            .unwrap_or_else(|| "rsa".into()),
        oid::EC_PUBLIC_KEY => match curve_of(spki) {
            Some(oid::SECP256R1) => "ecdsa-p256".into(),
            Some(oid::SECP384R1) => "ecdsa-p384".into(),
            Some(o) => format!("ec-{o}"),
            None => "ec".into(),
        },
        oid::ED25519 => "ed25519".into(),
        oid::X25519 => "x25519".into(),
        other => other.to_string(),
    }
}

/// The named curve of an `id-ecPublicKey` key.
pub fn curve_of(spki: &SubjectPublicKeyInfoOwned) -> Option<ObjectIdentifier> {
    spki.algorithm
        .parameters
        .as_ref()
        .and_then(|p| p.decode_as::<ObjectIdentifier>().ok())
}

/// RSA modulus size in bits, `None` for other keys.
pub fn rsa_bits(spki: &SubjectPublicKeyInfoOwned) -> Option<usize> {
    use rsa::traits::PublicKeyParts;
    use spki::DecodePublicKey;
    if spki.algorithm.oid != oid::RSA_ENCRYPTION {
        return None;
    }
    let der = spki.to_der().ok()?;
    rsa::RsaPublicKey::from_public_key_der(&der)
        .ok()
        .map(|k| k.n().bits() as usize)
}
