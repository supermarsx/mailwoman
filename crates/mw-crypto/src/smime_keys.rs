//! S/MIME private keys of every supported type (t32-e41): RSA (2048–4096),
//! ECDSA/ECDH P-256 and P-384, and Ed25519. A key travels as the frozen
//! `encryptedPrivateBundle`: a PKCS#8 `EncryptedPrivateKeyInfo` PEM (PBES2,
//! PBKDF2-HMAC-SHA256 + AES-256-CBC), never in the clear.

use der::asn1::{BitString, OctetString};
use der::{Decode, Encode};
use spki::{AlgorithmIdentifierOwned, SubjectPublicKeyInfoOwned};

use crate::error::{CryptoError, Result};
use crate::pkix::oid;
use crate::pkix::sig::Hash;
use crate::rng;

/// A decrypted private key. Held only for the length of one operation.
pub enum PrivateKey {
    Rsa(Box<rsa::RsaPrivateKey>),
    P256(p256::SecretKey),
    P384(p384::SecretKey),
    Ed25519(Box<ed25519_dalek::SigningKey>),
    /// A key-agreement-only key (RFC 8410), as imported from a PKCS#12 file.
    X25519(Box<x25519_dalek::StaticSecret>),
}

/// Which kind of key to generate.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum KeyType {
    Rsa2048,
    #[default]
    Rsa3072,
    Rsa4096,
    EcdsaP256,
    EcdsaP384,
    Ed25519,
}

fn err(e: impl std::fmt::Display) -> CryptoError {
    CryptoError::KeyGen(e.to_string())
}

impl PrivateKey {
    /// A fresh key of `kind`.
    pub fn generate(kind: KeyType) -> Result<PrivateKey> {
        Ok(match kind {
            KeyType::Rsa2048 | KeyType::Rsa3072 | KeyType::Rsa4096 => {
                let bits = match kind {
                    KeyType::Rsa2048 => 2048,
                    KeyType::Rsa4096 => 4096,
                    _ => 3072,
                };
                PrivateKey::Rsa(Box::new(
                    rsa::RsaPrivateKey::new(&mut rng::rc10(), bits).map_err(err)?,
                ))
            }
            // A random scalar: a value outside the group order (probability about
            // 2^-128) is drawn again.
            KeyType::EcdsaP256 => {
                PrivateKey::P256(random_scalar(32, |b| p256::SecretKey::from_slice(b).ok())?)
            }
            KeyType::EcdsaP384 => {
                PrivateKey::P384(random_scalar(48, |b| p384::SecretKey::from_slice(b).ok())?)
            }
            KeyType::Ed25519 => {
                let mut seed = [0u8; 32];
                rng::fill_random(&mut seed);
                PrivateKey::Ed25519(Box::new(ed25519_dalek::SigningKey::from_bytes(&seed)))
            }
        })
    }

    /// The public key as a SubjectPublicKeyInfo.
    pub fn public_spki(&self) -> Result<SubjectPublicKeyInfoOwned> {
        use spki::EncodePublicKey;
        let der = match self {
            PrivateKey::Rsa(k) => k
                .to_public_key()
                .to_public_key_der()
                .map_err(err)?
                .into_vec(),
            PrivateKey::P256(k) => k.public_key().to_public_key_der().map_err(err)?.into_vec(),
            PrivateKey::P384(k) => k.public_key().to_public_key_der().map_err(err)?.into_vec(),
            PrivateKey::Ed25519(k) => {
                return Ok(SubjectPublicKeyInfoOwned {
                    algorithm: AlgorithmIdentifierOwned {
                        oid: oid::ED25519,
                        parameters: None,
                    },
                    subject_public_key: BitString::from_bytes(k.verifying_key().as_bytes())
                        .map_err(err)?,
                });
            }
            PrivateKey::X25519(k) => {
                return Ok(SubjectPublicKeyInfoOwned {
                    algorithm: AlgorithmIdentifierOwned {
                        oid: oid::X25519,
                        parameters: None,
                    },
                    subject_public_key: BitString::from_bytes(
                        x25519_dalek::PublicKey::from(&**k).as_bytes(),
                    )
                    .map_err(err)?,
                });
            }
        };
        SubjectPublicKeyInfoOwned::from_der(&der).map_err(err)
    }

    /// The digest this key signs with by default: SHA-256 for RSA and P-256,
    /// SHA-384 for P-384, SHA-512 for Ed25519 (RFC 8419 §3).
    pub fn default_hash(&self) -> Hash {
        match self {
            PrivateKey::P384(_) => Hash::Sha384,
            PrivateKey::Ed25519(_) => Hash::Sha512,
            _ => Hash::Sha256,
        }
    }

    /// The signature algorithm identifier this key produces with `hash`
    /// (`pss` selects RSASSA-PSS for an RSA key).
    pub fn signature_algorithm(&self, hash: Hash, pss: bool) -> Result<AlgorithmIdentifierOwned> {
        let (o, params) = match self {
            PrivateKey::Rsa(_) if pss => (oid::RSASSA_PSS, Some(pss_params_any(hash)?)),
            PrivateKey::Rsa(_) => (
                match hash {
                    Hash::Sha384 => oid::SHA384_WITH_RSA,
                    Hash::Sha512 => oid::SHA512_WITH_RSA,
                    _ => oid::SHA256_WITH_RSA,
                },
                Some(der::Any::null()),
            ),
            PrivateKey::P256(_) | PrivateKey::P384(_) => (
                match hash {
                    Hash::Sha384 => oid::ECDSA_SHA384,
                    Hash::Sha512 => oid::ECDSA_SHA512,
                    _ => oid::ECDSA_SHA256,
                },
                None,
            ),
            PrivateKey::Ed25519(_) => (oid::ED25519, None),
            PrivateKey::X25519(_) => {
                return Err(CryptoError::Sign("an X25519 key cannot sign".into()));
            }
        };
        Ok(AlgorithmIdentifierOwned {
            oid: o,
            parameters: params,
        })
    }

    /// Sign `message` (hashing it with `hash` where the algorithm hashes).
    pub fn sign(&self, hash: Hash, pss: bool, message: &[u8]) -> Result<Vec<u8>> {
        use signature::SignatureEncoding;
        use signature::hazmat::PrehashSigner;
        let sign_err = |e: signature::Error| CryptoError::Sign(e.to_string());
        Ok(match self {
            PrivateKey::Rsa(k) if pss => {
                use signature::RandomizedSigner;
                let k = (**k).clone();
                let mut r = rng::rc10();
                match hash {
                    Hash::Sha384 => rsa::pss::SigningKey::<sha2::Sha384>::new(k)
                        .try_sign_with_rng(&mut r, message)
                        .map_err(sign_err)?
                        .to_vec(),
                    Hash::Sha512 => rsa::pss::SigningKey::<sha2::Sha512>::new(k)
                        .try_sign_with_rng(&mut r, message)
                        .map_err(sign_err)?
                        .to_vec(),
                    _ => rsa::pss::SigningKey::<sha2::Sha256>::new(k)
                        .try_sign_with_rng(&mut r, message)
                        .map_err(sign_err)?
                        .to_vec(),
                }
            }
            PrivateKey::Rsa(k) => {
                use signature::Signer;
                let k = (**k).clone();
                match hash {
                    Hash::Sha384 => rsa::pkcs1v15::SigningKey::<sha2::Sha384>::new(k)
                        .try_sign(message)
                        .map_err(sign_err)?
                        .to_vec(),
                    Hash::Sha512 => rsa::pkcs1v15::SigningKey::<sha2::Sha512>::new(k)
                        .try_sign(message)
                        .map_err(sign_err)?
                        .to_vec(),
                    _ => rsa::pkcs1v15::SigningKey::<sha2::Sha256>::new(k)
                        .try_sign(message)
                        .map_err(sign_err)?
                        .to_vec(),
                }
            }
            PrivateKey::P256(k) => {
                let sk = p256::ecdsa::SigningKey::from(k.clone());
                let sig: p256::ecdsa::Signature =
                    sk.sign_prehash(&hash.digest(message)).map_err(sign_err)?;
                sig.to_der().to_vec()
            }
            PrivateKey::P384(k) => {
                let sk = p384::ecdsa::SigningKey::from(k.clone());
                let sig: p384::ecdsa::Signature =
                    sk.sign_prehash(&hash.digest(message)).map_err(sign_err)?;
                sig.to_der().to_vec()
            }
            PrivateKey::Ed25519(k) => {
                use ed25519_dalek::Signer as _;
                k.sign(message).to_bytes().to_vec()
            }
            PrivateKey::X25519(_) => {
                return Err(CryptoError::Sign("an X25519 key cannot sign".into()));
            }
        })
    }

    /// `rsa-<bits>`, `ecdsa-p256`, `ecdsa-p384`, `ed25519`.
    pub fn algorithm(&self) -> String {
        use rsa::traits::PublicKeyParts;
        match self {
            PrivateKey::Rsa(k) => format!("rsa-{}", k.n().bits()),
            PrivateKey::P256(_) => "ecdsa-p256".into(),
            PrivateKey::P384(_) => "ecdsa-p384".into(),
            PrivateKey::Ed25519(_) => "ed25519".into(),
            PrivateKey::X25519(_) => "x25519".into(),
        }
    }

    /// PKCS#8 `PrivateKeyInfo` DER.
    pub fn to_pkcs8_der(&self) -> Result<Vec<u8>> {
        use pkcs8::EncodePrivateKey;
        Ok(match self {
            PrivateKey::Rsa(k) => k.to_pkcs8_der().map_err(err)?.as_bytes().to_vec(),
            PrivateKey::P256(k) => k.to_pkcs8_der().map_err(err)?.as_bytes().to_vec(),
            PrivateKey::P384(k) => k.to_pkcs8_der().map_err(err)?.as_bytes().to_vec(),
            // RFC 8410 section 7: the private key is a CurvePrivateKey (OCTET
            // STRING) inside the PrivateKeyInfo privateKey OCTET STRING.
            PrivateKey::Ed25519(k) => curve_pkcs8(oid::ED25519, &k.to_bytes())?,
            PrivateKey::X25519(k) => curve_pkcs8(oid::X25519, &k.to_bytes())?,
        })
    }

    /// Read a PKCS#8 `PrivateKeyInfo` DER of any supported type.
    pub fn from_pkcs8_der(der_bytes: &[u8]) -> Result<PrivateKey> {
        use pkcs8::DecodePrivateKey;
        let info = Pkcs8Info::from_der(der_bytes).map_err(|e| CryptoError::Parse(e.to_string()))?;
        let parse = |e: pkcs8::Error| CryptoError::Parse(e.to_string());
        Ok(match info.algorithm.oid {
            oid::RSA_ENCRYPTION => PrivateKey::Rsa(Box::new(
                rsa::RsaPrivateKey::from_pkcs8_der(der_bytes).map_err(parse)?,
            )),
            oid::EC_PUBLIC_KEY => {
                let curve = info
                    .algorithm
                    .parameters
                    .as_ref()
                    .and_then(|p| p.decode_as::<const_oid::ObjectIdentifier>().ok());
                match curve {
                    Some(oid::SECP256R1) => {
                        PrivateKey::P256(p256::SecretKey::from_pkcs8_der(der_bytes).map_err(parse)?)
                    }
                    Some(oid::SECP384R1) => {
                        PrivateKey::P384(p384::SecretKey::from_pkcs8_der(der_bytes).map_err(parse)?)
                    }
                    _ => return Err(CryptoError::Parse("unsupported elliptic curve".into())),
                }
            }
            oid::ED25519 | oid::X25519 => {
                let inner = OctetString::from_der(info.private_key.as_bytes())
                    .map_err(|e| CryptoError::Parse(e.to_string()))?;
                let seed: [u8; 32] = inner
                    .as_bytes()
                    .try_into()
                    .map_err(|_| CryptoError::Parse("bad curve private key".into()))?;
                if info.algorithm.oid == oid::ED25519 {
                    PrivateKey::Ed25519(Box::new(ed25519_dalek::SigningKey::from_bytes(&seed)))
                } else {
                    PrivateKey::X25519(Box::new(x25519_dalek::StaticSecret::from(seed)))
                }
            }
            other => {
                return Err(CryptoError::Parse(format!(
                    "unsupported private key algorithm {other}"
                )));
            }
        })
    }

    /// This key's public half is the one in `spki`.
    pub fn matches(&self, spki: &SubjectPublicKeyInfoOwned) -> bool {
        self.public_spki()
            .map(|mine| {
                mine.subject_public_key == spki.subject_public_key
                    && mine.algorithm.oid == spki.algorithm.oid
            })
            .unwrap_or(false)
    }
}

fn random_scalar<K>(len: usize, make: impl Fn(&[u8]) -> Option<K>) -> Result<K> {
    let mut buf = vec![0u8; len];
    for _ in 0..16 {
        rng::fill_random(&mut buf);
        if let Some(k) = make(&buf) {
            zeroize::Zeroize::zeroize(&mut buf);
            return Ok(k);
        }
    }
    Err(err("no valid scalar after 16 draws"))
}

/// `PrivateKeyInfo ::= SEQUENCE { version, privateKeyAlgorithm, privateKey,
/// attributes [0] IMPLICIT OPTIONAL }` (RFC 5208), enough to dispatch on the
/// algorithm and to write an Ed25519 key.
#[derive(der::Sequence)]
struct Pkcs8Info {
    version: u8,
    algorithm: AlgorithmIdentifierOwned,
    private_key: OctetString,
}

fn curve_pkcs8(o: const_oid::ObjectIdentifier, secret: &[u8; 32]) -> Result<Vec<u8>> {
    let inner = OctetString::new(secret.to_vec())
        .map_err(err)?
        .to_der()
        .map_err(err)?;
    Pkcs8Info {
        version: 0,
        algorithm: AlgorithmIdentifierOwned {
            oid: o,
            parameters: None,
        },
        private_key: OctetString::new(inner).map_err(err)?,
    }
    .to_der()
    .map_err(err)
}

/// RSASSA-PSS parameters for `hash`: MGF1 with the same hash, salt = hash size.
fn pss_params_any(hash: Hash) -> Result<der::Any> {
    #[derive(der::Sequence)]
    struct Params {
        #[asn1(context_specific = "0", tag_mode = "EXPLICIT")]
        hash: AlgorithmIdentifierOwned,
        #[asn1(context_specific = "1", tag_mode = "EXPLICIT")]
        mgf: AlgorithmIdentifierOwned,
        #[asn1(context_specific = "2", tag_mode = "EXPLICIT")]
        salt: u32,
    }
    let h = AlgorithmIdentifierOwned {
        oid: hash.oid(),
        parameters: Some(der::Any::null()),
    };
    let salt = match hash {
        Hash::Sha1 => 20,
        Hash::Sha256 => 32,
        Hash::Sha384 => 48,
        Hash::Sha512 => 64,
    };
    let p = Params {
        hash: h.clone(),
        mgf: AlgorithmIdentifierOwned {
            oid: oid::MGF1,
            parameters: Some(der::Any::encode_from(&h).map_err(err)?),
        },
        salt,
    };
    der::Any::encode_from(&p).map_err(err)
}

/// Wrap a key as the passphrase-encrypted PKCS#8 PEM bundle. An empty passphrase
/// is accepted here (a PKCS#12 file without a password re-wraps under the same
/// empty password); key generation refuses one.
pub fn wrap(key: &PrivateKey, passphrase: &str) -> Result<String> {
    let der = crate::smime::encrypt_pkcs8(&key.to_pkcs8_der()?, passphrase, 600_000)?;
    der::pem::encode_string("ENCRYPTED PRIVATE KEY", der::pem::LineEnding::LF, &der)
        .map_err(|e| CryptoError::Parse(e.to_string()))
}

/// Unwrap a bundle (encrypted PKCS#8 PEM; a clear PKCS#8 PEM is read too).
pub fn unwrap(bundle_pem: &str, passphrase: &str) -> Result<PrivateKey> {
    let (label, der_bytes) = der::pem::decode_vec(bundle_pem.trim().as_bytes())
        .map_err(|e| CryptoError::Parse(e.to_string()))?;
    match label {
        "ENCRYPTED PRIVATE KEY" => {
            let epki = pkcs8::EncryptedPrivateKeyInfoRef::try_from(der_bytes.as_slice())
                .map_err(|e| CryptoError::Parse(e.to_string()))?;
            let doc = epki
                .decrypt(passphrase.as_bytes())
                .map_err(|e| CryptoError::Decrypt(format!("bundle unlock: {e}")))?;
            PrivateKey::from_pkcs8_der(doc.as_bytes())
        }
        "PRIVATE KEY" => PrivateKey::from_pkcs8_der(&der_bytes),
        other => Err(CryptoError::Parse(format!("unexpected PEM label {other}"))),
    }
}
