//! Session theft detection: the pure rules.
//!
//! Four things are looked for on a cookie session, from the country and coarse
//! network of each request:
//!
//! 1. **Rotated-id reuse.** `/api/session/rotate` retires the old id and leaves
//!    a marker (a hash). Presenting the old id again after a short grace window
//!    (an in-flight request) means two parties hold the cookie: the whole
//!    session family is revoked.
//! 2. **Impossible travel.** A country change sooner than
//!    `security.theft.minTravelSecs` after the previous request.
//! 3. **Concurrent use.** The session is used alternately from two places inside
//!    a few minutes (A, B, A): from two countries by default, optionally from
//!    two networks of one country.
//! 4. **Client mismatch** (see [`crate::session_binding`]).
//!
//! The country-change *alert* is separate and always on by default: any request
//! from a country the session has not been seen in alerts once per new country
//! per session, with an admin-set cooldown between alerts.
//!
//! Country detection comes from an IP geolocation database and can be wrong;
//! VPNs and mobile networks produce false changes. That is why the defaults
//! respond to travel and concurrent use with a re-authentication (a step-up),
//! not a revoke, for webmail, and why users can lower the sensitivity.
use crate::session_binding::Fingerprint;
use serde::{Deserialize, Serialize};

pub(crate) const MAX_RECENT: usize = 6;
/// Alternation window: the other place was used this recently.
const CONCURRENT_WINDOW_SECS: i64 = 300;
const RETURN_WINDOW_SECS: i64 = 600;
const MAX_ALERTED: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Sensitivity {
    Low,
    Normal,
    High,
}
impl Sensitivity {
    pub(crate) fn parse(s: &str) -> Self {
        match s {
            "low" => Self::Low,
            "high" => Self::High,
            _ => Self::Normal,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Concurrent {
    Off,
    Country,
    Network,
}
impl Concurrent {
    pub(crate) fn parse(s: &str) -> Self {
        match s {
            "off" => Self::Off,
            "network" => Self::Network,
            _ => Self::Country,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Rules {
    pub theft: bool,
    pub grace_secs: i64,
    pub min_travel_secs: i64,
    pub concurrent: Concurrent,
    pub retention_days: i64,
    /// Global switch for the country-change alert.
    pub country_alert: bool,
    pub country_cooldown_secs: i64,
}
impl Rules {
    pub(crate) fn from_values(theft: &crate::sec_cfg::Values, alerts: &crate::sec_cfg::Values) -> Self {
        Self {
            theft: theft.flag("security.theft.enabled"),
            grace_secs: theft.num("security.theft.rotationGraceSecs").min(600) as i64,
            min_travel_secs: (theft.num("security.theft.minTravelSecs") as i64).clamp(60, 172_800),
            concurrent: Concurrent::parse(&theft.text("security.theft.concurrentUse")),
            retention_days: (theft.num("security.theft.retentionDays") as i64).clamp(1, 60),
            country_alert: alerts.flag("security.alerts.countryChangeEnabled"),
            country_cooldown_secs: (alerts.num("security.alerts.countryChangeCooldownSecs") as i64)
                .clamp(0, 86_400),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Seen {
    pub net: String,
    pub cc: Option<String>,
    pub ts: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Flag {
    pub reason: String,
    pub ts: i64,
}

/// Everything remembered about one session. Hashes, country codes and times.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct SessionSec {
    pub v: u8,
    /// "webmail" or "admin".
    pub surface: String,
    /// The account id (webmail) or admin id; an internal identifier.
    pub account: String,
    /// Shared by a session and the sessions rotated from it.
    pub family: String,
    pub created: i64,
    pub fp: Fingerprint,
    pub start_country: Option<String>,
    pub last_country: Option<String>,
    pub last_ts: i64,
    #[serde(default)]
    pub recent: Vec<Seen>,
    /// Countries already alerted on (the start country counts as alerted).
    #[serde(default)]
    pub alerted: Vec<String>,
    #[serde(default)]
    pub last_alert: i64,
    /// Last time each finding kind was raised, to debounce.
    #[serde(default)]
    pub noted: Vec<(String, i64)>,
    #[serde(default)]
    pub flagged: Option<Flag>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Finding {
    CountryChange { from: Option<String>, to: String },
    ImpossibleTravel { from: String, to: String, secs: i64 },
    Concurrent { from: String, to: String, cross_country: bool },
}
impl Finding {
}

impl SessionSec {
    pub(crate) fn new(
        surface: &str,
        account: &str,
        family: String,
        fp: Fingerprint,
        country: Option<String>,
        net: &str,
        now: i64,
    ) -> Self {
        Self {
            v: 1,
            surface: surface.into(),
            account: account.into(),
            family,
            created: now,
            fp,
            start_country: country.clone(),
            last_country: country.clone(),
            last_ts: now,
            recent: vec![Seen { net: net.into(), cc: country.clone(), ts: now }],
            alerted: country.into_iter().collect(),
            last_alert: 0,
            noted: Vec::new(),
            flagged: None,
        }
    }

    /// Whether `kind` was raised for this session in the last `secs`; records it if not.
    pub(crate) fn debounce(&mut self, kind: &str, now: i64, secs: i64) -> bool {
        self.noted.retain(|(_, ts)| now - *ts < 86_400);
        if self.noted.iter().any(|(k, ts)| k == kind && now - *ts < secs) {
            return false;
        }
        self.noted.retain(|(k, _)| k != kind);
        self.noted.push((kind.into(), now));
        true
    }

    /// Whether this request could produce a location finding (cheap pre-check, so
    /// user preferences are read only when something happened).
    pub(crate) fn location_event(&self, country: Option<&str>, net: &str) -> bool {
        let moved = country.is_some()
            && (self.last_country.as_deref() != country || self.start_country.as_deref() != country);
        moved || self.recent.first().is_some_and(|s| s.net != net)
    }

    /// Fold one request into the session's location history and return what it
    /// shows. `country_alert` is the effective (admin and user) country-change
    /// setting for this session's account.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn observe(
        &mut self,
        now: i64,
        country: Option<&str>,
        net: &str,
        rules: &Rules,
        sensitivity: Sensitivity,
        country_alert: bool,
    ) -> Vec<Finding> {
        let mut out = Vec::new();
        let min_travel = match sensitivity {
            Sensitivity::Low => rules.min_travel_secs / 2,
            Sensitivity::Normal => rules.min_travel_secs,
            Sensitivity::High => rules.min_travel_secs.saturating_mul(2),
        };
        let concurrent = match sensitivity {
            Sensitivity::Low => Concurrent::Off,
            Sensitivity::High if rules.concurrent != Concurrent::Off => Concurrent::Network,
            _ => rules.concurrent,
        };
        let previous_country = self.last_country.clone();
        let latest = self.recent.first().cloned();
        if let (Some(cc), Some(prev)) = (country, previous_country.as_deref())
            && cc != prev
            && rules.theft
        {
            let gap = now - self.last_ts;
            if gap < min_travel {
                out.push(Finding::ImpossibleTravel { from: prev.into(), to: cc.into(), secs: gap });
            }
        }
        if rules.theft
            && concurrent != Concurrent::Off
            && let Some(latest) = &latest
            && latest.net != net
            && now - latest.ts <= CONCURRENT_WINDOW_SECS
            && let Some(back) = self
                .recent
                .iter()
                .skip(1)
                .find(|s| s.net == net && now - s.ts <= RETURN_WINDOW_SECS)
        {
            let cross = back.cc.is_some() && latest.cc.is_some() && back.cc != latest.cc;
            if cross || concurrent == Concurrent::Network {
                out.push(Finding::Concurrent {
                    from: latest.cc.clone().unwrap_or_default(),
                    to: country.unwrap_or_default().into(),
                    cross_country: cross,
                });
            }
        }
        if let Some(cc) = country
            && rules.country_alert
            && country_alert
            && !self.alerted.iter().any(|a| a == cc)
            && now - self.last_alert >= rules.country_cooldown_secs
        {
            out.push(Finding::CountryChange { from: previous_country, to: cc.into() });
            if self.alerted.len() >= MAX_ALERTED {
                self.alerted.remove(0);
            }
            self.alerted.push(cc.into());
            self.last_alert = now;
        }
        // History: most recent place first, one entry per network.
        self.recent.retain(|s| s.net != net);
        self.recent.insert(0, Seen { net: net.into(), cc: country.map(str::to_owned), ts: now });
        self.recent.truncate(MAX_RECENT);
        if country.is_some() {
            self.last_country = country.map(str::to_owned);
        }
        self.last_ts = now;
        out
    }
}

/// A marker left when a session id is rotated away.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Tombstone {
    pub family: String,
    pub account: String,
    pub surface: String,
    pub rotated_at: i64,
    #[serde(default)]
    pub reported: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Reuse {
    /// Inside the grace window: a request already in flight.
    InFlight,
    /// Already revoked and reported.
    Reported,
    /// A retired id presented after the grace window.
    Theft,
}
pub(crate) fn classify_reuse(t: &Tombstone, now: i64, grace_secs: i64) -> Reuse {
    if now - t.rotated_at <= grace_secs {
        Reuse::InFlight
    } else if t.reported {
        Reuse::Reported
    } else {
        Reuse::Theft
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules() -> Rules {
        Rules {
            theft: true,
            grace_secs: 30,
            min_travel_secs: 3600,
            concurrent: Concurrent::Country,
            retention_days: 15,
            country_alert: true,
            country_cooldown_secs: 300,
        }
    }
    fn session(cc: &str, net: &str) -> SessionSec {
        SessionSec::new("webmail", "acct", "fam".into(), Fingerprint::default(), Some(cc.into()), net, 1000)
    }

    #[test]
    fn same_country_and_network_is_quiet() {
        let mut s = session("PT", "1.1");
        for i in 1..20 {
            assert!(s.observe(1000 + i * 60, Some("PT"), "1.1", &rules(), Sensitivity::Normal, true).is_empty());
        }
    }

    #[test]
    fn country_change_alerts_once_per_new_country_with_cooldown() {
        let mut s = session("PT", "1.1");
        let mut r = rules();
        r.theft = false; // isolate the alert from the travel rule
        let f = s.observe(8200, Some("ES"), "2.2", &r, Sensitivity::Normal, true);
        assert_eq!(f, vec![Finding::CountryChange { from: Some("PT".into()), to: "ES".into() }]);
        // Same country again: nothing.
        assert!(s.observe(8250, Some("ES"), "2.2", &r, Sensitivity::Normal, true).is_empty());
        // A third country inside the cooldown is held back, then alerts once it passes.
        assert!(s.observe(8300, Some("FR"), "3.3", &r, Sensitivity::Normal, true).is_empty());
        let later = s.observe(8600, Some("FR"), "3.3", &r, Sensitivity::Normal, true);
        assert_eq!(later.len(), 1);
        assert!(s.observe(8700, Some("FR"), "3.3", &r, Sensitivity::Normal, true).is_empty());
        // Back in the start country (already known): no alert.
        assert!(s.observe(20_000, Some("PT"), "1.1", &r, Sensitivity::Normal, true).is_empty());
    }

    #[test]
    fn global_off_and_user_mute_silence_the_country_alert_only() {
        let mut r = rules();
        let mut s = session("PT", "1.1");
        assert!(!s.observe(1000 + 7200, Some("ES"), "2.2", &r, Sensitivity::Normal, false).iter().any(|f| matches!(f, Finding::CountryChange { .. })));
        r.country_alert = false;
        let mut s = session("PT", "1.1");
        let f = s.observe(1000 + 10, Some("ES"), "2.2", &r, Sensitivity::Normal, true);
        assert_eq!(f.len(), 1, "travel is separate from the alert switch: {f:?}");
        assert!(matches!(f[0], Finding::ImpossibleTravel { .. }));
    }

    #[test]
    fn impossible_travel_respects_the_interval_and_sensitivity() {
        let r = rules();
        let mut s = session("PT", "1.1");
        let f = s.observe(1000 + 600, Some("JP"), "9.9", &r, Sensitivity::Normal, true);
        assert!(f.iter().any(|f| matches!(f, Finding::ImpossibleTravel { secs: 600, .. })));
        // 45 minutes: inside the hour at normal, outside at low (30 min), inside at high.
        for (sens, expect) in [(Sensitivity::Normal, true), (Sensitivity::Low, false), (Sensitivity::High, true)] {
            let mut s = session("PT", "1.1");
            let f = s.observe(1000 + 2700, Some("JP"), "9.9", &r, sens, true);
            assert_eq!(f.iter().any(|f| matches!(f, Finding::ImpossibleTravel { .. })), expect, "{sens:?}");
        }
        // Two hours later a country change is a plain alert, not travel.
        let mut s = session("PT", "1.1");
        let f = s.observe(1000 + 7200, Some("JP"), "9.9", &r, Sensitivity::Normal, true);
        assert!(!f.iter().any(|f| matches!(f, Finding::ImpossibleTravel { .. })));
    }

    #[test]
    fn alternating_countries_is_concurrent_use() {
        let r = rules();
        let mut s = session("PT", "1.1");
        let t = 1000 + 100_000;
        // PT was in use a minute before the RU request.
        s.last_ts = t - 100;
        s.recent[0].ts = t - 100;
        let _ = s.observe(t, Some("RU"), "7.7", &r, Sensitivity::Normal, false);
        let f = s.observe(t + 60, Some("PT"), "1.1", &r, Sensitivity::Normal, false);
        assert!(f.iter().any(|f| matches!(f, Finding::Concurrent { cross_country: true, .. })), "{f:?}");
        // Off: nothing.
        let mut off = rules();
        off.concurrent = Concurrent::Off;
        let mut s = session("PT", "1.1");
        let _ = s.observe(t, Some("RU"), "7.7", &off, Sensitivity::Normal, false);
        assert!(!s.observe(t + 60, Some("PT"), "1.1", &off, Sensitivity::Normal, false).iter().any(|f| matches!(f, Finding::Concurrent { .. })));
    }

    #[test]
    fn same_country_network_changes_need_the_network_mode_or_high_sensitivity() {
        let r = rules();
        let run = |r: &Rules, sens| {
            let mut s = session("PT", "1.1");
            let _ = s.observe(1100, Some("PT"), "2.2", r, sens, false);
            s.observe(1160, Some("PT"), "1.1", r, sens, false)
        };
        assert!(run(&r, Sensitivity::Normal).is_empty(), "wifi to cellular and back is normal");
        let mut net = rules();
        net.concurrent = Concurrent::Network;
        assert!(!run(&net, Sensitivity::Normal).is_empty());
        assert!(!run(&r, Sensitivity::High).is_empty());
        assert!(run(&net, Sensitivity::Low).is_empty());
    }

    #[test]
    fn unknown_country_never_alerts_or_moves_the_country() {
        let mut s = session("PT", "1.1");
        assert!(s.observe(1010, None, "5.5", &rules(), Sensitivity::Normal, true).is_empty());
        assert_eq!(s.last_country.as_deref(), Some("PT"));
    }

    #[test]
    fn rotation_reuse_has_a_grace_window() {
        let mut t = Tombstone { family: "f".into(), account: "a".into(), surface: "webmail".into(), rotated_at: 1000, reported: false };
        assert_eq!(classify_reuse(&t, 1020, 30), Reuse::InFlight);
        assert_eq!(classify_reuse(&t, 1030, 30), Reuse::InFlight);
        assert_eq!(classify_reuse(&t, 1031, 30), Reuse::Theft);
        assert_eq!(classify_reuse(&t, 1031, 0), Reuse::Theft);
        t.reported = true;
        assert_eq!(classify_reuse(&t, 5000, 30), Reuse::Reported);
    }

    #[test]
    fn debounce_limits_a_kind_per_session() {
        let mut s = session("PT", "1.1");
        assert!(s.debounce("impossibleTravel", 1000, 600));
        assert!(!s.debounce("impossibleTravel", 1500, 600));
        assert!(s.debounce("concurrentUse", 1500, 600));
        assert!(s.debounce("impossibleTravel", 1601, 600));
    }

    #[test]
    fn state_is_small_and_has_no_raw_identifiers() {
        let s = session("PT", "1.1");
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.len() < 1024, "{json}");
        assert!(!json.contains("password") && !json.contains("cookie"));
    }
}
