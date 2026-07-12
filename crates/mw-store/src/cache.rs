//! V1 message-cache repository (plan §2.3) layered over the existing [`Store`].
//!
//! These methods back `mw-engine`'s sync engine and JMAP surface: configured
//! accounts, mailbox/folder state, the cached message index keyed by
//! `(account, mailbox, UIDVALIDITY, UID)`, sealed bodies/envelopes, JWZ thread
//! roots, the POP3 UIDL set, and per-mailbox sync cursors.
//!
//! Design constraint (plan §1.9, §5): `mw-store` must **not** depend on
//! `mw-engine` (that would cycle — `mw-engine` depends on `mw-store`). So every
//! engine-owned value crosses this seam as an **opaque** primitive: flags as a
//! JSON `&str`, the sync cursor as a JSON `&str`, mailbox role as a plain
//! string, the envelope as raw bytes. The store assigns the opaque
//! `stable_id`/`thread_id`/`blob_ref` tokens and never interprets engine JSON.
//!
//! Stable-id scheme (plan §1.6): `stable_id` is an opaque random 256-bit token
//! (`seal::random_token`), allocated once when a message is first seen and
//! preserved across re-sync. On UIDVALIDITY change the same token is carried to
//! the message's new `(uidvalidity, uid)` by matching
//! `(message_id, internaldate, size)` within the mailbox.

use std::collections::BTreeSet;

use chrono::Utc;
use sqlx::Row;

use crate::{Store, StoreError, seal};

/// Kind of upstream account (mirrors the `accounts.kind` CHECK constraint).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountKind {
    Imap,
    Pop3,
}

impl AccountKind {
    fn as_str(self) -> &'static str {
        match self {
            AccountKind::Imap => "imap",
            AccountKind::Pop3 => "pop3",
        }
    }

    fn parse(s: &str) -> Result<Self, StoreError> {
        match s {
            "imap" => Ok(AccountKind::Imap),
            "pop3" => Ok(AccountKind::Pop3),
            other => Err(StoreError::Corrupt(format!(
                "unknown account kind {other:?}"
            ))),
        }
    }
}

/// Parameters to create a configured account. Credentials are sealed with the
/// existing [`crate::ServerKey`]; only `username` is stored in the clear (it is
/// already visible in the session and needed for reconnect/display).
#[derive(Debug, Clone)]
pub struct NewAccount<'a> {
    pub kind: AccountKind,
    pub host: &'a str,
    pub port: u16,
    pub tls: &'a str,
    pub username: &'a str,
    /// Opaque JSON the engine owns (leave-on-server policy, poll interval, …).
    pub sync_policy_json: &'a str,
}

/// A configured account row (credentials are never returned here — use
/// [`Store::account_credentials`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub id: String,
    pub kind: AccountKind,
    pub host: String,
    pub port: u16,
    pub tls: String,
    pub username: String,
    pub sync_policy_json: String,
}

/// Upsert parameters for a mailbox/folder, keyed by `(account, name,
/// uidvalidity)`.
#[derive(Debug, Clone)]
pub struct MailboxUpsert<'a> {
    pub account_id: &'a str,
    pub name: &'a str,
    /// Special-use role as an opaque lowercase string (engine maps to/from
    /// `mw_jmap::Mailbox.role`); `None` for an ordinary folder.
    pub role: Option<&'a str>,
    pub uidvalidity: u32,
    pub uidnext: u32,
    pub highestmodseq: u64,
    pub total: u32,
    pub unread: u32,
    pub parent_id: Option<&'a str>,
}

/// A mailbox/folder row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mailbox {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub role: Option<String>,
    pub uidvalidity: u32,
    pub uidnext: u32,
    pub highestmodseq: u64,
    pub total: u32,
    pub unread: u32,
    pub parent_id: Option<String>,
}

/// Upsert parameters for one cached message.
#[derive(Debug, Clone)]
pub struct MessageUpsert<'a> {
    pub account_id: &'a str,
    pub mailbox_id: &'a str,
    pub uid: u32,
    pub uidvalidity: u32,
    pub message_id: Option<&'a str>,
    pub thread_id: Option<&'a str>,
    /// INTERNALDATE as RFC3339 (used for sort + identity match).
    pub internaldate: Option<&'a str>,
    pub size: u64,
    /// Opaque JSON array of flags the engine owns.
    pub flags_json: &'a str,
    /// Parsed-envelope plaintext, sealed at rest here (`None` leaves it unset).
    pub envelope: Option<&'a [u8]>,
    /// Reference into `bodies` for the sealed raw/parsed body, if stored.
    pub blob_ref: Option<&'a str>,
}

/// A cached message row (envelope bytes fetched separately via
/// [`Store::get_envelope`] so listing stays cheap).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub stable_id: String,
    pub account_id: String,
    pub mailbox_id: String,
    pub uid: u32,
    pub uidvalidity: u32,
    pub message_id: Option<String>,
    pub thread_id: Option<String>,
    pub internaldate: Option<String>,
    pub size: u64,
    pub flags_json: String,
    pub blob_ref: Option<String>,
}

/// The backend coordinates a `stable_id` currently maps to (plan §1.6). The
/// engine translates these to/from [`mw_engine::backend::MessageRef`]; the store
/// never sees the enum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageLocation {
    pub mailbox_id: String,
    pub uidvalidity: u32,
    pub uid: u32,
}

// Small casts: SQLite stores signed 64-bit integers; UIDs/counts are u32 and
// MODSEQ/size are u64 (well within i64 for any real mailbox).
fn to_i64(v: u64) -> i64 {
    v as i64
}
fn u32_from(row: &sqlx::sqlite::SqliteRow, col: &str) -> u32 {
    row.get::<i64, _>(col) as u32
}
fn u64_from(row: &sqlx::sqlite::SqliteRow, col: &str) -> u64 {
    row.get::<i64, _>(col) as u64
}

impl Store {
    // ---- accounts -------------------------------------------------------

    /// Create a configured account, sealing its credentials. Returns the new id.
    pub async fn create_account(
        &self,
        acct: &NewAccount<'_>,
        creds: &crate::Credentials,
    ) -> Result<String, StoreError> {
        let id = seal::random_token();
        let sealed = self.key.seal(&crate::encode_creds(creds))?;
        sqlx::query(
            "INSERT INTO accounts (id, kind, host, port, tls, username, sealed_creds, sync_policy_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )
        .bind(&id)
        .bind(acct.kind.as_str())
        .bind(acct.host)
        .bind(acct.port as i64)
        .bind(acct.tls)
        .bind(acct.username)
        .bind(sealed)
        .bind(acct.sync_policy_json)
        .execute(&self.pool)
        .await?;
        Ok(id)
    }

    /// Fetch an account by id.
    pub async fn get_account(&self, id: &str) -> Result<Account, StoreError> {
        let row = sqlx::query(
            "SELECT id, kind, host, port, tls, username, sync_policy_json FROM accounts WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(StoreError::NotFound)?;
        Self::account_from_row(&row)
    }

    /// List all configured accounts (no credentials).
    pub async fn list_accounts(&self) -> Result<Vec<Account>, StoreError> {
        let rows = sqlx::query(
            "SELECT id, kind, host, port, tls, username, sync_policy_json FROM accounts ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(Self::account_from_row).collect()
    }

    /// Update an account's opaque sync-policy JSON.
    pub async fn update_sync_policy(&self, id: &str, policy_json: &str) -> Result<(), StoreError> {
        sqlx::query("UPDATE accounts SET sync_policy_json = ?2 WHERE id = ?1")
            .bind(id)
            .bind(policy_json)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Open the sealed credentials for an account.
    pub async fn account_credentials(&self, id: &str) -> Result<crate::Credentials, StoreError> {
        let row = sqlx::query("SELECT sealed_creds FROM accounts WHERE id = ?1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(StoreError::NotFound)?;
        let sealed: Vec<u8> = row.get("sealed_creds");
        crate::decode_creds(&self.key.open(&sealed)?)
    }

    fn account_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Account, StoreError> {
        Ok(Account {
            id: row.get("id"),
            kind: AccountKind::parse(row.get::<String, _>("kind").as_str())?,
            host: row.get("host"),
            port: u32_from(row, "port") as u16,
            tls: row.get("tls"),
            username: row.get("username"),
            sync_policy_json: row.get("sync_policy_json"),
        })
    }

    // ---- mailboxes ------------------------------------------------------

    /// Upsert a mailbox by `(account, name, uidvalidity)`, returning its stable
    /// opaque id. Counts/uidnext/highestmodseq/role/parent are refreshed on
    /// conflict; the id is preserved.
    pub async fn upsert_mailbox(&self, m: &MailboxUpsert<'_>) -> Result<String, StoreError> {
        let id = seal::random_token();
        let row = sqlx::query(
            "INSERT INTO mailboxes
                 (id, account_id, name, role, uidvalidity, uidnext, highestmodseq, total, unread, parent_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(account_id, name, uidvalidity) DO UPDATE SET
                 role = excluded.role,
                 uidnext = excluded.uidnext,
                 highestmodseq = excluded.highestmodseq,
                 total = excluded.total,
                 unread = excluded.unread,
                 parent_id = excluded.parent_id
             RETURNING id",
        )
        .bind(&id)
        .bind(m.account_id)
        .bind(m.name)
        .bind(m.role)
        .bind(m.uidvalidity as i64)
        .bind(m.uidnext as i64)
        .bind(to_i64(m.highestmodseq))
        .bind(m.total as i64)
        .bind(m.unread as i64)
        .bind(m.parent_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.get("id"))
    }

    /// List an account's mailboxes ordered by name.
    pub async fn list_mailboxes(&self, account_id: &str) -> Result<Vec<Mailbox>, StoreError> {
        let rows = sqlx::query(
            "SELECT id, account_id, name, role, uidvalidity, uidnext, highestmodseq, total, unread, parent_id
             FROM mailboxes WHERE account_id = ?1 ORDER BY name",
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(Self::mailbox_from_row).collect())
    }

    /// Fetch one mailbox by id.
    pub async fn get_mailbox(&self, id: &str) -> Result<Mailbox, StoreError> {
        let row = sqlx::query(
            "SELECT id, account_id, name, role, uidvalidity, uidnext, highestmodseq, total, unread, parent_id
             FROM mailboxes WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(StoreError::NotFound)?;
        Ok(Self::mailbox_from_row(&row))
    }

    /// Refresh a mailbox's counters after a sync.
    pub async fn update_mailbox_counts(
        &self,
        id: &str,
        uidnext: u32,
        highestmodseq: u64,
        total: u32,
        unread: u32,
    ) -> Result<(), StoreError> {
        sqlx::query(
            "UPDATE mailboxes SET uidnext = ?2, highestmodseq = ?3, total = ?4, unread = ?5 WHERE id = ?1",
        )
        .bind(id)
        .bind(uidnext as i64)
        .bind(to_i64(highestmodseq))
        .bind(total as i64)
        .bind(unread as i64)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Set (or clear) a mailbox's special-use role string.
    pub async fn set_mailbox_role(&self, id: &str, role: Option<&str>) -> Result<(), StoreError> {
        sqlx::query("UPDATE mailboxes SET role = ?2 WHERE id = ?1")
            .bind(id)
            .bind(role)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Handle a UIDVALIDITY change (plan §1.6): bump the mailbox's UIDVALIDITY
    /// and drop its stale sync cursor so the engine performs a full re-sync. The
    /// message rows are kept in place; re-syncing upserts carry each existing
    /// `stable_id` onto its new `(uidvalidity, uid)` via the identity heuristic
    /// in [`Store::upsert_message`].
    pub async fn revalidate_mailbox(
        &self,
        mailbox_id: &str,
        new_uidvalidity: u32,
    ) -> Result<(), StoreError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "UPDATE mailboxes SET uidvalidity = ?2, uidnext = 0, highestmodseq = 0 WHERE id = ?1",
        )
        .bind(mailbox_id)
        .bind(new_uidvalidity as i64)
        .execute(&mut *tx)
        .await?;
        sqlx::query("DELETE FROM sync_state WHERE mailbox_id = ?1")
            .bind(mailbox_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    fn mailbox_from_row(row: &sqlx::sqlite::SqliteRow) -> Mailbox {
        Mailbox {
            id: row.get("id"),
            account_id: row.get("account_id"),
            name: row.get("name"),
            role: row.get("role"),
            uidvalidity: u32_from(row, "uidvalidity"),
            uidnext: u32_from(row, "uidnext"),
            highestmodseq: u64_from(row, "highestmodseq"),
            total: u32_from(row, "total"),
            unread: u32_from(row, "unread"),
            parent_id: row.get("parent_id"),
        }
    }

    // ---- messages -------------------------------------------------------

    /// Upsert a cached message, allocating a stable id on first sight and
    /// preserving it thereafter. Resolution order (plan §1.6):
    ///
    /// 1. Existing row at `(account, mailbox, uidvalidity, uid)` → update in
    ///    place.
    /// 2. Else, when `message_id` + `internaldate` are present, an existing row
    ///    in the same mailbox with matching `(message_id, internaldate, size)`
    ///    → re-key it onto the new `(uidvalidity, uid)`, preserving `stable_id`
    ///    (this is the UIDVALIDITY-change / re-sync path).
    /// 3. Else insert a fresh row with a newly allocated `stable_id`.
    ///
    /// Returns the message's stable id.
    pub async fn upsert_message(&self, m: &MessageUpsert<'_>) -> Result<String, StoreError> {
        let sealed_env = match m.envelope {
            Some(bytes) => Some(self.key.seal(bytes)?),
            None => None,
        };
        let mut tx = self.pool.begin().await?;

        // (1) exact UID coordinates.
        let existing: Option<String> = sqlx::query_scalar(
            "SELECT stable_id FROM messages
             WHERE account_id = ?1 AND mailbox_id = ?2 AND uidvalidity = ?3 AND uid = ?4",
        )
        .bind(m.account_id)
        .bind(m.mailbox_id)
        .bind(m.uidvalidity as i64)
        .bind(m.uid as i64)
        .fetch_optional(&mut *tx)
        .await?;

        // (2) identity match across a UIDVALIDITY change.
        let stable_id = match existing {
            Some(id) => id,
            None => {
                let identity: Option<String> = match (m.message_id, m.internaldate) {
                    (Some(mid), Some(date)) => {
                        sqlx::query_scalar(
                            "SELECT stable_id FROM messages
                         WHERE account_id = ?1 AND mailbox_id = ?2 AND message_id = ?3
                           AND internaldate = ?4 AND size = ?5
                         LIMIT 1",
                        )
                        .bind(m.account_id)
                        .bind(m.mailbox_id)
                        .bind(mid)
                        .bind(date)
                        .bind(to_i64(m.size))
                        .fetch_optional(&mut *tx)
                        .await?
                    }
                    _ => None,
                };
                identity.unwrap_or_else(seal::random_token)
            }
        };

        // Upsert the full row under the resolved stable_id.
        sqlx::query(
            "INSERT INTO messages
                 (stable_id, account_id, mailbox_id, uid, uidvalidity, message_id, thread_id,
                  internaldate, size, flags_json, envelope_json, blob_ref)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(stable_id) DO UPDATE SET
                 mailbox_id = excluded.mailbox_id,
                 uid = excluded.uid,
                 uidvalidity = excluded.uidvalidity,
                 message_id = excluded.message_id,
                 thread_id = excluded.thread_id,
                 internaldate = excluded.internaldate,
                 size = excluded.size,
                 flags_json = excluded.flags_json,
                 envelope_json = COALESCE(excluded.envelope_json, messages.envelope_json),
                 blob_ref = COALESCE(excluded.blob_ref, messages.blob_ref)",
        )
        .bind(&stable_id)
        .bind(m.account_id)
        .bind(m.mailbox_id)
        .bind(m.uid as i64)
        .bind(m.uidvalidity as i64)
        .bind(m.message_id)
        .bind(m.thread_id)
        .bind(m.internaldate)
        .bind(to_i64(m.size))
        .bind(m.flags_json)
        .bind(sealed_env)
        .bind(m.blob_ref)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(stable_id)
    }

    /// Fetch a message row by stable id.
    pub async fn get_message(&self, stable_id: &str) -> Result<Message, StoreError> {
        let row = sqlx::query(
            "SELECT stable_id, account_id, mailbox_id, uid, uidvalidity, message_id, thread_id,
                    internaldate, size, flags_json, blob_ref
             FROM messages WHERE stable_id = ?1",
        )
        .bind(stable_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(StoreError::NotFound)?;
        Ok(Self::message_from_row(&row))
    }

    /// Map backend coordinates → stable id.
    pub async fn stable_id_for(
        &self,
        account_id: &str,
        mailbox_id: &str,
        uidvalidity: u32,
        uid: u32,
    ) -> Result<Option<String>, StoreError> {
        Ok(sqlx::query_scalar(
            "SELECT stable_id FROM messages
             WHERE account_id = ?1 AND mailbox_id = ?2 AND uidvalidity = ?3 AND uid = ?4",
        )
        .bind(account_id)
        .bind(mailbox_id)
        .bind(uidvalidity as i64)
        .bind(uid as i64)
        .fetch_optional(&self.pool)
        .await?)
    }

    /// Map stable id → backend coordinates (mailbox + UIDVALIDITY + UID).
    pub async fn message_location(
        &self,
        stable_id: &str,
    ) -> Result<Option<MessageLocation>, StoreError> {
        let row =
            sqlx::query("SELECT mailbox_id, uidvalidity, uid FROM messages WHERE stable_id = ?1")
                .bind(stable_id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(row.map(|r| MessageLocation {
            mailbox_id: r.get("mailbox_id"),
            uidvalidity: u32_from(&r, "uidvalidity"),
            uid: u32_from(&r, "uid"),
        }))
    }

    /// Stable ids in a mailbox for `Email/query`, newest first (INTERNALDATE
    /// desc, UID desc tie-break), with `limit`/`offset` paging.
    pub async fn list_message_ids(
        &self,
        mailbox_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<String>, StoreError> {
        Ok(sqlx::query_scalar(
            "SELECT stable_id FROM messages WHERE mailbox_id = ?1
             ORDER BY internaldate DESC, uid DESC
             LIMIT ?2 OFFSET ?3",
        )
        .bind(mailbox_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?)
    }

    /// Replace a message's opaque flags JSON (server-authoritative, SPEC §15.2).
    pub async fn set_flags(&self, stable_id: &str, flags_json: &str) -> Result<(), StoreError> {
        let n = sqlx::query("UPDATE messages SET flags_json = ?2 WHERE stable_id = ?1")
            .bind(stable_id)
            .bind(flags_json)
            .execute(&self.pool)
            .await?
            .rows_affected();
        if n == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }

    /// Assign a message to a thread.
    pub async fn set_thread(&self, stable_id: &str, thread_id: &str) -> Result<(), StoreError> {
        sqlx::query("UPDATE messages SET thread_id = ?2 WHERE stable_id = ?1")
            .bind(stable_id)
            .bind(thread_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Delete a cached message (EXPUNGE/VANISHED/dropped UIDL).
    pub async fn delete_message(&self, stable_id: &str) -> Result<(), StoreError> {
        sqlx::query("DELETE FROM messages WHERE stable_id = ?1")
            .bind(stable_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    fn message_from_row(row: &sqlx::sqlite::SqliteRow) -> Message {
        Message {
            stable_id: row.get("stable_id"),
            account_id: row.get("account_id"),
            mailbox_id: row.get("mailbox_id"),
            uid: u32_from(row, "uid"),
            uidvalidity: u32_from(row, "uidvalidity"),
            message_id: row.get("message_id"),
            thread_id: row.get("thread_id"),
            internaldate: row.get("internaldate"),
            size: u64_from(row, "size"),
            flags_json: row.get("flags_json"),
            blob_ref: row.get("blob_ref"),
        }
    }

    // ---- bodies (sealed at rest) ---------------------------------------

    /// Seal and store a raw/parsed body blob; returns its opaque `blob_ref`.
    pub async fn put_body(&self, account_id: &str, plaintext: &[u8]) -> Result<String, StoreError> {
        let blob_ref = seal::random_token();
        let sealed = self.key.seal(plaintext)?;
        sqlx::query("INSERT INTO bodies (blob_ref, account_id, sealed_bytes) VALUES (?1, ?2, ?3)")
            .bind(&blob_ref)
            .bind(account_id)
            .bind(sealed)
            .execute(&self.pool)
            .await?;
        Ok(blob_ref)
    }

    /// Open a stored body blob.
    pub async fn get_body(&self, blob_ref: &str) -> Result<Option<Vec<u8>>, StoreError> {
        let row = sqlx::query("SELECT sealed_bytes FROM bodies WHERE blob_ref = ?1")
            .bind(blob_ref)
            .fetch_optional(&self.pool)
            .await?;
        match row {
            Some(r) => {
                let sealed: Vec<u8> = r.get("sealed_bytes");
                Ok(Some(self.key.open(&sealed)?))
            }
            None => Ok(None),
        }
    }

    /// Open a message's sealed envelope bytes (for `Email/get` without
    /// re-parsing), if one was stored.
    pub async fn get_envelope(&self, stable_id: &str) -> Result<Option<Vec<u8>>, StoreError> {
        let row = sqlx::query("SELECT envelope_json FROM messages WHERE stable_id = ?1")
            .bind(stable_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(StoreError::NotFound)?;
        let sealed: Option<Vec<u8>> = row.get("envelope_json");
        match sealed {
            Some(bytes) => Ok(Some(self.key.open(&bytes)?)),
            None => Ok(None),
        }
    }

    // ---- threads --------------------------------------------------------

    /// Look up (or create) the thread id for a root Message-ID within an
    /// account. The engine computes JWZ roots; the store persists the mapping.
    pub async fn assign_thread(
        &self,
        account_id: &str,
        root_message_id: &str,
    ) -> Result<String, StoreError> {
        if let Some(existing) = self.thread_for_root(account_id, root_message_id).await? {
            return Ok(existing);
        }
        let thread_id = seal::random_token();
        let row = sqlx::query(
            "INSERT INTO threads (thread_id, account_id, root_message_id) VALUES (?1, ?2, ?3)
             ON CONFLICT(account_id, root_message_id) DO UPDATE SET root_message_id = excluded.root_message_id
             RETURNING thread_id",
        )
        .bind(&thread_id)
        .bind(account_id)
        .bind(root_message_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.get("thread_id"))
    }

    /// Look up an existing thread id by its root Message-ID.
    pub async fn thread_for_root(
        &self,
        account_id: &str,
        root_message_id: &str,
    ) -> Result<Option<String>, StoreError> {
        Ok(sqlx::query_scalar(
            "SELECT thread_id FROM threads WHERE account_id = ?1 AND root_message_id = ?2",
        )
        .bind(account_id)
        .bind(root_message_id)
        .fetch_optional(&self.pool)
        .await?)
    }

    // ---- pop3 uidl ------------------------------------------------------

    /// Record that a POP3 UIDL has been ingested, mapped to a stable id.
    pub async fn record_uidl(
        &self,
        account_id: &str,
        uidl: &str,
        stable_id: &str,
    ) -> Result<(), StoreError> {
        sqlx::query(
            "INSERT INTO pop3_uidl (account_id, uidl, stable_id) VALUES (?1, ?2, ?3)
             ON CONFLICT(account_id, uidl) DO UPDATE SET stable_id = excluded.stable_id",
        )
        .bind(account_id)
        .bind(uidl)
        .bind(stable_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// The set of UIDLs already ingested for an account (the POP3 sync cursor's
    /// `seen` set — feed it back to the backend to diff against LIST/UIDL).
    pub async fn seen_uidls(&self, account_id: &str) -> Result<BTreeSet<String>, StoreError> {
        let rows: Vec<String> =
            sqlx::query_scalar("SELECT uidl FROM pop3_uidl WHERE account_id = ?1")
                .bind(account_id)
                .fetch_all(&self.pool)
                .await?;
        Ok(rows.into_iter().collect())
    }

    /// Map a POP3 UIDL back to its stable id.
    pub async fn stable_id_for_uidl(
        &self,
        account_id: &str,
        uidl: &str,
    ) -> Result<Option<String>, StoreError> {
        Ok(sqlx::query_scalar(
            "SELECT stable_id FROM pop3_uidl WHERE account_id = ?1 AND uidl = ?2",
        )
        .bind(account_id)
        .bind(uidl)
        .fetch_optional(&self.pool)
        .await?)
    }

    // ---- sync state -----------------------------------------------------

    /// Persist a mailbox's opaque sync cursor JSON (the engine serializes
    /// `mw_engine::backend::SyncCursor`; the store keeps it verbatim).
    pub async fn save_cursor(
        &self,
        account_id: &str,
        mailbox_id: &str,
        cursor_json: &str,
    ) -> Result<(), StoreError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO sync_state (account_id, mailbox_id, cursor_json, last_sync_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(account_id, mailbox_id) DO UPDATE SET
                 cursor_json = excluded.cursor_json,
                 last_sync_at = excluded.last_sync_at",
        )
        .bind(account_id)
        .bind(mailbox_id)
        .bind(cursor_json)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Load a mailbox's persisted sync cursor JSON, if any.
    pub async fn load_cursor(
        &self,
        account_id: &str,
        mailbox_id: &str,
    ) -> Result<Option<String>, StoreError> {
        Ok(sqlx::query_scalar(
            "SELECT cursor_json FROM sync_state WHERE account_id = ?1 AND mailbox_id = ?2",
        )
        .bind(account_id)
        .bind(mailbox_id)
        .fetch_optional(&self.pool)
        .await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Credentials, ServerKey, Store};

    async fn store() -> Store {
        Store::open_in_memory(ServerKey::generate()).await.unwrap()
    }

    fn creds() -> Credentials {
        Credentials {
            username: "imap-user".into(),
            password: "hunter2".into(),
        }
    }

    async fn seed_account(s: &Store) -> String {
        s.create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example.org",
                port: 993,
                tls: "implicit",
                username: "imap-user",
                sync_policy_json: r#"{"keep":true}"#,
            },
            &creds(),
        )
        .await
        .unwrap()
    }

    async fn seed_mailbox(s: &Store, account_id: &str, name: &str, uidvalidity: u32) -> String {
        s.upsert_mailbox(&MailboxUpsert {
            account_id,
            name,
            role: Some("inbox"),
            uidvalidity,
            uidnext: 1,
            highestmodseq: 0,
            total: 0,
            unread: 0,
            parent_id: None,
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn migration_0002_applies_over_0001() {
        // open_in_memory runs both migrations; V0 API must still work too.
        let s = store().await;
        s.set_setting("theme", "grove-dark").await.unwrap();
        assert_eq!(
            s.get_setting("theme").await.unwrap().as_deref(),
            Some("grove-dark")
        );
        assert!(s.list_accounts().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn account_crud_and_sealed_credentials() {
        let s = store().await;
        let id = seed_account(&s).await;

        let got = s.get_account(&id).await.unwrap();
        assert_eq!(got.kind, AccountKind::Imap);
        assert_eq!(got.host, "imap.example.org");
        assert_eq!(got.port, 993);
        assert_eq!(got.username, "imap-user");

        // Credentials open only through the store's key.
        assert_eq!(s.account_credentials(&id).await.unwrap(), creds());

        s.update_sync_policy(&id, r#"{"keep":false}"#)
            .await
            .unwrap();
        assert_eq!(
            s.get_account(&id).await.unwrap().sync_policy_json,
            r#"{"keep":false}"#
        );

        assert_eq!(s.list_accounts().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn account_password_not_in_plaintext_at_rest() {
        let s = store().await;
        let id = seed_account(&s).await;
        let sealed: Vec<u8> = sqlx::query_scalar("SELECT sealed_creds FROM accounts WHERE id = ?1")
            .bind(&id)
            .fetch_one(&s.pool)
            .await
            .unwrap();
        assert!(!sealed.windows(7).any(|w| w == b"hunter2"));
    }

    #[tokio::test]
    async fn mailbox_upsert_is_idempotent_by_key() {
        let s = store().await;
        let account_id = seed_account(&s).await;
        let id1 = seed_mailbox(&s, &account_id, "INBOX", 100).await;

        // Same (account, name, uidvalidity) → same id, refreshed counts.
        let id2 = s
            .upsert_mailbox(&MailboxUpsert {
                account_id: &account_id,
                name: "INBOX",
                role: Some("inbox"),
                uidvalidity: 100,
                uidnext: 42,
                highestmodseq: 9,
                total: 5,
                unread: 2,
                parent_id: None,
            })
            .await
            .unwrap();
        assert_eq!(id1, id2);

        let mb = s.get_mailbox(&id1).await.unwrap();
        assert_eq!(mb.uidnext, 42);
        assert_eq!(mb.highestmodseq, 9);
        assert_eq!(mb.total, 5);
        assert_eq!(mb.unread, 2);

        s.update_mailbox_counts(&id1, 50, 12, 6, 1).await.unwrap();
        let mb = s.get_mailbox(&id1).await.unwrap();
        assert_eq!(
            (mb.uidnext, mb.highestmodseq, mb.total, mb.unread),
            (50, 12, 6, 1)
        );

        s.set_mailbox_role(&id1, None).await.unwrap();
        assert_eq!(s.get_mailbox(&id1).await.unwrap().role, None);

        assert_eq!(s.list_mailboxes(&account_id).await.unwrap().len(), 1);
    }

    fn msg<'a>(
        account_id: &'a str,
        mailbox_id: &'a str,
        uid: u32,
        uidvalidity: u32,
        message_id: &'a str,
        internaldate: &'a str,
    ) -> MessageUpsert<'a> {
        MessageUpsert {
            account_id,
            mailbox_id,
            uid,
            uidvalidity,
            message_id: Some(message_id),
            thread_id: None,
            internaldate: Some(internaldate),
            size: 1024,
            flags_json: r#"["Seen"]"#,
            envelope: None,
            blob_ref: None,
        }
    }

    #[tokio::test]
    async fn message_stable_id_is_allocated_once_and_preserved() {
        let s = store().await;
        let account_id = seed_account(&s).await;
        let mailbox_id = seed_mailbox(&s, &account_id, "INBOX", 100).await;

        let m = msg(
            &account_id,
            &mailbox_id,
            5,
            100,
            "<a@x>",
            "2026-07-01T10:00:00Z",
        );
        let id1 = s.upsert_message(&m).await.unwrap();
        // Re-ingesting the exact same coordinates keeps the id.
        let id2 = s.upsert_message(&m).await.unwrap();
        assert_eq!(id1, id2);

        // Forward + reverse map round-trip.
        assert_eq!(
            s.stable_id_for(&account_id, &mailbox_id, 100, 5)
                .await
                .unwrap(),
            Some(id1.clone())
        );
        let loc = s.message_location(&id1).await.unwrap().unwrap();
        assert_eq!(
            loc,
            MessageLocation {
                mailbox_id: mailbox_id.clone(),
                uidvalidity: 100,
                uid: 5
            }
        );
    }

    #[tokio::test]
    async fn stable_id_survives_uidvalidity_change() {
        let s = store().await;
        let account_id = seed_account(&s).await;
        let mailbox_id = seed_mailbox(&s, &account_id, "INBOX", 100).await;

        let id_before = s
            .upsert_message(&msg(
                &account_id,
                &mailbox_id,
                5,
                100,
                "<keep@x>",
                "2026-07-01T10:00:00Z",
            ))
            .await
            .unwrap();

        // Server reports a new UIDVALIDITY: re-key the mailbox, then re-sync.
        s.revalidate_mailbox(&mailbox_id, 200).await.unwrap();
        assert_eq!(s.get_mailbox(&mailbox_id).await.unwrap().uidvalidity, 200);
        assert!(
            s.load_cursor(&account_id, &mailbox_id)
                .await
                .unwrap()
                .is_none()
        );

        // Same message reappears under new (uidvalidity, uid); identity match
        // (message-id + internaldate + size) carries the stable id.
        let id_after = s
            .upsert_message(&msg(
                &account_id,
                &mailbox_id,
                9,
                200,
                "<keep@x>",
                "2026-07-01T10:00:00Z",
            ))
            .await
            .unwrap();
        assert_eq!(id_before, id_after);

        // And the row now lives at the new coordinates.
        let loc = s.message_location(&id_after).await.unwrap().unwrap();
        assert_eq!((loc.uidvalidity, loc.uid), (200, 9));
        // The old coordinates no longer resolve.
        assert_eq!(
            s.stable_id_for(&account_id, &mailbox_id, 100, 5)
                .await
                .unwrap(),
            None
        );
    }

    #[tokio::test]
    async fn distinct_messages_get_distinct_ids() {
        let s = store().await;
        let account_id = seed_account(&s).await;
        let mailbox_id = seed_mailbox(&s, &account_id, "INBOX", 100).await;
        let a = s
            .upsert_message(&msg(
                &account_id,
                &mailbox_id,
                1,
                100,
                "<a@x>",
                "2026-07-01T10:00:00Z",
            ))
            .await
            .unwrap();
        let b = s
            .upsert_message(&msg(
                &account_id,
                &mailbox_id,
                2,
                100,
                "<b@x>",
                "2026-07-02T10:00:00Z",
            ))
            .await
            .unwrap();
        assert_ne!(a, b);
    }

    #[tokio::test]
    async fn list_message_ids_sorted_newest_first_with_paging() {
        let s = store().await;
        let account_id = seed_account(&s).await;
        let mailbox_id = seed_mailbox(&s, &account_id, "INBOX", 100).await;

        let old = s
            .upsert_message(&msg(
                &account_id,
                &mailbox_id,
                1,
                100,
                "<1@x>",
                "2026-07-01T00:00:00Z",
            ))
            .await
            .unwrap();
        let mid = s
            .upsert_message(&msg(
                &account_id,
                &mailbox_id,
                2,
                100,
                "<2@x>",
                "2026-07-05T00:00:00Z",
            ))
            .await
            .unwrap();
        let new = s
            .upsert_message(&msg(
                &account_id,
                &mailbox_id,
                3,
                100,
                "<3@x>",
                "2026-07-09T00:00:00Z",
            ))
            .await
            .unwrap();

        let all = s.list_message_ids(&mailbox_id, 100, 0).await.unwrap();
        assert_eq!(all, vec![new.clone(), mid.clone(), old.clone()]);

        // Paging: limit 1, offset 1 → the middle one.
        assert_eq!(
            s.list_message_ids(&mailbox_id, 1, 1).await.unwrap(),
            vec![mid]
        );
    }

    #[tokio::test]
    async fn flags_thread_and_delete() {
        let s = store().await;
        let account_id = seed_account(&s).await;
        let mailbox_id = seed_mailbox(&s, &account_id, "INBOX", 100).await;
        let id = s
            .upsert_message(&msg(
                &account_id,
                &mailbox_id,
                7,
                100,
                "<f@x>",
                "2026-07-01T10:00:00Z",
            ))
            .await
            .unwrap();

        s.set_flags(&id, r#"["Seen","Flagged"]"#).await.unwrap();
        assert_eq!(
            s.get_message(&id).await.unwrap().flags_json,
            r#"["Seen","Flagged"]"#
        );
        assert!(matches!(
            s.set_flags("nope", "[]").await,
            Err(StoreError::NotFound)
        ));

        let thread_id = s.assign_thread(&account_id, "<f@x>").await.unwrap();
        s.set_thread(&id, &thread_id).await.unwrap();
        assert_eq!(
            s.get_message(&id).await.unwrap().thread_id.as_deref(),
            Some(thread_id.as_str())
        );

        s.delete_message(&id).await.unwrap();
        assert!(matches!(
            s.get_message(&id).await,
            Err(StoreError::NotFound)
        ));
    }

    #[tokio::test]
    async fn bodies_and_envelope_are_sealed_at_rest() {
        let s = store().await;
        let account_id = seed_account(&s).await;
        let mailbox_id = seed_mailbox(&s, &account_id, "INBOX", 100).await;

        let blob_ref = s
            .put_body(&account_id, b"From: a\r\n\r\nsecret-body")
            .await
            .unwrap();
        assert_eq!(
            s.get_body(&blob_ref).await.unwrap().unwrap(),
            b"From: a\r\n\r\nsecret-body"
        );
        assert!(s.get_body("missing").await.unwrap().is_none());

        // Raw blob is ciphertext, not the plaintext body.
        let raw: Vec<u8> =
            sqlx::query_scalar("SELECT sealed_bytes FROM bodies WHERE blob_ref = ?1")
                .bind(&blob_ref)
                .fetch_one(&s.pool)
                .await
                .unwrap();
        assert!(!raw.windows(11).any(|w| w == b"secret-body"));

        let mut m = msg(
            &account_id,
            &mailbox_id,
            3,
            100,
            "<e@x>",
            "2026-07-01T10:00:00Z",
        );
        let env = br#"{"subject":"private-subject"}"#;
        m.envelope = Some(env);
        m.blob_ref = Some(&blob_ref);
        let id = s.upsert_message(&m).await.unwrap();

        assert_eq!(s.get_envelope(&id).await.unwrap().unwrap(), env);
        assert_eq!(
            s.get_message(&id).await.unwrap().blob_ref.as_deref(),
            Some(blob_ref.as_str())
        );

        let sealed_env: Option<Vec<u8>> =
            sqlx::query_scalar("SELECT envelope_json FROM messages WHERE stable_id = ?1")
                .bind(&id)
                .fetch_one(&s.pool)
                .await
                .unwrap();
        assert!(
            !sealed_env
                .unwrap()
                .windows(15)
                .any(|w| w == b"private-subject")
        );
    }

    #[tokio::test]
    async fn threads_assign_is_idempotent() {
        let s = store().await;
        let account_id = seed_account(&s).await;
        let t1 = s.assign_thread(&account_id, "<root@x>").await.unwrap();
        let t2 = s.assign_thread(&account_id, "<root@x>").await.unwrap();
        assert_eq!(t1, t2);
        assert_eq!(
            s.thread_for_root(&account_id, "<root@x>").await.unwrap(),
            Some(t1)
        );
        assert_eq!(
            s.thread_for_root(&account_id, "<other@x>").await.unwrap(),
            None
        );
    }

    #[tokio::test]
    async fn pop3_uidl_set_tracking() {
        let s = store().await;
        let account_id = s
            .create_account(
                &NewAccount {
                    kind: AccountKind::Pop3,
                    host: "pop.example.org",
                    port: 995,
                    tls: "implicit",
                    username: "pop-user",
                    sync_policy_json: "{}",
                },
                &creds(),
            )
            .await
            .unwrap();

        s.record_uidl(&account_id, "UID-A", "stable-a")
            .await
            .unwrap();
        s.record_uidl(&account_id, "UID-B", "stable-b")
            .await
            .unwrap();
        // Re-record updates the mapping without duplicating.
        s.record_uidl(&account_id, "UID-A", "stable-a")
            .await
            .unwrap();

        let seen = s.seen_uidls(&account_id).await.unwrap();
        assert_eq!(seen.len(), 2);
        assert!(seen.contains("UID-A") && seen.contains("UID-B"));
        assert_eq!(
            s.stable_id_for_uidl(&account_id, "UID-B")
                .await
                .unwrap()
                .as_deref(),
            Some("stable-b")
        );
        assert_eq!(
            s.stable_id_for_uidl(&account_id, "UID-Z").await.unwrap(),
            None
        );
    }

    #[tokio::test]
    async fn sync_cursor_round_trip_is_opaque() {
        let s = store().await;
        let account_id = seed_account(&s).await;
        let mailbox_id = seed_mailbox(&s, &account_id, "INBOX", 100).await;

        assert!(
            s.load_cursor(&account_id, &mailbox_id)
                .await
                .unwrap()
                .is_none()
        );
        let cursor = r#"{"kind":"qresync","uidvalidity":100,"highestmodseq":42}"#;
        s.save_cursor(&account_id, &mailbox_id, cursor)
            .await
            .unwrap();
        assert_eq!(
            s.load_cursor(&account_id, &mailbox_id)
                .await
                .unwrap()
                .as_deref(),
            Some(cursor)
        );
        // Overwrite.
        let cursor2 = r#"{"kind":"condstore","uidvalidity":100,"modseq":99}"#;
        s.save_cursor(&account_id, &mailbox_id, cursor2)
            .await
            .unwrap();
        assert_eq!(
            s.load_cursor(&account_id, &mailbox_id)
                .await
                .unwrap()
                .as_deref(),
            Some(cursor2)
        );
    }
}
