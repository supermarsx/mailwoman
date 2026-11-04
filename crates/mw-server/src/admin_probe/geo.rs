//! Counting admin-discovery probes from an unknown country (the second rule of
//! `admin_probe`, driven by `security.trust.adminGeo*`).
//!
//! A source that is not in the known tier, not an authenticated administrator,
//! and whose country has had no recent successful sign-in (and is not on the
//! admin allow list) is counted here instead of getting the generous
//! `graceAllowance`: `threshold` probes inside `window_secs` start a block
//! (`block_on_first` makes the first probe enough). Block lengths escalate
//! through `block_secs` for repeats inside `repeat_window_secs`. The block is
//! written to the same tracker as every other admin-probe block, so the Security
//! centre lists it (reason "admin discovery from unknown country"), one click
//! lifts it, it is persisted and audited like the rest.
use crate::trust_tier::GeoRules;
use std::collections::{HashMap, VecDeque};
use std::net::IpAddr;

/// The reason shown for a block made by this rule.
pub(crate) const REASON: &str = "admin discovery from unknown country";
const CAPACITY: usize = 10_000;

#[derive(Default)]
struct State {
    hits: VecDeque<u64>,
    strikes: u32,
    last_block_end_ms: u64,
}
#[derive(Default)]
pub(crate) struct Counter {
    map: HashMap<IpAddr, State>,
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    Counted { hits: usize },
    Block { strike: u32, secs: u32 },
}
impl Counter {
    pub fn hit(&mut self, key: IpAddr, now_ms: u64, rules: &GeoRules) -> Verdict {
        if !self.map.contains_key(&key) && self.map.len() >= CAPACITY {
            let horizon = u64::from(rules.window_secs) * 1000;
            self.map.retain(|_, s| {
                s.strikes > 0 || s.hits.back().is_some_and(|t| now_ms.saturating_sub(*t) <= horizon)
            });
            if self.map.len() >= CAPACITY
                && let Some(oldest) = self
                    .map
                    .iter()
                    .min_by_key(|(_, s)| s.hits.back().copied().unwrap_or(0))
                    .map(|(k, _)| *k)
            {
                self.map.remove(&oldest);
            }
        }
        let state = self.map.entry(key).or_default();
        let window = u64::from(rules.window_secs) * 1000;
        while state
            .hits
            .front()
            .is_some_and(|t| now_ms.saturating_sub(*t) > window)
        {
            state.hits.pop_front();
        }
        if state.strikes > 0
            && now_ms.saturating_sub(state.last_block_end_ms)
                > u64::from(rules.repeat_window_secs) * 1000
        {
            state.strikes = 0;
        }
        state.hits.push_back(now_ms);
        let needed = if rules.block_on_first {
            1
        } else {
            rules.threshold.max(1) as usize
        };
        if state.hits.len() < needed {
            return Verdict::Counted {
                hits: state.hits.len(),
            };
        }
        state.hits.clear();
        state.strikes += 1;
        let secs = rules.block_secs[(state.strikes as usize - 1).min(2)];
        state.last_block_end_ms = now_ms + u64::from(secs) * 1000;
        Verdict::Block {
            strike: state.strikes,
            secs,
        }
    }
}

/// A coarse label for the path that was probed. Never the path itself.
pub(crate) fn path_class(signal: super::tracker::Signal) -> &'static str {
    use super::tracker::Signal::*;
    match signal {
        ProbePath => "probe-list",
        SlugGuess => "console-path-guess",
        SigninFailed => "console-signin",
        Unauthenticated => "console-api",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trust_tier::Policy;
    fn key() -> IpAddr {
        "203.0.113.9".parse().unwrap()
    }

    #[test]
    fn threshold_probes_in_the_window_block_and_escalate() {
        let rules = Policy::default().geo;
        let mut c = Counter::default();
        assert_eq!(c.hit(key(), 0, &rules), Verdict::Counted { hits: 1 });
        assert_eq!(c.hit(key(), 1000, &rules), Verdict::Counted { hits: 2 });
        assert_eq!(
            c.hit(key(), 2000, &rules),
            Verdict::Block { strike: 1, secs: 3600 }
        );
        let t = 3_700_000;
        for i in 0..2 {
            assert!(matches!(c.hit(key(), t + i, &rules), Verdict::Counted { .. }));
        }
        assert_eq!(
            c.hit(key(), t + 2, &rules),
            Verdict::Block { strike: 2, secs: 86_400 }
        );
        let t = t + 90_000_000;
        c.hit(key(), t, &rules);
        c.hit(key(), t + 1, &rules);
        assert_eq!(
            c.hit(key(), t + 2, &rules),
            Verdict::Block { strike: 3, secs: 604_800 }
        );
        let t = t + 700_000_000;
        c.hit(key(), t, &rules);
        c.hit(key(), t + 1, &rules);
        assert_eq!(
            c.hit(key(), t + 2, &rules),
            Verdict::Block { strike: 4, secs: 604_800 },
            "repeats stay at the longest block"
        );
    }

    #[test]
    fn probes_outside_the_window_do_not_add_up_and_a_quiet_period_resets_strikes() {
        let rules = Policy::default().geo;
        let mut c = Counter::default();
        for i in 0..10 {
            assert_eq!(
                c.hit(key(), i * 601_000, &rules),
                Verdict::Counted { hits: 1 }
            );
        }
        let mut c = Counter::default();
        for t in [0, 1, 2] {
            c.hit(key(), t, &rules);
        }
        let later = 31 * 86_400_000;
        c.hit(key(), later, &rules);
        c.hit(key(), later + 1, &rules);
        assert_eq!(
            c.hit(key(), later + 2, &rules),
            Verdict::Block { strike: 1, secs: 3600 }
        );
    }

    #[test]
    fn block_on_first_is_opt_in() {
        let mut rules = Policy::default().geo;
        assert!(!rules.block_on_first);
        rules.block_on_first = true;
        let mut c = Counter::default();
        assert_eq!(
            c.hit(key(), 5, &rules),
            Verdict::Block { strike: 1, secs: 3600 }
        );
    }
}
