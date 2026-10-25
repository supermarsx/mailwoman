//! PGP/MIME end to end (RFC 3156 with RFC 9788 header protection): build the
//! signed or encrypted entity a client sends, and open or verify one it receives.
//!
//! The sender side takes the message body as a complete MIME entity (what the
//! compose view produced, attachments included) plus the message's header fields.
//! The receiver side takes the raw message and returns the inner entity to render,
//! the protected header fields, the signature state and any Autocrypt gossip.
//!
//! Sending encrypts sign-then-encrypt in one OpenPGP message (the signature is
//! inside the ciphertext, RFC 3156 §6.2), so the encrypted part is one opaque blob
//! and attachments are encrypted with the body.

use serde::{Deserialize, Serialize};

use super::autocrypt::{self, AutocryptHeader};
use crate::error::{CryptoError, Result};
use crate::mime::{self, CryptoPart, MimeEntity, ProtectedHeaders, Protection};
use crate::verdict::Verification;

/// The result of [`encrypt_mime`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuiltEntity {
    /// The complete `multipart/encrypted` or `multipart/signed` entity
    /// (`Content-Type` header, blank line, body) the server places as the message
    /// body.
    pub entity: String,
    /// The `Content-Type` header value of `entity`.
    pub content_type: String,
    /// What the outer `Subject:` should say (`[...]` when the real one is hidden
    /// inside the encryption, else the real one).
    pub outer_subject: Option<String>,
    /// 1 or 2 for an encrypted entity, absent for a signed one.
    pub seipd_version: Option<u8>,
}

/// Build a PGP/MIME encrypted entity.
///
/// - `payload`: the body entity (headers + body) to protect.
/// - `headers`: the message's header fields; the protected ones are copied into
///   the payload (RFC 9788), and `Subject` is replaced outside by `[...]`.
/// - `recipients`: armored public keys (the sender's own key should be among them
///   so the sent copy stays readable).
/// - `sign_with`: `(bundle, passphrase)` to sign inside the encryption.
/// - `gossip`: `(address, armored key)` pairs written as `Autocrypt-Gossip` into
///   the payload headers (Autocrypt §2.7: when there is more than one recipient).
pub fn encrypt_mime(
    payload: &[u8],
    headers: &[(String, String)],
    recipients: &[String],
    sign_with: Option<(&str, &str)>,
    gossip: &[(String, String)],
) -> Result<BuiltEntity> {
    let subject = headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("subject"))
        .map(|(_, v)| v.clone());
    let outer: Vec<(String, String)> = subject
        .as_ref()
        .map(|_| vec![("Subject".to_string(), mime::OBSCURED_SUBJECT.to_string())])
        .unwrap_or_default();
    let mut protected = mime::protect_headers(payload, headers, &outer, Protection::Cipher, true);
    if !gossip.is_empty() {
        protected = add_gossip(&protected, gossip)?;
    }
    let encrypted = super::encrypt_with(&protected, recipients, sign_with)?;
    let entity = mime::pgp_encrypted_entity(&encrypted.armored);
    Ok(built(
        entity,
        subject.map(|_| mime::OBSCURED_SUBJECT.to_string()),
        Some(encrypted.seipd_version),
    ))
}

/// Build a PGP/MIME signed entity (`multipart/signed`), with RFC 9788 `hp="clear"`
/// header protection on the signed part.
pub fn sign_mime(
    payload: &[u8],
    headers: &[(String, String)],
    bundle: &str,
    passphrase: &str,
) -> Result<BuiltEntity> {
    let subject = headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("subject"))
        .map(|(_, v)| v.clone());
    let protected = mime::protect_headers(payload, headers, &[], Protection::Clear, false);
    let signed_part = mime::canonicalize(&protected);
    let sig = super::sign_detached(&signed_part, bundle, passphrase)?;
    let micalg = mime::pgp_micalg(sig.as_bytes())?;
    let entity = mime::pgp_signed_entity(&signed_part, &sig, &micalg);
    Ok(built(entity, subject, None))
}

fn built(entity: MimeEntity, outer_subject: Option<String>, seipd: Option<u8>) -> BuiltEntity {
    let content_type = entity
        .header("content-type")
        .unwrap_or_default()
        .to_string();
    // Only the Content-Type travels with the entity; MIME-Version belongs to the
    // message header the server writes.
    let body_entity = MimeEntity {
        headers: vec![("Content-Type".into(), content_type.clone())],
        body: entity.body,
    };
    BuiltEntity {
        entity: String::from_utf8_lossy(&body_entity.to_bytes()).into_owned(),
        content_type,
        outer_subject,
        seipd_version: seipd,
    }
}

fn add_gossip(payload: &[u8], gossip: &[(String, String)]) -> Result<Vec<u8>> {
    let mut entity = mime::parse_entity(payload);
    let mut lines = Vec::new();
    for (addr, key) in gossip {
        lines.push((
            "Autocrypt-Gossip".to_string(),
            autocrypt::gossip_header(key, addr)?,
        ));
    }
    let mut headers = lines;
    headers.extend(entity.headers);
    entity.headers = headers;
    Ok(entity.to_bytes())
}

/// A received PGP/MIME message, opened.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Opened {
    /// The entity to render (protected headers kept, legacy display removed, any
    /// inner `multipart/signed` unwrapped to its signed part).
    pub entity: String,
    pub protected: Option<ProtectedHeaders>,
    pub verification: Verification,
    /// Whether the message was encrypted (`false` for signed-only input).
    pub encrypted: bool,
    /// SEIPDv2 was used.
    pub aead: bool,
    /// The cryptographic part covers only part of the message ("partially
    /// signed"): `verification` describes the signed part, everything outside it
    /// is unsigned.
    pub partial: bool,
    /// `Autocrypt-Gossip` headers found inside the encrypted payload.
    pub gossip: Vec<AutocryptHeader>,
}

/// Open a received message: decrypt a `multipart/encrypted` (or inline PGP
/// message) with the locked bundle, verify any signature inside or around it
/// against `signers`, read header protection and gossip.
pub fn open_mime(raw: &[u8], bundle: &str, passphrase: &str, signers: &[&str]) -> Result<Opened> {
    open_mime_at(raw, bundle, passphrase, signers, crate::clock::now_secs())
}

/// [`open_mime`] judging keys at `now`.
pub fn open_mime_at(
    raw: &[u8],
    bundle: &str,
    passphrase: &str,
    signers: &[&str],
    now: u64,
) -> Result<Opened> {
    let analysis = mime::analyze(raw);
    let ciphertext = match &analysis.part {
        Some(CryptoPart::PgpEncrypted { ciphertext }) => ciphertext.clone(),
        Some(CryptoPart::InlinePgp { segments }) => segments
            .iter()
            .find_map(|s| match s {
                mime::InlineSegment::Message { armored } => Some(armored.clone().into_bytes()),
                _ => None,
            })
            .ok_or_else(|| CryptoError::Input("the message holds no encrypted part".into()))?,
        Some(CryptoPart::PgpSigned { .. }) => {
            let mut opened = verify_mime_at(raw, signers, now)?;
            opened.encrypted = false;
            return Ok(opened);
        }
        _ => {
            return Err(CryptoError::Input(
                "the message is not PGP-encrypted".into(),
            ));
        }
    };
    let decrypted = super::decrypt_at(&ciphertext, bundle, passphrase, signers, now)?;
    let inner = decrypted.plaintext;

    // Signed-then-encrypted as two layers (a multipart/signed inside): verify it.
    let inner_analysis = mime::analyze(&inner);
    let (entity_bytes, verification) = match inner_analysis.part {
        Some(CryptoPart::PgpSigned {
            signed, signature, ..
        }) if inner_analysis.path.is_empty() => {
            let v = super::verify_detached_at(&signed, &signature, signers, now)?;
            (signed, v)
        }
        _ => (inner, decrypted.verification),
    };

    let protected = mime::unprotect_headers(&entity_bytes);
    let gossip = {
        let e = mime::parse_entity(&entity_bytes);
        let block: String = e
            .headers
            .iter()
            .map(|(n, v)| format!("{n}: {v}\r\n"))
            .collect();
        autocrypt::gossip_from_headers(&block)
    };
    let rendered = protected
        .as_ref()
        .map(|p| p.entity.clone())
        .unwrap_or_else(|| String::from_utf8_lossy(&entity_bytes).into_owned());
    Ok(Opened {
        entity: rendered,
        protected,
        verification,
        encrypted: true,
        aead: decrypted.aead,
        partial: analysis.partial,
        gossip,
    })
}

/// Verify a received signed message (`multipart/signed` or inline cleartext
/// signature) against `signers`.
pub fn verify_mime(raw: &[u8], signers: &[&str]) -> Result<Opened> {
    verify_mime_at(raw, signers, crate::clock::now_secs())
}

/// [`verify_mime`] judging keys at `now`.
pub fn verify_mime_at(raw: &[u8], signers: &[&str], now: u64) -> Result<Opened> {
    let analysis = mime::analyze(raw);
    let (entity_bytes, verification) = match &analysis.part {
        Some(CryptoPart::PgpSigned {
            signed, signature, ..
        }) => (
            signed.clone(),
            super::verify_detached_at(signed, signature, signers, now)?,
        ),
        Some(CryptoPart::InlinePgp { segments }) => {
            let armored = segments
                .iter()
                .find_map(|s| match s {
                    mime::InlineSegment::Signed { armored } => Some(armored.as_str()),
                    _ => None,
                })
                .ok_or_else(|| CryptoError::Input("the message holds no signed part".into()))?;
            let (v, text) = super::verify_cleartext_at(armored, signers, now)?;
            (text.into_bytes(), v)
        }
        _ => {
            return Ok(Opened {
                entity: String::from_utf8_lossy(raw).into_owned(),
                protected: None,
                verification: Verification::not_signed("pgp"),
                encrypted: false,
                aead: false,
                partial: false,
                gossip: Vec::new(),
            });
        }
    };
    let protected = mime::unprotect_headers(&entity_bytes);
    let rendered = protected
        .as_ref()
        .map(|p| p.entity.clone())
        .unwrap_or_else(|| String::from_utf8_lossy(&entity_bytes).into_owned());
    Ok(Opened {
        entity: rendered,
        protected,
        verification,
        encrypted: false,
        aead: false,
        partial: analysis.partial,
        gossip: Vec::new(),
    })
}
