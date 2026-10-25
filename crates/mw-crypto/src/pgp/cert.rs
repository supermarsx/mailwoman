//! Certificate evaluation (t32-e37): which parts of an OpenPGP key are usable at a
//! given time, decided from *verified* self-signatures only.
//!
//! rPGP parses keys but leaves policy to the caller, and the code before this module
//! took the first subkey whose algorithm could encrypt and never looked at a
//! signature. Here every self-signature (direct-key, User ID binding, subkey binding,
//! revocation) is verified against the primary key before anything in it is used;
//! an unverifiable signature is ignored, never trusted.
//!
//! Rules, from RFC 9580 §5.2.3 and §10.1:
//! - The primary key's metadata (key flags, expiry, features) comes from the newest
//!   valid direct-key signature on a v6 key, and from the primary User ID's newest
//!   valid binding on a v4 key (falling back to the other one when missing).
//! - A key revocation with reason "superseded" or "retired" (soft) applies from its
//!   creation time; any other reason, or none, (hard) applies to every time.
//! - A subkey is usable when its newest valid binding is not expired, it is not
//!   revoked, the primary is usable, and — for a signing subkey — the binding carries
//!   a valid embedded back-signature from the subkey.
//! - When a binding has no Key Flags subpacket at all, the flags are derived from the
//!   algorithm (an old key that predates the subpacket).

use pgp::composed::{SignedPublicKey, SignedPublicSubKey};
use pgp::crypto::hash::HashAlgorithm;
use pgp::crypto::public_key::PublicKeyAlgorithm;
use pgp::packet::{KeyFlags, RevocationCode, Signature, SignatureType, SubpacketData};
use pgp::types::{KeyDetails, KeyVersion, PublicParams, Tag};
use serde::{Deserialize, Serialize};

use crate::clock;

/// Signatures dated this far in the future are still accepted (clock skew between
/// the signer and us). Anything later is ignored as not yet valid.
const CLOCK_SKEW_SECS: u64 = 300;

/// A revocation found on a key, a subkey or a User ID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Revocation {
    /// RFC 3339 creation time of the revocation signature.
    pub at: String,
    /// `"compromised"` | `"superseded"` | `"retired"` | `"user-id-invalid"` |
    /// `"unspecified"`.
    pub reason: String,
    /// A hard revocation invalidates every signature the key ever made; a soft one
    /// (superseded, retired) only those made after `at`.
    pub hard: bool,
    /// The free-text reason in the revocation, possibly empty.
    pub message: String,
}

/// One User ID with its state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PgpUserId {
    pub value: String,
    /// The address inside `Name <address>` (or the whole value when it is a bare
    /// address), lower-cased; `None` when the User ID holds no address.
    pub address: Option<String>,
    pub primary: bool,
    /// `"valid"` | `"revoked"` | `"invalid"` (no verifiable self-signature).
    pub state: String,
}

/// One subkey with its state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PgpSubkeyInfo {
    pub fingerprint: String,
    pub key_id: String,
    pub algorithm: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    /// `"valid"` | `"expired"` | `"revoked"` | `"invalid"` | `"not-yet-valid"`.
    pub state: String,
    /// `"certify"`, `"sign"`, `"encrypt-communications"`, `"encrypt-storage"`,
    /// `"authenticate"`.
    pub flags: Vec<String>,
    pub revocation: Option<Revocation>,
}

/// What a key is and whether it can be used now (`inspectKey`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PgpKeyInfo {
    pub fingerprint: String,
    pub key_id: String,
    /// OpenPGP key version: 4 or 6.
    pub version: u8,
    /// `ed25519`, `x25519`, `ed448`, `x448`, `eddsa-legacy`, `rsa-<bits>`,
    /// `ecdsa-<curve>`, `ecdh-<curve>`, `dsa`, `elgamal`.
    pub algorithm: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    /// `"valid"` | `"expired"` | `"revoked"` | `"invalid"` | `"not-yet-valid"`.
    pub state: String,
    pub flags: Vec<String>,
    pub revocation: Option<Revocation>,
    /// The key advertises SEIPDv2 (AEAD) support, or is a v6 key.
    pub seipd_v2: bool,
    pub user_ids: Vec<PgpUserId>,
    pub subkeys: Vec<PgpSubkeyInfo>,
    /// A usable encryption key exists now (primary or subkey).
    pub can_encrypt: bool,
    /// A usable signing key exists now (primary or subkey).
    pub can_sign: bool,
    /// The armor held secret key material.
    pub has_secret: bool,
}

/// The validity state of one key component at a point in time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum State {
    Valid,
    Expired,
    Revoked,
    Invalid,
    NotYetValid,
}

impl State {
    pub(crate) fn token(self) -> &'static str {
        match self {
            State::Valid => "valid",
            State::Expired => "expired",
            State::Revoked => "revoked",
            State::Invalid => "invalid",
            State::NotYetValid => "not-yet-valid",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RevocationAt {
    pub at: u64,
    pub hard: bool,
    pub code: Option<RevocationCode>,
    pub message: String,
}

impl RevocationAt {
    /// Does this revocation make the key unusable at time `t`?
    pub(crate) fn applies_at(&self, t: u64) -> bool {
        self.hard || t >= self.at
    }

    fn public(&self) -> Revocation {
        Revocation {
            at: clock::rfc3339(self.at),
            reason: reason_token(self.code).into(),
            hard: self.hard,
            message: self.message.clone(),
        }
    }
}

/// A subkey after evaluation.
pub(crate) struct SubEval<'a> {
    pub key: &'a SignedPublicSubKey,
    pub state: State,
    pub flags: KeyFlags,
    pub expires: Option<u64>,
    pub revocation: Option<RevocationAt>,
}

/// A User ID after evaluation.
pub(crate) struct UserEval {
    pub value: String,
    pub address: Option<String>,
    pub primary: bool,
    pub state: State,
}

/// A whole certificate evaluated at one time.
pub(crate) struct CertEval<'a> {
    pub cert: &'a SignedPublicKey,
    pub state: State,
    pub flags: KeyFlags,
    pub expires: Option<u64>,
    pub revocation: Option<RevocationAt>,
    pub seipd_v2: bool,
    pub users: Vec<UserEval>,
    pub subkeys: Vec<SubEval<'a>>,
}

/// A key to encrypt to: an encryption-capable subkey, or the primary itself.
pub(crate) enum EncTarget<'a> {
    Primary(&'a SignedPublicKey),
    Sub(&'a SignedPublicSubKey),
}

impl CertEval<'_> {
    /// The keys a message to this certificate is encrypted to, now: every usable
    /// subkey flagged for communications encryption (else for storage), and the
    /// primary only when no subkey qualifies and it carries the flag itself.
    pub(crate) fn encryption_targets(&self) -> Vec<EncTarget<'_>> {
        if self.state != State::Valid {
            return Vec::new();
        }
        let usable = |s: &&SubEval<'_>| s.state == State::Valid;
        let comms: Vec<_> = self
            .subkeys
            .iter()
            .filter(usable)
            .filter(|s| s.flags.encrypt_comms())
            .map(|s| EncTarget::Sub(s.key))
            .collect();
        if !comms.is_empty() {
            return comms;
        }
        let storage: Vec<_> = self
            .subkeys
            .iter()
            .filter(usable)
            .filter(|s| s.flags.encrypt_storage())
            .map(|s| EncTarget::Sub(s.key))
            .collect();
        if !storage.is_empty() {
            return storage;
        }
        if self.flags.encrypt_comms() || self.flags.encrypt_storage() {
            return vec![EncTarget::Primary(self.cert)];
        }
        Vec::new()
    }

    /// Fingerprints (upper hex) of the keys that may make data signatures now:
    /// the primary when flagged for signing, then every usable signing subkey.
    pub(crate) fn signing_fingerprints(&self) -> Vec<String> {
        let mut out = Vec::new();
        if self.state != State::Valid {
            return out;
        }
        if self.flags.sign() {
            out.push(fingerprint_hex(&self.cert.primary_key));
        }
        for s in &self.subkeys {
            if s.state == State::Valid && s.flags.sign() {
                out.push(fingerprint_hex(&s.key.key));
            }
        }
        out
    }

    /// The addresses of the valid User IDs, lower-cased, primary first.
    pub(crate) fn addresses(&self) -> Vec<String> {
        let mut users: Vec<&UserEval> = self
            .users
            .iter()
            .filter(|u| u.state == State::Valid)
            .collect();
        users.sort_by_key(|u| !u.primary);
        let mut out: Vec<String> = Vec::new();
        for u in users {
            if let Some(a) = &u.address
                && !out.contains(a)
            {
                out.push(a.clone());
            }
        }
        out
    }

    pub(crate) fn info(&self, has_secret: bool) -> PgpKeyInfo {
        let primary = &self.cert.primary_key;
        let can_encrypt = !self.encryption_targets().is_empty();
        let can_sign = !self.signing_fingerprints().is_empty();
        PgpKeyInfo {
            fingerprint: fingerprint_hex(primary),
            key_id: primary.legacy_key_id().to_string(),
            version: version_number(primary.version()),
            algorithm: algorithm_name(primary.public_params()),
            created_at: clock::rfc3339(u64::from(primary.created_at().as_secs())),
            expires_at: self.expires.map(clock::rfc3339),
            state: self.state.token().into(),
            flags: flag_tokens(&self.flags),
            revocation: self.revocation.as_ref().map(RevocationAt::public),
            seipd_v2: self.seipd_v2,
            user_ids: self
                .users
                .iter()
                .map(|u| PgpUserId {
                    value: u.value.clone(),
                    address: u.address.clone(),
                    primary: u.primary,
                    state: match u.state {
                        State::Valid => "valid",
                        State::Revoked => "revoked",
                        _ => "invalid",
                    }
                    .into(),
                })
                .collect(),
            subkeys: self
                .subkeys
                .iter()
                .map(|s| PgpSubkeyInfo {
                    fingerprint: fingerprint_hex(&s.key.key),
                    key_id: s.key.key.legacy_key_id().to_string(),
                    algorithm: algorithm_name(s.key.key.public_params()),
                    created_at: clock::rfc3339(u64::from(s.key.key.created_at().as_secs())),
                    expires_at: s.expires.map(clock::rfc3339),
                    state: s.state.token().into(),
                    flags: flag_tokens(&s.flags),
                    revocation: s.revocation.as_ref().map(RevocationAt::public),
                })
                .collect(),
            can_encrypt,
            can_sign,
            has_secret,
        }
    }
}

/// Evaluate `cert` at Unix time `at`.
pub(crate) fn evaluate(cert: &SignedPublicKey, at: u64) -> CertEval<'_> {
    let primary = &cert.primary_key;
    let created = u64::from(primary.created_at().as_secs());

    // Key revocations: only ones that verify against the primary key count.
    let revocation = newest_revocation(cert.details.revocation_signatures.iter().filter(|s| {
        s.typ() == Some(SignatureType::KeyRevocation) && s.verify_key(primary).is_ok()
    }));

    // Direct-key self-signatures valid at `at`.
    let direct = newest_direct(&cert.details.direct_signatures, primary, at);

    // User IDs: newest valid self-certification, newer revocation wins.
    let mut users: Vec<UserEval> = Vec::new();
    let mut primary_user_sig: Option<&Signature> = None;
    let mut primary_user_rank: Option<(bool, u64)> = None;
    for user in &cert.details.users {
        let value = String::from_utf8_lossy(user.id.id()).into_owned();
        let address = address_of(&value);
        let binding = newest_user_binding(user, primary, at);
        let revoked = user
            .signatures
            .iter()
            .filter(|s| {
                s.typ() == Some(SignatureType::CertRevocation)
                    && issued_by(s, primary)
                    && s.verify_certification(primary, Tag::UserId, &user.id)
                        .is_ok()
            })
            .filter_map(sig_created)
            .filter(|t| *t <= at + CLOCK_SKEW_SECS)
            .max();
        let state = match (binding, revoked) {
            (None, _) => State::Invalid,
            (Some(b), Some(r)) if r >= sig_created(b).unwrap_or(0) => State::Revoked,
            (Some(_), _) => State::Valid,
        };
        let is_primary = binding.is_some_and(Signature::is_primary);
        if state == State::Valid
            && let Some(b) = binding
        {
            let rank = (is_primary, sig_created(b).unwrap_or(0));
            if primary_user_rank.is_none_or(|r| rank > r) {
                primary_user_rank = Some(rank);
                primary_user_sig = Some(b);
            }
        }
        users.push(UserEval {
            value,
            address,
            primary: false,
            state,
        });
    }
    // Mark the chosen primary User ID.
    if let Some(sig) = primary_user_sig {
        for (u, user) in users.iter_mut().zip(&cert.details.users) {
            if user.signatures.iter().any(|s| std::ptr::eq(s, sig)) {
                u.primary = true;
            }
        }
    }

    // The signature that carries the primary key's metadata.
    let meta = match primary.version() {
        KeyVersion::V6 => direct.or(primary_user_sig),
        _ => primary_user_sig.or(direct),
    };
    let flags = meta
        .map(|s| flags_of(s, primary.algorithm()))
        .unwrap_or_default();
    let expires = meta.and_then(|s| expiry_of(s, created));
    let seipd_v2 = primary.version() == KeyVersion::V6
        || meta
            .and_then(Signature::features)
            .is_some_and(pgp::packet::Features::seipd_v2);

    let state = component_state(created, meta.is_some(), expires, revocation.as_ref(), at);

    // Subkeys.
    let subkeys = cert
        .public_subkeys
        .iter()
        .map(|sub| evaluate_subkey(cert, sub, state, at))
        .collect();

    CertEval {
        cert,
        state,
        flags,
        expires,
        revocation,
        seipd_v2,
        users,
        subkeys,
    }
}

fn evaluate_subkey<'a>(
    cert: &SignedPublicKey,
    sub: &'a SignedPublicSubKey,
    primary_state: State,
    at: u64,
) -> SubEval<'a> {
    let primary = &cert.primary_key;
    let created = u64::from(sub.key.created_at().as_secs());
    let binding = newest_subkey_binding(&sub.signatures, primary, &sub.key, at);
    let flags = binding
        .map(|s| flags_of(s, sub.key.algorithm()))
        .unwrap_or_default();
    // A signing subkey must agree to the binding (embedded primary key binding
    // signature made by the subkey), or anyone could claim someone else's signing
    // key as their own subkey.
    let back_signed = !flags.sign()
        || binding
            .and_then(Signature::embedded_signature)
            .is_some_and(|e| e.verify_primary_key_binding(&sub.key, primary).is_ok());
    let expires = binding.and_then(|s| expiry_of(s, created));
    let revocation = newest_revocation(sub.signatures.iter().filter(|s| {
        s.typ() == Some(SignatureType::SubkeyRevocation)
            && s.verify_subkey_binding(primary, &sub.key).is_ok()
    }));
    let own = component_state(
        created,
        binding.is_some() && back_signed,
        expires,
        revocation.as_ref(),
        at,
    );
    // The subkey is only as usable as its primary.
    let state = match (own, primary_state) {
        (State::Valid, State::Valid) => State::Valid,
        (State::Valid, p) => p,
        (o, _) => o,
    };
    SubEval {
        key: sub,
        state,
        flags,
        expires,
        revocation,
    }
}

/// The newest verified direct-key self-signature in force at `at`.
pub(crate) fn newest_direct<'a>(
    sigs: &'a [Signature],
    primary: &pgp::packet::PublicKey,
    at: u64,
) -> Option<&'a Signature> {
    newest_valid(
        sigs.iter()
            .filter(|s| s.typ() == Some(SignatureType::Key) && s.verify_key(primary).is_ok()),
        at,
    )
}

/// The newest verified self-certification of `user` in force at `at`.
pub(crate) fn newest_user_binding<'a>(
    user: &'a pgp::types::SignedUser,
    primary: &pgp::packet::PublicKey,
    at: u64,
) -> Option<&'a Signature> {
    newest_valid(
        user.signatures.iter().filter(|s| {
            s.is_certification()
                && s.typ() != Some(SignatureType::CertRevocation)
                && issued_by(s, primary)
                && s.verify_certification(primary, Tag::UserId, &user.id)
                    .is_ok()
        }),
        at,
    )
}

/// The newest verified binding of `subkey` to `primary` in force at `at`.
pub(crate) fn newest_subkey_binding<'a, K>(
    sigs: &'a [Signature],
    primary: &pgp::packet::PublicKey,
    subkey: &K,
    at: u64,
) -> Option<&'a Signature>
where
    K: KeyDetails + pgp::ser::Serialize,
{
    newest_valid(
        sigs.iter().filter(|s| {
            s.typ() == Some(SignatureType::SubkeyBinding)
                && s.verify_subkey_binding(primary, subkey).is_ok()
        }),
        at,
    )
}

fn component_state(
    created: u64,
    bound: bool,
    expires: Option<u64>,
    revocation: Option<&RevocationAt>,
    at: u64,
) -> State {
    if revocation.is_some_and(|r| r.applies_at(at)) {
        State::Revoked
    } else if !bound {
        State::Invalid
    } else if created > at + CLOCK_SKEW_SECS {
        State::NotYetValid
    } else if expires.is_some_and(|e| at >= e) {
        State::Expired
    } else {
        State::Valid
    }
}

/// The newest signature in `sigs` that is in force at `at` (created no later than
/// `at` + skew, not past its own signature expiry).
fn newest_valid<'a>(sigs: impl Iterator<Item = &'a Signature>, at: u64) -> Option<&'a Signature> {
    sigs.filter(|s| sig_in_force(s, at))
        .max_by_key(|s| sig_created(s).unwrap_or(0))
}

fn sig_in_force(sig: &Signature, at: u64) -> bool {
    let Some(created) = sig_created(sig) else {
        return false;
    };
    if created > at + CLOCK_SKEW_SECS {
        return false;
    }
    match sig.signature_expiration_time() {
        Some(d) if d.as_secs() != 0 => at < created + u64::from(d.as_secs()),
        _ => true,
    }
}

pub(crate) fn sig_created(sig: &Signature) -> Option<u64> {
    sig.created().map(|t| u64::from(t.as_secs()))
}

fn newest_revocation<'a>(sigs: impl Iterator<Item = &'a Signature>) -> Option<RevocationAt> {
    let mut best: Option<RevocationAt> = None;
    for sig in sigs {
        let at = sig_created(sig).unwrap_or(0);
        let code = sig.revocation_reason_code().copied();
        let hard = !matches!(
            code,
            Some(RevocationCode::KeySuperseded | RevocationCode::KeyRetired)
        );
        let message = sig
            .revocation_reason_string()
            .map(|b| String::from_utf8_lossy(b).into_owned())
            .unwrap_or_default();
        let candidate = RevocationAt {
            at,
            hard,
            code,
            message,
        };
        // A hard revocation outranks a soft one; among equals the earliest applies.
        best = match best {
            None => Some(candidate),
            Some(b) if candidate.hard && !b.hard => Some(candidate),
            Some(b) if candidate.hard == b.hard && candidate.at < b.at => Some(candidate),
            keep => keep,
        };
    }
    best
}

/// Key flags from a binding signature; derived from the algorithm when the
/// signature has no Key Flags subpacket at all (keys older than the subpacket).
fn flags_of(sig: &Signature, alg: PublicKeyAlgorithm) -> KeyFlags {
    let has_flags = sig.config().is_some_and(|c| {
        c.hashed_subpackets()
            .any(|p| matches!(p.data, SubpacketData::KeyFlags(_)))
    });
    if has_flags {
        return sig.key_flags();
    }
    let mut f = KeyFlags::default();
    if alg.can_sign() {
        f.set_sign(true);
        f.set_certify(true);
    }
    if alg.can_encrypt() {
        f.set_encrypt_comms(true);
        f.set_encrypt_storage(true);
    }
    f
}

/// Absolute expiry time from a Key Expiration Time subpacket (`None` = never).
fn expiry_of(sig: &Signature, key_created: u64) -> Option<u64> {
    match sig.key_expiration_time() {
        Some(d) if d.as_secs() != 0 => Some(key_created + u64::from(d.as_secs())),
        _ => None,
    }
}

/// The signature names `key` as its issuer (fingerprint or key id), or names no
/// issuer at all (then only verification can tell).
fn issued_by(sig: &Signature, key: &impl KeyDetails) -> bool {
    let fps = sig.issuer_fingerprint();
    let ids = sig.issuer_key_id();
    if fps.is_empty() && ids.is_empty() {
        return true;
    }
    let fp = key.fingerprint();
    let id = key.legacy_key_id();
    fps.iter().any(|f| **f == fp) || ids.iter().any(|i| **i == id)
}

/// A data signature uses a hash we refuse (MD5, SHA-1, RIPEMD-160: RFC 9580 §9.5
/// and §15).
pub(crate) fn weak_data_hash(sig: &Signature) -> bool {
    matches!(
        sig.hash_alg(),
        Some(HashAlgorithm::Md5 | HashAlgorithm::Sha1 | HashAlgorithm::Ripemd160)
    )
}

pub(crate) fn reason_token(code: Option<RevocationCode>) -> &'static str {
    match code {
        Some(RevocationCode::KeyCompromised) => "compromised",
        Some(RevocationCode::KeySuperseded) => "superseded",
        Some(RevocationCode::KeyRetired) => "retired",
        Some(RevocationCode::CertUserIdInvalid) => "user-id-invalid",
        _ => "unspecified",
    }
}

pub(crate) fn flag_tokens(f: &KeyFlags) -> Vec<String> {
    let mut out = Vec::new();
    if f.certify() {
        out.push("certify".into());
    }
    if f.sign() {
        out.push("sign".into());
    }
    if f.encrypt_comms() {
        out.push("encrypt-communications".into());
    }
    if f.encrypt_storage() {
        out.push("encrypt-storage".into());
    }
    if f.authentication() {
        out.push("authenticate".into());
    }
    out
}

pub(crate) fn fingerprint_hex(key: &impl KeyDetails) -> String {
    hex::encode(key.fingerprint().as_bytes()).to_uppercase()
}

pub(crate) fn version_number(v: KeyVersion) -> u8 {
    match v {
        KeyVersion::V2 => 2,
        KeyVersion::V3 => 3,
        KeyVersion::V4 => 4,
        KeyVersion::V5 => 5,
        KeyVersion::V6 => 6,
        _ => 0,
    }
}

/// A short algorithm name for display and policy (`rsa-3072`, `ed25519`, ...).
pub(crate) fn algorithm_name(params: &PublicParams) -> String {
    use pgp::types::{EcdhPublicParams, EcdsaPublicParams};
    match params {
        PublicParams::RSA(p) => format!("rsa-{}", rsa_bits(p)),
        PublicParams::DSA(_) => "dsa".into(),
        PublicParams::ECDSA(p) => match p {
            EcdsaPublicParams::P256 { .. } => "ecdsa-p256".into(),
            EcdsaPublicParams::P384 { .. } => "ecdsa-p384".into(),
            EcdsaPublicParams::P521 { .. } => "ecdsa-p521".into(),
            EcdsaPublicParams::Secp256k1 { .. } => "ecdsa-secp256k1".into(),
            other => format!("ecdsa-{}", curve_slug(other.curve().name())),
        },
        PublicParams::ECDH(p) => match p {
            EcdhPublicParams::Curve25519Legacy { .. } => "ecdh-cv25519".into(),
            EcdhPublicParams::P256 { .. } => "ecdh-p256".into(),
            EcdhPublicParams::P384 { .. } => "ecdh-p384".into(),
            EcdhPublicParams::P521 { .. } => "ecdh-p521".into(),
            other => format!("ecdh-{}", curve_slug(other.curve().name())),
        },
        PublicParams::Elgamal(_) => "elgamal".into(),
        PublicParams::EdDSALegacy(_) => "eddsa-legacy".into(),
        PublicParams::Ed25519(_) => "ed25519".into(),
        PublicParams::X25519(_) => "x25519".into(),
        PublicParams::X448(_) => "x448".into(),
        PublicParams::Ed448(_) => "ed448".into(),
        #[allow(unreachable_patterns)]
        _ => "unknown".into(),
    }
}

fn curve_slug(name: &str) -> String {
    name.to_ascii_lowercase()
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect()
}

/// RSA modulus size in bits, read from the serialized MPI (its first two octets
/// are the bit count, RFC 9580 §3.2), so no second `rsa` generation is needed here.
fn rsa_bits(p: &pgp::types::RsaPublicParams) -> u32 {
    use pgp::ser::Serialize;
    match p.to_bytes() {
        Ok(b) if b.len() >= 2 => u32::from(u16::from_be_bytes([b[0], b[1]])),
        _ => 0,
    }
}

/// The address in a User ID: the part inside the last `<...>`, else the whole
/// value when it looks like an address. Lower-cased.
pub(crate) fn address_of(user_id: &str) -> Option<String> {
    let candidate = match (user_id.rfind('<'), user_id.rfind('>')) {
        (Some(open), Some(close)) if open < close => user_id[open + 1..close].trim(),
        _ => user_id.trim(),
    };
    if candidate.contains('@') && !candidate.contains(char::is_whitespace) {
        Some(candidate.to_lowercase())
    } else {
        None
    }
}
