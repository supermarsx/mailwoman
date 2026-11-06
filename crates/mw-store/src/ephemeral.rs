//! Durable, sealed, short-lived ceremony state and session-store counters that
//! make a server restart (or a second instance behind a load balancer)
//! invisible to people who are mid-flow.
//!
//! Nothing here holds a long-lived secret. Entries expire on their own and are
//! consumed exactly once by [`Store::take_ephemeral`].

use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::backend::q;
use crate::{Store, StoreError};

/// How many live rows each session store holds (for the admin status line).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SessionCounts {
    pub mailbox: i64,
    pub native: i64,
    pub admin: i64,
}

fn hash_key(key: &str) -> String {
    Sha256::digest(key.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

impl Store {
    /// Store `payload` under `(kind, key)` for `ttl_secs`, sealed with the server
    /// key. Replaces an existing entry and sweeps expired ones.
    pub async fn put_ephemeral(
        &self,
        kind: &str,
        key: &str,
        payload: &[u8],
        ttl_secs: i64,
    ) -> Result<(), StoreError> {
        let now = Utc::now().timestamp();
        let sealed = self.key.seal(payload)?;
        q("DELETE FROM ephemeral_state WHERE expires_at <= ?1")
            .bind(now)
            .execute(&self.backend)
            .await?;
        q("DELETE FROM ephemeral_state WHERE kind = ?1 AND key_hash = ?2")
            .bind(kind)
            .bind(hash_key(key))
            .execute(&self.backend)
            .await?;
        q("INSERT INTO ephemeral_state (kind, key_hash, sealed, expires_at) VALUES (?1, ?2, ?3, ?4)")
            .bind(kind)
            .bind(hash_key(key))
            .bind(sealed)
            .bind(now + ttl_secs.max(1))
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Consume the entry exactly once. `None` when it is unknown, already taken,
    /// expired, or sealed under a different server key.
    pub async fn take_ephemeral(
        &self,
        kind: &str,
        key: &str,
    ) -> Result<Option<Vec<u8>>, StoreError> {
        let row = q("DELETE FROM ephemeral_state WHERE kind = ?1 AND key_hash = ?2 AND expires_at > ?3 RETURNING sealed")
            .bind(kind)
            .bind(hash_key(key))
            .bind(Utc::now().timestamp())
            .fetch_optional(&self.backend)
            .await?;
        Ok(row.and_then(|r| self.key.open(&r.get_blob("sealed")).ok()))
    }

    /// Count the persisted sessions that are still within their deadlines.
    pub async fn session_counts(&self) -> Result<SessionCounts, StoreError> {
        let now = Utc::now().timestamp();
        let count = |row: crate::backend::Row| row.get_i64("n");
        let mailbox = count(
            q("SELECT COUNT(*) AS n FROM sessions WHERE linked_internal = 0")
                .fetch_one(&self.backend)
                .await?,
        );
        let native = count(
            q("SELECT COUNT(*) AS n FROM native_sessions")
                .fetch_one(&self.backend)
                .await?,
        );
        let admin = count(
            q("SELECT COUNT(*) AS n FROM admin_sessions WHERE expires_at > ?1 AND absolute_expires_at > ?1")
                .bind(now)
                .fetch_one(&self.backend)
                .await?,
        );
        Ok(SessionCounts {
            mailbox,
            native,
            admin,
        })
    }

    /// Open up to `limit` stored session credentials with the current server key.
    /// Returns `(checked, unopenable)`: a non-zero second number means the key in
    /// use is not the one the sessions were sealed with (missing or replaced key
    /// file), and those sessions can no longer sign in.
    pub async fn session_key_check(&self, limit: i64) -> Result<(i64, i64), StoreError> {
        let rows = q("SELECT sealed_creds FROM sessions WHERE linked_internal = 0 ORDER BY last_seen DESC LIMIT ?1")
            .bind(limit)
            .fetch_all(&self.backend)
            .await?;
        let bad = rows
            .iter()
            .filter(|row| self.key.open(&row.get_blob("sealed_creds")).is_err())
            .count();
        Ok((rows.len() as i64, bad as i64))
    }

    /// Delete every persisted session (mailbox, native, admin). Used at startup
    /// when `session.persistAcrossRestarts` is off. Returns the rows removed.
    pub async fn purge_all_sessions(&self) -> Result<u64, StoreError> {
        let mut total = 0;
        for sql in [
            "DELETE FROM native_sessions",
            "DELETE FROM sessions",
            "DELETE FROM admin_sessions",
        ] {
            total += q(sql).execute(&self.backend).await?;
        }
        Ok(total)
    }

    /// Like [`touch_session`](Self::touch_session) but writes at most once per
    /// `min_age_secs`. `last_seen` is request telemetry, not the idle clock
    /// (`activity_at`), so skipping a write never extends a session.
    pub async fn touch_session_throttled(
        &self,
        id: &str,
        min_age_secs: i64,
    ) -> Result<(), StoreError> {
        if min_age_secs <= 0 {
            return self.touch_session(id).await;
        }
        let now = Utc::now();
        let cutoff = (now - chrono::Duration::seconds(min_age_secs)).to_rfc3339();
        q("UPDATE sessions SET last_seen = ?2 WHERE id = ?1 AND last_seen < ?3")
            .bind(id)
            .bind(now.to_rfc3339())
            .bind(cutoff)
            .execute(&self.backend)
            .await?;
        Ok(())
    }
}
