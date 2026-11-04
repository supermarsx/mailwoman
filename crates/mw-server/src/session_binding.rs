//! Session binding: tie a browser session to the client that opened it.
//!
//! A stolen session cookie is replayed from another machine. The server cannot
//! see a hardware key, but it can see stable request attributes, so a cookie
//! used from a different browser or operating system, or from a very different
//! place, is noticed.
//!
//! # What is bound
//! Keyed hashes (HMAC-SHA256 under a server-derived key, 64 bits kept) of:
//! * `browser` - the browser family from the User-Agent (chrome, edge, firefox,
//!   safari, opera, samsung, other), and its **major version** as a number;
//! * `os` - the operating system family (windows, macos, ios, android, linux,
//!   chromeos);
//! * `hints` - `Sec-CH-UA-Platform`, when the browser sends it;
//! * `language` - the primary tag of the first `Accept-Language` entry (off by
//!   default: people change it);
//! * `country` - the country of the source address (on by default);
//! * `network` - the IPv4 /16 or IPv6 /48 (off by default: mobile networks move).
//!
//! No raw header, address or User-Agent is stored. The one non-hashed value is
//! the browser's major version number, kept so an auto-update can be tolerated.
//! An ASN is not bound: the app has no ASN database, only a country one.
//!
//! # Matching
//! Only attributes present on both sides are compared; a missing header never
//! counts against the session. *Hard* attributes are the browser family, the OS
//! family and the version: a different family or OS is a mismatch, a version
//! that went back by more than two majors is one too. *Soft* attributes are the
//! hints platform, language, country and network. `tolerant` (default) accepts
//! any browser update and mismatches only when two soft attributes differ;
//! `strict` mismatches on any soft difference and on a version more than three
//! majors ahead. A request with no User-Agent at all is [`Verdict::Missing`],
//! which the caller treats as `detect`, never as a reason to lock out.
//!
//! # Exempt
//! Native shells and protocol clients that authenticate with a bearer token or
//! an API key (IMAP/SMTP/DAV app passwords, scoped keys, MCP) have no browser
//! fingerprint and are not bound; only the cookie-authenticated web sessions
//! (webmail and admin console) are.
use crate::sec_cfg;
use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

pub(crate) const PREFIX: &str = "security.sessionBinding.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Action {
    Off,
    Detect,
    StepUp,
    Revoke,
}
impl Action {
    fn parse(s: &str, fallback: Action) -> Action {
        match s {
            "off" => Action::Off,
            "detect" => Action::Detect,
            "step-up" => Action::StepUp,
            "revoke" => Action::Revoke,
            _ => fallback,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Attrs {
    pub browser: bool,
    pub os: bool,
    pub hints: bool,
    pub language: bool,
    pub country: bool,
    pub network: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct Policy {
    pub enabled: bool,
    pub attrs: Attrs,
    pub strict: bool,
    pub webmail: Action,
    pub admin: Action,
}
impl Default for Policy {
    fn default() -> Self {
        Self::from_values(&sec_cfg::Values::default())
    }
}
impl Policy {
    pub(crate) fn from_values(v: &sec_cfg::Values) -> Self {
        let list = v.list("security.sessionBinding.attributes");
        let has = |n: &str| list.iter().any(|a| a == n);
        Self {
            enabled: v.flag("security.sessionBinding.enabled"),
            attrs: Attrs {
                browser: has("browser"),
                os: has("os"),
                hints: has("hints"),
                language: has("language"),
                country: has("country"),
                network: has("network"),
            },
            strict: v.text("security.sessionBinding.strictness") == "strict",
            webmail: Action::parse(
                &v.text("security.sessionBinding.webmailAction"),
                Action::StepUp,
            ),
            admin: Action::parse(&v.text("security.sessionBinding.adminAction"), Action::Revoke),
        }
    }
    pub(crate) fn action(&self, admin: bool) -> Action {
        if !self.enabled {
            Action::Off
        } else if admin {
            self.admin
        } else {
            self.webmail
        }
    }
}

/// What the request says about its client, before any hashing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Client {
    pub family: Option<&'static str>,
    pub os: Option<&'static str>,
    pub major: Option<u32>,
    pub hint_platform: Option<String>,
    pub language: Option<String>,
}
impl Client {
    /// No browser identity at all: blank or absent User-Agent.
    pub(crate) fn blank(&self) -> bool {
        self.family.is_none() && self.os.is_none()
    }
}

fn token_version(ua: &str, token: &str) -> Option<u32> {
    let rest = &ua[ua.find(token)? + token.len()..];
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

pub(crate) fn parse_user_agent(ua: &str) -> (Option<&'static str>, Option<&'static str>, Option<u32>) {
    let ua = ua.trim();
    if ua.is_empty() {
        return (None, None, None);
    }
    let family_version = [
        ("Edg/", "edge"),
        ("EdgA/", "edge"),
        ("EdgiOS/", "edge"),
        ("Edge/", "edge"),
        ("OPR/", "opera"),
        ("OPiOS/", "opera"),
        ("SamsungBrowser/", "samsung"),
        ("Firefox/", "firefox"),
        ("FxiOS/", "firefox"),
        ("CriOS/", "chrome"),
        ("Chrome/", "chrome"),
    ]
    .iter()
    .find_map(|(tok, fam)| token_version(ua, tok).map(|v| (*fam, Some(v))))
    .or_else(|| {
        (ua.contains("Safari/") && ua.contains("Version/"))
            .then(|| ("safari", token_version(ua, "Version/")))
    })
    .unwrap_or(("other", None));
    let os = if ua.contains("Android") {
        Some("android")
    } else if ua.contains("iPhone") || ua.contains("iPad") || ua.contains("iPod") {
        Some("ios")
    } else if ua.contains("CrOS") {
        Some("chromeos")
    } else if ua.contains("Windows") {
        Some("windows")
    } else if ua.contains("Mac OS X") || ua.contains("Macintosh") {
        Some("macos")
    } else if ua.contains("Linux") || ua.contains("X11") {
        Some("linux")
    } else {
        None
    };
    (Some(family_version.0), os, family_version.1)
}

pub(crate) fn parse_client(headers: &HeaderMap) -> Client {
    let get = |name: &str| headers.get(name).and_then(|v| v.to_str().ok()).unwrap_or("");
    let (family, os, major) = parse_user_agent(get("user-agent"));
    let hint = get("sec-ch-ua-platform").trim().trim_matches('"').to_ascii_lowercase();
    let language = get("accept-language")
        .split([',', ';'])
        .next()
        .map(|t| t.trim().split('-').next().unwrap_or("").to_ascii_lowercase())
        .filter(|t| (2..=3).contains(&t.len()) && t.chars().all(|c| c.is_ascii_alphabetic()));
    Client {
        family,
        os,
        major,
        hint_platform: (!hint.is_empty() && hint.len() <= 24).then_some(hint),
        language,
    }
}

/// The stored form: keyed hashes only (the version is a bare number).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Fingerprint {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub b: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ph: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lg: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cc: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub net: Option<String>,
}

/// IPv4 /16 or IPv6 /48 of an address, as text, for the `network` attribute.
pub(crate) fn coarse_network(ip: IpAddr) -> String {
    match ip.to_canonical() {
        IpAddr::V4(v) => {
            let o = v.octets();
            format!("{}.{}", o[0], o[1])
        }
        IpAddr::V6(v) => {
            let s = v.segments();
            format!("{:x}:{:x}:{:x}", s[0], s[1], s[2])
        }
    }
}

pub(crate) fn derive(
    secret: &str,
    client: &Client,
    country: Option<&str>,
    ip: Option<IpAddr>,
    attrs: Attrs,
) -> Fingerprint {
    let h = |domain: &str, value: &str| {
        sec_cfg::hex(&sec_cfg::keyed_hash(secret, &format!("fp-{domain}"), value)[..8])
    };
    Fingerprint {
        b: attrs.browser.then_some(client.family).flatten().map(|f| h("browser", f)),
        v: attrs.browser.then_some(client.major).flatten(),
        os: attrs.os.then_some(client.os).flatten().map(|o| h("os", o)),
        ph: attrs
            .hints
            .then_some(client.hint_platform.as_deref())
            .flatten()
            .map(|p| h("hint", p)),
        lg: attrs
            .language
            .then_some(client.language.as_deref())
            .flatten()
            .map(|l| h("lang", l)),
        cc: attrs.country.then_some(country).flatten().map(|c| h("country", c)),
        net: attrs
            .network
            .then_some(ip)
            .flatten()
            .map(|ip| h("net", &coarse_network(ip))),
    }
}

/// After a match: follow a browser update (a higher major version) and learn
/// attributes the first request did not carry (for example a client hint).
pub(crate) fn merge_on_match(stored: &mut Fingerprint, current: &Fingerprint) {
    if stored.b == current.b
        && let (Some(old), Some(new)) = (stored.v, current.v)
        && new > old
    {
        stored.v = Some(new);
    }
    for (old, new) in [
        (&mut stored.b, &current.b),
        (&mut stored.os, &current.os),
        (&mut stored.ph, &current.ph),
        (&mut stored.lg, &current.lg),
    ] {
        if old.is_none() {
            old.clone_from(new);
        }
    }
    if stored.v.is_none() && stored.b == current.b {
        stored.v = current.v;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    Match,
    /// The attributes that differ.
    Mismatch(Vec<&'static str>),
    /// The request has no browser identity; never a reason to act beyond detect.
    Missing,
}

pub(crate) fn compare(stored: &Fingerprint, current: &Fingerprint, client: &Client, strict: bool) -> Verdict {
    if client.blank() {
        return Verdict::Missing;
    }
    let mut hard = Vec::new();
    let mut soft = Vec::new();
    let differs = |a: &Option<String>, b: &Option<String>| matches!((a, b), (Some(a), Some(b)) if a != b);
    if differs(&stored.b, &current.b) {
        hard.push("browser");
    }
    if differs(&stored.os, &current.os) {
        hard.push("os");
    }
    if let (Some(old), Some(new)) = (stored.v, current.v)
        && stored.b == current.b
    {
        let back = old.saturating_sub(new);
        let ahead = new.saturating_sub(old);
        if back > 2 || (strict && ahead > 3) {
            hard.push("version");
        }
    }
    for (name, a, b) in [
        ("hints", &stored.ph, &current.ph),
        ("language", &stored.lg, &current.lg),
        ("country", &stored.cc, &current.cc),
        ("network", &stored.net, &current.net),
    ] {
        if differs(a, b) {
            soft.push(name);
        }
    }
    let soft_limit = if strict { 1 } else { 2 };
    let mut out = hard;
    if soft.len() >= soft_limit {
        out.extend(soft);
    }
    if out.is_empty() {
        Verdict::Match
    } else {
        Verdict::Mismatch(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHROME_WIN_125: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36";
    const CHROME_WIN_126: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";
    const EDGE_WIN: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36 Edg/125.0.2535.51";
    const FIREFOX_LINUX: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:126.0) Gecko/20100101 Firefox/126.0";
    const SAFARI_MAC: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4 Safari/605.1.15";
    const SAFARI_IPHONE: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_4 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4 Mobile/15E148 Safari/604.1";
    const CHROME_ANDROID: &str = "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Mobile Safari/537.36";

    fn headers(ua: &str, extra: &[(&str, &str)]) -> HeaderMap {
        let mut m = HeaderMap::new();
        if !ua.is_empty() {
            m.insert("user-agent", ua.parse().unwrap());
        }
        for (k, v) in extra {
            m.insert(
                axum::http::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                v.parse().unwrap(),
            );
        }
        m
    }
    fn all() -> Attrs {
        Attrs { browser: true, os: true, hints: true, language: true, country: true, network: true }
    }
    fn fp(ua: &str, extra: &[(&str, &str)], country: &str, ip: &str, attrs: Attrs) -> (Fingerprint, Client) {
        let c = parse_client(&headers(ua, extra));
        (derive("k", &c, Some(country), ip.parse().ok(), attrs), c)
    }

    #[test]
    fn user_agents_parse_to_family_os_and_major() {
        assert_eq!(parse_user_agent(CHROME_WIN_125), (Some("chrome"), Some("windows"), Some(125)));
        assert_eq!(parse_user_agent(EDGE_WIN), (Some("edge"), Some("windows"), Some(125)));
        assert_eq!(parse_user_agent(FIREFOX_LINUX), (Some("firefox"), Some("linux"), Some(126)));
        assert_eq!(parse_user_agent(SAFARI_MAC), (Some("safari"), Some("macos"), Some(17)));
        assert_eq!(parse_user_agent(SAFARI_IPHONE), (Some("safari"), Some("ios"), Some(17)));
        assert_eq!(parse_user_agent(CHROME_ANDROID), (Some("chrome"), Some("android"), Some(125)));
        assert_eq!(parse_user_agent(""), (None, None, None));
        assert_eq!(parse_user_agent("curl/8.5.0").0, Some("other"));
    }

    #[test]
    fn a_browser_auto_update_matches_in_both_modes() {
        let (a, _) = fp(CHROME_WIN_125, &[], "PT", "203.0.113.5", all());
        let (b, cb) = fp(CHROME_WIN_126, &[], "PT", "203.0.113.5", all());
        assert_eq!(compare(&a, &b, &cb, false), Verdict::Match);
        assert_eq!(compare(&a, &b, &cb, true), Verdict::Match);
    }

    #[test]
    fn a_different_browser_family_or_os_is_rejected() {
        let (a, _) = fp(CHROME_WIN_125, &[], "PT", "203.0.113.5", all());
        let (edge, ce) = fp(EDGE_WIN, &[], "PT", "203.0.113.5", all());
        assert_eq!(compare(&a, &edge, &ce, false), Verdict::Mismatch(vec!["browser"]));
        let (ff, cf) = fp(FIREFOX_LINUX, &[], "PT", "203.0.113.5", all());
        match compare(&a, &ff, &cf, false) {
            Verdict::Mismatch(w) => assert!(w.contains(&"browser") && w.contains(&"os")),
            other => panic!("{other:?}"),
        }
        let (android, ca) = fp(CHROME_ANDROID, &[], "PT", "203.0.113.5", all());
        assert!(matches!(compare(&a, &android, &ca, false), Verdict::Mismatch(w) if w == ["os"]));
    }

    #[test]
    fn a_downgrade_is_a_mismatch_but_an_upgrade_is_not() {
        let old = CHROME_WIN_125.replace("125.0", "110.0");
        let (a, _) = fp(CHROME_WIN_125, &[], "PT", "203.0.113.5", all());
        let (b, cb) = fp(&old, &[], "PT", "203.0.113.5", all());
        assert_eq!(compare(&a, &b, &cb, false), Verdict::Mismatch(vec!["version"]));
        assert_eq!(compare(&b, &a, &parse_client(&headers(CHROME_WIN_125, &[])), false), Verdict::Match);
        // Strict also bounds how far ahead.
        assert_eq!(
            compare(&b, &a, &parse_client(&headers(CHROME_WIN_125, &[])), true),
            Verdict::Mismatch(vec!["version"])
        );
    }

    #[test]
    fn soft_attributes_need_two_changes_when_tolerant_and_one_when_strict() {
        let attrs = all();
        let (a, _) = fp(CHROME_WIN_125, &[("accept-language", "pt-PT,pt;q=0.9")], "PT", "203.0.113.5", attrs);
        // Country only (a trip or a VPN): tolerant allows, strict does not.
        let (b, cb) = fp(CHROME_WIN_125, &[("accept-language", "pt-PT")], "ES", "203.0.113.5", attrs);
        assert_eq!(compare(&a, &b, &cb, false), Verdict::Match);
        assert_eq!(compare(&a, &b, &cb, true), Verdict::Mismatch(vec!["country"]));
        // Country and network together: mismatch even when tolerant.
        let (c, cc) = fp(CHROME_WIN_125, &[("accept-language", "pt-PT")], "ES", "198.51.100.5", attrs);
        assert_eq!(compare(&a, &c, &cc, false), Verdict::Mismatch(vec!["country", "network"]));
    }

    #[test]
    fn missing_headers_never_count_against_the_session() {
        let (a, _) = fp(CHROME_WIN_125, &[("sec-ch-ua-platform", "\"Windows\"")], "PT", "203.0.113.5", all());
        assert!(a.ph.is_some());
        // The same browser without the hint header.
        let (b, cb) = fp(CHROME_WIN_125, &[], "PT", "203.0.113.5", all());
        assert!(b.ph.is_none());
        assert_eq!(compare(&a, &b, &cb, true), Verdict::Match);
        // No User-Agent at all is Missing, not a mismatch.
        let (n, cn) = fp("", &[], "PT", "203.0.113.5", all());
        assert_eq!(compare(&a, &n, &cn, true), Verdict::Missing);
    }

    #[test]
    fn only_bound_attributes_are_stored_and_nothing_is_raw() {
        let attrs = Attrs { browser: true, os: true, ..Default::default() };
        let (a, _) = fp(CHROME_WIN_125, &[("accept-language", "pt-PT")], "PT", "203.0.113.5", attrs);
        assert!(a.lg.is_none() && a.cc.is_none() && a.net.is_none() && a.ph.is_none());
        let json = serde_json::to_string(&a).unwrap();
        for raw in ["chrome", "windows", "Chrome", "203.0.113", "PT"] {
            assert!(!json.contains(raw), "{raw} leaked into {json}");
        }
        // A different key gives different hashes.
        let c = parse_client(&headers(CHROME_WIN_125, &[]));
        assert_ne!(
            derive("k1", &c, None, None, attrs).b,
            derive("k2", &c, None, None, attrs).b
        );
    }

    #[test]
    fn a_match_follows_updates_and_learns_late_attributes() {
        let (mut a, _) = fp(CHROME_WIN_125, &[], "PT", "203.0.113.5", all());
        let (b, _) = fp(CHROME_WIN_126, &[("sec-ch-ua-platform", "\"Windows\"")], "PT", "203.0.113.5", all());
        merge_on_match(&mut a, &b);
        assert_eq!(a.v, Some(126));
        assert!(a.ph.is_some(), "a hint seen later is learned");
        // It never overwrites a different value.
        let (ff, _) = fp(FIREFOX_LINUX, &[], "PT", "203.0.113.5", all());
        let before = a.clone();
        merge_on_match(&mut a, &ff);
        assert_eq!(a.b, before.b);
        assert_eq!(a.v, before.v);
    }

    #[test]
    fn per_surface_actions_and_defaults() {
        let p = Policy::default();
        assert!(p.enabled && !p.strict);
        assert_eq!(p.action(false), Action::StepUp);
        assert_eq!(p.action(true), Action::Revoke);
        assert!(p.attrs.browser && p.attrs.os && p.attrs.hints && p.attrs.country);
        assert!(!p.attrs.language && !p.attrs.network);
        let off = Policy { enabled: false, ..p };
        assert_eq!(off.action(true), Action::Off);
    }

    #[test]
    fn coarse_networks() {
        assert_eq!(coarse_network("203.0.113.5".parse().unwrap()), "203.0");
        assert_eq!(coarse_network("::ffff:203.0.113.5".parse().unwrap()), "203.0");
        assert_eq!(coarse_network("2001:db8:1:2::9".parse().unwrap()), "2001:db8:1");
    }
}
