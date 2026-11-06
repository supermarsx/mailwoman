//! Message priority from the headers senders use for it.
//!
//! `X-Priority` (1 and 2 high, 4 and 5 low, 3 normal, or a word), `Importance`
//! (RFC 2156), `Priority` (RFC 2156: `urgent`, `normal`, `non-urgent`) and
//! `X-MSMail-Priority`. `Precedence` (`bulk`, `list`) names a kind of mail, not
//! how urgent it is, so it is not read.

/// `"high"` or `"low"`; `None` for normal and for no priority header at all.
#[must_use]
pub fn priority_from_headers<'a>(
    mut get: impl FnMut(&str) -> Option<&'a str>,
) -> Option<&'static str> {
    // First header that says something wins, in this order.
    for (name, rule) in [
        ("X-Priority", x_priority as fn(&str) -> Option<Level>),
        ("Importance", word_level),
        ("X-MSMail-Priority", word_level),
        ("Priority", word_level),
    ] {
        if let Some(level) = get(name).and_then(rule) {
            return match level {
                Level::High => Some("high"),
                Level::Low => Some("low"),
                Level::Normal => None,
            };
        }
    }
    None
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Level {
    High,
    Normal,
    Low,
}

fn x_priority(value: &str) -> Option<Level> {
    let v = value.trim();
    match v.chars().next()? {
        '1' | '2' => Some(Level::High),
        '3' => Some(Level::Normal),
        '4' | '5' => Some(Level::Low),
        _ => word_level(v),
    }
}

fn word_level(value: &str) -> Option<Level> {
    let v = value.trim().to_ascii_lowercase();
    // "2 (High)", "5 (Lowest)": the word after the number is read too.
    let word = v
        .trim_start_matches(|c: char| c.is_ascii_digit() || c == '(' || c.is_whitespace())
        .trim_end_matches(')')
        .trim();
    match word {
        "high" | "highest" | "urgent" | "important" | "emergency" => Some(Level::High),
        "low" | "lowest" | "non-urgent" | "nonurgent" | "not urgent" => Some(Level::Low),
        "normal" | "medium" | "routine" => Some(Level::Normal),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn of(pairs: &[(&str, &str)]) -> Option<&'static str> {
        priority_from_headers(|n| {
            pairs
                .iter()
                .find(|(h, _)| h.eq_ignore_ascii_case(n))
                .map(|(_, v)| *v)
        })
    }

    #[test]
    fn x_priority_numbers_and_words() {
        assert_eq!(of(&[("X-Priority", "1")]), Some("high"));
        assert_eq!(of(&[("X-Priority", "2 (High)")]), Some("high"));
        assert_eq!(of(&[("X-Priority", "3 (Normal)")]), None);
        assert_eq!(of(&[("X-Priority", "4")]), Some("low"));
        assert_eq!(of(&[("X-Priority", "5 (Lowest)")]), Some("low"));
        assert_eq!(of(&[("X-Priority", "High")]), Some("high"));
        assert_eq!(of(&[("X-Priority", "Low")]), Some("low"));
    }

    #[test]
    fn importance_priority_and_msmail() {
        assert_eq!(of(&[("Importance", "High")]), Some("high"));
        assert_eq!(of(&[("importance", "low")]), Some("low"));
        assert_eq!(of(&[("Importance", "normal")]), None);
        assert_eq!(of(&[("Priority", "urgent")]), Some("high"));
        assert_eq!(of(&[("Priority", "non-urgent")]), Some("low"));
        assert_eq!(of(&[("X-MSMail-Priority", "High")]), Some("high"));
        assert_eq!(of(&[("X-MSMail-Priority", "Low")]), Some("low"));
    }

    #[test]
    fn precedence_is_not_priority_and_garbage_is_ignored() {
        assert_eq!(of(&[("Precedence", "bulk")]), None);
        assert_eq!(of(&[("Precedence", "junk"), ("Importance", "high")]), Some("high"));
        assert_eq!(of(&[("X-Priority", "banana"), ("Importance", "low")]), Some("low"));
        assert_eq!(of(&[]), None);
    }

    #[test]
    fn x_priority_wins_over_importance() {
        assert_eq!(of(&[("X-Priority", "3"), ("Importance", "high")]), None);
        assert_eq!(of(&[("X-Priority", "1"), ("Importance", "low")]), Some("high"));
    }
}
