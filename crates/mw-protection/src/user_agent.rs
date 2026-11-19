//! Bounded header-claim classification, never authenticated crawler identity.
use mw_store::request_protection_model::{Mode, Policy};
use regex::{RegexSet, RegexSetBuilder};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub const CATEGORIES: &[&str] = &[
    "missing",
    "uncommon",
    "automation",
    "search",
    "ai-training",
    "ai-search",
    "ai-user",
];
pub const MAX_UA: usize = 4096;

pub fn valid_patterns(value: &Value) -> bool {
    checked_patterns(value).is_ok()
}
fn checked_patterns(value: &Value) -> Result<RegexSet, ()> {
    let items = value.as_array().ok_or(())?;
    if items.len() > 32 {
        return Err(());
    }
    let patterns = items
        .iter()
        .map(Value::as_str)
        .collect::<Option<Vec<_>>>()
        .ok_or(())?;
    if patterns
        .iter()
        .any(|pattern| pattern.is_empty() || pattern.len() > 256)
    {
        return Err(());
    }
    compile(&patterns).map_err(|_| ())
}

fn compile(patterns: &[&str]) -> Result<RegexSet, regex::Error> {
    RegexSetBuilder::new(patterns)
        .case_insensitive(true)
        .size_limit(256 * 1024)
        .dfa_size_limit(256 * 1024)
        .build()
}

#[derive(Clone, Debug)]
pub struct Rules {
    pub mode: Mode,
    categories: BTreeSet<String>,
    weak_requests: u32,
    crawler_requests: u32,
    window_secs: u32,
    max_stage: u8,
    pub anonymous_only: bool,
    adaptive: bool,
    block_bots: bool,
    patterns: RegexSet,
    exceptions: RegexSet,
}
impl Rules {
    pub fn parse(values: &BTreeMap<String, Value>) -> Result<Self, ()> {
        let get = |key: &str| {
            values
                .get(&format!("{}userAgent{key}", crate::SETTINGS_PREFIX))
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
            block_bots: get("BlockBots")?.as_bool().ok_or(())?,
            patterns: checked_patterns(get("Patterns")?)?,
            exceptions: checked_patterns(get("Exceptions")?)?,
        })
    }
    pub fn classify(&self, header: Option<&str>) -> Option<&'static str> {
        if self.mode == Mode::Off {
            return None;
        }
        self.monitored_class(self.claimed_class(header))
    }
    /// Apply this budget's monitoring choices to an already classified claim.
    /// HTTP adapters can reuse the same claim for per-class policy and budgets.
    pub fn monitored_class(&self, claimed: Option<&'static str>) -> Option<&'static str> {
        if self.mode == Mode::Off {
            return None;
        }
        claimed.filter(|category| *category == "custom" || self.categories.contains(*category))
    }
    /// The claimed class of a header, independent of the budget mode and the
    /// monitored-category list (used for the per-class allow/limit/block action).
    /// Operator exceptions stay allowed; custom patterns are the `custom` class.
    pub fn claimed_class(&self, header: Option<&str>) -> Option<&'static str> {
        let ua = header.unwrap_or("");
        if ua.len() > MAX_UA {
            return Some("uncommon");
        }
        if self.exceptions.is_match(ua) {
            return None;
        }
        if self.patterns.is_match(ua) {
            return Some("custom");
        }
        classify(ua)
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
        if self.blocks_bot(category) {
            policy.adaptive_mode = self.mode;
            policy.max_stage = 2;
        }
        Some(policy)
    }
    pub fn blocks_bot(&self, category: &str) -> bool {
        self.block_bots
            && matches!(
                category,
                "automation" | "search" | "ai-training" | "ai-search" | "ai-user" | "custom"
            )
    }
    pub fn summary(&self) -> Value {
        json!({"mode":match self.mode{Mode::Off=>"off",Mode::Observe=>"observe",Mode::Enforce=>"enforce"},"categories":self.categories,"weakRequests":self.weak_requests,"crawlerRequests":self.crawler_requests,"windowSecs":self.window_secs,"maxAction":if self.max_stage==1{"throttle"}else{"block"},"scope":if self.anonymous_only{"anonymous"}else{"all"},"adaptive":self.adaptive,"blockBots":self.block_bots,"claimedOnly":true})
    }
}

// Pattern metadata preserves category precedence regardless of textual order.
const CLAIM_GROUPS: &[(&str, &[&str])] = &[
    (
        "ai-user",
        &[
            "chatgpt-user",
            "claude-user",
            "perplexity-user",
            "amzn-user",
            "mistralai-user",
            "meta-externalfetcher",
        ],
    ),
    (
        "ai-search",
        &[
            "oai-searchbot",
            "claude-searchbot",
            "perplexitybot",
            "duckassistbot",
            "amzn-searchbot",
        ],
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
            "google-extended",
            "applebot-extended",
        ],
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
        ],
    ),
];
const AUTOMATION_NAMES: &[&str] = &[
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
];
const BROWSER_FAMILIES: &[&str] = &[
    "chrome/",
    "chromium/",
    "firefox/",
    "safari/",
    "edg/",
    "opr/",
    "thunderbird/",
];
const MAIL_PREFIXES: &[&str] = &[
    "mailwoman/",
    "thunderbird/",
    "outlook/",
    "microsoft office/",
    "applemail/",
    "davx5/",
];
struct ClaimMatcher {
    automaton: aho_corasick::AhoCorasick,
    ranks: Vec<usize>,
}
fn token_boundary(bytes: &[u8], start: usize, end: usize) -> bool {
    let boundary = |byte: u8| !byte.is_ascii_alphanumeric() && byte != b'-' && byte != b'_';
    (start == 0 || boundary(bytes[start - 1])) && (end == bytes.len() || boundary(bytes[end]))
}
fn starts_ascii(value: &str, prefix: &str) -> bool {
    value
        .as_bytes()
        .get(..prefix.len())
        .is_some_and(|bytes| bytes.eq_ignore_ascii_case(prefix.as_bytes()))
}
fn ends_ascii(value: &str, suffix: &str) -> bool {
    value
        .as_bytes()
        .get(value.len().saturating_sub(suffix.len())..)
        .is_some_and(|bytes| bytes.eq_ignore_ascii_case(suffix.as_bytes()))
}
fn classify(raw: &str) -> Option<&'static str> {
    let ua = raw.trim();
    if ua.is_empty() {
        return Some("missing");
    }
    static CLAIMS: std::sync::LazyLock<ClaimMatcher> = std::sync::LazyLock::new(|| {
        let mut patterns = Vec::new();
        let mut ranks = Vec::new();
        for (rank, (_, names)) in CLAIM_GROUPS.iter().enumerate() {
            for name in *names {
                patterns.push(*name);
                ranks.push(rank);
            }
        }
        for name in AUTOMATION_NAMES {
            patterns.push(*name);
            ranks.push(4);
        }
        for family in BROWSER_FAMILIES {
            patterns.push(*family);
            ranks.push(5);
        }
        let automaton = aho_corasick::AhoCorasick::builder()
            .ascii_case_insensitive(true)
            .match_kind(aho_corasick::MatchKind::Standard)
            .build(patterns)
            .expect("bounded built-in user-agent claims");
        ClaimMatcher { automaton, ranks }
    });
    let mut best_rank = usize::MAX;
    let mut browser_family = false;
    for found in CLAIMS.automaton.find_overlapping_iter(ua) {
        let rank = CLAIMS.ranks[found.pattern().as_usize()];
        if rank == 5 {
            browser_family = true;
        } else if token_boundary(ua.as_bytes(), found.start(), found.end()) {
            best_rank = best_rank.min(rank);
            if best_rank == 0 {
                return Some(CLAIM_GROUPS[0].0);
            }
        }
    }
    if best_rank < CLAIM_GROUPS.len() {
        return Some(CLAIM_GROUPS[best_rank].0);
    }
    if best_rank == 4
        || ua
            .split(|character: char| !character.is_ascii_alphanumeric())
            .any(|product| {
                ["bot", "spider", "crawler"]
                    .iter()
                    .any(|suffix| ends_ascii(product, suffix))
            })
    {
        return Some("automation");
    }
    if starts_ascii(ua, "mozilla/5.0") && browser_family
        || MAIL_PREFIXES.iter().any(|prefix| starts_ascii(ua, prefix))
    {
        return None;
    }
    Some("uncommon")
}
#[cfg(test)]
#[path = "user_agent_legacy.rs"]
mod legacy;

pub fn kind(category: &str) -> Option<&'static str> {
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
        assert_eq!(classify("FakeGooglebot/1"), Some("automation"));
        assert_eq!(classify("Google-Extended"), Some("ai-training"));
        assert_eq!(classify("Applebot-Extended"), Some("ai-training"));
        assert_eq!(classify("AhrefsBot/7"), Some("automation"));
        assert_eq!(classify("CustomSpider/1"), Some("automation"));
        assert_eq!(classify("BrowserBotanical/1"), Some("uncommon"));
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

#[cfg(test)]
mod differential_tests {
    use super::*;
    #[test]
    fn optimized_claims_preserve_frozen_precedence_boundaries_and_browser_recognition() {
        let mut cases = vec![
            "".into(),
            "   ".into(),
            "\u{a0}\t".into(),
            "Mozilla/5.0 Chrome/140.0 Safari/537.36".into(),
            "BrowserBotanical/1".into(),
            "é-googlebot/1".into(),
            "curlish/1".into(),
            "JavaScript/2".into(),
            "applemail/1".into(),
            "Mozilla/5.0 NotABrowser".into(),
        ];
        for (_, names) in CLAIM_GROUPS {
            for name in *names {
                for prefix in ["", "Fake", "-", "_", "/", "(compatible; ", "é"] {
                    for suffix in ["", "/1.0", "-extension", "_other", ";", "x"] {
                        cases.push(format!("{prefix}{name}{suffix}"));
                    }
                }
                cases.push(name.to_ascii_uppercase());
                for (other, _) in CLAIM_GROUPS {
                    cases.push(format!("Googlebot/2 {name}/1 {other}"));
                }
            }
        }
        for name in AUTOMATION_NAMES {
            for prefix in ["", "x", "Mozilla/5.0 "] {
                cases.push(format!("{prefix}{name}/1"));
                cases.push(format!("{prefix}{name}-fake"));
            }
        }
        for family in BROWSER_FAMILIES {
            cases.push(format!("Mozilla/5.0 {family}1"));
            cases.push(format!("prefix {family}1"));
        }
        for prefix in MAIL_PREFIXES {
            cases.push(format!("{prefix}1"));
            cases.push(format!(" {prefix}1 "));
            cases.push(format!("x{prefix}1"));
        }
        cases.push("a".repeat(4096));
        cases.push(format!("{} GPTBot/1 Claude-User/1", "Browser/".repeat(490)));
        let fragments = [
            "Googlebot",
            "Applebot-Extended",
            "CLAUDE-USER",
            "GPTBot",
            "Mozilla/5.0",
            "Firefox/1",
            "curl",
            "HeadlessChrome",
            "fake",
            "é",
            "botanical",
            "spider",
            "-",
            "_",
            "/",
            ";",
            " ",
            "\t",
        ];
        let mut seed = 0x6b41_9cf3u64;
        for _ in 0..2000 {
            let mut input = String::new();
            for _ in 0..12 {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                input.push_str(fragments[(seed >> 32) as usize % fragments.len()]);
            }
            cases.push(input);
        }
        for input in cases {
            assert_eq!(classify(&input), legacy::classify(&input), "{input:?}");
        }
    }
    #[test]
    #[ignore = "manual release benchmark; run without concurrent builds"]
    fn policy_match_benchmark_user_agents() {
        assert!(
            !cfg!(debug_assertions),
            "use cargo test --release for performance measurements"
        );
        let ordinary=["Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/145.0.0.0 Safari/537.36","Mozilla/5.0 (X11; Linux x86_64; rv:145.0) Gecko/20100101 Firefox/145.0","Mailwoman/1.0","Thunderbird/145.0","Microsoft Office/16.0",""].map(String::from);
        crate::probe_path::benchmark::compare("ua/ordinary", &ordinary, legacy::classify, classify);
        let hostile = [
            "Mozilla/5.0 (compatible; Googlebot/2.1; +https://www.google.com/bot.html)",
            "GPTBot/1.0",
            "Claude-User/1.0",
            "curl/8",
            "python-requests/2.31",
            "FakeGooglebot/1",
            "Mozilla/5.0 ChatGPT-User/1 GPTBot/1",
        ]
        .map(String::from);
        crate::probe_path::benchmark::compare(
            "ua/claimed-bots",
            &hostile,
            legacy::classify,
            classify,
        );
        let adversarial = vec![
            "a".repeat(4096),
            format!("{}Botanical", "Mozilla/5.0 Browser/".repeat(190)),
            format!("{} Applebot-Extended/1", "junk/".repeat(780)),
        ];
        crate::probe_path::benchmark::compare(
            "ua/long-negative-overlap",
            &adversarial,
            legacy::classify,
            classify,
        );
    }
}

#[cfg(test)]
mod monitored_claim_tests {
    use super::*;
    fn rules(mode: &str) -> Rules {
        let values = [
            ("Mode", json!(mode)),
            ("Categories", json!(["search"])),
            ("WeakRequests", json!(3)),
            ("CrawlerRequests", json!(4)),
            ("WindowSecs", json!(60)),
            ("MaxAction", json!("block")),
            ("Scope", json!("anonymous")),
            ("Adaptive", json!(false)),
            ("BlockBots", json!(false)),
            ("Patterns", json!(["^custom-agent/"])),
            ("Exceptions", json!(["trusted"])),
        ]
        .into_iter()
        .map(|(key, value)| (format!("{}userAgent{key}", crate::SETTINGS_PREFIX), value))
        .collect();
        Rules::parse(&values).unwrap()
    }
    #[test]
    fn monitoring_does_not_reclassify_or_expand_category_policy() {
        let active = rules("observe");
        for (header, claim, monitored) in [
            ("Googlebot/1", Some("search"), Some("search")),
            ("curl/1", Some("automation"), None),
            ("custom-agent/1", Some("custom"), Some("custom")),
            ("custom-agent/trusted", None, None),
            ("", Some("missing"), None),
        ] {
            let claimed = active.claimed_class(Some(header));
            assert_eq!(claimed, claim);
            assert_eq!(active.monitored_class(claimed), monitored);
            assert_eq!(active.classify(Some(header)), monitored);
        }
        let off = rules("off");
        assert_eq!(off.claimed_class(Some("Googlebot/1")), Some("search"));
        assert_eq!(off.monitored_class(Some("search")), None);
        assert_eq!(off.classify(Some("Googlebot/1")), None);
        let oversized = "x".repeat(MAX_UA + 1);
        assert_eq!(active.claimed_class(Some(&oversized)), Some("uncommon"));
        assert_eq!(active.monitored_class(Some("uncommon")), None);
    }
}
