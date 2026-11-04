//! Store side of the first-sign-in rule (`signin.newUsers`): what the user
//! directory already holds about an address, and the pre-provisioned ("pending")
//! entries an administrator adds in advance.
//!
//! The rule itself (precedence, per-type matrix, the decision) lives in
//! `mw-server::signin_policy`. A pending entry is a `mail_user` row with origin
//! `provisioned` (admin-added) or `import` (imported / directory-synced) that has
//! no `first_signin_at` yet; `POST /admin/users` provisioning (`admin_users`) also
//! counts. Domain-wide entries (`*@domain` with a use cap) live in `signin_allow`.
use crate::{Store, StoreError, q};

/// What the directory holds for one address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryStanding {
    /// `provisioned`, `signin`, `sso` or `import` (`mail_user.origin`).
    pub origin: String,
    /// `active`, `disabled`, `deleting` or `deleted`.
    pub status: String,
    /// Set once the address has authenticated here at least once.
    pub first_signin_at: Option<i64>,
    /// A pending entry stops admitting at this time (unix seconds).
    pub expires_at: Option<i64>,
}

impl DirectoryStanding {
    /// The address has signed in before: it is an existing user, not a new one.
    pub fn has_signed_in(&self) -> bool {
        self.first_signin_at.is_some()
    }

    /// A pending entry: created in advance (`provisioned` or `import`), never
    /// signed in, active and not expired at `now`.
    pub fn is_pending(&self, now: i64) -> bool {
        matches!(self.origin.as_str(), "provisioned" | "import")
            && self.first_signin_at.is_none()
            && self.status == "active"
            && self.expires_at.is_none_or(|t| t > now)
    }

    /// Pending and added by an administrator (not imported or synced).
    pub fn is_pending_admin_entry(&self, now: i64) -> bool {
        self.origin == "provisioned" && self.is_pending(now)
    }
}

/// A live domain-wide entry that admits one address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllowEntry {
    pub id: String,
}

/// One pre-provisioned directory entry as the admin list shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingUser {
    pub address: String,
    pub domain: String,
    pub origin: String,
    pub note: Option<String>,
    pub created_at: i64,
    pub created_by: Option<String>,
    pub expires_at: Option<i64>,
    /// Set once a first sign-in consumed the entry.
    pub first_signin_at: Option<i64>,
    pub status: String,
}

fn glob(pattern: &str, text: &str) -> bool {
    let pattern = pattern.to_lowercase();
    let text = text.to_lowercase();
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == text;
    }
    let Some(mut tail) = text.strip_prefix(parts[0]) else {
        return false;
    };
    for part in &parts[1..parts.len() - 1] {
        let Some(index) = tail.find(part) else {
            return false;
        };
        tail = &tail[index + part.len()..];
    }
    tail.ends_with(parts[parts.len() - 1])
}

impl Store {
    /// The directory row for any of the given spellings of an address, if any.
    pub async fn directory_standing(
        &self,
        addresses: &[&str],
    ) -> Result<Option<DirectoryStanding>, StoreError> {
        let mut best: Option<DirectoryStanding> = None;
        for address in addresses {
            let Some(row) = q(
                "SELECT origin,status,first_signin_at,expires_at FROM mail_user WHERE address=?1",
            )
            .bind(*address)
            .fetch_optional(&self.backend)
            .await?
            else {
                continue;
            };
            let found = DirectoryStanding {
                origin: row.get_string("origin"),
                status: row.get_string("status"),
                first_signin_at: row.get_opt_i64("first_signin_at"),
                expires_at: row.get_opt_i64("expires_at"),
            };
            // A row that proves a prior sign-in outranks one that does not.
            if best.as_ref().is_none_or(|b| !b.has_signed_in()) {
                best = Some(found);
            }
        }
        if best.is_none() {
            // `POST /admin/users` provisions into `admin_users` (the username is the
            // address); the directory row appears at first sign-in or first
            // lifecycle change. That is a pending pre-provisioned entry too.
            for address in addresses {
                if q("SELECT username FROM admin_users WHERE lower(username)=lower(?1)")
                    .bind(*address)
                    .fetch_optional(&self.backend)
                    .await?
                    .is_some()
                {
                    best = Some(DirectoryStanding {
                        origin: "provisioned".into(),
                        status: "active".into(),
                        first_signin_at: None,
                        expires_at: None,
                    });
                    break;
                }
            }
        }
        Ok(best)
    }

    /// The first live `signin_allow` entry of `domain` whose local-part glob
    /// matches `local_part` (not expired, uses left).
    pub async fn signin_allow_match(
        &self,
        domain: &str,
        local_part: &str,
        now: i64,
    ) -> Result<Option<AllowEntry>, StoreError> {
        let rows = q("SELECT id,pattern FROM signin_allow \
             WHERE domain=?1 AND (expires_at IS NULL OR expires_at>?2) \
             AND (max_uses IS NULL OR uses<max_uses) ORDER BY created_at,id LIMIT 1001")
        .bind(domain)
        .bind(now)
        .fetch_all(&self.backend)
        .await?;
        if rows.len() > 1000 {
            return Err(StoreError::Corrupt(
                "sign-in allow entries exceed bounded lookup limit".into(),
            ));
        }
        Ok(rows
            .into_iter()
            .find(|row| glob(&row.get_string("pattern"), local_part))
            .map(|row| AllowEntry {
                id: row.get_string("id"),
            }))
    }

    /// Spend one use of a domain-wide entry. Compare-and-swap on the remaining
    /// uses: concurrent sign-ins cannot exceed the cap.
    pub async fn consume_signin_allow(&self, id: &str, now: i64) -> Result<bool, StoreError> {
        let n = q("UPDATE signin_allow SET uses=uses+1 \
             WHERE id=?1 AND (expires_at IS NULL OR expires_at>?2) \
             AND (max_uses IS NULL OR uses<max_uses)")
        .bind(id)
        .bind(now)
        .execute(&self.backend)
        .await?;
        Ok(n > 0)
    }

    /// Create or replace a domain-wide entry (`pattern` is a local-part glob).
    #[allow(clippy::too_many_arguments)]
    pub async fn put_signin_allow(
        &self,
        id: &str,
        domain: &str,
        pattern: &str,
        note: Option<&str>,
        expires_at: Option<i64>,
        max_uses: Option<i64>,
        created_by: &str,
        now: i64,
    ) -> Result<(), StoreError> {
        q("INSERT INTO signin_allow(id,domain,pattern,note,expires_at,max_uses,uses,created_by,created_at) \
           VALUES(?1,?2,?3,?4,?5,?6,0,?7,?8) \
           ON CONFLICT(domain,pattern) DO UPDATE SET note=excluded.note,expires_at=excluded.expires_at,max_uses=excluded.max_uses")
        .bind(id)
        .bind(domain)
        .bind(pattern)
        .bind(note.map(str::to_owned))
        .bind(expires_at)
        .bind(max_uses)
        .bind(created_by)
        .bind(now)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Add a pending pre-provisioned entry. `false` when the address already has a
    /// directory row (nothing is overwritten).
    pub async fn put_provisioned_user(
        &self,
        address: &str,
        origin: &str,
        note: Option<&str>,
        expires_at: Option<i64>,
        created_by: &str,
        now: i64,
    ) -> Result<bool, StoreError> {
        if !matches!(origin, "provisioned" | "import") {
            return Err(StoreError::Corrupt("invalid pending origin".into()));
        }
        let (local, domain) = address.rsplit_once('@').unwrap_or((address, ""));
        let n = q("INSERT INTO mail_user(address,domain,local_part,origin,status,note,created_at,created_by,expires_at) \
           VALUES(?1,?2,?3,?4,'active',?5,?6,?7,?8) ON CONFLICT(address) DO NOTHING")
        .bind(address)
        .bind(domain)
        .bind(local)
        .bind(origin)
        .bind(note.map(str::to_owned))
        .bind(now)
        .bind(created_by)
        .bind(expires_at)
        .execute(&self.backend)
        .await?;
        Ok(n == 1)
    }

    /// Pre-provisioned entries (pending and consumed), newest first, bounded.
    /// `domains` limits the list for a domain-scoped admin (`None` = all).
    pub async fn list_pending_users(
        &self,
        domains: Option<&[String]>,
        limit: i64,
    ) -> Result<Vec<PendingUser>, StoreError> {
        let rows = q("SELECT address,domain,origin,note,created_at,created_by,expires_at,first_signin_at,status \
             FROM mail_user WHERE origin IN ('provisioned','import') AND created_by IS NOT NULL \
             ORDER BY created_at DESC,address LIMIT ?1")
        .bind(limit.clamp(1, 1000))
        .fetch_all(&self.backend)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| PendingUser {
                address: r.get_string("address"),
                domain: r.get_string("domain"),
                origin: r.get_string("origin"),
                note: r.get_opt_string("note"),
                created_at: r.get_i64("created_at"),
                created_by: r.get_opt_string("created_by"),
                expires_at: r.get_opt_i64("expires_at"),
                first_signin_at: r.get_opt_i64("first_signin_at"),
                status: r.get_string("status"),
            })
            .filter(|p| {
                domains.is_none_or(|d| d.iter().any(|x| x.eq_ignore_ascii_case(&p.domain)))
            })
            .collect())
    }

    /// Change a pending entry's expiry. `false` when it is not pending.
    pub async fn extend_pending_user(
        &self,
        address: &str,
        expires_at: Option<i64>,
    ) -> Result<bool, StoreError> {
        let n = q("UPDATE mail_user SET expires_at=?2 WHERE address=?1 AND first_signin_at IS NULL \
                   AND origin IN ('provisioned','import') AND status='active'")
        .bind(address)
        .bind(expires_at)
        .execute(&self.backend)
        .await?;
        Ok(n == 1)
    }

    /// Remove a pending entry (before first sign-in). `false` when it is not pending.
    pub async fn revoke_pending_user(&self, address: &str) -> Result<bool, StoreError> {
        let n = q("DELETE FROM mail_user WHERE address=?1 AND first_signin_at IS NULL \
                   AND origin IN ('provisioned','import') AND status='active'")
        .bind(address)
        .execute(&self.backend)
        .await?;
        Ok(n == 1)
    }

    /// Delete pending entries that expired before `before` (cleanup, bounded).
    pub async fn purge_expired_pending(&self, before: i64) -> Result<u64, StoreError> {
        Ok(q("DELETE FROM mail_user WHERE address IN (SELECT address FROM mail_user \
              WHERE first_signin_at IS NULL AND origin IN ('provisioned','import') \
              AND expires_at IS NOT NULL AND expires_at<?1 LIMIT 1000)")
        .bind(before)
        .execute(&self.backend)
        .await?)
    }

    /// Consume a pending entry at first sign-in. Compare-and-swap on
    /// `first_signin_at IS NULL`: of two concurrent sign-ins exactly one gets `true`.
    pub async fn consume_pending_user(&self, address: &str, now: i64) -> Result<bool, StoreError> {
        let n = q("UPDATE mail_user SET first_signin_at=?2 WHERE address=?1 AND first_signin_at IS NULL \
                   AND origin IN ('provisioned','import') AND status='active' \
                   AND (expires_at IS NULL OR expires_at>?2)")
        .bind(address)
        .bind(now)
        .execute(&self.backend)
        .await?;
        Ok(n == 1)
    }
}

#[cfg(test)]
mod tests {
    use super::glob;
    use crate::{ServerKey, Store};

    #[test]
    fn local_part_globs() {
        assert!(glob("*", "anyone"));
        assert!(glob("staff.*", "Staff.Ada"));
        assert!(!glob("staff.*", "ada"));
        assert!(glob("ada", "ADA"));
        assert!(!glob("ada", "adam"));
    }

    #[tokio::test]
    async fn pending_entry_is_consumed_once_and_expires() {
        let s = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        assert!(s.put_provisioned_user("ada@x.test", "provisioned", Some("n"), Some(100), "root", 1).await.unwrap());
        assert!(!s.put_provisioned_user("ada@x.test", "provisioned", None, None, "root", 1).await.unwrap());
        let st = s.directory_standing(&["ada@x.test"]).await.unwrap().unwrap();
        assert!(st.is_pending(50) && st.is_pending_admin_entry(50));
        assert!(!st.is_pending(100), "expired");
        assert!(s.extend_pending_user("ada@x.test", Some(500)).await.unwrap());
        // two concurrent sign-ins: exactly one consumes
        let (a, b) = tokio::join!(s.consume_pending_user("ada@x.test", 60), s.consume_pending_user("ada@x.test", 60));
        assert_eq!([a.unwrap(), b.unwrap()].iter().filter(|x| **x).count(), 1);
        let st = s.directory_standing(&["ada@x.test"]).await.unwrap().unwrap();
        assert!(st.has_signed_in() && !st.is_pending(60));
        assert!(!s.revoke_pending_user("ada@x.test").await.unwrap(), "consumed entries are not revocable");
    }

    #[tokio::test]
    async fn revoke_purge_and_domain_wide_cap() {
        let s = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        s.put_provisioned_user("a@x.test", "provisioned", None, Some(10), "root", 1).await.unwrap();
        s.put_provisioned_user("b@x.test", "provisioned", None, None, "root", 1).await.unwrap();
        assert!(s.revoke_pending_user("b@x.test").await.unwrap());
        assert_eq!(s.purge_expired_pending(11).await.unwrap(), 1);
        assert!(s.directory_standing(&["a@x.test"]).await.unwrap().is_none());
        s.put_signin_allow("w1", "x.test", "*", None, None, Some(1), "root", 1).await.unwrap();
        assert!(s.signin_allow_match("x.test", "zed", 5).await.unwrap().is_some());
        let (a, b) = tokio::join!(s.consume_signin_allow("w1", 5), s.consume_signin_allow("w1", 5));
        assert_eq!([a.unwrap(), b.unwrap()].iter().filter(|x| **x).count(), 1);
        assert!(s.signin_allow_match("x.test", "zed", 5).await.unwrap().is_none());
    }
}
