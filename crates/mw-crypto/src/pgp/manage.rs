//! OpenPGP key management (t32-e37): revocation, revocation certificates, expiry
//! changes, adding and revoking subkeys, passphrase change.
//!
//! Every operation takes the passphrase-locked bundle and the passphrase, makes new
//! self-signatures with the primary key, and returns the updated public key and
//! bundle. The bundle stays locked: secret material is unlocked only inside the
//! signing call (rPGP unlocks per operation), and a passphrase change re-locks
//! every component before the key is serialized again.

use pgp::composed::{ArmorOptions, KeyType, SignedSecretKey, SignedSecretSubKey};
use pgp::crypto::ecc_curve::ECCCurve;
use pgp::packet::{
    KeyFlags, PubKeyInner, PublicSubkey, RevocationCode, SecretSubkey, Signature, SignatureConfig,
    SignatureType, Subpacket, SubpacketData,
};
use pgp::types::{Duration, KeyDetails, KeyVersion, Password, Tag};
use serde::{Deserialize, Serialize};

use super::{cert, now_timestamp, parse_secret, pw};
use crate::clock;
use crate::error::{CryptoError, Result};
use crate::rng;

/// The updated key after a management operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedKey {
    pub public_key_armored: String,
    pub encrypted_private_bundle: String,
}

/// Why a key or subkey is revoked (RFC 9580 §5.2.3.31).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RevocationReason {
    /// The secret key may be known to someone else (hard: every signature it made
    /// stops counting).
    Compromised,
    /// Replaced by a newer key (soft: older signatures still count).
    Superseded,
    /// No longer used (soft).
    Retired,
    /// No reason given (hard).
    #[default]
    Unspecified,
}

impl RevocationReason {
    fn code(self) -> RevocationCode {
        match self {
            RevocationReason::Compromised => RevocationCode::KeyCompromised,
            RevocationReason::Superseded => RevocationCode::KeySuperseded,
            RevocationReason::Retired => RevocationCode::KeyRetired,
            RevocationReason::Unspecified => RevocationCode::NoReason,
        }
    }
}

/// What a new subkey is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SubkeyPurpose {
    Encrypt,
    Sign,
}

fn err(e: impl std::fmt::Display) -> CryptoError {
    CryptoError::KeyGen(e.to_string())
}

fn finish(ssk: &SignedSecretKey) -> Result<ManagedKey> {
    Ok(ManagedKey {
        public_key_armored: ssk
            .to_public_key()
            .to_armored_string(ArmorOptions::default())
            .map_err(err)?,
        encrypted_private_bundle: ssk
            .to_armored_string(ArmorOptions::default())
            .map_err(err)?,
    })
}

/// Lock every secret component (primary and subkeys) under `passphrase`.
pub(crate) fn lock_all(ssk: &mut SignedSecretKey, passphrase: &str) -> Result<()> {
    ssk.primary_key
        .set_password(rng::pgp_rng(), &pw(passphrase))
        .map_err(err)?;
    for sub in &mut ssk.secret_subkeys {
        sub.key
            .set_password(rng::pgp_rng(), &pw(passphrase))
            .map_err(err)?;
    }
    Ok(())
}

/// Check the passphrase before any signature is made, so a wrong passphrase is
/// reported as such rather than as a signing failure deep inside rPGP.
fn check_passphrase(ssk: &SignedSecretKey, passphrase: &str) -> Result<()> {
    ssk.primary_key
        .unlock(&pw(passphrase), |_, _| Ok(()))
        .map_err(|e| CryptoError::Decrypt(e.to_string()))?
        .map_err(|e| CryptoError::Decrypt(e.to_string()))
}

/// The basic hashed subpackets of a new self-signature, plus the unhashed Issuer
/// Key ID a v4 signature carries.
fn base_config(ssk: &SignedSecretKey, typ: SignatureType) -> Result<SignatureConfig> {
    let key = &ssk.primary_key;
    let mut config = SignatureConfig::from_key(rng::pgp_rng(), key, typ).map_err(err)?;
    config.hashed_subpackets = vec![
        Subpacket::regular(SubpacketData::SignatureCreationTime(now_timestamp())).map_err(err)?,
        Subpacket::regular(SubpacketData::IssuerFingerprint(key.fingerprint())).map_err(err)?,
    ];
    if key.version() <= KeyVersion::V4 {
        config.unhashed_subpackets =
            vec![Subpacket::regular(SubpacketData::IssuerKeyId(key.legacy_key_id())).map_err(err)?];
    }
    Ok(config)
}

/// A new self-signature config copying the metadata of `old` (flags, preferences,
/// features, embedded back-signature) with a new creation time and key expiry.
fn renewed_config(
    ssk: &SignedSecretKey,
    old: &Signature,
    expires: Option<Duration>,
) -> Result<SignatureConfig> {
    let typ = old
        .typ()
        .ok_or_else(|| CryptoError::Input("signature has no type".into()))?;
    let mut config = base_config(ssk, typ)?;
    let keep = |p: &&Subpacket| {
        !matches!(
            p.data,
            SubpacketData::SignatureCreationTime(_)
                | SubpacketData::KeyExpirationTime(_)
                | SubpacketData::IssuerFingerprint(_)
                | SubpacketData::IssuerKeyId(_)
                | SubpacketData::SignatureExpirationTime(_)
        )
    };
    if let Some(old_config) = old.config() {
        config
            .hashed_subpackets
            .extend(old_config.hashed_subpackets().filter(keep).cloned());
        config
            .unhashed_subpackets
            .extend(old_config.unhashed_subpackets().filter(keep).cloned());
    }
    if let Some(d) = expires {
        config
            .hashed_subpackets
            .push(Subpacket::regular(SubpacketData::KeyExpirationTime(d)).map_err(err)?);
    }
    Ok(config)
}

/// The Key Expiration Time for a component created at `created` that should
/// expire at `expires_at` (`None` = never).
fn expiry_duration(created: u32, expires_at: Option<u64>) -> Result<Option<Duration>> {
    match expires_at {
        None => Ok(None),
        Some(at) => {
            let created = u64::from(created);
            if at <= created {
                return Err(CryptoError::Input(
                    "the expiry must be after the key's creation time".into(),
                ));
            }
            let secs = u32::try_from(at - created)
                .map_err(|_| CryptoError::Input("the expiry is too far in the future".into()))?;
            Ok(Some(Duration::from_secs(secs)))
        }
    }
}

/// Re-sign `ssk` so the primary key (and, with `include_subkeys`, every bound
/// subkey) expires at `expires_at` (Unix seconds; `None` = never). On a v6 key the
/// expiry lives on a new direct-key signature; on a v4 key on new User ID
/// self-certifications.
pub(crate) fn with_expiry(
    mut ssk: SignedSecretKey,
    passphrase: &Password,
    expires_at: Option<u64>,
    include_subkeys: bool,
) -> Result<SignedSecretKey> {
    let now = clock::now_secs();
    let primary_public = ssk.primary_key.public_key().clone();
    let primary_expiry = expiry_duration(primary_public.created_at().as_secs(), expires_at)?;

    if primary_public.version() == KeyVersion::V6 {
        let old = cert::newest_direct(&ssk.details.direct_signatures, &primary_public, now)
            .cloned()
            .ok_or_else(|| {
                CryptoError::Input("the key has no valid direct-key signature".into())
            })?;
        let config = renewed_config(&ssk, &old, primary_expiry)?;
        let sig = config
            .sign_key(&ssk.primary_key, passphrase, &primary_public)
            .map_err(err)?;
        ssk.details.direct_signatures.push(sig);
    } else {
        let mut new_sigs: Vec<(usize, Signature)> = Vec::new();
        for (i, user) in ssk.details.users.iter().enumerate() {
            let Some(old) = cert::newest_user_binding(user, &primary_public, now) else {
                continue;
            };
            let config = renewed_config(&ssk, old, primary_expiry)?;
            let sig = config
                .sign_certification(
                    &ssk.primary_key,
                    &primary_public,
                    passphrase,
                    Tag::UserId,
                    &user.id,
                )
                .map_err(err)?;
            new_sigs.push((i, sig));
        }
        if new_sigs.is_empty() {
            return Err(CryptoError::Input("the key has no valid User ID".into()));
        }
        for (i, sig) in new_sigs {
            ssk.details.users[i].signatures.push(sig);
        }
    }

    if include_subkeys {
        let mut new_sigs: Vec<(usize, Signature)> = Vec::new();
        for (i, sub) in ssk.secret_subkeys.iter().enumerate() {
            let sub_public = sub.key.public_key();
            let Some(old) =
                cert::newest_subkey_binding(&sub.signatures, &primary_public, sub_public, now)
            else {
                continue;
            };
            let d = expiry_duration(sub_public.created_at().as_secs(), expires_at)?;
            let config = renewed_config(&ssk, old, d)?;
            let sig = config
                .sign_subkey_binding(&ssk.primary_key, &primary_public, passphrase, sub_public)
                .map_err(err)?;
            new_sigs.push((i, sig));
        }
        for (i, sig) in new_sigs {
            ssk.secret_subkeys[i].signatures.push(sig);
        }
    }
    Ok(ssk)
}

/// Set (or remove, with `None`) the expiry of a key and its subkeys. `expires_at`
/// is a Unix time.
pub fn set_expiry(bundle: &str, passphrase: &str, expires_at: Option<u64>) -> Result<ManagedKey> {
    let ssk = parse_secret(bundle)?;
    check_passphrase(&ssk, passphrase)?;
    let ssk = with_expiry(ssk, &pw(passphrase), expires_at, true)?;
    finish(&ssk)
}

fn revocation_signature(
    ssk: &SignedSecretKey,
    passphrase: &str,
    reason: RevocationReason,
    message: &str,
) -> Result<Signature> {
    let mut config = base_config(ssk, SignatureType::KeyRevocation)?;
    config.hashed_subpackets.push(
        Subpacket::regular(SubpacketData::RevocationReason(
            reason.code(),
            pgp::bytes::Bytes::from(message.as_bytes().to_vec()),
        ))
        .map_err(err)?,
    );
    config
        .sign_key(
            &ssk.primary_key,
            &pw(passphrase),
            ssk.primary_key.public_key(),
        )
        .map_err(err)
}

/// Revoke the whole key. The returned public key carries the revocation; publish
/// it so correspondents stop encrypting to it.
pub fn revoke(
    bundle: &str,
    passphrase: &str,
    reason: RevocationReason,
    message: &str,
) -> Result<ManagedKey> {
    let mut ssk = parse_secret(bundle)?;
    check_passphrase(&ssk, passphrase)?;
    let sig = revocation_signature(&ssk, passphrase, reason, message)?;
    ssk.details.revocation_signatures.push(sig);
    finish(&ssk)
}

/// A revocation certificate: the key revocation signature alone, armored as a
/// `PGP PUBLIC KEY BLOCK` the way GnuPG writes and imports them. Kept aside, it
/// revokes the key later even if the secret key or its passphrase is lost.
pub fn revocation_certificate(
    bundle: &str,
    passphrase: &str,
    reason: RevocationReason,
    message: &str,
) -> Result<String> {
    let ssk = parse_secret(bundle)?;
    check_passphrase(&ssk, passphrase)?;
    let sig = revocation_signature(&ssk, passphrase, reason, message)?;
    let mut out = Vec::new();
    let mut headers = std::collections::BTreeMap::new();
    headers.insert(
        "Comment".to_string(),
        vec!["This is a revocation certificate".to_string()],
    );
    pgp::armor::write(
        &WithHeader(&sig),
        pgp::armor::BlockType::PublicKey,
        &mut out,
        Some(&headers),
        true,
    )
    .map_err(err)?;
    String::from_utf8(out).map_err(err)
}

/// A packet serialized with its packet header. `armor::write` serializes the bare
/// packet body, which is right for composed keys and messages but not for a lone
/// signature packet.
struct WithHeader<'a>(&'a Signature);

impl pgp::ser::Serialize for WithHeader<'_> {
    fn to_writer<W: std::io::Write>(&self, writer: &mut W) -> pgp::errors::Result<()> {
        pgp::packet::PacketTrait::to_writer_with_header(self.0, writer)
    }
    fn write_len(&self) -> usize {
        pgp::packet::PacketTrait::write_len_with_header(self.0)
    }
}

/// Attach a revocation certificate to a key (public armor or bundle) and return
/// the revoked public key. Refused when the certificate does not verify against
/// the key's primary key.
pub fn apply_revocation_certificate(key_armored: &str, certificate: &str) -> Result<String> {
    use pgp::packet::{Packet, PacketParser};
    use std::io::Read;
    let mut cert = super::public_of(key_armored)?;
    let mut d = pgp::armor::Dearmor::new(certificate.as_bytes());
    let mut bytes = Vec::new();
    d.read_to_end(&mut bytes)
        .map_err(|e| CryptoError::Parse(e.to_string()))?;
    let mut applied = false;
    for p in PacketParser::new(&bytes[..]) {
        if let Ok(Packet::Signature(sig)) = p
            && sig.typ() == Some(SignatureType::KeyRevocation)
            && sig.verify_key(&cert.primary_key).is_ok()
        {
            cert.details.revocation_signatures.push(sig);
            applied = true;
        }
    }
    if !applied {
        return Err(CryptoError::Input(
            "the revocation certificate is not for this key".into(),
        ));
    }
    cert.to_armored_string(ArmorOptions::default())
        .map_err(|e| CryptoError::Parse(e.to_string()))
}

/// Add a subkey for `purpose`, of the key's generation (v6: X25519 / Ed25519; v4:
/// ECDH Curve25519 / EdDSA), expiring at `expires_at` (Unix seconds) or never.
pub fn add_subkey(
    bundle: &str,
    passphrase: &str,
    purpose: SubkeyPurpose,
    expires_at: Option<u64>,
) -> Result<ManagedKey> {
    let mut ssk = parse_secret(bundle)?;
    check_passphrase(&ssk, passphrase)?;
    let version = ssk.primary_key.version();
    let key_type = match (version, purpose) {
        (KeyVersion::V6, SubkeyPurpose::Encrypt) => KeyType::X25519,
        (KeyVersion::V6, SubkeyPurpose::Sign) => KeyType::Ed25519,
        (_, SubkeyPurpose::Encrypt) => KeyType::ECDH(ECCCurve::Curve25519Legacy),
        (_, SubkeyPurpose::Sign) => KeyType::Ed25519Legacy,
    };
    let created = now_timestamp();
    let (public_params, secret_params) = key_type.generate(rng::pgp_rng()).map_err(err)?;
    let inner =
        PubKeyInner::new(version, key_type.to_alg(), created, None, public_params).map_err(err)?;
    let sub_public = PublicSubkey::from_inner(inner).map_err(err)?;
    let mut sub = SecretSubkey::new(sub_public, secret_params).map_err(err)?;

    let primary_public = ssk.primary_key.public_key().clone();
    let mut flags = KeyFlags::default();
    match purpose {
        SubkeyPurpose::Encrypt => {
            flags.set_encrypt_comms(true);
            flags.set_encrypt_storage(true);
        }
        SubkeyPurpose::Sign => flags.set_sign(true),
    }
    let mut config = base_config(&ssk, SignatureType::SubkeyBinding)?;
    config
        .hashed_subpackets
        .push(Subpacket::regular(SubpacketData::KeyFlags(flags)).map_err(err)?);
    if let Some(d) = expiry_duration(created.as_secs(), expires_at)? {
        config
            .hashed_subpackets
            .push(Subpacket::regular(SubpacketData::KeyExpirationTime(d)).map_err(err)?);
    }
    if purpose == SubkeyPurpose::Sign {
        // The subkey's own consent to the binding (back-signature), made while the
        // fresh subkey is still unlocked.
        let back = sub
            .sign_primary_key_binding(rng::pgp_rng(), &primary_public, &Password::empty())
            .map_err(err)?;
        config.hashed_subpackets.push(
            Subpacket::regular(SubpacketData::EmbeddedSignature(Box::new(back))).map_err(err)?,
        );
    }
    let binding = config
        .sign_subkey_binding(
            &ssk.primary_key,
            &primary_public,
            &pw(passphrase),
            sub.public_key(),
        )
        .map_err(err)?;
    sub.set_password(rng::pgp_rng(), &pw(passphrase))
        .map_err(err)?;
    ssk.secret_subkeys
        .push(SignedSecretSubKey::new(sub, vec![binding]));
    finish(&ssk)
}

/// Revoke one subkey (by fingerprint, upper or lower hex).
pub fn revoke_subkey(
    bundle: &str,
    passphrase: &str,
    subkey_fingerprint: &str,
    reason: RevocationReason,
    message: &str,
) -> Result<ManagedKey> {
    let mut ssk = parse_secret(bundle)?;
    check_passphrase(&ssk, passphrase)?;
    let wanted = subkey_fingerprint.to_uppercase();
    let idx = ssk
        .secret_subkeys
        .iter()
        .position(|s| cert::fingerprint_hex(&s.key) == wanted)
        .ok_or_else(|| CryptoError::Input("no subkey with that fingerprint".into()))?;
    let primary_public = ssk.primary_key.public_key().clone();
    let mut config = base_config(&ssk, SignatureType::SubkeyRevocation)?;
    config.hashed_subpackets.push(
        Subpacket::regular(SubpacketData::RevocationReason(
            reason.code(),
            pgp::bytes::Bytes::from(message.as_bytes().to_vec()),
        ))
        .map_err(err)?,
    );
    // A subkey revocation is computed over the primary and the subkey, exactly like
    // a binding (RFC 9580 §5.2.4), so the binding signer produces it.
    let sig = config
        .sign_subkey_binding(
            &ssk.primary_key,
            &primary_public,
            &pw(passphrase),
            ssk.secret_subkeys[idx].key.public_key(),
        )
        .map_err(err)?;
    ssk.secret_subkeys[idx].signatures.push(sig);
    finish(&ssk)
}

/// Change the passphrase that locks every secret component. Fails (and changes
/// nothing) when `old` does not unlock the key.
pub fn change_passphrase(bundle: &str, old: &str, new: &str) -> Result<String> {
    if new.is_empty() {
        return Err(CryptoError::Input("the new passphrase is empty".into()));
    }
    let mut ssk = parse_secret(bundle)?;
    ssk.primary_key
        .remove_password(&pw(old))
        .map_err(|e| CryptoError::Decrypt(e.to_string()))?;
    for sub in &mut ssk.secret_subkeys {
        sub.key
            .remove_password(&pw(old))
            .map_err(|e| CryptoError::Decrypt(e.to_string()))?;
    }
    lock_all(&mut ssk, new)?;
    ssk.to_armored_string(ArmorOptions::default()).map_err(err)
}
