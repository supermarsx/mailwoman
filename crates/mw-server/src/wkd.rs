//! Web Key Directory (WKD) publishing (SPEC §7.3, plan §3 e7) —
//! `draft-koch-openpgp-webkey-service`. Serves the deployment's OWN published
//! public keys at `/.well-known/openpgpkey/...` so external senders can discover
//! them; keys are PUBLIC by design, so the path is unauthenticated. Only keys the
//! operator has explicitly published (dropped into `MW_WKD_DIR`) are ever served.
//!
//! ## Publishing model (self-contained, no engine coupling)
//! `MW_WKD_DIR` holds published public keys, either:
//! * as **address-named files** — `alice@example.org` (optionally with an
//!   `.asc`/`.pgp`/`.gpg`/`.key`/`.pub` extension), binary or ASCII-armored; the
//!   server computes the WKD hash of each local-part on lookup; or
//! * in the **standard gpg-wks layout** — `<domain>/hu/<zbase32hash>` (binary),
//!   exactly what `gpg-wks-client --install-key` produces.
//!
//! Either way the served body is the **binary** transferable public key
//! (`application/octet-stream`), dearmoring armored input as needed — matching the
//! WKD spec (armored keys are not served over WKD).

use std::path::{Path, PathBuf};

use base64::Engine as _;

/// The z-base-32 alphabet WKD uses to encode the SHA-1 local-part hash
/// (`draft-koch-openpgp-webkey-service` §3.1).
const ZBASE32: &[u8; 32] = b"ybndrfg8ejkmcpqxot1uwisza345h769";

/// Compute the WKD `hu/<hash>` path segment for a mailbox local-part: SHA-1 of
/// the lower-cased UTF-8 local-part, z-base-32 encoded to 32 characters.
pub fn wkd_hash(localpart: &str) -> String {
    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    hasher.update(localpart.to_lowercase().as_bytes());
    zbase32_encode(&hasher.finalize())
}

/// z-base-32 encode arbitrary bytes (MSB-first, no padding). A 20-byte SHA-1
/// digest yields exactly 32 characters (160 bits / 5).
fn zbase32_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(5) * 8);
    let mut buffer: u32 = 0;
    let mut bits: u32 = 0;
    for &b in data {
        buffer = (buffer << 8) | b as u32;
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            out.push(ZBASE32[((buffer >> bits) & 0x1f) as usize] as char);
        }
    }
    if bits > 0 {
        out.push(ZBASE32[((buffer << (5 - bits)) & 0x1f) as usize] as char);
    }
    out
}

/// Whether `hash` is a well-formed WKD local-part hash (32 z-base-32 chars). Also
/// a path-traversal guard: a valid hash contains no `.`/`/`/`\`.
pub fn valid_hash(hash: &str) -> bool {
    hash.len() == 32 && hash.bytes().all(|b| ZBASE32.contains(&b))
}

/// Whether `domain` is a plausible DNS name (a-z, 0-9, `.`, `-`) with no empty
/// labels — a path-traversal guard for the standard-layout lookup path.
pub fn valid_domain(domain: &str) -> bool {
    !domain.is_empty()
        && domain.len() <= 253
        && !domain.contains("..")
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && domain
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
}

/// A published-key directory rooted at `MW_WKD_DIR`.
pub struct WkdDirectory {
    root: PathBuf,
}

impl WkdDirectory {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Look up the published key for `(domain, hash)`, returning the BINARY
    /// transferable public key (dearmored if the stored file is armored). Tries
    /// the standard `<domain>/hu/<hash>` layout first, then address-named files.
    /// Callers MUST validate `domain`/`hash` (see [`valid_domain`]/[`valid_hash`])
    /// before calling — this method assumes they are traversal-safe.
    pub fn lookup(&self, domain: &str, hash: &str) -> Option<Vec<u8>> {
        // 1. Standard gpg-wks layout: <root>/<domain>/hu/<hash>.
        let hu = self.root.join(domain).join("hu").join(hash);
        if let Some(bytes) = read_key_file(&hu) {
            return Some(bytes);
        }
        // 2. Address-named files at the root: <local>@<domain>[.ext].
        for entry in std::fs::read_dir(&self.root).ok()?.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let addr = strip_key_ext(&name);
            if let Some((local, dom)) = addr.rsplit_once('@')
                && dom.eq_ignore_ascii_case(domain)
                && wkd_hash(local) == hash
            {
                return read_key_file(&entry.path());
            }
        }
        None
    }
}

/// Strip a recognised public-key file extension from a filename.
fn strip_key_ext(name: &str) -> &str {
    for ext in [".asc", ".pgp", ".gpg", ".key", ".pub"] {
        if let Some(stripped) = name.strip_suffix(ext) {
            return stripped;
        }
    }
    name
}

/// Read a published key file, returning binary transferable-key bytes. Armored
/// input (`-----BEGIN PGP …`) is dearmored; binary input is returned verbatim.
fn read_key_file(path: &Path) -> Option<Vec<u8>> {
    let bytes = std::fs::read(path).ok()?;
    if bytes.starts_with(b"-----BEGIN PGP") {
        dearmor(std::str::from_utf8(&bytes).ok()?)
    } else {
        Some(bytes)
    }
}

/// Decode an ASCII-armored OpenPGP block to its binary body (RFC 9580 §6.2):
/// skip the `-----BEGIN …` line, skip armor headers up to the blank line, then
/// base64-decode the radix-64 body, stopping at the `=CRC` checksum or the
/// `-----END …` line.
fn dearmor(text: &str) -> Option<Vec<u8>> {
    let mut lines = text.lines();
    for line in lines.by_ref() {
        if line.trim_start().starts_with("-----BEGIN PGP") {
            break;
        }
    }
    let mut past_headers = false;
    let mut body = String::new();
    for line in lines {
        let t = line.trim();
        if !past_headers {
            // Armor headers (Version/Comment/…) end at the first blank line.
            if t.is_empty() {
                past_headers = true;
            }
            continue;
        }
        if t.starts_with("-----END") || t.starts_with('=') {
            break;
        }
        body.push_str(t);
    }
    base64::engine::general_purpose::STANDARD
        .decode(body.as_bytes())
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wkd_hash_is_32_zbase32_chars() {
        // Known WKD test vector: "Joe.Doe" → "iy9q119eutrkn8s1mk4r39qejnbu3n5q".
        assert_eq!(wkd_hash("Joe.Doe"), "iy9q119eutrkn8s1mk4r39qejnbu3n5q");
        let h = wkd_hash("alice");
        assert!(valid_hash(&h), "{h}");
    }

    #[test]
    fn hash_is_case_insensitive_on_localpart() {
        assert_eq!(wkd_hash("Alice"), wkd_hash("alice"));
    }

    #[test]
    fn rejects_traversal_domains() {
        assert!(!valid_domain(".."));
        assert!(!valid_domain("a/../b"));
        assert!(!valid_domain(".example.org"));
        assert!(valid_domain("example.org"));
        assert!(valid_domain("mail-1.example.org"));
    }

    #[test]
    fn dearmor_roundtrips_binary() {
        let raw = b"\x99\x01\x02\x03binary key material\xff\x00".to_vec();
        let armored = format!(
            "-----BEGIN PGP PUBLIC KEY BLOCK-----\nComment: test\n\n{}\n=abcd\n-----END PGP PUBLIC KEY BLOCK-----\n",
            base64::engine::general_purpose::STANDARD.encode(&raw)
        );
        assert_eq!(dearmor(&armored).unwrap(), raw);
    }

    #[test]
    fn dearmor_handles_no_headers() {
        let raw = b"no-header-armor".to_vec();
        let armored = format!(
            "-----BEGIN PGP PUBLIC KEY BLOCK-----\n\n{}\n-----END PGP PUBLIC KEY BLOCK-----",
            base64::engine::general_purpose::STANDARD.encode(&raw)
        );
        assert_eq!(dearmor(&armored).unwrap(), raw);
    }

    #[test]
    fn lookup_by_address_name_binary() {
        let dir = tempdir();
        let key = b"\x98binary-pgp-key".to_vec();
        std::fs::write(dir.join("alice@example.org"), &key).unwrap();
        let wkd = WkdDirectory::new(dir.clone());
        let hash = wkd_hash("alice");
        assert_eq!(wkd.lookup("example.org", &hash), Some(key));
        assert_eq!(wkd.lookup("other.org", &hash), None);
    }

    #[test]
    fn lookup_by_address_name_armored_is_dearmored() {
        let dir = tempdir();
        let raw = b"\x99\x01armored-body".to_vec();
        let armored = format!(
            "-----BEGIN PGP PUBLIC KEY BLOCK-----\n\n{}\n-----END PGP PUBLIC KEY BLOCK-----\n",
            base64::engine::general_purpose::STANDARD.encode(&raw)
        );
        std::fs::write(dir.join("bob@example.org.asc"), armored).unwrap();
        let wkd = WkdDirectory::new(dir);
        assert_eq!(wkd.lookup("example.org", &wkd_hash("bob")), Some(raw));
    }

    #[test]
    fn lookup_standard_hu_layout() {
        let dir = tempdir();
        let hash = wkd_hash("carol");
        let hu = dir.join("example.org").join("hu");
        std::fs::create_dir_all(&hu).unwrap();
        let key = b"hu-layout-key".to_vec();
        std::fs::write(hu.join(&hash), &key).unwrap();
        let wkd = WkdDirectory::new(dir);
        assert_eq!(wkd.lookup("example.org", &hash), Some(key));
    }

    fn tempdir() -> PathBuf {
        static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let p = std::env::temp_dir().join(format!("mw-wkd-test-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}
