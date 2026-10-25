//! Certificate revocation for S/MIME (t32-e41, DA46): where to ask (AIA OCSP and
//! CRL distribution point URLs), the OCSP request (RFC 6960, SHA-1 CertID as the
//! RFC 5019 profile uses), and checking an OCSP response or a CRL against the
//! certificate and its issuer — signature, responder authorisation, CertID or
//! issuer match, freshness.
//!
//! Fetching is the server's job (through the egress gate, traffic class `pki`);
//! only the certificate's public identifiers leave the browser, never message
//! content. The browser gets back a [`RevocationResult`] per certificate and feeds
//! it to verification.
//!
//! The OCSP structures are defined here on `der` 0.8 like the PKCS#12 and
//! AuthEnvelopedData ones in `smime`: the `x509-ocsp` crate is built on the
//! previous RustCrypto generation (`x509-cert` 0.2 / `der` 0.7), whose types do not
//! interoperate with this crate's `x509-cert` 0.3.

use der::asn1::{BitString, GeneralizedTime, Null, OctetString};
use der::{Decode, Encode, Sequence};
use serde::{Deserialize, Serialize};
use sha1::Digest;
use spki::AlgorithmIdentifierOwned;
use x509_cert::Certificate;
use x509_cert::crl::CertificateList;
use x509_cert::ext::Extensions;
use x509_cert::ext::pkix::name::{DistributionPointName, GeneralName};
use x509_cert::ext::pkix::{
    AuthorityInfoAccessSyntax, CrlDistributionPoints, CrlReason, ExtendedKeyUsage, KeyUsage,
    KeyUsages,
};
use x509_cert::name::Name;
use x509_cert::serial_number::SerialNumber;

use super::oid;
use super::path::{fingerprint, name_eq};
use super::sig::{self, Use};
use crate::clock;
use crate::error::{CryptoError, Result};

/// OCSP and CRL times may run slightly ahead of our clock.
const SKEW_SECS: u64 = 300;
/// Without nextUpdate an OCSP response cannot authorize indefinite replay.
/// RFC 6960 section 3.2 requires thisUpdate to be sufficiently recent.
const OCSP_NO_NEXT_MAX_AGE_SECS: u64 = 24 * 60 * 60;

/// What the browser sends the server to have one certificate checked
/// (`POST /api/smime/revocation`): only public identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevocationQuery {
    /// SHA-256 of the certificate being checked (the key results come back under).
    pub cert_fingerprint: String,
    /// The certificate's serial number, hex.
    pub serial: String,
    /// The issuer certificate, base64 DER.
    pub issuer_der: String,
    pub ocsp_urls: Vec<String>,
    pub crl_urls: Vec<String>,
}

/// The revocation status of one certificate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevocationResult {
    pub cert_fingerprint: String,
    /// `"good"` | `"revoked"` | `"unknown"`.
    pub status: String,
    /// `"ocsp"` | `"crl"` | `"none"` (nothing could be fetched or checked).
    pub source: String,
    pub this_update: Option<String>,
    /// Cache the result until this time (RFC 3339).
    pub next_update: Option<String>,
    pub revoked_at: Option<String>,
    /// CRL reason code name, when given.
    pub reason: Option<String>,
}

impl RevocationResult {
    /// "Could not be checked" for a certificate.
    pub fn unknown(cert_fingerprint: &str) -> Self {
        RevocationResult {
            cert_fingerprint: cert_fingerprint.into(),
            status: "unknown".into(),
            source: "none".into(),
            this_update: None,
            next_update: None,
            revoked_at: None,
            reason: None,
        }
    }
}

/// The OCSP and CRL URLs a certificate names (HTTP only: LDAP and other schemes
/// are skipped).
pub fn revocation_urls(cert: &Certificate) -> (Vec<String>, Vec<String>) {
    let mut ocsp = Vec::new();
    let mut crl = Vec::new();
    let tbs = cert.tbs_certificate();
    if let Ok(Some((_, aia))) = tbs.get_extension::<AuthorityInfoAccessSyntax>() {
        for ad in &aia.0 {
            if ad.access_method == oid::AD_OCSP
                && let GeneralName::UniformResourceIdentifier(u) = &ad.access_location
                && is_http(u.as_str())
            {
                ocsp.push(u.as_str().to_string());
            }
        }
    }
    if let Ok(Some((_, dps))) = tbs.get_extension::<CrlDistributionPoints>() {
        for dp in &dps.0 {
            if let Some(DistributionPointName::FullName(names)) = &dp.distribution_point {
                for n in names {
                    if let GeneralName::UniformResourceIdentifier(u) = n
                        && is_http(u.as_str())
                    {
                        crl.push(u.as_str().to_string());
                    }
                }
            }
        }
    }
    (ocsp, crl)
}

fn is_http(u: &str) -> bool {
    let l = u.to_ascii_lowercase();
    l.starts_with("http://") || l.starts_with("https://")
}

/// The query for `cert`, issued by `issuer`.
pub fn query_for(cert: &Certificate, issuer: &Certificate) -> RevocationQuery {
    use base64::Engine;
    let (ocsp_urls, crl_urls) = revocation_urls(cert);
    RevocationQuery {
        cert_fingerprint: fingerprint(cert),
        serial: hex::encode(cert.tbs_certificate().serial_number().as_bytes()),
        issuer_der: base64::engine::general_purpose::STANDARD
            .encode(issuer.to_der().unwrap_or_default()),
        ocsp_urls,
        crl_urls,
    }
}

// ── OCSP ASN.1 (RFC 6960 §4) ─────────────────────────────────────────────────

#[derive(Clone, Debug, Sequence)]
struct CertId {
    hash_algorithm: AlgorithmIdentifierOwned,
    issuer_name_hash: OctetString,
    issuer_key_hash: OctetString,
    serial_number: SerialNumber,
}

#[derive(Clone, Debug, Sequence)]
struct Request {
    req_cert: CertId,
    #[asn1(context_specific = "0", tag_mode = "EXPLICIT", optional = "true")]
    single_request_extensions: Option<Extensions>,
}

#[derive(Clone, Debug, Sequence)]
struct TbsRequest {
    request_list: Vec<Request>,
}

#[derive(Clone, Debug, Sequence)]
struct OcspRequest {
    tbs_request: TbsRequest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, der::Enumerated)]
#[repr(u32)]
enum OcspResponseStatus {
    Successful = 0,
    MalformedRequest = 1,
    InternalError = 2,
    TryLater = 3,
    SigRequired = 5,
    Unauthorized = 6,
}

#[derive(Clone, Debug, Sequence)]
struct ResponseBytes {
    response_type: const_oid::ObjectIdentifier,
    response: OctetString,
}

#[derive(Clone, Debug, Sequence)]
struct OcspResponse {
    response_status: OcspResponseStatus,
    #[asn1(context_specific = "0", tag_mode = "EXPLICIT", optional = "true")]
    response_bytes: Option<ResponseBytes>,
}

#[derive(Clone, Debug, Sequence)]
struct BasicOcspResponse {
    tbs_response_data: ResponseData,
    signature_algorithm: AlgorithmIdentifierOwned,
    signature: BitString,
    #[asn1(context_specific = "0", tag_mode = "EXPLICIT", optional = "true")]
    certs: Option<Vec<Certificate>>,
}

fn v1() -> u8 {
    0
}

#[derive(Clone, Debug, Sequence)]
struct ResponseData {
    #[asn1(context_specific = "0", tag_mode = "EXPLICIT", default = "v1")]
    version: u8,
    responder_id: ResponderId,
    produced_at: GeneralizedTime,
    responses: Vec<SingleResponse>,
    #[asn1(context_specific = "1", tag_mode = "EXPLICIT", optional = "true")]
    response_extensions: Option<Extensions>,
}

#[derive(Clone, Debug, der::Choice)]
enum ResponderId {
    #[asn1(context_specific = "1", tag_mode = "EXPLICIT", constructed = "true")]
    ByName(Name),
    #[asn1(context_specific = "2", tag_mode = "EXPLICIT", constructed = "true")]
    ByKey(OctetString),
}

#[derive(Clone, Debug, Sequence)]
struct SingleResponse {
    cert_id: CertId,
    cert_status: CertStatus,
    this_update: GeneralizedTime,
    #[asn1(context_specific = "0", tag_mode = "EXPLICIT", optional = "true")]
    next_update: Option<GeneralizedTime>,
    #[asn1(context_specific = "1", tag_mode = "EXPLICIT", optional = "true")]
    single_extensions: Option<Extensions>,
}

#[derive(Clone, Debug, der::Choice)]
enum CertStatus {
    #[asn1(context_specific = "0", tag_mode = "IMPLICIT")]
    Good(Null),
    #[asn1(context_specific = "1", tag_mode = "IMPLICIT", constructed = "true")]
    Revoked(RevokedInfo),
    #[asn1(context_specific = "2", tag_mode = "IMPLICIT")]
    Unknown(Null),
}

#[derive(Clone, Debug, Sequence)]
struct RevokedInfo {
    revocation_time: GeneralizedTime,
    #[asn1(context_specific = "0", tag_mode = "EXPLICIT", optional = "true")]
    revocation_reason: Option<CrlReason>,
}

/// `id-pkix-ocsp-basic`.
const OCSP_BASIC: const_oid::ObjectIdentifier =
    const_oid::ObjectIdentifier::new_unwrap("1.3.6.1.5.5.7.48.1.1");

fn parse(e: impl std::fmt::Display) -> CryptoError {
    CryptoError::Parse(e.to_string())
}

fn sha1_cert_id(serial: &[u8], issuer: &Certificate) -> Result<CertId> {
    let name_der = issuer.tbs_certificate().subject().to_der().map_err(parse)?;
    let key = issuer
        .tbs_certificate()
        .subject_public_key_info()
        .subject_public_key
        .raw_bytes();
    Ok(CertId {
        hash_algorithm: AlgorithmIdentifierOwned {
            oid: oid::SHA1,
            parameters: Some(der::Any::null()),
        },
        issuer_name_hash: OctetString::new(sha1::Sha1::digest(&name_der).to_vec())
            .map_err(parse)?,
        issuer_key_hash: OctetString::new(sha1::Sha1::digest(key).to_vec()).map_err(parse)?,
        serial_number: SerialNumber::new(serial).map_err(parse)?,
    })
}

/// The serial (bytes) and issuer certificate a query names.
fn query_parts(q: &RevocationQuery) -> Result<(Vec<u8>, Certificate)> {
    use base64::Engine;
    let serial = hex::decode(q.serial.trim()).map_err(parse)?;
    let issuer_der = base64::engine::general_purpose::STANDARD
        .decode(q.issuer_der.trim())
        .map_err(parse)?;
    Ok((serial, Certificate::from_der(&issuer_der).map_err(parse)?))
}

/// A DER OCSP request for the certificate a query names. No nonce, as the RFC
/// 5019 profile most responders cache by.
pub fn ocsp_request(q: &RevocationQuery) -> Result<Vec<u8>> {
    let (serial, issuer) = query_parts(q)?;
    let serial = &serial[..];
    OcspRequest {
        tbs_request: TbsRequest {
            request_list: vec![Request {
                req_cert: sha1_cert_id(serial, &issuer)?,
                single_request_extensions: None,
            }],
        },
    }
    .to_der()
    .map_err(parse)
}

fn gt_secs(t: &GeneralizedTime) -> u64 {
    t.to_unix_duration().as_secs()
}

/// Check a DER OCSP response for the certificate a query names, at `now`. The
/// response must be signed by the issuer, or by a responder certificate the
/// issuer signed with `id-kp-OCSPSigning`; it must name this certificate's
/// CertID; it must be current (`thisUpdate` not in the future, `nextUpdate` not
/// passed).
pub fn check_ocsp_response(
    response_der: &[u8],
    q: &RevocationQuery,
    now: u64,
) -> Result<RevocationResult> {
    let (serial, issuer) = query_parts(q)?;
    let serial = &serial[..];
    let resp = OcspResponse::from_der(response_der).map_err(parse)?;
    if resp.response_status != OcspResponseStatus::Successful {
        return Err(CryptoError::Input(format!(
            "OCSP responder answered {:?}",
            resp.response_status
        )));
    }
    let bytes = resp
        .response_bytes
        .ok_or_else(|| CryptoError::Parse("OCSP response without a body".into()))?;
    if bytes.response_type != OCSP_BASIC {
        return Err(CryptoError::Parse(
            "OCSP response is not a basic response".into(),
        ));
    }
    let basic = BasicOcspResponse::from_der(bytes.response.as_bytes()).map_err(parse)?;
    if basic.tbs_response_data.version != 0
        || basic
            .tbs_response_data
            .response_extensions
            .iter()
            .flatten()
            .any(|e| e.critical)
    {
        return Err(CryptoError::Input(
            "unsupported OCSP version or critical response extension".into(),
        ));
    }
    let tbs = basic.tbs_response_data.to_der().map_err(parse)?;
    let sig_bytes = basic
        .signature
        .as_bytes()
        .ok_or_else(|| CryptoError::Parse("OCSP signature has unused bits".into()))?;

    // Who signed it: the issuer itself, or a delegated responder.
    let signed_by_issuer = sig::verify(
        issuer.tbs_certificate().subject_public_key_info(),
        &basic.signature_algorithm,
        None,
        &tbs,
        sig_bytes,
        Use::Certificate,
    )
    .is_ok();
    let signed_by_delegate = !signed_by_issuer
        && basic.certs.iter().flatten().any(|r| {
            responder_authorised(r, &issuer, now)
                && sig::verify(
                    r.tbs_certificate().subject_public_key_info(),
                    &basic.signature_algorithm,
                    None,
                    &tbs,
                    sig_bytes,
                    Use::Certificate,
                )
                .is_ok()
        });
    if !signed_by_issuer && !signed_by_delegate {
        return Err(CryptoError::Input(
            "the OCSP response is not signed by the issuer or its responder".into(),
        ));
    }

    let want = sha1_cert_id(serial, &issuer)?;
    let single = basic
        .tbs_response_data
        .responses
        .iter()
        .find(|s| cert_id_matches(&s.cert_id, &want, serial, &issuer))
        .ok_or_else(|| CryptoError::Input("the OCSP response is for another certificate".into()))?;
    if single
        .single_extensions
        .iter()
        .flatten()
        .any(|e| e.critical)
    {
        return Err(CryptoError::Input(
            "unsupported critical OCSP status extension".into(),
        ));
    }
    let this = gt_secs(&single.this_update);
    if single.next_update.is_none()
        && now.saturating_sub(this) > OCSP_NO_NEXT_MAX_AGE_SECS + SKEW_SECS
    {
        return Err(CryptoError::Input(
            "the OCSP response without nextUpdate is too old".into(),
        ));
    }
    if single
        .next_update
        .as_ref()
        .is_some_and(|next| gt_secs(next) < this)
    {
        return Err(CryptoError::Input(
            "the OCSP response has an invalid time interval".into(),
        ));
    }
    if this > now.saturating_add(SKEW_SECS) {
        return Err(CryptoError::Input(
            "the OCSP response is dated in the future".into(),
        ));
    }
    if let Some(next) = &single.next_update
        && gt_secs(next).saturating_add(SKEW_SECS) < now
    {
        return Err(CryptoError::Input("the OCSP response has expired".into()));
    }
    let mut out = RevocationResult {
        cert_fingerprint: q.cert_fingerprint.clone(),
        status: "good".into(),
        source: "ocsp".into(),
        this_update: Some(clock::rfc3339(this)),
        next_update: Some(clock::rfc3339(
            single
                .next_update
                .as_ref()
                .map(gt_secs)
                .unwrap_or_else(|| this.saturating_add(OCSP_NO_NEXT_MAX_AGE_SECS)),
        )),
        revoked_at: None,
        reason: None,
    };
    match &single.cert_status {
        CertStatus::Good(_) => {}
        CertStatus::Revoked(info) => {
            out.status = "revoked".into();
            out.revoked_at = Some(clock::rfc3339(gt_secs(&info.revocation_time)));
            out.reason = info.revocation_reason.map(|r| format!("{r:?}"));
        }
        CertStatus::Unknown(_) => out.status = "unknown".into(),
    }
    Ok(out)
}

fn cert_id_matches(got: &CertId, want: &CertId, serial: &[u8], issuer: &Certificate) -> bool {
    let want_serial = SerialNumber::<x509_cert::certificate::Rfc5280>::new(serial)
        .map(|s| s.as_bytes().to_vec())
        .unwrap_or_default();
    if got.serial_number.as_bytes() != want_serial {
        return false;
    }
    if got.hash_algorithm.oid == oid::SHA1 {
        return got.issuer_name_hash == want.issuer_name_hash
            && got.issuer_key_hash == want.issuer_key_hash;
    }
    // Responders may answer with a SHA-256 CertID.
    if got.hash_algorithm.oid == oid::SHA256 {
        let name_der = issuer
            .tbs_certificate()
            .subject()
            .to_der()
            .unwrap_or_default();
        let key = issuer
            .tbs_certificate()
            .subject_public_key_info()
            .subject_public_key
            .raw_bytes();
        return got.issuer_name_hash.as_bytes() == sha2::Sha256::digest(&name_der).as_slice()
            && got.issuer_key_hash.as_bytes() == sha2::Sha256::digest(key).as_slice();
    }
    false
}

/// A delegated OCSP responder: issued by `issuer`, carrying `id-kp-OCSPSigning`,
/// valid now.
fn responder_authorised(r: &Certificate, issuer: &Certificate, now: u64) -> bool {
    let tbs = r.tbs_certificate();
    if !name_eq(tbs.issuer(), issuer.tbs_certificate().subject()) {
        return false;
    }
    let v = tbs.validity();
    if now < v.not_before.to_unix_duration().as_secs()
        || now > v.not_after.to_unix_duration().as_secs()
    {
        return false;
    }
    let eku_ok = matches!(
        tbs.get_extension::<ExtendedKeyUsage>(),
        Ok(Some((_, eku))) if eku.0.contains(&oid::KP_OCSP_SIGNING)
    );
    let Ok(rtbs) = tbs.to_der() else {
        return false;
    };
    eku_ok
        && r.signature().as_bytes().is_some_and(|s| {
            sig::verify(
                issuer.tbs_certificate().subject_public_key_info(),
                r.signature_algorithm(),
                None,
                &rtbs,
                s,
                Use::Certificate,
            )
            .is_ok()
        })
}

/// Check a DER CRL for the certificate a query names, at `now`: signed by the
/// issuer, issued by its name, current (`nextUpdate` not passed; a stale CRL
/// answers `unknown`).
pub fn check_crl(crl_der: &[u8], q: &RevocationQuery, now: u64) -> Result<RevocationResult> {
    let (serial, issuer) = query_parts(q)?;
    let serial = &serial[..];
    let crl =
        CertificateList::<x509_cert::certificate::Rfc5280>::from_der(crl_der).map_err(parse)?;
    let tbs = &crl.tbs_cert_list;
    // We support complete, direct CRLs. A delta needs its base; a scoped or
    // indirect list needs distribution-point processing. Absence from any of
    // those lists is not evidence that a certificate is unrevoked (RFC 5280 5.2).
    let delta = const_oid::ObjectIdentifier::new_unwrap("2.5.29.27");
    let scope = const_oid::ObjectIdentifier::new_unwrap("2.5.29.28");
    if tbs
        .crl_extensions
        .iter()
        .flatten()
        .any(|e| e.critical || e.extn_id == delta || e.extn_id == scope)
        || tbs.revoked_certificates.iter().flatten().any(|entry| {
            entry
                .crl_entry_extensions
                .iter()
                .flatten()
                .any(|e| e.critical)
        })
    {
        return Err(CryptoError::Input(
            "unsupported partial CRL or critical extension".into(),
        ));
    }
    match issuer.tbs_certificate().get_extension::<KeyUsage>() {
        Ok(Some((_, usage))) if !usage.0.contains(KeyUsages::CRLSign) => {
            return Err(CryptoError::Input("issuer key cannot sign CRLs".into()));
        }
        Err(_) => return Err(CryptoError::Input("malformed CRL issuer key usage".into())),
        _ => {}
    }

    if !name_eq(&tbs.issuer, issuer.tbs_certificate().subject()) {
        return Err(CryptoError::Input("the CRL is from another issuer".into()));
    }
    let tbs_der = tbs.to_der().map_err(parse)?;
    let sig_bytes = crl
        .signature
        .as_bytes()
        .ok_or_else(|| CryptoError::Parse("CRL signature has unused bits".into()))?;
    sig::verify(
        issuer.tbs_certificate().subject_public_key_info(),
        &crl.signature_algorithm,
        None,
        &tbs_der,
        sig_bytes,
        Use::Certificate,
    )
    .map_err(|_| CryptoError::Input("the CRL signature does not verify".into()))?;
    let this = tbs.this_update.to_unix_duration().as_secs();
    let next = tbs.next_update.map(|t| t.to_unix_duration().as_secs());
    let mut out = RevocationResult {
        cert_fingerprint: q.cert_fingerprint.clone(),
        status: "good".into(),
        source: "crl".into(),
        this_update: Some(clock::rfc3339(this)),
        next_update: next.map(clock::rfc3339),
        revoked_at: None,
        reason: None,
    };
    if this > now.saturating_add(SKEW_SECS)
        || next.is_none_or(|n| n < this || n.saturating_add(SKEW_SECS) < now)
    {
        out.status = "unknown".into();
        return Ok(out);
    }
    let want = SerialNumber::<x509_cert::certificate::Rfc5280>::new(serial).map_err(parse)?;
    if let Some(entry) = tbs
        .revoked_certificates
        .iter()
        .flatten()
        .find(|r| r.serial_number.as_bytes() == want.as_bytes())
    {
        out.status = "revoked".into();
        out.revoked_at = Some(clock::rfc3339(
            entry.revocation_date.to_unix_duration().as_secs(),
        ));
        out.reason = entry
            .crl_entry_extensions
            .iter()
            .flatten()
            .find(|e| e.extn_id == const_oid::ObjectIdentifier::new_unwrap("2.5.29.21"))
            .and_then(|e| CrlReason::from_der(e.extn_value.as_bytes()).ok())
            .map(|r| format!("{r:?}"));
    }
    Ok(out)
}

#[cfg(test)]
mod security_review_tests {
    use super::*;

    #[test]
    fn ocsp_without_next_update_requires_a_recent_this_update() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/crypto/smime/e41");
        let cert =
            crate::smime_cms::parse_certificate(&std::fs::read(dir.join("alice.pem")).unwrap())
                .unwrap();
        let key =
            crate::smime_keys::unwrap(&std::fs::read_to_string(dir.join("alice.key")).unwrap(), "")
                .unwrap();
        let now = clock::now_secs();
        let time = |seconds| {
            GeneralizedTime::from_unix_duration(std::time::Duration::from_secs(seconds)).unwrap()
        };
        for age in [60, 365 * 86400] {
            let tbs_response_data = ResponseData {
                version: 0,
                responder_id: ResponderId::ByName(cert.tbs_certificate().subject().clone()),
                produced_at: time(now - age),
                responses: vec![SingleResponse {
                    cert_id: sha1_cert_id(cert.tbs_certificate().serial_number().as_bytes(), &cert)
                        .unwrap(),
                    cert_status: CertStatus::Good(Null),
                    this_update: time(now - age),
                    next_update: None,
                    single_extensions: None,
                }],
                response_extensions: None,
            };
            let signature_algorithm = key.signature_algorithm(sig::Hash::Sha256, false).unwrap();
            let signature = BitString::from_bytes(
                &key.sign(
                    sig::Hash::Sha256,
                    false,
                    &tbs_response_data.to_der().unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
            let basic = BasicOcspResponse {
                tbs_response_data,
                signature_algorithm,
                signature,
                certs: None,
            };
            let response = OcspResponse {
                response_status: OcspResponseStatus::Successful,
                response_bytes: Some(ResponseBytes {
                    response_type: OCSP_BASIC,
                    response: OctetString::new(basic.to_der().unwrap()).unwrap(),
                }),
            };
            let query = query_for(&cert, &cert);
            let checked = check_ocsp_response(&response.to_der().unwrap(), &query, now);
            if age == 60 {
                let result = checked.expect("a recent authentic response remains usable");
                assert_eq!(result.status, "good");
                assert_eq!(
                    result.next_update,
                    Some(clock::rfc3339(now - age + OCSP_NO_NEXT_MAX_AGE_SECS))
                );
            } else {
                assert!(
                    checked.is_err(),
                    "an authentic year-old good response must not conceal a later revocation"
                );
            }
        }
    }
}
