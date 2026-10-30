//! Bounded header-claim classification, never authenticated crawler identity.
use mw_store::request_protection_model::{Mode, Policy};
use regex::{RegexSet, RegexSetBuilder};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const CATEGORIES: &[&str] = &[
    "missing",
    "uncommon",
    "automation",
    "search",
    "ai-training",
    "ai-search",
    "ai-user",
];
pub(super) const MAX_UA: usize = 4096;

pub(crate) fn valid_patterns(value: &Value) -> bool {
    let Some(items) = value.as_array() else {
        return false;
    };
    if items.len() > 32 {
        return false;
    }
    let Some(patterns) = items.iter().map(Value::as_str).collect::<Option<Vec<_>>>() else {
        return false;
    };
    patterns.iter().all(|s| !s.is_empty() && s.len() <= 256) && compile(&patterns).is_ok()
}
fn compile(patterns: &[&str]) -> Result<RegexSet, regex::Error> {
    RegexSetBuilder::new(patterns)
        .case_insensitive(true)
        .size_limit(256 * 1024)
        .dfa_size_limit(256 * 1024)
        .build()
}

#[derive(Clone, Debug)]
pub(super) struct Rules {
    pub mode: Mode,
    categories: BTreeSet<String>,
    weak_requests: u32,
    crawler_requests: u32,
    window_secs: u32,
    max_stage: u8,
    pub anonymous_only: bool,
    adaptive: bool,
    patterns: RegexSet,
    exceptions: RegexSet,
}
impl Rules {
    pub fn parse(values: &BTreeMap<String, Value>) -> Result<Self, ()> {
        let get = |key: &str| {
            values
                .get(&format!("{}userAgent{key}", super::config::PREFIX))
                .ok_or(())
        };
        let number = |key: &str| {
            get(key)?
                .as_u64()
                .and_then(|n| u32::try_from(n).ok())
                .ok_or(())
        };
        let strings = |key: &str| -> Result<Vec<&str>, ()> {
            get(key)?
                .as_array()
                .ok_or(())?
                .iter()
                .map(|s| s.as_str().ok_or(()))
                .collect()
        };
        let categories = strings("Categories")?;
        if categories.iter().any(|value| !CATEGORIES.contains(value)) {
            return Err(());
        }
        if !(1..=1_000_000).contains(&number("WeakRequests")?)
            || !(1..=1_000_000).contains(&number("CrawlerRequests")?)
            || !(1..=3600).contains(&number("WindowSecs")?)
            || !valid_patterns(get("Patterns")?)
            || !valid_patterns(get("Exceptions")?)
        {
            return Err(());
        }
        Ok(Self {
            mode: match get("Mode")?.as_str() {
                Some("off") => Mode::Off,
                Some("observe") => Mode::Observe,
                Some("enforce") => Mode::Enforce,
                _ => return Err(()),
            },
            categories: categories.into_iter().map(str::to_owned).collect(),
            weak_requests: number("WeakRequests")?,
            crawler_requests: number("CrawlerRequests")?,
            window_secs: number("WindowSecs")?,
            max_stage: match get("MaxAction")?.as_str() {
                Some("throttle") => 1,
                Some("block") => 2,
                _ => return Err(()),
            },
            anonymous_only: match get("Scope")?.as_str() {
                Some("anonymous") => true,
                Some("all") => false,
                _ => return Err(()),
            },
            adaptive: get("Adaptive")?.as_bool().ok_or(())?,
            patterns: compile(&strings("Patterns")?).map_err(|_| ())?,
            exceptions: compile(&strings("Exceptions")?).map_err(|_| ())?,
        })
    }
    pub fn classify(&self, header: Option<&str>) -> Option<&'static str> {
        if self.mode == Mode::Off {
            return None;
        }
        // Unbounded or non-text input is weak/uncommon, never an immediate block.
        let ua = header.unwrap_or("");
        if ua.len() > MAX_UA {
            return self.categories.contains("uncommon").then_some("uncommon");
        }
        if self.exceptions.is_match(ua) {
            return None;
        }
        if self.patterns.is_match(ua) {
            return Some("custom");
        }
        let classification = classify(ua);
        classification.filter(|category| self.categories.contains(*category))
    }
    pub fn policy(&self, base: &Policy, category: &str) -> Option<Policy> {
        if self.mode == Mode::Off || (category != "custom" && !self.categories.contains(category)) {
            return None;
        }
        let mut policy = base.clone();
        policy.rate_mode = self.mode;
        policy.rate_limit =
            if matches!(category, "search" | "ai-training" | "ai-search" | "ai-user") {
                self.crawler_requests
            } else {
                self.weak_requests
            };
        policy.rate_window_secs = self.window_secs;
        policy.adaptive_mode = if self.adaptive { self.mode } else { Mode::Off };
        policy.max_stage = self.max_stage;
        Some(policy)
    }
    pub fn summary(&self) -> Value {
        json!({"mode":match self.mode{Mode::Off=>"off",Mode::Observe=>"observe",Mode::Enforce=>"enforce"},"categories":self.categories,"weakRequests":self.weak_requests,"crawlerRequests":self.crawler_requests,"windowSecs":self.window_secs,"maxAction":if self.max_stage==1{"throttle"}else{"block"},"scope":if self.anonymous_only{"anonymous"}else{"all"},"adaptive":self.adaptive,"claimedOnly":true})
    }
}

fn token(ua: &str, needle: &str) -> bool {
    ua.match_indices(needle).any(|(at, _)| {
        let boundary = |byte: u8| !byte.is_ascii_alphanumeric() && byte != b'-' && byte != b'_';
        (at == 0 || boundary(ua.as_bytes()[at - 1]))
            && (at + needle.len() == ua.len() || boundary(ua.as_bytes()[at + needle.len()]))
    })
}
fn classify(raw: &str) -> Option<&'static str> {
    let ua = raw.trim().to_ascii_lowercase();
    if ua.is_empty() {
        return Some("missing");
    }
    // Official references and the limits of these claims: docs/admin/crawler-identification.md.
    for (category, names) in [
        (
            "ai-user",
            &[
                "chatgpt-user",
                "claude-user",
                "perplexity-user",
                "amzn-user",
                "mistralai-user",
                "meta-externalfetcher",
            ][..],
        ),
        (
            "ai-search",
            &[
                "oai-searchbot",
                "claude-searchbot",
                "perplexitybot",
                "duckassistbot",
                "amzn-searchbot",
            ][..],
        ),
        (
            "ai-training",
            &[
                "gptbot",
                "claudebot",
                "ccbot",
                "amazonbot",
                "google-cloudvertexbot",
                "bytespider",
                "meta-externalagent",
                "facebookbot",
            ][..],
        ),
        (
            "search",
            &[
                "googlebot",
                "googlebot-image",
                "googlebot-news",
                "googlebot-video",
                "googlebot-storebot",
                "google-inspectiontool",
                "googleother",
                "googleother-image",
                "googleother-video",
                "bingbot",
                "adidxbot",
                "applebot",
                "duckduckbot",
                "yandexbot",
                "yandexmobilebot",
                "yandexcombot",
            ][..],
        ),
    ] {
        if names.iter().any(|name| token(&ua, name)) {
            return Some(category);
        }
    }
    if [
        "curl",
        "wget",
        "python-requests",
        "python-urllib",
        "httpx",
        "go-http-client",
        "headlesschrome",
        "phantomjs",
        "selenium",
        "scrapy",
        "aiohttp",
        "libwww-perl",
        "java",
    ]
    .iter()
    .any(|name| token(&ua, name))
    {
        return Some("automation");
    }
    if ua.starts_with("mozilla/5.0")
        && [
            "chrome/",
            "chromium/",
            "firefox/",
            "safari/",
            "edg/",
            "opr/",
            "thunderbird/",
        ]
        .iter()
        .any(|family| ua.contains(family))
        || [
            "mailwoman/",
            "thunderbird/",
            "outlook/",
            "microsoft office/",
            "applemail/",
            "davx5/",
        ]
        .iter()
        .any(|family| ua.starts_with(family))
    {
        return None;
    }
    Some("uncommon")
}

pub(super) fn kind(category: &str) -> Option<&'static str> {
    match category {
        "missing" => Some("ua-missing"),
        "uncommon" => Some("ua-uncommon"),
        "automation" => Some("ua-automation"),
        "search" => Some("ua-search"),
        "ai-training" => Some("ua-ai-training"),
        "ai-search" => Some("ua-ai-search"),
        "ai-user" => Some("ua-ai-user"),
        "custom" => Some("ua-custom"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn claims_are_categories_not_verified_identity() {
        for (ua, class) in [
            ("", "missing"),
            ("a new tool/1", "uncommon"),
            ("curl/8", "automation"),
            ("Mozilla/5.0 (compatible; Googlebot/2.1)", "search"),
            ("GPTBot/1.2", "ai-training"),
            ("OAI-SearchBot/1.0", "ai-search"),
            ("Claude-User/1.0", "ai-user"),
        ] {
            assert_eq!(classify(ua), Some(class));
        }
        assert_eq!(classify("Mozilla/5.0 Chrome/140.0 Safari/537.36"), None);
        assert_eq!(classify("FakeGooglebot/1"), Some("uncommon"));
        assert_eq!(classify("Google-Extended"), Some("uncommon"));
        assert_eq!(classify("Applebot-Extended"), Some("uncommon"));
    }
    #[test]
    fn regex_rules_are_bounded_and_do_not_support_backtracking_extensions() {
        assert!(valid_patterns(&json!([
            "^trusted-client/[0-9]+$",
            "custom-bot"
        ])));
        assert!(!valid_patterns(&json!(["(?=unsafe)"])));
        assert!(!valid_patterns(&json!(["x".repeat(257)])));
        assert!(!valid_patterns(&json!(vec!["x"; 33])));
    }
}
