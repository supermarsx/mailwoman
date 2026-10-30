//! t94: per-user Focused inbox overrides (`focused_sender`, migration 0076).
//!
//! One row per (owner, sender): "Always move to Focused" or "Always move to
//! Other". The owner is the personal owner subject the server resolves from the
//! session (the same owner as `personal_signature`), never a value from the
//! request, so one user can neither read nor change another's rows. Senders are
//! stored lower-cased after [`normalize_focused_sender`].

use chrono::Utc;

use crate::backend::q;
use crate::{Store, StoreError};

/// At most this many overrides per owner.
pub const MAX_FOCUSED_SENDERS: usize = 1000;

/// One override.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusedSenderRow {
    pub sender: String,
    /// `focused` or `other`.
    pub tab: String,
    /// Unix milliseconds of the last write.
    pub updated_at: i64,
}

/// What a write did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedSenderWrite {
    Applied,
    LimitReached,
}

/// A tab value the table accepts.
pub fn valid_focus_tab(tab: &str) -> bool {
    matches!(tab, "focused" | "other")
}

/// One mailbox address, lower-cased; display names and lists are refused.
pub fn normalize_focused_sender(address: &str) -> Result<String, StoreError> {
    Ok(crate::receipts::normalize_sender(address)?.to_lowercase())
}

impl Store {
    /// The owner's overrides, by sender.
    pub async fn list_focused_senders(
        &self,
        owner: &str,
    ) -> Result<Vec<FocusedSenderRow>, StoreError> {
        let rows = q("SELECT sender, tab, updated_at FROM focused_sender WHERE owner_subject = ?1 ORDER BY sender LIMIT 1001")
            .bind(owner)
            .fetch_all(&self.backend)
            .await?;
        Ok(rows
            .iter()
            .map(|row| FocusedSenderRow {
                sender: row.get_string("sender"),
                tab: row.get_string("tab"),
                updated_at: row.get_i64("updated_at"),
            })
            .collect())
    }

    /// Set (`Some(tab)`) or remove (`None`) the owner's override for `sender`.
    pub async fn set_focused_sender(
        &self,
        owner: &str,
        sender: &str,
        tab: Option<&str>,
    ) -> Result<FocusedSenderWrite, StoreError> {
        let sender = normalize_focused_sender(sender)?;
        let Some(tab) = tab else {
            q("DELETE FROM focused_sender WHERE owner_subject = ?1 AND sender = ?2")
                .bind(owner)
                .bind(&sender)
                .execute(&self.backend)
                .await?;
            return Ok(FocusedSenderWrite::Applied);
        };
        if !valid_focus_tab(tab) {
            return Err(StoreError::Corrupt("invalid focused inbox tab".into()));
        }
        let mut tx = self.backend.begin().await?;
        let existing =
            q("SELECT sender FROM focused_sender WHERE owner_subject = ?1 AND sender = ?2")
                .bind(owner)
                .bind(&sender)
                .fetch_all_tx(&mut tx)
                .await?;
        if existing.is_empty() {
            let count = q("SELECT sender FROM focused_sender WHERE owner_subject = ?1 LIMIT 1001")
                .bind(owner)
                .fetch_all_tx(&mut tx)
                .await?
                .len();
            if count >= MAX_FOCUSED_SENDERS {
                tx.commit().await?;
                return Ok(FocusedSenderWrite::LimitReached);
            }
        }
        q("INSERT INTO focused_sender (owner_subject, sender, tab, updated_at) VALUES (?1, ?2, ?3, ?4)
           ON CONFLICT(owner_subject, sender) DO UPDATE SET tab = excluded.tab, updated_at = excluded.updated_at")
            .bind(owner)
            .bind(&sender)
            .bind(tab)
            .bind(Utc::now().timestamp_millis())
            .execute_tx(&mut tx)
            .await?;
        tx.commit().await?;
        Ok(FocusedSenderWrite::Applied)
    }

    /// The one-time import of a browser's old local overrides: each valid entry
    /// the owner has no row for is added, up to the limit; rows already on the
    /// server win. Invalid entries are skipped. Returns how many were added.
    pub async fn import_focused_senders(
        &self,
        owner: &str,
        entries: &[(String, String)],
    ) -> Result<usize, StoreError> {
        let mut added = 0;
        let mut have = self.list_focused_senders(owner).await?.len();
        for (sender, tab) in entries {
            if have >= MAX_FOCUSED_SENDERS {
                break;
            }
            let Ok(sender) = normalize_focused_sender(sender) else {
                continue;
            };
            if !valid_focus_tab(tab) {
                continue;
            }
            let inserted = q("INSERT INTO focused_sender (owner_subject, sender, tab, updated_at) VALUES (?1, ?2, ?3, ?4)
               ON CONFLICT(owner_subject, sender) DO NOTHING")
                .bind(owner)
                .bind(&sender)
                .bind(tab.as_str())
                .bind(Utc::now().timestamp_millis())
                .execute(&self.backend)
                .await?;
            if inserted > 0 {
                added += 1;
                have += 1;
            }
        }
        Ok(added)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServerKey;

    #[tokio::test]
    async fn overrides_are_per_owner_normalized_and_bounded() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        assert_eq!(
            store
                .set_focused_sender("ann", "News@Shop.Example", Some("other"))
                .await
                .unwrap(),
            FocusedSenderWrite::Applied
        );
        store
            .set_focused_sender("ann", "boss@work.example", Some("focused"))
            .await
            .unwrap();
        store
            .set_focused_sender("bob", "news@shop.example", Some("focused"))
            .await
            .unwrap();
        let ann = store.list_focused_senders("ann").await.unwrap();
        assert_eq!(
            ann.iter()
                .map(|r| (r.sender.as_str(), r.tab.as_str()))
                .collect::<Vec<_>>(),
            [
                ("boss@work.example", "focused"),
                ("news@shop.example", "other")
            ]
        );
        // Another owner's row with the same sender is untouched by ann's writes.
        store
            .set_focused_sender("ann", "news@shop.example", None)
            .await
            .unwrap();
        assert_eq!(store.list_focused_senders("ann").await.unwrap().len(), 1);
        let bob = store.list_focused_senders("bob").await.unwrap();
        assert_eq!(bob.len(), 1);
        assert_eq!(bob[0].tab, "focused");
        // Changing the tab updates in place.
        store
            .set_focused_sender("ann", "boss@work.example", Some("other"))
            .await
            .unwrap();
        assert_eq!(
            store.list_focused_senders("ann").await.unwrap()[0].tab,
            "other"
        );
        // Refused input.
        assert!(
            store
                .set_focused_sender("ann", "Name <a@b.example>", Some("other"))
                .await
                .is_err()
        );
        assert!(
            store
                .set_focused_sender("ann", "a@b.example", Some("spam"))
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn the_limit_refuses_a_new_sender_but_not_a_change() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let entries: Vec<(String, String)> = (0..MAX_FOCUSED_SENDERS)
            .map(|i| (format!("s{i}@x.example"), "other".to_string()))
            .collect();
        assert_eq!(
            store.import_focused_senders("ann", &entries).await.unwrap(),
            MAX_FOCUSED_SENDERS
        );
        assert_eq!(
            store
                .set_focused_sender("ann", "new@x.example", Some("other"))
                .await
                .unwrap(),
            FocusedSenderWrite::LimitReached
        );
        assert_eq!(
            store
                .set_focused_sender("ann", "s1@x.example", Some("focused"))
                .await
                .unwrap(),
            FocusedSenderWrite::Applied
        );
    }

    #[tokio::test]
    async fn import_keeps_server_rows_and_skips_invalid_entries() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        store
            .set_focused_sender("ann", "a@x.example", Some("focused"))
            .await
            .unwrap();
        let added = store
            .import_focused_senders(
                "ann",
                &[
                    ("A@x.example".into(), "other".into()),
                    ("b@x.example".into(), "other".into()),
                    ("not an address".into(), "other".into()),
                    ("c@x.example".into(), "bogus".into()),
                ],
            )
            .await
            .unwrap();
        assert_eq!(added, 1);
        let rows = store.list_focused_senders("ann").await.unwrap();
        assert_eq!(
            rows.iter()
                .map(|r| (r.sender.as_str(), r.tab.as_str()))
                .collect::<Vec<_>>(),
            [("a@x.example", "focused"), ("b@x.example", "other")]
        );
    }
}
