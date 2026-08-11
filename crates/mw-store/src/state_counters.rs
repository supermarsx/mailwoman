//! Batched `(account, type)` state-counter reads (26.20 t22-e0).
//!
//! `sessionState` folds twelve counters — three mail (`changes`), seven PIM
//! (`pim_changes`) and two crypto/security (`crypto_changes`). Reading them one
//! at a time through [`Store::current_state`] and friends costs twelve
//! statements, issued **sequentially**, on **every** JMAP request. On SQLite
//! that is a couple of milliseconds; on Postgres it is twelve round trips, and
//! it is paid before any method does any work.
//!
//! These three readers return a whole log's counters in one statement each.
//!
//! **No `IN (…)` list, deliberately.** Two reasons. Mechanically, [`q`] takes a
//! `&'static str`, so a placeholder list sized to the caller's slice cannot be
//! built. Behaviourally, `GROUP BY` only emits groups that have rows: a type the
//! account has never touched is *absent from the result*, not present as zero.
//! Enumerating the wanted types in SQL would not change that — it is the caller
//! that has to treat "absent" as zero, and the map these return makes that the
//! obvious reading rather than a subtlety. `MAX(state)` itself is never `NULL`
//! here, because a group exists only where at least one row does.

use std::collections::HashMap;

use crate::backend::q;
use crate::{Store, StoreError};

impl Store {
    /// Every mail `(type → current state)` counter for one account, in one
    /// statement. Types with no rows are **absent** from the map; read them as
    /// `0`, exactly as [`Store::current_state`] returns `0` for them.
    pub async fn current_states(
        &self,
        account_id: &str,
    ) -> Result<HashMap<String, u64>, StoreError> {
        self.counters(
            "SELECT type, MAX(state) AS state FROM changes
             WHERE account_id = ?1 GROUP BY type",
            account_id,
        )
        .await
    }

    /// Every PIM `(type → current state)` counter for one account, in one
    /// statement. Absent types read as `0` (see [`Store::current_states`]).
    pub async fn current_pim_states(
        &self,
        account_id: &str,
    ) -> Result<HashMap<String, u64>, StoreError> {
        self.counters(
            "SELECT type, MAX(state) AS state FROM pim_changes
             WHERE account_id = ?1 GROUP BY type",
            account_id,
        )
        .await
    }

    /// Every crypto/security `(type → current state)` counter for one account,
    /// in one statement. Absent types read as `0`.
    pub async fn current_crypto_states(
        &self,
        account_id: &str,
    ) -> Result<HashMap<String, u64>, StoreError> {
        self.counters(
            "SELECT type, MAX(state) AS state FROM crypto_changes
             WHERE account_id = ?1 GROUP BY type",
            account_id,
        )
        .await
    }

    async fn counters(
        &self,
        sql: &'static str,
        account_id: &str,
    ) -> Result<HashMap<String, u64>, StoreError> {
        let rows = q(sql).bind(account_id).fetch_all(&self.backend).await?;
        Ok(rows
            .iter()
            .map(|r| (r.get_string("type"), r.get_i64("state").max(0) as u64))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::{AccountKind, Credentials, NewAccount, ServerKey, Store};

    async fn store() -> Store {
        Store::open_in_memory(ServerKey::generate()).await.unwrap()
    }

    async fn account(s: &Store) -> String {
        s.create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "h",
                port: 993,
                tls: "implicit",
                username: "u",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "u".into(),
                password: "p".into(),
            },
        )
        .await
        .unwrap()
    }

    /// The batch reader must agree with the per-type reader for every type,
    /// including the types that were never written.
    #[tokio::test]
    async fn batched_counters_agree_with_per_type_reads() {
        let s = store().await;
        let acct = account(&s).await;

        s.record_change(&acct, "Email", "e1", "created")
            .await
            .unwrap();
        s.record_change(&acct, "Email", "e2", "created")
            .await
            .unwrap();
        s.record_change(&acct, "Mailbox", "m1", "created")
            .await
            .unwrap();
        s.record_pim_change(&acct, "Note", "n1", "created")
            .await
            .unwrap();
        s.record_crypto_change(&acct, "MailRule", "r1", "created")
            .await
            .unwrap();

        let mail = s.current_states(&acct).await.unwrap();
        for kind in ["Email", "Mailbox", "EmailSubmission", "Thread"] {
            assert_eq!(
                mail.get(kind).copied().unwrap_or(0),
                s.current_state(&acct, kind).await.unwrap(),
                "mail counter {kind}"
            );
        }

        let pim = s.current_pim_states(&acct).await.unwrap();
        for kind in ["Note", "Calendar", "CalendarEvent", "Task", "ContactCard"] {
            assert_eq!(
                pim.get(kind).copied().unwrap_or(0),
                s.current_pim_state(&acct, kind).await.unwrap(),
                "PIM counter {kind}"
            );
        }

        let crypto = s.current_crypto_states(&acct).await.unwrap();
        for kind in ["MailRule", "CryptoKey"] {
            assert_eq!(
                crypto.get(kind).copied().unwrap_or(0),
                s.current_crypto_state(&acct, kind).await.unwrap(),
                "crypto counter {kind}"
            );
        }

        // The populated ones are actually present, or the loops above would
        // pass by comparing 0 against 0 throughout.
        assert_eq!(mail.get("Email").copied(), Some(2));
        assert_eq!(mail.get("Mailbox").copied(), Some(1));
        assert_eq!(pim.get("Note").copied(), Some(1));
        assert_eq!(crypto.get("MailRule").copied(), Some(1));
        // …and the untouched ones are ABSENT rather than zero-valued.
        assert!(!mail.contains_key("EmailSubmission"));
        assert!(!crypto.contains_key("CryptoKey"));
    }

    /// An account with no rows in a log yields an empty map, not an error and
    /// not a row of zeroes.
    #[tokio::test]
    async fn batched_counters_are_empty_for_an_untouched_account() {
        let s = store().await;
        for map in [
            s.current_states("nobody").await.unwrap(),
            s.current_pim_states("nobody").await.unwrap(),
            s.current_crypto_states("nobody").await.unwrap(),
        ] {
            assert!(map.is_empty(), "untouched account has no counters: {map:?}");
        }
    }

    /// One account's counters never leak into another's.
    #[tokio::test]
    async fn batched_counters_are_account_scoped() {
        let s = store().await;
        let acct = account(&s).await;
        s.record_change(&acct, "Email", "e1", "created")
            .await
            .unwrap();

        assert_eq!(s.current_states(&acct).await.unwrap().len(), 1);
        assert!(s.current_states("other").await.unwrap().is_empty());
    }
}
