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

    /// Whether the original telemetry UPDATE may change anything. Only SQLite
    /// uses this proof: sessions.last_seen is TEXT with default BINARY collation,
    /// so Rust byte ordering matches its exact `last_seen < cutoff` predicate.
    /// PostgreSQL keeps the original path because database collation can differ.
    /// This fresh-row hint NEVER authenticates, extends idle clocks, or survives
    /// a request. Missing metadata and an unbounded/disabled throttle fail open
    /// toward the original atomic UPDATE, not toward authentication.
    pub fn session_touch_may_be_due_at(
        &self,
        last_seen: Option<&str>,
        min_age_secs: i64,
        now: chrono::DateTime<Utc>,
    ) -> bool {
        if min_age_secs <= 0 || !matches!(&self.backend, crate::backend::Backend::Sqlite(_)) {
            return true;
        }
        let Some(last_seen) = last_seen else {
            return true;
        };
        let Some(age) = chrono::Duration::try_seconds(min_age_secs) else {
            return true;
        };
        let Some(cutoff) = now.checked_sub_signed(age) else {
            return true;
        };
        last_seen < cutoff.to_rfc3339().as_str()
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

#[cfg(test)]
mod session_touch_tests {
    use super::*;
    use crate::{Credentials, ServerKey};
    use chrono::{DateTime, Duration, TimeZone};
    async fn session(store: &Store) -> String {
        store
            .create_session(
                "account-one",
                "person@example.test",
                "https://mail.example.test/session",
                "https://mail.example.test/api",
                &Credentials {
                    username: "person@example.test".into(),
                    password: "secret".into(),
                },
            )
            .await
            .unwrap()
    }
    #[tokio::test]
    async fn fresh_activity_hint_matches_the_original_sqlite_binary_predicate() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let id = session(&store).await;
        let now = Utc.with_ymd_and_hms(2026, 11, 20, 12, 0, 0).unwrap();
        let cutoff = now - Duration::seconds(15);
        let candidates = [
            (cutoff - Duration::nanoseconds(1)).to_rfc3339(),
            cutoff.to_rfc3339(),
            (cutoff + Duration::nanoseconds(1)).to_rfc3339(),
            now.to_rfc3339(),
            (now - Duration::days(5)).to_rfc3339(),
            (now + Duration::days(5)).to_rfc3339(),
            "2026-11-20T12:00:00Z".into(),
            "2026-11-20T13:00:00+01:00".into(),
            "".into(),
            "malformed timestamp".into(),
            "é".into(),
        ];
        for last_seen in candidates {
            q("UPDATE sessions SET last_seen=?2 WHERE id=?1")
                .bind(&id)
                .bind(&last_seen)
                .execute(&store.backend)
                .await
                .unwrap();
            let (_, fresh) = store.get_primary_session_with_activity(&id).await.unwrap();
            let original=q("SELECT CASE WHEN last_seen < ?2 THEN 1 ELSE 0 END AS due FROM sessions WHERE id=?1").bind(&id).bind(cutoff.to_rfc3339()).fetch_one(&store.backend).await.unwrap().get_i64("due")!=0;
            assert_eq!(
                store.session_touch_may_be_due_at(fresh.as_deref(), 15, now),
                original,
                "{last_seen:?}"
            );
        }
        assert!(!store.session_touch_may_be_due_at(Some(&cutoff.to_rfc3339()), 15, now));
        assert!(!store.session_touch_may_be_due_at(
            Some(&now.to_rfc3339()),
            15,
            now - Duration::days(1)
        ));
        assert!(store.session_touch_may_be_due_at(
            Some(&now.to_rfc3339()),
            15,
            now + Duration::days(1)
        ));
        for age in [0, -1] {
            assert!(store.session_touch_may_be_due_at(Some(&now.to_rfc3339()), age, now));
        }
        assert!(store.session_touch_may_be_due_at(None, 15, now));
        assert!(store.session_touch_may_be_due_at(Some(&now.to_rfc3339()), i64::MAX, now));
        store.close().await;
    }
    #[tokio::test]
    async fn fresh_primary_read_uses_one_query_and_avoids_noop_write_admission() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let id = session(&store).await;
        let (loaded, queries) =
            crate::pool_telemetry::measure_queries(store.get_primary_session_with_activity(&id))
                .await;
        let (loaded, last_seen) = loaded.unwrap();
        assert_eq!(loaded.id, id);
        assert_eq!(queries.read.calls, 1);
        assert_eq!(queries.write.calls, 0);
        let before = store.session_activity_clock(&id).await.unwrap();
        let (_, queries) = crate::pool_telemetry::measure_queries(async {
            if store.session_touch_may_be_due_at(last_seen.as_deref(), 15, Utc::now()) {
                store.touch_session_throttled(&id, 15).await.unwrap();
            }
        })
        .await;
        assert_eq!(queries.write.calls, 0);
        assert_eq!(queries.acquire.calls, 0);
        assert_eq!(store.session_activity_clock(&id).await.unwrap(), before);
        q("UPDATE sessions SET last_seen=?2 WHERE id=?1")
            .bind(&id)
            .bind((Utc::now() - Duration::seconds(60)).to_rfc3339())
            .execute(&store.backend)
            .await
            .unwrap();
        let (_, old) = store.get_primary_session_with_activity(&id).await.unwrap();
        assert!(store.session_touch_may_be_due_at(old.as_deref(), 15, Utc::now()));
        store.touch_session_throttled(&id, 15).await.unwrap();
        assert_eq!(
            store.session_activity_clock(&id).await.unwrap(),
            before,
            "telemetry never extends the authentication idle clock"
        );
        store.close().await;
    }
    #[tokio::test]
    async fn activity_metadata_never_bypasses_revocation_internal_fences_or_credential_checks() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let id = session(&store).await;
        let expected = store.get_primary_session(&id).await.unwrap();
        let (actual, _) = store.get_primary_session_with_activity(&id).await.unwrap();
        assert_eq!(actual.id, expected.id);
        assert_eq!(actual.account_id, expected.account_id);
        assert_eq!(actual.username, expected.username);
        assert_eq!(actual.jmap_url, expected.jmap_url);
        assert_eq!(actual.api_url, expected.api_url);
        assert_eq!(actual.credentials, expected.credentials);
        q("UPDATE sessions SET last_seen=?2 WHERE id=?1")
            .bind(&id)
            .bind(vec![0xffu8])
            .execute(&store.backend)
            .await
            .unwrap();
        assert!(
            store.get_primary_session_with_activity(&id).await.is_ok(),
            "unreadable telemetry cannot break authentication"
        );
        let mut wrong_key = store.clone();
        wrong_key.key = ServerKey::generate();
        assert!(
            wrong_key
                .get_primary_session_with_activity(&id)
                .await
                .is_err()
        );
        q("UPDATE sessions SET linked_internal=1 WHERE id=?1")
            .bind(&id)
            .execute(&store.backend)
            .await
            .unwrap();
        assert!(matches!(
            store.get_primary_session_with_activity(&id).await,
            Err(StoreError::NotFound)
        ));
        assert!(matches!(
            store.get_primary_session(&id).await,
            Err(StoreError::NotFound)
        ));
        q("UPDATE sessions SET linked_internal=0 WHERE id=?1")
            .bind(&id)
            .execute(&store.backend)
            .await
            .unwrap();
        store.delete_session(&id).await.unwrap();
        assert!(matches!(
            store.get_primary_session_with_activity(&id).await,
            Err(StoreError::NotFound)
        ));
        store.close().await;
    }
    #[tokio::test]
    async fn concurrent_due_snapshots_keep_the_original_atomic_update_and_clock() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let id = session(&store).await;
        let clock = store.session_activity_clock(&id).await.unwrap();
        q("UPDATE sessions SET last_seen=?2 WHERE id=?1")
            .bind(&id)
            .bind((Utc::now() - Duration::seconds(60)).to_rfc3339())
            .execute(&store.backend)
            .await
            .unwrap();
        q("CREATE TABLE session_touch_probe(total INTEGER NOT NULL)")
            .execute(&store.backend)
            .await
            .unwrap();
        q("INSERT INTO session_touch_probe(total) VALUES(0)")
            .execute(&store.backend)
            .await
            .unwrap();
        q("CREATE TRIGGER session_touch_probe_trigger AFTER UPDATE OF last_seen ON sessions WHEN OLD.last_seen<>NEW.last_seen BEGIN UPDATE session_touch_probe SET total=total+1; END").execute(&store.backend).await.unwrap();
        let barrier = std::sync::Arc::new(tokio::sync::Barrier::new(16));
        let mut tasks = Vec::new();
        for _ in 0..16 {
            let store = store.clone();
            let id = id.clone();
            let barrier = barrier.clone();
            tasks.push(tokio::spawn(async move {
                let (_, last_seen) = store.get_primary_session_with_activity(&id).await.unwrap();
                barrier.wait().await;
                assert!(store.session_touch_may_be_due_at(last_seen.as_deref(), 15, Utc::now()));
                store.touch_session_throttled(&id, 15).await.unwrap();
            }));
        }
        for task in tasks {
            task.await.unwrap();
        }
        assert_eq!(
            q("SELECT total FROM session_touch_probe")
                .fetch_one(&store.backend)
                .await
                .unwrap()
                .get_i64("total"),
            1
        );
        assert_eq!(store.session_activity_clock(&id).await.unwrap(), clock);
        let (_, last_seen) = store.get_primary_session_with_activity(&id).await.unwrap();
        let last = DateTime::parse_from_rfc3339(last_seen.as_deref().unwrap())
            .unwrap()
            .with_timezone(&Utc);
        assert!((Utc::now() - last).num_seconds() < 15);
        store.close().await;
    }
}
