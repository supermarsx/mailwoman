//! Server-side pending-flow state for SSO logins (plan §3, task §1).
//!
//! An SSO flow is a redirect round-trip: [`begin`](crate::SsoLogin::begin) mints
//! per-flow secret material ([`PendingState`]) that
//! [`complete`](crate::SsoLogin::complete) needs to validate the callback — the
//! PKCE verifier + nonce (OIDC) or the AuthnRequest RequestID + RelayState (SAML).
//! That material MUST be held server-side, keyed by an **opaque short-TTL state
//! token**, and consumed exactly once (replay/CSRF defence, plan §9 R3) — never
//! trusted from the browser.
//!
//! [`PendingStore`] is an in-memory, TTL-swept, one-shot map mirroring `mw-server`'s
//! `SessionGuard`. A deployment that terminates SSO across multiple server
//! processes can back the same shape with a store row instead; e3 chooses. The
//! opaque token is 256 bits of CSPRNG entropy.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use rand::RngCore;
use serde::{Deserialize, Serialize};

/// Per-flow secret material a login round-trip must remember (plan §3). Held
/// server-side under an opaque state token; NEVER round-tripped through the browser
/// in the clear.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum PendingState {
    /// OIDC auth-code + PKCE flow.
    Oidc {
        /// The PKCE code verifier (proves the token request came from this client).
        pkce_verifier: String,
        /// The nonce bound into the auth request + checked in the ID token (replay).
        nonce: String,
        /// The opaque post-login return target to restore.
        relay_state: Option<String>,
    },
    /// SAML SP-initiated flow.
    Saml {
        /// The AuthnRequest `ID`, matched against the response `InResponseTo`
        /// (replay/CSRF).
        request_id: String,
        /// The opaque post-login return target to restore.
        relay_state: Option<String>,
    },
}

impl PendingState {
    /// The opaque post-login return target for this flow, if any.
    pub fn relay_state(&self) -> Option<&str> {
        match self {
            PendingState::Oidc { relay_state, .. } | PendingState::Saml { relay_state, .. } => {
                relay_state.as_deref()
            }
        }
    }
}

/// Mint an opaque 256-bit state token (hex). Used as the [`PendingStore`] key and
/// echoed to the IdP as the OAuth `state` / SAML `RelayState` correlator.
pub fn new_state_token() -> String {
    let mut b = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut b);
    let mut s = String::with_capacity(64);
    for byte in b {
        use std::fmt::Write as _;
        let _ = write!(s, "{byte:02x}");
    }
    s
}

struct Entry {
    state: PendingState,
    expires_at: Instant,
}

/// A one-shot, TTL-swept store for in-flight SSO [`PendingState`], keyed by the
/// opaque state token (plan §3). [`take`](PendingStore::take) removes the entry so a
/// state token can be redeemed at most once (replay defence).
pub struct PendingStore {
    ttl: Duration,
    inner: Mutex<HashMap<String, Entry>>,
}

impl PendingStore {
    /// A store whose entries expire `ttl` after insertion (login round-trips are
    /// short; a few minutes is typical).
    pub fn new(ttl: Duration) -> Self {
        PendingStore {
            ttl,
            inner: Mutex::new(HashMap::new()),
        }
    }

    /// Persist `state`, returning the freshly minted opaque state token to echo to
    /// the IdP and set as a short-TTL cookie / hidden field. Sweeps expired entries
    /// opportunistically.
    pub fn insert(&self, state: PendingState) -> String {
        let token = new_state_token();
        let expires_at = Instant::now() + self.ttl;
        let mut map = self.inner.lock().expect("PendingStore mutex poisoned");
        map.retain(|_, e| e.expires_at > Instant::now());
        map.insert(token.clone(), Entry { state, expires_at });
        token
    }

    /// Redeem a state token exactly once: returns the [`PendingState`] and removes
    /// it. Returns `None` if the token is unknown, already redeemed, or expired
    /// (all of which the caller must treat as [`SsoError::Replay`](crate::SsoError::Replay)).
    pub fn take(&self, token: &str) -> Option<PendingState> {
        let mut map = self.inner.lock().expect("PendingStore mutex poisoned");
        let entry = map.remove(token)?;
        if entry.expires_at <= Instant::now() {
            return None;
        }
        Some(entry.state)
    }

    /// Drop every expired entry (a deployment may call this on a timer).
    pub fn sweep(&self) {
        let mut map = self.inner.lock().expect("PendingStore mutex poisoned");
        map.retain(|_, e| e.expires_at > Instant::now());
    }

    /// The number of live (unexpired, unredeemed) entries — for tests/metrics.
    pub fn len(&self) -> usize {
        let mut map = self.inner.lock().expect("PendingStore mutex poisoned");
        map.retain(|_, e| e.expires_at > Instant::now());
        map.len()
    }

    /// Whether the store holds no live entries.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oidc_state() -> PendingState {
        PendingState::Oidc {
            pkce_verifier: "verifier".into(),
            nonce: "nonce".into(),
            relay_state: Some("/inbox".into()),
        }
    }

    #[test]
    fn tokens_are_opaque_and_distinct() {
        let a = new_state_token();
        let b = new_state_token();
        assert_eq!(a.len(), 64, "256-bit hex token");
        assert_ne!(a, b);
    }

    #[test]
    fn insert_then_take_returns_state_once() {
        let store = PendingStore::new(Duration::from_secs(300));
        let token = store.insert(oidc_state());
        assert_eq!(store.len(), 1);
        assert_eq!(store.take(&token), Some(oidc_state()));
        // Second redemption fails — one-shot (replay defence).
        assert_eq!(store.take(&token), None);
        assert!(store.is_empty());
    }

    #[test]
    fn unknown_token_is_none() {
        let store = PendingStore::new(Duration::from_secs(300));
        assert_eq!(store.take("deadbeef"), None);
    }

    #[test]
    fn expired_entries_are_not_returned() {
        let store = PendingStore::new(Duration::from_millis(0));
        let token = store.insert(oidc_state());
        // TTL of 0 ⇒ already expired on read.
        assert_eq!(store.take(&token), None);
    }

    #[test]
    fn pending_state_serde_and_relay() {
        let s = PendingState::Saml {
            request_id: "_abc123".into(),
            relay_state: Some("/calendar".into()),
        };
        assert_eq!(s.relay_state(), Some("/calendar"));
        let json = serde_json::to_string(&s).unwrap();
        let back: PendingState = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }
}
