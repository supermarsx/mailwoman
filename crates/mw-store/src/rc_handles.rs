//! Short opaque handles for signed remote-content links, backed by the shared
//! `settings` table so every replica resolves the same handle.
//!
//! A row is `rc.h.<handle>` = `<expiry, 12 zero-padded digits>|<claims JSON>`.
//! The zero-padded expiry makes `ORDER BY value` an expiry ordering, which is
//! how expired rows are purged and how the oldest rows are evicted at the cap.
//! The store never interprets the claims; the caller owns their meaning.
use crate::{Store, StoreError, q};

const PREFIX: &str = "rc.h.";
/// Handle length: 16 random bytes, base64url without padding.
pub const HANDLE_LEN: usize = 22;
/// Largest claims document accepted for one handle.
pub const MAX_CLAIMS_BYTES: usize = 8192;

/// True when `s` has the shape of a handle (never a legacy `payload.tag` token).
pub fn is_handle(s: &str) -> bool {
    s.len() == HANDLE_LEN && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

impl Store {
    /// Store `entries` as `(handle, expiry_unix, claims_json)`. Expired rows are
    /// purged first; if the live count would then exceed `max_live`, the rows
    /// closest to expiry are evicted to make room.
    pub async fn rc_handles_put(
        &self,
        entries: &[(String, i64, String)],
        now: i64,
        max_live: usize,
    ) -> Result<(), StoreError> {
        if entries.is_empty() {
            return Ok(());
        }
        if entries.len() > max_live {
            return Err(StoreError::Corrupt("too many remote-content handles".into()));
        }
        for (h, x, json) in entries {
            if !is_handle(h) || *x < 0 || *x > 999_999_999_999 || json.len() > MAX_CLAIMS_BYTES {
                return Err(StoreError::Corrupt("invalid remote-content handle".into()));
            }
        }
        let mut tx = self.backend.begin().await?;
        crate::branding::lock(&mut tx).await?;
        q("DELETE FROM settings WHERE key LIKE 'rc.h.%' AND value < ?1")
            .bind(format!("{:012}|", now.max(0)))
            .execute_tx(&mut tx)
            .await?;
        let live = q("SELECT COUNT(*) AS n FROM settings WHERE key LIKE 'rc.h.%'")
            .fetch_all_tx(&mut tx)
            .await?
            .first()
            .map_or(0, |r| r.get_i64("n")) as usize;
        if live + entries.len() > max_live {
            let excess = (live + entries.len() - max_live) as i64;
            q("DELETE FROM settings WHERE key IN (SELECT key FROM settings WHERE key LIKE 'rc.h.%' ORDER BY value, key LIMIT ?1)")
                .bind(excess)
                .execute_tx(&mut tx)
                .await?;
        }
        for (h, x, json) in entries {
            q("INSERT INTO settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO NOTHING")
                .bind(format!("{PREFIX}{h}"))
                .bind(format!("{x:012}|{json}"))
                .execute_tx(&mut tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// The claims JSON for a live handle; `None` if unknown, expired or evicted.
    pub async fn rc_handle_get(&self, handle: &str, now: i64) -> Result<Option<String>, StoreError> {
        if !is_handle(handle) {
            return Ok(None);
        }
        let Some(value) = self.get_setting(&format!("{PREFIX}{handle}")).await? else {
            return Ok(None);
        };
        let Some((x, json)) = value.split_once('|') else {
            return Ok(None);
        };
        Ok(match x.parse::<i64>() {
            Ok(x) if x >= now => Some(json.to_string()),
            _ => None,
        })
    }

    /// Live handle count (for tests and metrics).
    pub async fn rc_handle_count(&self) -> Result<usize, StoreError> {
        let row = q("SELECT COUNT(*) AS n FROM settings WHERE key LIKE 'rc.h.%'")
            .fetch_one(&self.backend)
            .await?;
        Ok(row.get_i64("n") as usize)
    }
}
