//! Zero-access device-pairing relay (SPEC §9.1) — the in-memory store behind
//! `POST /api/zeroaccess/pair/offer`, `POST /api/zeroaccess/pair/envelope` and
//! `GET /api/zeroaccess/pair/envelope/{id}`.
//!
//! The ceremony: the NEW device asks for an offer (it gets a random pairing id);
//! an EXISTING device of the same account seals the root key to the new device's
//! ephemeral public point and deposits the envelope under that id; the new device
//! polls the id until the envelope is there. The envelope is ciphertext only the
//! new device's in-worker ephemeral secret opens.
//!
//! Rules this type enforces (t56):
//! * an id exists only once the offer route minted it — a deposit under any other
//!   id is refused, so the map cannot be filled with caller-chosen ids;
//! * a deposit is accepted only from the account that made the offer;
//! * a read is answered only to the session that made the offer, and the first
//!   read that returns an envelope removes the entry (single use);
//! * an entry lives [`PAIRING_TTL`] from its offer;
//! * an account holds at most [`MAX_PER_ACCOUNT`] live offers (a new one evicts
//!   its oldest) and the whole map at most [`MAX_TOTAL`];
//! * an envelope is at most [`MAX_ENVELOPE_LEN`] bytes of base64.
//!
//! Unknown, expired, foreign and already-read ids all answer the same way
//! ([`PairingError::NotFound`]), so a caller learns nothing about ids it does
//! not own.
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// How long an offer (and the envelope deposited under it) is kept.
pub(crate) const PAIRING_TTL: Duration = Duration::from_secs(10 * 60);
/// Live offers per account; a further offer evicts that account's oldest.
pub(crate) const MAX_PER_ACCOUNT: usize = 4;
/// Live offers in the process.
pub(crate) const MAX_TOTAL: usize = 4096;
/// Largest accepted envelope (base64 text). A sealed root key is about 100 bytes.
pub(crate) const MAX_ENVELOPE_LEN: usize = 16 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PairingError {
    /// The id is unknown, expired, already read, or belongs to someone else.
    NotFound,
    /// The relay holds [`MAX_TOTAL`] live offers.
    Full,
    /// The envelope is longer than [`MAX_ENVELOPE_LEN`].
    TooLarge,
}

struct Entry {
    account_id: String,
    session_id: String,
    created: Instant,
    envelope: Option<String>,
}

#[derive(Default)]
pub(crate) struct PairingRelay {
    entries: Mutex<HashMap<String, Entry>>,
}

impl PairingRelay {
    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, Entry>> {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn prune(entries: &mut HashMap<String, Entry>, now: Instant) {
        entries.retain(|_, e| now.saturating_duration_since(e.created) < PAIRING_TTL);
    }

    /// Record an offer under the fresh random `id` for this account and session.
    pub(crate) fn offer(
        &self,
        id: String,
        account_id: &str,
        session_id: &str,
        now: Instant,
    ) -> Result<(), PairingError> {
        let mut entries = self.lock();
        Self::prune(&mut entries, now);
        let mut own: Vec<(Instant, String)> = entries
            .iter()
            .filter(|(_, e)| e.account_id == account_id)
            .map(|(k, e)| (e.created, k.clone()))
            .collect();
        if own.len() >= MAX_PER_ACCOUNT {
            own.sort();
            for (_, k) in own.iter().take(own.len() + 1 - MAX_PER_ACCOUNT) {
                entries.remove(k);
            }
        }
        if entries.len() >= MAX_TOTAL {
            return Err(PairingError::Full);
        }
        entries.insert(
            id,
            Entry {
                account_id: account_id.to_owned(),
                session_id: session_id.to_owned(),
                created: now,
                envelope: None,
            },
        );
        Ok(())
    }

    /// Deposit the sealed envelope under an offered id (any session of the
    /// offering account; a later deposit replaces an unread one).
    pub(crate) fn deposit(
        &self,
        id: &str,
        account_id: &str,
        envelope: String,
        now: Instant,
    ) -> Result<(), PairingError> {
        if envelope.len() > MAX_ENVELOPE_LEN {
            return Err(PairingError::TooLarge);
        }
        let mut entries = self.lock();
        Self::prune(&mut entries, now);
        match entries.get_mut(id) {
            Some(e) if e.account_id == account_id => {
                e.envelope = Some(envelope);
                Ok(())
            }
            _ => Err(PairingError::NotFound),
        }
    }

    /// The offering session's poll: `Ok(None)` while nothing is deposited,
    /// `Ok(Some(_))` once (the entry is then removed).
    pub(crate) fn take(
        &self,
        id: &str,
        session_id: &str,
        now: Instant,
    ) -> Result<Option<String>, PairingError> {
        let mut entries = self.lock();
        Self::prune(&mut entries, now);
        match entries.get(id) {
            Some(e) if e.session_id == session_id => {}
            _ => return Err(PairingError::NotFound),
        }
        if entries.get(id).is_some_and(|e| e.envelope.is_some()) {
            Ok(entries.remove(id).and_then(|e| e.envelope))
        } else {
            Ok(None)
        }
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.lock().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn relay_with_offer(now: Instant) -> PairingRelay {
        let relay = PairingRelay::default();
        relay.offer("p1".into(), "alice", "alice-new", now).unwrap();
        relay
    }

    #[test]
    fn happy_path_other_session_of_same_account_deposits_offering_session_reads_once() {
        let now = Instant::now();
        let relay = relay_with_offer(now);
        assert_eq!(
            relay.take("p1", "alice-new", now),
            Ok(None),
            "nothing deposited yet"
        );
        relay.deposit("p1", "alice", "ZW52".into(), now).unwrap();
        assert_eq!(relay.take("p1", "alice-new", now), Ok(Some("ZW52".into())));
        assert_eq!(
            relay.take("p1", "alice-new", now),
            Err(PairingError::NotFound),
            "single use: the entry is gone after the envelope is read"
        );
    }

    #[test]
    fn another_user_can_neither_read_nor_deposit() {
        let now = Instant::now();
        let relay = relay_with_offer(now);
        relay.deposit("p1", "alice", "ZW52".into(), now).unwrap();
        assert_eq!(
            relay.take("p1", "mallory-session", now),
            Err(PairingError::NotFound)
        );
        assert_eq!(
            relay.deposit("p1", "mallory", "ZXZpbA==".into(), now),
            Err(PairingError::NotFound),
            "a foreign account cannot replace the envelope"
        );
        assert_eq!(
            relay.take("p1", "alice-new", now),
            Ok(Some("ZW52".into())),
            "the foreign attempts changed nothing"
        );
    }

    #[test]
    fn another_session_of_the_same_account_cannot_read() {
        let now = Instant::now();
        let relay = relay_with_offer(now);
        relay.deposit("p1", "alice", "ZW52".into(), now).unwrap();
        assert_eq!(
            relay.take("p1", "alice-old", now),
            Err(PairingError::NotFound)
        );
    }

    #[test]
    fn a_deposit_needs_a_minted_offer() {
        let now = Instant::now();
        let relay = PairingRelay::default();
        assert_eq!(
            relay.deposit("chosen-by-caller", "alice", "ZW52".into(), now),
            Err(PairingError::NotFound)
        );
        assert_eq!(relay.len(), 0, "nothing stored under a caller-chosen id");
    }

    #[test]
    fn entries_expire() {
        let now = Instant::now();
        let relay = relay_with_offer(now);
        relay.deposit("p1", "alice", "ZW52".into(), now).unwrap();
        let later = now + PAIRING_TTL;
        assert_eq!(
            relay.take("p1", "alice-new", later),
            Err(PairingError::NotFound)
        );
        assert_eq!(relay.len(), 0, "expired entries are dropped");
        let relay = relay_with_offer(now);
        assert_eq!(
            relay.deposit("p1", "alice", "ZW52".into(), later),
            Err(PairingError::NotFound)
        );
    }

    #[test]
    fn the_map_is_bounded() {
        let now = Instant::now();
        let relay = PairingRelay::default();
        for i in 0..(MAX_PER_ACCOUNT + 3) {
            relay
                .offer(
                    format!("a{i}"),
                    "alice",
                    "s",
                    now + Duration::from_millis(i as u64),
                )
                .unwrap();
        }
        assert_eq!(
            relay.len(),
            MAX_PER_ACCOUNT,
            "one account keeps its newest offers only"
        );
        assert_eq!(
            relay.take("a0", "s", now),
            Err(PairingError::NotFound),
            "oldest evicted"
        );
        let last = format!("a{}", MAX_PER_ACCOUNT + 2);
        assert_eq!(relay.take(&last, "s", now), Ok(None));

        let relay = PairingRelay::default();
        for i in 0..MAX_TOTAL {
            relay
                .offer(format!("p{i}"), &format!("acct{i}"), "s", now)
                .unwrap();
        }
        assert_eq!(
            relay.offer("one-more".into(), "late", "s", now),
            Err(PairingError::Full)
        );
        assert_eq!(relay.len(), MAX_TOTAL);
    }

    #[test]
    fn an_oversized_envelope_is_refused() {
        let now = Instant::now();
        let relay = relay_with_offer(now);
        assert_eq!(
            relay.deposit("p1", "alice", "A".repeat(MAX_ENVELOPE_LEN + 1), now),
            Err(PairingError::TooLarge)
        );
        relay
            .deposit("p1", "alice", "A".repeat(MAX_ENVELOPE_LEN), now)
            .unwrap();
    }
}
