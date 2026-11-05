//! Bounded in-memory history of request-protection decisions, one bucket per
//! minute for the last 24 hours (the same idea as `remote_image_history.rs`).
//!
//! The Aggregate and Adaptive pages chart these counts. A bucket holds only
//! counts: how many requests were allowed, throttled or refused, per budget and
//! per claimed user-agent class, per trust tier, and per adaptive rule. It never
//! holds an address, an account, a fingerprint or a user-agent string. The
//! history lives in this process only: it is empty after a restart and is not
//! shared between replicas. It is always on and has no setting; memory is
//! bounded by [`KEEP_MINUTES`] buckets with at most [`MAX_CLASSES`] user-agent
//! classes each.

use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

use mw_store::request_protection_model::{Action, Decision, Reason};
use serde_json::{Value, json};

pub(crate) const KEEP_MINUTES: usize = 24 * 60;
/// The most user-agent classes one bucket tracks; later ones fold into `other`.
const MAX_CLASSES: usize = 16;
/// The most points a series carries.
const POINTS: i64 = 60;

/// Allowed, throttled, refused.
type Triple = [u32; 3];

/// Budgets the answer is split by: the per-address, account and administrator budgets, and the user-agent budgets.
const BUDGETS: [&str; 4] = ["client", "account", "admin", "userAgent"];
/// Adaptive rules, in the order of [`rule_index`].
const RULES: [&str; 6] = [
    "shortAbsolute",
    "shortRatio",
    "longAbsolute",
    "longRatio",
    "probePath",
    "botUserAgent",
];
/// Observed and enforced throttles and blocks, in this order.
const ACTIONS: [&str; 4] = ["throttleObserved", "throttleEnforced", "blockObserved", "blockEnforced"];

#[derive(Clone, Debug, Default, PartialEq)]
struct Bucket {
    minute: i64,
    total: Triple,
    budgets: [Triple; 4],
    classes: Vec<(String, Triple)>,
    tiers: [u32; 3],
    rules: [u32; 6],
    actions: [u32; 4],
}

#[derive(Default)]
struct Ring {
    buckets: VecDeque<Bucket>,
}

fn rule_index(reason: Reason) -> Option<usize> {
    match reason {
        Reason::RequestRate => None,
        Reason::ShortAbsolute => Some(0),
        Reason::ShortRatio => Some(1),
        Reason::LongAbsolute => Some(2),
        Reason::LongRatio => Some(3),
        Reason::ProbePath => Some(4),
        Reason::BotUserAgent => Some(5),
    }
}
fn budget_index(kind: &str) -> usize {
    match kind {
        "client" => 0,
        "account" => 1,
        "admin" => 2,
        _ => 3,
    }
}
/// 0 allowed, 1 throttled, 2 refused.
fn outcome(decision: &Decision) -> usize {
    if !decision.enforced {
        return 0;
    }
    match decision.action {
        Action::Allow => 0,
        Action::Throttle => 1,
        Action::Block => 2,
    }
}

impl Ring {
    fn bucket(&mut self, now: i64) -> &mut Bucket {
        let minute = now.div_euclid(60);
        if self.buckets.back().is_none_or(|b| b.minute < minute) {
            while self.buckets.len() >= KEEP_MINUTES {
                self.buckets.pop_front();
            }
            self.buckets.push_back(Bucket { minute, ..Bucket::default() });
        }
        // A clock that stepped back keeps counting into the newest bucket.
        self.buckets.back_mut().expect("a bucket was just pushed")
    }
    fn decision(&mut self, now: i64, kind: &str, decision: &Decision) {
        let which = outcome(decision);
        let b = self.bucket(now);
        b.total[which] = b.total[which].saturating_add(1);
        let i = budget_index(kind);
        b.budgets[i][which] = b.budgets[i][which].saturating_add(1);
        if let Some(class) = kind.strip_prefix("ua-") {
            Self::class_count(b, class, which);
        }
        if decision.action != Action::Allow
            && let Some(rule) = decision.reason.and_then(rule_index)
        {
            b.rules[rule] = b.rules[rule].saturating_add(1);
            let action = match (decision.action, decision.enforced) {
                (Action::Throttle, false) => 0,
                (Action::Throttle, true) => 1,
                (_, false) => 2,
                (_, true) => 3,
            };
            b.actions[action] = b.actions[action].saturating_add(1);
        }
    }
    fn class_count(b: &mut Bucket, class: &str, which: usize) {
        let at = b.classes.iter().position(|(name, _)| name == class).or_else(|| {
            let name = if b.classes.len() < MAX_CLASSES { class } else { "other" };
            b.classes.iter().position(|(n, _)| n == name).or_else(|| {
                b.classes.push((name.to_owned(), [0; 3]));
                Some(b.classes.len() - 1)
            })
        });
        if let Some(at) = at {
            b.classes[at].1[which] = b.classes[at].1[which].saturating_add(1);
        }
    }
    fn class_refused(&mut self, now: i64, class: &str) {
        let b = self.bucket(now);
        b.total[2] = b.total[2].saturating_add(1);
        b.budgets[3][2] = b.budgets[3][2].saturating_add(1);
        Self::class_count(b, class, 2);
    }
    fn tier(&mut self, now: i64, tier: usize) {
        let b = self.bucket(now);
        b.tiers[tier.min(2)] = b.tiers[tier.min(2)].saturating_add(1);
    }
    /// The answer for the last `range_secs` seconds as of `now`.
    fn summary(&self, now: i64, range_secs: i64) -> Value {
        let range_secs = range_secs.clamp(60, (KEEP_MINUTES as i64) * 60);
        let step = ((range_secs / POINTS) / 60).max(1) * 60;
        let end = now.div_euclid(step) * step + step;
        // The oldest step is included whole, so a step wider than a minute never loses
        // buckets at the edge of the range.
        let start = (now - range_secs).div_euclid(step) * step;
        let n = ((end - start) / step) as usize;
        let mut points = vec![[0u32; 3]; n];
        let mut totals = [0u64; 3];
        let mut budgets = [[0u64; 3]; 4];
        let mut classes: Vec<(String, [u64; 3])> = Vec::new();
        let mut tiers = [0u64; 3];
        let mut rules = [0u64; 6];
        let mut action_points = vec![[0u32; 4]; n];
        for b in self.buckets.iter().filter(|b| b.minute * 60 >= start && b.minute * 60 < end) {
            let at = ((b.minute * 60 - start) / step) as usize;
            for k in 0..3 {
                points[at][k] = points[at][k].saturating_add(b.total[k]);
                totals[k] += u64::from(b.total[k]);
            }
            for (i, row) in b.budgets.iter().enumerate() {
                for k in 0..3 {
                    budgets[i][k] += u64::from(row[k]);
                }
            }
            for (name, row) in &b.classes {
                let slot = match classes.iter().position(|(n, _)| n == name) {
                    Some(slot) => slot,
                    None => {
                        classes.push((name.clone(), [0; 3]));
                        classes.len() - 1
                    }
                };
                for k in 0..3 {
                    classes[slot].1[k] += u64::from(row[k]);
                }
            }
            for k in 0..3 {
                tiers[k] += u64::from(b.tiers[k]);
            }
            for k in 0..6 {
                rules[k] += u64::from(b.rules[k]);
            }
            for k in 0..4 {
                action_points[at][k] = action_points[at][k].saturating_add(b.actions[k]);
            }
        }
        classes.sort_by(|a, b| {
            let sum = |x: &[u64; 3]| x.iter().sum::<u64>();
            sum(&b.1).cmp(&sum(&a.1)).then_with(|| a.0.cmp(&b.0))
        });
        let mut ranked: Vec<(usize, u64)> = rules.iter().copied().enumerate().filter(|(_, n)| *n > 0).collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        let action_totals: Vec<u64> = (0..4).map(|k| action_points.iter().map(|p| u64::from(p[k])).sum()).collect();
        json!({
            "rangeSecs": range_secs,
            "stepSecs": step,
            "start": start,
            "scope": "this server process since it started; not shared between replicas",
            "points": (0..n).map(|i| json!({
                "t": start + i as i64 * step,
                "allowed": points[i][0], "throttled": points[i][1], "blocked": points[i][2],
                "actions": ACTIONS.iter().enumerate().map(|(k, name)| (name.to_string(), json!(action_points[i][k]))).collect::<serde_json::Map<_, _>>(),
            })).collect::<Vec<_>>(),
            "totals": {"evaluated": totals.iter().sum::<u64>(), "allowed": totals[0], "throttled": totals[1], "blocked": totals[2]},
            "budgets": BUDGETS.iter().enumerate().map(|(i, name)| json!({
                "budget": name, "allowed": budgets[i][0], "throttled": budgets[i][1], "blocked": budgets[i][2],
            })).collect::<Vec<_>>(),
            "userAgents": classes.iter().map(|(name, row)| json!({"class": name, "allowed": row[0], "throttled": row[1], "blocked": row[2]})).collect::<Vec<_>>(),
            "tiers": {"unknown": tiers[0], "known": tiers[1], "authenticated": tiers[2]},
            "actions": ACTIONS.iter().enumerate().map(|(k, name)| (name.to_string(), json!(action_totals[k]))).collect::<serde_json::Map<_, _>>(),
            "rules": ranked.iter().map(|(i, n)| json!({"rule": RULES[*i], "count": n})).collect::<Vec<_>>(),
        })
    }
}

fn ring() -> &'static Mutex<Ring> {
    static RING: OnceLock<Mutex<Ring>> = OnceLock::new();
    RING.get_or_init(|| Mutex::new(Ring::default()))
}
fn now() -> i64 {
    chrono::Utc::now().timestamp()
}
fn with(f: impl FnOnce(&mut Ring)) {
    if let Ok(mut ring) = ring().lock() {
        f(&mut ring);
    }
}

/// One answer of a budget (`client`, `account`, `admin` or `ua-<class>`).
pub(crate) fn note_decision(kind: &str, decision: &Decision) {
    with(|r| r.decision(now(), kind, decision));
}
/// An anonymous request refused by its claimed user-agent class action.
pub(crate) fn note_class_refused(class: &str) {
    with(|r| r.class_refused(now(), class));
}
/// The trust tier of a request: 0 unknown, 1 known, 2 authenticated.
pub(crate) fn note_tier(tier: usize) {
    with(|r| r.tier(now(), tier));
}
/// The history for the last `range_secs` seconds.
pub(crate) fn summary(range_secs: i64) -> Value {
    ring().lock().map(|r| r.summary(now(), range_secs)).unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(action: Action, reason: Option<Reason>, enforced: bool) -> Decision {
        Decision { action, reason, enforced, retry_after_secs: 0 }
    }

    #[test]
    fn counts_per_budget_class_tier_and_rule_without_identity() {
        let mut ring = Ring::default();
        let t = 1_800_000_000;
        ring.decision(t, "client", &d(Action::Allow, None, false));
        ring.decision(t, "client", &d(Action::Throttle, Some(Reason::RequestRate), true));
        ring.decision(t + 1, "ua-ai-training", &d(Action::Block, Some(Reason::BotUserAgent), true));
        ring.decision(t + 2, "account", &d(Action::Throttle, Some(Reason::ShortRatio), false));
        ring.class_refused(t + 3, "ai-training");
        ring.tier(t, 0);
        ring.tier(t, 7);
        let v = ring.summary(t + 5, 900);
        assert_eq!(v["totals"], json!({"evaluated": 5, "allowed": 2, "throttled": 1, "blocked": 2}));
        assert_eq!(v["budgets"][0]["throttled"], 1);
        assert_eq!(v["budgets"][3]["blocked"], 2);
        assert_eq!(v["userAgents"][0], json!({"class": "ai-training", "allowed": 0, "throttled": 0, "blocked": 2}));
        assert_eq!(v["tiers"], json!({"unknown": 1, "known": 0, "authenticated": 1}));
        assert_eq!(v["rules"][0]["count"], 1);
        assert_eq!(v["actions"]["blockEnforced"], 1);
        assert_eq!(v["actions"]["throttleObserved"], 1);
        // Counts only: the answer has no field that could name a source.
        assert!(v.get("sources").is_none() && v.get("subjects").is_none());
    }

    #[test]
    fn series_are_bounded_aligned_and_old_buckets_fall_out() {
        let mut ring = Ring::default();
        let t = 1_800_000_000;
        for m in 0..(KEEP_MINUTES as i64 + 100) {
            ring.decision(t + m * 60, "client", &d(Action::Allow, None, false));
        }
        assert_eq!(ring.buckets.len(), KEEP_MINUTES);
        let day = ring.summary(t + (KEEP_MINUTES as i64 + 99) * 60, 86_400);
        assert!(day["points"].as_array().unwrap().len() <= POINTS as usize + 1);
        assert_eq!(day["totals"]["evaluated"].as_u64().unwrap(), KEEP_MINUTES as u64);
        let quarter = ring.summary(t + (KEEP_MINUTES as i64 + 99) * 60, 900);
        assert_eq!(quarter["stepSecs"], 60);
        let seen = quarter["totals"]["evaluated"].as_u64().unwrap();
        assert!((15..=16).contains(&seen), "{seen}");
    }

    #[test]
    fn user_agent_classes_are_capped_per_bucket() {
        let mut ring = Ring::default();
        for i in 0..40 {
            ring.decision(1_800_000_000, &format!("ua-c{i}"), &d(Action::Allow, None, false));
        }
        assert!(ring.buckets[0].classes.len() <= MAX_CLASSES + 1);
        assert!(ring.buckets[0].classes.iter().any(|(n, _)| n == "other"));
    }
}
