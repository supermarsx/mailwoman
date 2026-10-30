//! Fetching and unpacking a country database (t96).
//!
//! Every download goes through the egress gate as one
//! [`mw_egress::TrafficClass::GeoipDatabase`] request: HTTPS only, the address
//! policy on every hop, redirects only to the starting host or a listed one, a
//! size ceiling and per-phase timeouts, and the deployment's egress route when one
//! carries the class. Nothing here logs or returns a URL: a MaxMind or IPinfo URL
//! carries the licence key.

use std::io::Read;
use std::time::Duration;

use sha2::{Digest, Sha256};

use super::provider::Plan;

/// The largest download accepted (compressed or not).
pub(crate) const MAX_DOWNLOAD_BYTES: usize = 128 * 1024 * 1024;
/// The largest database accepted after decompression.
pub(crate) const MAX_UNPACKED_BYTES: usize = mw_engine::security::geoip::MAX_DATABASE_BYTES;

/// Why a download failed, in words an administrator can act on. Never a URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FetchError {
    /// The provider answered 404 (DB-IP: this month's file is not out yet).
    NotFound,
    /// Any other failure, already phrased for the console.
    Failed(String),
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchError::NotFound => {
                f.write_str("the provider has no file at that address (HTTP 404)")
            }
            FetchError::Failed(m) => f.write_str(m),
        }
    }
}

fn describe(refusal: mw_egress::Refusal) -> FetchError {
    use mw_egress::Refusal as R;
    match refusal {
        R::Status(404) => FetchError::NotFound,
        R::Status(401 | 403) => FetchError::Failed(
            "the provider refused the request (HTTP 401/403): check the licence key".into(),
        ),
        R::Status(429) => {
            FetchError::Failed("the provider is rate limiting downloads (HTTP 429)".into())
        }
        R::Status(code) => FetchError::Failed(format!("the provider answered HTTP {code}")),
        R::TooLarge => FetchError::Failed("the download is larger than the 128 MB limit".into()),
        R::Timeout => FetchError::Failed("the download timed out".into()),
        R::Blocked => FetchError::Failed("the egress policy does not permit that address".into()),
        R::BadRequest(reason) => FetchError::Failed(format!("refused: {reason}")),
        R::Upstream => FetchError::Failed(
            "the connection to the provider failed (network, TLS or too many redirects)".into(),
        ),
    }
}

/// The per-phase bounds of one download: a few seconds to connect, minutes for
/// the body.
const TIMEOUTS: mw_egress::FetchTimeouts = mw_egress::FetchTimeouts {
    connect: Duration::from_secs(15),
    headers: Duration::from_secs(30),
    idle: Duration::from_secs(30),
    total: Duration::from_secs(300),
    max_redirects: 3,
};

/// GET `url` under the plan's redirect hosts and roots, at most `limit` bytes.
async fn get(plan: &Plan, url: &reqwest::Url, limit: usize) -> Result<Vec<u8>, FetchError> {
    let hosts: Vec<&str> = plan.redirect_hosts.iter().map(String::as_str).collect();
    let download = mw_egress::Download {
        redirect_hosts: &hosts,
        additional_roots: &plan.roots,
    };
    let request = mw_egress::Request::download(mw_egress::TrafficClass::GeoipDatabase, download)
        .map_err(describe)?
        .accept("application/octet-stream, application/gzip, */*")
        .response_limit(limit)
        .timeouts(TIMEOUTS);
    mw_egress::plan::send(url.clone(), &request)
        .await
        .outcome
        .map_err(describe)
}

/// What a download produced.
#[derive(Debug)]
pub(crate) struct Downloaded {
    /// The `.mmdb` bytes, unpacked.
    pub database: Vec<u8>,
    /// SHA-256 of what was downloaded (the archive when there was one).
    pub archive_sha256: String,
    /// Whether a provider checksum was checked.
    pub checksum_verified: bool,
}

/// The first SHA-256 (64 hex digits) in a checksum file, lower-cased.
pub(crate) fn parse_sha256(text: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(text).ok()?;
    text.split_whitespace()
        .next()
        .filter(|t| t.len() == 64 && t.bytes().all(|b| b.is_ascii_hexdigit()))
        .map(str::to_ascii_lowercase)
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The provider's checksum for the download, when the plan names a checksum file.
pub(crate) async fn published_checksum(plan: &Plan) -> Result<Option<String>, FetchError> {
    let Some(url) = &plan.checksum_url else {
        return Ok(None);
    };
    let text = get(plan, url, 4096).await?;
    parse_sha256(&text)
        .map(Some)
        .ok_or_else(|| FetchError::Failed("the provider's checksum file is not a SHA-256".into()))
}

/// Download, verify the checksum when the provider publishes one, and unpack.
pub(crate) async fn fetch(plan: &Plan) -> Result<Downloaded, FetchError> {
    let expected = published_checksum(plan).await?;
    let bytes = get(plan, &plan.url, MAX_DOWNLOAD_BYTES).await?;
    let archive_sha256 = sha256_hex(&bytes);
    if let Some(expected) = &expected
        && *expected != archive_sha256
    {
        return Err(FetchError::Failed(
            "the download does not match the provider's SHA-256 checksum".into(),
        ));
    }
    let database = tokio::task::spawn_blocking(move || unpack(&bytes))
        .await
        .map_err(|_| FetchError::Failed("unpacking failed".into()))?
        .map_err(FetchError::Failed)?;
    Ok(Downloaded {
        database,
        archive_sha256,
        checksum_verified: expected.is_some(),
    })
}

/// Unpack a download: gzip (DB-IP `.mmdb.gz`), gzip+tar (MaxMind `.tar.gz`,
/// the first `*.mmdb` member), or a bare `.mmdb`. Decompression is capped.
pub(crate) fn unpack(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let plain = if bytes.starts_with(&[0x1f, 0x8b]) {
        let mut out = Vec::new();
        flate2::read::GzDecoder::new(bytes)
            .take(MAX_UNPACKED_BYTES as u64 + 1)
            .read_to_end(&mut out)
            .map_err(|_| "the gzip data is damaged".to_string())?;
        if out.len() > MAX_UNPACKED_BYTES {
            return Err("the database is larger than the 256 MB limit when unpacked".into());
        }
        out
    } else {
        bytes.to_vec()
    };
    if plain.len() > 262 && &plain[257..262] == b"ustar" {
        return untar_mmdb(&plain);
    }
    Ok(plain)
}

/// The first regular `*.mmdb` file in a ustar archive.
fn untar_mmdb(tar: &[u8]) -> Result<Vec<u8>, String> {
    let mut at = 0usize;
    while at + 512 <= tar.len() {
        let header = &tar[at..at + 512];
        if header.iter().all(|b| *b == 0) {
            break;
        }
        let field = |range: std::ops::Range<usize>| {
            let raw = &header[range];
            let end = raw.iter().position(|b| *b == 0).unwrap_or(raw.len());
            String::from_utf8_lossy(&raw[..end]).trim().to_string()
        };
        let name = field(0..100);
        let prefix = field(345..500);
        let size = usize::from_str_radix(&field(124..136), 8)
            .map_err(|_| "the archive is damaged".to_string())?;
        let kind = header[156];
        let start = at + 512;
        let end = start
            .checked_add(size)
            .filter(|end| *end <= tar.len())
            .ok_or_else(|| "the archive is truncated".to_string())?;
        let full = if prefix.is_empty() {
            name
        } else {
            format!("{prefix}/{name}")
        };
        if matches!(kind, b'0' | 0) && full.ends_with(".mmdb") {
            return Ok(tar[start..end].to_vec());
        }
        at = start + size.div_ceil(512) * 512;
    }
    Err("the archive holds no .mmdb file".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn gzip(bytes: &[u8]) -> Vec<u8> {
        let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        e.write_all(bytes).unwrap();
        e.finish().unwrap()
    }

    pub(crate) fn tar_with(name: &str, body: &[u8]) -> Vec<u8> {
        let mut h = [0u8; 512];
        h[..name.len()].copy_from_slice(name.as_bytes());
        h[100..107].copy_from_slice(b"0000644");
        let size = format!("{:011o}", body.len());
        h[124..135].copy_from_slice(size.as_bytes());
        h[156] = b'0';
        h[257..263].copy_from_slice(b"ustar\0");
        h[263..265].copy_from_slice(b"00");
        let mut out = h.to_vec();
        out.extend_from_slice(body);
        out.resize(out.len().div_ceil(512) * 512, 0);
        out.extend_from_slice(&[0u8; 1024]);
        out
    }

    #[test]
    fn unpacks_bare_gzip_and_tar_gz() {
        assert_eq!(unpack(b"MMDB").unwrap(), b"MMDB");
        assert_eq!(unpack(&gzip(b"MMDB")).unwrap(), b"MMDB");
        let mut tar = tar_with("GeoLite2-Country_20261027/LICENSE.txt", b"licence");
        tar.truncate(tar.len() - 1024);
        tar.extend(tar_with(
            "GeoLite2-Country_20261027/GeoLite2-Country.mmdb",
            b"the database",
        ));
        assert_eq!(unpack(&gzip(&tar)).unwrap(), b"the database");
        assert!(unpack(&gzip(&tar_with("README", b"x"))).is_err());
        assert!(unpack(&[0x1f, 0x8b, 0, 1, 2]).is_err());
    }

    #[test]
    fn checksum_files_parse() {
        let sum = "a".repeat(64);
        assert_eq!(
            parse_sha256(format!("{sum}  GeoLite2-Country_20261027.tar.gz\n").as_bytes()),
            Some(sum.clone())
        );
        assert_eq!(parse_sha256(b"not a checksum"), None);
    }
}
