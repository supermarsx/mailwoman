//! The per-source score, independent of HTTP and of the wall clock.
//!
//! Every function takes `now_ms`, so tests drive time directly. A source is an
//! IPv4 address or an IPv6 /64 ([`source_key`]). The score is a leaky bucket: each
//! signal adds one point and the score drains at `grace / window` points per
//! second, so `grace` misses inside one window never leave the grace allowance.
//! Above it a signal is slowed (tarpit). At `grace + blockAfter` the source is
//! blocked; each block within `repeatWindow` of the previous one doubles the
//! duration up to `maxBlock`.
use std::collections::HashMap;
use std::net::{IpAddr, Ipv6Addr};

/// The kinds of evidence. Each costs one point.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Signal {
    /// A path on the probe list (`/wp-admin`, `/.env`, …).
    ProbePath,
    /// A path at the console's depth that answered 404.
    SlugGuess,
    /// A refused admin sign-in at the real console path.
    SigninFailed,
    /// An admin API call without a valid admin session.
    Unauthenticated,
}

impl Signal {
    pub const ALL: [Signal; 4] = [
        Signal::ProbePath,
        Signal::SlugGuess,
        Signal::SigninFailed,
        Signal::Unauthenticated,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            Signal::ProbePath => "probe-path",
            Signal::SlugGuess => "slug-guess",
            Signal::SigninFailed => "admin-signin-failed",
            Signal::Unauthenticated => "admin-unauthenticated",
        }
    }
    pub fn parse(raw: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.as_str() == raw)
    }
    fn bit(self) -> u8 {
        1 << (self as u8)
    }
}

/// The numbers the tracker needs from the policy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Limits {
    pub grace: u32,
    pub window_secs: u32,
    pub block_after: u32,
    pub block_secs: u32,
    pub max_block_secs: u32,
    pub repeat_window_secs: u32,
    pub signals: u8,
}

impl Limits {
    pub fn counts(&self, signal: Signal) -> bool {
        self.signals & signal.bit() != 0
    }
    pub fn signal_mask(list: &[Signal]) -> u8 {
        list.iter().fold(0, |mask, s| mask | s.bit())
    }
    /// Points drained per millisecond. A zero grace still drains one point per window.
    fn drain_per_ms(&self) -> f64 {
        f64::from(self.grace.max(1)) / (f64::from(self.window_secs.max(1)) * 1000.0)
    }
    fn block_ms(&self, strike: u32) -> u64 {
        let doubled =
            u64::from(self.block_secs).saturating_mul(1u64 << strike.saturating_sub(1).min(20));
        doubled.min(u64::from(self.max_block_secs.max(self.block_secs))) * 1000
    }
}

/// The key a source is counted under: the IPv4 address, or the IPv6 /64.
/// IPv4-mapped IPv6 is folded to IPv4 first.
pub(crate) fn source_key(ip: IpAddr) -> IpAddr {
    match ip.to_canonical() {
        IpAddr::V4(v4) => IpAddr::V4(v4),
        IpAddr::V6(v6) => {
            let s = v6.segments();
            IpAddr::V6(Ipv6Addr::new(s[0], s[1], s[2], s[3], 0, 0, 0, 0))
        }
    }
}

/// What a signal led to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Outcome {
    /// Inside the grace allowance.
    Grace { score: f64 },
    /// Above the grace allowance: slow this answer.
    Tarpit { score: f64 },
    /// This signal started a block.
    Blocked { until_ms: u64, strike: u32 },
    /// The source was already blocked (a request that raced the block).
    AlreadyBlocked { until_ms: u64 },
}

#[derive(Clone, Debug)]
pub(crate) struct Entry {
    score: f64,
    updated_ms: u64,
    pub last_seen_ms: u64,
    pub strikes: u32,
    pub last_block_end_ms: u64,
    pub blocked_until_ms: Option<u64>,
    pub blocked_at_ms: Option<u64>,
    pub signals: u64,
    pub last_signal: Option<Signal>,
    /// Session-cookie fingerprints already counted as `Unauthenticated`, so an
    /// expired console tab that keeps polling costs one point, not one per poll.
    stale_cookies: Vec<u64>,
}

impl Entry {
    fn new(now_ms: u64) -> Self {
        Self {
            score: 0.0,
            updated_ms: now_ms,
            last_seen_ms: now_ms,
            strikes: 0,
            last_block_end_ms: 0,
            blocked_until_ms: None,
            blocked_at_ms: None,
            signals: 0,
            last_signal: None,
            stale_cookies: Vec::new(),
        }
    }
    pub fn score(&self, now_ms: u64, limits: &Limits) -> f64 {
        let elapsed = now_ms.saturating_sub(self.updated_ms) as f64;
        (self.score - elapsed * limits.drain_per_ms()).max(0.0)
    }
    fn settle(&mut self, now_ms: u64, limits: &Limits) {
        self.score = self.score(now_ms, limits);
        self.updated_ms = now_ms;
        if let Some(until) = self.blocked_until_ms
            && until <= now_ms
        {
            self.blocked_until_ms = None;
            self.blocked_at_ms = None;
            self.last_block_end_ms = until;
        }
        if self.strikes > 0
            && self.blocked_until_ms.is_none()
            && now_ms.saturating_sub(self.last_block_end_ms)
                > u64::from(limits.repeat_window_secs) * 1000
        {
            self.strikes = 0;
        }
    }
    /// Nothing left worth remembering: no score, no block, no strike.
    fn idle(&self, now_ms: u64, limits: &Limits) -> bool {
        let mut probe = self.clone();
        probe.settle(now_ms, limits);
        probe.score <= f64::EPSILON && probe.blocked_until_ms.is_none() && probe.strikes == 0
    }
}

/// The bounded table of sources.
pub(crate) struct Tracker {
    entries: HashMap<IpAddr, Entry>,
    capacity: usize,
    /// Sources dropped to keep the table bounded.
    pub evicted: u64,
}

impl Tracker {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: HashMap::new(),
            capacity: capacity.max(1),
            evicted: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// `Some(until)` while `key` is blocked.
    pub fn blocked_until(&mut self, key: IpAddr, now_ms: u64, limits: &Limits) -> Option<u64> {
        let entry = self.entries.get_mut(&key)?;
        entry.settle(now_ms, limits);
        entry.blocked_until_ms
    }

    /// Count one signal. `cookie` is a fingerprint of the admin session cookie
    /// the request carried, for [`Signal::Unauthenticated`].
    pub fn record(
        &mut self,
        key: IpAddr,
        signal: Signal,
        cookie: Option<u64>,
        now_ms: u64,
        limits: &Limits,
    ) -> Option<Outcome> {
        if !limits.counts(signal) {
            return None;
        }
        if !self.entries.contains_key(&key) {
            self.make_room(now_ms, limits);
            self.entries.insert(key, Entry::new(now_ms));
        }
        let entry = self.entries.get_mut(&key).expect("inserted above");
        entry.settle(now_ms, limits);
        entry.last_seen_ms = now_ms;
        if let Some(until) = entry.blocked_until_ms {
            return Some(Outcome::AlreadyBlocked { until_ms: until });
        }
        if let Some(fp) = cookie {
            if entry.stale_cookies.contains(&fp) {
                return Some(Outcome::Grace { score: entry.score });
            }
            if entry.stale_cookies.len() < 4 {
                entry.stale_cookies.push(fp);
            }
        }
        entry.score += 1.0;
        entry.signals += 1;
        entry.last_signal = Some(signal);
        let grace = f64::from(limits.grace);
        // `score` drains between misses, so a burst lands just under whole numbers;
        // the thresholds are compared to the nearest point.
        if entry.score + 0.5 >= grace + f64::from(limits.block_after.max(1)) {
            entry.strikes += 1;
            let until = now_ms + limits.block_ms(entry.strikes);
            entry.blocked_until_ms = Some(until);
            entry.blocked_at_ms = Some(now_ms);
            // Drained during the block, so an expired block starts inside grace.
            entry.score = grace;
            entry.stale_cookies.clear();
            return Some(Outcome::Blocked {
                until_ms: until,
                strike: entry.strikes,
            });
        }
        Some(if entry.score > grace + 0.5 {
            Outcome::Tarpit { score: entry.score }
        } else {
            Outcome::Grace { score: entry.score }
        })
    }

    /// Forget `key` entirely (a successful admin sign-in from it).
    pub fn clear(&mut self, key: IpAddr) -> bool {
        self.entries.remove(&key).is_some()
    }

    /// Lift a block; the strike count is kept so a relapse still escalates.
    pub fn unblock(&mut self, key: IpAddr, now_ms: u64) -> bool {
        let Some(entry) = self.entries.get_mut(&key) else {
            return false;
        };
        if entry.blocked_until_ms.take().is_none() {
            return false;
        }
        entry.blocked_at_ms = None;
        entry.last_block_end_ms = now_ms;
        entry.score = 0.0;
        entry.updated_ms = now_ms;
        true
    }

    /// Restore a block read back at start.
    pub fn restore(
        &mut self,
        key: IpAddr,
        until_ms: u64,
        strikes: u32,
        now_ms: u64,
        limits: &Limits,
    ) {
        if until_ms <= now_ms {
            return;
        }
        if !self.entries.contains_key(&key) {
            self.make_room(now_ms, limits);
        }
        let entry = self
            .entries
            .entry(key)
            .or_insert_with(|| Entry::new(now_ms));
        entry.blocked_until_ms = Some(until_ms);
        entry.blocked_at_ms = Some(now_ms);
        entry.strikes = strikes.max(1);
        entry.score = f64::from(limits.grace);
    }

    /// Every source, settled to `now_ms`.
    pub fn snapshot(&mut self, now_ms: u64, limits: &Limits) -> Vec<(IpAddr, Entry, f64)> {
        self.entries
            .iter_mut()
            .map(|(key, entry)| {
                entry.settle(now_ms, limits);
                (*key, entry.clone(), entry.score)
            })
            .collect()
    }

    /// Keep the table under its capacity: first drop every idle source, then the
    /// least recently seen unblocked one, and only when every source is blocked,
    /// the block that ends soonest.
    fn make_room(&mut self, now_ms: u64, limits: &Limits) {
        if self.entries.len() < self.capacity {
            return;
        }
        let before = self.entries.len();
        self.entries.retain(|_, entry| !entry.idle(now_ms, limits));
        self.evicted += (before - self.entries.len()) as u64;
        if self.entries.len() < self.capacity {
            return;
        }
        let victim = self
            .entries
            .iter()
            .filter(|(_, e)| e.blocked_until_ms.is_none())
            .min_by_key(|(_, e)| e.last_seen_ms)
            .or_else(|| {
                self.entries
                    .iter()
                    .min_by_key(|(_, e)| e.blocked_until_ms.unwrap_or(0))
            })
            .map(|(key, _)| *key);
        if let Some(key) = victim {
            self.entries.remove(&key);
            self.evicted += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> Limits {
        Limits {
            grace: 10,
            window_secs: 600,
            block_after: 10,
            block_secs: 900,
            max_block_secs: 86_400,
            repeat_window_secs: 86_400,
            signals: Limits::signal_mask(&Signal::ALL),
        }
    }
    fn ip(raw: &str) -> IpAddr {
        source_key(raw.parse().unwrap())
    }

    #[test]
    fn the_grace_allowance_is_ignored_and_grace_plus_block_after_blocks() {
        let l = limits();
        let mut t = Tracker::new(100);
        let a = ip("203.0.113.7");
        for n in 1..=10 {
            assert!(
                matches!(
                    t.record(a, Signal::ProbePath, None, 0, &l),
                    Some(Outcome::Grace { .. })
                ),
                "miss {n} is inside grace"
            );
        }
        for n in 11..20 {
            assert!(
                matches!(
                    t.record(a, Signal::SlugGuess, None, 0, &l),
                    Some(Outcome::Tarpit { .. })
                ),
                "miss {n} is slowed, not blocked"
            );
        }
        assert_eq!(t.blocked_until(a, 0, &l), None);
        assert_eq!(
            t.record(a, Signal::ProbePath, None, 0, &l),
            Some(Outcome::Blocked {
                until_ms: 900_000,
                strike: 1
            })
        );
        assert_eq!(t.blocked_until(a, 1, &l), Some(900_000));
        assert_eq!(t.blocked_until(a, 900_000, &l), None, "blocks expire");
    }

    #[test]
    fn the_score_drains_with_a_mock_clock() {
        let l = limits();
        let mut t = Tracker::new(100);
        let a = ip("198.51.100.1");
        for _ in 0..10 {
            t.record(a, Signal::ProbePath, None, 0, &l);
        }
        // One point drains every 60 s (10 per 600 s).
        assert!(matches!(
            t.record(a, Signal::ProbePath, None, 60_000, &l),
            Some(Outcome::Grace { .. })
        ));
        // A slow, steady prober at one miss a minute never leaves grace.
        for minute in 2..200u64 {
            assert!(
                matches!(
                    t.record(a, Signal::SlugGuess, None, minute * 60_000, &l),
                    Some(Outcome::Grace { .. })
                ),
                "minute {minute}"
            );
        }
        // After a quiet window the whole allowance is back.
        let later = 200 * 60_000 + 600_000;
        let e = t.snapshot(later, &l);
        assert!(e[0].2 < 1e-6, "{}", e[0].2);
        // Twenty quick misses at once still block.
        let mut last = None;
        for _ in 0..20 {
            last = t.record(a, Signal::ProbePath, None, later, &l);
        }
        assert!(matches!(last, Some(Outcome::Blocked { .. })));
    }

    #[test]
    fn repeat_blocks_back_off_and_expire() {
        let l = limits();
        let mut t = Tracker::new(100);
        let a = ip("192.0.2.9");
        let burst = |t: &mut Tracker, now| {
            let mut last = None;
            for _ in 0..20 {
                last = t.record(a, Signal::ProbePath, None, now, &l);
                if matches!(last, Some(Outcome::Blocked { .. })) {
                    break;
                }
            }
            last
        };
        assert_eq!(
            burst(&mut t, 0),
            Some(Outcome::Blocked {
                until_ms: 900_000,
                strike: 1
            })
        );
        assert_eq!(
            burst(&mut t, 1_000_000),
            Some(Outcome::Blocked {
                until_ms: 1_000_000 + 1_800_000,
                strike: 2
            })
        );
        assert_eq!(
            burst(&mut t, 3_000_000),
            Some(Outcome::Blocked {
                until_ms: 3_000_000 + 3_600_000,
                strike: 3
            })
        );
        // A day without a block forgets the strikes.
        let calm = 3_000_000 + 3_600_000 + 86_400_001;
        assert_eq!(
            burst(&mut t, calm),
            Some(Outcome::Blocked {
                until_ms: calm + 900_000,
                strike: 1
            })
        );
        // The cap holds.
        let mut now = calm + 900_000;
        let mut longest = 0;
        for _ in 0..12 {
            if let Some(Outcome::Blocked { until_ms, .. }) = burst(&mut t, now) {
                longest = until_ms - now;
                now = until_ms;
            }
        }
        assert_eq!(longest, 86_400_000);
    }

    #[test]
    fn a_clear_forgets_the_score_and_an_unblock_lifts_the_block() {
        let l = limits();
        let mut t = Tracker::new(100);
        let a = ip("203.0.113.8");
        for _ in 0..15 {
            t.record(a, Signal::SigninFailed, None, 0, &l);
        }
        assert!(t.clear(a));
        for _ in 0..10 {
            assert!(matches!(
                t.record(a, Signal::SigninFailed, None, 1, &l),
                Some(Outcome::Grace { .. })
            ));
        }
        for _ in 0..10 {
            t.record(a, Signal::SigninFailed, None, 1, &l);
        }
        assert!(t.blocked_until(a, 2, &l).is_some());
        assert!(t.unblock(a, 2));
        assert_eq!(t.blocked_until(a, 3, &l), None);
        assert!(!t.unblock(a, 3));
    }

    #[test]
    fn one_expired_console_tab_costs_one_point() {
        let l = limits();
        let mut t = Tracker::new(100);
        let a = ip("203.0.113.9");
        for _ in 0..500 {
            t.record(a, Signal::Unauthenticated, Some(42), 0, &l);
        }
        assert_eq!(t.snapshot(0, &l)[0].2, 1.0);
        assert_eq!(t.blocked_until(a, 0, &l), None);
    }

    #[test]
    fn signals_left_out_of_the_policy_cost_nothing() {
        let mut l = limits();
        l.signals = Limits::signal_mask(&[Signal::SigninFailed]);
        let mut t = Tracker::new(100);
        let a = ip("203.0.113.10");
        for _ in 0..100 {
            assert_eq!(t.record(a, Signal::SlugGuess, None, 0, &l), None);
        }
        assert_eq!(t.len(), 0);
    }

    #[test]
    fn ipv6_counts_per_64_and_mapped_ipv4_folds() {
        assert_eq!(ip("2001:db8:1:2:aaaa::1"), ip("2001:db8:1:2:bbbb::9"));
        assert_ne!(ip("2001:db8:1:2::1"), ip("2001:db8:1:3::1"));
        assert_eq!(ip("::ffff:192.0.2.1"), ip("192.0.2.1"));
    }

    #[test]
    fn the_table_stays_bounded_and_keeps_blocks_over_idle_sources() {
        let l = limits();
        let mut t = Tracker::new(4);
        let blocked = ip("192.0.2.1");
        for _ in 0..20 {
            t.record(blocked, Signal::ProbePath, None, 0, &l);
        }
        for n in 2..100u8 {
            t.record(
                ip(&format!("192.0.2.{n}")),
                Signal::ProbePath,
                None,
                u64::from(n),
                &l,
            );
            assert!(t.len() <= 4);
        }
        assert!(t.blocked_until(blocked, 100, &l).is_some());
        assert!(t.evicted > 0);
    }
}
