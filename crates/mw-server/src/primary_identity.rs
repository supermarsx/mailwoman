//! Primary identity evidence captured by successful authentication handlers.
//! No existing-session or email-based backfill is permitted.
use crate::AppState;
use mw_store::{StoreError, identity_principal::VerifiedPrimary};
use sha2::{Digest, Sha256};

#[derive(Clone)]
pub(crate) struct VerifiedLoginIdentity {
    pub kind: &'static str,
    pub authority: String,
    pub subject: String,
}
impl VerifiedLoginIdentity {
    pub(crate) fn proxy(origin: &str, account: &str) -> Result<Self, StoreError> {
        let mut url = reqwest::Url::parse(origin)
            .map_err(|_| StoreError::Corrupt("invalid verified JMAP authority".into()))?;
        if !matches!(url.scheme(), "https" | "http")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || account.is_empty()
        {
            return Err(StoreError::Corrupt("missing verified JMAP identity".into()));
        }
        // Account IDs are scoped to a service, not an entire HTTPS origin.
        // Include the canonical path and query while storing no URL credentials
        // or query secrets in the durable identity authority.
        url.set_fragment(None);
        let realm: String = Sha256::digest(url.as_str().as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        Ok(Self {
            kind: "mailbox",
            authority: format!("jmap-url-v1:{realm}"),
            subject: account.into(),
        })
    }
    pub(crate) async fn engine(state: &AppState, account: &str) -> Result<Self, StoreError> {
        let row = state.store.get_account(account).await?;
        if row.kind == mw_store::AccountKind::Ews {
            let provider = state.store.provider_account(account).await?
                .ok_or_else(|| StoreError::Corrupt("missing verified provider binding".into()))?;
            return Ok(Self {
                kind: "mailbox",
                authority: format!("ews-url-v1:{}", mw_store::provider_accounts::ews_authority(&provider.endpoint_url)),
                subject: provider.backend_username,
            });
        }
        let protocol = match row.kind {
            mw_store::AccountKind::Imap => "imap",
            mw_store::AccountKind::Pop3 => "pop3",
            mw_store::AccountKind::Ews => return Err(StoreError::Corrupt("invalid provider binding".into())),
        };
        Ok(Self {
            kind: "mailbox",
            authority: format!("{protocol}:{}:{}", row.host.to_ascii_lowercase(), row.port),
            subject: row.id,
        })
    }
    pub(crate) fn header(subject: &str) -> Self {
        // Header authentication is its own deployment trust realm. It cannot
        // silently merge into password or external-SSO principals.
        Self {
            kind: "header",
            authority: "deployment:trusted-header".into(),
            subject: subject.into(),
        }
    }
    pub(crate) async fn bind(
        &self,
        state: &AppState,
        session: &str,
        account: &str,
    ) -> Result<(), StoreError> {
        let bound = state
            .store
            .bind_verified_primary(
                session,
                VerifiedPrimary {
                    account_id: account,
                    kind: self.kind,
                    authority: &self.authority,
                    subject: &self.subject,
                },
                chrono::Utc::now().timestamp(),
            )
            .await?;
        if bound.is_none() {
            return Err(StoreError::Corrupt(
                "verified primary identity is unavailable".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn proxy_identity_is_authority_and_verified_account_not_address() {
        let first =
            VerifiedLoginIdentity::proxy("https://mail.example/jmap/session", "upstream-account")
                .unwrap();
        let same = VerifiedLoginIdentity::proxy(
            "https://MAIL.example:443/jmap/session",
            "upstream-account",
        )
        .unwrap();
        let foreign =
            VerifiedLoginIdentity::proxy("https://other.example/jmap", "upstream-account").unwrap();
        assert_eq!(first.authority, same.authority);
        assert_ne!(first.authority, foreign.authority);
        assert_eq!(first.subject, "upstream-account");
        assert!(VerifiedLoginIdentity::proxy("file:///tmp/mail", "account").is_err());
        assert!(VerifiedLoginIdentity::proxy("https://mail.example", "").is_err());
    }
    #[tokio::test]
    async fn distinct_path_services_cannot_merge_the_same_upstream_account_id() {
        use mw_store::{Credentials, ServerKey, Store};
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let mut principals = Vec::new();
        for endpoint in [
            "https://mail.example/tenant-a/session",
            "https://mail.example/tenant-b/session",
            "https://mail.example/tenant-a/session?tenant=other",
        ] {
            let identity = VerifiedLoginIdentity::proxy(endpoint, "1").unwrap();
            let session = store
                .create_session(
                    "1",
                    "same@example.org",
                    endpoint,
                    endpoint,
                    &Credentials {
                        username: "same@example.org".into(),
                        password: "fixture".into(),
                    },
                )
                .await
                .unwrap();
            store
                .set_session_authentication(&session, "password")
                .await
                .unwrap();
            let principal = store
                .bind_verified_primary(
                    &session,
                    VerifiedPrimary {
                        account_id: "1",
                        kind: identity.kind,
                        authority: &identity.authority,
                        subject: &identity.subject,
                    },
                    1,
                )
                .await
                .unwrap()
                .unwrap();
            principals.push(principal.principal_id);
            assert!(!identity.authority.contains("tenant"));
        }
        assert_eq!(
            principals
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            3
        );
    }
}
