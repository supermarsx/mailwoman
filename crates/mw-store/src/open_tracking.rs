//! 0032 open-tracking repository (26.20 t29-e8): dual-backend `Store` methods over
//! `open_tracking` and `open_events` (both dialects).
//!
//! A sender mints a token for one outgoing message, binds the sent email id to it,
//! and later reads how often and when the message's tracking image was loaded. The
//! store only ever sees the token's SHA-256 ([`open_token_hash`]); the subject and
//! recipients are sealed under the server key. A recorded load is a timestamp and
//! nothing else — the caller has no address or user agent to pass in.
//!
//! Retention is applied on every mint: unbound tokens older than
//! [`UNBOUND_TTL_HOURS`] and every token older than [`RETENTION_DAYS`] are deleted,
//! with their events. Each token keeps at most [`MAX_EVENTS_PER_TOKEN`] event rows;
//! after that only `open_count` / `last_open_at` move.

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use sha2::{Digest, Sha256};

use crate::backend::q;
use crate::{Store, StoreError};

/// Unbound tokens (minted, never tied to a sent message) live this long.
pub const UNBOUND_TTL_HOURS: i64 = 24;
/// Every token, bound or not, is deleted after this many days.
pub const RETENTION_DAYS: i64 = 90;
/// Event rows kept per token; later loads only move the totals.
pub const MAX_EVENTS_PER_TOKEN: i64 = 200;

/// Hex SHA-256 of a wire token: the only form the store keeps.
pub fn open_token_hash(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    let mut s = String::with_capacity(digest.len() * 2);
    for b in digest {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// One minted token, subject and recipients unsealed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenTokenRow {
    pub token_hash: String,
    pub account_id: String,
    pub email_id: Option<String>,
    pub subject: String,
    pub recipients_json: String,
    pub created_at: String,
    pub open_count: i64,
    pub first_open_at: Option<String>,
    pub last_open_at: Option<String>,
}

/// What `GET /api/track/open?emailId=` reports for one sent message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenSummary {
    pub count: i64,
    pub first: Option<String>,
    pub last: Option<String>,
}

/// Result of binding a sent email id to a token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenBind {
    Bound,
    /// No such token for this account (another account's token looks the same).
    NotFound,
    /// The token is already bound to a different email id.
    AlreadyBound,
}

fn ts(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Millis, true)
}

impl Store {
    /// Store a freshly minted token for `account_id`, created at `now`, after
    /// deleting what retention says has expired relative to `now`.
    pub async fn create_open_token(
        &self,
        token_hash: &str,
        account_id: &str,
        subject: &str,
        recipients_json: &str,
        now: DateTime<Utc>,
    ) -> Result<(), StoreError> {
        self.prune_open_tracking(now).await?;
        let subject_sealed = self.key.seal(subject.as_bytes())?;
        let recipients_sealed = self.key.seal(recipients_json.as_bytes())?;
        q("INSERT INTO open_tracking
                 (token_hash, account_id, email_id, subject_sealed, recipients_sealed, created_at, open_count)
             VALUES (?1, ?2, NULL, ?3, ?4, ?5, 0)")
        .bind(token_hash)
        .bind(account_id)
        .bind(subject_sealed)
        .bind(recipients_sealed)
        .bind(ts(now))
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Delete unbound tokens older than [`UNBOUND_TTL_HOURS`] and all tokens older
    /// than [`RETENTION_DAYS`] (relative to `now`), events first. Returns the
    /// number of tokens deleted.
    pub async fn prune_open_tracking(&self, now: DateTime<Utc>) -> Result<u64, StoreError> {
        let unbound_cutoff = ts(now - Duration::hours(UNBOUND_TTL_HOURS));
        let all_cutoff = ts(now - Duration::days(RETENTION_DAYS));
        let mut tx = self.backend.begin().await?;
        q("DELETE FROM open_events WHERE token_hash IN (
                 SELECT token_hash FROM open_tracking
                  WHERE (email_id IS NULL AND created_at < ?1) OR created_at < ?2)")
        .bind(&unbound_cutoff)
        .bind(&all_cutoff)
        .execute_tx(&mut tx)
        .await?;
        let n = q("DELETE FROM open_tracking
                    WHERE (email_id IS NULL AND created_at < ?1) OR created_at < ?2")
        .bind(&unbound_cutoff)
        .bind(&all_cutoff)
        .execute_tx(&mut tx)
        .await?;
        tx.commit().await?;
        Ok(n)
    }

    /// Tie a sent message to the caller's own token. Rebinding to the same id is
    /// idempotent; a different id is refused.
    pub async fn bind_open_token(
        &self,
        account_id: &str,
        token_hash: &str,
        email_id: &str,
    ) -> Result<OpenBind, StoreError> {
        let n = q("UPDATE open_tracking SET email_id = ?3
                    WHERE token_hash = ?1 AND account_id = ?2
                      AND (email_id IS NULL OR email_id = ?3)")
        .bind(token_hash)
        .bind(account_id)
        .bind(email_id)
        .execute(&self.backend)
        .await?;
        if n > 0 {
            return Ok(OpenBind::Bound);
        }
        let exists =
            q("SELECT COUNT(*) FROM open_tracking WHERE token_hash = ?1 AND account_id = ?2")
                .bind(token_hash)
                .bind(account_id)
                .fetch_scalar_i64(&self.backend)
                .await?;
        Ok(if exists > 0 {
            OpenBind::AlreadyBound
        } else {
            OpenBind::NotFound
        })
    }

    /// Record one load of the tracking image at `at`. Returns `false` (and writes
    /// nothing) when no token has this hash. The event row is written only while
    /// the token has fewer than [`MAX_EVENTS_PER_TOKEN`]; the totals always move.
    pub async fn record_open(
        &self,
        token_hash: &str,
        at: DateTime<Utc>,
    ) -> Result<bool, StoreError> {
        let at = ts(at);
        let mut tx = self.backend.begin().await?;
        let n = q("UPDATE open_tracking
                      SET open_count = open_count + 1,
                          first_open_at = COALESCE(first_open_at, ?2),
                          last_open_at = ?2
                    WHERE token_hash = ?1")
        .bind(token_hash)
        .bind(&at)
        .execute_tx(&mut tx)
        .await?;
        if n == 0 {
            tx.commit().await?;
            return Ok(false);
        }
        q("INSERT INTO open_events (token_hash, at)
             SELECT ?1, ?2
              WHERE (SELECT COUNT(*) FROM open_events WHERE token_hash = ?1) < ?3")
        .bind(token_hash)
        .bind(&at)
        .bind(MAX_EVENTS_PER_TOKEN)
        .execute_tx(&mut tx)
        .await?;
        tx.commit().await?;
        Ok(true)
    }

    /// Totals for one sent message of `account_id`, over every token bound to it.
    /// `None` when the account has no token bound to that email id.
    pub async fn open_summary(
        &self,
        account_id: &str,
        email_id: &str,
    ) -> Result<Option<OpenSummary>, StoreError> {
        let r = q("SELECT COUNT(*) AS n,
                          CAST(COALESCE(SUM(open_count), 0) AS BIGINT) AS c,
                          MIN(first_open_at) AS f,
                          MAX(last_open_at) AS l
                     FROM open_tracking WHERE account_id = ?1 AND email_id = ?2")
        .bind(account_id)
        .bind(email_id)
        .fetch_one(&self.backend)
        .await?;
        if r.get_i64("n") == 0 {
            return Ok(None);
        }
        Ok(Some(OpenSummary {
            count: r.get_i64("c"),
            first: r.get_opt_string("f"),
            last: r.get_opt_string("l"),
        }))
    }

    /// One of `account_id`'s tokens, subject and recipients unsealed.
    pub async fn get_open_token(
        &self,
        account_id: &str,
        token_hash: &str,
    ) -> Result<Option<OpenTokenRow>, StoreError> {
        let Some(r) = q(
            "SELECT token_hash, account_id, email_id, subject_sealed, recipients_sealed,
                                created_at, open_count, first_open_at, last_open_at
                           FROM open_tracking WHERE token_hash = ?1 AND account_id = ?2",
        )
        .bind(token_hash)
        .bind(account_id)
        .fetch_optional(&self.backend)
        .await?
        else {
            return Ok(None);
        };
        let unseal = |col: &str| -> Result<String, StoreError> {
            let plain = self.key.open(&r.get_blob(col))?;
            String::from_utf8(plain).map_err(|e| StoreError::Corrupt(e.to_string()))
        };
        Ok(Some(OpenTokenRow {
            token_hash: r.get_string("token_hash"),
            account_id: r.get_string("account_id"),
            email_id: r.get_opt_string("email_id"),
            subject: unseal("subject_sealed")?,
            recipients_json: unseal("recipients_sealed")?,
            created_at: r.get_string("created_at"),
            open_count: r.get_i64("open_count"),
            first_open_at: r.get_opt_string("first_open_at"),
            last_open_at: r.get_opt_string("last_open_at"),
        }))
    }

    /// The recorded load times of one token, oldest first (at most
    /// [`MAX_EVENTS_PER_TOKEN`]).
    pub async fn open_event_times(&self, token_hash: &str) -> Result<Vec<String>, StoreError> {
        Ok(
            q("SELECT at FROM open_events WHERE token_hash = ?1 ORDER BY at")
                .bind(token_hash)
                .fetch_all_scalar_string(&self.backend)
                .await?,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServerKey;

    async fn store() -> Store {
        Store::open_in_memory(ServerKey::generate()).await.unwrap()
    }

    #[tokio::test]
    async fn mint_bind_record_summarise() {
        let s = store().await;
        let h = open_token_hash("tok-1");
        let now = Utc::now();
        s.create_open_token(&h, "acct-a", "Hello", r#"["b@example.org"]"#, now)
            .await
            .unwrap();
        assert_eq!(s.open_summary("acct-a", "E1").await.unwrap(), None);
        assert_eq!(
            s.bind_open_token("acct-a", &h, "E1").await.unwrap(),
            OpenBind::Bound
        );
        let zero = s.open_summary("acct-a", "E1").await.unwrap().unwrap();
        assert_eq!((zero.count, zero.first, zero.last), (0, None, None));

        assert!(s.record_open(&h, now).await.unwrap());
        let one = s.open_summary("acct-a", "E1").await.unwrap().unwrap();
        assert_eq!(one.count, 1);
        assert_eq!(one.first, Some(ts(now)));
        assert_eq!(one.first, one.last);

        // The token row reads back unsealed, and the subject is not stored in clear.
        let row = s.get_open_token("acct-a", &h).await.unwrap().unwrap();
        assert_eq!(row.subject, "Hello");
        assert_eq!(row.recipients_json, r#"["b@example.org"]"#);
        assert_eq!(row.email_id.as_deref(), Some("E1"));
    }

    #[tokio::test]
    async fn bind_is_scoped_and_single() {
        let s = store().await;
        let h = open_token_hash("tok-2");
        s.create_open_token(&h, "acct-a", "s", "[]", Utc::now())
            .await
            .unwrap();
        assert_eq!(
            s.bind_open_token("acct-b", &h, "E1").await.unwrap(),
            OpenBind::NotFound
        );
        assert_eq!(
            s.bind_open_token("acct-a", &h, "E1").await.unwrap(),
            OpenBind::Bound
        );
        assert_eq!(
            s.bind_open_token("acct-a", &h, "E1").await.unwrap(),
            OpenBind::Bound
        );
        assert_eq!(
            s.bind_open_token("acct-a", &h, "E2").await.unwrap(),
            OpenBind::AlreadyBound
        );
        assert_eq!(s.open_summary("acct-b", "E1").await.unwrap(), None);
        assert!(s.get_open_token("acct-b", &h).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn unknown_token_records_nothing() {
        let s = store().await;
        assert!(
            !s.record_open(&open_token_hash("nope"), Utc::now())
                .await
                .unwrap()
        );
        assert!(
            s.open_event_times(&open_token_hash("nope"))
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn events_stop_at_the_cap_and_the_count_keeps_moving() {
        let s = store().await;
        let h = open_token_hash("tok-3");
        let t0 = Utc::now();
        s.create_open_token(&h, "acct-a", "s", "[]", t0)
            .await
            .unwrap();
        s.bind_open_token("acct-a", &h, "E1").await.unwrap();
        let total = MAX_EVENTS_PER_TOKEN + 5;
        for i in 0..total {
            s.record_open(&h, t0 + Duration::seconds(i)).await.unwrap();
        }
        assert_eq!(
            s.open_event_times(&h).await.unwrap().len() as i64,
            MAX_EVENTS_PER_TOKEN
        );
        let sum = s.open_summary("acct-a", "E1").await.unwrap().unwrap();
        assert_eq!(sum.count, total);
        assert_eq!(sum.last, Some(ts(t0 + Duration::seconds(total - 1))));
    }

    #[tokio::test]
    async fn mint_prunes_expired_rows() {
        let s = store().await;
        let now = Utc::now();
        let old_unbound = open_token_hash("old-unbound");
        let young_unbound = open_token_hash("young-unbound");
        let old_bound = open_token_hash("old-bound");
        let ancient_bound = open_token_hash("ancient-bound");
        s.create_open_token(&old_unbound, "a", "s", "[]", now - Duration::hours(25))
            .await
            .unwrap();
        s.create_open_token(&young_unbound, "a", "s", "[]", now - Duration::hours(23))
            .await
            .unwrap();
        s.create_open_token(&old_bound, "a", "s", "[]", now - Duration::days(89))
            .await
            .unwrap();
        s.create_open_token(&ancient_bound, "a", "s", "[]", now - Duration::days(91))
            .await
            .unwrap();
        s.bind_open_token("a", &old_bound, "E1").await.unwrap();
        s.bind_open_token("a", &ancient_bound, "E2").await.unwrap();
        s.record_open(&ancient_bound, now - Duration::days(90))
            .await
            .unwrap();
        // Precondition: the ancient token has an event to lose.
        assert_eq!(s.open_event_times(&ancient_bound).await.unwrap().len(), 1);

        s.create_open_token(&open_token_hash("new"), "a", "s", "[]", now)
            .await
            .unwrap();

        assert!(s.get_open_token("a", &old_unbound).await.unwrap().is_none());
        assert!(
            s.get_open_token("a", &young_unbound)
                .await
                .unwrap()
                .is_some()
        );
        assert!(s.get_open_token("a", &old_bound).await.unwrap().is_some());
        assert!(
            s.get_open_token("a", &ancient_bound)
                .await
                .unwrap()
                .is_none()
        );
        assert!(s.open_event_times(&ancient_bound).await.unwrap().is_empty());
    }

    #[test]
    fn hash_is_hex_sha256() {
        assert_eq!(
            open_token_hash("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
