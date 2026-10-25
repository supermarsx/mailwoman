//! COSE_Key (RFC 9052/9053) decode for ES256 (ECDSA/P-256/SHA-256),
//! EdDSA (Ed25519), and RS256 (RSA PKCS#1 v1.5/SHA-256).
//!
//! A credential public key is a CBOR map keyed by small integer labels:
//!   1  = kty  (2 = EC2, 1 = OKP)
//!   3  = alg  (-7 = ES256, -8 = EdDSA)
//!   -1 = crv  (1 = P-256 for EC2, 6 = Ed25519 for OKP)
//!   -2 = x    (byte string)
//!   -3 = y    (byte string, EC2 only)

use crate::MfaError;
use crate::cbor::Reader;

// COSE integer labels.
const LABEL_KTY: i64 = 1;
const LABEL_ALG: i64 = 3;
const LABEL_CRV: i64 = -1;
const LABEL_X: i64 = -2;
const LABEL_Y: i64 = -3;

// kty values.
const KTY_OKP: i64 = 1;
const KTY_EC2: i64 = 2;

// alg values.
const ALG_ES256: i64 = -7;
const ALG_EDDSA: i64 = -8;

// crv values.
const CRV_P256: i64 = 1;
const CRV_ED25519: i64 = 6;

/// The COSE signature algorithm bound to a stored credential.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoseAlg {
    /// ECDSA with P-256 and SHA-256 (`-7`).
    Es256,
    /// EdDSA with Ed25519 (`-8`).
    EdDsa,
    /// RSA PKCS#1 v1.5 with SHA-256 (`-257`).
    Rs256,
}

/// A decoded credential public key with its raw coordinates.
#[derive(Debug, Clone)]
pub enum CoseKey {
    /// P-256 public key: fixed 32-byte affine `x`/`y`.
    Es256 {
        x: [u8; 32],
        y: [u8; 32],
    },
    /// Ed25519 public key: 32-byte compressed point.
    EdDsa {
        x: [u8; 32],
    },
    Rs256 {
        n: Vec<u8>,
        e: Vec<u8>,
    },
}

impl CoseKey {
    /// The algorithm this key verifies under.
    pub fn alg(&self) -> CoseAlg {
        match self {
            CoseKey::Es256 { .. } => CoseAlg::Es256,
            CoseKey::EdDsa { .. } => CoseAlg::EdDsa,
            CoseKey::Rs256 { .. } => CoseAlg::Rs256,
        }
    }

    /// Decode a COSE_Key from its raw CBOR bytes. Rejects anything that is not a
    /// complete, consistent ES256, EdDSA, or RS256 key.
    pub fn from_cbor(bytes: &[u8]) -> Result<CoseKey, MfaError> {
        let mut r = Reader::new(bytes);
        let key = Self::decode(&mut r)?;
        if r.pos != bytes.len() {
            return Err(MfaError::Cose("trailing key data".into()));
        }
        Ok(key)
    }

    /// Decode a COSE_Key from a reader positioned at the map header, leaving the
    /// reader just past the map (used to locate the key inside `authData`).
    pub(crate) fn decode(r: &mut Reader) -> Result<CoseKey, MfaError> {
        let n = r.map_len()?;
        let mut kty: Option<i64> = None;
        let mut alg: Option<i64> = None;
        let mut crv: Option<i64> = None;
        let mut modulus: Option<Vec<u8>> = None;
        let mut x: Option<Vec<u8>> = None;
        let mut y: Option<Vec<u8>> = None;
        let mut labels = std::collections::BTreeSet::new();

        for _ in 0..n {
            let label = r.int()?;
            if !labels.insert(label) {
                return Err(MfaError::Cose("duplicate key label".into()));
            }
            match label {
                LABEL_KTY => kty = Some(r.int()?),
                LABEL_ALG => alg = Some(r.int()?),
                LABEL_CRV => {
                    if r.next_major()? == crate::cbor::MAJOR_BYTES {
                        modulus = Some(r.bytes()?.to_vec());
                    } else {
                        crv = Some(r.int()?);
                    }
                }
                LABEL_X => x = Some(r.bytes()?.to_vec()),
                LABEL_Y => y = Some(r.bytes()?.to_vec()),
                // Unknown labels (e.g. key_ops) — skip the value.
                _ => r.skip()?,
            }
        }

        let err = |m: &str| MfaError::Cose(m.to_string());
        let kty = kty.ok_or_else(|| err("missing kty"))?;
        let alg = alg.ok_or_else(|| err("missing alg"))?;

        match alg {
            ALG_ES256 => {
                if kty != KTY_EC2 {
                    return Err(err("ES256 requires kty EC2"));
                }
                if crv != Some(CRV_P256) {
                    return Err(err("ES256 requires crv P-256"));
                }
                let x =
                    pad32(&x.ok_or_else(|| err("missing x"))?).ok_or_else(|| err("bad x len"))?;
                let y =
                    pad32(&y.ok_or_else(|| err("missing y"))?).ok_or_else(|| err("bad y len"))?;
                Ok(CoseKey::Es256 { x, y })
            }
            ALG_EDDSA => {
                if kty != KTY_OKP {
                    return Err(err("EdDSA requires kty OKP"));
                }
                if crv != Some(CRV_ED25519) {
                    return Err(err("EdDSA requires crv Ed25519"));
                }
                let xv = x.ok_or_else(|| err("missing x"))?;
                let x: [u8; 32] = xv
                    .as_slice()
                    .try_into()
                    .map_err(|_| err("Ed25519 x must be 32 bytes"))?;
                Ok(CoseKey::EdDsa { x })
            }
            -257 => {
                if kty != 3 {
                    return Err(err("RS256 requires kty RSA"));
                }
                let n = modulus.ok_or_else(|| err("missing RSA modulus"))?;
                let e = x.ok_or_else(|| err("missing RSA exponent"))?;
                rsa_public_key(&n, &e)?;
                Ok(CoseKey::Rs256 { n, e })
            }
            _ => Err(MfaError::UnsupportedAlg),
        }
    }
}

pub(crate) fn rsa_public_key(n: &[u8], e: &[u8]) -> Result<rsa::RsaPublicKey, MfaError> {
    let error = || MfaError::Cose("invalid RSA key (2048..8192-bit modulus required)".into());
    if !(256..=1024).contains(&n.len())
        || n[0] == 0
        || (n.len() == 256 && n[0] & 0x80 == 0)
        || e.is_empty()
        || e.len() > 8
    {
        return Err(error());
    }
    let n = rsa::BoxedUint::from_be_slice(n, (n.len() * 8) as u32).map_err(|_| error())?;
    let e = rsa::BoxedUint::from_be_slice(e, 64).map_err(|_| error())?;
    rsa::RsaPublicKey::new(n, e).map_err(|_| error())
}

/// Right-align a big-endian field element into a fixed 32-byte array. Accepts a
/// short encoding (leading zeros stripped) but rejects anything longer than 32.
fn pad32(v: &[u8]) -> Option<[u8; 32]> {
    if v.len() > 32 {
        return None;
    }
    let mut out = [0u8; 32];
    out[32 - v.len()..].copy_from_slice(v);
    Some(out)
}
