//! Owner-bound mailbox delegation. Legacy `child_account` rows are intentionally
//! not read: they contain no verified principal or authentication provenance.
use crate::{
    Session, Store, StoreError,
    backend::{Row, Tx},
    q,
};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkedAccount {
    pub id: String,
    pub account_id: String,
    pub display_name: String,
    pub revision: String,
    pub created_at: i64,
}

fn projection(row: &Row) -> LinkedAccount {
    LinkedAccount {
        id: row.get_string("id"),
        account_id: row.get_string("account_id"),
        display_name: row.get_string("display_name"),
        revision: row.get_string("revision"),
        created_at: row.get_i64("created_at"),
    }
}

async fn serialize(tx: &mut Tx) -> Result<(), StoreError> {
    q("UPDATE identity_serialization SET id=id WHERE id=1")
        .execute_tx(tx)
        .await?;
    Ok(())
}

/// Contains credentials; deliberately has no Debug/Serialize implementation.
pub struct LinkedSessionAuthority {
    pub parent: Session,
    pub source: Session,
    pub link: LinkedAccount,
}

impl Store {
    /// Bound retained private sessions during active use. A logout may cascade
    /// the context association before a background janitor sees its Session.
    /// Each creation removes up to 128 such orphans or absolute-expired sources.
    async fn prune_linked_sessions(&self) -> Result<(), StoreError> {
        let cutoff = (chrono::Utc::now() - chrono::Duration::days(15)).to_rfc3339();
        q("DELETE FROM sessions WHERE id IN (SELECT s.id FROM sessions s WHERE s.linked_internal<>0 AND (s.created_at<?1 OR (s.linked_internal=2 AND NOT EXISTS(SELECT 1 FROM linked_context c WHERE c.session_id=s.id))) LIMIT 128)")
            .bind(cutoff).execute(&self.backend).await?;
        Ok(())
    }
    /// Raw cookie/native authentication must only accept kind zero. Internal IDs
    /// are never returned by an API, even though they share the session table.
    pub async fn linked_session_kind(&self, session: &str) -> Result<Option<i64>, StoreError> {
        Ok(q("SELECT CAST(linked_internal AS BIGINT) AS linked_internal FROM sessions WHERE id=?1")
            .bind(session)
            .fetch_optional(&self.backend)
            .await?
            .map(|r| r.get_i64("linked_internal")))
    }

    pub async fn linked_accounts(&self, parent: &str) -> Result<Vec<LinkedAccount>, StoreError> {
        Ok(q("SELECT l.id,l.account_id,l.display_name,l.revision,l.created_at FROM linked_account l JOIN session_principal sp ON sp.principal_id=l.owner_principal_id JOIN sessions s ON s.id=sp.session_id AND s.account_id=sp.primary_account_id JOIN identity_principal p ON p.id=sp.principal_id JOIN identity_binding b ON b.id=sp.binding_id AND b.principal_id=p.id WHERE sp.session_id=?1 AND s.linked_internal=0 AND p.state='active' AND b.retired_at IS NULL ORDER BY l.created_at,l.id LIMIT 16")
            .bind(parent).fetch_all(&self.backend).await?.iter().map(projection).collect())
    }

    /// Promote a separately completed password(+factor) login into a private
    /// linked source. The handler must enforce both accounts' current policy and
    /// clocks. This transaction repeats provenance checks and the hard quota.
    pub async fn attach_verified_link(
        &self,
        parent: &str,
        child: &str,
        display: &str,
        maximum: usize,
        now: i64,
    ) -> Result<Option<LinkedAccount>, StoreError> {
        self.prune_linked_sessions().await?;
        let display = display.trim();
        if parent == child
            || display.is_empty()
            || display.chars().count() > 128
            || display.chars().any(char::is_control)
            || !(1..=16).contains(&maximum)
            || now < 0
        {
            return Ok(None);
        }
        let Some(subject) = self.session_personal_subject(child).await? else {
            return Ok(None);
        };
        if self.session_personal_subject(parent).await?.as_ref() == Some(&subject) {
            return Ok(None);
        }
        let child_snapshot = self.get_session(child).await?;
        let mut tx = self.backend.begin().await?;
        serialize(&mut tx).await?;
        let owner = q("SELECT sp.principal_id FROM session_principal sp JOIN sessions s ON s.id=sp.session_id AND s.account_id=sp.primary_account_id JOIN identity_principal p ON p.id=sp.principal_id JOIN identity_binding b ON b.id=sp.binding_id AND b.principal_id=p.id WHERE sp.session_id=?1 AND s.linked_internal=0 AND p.state='active' AND b.retired_at IS NULL")
            .bind(parent).fetch_all_tx(&mut tx).await?;
        let Some(owner) = owner.first().map(|r| r.get_string("principal_id")) else {
            return Ok(None);
        };
        let verified = q("SELECT sp.principal_id,s.account_id FROM session_principal sp JOIN sessions s ON s.id=sp.session_id AND s.account_id=sp.primary_account_id JOIN identity_principal p ON p.id=sp.principal_id JOIN identity_binding b ON b.id=sp.binding_id AND b.principal_id=p.id WHERE sp.session_id=?1 AND s.linked_internal=0 AND s.authentication_method IN ('password','password+factor') AND s.jmap_url=?2 AND s.api_url=?3 AND p.state='active' AND b.retired_at IS NULL")
            .bind(child).bind(&child_snapshot.jmap_url).bind(&child_snapshot.api_url).fetch_all_tx(&mut tx).await?;
        let Some(verified) = verified.first() else {
            return Ok(None);
        };
        let count = q("SELECT COUNT(*) AS total FROM linked_account WHERE owner_principal_id=?1")
            .bind(&owner)
            .fetch_all_tx(&mut tx)
            .await?[0]
            .get_i64("total");
        if count >= maximum as i64 {
            return Ok(None);
        }
        if !q("SELECT id FROM linked_account WHERE owner_principal_id=?1 AND child_subject=?2")
            .bind(&owner)
            .bind(&subject)
            .fetch_all_tx(&mut tx)
            .await?
            .is_empty()
        {
            return Ok(None);
        }
        let id = uuid::Uuid::new_v4().to_string();
        let revision = uuid::Uuid::new_v4().to_string();
        let sealed = self.key.seal_bound(
            child.as_bytes(),
            "linked-source-v1",
            &[&owner, &id, &subject],
        )?;
        q("INSERT INTO linked_account(id,owner_principal_id,child_principal_id,child_subject,account_id,display_name,sealed_session,revision,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)")
            .bind(&id).bind(&owner).bind(verified.get_string("principal_id")).bind(&subject)
            .bind(&child_snapshot.account_id).bind(display).bind(sealed).bind(&revision).bind(now).execute_tx(&mut tx).await?;
        q("UPDATE sessions SET linked_internal=1,client_type='linked',device_label='Linked mailbox access' WHERE id=?1")
            .bind(child)
            .execute_tx(&mut tx)
            .await?;
        tx.commit().await?;
        Ok(Some(LinkedAccount {
            id,
            account_id: child_snapshot.account_id,
            display_name: display.into(),
            revision,
            created_at: now,
        }))
    }

    /// Resolve the source only for a live verified primary owner. The sealed
    /// reference is bound to owner+link+child subject, never an untrusted selector.
    pub async fn linked_source(
        &self,
        parent: &str,
        link: &str,
    ) -> Result<Option<(LinkedAccount, Session)>, StoreError> {
        let Some(row) = q("SELECT l.id,l.account_id,l.display_name,l.revision,l.created_at,l.owner_principal_id,l.child_principal_id,l.child_subject,l.sealed_session FROM linked_account l JOIN session_principal sp ON sp.principal_id=l.owner_principal_id JOIN sessions s ON s.id=sp.session_id AND s.account_id=sp.primary_account_id JOIN identity_principal p ON p.id=sp.principal_id JOIN identity_binding b ON b.id=sp.binding_id AND b.principal_id=p.id WHERE l.id=?1 AND sp.session_id=?2 AND s.linked_internal=0 AND p.state='active' AND b.retired_at IS NULL")
            .bind(link).bind(parent).fetch_optional(&self.backend).await? else { return Ok(None) };
        let owner = row.get_string("owner_principal_id");
        let subject = row.get_string("child_subject");
        let bytes = self.key.open_bound(
            &row.get_blob("sealed_session"),
            "linked-source-v1",
            &[&owner, link, &subject],
        )?;
        let id = String::from_utf8(bytes)
            .map_err(|_| StoreError::Corrupt("invalid linked source".into()))?;
        if self.linked_session_kind(&id).await? != Some(1) {
            return Ok(None);
        }
        let Some(principal) = self.primary_principal(&id).await? else {
            return Ok(None);
        };
        if principal.principal_id != row.get_string("child_principal_id")
            || self.session_personal_subject(&id).await?.as_ref() != Some(&subject)
        {
            return Ok(None);
        }
        let session = match self.get_session(&id).await {
            Ok(s) => s,
            Err(StoreError::NotFound) => return Ok(None),
            Err(e) => return Err(e),
        };
        if session.account_id != row.get_string("account_id") {
            return Ok(None);
        }
        Ok(Some((projection(&row), session)))
    }

    /// One private delegated session per primary session/link. Cloning preserves
    /// original authentication clocks and verified binding, including MFA age.
    pub async fn linked_context(
        &self,
        parent: &str,
        link: &str,
    ) -> Result<Option<Session>, StoreError> {
        let Some((entry, source)) = self.linked_source(parent, link).await? else {
            return Ok(None);
        };
        // Normal requests use a read-only fast path. Final admission revalidates
        // authority, so a concurrent unlink cannot turn this read into a grant.
        if let Some(row)=q("SELECT c.session_id FROM linked_context c JOIN sessions s ON s.id=c.session_id WHERE c.owner_session_id=?1 AND c.link_id=?2 AND c.link_revision=?3 AND s.linked_internal=2")
            .bind(parent).bind(link).bind(&entry.revision).fetch_optional(&self.backend).await? {
            return match self.get_session(&row.get_string("session_id")).await {
                Ok(session)=>Ok(Some(session)),Err(StoreError::NotFound)=>Ok(None),Err(error)=>Err(error)
            };
        }
        self.prune_linked_sessions().await?;
        let mut tx = self.backend.begin().await?;
        serialize(&mut tx).await?;
        let permitted = q("SELECT l.id FROM linked_account l JOIN session_principal own ON own.principal_id=l.owner_principal_id JOIN sessions p ON p.id=own.session_id JOIN identity_principal op ON op.id=own.principal_id JOIN identity_binding ob ON ob.id=own.binding_id JOIN session_principal child ON child.principal_id=l.child_principal_id JOIN sessions s ON s.id=child.session_id JOIN identity_principal cp ON cp.id=child.principal_id JOIN identity_binding cb ON cb.id=child.binding_id WHERE l.id=?1 AND l.revision=?2 AND p.id=?3 AND p.linked_internal=0 AND s.id=?4 AND s.linked_internal=1 AND op.state='active' AND cp.state='active' AND ob.retired_at IS NULL AND cb.retired_at IS NULL")
            .bind(link).bind(&entry.revision).bind(parent).bind(&source.id).fetch_all_tx(&mut tx).await?;
        if permitted.is_empty() {
            return Ok(None);
        }
        let existing = q("SELECT c.session_id FROM linked_context c JOIN sessions s ON s.id=c.session_id WHERE c.owner_session_id=?1 AND c.link_id=?2 AND c.link_revision=?3 AND s.linked_internal=2")
            .bind(parent).bind(link).bind(&entry.revision).fetch_all_tx(&mut tx).await?;
        let id = if let Some(row) = existing.first() {
            row.get_string("session_id")
        } else {
            let id = crate::seal::random_token();
            q("INSERT INTO sessions(id,account_id,username,jmap_url,api_url,sealed_creds,created_at,last_seen,activity_at,authentication_method,user_agent,ip,client_type,device_label,client_version,linked_internal) SELECT ?1,account_id,username,jmap_url,api_url,sealed_creds,created_at,last_seen,activity_at,authentication_method,user_agent,ip,'linked',device_label,client_version,2 FROM sessions WHERE id=?2 AND linked_internal=1")
                .bind(&id).bind(&source.id).execute_tx(&mut tx).await?;
            q("INSERT INTO session_principal(session_id,principal_id,binding_id,primary_account_id,auth_time,authentication_method) SELECT ?1,principal_id,binding_id,primary_account_id,auth_time,authentication_method FROM session_principal WHERE session_id=?2")
                .bind(&id).bind(&source.id).execute_tx(&mut tx).await?;
            q("INSERT INTO linked_context(session_id,owner_session_id,link_id,link_revision) VALUES(?1,?2,?3,?4)")
                .bind(&id).bind(parent).bind(link).bind(&entry.revision).execute_tx(&mut tx).await?;
            id
        };
        tx.commit().await?;
        Ok(Some(self.get_session(&id).await?))
    }

    /// Called on every delegated admission, including signed resource requests.
    /// A cascaded association disappearing never makes an internal session normal.
    pub async fn linked_context_authority(
        &self,
        session: &str,
    ) -> Result<Option<LinkedSessionAuthority>, StoreError> {
        let Some(row) = q("SELECT c.owner_session_id,c.link_id,c.link_revision FROM linked_context c JOIN sessions s ON s.id=c.session_id WHERE c.session_id=?1 AND s.linked_internal=2")
            .bind(session).fetch_optional(&self.backend).await? else { return Ok(None) };
        let parent_id = row.get_string("owner_session_id");
        let Some((link, source)) = self
            .linked_source(&parent_id, &row.get_string("link_id"))
            .await?
        else {
            return Ok(None);
        };
        if link.revision != row.get_string("link_revision")
            || self.primary_principal(session).await? != self.primary_principal(&source.id).await?
        {
            return Ok(None);
        }
        let parent = match self.get_session(&parent_id).await {
            Ok(s) => s,
            Err(StoreError::NotFound) => return Ok(None),
            Err(e) => return Err(e),
        };
        Ok(Some(LinkedSessionAuthority {
            parent,
            source,
            link,
        }))
    }

    /// The link this session's owner marked as the mailbox to open first, as
    /// stored. It may name a link that was removed since; callers compare it
    /// with the current verified links before using it.
    pub async fn linked_primary(&self, parent: &str) -> Result<Option<String>, StoreError> {
        Ok(q("SELECT lp.link_id FROM linked_primary lp JOIN session_principal sp ON sp.principal_id=lp.owner_principal_id JOIN sessions s ON s.id=sp.session_id AND s.account_id=sp.primary_account_id JOIN identity_principal p ON p.id=sp.principal_id JOIN identity_binding b ON b.id=sp.binding_id AND b.principal_id=p.id WHERE sp.session_id=?1 AND s.linked_internal=0 AND p.state='active' AND b.retired_at IS NULL")
            .bind(parent).fetch_optional(&self.backend).await?.map(|r| r.get_string("link_id")))
    }

    /// Mark one of the owner's own links as the mailbox to open first, or clear
    /// the mark (`None`: the primary account). Returns false when the session
    /// has no verified owner or the link is not one of that owner's. This only
    /// records a preference: opening the link still goes through the normal
    /// switch authorization.
    pub async fn set_linked_primary(
        &self,
        parent: &str,
        link: Option<&str>,
        now: i64,
    ) -> Result<bool, StoreError> {
        let mut tx = self.backend.begin().await?;
        serialize(&mut tx).await?;
        let owner = q("SELECT sp.principal_id FROM session_principal sp JOIN sessions s ON s.id=sp.session_id AND s.account_id=sp.primary_account_id JOIN identity_principal p ON p.id=sp.principal_id JOIN identity_binding b ON b.id=sp.binding_id AND b.principal_id=p.id WHERE sp.session_id=?1 AND s.linked_internal=0 AND p.state='active' AND b.retired_at IS NULL")
            .bind(parent).fetch_all_tx(&mut tx).await?;
        let Some(owner) = owner.first().map(|r| r.get_string("principal_id")) else {
            return Ok(false);
        };
        match link {
            None => {
                q("DELETE FROM linked_primary WHERE owner_principal_id=?1")
                    .bind(&owner)
                    .execute_tx(&mut tx)
                    .await?;
            }
            Some(link) => {
                let owned = q("SELECT id FROM linked_account WHERE id=?1 AND owner_principal_id=?2")
                    .bind(link)
                    .bind(&owner)
                    .fetch_all_tx(&mut tx)
                    .await?;
                if owned.is_empty() {
                    return Ok(false);
                }
                q("INSERT INTO linked_primary(owner_principal_id,link_id,updated_at) VALUES(?1,?2,?3) ON CONFLICT(owner_principal_id) DO UPDATE SET link_id=excluded.link_id,updated_at=excluded.updated_at")
                    .bind(&owner).bind(link).bind(now).execute_tx(&mut tx).await?;
            }
        }
        tx.commit().await?;
        Ok(true)
    }

    /// The owner's stored mailbox arrangement: `(order_json, nicknames_json,
    /// revision)`. Validation against the current links is the caller's.
    pub async fn linked_layout(
        &self,
        parent: &str,
    ) -> Result<Option<(String, String, String)>, StoreError> {
        Ok(q("SELECT ll.order_json,ll.nicknames_json,ll.revision FROM linked_layout ll JOIN session_principal sp ON sp.principal_id=ll.owner_principal_id JOIN sessions s ON s.id=sp.session_id AND s.account_id=sp.primary_account_id JOIN identity_principal p ON p.id=sp.principal_id JOIN identity_binding b ON b.id=sp.binding_id AND b.principal_id=p.id WHERE sp.session_id=?1 AND s.linked_internal=0 AND p.state='active' AND b.retired_at IS NULL")
            .bind(parent).fetch_optional(&self.backend).await?
            .map(|r| (r.get_string("order_json"), r.get_string("nicknames_json"), r.get_string("revision"))))
    }

    /// Replace the arrangement when `expected` is its current revision (`""`
    /// when none is stored yet). Returns the new revision, or `None` when the
    /// session has no verified owner or the revision moved on.
    pub async fn put_linked_layout(
        &self,
        parent: &str,
        expected: &str,
        order_json: &str,
        nicknames_json: &str,
        now: i64,
    ) -> Result<Option<String>, StoreError> {
        let mut tx = self.backend.begin().await?;
        serialize(&mut tx).await?;
        let owner = q("SELECT sp.principal_id FROM session_principal sp JOIN sessions s ON s.id=sp.session_id AND s.account_id=sp.primary_account_id JOIN identity_principal p ON p.id=sp.principal_id JOIN identity_binding b ON b.id=sp.binding_id AND b.principal_id=p.id WHERE sp.session_id=?1 AND s.linked_internal=0 AND p.state='active' AND b.retired_at IS NULL")
            .bind(parent).fetch_all_tx(&mut tx).await?;
        let Some(owner) = owner.first().map(|r| r.get_string("principal_id")) else {
            return Ok(None);
        };
        let current = q("SELECT revision FROM linked_layout WHERE owner_principal_id=?1")
            .bind(&owner)
            .fetch_all_tx(&mut tx)
            .await?
            .first()
            .map(|r| r.get_string("revision"))
            .unwrap_or_default();
        if current != expected {
            return Ok(None);
        }
        let revision = uuid::Uuid::new_v4().to_string();
        q("INSERT INTO linked_layout(owner_principal_id,order_json,nicknames_json,revision,updated_at) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(owner_principal_id) DO UPDATE SET order_json=excluded.order_json,nicknames_json=excluded.nicknames_json,revision=excluded.revision,updated_at=excluded.updated_at")
            .bind(&owner).bind(order_json).bind(nicknames_json).bind(&revision).bind(now).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(Some(revision))
    }

    /// Exact-revision unlink. Mailbox data and unrelated child sessions survive.
    pub async fn unlink_account(
        &self,
        parent: &str,
        link: &str,
        revision: &str,
    ) -> Result<bool, StoreError> {
        let mut tx = self.backend.begin().await?;
        serialize(&mut tx).await?;
        let rows = q("SELECT l.owner_principal_id,l.child_subject,l.sealed_session FROM linked_account l JOIN session_principal sp ON sp.principal_id=l.owner_principal_id JOIN sessions s ON s.id=sp.session_id JOIN identity_principal p ON p.id=sp.principal_id JOIN identity_binding b ON b.id=sp.binding_id WHERE l.id=?1 AND l.revision=?2 AND sp.session_id=?3 AND s.linked_internal=0 AND p.state='active' AND b.retired_at IS NULL")
            .bind(link).bind(revision).bind(parent).fetch_all_tx(&mut tx).await?;
        let Some(row) = rows.first() else {
            return Ok(false);
        };
        let source = self.key.open_bound(
            &row.get_blob("sealed_session"),
            "linked-source-v1",
            &[
                &row.get_string("owner_principal_id"),
                link,
                &row.get_string("child_subject"),
            ],
        )?;
        let source = String::from_utf8(source)
            .map_err(|_| StoreError::Corrupt("invalid linked source".into()))?;
        q("DELETE FROM sessions WHERE id IN (SELECT session_id FROM linked_context WHERE link_id=?1)").bind(link).execute_tx(&mut tx).await?;
        q("DELETE FROM sessions WHERE id=?1 AND linked_internal=1")
            .bind(source)
            .execute_tx(&mut tx)
            .await?;
        q("DELETE FROM linked_account WHERE id=?1")
            .bind(link)
            .execute_tx(&mut tx)
            .await?;
        tx.commit().await?;
        Ok(true)
    }

    /// Keep the normal login factor token server-side and owner-session bound.
    /// Four outstanding challenges per parent; expiry is ten minutes at most.
    pub async fn create_linked_challenge(
        &self,
        parent: &str,
        pending: &str,
        display: &str,
        now: i64,
    ) -> Result<Option<String>, StoreError> {
        if pending.is_empty()
            || pending.len() > 4096
            || display.chars().count() > 128
            || display.chars().any(char::is_control)
            || now < 0
        {
            return Ok(None);
        }
        let mut tx = self.backend.begin().await?;
        serialize(&mut tx).await?;
        q("DELETE FROM linked_challenge WHERE expires_at<=?1")
            .bind(now)
            .execute_tx(&mut tx)
            .await?;
        let owner = q("SELECT sp.session_id FROM session_principal sp JOIN sessions s ON s.id=sp.session_id JOIN identity_principal p ON p.id=sp.principal_id JOIN identity_binding b ON b.id=sp.binding_id WHERE sp.session_id=?1 AND s.linked_internal=0 AND p.state='active' AND b.retired_at IS NULL")
            .bind(parent).fetch_all_tx(&mut tx).await?;
        if owner.is_empty() {
            return Ok(None);
        }
        let count = q("SELECT COUNT(*) AS total FROM linked_challenge WHERE owner_session_id=?1")
            .bind(parent)
            .fetch_all_tx(&mut tx)
            .await?[0]
            .get_i64("total");
        if count >= 4 {
            return Ok(None);
        }
        let id = uuid::Uuid::new_v4().to_string();
        let sealed =
            self.key
                .seal_bound(pending.as_bytes(), "linked-challenge-v1", &[parent, &id])?;
        q("INSERT INTO linked_challenge(id,owner_session_id,sealed_pending,display_name,expires_at) VALUES(?1,?2,?3,?4,?5)")
            .bind(&id).bind(parent).bind(sealed).bind(display).bind(now.saturating_add(600)).execute_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(Some(id))
    }

    pub async fn linked_challenge(
        &self,
        parent: &str,
        id: &str,
        now: i64,
    ) -> Result<Option<(String, String)>, StoreError> {
        let Some(row) = q("SELECT c.sealed_pending,c.display_name FROM linked_challenge c JOIN sessions s ON s.id=c.owner_session_id JOIN session_principal sp ON sp.session_id=s.id JOIN identity_principal p ON p.id=sp.principal_id JOIN identity_binding b ON b.id=sp.binding_id WHERE c.id=?1 AND c.owner_session_id=?2 AND c.expires_at>?3 AND s.linked_internal=0 AND p.state='active' AND b.retired_at IS NULL")
            .bind(id).bind(parent).bind(now).fetch_optional(&self.backend).await? else { return Ok(None) };
        let token = self.key.open_bound(
            &row.get_blob("sealed_pending"),
            "linked-challenge-v1",
            &[parent, id],
        )?;
        let token = String::from_utf8(token)
            .map_err(|_| StoreError::Corrupt("invalid linked challenge".into()))?;
        Ok(Some((token, row.get_string("display_name"))))
    }

    pub async fn delete_linked_challenge(&self, parent: &str, id: &str) -> Result<(), StoreError> {
        q("DELETE FROM linked_challenge WHERE id=?1 AND owner_session_id=?2")
            .bind(id)
            .bind(parent)
            .execute(&self.backend)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Credentials, ServerKey, identity_principal::VerifiedPrimary};

    async fn verified(store: &Store, realm: &str, raw: &str) -> String {
        let url = format!("https://mail.test/{realm}/session");
        let id = store
            .create_session(
                raw,
                "same@mail.test",
                &url,
                &format!("https://mail.test/{realm}/api"),
                &Credentials {
                    username: "same".into(),
                    password: "synthetic".into(),
                },
            )
            .await
            .unwrap();
        store
            .set_session_authentication(&id, "password+factor")
            .await
            .unwrap();
        store
            .bind_verified_primary(
                &id,
                VerifiedPrimary {
                    account_id: raw,
                    kind: "mailbox",
                    authority: realm,
                    subject: "verified",
                },
                100,
            )
            .await
            .unwrap()
            .unwrap();
        id
    }

    #[tokio::test]
    async fn linked_accounts_copy_preserves_owner_and_internal_session_marker() {
        let root = std::env::temp_dir().join(format!("mw-linked-copy-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("source.sqlite");
        let key = || ServerKey::from_bytes(&[83; 32]).unwrap();
        let source = Store::open(path.to_str().unwrap(), key()).await.unwrap();
        let parent = verified(&source, "copy-parent", "1").await;
        let child = verified(&source, "copy-child", "1").await;
        let link = source
            .attach_verified_link(&parent, &child, "Copied", 2, 200)
            .await
            .unwrap()
            .unwrap();
        let context = source
            .linked_context(&parent, &link.id)
            .await
            .unwrap()
            .unwrap();
        let challenge = source
            .create_linked_challenge(&parent, "process-local-factor", "Child", 300)
            .await
            .unwrap()
            .unwrap();
        let destination = Store::open_in_memory(key()).await.unwrap();
        destination
            .migrate_from_sqlite(path.to_str().unwrap())
            .await
            .unwrap();
        assert_eq!(
            destination.linked_session_kind(&context.id).await.unwrap(),
            Some(2)
        );
        assert_eq!(
            destination.linked_session_kind(&child).await.unwrap(),
            Some(1)
        );
        assert!(
            destination
                .linked_context_authority(&context.id)
                .await
                .unwrap()
                .is_some()
        );
        assert_eq!(
            destination
                .session_storage_authority(&context.id)
                .await
                .unwrap(),
            source.session_storage_authority(&context.id).await.unwrap()
        );
        assert!(
            destination
                .linked_challenge(&parent, &challenge, 301)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            destination
                .unlink_account(&parent, &link.id, &link.revision)
                .await
                .unwrap()
        );
        assert!(
            source
                .linked_context_authority(&context.id)
                .await
                .unwrap()
                .is_some()
        );
        drop(source);
        drop(destination);
        // Test-owned exact directory only; no shared runtime or saved user data.
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn linked_accounts_live_postgres_serializes_quota_and_copies_authority() {
        let Ok(dsn) = std::env::var("MW_LINKED_TEST_PG") else {
            return;
        };
        let store = Store::open_postgres(&dsn, ServerKey::from_bytes(&[84; 32]).unwrap())
            .await
            .unwrap();
        let unique = uuid::Uuid::new_v4().to_string();
        let parent = verified(&store, &format!("parent-{unique}"), "1").await;
        let first = verified(&store, &format!("first-{unique}"), "1").await;
        let second = verified(&store, &format!("second-{unique}"), "1").await;
        let (a, b) = tokio::join!(
            store.attach_verified_link(&parent, &first, "First", 1, 200),
            store.attach_verified_link(&parent, &second, "Second", 1, 200)
        );
        let a = a.unwrap();
        let b = b.unwrap();
        assert_ne!(a.is_some(), b.is_some());
        let link = a.or(b).unwrap();
        let (a, b) = tokio::join!(
            store.linked_context(&parent, &link.id),
            store.linked_context(&parent, &link.id)
        );
        let context = a.unwrap().unwrap();
        assert_eq!(context.id, b.unwrap().unwrap().id);
        assert!(
            store
                .linked_context_authority(&context.id)
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            store
                .unlink_account(&parent, &link.id, &link.revision)
                .await
                .unwrap()
        );
        assert!(
            store
                .linked_context_authority(&context.id)
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn linked_accounts_preserve_child_authority_and_revoke_with_parent() {
        let store = Store::open_in_memory(ServerKey::from_bytes(&[81; 32]).unwrap())
            .await
            .unwrap();
        let parent = verified(&store, "parent", "1").await;
        let child = verified(&store, "child", "1").await;
        let foreign = verified(&store, "foreign", "1").await;
        let child_authority = store.session_storage_authority(&child).await.unwrap();
        let clock = store.session_activity_clock(&child).await.unwrap();
        let principal = store.primary_principal(&child).await.unwrap();
        let link = store
            .attach_verified_link(&parent, &child, "Child", 2, 200)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(store.linked_session_kind(&child).await.unwrap(), Some(1));
        assert!(
            store
                .linked_source(&foreign, &link.id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(store.linked_accounts(&foreign).await.unwrap().is_empty());
        assert!(
            !store
                .unlink_account(&foreign, &link.id, &link.revision)
                .await
                .unwrap()
        );
        let context = store
            .linked_context(&parent, &link.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            store.linked_session_kind(&context.id).await.unwrap(),
            Some(2)
        );
        assert_eq!(
            store.session_storage_authority(&context.id).await.unwrap(),
            child_authority
        );
        assert_eq!(
            store.session_activity_clock(&context.id).await.unwrap(),
            clock
        );
        assert_eq!(
            store.primary_principal(&context.id).await.unwrap(),
            principal
        );
        assert_ne!(
            store.session_storage_authority(&parent).await.unwrap(),
            child_authority
        );
        assert_eq!(
            store
                .linked_context(&parent, &link.id)
                .await
                .unwrap()
                .unwrap()
                .id,
            context.id
        );
        assert!(
            store
                .linked_context_authority(&context.id)
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            store
                .attach_verified_link(&context.id, &foreign, "Nested", 2, 201)
                .await
                .unwrap()
                .is_none()
        );
        store.delete_session(&parent).await.unwrap();
        assert!(
            store
                .linked_context_authority(&context.id)
                .await
                .unwrap()
                .is_none()
        );
        // Cascading association removal must not turn the retained ID into a login.
        assert_eq!(
            store.linked_session_kind(&context.id).await.unwrap(),
            Some(2)
        );
        assert!(store.get_session(&child).await.is_ok());
    }

    #[tokio::test]
    async fn linked_accounts_quota_unlink_and_challenge_owner_fences() {
        let store = Store::open_in_memory(ServerKey::from_bytes(&[82; 32]).unwrap())
            .await
            .unwrap();
        let parent = verified(&store, "parent", "1").await;
        let child = verified(&store, "child", "2").await;
        let independent_child = verified(&store, "child", "2").await;
        let third = verified(&store, "third", "3").await;
        let link = store
            .attach_verified_link(&parent, &child, "Child", 1, 200)
            .await
            .unwrap()
            .unwrap();
        assert!(
            store
                .attach_verified_link(&parent, &third, "Third", 1, 200)
                .await
                .unwrap()
                .is_none()
        );
        let context = store
            .linked_context(&parent, &link.id)
            .await
            .unwrap()
            .unwrap();
        assert!(
            !store
                .unlink_account(&parent, &link.id, "stale")
                .await
                .unwrap()
        );
        assert!(
            store
                .unlink_account(&parent, &link.id, &link.revision)
                .await
                .unwrap()
        );
        assert!(store.get_session(&context.id).await.is_err());
        assert!(store.get_session(&child).await.is_err());
        assert!(store.get_session(&independent_child).await.is_ok());
        let challenge = store
            .create_linked_challenge(&parent, "secret-pending", "Child", 300)
            .await
            .unwrap()
            .unwrap();
        assert!(
            store
                .linked_challenge(&third, &challenge, 301)
                .await
                .unwrap()
                .is_none()
        );
        assert_eq!(
            store
                .linked_challenge(&parent, &challenge, 899)
                .await
                .unwrap()
                .unwrap()
                .0,
            "secret-pending"
        );
        assert!(
            store
                .linked_challenge(&parent, &challenge, 900)
                .await
                .unwrap()
                .is_none()
        );
        for _ in 0..3 {
            assert!(
                store
                    .create_linked_challenge(&parent, "pending", "Child", 301)
                    .await
                    .unwrap()
                    .is_some()
            );
        }
        assert!(
            store
                .create_linked_challenge(&parent, "pending", "Child", 301)
                .await
                .unwrap()
                .is_none()
        );
        store.delete_session(&parent).await.unwrap();
        assert!(
            store
                .linked_challenge(&parent, &challenge, 302)
                .await
                .unwrap()
                .is_none()
        );
    }
}
