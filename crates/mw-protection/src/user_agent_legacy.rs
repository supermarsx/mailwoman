//! Frozen pre-optimization claim classifier; test/benchmark use only.
fn token(ua: &str, needle: &str) -> bool {
    ua.match_indices(needle).any(|(at, _)| {
        let boundary = |byte: u8| !byte.is_ascii_alphanumeric() && byte != b'-' && byte != b'_';
        (at == 0 || boundary(ua.as_bytes()[at - 1]))
            && (at + needle.len() == ua.len() || boundary(ua.as_bytes()[at + needle.len()]))
    })
}
pub(super) fn classify(raw: &str) -> Option<&'static str> {
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
                "google-extended",
                "applebot-extended",
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
    if ua
        .split(|character: char| !character.is_ascii_alphanumeric())
        .any(|product| {
            product.ends_with("bot") || product.ends_with("spider") || product.ends_with("crawler")
        })
        || [
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
