//! Remote-content cache, bounded admission and pure policy services.
//! Authentication, SSRF checks and HTTP routing remain the owning application's responsibility.
pub const MAX_CACHE_BYTES: usize = 32 * 1024 * 1024;
pub const CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(3600);
pub use mw_observability::resources;
pub mod adaptive;
pub mod browser_cache;
pub mod capacity;
pub mod demand;
pub mod fetch_limits;
pub mod history;
pub mod policy;
pub mod resize;
pub mod slots;
pub mod tier;
pub use fetch_limits::FetchLimits;

/// Quoted-hex `ETag` of the re-encoded bytes.
pub fn etag_for(bytes: &[u8]) -> String {
    use sha2::Digest;
    let digest = sha2::Sha256::digest(bytes);
    let mut s = String::with_capacity(2 + digest.len() * 2 + 1);
    s.push('"');
    for b in digest {
        s.push_str(&format!("{b:02x}"));
    }
    s.push('"');
    s
}
