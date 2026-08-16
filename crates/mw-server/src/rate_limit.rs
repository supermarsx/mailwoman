//! A keyed token-bucket rate limiter with a **bounded** key space (t22-e9).
//!
//! # Why this is not the limiter in `image_proxy.rs`
//! That one is per-account and its own comment states the safety argument plainly:
//! *"The account map is bounded by the deployment's account count … no eviction
//! needed."* That is **true for an account key and false for an IP key**, and the
//! code looks identical either way. An unauthenticated caller rotates source
//! addresses — a single IPv6 /64 mints 2⁶⁴ of them — and every new key inserts a
//! bucket that is never removed. A rate limit added to stop an abuse channel would
//! then *be* an unauthenticated remote memory-exhaustion channel: a mitigation
//! strictly worse than the thing it mitigates.
//!
//! This is the third instance in 26.20 of one shape: **sound reasoning attached to
//! code, carried to a place where its premise no longer holds.** The others were the
//! ManageSieve RFC1918 allowance (legitimate for a host a user configured *with
//! credentials*, not for a domain an anonymous caller names) and the `no eviction
//! needed` comment above. The premise, not the conclusion, is what has to be
//! re-checked when logic is reused.
//!
//! So the key space here is **capacity-bounded**, and callers keying on anything
//! attacker-chosen must use this rather than an unbounded map.
//!
//! # Eviction, and why the first pass is lossless
//! A bucket that has refilled to full is **indistinguishable from a key we have
//! never seen**, because a new key starts at full burst. Dropping those therefore
//! discards no information at all, and under any realistic load it is the only
//! eviction that ever runs. Only when every retained key is actively spending does
//! the limiter fall back to dropping the oldest tenth — a lossy pass, but one that
//! costs an attacker their own budget to trigger.
//!
//! Both passes are `O(n)`, and both are amortised: the first frees every idle key at
//! once, the second frees a tenth of capacity. Evicting a *single* key per insert
//! would be `O(n)` on **every** request under exactly the key-rotation attack this
//! exists to survive, turning the fix into a CPU amplifier — the same mistake in a
//! different resource.
//!
//! State is a process-local static in the callers, so it resets on restart and is
//! per-replica: N replicas each admit the full rate. That is the same trade the
//! image proxy documents, and it is a fan-out bound rather than a security boundary.

use std::collections::HashMap;
use std::time::Instant;

/// One key's bucket.
struct TokenBucket {
    tokens: f64,
    last: Instant,
}

/// A token-bucket limiter over an arbitrary string key with a hard cap on how many
/// keys it will retain.
pub(crate) struct KeyedRateLimiter {
    buckets: HashMap<String, TokenBucket>,
    /// Most tokens a key may hold — also what a never-seen key starts with.
    burst: f64,
    /// Sustained tokens added per second.
    refill_per_sec: f64,
    /// Hard ceiling on retained keys. Reached ⇒ evict before inserting.
    capacity: usize,
}

impl KeyedRateLimiter {
    pub(crate) fn new(burst: f64, refill_per_sec: f64, capacity: usize) -> Self {
        Self {
            buckets: HashMap::new(),
            burst,
            refill_per_sec,
            capacity,
        }
    }

    /// Charge one token to `key`, refilling for elapsed time first. `true` when a
    /// token was available (allow), `false` when the bucket is spent (→ `429`).
    pub(crate) fn check(&mut self, key: &str) -> bool {
        let now = Instant::now();
        if !self.buckets.contains_key(key) {
            self.make_room(now);
        }
        let (burst, refill) = (self.burst, self.refill_per_sec);
        let b = self.buckets.entry(key.to_string()).or_insert(TokenBucket {
            tokens: burst,
            last: now,
        });
        let elapsed = now.saturating_duration_since(b.last).as_secs_f64();
        b.tokens = (b.tokens + elapsed * refill).min(burst);
        b.last = now;
        if b.tokens >= 1.0 {
            b.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Ensure there is room for one more key. See the module docs for why the first
    /// pass is lossless and why both passes are batched rather than per-insert.
    fn make_room(&mut self, now: Instant) {
        if self.buckets.len() < self.capacity {
            return;
        }
        let (burst, refill) = (self.burst, self.refill_per_sec);
        self.buckets.retain(|_, b| {
            let elapsed = now.saturating_duration_since(b.last).as_secs_f64();
            (b.tokens + elapsed * refill) < burst
        });
        if self.buckets.len() < self.capacity {
            return;
        }
        // Every retained key is still spending. Drop the oldest tenth by last use.
        //
        // By COUNT, not by a timestamp threshold. A threshold (`retain(last >
        // cutoff)`) is tie-vulnerable: if every retained bucket shared one `Instant`
        // it would drop the entire map, handing every key a fresh burst — a
        // rate-limit bypass rather than an eviction. Taking a fixed number is exact
        // whatever the clock resolution does.
        let drop_n = (self.capacity / 10).max(1);
        let mut by_age: Vec<(Instant, String)> = self
            .buckets
            .iter()
            .map(|(k, b)| (b.last, k.clone()))
            .collect();
        by_age.sort_unstable_by_key(|(t, _)| *t);
        for (_, key) in by_age.into_iter().take(drop_n) {
            self.buckets.remove(&key);
        }
    }

    /// Retained key count — the thing the capacity bound is about.
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.buckets.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spends_a_burst_then_refuses() {
        let mut rl = KeyedRateLimiter::new(3.0, 0.0, 16);
        assert!(rl.check("a"));
        assert!(rl.check("a"));
        assert!(rl.check("a"));
        assert!(!rl.check("a"), "the fourth call has no token left");
    }

    #[test]
    fn buckets_are_independent_per_key() {
        let mut rl = KeyedRateLimiter::new(1.0, 0.0, 16);
        assert!(rl.check("a"));
        assert!(!rl.check("a"));
        assert!(rl.check("b"), "b must not be charged for a's spending");
    }

    #[test]
    fn refills_over_time() {
        let mut rl = KeyedRateLimiter::new(1.0, 1_000_000.0, 16);
        assert!(rl.check("a"));
        // A huge refill rate means even a microsecond of elapsed time restores the
        // bucket; this asserts the refill path runs at all, without sleeping.
        std::thread::yield_now();
        assert!(rl.check("a"), "the bucket must refill with elapsed time");
    }

    /// The reason this type exists. Without a bound, this test would grow the map to
    /// 50 000 entries — the memory-exhaustion channel described in the module docs.
    #[test]
    fn key_space_stays_bounded_under_key_rotation() {
        const CAP: usize = 512;
        let mut rl = KeyedRateLimiter::new(20.0, 0.5, CAP);
        for i in 0..50_000 {
            rl.check(&format!("2001:db8:{i:x}::/64"));
            assert!(
                rl.len() <= CAP,
                "retained {} keys at iteration {i}, cap is {CAP} — the key space is \
                 unbounded and an attacker rotating addresses exhausts memory",
                rl.len()
            );
        }
        assert!(
            rl.len() > 0,
            "asserted before the bound so a limiter that retained NOTHING — and so \
             enforced nothing — cannot pass this test as if it were bounded"
        );
    }

    /// A full bucket carries no information, so evicting it cannot change a verdict.
    #[test]
    fn the_lossless_pass_clears_refilled_keys_without_a_lossy_drop() {
        let mut rl = KeyedRateLimiter::new(1.0, 1_000_000.0, 4);
        for k in ["a", "b", "c", "d"] {
            assert!(rl.check(k));
        }
        std::thread::yield_now();
        // All four refilled to full ⇒ indistinguishable from never-seen ⇒ the
        // lossless pass makes room and the fifth key fits.
        assert!(rl.check("e"));
        assert!(rl.len() <= 4);
    }

    /// Eviction must not hand an active abuser a fresh burst. It cannot, because it
    /// drops by OLDEST last-use and every `check` updates `last` — so the one key
    /// hammering the endpoint is always among the newest, never the oldest.
    #[test]
    fn an_actively_spending_key_is_never_the_one_evicted() {
        const CAP: usize = 32;
        let mut rl = KeyedRateLimiter::new(2.0, 0.0, CAP);
        assert!(rl.check("abuser"));
        assert!(rl.check("abuser"));
        assert!(!rl.check("abuser"), "abuser is spent");

        // Churn far more keys than capacity, with the abuser still hammering — which
        // is what an abuser does, and what keeps its `last` recent.
        for i in 0..CAP * 20 {
            rl.check(&format!("other{i}"));
            assert!(
                !rl.check("abuser"),
                "abuser regained a token at iteration {i} — eviction dropped an \
                 exhausted, ACTIVE bucket and reset its budget"
            );
        }
    }
}
