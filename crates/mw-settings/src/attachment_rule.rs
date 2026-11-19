//! Administrator attachment-rule validation, shared by the registry and reader policy.
pub fn valid_rule(rule: &str, mime: bool) -> bool {
    if rule.is_empty() || rule.len() > 127 || rule != rule.to_ascii_lowercase() {
        return false;
    }
    if mime {
        let Some((top, sub)) = rule.split_once('/') else {
            return false;
        };
        let token = |s: &str| {
            !s.is_empty()
                && s.bytes().all(|b| {
                    b.is_ascii_lowercase()
                        || b.is_ascii_digit()
                        || matches!(
                            b,
                            b'!' | b'#' | b'$' | b'&' | b'^' | b'_' | b'.' | b'+' | b'-'
                        )
                })
        };
        token(top) && (sub == "*" || token(sub))
    } else {
        rule.len() <= 32
            && rule.split('.').all(|part| {
                !part.is_empty()
                    && part.bytes().all(|b| {
                        b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'-')
                    })
            })
    }
}
