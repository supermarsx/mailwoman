//! Public-key discovery helpers (C2). The **keys.openpgp.org Verifying Key Server
//! (VKS)** HTTP API complements the WKD path in [`crate::pgp`]: WKD serves a key
//! from the address' own domain, VKS is a central, identity-verifying pool. Pure
//! URL derivation is testable offline; the HTTPS GET is native-only and rides the
//! same rustls `reqwest` as [`crate::pgp::wkd_fetch`].
//!
//! VKS (draft-shaw-openpgp-hkp-… successor) exposes three lookup routes returning
//! an ASCII-armored transferable public key (`application/pgp-keys`), or `404` when
//! nothing is published/verified:
//! - `by-email/<addr>`       — only returns a key whose address is **verified**.
//! - `by-fingerprint/<FPR>`  — full (v4/v6) fingerprint, uppercase hex.
//! - `by-keyid/<KEYID>`      — 16-hex 64-bit key id.

use crate::error::{CryptoError, Result};
#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
use crate::types::CryptoKey;

/// The default keys.openpgp.org VKS base URL (no trailing slash).
pub const VKS_DEFAULT_BASE: &str = "https://keys.openpgp.org";

/// The VKS `by-email` URL for `email` against `base` (e.g. [`VKS_DEFAULT_BASE`]).
/// The address is lowercased and percent-encoded into the path segment.
pub fn vks_url_by_email(base: &str, email: &str) -> String {
    format!(
        "{}/vks/v1/by-email/{}",
        base.trim_end_matches('/'),
        urlencode(&email.trim().to_lowercase())
    )
}

/// The VKS `by-fingerprint` URL for `fingerprint` (uppercased hex, non-hex stripped).
pub fn vks_url_by_fingerprint(base: &str, fingerprint: &str) -> String {
    let fpr: String = fingerprint
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .collect::<String>()
        .to_uppercase();
    format!(
        "{}/vks/v1/by-fingerprint/{}",
        base.trim_end_matches('/'),
        fpr
    )
}

/// The VKS `by-keyid` URL for a 64-bit `key_id` (uppercased hex, non-hex stripped).
pub fn vks_url_by_keyid(base: &str, key_id: &str) -> String {
    let kid: String = key_id
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .collect::<String>()
        .to_uppercase();
    format!("{}/vks/v1/by-keyid/{}", base.trim_end_matches('/'), kid)
}

/// Look up a public key by email via keys.openpgp.org VKS (native only — HTTPS GET).
/// Returns the parsed [`CryptoKey`] (`source = "vks"`, `trust = "unverified"` — VKS
/// verifies address control, not identity, so the keyring still applies TOFU).
#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub async fn vks_lookup_by_email(email: &str) -> Result<CryptoKey> {
    let url = vks_url_by_email(VKS_DEFAULT_BASE, email);
    let armored = vks_get(&url).await?;
    let (mut key, _) = crate::pgp::parse_key(&armored, vec![email.trim().to_lowercase()])?;
    key.source = "vks".into();
    Ok(key)
}

/// Look up a public key by fingerprint via keys.openpgp.org VKS (native only).
#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub async fn vks_lookup_by_fingerprint(fingerprint: &str) -> Result<CryptoKey> {
    let url = vks_url_by_fingerprint(VKS_DEFAULT_BASE, fingerprint);
    let armored = vks_get(&url).await?;
    let (mut key, _) = crate::pgp::parse_key(&armored, vec![])?;
    key.source = "vks".into();
    Ok(key)
}

/// HTTPS GET a VKS URL, returning the armored key body. `404` → a clear "not found".
#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
async fn vks_get(url: &str) -> Result<String> {
    let resp = reqwest::get(url)
        .await
        .map_err(|e| CryptoError::Io(e.to_string()))?;
    if resp.status().as_u16() == 404 {
        return Err(CryptoError::Input(
            "no key published for that lookup".into(),
        ));
    }
    if !resp.status().is_success() {
        return Err(CryptoError::Io(format!(
            "VKS lookup failed: HTTP {}",
            resp.status()
        )));
    }
    let body = resp
        .text()
        .await
        .map_err(|e| CryptoError::Io(e.to_string()))?;
    if !body.contains("BEGIN PGP PUBLIC KEY") {
        return Err(CryptoError::Parse(
            "VKS response is not an armored public key".into(),
        ));
    }
    Ok(body)
}

/// Minimal percent-encoding for a path segment (RFC 3986 unreserved kept verbatim;
/// everything else — including `@` → `%40` — is escaped).
fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn by_email_url_lowercases_and_encodes() {
        let url = vks_url_by_email(VKS_DEFAULT_BASE, "Joe.Doe@Example.ORG");
        assert_eq!(
            url,
            "https://keys.openpgp.org/vks/v1/by-email/joe.doe%40example.org"
        );
    }

    #[test]
    fn by_fingerprint_url_uppercases_hex() {
        let url = vks_url_by_fingerprint(VKS_DEFAULT_BASE, "abcd 1234:ef");
        assert_eq!(
            url,
            "https://keys.openpgp.org/vks/v1/by-fingerprint/ABCD1234EF"
        );
    }

    #[test]
    fn base_trailing_slash_trimmed() {
        let url = vks_url_by_keyid("https://keys.openpgp.org/", "DEADbeef00112233");
        assert_eq!(
            url,
            "https://keys.openpgp.org/vks/v1/by-keyid/DEADBEEF00112233"
        );
    }
}
