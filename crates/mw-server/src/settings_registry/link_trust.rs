//! Trusted link domains (t39 R7): `links.trustedDomains` and
//! `links.trustedSubdomains`.
//!
//! The web reader decides whether a reviewed message link may open without the
//! destination step (`apps/web/src/viewers/linkTrust.ts`); this module holds
//! the same rules so the registry refuses a stored entry the reader would
//! ignore, and so both sides are checked against one set of vectors
//! (`apps/web/src/viewers/linkTrust.vectors.json`).
//!
//! Matching is on whole DNS labels. `example.com` trusts `example.com`, and
//! with subdomains on also `mail.example.com`; it never trusts
//! `evilexample.com`, `example.com.evil.test`, an IP literal, or anything
//! through a single-label entry such as `com`.

/// The canonical form of a list entry: lowercase ASCII (A-label) labels of
/// 1-63 letters, digits and inner hyphens, at least two labels, a last label
/// that is not all digits, at most 253 bytes. One trailing dot is dropped.
/// Anything else (a wildcard, scheme, path, port, address, empty label) is
/// `None`.
pub fn normalize(value: &str) -> Option<String> {
    let trimmed = value.trim();
    let domain = trimmed.strip_suffix('.').unwrap_or(trimmed);
    if domain.is_empty() || domain.len() > 253 || !domain.is_ascii() {
        return None;
    }
    let domain = domain.to_ascii_lowercase();
    let labels: Vec<&str> = domain.split('.').collect();
    if labels.len() < 2 || labels.iter().any(|label| !valid_label(label)) {
        return None;
    }
    if labels
        .last()
        .is_some_and(|last| last.bytes().all(|b| b.is_ascii_digit()))
    {
        return None;
    }
    Some(domain)
}

fn valid_label(label: &str) -> bool {
    let bytes = label.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= 63
        && bytes
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
        && bytes[0] != b'-'
        && bytes[bytes.len() - 1] != b'-'
}

/// Whether a stored entry is already canonical, which is what the registry
/// accepts (the web app normalizes before it writes).
pub fn is_canonical(entry: &str) -> bool {
    normalize(entry).as_deref() == Some(entry)
}

/// Whether `host` (a URL hostname) is trusted by `domains`.
pub fn is_trusted<S: AsRef<str>>(host: &str, domains: &[S], subdomains: bool) -> bool {
    let Some(host) = normalize(host) else {
        return false;
    };
    domains
        .iter()
        .filter_map(|d| normalize(d.as_ref()))
        .any(|domain| {
            host == domain
                || (subdomains
                    && host.len() > domain.len()
                    && host.ends_with(&domain)
                    && host.as_bytes()[host.len() - domain.len() - 1] == b'.')
        })
}
