//! Which addresses a certificate is for, and whether a sender's address is one of
//! them (t32-ext §10.3 W): the subjectAltName `rfc822Name` entries, exactly; the
//! subject's PKCS#9 `emailAddress` only when the certificate has no
//! subjectAltName extension at all. Domains compare after IDN normalisation (lower case,
//! A-labels via Punycode, RFC 3492); the local part compares case-insensitively
//! unless the policy asks for an exact match.

use der::Decode;
use x509_cert::Certificate;
use x509_cert::ext::pkix::SubjectAltName;
use x509_cert::ext::pkix::name::GeneralName;

use super::oid;

/// The addresses a certificate names.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CertAddresses {
    /// A SAN extension suppresses legacy DN fallback even if it has no email.
    pub has_san: bool,
    /// subjectAltName `rfc822Name` entries, as written.
    pub san: Vec<String>,
    /// Subject DN `emailAddress` attributes, as written.
    pub dn: Vec<String>,
}

impl CertAddresses {
    /// The addresses that count for matching: the SAN ones, or the DN ones when
    /// there is no SAN address.
    pub fn effective(&self) -> &[String] {
        if self.has_san { &self.san } else { &self.dn }
    }
}

pub fn cert_addresses(cert: &Certificate) -> CertAddresses {
    let mut out = CertAddresses::default();
    for ext in cert.tbs_certificate().extensions().into_iter().flatten() {
        if ext.extn_id != oid::SUBJECT_ALT_NAME {
            continue;
        }
        out.has_san = true;
        if let Ok(san) = SubjectAltName::from_der(ext.extn_value.as_bytes()) {
            for name in &san.0 {
                if let GeneralName::Rfc822Name(addr) = name {
                    out.san.push(addr.as_str().trim().to_string());
                }
            }
        }
    }
    for rdn in cert.tbs_certificate().subject().iter_rdn() {
        for atv in rdn.iter() {
            if atv.oid == oid::EMAIL_ADDRESS
                && let Ok(addr) = core::str::from_utf8(atv.value.value())
            {
                out.dn.push(addr.trim().to_string());
            }
        }
    }
    out
}

/// Does `sender` match one of the certificate's addresses?
pub fn matches_sender(cert: &Certificate, sender: &str, local_case_insensitive: bool) -> bool {
    let Some(want) = normalize_address(sender, local_case_insensitive) else {
        return false;
    };
    cert_addresses(cert)
        .effective()
        .iter()
        .filter_map(|a| normalize_address(a, local_case_insensitive))
        .any(|a| a == want)
}

/// `local@domain` with the domain IDN-normalised and, when `local_ci`, the local
/// part lower-cased. `None` for something that is not an address.
pub fn normalize_address(addr: &str, local_ci: bool) -> Option<String> {
    let addr = addr.trim().trim_start_matches('<').trim_end_matches('>');
    let (local, domain) = addr.rsplit_once('@')?;
    if local.is_empty() || domain.is_empty() {
        return None;
    }
    let local = if local_ci {
        local.to_lowercase()
    } else {
        local.to_string()
    };
    Some(format!("{local}@{}", idn_domain(domain)?))
}

/// A domain in its comparable ASCII form: lower case, trailing dot dropped, each
/// non-ASCII label replaced by its `xn--` A-label.
pub fn idn_domain(domain: &str) -> Option<String> {
    let domain = domain.trim_end_matches('.').to_lowercase();
    let labels: Option<Vec<String>> = domain
        .split('.')
        .map(|label| {
            if label.is_empty() {
                None
            } else if label.is_ascii() {
                Some(label.to_string())
            } else {
                punycode_encode(label).map(|p| format!("xn--{p}"))
            }
        })
        .collect();
    labels.map(|l| l.join("."))
}

/// Punycode encoding (RFC 3492 §6.3) of one label.
pub fn punycode_encode(input: &str) -> Option<String> {
    const BASE: u32 = 36;
    const TMIN: u32 = 1;
    const TMAX: u32 = 26;
    const SKEW: u32 = 38;
    const DAMP: u32 = 700;
    const INITIAL_BIAS: u32 = 72;
    const INITIAL_N: u32 = 128;

    fn adapt(mut delta: u32, numpoints: u32, first: bool) -> u32 {
        delta = if first { delta / DAMP } else { delta / 2 };
        delta += delta / numpoints;
        let mut k = 0;
        while delta > ((BASE - TMIN) * TMAX) / 2 {
            delta /= BASE - TMIN;
            k += BASE;
        }
        k + (((BASE - TMIN + 1) * delta) / (delta + SKEW))
    }
    fn digit(d: u32) -> char {
        if d < 26 {
            (b'a' + d as u8) as char
        } else {
            (b'0' + (d - 26) as u8) as char
        }
    }

    let cps: Vec<u32> = input.chars().map(u32::from).collect();
    let mut out: String = input.chars().filter(char::is_ascii).collect();
    let basic = out.len() as u32;
    let mut h = basic;
    if basic > 0 {
        out.push('-');
    }
    let mut n = INITIAL_N;
    let mut delta: u32 = 0;
    let mut bias = INITIAL_BIAS;
    while (h as usize) < cps.len() {
        let m = *cps.iter().filter(|&&c| c >= n).min()?;
        delta = delta.checked_add((m - n).checked_mul(h + 1)?)?;
        n = m;
        for &c in &cps {
            if c < n {
                delta = delta.checked_add(1)?;
            }
            if c == n {
                let mut q = delta;
                let mut k = BASE;
                loop {
                    let t = if k <= bias {
                        TMIN
                    } else if k >= bias + TMAX {
                        TMAX
                    } else {
                        k - bias
                    };
                    if q < t {
                        break;
                    }
                    out.push(digit(t + (q - t) % (BASE - t)));
                    q = (q - t) / (BASE - t);
                    k += BASE;
                }
                out.push(digit(q));
                bias = adapt(delta, h + 1, h == basic);
                delta = 0;
                h += 1;
            }
        }
        delta = delta.checked_add(1)?;
        n = n.checked_add(1)?;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn punycode_rfc3492_samples() {
        // RFC 3492 §7.1 (B) Chinese (simplified) and (L) Japanese samples, and the
        // common "bücher" example.
        assert_eq!(
            punycode_encode("他们为什么不说中文").as_deref(),
            Some("ihqwcrb4cv8a8dqg056pqjye")
        );
        assert_eq!(
            punycode_encode("3年B組金八先生").as_deref(),
            Some("3B-ww4c5e180e575a65lsy2b")
        );
        assert_eq!(punycode_encode("bücher").as_deref(), Some("bcher-kva"));
    }

    #[test]
    fn addresses_normalise() {
        assert_eq!(
            normalize_address("Ann@Bücher.Example.", true).as_deref(),
            Some("ann@xn--bcher-kva.example")
        );
        assert_eq!(
            normalize_address("Ann@example.com", false).as_deref(),
            Some("Ann@example.com")
        );
        assert!(normalize_address("no-at", true).is_none());
        assert!(normalize_address("a@b..c", true).is_none());
    }
}
