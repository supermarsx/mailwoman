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

/// Coalesced `last_seen` writes (t50hp-e22).
#[path = "session_touch_buffer.rs"]
pub mod touch_buffer;

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
    /// Last successful mail check, isolated from settings revisions. One bounded,
    /// sealed row per account; expires after a year without use.
    pub async fn mail_check_checkpoint(&self, account: &str) -> Result<Option<i64>, StoreError> {
        let row = q("SELECT sealed FROM ephemeral_state WHERE kind='mail-check' AND key_hash=?1 AND expires_at>?2")
            .bind(hash_key(account)).bind(Utc::now().timestamp()).fetch_optional(&self.backend).await?;
        Ok(row
            .and_then(|r| self.key.open(&r.get_blob("sealed")).ok())
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .and_then(|v| v.parse().ok()))
    }

    /// Compare-and-set: only one browser may acknowledge a given baseline.
    /// Never move the clock backwards, including delayed requests from old tabs.
    pub async fn advance_mail_check(
        &self,
        account: &str,
        expected: Option<i64>,
        checked: i64,
    ) -> Result<bool, StoreError> {
        const TTL: i64 = 366 * 86400;
        let now = Utc::now().timestamp();
        if checked > now
            || checked < now - 600
            || expected.is_some_and(|old| old < 0 || old >= checked)
        {
            return Ok(false);
        }
        let sealed = self.key.seal(checked.to_string().as_bytes())?;
        let hash = hash_key(account);
        let changed = if let Some(old) = expected {
            q("UPDATE ephemeral_state SET sealed=?3,expires_at=?4 WHERE kind='mail-check' AND key_hash=?1 AND expires_at=?2")
                .bind(hash).bind(old + TTL).bind(sealed).bind(checked + TTL).execute(&self.backend).await?
        } else {
            // Expired rows may be replaced, but never a live concurrent claim.
            q("INSERT INTO ephemeral_state(kind,key_hash,sealed,expires_at) VALUES('mail-check',?1,?2,?3) ON CONFLICT(kind,key_hash) DO UPDATE SET sealed=excluded.sealed,expires_at=excluded.expires_at WHERE ephemeral_state.expires_at<=?4")
                .bind(hash).bind(sealed).bind(checked + TTL).bind(now).execute(&self.backend).await?
        };
        Ok(changed == 1)
    }

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
    /// Three independent reads, sent as one read unit.
    pub async fn session_counts(&self) -> Result<SessionCounts, StoreError> {
        let now = Utc::now().timestamp();
        let rows = self
            .backend
            .fetch_all_unit(vec![
                q("SELECT COUNT(*) AS n FROM sessions WHERE linked_internal = 0"),
                q("SELECT COUNT(*) AS n FROM native_sessions"),
                q("SELECT COUNT(*) AS n FROM admin_sessions WHERE expires_at > ?1 AND absolute_expires_at > ?1")
                    .bind(now),
            ])
            .await?;
        let count = |i: usize| -> Result<i64, StoreError> {
            Ok(rows[i]
                .first()
                .ok_or_else(|| StoreError::Corrupt("session count returned no row".into()))?
                .get_i64("n"))
        };
        Ok(SessionCounts {
            mailbox: count(0)?,
            native: count(1)?,
            admin: count(2)?,
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

    /// Whether the original telemetry UPDATE may change anything. sessions.last_seen
    /// is TEXT on both backends, and the throttled UPDATE compares it in byte
    /// order on both: SQLite's default BINARY collation, and an explicit
    /// `COLLATE "C"` on PostgreSQL (the database collation may order text
    /// differently). Rust byte ordering therefore matches the exact
    /// `last_seen < cutoff` predicate on either backend. The `last_seen` passed
    /// in was read from the same row earlier in the same request; a concurrent
    /// writer only moves it forward, so skipping an UPDATE judged not due skips
    /// a statement that would match no row.
    /// This fresh-row hint NEVER authenticates, extends idle clocks, or survives
    /// a request. Missing metadata and an unbounded/disabled throttle fail open
    /// toward the original atomic UPDATE, not toward authentication.
    pub fn session_touch_may_be_due_at(
        &self,
        last_seen: Option<&str>,
        min_age_secs: i64,
        now: chrono::DateTime<Utc>,
    ) -> bool {
        if min_age_secs <= 0 {
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
        let at = Utc::now();
        let now = at.to_rfc3339();
        let cutoff = (at - chrono::Duration::seconds(min_age_secs)).to_rfc3339();
        // A buffered store records the time in memory; at its cap (or without
        // a buffer) this is the original UPDATE.
        if self.buffer_session_touch(id, &now, &cutoff) {
            return Ok(());
        }
        self.touch_session_before(id, &now, &cutoff).await?;
        Ok(())
    }

    /// The throttled telemetry UPDATE. Byte-order comparison on both backends,
    /// so [`Self::session_touch_may_be_due_at`] predicts it exactly.
    async fn touch_session_before(
        &self,
        id: &str,
        now: &str,
        cutoff: &str,
    ) -> Result<u64, StoreError> {
        let sql = match self.backend.dialect() {
            crate::backend::Dialect::Sqlite => {
                "UPDATE sessions SET last_seen = ?2 WHERE id = ?1 AND last_seen < ?3"
            }
            crate::backend::Dialect::Postgres => {
                "UPDATE sessions SET last_seen = ?2 WHERE id = ?1 AND last_seen < ?3 COLLATE \"C\""
            }
        };
        Ok(q(sql).bind(id).bind(now).bind(cutoff).execute(&self.backend).await?)
    }
}

#[cfg(test)]
mod session_touch_tests {
    use super::*;
    use crate::{Credentials, ServerKey};
    use chrono::{DateTime, Duration, TimeZone};
    #[tokio::test]
    async fn mail_checks_are_account_scoped_atomic_and_monotonic() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let now = Utc::now().timestamp();
        assert_eq!(store.mail_check_checkpoint("a").await.unwrap(), None);
        let (a, b) = tokio::join!(
            store.advance_mail_check("a", None, now - 120),
            store.advance_mail_check("a", None, now - 120)
        );
        assert_ne!(
            a.unwrap(),
            b.unwrap(),
            "only one tab may claim the first check"
        );
        assert_eq!(
            store.mail_check_checkpoint("a").await.unwrap(),
            Some(now - 120)
        );
        assert_eq!(store.mail_check_checkpoint("b").await.unwrap(), None);
        let (a, b) = tokio::join!(
            store.advance_mail_check("a", Some(now - 120), now),
            store.advance_mail_check("a", Some(now - 120), now)
        );
        assert_ne!(
            a.unwrap(),
            b.unwrap(),
            "only one tab may acknowledge a baseline"
        );
        assert!(
            !store
                .advance_mail_check("a", Some(now - 120), now - 60)
                .await
                .unwrap()
        );
        assert!(
            !store
                .advance_mail_check("a", Some(now), now - 1)
                .await
                .unwrap()
        );
        assert!(
            !store
                .advance_mail_check("a", Some(now), now + 3600)
                .await
                .unwrap()
        );
        assert_eq!(store.mail_check_checkpoint("a").await.unwrap(), Some(now));
        // A checkpoint never publishes a settings revision on each check.
        assert_eq!(
            q("SELECT COUNT(*) AS n FROM personal_setting_value")
                .fetch_one(&store.backend)
                .await
                .unwrap()
                .get_i64("n"),
            0
        );
        store.close().await;
    }

    #[tokio::test]
    async fn expired_mail_checks_restart_silently_without_exposing_their_payload() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let now = Utc::now().timestamp();
        assert!(
            store
                .advance_mail_check("account", None, now - 2)
                .await
                .unwrap()
        );
        let row = q("SELECT sealed,key_hash FROM ephemeral_state WHERE kind='mail-check'")
            .fetch_one(&store.backend)
            .await
            .unwrap();
        assert_ne!(row.get_blob("sealed"), (now - 2).to_string().as_bytes());
        assert_ne!(row.get_string("key_hash"), "account");
        q("UPDATE ephemeral_state SET expires_at=0 WHERE kind='mail-check'")
            .execute(&store.backend)
            .await
            .unwrap();
        assert_eq!(store.mail_check_checkpoint("account").await.unwrap(), None);
        assert!(
            store
                .advance_mail_check("account", None, now)
                .await
                .unwrap()
        );
        store.close().await;
    }
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
    /// The precheck must agree with the real UPDATE on every backend. Values
    /// include one that a case-insensitive database collation orders on the
    /// other side of the cutoff ('t' versus 'T').
    async fn precheck_matches_update(store: &Store, id: &str) {
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
            "2026-11-20t11:59:44+00:00".into(),
            "2026-11-20T11:59:44.999+00:00".into(),
            "".into(),
            "malformed timestamp".into(),
            "é".into(),
        ];
        for last_seen in candidates {
            q("UPDATE sessions SET last_seen=?2 WHERE id=?1")
                .bind(id)
                .bind(&last_seen)
                .execute(&store.backend)
                .await
                .unwrap();
            let (_, fresh) = store.get_primary_session_with_activity(id).await.unwrap();
            let due = store.session_touch_may_be_due_at(fresh.as_deref(), 15, now);
            let changed = store
                .touch_session_before(id, "9999-12-31T00:00:00+00:00", &cutoff.to_rfc3339())
                .await
                .unwrap();
            assert_eq!(due, changed == 1, "{last_seen:?}");
        }
    }
    #[tokio::test]
    async fn session_counts_are_one_read_unit() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let first = session(&store).await;
        session(&store).await;
        q("UPDATE sessions SET linked_internal=1 WHERE id=?1")
            .bind(&first)
            .execute(&store.backend)
            .await
            .unwrap();
        let (counts, queries) =
            crate::pool_telemetry::measure_queries(store.session_counts()).await;
        let counts = counts.unwrap();
        assert_eq!((counts.mailbox, counts.native, counts.admin), (1, 0, 0));
        assert_eq!(queries.read.calls, 3);
        assert_eq!(queries.acquire.calls, 1);
        store.close().await;
    }
    #[tokio::test]
    async fn precheck_predicts_the_sqlite_update() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let id = session(&store).await;
        precheck_matches_update(&store, &id).await;
        store.close().await;
    }
    #[tokio::test]
    async fn precheck_predicts_the_postgres_update_and_skips_the_write() {
        let Ok(url) = std::env::var("MW_TEST_PG") else {
            mw_test_gate::skip("session touch precheck PostgreSQL leg: MW_TEST_PG not configured");
            return;
        };
        let store = Store::open_postgres(&url, ServerKey::generate()).await.unwrap();
        let id = session(&store).await;
        precheck_matches_update(&store, &id).await;
        // A fresh row: no UPDATE statement and no connection acquisition.
        q("UPDATE sessions SET last_seen=?2 WHERE id=?1")
            .bind(&id)
            .bind(Utc::now().to_rfc3339())
            .execute(&store.backend)
            .await
            .unwrap();
        let (_, last_seen) = store.get_primary_session_with_activity(&id).await.unwrap();
        let (_, queries) = crate::pool_telemetry::measure_queries(async {
            if store.session_touch_may_be_due_at(last_seen.as_deref(), 15, Utc::now()) {
                store.touch_session_throttled(&id, 15).await.unwrap();
            }
        })
        .await;
        assert_eq!(queries.write.calls, 0);
        assert_eq!(queries.acquire.calls, 0);
        store.delete_session(&id).await.unwrap();
        store.close().await;
    }
}
