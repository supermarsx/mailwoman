//! 0033 admin accounts, roles and the admin TOTP factor (26.20 t32-e24c, f02).
//!
//! `admin_account` holds the people who may sign in to the admin console; mail
//! users live elsewhere (`admin_users`, 0007, is the provisioned-mail-user table
//! despite its name). Passwords are Argon2id PHC strings, hashed by the caller.
//! New environment bootstrap accounts persist an Argon2id hash. A NULL hash
//! remains supported for legacy environment-bootstrap rows until password reset.
//!
//! Roles: `admin_role` (+ `admin_role_capability` for custom roles; a built-in
//! role's capabilities are defined in code, `mw-server/src/capabilities.rs`),
//! `admin_account_role` (`domains_json` NULL = all domains) and
//! `admin_capability_override` (`allow` | `deny`). Capability ids are opaque here.
//!
//! The TOTP shared secret is sealed under the store `ServerKey`, like the mailbox
//! factor (`totp_secrets`); `last_step` is the replay guard, advanced by
//! compare-and-swap.
//!
//! Authored in the SQLite `?n` style so it runs identically on SQLite or Postgres
//! through [`crate::backend`].

use std::fmt;

use crate::backend::q;
use crate::{Store, StoreError};

/// One admin account (0033 `admin_account`).
#[derive(Clone, PartialEq, Eq)]
pub struct AdminAccountRow {
    pub id: String,
    pub username: String,
    pub display_name: String,
    /// Argon2id PHC string; `None` only for legacy environment-bootstrap rows.
    pub password_hash: Option<String>,
    pub disabled: bool,
    /// `db` | `env-bootstrap`.
    pub source: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_login_at: Option<i64>,
}

/// Hand-written so the password hash never reaches a log line.
impl fmt::Debug for AdminAccountRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AdminAccountRow")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("display_name", &self.display_name)
            .field(
                "password_hash",
                &self.password_hash.as_ref().map(|_| "<redacted>"),
            )
            .field("disabled", &self.disabled)
            .field("source", &self.source)
            .finish()
    }
}

/// One role (0033 `admin_role`), with its stored capabilities (empty for a
/// built-in role, whose capabilities are defined in code).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminRoleRow {
    pub id: String,
    pub name: String,
    pub builtin: bool,
    pub capabilities: Vec<String>,
}

/// One role assignment: the role and the domains it is limited to (`None` = all).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminRoleGrant {
    pub role_id: String,
    pub domains: Option<Vec<String>>,
}

/// One live admin session (0007 `admin_sessions` + 0030 deadlines).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminSessionRow {
    pub token_hash: String,
    pub admin_id: String,
    pub created_at: String,
    pub expires_at: i64,
    pub absolute_expires_at: i64,
}

fn account(r: &crate::Row) -> AdminAccountRow {
    AdminAccountRow {
        id: r.get_string("id"),
        username: r.get_string("username"),
        display_name: r.get_string("display_name"),
        password_hash: r.get_opt_string("password_hash"),
        disabled: r.get_i64("disabled") != 0,
        source: r.get_string("source"),
        created_at: r.get_i64("created_at"),
        updated_at: r.get_i64("updated_at"),
        last_login_at: r.get_opt_i64("last_login_at"),
    }
}

// Updating an existing singleton role acquires a write lock on SQLite and a
// row lock on Postgres until commit. Every guarded mutation takes this lock
// before reading the invariant, so two servers cannot both remove a survivor.
async fn lock_admin_accounts(tx: &mut crate::backend::Tx) -> Result<(), StoreError> {
    q("UPDATE admin_role SET name = name WHERE id = 'super_admin'")
        .execute_tx(tx)
        .await?;
    Ok(())
}

async fn is_last_super(tx: &mut crate::backend::Tx, id: &str) -> Result<bool, StoreError> {
    Ok(q("SELECT a.id FROM admin_account a JOIN admin_account_role r ON r.account_id = a.id WHERE a.id = ?1 AND a.disabled = 0 AND r.role_id = 'super_admin' AND r.domains_json IS NULL AND NOT EXISTS (SELECT 1 FROM admin_account b JOIN admin_account_role s ON s.account_id = b.id WHERE b.id <> ?1 AND b.disabled = 0 AND s.role_id = 'super_admin' AND s.domains_json IS NULL AND NOT EXISTS (SELECT 1 FROM admin_capability_override o WHERE o.account_id = b.id AND o.effect = 'deny'))")
        .bind(id).fetch_opt_scalar_string_tx(tx).await?.is_some())
}

async fn write_roles(
    tx: &mut crate::backend::Tx,
    id: &str,
    grants: &[AdminRoleGrant],
) -> Result<(), StoreError> {
    q("DELETE FROM admin_account_role WHERE account_id = ?1")
        .bind(id)
        .execute_tx(tx)
        .await?;
    for grant in grants {
        q("INSERT INTO admin_account_role (account_id, role_id, domains_json) VALUES (?1, ?2, ?3)")
            .bind(id)
            .bind(grant.role_id.as_str())
            .bind(
                grant
                    .domains
                    .as_ref()
                    .map(|ds| serde_json::to_string(ds).expect("string list")),
            )
            .execute_tx(tx)
            .await?;
    }
    Ok(())
}

impl Store {
    /// Session-bound CSRF token: HMAC-SHA256(server key, "csrf:" || session id).
    pub fn session_csrf_token(&self, session_id: &str) -> String {
        use sha2::{Digest, Sha256};
        let hex = self.key.to_hex();
        let mut ipad = [0x36u8; 64];
        let mut opad = [0x5cu8; 64];
        for (i, pair) in hex.as_bytes().as_chunks::<2>().0.iter().enumerate() {
            let byte = ((pair[0] as char).to_digit(16).expect("key hex") * 16
                + (pair[1] as char).to_digit(16).expect("key hex")) as u8;
            ipad[i] ^= byte;
            opad[i] ^= byte;
        }
        let inner = Sha256::new()
            .chain_update(ipad)
            .chain_update(b"csrf:")
            .chain_update(session_id.as_bytes())
            .finalize();
        Sha256::new()
            .chain_update(opad)
            .chain_update(inner)
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    /// Create an admin session with validated deployment timeout settings.
    pub async fn put_admin_session_with_timeouts(
        &self,
        hash: &str,
        username: &str,
        now: i64,
        idle: i64,
        absolute: i64,
    ) -> Result<(), StoreError> {
        let idle = idle.clamp(300, 7200);
        let absolute = absolute.clamp(3600, 86400);
        let stamp = chrono::DateTime::from_timestamp(now, 0)
            .ok_or_else(|| StoreError::Corrupt("invalid session timestamp".into()))?
            .to_rfc3339();
        q("DELETE FROM admin_sessions WHERE expires_at <= ?1 OR absolute_expires_at <= ?1")
            .bind(now)
            .execute(&self.backend)
            .await?;
        q("INSERT INTO admin_sessions (token_hash, admin_id, created_at, last_seen, expires_at, absolute_expires_at) VALUES (?1, ?2, ?3, ?3, ?4, ?5)")
            .bind(hash).bind(username).bind(stamp).bind(now + idle.min(absolute)).bind(now + absolute).execute(&self.backend).await?;
        Ok(())
    }

    /// Resolve and refresh a session using the current idle policy, never beyond
    /// its persisted absolute cap. The conditional update cannot revive a revoke.
    pub async fn get_admin_session_with_idle(
        &self,
        hash: &str,
        now: i64,
        idle: i64,
    ) -> Result<Option<String>, StoreError> {
        let mut tx = self.backend.begin().await?;
        let refreshed = q("UPDATE admin_sessions SET expires_at = CASE WHEN absolute_expires_at < ?2 THEN absolute_expires_at ELSE ?2 END WHERE token_hash = ?1 AND expires_at > ?3 AND absolute_expires_at > ?3")
            .bind(hash).bind(now + idle.clamp(300, 7200)).bind(now).execute_tx(&mut tx).await?;
        let username = if refreshed == 1 {
            q("SELECT admin_id FROM admin_sessions WHERE token_hash = ?1")
                .bind(hash)
                .fetch_opt_scalar_string_tx(&mut tx)
                .await?
        } else {
            None
        };
        tx.commit().await?;
        Ok(username)
    }

    // ---- accounts ----------------------------------------------------------

    /// Every admin account, by username.
    pub async fn list_admin_accounts(&self) -> Result<Vec<AdminAccountRow>, StoreError> {
        let rows = q(
            "SELECT id, username, display_name, password_hash, disabled, source, \
                      created_at, updated_at, last_login_at FROM admin_account ORDER BY username",
        )
        .fetch_all(&self.backend)
        .await?;
        Ok(rows.iter().map(account).collect())
    }

    /// How many admin accounts exist.
    pub async fn count_admin_accounts(&self) -> Result<i64, StoreError> {
        Ok(q("SELECT COUNT(*) FROM admin_account")
            .fetch_scalar_i64(&self.backend)
            .await?)
    }

    /// The account with `id`.
    pub async fn get_admin_account(&self, id: &str) -> Result<Option<AdminAccountRow>, StoreError> {
        let row = q(
            "SELECT id, username, display_name, password_hash, disabled, source, \
                     created_at, updated_at, last_login_at FROM admin_account WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.as_ref().map(account))
    }

    /// The account named `username` (exact match; usernames are stored lower-case
    /// by the server).
    pub async fn get_admin_account_by_username(
        &self,
        username: &str,
    ) -> Result<Option<AdminAccountRow>, StoreError> {
        let row = q(
            "SELECT id, username, display_name, password_hash, disabled, source, \
                     created_at, updated_at, last_login_at FROM admin_account WHERE username = ?1",
        )
        .bind(username)
        .fetch_optional(&self.backend)
        .await?;
        Ok(row.as_ref().map(account))
    }

    /// Insert `row`. A taken username fails with the database's unique error.
    pub async fn insert_admin_account(&self, row: &AdminAccountRow) -> Result<(), StoreError> {
        q(
            "INSERT INTO admin_account (id, username, display_name, password_hash, disabled, \
           source, created_at, updated_at, last_login_at) \
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )
        .bind(row.id.as_str())
        .bind(row.username.as_str())
        .bind(row.display_name.as_str())
        .bind(row.password_hash.clone())
        .bind(i64::from(row.disabled))
        .bind(row.source.as_str())
        .bind(row.created_at)
        .bind(row.updated_at)
        .bind(row.last_login_at)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Atomically create an account and its grants. Bootstrap uses the built-in
    /// role row as a database lock, shared with all guarded account mutations.
    pub async fn insert_admin_account_with_roles(
        &self,
        row: &AdminAccountRow,
        grants: &[AdminRoleGrant],
        bootstrap: bool,
    ) -> Result<bool, StoreError> {
        let mut tx = self.backend.begin().await?;
        lock_admin_accounts(&mut tx).await?;
        if bootstrap
            && q("SELECT id FROM admin_account LIMIT 1")
                .fetch_opt_scalar_string_tx(&mut tx)
                .await?
                .is_some()
        {
            tx.commit().await?;
            return Ok(false);
        }
        q("INSERT INTO admin_account (id, username, display_name, password_hash, disabled, source, created_at, updated_at, last_login_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)")
            .bind(row.id.as_str()).bind(row.username.as_str()).bind(row.display_name.as_str())
            .bind(row.password_hash.clone()).bind(i64::from(row.disabled)).bind(row.source.as_str())
            .bind(row.created_at).bind(row.updated_at).bind(row.last_login_at)
            .execute_tx(&mut tx).await?;
        write_roles(&mut tx, &row.id, grants).await?;
        tx.commit().await?;
        Ok(true)
    }

    /// Serialize the invariant check and mutation across server processes.
    /// False means the change would remove the last enabled full super admin.
    pub async fn change_admin_account_guarded(
        &self,
        id: &str,
        display_name: Option<&str>,
        disabled: bool,
        grants: Option<&[AdminRoleGrant]>,
        delete: bool,
        now: i64,
    ) -> Result<bool, StoreError> {
        let mut tx = self.backend.begin().await?;
        lock_admin_accounts(&mut tx).await?;
        let removes_super = delete
            || disabled
            || grants.is_some_and(|gs| {
                !gs.iter()
                    .any(|g| g.role_id == "super_admin" && g.domains.is_none())
            });
        if removes_super && is_last_super(&mut tx, id).await? {
            tx.commit().await?;
            return Ok(false);
        }
        if delete {
            q("DELETE FROM admin_sessions WHERE admin_id = (SELECT username FROM admin_account WHERE id = ?1)")
                .bind(id).execute_tx(&mut tx).await?;
            for sql in [
                "DELETE FROM admin_account_role WHERE account_id = ?1",
                "DELETE FROM admin_capability_override WHERE account_id = ?1",
                "DELETE FROM admin_totp WHERE account_id = ?1",
                "DELETE FROM admin_account WHERE id = ?1",
            ] {
                q(sql).bind(id).execute_tx(&mut tx).await?;
            }
        } else {
            if let Some(name) = display_name {
                q("UPDATE admin_account SET display_name = ?2, disabled = ?3, updated_at = ?4 WHERE id = ?1")
                    .bind(id).bind(name).bind(i64::from(disabled)).bind(now).execute_tx(&mut tx).await?;
                if disabled {
                    q("DELETE FROM admin_sessions WHERE admin_id = (SELECT username FROM admin_account WHERE id = ?1)")
                        .bind(id).execute_tx(&mut tx).await?;
                }
            }
            if let Some(gs) = grants {
                write_roles(&mut tx, id, gs).await?;
            }
        }
        tx.commit().await?;
        Ok(true)
    }

    /// Change an account's display name and disabled flag.
    pub async fn update_admin_account(
        &self,
        id: &str,
        display_name: &str,
        disabled: bool,
        now: i64,
    ) -> Result<bool, StoreError> {
        let n = q(
            "UPDATE admin_account SET display_name = ?2, disabled = ?3, updated_at = ?4 \
                   WHERE id = ?1",
        )
        .bind(id)
        .bind(display_name)
        .bind(i64::from(disabled))
        .bind(now)
        .execute(&self.backend)
        .await?;
        Ok(n == 1)
    }

    /// Replace an account's password hash (and make it a database account).
    pub async fn set_admin_password(
        &self,
        id: &str,
        password_hash: &str,
        now: i64,
    ) -> Result<bool, StoreError> {
        let n = q(
            "UPDATE admin_account SET password_hash = ?2, source = 'db', updated_at = ?3 \
                   WHERE id = ?1",
        )
        .bind(id)
        .bind(password_hash)
        .bind(now)
        .execute(&self.backend)
        .await?;
        Ok(n == 1)
    }

    /// Record a successful sign-in.
    pub async fn touch_admin_login(&self, id: &str, now: i64) -> Result<(), StoreError> {
        q("UPDATE admin_account SET last_login_at = ?2 WHERE id = ?1")
            .bind(id)
            .bind(now)
            .execute(&self.backend)
            .await?;
        Ok(())
    }

    /// Delete an account (its roles, overrides and TOTP go with it).
    pub async fn delete_admin_account(&self, id: &str) -> Result<bool, StoreError> {
        let mut tx = self.backend.begin().await?;
        for sql in [
            "DELETE FROM admin_account_role WHERE account_id = ?1",
            "DELETE FROM admin_capability_override WHERE account_id = ?1",
            "DELETE FROM admin_totp WHERE account_id = ?1",
        ] {
            q(sql).bind(id).execute_tx(&mut tx).await?;
        }
        let n = q("DELETE FROM admin_account WHERE id = ?1")
            .bind(id)
            .execute_tx(&mut tx)
            .await?;
        tx.commit().await?;
        Ok(n == 1)
    }

    // ---- roles -------------------------------------------------------------

    /// Make sure each built-in role `(id, name)` has its row (`builtin = 1`).
    /// Existing rows keep their id; the name is brought up to date.
    pub async fn ensure_builtin_admin_roles(
        &self,
        roles: &[(&str, &str)],
    ) -> Result<(), StoreError> {
        for (id, name) in roles {
            q(
                "INSERT INTO admin_role (id, name, builtin) VALUES (?1, ?2, 1) \
               ON CONFLICT(id) DO UPDATE SET name = excluded.name, builtin = 1",
            )
            .bind(*id)
            .bind(*name)
            .execute(&self.backend)
            .await?;
        }
        Ok(())
    }

    /// Every role with its stored capabilities, built-in roles first, then by name.
    pub async fn list_admin_roles(&self) -> Result<Vec<AdminRoleRow>, StoreError> {
        let rows = q("SELECT id, name, builtin FROM admin_role ORDER BY builtin DESC, name")
            .fetch_all(&self.backend)
            .await?;
        let caps = q("SELECT role_id, capability FROM admin_role_capability \
                      ORDER BY role_id, capability")
        .fetch_all(&self.backend)
        .await?;
        Ok(rows
            .iter()
            .map(|r| {
                let id = r.get_string("id");
                AdminRoleRow {
                    capabilities: caps
                        .iter()
                        .filter(|c| c.get_string("role_id") == id)
                        .map(|c| c.get_string("capability"))
                        .collect(),
                    name: r.get_string("name"),
                    builtin: r.get_i64("builtin") != 0,
                    id,
                }
            })
            .collect())
    }

    /// The role with `id`.
    pub async fn get_admin_role(&self, id: &str) -> Result<Option<AdminRoleRow>, StoreError> {
        Ok(self
            .list_admin_roles()
            .await?
            .into_iter()
            .find(|r| r.id == id))
    }

    /// Insert a custom role, or replace its name and capabilities.
    pub async fn put_admin_role(
        &self,
        id: &str,
        name: &str,
        capabilities: &[String],
    ) -> Result<(), StoreError> {
        let mut tx = self.backend.begin().await?;
        q(
            "INSERT INTO admin_role (id, name, builtin) VALUES (?1, ?2, 0) \
           ON CONFLICT(id) DO UPDATE SET name = excluded.name",
        )
        .bind(id)
        .bind(name)
        .execute_tx(&mut tx)
        .await?;
        q("DELETE FROM admin_role_capability WHERE role_id = ?1")
            .bind(id)
            .execute_tx(&mut tx)
            .await?;
        for cap in capabilities {
            q("INSERT INTO admin_role_capability (role_id, capability) VALUES (?1, ?2)")
                .bind(id)
                .bind(cap.as_str())
                .execute_tx(&mut tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// Delete a custom role and its assignments.
    pub async fn delete_admin_role(&self, id: &str) -> Result<bool, StoreError> {
        let mut tx = self.backend.begin().await?;
        if q("SELECT id FROM admin_role WHERE id = ?1 AND builtin = 0")
            .bind(id)
            .fetch_opt_scalar_string_tx(&mut tx)
            .await?
            .is_none()
        {
            tx.commit().await?;
            return Ok(false);
        }
        for sql in [
            "DELETE FROM admin_account_role WHERE role_id = ?1",
            "DELETE FROM admin_role_capability WHERE role_id = ?1",
        ] {
            q(sql).bind(id).execute_tx(&mut tx).await?;
        }
        let n = q("DELETE FROM admin_role WHERE id = ?1 AND builtin = 0")
            .bind(id)
            .execute_tx(&mut tx)
            .await?;
        tx.commit().await?;
        Ok(n == 1)
    }

    /// The roles assigned to an account.
    pub async fn admin_account_roles(
        &self,
        account_id: &str,
    ) -> Result<Vec<AdminRoleGrant>, StoreError> {
        let rows = q("SELECT role_id, domains_json FROM admin_account_role \
                      WHERE account_id = ?1 ORDER BY role_id")
        .bind(account_id)
        .fetch_all(&self.backend)
        .await?;
        rows.iter()
            .map(|r| {
                let domains = match r.get_opt_string("domains_json") {
                    None => None,
                    Some(j) => Some(
                        serde_json::from_str::<Vec<String>>(&j)
                            .map_err(|e| StoreError::Corrupt(format!("domains_json: {e}")))?,
                    ),
                };
                Ok(AdminRoleGrant {
                    role_id: r.get_string("role_id"),
                    domains,
                })
            })
            .collect()
    }

    /// Replace an account's role assignments.
    pub async fn set_admin_account_roles(
        &self,
        account_id: &str,
        grants: &[AdminRoleGrant],
    ) -> Result<(), StoreError> {
        let mut tx = self.backend.begin().await?;
        q("DELETE FROM admin_account_role WHERE account_id = ?1")
            .bind(account_id)
            .execute_tx(&mut tx)
            .await?;
        for g in grants {
            let domains = g
                .domains
                .as_ref()
                .map(|d| serde_json::to_string(d).expect("a string list serialises"));
            q(
                "INSERT INTO admin_account_role (account_id, role_id, domains_json) \
               VALUES (?1, ?2, ?3)",
            )
            .bind(account_id)
            .bind(g.role_id.as_str())
            .bind(domains)
            .execute_tx(&mut tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// The accounts holding role `role_id`.
    pub async fn admin_accounts_with_role(&self, role_id: &str) -> Result<Vec<String>, StoreError> {
        Ok(
            q("SELECT account_id FROM admin_account_role WHERE role_id = ?1 ORDER BY account_id")
                .bind(role_id)
                .fetch_all_scalar_string(&self.backend)
                .await?,
        )
    }

    /// An account's capability overrides as `(capability, effect)`.
    pub async fn admin_overrides(
        &self,
        account_id: &str,
    ) -> Result<Vec<(String, String)>, StoreError> {
        let rows = q("SELECT capability, effect FROM admin_capability_override \
                      WHERE account_id = ?1 ORDER BY capability")
        .bind(account_id)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows
            .iter()
            .map(|r| (r.get_string("capability"), r.get_string("effect")))
            .collect())
    }

    /// Replace overrides without allowing deny rules to bypass the last-admin
    /// invariant. Uses the same database lock as disables, deletes and demotions.
    pub async fn set_admin_overrides_guarded(
        &self,
        id: &str,
        overrides: &[(String, String)],
    ) -> Result<bool, StoreError> {
        let mut tx = self.backend.begin().await?;
        lock_admin_accounts(&mut tx).await?;
        if overrides.iter().any(|(_, effect)| effect == "deny")
            && is_last_super(&mut tx, id).await?
        {
            tx.commit().await?;
            return Ok(false);
        }
        q("DELETE FROM admin_capability_override WHERE account_id = ?1")
            .bind(id)
            .execute_tx(&mut tx)
            .await?;
        for (cap, effect) in overrides {
            q("INSERT INTO admin_capability_override (account_id, capability, effect) VALUES (?1, ?2, ?3)")
                .bind(id).bind(cap.as_str()).bind(effect.as_str()).execute_tx(&mut tx).await?;
        }
        tx.commit().await?;
        Ok(true)
    }

    /// Replace an account's capability overrides.
    pub async fn set_admin_overrides(
        &self,
        account_id: &str,
        overrides: &[(String, String)],
    ) -> Result<(), StoreError> {
        let mut tx = self.backend.begin().await?;
        q("DELETE FROM admin_capability_override WHERE account_id = ?1")
            .bind(account_id)
            .execute_tx(&mut tx)
            .await?;
        for (cap, effect) in overrides {
            q(
                "INSERT INTO admin_capability_override (account_id, capability, effect) \
               VALUES (?1, ?2, ?3)",
            )
            .bind(account_id)
            .bind(cap.as_str())
            .bind(effect.as_str())
            .execute_tx(&mut tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    // ---- TOTP --------------------------------------------------------------

    /// Store (sealed) an admin's TOTP secret, replacing any earlier one and
    /// resetting the replay guard.
    pub async fn put_admin_totp(
        &self,
        account_id: &str,
        secret: &[u8],
        now: i64,
    ) -> Result<(), StoreError> {
        let sealed = self.key.seal(secret)?;
        q(
            "INSERT INTO admin_totp (account_id, sealed_secret, last_step, enrolled_at) \
           VALUES (?1, ?2, 0, ?3) \
           ON CONFLICT(account_id) DO UPDATE SET sealed_secret = excluded.sealed_secret, \
           last_step = 0, enrolled_at = excluded.enrolled_at",
        )
        .bind(account_id)
        .bind(sealed)
        .bind(now)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    /// Confirm a first factor and consume its enrollment code in one write.
    /// Existing factors cannot be replaced through the self-enrollment route.
    pub async fn enroll_admin_totp(
        &self,
        id: &str,
        secret: &[u8],
        step: i64,
        now: i64,
    ) -> Result<bool, StoreError> {
        let sealed = self.key.seal(secret)?;
        Ok(q("INSERT INTO admin_totp (account_id, sealed_secret, last_step, enrolled_at) VALUES (?1, ?2, ?3, ?4) ON CONFLICT(account_id) DO NOTHING")
            .bind(id).bind(sealed).bind(step).bind(now).execute(&self.backend).await? == 1)
    }

    /// An admin's TOTP secret (unsealed), if enrolled.
    pub async fn get_admin_totp(&self, account_id: &str) -> Result<Option<Vec<u8>>, StoreError> {
        let row = q("SELECT sealed_secret FROM admin_totp WHERE account_id = ?1")
            .bind(account_id)
            .fetch_optional(&self.backend)
            .await?;
        match row {
            None => Ok(None),
            Some(r) => Ok(Some(self.key.open(&r.get_blob("sealed_secret"))?)),
        }
    }

    /// Remove an admin's TOTP factor.
    pub async fn delete_admin_totp(&self, account_id: &str) -> Result<bool, StoreError> {
        let n = q("DELETE FROM admin_totp WHERE account_id = ?1")
            .bind(account_id)
            .execute(&self.backend)
            .await?;
        Ok(n == 1)
    }

    /// Advance the admin's last consumed TOTP step to `step` only if it grows;
    /// `true` when this call consumed the step (the code was fresh).
    pub async fn advance_admin_totp_step(
        &self,
        account_id: &str,
        step: i64,
    ) -> Result<bool, StoreError> {
        let n = q("UPDATE admin_totp SET last_step = ?2 WHERE account_id = ?1 AND last_step < ?3")
            .bind(account_id)
            .bind(step)
            .bind(step)
            .execute(&self.backend)
            .await?;
        Ok(n == 1)
    }

    // ---- audit -------------------------------------------------------------

    /// Append an audit row with the RBAC columns (0033): the roles the actor held
    /// (comma-separated), the capability the write used, and the redacted object
    /// before and after.
    pub async fn append_admin_audit(
        &self,
        row: &crate::AuditRow,
        admin_role: Option<&str>,
        capability: Option<&str>,
        before_json: Option<&str>,
        after_json: Option<&str>,
    ) -> Result<(), StoreError> {
        q(
            "INSERT INTO audit_log (id, ts, actor, actor_kind, action, target, detail_json, ip, \
           admin_role, capability, before_json, after_json) \
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        )
        .bind(row.id.as_str())
        .bind(row.ts.as_str())
        .bind(row.actor.as_str())
        .bind(row.actor_kind.as_str())
        .bind(row.action.as_str())
        .bind(row.target.clone())
        .bind(row.detail_json.as_str())
        .bind(row.ip.clone())
        .bind(admin_role)
        .bind(capability)
        .bind(before_json)
        .bind(after_json)
        .execute(&self.backend)
        .await?;
        Ok(())
    }

    // ---- sessions ----------------------------------------------------------

    /// Every live admin session, newest first.
    pub async fn list_admin_sessions(&self, now: i64) -> Result<Vec<AdminSessionRow>, StoreError> {
        let rows = q(
            "SELECT token_hash, admin_id, created_at, expires_at, absolute_expires_at \
                      FROM admin_sessions WHERE expires_at > ?1 AND absolute_expires_at > ?2 \
                      ORDER BY created_at DESC",
        )
        .bind(now)
        .bind(now)
        .fetch_all(&self.backend)
        .await?;
        Ok(rows
            .iter()
            .map(|r| AdminSessionRow {
                token_hash: r.get_string("token_hash"),
                admin_id: r.get_string("admin_id"),
                created_at: r.get_string("created_at"),
                expires_at: r.get_i64("expires_at"),
                absolute_expires_at: r.get_i64("absolute_expires_at"),
            })
            .collect())
    }

    /// End every session of `admin_id`; returns how many ended.
    pub async fn delete_admin_sessions_of(&self, admin_id: &str) -> Result<u64, StoreError> {
        Ok(q("DELETE FROM admin_sessions WHERE admin_id = ?1")
            .bind(admin_id)
            .execute(&self.backend)
            .await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn store() -> Store {
        Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap()
    }

    fn row(id: &str, username: &str) -> AdminAccountRow {
        AdminAccountRow {
            id: id.into(),
            username: username.into(),
            display_name: username.into(),
            password_hash: Some("$argon2id$v=19$m=19456,t=2,p=1$c2FsdA$aGFzaA".into()),
            disabled: false,
            source: "db".into(),
            created_at: 1,
            updated_at: 1,
            last_login_at: None,
        }
    }

    #[tokio::test]
    async fn accounts_roles_overrides_totp() {
        let s = store().await;
        assert_eq!(s.count_admin_accounts().await.unwrap(), 0);
        s.insert_admin_account(&row("a1", "alice")).await.unwrap();
        assert!(
            s.insert_admin_account(&row("a2", "alice")).await.is_err(),
            "username is unique"
        );
        let a = s
            .get_admin_account_by_username("alice")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(a.id, "a1");
        assert!(!format!("{a:?}").contains("argon2"), "hash is redacted");
        assert!(
            s.update_admin_account("a1", "Alice", true, 5)
                .await
                .unwrap()
        );
        let a = s.get_admin_account("a1").await.unwrap().unwrap();
        assert!(a.disabled && a.display_name == "Alice" && a.updated_at == 5);
        s.touch_admin_login("a1", 9).await.unwrap();
        assert_eq!(
            s.get_admin_account("a1")
                .await
                .unwrap()
                .unwrap()
                .last_login_at,
            Some(9)
        );

        s.ensure_builtin_admin_roles(&[("super_admin", "Super admin")])
            .await
            .unwrap();
        s.put_admin_role(
            "r1",
            "Mail desk",
            &["users.read".into(), "users.write".into()],
        )
        .await
        .unwrap();
        let roles = s.list_admin_roles().await.unwrap();
        assert_eq!(roles[0].id, "super_admin");
        assert!(roles[0].builtin);
        assert_eq!(roles[1].capabilities, vec!["users.read", "users.write"]);
        s.set_admin_account_roles(
            "a1",
            &[
                AdminRoleGrant {
                    role_id: "r1".into(),
                    domains: Some(vec!["example.com".into()]),
                },
                AdminRoleGrant {
                    role_id: "super_admin".into(),
                    domains: None,
                },
            ],
        )
        .await
        .unwrap();
        let grants = s.admin_account_roles("a1").await.unwrap();
        assert_eq!(grants.len(), 2);
        assert_eq!(
            grants[0].domains.as_deref(),
            Some(&["example.com".to_string()][..])
        );
        assert_eq!(grants[1].domains, None);
        assert_eq!(
            s.admin_accounts_with_role("super_admin").await.unwrap(),
            vec!["a1"]
        );
        assert!(
            !s.delete_admin_role("super_admin").await.unwrap(),
            "built-in stays"
        );
        s.set_admin_overrides("a1", &[("logs.read".into(), "deny".into())])
            .await
            .unwrap();
        assert_eq!(
            s.admin_overrides("a1").await.unwrap(),
            vec![("logs.read".to_string(), "deny".to_string())]
        );

        s.put_admin_totp("a1", b"0123456789", 3).await.unwrap();
        assert_eq!(
            s.get_admin_totp("a1").await.unwrap().as_deref(),
            Some(&b"0123456789"[..])
        );
        assert!(s.advance_admin_totp_step("a1", 10).await.unwrap());
        assert!(
            !s.advance_admin_totp_step("a1", 10).await.unwrap(),
            "replay"
        );
        assert!(s.delete_admin_role("r1").await.unwrap());
        assert_eq!(s.admin_account_roles("a1").await.unwrap().len(), 1);
        assert!(s.delete_admin_account("a1").await.unwrap());
        assert_eq!(s.get_admin_totp("a1").await.unwrap(), None);
        assert!(s.admin_account_roles("a1").await.unwrap().is_empty());
    }
}
