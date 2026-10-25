//! PKCS#12 import and export for S/MIME keys (RFC 7292), moved out of `smime` in
//! t32-e41 so it can be compiled only where private keys are handled: the browser
//! guest and this crate's tests (`client-private-ops`, DA48). Keys of every type
//! [`crate::smime_keys`] supports go in and out; the password-integrity MAC is
//! written on export and checked on import.

use cms::content_info::ContentInfo;
use const_oid::ObjectIdentifier;
use const_oid::db::rfc5911::{ID_DATA, ID_ENCRYPTED_DATA};
use der::asn1::{OctetString, SetOfVec};
use der::{Any, Decode, DecodePem, Encode, Tag};
use sha2::{Digest, Sha256};
use spki::AlgorithmIdentifierOwned;
use x509_cert::Certificate;
use x509_cert::attr::Attribute;

use crate::error::{CryptoError, Result};
use crate::rng;
use crate::smime::{CertSummary, describe_certificate};
use crate::smime_keys::{self, PrivateKey};

const OID_SHA_256: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.1");

/// PBKDF2 rounds for the key inside an exported PKCS#12. Lower than the vault's
/// own wrap: Windows refuses a PKCS#12 with an iteration count past 600 000, and
/// other importers apply similar limits.
const PKCS12_PBKDF2_ROUNDS: u32 = 210_000;
/// Rounds of the PKCS#12 key derivation for the integrity MAC of an export. With
/// the key's rounds this stays under the 600 000 that Windows accepts even if an
/// importer adds the two counts together.
const PKCS12_MAC_ROUNDS: u32 = 210_000;
/// The most MAC rounds [`import_pkcs12`] will run for a file: a count beyond this
/// is a way to make the importer spin, not a security setting.
const PKCS12_MAC_ROUNDS_MAX: u32 = 10_000_000;

fn parse(e: impl std::fmt::Display) -> CryptoError {
    CryptoError::Parse(e.to_string())
}

fn alg(oid: ObjectIdentifier, parameters: Option<Any>) -> AlgorithmIdentifierOwned {
    AlgorithmIdentifierOwned { oid, parameters }
}

fn attribute(oid: ObjectIdentifier, value: Any) -> Result<Attribute> {
    Ok(Attribute {
        oid,
        values: SetOfVec::try_from(vec![value]).map_err(parse)?,
    })
}

/// Result of [`import_pkcs12`].
pub struct Pkcs12Import {
    pub cert_pem: String,
    pub fingerprint: String,
    pub encrypted_private_bundle: String,
    /// The addresses the certificate names (subject alternative name, then subject).
    pub addresses: Vec<String>,
    /// The key's algorithm, read from the certificate.
    pub algorithm: String,
    /// The certificate's `notAfter`, RFC 3339.
    pub not_after: String,
}

fn summarize(cert: &Certificate) -> Result<CertSummary> {
    describe_certificate(&cert.to_der().map_err(parse)?)
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
    // A file that carries an integrity MAC must pass it before anything in it is
    // used. A file without one is still read: the key bag is encrypted either way.
    if let Some(mac) = &pfx.mac_data {
        verify_pkcs12_mac(mac, auth_safe_bytes.as_bytes(), password)?;
    }
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

    let key = PrivateKey::from_pkcs8_der(&key_pkcs8)
        .map_err(|e| CryptoError::Pkcs12(format!("key parse: {e}")))?;
    if !key.matches(cert.tbs_certificate().subject_public_key_info()) {
        return Err(CryptoError::Pkcs12(
            "the key in the file is not the certificate's key".into(),
        ));
    }
    let encrypted_private_bundle = smime_keys::wrap(&key, password)?;

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

/// Write `cert_pem` and the key in `bundle` as a PKCS#12 (`.p12`) file protected by
/// `passphrase` — the reverse of [`import_pkcs12`], and the form other mail
/// programs import. Refused when the certificate is not for the key.
///
/// Layout: an unencrypted certificate bag and a PKCS#8-shrouded key bag (PBES2:
/// PBKDF2-HMAC-SHA256 + AES-256-CBC), each in its own `data` content, and a
/// `macData` over both (RFC 7292 §4: HMAC-SHA-256, key from the appendix B.2
/// derivation with [`PKCS12_MAC_ROUNDS`] rounds) — the password-integrity mode
/// OpenSSL 3 writes by default.
pub fn export_pkcs12(
    cert_pem: &str,
    encrypted_private_bundle: &str,
    passphrase: &str,
) -> Result<Vec<u8>> {
    let cert = Certificate::from_pem(cert_pem).map_err(parse)?;
    let key = smime_keys::unwrap(encrypted_private_bundle, passphrase)?;
    if !key.matches(cert.tbs_certificate().subject_public_key_info()) {
        return Err(CryptoError::Input(
            "the certificate is not for this private key".into(),
        ));
    }
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

    let pkcs8 = key.to_pkcs8_der()?;
    let shrouded = crate::smime::encrypt_pkcs8(&pkcs8, passphrase, PKCS12_PBKDF2_ROUNDS)?;
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

    let mut mac_salt = [0u8; 16];
    rng::fill_random(&mut mac_salt);
    let mac = pkcs12_mac::<Sha256>(
        passphrase,
        &mac_salt,
        PKCS12_MAC_ROUNDS,
        &authenticated_safe,
    )?;

    Pfx {
        version: 3,
        auth_safe: ContentInfo {
            content_type: ID_DATA,
            content: Any::new(Tag::OctetString, authenticated_safe).map_err(parse)?,
        },
        mac_data: Some(MacData {
            mac: DigestInfo {
                algorithm: alg(OID_SHA_256, Some(Any::null())),
                digest: OctetString::new(mac).map_err(parse)?,
            },
            mac_salt: OctetString::new(&mac_salt[..]).map_err(parse)?,
            iterations: PKCS12_MAC_ROUNDS,
        }),
    }
    .to_der()
    .map_err(parse)
}

// ── PKCS#12 password integrity (RFC 7292 §4 + appendix B) ─────────────────────

/// A hash the PKCS#12 MAC can be built on. `BLOCK` is the hash's input block
/// size in octets — `v` in RFC 7292 appendix B.2.
trait Pkcs12Hash: Digest {
    const BLOCK: usize;
    fn hmac(key: &[u8], message: &[u8]) -> Result<Vec<u8>>;
    fn hmac_matches(key: &[u8], message: &[u8], expected: &[u8]) -> Result<bool>;
}

macro_rules! pkcs12_hash {
    ($hash:ty, $block:expr) => {
        impl Pkcs12Hash for $hash {
            const BLOCK: usize = $block;
            fn hmac(key: &[u8], message: &[u8]) -> Result<Vec<u8>> {
                use hmac::{KeyInit, Mac};
                let mut mac = hmac::Hmac::<$hash>::new_from_slice(key)
                    .map_err(|e| CryptoError::Pkcs12(e.to_string()))?;
                mac.update(message);
                Ok(mac.finalize().into_bytes().to_vec())
            }
            fn hmac_matches(key: &[u8], message: &[u8], expected: &[u8]) -> Result<bool> {
                use hmac::{KeyInit, Mac};
                let mut mac = hmac::Hmac::<$hash>::new_from_slice(key)
                    .map_err(|e| CryptoError::Pkcs12(e.to_string()))?;
                mac.update(message);
                // Constant-time comparison.
                Ok(mac.verify_slice(expected).is_ok())
            }
        }
    };
}
pkcs12_hash!(sha1::Sha1, 64);
pkcs12_hash!(Sha256, 64);
pkcs12_hash!(sha2::Sha384, 128);
pkcs12_hash!(sha2::Sha512, 128);

/// A PKCS#12 password as the key derivation takes it: a BMPString — UTF-16,
/// big-endian — with a terminating NUL (RFC 7292 appendix B.1). An empty
/// password is the terminator alone.
fn pkcs12_password(password: &str) -> Vec<u8> {
    password
        .encode_utf16()
        .chain(core::iter::once(0))
        .flat_map(u16::to_be_bytes)
        .collect()
}

/// The MAC key of RFC 7292 appendix B.2 (`ID` = 3), one hash output long.
///
/// This is the appendix's derivation restricted to the case this module needs:
/// the wanted key is exactly one hash output, so step 6's single block `A_1` is
/// the whole result and the carry step that prepares further blocks never runs.
/// It is not a primitive of ours — it only arranges calls to the hash — and it
/// is pinned from outside: `tests/smime_generate.rs` has `openssl` verify MACs
/// made with it and has it verify MACs `openssl` made.
fn pkcs12_mac_key<H: Pkcs12Hash>(password: &[u8], salt: &[u8], rounds: u32) -> Vec<u8> {
    // Steps 2-3: salt and password, each repeated to a whole number of blocks.
    let fill = |x: &[u8]| -> Vec<u8> {
        let len = H::BLOCK * x.len().div_ceil(H::BLOCK);
        x.iter().copied().cycle().take(len).collect()
    };
    let mut h = H::new();
    h.update(vec![3u8; H::BLOCK]); // step 1: the diversifier D
    h.update(fill(salt));
    h.update(fill(password));
    let mut a = h.finalize().to_vec();
    for _ in 1..rounds {
        a = H::digest(&a).to_vec();
    }
    a
}

/// `HMAC(key = B.2(password, salt, rounds), message)`.
fn pkcs12_mac<H: Pkcs12Hash>(
    password: &str,
    salt: &[u8],
    rounds: u32,
    message: &[u8],
) -> Result<Vec<u8>> {
    H::hmac(
        &pkcs12_mac_key::<H>(&pkcs12_password(password), salt, rounds),
        message,
    )
}

/// Check a file's `macData` against its authenticated safe.
fn verify_pkcs12_mac(mac: &MacData, authenticated_safe: &[u8], password: &str) -> Result<()> {
    fn check<H: Pkcs12Hash>(mac: &MacData, message: &[u8], password: &str) -> Result<bool> {
        let derive = |pw: &[u8]| pkcs12_mac_key::<H>(pw, mac.mac_salt.as_bytes(), mac.iterations);
        if H::hmac_matches(
            &derive(&pkcs12_password(password)),
            message,
            mac.mac.digest.as_bytes(),
        )? {
            return Ok(true);
        }
        // "No password" is written two ways in the wild: the terminator alone
        // (above) or no octets at all. Accept either for an empty password.
        Ok(password.is_empty()
            && H::hmac_matches(&derive(&[]), message, mac.mac.digest.as_bytes())?)
    }

    if mac.iterations == 0 || mac.iterations > PKCS12_MAC_ROUNDS_MAX {
        return Err(CryptoError::Pkcs12(format!(
            "integrity MAC iteration count {} is out of range",
            mac.iterations
        )));
    }
    let oid = mac.mac.algorithm.oid;
    let ok = if oid == OID_SHA_256 {
        check::<Sha256>(mac, authenticated_safe, password)?
    } else if oid == OID_SHA_1 {
        check::<sha1::Sha1>(mac, authenticated_safe, password)?
    } else if oid == OID_SHA_384 {
        check::<sha2::Sha384>(mac, authenticated_safe, password)?
    } else if oid == OID_SHA_512 {
        check::<sha2::Sha512>(mac, authenticated_safe, password)?
    } else {
        // PBMAC1 (RFC 9579) and anything newer: not checked here, so not accepted.
        return Err(CryptoError::Pkcs12(format!(
            "integrity MAC algorithm {oid} is not supported"
        )));
    };
    if ok {
        Ok(())
    } else {
        Err(CryptoError::Pkcs12(
            "integrity check failed: wrong password, or the file was altered".into(),
        ))
    }
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

/// `id-sha1`, `id-sha384`, `id-sha512` — the other digests a PKCS#12 MAC may name.
const OID_SHA_1: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.3.14.3.2.26");
const OID_SHA_384: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.2");
const OID_SHA_512: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.3");

/// `PFX ::= SEQUENCE { version INTEGER, authSafe ContentInfo, macData MacData OPTIONAL }`.
#[derive(der::Sequence)]
struct Pfx {
    version: u8,
    auth_safe: ContentInfo,
    #[asn1(optional = "true")]
    mac_data: Option<MacData>,
}

/// `MacData ::= SEQUENCE { mac DigestInfo, macSalt OCTET STRING, iterations INTEGER DEFAULT 1 }`.
#[derive(der::Sequence)]
struct MacData {
    mac: DigestInfo,
    mac_salt: OctetString,
    #[asn1(default = "one_iteration")]
    iterations: u32,
}

fn one_iteration() -> u32 {
    1
}

/// `DigestInfo ::= SEQUENCE { digestAlgorithm AlgorithmIdentifier, digest OCTET STRING }`.
#[derive(der::Sequence)]
struct DigestInfo {
    algorithm: AlgorithmIdentifierOwned,
    digest: OctetString,
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
