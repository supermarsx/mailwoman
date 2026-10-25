//! Autocrypt Level 1 (https://autocrypt.org/level1.html): the `Autocrypt:` header
//! a sender puts on outgoing mail, `Autocrypt-Gossip:` headers inside an encrypted
//! message, and the encryption recommendation (§2.4).
//!
//! Header values follow §2.1: `addr` and `keydata` are required; `prefer-encrypt`
//! is `mutual` or absent; an unknown attribute whose name does not start with `_`
//! makes the whole header invalid (a "critical" attribute this version does not
//! know), while `_`-prefixed ones are ignored. The key sent is reduced to what
//! §2.1.1 asks for: the primary key, one encryption subkey, the User ID for the
//! address, and their newest self-signatures.

use base64::Engine;
use pgp::composed::{ArmorOptions, Deserializable, SignedKeyDetails, SignedPublicKey};
use pgp::types::SignedUser;
use serde::{Deserialize, Serialize};

use super::{build_crypto_key, cert, parse_public, to_binary};
use crate::clock;
use crate::error::{CryptoError, Result};
use crate::types::CryptoKey;

/// A parsed `Autocrypt:` or `Autocrypt-Gossip:` header.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutocryptHeader {
    /// The `addr` attribute, lower-cased.
    pub addr: String,
    /// `prefer-encrypt=mutual` was present (always `false` for gossip).
    pub prefer_encrypt: bool,
    pub key: CryptoKey,
}

/// What Autocrypt knows about a correspondent, for [`recommend`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerState {
    /// Unix time of the newest message seen from the peer.
    pub last_seen: u64,
    /// Unix time of the newest message that carried an `Autocrypt:` header.
    pub autocrypt_timestamp: Option<u64>,
    /// A key from the peer's own `Autocrypt:` header is on record.
    pub has_autocrypt_key: bool,
    /// A key from someone else's `Autocrypt-Gossip:` header is on record.
    pub has_gossip_key: bool,
    /// The peer's last `Autocrypt:` header said `prefer-encrypt=mutual`.
    pub prefer_encrypt: bool,
}

/// Autocrypt §2.4: a key not seen in a header for 35 days before the peer's last
/// message is stale.
const STALE_SECS: u64 = 35 * 24 * 60 * 60;

/// The §2.4 recommendation for one recipient: `"disable"` (no key), `"discourage"`
/// (stale or gossip-only key), `"available"`, or `"encrypt"` (both sides prefer
/// encryption).
pub fn recommend(peer: Option<&PeerState>, own_prefer_encrypt: bool) -> &'static str {
    let Some(p) = peer else {
        return "disable";
    };
    if !p.has_autocrypt_key && !p.has_gossip_key {
        return "disable";
    }
    if !p.has_autocrypt_key {
        return "discourage";
    }
    let stale = p
        .autocrypt_timestamp
        .is_none_or(|t| p.last_seen > t + STALE_SECS);
    if stale {
        return "discourage";
    }
    if own_prefer_encrypt && p.prefer_encrypt {
        "encrypt"
    } else {
        "available"
    }
}

/// The recommendation for a whole message (§2.4.2): the weakest per-recipient
/// value, except that every recipient on `encrypt` gives `encrypt`.
pub fn recommend_message(per_recipient: &[&str]) -> &'static str {
    let order = ["disable", "discourage", "available", "encrypt"];
    let weakest = per_recipient
        .iter()
        .filter_map(|r| order.iter().position(|o| o == r))
        .min();
    match weakest {
        None => "disable",
        Some(i) => order[i],
    }
}

/// An `Autocrypt:` header value (`addr=...; [prefer-encrypt=mutual; ]keydata=...`)
/// for `public_key_armored`, with the key reduced to the §2.1.1 minimum for `addr`.
/// The value is unfolded; the mail builder folds it.
pub fn autocrypt_header(
    public_key_armored: &str,
    addr: &str,
    prefer_encrypt: bool,
) -> Result<String> {
    let keydata = minimal_keydata(public_key_armored, addr)?;
    let prefer = if prefer_encrypt {
        "prefer-encrypt=mutual; "
    } else {
        ""
    };
    Ok(format!("addr={}; {prefer}keydata={keydata}", addr.trim()))
}

/// An `Autocrypt-Gossip:` header value (no `prefer-encrypt`, §2.7). Gossip headers
/// go in the headers of the encrypted inner part, one per recipient.
pub fn gossip_header(public_key_armored: &str, addr: &str) -> Result<String> {
    let keydata = minimal_keydata(public_key_armored, addr)?;
    Ok(format!("addr={}; keydata={keydata}", addr.trim()))
}

fn minimal_keydata(public_key_armored: &str, addr: &str) -> Result<String> {
    let cert = parse_public(public_key_armored)?;
    let minimal = minimal_key(&cert, addr, clock::now_secs())?;
    Ok(base64::engine::general_purpose::STANDARD.encode(to_binary(&minimal)?))
}

/// Reduce `cert` to the primary key, its newest direct-key signature, the User ID
/// carrying `addr` (or the primary User ID) with its newest self-signature, one
/// usable encryption subkey with its newest binding, and any key revocations.
pub fn minimal_key(cert: &SignedPublicKey, addr: &str, now: u64) -> Result<SignedPublicKey> {
    let primary = &cert.primary_key;
    let eval = cert::evaluate(cert, now);
    let wanted = addr.trim().to_lowercase();

    let direct: Vec<_> = cert::newest_direct(&cert.details.direct_signatures, primary, now)
        .cloned()
        .into_iter()
        .collect();

    let pick_user = |pred: &dyn Fn(&cert::UserEval) -> bool| {
        cert.details
            .users
            .iter()
            .zip(&eval.users)
            .find(|(_, e)| e.state == cert::State::Valid && pred(e))
            .and_then(|(u, _)| {
                cert::newest_user_binding(u, primary, now).map(|sig| SignedUser {
                    id: u.id.clone(),
                    signatures: vec![sig.clone()],
                })
            })
    };
    let user = pick_user(&|e| e.address.as_deref() == Some(wanted.as_str()))
        .or_else(|| pick_user(&|e| e.primary));

    let sub = eval
        .subkeys
        .iter()
        .filter(|s| {
            s.state == cert::State::Valid && (s.flags.encrypt_comms() || s.flags.encrypt_storage())
        })
        .max_by_key(|s| pgp::types::KeyDetails::created_at(&s.key.key).as_secs())
        .and_then(|s| {
            cert::newest_subkey_binding(&s.key.signatures, primary, &s.key.key, now).map(|b| {
                pgp::composed::SignedPublicSubKey {
                    key: s.key.key.clone(),
                    signatures: vec![b.clone()],
                }
            })
        });
    if sub.is_none() && !(eval.flags.encrypt_comms() || eval.flags.encrypt_storage()) {
        return Err(CryptoError::Input(
            "the key has no usable encryption key for Autocrypt".into(),
        ));
    }

    Ok(SignedPublicKey {
        primary_key: primary.clone(),
        details: SignedKeyDetails {
            revocation_signatures: cert.details.revocation_signatures.clone(),
            direct_signatures: direct,
            users: user.into_iter().collect(),
            user_attributes: Vec::new(),
        },
        public_subkeys: sub.into_iter().collect(),
    })
}

/// Parse an `Autocrypt:` header value (§2.1).
pub fn parse_autocrypt(header_value: &str) -> Result<AutocryptHeader> {
    parse_value(header_value, false)
}

/// Parse an `Autocrypt-Gossip:` header value (§2.7; `prefer-encrypt` is ignored).
pub fn parse_gossip(header_value: &str) -> Result<AutocryptHeader> {
    parse_value(header_value, true)
}

/// Parse an Autocrypt Level 1 header value into a [`CryptoKey`] (`source =
/// "autocrypt-header"`).
pub fn parse_autocrypt_header(header_value: &str) -> Result<CryptoKey> {
    parse_autocrypt(header_value).map(|h| h.key)
}

/// Every valid `Autocrypt-Gossip:` header in an RFC 5322 header block (the
/// decrypted inner part's headers). Invalid ones are skipped.
pub fn gossip_from_headers(header_block: &str) -> Vec<AutocryptHeader> {
    crate::mime::header_values(header_block, "autocrypt-gossip")
        .iter()
        .filter_map(|v| parse_gossip(v).ok())
        .collect()
}

fn parse_value(header_value: &str, gossip: bool) -> Result<AutocryptHeader> {
    let mut addr = None;
    let mut keydata = None;
    let mut prefer_encrypt = false;
    for part in header_value.split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (name, value) = part.split_once('=').ok_or_else(|| {
            CryptoError::Parse(format!("autocrypt: malformed attribute {part:?}"))
        })?;
        let name = name.trim().to_ascii_lowercase();
        let value = value.trim();
        match name.as_str() {
            "addr" => addr = Some(value.to_lowercase()),
            "keydata" => keydata = Some(value.split_whitespace().collect::<String>()),
            "prefer-encrypt" => prefer_encrypt = !gossip && value.eq_ignore_ascii_case("mutual"),
            n if n.starts_with('_') => {}
            n => {
                return Err(CryptoError::Parse(format!(
                    "autocrypt: unknown critical attribute {n:?}"
                )));
            }
        }
    }
    let addr = addr.ok_or_else(|| CryptoError::Parse("autocrypt: no addr".into()))?;
    let keydata = keydata.ok_or_else(|| CryptoError::Parse("autocrypt: no keydata".into()))?;
    let der = base64::engine::general_purpose::STANDARD
        .decode(keydata.as_bytes())
        .map_err(|e| CryptoError::Parse(e.to_string()))?;
    let cert = SignedPublicKey::from_bytes(std::io::Cursor::new(der))
        .map_err(|e| CryptoError::Parse(e.to_string()))?;
    let armored = cert
        .to_armored_string(ArmorOptions::default())
        .map_err(|e| CryptoError::Parse(e.to_string()))?;
    let source = if gossip {
        "autocrypt-gossip"
    } else {
        "autocrypt-header"
    };
    let mut key = build_crypto_key(&cert, &armored, vec![addr.clone()], false, source);
    key.autocrypt = true;
    Ok(AutocryptHeader {
        addr,
        prefer_encrypt,
        key,
    })
}
