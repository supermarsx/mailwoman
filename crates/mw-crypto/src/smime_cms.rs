//! CMS for S/MIME 4.0 (t32-e41; RFC 5652, RFC 8551): signing (detached by
//! default, or opaque), verification with externally supplied content, chain and
//! revocation state, and encryption with key transport (RSA) or key agreement
//! (ECDH P-256 / P-384 / X25519, RFC 5753 / RFC 8418, AES key wrap) under
//! AES-256-GCM AuthEnvelopedData (RFC 5083/5084) or AES-256-CBC EnvelopedData.
//!
//! Decryption is compiled only into the browser guest and this crate's tests
//! (`client-private-ops`, DA48): the server binary contains no RSA decrypt.

use cms::cert::{CertificateChoices, IssuerAndSerialNumber};
use cms::content_info::{CmsVersion, ContentInfo};
use cms::enveloped_data::{
    EncryptedContentInfo, EnvelopedData, KeyAgreeRecipientIdentifier, KeyAgreeRecipientInfo,
    KeyTransRecipientInfo, OriginatorIdentifierOrKey, OriginatorPublicKey, RecipientEncryptedKey,
    RecipientIdentifier, RecipientInfo, RecipientInfos,
};
use cms::signed_data::{
    CertificateSet, EncapsulatedContentInfo, SignedData, SignerIdentifier, SignerInfo, SignerInfos,
};
use der::asn1::{BitString, OctetString, SetOfVec};
use der::{Any, Decode, Encode, Tag};
use serde::{Deserialize, Serialize};
use spki::{AlgorithmIdentifierOwned, ObjectIdentifier};
use x509_cert::Certificate;
use x509_cert::attr::Attribute;
use x509_cert::ext::pkix::SubjectKeyIdentifier;
use x509_cert::time::Time;

use crate::clock;
use crate::error::{CryptoError, Result};
use crate::pkix::path::{self, PathError, Policy, Purpose, fingerprint, name_eq};
use crate::pkix::revocation::{RevocationQuery, RevocationResult, query_for};
use crate::pkix::sig::{self, Hash, Use};
use crate::pkix::{name, oid};
use crate::rng;
use crate::smime_keys;
#[cfg(any(target_arch = "wasm32", feature = "client-private-ops"))]
use crate::smime_keys::PrivateKey;
use crate::smime_trust::TrustConfig;
use crate::verdict::{self, Verification};

fn parse(e: impl std::fmt::Display) -> CryptoError {
    CryptoError::Parse(e.to_string())
}

fn alg(o: ObjectIdentifier, parameters: Option<Any>) -> AlgorithmIdentifierOwned {
    AlgorithmIdentifierOwned { oid: o, parameters }
}

fn attribute(o: ObjectIdentifier, value: Any) -> Result<Attribute> {
    Ok(Attribute {
        oid: o,
        values: SetOfVec::try_from(vec![value]).map_err(parse)?,
    })
}

fn issuer_and_serial(cert: &Certificate) -> IssuerAndSerialNumber {
    IssuerAndSerialNumber {
        issuer: cert.tbs_certificate().issuer().clone(),
        serial_number: cert.tbs_certificate().serial_number().clone(),
    }
}

/// One certificate, PEM or DER.
pub fn parse_certificate(bytes: &[u8]) -> Result<Certificate> {
    path::parse_certs(bytes)
        .into_iter()
        .next()
        .ok_or_else(|| CryptoError::Parse("not a certificate".into()))
}

fn x509_time(unix_secs: u64) -> Result<Time> {
    use der::asn1::{GeneralizedTime, UtcTime};
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

// ── signing ──────────────────────────────────────────────────────────────────

/// How to sign.
#[derive(Debug, Clone)]
pub struct SignOptions {
    /// `true` (default): no encapsulated content, for `multipart/signed`.
    /// `false`: the content inside, for `application/pkcs7-mime; smime-type=signed-data`.
    pub detached: bool,
    /// Digest; `None` = the key's default (SHA-256, SHA-384 for P-384, SHA-512
    /// for Ed25519).
    pub hash: Option<Hash>,
    /// RSASSA-PSS instead of PKCS#1 v1.5 for an RSA key.
    pub pss: bool,
    /// Further certificates to include (the issuing chain), PEM or DER each.
    pub chain: Vec<Vec<u8>>,
    /// Signing time; `None` = now.
    pub signing_time: Option<u64>,
}

impl Default for SignOptions {
    fn default() -> Self {
        SignOptions {
            detached: true,
            hash: None,
            pss: false,
            chain: Vec::new(),
            signing_time: None,
        }
    }
}

/// The content-encryption algorithms we advertise in SMIMECapabilities, strongest
/// first (RFC 8551 §2.5.2): AES-256-GCM, AES-128-GCM, AES-256-CBC, AES-128-CBC.
const ADVERTISED: [ObjectIdentifier; 4] = [
    oid::AES256_GCM,
    oid::AES128_GCM,
    oid::AES256_CBC,
    oid::AES128_CBC,
];

#[derive(der::Sequence)]
struct SmimeCapability {
    capability_id: ObjectIdentifier,
    #[asn1(optional = "true")]
    parameters: Option<Any>,
}

/// Sign `data` with the key in `bundle` and the certificate `cert` (PEM or DER).
/// Returns DER `ContentInfo(SignedData)`.
pub fn sign(
    data: &[u8],
    cert: &[u8],
    bundle: &str,
    passphrase: &str,
    opts: &SignOptions,
) -> Result<Vec<u8>> {
    let cert = parse_certificate(cert)?;
    let key = smime_keys::unwrap(bundle, passphrase)?;
    if !key.matches(cert.tbs_certificate().subject_public_key_info()) {
        return Err(CryptoError::Input(
            "the certificate is not for this private key".into(),
        ));
    }
    let hash = opts.hash.unwrap_or_else(|| key.default_hash());
    if hash == Hash::Sha1 {
        return Err(CryptoError::Input("SHA-1 is not used for signing".into()));
    }
    let digest = hash.digest(data);
    let caps: Vec<SmimeCapability> = ADVERTISED
        .iter()
        .map(|o| SmimeCapability {
            capability_id: *o,
            parameters: None,
        })
        .collect();
    let signed_attrs: SetOfVec<Attribute> = SetOfVec::try_from(vec![
        attribute(
            oid::CONTENT_TYPE,
            Any::encode_from(&oid::ID_DATA).map_err(parse)?,
        )?,
        attribute(
            oid::SIGNING_TIME,
            Any::encode_from(&x509_time(
                opts.signing_time.unwrap_or_else(clock::now_secs),
            )?)
            .map_err(parse)?,
        )?,
        attribute(
            oid::MESSAGE_DIGEST,
            Any::new(Tag::OctetString, digest.as_slice()).map_err(parse)?,
        )?,
        attribute(
            oid::SMIME_CAPABILITIES,
            Any::encode_from(&caps).map_err(parse)?,
        )?,
    ])
    .map_err(parse)?;
    let signature = key.sign(hash, opts.pss, &signed_attrs.to_der().map_err(parse)?)?;

    let si = SignerInfo {
        version: CmsVersion::V1,
        sid: SignerIdentifier::IssuerAndSerialNumber(issuer_and_serial(&cert)),
        digest_alg: alg(hash.oid(), None),
        signed_attrs: Some(signed_attrs),
        signature_algorithm: key.signature_algorithm(hash, opts.pss)?,
        signature: OctetString::new(signature).map_err(parse)?,
        unsigned_attrs: None,
    };
    let mut certs = vec![CertificateChoices::Certificate(cert)];
    for c in &opts.chain {
        for c in path::parse_certs(c) {
            certs.push(CertificateChoices::Certificate(c));
        }
    }
    let sd = SignedData {
        version: CmsVersion::V1,
        digest_algorithms: SetOfVec::try_from(vec![alg(hash.oid(), None)]).map_err(parse)?,
        encap_content_info: EncapsulatedContentInfo {
            econtent_type: oid::ID_DATA,
            econtent: if opts.detached {
                None
            } else {
                Some(Any::new(Tag::OctetString, data).map_err(parse)?)
            },
        },
        certificates: Some(CertificateSet(SetOfVec::try_from(certs).map_err(parse)?)),
        crls: None,
        signer_infos: SignerInfos(SetOfVec::try_from(vec![si]).map_err(parse)?),
    };
    ContentInfo {
        content_type: oid::ID_SIGNED_DATA,
        content: Any::encode_from(&sd).map_err(parse)?,
    }
    .to_der()
    .map_err(parse)
}

// ── verification ─────────────────────────────────────────────────────────────

/// What a verification checks against.
#[derive(Debug, Clone)]
pub struct VerifyOptions {
    /// The signed content for a detached signature (`multipart/signed` part
    /// bytes, canonical CRLF). Ignored when the SignedData carries its content.
    pub content: Option<Vec<u8>>,
    /// Certificates known besides the embedded ones (harvested, directory).
    pub extra_certs: Vec<Vec<u8>>,
    pub trust: TrustConfig,
    /// The message's From address; `None` skips the address check.
    pub sender: Option<String>,
    /// Compare the local part case-insensitively (the default).
    pub local_case_insensitive: bool,
    pub allow_any_eku: bool,
    pub min_rsa_bits: usize,
    /// Revocation results from the server, keyed by certificate fingerprint.
    pub revocation: Vec<RevocationResult>,
    /// `hard-fail` policy: an unchecked revocation does not count as valid.
    pub hard_fail: bool,
    pub now: u64,
}

impl Default for VerifyOptions {
    fn default() -> Self {
        VerifyOptions {
            content: None,
            extra_certs: Vec::new(),
            trust: TrustConfig::bundled(),
            sender: None,
            local_case_insensitive: true,
            allow_any_eku: false,
            min_rsa_bits: 2048,
            revocation: Vec::new(),
            hard_fail: false,
            now: 0,
        }
    }
}

/// A certificate in a verified chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainCert {
    pub fingerprint: String,
    pub subject: String,
    pub issuer: String,
    pub not_before: String,
    pub not_after: String,
    pub algorithm: String,
    pub addresses: Vec<String>,
    pub cert_pem: String,
}

/// The result of [`verify`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmimeVerification {
    pub verification: Verification,
    /// Signer first, then intermediates, then the anchor when one was reached.
    pub chain: Vec<ChainCert>,
    /// `user` | `admin` | `mozilla` when the chain reached an anchor.
    pub anchor: Option<String>,
    /// What to send to the server for a revocation check (only public data).
    pub revocation_queries: Vec<RevocationQuery>,
    /// Content-encryption algorithms the signer advertised (SMIMECapabilities):
    /// `aes256-gcm`, `aes128-gcm`, `aes256-cbc`, `aes128-cbc`, or OIDs.
    pub capabilities: Vec<String>,
    /// The encapsulated content of an opaque signature.
    pub content: Option<Vec<u8>>,
    /// Every certificate the message carried, PEM (for harvesting).
    pub certificates: Vec<String>,
}

fn cert_pem(c: &Certificate) -> String {
    der::pem::encode_string(
        "CERTIFICATE",
        der::pem::LineEnding::LF,
        &c.to_der().unwrap_or_default(),
    )
    .unwrap_or_default()
}

fn chain_cert(c: &Certificate) -> ChainCert {
    let tbs = c.tbs_certificate();
    let t = |t: Time| clock::rfc3339(t.to_unix_duration().as_secs());
    ChainCert {
        fingerprint: fingerprint(c),
        subject: tbs.subject().to_string(),
        issuer: tbs.issuer().to_string(),
        not_before: t(tbs.validity().not_before),
        not_after: t(tbs.validity().not_after),
        algorithm: sig::key_algorithm(tbs.subject_public_key_info()),
        addresses: name::cert_addresses(c).effective().to_vec(),
        cert_pem: cert_pem(c),
    }
}

fn capability_name(o: &ObjectIdentifier) -> String {
    match *o {
        oid::AES256_GCM => "aes256-gcm".into(),
        oid::AES128_GCM => "aes128-gcm".into(),
        oid::AES256_CBC => "aes256-cbc".into(),
        oid::AES128_CBC => "aes128-cbc".into(),
        other => other.to_string(),
    }
}

/// The SignedData inside a DER ContentInfo.
fn signed_data(cms_der: &[u8]) -> Result<SignedData> {
    let ci = ContentInfo::from_der(cms_der).map_err(parse)?;
    if ci.content_type != oid::ID_SIGNED_DATA {
        return Err(CryptoError::Parse("not a SignedData".into()));
    }
    ci.content
        .decode_as::<SignedData>()
        .map_err(|_| CryptoError::Parse("not a SignedData".into()))
}

/// Is this a SignedData (`signed-data`), and does it carry its content?
pub fn signed_data_has_content(cms_der: &[u8]) -> Option<bool> {
    signed_data(cms_der)
        .ok()
        .map(|sd| sd.encap_content_info.econtent.is_some())
}

fn find_signer<'a>(sid: &SignerIdentifier, certs: &'a [Certificate]) -> Option<&'a Certificate> {
    certs.iter().find(|c| match sid {
        SignerIdentifier::IssuerAndSerialNumber(ias) => {
            c.tbs_certificate().serial_number() == &ias.serial_number
                && name_eq(c.tbs_certificate().issuer(), &ias.issuer)
        }
        SignerIdentifier::SubjectKeyIdentifier(ski) => ski_of(c).as_ref() == Some(ski),
    })
}

fn ski_of(c: &Certificate) -> Option<SubjectKeyIdentifier> {
    match c.tbs_certificate().get_extension::<SubjectKeyIdentifier>() {
        Ok(Some((_, ski))) => Some(ski),
        _ => None,
    }
}

/// Verify a CMS SignedData (DER).
pub fn verify(cms_der: &[u8], opts: &VerifyOptions) -> Result<SmimeVerification> {
    let sd = signed_data(cms_der)?;
    let now = if opts.now == 0 {
        clock::now_secs()
    } else {
        opts.now
    };
    let embedded: Vec<Certificate> = sd
        .certificates
        .iter()
        .flat_map(|set| set.0.iter())
        .filter_map(|c| match c {
            CertificateChoices::Certificate(c) => Some(c.clone()),
            _ => None,
        })
        .collect();
    let mut known = embedded.clone();
    for c in &opts.extra_certs {
        known.extend(path::parse_certs(c));
    }
    let econtent = sd
        .encap_content_info
        .econtent
        .as_ref()
        .map(|a| a.value().to_vec());
    let content = match (&econtent, &opts.content) {
        (Some(c), _) => c.clone(),
        (None, Some(c)) => c.clone(),
        (None, None) => {
            return Err(CryptoError::Input(
                "a detached signature needs the signed content".into(),
            ));
        }
    };

    let mut best: Option<SmimeVerification> = None;
    for si in sd.signer_infos.0.iter() {
        let v = verify_signer(si, &sd, &content, &known, opts, now)?;
        best = Some(match best {
            None => v,
            Some(b) if rank(&v.verification.state) > rank(&b.verification.state) => v,
            Some(b) => b,
        });
    }
    let mut out = best.unwrap_or_else(|| empty_result(Verification::not_signed("smime")));
    out.content = econtent;
    out.certificates = embedded.iter().map(cert_pem).collect();
    Ok(out)
}

fn rank(state: &str) -> u8 {
    match state {
        verdict::VALID => 9,
        verdict::REVOCATION_UNCHECKED => 8,
        verdict::ADDRESS_MISMATCH => 7,
        verdict::UNTRUSTED => 6,
        verdict::KEY_EXPIRED | verdict::NOT_YET_VALID => 5,
        verdict::UNKNOWN_KEY => 3,
        verdict::REVOKED => 2,
        verdict::BAD_SIGNATURE => 1,
        _ => 0,
    }
}

fn empty_result(v: Verification) -> SmimeVerification {
    SmimeVerification {
        verification: v,
        chain: Vec::new(),
        anchor: None,
        revocation_queries: Vec::new(),
        capabilities: Vec::new(),
        content: None,
        certificates: Vec::new(),
    }
}

fn verify_signer(
    si: &SignerInfo,
    sd: &SignedData,
    content: &[u8],
    known: &[Certificate],
    opts: &VerifyOptions,
    now: u64,
) -> Result<SmimeVerification> {
    let mut v = Verification::new("smime", verdict::UNKNOWN_KEY);
    let Some(signer) = find_signer(&si.sid, known).cloned() else {
        return Ok(empty_result(v));
    };
    let tbs = signer.tbs_certificate();
    let fp = fingerprint(&signer);
    v.signer_fingerprint = Some(fp.clone());
    v.signer_addresses = name::cert_addresses(&signer).effective().to_vec();
    v.signature.signer_key_id = Some(fp[..16].to_string());
    v.signature.algorithm = Some(sig::key_algorithm(tbs.subject_public_key_info()));
    v.signature.key_created_at = Some(clock::rfc3339(
        tbs.validity().not_before.to_unix_duration().as_secs(),
    ));
    v.signature.key_expires_at = Some(clock::rfc3339(
        tbs.validity().not_after.to_unix_duration().as_secs(),
    ));
    let mut result = empty_result(v);

    // Signed attributes: messageDigest, contentType, signingTime, capabilities.
    let Some(hash) = Hash::from_oid(&si.digest_alg.oid) else {
        result.verification = result
            .verification
            .with(verdict::BAD_SIGNATURE, Some("unsupported-algorithm"));
        return Ok(result);
    };
    if hash == Hash::Sha1 {
        result.verification = result
            .verification
            .with(verdict::BAD_SIGNATURE, Some("weak-hash"));
        return Ok(result);
    }
    let mut signing_time = None;
    let signed_message = match &si.signed_attrs {
        Some(attrs) => {
            let first = |o: ObjectIdentifier| {
                attrs
                    .iter()
                    .find(|a| a.oid == o)
                    .and_then(|a| a.values.as_slice().first())
            };
            let md_ok = first(oid::MESSAGE_DIGEST)
                .and_then(|v| v.decode_as::<OctetString>().ok())
                .is_some_and(|md| md.as_bytes() == hash.digest(content).as_slice());
            let ct_ok = first(oid::CONTENT_TYPE)
                .and_then(|v| v.decode_as::<ObjectIdentifier>().ok())
                .is_some_and(|ct| ct == sd.encap_content_info.econtent_type);
            if !md_ok || !ct_ok {
                result.verification = result
                    .verification
                    .with(verdict::BAD_SIGNATURE, Some("tampered"));
                return Ok(result);
            }
            signing_time = first(oid::SIGNING_TIME)
                .and_then(|v| v.decode_as::<Time>().ok())
                .map(|t| t.to_unix_duration().as_secs());
            if let Some(caps) = first(oid::SMIME_CAPABILITIES)
                .and_then(|v| v.decode_as::<Vec<SmimeCapability>>().ok())
            {
                result.capabilities = caps
                    .iter()
                    .map(|c| capability_name(&c.capability_id))
                    .collect();
            }
            SetOfVec::from_iter(attrs.iter().cloned())
                .map_err(parse)?
                .to_der()
                .map_err(parse)?
        }
        None => content.to_vec(),
    };
    result.verification.signed_at = signing_time.map(clock::rfc3339);

    match sig::verify(
        tbs.subject_public_key_info(),
        &si.signature_algorithm,
        Some(hash),
        &signed_message,
        si.signature.as_bytes(),
        Use::Message,
    ) {
        Ok(()) => {}
        Err(sig::SigError::Weak) => {
            result.verification = result
                .verification
                .with(verdict::BAD_SIGNATURE, Some("weak-hash"));
            return Ok(result);
        }
        Err(sig::SigError::Unsupported(_)) => {
            result.verification = result
                .verification
                .with(verdict::BAD_SIGNATURE, Some("unsupported-algorithm"));
            return Ok(result);
        }
        Err(_) => {
            result.verification = result
                .verification
                .with(verdict::BAD_SIGNATURE, Some("tampered"));
            return Ok(result);
        }
    }

    // signingTime is an assertion by the signer, not a trusted timestamp.
    // Validate at the caller's clock; backdating must not bypass expiration.
    let at = now;
    let anchors = opts.trust.anchors();
    let intermediates: Vec<Certificate> = known.iter().filter(|c| *c != &signer).cloned().collect();
    let validated = path::validate(
        &signer,
        &intermediates,
        &anchors,
        &Policy {
            purpose: Purpose::Sign,
            allow_any_eku: opts.allow_any_eku,
            min_rsa_bits: opts.min_rsa_bits,
            at,
        },
    );
    result.chain = validated.path.iter().map(chain_cert).collect();
    if let Some((anchor, source)) = &validated.anchor {
        if fingerprint(anchor) != fp {
            result.chain.push(chain_cert(anchor));
        }
        result.anchor = Some(source.token().into());
    }

    // Revocation queries for every certificate below the anchor.
    let mut issuers: Vec<Certificate> = validated.path.iter().skip(1).cloned().collect();
    if let Some((anchor, _)) = &validated.anchor {
        issuers.push(anchor.clone());
    }
    for (c, issuer) in validated.path.iter().zip(issuers.iter()) {
        if fingerprint(c) != fingerprint(issuer) {
            result.revocation_queries.push(query_for(c, issuer));
        }
    }
    let revocation = revocation_state(&result.revocation_queries, &opts.revocation);
    result.verification.signature.revocation_status = Some(revocation.into());

    let chain_status = match &validated.error {
        None => "trusted",
        Some(PathError::Expired(_)) | Some(PathError::NotYetValid(_)) => "expired",
        Some(_) => "untrusted",
    };
    result.verification.signature.chain_status = Some(chain_status.into());

    // Precedence: revoked > expiry > untrusted > address > revocation unchecked.
    let state_detail: (&str, Option<&str>) = if revocation == "revoked" {
        (verdict::REVOKED, None)
    } else {
        match &validated.error {
            Some(PathError::Expired(_)) => (verdict::KEY_EXPIRED, None),
            Some(PathError::NotYetValid(_)) => (verdict::NOT_YET_VALID, None),
            Some(e) => (verdict::UNTRUSTED, Some(e.token())),
            None => {
                let addr_ok = opts
                    .sender
                    .as_deref()
                    .is_none_or(|s| name::matches_sender(&signer, s, opts.local_case_insensitive));
                if !addr_ok {
                    (verdict::ADDRESS_MISMATCH, None)
                } else if revocation == "good" {
                    (verdict::VALID, None)
                } else if opts.hard_fail {
                    (verdict::REVOCATION_UNCHECKED, Some("hard-fail"))
                } else {
                    (verdict::REVOCATION_UNCHECKED, None)
                }
            }
        }
    };
    result.verification = result.verification.with(state_detail.0, state_detail.1);
    if state_detail.1 == Some("hard-fail") {
        // Under hard-fail an unchecked revocation is not a valid signature.
        result.verification.signature.status = "unverified-key".into();
    }
    Ok(result)
}

/// `good` when every certificate checked good, `revoked` when any is revoked,
/// else `unknown`.
fn revocation_state(queries: &[RevocationQuery], results: &[RevocationResult]) -> &'static str {
    let mut all_good = !queries.is_empty();
    for q in queries {
        match results
            .iter()
            .find(|r| r.cert_fingerprint.eq_ignore_ascii_case(&q.cert_fingerprint))
            .map(|r| r.status.as_str())
        {
            Some("revoked") => return "revoked",
            Some("good") => {}
            _ => all_good = false,
        }
    }
    if all_good { "good" } else { "unknown" }
}

// ── encryption ───────────────────────────────────────────────────────────────

/// Content encryption choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Cipher {
    /// AES-256-GCM when every recipient is known to read it, else AES-256-CBC.
    #[default]
    Auto,
    Cbc,
    Gcm,
}

/// One recipient.
#[derive(Debug, Clone)]
pub struct Recipient {
    /// The recipient's certificate, PEM or DER.
    pub cert: Vec<u8>,
    /// The recipient advertised AES-GCM (SMIMECapabilities from their signed
    /// mail); `None` when unknown.
    pub gcm: Option<bool>,
}

/// Encrypted content and its S/MIME type.
#[derive(Debug, Clone)]
pub struct Encrypted {
    pub der: Vec<u8>,
    /// `enveloped-data` or `authEnveloped-data`.
    pub smime_type: &'static str,
}

/// `GCMParameters ::= SEQUENCE { aes-nonce OCTET STRING, aes-ICVlen INTEGER
/// DEFAULT 12 }` (RFC 5084 §3.2).
#[derive(der::Sequence)]
pub(crate) struct GcmParameters {
    pub nonce: OctetString,
    #[asn1(default = "default_icv")]
    pub icv_len: u32,
}

fn default_icv() -> u32 {
    12
}

/// `ECC-CMS-SharedInfo` (RFC 5753 §7.2).
#[derive(der::Sequence)]
struct EccCmsSharedInfo {
    key_info: AlgorithmIdentifierOwned,
    #[asn1(context_specific = "0", tag_mode = "EXPLICIT", optional = "true")]
    entity_u_info: Option<OctetString>,
    #[asn1(context_specific = "2", tag_mode = "EXPLICIT")]
    supp_pub_info: OctetString,
}

/// ANSI X9.63 KDF (SEC 1 §3.6.1) with `hash`, `len` octets.
fn x963_kdf(hash: Hash, z: &[u8], shared_info: &[u8], len: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(len + 64);
    let mut counter: u32 = 1;
    while out.len() < len {
        let mut input = Vec::with_capacity(z.len() + 4 + shared_info.len());
        input.extend_from_slice(z);
        input.extend_from_slice(&counter.to_be_bytes());
        input.extend_from_slice(shared_info);
        out.extend_from_slice(&hash.digest(&input));
        counter += 1;
    }
    out.truncate(len);
    out
}

/// The KEK for a key-agreement recipient (RFC 5753 §3.1.1 / RFC 8418 §2).
fn kari_kek(
    kdf_hash: Hash,
    wrap_alg: ObjectIdentifier,
    z: &[u8],
    ukm: Option<&[u8]>,
) -> Result<Vec<u8>> {
    let key_len = match wrap_alg {
        oid::AES128_WRAP => 16,
        oid::AES256_WRAP => 32,
        other => {
            return Err(CryptoError::Decrypt(format!(
                "unsupported key wrap {other}"
            )));
        }
    };
    let info = EccCmsSharedInfo {
        key_info: alg(wrap_alg, None),
        entity_u_info: ukm
            .map(|u| OctetString::new(u.to_vec()))
            .transpose()
            .map_err(parse)?,
        supp_pub_info: OctetString::new(((key_len * 8) as u32).to_be_bytes().to_vec())
            .map_err(parse)?,
    }
    .to_der()
    .map_err(parse)?;
    Ok(x963_kdf(kdf_hash, z, &info, key_len))
}

fn aes_wrap(kek: &[u8], cek: &[u8]) -> Result<Vec<u8>> {
    use aes::cipher::KeyInit;
    let mut buf = vec![0u8; cek.len() + 8];
    match kek.len() {
        32 => aes_kw::KwAes256::new_from_slice(kek)
            .map_err(|_| CryptoError::Encrypt("bad KEK".into()))?
            .wrap_key(cek, &mut buf)
            .map_err(|e| CryptoError::Encrypt(e.to_string()))?,
        16 => aes_kw::KwAes128::new_from_slice(kek)
            .map_err(|_| CryptoError::Encrypt("bad KEK".into()))?
            .wrap_key(cek, &mut buf)
            .map_err(|e| CryptoError::Encrypt(e.to_string()))?,
        _ => return Err(CryptoError::Encrypt("bad KEK length".into())),
    };
    Ok(buf)
}

fn kdf_for_curve(curve: Option<ObjectIdentifier>) -> (ObjectIdentifier, Hash) {
    match curve {
        Some(oid::SECP384R1) => (oid::DH_SINGLE_PASS_STD_SHA384KDF, Hash::Sha384),
        _ => (oid::DH_SINGLE_PASS_STD_SHA256KDF, Hash::Sha256),
    }
}

/// The recipient info for one certificate and content key.
fn recipient_info(cert: &Certificate, cek: &[u8]) -> Result<RecipientInfo> {
    let spki = cert.tbs_certificate().subject_public_key_info();
    match spki.algorithm.oid {
        oid::RSA_ENCRYPTION => {
            use spki::DecodePublicKey;
            let key = rsa::RsaPublicKey::from_public_key_der(&spki.to_der().map_err(parse)?)
                .map_err(|e| CryptoError::Encrypt(e.to_string()))?;
            let enc = key
                .encrypt(&mut rng::rc10(), rsa::Oaep::<sha2::Sha256>::new(), cek)
                .map_err(|e| CryptoError::Encrypt(e.to_string()))?;
            #[derive(der::Sequence)]
            struct Params {
                #[asn1(context_specific = "0", tag_mode = "EXPLICIT")]
                hash: AlgorithmIdentifierOwned,
                #[asn1(context_specific = "1", tag_mode = "EXPLICIT")]
                mgf: AlgorithmIdentifierOwned,
            }
            let hash = alg(oid::SHA256, Some(Any::null()));
            let parameters = Params {
                hash: hash.clone(),
                mgf: alg(oid::MGF1, Some(Any::encode_from(&hash).map_err(parse)?)),
            };
            Ok(RecipientInfo::Ktri(KeyTransRecipientInfo {
                version: CmsVersion::V0,
                rid: RecipientIdentifier::IssuerAndSerialNumber(issuer_and_serial(cert)),
                key_enc_alg: alg(
                    oid::RSAES_OAEP,
                    Some(Any::encode_from(&parameters).map_err(parse)?),
                ),
                enc_key: OctetString::new(enc).map_err(parse)?,
            }))
        }
        oid::EC_PUBLIC_KEY | oid::X25519 => {
            let curve = sig::curve_of(spki);
            let (z, originator) = match (spki.algorithm.oid, curve) {
                (oid::X25519, _) => {
                    let peer: [u8; 32] = spki
                        .subject_public_key
                        .as_bytes()
                        .and_then(|b| b.try_into().ok())
                        .ok_or_else(|| CryptoError::Encrypt("bad X25519 key".into()))?;
                    let mut seed = [0u8; 32];
                    rng::fill_random(&mut seed);
                    let eph = x25519_dalek::StaticSecret::from(seed);
                    let z = eph.diffie_hellman(&x25519_dalek::PublicKey::from(peer));
                    if !z.was_contributory() {
                        return Err(CryptoError::Encrypt(
                            "noncontributory X25519 recipient".into(),
                        ));
                    }
                    let public = x25519_dalek::PublicKey::from(&eph);
                    (
                        z.as_bytes().to_vec(),
                        OriginatorPublicKey {
                            algorithm: alg(oid::X25519, None),
                            public_key: BitString::from_bytes(public.as_bytes()).map_err(parse)?,
                        },
                    )
                }
                (_, Some(oid::SECP256R1)) => {
                    let peer =
                        p256::PublicKey::from_sec1_bytes(spki.subject_public_key.raw_bytes())
                            .map_err(|_| CryptoError::Encrypt("bad P-256 key".into()))?;
                    let eph: p256::ecdh::EphemeralSecret =
                        p256::elliptic_curve::Generate::generate_from_rng(&mut rng::rc10());
                    let z = eph.diffie_hellman(&peer);
                    let public = eph.public_key().to_sec1_bytes();
                    (
                        z.raw_secret_bytes().to_vec(),
                        OriginatorPublicKey {
                            algorithm: alg(oid::EC_PUBLIC_KEY, None),
                            public_key: BitString::from_bytes(&public).map_err(parse)?,
                        },
                    )
                }
                (_, Some(oid::SECP384R1)) => {
                    let peer =
                        p384::PublicKey::from_sec1_bytes(spki.subject_public_key.raw_bytes())
                            .map_err(|_| CryptoError::Encrypt("bad P-384 key".into()))?;
                    let eph: p384::ecdh::EphemeralSecret =
                        p384::elliptic_curve::Generate::generate_from_rng(&mut rng::rc10());
                    let z = eph.diffie_hellman(&peer);
                    let public = eph.public_key().to_sec1_bytes();
                    (
                        z.raw_secret_bytes().to_vec(),
                        OriginatorPublicKey {
                            algorithm: alg(oid::EC_PUBLIC_KEY, None),
                            public_key: BitString::from_bytes(&public).map_err(parse)?,
                        },
                    )
                }
                _ => return Err(CryptoError::Encrypt("unsupported recipient curve".into())),
            };
            let (kdf_oid, kdf_hash) = kdf_for_curve(curve);
            let kek = kari_kek(kdf_hash, oid::AES256_WRAP, &z, None)?;
            let wrapped = aes_wrap(&kek, cek)?;
            Ok(RecipientInfo::Kari(KeyAgreeRecipientInfo {
                version: CmsVersion::V3,
                originator: OriginatorIdentifierOrKey::OriginatorKey(originator),
                ukm: None,
                key_enc_alg: alg(
                    kdf_oid,
                    Some(Any::encode_from(&alg(oid::AES256_WRAP, None)).map_err(parse)?),
                ),
                recipient_enc_keys: vec![RecipientEncryptedKey {
                    rid: KeyAgreeRecipientIdentifier::IssuerAndSerialNumber(issuer_and_serial(
                        cert,
                    )),
                    enc_key: OctetString::new(wrapped).map_err(parse)?,
                }],
            }))
        }
        other => Err(CryptoError::Encrypt(format!(
            "unsupported recipient key algorithm {other}"
        ))),
    }
}

/// Encrypt `plaintext` to `recipients`. Returns DER `ContentInfo` of
/// AuthEnvelopedData (AES-256-GCM) or EnvelopedData (AES-256-CBC).
pub fn encrypt(plaintext: &[u8], recipients: &[Recipient], cipher: Cipher) -> Result<Encrypted> {
    if recipients.is_empty() {
        return Err(CryptoError::Input("no recipients".into()));
    }
    let certs: Vec<Certificate> = recipients
        .iter()
        .map(|r| parse_certificate(&r.cert))
        .collect::<Result<_>>()?;
    for cert in &certs {
        if let Some(error) = path::check_leaf(
            cert,
            &Policy {
                purpose: Purpose::Encrypt,
                at: clock::now_secs(),
                ..Policy::default()
            },
        ) {
            return Err(CryptoError::Encrypt(format!(
                "recipient certificate: {}",
                error.token()
            )));
        }
    }
    let gcm = match cipher {
        Cipher::Gcm => true,
        Cipher::Cbc => false,
        Cipher::Auto => recipients.iter().all(|r| r.gcm == Some(true)),
    };
    let mut cek = [0u8; 32];
    rng::fill_random(&mut cek);
    let infos: Vec<RecipientInfo> = certs
        .iter()
        .map(|c| recipient_info(c, &cek))
        .collect::<Result<_>>()?;
    let any_kari = infos.iter().any(|i| matches!(i, RecipientInfo::Kari(_)));
    let recip_infos = RecipientInfos(SetOfVec::try_from(infos).map_err(parse)?);

    let out = if gcm {
        use aes_gcm::aead::{Aead, KeyInit};
        use aes_gcm::{Aes256Gcm, Nonce};
        let mut nonce = [0u8; 12];
        rng::fill_random(&mut nonce);
        let mut sealed = Aes256Gcm::new((&cek).into())
            .encrypt(Nonce::from_slice(&nonce), plaintext)
            .map_err(|_| CryptoError::Encrypt("aes-gcm seal".into()))?;
        let tag = sealed.split_off(sealed.len() - 16);
        let aed = cms::authenveloped_data::AuthEnvelopedData {
            version: CmsVersion::V0,
            originator_info: None,
            recip_infos,
            auth_encrypted_content_info: EncryptedContentInfo {
                content_type: oid::ID_DATA,
                content_enc_alg: alg(
                    oid::AES256_GCM,
                    Some(
                        Any::encode_from(&GcmParameters {
                            nonce: OctetString::new(nonce.to_vec()).map_err(parse)?,
                            icv_len: 16,
                        })
                        .map_err(parse)?,
                    ),
                ),
                encrypted_content: Some(OctetString::new(sealed).map_err(parse)?),
            },
            auth_attrs: None,
            mac: OctetString::new(tag).map_err(parse)?,
            unauth_attrs: None,
        };
        Encrypted {
            der: ContentInfo {
                content_type: oid::ID_AUTH_ENVELOPED_DATA,
                content: Any::encode_from(&aed).map_err(parse)?,
            }
            .to_der()
            .map_err(parse)?,
            smime_type: "authEnveloped-data",
        }
    } else {
        let mut iv = [0u8; 16];
        rng::fill_random(&mut iv);
        let ct = aes_cbc_encrypt(&cek, &iv, plaintext)?;
        let ed = EnvelopedData {
            version: if any_kari {
                CmsVersion::V2
            } else {
                CmsVersion::V0
            },
            originator_info: None,
            recip_infos,
            encrypted_content: EncryptedContentInfo {
                content_type: oid::ID_DATA,
                content_enc_alg: alg(
                    oid::AES256_CBC,
                    Some(Any::new(Tag::OctetString, &iv[..]).map_err(parse)?),
                ),
                encrypted_content: Some(OctetString::new(ct).map_err(parse)?),
            },
            unprotected_attrs: None,
        };
        Encrypted {
            der: ContentInfo {
                content_type: oid::ID_ENVELOPED_DATA,
                content: Any::encode_from(&ed).map_err(parse)?,
            }
            .to_der()
            .map_err(parse)?,
            smime_type: "enveloped-data",
        }
    };
    zeroize::Zeroize::zeroize(&mut cek);
    Ok(out)
}

fn aes_cbc_encrypt(key: &[u8], iv: &[u8], pt: &[u8]) -> Result<Vec<u8>> {
    use cbc::cipher::{BlockModeEncrypt, KeyIvInit, block_padding::Pkcs7};
    let enc = cbc::Encryptor::<aes::Aes256>::new_from_slices(key, iv)
        .map_err(|e| CryptoError::Encrypt(e.to_string()))?;
    Ok(enc.encrypt_padded_vec::<Pkcs7>(pt))
}

// ── decryption (client only, DA48) ───────────────────────────────────────────

/// Decrypt an EnvelopedData or AuthEnvelopedData (DER `ContentInfo`) with the key
/// in `bundle`. `cert` (PEM or DER) selects the recipient info addressed to it;
/// without it every recipient info is tried.
#[cfg(any(target_arch = "wasm32", feature = "client-private-ops"))]
pub fn decrypt(
    der_bytes: &[u8],
    cert: Option<&[u8]>,
    bundle: &str,
    passphrase: &str,
) -> Result<Vec<u8>> {
    let key = smime_keys::unwrap(bundle, passphrase)?;
    let cert = cert.map(parse_certificate).transpose()?;
    let ci = ContentInfo::from_der(der_bytes).map_err(parse)?;
    if ci.content_type == oid::ID_AUTH_ENVELOPED_DATA {
        let aed = ci
            .content
            .decode_as::<cms::authenveloped_data::AuthEnvelopedData>()
            .map_err(|_| CryptoError::Parse("not an AuthEnvelopedData".into()))?;
        let cek = unwrap_cek(&aed.recip_infos, cert.as_ref(), &key)?;
        let eci = &aed.auth_encrypted_content_info;
        let ct = eci
            .encrypted_content
            .as_ref()
            .ok_or_else(|| CryptoError::Decrypt("no encrypted content".into()))?;
        decrypt_gcm(&cek, eci, ct.as_bytes(), aed.mac.as_bytes())
    } else if ci.content_type == oid::ID_ENVELOPED_DATA {
        let ed = ci
            .content
            .decode_as::<EnvelopedData>()
            .map_err(|_| CryptoError::Parse("not an EnvelopedData".into()))?;
        let cek = unwrap_cek(&ed.recip_infos, cert.as_ref(), &key)?;
        let eci = &ed.encrypted_content;
        let ct = eci
            .encrypted_content
            .as_ref()
            .ok_or_else(|| CryptoError::Decrypt("no encrypted content".into()))?;
        match eci.content_enc_alg.oid {
            oid::AES256_CBC | oid::AES128_CBC => {
                let iv = eci
                    .content_enc_alg
                    .parameters
                    .as_ref()
                    .ok_or_else(|| CryptoError::Decrypt("no content IV".into()))?
                    .decode_as::<OctetString>()
                    .map_err(parse)?;
                aes_cbc_decrypt(&cek, iv.as_bytes(), ct.as_bytes())
            }
            // An EnvelopedData with GCM content is not standard, but some
            // implementations send it; the tag is then appended to the content.
            other => Err(CryptoError::Decrypt(format!(
                "unsupported content cipher {other}"
            ))),
        }
    } else {
        Err(CryptoError::Parse("not an enveloped message".into()))
    }
}

#[cfg(any(target_arch = "wasm32", feature = "client-private-ops"))]
fn decrypt_gcm(cek: &[u8], eci: &EncryptedContentInfo, ct: &[u8], tag: &[u8]) -> Result<Vec<u8>> {
    use aes_gcm::aead::{Aead, KeyInit};
    use aes_gcm::{Aes128Gcm, Aes256Gcm, Nonce};
    let params: GcmParameters = eci
        .content_enc_alg
        .parameters
        .as_ref()
        .ok_or_else(|| CryptoError::Decrypt("no GCM parameters".into()))?
        .decode_as()
        .map_err(parse)?;
    if params.nonce.as_bytes().len() != 12
        || tag.len() != params.icv_len as usize
        || tag.len() != 16
    {
        return Err(CryptoError::Decrypt(
            "unsupported GCM nonce or tag length".into(),
        ));
    }
    let mut sealed = ct.to_vec();
    sealed.extend_from_slice(tag);
    let nonce = Nonce::from_slice(params.nonce.as_bytes());
    let opened = match (eci.content_enc_alg.oid, cek.len()) {
        (oid::AES256_GCM, 32) => Aes256Gcm::new_from_slice(cek)
            .map_err(|_| CryptoError::Decrypt("bad key".into()))?
            .decrypt(nonce, sealed.as_slice()),
        (oid::AES128_GCM, 16) => Aes128Gcm::new_from_slice(cek)
            .map_err(|_| CryptoError::Decrypt("bad key".into()))?
            .decrypt(nonce, sealed.as_slice()),
        _ => {
            return Err(CryptoError::Decrypt(
                "content cipher and key do not match".into(),
            ));
        }
    };
    opened.map_err(|_| CryptoError::Decrypt("aes-gcm open (bad key, tag or ciphertext)".into()))
}

#[cfg(any(target_arch = "wasm32", feature = "client-private-ops"))]
fn aes_cbc_decrypt(key: &[u8], iv: &[u8], ct: &[u8]) -> Result<Vec<u8>> {
    use cbc::cipher::{BlockModeDecrypt, KeyIvInit, block_padding::Pkcs7};
    if iv.len() != 16 {
        return Err(CryptoError::Decrypt("bad AES-CBC IV length".into()));
    }
    let out = match key.len() {
        32 => cbc::Decryptor::<aes::Aes256>::new_from_slices(key, iv)
            .map_err(|e| CryptoError::Decrypt(e.to_string()))?
            .decrypt_padded_vec::<Pkcs7>(ct),
        16 => cbc::Decryptor::<aes::Aes128>::new_from_slices(key, iv)
            .map_err(|e| CryptoError::Decrypt(e.to_string()))?
            .decrypt_padded_vec::<Pkcs7>(ct),
        _ => return Err(CryptoError::Decrypt("bad AES-CBC key length".into())),
    };
    out.map_err(|e| CryptoError::Decrypt(e.to_string()))
}

#[cfg(any(target_arch = "wasm32", feature = "client-private-ops"))]
fn ias_matches(ias: &IssuerAndSerialNumber, cert: Option<&Certificate>) -> bool {
    cert.is_none_or(|c| {
        c.tbs_certificate().serial_number() == &ias.serial_number
            && name_eq(c.tbs_certificate().issuer(), &ias.issuer)
    })
}

/// Recover the content-encryption key from the recipient infos.
#[cfg(any(target_arch = "wasm32", feature = "client-private-ops"))]
fn unwrap_cek(
    infos: &RecipientInfos,
    cert: Option<&Certificate>,
    key: &PrivateKey,
) -> Result<Vec<u8>> {
    for ri in infos.0.iter() {
        match (ri, key) {
            (RecipientInfo::Ktri(k), PrivateKey::Rsa(rsa_key)) => {
                let addressed = match &k.rid {
                    RecipientIdentifier::IssuerAndSerialNumber(ias) => ias_matches(ias, cert),
                    RecipientIdentifier::SubjectKeyIdentifier(ski) => {
                        cert.is_none_or(|c| ski_of(c).as_ref() == Some(ski))
                    }
                };
                if !addressed {
                    continue;
                }
                if let Ok(cek) = ktri_decrypt(rsa_key, k) {
                    return Ok(cek);
                }
            }
            (
                RecipientInfo::Kari(k),
                PrivateKey::P256(_) | PrivateKey::P384(_) | PrivateKey::X25519(_),
            ) => {
                for rek in &k.recipient_enc_keys {
                    let addressed = match &rek.rid {
                        KeyAgreeRecipientIdentifier::IssuerAndSerialNumber(ias) => {
                            ias_matches(ias, cert)
                        }
                        KeyAgreeRecipientIdentifier::RKeyId(rk) => cert
                            .is_none_or(|c| ski_of(c).as_ref() == Some(&rk.subject_key_identifier)),
                    };
                    if addressed && let Ok(cek) = kari_decrypt(key, k, rek) {
                        return Ok(cek);
                    }
                }
            }
            _ => {}
        }
    }
    Err(CryptoError::Decrypt(
        "no recipient info this key can open".into(),
    ))
}

#[cfg(any(target_arch = "wasm32", feature = "client-private-ops"))]
fn ktri_decrypt(key: &rsa::RsaPrivateKey, k: &KeyTransRecipientInfo) -> Result<Vec<u8>> {
    let ct = k.enc_key.as_bytes();
    match k.key_enc_alg.oid {
        oid::RSA_ENCRYPTION => key
            .decrypt(rsa::Pkcs1v15Encrypt, ct)
            .map_err(|e| CryptoError::Decrypt(e.to_string())),
        oid::RSAES_OAEP => {
            // RSAES-OAEP-params (RFC 4055 §4.1): hash [0] DEFAULT sha1, MGF1 with
            // the same hash, label empty.
            let hash = oaep_hash(&k.key_enc_alg)?;
            let r = match hash {
                Hash::Sha1 => key.decrypt(rsa::Oaep::<sha1::Sha1>::new(), ct),
                Hash::Sha256 => key.decrypt(rsa::Oaep::<sha2::Sha256>::new(), ct),
                Hash::Sha384 => key.decrypt(rsa::Oaep::<sha2::Sha384>::new(), ct),
                Hash::Sha512 => key.decrypt(rsa::Oaep::<sha2::Sha512>::new(), ct),
            };
            r.map_err(|e| CryptoError::Decrypt(e.to_string()))
        }
        other => Err(CryptoError::Decrypt(format!(
            "unsupported key transport {other}"
        ))),
    }
}

#[cfg(any(target_arch = "wasm32", feature = "client-private-ops"))]
fn oaep_hash(a: &AlgorithmIdentifierOwned) -> Result<Hash> {
    #[derive(der::Sequence)]
    struct OaepParams {
        #[asn1(context_specific = "0", tag_mode = "EXPLICIT", optional = "true")]
        hash: Option<AlgorithmIdentifierOwned>,
        #[asn1(context_specific = "1", tag_mode = "EXPLICIT", optional = "true")]
        mgf: Option<AlgorithmIdentifierOwned>,
        #[asn1(context_specific = "2", tag_mode = "EXPLICIT", optional = "true")]
        label: Option<AlgorithmIdentifierOwned>,
    }
    let Some(p) = &a.parameters else {
        return Ok(Hash::Sha1);
    };
    let p: OaepParams = p.decode_as().map_err(parse)?;
    let hash = match &p.hash {
        Some(h) => Hash::from_oid(&h.oid)
            .ok_or_else(|| CryptoError::Decrypt("unsupported OAEP hash".into()))?,
        None => Hash::Sha1,
    };
    let mgf_hash = match &p.mgf {
        Some(m) if m.oid == oid::MGF1 => m
            .parameters
            .as_ref()
            .and_then(|x| x.decode_as::<AlgorithmIdentifierOwned>().ok())
            .and_then(|x| Hash::from_oid(&x.oid)),
        Some(_) => None,
        None => Some(Hash::Sha1),
    };
    if mgf_hash != Some(hash) || p.label.is_some() {
        return Err(CryptoError::Decrypt("unsupported OAEP parameters".into()));
    }
    Ok(hash)
}

#[cfg(any(target_arch = "wasm32", feature = "client-private-ops"))]
fn kari_decrypt(
    key: &PrivateKey,
    k: &KeyAgreeRecipientInfo,
    rek: &RecipientEncryptedKey,
) -> Result<Vec<u8>> {
    let OriginatorIdentifierOrKey::OriginatorKey(orig) = &k.originator else {
        return Err(CryptoError::Decrypt(
            "key agreement without an originator key".into(),
        ));
    };
    let point = orig.public_key.raw_bytes();
    let z = match key {
        PrivateKey::P256(sk) => {
            let peer = p256::PublicKey::from_sec1_bytes(point)
                .map_err(|_| CryptoError::Decrypt("bad originator key".into()))?;
            p256::ecdh::diffie_hellman(sk.to_nonzero_scalar(), peer.as_affine())
                .raw_secret_bytes()
                .to_vec()
        }
        PrivateKey::P384(sk) => {
            let peer = p384::PublicKey::from_sec1_bytes(point)
                .map_err(|_| CryptoError::Decrypt("bad originator key".into()))?;
            p384::ecdh::diffie_hellman(sk.to_nonzero_scalar(), peer.as_affine())
                .raw_secret_bytes()
                .to_vec()
        }
        PrivateKey::X25519(sk) => {
            let peer: [u8; 32] = point
                .try_into()
                .map_err(|_| CryptoError::Decrypt("bad originator key".into()))?;
            let shared = sk.diffie_hellman(&x25519_dalek::PublicKey::from(peer));
            if !shared.was_contributory() {
                return Err(CryptoError::Decrypt(
                    "noncontributory X25519 originator".into(),
                ));
            }
            shared.as_bytes().to_vec()
        }
        _ => return Err(CryptoError::Decrypt("key cannot do key agreement".into())),
    };
    let kdf_hash = match k.key_enc_alg.oid {
        oid::DH_SINGLE_PASS_STD_SHA1KDF => Hash::Sha1,
        oid::DH_SINGLE_PASS_STD_SHA256KDF => Hash::Sha256,
        oid::DH_SINGLE_PASS_STD_SHA384KDF => Hash::Sha384,
        oid::DH_SINGLE_PASS_STD_SHA512KDF => Hash::Sha512,
        other => {
            return Err(CryptoError::Decrypt(format!(
                "unsupported key agreement {other}"
            )));
        }
    };
    let wrap: AlgorithmIdentifierOwned = k
        .key_enc_alg
        .parameters
        .as_ref()
        .ok_or_else(|| CryptoError::Decrypt("no key wrap algorithm".into()))?
        .decode_as()
        .map_err(parse)?;
    let kek = kari_kek(kdf_hash, wrap.oid, &z, k.ukm.as_ref().map(|u| u.as_bytes()))?;
    aes_unwrap(&kek, rek.enc_key.as_bytes())
}

#[cfg(any(target_arch = "wasm32", feature = "client-private-ops"))]
fn aes_unwrap(kek: &[u8], wrapped: &[u8]) -> Result<Vec<u8>> {
    use aes::cipher::KeyInit;
    if wrapped.len() < 16 {
        return Err(CryptoError::Decrypt("wrapped key too short".into()));
    }
    let mut buf = vec![0u8; wrapped.len() - 8];
    let bad = |_| CryptoError::Decrypt("key unwrap failed".into());
    match kek.len() {
        32 => aes_kw::KwAes256::new_from_slice(kek)
            .map_err(|_| CryptoError::Decrypt("bad KEK".into()))?
            .unwrap_key(wrapped, &mut buf)
            .map_err(bad)?,
        16 => aes_kw::KwAes128::new_from_slice(kek)
            .map_err(|_| CryptoError::Decrypt("bad KEK".into()))?
            .unwrap_key(wrapped, &mut buf)
            .map_err(bad)?,
        _ => return Err(CryptoError::Decrypt("bad KEK length".into())),
    };
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SEC 1 §3.6.1 shape: the output is H(Z || 00000001 || info) truncated.
    #[test]
    fn x963_kdf_is_the_counter_construction() {
        let out = x963_kdf(Hash::Sha256, b"z", b"info", 40);
        let mut first = b"z".to_vec();
        first.extend_from_slice(&1u32.to_be_bytes());
        first.extend_from_slice(b"info");
        let mut second = b"z".to_vec();
        second.extend_from_slice(&2u32.to_be_bytes());
        second.extend_from_slice(b"info");
        let mut want = Hash::Sha256.digest(&first);
        want.extend_from_slice(&Hash::Sha256.digest(&second)[..8]);
        assert_eq!(out, want);
    }
}
