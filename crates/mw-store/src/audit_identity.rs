//! Safe actor labels for audit presentation. Never reads credential columns.
use crate::{Store, StoreError, q};

#[derive(Clone, Debug)]
pub struct AuditIdentity {
    pub id: String,
    pub label: String,
    pub address: Option<String>,
}

impl Store {
    pub async fn audit_identity(
        &self,
        kind: &str,
        actor: &str,
    ) -> Result<Option<AuditIdentity>, StoreError> {
        if actor.is_empty() || actor.len() > 512 {
            return Ok(None);
        }
        if kind == "admin" {
            return Ok(q("SELECT id,username,display_name FROM admin_account WHERE id=?1 OR username=?1 ORDER BY CASE WHEN id=?1 THEN 0 ELSE 1 END LIMIT 1").bind(actor).fetch_optional(&self.backend).await?.map(|r| {
                let username=r.get_string("username"); let display=r.get_string("display_name");
                AuditIdentity { id:r.get_string("id"),label:if display.trim().is_empty() || display==username {username} else {format!("{display} ({username})")},address:None }
            }));
        }
        if kind != "user" {
            return Ok(None);
        }
        let mut address = q("SELECT address FROM mail_user_account WHERE account_id=?1")
            .bind(actor)
            .fetch_opt_scalar_string(&self.backend)
            .await?;
        if address.is_none() {
            address = q("SELECT address FROM signin_identity WHERE account_id=?1")
                .bind(actor)
                .fetch_opt_scalar_string(&self.backend)
                .await?;
        }
        if address.is_none() {
            address = q("SELECT username FROM admin_users WHERE username=?1")
                .bind(actor)
                .fetch_opt_scalar_string(&self.backend)
                .await?;
        }
        if address.is_none() {
            address = q("SELECT address FROM mail_user WHERE address=?1")
                .bind(actor)
                .fetch_opt_scalar_string(&self.backend)
                .await?;
        }
        let Some(address) = address else {
            return Ok(None);
        };
        let display = q("SELECT display_name FROM mail_user WHERE address=?1")
            .bind(&address)
            .fetch_optional(&self.backend)
            .await?
            .and_then(|r| r.get_opt_string("display_name"))
            .filter(|s| !s.trim().is_empty() && s != &address);
        Ok(Some(AuditIdentity {
            id: actor.into(),
            label: display.map_or_else(|| address.clone(), |name| format!("{name} ({address})")),
            address: Some(address),
        }))
    }
}

/// Audit-log retention and pruning (in its own file; re-exported here so the
/// crate root needs no new module line).
#[path = "audit_retention.rs"]
pub mod retention;
