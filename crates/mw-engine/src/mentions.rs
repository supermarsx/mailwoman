//! Addressed mentions are indexed once; recipient headers never imply a mention.
use mw_jmap::Email;
pub(crate) const PREFIX: &str = "$mw-mention:";
pub(crate) fn addresses(email: &Email) -> Vec<String> {
    let body = |parts: &[mw_jmap::EmailBodyPart]| {
        parts
            .iter()
            .filter_map(|part| part.part_id.as_ref())
            .filter_map(|id| email.body_values.get(id))
            .map(|value| value.value.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    };
    mw_sanitize::mentions::mentioned_addresses(&body(&email.html_body), &body(&email.text_body))
}
pub(crate) fn keywords(email: &Email) -> Vec<String> {
    addresses(email)
        .into_iter()
        .map(|address| format!("{PREFIX}{address}"))
        .collect()
}
pub(crate) fn from_keywords(keywords: &[String]) -> Vec<String> {
    keywords
        .iter()
        .filter_map(|value| value.strip_prefix(PREFIX))
        .map(str::to_string)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn index_and_projection_share_exact_addresses_not_recipient_lists() {
        let email: Email = serde_json::from_value(serde_json::json!({"to":[{"email":"recipient@example.org"}],"htmlBody":[{"partId":"1","type":"text/html"}],"bodyValues":{"1":{"value":"<p><a href='mailto:alias%40example.org'>@Alias</a></p><blockquote>@old@example.org</blockquote>"}}})).unwrap();
        let keys = keywords(&email);
        assert_eq!(keys, ["$mw-mention:alias@example.org"]);
        assert_eq!(from_keywords(&keys), ["alias@example.org"]);
        let only_to: Email =
            serde_json::from_value(serde_json::json!({"to":[{"email":"recipient@example.org"}]}))
                .unwrap();
        assert!(keywords(&only_to).is_empty());
    }
}
