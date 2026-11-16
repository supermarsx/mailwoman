//! RFC 5929 tls-server-end-point from the verified origin leaf certificate.
//! Unsupported signature digest algorithms fail closed; the proxy certificate
//! and unauthenticated response headers never participate in this value.
use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};
use x509_cert::der::{Decode, Sequence};
use x509_cert::{AlgorithmIdentifier, Certificate};

#[derive(Sequence)]
struct PssParameters {
    #[asn1(context_specific = "0", tag_mode = "EXPLICIT", optional = "true")]
    hash: Option<AlgorithmIdentifier>,
    #[asn1(context_specific = "1", tag_mode = "EXPLICIT", optional = "true")]
    mask: Option<AlgorithmIdentifier>,
    #[asn1(context_specific = "2", tag_mode = "EXPLICIT", optional = "true")]
    salt: Option<u16>,
    #[asn1(context_specific = "3", tag_mode = "EXPLICIT", optional = "true")]
    trailer: Option<u16>,
}

pub(crate) fn certificate_digest(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let certificate = Certificate::from_der(bytes).map_err(|_| "Invalid origin certificate")?;
    let algorithm = certificate.signature_algorithm();
    let digest = match algorithm.oid.to_string().as_str() {
        "1.2.840.113549.1.1.4"
        | "1.2.840.113549.1.1.5"
        | "1.2.840.10045.4.1"
        | "1.2.840.10040.4.3" => "sha256",
        "1.2.840.113549.1.1.14" | "1.2.840.10045.4.3.1" | "2.16.840.1.101.3.4.3.1" => "sha224",
        "1.2.840.113549.1.1.11" | "1.2.840.10045.4.3.2" | "2.16.840.1.101.3.4.3.2" => "sha256",
        "1.2.840.113549.1.1.12" | "1.2.840.10045.4.3.3" => "sha384",
        "1.2.840.113549.1.1.13" | "1.2.840.10045.4.3.4" => "sha512",
        "1.2.840.113549.1.1.10" => {
            let parameters = algorithm
                .parameters
                .as_ref()
                .ok_or("Missing origin PSS parameters")?
                .decode_as::<PssParameters>()
                .map_err(|_| "Invalid origin PSS parameters")?;
            let oid = parameters
                .hash
                .as_ref()
                .map(|hash| hash.oid.to_string())
                .unwrap_or_else(|| "1.3.14.3.2.26".into());
            if let Some(mask) = &parameters.mask {
                if mask.oid.to_string() != "1.2.840.113549.1.1.8" {
                    return Err("Unsupported origin PSS mask".into());
                }
                let hash = mask
                    .parameters
                    .as_ref()
                    .ok_or("Missing origin PSS mask hash")?
                    .decode_as::<AlgorithmIdentifier>()
                    .map_err(|_| "Invalid origin PSS mask hash")?;
                if hash.oid.to_string() != oid {
                    return Err(
                        "Multiple origin signature hashes are unsupported for channel binding"
                            .into(),
                    );
                }
            }
            if parameters.mask.is_none() && oid != "1.3.14.3.2.26" {
                return Err(
                    "Multiple origin signature hashes are unsupported for channel binding".into(),
                );
            }
            match oid.as_str() {
                "1.3.14.3.2.26" => "sha256",
                "2.16.840.1.101.3.4.2.4" => "sha224",
                "2.16.840.1.101.3.4.2.1" => "sha256",
                "2.16.840.1.101.3.4.2.2" => "sha384",
                "2.16.840.1.101.3.4.2.3" => "sha512",
                _ => return Err("Unsupported origin signature hash".into()),
            }
        }
        _ => {
            return Err(
                "Origin certificate signature has no supported channel binding hash".into(),
            );
        }
    };
    Ok(match digest {
        "sha224" => Sha224::digest(bytes).to_vec(),
        "sha384" => Sha384::digest(bytes).to_vec(),
        "sha512" => Sha512::digest(bytes).to_vec(),
        _ => Sha256::digest(bytes).to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hashes_the_leaf_certificate_and_rejects_invalid_der() {
        let issued = rcgen::generate_simple_self_signed(vec!["exchange.test".into()]).unwrap();
        let bytes = issued.cert.der();
        assert_eq!(
            certificate_digest(bytes).unwrap(),
            Sha256::digest(bytes).to_vec()
        );
        assert!(certificate_digest(b"not a certificate").is_err());
        let key = rcgen::KeyPair::generate_for(&rcgen::PKCS_ECDSA_P384_SHA384).unwrap();
        let certificate = rcgen::CertificateParams::new(vec!["exchange.test".into()])
            .unwrap()
            .self_signed(&key)
            .unwrap();
        assert_eq!(
            certificate_digest(certificate.der()).unwrap(),
            Sha384::digest(certificate.der()).to_vec(),
            "RFC5929 follows the certificate signature hash"
        );
    }
}
