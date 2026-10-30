//! Deterministic request-protection decisions. Time is supplied by the caller.
//! Persistence serializes admission and observations across server replicas.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Off,
    Observe,
    Enforce,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Signal {
    Normal,
    Malformed,
    Oversized,
    CredentialRejected,
    PermissionDenied,
    BodyDeadline,
    AdminRouteMiss,
    Ignored,
}
impl Signal {
    fn bit(self) -> u8 {
        match self {
            Self::Malformed => 1,
            Self::Oversized => 2,
            Self::CredentialRejected => 4,
            Self::PermissionDenied => 8,
            Self::BodyDeadline => 16,
            Self::AdminRouteMiss => 32,
            _ => 0,
        }
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct WindowPolicy {
    pub seconds: u32,
    pub minimum_samples: u32,
    pub absolute_anomalies: u32,
    pub ratio_percent: u8,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Policy {
    pub rate_mode: Mode,
    pub adaptive_mode: Mode,
    pub rate_limit: u32,
    pub rate_window_secs: u32,
    pub short: WindowPolicy,
    pub long: WindowPolicy,
    pub normal_cap_per_bucket: u32,
    pub signals: u8,
    pub throttle_secs: u32,
    pub block_secs: u32,
    pub decay_secs: u32,
    /// Cap escalation without creating a separate decision engine.
    #[serde(default = "default_max_stage")]
    pub max_stage: u8,
}
fn default_max_stage() -> u8 {
    2
}
impl Policy {
    pub fn valid(&self) -> bool {
        let window = |value: WindowPolicy| {
            value.minimum_samples > 0
                && value.minimum_samples <= 1_000_000
                && value.absolute_anomalies <= 1_000_000
                && value.ratio_percent <= 100
        };
        self.rate_limit <= 1_000_000
            && (1..=3600).contains(&self.rate_window_secs)
            && (60..=3600).contains(&self.short.seconds)
            && (3600..=2_592_000).contains(&self.long.seconds)
            && window(self.short)
            && window(self.long)
            && (1..=1_000_000).contains(&self.normal_cap_per_bucket)
            && self.signals & !63 == 0
            && (1..=300).contains(&self.throttle_secs)
            && (1..=86400).contains(&self.block_secs)
            && (3600..=2_592_000).contains(&self.decay_secs)
            && (1..=2).contains(&self.max_stage)
    }
    pub fn off(&self) -> bool {
        self.rate_mode == Mode::Off && self.adaptive_mode == Mode::Off
    }
    pub fn retention_secs(&self) -> u32 {
        self.long
            .seconds
            .max(self.decay_secs)
            .max(self.block_secs)
            .max(self.rate_window_secs)
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum Action {
    #[default]
    Allow,
    Throttle,
    Block,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Reason {
    RequestRate,
    ShortAbsolute,
    ShortRatio,
    LongAbsolute,
    LongRatio,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Decision {
    pub action: Action,
    pub reason: Option<Reason>,
    pub enforced: bool,
    pub retry_after_secs: u32,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Counts {
    pub normal: u64,
    pub anomalous: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub short: Counts,
    pub long: Counts,
    pub short_window_secs: u32,
    pub long_window_secs: u32,
    pub active_until: i64,
    pub stage: u8,
    pub last_anomaly: Option<i64>,
    #[serde(default)]
    pub last_signal: Option<Signal>,
    #[serde(default)]
    pub rate_count: u32,
    #[serde(default)]
    pub rate_window_secs: u32,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct Bucket {
    at: i64,
    normal: u32,
    anomalous: u32,
}
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
struct Window {
    seconds: u32,
    buckets: Vec<Bucket>,
}
impl Window {
    fn prune(&mut self, seconds: u32, now: i64) {
        if self.seconds != seconds {
            self.seconds = seconds;
            self.buckets.clear();
        }
        let width = i64::from(seconds.div_ceil(64).max(1));
        self.buckets.retain(|bucket| {
            bucket.at.saturating_add(width) > now.saturating_sub(i64::from(seconds))
        });
    }
    fn record(&mut self, policy: WindowPolicy, now: i64, anomalous: bool, normal_cap: u32) {
        self.prune(policy.seconds, now);
        let width = i64::from(policy.seconds.div_ceil(64).max(1));
        let at = now.div_euclid(width) * width;
        if self.buckets.last().is_none_or(|bucket| bucket.at != at) {
            self.buckets.push(Bucket {
                at,
                normal: 0,
                anomalous: 0,
            });
        }
        let bucket = self.buckets.last_mut().expect("bucket inserted");
        if anomalous {
            bucket.anomalous = bucket.anomalous.saturating_add(1);
        } else {
            bucket.normal = bucket.normal.saturating_add(1).min(normal_cap);
        }
    }
    fn counts(&self) -> Counts {
        self.buckets
            .iter()
            .fold(Counts::default(), |mut total, bucket| {
                total.normal += u64::from(bucket.normal);
                total.anomalous += u64::from(bucket.anomalous);
                total
            })
    }
}
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct State {
    clock: i64,
    rate_start: i64,
    rate_count: u32,
    rate_window: u32,
    short: Window,
    long: Window,
    stage: u8,
    until: i64,
    last_anomaly: Option<i64>,
    reason: Option<Reason>,
    #[serde(default)]
    last_signal: Option<Signal>,
}
impl State {
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            short: self.short.counts(),
            long: self.long.counts(),
            short_window_secs: self.short.seconds,
            long_window_secs: self.long.seconds,
            active_until: self.until,
            stage: self.stage,
            last_anomaly: self.last_anomaly,
            last_signal: self.last_signal,
            rate_count: self.rate_count,
            rate_window_secs: self.rate_window,
        }
    }
    pub fn valid(&self, now: i64) -> bool {
        let window = |value: &Window| {
            value.buckets.len() <= 65
                && value.buckets.windows(2).all(|pair| pair[0].at < pair[1].at)
                && value.buckets.iter().all(|bucket| bucket.at <= self.clock)
        };
        self.clock >= 0
            && self.clock <= now.saturating_add(300)
            && self.stage <= 2
            && window(&self.short)
            && window(&self.long)
    }
    pub fn active_until(&self) -> i64 {
        self.until
    }
    pub fn stage(&self) -> u8 {
        self.stage
    }
    pub fn transition_from(&self, previous: &Self, decision: Decision, policy: &Policy) -> bool {
        decision.action != Action::Allow
            && (self.until != previous.until
                || self.stage != previous.stage
                || (decision.reason == Some(Reason::RequestRate)
                    && (previous.rate_start != self.rate_start
                        || previous.rate_count <= policy.rate_limit)))
    }
    fn time(&mut self, now: i64) -> i64 {
        self.clock = self.clock.max(now);
        self.clock
    }
    fn decision(&self, mode: Mode, now: i64) -> Decision {
        Decision {
            action: if self.stage > 1 {
                Action::Block
            } else {
                Action::Throttle
            },
            reason: self.reason,
            enforced: mode == Mode::Enforce,
            retry_after_secs: self.until.saturating_sub(now).clamp(1, i64::from(u32::MAX)) as u32,
        }
    }
    fn decay(&mut self, policy: &Policy, now: i64) {
        if self.stage > policy.max_stage {
            self.stage = policy.max_stage;
            self.until = self
                .until
                .min(now.saturating_add(i64::from(policy.throttle_secs)));
        }
        if self.until <= now
            && self
                .last_anomaly
                .is_some_and(|last| now.saturating_sub(last) >= i64::from(policy.decay_secs))
        {
            self.stage = 0;
            self.reason = None;
        }
    }
    pub fn admit(&mut self, policy: &Policy, now: i64) -> Decision {
        if policy.rate_mode == Mode::Off && policy.adaptive_mode == Mode::Off {
            return Decision::default();
        }
        let now = self.time(now);
        self.decay(policy, now);
        let adaptive = (policy.adaptive_mode != Mode::Off && self.until > now && self.stage > 0)
            .then(|| self.decision(policy.adaptive_mode, now));
        if policy.rate_mode == Mode::Off || policy.rate_limit == 0 {
            return adaptive.unwrap_or_default();
        }
        let width = i64::from(policy.rate_window_secs.max(1));
        let start = now.div_euclid(width) * width;
        if self.rate_start != start || self.rate_window != policy.rate_window_secs {
            self.rate_start = start;
            self.rate_count = 0;
            self.rate_window = policy.rate_window_secs;
        }
        self.rate_count = self
            .rate_count
            .saturating_add(1)
            .min(policy.rate_limit.saturating_add(1));
        let rate = if self.rate_count > policy.rate_limit {
            Decision {
                action: Action::Throttle,
                reason: Some(Reason::RequestRate),
                enforced: policy.rate_mode == Mode::Enforce,
                retry_after_secs: (start + width - now) as u32,
            }
        } else {
            Decision::default()
        };
        match adaptive {
            Some(decision) if decision.enforced || !rate.enforced => decision,
            _ => rate,
        }
    }
    pub fn observe(&mut self, policy: &Policy, now: i64, signal: Signal) -> Decision {
        if policy.adaptive_mode == Mode::Off || signal == Signal::Ignored {
            return Decision::default();
        }
        let anomaly = signal.bit() & policy.signals != 0;
        if signal != Signal::Normal && !anomaly {
            return Decision::default();
        }
        let now = self.time(now);
        self.decay(policy, now);
        self.short
            .record(policy.short, now, anomaly, policy.normal_cap_per_bucket);
        self.long
            .record(policy.long, now, anomaly, policy.normal_cap_per_bucket);
        if !anomaly {
            return Decision::default();
        }
        self.last_anomaly = Some(now);
        self.last_signal = Some(signal);
        if self.until > now && self.stage > 0 {
            return self.decision(policy.adaptive_mode, now);
        }
        let reason = trigger(
            self.short.counts(),
            policy.short,
            Reason::ShortAbsolute,
            Reason::ShortRatio,
        )
        .or_else(|| {
            trigger(
                self.long.counts(),
                policy.long,
                Reason::LongAbsolute,
                Reason::LongRatio,
            )
        });
        let Some(reason) = reason else {
            return Decision::default();
        };
        self.stage = self.stage.saturating_add(1).min(policy.max_stage);
        self.reason = Some(reason);
        self.until = now.saturating_add(i64::from(if self.stage == 1 {
            policy.throttle_secs
        } else {
            policy.block_secs
        }));
        self.decision(policy.adaptive_mode, now)
    }
    pub fn counts(&mut self, policy: &Policy, now: i64) -> (Counts, Counts) {
        let now = self.time(now);
        self.short.prune(policy.short.seconds, now);
        self.long.prune(policy.long.seconds, now);
        (self.short.counts(), self.long.counts())
    }
}
fn trigger(
    counts: Counts,
    policy: WindowPolicy,
    absolute: Reason,
    ratio: Reason,
) -> Option<Reason> {
    if policy.absolute_anomalies > 0 && counts.anomalous >= u64::from(policy.absolute_anomalies) {
        return Some(absolute);
    }
    let total = counts.normal + counts.anomalous;
    (policy.ratio_percent > 0
        && total >= u64::from(policy.minimum_samples)
        && counts.anomalous * 100 >= total * u64::from(policy.ratio_percent))
    .then_some(ratio)
}

// ── t97: the "what would happen" simulator ───────────────────────────────────
// A pure evaluation of one sample request against a policy, with the same rules
// as `State::admit` and `State::observe`. It never touches stored counters.

/// The sample the Adaptive protection page describes.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SimInput {
    /// Requests from this subject in the current rate window, this one included.
    #[serde(default)]
    pub requests_in_window: u32,
    /// Counts in the short window before this request.
    #[serde(default)]
    pub short: Counts,
    /// Counts in the long window before this request.
    #[serde(default)]
    pub long: Counts,
    /// What this request is (`normal` or an anomaly kind).
    #[serde(default)]
    pub signal: Option<Signal>,
    /// A throttle already ran and has not decayed (the next crossing blocks).
    #[serde(default)]
    pub previously_throttled: bool,
}
/// One rule and whether it fired.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimRule {
    pub rule: Reason,
    pub enabled: bool,
    pub value: u64,
    pub threshold: u64,
    pub fired: bool,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimOutcome {
    pub action: Action,
    /// The rule that decided, if any.
    pub reason: Option<Reason>,
    /// `false` in observe mode: the action is only recorded.
    pub enforced: bool,
    /// How long the action lasts, seconds.
    pub duration_secs: u32,
    /// Whether this request's signal counts as an anomaly under the policy.
    pub signal_counted: bool,
    pub rules: Vec<SimRule>,
}
fn ratio_value(counts: Counts) -> u64 {
    (counts.anomalous * 100)
        .checked_div(counts.normal + counts.anomalous)
        .unwrap_or(0)
}
fn sim_window(
    counts: Counts,
    w: WindowPolicy,
    adaptive_on: bool,
    anomaly: bool,
    absolute: Reason,
    ratio: Reason,
) -> [SimRule; 2] {
    let total = counts.normal + counts.anomalous;
    [
        SimRule {
            rule: absolute,
            enabled: adaptive_on && w.absolute_anomalies > 0,
            value: counts.anomalous,
            threshold: u64::from(w.absolute_anomalies),
            fired: anomaly
                && w.absolute_anomalies > 0
                && counts.anomalous >= u64::from(w.absolute_anomalies),
        },
        SimRule {
            rule: ratio,
            enabled: adaptive_on && w.ratio_percent > 0,
            value: ratio_value(counts),
            threshold: u64::from(w.ratio_percent),
            fired: anomaly
                && w.ratio_percent > 0
                && total >= u64::from(w.minimum_samples)
                && counts.anomalous * 100 >= total * u64::from(w.ratio_percent),
        },
    ]
}
/// Evaluate one sample request: the rate budget at admission, then the adaptive
/// windows once its signal is known. Both are reported; the decision follows
/// `State::admit` (an enforced adaptive action wins over an observed rate one).
pub fn simulate(policy: &Policy, input: &SimInput) -> SimOutcome {
    let signal = input.signal.unwrap_or(Signal::Normal);
    let adaptive_on = policy.adaptive_mode != Mode::Off;
    let anomaly = adaptive_on && signal.bit() & policy.signals != 0;
    let counted = |c: Counts| Counts {
        normal: c.normal + u64::from(adaptive_on && signal == Signal::Normal),
        anomalous: c.anomalous + u64::from(anomaly),
    };
    let rate_on = policy.rate_mode != Mode::Off && policy.rate_limit > 0;
    let mut rules = vec![SimRule {
        rule: Reason::RequestRate,
        enabled: rate_on,
        value: u64::from(input.requests_in_window),
        threshold: u64::from(policy.rate_limit),
        fired: rate_on && input.requests_in_window > policy.rate_limit,
    }];
    rules.extend(sim_window(
        counted(input.short),
        policy.short,
        adaptive_on,
        anomaly,
        Reason::ShortAbsolute,
        Reason::ShortRatio,
    ));
    rules.extend(sim_window(
        counted(input.long),
        policy.long,
        adaptive_on,
        anomaly,
        Reason::LongAbsolute,
        Reason::LongRatio,
    ));
    let adaptive = rules[1..].iter().find(|r| r.fired).map(|r| {
        let block = input.previously_throttled && policy.max_stage > 1;
        (
            if block {
                Action::Block
            } else {
                Action::Throttle
            },
            r.rule,
            policy.adaptive_mode == Mode::Enforce,
            if block {
                policy.block_secs
            } else {
                policy.throttle_secs
            },
        )
    });
    let rate = rules[0].fired.then_some((
        Action::Throttle,
        Reason::RequestRate,
        policy.rate_mode == Mode::Enforce,
        policy.rate_window_secs,
    ));
    let chosen = match (adaptive, rate) {
        (Some(a), Some(r)) => Some(if a.2 || !r.2 { a } else { r }),
        (a, r) => a.or(r),
    };
    let (action, reason, enforced, duration_secs) = match chosen {
        Some((action, reason, enforced, secs)) => (action, Some(reason), enforced, secs),
        None => (Action::Allow, None, false, 0),
    };
    SimOutcome {
        action,
        reason,
        enforced,
        duration_secs,
        signal_counted: anomaly,
        rules,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn policy() -> Policy {
        Policy {
            rate_mode: Mode::Off,
            adaptive_mode: Mode::Enforce,
            rate_limit: 10,
            rate_window_secs: 60,
            short: WindowPolicy {
                seconds: 60,
                minimum_samples: 10,
                absolute_anomalies: 10,
                ratio_percent: 50,
            },
            long: WindowPolicy {
                seconds: 7 * 86400,
                minimum_samples: 100,
                absolute_anomalies: 3,
                ratio_percent: 50,
            },
            normal_cap_per_bucket: 1000,
            signals: 7,
            throttle_secs: 3,
            block_secs: 300,
            decay_secs: 3600,
            max_stage: 2,
        }
    }
    #[test]
    fn throttle_cap_prevents_repeat_escalation_and_downgrades_existing_blocks() {
        let mut policy = policy();
        policy.max_stage = 1;
        policy.short.absolute_anomalies = 1;
        let mut state = State::default();
        for now in [10, 20, 30, 40] {
            assert_eq!(
                state.observe(&policy, now, Signal::Malformed).action,
                Action::Throttle
            );
            assert_eq!(state.admit(&policy, now + 1).action, Action::Throttle);
        }
        state.stage = 2;
        state.until = 10_000;
        let decision = state.admit(&policy, 50);
        assert_eq!(decision.action, Action::Throttle);
        assert!(decision.retry_after_secs <= policy.throttle_secs);
    }
    #[test]
    fn sparse_anomalies_accumulate_over_days_without_short_bursts() {
        let mut state = State::default();
        let policy = policy();
        for day in [0, 2] {
            assert_eq!(
                state
                    .observe(&policy, day * 86400, Signal::Malformed)
                    .action,
                Action::Allow
            );
        }
        let decision = state.observe(&policy, 4 * 86400, Signal::Malformed);
        assert_eq!(decision.reason, Some(Reason::LongAbsolute));
        assert!(decision.enforced);
        assert_eq!(state.counts(&policy, 4 * 86400).0.anomalous, 1);
    }
    #[test]
    fn normal_padding_cannot_hide_an_independent_absolute_budget() {
        let mut state = State::default();
        let policy = policy();
        for second in 0..10000 {
            state.observe(&policy, second, Signal::Normal);
        }
        for second in [10001, 10002] {
            state.observe(&policy, second, Signal::Malformed);
        }
        assert_eq!(
            state.observe(&policy, 10003, Signal::Malformed).reason,
            Some(Reason::LongAbsolute)
        );
    }
    #[test]
    fn ratio_requires_minimum_samples() {
        let mut state = State::default();
        let mut policy = policy();
        policy.short.absolute_anomalies = 0;
        policy.long.absolute_anomalies = 0;
        for _ in 0..9 {
            assert_eq!(
                state.observe(&policy, 1, Signal::Oversized).action,
                Action::Allow
            );
        }
        assert_eq!(
            state.observe(&policy, 1, Signal::Oversized).reason,
            Some(Reason::ShortRatio)
        );
    }
    #[test]
    fn active_action_deadline_does_not_slide_and_quiet_time_recovers() {
        let mut state = State::default();
        let policy = policy();
        for _ in 0..3 {
            state.observe(&policy, 1, Signal::CredentialRejected);
        }
        assert_eq!(state.admit(&policy, 2).retry_after_secs, 2);
        state.observe(&policy, 2, Signal::CredentialRejected);
        assert_eq!(state.admit(&policy, 3).retry_after_secs, 1);
        assert_eq!(state.admit(&policy, 4).action, Action::Allow);
        state.observe(&policy, 4, Signal::CredentialRejected);
        assert_eq!(state.admit(&policy, 5).action, Action::Block);
        assert_eq!(state.admit(&policy, 4000).action, Action::Allow);
        assert_eq!(
            state
                .observe(&policy, 4000, Signal::CredentialRejected)
                .action,
            Action::Throttle
        );
    }
    #[test]
    fn off_and_ignored_capacity_responses_do_not_mutate_state() {
        let mut state = State::default();
        let mut policy = policy();
        policy.adaptive_mode = Mode::Off;
        let original = state.clone();
        state.admit(&policy, 10);
        state.observe(&policy, 10, Signal::Malformed);
        assert_eq!(state, original);
        policy.adaptive_mode = Mode::Enforce;
        state.observe(&policy, 10, Signal::Ignored);
        assert_eq!(state, original);
    }
    #[test]
    fn windows_and_learning_are_bounded_and_expire() {
        let mut state = State::default();
        let policy = policy();
        for second in 0..10000 {
            state.observe(&policy, second * 1000, Signal::Normal);
        }
        assert!(state.short.buckets.len() <= 65);
        assert!(state.long.buckets.len() <= 65);
        assert_eq!(
            state.counts(&policy, 11000000),
            (Counts::default(), Counts::default())
        );
    }
    #[test]
    fn observe_mode_reports_without_blocking() {
        let mut state = State::default();
        let mut policy = policy();
        policy.adaptive_mode = Mode::Observe;
        for _ in 0..2 {
            state.observe(&policy, 1, Signal::Malformed);
        }
        let decision = state.observe(&policy, 1, Signal::Malformed);
        assert_eq!(decision.action, Action::Throttle);
        assert!(!decision.enforced);
    }
    #[test]
    fn observation_never_overrides_an_independent_enforced_limit() {
        let mut state = State::default();
        let mut policy = policy();
        policy.adaptive_mode = Mode::Observe;
        policy.rate_mode = Mode::Enforce;
        policy.rate_limit = 1;
        for _ in 0..3 {
            state.observe(&policy, 1, Signal::Malformed);
        }
        state.admit(&policy, 1);
        let decision = state.admit(&policy, 1);
        assert!(decision.enforced);
        assert_eq!(decision.reason, Some(Reason::RequestRate));
        policy.adaptive_mode = Mode::Enforce;
        policy.rate_mode = Mode::Observe;
        let decision = state.admit(&policy, 1);
        assert!(decision.enforced);
        assert_eq!(decision.reason, Some(Reason::LongAbsolute));
    }
    #[test]
    fn simulator_agrees_with_the_live_state_machine() {
        let policy = policy();
        // The live machine and the simulator pick the same action and rule for
        // the next malformed request after `previous` earlier ones.
        for previous in [0_i64, 1, 2, 5] {
            let mut state = State::default();
            for i in 0..previous {
                state.observe(&policy, i, Signal::Malformed);
            }
            let throttled = state.stage() > 0;
            let (short, long) = state.counts(&policy, previous);
            let live = state.observe(&policy, previous, Signal::Malformed);
            let sim = simulate(
                &policy,
                &SimInput {
                    requests_in_window: 1,
                    short,
                    long,
                    signal: Some(Signal::Malformed),
                    previously_throttled: throttled,
                },
            );
            assert!(sim.signal_counted);
            if !throttled {
                assert_eq!((sim.action, sim.reason), (live.action, live.reason));
            }
        }
        let quiet = simulate(
            &policy,
            &SimInput {
                requests_in_window: 1,
                signal: Some(Signal::Normal),
                ..Default::default()
            },
        );
        assert_eq!(quiet.action, Action::Allow);
        assert!(!quiet.signal_counted);
        let sample = |previously_throttled| SimInput {
            requests_in_window: 1,
            long: Counts {
                normal: 0,
                anomalous: 2,
            },
            signal: Some(Signal::Malformed),
            previously_throttled,
            ..Default::default()
        };
        let blocked = simulate(&policy, &sample(true));
        assert_eq!(
            (blocked.action, blocked.reason, blocked.duration_secs),
            (Action::Block, Some(Reason::LongAbsolute), 300)
        );
        let mut observe = policy.clone();
        observe.adaptive_mode = Mode::Observe;
        let observed = simulate(&observe, &sample(false));
        assert_eq!(
            (observed.action, observed.enforced, observed.duration_secs),
            (Action::Throttle, false, 3)
        );
        let mut rate = policy.clone();
        rate.rate_mode = Mode::Enforce;
        let limited = simulate(
            &rate,
            &SimInput {
                requests_in_window: 11,
                signal: Some(Signal::Normal),
                ..Default::default()
            },
        );
        assert_eq!(
            (limited.action, limited.reason, limited.enforced),
            (Action::Throttle, Some(Reason::RequestRate), true)
        );
        // An anomaly kind the policy does not select is not counted.
        let ignored = simulate(
            &policy,
            &SimInput {
                requests_in_window: 1,
                long: Counts {
                    normal: 0,
                    anomalous: 9,
                },
                signal: Some(Signal::PermissionDenied),
                ..Default::default()
            },
        );
        assert_eq!(ignored.action, Action::Allow);
    }
}
