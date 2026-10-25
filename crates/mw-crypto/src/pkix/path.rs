//! Certificate path building and validation for S/MIME (t32-e41, RFC 5280 §6 as
//! far as email needs it): from the signer's or recipient's certificate up to a
//! trust anchor, checking every signature, the validity windows, basicConstraints
//! and pathLenConstraint, keyUsage, extendedKeyUsage `emailProtection`, the
//! minimum RSA size, and Mozilla's per-root email distrust date.
//!
//! Not implemented (stated, not silently skipped): name constraints and
//! certificate policies. A path through a CA certificate that marks either as
//! critical is refused rather than accepted unchecked.

use der::{Decode, Encode};
use x509_cert::Certificate;
use x509_cert::ext::pkix::{BasicConstraints, ExtendedKeyUsage, KeyUsage, KeyUsages};

use super::oid;
use super::sig::{self, Use};

/// Where a trust anchor came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchorSource {
    /// The user trusted this certificate (a root, or a leaf they accepted).
    User,
    /// An administrator added this root (globally or for the domain).
    Admin,
    /// The bundled Mozilla email-protection root set.
    Mozilla,
}

impl AnchorSource {
    pub fn token(self) -> &'static str {
        match self {
            AnchorSource::User => "user",
            AnchorSource::Admin => "admin",
            AnchorSource::Mozilla => "mozilla",
        }
    }
}

/// A trust anchor.
#[derive(Debug, Clone)]
pub struct Anchor {
    pub cert: Certificate,
    pub source: AnchorSource,
    /// Certificates issued under this root with a notBefore after this time (Unix
    /// seconds) are not trusted for email (NSS `CKA_NSS_EMAIL_DISTRUST_AFTER`).
    pub distrust_after: Option<u64>,
}

/// What the end-entity certificate is being used for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Purpose {
    /// Checking a signature (digitalSignature or nonRepudiation).
    Sign,
    /// Encrypting to it (keyEncipherment for RSA, keyAgreement for EC/X25519).
    Encrypt,
}

/// Validation settings.
#[derive(Debug, Clone, Copy)]
pub struct Policy {
    pub purpose: Purpose,
    /// Accept `anyExtendedKeyUsage` in place of `emailProtection`.
    pub allow_any_eku: bool,
    /// Smallest RSA modulus accepted anywhere in the path.
    pub min_rsa_bits: usize,
    /// The time the path must be valid at (Unix seconds): the signing time for a
    /// signature, now for encryption.
    pub at: u64,
}

impl Default for Policy {
    fn default() -> Self {
        Policy {
            purpose: Purpose::Sign,
            allow_any_eku: false,
            min_rsa_bits: 2048,
            at: 0,
        }
    }
}

/// Why a path is not acceptable, most specific first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathError {
    /// A certificate in the path was expired at the check time (its SHA-256).
    Expired(String),
    /// A certificate in the path was not yet valid at the check time.
    NotYetValid(String),
    /// A constraint failed: `key-usage`, `eku`, `basic-constraints`, `path-length`,
    /// `weak-key`, `critical-extension`, `distrusted-root`, `chain-signature`.
    Constraint(&'static str),
    /// No path to any trust anchor.
    NoAnchor,
}

impl PathError {
    pub fn token(&self) -> &'static str {
        match self {
            PathError::Expired(_) => "expired",
            PathError::NotYetValid(_) => "not-yet-valid",
            PathError::Constraint(c) => c,
            PathError::NoAnchor => "no-anchor",
        }
    }
}

/// The result of path validation.
#[derive(Debug, Clone)]
pub struct Validated {
    /// Leaf first, then intermediates up to (not including) the anchor.
    pub path: Vec<Certificate>,
    /// The anchor reached, when the path is trusted.
    pub anchor: Option<(Certificate, AnchorSource)>,
    /// `None` when the path is trusted and every check passed.
    pub error: Option<PathError>,
}

const MAX_DEPTH: usize = 8;

/// Build a path from `leaf` through `intermediates` to one of `anchors` and
/// validate it under `policy`.
pub fn validate(
    leaf: &Certificate,
    intermediates: &[Certificate],
    anchors: &[Anchor],
    policy: &Policy,
) -> Validated {
    // The leaf's own checks come first: they do not depend on the path.
    let leaf_error = check_leaf(leaf, policy);

    // A leaf the user (or an admin) trusted directly is its own anchor.
    let leaf_der = leaf.to_der().unwrap_or_default();
    if let Some(a) = anchors
        .iter()
        .find(|a| a.cert.to_der().ok().as_deref() == Some(&leaf_der[..]))
    {
        return Validated {
            path: vec![leaf.clone()],
            anchor: Some((a.cert.clone(), a.source)),
            error: leaf_error,
        };
    }

    let mut best: Option<Validated> = None;
    let mut path = vec![leaf.clone()];
    search(&mut path, intermediates, anchors, policy, &mut best);
    match best {
        Some(mut v) => {
            // A leaf problem outranks nothing found further up, but a problem up
            // the path (expired intermediate) is reported only if the leaf is fine.
            if leaf_error.is_some() {
                v.error = leaf_error;
            }
            v
        }
        None => Validated {
            path: vec![leaf.clone()],
            anchor: None,
            error: Some(leaf_error.unwrap_or(PathError::NoAnchor)),
        },
    }
}

/// Depth-first search over candidate issuers. Records the first fully valid path,
/// or else the first anchored path with an error, in `best`.
fn search(
    path: &mut Vec<Certificate>,
    intermediates: &[Certificate],
    anchors: &[Anchor],
    policy: &Policy,
    best: &mut Option<Validated>,
) {
    if best.as_ref().is_some_and(|b| b.error.is_none()) || path.len() > MAX_DEPTH {
        return;
    }
    let current = path.last().cloned().expect("path is never empty");
    for a in anchors {
        // A directly trusted mailbox certificate is a pin for that certificate,
        // not a delegation to mint identities. Only CA anchors may issue children.
        // Exact leaf pins are handled by validate before path search.
        if !matches!(a.cert.tbs_certificate().get_extension::<BasicConstraints>(), Ok(Some((_, bc))) if bc.ca)
        {
            continue;
        }
        if !name_eq(
            a.cert.tbs_certificate().subject(),
            current.tbs_certificate().issuer(),
        ) {
            continue;
        }
        if !signed_by(&current, &a.cert) {
            continue;
        }
        let error = check_path(path, a, policy);
        let candidate = Validated {
            path: path.clone(),
            anchor: Some((a.cert.clone(), a.source)),
            error,
        };
        let better = match best {
            None => true,
            Some(b) => b.error.is_some() && candidate.error.is_none(),
        };
        if better {
            *best = Some(candidate);
        }
        if best.as_ref().is_some_and(|b| b.error.is_none()) {
            return;
        }
    }
    for i in intermediates {
        if !name_eq(
            i.tbs_certificate().subject(),
            current.tbs_certificate().issuer(),
        ) {
            continue;
        }
        if path.iter().any(|p| p == i) || !signed_by(&current, i) {
            continue;
        }
        path.push(i.clone());
        search(path, intermediates, anchors, policy, best);
        path.pop();
        if best.as_ref().is_some_and(|b| b.error.is_none()) {
            return;
        }
    }
}

fn signed_by(child: &Certificate, parent: &Certificate) -> bool {
    let Ok(tbs) = child.tbs_certificate().to_der() else {
        return false;
    };
    let Some(sig) = child.signature().as_bytes() else {
        return false;
    };
    sig::verify(
        parent.tbs_certificate().subject_public_key_info(),
        child.signature_algorithm(),
        None,
        &tbs,
        sig,
        Use::Certificate,
    )
    .is_ok()
}

/// Checks on the path above the leaf: CA flags, path lengths, validity, EKU,
/// critical extensions we do not process, key sizes, the anchor's distrust date.
fn check_path(path: &[Certificate], anchor: &Anchor, policy: &Policy) -> Option<PathError> {
    // Validity first (the most specific thing a user can act on).
    for c in path {
        if let Some(e) = validity_error(c, policy.at) {
            return Some(e);
        }
    }
    for (depth, c) in path.iter().enumerate().skip(1) {
        // `depth - 1` intermediates sit below this CA certificate.
        if let Some(e) = check_ca(c, depth - 1) {
            return Some(e);
        }
        if let Some(e) = eku_error(c, true, policy.allow_any_eku) {
            return Some(e);
        }
        if weak_key(c, policy.min_rsa_bits) {
            return Some(PathError::Constraint("weak-key"));
        }
    }
    if let Some(after) = anchor.distrust_after {
        let leaf = &path[0];
        let not_before = leaf
            .tbs_certificate()
            .validity()
            .not_before
            .to_unix_duration()
            .as_secs();
        if not_before > after {
            return Some(PathError::Constraint("distrusted-root"));
        }
    }
    None
}

pub(crate) fn check_leaf(leaf: &Certificate, policy: &Policy) -> Option<PathError> {
    if let Some(e) = validity_error(leaf, policy.at) {
        return Some(e);
    }
    if weak_key(leaf, policy.min_rsa_bits) {
        return Some(PathError::Constraint("weak-key"));
    }
    if let Some(e) = unhandled_critical(leaf) {
        return Some(e);
    }
    let ku = match leaf.tbs_certificate().get_extension::<KeyUsage>() {
        Ok(ku) => ku,
        Err(_) => return Some(PathError::Constraint("key-usage")),
    };
    if let Some((_, ku)) = ku {
        let ok = match policy.purpose {
            Purpose::Sign => {
                ku.0.contains(KeyUsages::DigitalSignature)
                    || ku.0.contains(KeyUsages::NonRepudiation)
            }
            Purpose::Encrypt => {
                let alg = leaf
                    .tbs_certificate()
                    .subject_public_key_info()
                    .algorithm
                    .oid;
                if alg == oid::RSA_ENCRYPTION {
                    ku.0.contains(KeyUsages::KeyEncipherment)
                } else {
                    ku.0.contains(KeyUsages::KeyAgreement)
                }
            }
        };
        if !ok {
            return Some(PathError::Constraint("key-usage"));
        }
    }
    eku_error(leaf, false, policy.allow_any_eku)
}

fn validity_error(c: &Certificate, at: u64) -> Option<PathError> {
    let v = c.tbs_certificate().validity();
    let nb = v.not_before.to_unix_duration().as_secs();
    let na = v.not_after.to_unix_duration().as_secs();
    if at < nb {
        Some(PathError::NotYetValid(fingerprint(c)))
    } else if at > na {
        Some(PathError::Expired(fingerprint(c)))
    } else {
        None
    }
}

/// A CA certificate must say so (basicConstraints cA), may sign certificates
/// (keyUsage keyCertSign when present), and allow `below` intermediates under it.
fn check_ca(c: &Certificate, below: usize) -> Option<PathError> {
    match c.tbs_certificate().get_extension::<BasicConstraints>() {
        Ok(Some((_, bc))) if bc.ca => {
            if bc.path_len_constraint.is_some_and(|n| (n as usize) < below) {
                return Some(PathError::Constraint("path-length"));
            }
        }
        _ => return Some(PathError::Constraint("basic-constraints")),
    }
    match c.tbs_certificate().get_extension::<KeyUsage>() {
        Ok(Some((_, ku))) if !ku.0.contains(KeyUsages::KeyCertSign) => {
            return Some(PathError::Constraint("key-usage"));
        }
        Err(_) => return Some(PathError::Constraint("key-usage")),
        _ => {}
    }
    unhandled_critical(c)
}

/// The extended key usage allows email protection. Absent EKU allows everything
/// (RFC 5280 §4.2.1.12); on a CA certificate a present EKU restricts what it may
/// issue for (the Mozilla policy).
fn eku_error(c: &Certificate, _is_ca: bool, allow_any: bool) -> Option<PathError> {
    match c.tbs_certificate().get_extension::<ExtendedKeyUsage>() {
        Ok(Some((_, eku))) => {
            let ok = eku.0.contains(&oid::KP_EMAIL_PROTECTION)
                || (allow_any && eku.0.contains(&oid::ANY_EXT_KEY_USAGE));
            if ok {
                None
            } else {
                Some(PathError::Constraint("eku"))
            }
        }
        Ok(None) => None,
        Err(_) => Some(PathError::Constraint("eku")),
    }
}

/// A critical extension this module does not process (name constraints,
/// certificate policies, policy constraints, ...) means the certificate cannot be
/// validated here.
fn unhandled_critical(c: &Certificate) -> Option<PathError> {
    const HANDLED: [const_oid::ObjectIdentifier; 6] = [
        oid::KEY_USAGE,
        oid::BASIC_CONSTRAINTS,
        oid::EXT_KEY_USAGE,
        oid::SUBJECT_ALT_NAME,
        oid::SUBJECT_KEY_ID,
        oid::AUTHORITY_KEY_ID,
    ];
    let bad = c
        .tbs_certificate()
        .extensions()
        .into_iter()
        .flatten()
        .any(|e| e.critical && !HANDLED.contains(&e.extn_id));
    if bad {
        Some(PathError::Constraint("critical-extension"))
    } else {
        None
    }
}

fn weak_key(c: &Certificate, min_rsa_bits: usize) -> bool {
    sig::rsa_bits(c.tbs_certificate().subject_public_key_info()).is_some_and(|b| b < min_rsa_bits)
}

/// SHA-256 over the certificate DER, upper-case hex.
pub fn fingerprint(c: &Certificate) -> String {
    use sha2::Digest;
    hex::encode(sha2::Sha256::digest(c.to_der().unwrap_or_default())).to_uppercase()
}

/// RFC 5280 §7.1 name comparison, simplified: equal DER, or the same attributes
/// in the same order with string values equal after case folding and collapsing
/// white space (CAs re-encode names with different string types).
pub fn name_eq(a: &x509_cert::name::Name, b: &x509_cert::name::Name) -> bool {
    if a == b {
        return true;
    }
    let ra: Vec<_> = a.iter_rdn().collect();
    let rb: Vec<_> = b.iter_rdn().collect();
    if ra.len() != rb.len() {
        return false;
    }
    ra.iter().zip(rb.iter()).all(|(x, y)| {
        let xs: Vec<_> = x.iter().collect();
        let ys: Vec<_> = y.iter().collect();
        xs.len() == ys.len()
            && xs
                .iter()
                .zip(ys.iter())
                .all(|(p, q)| p.oid == q.oid && fold(p.value.value()) == fold(q.value.value()))
    })
}

fn fold(v: &[u8]) -> String {
    String::from_utf8_lossy(v)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Parse a list of certificates from PEM text (one or many) or a single DER.
pub fn parse_certs(bytes: &[u8]) -> Vec<Certificate> {
    let trimmed = bytes.trim_ascii_start();
    if trimmed.starts_with(b"-----BEGIN") {
        Certificate::load_pem_chain(trimmed).unwrap_or_default()
    } else {
        Certificate::from_der(bytes)
            .map(|c| vec![c])
            .unwrap_or_default()
    }
}
