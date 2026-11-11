//! Bounded display patterns, never date parsing or executable templates.
pub(crate) fn valid_region(value: &str) -> bool {
    value == "auto" || value.len() == 2 && value.bytes().all(|byte| byte.is_ascii_uppercase())
}
pub(crate) fn valid_format(value: &str, date: bool) -> bool {
    if ["auto", "short", "medium", "long", "full"].contains(&value) { return true; }
    if value.is_empty() || value.len() > 120 || value.chars().any(char::is_control) { return false; }
    let tokens: &[&str] = if date { &["yyyy", "yy", "MMMM", "MMM", "MM", "M", "dd", "d", "EEEE", "EEE"] }
        else { &["HH", "H", "hh", "h", "mm", "m", "ss", "s", "aaaa", "a", "zzzz", "z"] };
    let mut rest = value;
    let mut seen = false;
    while !rest.is_empty() {
        if let Some(next) = rest.strip_prefix("''") { rest = next; continue; }
        if let Some(next) = rest.strip_prefix('\'') {
            rest = next;
            let mut closed = false;
            while !rest.is_empty() {
                if let Some(next) = rest.strip_prefix("''") { rest = next; }
                else if let Some(next) = rest.strip_prefix('\'') { rest = next; closed = true; break; }
                else { rest = &rest[rest.chars().next().unwrap().len_utf8()..]; }
            }
            if !closed { return false; }
        } else if let Some(token) = tokens.iter().find(|token| rest.starts_with(**token)) {
            rest = &rest[token.len()..]; seen = true;
        } else {
            let ch = rest.chars().next().unwrap();
            if ch.is_ascii_alphabetic() { return false; }
            rest = &rest[ch.len_utf8()..];
        }
    }
    seen
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_presets_custom_order_and_quoted_localized_text() {
        for value in ["auto", "short", "yyyy-MM-dd", "dd/MM/yyyy", "EEEE, d 'de' MMMM 'de' yyyy", "yyyy年MM月dd日", "dd 'o''clock' MM yyyy"] { assert!(valid_format(value, true), "{value}"); }
        for value in ["auto", "medium", "HH:mm:ss", "h:mm a", "hh:mm:ss a z", "H 'hours' m"] { assert!(valid_format(value, false), "{value}"); }
    }
    #[test]
    fn rejects_typos_control_characters_unclosed_quotes_and_unbounded_input() {
        for value in ["", "bogus", "YYYY-MM-DD", "dd/MM/yyyy\n", "yyyy 'unfinished", "'literal only'", "HH:mm"] { assert!(!valid_format(value, true), "{value:?}"); }
        for value in ["yyyy-MM-dd", "HH:MM", "HH:mm\u{0085}"] { assert!(!valid_format(value, false), "{value:?}"); }
        assert!(!valid_format(&"dd/".repeat(50), true));
    }
    #[test]
    fn regions_accept_auto_or_uppercase_two_letter_tags() {
        for value in ["auto", "PT", "GB", "US", "BR"] { assert!(valid_region(value)); }
        for value in ["", "pt", "en-GB", "USA", "1A", "PT\n"] { assert!(!valid_region(value)); }
    }
}
