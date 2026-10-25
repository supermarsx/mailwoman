//! Local format/check-digit detectors, not verification of issued identities.
//! CC follows AMA's "Validação de Número de Documento" algorithm (2009).
//! Identifiers are never returned or logged. Separators are accepted, but a
//! matching substring inside an identifier or a longer digit run is refused.
use regex::Regex;
use std::sync::LazyLock;

static NINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[0-9](?:[ \t\x{00a0}.-]?[0-9]){8}").unwrap());
static TAX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:PT[ \t\x{00a0}]?)?[0-9](?:[ \t\x{00a0}.-]?[0-9]){8}").unwrap()
});
static ELEVEN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[0-9](?:[ \t\x{00a0}.-]?[0-9]){10}").unwrap());
static CARD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)[0-9]{8}[ \t\x{00a0}-]?[0-9][ \t\x{00a0}-]?[a-z0-9]{2}[0-9]").unwrap()
});

fn scan(text: &str, pattern: &Regex, valid: impl Fn(&[u8]) -> bool) -> bool {
    pattern.find_iter(text).any(|m| {
        let word = |c: char| c.is_alphanumeric() || c == '_';
        if text[..m.start()].chars().next_back().is_some_and(word)
            || text[m.end()..].chars().next().is_some_and(word)
        {
            return false;
        }
        let compact: Vec<u8> = m
            .as_str()
            .bytes()
            .filter(u8::is_ascii_alphanumeric)
            .map(|b| {
                if b.is_ascii_digit() {
                    b - b'0'
                } else {
                    b.to_ascii_uppercase() - b'A' + 10
                }
            })
            .collect();
        valid(&compact)
    })
}

fn mod11(digits: &[u8]) -> bool {
    let sum: u32 = digits[..8]
        .iter()
        .enumerate()
        .map(|(i, d)| u32::from(*d) * (9 - i as u32))
        .sum();
    let check = 11 - sum % 11;
    u32::from(digits[8]) == if check >= 10 { 0 } else { check }
}

pub(super) fn nif(text: &str) -> bool {
    // Do not freeze an allocation-prefix list: valid newer allocations must
    // still be protected. The checksum and nonzero first digit are stable.
    scan(text, &TAX, |d| {
        let digits = if d.len() == 11 { &d[2..] } else { d };
        digits[0] != 0 && mod11(digits)
    })
}

pub(super) fn niss(text: &str) -> bool {
    scan(text, &ELEVEN, |d| {
        let weights = [29, 23, 19, 17, 13, 11, 7, 5, 3, 2];
        let sum: u32 = d[..10]
            .iter()
            .zip(weights)
            .map(|(n, w)| u32::from(*n) * w)
            .sum();
        matches!(d[0], 1 | 2) && u32::from(d[10]) == 9 - sum % 10
    })
}

pub(super) fn citizen_card(text: &str) -> bool {
    scan(text, &CARD, |d| {
        let sum: u32 = d
            .iter()
            .rev()
            .enumerate()
            .map(|(i, n)| {
                let value = u32::from(*n) * if i % 2 == 1 { 2 } else { 1 };
                if i % 2 == 1 && value >= 10 {
                    value - 9
                } else {
                    value
                }
            })
            .sum();
        sum.is_multiple_of(10)
    }) || scan(text, &NINE, |d| {
        // Legacy BI / civil number with its check digit. Leading zero is valid.
        d[..8].iter().any(|d| *d != 0) && mod11(d)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn portuguese_nif_checks_and_boundaries() {
        for value in [
            "123456789",
            "NIF: 123 456 789.",
            "123.456.789",
            "123\u{a0}456\u{a0}789",
            "PT123456789",
            "pt 123 456 789",
        ] {
            assert!(nif(value), "{value}");
        }
        for value in [
            "123456788",
            "000000000",
            "9123456789",
            "1234567890",
            "x123456789",
            "123456789é",
        ] {
            assert!(!nif(value), "{value}");
        }
    }
    #[test]
    fn portuguese_niss_checks_and_boundaries() {
        // Synthetic checksum fixture: weighted prefix sum 29; final digit 0.
        for value in ["10000000000", "NISS: 100 000 000 00"] {
            assert!(niss(value), "{value}");
        }
        for value in [
            "10000000001",
            "00000000009",
            "310000000000",
            "100000000000",
            "id10000000000",
        ] {
            assert!(!niss(value), "{value}");
        }
    }
    #[test]
    fn citizen_card_official_example_and_legacy_civil_number() {
        for value in [
            "00000000 0 ZZ4",
            "000000000zz4",
            "CC: 00000000-0-ZZ4",
            "12345678 9",
        ] {
            assert!(citizen_card(value), "{value}");
        }
        for value in [
            "00000000 0 ZZ5",
            "x000000000ZZ4",
            "000000000ZZ40",
            "123456788",
        ] {
            assert!(!citizen_card(value), "{value}");
        }
    }
}
