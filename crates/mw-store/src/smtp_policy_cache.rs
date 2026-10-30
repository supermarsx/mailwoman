//! Durable, bounded MTA-STS cache. Never evict a live policy to make room.
use crate::{Store, StoreError, backend::q};

const PREFIX: &str = "smtp.transport.sts.";
const MAX_ENTRIES: usize = 1024;

impl Store {
    pub async fn cache_smtp_policy(
        &self,
        digest: &str,
        value: &str,
        now: u64,
    ) -> Result<(), StoreError> {
        if digest.len() != 64
            || !digest.bytes().all(|b| b.is_ascii_hexdigit())
            || value.len() > 65536
        {
            return Err(StoreError::Corrupt(
                "invalid SMTP policy cache entry".into(),
            ));
        }
        let incoming: serde_json::Value = serde_json::from_str(value)
            .map_err(|_| StoreError::Corrupt("invalid SMTP policy JSON".into()))?;
        let fetched = incoming
            .get("fetchedAt")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| StoreError::Corrupt("missing SMTP policy timestamp".into()))?;
        let expires = incoming
            .get("expiresAt")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| StoreError::Corrupt("missing SMTP policy expiry".into()))?;
        if expires < fetched || fetched > now.saturating_add(60) {
            return Err(StoreError::Corrupt("invalid SMTP policy timestamps".into()));
        }
        let key = format!("{PREFIX}{}", digest.to_ascii_lowercase());
        let mut tx = self.backend.begin().await?;
        crate::branding::lock(&mut tx).await?;
        let rows = q("SELECT key,value FROM settings WHERE key LIKE 'smtp.transport.sts.%'")
            .fetch_all_tx(&mut tx)
            .await?;
        let mut live = 0;
        let mut exists = false;
        for row in rows {
            let old_key = row.get_string("key");
            let old: serde_json::Value = serde_json::from_str(&row.get_string("value"))
                .map_err(|_| StoreError::Corrupt("invalid cached SMTP policy JSON".into()))?;
            let old_expiry = old
                .get("expiresAt")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| StoreError::Corrupt("invalid cached SMTP policy".into()))?;
            if old_key == key {
                exists = true;
                if old
                    .get("fetchedAt")
                    .and_then(|v| v.as_u64())
                    .is_some_and(|time| time >= fetched)
                {
                    return Ok(()); // A stale or same-second competing fetch must not replace the winner.
                }
            }
            if old_expiry <= now && old_key != key {
                q("DELETE FROM settings WHERE key=?1")
                    .bind(&old_key)
                    .execute_tx(&mut tx)
                    .await?;
            } else {
                live += 1;
            }
        }
        if !exists && live >= MAX_ENTRIES {
            return Err(StoreError::Corrupt(
                "SMTP policy cache is full of unexpired policies".into(),
            ));
        }
        q("INSERT INTO settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value")
            .bind(&key).bind(value).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(())
    }
}
