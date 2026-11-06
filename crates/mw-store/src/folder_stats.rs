//! What the cache knows about one folder, for the folder Properties dialog.
//!
//! Everything here is read from the `messages` rows the sync keeps, so it is
//! the size of what has been synced, not a count taken on the server. The
//! caller compares [`FolderStats::cached`] with the folder's own total and says
//! "approximate" when they differ.

use crate::backend::q;
use crate::{Store, StoreError};

/// How many of the largest messages [`Store::folder_stats`] returns.
pub const LARGEST_MESSAGES: i64 = 5;

/// One of the biggest messages of a folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LargeMessage {
    pub stable_id: String,
    pub size: u64,
}

/// Aggregates over the cached messages of one folder.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FolderStats {
    /// Messages in the cache for this folder.
    pub cached: u64,
    /// Sum of their sizes in bytes.
    pub size_bytes: u64,
    /// Oldest and newest INTERNALDATE (RFC 3339), when any message has one.
    pub oldest: Option<String>,
    pub newest: Option<String>,
    pub flagged: u64,
    pub drafts: u64,
    pub unread: u64,
    /// Messages whose body is held locally.
    pub bodies_cached: u64,
    pub largest: Vec<LargeMessage>,
}

impl Store {
    /// Aggregates over one folder's cached messages.
    pub async fn folder_stats(&self, mailbox_id: &str) -> Result<FolderStats, StoreError> {
        let count = |sql: &'static str| {
            q(sql).bind(mailbox_id).fetch_scalar_i64(&self.backend)
        };
        let cached = count("SELECT COUNT(*) FROM messages WHERE mailbox_id = ?1").await?;
        let size = count("SELECT COALESCE(SUM(size), 0) FROM messages WHERE mailbox_id = ?1").await?;
        let flagged = count(
            "SELECT COUNT(*) FROM messages WHERE mailbox_id = ?1 AND flags_json LIKE '%\"Flagged\"%'",
        )
        .await?;
        let drafts = count(
            "SELECT COUNT(*) FROM messages WHERE mailbox_id = ?1 AND flags_json LIKE '%\"Draft\"%'",
        )
        .await?;
        let unread = count(
            "SELECT COUNT(*) FROM messages WHERE mailbox_id = ?1 AND flags_json NOT LIKE '%\"Seen\"%'",
        )
        .await?;
        let bodies_cached = count(
            "SELECT COUNT(*) FROM messages WHERE mailbox_id = ?1 AND blob_ref IS NOT NULL",
        )
        .await?;
        let oldest = q("SELECT MIN(internaldate) FROM messages WHERE mailbox_id = ?1")
            .bind(mailbox_id)
            .fetch_opt_scalar_string(&self.backend)
            .await?;
        let newest = q("SELECT MAX(internaldate) FROM messages WHERE mailbox_id = ?1")
            .bind(mailbox_id)
            .fetch_opt_scalar_string(&self.backend)
            .await?;
        let rows = q("SELECT stable_id, size FROM messages WHERE mailbox_id = ?1
             ORDER BY size DESC, stable_id LIMIT ?2")
        .bind(mailbox_id)
        .bind(LARGEST_MESSAGES)
        .fetch_all(&self.backend)
        .await?;
        let largest = rows
            .iter()
            .map(|row| LargeMessage {
                stable_id: row.get_string("stable_id"),
                size: row.get_i64("size").max(0) as u64,
            })
            .collect();
        let n = |v: i64| v.max(0) as u64;
        Ok(FolderStats {
            cached: n(cached),
            size_bytes: n(size),
            oldest,
            newest,
            flagged: n(flagged),
            drafts: n(drafts),
            unread: n(unread),
            bodies_cached: n(bodies_cached),
            largest,
        })
    }
}
