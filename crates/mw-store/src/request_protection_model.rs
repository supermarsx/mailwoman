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
    /// High-confidence scanner evidence, independent of selected weak signals.
    ProbePath,
    BotUserAgent,
    Ignored,
}
impl Signal {
    fn strong_reason(self) -> Option<Reason> {
        match self {
            Self::ProbePath => Some(Reason::ProbePath),
            Self::BotUserAgent => Some(Reason::BotUserAgent),
            _ => None,
        }
    }
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
    ProbePath,
    BotUserAgent,
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
    /// Trusted source geolocation only; aggregate account subjects have none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
    /// The last trusted transport source. Never an authenticated identity token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_ip: Option<String>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    country_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_ip: Option<String>,
}
fn valid_country_code(country: &str) -> bool {
    country.len() == 2 && country.bytes().all(|byte| byte.is_ascii_uppercase())
}
impl State {
    /// Keep one canonical transport address, preserving known data on bad input.
    pub fn set_source_ip(&mut self, ip: &str) -> bool {
        let Ok(ip) = ip.parse::<std::net::IpAddr>() else {
            return false;
        };
        self.source_ip = Some(ip.to_canonical().to_string());
        true
    }
    /// Accept only trusted ISO-style two-letter codes, preserving known data
    /// when a lookup is unavailable or malformed. Never accepts an IP address.
    pub fn set_country_code(&mut self, country: &str) -> bool {
        if !valid_country_code(country) {
            return false;
        }
        self.country_code = Some(country.to_owned());
        true
    }
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
            country_code: self.country_code.clone(),
            source_ip: self.source_ip.clone(),
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
            && self.country_code.as_deref().is_none_or(valid_country_code)
            && self.source_ip.as_deref().is_none_or(|value| {
                value
                    .parse::<std::net::IpAddr>()
                    .is_ok_and(|ip| ip.to_canonical().to_string() == value)
            })
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
                || self.reason != previous.reason
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
    /// Inspect an existing probe action without charging or changing any state.
    /// Policy changes may downgrade the returned action, but never persist it.
    pub fn probe_decision(&self, policy: &Policy, now: i64) -> Decision {
        if policy.adaptive_mode == Mode::Off || self.reason != Some(Reason::ProbePath) {
            return Decision::default();
        }
        let mut current = self.clone();
        let now = current.time(now);
        current.decay(policy, now);
        if current.until > now && current.stage > 0 {
            current.decision(policy.adaptive_mode, now)
        } else {
            Decision::default()
        }
    }
    pub fn observe(&mut self, policy: &Policy, now: i64, signal: Signal) -> Decision {
        if policy.adaptive_mode == Mode::Off || signal == Signal::Ignored {
            return Decision::default();
        }
        let anomaly = signal.strong_reason().is_some() || signal.bit() & policy.signals != 0;
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
        if let Some(reason) = signal.strong_reason() {
            // A later bot hint cannot erase an existing, active source probe.
            if reason == Reason::BotUserAgent
                && self.reason == Some(Reason::ProbePath)
                && self.until > now
                && self.stage > 0
            {
                return self.decision(policy.adaptive_mode, now);
            }
            if self.reason != Some(reason) || self.until <= now || self.stage == 0 {
                self.stage = policy.max_stage;
                self.reason = Some(reason);
                self.until = now.saturating_add(i64::from(if self.stage > 1 {
                    policy.block_secs
                } else {
                    policy.throttle_secs
                }));
            }
            return self.decision(policy.adaptive_mode, now);
        }
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
    let anomaly =
        adaptive_on && (signal.strong_reason().is_some() || signal.bit() & policy.signals != 0);
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
    if let Some(reason) = signal.strong_reason() {
        rules.push(SimRule {
            rule: reason,
            enabled: adaptive_on,
            value: 1,
            threshold: 1,
            fired: adaptive_on,
        });
    }
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
        let block = (input.previously_throttled
            || matches!(r.rule, Reason::ProbePath | Reason::BotUserAgent))
            && policy.max_stage > 1;
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
    fn source_ip_is_canonical_validated_optional_and_legacy_compatible() {
        let old = serde_json::to_value(State::default()).unwrap();
        assert!(old.get("source_ip").is_none());
        let mut state: State = serde_json::from_value(old).unwrap();
        let snapshot = serde_json::to_value(state.snapshot()).unwrap();
        assert!(snapshot.get("sourceIp").is_none());
        assert!(
            serde_json::from_value::<Snapshot>(snapshot)
                .unwrap()
                .source_ip
                .is_none()
        );
        assert!(state.set_source_ip("::ffff:192.0.2.1"));
        assert_eq!(state.snapshot().source_ip.as_deref(), Some("192.0.2.1"));
        assert!(!state.set_source_ip("spoofed, 192.0.2.1"));
        assert_eq!(state.snapshot().source_ip.as_deref(), Some("192.0.2.1"));
        assert!(state.set_source_ip("2001:DB8:0:0:0:0:0:1"));
        assert_eq!(state.snapshot().source_ip.as_deref(), Some("2001:db8::1"));
        let encoded = serde_json::to_value(&state).unwrap();
        assert_eq!(
            serde_json::from_value::<State>(encoded.clone()).unwrap(),
            state
        );
        assert_eq!(
            serde_json::to_value(state.snapshot()).unwrap()["sourceIp"],
            "2001:db8::1"
        );
        let mut invalid = encoded;
        invalid["source_ip"] = serde_json::json!("2001:DB8::1");
        assert!(!serde_json::from_value::<State>(invalid).unwrap().valid(0));
    }

    #[test]
    fn bot_first_hit_respects_modes_and_caps_without_sliding_or_spending_rate_budget() {
        for mode in [Mode::Off, Mode::Observe, Mode::Enforce] {
            for cap in [1, 2] {
                let mut policy = policy();
                policy.signals = 0;
                policy.adaptive_mode = mode;
                policy.rate_mode = Mode::Enforce;
                policy.max_stage = cap;
                policy.block_secs = 41;
                policy.throttle_secs = 7;
                let mut state = State::default();
                state.admit(&policy, 100);
                let before = state.clone();
                let decision = state.observe(&policy, 100, Signal::BotUserAgent);
                assert_eq!(state.rate_count, before.rate_count);
                assert_eq!(state.probe_decision(&policy, 101), Decision::default());
                let simulated = simulate(
                    &policy,
                    &SimInput {
                        signal: Some(Signal::BotUserAgent),
                        requests_in_window: 1,
                        ..Default::default()
                    },
                );
                assert_eq!(
                    (
                        simulated.action,
                        simulated.reason,
                        simulated.enforced,
                        simulated.duration_secs
                    ),
                    (
                        decision.action,
                        decision.reason,
                        decision.enforced,
                        decision.retry_after_secs
                    )
                );
                if mode == Mode::Off {
                    assert_eq!(decision, Decision::default());
                    assert_eq!(state, before);
                } else {
                    assert_eq!(decision.reason, Some(Reason::BotUserAgent));
                    assert_eq!(decision.enforced, mode == Mode::Enforce);
                    assert_eq!(
                        decision.action,
                        if cap == 2 {
                            Action::Block
                        } else {
                            Action::Throttle
                        }
                    );
                    assert_eq!(decision.retry_after_secs, if cap == 2 { 41 } else { 7 });
                    let deadline = state.active_until();
                    assert_eq!(
                        state
                            .observe(&policy, 101, Signal::BotUserAgent)
                            .retry_after_secs,
                        decision.retry_after_secs - 1
                    );
                    assert_eq!(state.active_until(), deadline);
                }
            }
        }
        assert_eq!(
            serde_json::to_string(&Signal::BotUserAgent).unwrap(),
            "\"bot-user-agent\""
        );
        assert_eq!(
            serde_json::from_str::<Signal>("\"bot-user-agent\"").unwrap(),
            Signal::BotUserAgent
        );
    }

    #[test]
    fn server_bot_policy_can_upgrade_a_legacy_throttle_without_erasing_active_probe_evidence() {
        let mut legacy = policy();
        legacy.max_stage = 1;
        legacy.short.absolute_anomalies = 1;
        let mut state = State::default();
        assert_eq!(
            state.observe(&legacy, 100, Signal::Malformed).action,
            Action::Throttle
        );
        let mut bot_policy = legacy.clone();
        bot_policy.max_stage = 2;
        let decision = state.observe(&bot_policy, 101, Signal::BotUserAgent);
        assert_eq!(decision.action, Action::Block);
        assert_eq!(decision.reason, Some(Reason::BotUserAgent));
        assert_eq!(decision.retry_after_secs, bot_policy.block_secs);
        assert_eq!(state.probe_decision(&bot_policy, 102), Decision::default());
        let mut probe = State::default();
        probe.observe(&bot_policy, 100, Signal::ProbePath);
        let until = probe.active_until();
        assert_eq!(
            probe.observe(&bot_policy, 101, Signal::BotUserAgent).reason,
            Some(Reason::ProbePath)
        );
        assert_eq!(probe.active_until(), until);
        assert_eq!(
            probe.probe_decision(&bot_policy, 102).reason,
            Some(Reason::ProbePath)
        );
    }

    #[test]
    fn country_metadata_validates_round_trips_and_keeps_legacy_state_compatible() {
        let old = serde_json::to_string(&State::default()).unwrap();
        assert!(!old.contains("country"));
        let mut state: State = serde_json::from_str(&old).unwrap();
        let old_snapshot = serde_json::to_value(state.snapshot()).unwrap();
        assert!(old_snapshot.get("countryCode").is_none());
        assert!(
            serde_json::from_value::<Snapshot>(old_snapshot)
                .unwrap()
                .country_code
                .is_none()
        );
        for invalid in ["pt", "P", "PRT", "P1", "ＰＴ", "192.0.2.1"] {
            assert!(!state.set_country_code(invalid));
            assert!(state.snapshot().country_code.is_none());
        }
        assert!(state.set_country_code("PT"));
        assert!(!state.set_country_code("pt"));
        assert_eq!(state.snapshot().country_code.as_deref(), Some("PT"));
        let encoded = serde_json::to_value(&state).unwrap();
        let restored: State = serde_json::from_value(encoded.clone()).unwrap();
        assert_eq!(restored, state);
        assert_eq!(
            serde_json::to_value(restored.snapshot()).unwrap()["countryCode"],
            "PT"
        );
        let mut corrupt = encoded;
        corrupt["country_code"] = serde_json::json!("pt");
        assert!(!serde_json::from_value::<State>(corrupt).unwrap().valid(0));
    }

    #[test]
    fn first_probe_blocks_independently_of_weak_signals_without_spending_rate_budget() {
        let mut policy = policy();
        policy.signals = 0;
        policy.block_secs = 37;
        policy.rate_mode = Mode::Enforce;
        let mut state = State::default();
        assert_eq!(state.admit(&policy, 100).action, Action::Allow);
        let rate_count = state.rate_count;
        let decision = state.observe(&policy, 100, Signal::ProbePath);
        assert_eq!(
            decision,
            Decision {
                action: Action::Block,
                reason: Some(Reason::ProbePath),
                enforced: true,
                retry_after_secs: 37
            }
        );
        assert_eq!(state.stage(), 2);
        assert_eq!(state.snapshot().short.anomalous, 1);
        assert_eq!(state.snapshot().long.anomalous, 1);
        assert_eq!(state.snapshot().last_signal, Some(Signal::ProbePath));
        assert_eq!(state.rate_count, rate_count);
        assert_eq!(
            state
                .observe(&policy, 101, Signal::ProbePath)
                .retry_after_secs,
            36
        );
        assert_eq!(state.active_until(), 137);
        assert_eq!(
            serde_json::to_string(&Signal::ProbePath).unwrap(),
            "\"probe-path\""
        );
        assert_eq!(
            serde_json::from_str::<Signal>("\"probe-path\"").unwrap(),
            Signal::ProbePath
        );
    }

    #[test]
    fn probe_respects_observe_off_and_the_configured_stage_cap() {
        let mut policy = policy();
        policy.signals = 0;
        policy.adaptive_mode = Mode::Observe;
        let mut state = State::default();
        let decision = state.observe(&policy, 100, Signal::ProbePath);
        assert_eq!(decision.action, Action::Block);
        assert_eq!(decision.reason, Some(Reason::ProbePath));
        assert!(!decision.enforced);
        let before = state.clone();
        policy.adaptive_mode = Mode::Off;
        assert_eq!(
            state.observe(&policy, 101, Signal::ProbePath),
            Decision::default()
        );
        assert_eq!(state.probe_decision(&policy, 101), Decision::default());
        assert_eq!(state, before);
        policy.adaptive_mode = Mode::Enforce;
        policy.max_stage = 1;
        policy.throttle_secs = 7;
        let mut fresh = State::default();
        let decision = fresh.observe(&policy, 100, Signal::ProbePath);
        assert_eq!(
            decision,
            Decision {
                action: Action::Throttle,
                reason: Some(Reason::ProbePath),
                enforced: true,
                retry_after_secs: 7
            }
        );
        assert_eq!(fresh.stage(), 1);
    }

    #[test]
    fn probe_read_is_nonmutating_expires_and_only_returns_probe_actions() {
        let mut policy = policy();
        policy.block_secs = 37;
        let mut state = State::default();
        state.observe(&policy, 100, Signal::ProbePath);
        let original = state.clone();
        assert_eq!(state.probe_decision(&policy, 136).retry_after_secs, 1);
        assert_eq!(state.probe_decision(&policy, 137), Decision::default());
        policy.adaptive_mode = Mode::Observe;
        assert!(!state.probe_decision(&policy, 101).enforced);
        policy.max_stage = 1;
        assert_eq!(state.probe_decision(&policy, 101).action, Action::Throttle);
        assert_eq!(
            state.probe_decision(&policy, 101).retry_after_secs,
            policy.throttle_secs
        );
        assert_eq!(state, original);
        state.reason = Some(Reason::LongAbsolute);
        assert_eq!(state.probe_decision(&policy, 101), Decision::default());
    }

    #[test]
    fn probe_upgrades_active_weak_actions_and_audits_changed_evidence() {
        let policy = policy();
        let mut state = State {
            stage: 1,
            until: 110,
            reason: Some(Reason::ShortAbsolute),
            ..Default::default()
        };
        assert_eq!(
            state.observe(&policy, 100, Signal::ProbePath).action,
            Action::Block
        );
        assert_eq!(state.active_until(), 400);
        // A block that has the same duration still needs a transition when its
        // evidence changes from a statistical crossing to a known probe.
        state.reason = Some(Reason::LongAbsolute);
        let previous = state.clone();
        let decision = state.observe(&policy, 100, Signal::ProbePath);
        assert!(state.transition_from(&previous, decision, &policy));
        assert_eq!(decision.reason, Some(Reason::ProbePath));
    }

    #[test]
    fn probe_simulation_agrees_with_first_observation_in_each_mode_and_stage_cap() {
        for mode in [Mode::Off, Mode::Observe, Mode::Enforce] {
            for cap in [1, 2] {
                let mut policy = policy();
                policy.signals = 0;
                policy.adaptive_mode = mode;
                policy.max_stage = cap;
                let live = State::default().observe(&policy, 100, Signal::ProbePath);
                let sim = simulate(
                    &policy,
                    &SimInput {
                        signal: Some(Signal::ProbePath),
                        ..Default::default()
                    },
                );
                assert_eq!(
                    (sim.action, sim.reason, sim.enforced, sim.duration_secs),
                    (
                        live.action,
                        live.reason,
                        live.enforced,
                        live.retry_after_secs
                    )
                );
                assert_eq!(sim.signal_counted, mode != Mode::Off);
            }
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
