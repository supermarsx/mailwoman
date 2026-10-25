//! Bounded, ephemeral counts: no raw URLs, session secrets or message IDs retained.
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

const MAX_MESSAGES: usize = 1024;
const MAX_URLS: usize = 256;

#[derive(Default)]
struct Message {
    urls: HashMap<[u8; 32], i64>,
    limited_until: i64,
}

#[derive(Default)]
struct Counts(HashMap<[u8; 32], Message>);

impl Counts {
    fn prune(&mut self, now: i64) {
        self.0.retain(|_, message| {
            message.urls.retain(|_, expires| *expires > now);
            !message.urls.is_empty() || message.limited_until > now
        });
    }

    fn record(&mut self, key: [u8; 32], url: [u8; 32], expires: i64, now: i64) {
        self.prune(now);
        if expires <= now {
            return;
        }
        if !self.0.contains_key(&key)
            && self.0.len() >= MAX_MESSAGES
            && let Some(oldest) = self
                .0
                .iter()
                .min_by_key(|(_, message)| message.urls.values().max().copied().unwrap_or(0))
                .map(|(key, _)| *key)
        {
            self.0.remove(&oldest);
        }
        let message = self.0.entry(key).or_default();
        if message.urls.contains_key(&url) || message.urls.len() < MAX_URLS {
            message
                .urls
                .entry(url)
                .and_modify(|until| *until = (*until).max(expires))
                .or_insert(expires);
        } else {
            message.limited_until = message.limited_until.max(expires);
        }
    }
}

fn counts() -> &'static Mutex<Counts> {
    static COUNTS: OnceLock<Mutex<Counts>> = OnceLock::new();
    COUNTS.get_or_init(|| Mutex::new(Counts::default()))
}

fn key(account: &str, session: &str, message: &str) -> [u8; 32] {
    let mut hash = Sha256::new();
    for value in [account, session, message] {
        hash.update((value.len() as u64).to_be_bytes());
        hash.update(value.as_bytes());
    }
    hash.finalize().into()
}

pub(super) fn record(account: &str, session: &str, message: &str, url: &str, expires: i64) {
    counts().lock().expect("tracker stats lock").record(
        key(account, session, message),
        Sha256::digest(url.as_bytes()).into(),
        expires,
        chrono::Utc::now().timestamp(),
    );
}

pub(super) fn read(account: &str, session: &str, message: &str) -> (usize, bool) {
    let now = chrono::Utc::now().timestamp();
    let mut counts = counts().lock().expect("tracker stats lock");
    counts.prune(now);
    counts
        .0
        .get(&key(account, session, message))
        .map_or((0, false), |message| {
            (message.urls.len(), message.limited_until > now)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deduplicates_and_expires_without_crossing_authorities_or_messages() {
        let mut counts = Counts::default();
        let first = key("a", "s", "m");
        counts.record(first, [1; 32], 100, 1);
        counts.record(first, [1; 32], 200, 1);
        assert_eq!(counts.0[&first].urls.len(), 1);
        for other in [
            key("b", "s", "m"),
            key("a", "new-authority", "m"),
            key("a", "s", "other"),
        ] {
            assert!(!counts.0.contains_key(&other));
        }
        counts.prune(100);
        assert_eq!(counts.0[&first].urls.len(), 1);
        counts.prune(200);
        assert!(counts.0.is_empty());
    }

    #[test]
    fn bounded_per_message_and_globally() {
        let mut counts = Counts::default();
        for i in 0..=MAX_URLS {
            counts.record([0; 32], Sha256::digest(i.to_be_bytes()).into(), 100, 1);
        }
        assert_eq!(counts.0[&[0; 32]].urls.len(), MAX_URLS);
        assert_eq!(counts.0[&[0; 32]].limited_until, 100);
        for i in 0..MAX_MESSAGES {
            counts.record(Sha256::digest(i.to_be_bytes()).into(), [1; 32], 100, 1);
        }
        assert_eq!(counts.0.len(), MAX_MESSAGES);
    }
}
