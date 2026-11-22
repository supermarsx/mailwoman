//! Batched `(account, type)` state-counter reads (26.20 t22-e0).
//!
//! `sessionState` folds twelve counters — three mail (`changes`), seven PIM
//! (`pim_changes`) and two crypto/security (`crypto_changes`). Reading them one
//! at a time through [`Store::current_state`] and friends costs twelve
//! statements, issued **sequentially**, on **every** JMAP request. On SQLite
//! that is a couple of milliseconds; on Postgres it is twelve round trips, and
//! it is paid before any method does any work.
//!
//! The general readers return a whole log's counters in one statement each.
//! The hot mail path has an indexed reader for its three fixed datatypes.
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

const CORE_MAIL_STATES_SQL: &str = "SELECT 'Email' AS type, MAX(state) AS state FROM (
                SELECT MAX(state) AS state FROM changes WHERE account_id=?1 AND type='Email'
                UNION ALL SELECT head_state AS state FROM change_log_checkpoint WHERE account_id=?1 AND type='Email'
             ) email_head HAVING MAX(state) IS NOT NULL
             UNION ALL
             SELECT 'Mailbox' AS type, MAX(state) AS state FROM (
                SELECT MAX(state) AS state FROM changes WHERE account_id=?1 AND type='Mailbox'
                UNION ALL SELECT head_state AS state FROM change_log_checkpoint WHERE account_id=?1 AND type='Mailbox'
             ) mailbox_head HAVING MAX(state) IS NOT NULL
             UNION ALL
             SELECT 'EmailSubmission' AS type, MAX(state) AS state FROM (
                SELECT MAX(state) AS state FROM changes WHERE account_id=?1 AND type='EmailSubmission'
                UNION ALL SELECT head_state AS state FROM change_log_checkpoint WHERE account_id=?1 AND type='EmailSubmission'
             ) submission_head HAVING MAX(state) IS NOT NULL";

/// Request-local counters, never an authorization or cross-request cache.
pub struct SessionStateCounters {
    pub mail: HashMap<String, u64>,
    pub pim: HashMap<String, u64>,
    pub crypto: HashMap<String, u64>,
}

fn session_states_sql() -> &'static str {
    static SQL: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    SQL.get_or_init(|| format!(
        "SELECT 'mail' AS family,type,state FROM ({CORE_MAIL_STATES_SQL}) mail_counters
         UNION ALL SELECT 'pim',type,MAX(state) FROM pim_changes
         WHERE account_id=?1 AND type IN ('Calendar','CalendarEvent','Task','Note','AddressBook','ContactCard','ContactGroup') GROUP BY type
         UNION ALL SELECT 'crypto',type,MAX(state) FROM crypto_changes
         WHERE account_id=?1 AND type IN ('CryptoKey','MailRule') GROUP BY type"
    ))
}

impl Store {
    /// Fresh closed-family JMAP state in one statement, at the caller's
    /// original boundary. Retained mail checkpoints use the
    /// identical indexed probes as the scalar family reader. The result has at
    /// most twelve rows. It must not replace an earlier method's state token.
    ///
    /// A failed joint query retries the original three readers independently:
    /// one broken log must not erase successful other-family counters.
    pub async fn session_state_counters(&self, account_id: &str) -> SessionStateCounters {
        if let Ok(rows) = q(session_states_sql()).bind(account_id).fetch_all(&self.backend).await {
            let mut result = SessionStateCounters {mail:HashMap::new(),pim:HashMap::new(),crypto:HashMap::new()};
            for row in rows {
                let map = match row.get_string("family").as_str() {
                    "mail" => &mut result.mail,
                    "pim" => &mut result.pim,
                    "crypto" => &mut result.crypto,
                    _ => unreachable!("closed compiled counter families"),
                };
                map.insert(row.get_string("type"), row.get_i64("state").max(0) as u64);
            }
            return result;
        }
        let (mail,pim,crypto) = tokio::join!(self.current_core_mail_states(account_id),
            self.current_pim_states(account_id),self.current_crypto_states(account_id));
        SessionStateCounters {mail:mail.unwrap_or_default(),pim:pim.unwrap_or_default(),crypto:crypto.unwrap_or_default()}
    }
    /// The three mail counters consumed by JMAP session state and mail push.
    /// One statement gives one snapshot; indexed MAX probes avoid scanning an
    /// account's retained history. Untouched counters remain absent. Custom
    /// datatype callers must continue using [`Store::current_states`].
    pub async fn current_core_mail_states(
        &self,
        account_id: &str,
    ) -> Result<HashMap<String, u64>, StoreError> {
        self.counters(CORE_MAIL_STATES_SQL, account_id).await
    }

    /// Every mail `(type → current state)` counter for one account, in one
    /// statement. Types with no rows are **absent** from the map; read them as
    /// `0`, exactly as [`Store::current_state`] returns `0` for them.
    pub async fn current_states(
        &self,
        account_id: &str,
    ) -> Result<HashMap<String, u64>, StoreError> {
        self.counters(
            "SELECT type, MAX(state) AS state FROM (
                SELECT type,state FROM changes WHERE account_id=?1
                UNION ALL SELECT type,head_state AS state FROM change_log_checkpoint WHERE account_id=?1
             ) counters GROUP BY type",
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

    #[tokio::test]
    async fn session_frame_is_one_fresh_read_with_closed_families_and_checkpoints() {
        let s=store().await;
        let owner=account(&s).await;
        let (empty,counts)=crate::pool_telemetry::measure_queries(s.session_state_counters(&owner)).await;
        assert!(empty.mail.is_empty() && empty.pim.is_empty() && empty.crypto.is_empty());
        assert_eq!(counts.read.calls,1);
        crate::q("INSERT INTO change_log_checkpoint(account_id,type,head_state,pruned_through) VALUES(?1,'Email',17,17)")
            .bind(&owner).execute(&s.backend).await.unwrap();
        s.record_change(&owner,"Mailbox","m","created").await.unwrap();
        s.record_pim_change(&owner,"Note","n","created").await.unwrap();
        s.record_crypto_change(&owner,"MailRule","r","created").await.unwrap();
        s.record_pim_change(&owner,"UnfoldedCustomType","custom","created").await.unwrap();
        let (frame,counts)=crate::pool_telemetry::measure_queries(s.session_state_counters(&owner)).await;
        assert_eq!(counts.read.calls,1);
        assert_eq!(counts.acquire.calls,1);
        assert_eq!(frame.mail,s.current_core_mail_states(&owner).await.unwrap());
        assert_eq!(frame.mail["Email"],17);
        assert_eq!(frame.pim["Note"],1);
        assert!(!frame.pim.contains_key("UnfoldedCustomType"));
        assert_eq!(frame.crypto,s.current_crypto_states(&owner).await.unwrap());
        let foreign=s.session_state_counters("foreign").await;
        assert!(foreign.mail.is_empty() && foreign.pim.is_empty() && foreign.crypto.is_empty());
        s.record_pim_change(&owner,"Note","n2","updated").await.unwrap();
        assert_eq!(s.session_state_counters(&owner).await.pim["Note"],2,
            "a prior frame must never become a cached response state");
    }

    #[tokio::test]
    async fn session_frame_failed_family_retains_original_independent_defaults() {
        let s=store().await;
        let owner=account(&s).await;
        s.record_change(&owner,"Email","e","created").await.unwrap();
        s.record_pim_change(&owner,"Note","n","created").await.unwrap();
        s.record_crypto_change(&owner,"CryptoKey","k","created").await.unwrap();
        crate::q("ALTER TABLE pim_changes RENAME TO withheld_pim_changes")
            .execute(&s.backend).await.unwrap();
        let (frame,counts)=crate::pool_telemetry::measure_queries(s.session_state_counters(&owner)).await;
        assert_eq!(counts.read.calls,4,"failed joint statement plus all three original readers");
        assert_eq!(counts.read.failures,2,"joint and missing-PIM reads fail independently");
        assert_eq!(frame.mail["Email"],1);
        assert!(frame.pim.is_empty());
        assert_eq!(frame.crypto["CryptoKey"],1);
        crate::q("ALTER TABLE withheld_pim_changes RENAME TO pim_changes")
            .execute(&s.backend).await.unwrap();
        assert_eq!(s.session_state_counters(&owner).await.pim["Note"],1);
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
    #[tokio::test]
    async fn core_mail_reader_preserves_checkpoints_and_generic_custom_counters() {
        let s = store().await;
        let account = account(&s).await;
        assert!(
            s.current_core_mail_states(&account)
                .await
                .unwrap()
                .is_empty()
        );
        for (kind, head) in [
            ("Email", 13),
            ("Mailbox", 7),
            ("EmailSubmission", 2),
            ("CustomKind", 9),
        ] {
            crate::q("INSERT INTO change_log_checkpoint(account_id,type,head_state,pruned_through) VALUES(?1,?2,?3,?3)").bind(&account).bind(kind).bind(head as i64).execute(&s.backend).await.unwrap();
        }
        let core = s.current_core_mail_states(&account).await.unwrap();
        let all = s.current_states(&account).await.unwrap();
        assert_eq!(core.len(), 3);
        for kind in ["Email", "Mailbox", "EmailSubmission"] {
            assert_eq!(core[kind], all[kind]);
            assert_eq!(core[kind], s.current_state(&account, kind).await.unwrap());
        }
        assert_eq!(all["CustomKind"], 9);
        assert!(!core.contains_key("CustomKind"));
        assert!(
            s.current_core_mail_states("foreign-account")
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            s.record_change(&account, "Email", "next", "updated")
                .await
                .unwrap(),
            14
        );
        assert_eq!(
            s.current_core_mail_states(&account).await.unwrap()["Email"],
            14
        );
        assert_eq!(s.current_states(&account).await.unwrap()["CustomKind"], 9);
    }
    #[tokio::test]
    async fn core_mail_reader_uses_indexed_heads_without_scanning_retained_history() {
        use super::CORE_MAIL_STATES_SQL;
        use crate::backend::Backend;
        use sqlx::Row;
        use std::sync::{
            Arc,
            atomic::{AtomicU64, Ordering},
        };
        let s = store().await;
        let account = account(&s).await;
        s.record_changes(
            &account,
            "Email",
            &(0..20000)
                .map(|n| format!("message-{n}"))
                .collect::<Vec<_>>(),
            "updated",
        )
        .await
        .unwrap();
        s.record_changes(
            &account,
            "CustomKind",
            &(0..5000).map(|n| format!("custom-{n}")).collect::<Vec<_>>(),
            "updated",
        )
        .await
        .unwrap();
        s.record_change(&account, "Mailbox", "mailbox", "updated")
            .await
            .unwrap();
        s.record_change(&account, "EmailSubmission", "submission", "updated")
            .await
            .unwrap();
        let Backend::Sqlite(pool) = &s.backend else {
            unreachable!()
        };
        let pin = pool.pin().await;
        let plan = sqlx::query(&format!("EXPLAIN QUERY PLAN {CORE_MAIL_STATES_SQL}"))
            .bind(&account)
            .fetch_all(pin.pool())
            .await
            .unwrap()
            .into_iter()
            .map(|row| row.get::<String, _>("detail"))
            .collect::<Vec<_>>();
        assert_eq!(
            plan.iter()
                .filter(|line| line.contains("SEARCH changes USING"))
                .count(),
            3,
            "{plan:?}"
        );
        assert_eq!(
            plan.iter()
                .filter(|line| line.contains("SEARCH change_log_checkpoint USING"))
                .count(),
            3,
            "{plan:?}"
        );
        assert!(
            !plan.iter().any(|line| line.contains("SCAN changes")),
            "{plan:?}"
        );
        let count = Arc::new(AtomicU64::new(0));
        let observed = count.clone();
        let mut connection = pin.pool().acquire().await.unwrap();
        connection
            .lock_handle()
            .await
            .unwrap()
            .set_progress_handler(1000, move || {
                observed.fetch_add(1, Ordering::Relaxed);
                true
            });
        drop(connection);
        let core = s.current_core_mail_states(&account).await.unwrap();
        let fast_steps = count.swap(0, Ordering::Relaxed);
        let generic = s.current_states(&account).await.unwrap();
        let generic_steps = count.load(Ordering::Relaxed);
        pin.pool()
            .acquire()
            .await
            .unwrap()
            .lock_handle()
            .await
            .unwrap()
            .remove_progress_handler();
        assert!(
            generic_steps > 100,
            "control must prove the VM probe observes the retained scan: {generic_steps}"
        );
        assert!(
            fast_steps <= 2,
            "indexed heads must not scan retained rows: {fast_steps}"
        );
        for kind in ["Email", "Mailbox", "EmailSubmission"] {
            assert_eq!(core[kind], generic[kind]);
        }
        assert_eq!(generic["CustomKind"], 1);
        assert!(!core.contains_key("CustomKind"));
    }
    #[tokio::test]
    async fn postgres_core_mail_reader_preserves_generic_and_checkpoint_contract() {
        let Ok(dsn) = std::env::var("MW_BG_TEST_PG") else {
            eprintln!("SKIP PostgreSQL core mail counters: MW_BG_TEST_PG is unset");
            return;
        };
        let s = Store::open_postgres(&dsn, ServerKey::generate())
            .await
            .unwrap();
        let name = format!("core-counters-{}", uuid::Uuid::new_v4());
        let account = s
            .create_account(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host: "fixture.invalid",
                    port: 993,
                    tls: "implicit",
                    username: &name,
                    sync_policy_json: "{}",
                },
                &Credentials {
                    username: name.clone(),
                    password: "fixture".into(),
                },
            )
            .await
            .unwrap();
        assert!(
            s.current_core_mail_states(&account)
                .await
                .unwrap()
                .is_empty()
        );
        for kind in ["Email", "Mailbox", "EmailSubmission", "CustomKind"] {
            s.record_change(&account, kind, "fixture", "updated")
                .await
                .unwrap();
            crate::q("INSERT INTO change_log_checkpoint(account_id,type,head_state,pruned_through) VALUES(?1,?2,1,1)").bind(&account).bind(kind).execute(&s.backend).await.unwrap();
        }
        crate::q("DELETE FROM changes WHERE account_id=?1")
            .bind(&account)
            .execute(&s.backend)
            .await
            .unwrap();
        let core = s.current_core_mail_states(&account).await.unwrap();
        let generic = s.current_states(&account).await.unwrap();
        assert_eq!(core.len(), 3);
        for kind in ["Email", "Mailbox", "EmailSubmission"] {
            assert_eq!(core[kind], generic[kind]);
            assert_eq!(core[kind], 1);
        }
        assert_eq!(generic["CustomKind"], 1);
        assert_eq!(
            s.record_change(&account, "Email", "next", "updated")
                .await
                .unwrap(),
            2
        );
        assert_eq!(
            s.current_core_mail_states(&account).await.unwrap()["Email"],
            2
        );
        crate::q("DELETE FROM accounts WHERE id=?1")
            .bind(&account)
            .execute(&s.backend)
            .await
            .unwrap();
        s.close().await;
    }
}
