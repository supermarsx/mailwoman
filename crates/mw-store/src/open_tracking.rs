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
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::backend::q;
use crate::{Store, StoreError};

/// Unbound tokens (minted, never tied to a sent message) live this long.
pub const UNBOUND_TTL_HOURS: i64 = 24;
/// Every token, bound or not, is deleted after this many days.
pub const RETENTION_DAYS: i64 = 365;
/// Event rows kept per token; later loads only move the totals.
pub const MAX_EVENTS_PER_TOKEN: i64 = 200;

/// Effective hot administrator retention; validated before any durable write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenTrackingRetention {
    pub retention_days: i64,
    pub unbound_ttl_hours: i64,
}

impl Default for OpenTrackingRetention {
    fn default() -> Self {
        Self {
            retention_days: RETENTION_DAYS,
            unbound_ttl_hours: UNBOUND_TTL_HOURS,
        }
    }
}

impl OpenTrackingRetention {
    pub fn valid(self) -> bool {
        (1..=400).contains(&self.retention_days) && (1..=168).contains(&self.unbound_ttl_hours)
    }
    fn cutoffs(self, now: DateTime<Utc>) -> Result<(String, String), StoreError> {
        if !self.valid() {
            return Err(StoreError::Corrupt(
                "invalid open tracking retention".into(),
            ));
        }
        let unbound = now.checked_sub_signed(Duration::hours(self.unbound_ttl_hours));
        let all = now.checked_sub_signed(Duration::days(self.retention_days));
        match (unbound, all) {
            (Some(unbound), Some(all)) => Ok((ts(unbound), ts(all))),
            _ => Err(StoreError::Corrupt("invalid open tracking time".into())),
        }
    }
}

/// Verified server-derived feature-policy identity, never supplied by a pixel
/// request or taken from a client's send payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OpenTrackingPolicySubject {
    pub account_id: String,
    pub personal_owner: Option<String>,
    pub domain: String,
    pub address: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SealedPolicySubject {
    token_hash: String,
    subject: OpenTrackingPolicySubject,
}

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
        self.create_open_token_with_policy(
            token_hash,
            account_id,
            subject,
            recipients_json,
            now,
            OpenTrackingRetention::default(),
            None,
        )
        .await
    }

    /// Mint with the current retention and an optional verified policy identity.
    /// Legacy tokens deliberately have no identity metadata.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_open_token_with_policy(
        &self,
        token_hash: &str,
        account_id: &str,
        subject: &str,
        recipients_json: &str,
        now: DateTime<Utc>,
        retention: OpenTrackingRetention,
        policy_subject: Option<&OpenTrackingPolicySubject>,
    ) -> Result<(), StoreError> {
        retention.cutoffs(now)?;
        let policy_sealed = match policy_subject {
            Some(policy) => {
                if policy.account_id != account_id || policy.account_id.is_empty() {
                    return Err(StoreError::Corrupt(
                        "open tracking policy account mismatch".into(),
                    ));
                }
                let json = serde_json::to_vec(&SealedPolicySubject {
                    token_hash: token_hash.into(),
                    subject: policy.clone(),
                })
                .map_err(|error| StoreError::Corrupt(error.to_string()))?;
                Some(self.key.seal(&json)?)
            }
            None => None,
        };
        let subject_sealed = self.key.seal(subject.as_bytes())?;
        let recipients_sealed = self.key.seal(recipients_json.as_bytes())?;
        self.prune_open_tracking_with_retention(now, retention)
            .await?;
        q("INSERT INTO open_tracking
                 (token_hash, account_id, email_id, subject_sealed, recipients_sealed, created_at, open_count, policy_subject_sealed)
             VALUES (?1, ?2, NULL, ?3, ?4, ?5, 0, ?6)")
        .bind(token_hash)
        .bind(account_id)
        .bind(subject_sealed)
        .bind(recipients_sealed)
        .bind(ts(now))
        .bind(policy_sealed)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Delete unbound tokens older than [`UNBOUND_TTL_HOURS`] and all tokens older
    /// than [`RETENTION_DAYS`] (relative to `now`), events first. Returns the
    /// number of tokens deleted.
    pub async fn prune_open_tracking(&self, now: DateTime<Utc>) -> Result<u64, StoreError> {
        self.prune_open_tracking_with_retention(now, OpenTrackingRetention::default())
            .await
    }

    pub async fn prune_open_tracking_with_retention(
        &self,
        now: DateTime<Utc>,
        retention: OpenTrackingRetention,
    ) -> Result<u64, StoreError> {
        let (unbound_cutoff, all_cutoff) = retention.cutoffs(now)?;
        let mut tx = self.backend.begin().await?;
        // Lock token rows before their events, as record_open does. A concurrent
        // recorder cannot insert an event between event deletion/token deletion.
        q("UPDATE open_tracking SET open_count=open_count WHERE (email_id IS NULL AND created_at <= ?1) OR created_at <= ?2")
            .bind(&unbound_cutoff).bind(&all_cutoff).execute_tx(&mut tx).await?;
        q("DELETE FROM open_events WHERE token_hash IN (
                 SELECT token_hash FROM open_tracking
                  WHERE (email_id IS NULL AND created_at <= ?1) OR created_at <= ?2)")
        .bind(&unbound_cutoff)
        .bind(&all_cutoff)
        .execute_tx(&mut tx)
        .await?;
        let n = q("DELETE FROM open_tracking
                    WHERE (email_id IS NULL AND created_at <= ?1) OR created_at <= ?2")
        .bind(&unbound_cutoff)
        .bind(&all_cutoff)
        .execute_tx(&mut tx)
        .await?;
        tx.commit().await?;
        Ok(n)
    }

    /// Decrypt only this token's bound policy subject. Legacy/missing rows have
    /// no subject; corrupted or swapped metadata fails closed.
    pub async fn open_tracking_policy_subject(
        &self,
        token_hash: &str,
    ) -> Result<Option<OpenTrackingPolicySubject>, StoreError> {
        let Some(row) =
            q("SELECT account_id,policy_subject_sealed FROM open_tracking WHERE token_hash=?1")
                .bind(token_hash)
                .fetch_optional(&self.backend)
                .await?
        else {
            return Ok(None);
        };
        let Some(sealed) = row.get_opt_blob("policy_subject_sealed") else {
            return Ok(None);
        };
        let plain = self.key.open(&sealed)?;
        let stored: SealedPolicySubject = serde_json::from_slice(&plain)
            .map_err(|error| StoreError::Corrupt(error.to_string()))?;
        if stored.token_hash != token_hash
            || stored.subject.account_id != row.get_string("account_id")
        {
            return Err(StoreError::Corrupt(
                "open tracking policy binding mismatch".into(),
            ));
        }
        Ok(Some(stored.subject))
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
        self.record_open_with_retention(token_hash, at, OpenTrackingRetention::default())
            .await
    }

    /// Expired pixels never revive a token. Targeted expiry cleanup and recording
    /// are atomic; unrelated tokens are left for configured global pruning.
    pub async fn record_open_with_retention(
        &self,
        token_hash: &str,
        at: DateTime<Utc>,
        retention: OpenTrackingRetention,
    ) -> Result<bool, StoreError> {
        let (unbound_cutoff, all_cutoff) = retention.cutoffs(at)?;
        let at = ts(at);
        let mut tx = self.backend.begin().await?;
        q("UPDATE open_tracking SET open_count=open_count WHERE token_hash=?1")
            .bind(token_hash)
            .execute_tx(&mut tx)
            .await?;
        q("DELETE FROM open_events WHERE token_hash=?1 AND EXISTS(SELECT 1 FROM open_tracking WHERE token_hash=?1 AND ((email_id IS NULL AND created_at<=?2) OR created_at<=?3))")
            .bind(token_hash).bind(&unbound_cutoff).bind(&all_cutoff).execute_tx(&mut tx).await?;
        q("DELETE FROM open_tracking WHERE token_hash=?1 AND ((email_id IS NULL AND created_at<=?2) OR created_at<=?3)")
            .bind(token_hash).bind(&unbound_cutoff).bind(&all_cutoff).execute_tx(&mut tx).await?;
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
    async fn configured_retention_extends_mint_pruning_and_shortened_expiry_removes_events() {
        let s = store().await;
        let now = Utc::now();
        let longer = OpenTrackingRetention {
            retention_days: 200,
            unbound_ttl_hours: 48,
        };
        let old = open_token_hash("custom-bound");
        s.create_open_token_with_policy(
            &old,
            "a",
            "s",
            "[]",
            now - Duration::days(110),
            longer,
            None,
        )
        .await
        .unwrap();
        s.bind_open_token("a", &old, "E1").await.unwrap();
        assert!(
            s.record_open_with_retention(&old, now, longer)
                .await
                .unwrap()
        );
        let unbound = open_token_hash("custom-unbound");
        s.create_open_token_with_policy(
            &unbound,
            "a",
            "s",
            "[]",
            now - Duration::hours(36),
            longer,
            None,
        )
        .await
        .unwrap();
        s.create_open_token_with_policy(
            &open_token_hash("fresh"),
            "a",
            "s",
            "[]",
            now,
            longer,
            None,
        )
        .await
        .unwrap();
        assert!(s.get_open_token("a", &old).await.unwrap().is_some());
        assert!(s.get_open_token("a", &unbound).await.unwrap().is_some());
        let shorter = OpenTrackingRetention {
            retention_days: 100,
            unbound_ttl_hours: 48,
        };
        assert!(
            !s.record_open_with_retention(&old, now, shorter)
                .await
                .unwrap()
        );
        assert!(s.get_open_token("a", &old).await.unwrap().is_none());
        assert!(s.open_event_times(&old).await.unwrap().is_empty());
        assert_eq!(s.open_summary("a", "E1").await.unwrap(), None);
        assert!(
            s.get_open_token("a", &unbound).await.unwrap().is_some(),
            "pixel expiry must not silently run a global default-policy prune"
        );
        assert_eq!(
            s.prune_open_tracking_with_retention(
                now,
                OpenTrackingRetention {
                    retention_days: 100,
                    unbound_ttl_hours: 24
                }
            )
            .await
            .unwrap(),
            1
        );
        assert!(s.get_open_token("a", &unbound).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn unbound_and_bound_tokens_expire_at_the_configured_boundary_without_a_new_mint() {
        let s = store().await;
        let now = Utc::now();
        let retention = OpenTrackingRetention {
            retention_days: 1,
            unbound_ttl_hours: 2,
        };
        let unbound = open_token_hash("boundary-unbound");
        s.create_open_token_with_policy(&unbound, "a", "s", "[]", now, retention, None)
            .await
            .unwrap();
        assert!(
            s.record_open_with_retention(
                &unbound,
                now + Duration::hours(2) - Duration::milliseconds(1),
                retention
            )
            .await
            .unwrap()
        );
        assert!(
            !s.record_open_with_retention(&unbound, now + Duration::hours(2), retention)
                .await
                .unwrap()
        );
        assert!(s.open_event_times(&unbound).await.unwrap().is_empty());
        let bound = open_token_hash("boundary-bound");
        s.create_open_token_with_policy(&bound, "a", "s", "[]", now, retention, None)
            .await
            .unwrap();
        s.bind_open_token("a", &bound, "E2").await.unwrap();
        assert!(
            s.record_open_with_retention(&bound, now + Duration::hours(3), retention)
                .await
                .unwrap()
        );
        assert!(
            !s.record_open_with_retention(&bound, now + Duration::days(1), retention)
                .await
                .unwrap()
        );
        assert!(s.get_open_token("a", &bound).await.unwrap().is_none());
        assert!(s.open_event_times(&bound).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn verified_policy_subject_is_sealed_per_token_and_legacy_metadata_stays_absent() {
        let s = store().await;
        let now = Utc::now();
        let first = open_token_hash("subject-first");
        let second = open_token_hash("subject-second");
        let legacy = open_token_hash("subject-legacy");
        let subject = OpenTrackingPolicySubject {
            account_id: "a".into(),
            personal_owner: Some("verified-personal-owner".into()),
            domain: "example.org".into(),
            address: "User+tag@example.org".into(),
        };
        s.create_open_token_with_policy(
            &first,
            "a",
            "s",
            "[]",
            now,
            OpenTrackingRetention::default(),
            Some(&subject),
        )
        .await
        .unwrap();
        let other = OpenTrackingPolicySubject {
            address: "Other@example.org".into(),
            personal_owner: None,
            ..subject.clone()
        };
        s.create_open_token_with_policy(
            &second,
            "a",
            "s",
            "[]",
            now,
            OpenTrackingRetention::default(),
            Some(&other),
        )
        .await
        .unwrap();
        s.create_open_token(&legacy, "a", "s", "[]", now)
            .await
            .unwrap();
        assert_eq!(
            s.open_tracking_policy_subject(&first).await.unwrap(),
            Some(subject.clone())
        );
        assert_eq!(
            s.open_tracking_policy_subject(&second).await.unwrap(),
            Some(other)
        );
        assert_eq!(s.open_tracking_policy_subject(&legacy).await.unwrap(), None);
        assert_eq!(
            s.open_tracking_policy_subject("unknown").await.unwrap(),
            None
        );
        let row = q("SELECT policy_subject_sealed FROM open_tracking WHERE token_hash=?1")
            .bind(&first)
            .fetch_one(&s.backend)
            .await
            .unwrap();
        let sealed = row.get_blob("policy_subject_sealed");
        assert!(
            !sealed
                .windows(subject.address.len())
                .any(|part| part == subject.address.as_bytes())
        );
        q("UPDATE open_tracking SET policy_subject_sealed=?1 WHERE token_hash=?2")
            .bind(sealed)
            .bind(&second)
            .execute(&s.backend)
            .await
            .unwrap();
        assert!(
            s.open_tracking_policy_subject(&second).await.is_err(),
            "a valid sealed identity copied to another token must fail closed"
        );
        assert_eq!(
            s.open_tracking_policy_subject(&first).await.unwrap(),
            Some(subject)
        );
    }

    #[tokio::test]
    async fn invalid_policy_or_mismatched_subject_changes_nothing_including_expired_rows() {
        let s = store().await;
        let now = Utc::now();
        let old = open_token_hash("untouched-old");
        s.create_open_token(&old, "a", "s", "[]", now - Duration::days(2))
            .await
            .unwrap();
        for invalid in [
            OpenTrackingRetention {
                retention_days: 0,
                unbound_ttl_hours: 1,
            },
            OpenTrackingRetention {
                retention_days: 401,
                unbound_ttl_hours: 1,
            },
            OpenTrackingRetention {
                retention_days: 1,
                unbound_ttl_hours: 0,
            },
            OpenTrackingRetention {
                retention_days: 1,
                unbound_ttl_hours: 169,
            },
        ] {
            assert!(
                s.create_open_token_with_policy("invalid", "a", "s", "[]", now, invalid, None)
                    .await
                    .is_err()
            );
            assert!(
                s.prune_open_tracking_with_retention(now, invalid)
                    .await
                    .is_err()
            );
            assert!(
                s.record_open_with_retention(&old, now, invalid)
                    .await
                    .is_err()
            );
            assert!(s.get_open_token("a", &old).await.unwrap().is_some());
            assert!(s.open_event_times(&old).await.unwrap().is_empty());
            assert!(s.get_open_token("a", "invalid").await.unwrap().is_none());
        }
        let mismatch = OpenTrackingPolicySubject {
            account_id: "other".into(),
            personal_owner: None,
            domain: "example.org".into(),
            address: "x@example.org".into(),
        };
        assert!(
            s.create_open_token_with_policy(
                "mismatch",
                "a",
                "s",
                "[]",
                now,
                OpenTrackingRetention::default(),
                Some(&mismatch)
            )
            .await
            .is_err()
        );
        assert!(s.get_open_token("a", &old).await.unwrap().is_some());
        assert!(s.get_open_token("a", "mismatch").await.unwrap().is_none());
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
