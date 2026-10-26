//! 0033 user directory repository (26.20 t32-q e66, worker f06): dual-backend
//! `Store` methods over `mail_user` and `mail_user_account`.
//!
//! # Tables
//! * `mail_user` — one row per canonical address that is a user of this server,
//!   whatever the mode: `origin` says how the row came to be (`provisioned` by an
//!   admin, first `signin`, `sso`, or the boot backfill's `import`), `status` is
//!   `active`, `disabled`, `deleting` or `deleted` (a tombstone).
//! * `mail_user_account` — the session and engine account ids mapped to the
//!   address (engine store id, proxy upstream id, header-auth or SSO id, a linked
//!   `child`).
//!
//! The account-gate flags stay the authority for "disabled" (t32-q DQ7): the
//! `status` column is a mirror for listing and filtering, written by the same
//! writer as the flags and reconciled at boot. `mw-server` decides what a
//! canonical address is; this module stores what it is given.
//!
//! `origin` is written on insert only (t32-q A12): a sign-in never turns a
//! provisioned user into a self-signed one. Reusing a tombstone is the only
//! overwrite.
//!
//! Timestamps are unix seconds (0033 wire contract).

use crate::backend::{Dialect, Row, q};
use crate::{Store, StoreError};

/// How a directory row came to be (`mail_user.origin`).
pub const ORIGINS: [&str; 4] = ["provisioned", "signin", "sso", "import"];
/// The directory statuses (`mail_user.status`).
pub const STATUSES: [&str; 4] = ["active", "disabled", "deleting", "deleted"];
/// The account-id kinds (`mail_user_account.mode`).
pub const ACCOUNT_MODES: [&str; 5] = ["engine", "proxy", "header", "sso", "child"];

/// `last_signin_at` and `last_seen` are written at most this often (t32-q DQ24).
pub const SIGNIN_TOUCH_SECS: i64 = 3600;

/// One `mail_user` row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailUserRow {
    pub address: String,
    /// The canonical domain; `""` for a bare name kept as typed.
    pub domain: String,
    pub local_part: String,
    pub origin: String,
    pub status: String,
    pub display_name: Option<String>,
    pub note: Option<String>,
    pub created_at: i64,
    pub created_by: Option<String>,
    pub first_signin_at: Option<i64>,
    pub last_signin_at: Option<i64>,
    pub status_changed_at: Option<i64>,
    pub status_changed_by: Option<String>,
    pub status_reason: Option<String>,
    pub expires_at: Option<i64>,
    pub deleted_at: Option<i64>,
}

impl MailUserRow {
    /// A fresh row for `address` with `origin`, everything else empty.
    pub fn new(address: &str, domain: &str, local_part: &str, origin: &str, now: i64) -> Self {
        Self {
            address: address.to_string(),
            domain: domain.to_string(),
            local_part: local_part.to_string(),
            origin: origin.to_string(),
            status: "active".to_string(),
            display_name: None,
            note: None,
            created_at: now,
            created_by: None,
            first_signin_at: None,
            last_signin_at: None,
            status_changed_at: None,
            status_changed_by: None,
            status_reason: None,
            expires_at: None,
            deleted_at: None,
        }
    }

    /// A live row: not deleted, and not a provisioned record that lapsed unused
    /// (t32-q A20 "Existing").
    pub fn is_live(&self, now: i64) -> bool {
        match self.status.as_str() {
            "active" | "disabled" => {}
            _ => return false,
        }
        !(self.origin == "provisioned"
            && self.first_signin_at.is_none()
            && self.expires_at.is_some_and(|e| e <= now))
    }

    /// Whether the user has signed in here before (for the `closed` policy).
    pub fn has_signed_in(&self) -> bool {
        self.first_signin_at.is_some() || matches!(self.origin.as_str(), "signin" | "sso" | "import")
    }
}

/// One `mail_user_account` row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailUserAccountRow {
    pub account_id: String,
    pub address: String,
    pub mode: String,
    pub first_seen: i64,
    pub last_seen: i64,
}

/// The order of a directory listing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MailUserSort {
    #[default]
    Address,
    Domain,
    Status,
    Created,
    FirstSignin,
    LastSignin,
}

impl MailUserSort {
    /// The wire name (`sort=` of `GET /admin/users`).
    pub fn as_str(self) -> &'static str {
        match self {
            MailUserSort::Address => "address",
            MailUserSort::Domain => "domain",
            MailUserSort::Status => "status",
            MailUserSort::Created => "created",
            MailUserSort::FirstSignin => "firstSignin",
            MailUserSort::LastSignin => "lastSignin",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        [
            MailUserSort::Address,
            MailUserSort::Domain,
            MailUserSort::Status,
            MailUserSort::Created,
            MailUserSort::FirstSignin,
            MailUserSort::LastSignin,
        ]
        .into_iter()
        .find(|v| v.as_str() == s)
    }
}

/// A directory listing's filter. `None` fields do not filter.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MailUserFilter {
    /// Matches the address or the display name, case-insensitively.
    pub q: Option<String>,
    /// Exactly this domain (`""` = bare names).
    pub domain: Option<String>,
    /// Only these domains (a domain-scoped admin's grant). An empty list matches
    /// nothing; bare names (`""`) are never in a domain scope.
    pub domains: Option<Vec<String>>,
    pub status: Option<String>,
    pub origin: Option<String>,
    /// Tombstones are left out unless the status filter asks for them.
    pub include_deleted: bool,
}

/// What [`Store::mint_mail_user`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MintOutcome {
    /// A new row (or a reused tombstone) was written.
    Created,
    /// The row existed; its sign-in times were updated.
    Existing,
    /// The row is being deleted: nothing was written (t32-q R6, DQ26).
    Deleting,
}

/// The directory counts of the admin dashboard's `users` block.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MailUserCounts {
    pub total: i64,
    pub active: i64,
    pub disabled: i64,
    pub provisioned_never_signed_in: i64,
    pub new_7d: i64,
}

const COLS: &str = "address, domain, local_part, origin, status, display_name, note, created_at, \
     created_by, first_signin_at, last_signin_at, status_changed_at, status_changed_by, \
     status_reason, expires_at, deleted_at";

fn user_from_row(r: &Row) -> MailUserRow {
    MailUserRow {
        address: r.get_string("address"),
        domain: r.get_string("domain"),
        local_part: r.get_string("local_part"),
        origin: r.get_string("origin"),
        status: r.get_string("status"),
        display_name: r.get_opt_string("display_name"),
        note: r.get_opt_string("note"),
        created_at: r.get_i64("created_at"),
        created_by: r.get_opt_string("created_by"),
        first_signin_at: r.get_opt_i64("first_signin_at"),
        last_signin_at: r.get_opt_i64("last_signin_at"),
        status_changed_at: r.get_opt_i64("status_changed_at"),
        status_changed_by: r.get_opt_string("status_changed_by"),
        status_reason: r.get_opt_string("status_reason"),
        expires_at: r.get_opt_i64("expires_at"),
        deleted_at: r.get_opt_i64("deleted_at"),
    }
}

fn account_from_row(r: &Row) -> MailUserAccountRow {
    MailUserAccountRow {
        account_id: r.get_string("account_id"),
        address: r.get_string("address"),
        mode: r.get_string("mode"),
        first_seen: r.get_i64("first_seen"),
        last_seen: r.get_i64("last_seen"),
    }
}

/// `%`, `_` and `\` in a search term match only themselves.
fn like_term(term: &str) -> String {
    format!(
        "%{}%",
        term.trim()
            .to_lowercase()
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    )
}

/// The domain scope as `,a,b,` for a membership test, or `None` for all.
fn scope_list(domains: &Option<Vec<String>>) -> Option<String> {
    domains.as_ref().map(|d| {
        let mut s = String::from(",");
        for x in d {
            s.push_str(&x.to_ascii_lowercase());
            s.push(',');
        }
        s
    })
}

// The filter part shared by the listing and the count. ?1 q, ?2 domain, ?3 scope
// list, ?4 status, ?5 origin, ?6 include deleted (0/1). The scope membership test
// is spelled per dialect (`instr` / `strpos`).
macro_rules! user_where {
    ($member:literal) => {
        concat!(
            " WHERE (?1 IS NULL OR lower(address) LIKE ?1 ESCAPE '\\' \
               OR lower(COALESCE(display_name, '')) LIKE ?1 ESCAPE '\\') \
             AND (?2 IS NULL OR domain = ?2) \
             AND (?3 IS NULL OR (domain <> '' AND ",
            $member,
            " > 0)) \
             AND (?4 IS NULL OR status = ?4) \
             AND (?5 IS NULL OR origin = ?5) \
             AND (?6 = 1 OR ?4 IS NOT NULL OR status <> 'deleted')"
        )
    };
}

macro_rules! user_order {
    () => {
        " ORDER BY \
           CASE WHEN ?7 = 'domain' AND ?8 = 0 THEN domain END ASC, \
           CASE WHEN ?7 = 'domain' AND ?8 = 1 THEN domain END DESC, \
           CASE WHEN ?7 = 'status' AND ?8 = 0 THEN status END ASC, \
           CASE WHEN ?7 = 'status' AND ?8 = 1 THEN status END DESC, \
           CASE WHEN ?7 = 'created' AND ?8 = 0 THEN created_at END ASC, \
           CASE WHEN ?7 = 'created' AND ?8 = 1 THEN created_at END DESC, \
           CASE WHEN ?7 = 'firstSignin' AND ?8 = 0 THEN COALESCE(first_signin_at, -1) END ASC, \
           CASE WHEN ?7 = 'firstSignin' AND ?8 = 1 THEN COALESCE(first_signin_at, -1) END DESC, \
           CASE WHEN ?7 = 'lastSignin' AND ?8 = 0 THEN COALESCE(last_signin_at, -1) END ASC, \
           CASE WHEN ?7 = 'lastSignin' AND ?8 = 1 THEN COALESCE(last_signin_at, -1) END DESC, \
           CASE WHEN ?7 = 'address' AND ?8 = 1 THEN address END DESC, \
           address ASC \
         LIMIT ?9 OFFSET ?10"
    };
}

const LIST_SQLITE: &str = concat!(
    "SELECT address, domain, local_part, origin, status, display_name, note, created_at, \
     created_by, first_signin_at, last_signin_at, status_changed_at, status_changed_by, \
     status_reason, expires_at, deleted_at FROM mail_user",
    user_where!("instr(?3, ',' || domain || ',')"),
    user_order!()
);
const LIST_PG: &str = concat!(
    "SELECT address, domain, local_part, origin, status, display_name, note, created_at, \
     created_by, first_signin_at, last_signin_at, status_changed_at, status_changed_by, \
     status_reason, expires_at, deleted_at FROM mail_user",
    user_where!("strpos(?3, ',' || domain || ',')"),
    user_order!()
);
const COUNT_SQLITE: &str = concat!(
    "SELECT COUNT(*) FROM mail_user",
    user_where!("instr(?3, ',' || domain || ',')")
);
const COUNT_PG: &str = concat!(
    "SELECT COUNT(*) FROM mail_user",
    user_where!("strpos(?3, ',' || domain || ',')")
);

impl Store {
    /// The directory row of `address`, tombstones included.
    pub async fn get_mail_user(&self, address: &str) -> Result<Option<MailUserRow>, StoreError> {
        let row = q("SELECT address, domain, local_part, origin, status, display_name, note, \
             created_at, created_by, first_signin_at, last_signin_at, status_changed_at, \
             status_changed_by, status_reason, expires_at, deleted_at \
             FROM mail_user WHERE address = ?1")
        .bind(address)
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.as_ref().map(user_from_row))
    }

    /// Insert `row` unless the address already has a live row. A tombstone
    /// (`deleted`) is reused: every field is replaced by `row`'s. Returns whether
    /// a row was written.
    pub async fn insert_mail_user(&self, row: &MailUserRow) -> Result<bool, StoreError> {
        let n = q("INSERT INTO mail_user (address, domain, local_part, origin, status, \
             display_name, note, created_at, created_by, first_signin_at, last_signin_at, \
             status_changed_at, status_changed_by, status_reason, expires_at, deleted_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, NULL) \
             ON CONFLICT(address) DO UPDATE SET domain = excluded.domain, \
               local_part = excluded.local_part, origin = excluded.origin, \
               status = excluded.status, display_name = excluded.display_name, \
               note = excluded.note, created_at = excluded.created_at, \
               created_by = excluded.created_by, first_signin_at = excluded.first_signin_at, \
               last_signin_at = excluded.last_signin_at, \
               status_changed_at = excluded.status_changed_at, \
               status_changed_by = excluded.status_changed_by, \
               status_reason = excluded.status_reason, expires_at = excluded.expires_at, \
               deleted_at = NULL \
             WHERE mail_user.status = 'deleted'")
        .bind(&row.address)
        .bind(&row.domain)
        .bind(&row.local_part)
        .bind(&row.origin)
        .bind(&row.status)
        .bind(row.display_name.clone())
        .bind(row.note.clone())
        .bind(row.created_at)
        .bind(row.created_by.clone())
        .bind(row.first_signin_at)
        .bind(row.last_signin_at)
        .bind(row.status_changed_at)
        .bind(row.status_changed_by.clone())
        .bind(row.status_reason.clone())
        .bind(row.expires_at)
        .execute(&self.backend)
        .await?;
        Ok(n == 1)
    }

    /// Change the admin-editable fields of a live row. Returns whether it exists.
    pub async fn update_mail_user_details(
        &self,
        address: &str,
        display_name: Option<&str>,
        note: Option<&str>,
        expires_at: Option<i64>,
    ) -> Result<bool, StoreError> {
        let n = q("UPDATE mail_user SET display_name = ?2, note = ?3, expires_at = ?4 \
             WHERE address = ?1 AND status <> 'deleted'")
        .bind(address)
        .bind(display_name)
        .bind(note)
        .bind(expires_at)
        .execute(&self.backend)
        .await?;
        Ok(n == 1)
    }

    /// The sign-in write at session mint (t32-q §1(d), A2, A12): in one
    /// transaction, insert the row with `origin` (or reuse a tombstone, resetting
    /// its dates), set `first_signin_at` when it is empty and `last_signin_at` at
    /// most once an hour, and map `account_id` to the address. Writes nothing when
    /// the row is `deleting` (the compare-and-set of R6).
    #[allow(clippy::too_many_arguments)]
    pub async fn mint_mail_user(
        &self,
        address: &str,
        domain: &str,
        local_part: &str,
        origin: &str,
        account_id: &str,
        mode: &str,
        now: i64,
    ) -> Result<MintOutcome, StoreError> {
        let mut tx = self.backend.begin().await?;
        let status = q("SELECT status FROM mail_user WHERE address = ?1")
            .bind(address)
            .fetch_opt_scalar_string_tx(&mut tx)
            .await?;
        let outcome = match status.as_deref() {
            Some("deleting") => return Ok(MintOutcome::Deleting),
            None | Some("deleted") => {
                q("INSERT INTO mail_user (address, domain, local_part, origin, status, \
                     created_at, first_signin_at, last_signin_at) \
                     VALUES (?1, ?2, ?3, ?4, 'active', ?5, ?5, ?5) \
                     ON CONFLICT(address) DO UPDATE SET domain = excluded.domain, \
                       local_part = excluded.local_part, origin = excluded.origin, \
                       status = 'active', display_name = NULL, note = NULL, \
                       created_at = excluded.created_at, created_by = NULL, \
                       first_signin_at = excluded.first_signin_at, \
                       last_signin_at = excluded.last_signin_at, status_changed_at = NULL, \
                       status_changed_by = NULL, status_reason = NULL, expires_at = NULL, \
                       deleted_at = NULL \
                     WHERE mail_user.status = 'deleted'")
                .bind(address)
                .bind(domain)
                .bind(local_part)
                .bind(origin)
                .bind(now)
                .execute_tx(&mut tx)
                .await?;
                MintOutcome::Created
            }
            Some(_) => {
                // Compare-and-set: a delete that started since the read wins.
                let n = q("UPDATE mail_user SET first_signin_at = COALESCE(first_signin_at, ?2), \
                       last_signin_at = CASE WHEN last_signin_at IS NULL \
                         OR last_signin_at <= ?2 - ?3 THEN ?2 ELSE last_signin_at END \
                     WHERE address = ?1 AND status IN ('active', 'disabled')")
                .bind(address)
                .bind(now)
                .bind(SIGNIN_TOUCH_SECS)
                .execute_tx(&mut tx)
                .await?;
                if n == 0 {
                    return Ok(MintOutcome::Deleting);
                }
                MintOutcome::Existing
            }
        };
        if !account_id.is_empty() {
            map_account_tx(&mut tx, account_id, address, mode, now).await?;
        }
        tx.commit().await?;
        Ok(outcome)
    }

    /// Map `account_id` to `address` (insert, or move it to this address).
    /// `last_seen` is written at most once an hour. An empty id is not mapped.
    pub async fn map_mail_user_account(
        &self,
        account_id: &str,
        address: &str,
        mode: &str,
        now: i64,
    ) -> Result<(), StoreError> {
        if account_id.is_empty() {
            return Ok(());
        }
        let mut tx = self.backend.begin().await?;
        map_account_tx(&mut tx, account_id, address, mode, now).await?;
        tx.commit().await?;
        Ok(())
    }

    /// The address `account_id` is mapped to.
    pub async fn mail_user_address_of(
        &self,
        account_id: &str,
    ) -> Result<Option<String>, StoreError> {
        if account_id.is_empty() {
            return Ok(None);
        }
        Ok(
            q("SELECT address FROM mail_user_account WHERE account_id = ?1")
                .bind(account_id)
                .fetch_opt_scalar_string(&self.backend)
                .await?,
        )
    }

    /// The directory row `account_id` is mapped to.
    pub async fn mail_user_of_account(
        &self,
        account_id: &str,
    ) -> Result<Option<MailUserRow>, StoreError> {
        match self.mail_user_address_of(account_id).await? {
            Some(a) => self.get_mail_user(&a).await,
            None => Ok(None),
        }
    }

    /// The account ids mapped to `address`, oldest first.
    pub async fn mail_user_accounts(
        &self,
        address: &str,
    ) -> Result<Vec<MailUserAccountRow>, StoreError> {
        let rows = q("SELECT account_id, address, mode, first_seen, last_seen \
             FROM mail_user_account WHERE address = ?1 ORDER BY first_seen, account_id")
        .bind(address)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(account_from_row).collect())
    }

    /// The account ids mapped to each of `addresses` (one query; for a page of
    /// the listing).
    pub async fn mail_user_accounts_for(
        &self,
        addresses: &[String],
    ) -> Result<Vec<MailUserAccountRow>, StoreError> {
        let mut out = Vec::new();
        for a in addresses {
            out.extend(self.mail_user_accounts(a).await?);
        }
        Ok(out)
    }

    /// Remove the mapping of `account_id`. Returns whether there was one.
    pub async fn unmap_mail_user_account(&self, account_id: &str) -> Result<bool, StoreError> {
        let n = q("DELETE FROM mail_user_account WHERE account_id = ?1")
            .bind(account_id)
            .execute(&self.backend)
            .await?;
        Ok(n == 1)
    }

    /// Set the status mirror of a row that is not a tombstone. `from`, when
    /// given, is a compare-and-set: the write happens only while the status is one
    /// of those. Returns whether a row changed.
    pub async fn set_mail_user_status(
        &self,
        address: &str,
        status: &str,
        from: Option<&[&str]>,
        reason: Option<&str>,
        by: Option<&str>,
        now: i64,
    ) -> Result<bool, StoreError> {
        if !STATUSES.contains(&status) {
            return Err(StoreError::Corrupt(format!("unknown user status {status}")));
        }
        let allowed = match from {
            Some(list) => format!(",{},", list.join(",")),
            None => ",active,disabled,deleting,".to_string(),
        };
        let sql = match self.backend.dialect() {
            Dialect::Sqlite => {
                "UPDATE mail_user SET status = ?2, status_reason = ?3, status_changed_by = ?4, \
                 status_changed_at = ?5, deleted_at = CASE WHEN ?2 = 'deleted' THEN ?5 ELSE NULL END \
                 WHERE address = ?1 AND instr(?6, ',' || status || ',') > 0"
            }
            Dialect::Postgres => {
                "UPDATE mail_user SET status = ?2, status_reason = ?3, status_changed_by = ?4, \
                 status_changed_at = ?5, deleted_at = CASE WHEN ?2 = 'deleted' THEN ?5 ELSE NULL END \
                 WHERE address = ?1 AND strpos(?6, ',' || status || ',') > 0"
            }
        };
        let n = q(sql)
            .bind(address)
            .bind(status)
            .bind(reason)
            .bind(by)
            .bind(now)
            .bind(allowed)
            .execute(&self.backend)
            .await?;
        Ok(n == 1)
    }

    /// Turn a row into a tombstone after its data was erased: status `deleted`,
    /// the personal fields cleared, address, domain and dates kept.
    pub async fn tombstone_mail_user(
        &self,
        address: &str,
        by: Option<&str>,
        now: i64,
    ) -> Result<bool, StoreError> {
        let n = q("UPDATE mail_user SET status = 'deleted', deleted_at = ?2, \
             status_changed_at = ?2, status_changed_by = ?3, display_name = NULL, note = NULL, \
             status_reason = NULL WHERE address = ?1")
        .bind(address)
        .bind(now)
        .bind(by)
        .execute(&self.backend)
        .await?;
        q("DELETE FROM mail_user_account WHERE address = ?1")
            .bind(address)
            .execute(&self.backend)
            .await?;
        Ok(n == 1)
    }

    /// One page of the directory and the offset of the next page (`None` at the
    /// end). `sort` and `desc` order it; the address breaks ties.
    pub async fn list_mail_users(
        &self,
        filter: &MailUserFilter,
        sort: MailUserSort,
        desc: bool,
        offset: i64,
        limit: i64,
    ) -> Result<(Vec<MailUserRow>, Option<i64>), StoreError> {
        let limit = limit.clamp(1, 500);
        let offset = offset.max(0);
        let sql = match self.backend.dialect() {
            Dialect::Sqlite => LIST_SQLITE,
            Dialect::Postgres => LIST_PG,
        };
        let rows = bind_filter(q(sql), filter)
            .bind(sort.as_str())
            .bind(i64::from(desc))
            .bind(limit + 1)
            .bind(offset)
            .fetch_all(&self.backend)
            .await?;
        let mut out: Vec<MailUserRow> = rows.iter().map(user_from_row).collect();
        let next = if out.len() as i64 > limit {
            out.truncate(limit as usize);
            Some(offset + limit)
        } else {
            None
        };
        Ok((out, next))
    }

    /// How many rows `filter` matches.
    pub async fn count_mail_users(&self, filter: &MailUserFilter) -> Result<i64, StoreError> {
        let sql = match self.backend.dialect() {
            Dialect::Sqlite => COUNT_SQLITE,
            Dialect::Postgres => COUNT_PG,
        };
        Ok(bind_filter(q(sql), filter)
            .fetch_scalar_i64(&self.backend)
            .await?)
    }

    /// Every live (not deleted) row, in address order. For the boot reconcile
    /// and the policy impact counts, not for a request path.
    pub async fn all_mail_users(&self) -> Result<Vec<MailUserRow>, StoreError> {
        let rows = q("SELECT address, domain, local_part, origin, status, display_name, note, \
             created_at, created_by, first_signin_at, last_signin_at, status_changed_at, \
             status_changed_by, status_reason, expires_at, deleted_at \
             FROM mail_user WHERE status <> 'deleted' ORDER BY address")
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(user_from_row).collect())
    }

    /// The dashboard's directory counts over `domains` (`None` = every row; an
    /// empty list counts nothing; bare names are counted only for `None`).
    pub async fn mail_user_counts(
        &self,
        domains: Option<&[String]>,
        now: i64,
    ) -> Result<MailUserCounts, StoreError> {
        let mut c = MailUserCounts::default();
        let week_ago = now - 7 * 86_400;
        for u in self.all_mail_users().await? {
            if let Some(scope) = domains
                && (u.domain.is_empty() || !scope.iter().any(|d| d.eq_ignore_ascii_case(&u.domain)))
            {
                continue;
            }
            c.total += 1;
            match u.status.as_str() {
                "active" => c.active += 1,
                "disabled" => c.disabled += 1,
                _ => {}
            }
            if u.origin == "provisioned" && u.first_signin_at.is_none() {
                c.provisioned_never_signed_in += 1;
            }
            if u.created_at >= week_ago {
                c.new_7d += 1;
            }
        }
        Ok(c)
    }

    /// Mark every live `signin`/`import` row of `domain` (`None`: every domain)
    /// `provisioned` (`POST /admin/users/adopt`). Returns how many changed.
    pub async fn adopt_mail_users(&self, domain: Option<&str>) -> Result<u64, StoreError> {
        Ok(q("UPDATE mail_user SET origin = 'provisioned' \
             WHERE origin IN ('signin', 'import') AND status IN ('active', 'disabled') \
             AND (?1 IS NULL OR domain = ?1)")
        .bind(domain)
        .execute(&self.backend)
        .await?)
    }

    /// The earliest sign-in record this server holds (`historyFrom` of the policy
    /// impact, t32-q A13): the oldest of the directory's first sign-ins and the
    /// engine accounts' creation.
    pub async fn mail_user_history_from(&self) -> Result<Option<i64>, StoreError> {
        let row = q("SELECT MIN(COALESCE(first_signin_at, created_at)) AS t FROM mail_user \
             WHERE origin <> 'provisioned' OR first_signin_at IS NOT NULL")
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.and_then(|r| r.get_opt_i64("t")))
    }
}

fn bind_filter(sql: crate::backend::Sql, f: &MailUserFilter) -> crate::backend::Sql {
    sql.bind(f.q.as_deref().filter(|s| !s.trim().is_empty()).map(like_term))
        .bind(f.domain.clone())
        .bind(scope_list(&f.domains))
        .bind(f.status.clone())
        .bind(f.origin.clone())
        .bind(i64::from(f.include_deleted))
}

async fn map_account_tx(
    tx: &mut crate::backend::Tx,
    account_id: &str,
    address: &str,
    mode: &str,
    now: i64,
) -> Result<(), StoreError> {
    if !ACCOUNT_MODES.contains(&mode) {
        return Err(StoreError::Corrupt(format!("unknown account mode {mode}")));
    }
    q("INSERT INTO mail_user_account (account_id, address, mode, first_seen, last_seen) \
         VALUES (?1, ?2, ?3, ?4, ?4) \
         ON CONFLICT(account_id) DO UPDATE SET address = excluded.address, mode = excluded.mode, \
           last_seen = CASE WHEN mail_user_account.address <> excluded.address \
             OR mail_user_account.last_seen <= excluded.last_seen - ?5 \
             THEN excluded.last_seen ELSE mail_user_account.last_seen END")
    .bind(account_id)
    .bind(address)
    .bind(mode)
    .bind(now)
    .bind(SIGNIN_TOUCH_SECS)
    .execute_tx(tx)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServerKey;

    async fn store() -> Store {
        Store::open_in_memory(ServerKey::generate()).await.unwrap()
    }

    fn user(address: &str, origin: &str, now: i64) -> MailUserRow {
        let (local, domain) = address.rsplit_once('@').unwrap_or((address, ""));
        MailUserRow::new(address, domain, local, origin, now)
    }

    #[tokio::test]
    async fn directory_round_trip() {
        directory(&store().await).await;
    }

    #[tokio::test]
    async fn mint_and_mapping() {
        mint(&store().await).await;
    }

    #[tokio::test]
    async fn listing_filters_sorts_and_pages() {
        listing(&store().await).await;
    }

    async fn directory(s: &Store) {
        assert_eq!(s.get_mail_user("a@d.test").await.unwrap(), None);
        let mut a = user("a@d.test", "provisioned", 100);
        a.display_name = Some("Alice".into());
        a.created_by = Some("root".into());
        assert!(s.insert_mail_user(&a).await.unwrap());
        assert_eq!(s.get_mail_user("a@d.test").await.unwrap().as_ref(), Some(&a));
        // A second insert over a live row writes nothing.
        let other = user("a@d.test", "signin", 200);
        assert!(!s.insert_mail_user(&other).await.unwrap());
        assert_eq!(s.get_mail_user("a@d.test").await.unwrap().unwrap().origin, "provisioned");

        assert!(
            s.update_mail_user_details("a@d.test", Some("A"), Some("n"), Some(5))
                .await
                .unwrap()
        );
        let got = s.get_mail_user("a@d.test").await.unwrap().unwrap();
        assert_eq!((got.display_name.as_deref(), got.note.as_deref(), got.expires_at), (Some("A"), Some("n"), Some(5)));

        // Status mirror with and without a compare-and-set.
        assert!(
            s.set_mail_user_status("a@d.test", "disabled", None, Some("why"), Some("root"), 300)
                .await
                .unwrap()
        );
        let got = s.get_mail_user("a@d.test").await.unwrap().unwrap();
        assert_eq!(got.status, "disabled");
        assert_eq!(got.status_reason.as_deref(), Some("why"));
        assert_eq!(got.status_changed_at, Some(300));
        assert!(
            !s.set_mail_user_status("a@d.test", "deleting", Some(&["active"]), None, None, 301)
                .await
                .unwrap(),
            "compare-and-set refuses a status not listed"
        );
        assert!(
            s.set_mail_user_status("a@d.test", "deleting", Some(&["active", "disabled"]), None, None, 302)
                .await
                .unwrap()
        );
        assert!(s.set_mail_user_status("a@d.test", "bogus", None, None, None, 1).await.is_err());

        // Tombstone, then reuse by provisioning.
        s.map_mail_user_account("acct-1", "a@d.test", "engine", 300)
            .await
            .unwrap();
        assert!(s.tombstone_mail_user("a@d.test", Some("root"), 400).await.unwrap());
        let t = s.get_mail_user("a@d.test").await.unwrap().unwrap();
        assert_eq!((t.status.as_str(), t.deleted_at, t.display_name), ("deleted", Some(400), None));
        assert!(s.mail_user_accounts("a@d.test").await.unwrap().is_empty());
        assert!(
            !s.set_mail_user_status("a@d.test", "active", None, None, None, 401)
                .await
                .unwrap(),
            "a tombstone is not revived by the status mirror"
        );
        let again = user("a@d.test", "provisioned", 500);
        assert!(s.insert_mail_user(&again).await.unwrap());
        let got = s.get_mail_user("a@d.test").await.unwrap().unwrap();
        assert_eq!((got.status.as_str(), got.created_at, got.deleted_at), ("active", 500, None));
    }

    async fn mint(s: &Store) {
        // A first sign-in creates the row with its origin and maps the account.
        assert_eq!(
            s.mint_mail_user("b@d.test", "d.test", "b", "signin", "acct-b", "engine", 1000)
                .await
                .unwrap(),
            MintOutcome::Created
        );
        let b = s.get_mail_user("b@d.test").await.unwrap().unwrap();
        assert_eq!((b.origin.as_str(), b.first_signin_at, b.last_signin_at), ("signin", Some(1000), Some(1000)));
        assert_eq!(s.mail_user_address_of("acct-b").await.unwrap().as_deref(), Some("b@d.test"));

        // A provisioned row keeps its origin (A12); last sign-in at most hourly.
        let p = user("p@d.test", "provisioned", 10);
        s.insert_mail_user(&p).await.unwrap();
        for (now, expect_last) in [(2000, 2000), (2100, 2000), (2000 + SIGNIN_TOUCH_SECS, 2000 + SIGNIN_TOUCH_SECS)] {
            assert_eq!(
                s.mint_mail_user("p@d.test", "d.test", "p", "signin", "acct-p", "proxy", now)
                    .await
                    .unwrap(),
                MintOutcome::Existing
            );
            let got = s.get_mail_user("p@d.test").await.unwrap().unwrap();
            assert_eq!(got.origin, "provisioned");
            assert_eq!(got.first_signin_at, Some(2000));
            assert_eq!(got.last_signin_at, Some(expect_last), "at {now}");
        }

        // `deleting` refuses the mint and writes nothing.
        s.set_mail_user_status("p@d.test", "deleting", None, None, None, 3)
            .await
            .unwrap();
        assert_eq!(
            s.mint_mail_user("p@d.test", "d.test", "p", "signin", "acct-new", "engine", 9000)
                .await
                .unwrap(),
            MintOutcome::Deleting
        );
        assert_eq!(s.mail_user_address_of("acct-new").await.unwrap(), None);

        // A tombstone is reused as a fresh self-signed user with dates reset.
        s.tombstone_mail_user("p@d.test", None, 9100).await.unwrap();
        assert_eq!(
            s.mint_mail_user("p@d.test", "d.test", "p", "signin", "acct-p2", "engine", 9200)
                .await
                .unwrap(),
            MintOutcome::Created
        );
        let got = s.get_mail_user("p@d.test").await.unwrap().unwrap();
        assert_eq!((got.origin.as_str(), got.status.as_str(), got.created_at, got.first_signin_at), ("signin", "active", 9200, Some(9200)));

        // Mapping: moves to another address; unmap; empty id ignored; bad mode refused.
        s.map_mail_user_account("acct-b", "b2@d.test", "proxy", 1100).await.unwrap();
        assert_eq!(s.mail_user_address_of("acct-b").await.unwrap().as_deref(), Some("b2@d.test"));
        s.map_mail_user_account("", "b@d.test", "proxy", 1).await.unwrap();
        assert_eq!(s.mail_user_address_of("").await.unwrap(), None);
        assert!(s.map_mail_user_account("x", "b@d.test", "weird", 1).await.is_err());
        assert!(s.unmap_mail_user_account("acct-b").await.unwrap());
        assert!(!s.unmap_mail_user_account("acct-b").await.unwrap());
        assert_eq!(
            s.mail_user_of_account("acct-p2").await.unwrap().map(|u| u.address),
            Some("p@d.test".to_string())
        );
    }

    async fn listing(s: &Store) {
        for (addr, origin, created) in [
            ("ann@one.test", "signin", 10),
            ("bob@one.test", "provisioned", 20),
            ("cat@two.test", "import", 30),
            ("dan", "signin", 40),
            ("eve@two.test", "sso", 50),
        ] {
            s.insert_mail_user(&user(addr, origin, created)).await.unwrap();
        }
        s.set_mail_user_status("cat@two.test", "disabled", None, None, None, 1).await.unwrap();
        s.insert_mail_user(&user("zed@one.test", "signin", 60)).await.unwrap();
        s.tombstone_mail_user("zed@one.test", None, 70).await.unwrap();

        let addrs = |rows: &[MailUserRow]| rows.iter().map(|r| r.address.clone()).collect::<Vec<_>>();
        let all = MailUserFilter::default();
        let (rows, next) = s.list_mail_users(&all, MailUserSort::Address, false, 0, 50).await.unwrap();
        assert_eq!(addrs(&rows), ["ann@one.test", "bob@one.test", "cat@two.test", "dan", "eve@two.test"]);
        assert_eq!(next, None);
        assert_eq!(s.count_mail_users(&all).await.unwrap(), 5);

        // Paging.
        let (p1, next) = s.list_mail_users(&all, MailUserSort::Address, false, 0, 2).await.unwrap();
        assert_eq!(addrs(&p1), ["ann@one.test", "bob@one.test"]);
        let (p2, _) = s.list_mail_users(&all, MailUserSort::Address, false, next.unwrap(), 2).await.unwrap();
        assert_eq!(addrs(&p2), ["cat@two.test", "dan"]);

        // Sorts.
        let (rows, _) = s.list_mail_users(&all, MailUserSort::Created, true, 0, 50).await.unwrap();
        assert_eq!(addrs(&rows)[0], "eve@two.test");
        let (rows, _) = s.list_mail_users(&all, MailUserSort::Address, true, 0, 50).await.unwrap();
        assert_eq!(addrs(&rows)[0], "eve@two.test");

        // Filters.
        let f = |m: fn(&mut MailUserFilter)| {
            let mut f = MailUserFilter::default();
            m(&mut f);
            f
        };
        let (rows, _) = s.list_mail_users(&f(|f| f.q = Some("AN".into())), MailUserSort::Address, false, 0, 50).await.unwrap();
        assert_eq!(addrs(&rows), ["ann@one.test", "dan"]);
        let (rows, _) = s.list_mail_users(&f(|f| f.q = Some("%".into())), MailUserSort::Address, false, 0, 50).await.unwrap();
        assert!(rows.is_empty(), "% matches only itself");
        let (rows, _) = s.list_mail_users(&f(|f| f.domain = Some("two.test".into())), MailUserSort::Address, false, 0, 50).await.unwrap();
        assert_eq!(addrs(&rows), ["cat@two.test", "eve@two.test"]);
        let (rows, _) = s.list_mail_users(&f(|f| f.domains = Some(vec!["ONE.test".into()])), MailUserSort::Address, false, 0, 50).await.unwrap();
        assert_eq!(addrs(&rows), ["ann@one.test", "bob@one.test"], "scope: bare names never in a domain scope");
        let (rows, _) = s.list_mail_users(&f(|f| f.domains = Some(vec![])), MailUserSort::Address, false, 0, 50).await.unwrap();
        assert!(rows.is_empty());
        let (rows, _) = s.list_mail_users(&f(|f| f.status = Some("disabled".into())), MailUserSort::Address, false, 0, 50).await.unwrap();
        assert_eq!(addrs(&rows), ["cat@two.test"]);
        let (rows, _) = s.list_mail_users(&f(|f| f.status = Some("deleted".into())), MailUserSort::Address, false, 0, 50).await.unwrap();
        assert_eq!(addrs(&rows), ["zed@one.test"]);
        let (rows, _) = s.list_mail_users(&f(|f| f.origin = Some("signin".into())), MailUserSort::Address, false, 0, 50).await.unwrap();
        assert_eq!(addrs(&rows), ["ann@one.test", "dan"]);

        let c = s.mail_user_counts(None, 60).await.unwrap();
        assert_eq!((c.total, c.active, c.disabled, c.provisioned_never_signed_in), (5, 4, 1, 1));
        let c = s.mail_user_counts(Some(&["two.test".to_string()]), 60).await.unwrap();
        assert_eq!(c.total, 2);

        assert_eq!(s.adopt_mail_users(Some("one.test")).await.unwrap(), 1);
        assert_eq!(s.get_mail_user("ann@one.test").await.unwrap().unwrap().origin, "provisioned");
        assert_eq!(s.adopt_mail_users(None).await.unwrap(), 2, "cat (import) and dan (signin)");
    }

    /// `is_live` and `has_signed_in` (t32-q A20, `closed`).
    #[test]
    fn liveness_and_history() {
        let mut u = user("x@d.test", "provisioned", 1);
        assert!(u.is_live(10) && !u.has_signed_in());
        u.expires_at = Some(5);
        assert!(!u.is_live(10), "a provisioned record past its expiry lapses unused");
        u.first_signin_at = Some(3);
        assert!(u.is_live(10) && u.has_signed_in());
        u.status = "deleting".into();
        assert!(!u.is_live(10));
        assert!(user("y", "import", 1).has_signed_in());
    }

    fn pg_dsn() -> Option<String> {
        std::env::var("DATABASE_URL_PG")
            .ok()
            .or_else(|| std::env::var("MW_TEST_PG").ok())
            .filter(|s| !s.trim().is_empty())
    }

    /// The Postgres legs (`MW_REQUIRE_LIVE` turns a skip into a failure).
    #[tokio::test]
    async fn postgres_legs() {
        let Some(dsn) = pg_dsn() else {
            mw_test_gate::skip(
                "[mw-store] mail_users Postgres legs (set DATABASE_URL_PG or MW_TEST_PG to a live \
                 postgres:16 to run them). The SQLite path still asserted.",
            );
            return;
        };
        let s = Store::open_postgres(&dsn, ServerKey::generate())
            .await
            .expect("DATABASE_URL_PG is set but Postgres is not reachable");
        for t in ["mail_user_account", "mail_user"] {
            let sql: &'static str = if t == "mail_user" {
                "DELETE FROM mail_user"
            } else {
                "DELETE FROM mail_user_account"
            };
            q(sql).execute(&s.backend).await.unwrap();
        }
        directory(&s).await;
        q("DELETE FROM mail_user").execute(&s.backend).await.unwrap();
        q("DELETE FROM mail_user_account").execute(&s.backend).await.unwrap();
        mint(&s).await;
        q("DELETE FROM mail_user").execute(&s.backend).await.unwrap();
        q("DELETE FROM mail_user_account").execute(&s.backend).await.unwrap();
        listing(&s).await;
    }
}
