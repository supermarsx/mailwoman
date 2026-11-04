//! Small bounded key-value state for session security (binding, rotation
//! markers, notices, the spike baseline), kept in the existing `settings` table.
//!
//! Only keys under the [`PREFIXES`] below may be deleted through this API, so
//! it cannot remove real configuration. Values are JSON written by
//! `mw-server`; nothing here parses them.
use crate::{Store, StoreError, backend::q};

/// Key families this API may delete.
pub const PREFIXES: &[&str] = &[
    "sess-sec:v1:",
    "sess-rot:v1:",
    "sec-notice:v1:",
    "sec-alerts:v1:",
    ACCOUNT_GEO_PREFIX,
];

/// Per-account countries seen in successful sign-ins (`account_geo_history` in
/// `mw-server`): `acct-geo:v1:<account id>`.
pub const ACCOUNT_GEO_PREFIX: &str = "acct-geo:v1:";

impl Store {
    /// Delete one session-security state row. Returns whether a row existed.
    pub async fn delete_sec_state(&self, key: &str) -> Result<bool, StoreError> {
        if !PREFIXES.iter().any(|p| key.starts_with(p)) {
            return Err(StoreError::Corrupt("not a session-security state key".into()));
        }
        Ok(q("DELETE FROM settings WHERE key = ?1")
            .bind(key)
            .execute(&self.backend)
            .await?
            > 0)
    }
}
