//! The signature state vocabulary shared by OpenPGP and S/MIME (t32-ext §U and
//! §10.3 W: "the same vocabulary as U").
//!
//! The frozen [`SignatureVerdict`] keeps its four `status` values for existing
//! callers; [`Verification`] carries the full state next to it, and `status` is
//! derived from the state by one table ([`status_for`]) so the two never disagree.
//!
//! States:
//!
//! | state | meaning | `status` |
//! |---|---|---|
//! | `valid` | good signature, key/certificate usable when it signed, not revoked | `verified` |
//! | `untrusted` | good signature, but the S/MIME chain reaches no trust anchor | `unverified-key` |
//! | `unknown-key` | no key or certificate to check the signature against | `unverified-key` |
//! | `key-expired` | the key or certificate was expired when it signed | `unverified-key` |
//! | `not-yet-valid` | the key or certificate was not yet valid when it signed | `unverified-key` |
//! | `revoked` | the key or certificate is revoked | `invalid` |
//! | `revocation-unchecked` | S/MIME, otherwise valid, revocation could not be checked | `verified` |
//! | `address-mismatch` | the signer's address is not the sender's | `unverified-key` |
//! | `bad-signature` | the signature does not verify (tampered, wrong key, refused algorithm) | `invalid` |
//! | `not-signed` | there is no signature | `none` |
//! | `partial` | only part of the message is signed (message-level results carry a separate `partial` flag and keep the signed part's state; this token is for callers that need one value) | `unverified-key` |
//!
//! OpenPGP has no chain: the core reports `valid` for a good signature by a usable
//! key, and the caller lowers it to "valid, unknown or untrusted key" from its own
//! keyring trust (t32-ext §U: no green state without a trusted key).

use serde::{Deserialize, Serialize};

use crate::types::SignatureVerdict;

pub const VALID: &str = "valid";
pub const UNTRUSTED: &str = "untrusted";
pub const UNKNOWN_KEY: &str = "unknown-key";
pub const KEY_EXPIRED: &str = "key-expired";
pub const NOT_YET_VALID: &str = "not-yet-valid";
pub const REVOKED: &str = "revoked";
pub const REVOCATION_UNCHECKED: &str = "revocation-unchecked";
pub const ADDRESS_MISMATCH: &str = "address-mismatch";
pub const BAD_SIGNATURE: &str = "bad-signature";
pub const NOT_SIGNED: &str = "not-signed";
pub const PARTIAL: &str = "partial";

/// A signature check with its full state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Verification {
    /// One of the states in the module table.
    pub state: String,
    /// A short machine token saying why, when the state alone does not (for
    /// `bad-signature`: `tampered`, `weak-hash`, `not-a-signing-key`,
    /// `unbound-key`, `malformed`; for S/MIME chain states the failing check).
    pub detail: Option<String>,
    /// The frozen four-state verdict, `status` derived from `state`.
    pub signature: SignatureVerdict,
    /// Primary key fingerprint (OpenPGP) or certificate SHA-256 fingerprint
    /// (S/MIME), upper-case hex.
    pub signer_fingerprint: Option<String>,
    /// The signer's addresses (OpenPGP User IDs, S/MIME rfc822Name SANs).
    pub signer_addresses: Vec<String>,
    /// RFC 3339 signing time from the signature, when it carries one.
    pub signed_at: Option<String>,
}

/// The frozen `status` for a state.
pub fn status_for(state: &str) -> &'static str {
    match state {
        VALID | REVOCATION_UNCHECKED => "verified",
        REVOKED | BAD_SIGNATURE => "invalid",
        NOT_SIGNED => "none",
        _ => "unverified-key",
    }
}

impl Verification {
    /// An empty result of `kind` (`"pgp"` | `"smime"`) in `state`.
    pub fn new(kind: &str, state: &str) -> Self {
        Verification {
            state: state.into(),
            detail: None,
            signature: SignatureVerdict {
                kind: kind.into(),
                status: status_for(state).into(),
                signer_key_id: None,
                algorithm: None,
                key_created_at: None,
                key_expires_at: None,
                chain_status: None,
                revocation_status: None,
                key_changed: false,
            },
            signer_fingerprint: None,
            signer_addresses: Vec::new(),
            signed_at: None,
        }
    }

    /// `not-signed` of `kind`.
    pub fn not_signed(kind: &str) -> Self {
        Self::new(kind, NOT_SIGNED)
    }

    /// Change the state, keeping `status` in step.
    pub fn set_state(&mut self, state: &str) {
        self.state = state.into();
        self.signature.status = status_for(state).into();
    }

    /// Set the state and its detail token.
    pub fn with(mut self, state: &str, detail: Option<&str>) -> Self {
        self.set_state(state);
        self.detail = detail.map(str::to_string);
        self
    }
}
