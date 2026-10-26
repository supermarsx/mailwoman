//! Durable draft save claims. No operation may be replayed after uncertain APPEND.
use crate::{
    Store, StoreError,
    backend::{Row, q},
    seal,
};
use sha2::{Digest, Sha256};

pub const MAX_DRAFT_STATE: usize = 1024 * 1024;
pub const MAX_ACCOUNT_DRAFTS: i64 = 100;

pub struct DraftSave<'a> {
    pub owner: &'a str,
    pub account_id: &'a str,
    pub id: &'a str,
    pub expected_revision: Option<&'a str>,
    pub operation_id: &'a str,
    pub state: &'a [u8],
    pub summary: &'a [u8],
    pub now: i64,
}
/// Sensitive editor bytes deliberately do not implement Debug.
pub struct AccountDraft {
    pub id: String,
    pub account_id: String,
    pub revision: Option<String>,
    pub email_id: Option<String>,
    pub state: Vec<u8>,
    pub summary: Vec<u8>,
    pub updated_at: i64,
    pub status: String,
    pub cleanup_email_id: Option<String>,
}
pub enum DraftClaim {
    Claimed,
    Saved(Box<AccountDraft>),
    Conflict,
    Uncertain,
    Refused,
    CleanupPending,
    Limit,
    Invalid,
}
pub enum DraftDiscard {
    Ready(Box<AccountDraft>),
    Gone,
    Conflict,
    Uncertain,
    CleanupPending,
}
fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}
fn fingerprint(input: &DraftSave<'_>) -> String {
    let mut hash = Sha256::new();
    for bytes in [
        input.expected_revision.unwrap_or("").as_bytes(),
        input.state,
        input.summary,
    ] {
        hash.update((bytes.len() as u64).to_be_bytes());
        hash.update(bytes);
    }
    hash.finalize().iter().map(|b| format!("{b:02x}")).collect()
}
impl Store {
    fn decode_account_draft(
        &self,
        owner: &str,
        row: &Row,
        include_state: bool,
    ) -> Result<AccountDraft, StoreError> {
        let id = row.get_string("id");
        let operation = row.get_string("operation_id");
        let revision = row.get_opt_string("revision");
        let status = row.get_string("operation_state");
        let pending = matches!(status.as_str(), "claimed" | "uncertain");
        let context = if pending {
            operation.as_str()
        } else {
            revision.as_deref().unwrap_or("")
        };
        let purpose = if pending {
            "account-draft-pending"
        } else {
            "account-draft"
        };
        let open = |field: &str, kind: &str| -> Result<Vec<u8>, StoreError> {
            match row.get_opt_blob(field) {
                Some(bytes) => {
                    Ok(self
                        .key
                        .open_bound(&bytes, purpose, &[owner, &id, context, kind])?)
                }
                None => Ok(Vec::new()),
            }
        };
        let state = if include_state {
            open(
                if pending {
                    "sealed_pending_state"
                } else {
                    "sealed_state"
                },
                "state",
            )?
        } else {
            Vec::new()
        };
        let summary = open(
            if pending {
                "sealed_pending_summary"
            } else {
                "sealed_summary"
            },
            "summary",
        )?;
        Ok(AccountDraft {
            id,
            account_id: row.get_string("account_id"),
            revision,
            email_id: row.get_opt_string("email_id"),
            state,
            summary,
            updated_at: row.get_i64("updated_at"),
            status: if pending { "uncertain".into() } else { status },
            cleanup_email_id: row.get_opt_string("cleanup_email_id"),
        })
    }
    pub async fn account_draft(
        &self,
        owner: &str,
        id: &str,
    ) -> Result<Option<AccountDraft>, StoreError> {
        q("SELECT * FROM account_draft WHERE owner=?1 AND id=?2 AND request_hash<>''")
            .bind(owner)
            .bind(id)
            .fetch_optional(&self.backend)
            .await?
            .map(|row| self.decode_account_draft(owner, &row, true))
            .transpose()
    }
    pub async fn account_draft_for_email(
        &self,
        owner: &str,
        email: &str,
    ) -> Result<Option<AccountDraft>, StoreError> {
        q("SELECT * FROM account_draft WHERE owner=?1 AND email_id=?2 AND request_hash<>'' LIMIT 1")
            .bind(owner)
            .bind(email)
            .fetch_optional(&self.backend)
            .await?
            .map(|row| self.decode_account_draft(owner, &row, true))
            .transpose()
    }
    pub async fn account_drafts(&self, owner: &str) -> Result<Vec<AccountDraft>, StoreError> {
        // Do not read megabytes of editor bodies for the list projection.
        let rows=q("SELECT id,account_id,revision,email_id,sealed_summary,updated_at,operation_id,operation_state,sealed_pending_summary,cleanup_email_id FROM account_draft WHERE owner=?1 AND request_hash<>'' ORDER BY updated_at DESC,id LIMIT 100").bind(owner).fetch_all(&self.backend).await?;
        rows.iter()
            .map(|row| self.decode_account_draft(owner, row, false))
            .collect()
    }
    pub async fn claim_account_draft(
        &self,
        input: &DraftSave<'_>,
    ) -> Result<DraftClaim, StoreError> {
        if input.owner.is_empty()
            || input.owner.len() > 256
            || input.account_id.is_empty()
            || input.account_id.len() > 512
            || !identifier(input.id)
            || !identifier(input.operation_id)
            || input.state.is_empty()
            || input.state.len() > MAX_DRAFT_STATE
            || input.summary.len() > 8192
        {
            return Ok(DraftClaim::Invalid);
        }
        let hash = fingerprint(input);
        let mut tx = self.backend.begin().await?;
        let lock = format!("draft-owner:{}", input.owner);
        q("INSERT INTO settings(key,value) VALUES(?1,'0') ON CONFLICT(key) DO NOTHING")
            .bind(&lock)
            .execute_tx(&mut tx)
            .await?;
        q("UPDATE settings SET value=value WHERE key=?1")
            .bind(&lock)
            .execute_tx(&mut tx)
            .await?;
        let old = q("SELECT * FROM account_draft WHERE owner=?1 AND id=?2")
            .bind(input.owner)
            .bind(input.id)
            .fetch_all_tx(&mut tx)
            .await?
            .into_iter()
            .next();
        if let Some(row) = &old {
            if row.get_string("request_hash").is_empty() {
                return Ok(DraftClaim::Conflict);
            }
            if row.get_string("account_id") != input.account_id {
                return Ok(DraftClaim::Conflict);
            }
            let status = row.get_string("operation_state");
            if row.get_string("operation_id") == input.operation_id {
                if row.get_string("request_hash") != hash {
                    return Ok(DraftClaim::Conflict);
                }
                return Ok(match status.as_str() {
                    "saved" => DraftClaim::Saved(Box::new(self.decode_account_draft(
                        input.owner,
                        row,
                        true,
                    )?)),
                    "refused" => DraftClaim::Refused,
                    _ => DraftClaim::Uncertain,
                });
            }
            if matches!(status.as_str(), "claimed" | "uncertain") {
                return Ok(DraftClaim::Uncertain);
            }
            if row.get_opt_string("revision").as_deref() != input.expected_revision {
                return Ok(DraftClaim::Conflict);
            }
            if row.get_opt_string("cleanup_email_id").is_some() {
                return Ok(DraftClaim::CleanupPending);
            }
        } else {
            if input.expected_revision.is_some() {
                return Ok(DraftClaim::Conflict);
            }
            let rows =
                q("SELECT COUNT(*) AS n FROM account_draft WHERE owner=?1 AND request_hash<>''")
                    .bind(input.owner)
                    .fetch_all_tx(&mut tx)
                    .await?;
            if rows[0].get_i64("n") >= MAX_ACCOUNT_DRAFTS {
                return Ok(DraftClaim::Limit);
            }
        }
        let state = self.key.seal_bound(
            input.state,
            "account-draft-pending",
            &[input.owner, input.id, input.operation_id, "state"],
        )?;
        let summary = self.key.seal_bound(
            input.summary,
            "account-draft-pending",
            &[input.owner, input.id, input.operation_id, "summary"],
        )?;
        q("INSERT INTO account_draft(owner,id,account_id,updated_at,operation_id,request_hash,operation_state,sealed_pending_state,sealed_pending_summary) VALUES(?1,?2,?3,?4,?5,?6,'claimed',?7,?8) ON CONFLICT(owner,id) DO UPDATE SET updated_at=excluded.updated_at,operation_id=excluded.operation_id,request_hash=excluded.request_hash,operation_state='claimed',sealed_pending_state=excluded.sealed_pending_state,sealed_pending_summary=excluded.sealed_pending_summary")
            .bind(input.owner).bind(input.id).bind(input.account_id).bind(input.now).bind(input.operation_id).bind(hash).bind(state).bind(summary).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(DraftClaim::Claimed)
    }
    /// Rebind temporary upload references to the confirmed message's owned parts.
    pub async fn rebind_account_draft_state(
        &self,
        owner: &str,
        id: &str,
        operation: &str,
        state: &[u8],
    ) -> Result<bool, StoreError> {
        if state.len() > MAX_DRAFT_STATE {
            return Ok(false);
        }
        let sealed = self.key.seal_bound(
            state,
            "account-draft-pending",
            &[owner, id, operation, "state"],
        )?;
        Ok(q("UPDATE account_draft SET sealed_pending_state=?1 WHERE owner=?2 AND id=?3 AND operation_id=?4 AND operation_state='claimed'").bind(sealed).bind(owner).bind(id).bind(operation).execute(&self.backend).await?>0)
    }
    /// Called only after confirmed upstream persistence and local message indexing.
    pub async fn complete_account_draft(
        &self,
        owner: &str,
        id: &str,
        operation: &str,
        email_id: &str,
        now: i64,
    ) -> Result<Option<AccountDraft>, StoreError> {
        let mut tx = self.backend.begin().await?;
        q("UPDATE account_draft SET updated_at=updated_at WHERE owner=?1 AND id=?2")
            .bind(owner)
            .bind(id)
            .execute_tx(&mut tx)
            .await?;
        let Some(row)=q("SELECT * FROM account_draft WHERE owner=?1 AND id=?2 AND operation_id=?3 AND operation_state='claimed'").bind(owner).bind(id).bind(operation).fetch_all_tx(&mut tx).await?.into_iter().next()else{return Ok(None)};
        let pending = self.decode_account_draft(owner, &row, true)?;
        let revision = seal::random_token();
        let state = self.key.seal_bound(
            &pending.state,
            "account-draft",
            &[owner, id, &revision, "state"],
        )?;
        let summary = self.key.seal_bound(
            &pending.summary,
            "account-draft",
            &[owner, id, &revision, "summary"],
        )?;
        q("UPDATE account_draft SET revision=?1,cleanup_email_id=email_id,email_id=?2,sealed_state=?3,sealed_summary=?4,updated_at=?5,operation_state='saved',sealed_pending_state=NULL,sealed_pending_summary=NULL WHERE owner=?6 AND id=?7 AND operation_id=?8 AND operation_state='claimed'")
            .bind(&revision).bind(email_id).bind(state).bind(summary).bind(now).bind(owner).bind(id).bind(operation).execute_tx(&mut tx).await?;
        let saved = q("SELECT * FROM account_draft WHERE owner=?1 AND id=?2")
            .bind(owner)
            .bind(id)
            .fetch_all_tx(&mut tx)
            .await?
            .into_iter()
            .next()
            .ok_or(StoreError::NotFound)?;
        let saved = self.decode_account_draft(owner, &saved, true)?;
        tx.commit().await?;
        Ok(Some(saved))
    }
    /// Refusal is only legal before entering a possibly mutating APPEND operation.
    pub async fn refuse_account_draft_before_append(
        &self,
        owner: &str,
        id: &str,
        operation: &str,
    ) -> Result<bool, StoreError> {
        Ok(q("UPDATE account_draft SET operation_state='refused',sealed_pending_state=NULL,sealed_pending_summary=NULL WHERE owner=?1 AND id=?2 AND operation_id=?3 AND operation_state='claimed'").bind(owner).bind(id).bind(operation).execute(&self.backend).await?>0)
    }
    pub async fn uncertain_account_draft(
        &self,
        owner: &str,
        id: &str,
        operation: &str,
    ) -> Result<bool, StoreError> {
        Ok(q("UPDATE account_draft SET operation_state='uncertain' WHERE owner=?1 AND id=?2 AND operation_id=?3 AND operation_state='claimed'").bind(owner).bind(id).bind(operation).execute(&self.backend).await?>0)
    }
    /// Reserve the exact saved revision for disposal before any upstream delete.
    /// A current-email cleanup marker blocks concurrent replacements.
    pub async fn claim_account_draft_discard(
        &self,
        owner: &str,
        id: &str,
        revision: Option<&str>,
    ) -> Result<DraftDiscard, StoreError> {
        let mut tx = self.backend.begin().await?;
        q("UPDATE account_draft SET updated_at=updated_at WHERE owner=?1 AND id=?2")
            .bind(owner)
            .bind(id)
            .execute_tx(&mut tx)
            .await?;
        let row = q("SELECT * FROM account_draft WHERE owner=?1 AND id=?2")
            .bind(owner)
            .bind(id)
            .fetch_all_tx(&mut tx)
            .await?
            .into_iter()
            .next();
        let Some(row) = row else {
            return Ok(DraftDiscard::Gone);
        };
        if row.get_string("request_hash").is_empty() {
            return Ok(DraftDiscard::Gone);
        }
        if row.get_opt_string("revision").as_deref() != revision {
            return Ok(DraftDiscard::Conflict);
        }
        if matches!(
            row.get_string("operation_state").as_str(),
            "claimed" | "uncertain"
        ) {
            return Ok(DraftDiscard::Uncertain);
        }
        let email = row.get_opt_string("email_id");
        if email.is_none() {
            q("UPDATE account_draft SET request_hash='',operation_state='refused',sealed_state=NULL,sealed_summary=NULL,sealed_pending_state=NULL,sealed_pending_summary=NULL,cleanup_email_id=NULL WHERE owner=?1 AND id=?2")
                .bind(owner)
                .bind(id)
                .execute_tx(&mut tx)
                .await?;
            tx.commit().await?;
            return Ok(DraftDiscard::Gone);
        }
        if row
            .get_opt_string("cleanup_email_id")
            .is_some_and(|old| Some(old) != email)
        {
            return Ok(DraftDiscard::CleanupPending);
        }
        q("UPDATE account_draft SET cleanup_email_id=email_id WHERE owner=?1 AND id=?2")
            .bind(owner)
            .bind(id)
            .execute_tx(&mut tx)
            .await?;
        let mut result = self.decode_account_draft(owner, &row, true)?;
        result.cleanup_email_id = email;
        tx.commit().await?;
        Ok(DraftDiscard::Ready(Box::new(result)))
    }
    pub async fn complete_account_draft_discard(
        &self,
        owner: &str,
        id: &str,
        revision: &str,
        email: &str,
    ) -> Result<bool, StoreError> {
        Ok(q("UPDATE account_draft SET request_hash='',operation_state='refused',email_id=NULL,sealed_state=NULL,sealed_summary=NULL,sealed_pending_state=NULL,sealed_pending_summary=NULL,cleanup_email_id=NULL WHERE owner=?1 AND id=?2 AND revision=?3 AND email_id=?4 AND cleanup_email_id=?4 AND operation_state IN ('saved','refused')").bind(owner).bind(id).bind(revision).bind(email).execute(&self.backend).await?>0)
    }
    pub async fn account_draft_cleanup_done(
        &self,
        owner: &str,
        id: &str,
        email_id: &str,
    ) -> Result<bool, StoreError> {
        Ok(q("UPDATE account_draft SET cleanup_email_id=NULL WHERE owner=?1 AND id=?2 AND cleanup_email_id=?3 AND email_id<>?3").bind(owner).bind(id).bind(email_id).execute(&self.backend).await?>0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input<'a>(owner: &'a str, op: &'a str, revision: Option<&'a str>) -> DraftSave<'a> {
        DraftSave {
            owner,
            account_id: "same-wire-id",
            id: "draft-1",
            expected_revision: revision,
            operation_id: op,
            state: b"private editor body",
            summary: b"private subject",
            now: 1,
        }
    }
    #[tokio::test]
    async fn claims_are_atomic_realm_scoped_and_never_replay_uncertainty() {
        let store = Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        let first = input("realm-one", "operation-one", None);
        let second = input("realm-one", "operation-two", None);
        let (a, b) = tokio::join!(
            store.claim_account_draft(&first),
            store.claim_account_draft(&second)
        );
        assert_ne!(
            matches!(a.unwrap(), DraftClaim::Claimed),
            matches!(b.unwrap(), DraftClaim::Claimed)
        );
        assert!(
            store
                .account_draft("realm-two", "draft-1")
                .await
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            store
                .claim_account_draft(&input("realm-two", "operation-one", None))
                .await
                .unwrap(),
            DraftClaim::Claimed
        ));
        assert!(matches!(
            store
                .claim_account_draft(&input("realm-one", "operation-three", None))
                .await
                .unwrap(),
            DraftClaim::Uncertain
        ));
        let raw = q("SELECT sealed_pending_state FROM account_draft WHERE owner='realm-one'")
            .fetch_one(&store.backend)
            .await
            .unwrap()
            .get_blob("sealed_pending_state");
        assert!(!raw.windows(7).any(|bytes| bytes == b"private"));
    }
    #[tokio::test]
    async fn saved_replay_and_replacement_preserve_old_until_confirmed() {
        let store = Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        let first = input("realm", "operation-one", None);
        assert!(matches!(
            store.claim_account_draft(&first).await.unwrap(),
            DraftClaim::Claimed
        ));
        let saved = store
            .complete_account_draft("realm", "draft-1", "operation-one", "email-old", 2)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(saved.state, b"private editor body");
        assert!(matches!(
            store.claim_account_draft(&first).await.unwrap(),
            DraftClaim::Saved(_)
        ));
        let mut changed = input("realm", "operation-one", None);
        changed.state = b"different";
        assert!(matches!(
            store.claim_account_draft(&changed).await.unwrap(),
            DraftClaim::Conflict
        ));
        let next = input("realm", "operation-two", saved.revision.as_deref());
        assert!(matches!(
            store.claim_account_draft(&next).await.unwrap(),
            DraftClaim::Claimed
        ));
        assert_eq!(
            store
                .account_draft("realm", "draft-1")
                .await
                .unwrap()
                .unwrap()
                .email_id
                .as_deref(),
            Some("email-old")
        );
        let replaced = store
            .complete_account_draft("realm", "draft-1", "operation-two", "email-new", 3)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(replaced.cleanup_email_id.as_deref(), Some("email-old"));
        assert!(matches!(
            store
                .claim_account_draft(&input(
                    "realm",
                    "operation-three",
                    replaced.revision.as_deref()
                ))
                .await
                .unwrap(),
            DraftClaim::CleanupPending
        ));
        assert!(
            !store
                .account_draft_cleanup_done("other", "draft-1", "email-old")
                .await
                .unwrap()
        );
        assert!(
            store
                .account_draft_cleanup_done("realm", "draft-1", "email-old")
                .await
                .unwrap()
        );
    }
}
