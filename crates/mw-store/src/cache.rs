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

use std::collections::{BTreeSet, HashMap, HashSet};

use chrono::Utc;

use crate::backend::Dialect;
use crate::{Row, Store, StoreError, q, seal};

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

// ---- batch-lookup plumbing (26.20 t22-e1) ----------------------------------
//
// A batch getter takes N ids and must issue ONE statement, not N. The query
// layer authors SQL as `&'static str` (see `backend::q`), so an `IN (?1, …, ?n)`
// list — whose text depends on N — cannot be built at all, and a per-arity
// pre-generated table would just move the loop. Instead the whole id list
// crosses as ONE text parameter holding a JSON array, and the database expands
// it back into rows:
//
//   SQLite   `json_each(?1)`                     → column `value`
//   Postgres `json_array_elements_text(?1::json)` → column `value`
//
// Both are core, always-available functions (SQLite has had JSON1 compiled in by
// default since 3.38 and sqlx bundles far newer; `json_array_elements_text` has
// been in Postgres since 9.3), so this adds no dependency and no extension. The
// two strings are the only per-dialect divergence, selected by
// `Backend::dialect()` exactly like `autocomplete_contacts` selects LIKE/ILIKE.
//
// Why not a delimited string with `string_to_array`: ids reach these methods
// straight from a JMAP request body, so a caller-supplied id containing the
// delimiter would silently split into two lookups. JSON encodes the boundary.

/// Encode a batch of ids as the single JSON-array text parameter the batch
/// getters bind (see the module note above).
pub(crate) fn id_list_json(ids: &[String]) -> String {
    serde_json::to_string(ids).expect("a Vec<String> always serializes")
}

/// Index a batch result set by its `stable_id` column, then project it back onto
/// the caller's id order.
///
/// Returning `Vec<Option<T>>` positionally — rather than a map, or a `Vec<T>` of
/// whatever the database happened to return — is what makes a batch getter a
/// drop-in for the loop it replaces: element `i` is exactly what
/// `get_x(&ids[i])` would have produced, holes included, and a repeated id
/// yields the same value at each of its positions.
pub(crate) fn project_in_order<T: Clone>(
    ids: &[String],
    found: HashMap<String, T>,
) -> Vec<Option<T>> {
    ids.iter().map(|id| found.get(id).cloned()).collect()
}

/// Whether an opaque `flags_json` value denotes an UNREAD message.
///
/// **This is the one place `mw-store` looks inside engine-owned JSON**, and it is
/// a deliberate, narrow exception to the seam rule at the top of this module
/// (t22 OQ-13). The alternative — recomputing `unread` with a `COUNT(*)` per
/// query — puts a full mailbox scan back on precisely the path 26.20 exists to
/// bound, and the counter has to be maintained by whoever performs the write.
///
/// The contract is exact and deliberately narrow: a message is read iff its
/// flags array contains the JSON **string** `"Seen"`, which is what
/// `mw_engine::mapping::flags_to_json` emits for `Flag::Seen` (serde's default
/// enum encoding; a custom keyword is an object, `{"Keyword":"$Junk"}`, and can
/// never collide). Anything that does not parse as a JSON array counts as
/// unread, mirroring `flags_from_json`'s `unwrap_or_default` — a corrupt value
/// must not fail a flag write.
fn flags_json_is_unread(flags_json: &str) -> bool {
    !serde_json::from_str::<Vec<serde_json::Value>>(flags_json)
        .map(|flags| flags.iter().any(|f| f.as_str() == Some("Seen")))
        .unwrap_or(false)
}

// Small casts: SQLite stores signed 64-bit integers; UIDs/counts are u32 and
// MODSEQ/size are u64 (well within i64 for any real mailbox).
fn to_i64(v: u64) -> i64 {
    v as i64
}
fn u32_from(row: &Row, col: &str) -> u32 {
    row.get_i64(col) as u32
}
fn u64_from(row: &Row, col: &str) -> u64 {
    row.get_i64(col) as u64
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
        q(
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
        .execute(&self.backend)
        .await?;
        Ok(id)
    }

    /// Fetch an account by id.
    pub async fn get_account(&self, id: &str) -> Result<Account, StoreError> {
        let row = q(
            "SELECT id, kind, host, port, tls, username, sync_policy_json FROM accounts WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.backend)
        .await?
        .ok_or(StoreError::NotFound)?;
        Self::account_from_row(&row)
    }

    /// List all configured accounts (no credentials).
    pub async fn list_accounts(&self) -> Result<Vec<Account>, StoreError> {
        let rows = q(
            "SELECT id, kind, host, port, tls, username, sync_policy_json FROM accounts ORDER BY id",
        )
        .fetch_all(&self.backend)
        .await?;
        rows.iter().map(Self::account_from_row).collect()
    }

    /// Update an account's opaque sync-policy JSON.
    pub async fn update_sync_policy(&self, id: &str, policy_json: &str) -> Result<(), StoreError> {
        q("UPDATE accounts SET sync_policy_json = ?2 WHERE id = ?1")
            .bind(id)
            .bind(policy_json)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Open the sealed credentials for an account.
    pub async fn account_credentials(&self, id: &str) -> Result<crate::Credentials, StoreError> {
        let row = q("SELECT sealed_creds FROM accounts WHERE id = ?1")
            .bind(id)
            .fetch_optional(&self.backend)
            .await?
            .ok_or(StoreError::NotFound)?;
        crate::decode_creds(&self.key.open(&row.get_blob("sealed_creds"))?)
    }

    fn account_from_row(row: &Row) -> Result<Account, StoreError> {
        Ok(Account {
            id: row.get_string("id"),
            kind: AccountKind::parse(row.get_string("kind").as_str())?,
            host: row.get_string("host"),
            port: u32_from(row, "port") as u16,
            tls: row.get_string("tls"),
            username: row.get_string("username"),
            sync_policy_json: row.get_string("sync_policy_json"),
        })
    }

    // ---- mailboxes ------------------------------------------------------

    /// Upsert a mailbox by `(account, name, uidvalidity)`, returning its stable
    /// opaque id. Counts/uidnext/highestmodseq/role/parent are refreshed on
    /// conflict; the id is preserved.
    pub async fn upsert_mailbox(&self, m: &MailboxUpsert<'_>) -> Result<String, StoreError> {
        let id = seal::random_token();
        let row = q(
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
        .fetch_one(&self.backend)
        .await?;
        Ok(row.get_string("id"))
    }

    /// List an account's mailboxes ordered by name.
    pub async fn list_mailboxes(&self, account_id: &str) -> Result<Vec<Mailbox>, StoreError> {
        let rows = q(
            "SELECT id, account_id, name, role, uidvalidity, uidnext, highestmodseq, total, unread, parent_id
             FROM mailboxes WHERE account_id = ?1 ORDER BY name",
        )
        .bind(account_id)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(Self::mailbox_from_row).collect())
    }

    /// Fetch one mailbox by id.
    pub async fn get_mailbox(&self, id: &str) -> Result<Mailbox, StoreError> {
        let row = q(
            "SELECT id, account_id, name, role, uidvalidity, uidnext, highestmodseq, total, unread, parent_id
             FROM mailboxes WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.backend)
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
        q(
            "UPDATE mailboxes SET uidnext = ?2, highestmodseq = ?3, total = ?4, unread = ?5 WHERE id = ?1",
        )
        .bind(id)
        .bind(uidnext as i64)
        .bind(to_i64(highestmodseq))
        .bind(total as i64)
        .bind(unread as i64)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Set (or clear) a mailbox's special-use role string.
    pub async fn set_mailbox_role(&self, id: &str, role: Option<&str>) -> Result<(), StoreError> {
        q("UPDATE mailboxes SET role = ?2 WHERE id = ?1")
            .bind(id)
            .bind(role)
            .execute(&self.backend)
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
        let mut tx = self.backend.begin().await?;
        q("UPDATE mailboxes SET uidvalidity = ?2, uidnext = 0, highestmodseq = 0 WHERE id = ?1")
            .bind(mailbox_id)
            .bind(new_uidvalidity as i64)
            .execute_tx(&mut tx)
            .await?;
        q("DELETE FROM sync_state WHERE mailbox_id = ?1")
            .bind(mailbox_id)
            .execute_tx(&mut tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    fn mailbox_from_row(row: &Row) -> Mailbox {
        Mailbox {
            id: row.get_string("id"),
            account_id: row.get_string("account_id"),
            name: row.get_string("name"),
            role: row.get_opt_string("role"),
            uidvalidity: u32_from(row, "uidvalidity"),
            uidnext: u32_from(row, "uidnext"),
            highestmodseq: u64_from(row, "highestmodseq"),
            total: u32_from(row, "total"),
            unread: u32_from(row, "unread"),
            parent_id: row.get_opt_string("parent_id"),
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
        let mut tx = self.backend.begin().await?;

        // (1) exact UID coordinates.
        let existing: Option<String> = q("SELECT stable_id FROM messages
             WHERE account_id = ?1 AND mailbox_id = ?2 AND uidvalidity = ?3 AND uid = ?4")
        .bind(m.account_id)
        .bind(m.mailbox_id)
        .bind(m.uidvalidity as i64)
        .bind(m.uid as i64)
        .fetch_opt_scalar_string_tx(&mut tx)
        .await?;

        // (2) identity match across a UIDVALIDITY change.
        let stable_id = match existing {
            Some(id) => id,
            None => {
                let identity: Option<String> = match (m.message_id, m.internaldate) {
                    (Some(mid), Some(date)) => {
                        q("SELECT stable_id FROM messages
                         WHERE account_id = ?1 AND mailbox_id = ?2 AND message_id = ?3
                           AND internaldate = ?4 AND size = ?5
                         LIMIT 1")
                        .bind(m.account_id)
                        .bind(m.mailbox_id)
                        .bind(mid)
                        .bind(date)
                        .bind(to_i64(m.size))
                        .fetch_opt_scalar_string_tx(&mut tx)
                        .await?
                    }
                    _ => None,
                };
                identity.unwrap_or_else(seal::random_token)
            }
        };

        // Upsert the full row under the resolved stable_id.
        q("INSERT INTO messages
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
                 blob_ref = COALESCE(excluded.blob_ref, messages.blob_ref)")
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
        .execute_tx(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(stable_id)
    }

    /// Fetch a message row by stable id.
    pub async fn get_message(&self, stable_id: &str) -> Result<Message, StoreError> {
        let row = q(
            "SELECT stable_id, account_id, mailbox_id, uid, uidvalidity, message_id, thread_id,
                    internaldate, size, flags_json, blob_ref
             FROM messages WHERE stable_id = ?1",
        )
        .bind(stable_id)
        .fetch_optional(&self.backend)
        .await?
        .ok_or(StoreError::NotFound)?;
        Ok(Self::message_from_row(&row))
    }

    /// Fetch a whole page of message rows in **one** statement (26.20 t22-e1).
    ///
    /// The batch form of [`Store::get_message`]: the returned vector is the same
    /// length as `stable_ids` and positionally aligned with it, where `None` is
    /// the id that [`Store::get_message`] would have answered with
    /// [`StoreError::NotFound`]. Ordering, duplicates and holes are all
    /// preserved, so `get_messages(ids)` is interchangeable with
    /// `ids.map(get_message)` as a *sequence*.
    ///
    /// `Email/get` for a 50-id page previously issued 50 of these round trips
    /// (150 store statements across the three getters, inside 163 for the
    /// request). It issues one here, whatever the page size. That is invisible on
    /// SQLite and worth ~12× on Postgres, where every one of those awaits was a
    /// network round trip.
    ///
    /// An empty slice issues **no** statement.
    pub async fn get_messages(
        &self,
        stable_ids: &[String],
    ) -> Result<Vec<Option<Message>>, StoreError> {
        if stable_ids.is_empty() {
            return Ok(Vec::new());
        }
        let sql = match self.backend.dialect() {
            Dialect::Sqlite => {
                "SELECT stable_id, account_id, mailbox_id, uid, uidvalidity, message_id, thread_id,
                        internaldate, size, flags_json, blob_ref
                 FROM messages WHERE stable_id IN (SELECT value FROM json_each(?1))"
            }
            Dialect::Postgres => {
                "SELECT stable_id, account_id, mailbox_id, uid, uidvalidity, message_id, thread_id,
                        internaldate, size, flags_json, blob_ref
                 FROM messages
                 WHERE stable_id IN (SELECT value FROM json_array_elements_text(?1::json))"
            }
        };
        let rows = q(sql)
            .bind(id_list_json(stable_ids))
            .fetch_all(&self.backend)
            .await?;
        let found: HashMap<String, Message> = rows
            .iter()
            .map(|r| {
                let m = Self::message_from_row(r);
                (m.stable_id.clone(), m)
            })
            .collect();
        Ok(project_in_order(stable_ids, found))
    }

    /// Open a whole page of sealed envelopes in **one** statement (26.20 t22-e1).
    ///
    /// The batch form of [`Store::get_envelope`], positionally aligned with
    /// `stable_ids` exactly as [`Store::get_messages`] is. `None` covers **both**
    /// of the single getter's negative answers — no such message, and a message
    /// with no stored envelope — because every caller (`Engine::build_email`,
    /// the snippet path) already collapses them: with no envelope bytes it falls
    /// back to re-parsing the sealed body. Callers that must tell the two apart
    /// have the row itself from [`Store::get_messages`], which is `None` only in
    /// the first case.
    ///
    /// Each envelope is opened under the store [`crate::ServerKey`] as it is
    /// mapped; a seal failure fails the whole batch rather than being silently
    /// projected to `None`.
    pub async fn get_envelopes(
        &self,
        stable_ids: &[String],
    ) -> Result<Vec<Option<Vec<u8>>>, StoreError> {
        if stable_ids.is_empty() {
            return Ok(Vec::new());
        }
        let sql = match self.backend.dialect() {
            Dialect::Sqlite => {
                "SELECT stable_id, envelope_json FROM messages
                 WHERE stable_id IN (SELECT value FROM json_each(?1))"
            }
            Dialect::Postgres => {
                "SELECT stable_id, envelope_json FROM messages
                 WHERE stable_id IN (SELECT value FROM json_array_elements_text(?1::json))"
            }
        };
        let rows = q(sql)
            .bind(id_list_json(stable_ids))
            .fetch_all(&self.backend)
            .await?;
        let mut found: HashMap<String, Vec<u8>> = HashMap::with_capacity(rows.len());
        for r in &rows {
            if let Some(sealed) = r.get_opt_blob("envelope_json") {
                found.insert(r.get_string("stable_id"), self.key.open(&sealed)?);
            }
        }
        Ok(project_in_order(stable_ids, found))
    }

    /// How many messages a mailbox holds (`Mailbox.totalEmails`, and the
    /// `calculateTotal` half of `Email/query`).
    ///
    /// An exact `COUNT(*)`, answered from an index without touching the table on
    /// both backends — and on both it is the **two-column**
    /// `idx_messages_mailbox_date` from 0002 that gets chosen, not 0023's wider
    /// covering index: a count needs no ordering, so the narrower index wins.
    /// Measured with and without 0023 present: Postgres `Index Only Scan`,
    /// 4.2 ms at 20 000; SQLite `USING COVERING INDEX`, 17 ms at 20 000 and
    /// 143 ms at 200 000.
    ///
    /// Exact rather than estimated, and once per query rather than per page: at
    /// those costs it is over budget for every page and comfortable for a
    /// `calculateTotal` the client asks for (t22 OQ-5). The alternative — a
    /// maintained total counter — is what `Mailbox.unread` was, and V8 is the
    /// record of how that ends when only some write paths maintain it.
    pub async fn count_messages_in_mailbox(&self, mailbox_id: &str) -> Result<u64, StoreError> {
        let n = q("SELECT COUNT(*) FROM messages WHERE mailbox_id = ?1")
            .bind(mailbox_id)
            .fetch_scalar_i64(&self.backend)
            .await?;
        Ok(n.max(0) as u64)
    }

    /// Map backend coordinates → stable id.
    pub async fn stable_id_for(
        &self,
        account_id: &str,
        mailbox_id: &str,
        uidvalidity: u32,
        uid: u32,
    ) -> Result<Option<String>, StoreError> {
        Ok(q("SELECT stable_id FROM messages
             WHERE account_id = ?1 AND mailbox_id = ?2 AND uidvalidity = ?3 AND uid = ?4")
        .bind(account_id)
        .bind(mailbox_id)
        .bind(uidvalidity as i64)
        .bind(uid as i64)
        .fetch_opt_scalar_string(&self.backend)
        .await?)
    }

    /// Map stable id → backend coordinates (mailbox + UIDVALIDITY + UID).
    pub async fn message_location(
        &self,
        stable_id: &str,
    ) -> Result<Option<MessageLocation>, StoreError> {
        let row = q("SELECT mailbox_id, uidvalidity, uid FROM messages WHERE stable_id = ?1")
            .bind(stable_id)
            .fetch_optional(&self.backend)
            .await?;
        Ok(row.map(|r| MessageLocation {
            mailbox_id: r.get_string("mailbox_id"),
            uidvalidity: u32_from(&r, "uidvalidity"),
            uid: u32_from(&r, "uid"),
        }))
    }

    /// Stable ids in a mailbox for `Email/query`, newest first (INTERNALDATE
    /// desc, UID desc, then `stable_id` as a total-order tie-break), with
    /// `limit`/`offset` paging.
    ///
    /// The sort is **total** (26.20 t22-e1): `(internaldate, uid)` is unique in
    /// every mailbox a real server produces, but nothing enforces it, and under
    /// `OFFSET` paging a non-deterministic order between two ties can show one
    /// row on two pages and drop another entirely. `stable_id` last matches
    /// `idx_messages_mailbox_page` (0023) column for column, so the total order
    /// costs no sort.
    ///
    /// A non-positive `limit` returns empty **without issuing a statement**: the
    /// paged callers clamp a client's `limit` and a clamp that lands on 0 should
    /// cost nothing, not a round trip that can only return nothing. A negative
    /// `offset` is clamped to 0 (SQLite treats a negative OFFSET as 0, Postgres
    /// rejects it — the clamp makes the two agree).
    pub async fn list_message_ids(
        &self,
        mailbox_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<String>, StoreError> {
        if limit <= 0 {
            return Ok(Vec::new());
        }
        Ok(q("SELECT stable_id FROM messages WHERE mailbox_id = ?1
             ORDER BY internaldate DESC, uid DESC, stable_id
             LIMIT ?2 OFFSET ?3")
        .bind(mailbox_id)
        .bind(limit)
        .bind(offset.max(0))
        .fetch_all_scalar_string(&self.backend)
        .await?)
    }

    /// Replace a message's opaque flags JSON (server-authoritative, SPEC §15.2)
    /// **and keep the mailbox's stored `unread` counter true** (26.20 t22-e1,
    /// finding V8).
    ///
    /// Before this, `mailboxes.unread` was only ever written by a sync
    /// (`upsert_mailbox`/`update_mailbox_counts`, from the server's own numbers).
    /// A local `Email/set` that marked messages read moved no counter at all:
    /// measured, 551 successful `$seen` sets over a 20 000-message folder left
    /// `Mailbox/get` still reporting `unreadEmails: 20000`. Every unread badge
    /// and every "1–N of M" built on that field was wrong from the first message
    /// a user read until the next full sync.
    ///
    /// **The invariant lives here, in the write path**, rather than in a
    /// `COUNT(*)` at read time (t22 OQ-13): recounting puts a full mailbox scan
    /// back on exactly the path this tag exists to bound, and it would have to be
    /// paid by every reader instead of by the one writer that changed something.
    /// The same reasoning applies to [`Store::relocate_message`] and
    /// [`Store::delete_message`], which maintain it too — a counter that only one
    /// of the three paths maintains is still wrong, just less often.
    ///
    /// **Cost: two statements, three when the read/unread state flips**, and
    /// deliberately no transaction. `Email/set` over a large selection calls this
    /// once per message, so a `BEGIN`/`COMMIT` pair here would add two Postgres
    /// round trips per id to the very operation 26.20 is trying to make cheap —
    /// and a store transaction taken per message would contend with any batch
    /// transaction the caller holds, which on SQLite means a writer waiting out
    /// `busy_timeout` against itself. Each statement is individually atomic and
    /// the counter statement is a self-contained `unread = unread ± 1`, never a
    /// read-modify-write from Rust. What that leaves is: a crash between the two
    /// statements, or two writers flipping the *same* message concurrently, can
    /// drift the counter by one until the next sync overwrites it. The
    /// decrement's `unread > 0` guard makes drift-to-negative unrepresentable,
    /// and the value it replaces was not stale-by-one but stale-by-everything.
    pub async fn set_flags(&self, stable_id: &str, flags_json: &str) -> Result<(), StoreError> {
        // The stored row first: which mailbox's counter this message contributes
        // to, and whether it is contributing right now. Read before the write —
        // afterwards the old value is gone.
        let row = q("SELECT mailbox_id, flags_json FROM messages WHERE stable_id = ?1")
            .bind(stable_id)
            .fetch_optional(&self.backend)
            .await?
            .ok_or(StoreError::NotFound)?;
        let mailbox_id = row.get_string("mailbox_id");
        let was_unread = flags_json_is_unread(&row.get_string("flags_json"));

        let n = q("UPDATE messages SET flags_json = ?2 WHERE stable_id = ?1")
            .bind(stable_id)
            .bind(flags_json)
            .execute(&self.backend)
            .await?;
        if n == 0 {
            // Deleted between the read and the write; the counter statement below
            // would be wrong, so skip it.
            return Err(StoreError::NotFound);
        }

        // Third statement only when the state actually flipped: re-setting the
        // same keywords, or changing an unrelated one (`\Flagged`), must not move
        // the counter.
        match (was_unread, flags_json_is_unread(flags_json)) {
            (true, false) => self.decrement_unread(&mailbox_id).await?,
            (false, true) => self.increment_unread(&mailbox_id).await?,
            _ => {}
        }
        Ok(())
    }

    /// Apply a whole page of flag writes in a **bounded** number of statements,
    /// keeping every touched mailbox's `unread` counter true (26.20 t22-e1).
    ///
    /// `updates` is `(stable_id, flags_json)` pairs. The returned vector is
    /// positionally aligned with it: `true` where the id named a stored message
    /// and its flags were written, `false` where no such message exists — the
    /// batch equivalent of [`Store::set_flags`]'s [`StoreError::NotFound`], and
    /// what a caller needs to fill JMAP's `notUpdated`. A missing id is skipped,
    /// not an error, because one bad id in a 500-id `Email/set` must not discard
    /// the other 499.
    ///
    /// **Three statements for 500 ids** in one mailbox — a read of the old
    /// state, one JSON-joined `UPDATE`, and one counter update per mailbox
    /// actually affected — against 1 500 for the same work through
    /// [`Store::set_flags`] in a loop. That difference is the whole reason this
    /// exists: `Email/set` over a large selection is the single most expensive
    /// measured operation in this tag, and per-message store round trips are the
    /// part of it that lives on this side of the seam.
    ///
    /// Duplicate ids resolve **last-wins**, decided in Rust before the statement
    /// is built rather than left to whichever row the JSON join happens to
    /// match, so the result does not depend on the database's evaluation order.
    ///
    /// Same transactional posture as [`Store::set_flags`], for the same reasons
    /// argued there: the statements are individually atomic and are not wrapped
    /// in a transaction. The window is smaller here — three statements per page
    /// rather than three per message.
    pub async fn set_flags_batch(
        &self,
        updates: &[(String, String)],
    ) -> Result<Vec<bool>, StoreError> {
        if updates.is_empty() {
            return Ok(Vec::new());
        }
        // Last-wins, and the ids in request order for the id-list parameter.
        let mut desired: HashMap<&str, &str> = HashMap::with_capacity(updates.len());
        for (id, flags) in updates {
            desired.insert(id.as_str(), flags.as_str());
        }
        let ids: Vec<String> = desired.keys().map(|id| (*id).to_string()).collect();

        // (1) the stored state of every id: which mailbox it counts towards and
        // whether it is counting right now.
        let sql = match self.backend.dialect() {
            Dialect::Sqlite => {
                "SELECT stable_id, mailbox_id, flags_json FROM messages
                 WHERE stable_id IN (SELECT value FROM json_each(?1))"
            }
            Dialect::Postgres => {
                "SELECT stable_id, mailbox_id, flags_json FROM messages
                 WHERE stable_id IN (SELECT value FROM json_array_elements_text(?1::json))"
            }
        };
        let rows = q(sql)
            .bind(id_list_json(&ids))
            .fetch_all(&self.backend)
            .await?;

        // Net unread delta per mailbox, and which ids actually exist.
        let mut delta: HashMap<String, i64> = HashMap::new();
        let mut present: HashSet<String> = HashSet::with_capacity(rows.len());
        for r in &rows {
            let stable_id = r.get_string("stable_id");
            let Some(flags) = desired.get(stable_id.as_str()) else {
                continue;
            };
            let was_unread = flags_json_is_unread(&r.get_string("flags_json"));
            match (was_unread, flags_json_is_unread(flags)) {
                (true, false) => *delta.entry(r.get_string("mailbox_id")).or_default() -= 1,
                (false, true) => *delta.entry(r.get_string("mailbox_id")).or_default() += 1,
                _ => {}
            }
            present.insert(stable_id);
        }
        if present.is_empty() {
            return Ok(updates.iter().map(|_| false).collect());
        }

        // (2) one UPDATE for the whole page, joined against the pairs encoded as
        // a JSON array of two-element arrays. Reverse order plus a first-seen
        // filter IS the last-wins rule, applied here rather than left to the
        // join. The JSON is ours, not stored data, so it is always well formed.
        let mut emitted: HashSet<&str> = HashSet::with_capacity(present.len());
        let mut pairs: Vec<[&str; 2]> = Vec::with_capacity(present.len());
        for (id, flags) in updates.iter().rev() {
            if present.contains(id.as_str()) && emitted.insert(id.as_str()) {
                pairs.push([id.as_str(), flags.as_str()]);
            }
        }
        let payload = serde_json::to_string(&pairs).expect("pairs always serialize");
        let sql = match self.backend.dialect() {
            // `UPDATE … FROM` (SQLite 3.33+, Postgres always) so the id list is
            // expanded ONCE. As a correlated subquery per row it would re-parse
            // the whole payload for every message in the page.
            Dialect::Sqlite => {
                "UPDATE messages SET flags_json = j.f
                 FROM (SELECT json_extract(value, '$[0]') AS id,
                              json_extract(value, '$[1]') AS f
                       FROM json_each(?1)) j
                 WHERE messages.stable_id = j.id"
            }
            Dialect::Postgres => {
                "UPDATE messages SET flags_json = j.f
                 FROM (SELECT x ->> 0 AS id, x ->> 1 AS f
                       FROM json_array_elements(?1::json) x) j
                 WHERE messages.stable_id = j.id"
            }
        };
        q(sql).bind(payload).execute(&self.backend).await?;

        // (3) one counter statement per mailbox that actually changed. The floor
        // is a `CASE` rather than `MAX`/`GREATEST`, which are not the same
        // function in the two dialects.
        for (mailbox_id, d) in delta {
            if d == 0 {
                continue;
            }
            q("UPDATE mailboxes
                 SET unread = CASE WHEN unread + ?2 < 0 THEN 0 ELSE unread + ?2 END
                 WHERE id = ?1")
            .bind(&mailbox_id)
            .bind(d)
            .execute(&self.backend)
            .await?;
        }

        Ok(updates
            .iter()
            .map(|(id, _)| present.contains(id.as_str()))
            .collect())
    }

    /// One message left a mailbox's unread population (read, moved out, deleted).
    ///
    /// The `unread > 0` guard is the invariant that matters more than the
    /// arithmetic: `unread` is read back as a `u32`, so a counter driven below
    /// zero by drift would wrap to ~4 billion in `Mailbox/get` rather than
    /// showing a small error.
    async fn decrement_unread(&self, mailbox_id: &str) -> Result<(), StoreError> {
        q("UPDATE mailboxes SET unread = unread - 1 WHERE id = ?1 AND unread > 0")
            .bind(mailbox_id)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// One message joined a mailbox's unread population (marked unread, moved in).
    async fn increment_unread(&self, mailbox_id: &str) -> Result<(), StoreError> {
        q("UPDATE mailboxes SET unread = unread + 1 WHERE id = ?1")
            .bind(mailbox_id)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Assign a message to a thread.
    pub async fn set_thread(&self, stable_id: &str, thread_id: &str) -> Result<(), StoreError> {
        q("UPDATE messages SET thread_id = ?2 WHERE stable_id = ?1")
            .bind(stable_id)
            .bind(thread_id)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Delete a cached message (EXPUNGE/VANISHED/dropped UIDL), along with any
    /// content-derived data keyed on its stable id.
    ///
    /// The A8 embedding (0022) is dropped here rather than at the three call sites
    /// that destroy messages today (the IMAP removal sweep, `Email/set` destroy, and
    /// the source drop after a cross-account move). Wiring those individually is the
    /// version that silently misses the fourth one somebody adds later; this is the
    /// single choke point they all pass through, so the guarantee does not depend on
    /// that list staying complete.
    ///
    /// **Embedding first, deliberately.** An embedding is a lossy but partially
    /// invertible projection of the message's text, so "the user deleted it" has to
    /// mean the vector is gone too. Dropping the `messages` row first would leave an
    /// orphan vector behind on a transient failure of the second statement — the exact
    /// retention defect this call exists to close. In this order a failure leaves the
    /// message intact and the caller retries; there is no state where the vector
    /// outlives the message.
    /// **Unread counter (26.20 t22-e1).** Deleting an unread message removes it
    /// from its mailbox's unread population, so the stored counter follows. The
    /// `DELETE` returns the row it removed (`RETURNING`, which both backends
    /// support and this module already relies on for `upsert_mailbox`), so the
    /// mailbox and the message's read state are known **from the delete itself**
    /// — no separate read to race with, and nothing is decremented unless a row
    /// really went. See [`Store::set_flags`] for why the counter is maintained in
    /// the write path at all.
    pub async fn delete_message(&self, stable_id: &str) -> Result<(), StoreError> {
        self.delete_message_embedding(stable_id).await?;
        let deleted = q("DELETE FROM messages WHERE stable_id = ?1
             RETURNING mailbox_id, flags_json")
        .bind(stable_id)
        .fetch_optional(&self.backend)
        .await?;
        // Deleting a message that is not there stays a no-op, not an error.
        if let Some(row) = deleted
            && flags_json_is_unread(&row.get_string("flags_json"))
        {
            self.decrement_unread(&row.get_string("mailbox_id")).await?;
        }
        Ok(())
    }

    /// Move a message to a new mailbox **preserving its `stable_id`** (plan
    /// §1.4). This is the single move path in V2: it updates the `messages` row
    /// in place at the new `(mailbox_id, uidvalidity, uid)`, so `message_meta`,
    /// tags, and the search index all stay keyed on the same id. Replaces the V1
    /// delete+reinsert in `Engine::move_email` (jmap.rs:435), which minted a new
    /// id and broke tag/meta/index keying.
    ///
    /// The single move path in V2: an in-place UPDATE that preserves the row's
    /// `stable_id`. `message_meta`, `tags`, and the engine's search index all key
    /// on that id, so they survive the move untouched (the engine re-keys the
    /// index's stored `mailboxId` separately). Returns [`StoreError::NotFound`]
    /// if no row carries `stable_id`.
    ///
    /// **Unread counter (26.20 t22-e1).** A move carries the message's unread
    /// contribution with it: an unread message leaving INBOX for Archive
    /// decrements INBOX and increments Archive, in that order, and a message that
    /// is already read moves no counter at all. Both counter statements run
    /// **before** the row is re-keyed, because afterwards the source mailbox is
    /// no longer recoverable from the row. If the re-key then finds nothing —
    /// the message was destroyed concurrently — the counters are put back rather
    /// than left leaning, since this method's contract is that a `NotFound`
    /// changed nothing. See [`Store::set_flags`] for why the invariant lives in
    /// the write path.
    pub async fn relocate_message(
        &self,
        stable_id: &str,
        new_mailbox_id: &str,
        new_uid: u32,
        new_uidvalidity: u32,
    ) -> Result<(), StoreError> {
        let current = q("SELECT mailbox_id, flags_json FROM messages WHERE stable_id = ?1")
            .bind(stable_id)
            .fetch_optional(&self.backend)
            .await?;
        let carries_unread = match &current {
            Some(row) => {
                let from = row.get_string("mailbox_id");
                from != new_mailbox_id && flags_json_is_unread(&row.get_string("flags_json"))
            }
            // No row: fall through to the UPDATE, which reports NotFound.
            None => false,
        };
        let from_mailbox = current.map(|row| row.get_string("mailbox_id"));
        if carries_unread {
            let from = from_mailbox.as_deref().expect("set with carries_unread");
            self.decrement_unread(from).await?;
            self.increment_unread(new_mailbox_id).await?;
        }

        let n = q(
            "UPDATE messages SET mailbox_id = ?2, uid = ?3, uidvalidity = ?4 WHERE stable_id = ?1",
        )
        .bind(stable_id)
        .bind(new_mailbox_id)
        .bind(new_uid as i64)
        .bind(new_uidvalidity as i64)
        .execute(&self.backend)
        .await?;
        if n == 0 {
            if carries_unread {
                let from = from_mailbox.as_deref().expect("set with carries_unread");
                self.decrement_unread(new_mailbox_id).await?;
                self.increment_unread(from).await?;
            }
            return Err(StoreError::NotFound);
        }
        Ok(())
    }

    fn message_from_row(row: &Row) -> Message {
        Message {
            stable_id: row.get_string("stable_id"),
            account_id: row.get_string("account_id"),
            mailbox_id: row.get_string("mailbox_id"),
            uid: u32_from(row, "uid"),
            uidvalidity: u32_from(row, "uidvalidity"),
            message_id: row.get_opt_string("message_id"),
            thread_id: row.get_opt_string("thread_id"),
            internaldate: row.get_opt_string("internaldate"),
            size: u64_from(row, "size"),
            flags_json: row.get_string("flags_json"),
            blob_ref: row.get_opt_string("blob_ref"),
        }
    }

    // ---- bodies (sealed at rest) ---------------------------------------

    /// Seal and store a raw/parsed body blob; returns its opaque `blob_ref`.
    pub async fn put_body(&self, account_id: &str, plaintext: &[u8]) -> Result<String, StoreError> {
        let blob_ref = seal::random_token();
        let sealed = self.key.seal(plaintext)?;
        q("INSERT INTO bodies (blob_ref, account_id, sealed_bytes) VALUES (?1, ?2, ?3)")
            .bind(&blob_ref)
            .bind(account_id)
            .bind(sealed)
            .execute(&self.backend)
            .await?;
        Ok(blob_ref)
    }

    /// Open a stored body blob.
    pub async fn get_body(&self, blob_ref: &str) -> Result<Option<Vec<u8>>, StoreError> {
        let row = q("SELECT sealed_bytes FROM bodies WHERE blob_ref = ?1")
            .bind(blob_ref)
            .fetch_optional(&self.backend)
            .await?;
        match row {
            Some(r) => Ok(Some(self.key.open(&r.get_blob("sealed_bytes"))?)),
            None => Ok(None),
        }
    }

    /// Open a message's sealed envelope bytes (for `Email/get` without
    /// re-parsing), if one was stored.
    pub async fn get_envelope(&self, stable_id: &str) -> Result<Option<Vec<u8>>, StoreError> {
        let row = q("SELECT envelope_json FROM messages WHERE stable_id = ?1")
            .bind(stable_id)
            .fetch_optional(&self.backend)
            .await?
            .ok_or(StoreError::NotFound)?;
        match row.get_opt_blob("envelope_json") {
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
        let row = q(
            "INSERT INTO threads (thread_id, account_id, root_message_id) VALUES (?1, ?2, ?3)
             ON CONFLICT(account_id, root_message_id) DO UPDATE SET root_message_id = excluded.root_message_id
             RETURNING thread_id",
        )
        .bind(&thread_id)
        .bind(account_id)
        .bind(root_message_id)
        .fetch_one(&self.backend)
        .await?;
        Ok(row.get_string("thread_id"))
    }

    /// Read-only: the JWZ root Message-ID already recorded for a stored message,
    /// looked up by that message's own Message-ID within an account (joins
    /// `messages` → `threads`). Used by the engine's incremental JWZ ingest to
    /// converge a newly-arriving reply onto the thread its referenced ancestors
    /// already belong to, without re-threading history (new-ingest-only). Keyed
    /// off the existing `message_id` column (indexed by `idx_messages_message_id`)
    /// so no migration is required. Returns `None` when no stored message carries
    /// that Message-ID or it is not yet threaded.
    pub async fn thread_root_for_message_id(
        &self,
        account_id: &str,
        message_id: &str,
    ) -> Result<Option<String>, StoreError> {
        Ok(q("SELECT t.root_message_id FROM messages m \
             JOIN threads t ON m.thread_id = t.thread_id \
             WHERE m.account_id = ?1 AND m.message_id = ?2 LIMIT 1")
        .bind(account_id)
        .bind(message_id)
        .fetch_opt_scalar_string(&self.backend)
        .await?)
    }

    /// Look up an existing thread id by its root Message-ID.
    pub async fn thread_for_root(
        &self,
        account_id: &str,
        root_message_id: &str,
    ) -> Result<Option<String>, StoreError> {
        Ok(
            q("SELECT thread_id FROM threads WHERE account_id = ?1 AND root_message_id = ?2")
                .bind(account_id)
                .bind(root_message_id)
                .fetch_opt_scalar_string(&self.backend)
                .await?,
        )
    }

    // ---- pop3 uidl ------------------------------------------------------

    /// Record that a POP3 UIDL has been ingested, mapped to a stable id.
    pub async fn record_uidl(
        &self,
        account_id: &str,
        uidl: &str,
        stable_id: &str,
    ) -> Result<(), StoreError> {
        q(
            "INSERT INTO pop3_uidl (account_id, uidl, stable_id) VALUES (?1, ?2, ?3)
             ON CONFLICT(account_id, uidl) DO UPDATE SET stable_id = excluded.stable_id",
        )
        .bind(account_id)
        .bind(uidl)
        .bind(stable_id)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// The set of UIDLs already ingested for an account (the POP3 sync cursor's
    /// `seen` set — feed it back to the backend to diff against LIST/UIDL).
    pub async fn seen_uidls(&self, account_id: &str) -> Result<BTreeSet<String>, StoreError> {
        let rows = q("SELECT uidl FROM pop3_uidl WHERE account_id = ?1")
            .bind(account_id)
            .fetch_all_scalar_string(&self.backend)
            .await?;
        Ok(rows.into_iter().collect())
    }

    /// Map a POP3 UIDL back to its stable id.
    pub async fn stable_id_for_uidl(
        &self,
        account_id: &str,
        uidl: &str,
    ) -> Result<Option<String>, StoreError> {
        Ok(
            q("SELECT stable_id FROM pop3_uidl WHERE account_id = ?1 AND uidl = ?2")
                .bind(account_id)
                .bind(uidl)
                .fetch_opt_scalar_string(&self.backend)
                .await?,
        )
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
        q(
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
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Load a mailbox's persisted sync cursor JSON, if any.
    pub async fn load_cursor(
        &self,
        account_id: &str,
        mailbox_id: &str,
    ) -> Result<Option<String>, StoreError> {
        Ok(
            q("SELECT cursor_json FROM sync_state WHERE account_id = ?1 AND mailbox_id = ?2")
                .bind(account_id)
                .bind(mailbox_id)
                .fetch_opt_scalar_string(&self.backend)
                .await?,
        )
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
        let sealed = q("SELECT sealed_creds FROM accounts WHERE id = ?1")
            .bind(&id)
            .fetch_one(s.backend())
            .await
            .unwrap()
            .get_blob("sealed_creds");
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

    /// 26.20 t22-e1. The batch getters must be interchangeable with the loop
    /// they replace **as a sequence**, not as a set: `Email/get` answers ids in
    /// the order the client asked for them, so a batch method that returns "the
    /// rows the database happened to produce" is a different method with the same
    /// name. The request order here is deliberately not insertion order and
    /// deliberately contains a repeat and a hole, both of which a set comparison
    /// would wave through.
    ///
    /// The body is shared with the live-Postgres leg below, because the SQL
    /// these methods issue is **per dialect** (`json_each` against
    /// `json_array_elements_text`): a contract asserted on SQLite alone says
    /// nothing about the statement Postgres actually runs.
    #[tokio::test]
    async fn batch_getters_equal_the_loop_they_replace_as_a_sequence() {
        assert_batch_getters_match_the_loop(&store().await).await;
    }

    async fn assert_batch_getters_match_the_loop(s: &Store) {
        let account_id = seed_account(s).await;
        let mailbox_id = seed_mailbox(&s, &account_id, "INBOX", 100).await;

        let mut ids = Vec::new();
        for (uid, date) in [
            (1, "2026-07-01T10:00:00Z"),
            (2, "2026-07-02T10:00:00Z"),
            (3, "2026-07-03T10:00:00Z"),
        ] {
            let mut m = msg(&account_id, &mailbox_id, uid, 100, "<b@x>", date);
            let env = format!(r#"{{"subject":"s{uid}"}}"#);
            // Only the middle message carries an envelope, so `None` in the
            // result covers both of its meanings.
            if uid == 2 {
                m.envelope = Some(env.as_bytes());
            }
            ids.push(s.upsert_message(&m).await.unwrap());
        }

        // Reverse insertion order, one id twice, one that was never stored.
        let asked: Vec<String> = vec![
            ids[2].clone(),
            ids[0].clone(),
            "no-such-message".into(),
            ids[0].clone(),
            ids[1].clone(),
        ];

        let mut one_by_one = Vec::new();
        for id in &asked {
            one_by_one.push(s.get_message(id).await.ok());
        }
        assert_eq!(s.get_messages(&asked).await.unwrap(), one_by_one);
        // Spelled out, in case the loop above ever stops being the oracle: the
        // answer tracks the REQUEST order, and the hole is positional.
        let batch = s.get_messages(&asked).await.unwrap();
        assert_eq!(batch[0].as_ref().unwrap().uid, 3);
        assert_eq!(batch[1].as_ref().unwrap().uid, 1);
        assert!(batch[2].is_none());
        assert_eq!(batch[3].as_ref().unwrap().uid, 1);
        assert_eq!(batch[4].as_ref().unwrap().uid, 2);

        let mut envs = Vec::new();
        for id in &asked {
            envs.push(s.get_envelope(id).await.ok().flatten());
        }
        assert_eq!(s.get_envelopes(&asked).await.unwrap(), envs);
        assert_eq!(
            s.get_envelopes(&asked).await.unwrap()[4].as_deref(),
            Some(br#"{"subject":"s2"}"#.as_slice()),
            "the one message with a sealed envelope opens through the batch path"
        );

        // The third getter lives in `v2.rs` but is the same contract and the same
        // per-dialect SQL, so it is asserted on whichever backend runs this.
        s.upsert_message_meta(
            &ids[0],
            &crate::StoredMeta {
                pinned: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        let mut metas = Vec::new();
        for id in &asked {
            metas.push(s.get_message_meta(id).await.unwrap());
        }
        assert_eq!(s.get_message_metas(&asked).await.unwrap(), metas);
        assert!(
            s.get_message_metas(&asked).await.unwrap()[1]
                .as_ref()
                .unwrap()
                .pinned
        );

        // An empty request is an empty answer and issues no statement.
        assert!(s.get_messages(&[]).await.unwrap().is_empty());
        assert!(s.get_envelopes(&[]).await.unwrap().is_empty());
        assert!(s.get_message_metas(&[]).await.unwrap().is_empty());

        assert_eq!(s.count_messages_in_mailbox(&mailbox_id).await.unwrap(), 3);
        assert_eq!(s.count_messages_in_mailbox("nope").await.unwrap(), 0);
    }

    /// 26.20 t22-e1, finding V8. The stored `unread` counter has to survive
    /// **every** local write path, so this asserts exact values across a flag
    /// write, a move and a delete — plus the no-op cases, which is where a
    /// counter maintained by a happy-path-only fix drifts.
    ///
    /// Shared with the live-Postgres leg below: the delete path reads the row it
    /// removed with `DELETE … RETURNING`, which is not the same code in the two
    /// engines even though it is the same SQL.
    #[tokio::test]
    async fn unread_counter_holds_across_flag_write_move_and_delete() {
        assert_unread_invariant(&store().await).await;
    }

    async fn assert_unread_invariant(s: &Store) {
        let account_id = seed_account(s).await;
        let inbox = seed_mailbox(s, &account_id, "INBOX", 100).await;
        let archive = seed_mailbox(s, &account_id, "Archive", 100).await;

        async fn unread(s: &Store, mailbox_id: &str) -> u32 {
            s.get_mailbox(mailbox_id).await.unwrap().unread
        }

        let mut ids = Vec::new();
        // Distinct Message-IDs: same `(message_id, internaldate, size)` in one
        // mailbox is the UIDVALIDITY-change identity match, which would fold
        // these three into one row.
        for (uid, mid) in [(1u32, "<u1@x>"), (2, "<u2@x>"), (3, "<u3@x>")] {
            let mut m = msg(&account_id, &inbox, uid, 100, mid, "2026-07-01T10:00:00Z");
            m.flags_json = "[]";
            ids.push(s.upsert_message(&m).await.unwrap());
        }
        // The counter starts where a sync left it: three unread messages.
        s.update_mailbox_counts(&inbox, 4, 0, 3, 3).await.unwrap();
        assert_eq!(unread(&s, &inbox).await, 3);

        // ---- flag write -----------------------------------------------------
        s.set_flags(&ids[0], r#"["Seen"]"#).await.unwrap();
        assert_eq!(unread(&s, &inbox).await, 2, "reading a message decrements");
        // Re-setting the same state must not decrement twice — the delta is on
        // the transition, not on the write.
        s.set_flags(&ids[0], r#"["Seen","Flagged"]"#).await.unwrap();
        assert_eq!(unread(&s, &inbox).await, 2);
        // An unrelated keyword on a still-unread message moves nothing.
        s.set_flags(&ids[1], r#"["Flagged"]"#).await.unwrap();
        assert_eq!(unread(&s, &inbox).await, 2);
        // Marking it unread again puts it back.
        s.set_flags(&ids[0], r#"["Flagged"]"#).await.unwrap();
        assert_eq!(unread(&s, &inbox).await, 3);
        // A custom keyword is `{"Keyword":"…"}`, never the bare string `"Seen"`,
        // so nothing here can be mistaken for the seen flag.
        s.set_flags(&ids[0], r#"[{"Keyword":"$SeenByAssistant"}]"#)
            .await
            .unwrap();
        assert_eq!(unread(&s, &inbox).await, 3);
        // A flags value the engine could never write must not fail the write or
        // move the counter off its own arithmetic: unparseable counts as unread.
        s.set_flags(&ids[0], "not json").await.unwrap();
        assert_eq!(unread(&s, &inbox).await, 3);
        s.set_flags(&ids[0], "[]").await.unwrap();
        assert_eq!(unread(&s, &inbox).await, 3);

        // A write to a message that does not exist is still an error, and still
        // moves no counter.
        assert!(matches!(
            s.set_flags("nope", r#"["Seen"]"#).await,
            Err(StoreError::NotFound)
        ));
        assert_eq!(unread(&s, &inbox).await, 3);

        // ---- move -----------------------------------------------------------
        s.relocate_message(&ids[1], &archive, 7, 100).await.unwrap();
        assert_eq!(unread(&s, &inbox).await, 2, "the source loses it");
        assert_eq!(unread(&s, &archive).await, 1, "the destination gains it");
        // A read message carries nothing across a move.
        s.set_flags(&ids[2], r#"["Seen"]"#).await.unwrap();
        assert_eq!(unread(&s, &inbox).await, 1);
        s.relocate_message(&ids[2], &archive, 8, 100).await.unwrap();
        assert_eq!(unread(&s, &inbox).await, 1);
        assert_eq!(unread(&s, &archive).await, 1);
        // A "move" that does not change mailbox is not a move.
        s.relocate_message(&ids[1], &archive, 9, 100).await.unwrap();
        assert_eq!(unread(&s, &archive).await, 1);
        // A move of a message that is not there changes neither counter.
        assert!(matches!(
            s.relocate_message("nope", &archive, 10, 100).await,
            Err(StoreError::NotFound)
        ));
        assert_eq!(
            (unread(&s, &inbox).await, unread(&s, &archive).await),
            (1, 1)
        );

        // ---- delete ---------------------------------------------------------
        // ids[2] is read and lives in Archive: deleting it leaves the counter.
        s.delete_message(&ids[2]).await.unwrap();
        assert_eq!(unread(&s, &archive).await, 1);
        // ids[1] is unread and lives in Archive: deleting it takes the counter
        // down with it.
        s.delete_message(&ids[1]).await.unwrap();
        assert_eq!(unread(&s, &archive).await, 0);
        // Deleting something that is already gone is a no-op, counter included.
        s.delete_message(&ids[1]).await.unwrap();
        assert_eq!(unread(&s, &archive).await, 0);

        // ---- the floor ------------------------------------------------------
        // `unread` is read back as u32, so a counter driven below zero would not
        // read as -1 but as ~4 billion. ids[0] is unread in INBOX; force the
        // stored counter to 0 first and confirm the delete cannot underflow it.
        s.update_mailbox_counts(&inbox, 4, 0, 1, 0).await.unwrap();
        s.delete_message(&ids[0]).await.unwrap();
        assert_eq!(unread(&s, &inbox).await, 0);
    }

    /// 26.20 t22-e1. `set_flags_batch` must be the loop it replaces — same rows
    /// written, same counter arithmetic — while costing a bounded number of
    /// statements. Two mailboxes, because the counter is per-mailbox and a
    /// single-folder fixture would pass a version that adds every delta to
    /// whichever mailbox it saw first.
    #[tokio::test]
    async fn set_flags_batch_matches_the_loop_and_maintains_every_counter() {
        assert_batch_flag_write(&store().await).await;
    }

    async fn assert_batch_flag_write(s: &Store) {
        let account_id = seed_account(s).await;
        let inbox = seed_mailbox(s, &account_id, "INBOX", 100).await;
        let archive = seed_mailbox(s, &account_id, "Archive", 100).await;

        let mut inbox_ids = Vec::new();
        for (uid, mid) in [(1u32, "<a1@x>"), (2, "<a2@x>"), (3, "<a3@x>")] {
            let mut m = msg(&account_id, &inbox, uid, 100, mid, "2026-07-01T10:00:00Z");
            m.flags_json = "[]";
            inbox_ids.push(s.upsert_message(&m).await.unwrap());
        }
        let mut archive_ids = Vec::new();
        for (uid, mid) in [(4u32, "<b1@x>"), (5, "<b2@x>")] {
            let mut m = msg(&account_id, &archive, uid, 100, mid, "2026-07-01T10:00:00Z");
            m.flags_json = "[]";
            archive_ids.push(s.upsert_message(&m).await.unwrap());
        }
        s.update_mailbox_counts(&inbox, 4, 0, 3, 3).await.unwrap();
        s.update_mailbox_counts(&archive, 6, 0, 2, 2).await.unwrap();

        // Two mailboxes at once; one id repeated with conflicting flags (the
        // LAST must win); one id that names nothing; one write that does not
        // change the read state at all.
        let updates: Vec<(String, String)> = vec![
            (inbox_ids[0].clone(), r#"["Seen"]"#.into()),
            (inbox_ids[1].clone(), r#"["Seen"]"#.into()),
            (inbox_ids[1].clone(), r#"["Flagged"]"#.into()), // later: still unread
            (archive_ids[0].clone(), r#"["Seen"]"#.into()),
            ("no-such-message".into(), r#"["Seen"]"#.into()),
        ];
        let written = s.set_flags_batch(&updates).await.unwrap();
        assert_eq!(
            written,
            vec![true, true, true, true, false],
            "the answer is positional, and only the unknown id is false"
        );

        assert_eq!(
            s.get_message(&inbox_ids[0]).await.unwrap().flags_json,
            r#"["Seen"]"#
        );
        assert_eq!(
            s.get_message(&inbox_ids[1]).await.unwrap().flags_json,
            r#"["Flagged"]"#,
            "duplicate ids resolve last-wins, not database-order-wins"
        );
        assert_eq!(
            s.get_message(&inbox_ids[2]).await.unwrap().flags_json,
            "[]",
            "a message not named in the batch is untouched"
        );
        assert_eq!(
            s.get_message(&archive_ids[1]).await.unwrap().flags_json,
            "[]"
        );

        // One read in INBOX (ids[1] ended up `\Flagged`, still unread), one in
        // Archive. Both counters move, each by its own amount.
        assert_eq!(s.get_mailbox(&inbox).await.unwrap().unread, 2);
        assert_eq!(s.get_mailbox(&archive).await.unwrap().unread, 1);

        // Idempotent: replaying the same batch moves nothing.
        s.set_flags_batch(&updates).await.unwrap();
        assert_eq!(s.get_mailbox(&inbox).await.unwrap().unread, 2);
        assert_eq!(s.get_mailbox(&archive).await.unwrap().unread, 1);

        // Back to unread, in both directions at once.
        let back: Vec<(String, String)> = vec![
            (inbox_ids[0].clone(), "[]".into()),
            (archive_ids[0].clone(), "[]".into()),
            (archive_ids[1].clone(), r#"["Seen"]"#.into()),
        ];
        assert_eq!(s.set_flags_batch(&back).await.unwrap(), vec![true; 3]);
        assert_eq!(s.get_mailbox(&inbox).await.unwrap().unread, 3);
        assert_eq!(
            s.get_mailbox(&archive).await.unwrap().unread,
            1,
            "+1 and -1 in one mailbox in one batch nets to zero"
        );

        // Degenerate inputs.
        assert!(s.set_flags_batch(&[]).await.unwrap().is_empty());
        assert_eq!(
            s.set_flags_batch(&[("nope".into(), "[]".into())])
                .await
                .unwrap(),
            vec![false]
        );
        assert_eq!(s.get_mailbox(&inbox).await.unwrap().unread, 3);
    }

    /// 26.20 t22-e1. `LIMIT`/`OFFSET` reaching SQL is what makes a page cheap
    /// (V10), so the degenerate arguments have to behave rather than reach the
    /// database as something a backend interprets differently — Postgres rejects
    /// a negative `OFFSET` outright, SQLite reads it as 0.
    #[tokio::test]
    async fn list_message_ids_paging_edges_and_total_order() {
        let s = store().await;
        let account_id = seed_account(&s).await;
        let mailbox_id = seed_mailbox(&s, &account_id, "INBOX", 100).await;

        // Two messages sharing an (internaldate, uid) is what a total order is
        // for: the pair below is identical on every column the sort used to name.
        let mut tied = Vec::new();
        for i in 0..2 {
            let mut m = msg(
                &account_id,
                &mailbox_id,
                9,
                100 + i, // distinct uidvalidity keeps the row unique, not the sort
                "<tie@x>",
                "2026-07-05T00:00:00Z",
            );
            m.message_id = Some(if i == 0 { "<tie-a@x>" } else { "<tie-b@x>" });
            tied.push(s.upsert_message(&m).await.unwrap());
        }
        tied.sort();

        // Page 1 then page 2 of size 1 must together be both rows, exactly once
        // each — the property `OFFSET` paging silently loses without a total
        // order.
        let first = s.list_message_ids(&mailbox_id, 1, 0).await.unwrap();
        let second = s.list_message_ids(&mailbox_id, 1, 1).await.unwrap();
        let mut paged = [first, second].concat();
        paged.sort();
        assert_eq!(paged, tied);

        // Degenerate arguments.
        assert!(
            s.list_message_ids(&mailbox_id, 0, 0)
                .await
                .unwrap()
                .is_empty()
        );
        assert!(
            s.list_message_ids(&mailbox_id, -1, 0)
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            s.list_message_ids(&mailbox_id, 10, -5).await.unwrap().len(),
            2
        );
        // Past the end is empty, not an error.
        assert!(
            s.list_message_ids(&mailbox_id, 10, 99)
                .await
                .unwrap()
                .is_empty()
        );
    }

    /// Open the live-Postgres store for the env-gated legs, or explain why they
    /// are not running. `DATABASE_URL_PG` / `MW_TEST_PG`, the same convention as
    /// `tests/backend_parity.rs`.
    async fn live_pg(what: &str) -> Option<Store> {
        let Some(dsn) = std::env::var("DATABASE_URL_PG")
            .ok()
            .or_else(|| std::env::var("MW_TEST_PG").ok())
            .filter(|s| !s.trim().is_empty())
        else {
            eprintln!(
                "[mw-store] t22-e1 {what}: Postgres path SKIPPED (set DATABASE_URL_PG or \
                 MW_TEST_PG to a live postgres:16 to run it). The SQLite path still asserted."
            );
            return None;
        };
        Some(
            Store::open_postgres(&dsn, ServerKey::generate())
                .await
                .expect("DATABASE_URL_PG is set but Postgres is not reachable"),
        )
    }

    /// 26.20 t22-e1. The batch getters and the unread counter, on the backend
    /// whose SQL differs.
    ///
    /// Four things here exist only on this path and are untested by every SQLite
    /// assertion above: the `?1::json` cast plus `json_array_elements_text` that
    /// expands the id list, the `UPDATE … FROM json_array_elements` join behind
    /// `set_flags_batch`, `DELETE … RETURNING` as Postgres implements it, and the
    /// `unread` column arriving as `BIGINT` rather than `INTEGER`.
    #[tokio::test]
    async fn postgres_batch_getters_and_unread_counter() {
        let Some(s) = live_pg("batch getters + unread counter").await else {
            return;
        };
        assert_batch_getters_match_the_loop(&s).await;
        assert_unread_invariant(&s).await;
        assert_batch_flag_write(&s).await;
    }

    /// 26.20 t22-e1, per-backend half 1 of 2 (V10). An index that exists but is
    /// not chosen is worth nothing, so this asserts the **plan**, and it asserts
    /// it at a **deep offset** — page 1 was already fast before 0023, and an
    /// assertion written at `OFFSET 0` proves nothing on either backend.
    ///
    /// The negative control is the load-bearing half: the same query with 0023
    /// dropped must get the bad plan back. Without it, this test passes on a
    /// planner that ignores the index entirely.
    #[tokio::test]
    async fn sqlite_page_query_uses_the_covering_index_and_sorts_nothing() {
        // Each leg gets its OWN database, and the control drops 0023's index
        // before any statement runs against it. Dropping the index mid-store and
        // re-explaining does not work: the plan text comes from bytecode built at
        // prepare time, and sqlx's per-connection statement cache hands the same
        // compiled statement back, so the control silently re-reads the plan it
        // was supposed to disprove.
        async fn plan(drop_0023: bool) -> String {
            let s = store().await;
            if drop_0023 {
                q("DROP INDEX idx_messages_mailbox_page")
                    .execute(s.backend())
                    .await
                    .unwrap();
            }
            let account_id = seed_account(&s).await;
            let mailbox_id = seed_mailbox(&s, &account_id, "INBOX", 100).await;
            // SQLite's planner chooses from the schema and the query shape, not
            // from table statistics (no ANALYZE has run), so 40 rows exercise the
            // same decision 20 000 would.
            for uid in 1..=40u32 {
                let date = format!("2026-07-01T00:{uid:02}:00Z");
                let m = msg(&account_id, &mailbox_id, uid, 100, "<p@x>", &date);
                s.upsert_message(&m).await.unwrap();
            }
            let rows = q("EXPLAIN QUERY PLAN
                 SELECT stable_id FROM messages WHERE mailbox_id = ?1
                 ORDER BY internaldate DESC, uid DESC, stable_id
                 LIMIT 5 OFFSET 30")
            .bind(&mailbox_id)
            .fetch_all(s.backend())
            .await
            .unwrap();
            rows.iter()
                .map(|r| r.get_string("detail"))
                .collect::<Vec<_>>()
                .join(" | ")
        }

        let with_index = plan(false).await;
        assert!(
            with_index.contains("idx_messages_mailbox_page"),
            "0023's index must be the one chosen; plan was: {with_index}"
        );
        assert!(
            with_index.contains("COVERING INDEX"),
            "the page query selects only indexed columns, so it must not touch \
             the table; plan was: {with_index}"
        );
        assert!(
            !with_index.contains("TEMP B-TREE"),
            "the index carries both sort terms in the queried direction, so there \
             is nothing left to sort; plan was: {with_index}"
        );

        // Negative control: without 0023 the sort comes back.
        let without_index = plan(true).await;
        assert!(
            without_index.contains("TEMP B-TREE"),
            "with 0023 dropped the planner must fall back to sorting — if it does \
             not, this test proves nothing about the index. Plan was: \
             {without_index}"
        );
    }

    /// 26.20 t22-e1, per-backend half 2 of 2 (V10). The Postgres half had to be
    /// written differently from the plan's original phrasing twice over.
    ///
    /// First, the offset: at `OFFSET 0` Postgres already answers a real `LIMIT`
    /// in 0.405 ms from the **0002** index, so "the plan gains a sort when 0023
    /// is dropped" is not even true there. At a deep offset it is.
    ///
    /// Second — and this is not in the plan document — the **corpus**. Three
    /// properties of it decide the plan, and each was found by a run that
    /// disagreed with the previous one:
    ///
    /// * **Selectivity.** Where the target mailbox is 100 % of `messages`, which
    ///   is what a single-folder benchmark builds and what the scale verifier
    ///   measured, Postgres picks Seq Scan + Sort at a deep offset **with or
    ///   without** 0023 — a predicate matching every row cannot be served better
    ///   by an index, and the seq scan wins on cost honestly. Hence the second
    ///   mailbox: the folder under test is 20 % of the table.
    /// * **Id width.** `stable_id` is 64 characters in production, and index
    ///   entries that wide cost enough to flip the planner's arithmetic. Seeding
    ///   short ids makes this test pass for the wrong reason.
    /// * **The visibility map.** An Index Only Scan over a table that has never
    ///   been vacuumed is one heap fetch per row, so `ANALYZE` alone leaves
    ///   Postgres preferring the Seq Scan. Autovacuum is what makes a live
    ///   deployment the vacuumed case; a table bulk-loaded a millisecond ago is
    ///   the artificial one, so this vacuums before measuring.
    ///
    /// Each of those produced a *failing* run of this test against an index that
    /// was doing its job, which is the argument for stating them here rather
    /// than tuning the assertion until it passed.
    ///
    /// Runs only against a live server (`DATABASE_URL_PG` / `MW_TEST_PG`, the
    /// same convention as `tests/backend_parity.rs`); prints why it did not run
    /// otherwise, rather than passing silently.
    #[tokio::test]
    async fn postgres_deep_offset_page_uses_the_covering_index() {
        let Some(s) = live_pg("deep-offset plan").await else {
            return;
        };
        let account_id = seed_account(&s).await;
        let mailbox_id = seed_mailbox(&s, &account_id, "INBOX", 100).await;
        let other_id = seed_mailbox(&s, &account_id, "Archive", 100).await;

        // 20 000 rows in the folder under test and 80 000 in a neighbour, each in
        // ONE statement, with ids the width the store really mints (the account
        // id is itself a 64-character token, so `id || n` is production-shaped
        // and unique across concurrent runs).
        q("INSERT INTO messages
                (stable_id, account_id, mailbox_id, uid, uidvalidity, internaldate, size, flags_json)
             SELECT ?1 || n::text, ?2, ?3, n, 100, to_char(n, 'FM00000000'), 0, '[]'
             FROM generate_series(1, 20000) AS n")
        .bind(&account_id)
        .bind(&account_id)
        .bind(&mailbox_id)
        .execute(s.backend())
        .await
        .expect("bulk seed");
        q("INSERT INTO messages
                (stable_id, account_id, mailbox_id, uid, uidvalidity, internaldate, size, flags_json)
             SELECT 'other-' || ?1 || n::text, ?2, ?3, n, 100, to_char(n, 'FM00000000'), 0, '[]'
             FROM generate_series(1, 80000) AS n")
        .bind(&account_id)
        .bind(&account_id)
        .bind(&other_id)
        .execute(s.backend())
        .await
        .expect("bulk seed neighbour");
        // VACUUM builds the visibility map an Index Only Scan needs, and cannot
        // run through the extended protocol (it would be inside an implicit
        // transaction), so it goes out on the simple-query path — the same
        // reasoning as `Store::reclaim_note_metadata_residue`.
        {
            use sqlx::Executor as _;
            let crate::backend::Backend::Postgres(pool) = s.backend() else {
                unreachable!("opened via open_postgres")
            };
            pool.execute("VACUUM ANALYZE messages").await.unwrap();
        }

        async fn plan(s: &Store, mailbox_id: &str) -> String {
            let rows = q("EXPLAIN (ANALYZE, BUFFERS)
                 SELECT stable_id FROM messages WHERE mailbox_id = ?1
                 ORDER BY internaldate DESC, uid DESC, stable_id
                 LIMIT 50 OFFSET 19950")
            .bind(mailbox_id)
            .fetch_all(s.backend())
            .await
            .unwrap();
            rows.iter()
                .map(|r| r.get_string_idx(0))
                .collect::<Vec<_>>()
                .join("\n")
        }

        // Collect both plans BEFORE asserting: a failed assertion must not leave
        // a shared database without the index.
        let with_index = plan(&s, &mailbox_id).await;
        q("DROP INDEX idx_messages_mailbox_page")
            .execute(s.backend())
            .await
            .unwrap();
        q("ANALYZE messages").execute(s.backend()).await.unwrap();
        let without_index = plan(&s, &mailbox_id).await;
        q("CREATE INDEX IF NOT EXISTS idx_messages_mailbox_page
             ON messages (mailbox_id, internaldate DESC, uid DESC, stable_id)")
        .execute(s.backend())
        .await
        .unwrap();
        // The account cascade takes the mailbox and all 20 000 rows with it.
        q("DELETE FROM accounts WHERE id = ?1")
            .bind(&account_id)
            .execute(s.backend())
            .await
            .unwrap();

        assert!(
            with_index.contains("idx_messages_mailbox_page"),
            "0023's index must be the one chosen at a deep offset; plan was:\n{with_index}"
        );
        assert!(
            !with_index.contains("Seq Scan"),
            "a deep-offset page must not fall back to a sequential scan; plan was:\n{with_index}"
        );
        assert!(
            !with_index.contains("Sort Method"),
            "the index carries both sort terms in the queried direction, so a \
             deep-offset page must not sort at all; plan was:\n{with_index}"
        );
        assert!(
            without_index.contains("Seq Scan") && without_index.contains("Sort Method"),
            "negative control: with 0023 dropped the deep-offset page must go back \
             to scanning and sorting — if it does not, this test proves nothing \
             about the index. Plan was:\n{without_index}"
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

    /// S3 (26.19): an expunged message must not leave its content-derived embedding
    /// behind. **Fails against the pre-fix code**, on the first assertion:
    /// `delete_message_embedding` had no production caller at all, so the vector
    /// survived the message indefinitely — until an operator happened to run the
    /// account-wide escape hatch.
    #[tokio::test]
    async fn deleting_a_message_drops_its_embedding() {
        let s = store().await;
        let account_id = seed_account(&s).await;
        let mailbox_id = seed_mailbox(&s, &account_id, "INBOX", 100).await;

        let doomed = s
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
        let kept = s
            .upsert_message(&msg(
                &account_id,
                &mailbox_id,
                2,
                100,
                "<b@x>",
                "2026-07-01T11:00:00Z",
            ))
            .await
            .unwrap();

        s.put_message_embedding(&doomed, &account_id, "m", &[0.5, 0.25])
            .await
            .unwrap();
        s.put_message_embedding(&kept, &account_id, "m", &[0.25, 0.5])
            .await
            .unwrap();
        assert_eq!(s.count_message_embeddings(&account_id).await.unwrap(), 2);

        s.delete_message(&doomed).await.unwrap();

        assert!(
            s.get_message_embedding(&doomed).await.unwrap().is_none(),
            "an expunged message's embedding must not outlive it — a vector is a \
             partially invertible projection of the message text"
        );
        // Control: ONLY the expunged message's vector went. A blanket wipe would
        // satisfy the assertion above while silently destroying the rest of the
        // account's cache.
        assert!(
            s.get_message_embedding(&kept).await.unwrap().is_some(),
            "another message's embedding is untouched"
        );
        assert_eq!(s.count_message_embeddings(&account_id).await.unwrap(), 1);

        // Deleting a message that never had an embedding is still a no-op, not an
        // error: most deployments never configure Assist at all.
        s.delete_message(&kept).await.unwrap();
        assert_eq!(s.count_message_embeddings(&account_id).await.unwrap(), 0);
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
        let raw = q("SELECT sealed_bytes FROM bodies WHERE blob_ref = ?1")
            .bind(&blob_ref)
            .fetch_one(s.backend())
            .await
            .unwrap()
            .get_blob("sealed_bytes");
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

        let sealed_env = q("SELECT envelope_json FROM messages WHERE stable_id = ?1")
            .bind(&id)
            .fetch_one(s.backend())
            .await
            .unwrap()
            .get_opt_blob("envelope_json");
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
