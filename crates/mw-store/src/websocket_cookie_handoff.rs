//! Purpose/scope-bound transient cookie-delivery ciphertext; no SQL or sessions
//! are created by this helper. Server-only typed issuers own the payload schema.
use crate::backend::{Backend, Dialect, q};
use crate::{Store, StoreError};
use std::sync::OnceLock;
const MAX_SEALED: usize = 8192;
impl Store {
    pub fn seal_websocket_cookie_handoff(
        &self,
        binding: &str,
        bytes: &[u8],
    ) -> Result<Vec<u8>, StoreError> {
        if binding.is_empty() || binding.len() > 4096 || bytes.len() > MAX_SEALED - 44 {
            return Err(StoreError::Invalid("cookie delivery bound"));
        }
        Ok(self.key.seal_bound(
            bytes,
            "mw-websocket-cookie-delivery-v1",
            &[&self.cache_scope().to_string(), binding],
        )?)
    }
    pub fn open_websocket_cookie_handoff(
        &self,
        binding: &str,
        sealed: &[u8],
    ) -> Result<Vec<u8>, StoreError> {
        if binding.is_empty() || binding.len() > 4096 || sealed.len() > MAX_SEALED {
            return Err(StoreError::Invalid("cookie delivery bound"));
        }
        Ok(self.key.open_bound(
            sealed,
            "mw-websocket-cookie-delivery-v1",
            &[&self.cache_scope().to_string(), binding],
        )?)
    }
    /// Fresh primary actor metadata only. A socket retains this identity, never
    /// an authenticated Session/grant. Linked selections remain request-owned.
    pub async fn websocket_primary_actor(
        &self,
        session: &str,
    ) -> Result<Option<WebsocketPrimaryActor>, StoreError> {
        if session.is_empty() || session.len() > 1024 {
            return Ok(None);
        }
        let before = match &self.backend {
            Backend::Sqlite(live) => Some(live.pin().await.active_options().get_filename().to_path_buf()),
            Backend::Postgres(_) => None,
        };
        let row = q(actor_sql(self.backend.dialect()))
            .bind(session)
            .fetch_optional(&self.backend)
            .await?;
        let after = match &self.backend {
            Backend::Sqlite(live) => Some(live.pin().await.active_options().get_filename().to_path_buf()),
            Backend::Postgres(_) => None,
        };
        if before != after {
            return Err(StoreError::SourceChanged);
        }
        let Some(row) = row else {
            return Ok(None);
        };
        let mut fields = Vec::new();
        for key in [
            "id",
            "account_id",
            "username",
            "jmap_url",
            "api_url",
            "principal_id",
            "binding_id",
        ] {
            let Some(value) = row.try_get_opt_string(key)? else {
                return Ok(None);
            };
            fields.push(value);
        }
        let fields: [String; 7] = fields
            .try_into()
            .map_err(|_| StoreError::Invalid("socket actor shape"))?;
        Ok(Some(WebsocketPrimaryActor {
            scope: self.cache_scope(),
            fields,
        }))
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct WebsocketPrimaryActor {
    scope: uuid::Uuid,
    fields: [String; 7],
}
impl std::fmt::Debug for WebsocketPrimaryActor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebsocketPrimaryActor")
            .finish_non_exhaustive()
    }
}
impl WebsocketPrimaryActor {
    pub fn session_id(&self) -> &str {
        &self.fields[0]
    }
    pub fn account_id(&self) -> &str {
        &self.fields[1]
    }
    /// Pure comparison using a native composite's already-fresh fields. This
    /// never authenticates or substitutes for the caller's complete checks.
    pub fn matches_native(
        &self,
        store: &Store,
        session: &crate::Session,
        principal: &str,
        binding: &str,
    ) -> bool {
        self.scope == store.cache_scope()
            && self.fields.iter().map(String::as_str).eq([
                session.id.as_str(),
                session.account_id.as_str(),
                session.username.as_str(),
                session.jmap_url.as_str(),
                session.api_url.as_str(),
                principal,
                binding,
            ])
    }
}
fn actor_sql(dialect: Dialect) -> &'static str {
    static SQL: OnceLock<[String; 2]> = OnceLock::new();
    &SQL.get_or_init(||std::array::from_fn(|pg|{
        let fields=[("id","s.id"),("account_id","s.account_id"),("username","s.username"),("jmap_url","s.jmap_url"),("api_url","s.api_url"),("principal_id","sp.principal_id"),("binding_id","sp.binding_id")];
        let columns=fields.iter().map(|(key,value)|{
            let typed=if pg==1{"TRUE".into()}else{format!("typeof({value})='text'")};
            format!("CASE WHEN {typed} AND octet_length({value})<=1024 THEN {value} ELSE CAST(NULL AS TEXT) END AS {key}")
        }).collect::<Vec<_>>().join(",");
        format!("SELECT {columns} FROM sessions s JOIN session_principal sp ON sp.session_id=s.id AND sp.primary_account_id=s.account_id JOIN identity_principal p ON p.id=sp.principal_id JOIN identity_binding b ON b.id=sp.binding_id AND b.principal_id=p.id WHERE s.id=?1 AND s.linked_internal=0 AND p.state='active' AND b.retired_at IS NULL")
    }))[usize::from(dialect==Dialect::Postgres)]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn cookie_grant_is_bound_to_store_and_exact_delivery_token_and_bounded() {
        let key = crate::ServerKey::generate();
        let first = Store::open_in_memory(key.clone()).await.unwrap();
        let second = Store::open_in_memory(key).await.unwrap();
        let payload = b"fixture-only-server-cookie-grant";
        let sealed = first
            .seal_websocket_cookie_handoff("receipt-a", payload)
            .unwrap();
        assert_eq!(
            first
                .open_websocket_cookie_handoff("receipt-a", &sealed)
                .unwrap(),
            payload
        );
        assert!(
            first
                .open_websocket_cookie_handoff("receipt-b", &sealed)
                .is_err()
        );
        assert!(
            second
                .open_websocket_cookie_handoff("receipt-a", &sealed)
                .is_err()
        );
        assert!(
            first
                .seal_websocket_cookie_handoff("receipt-a", &vec![0; 8192])
                .is_err()
        );
        assert!(
            first
                .open_websocket_cookie_handoff("receipt-a", &vec![0; 8193])
                .is_err()
        );
    }
    #[tokio::test]
    async fn socket_primary_actor_tracks_authoritative_identity_without_credentials() {
        let store = Store::open_in_memory(crate::ServerKey::generate())
            .await
            .unwrap();
        let credentials = crate::Credentials {
            username: "owner@socket.test".into(),
            password: "fixture-not-an-upstream-secret".into(),
        };
        let account = store
            .create_account(
                &crate::NewAccount {
                    kind: crate::AccountKind::Imap,
                    host: "localhost",
                    port: 9,
                    tls: "starttls",
                    username: &credentials.username,
                    sync_policy_json: "{}",
                },
                &credentials,
            )
            .await
            .unwrap();
        let session = store
            .create_session(
                &account,
                &credentials.username,
                "engine",
                "engine",
                &credentials,
            )
            .await
            .unwrap();
        store
            .bind_verified_primary(
                &session,
                crate::identity_principal::VerifiedPrimary {
                    account_id: &account,
                    kind: "mailbox",
                    authority: "socket-test",
                    subject: &account,
                },
                chrono::Utc::now().timestamp(),
            )
            .await
            .unwrap()
            .unwrap();
        let actor = store
            .websocket_primary_actor(&session)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(actor.account_id(), account);
        assert_eq!(actor.session_id(), session);
        let before = format!("{actor:?}");
        assert!(!before.contains(&session) && !before.contains(&account));
        store.touch_session(&session).await.unwrap();
        assert_eq!(
            store
                .websocket_primary_actor(&session)
                .await
                .unwrap()
                .unwrap(),
            actor
        );
        store.delete_session(&session).await.unwrap();
        assert!(
            store
                .websocket_primary_actor(&session)
                .await
                .unwrap()
                .is_none()
        );
    }
}
