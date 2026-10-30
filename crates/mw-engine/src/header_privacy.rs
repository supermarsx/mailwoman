//! Native account send-time policy, shared by SMTP and provider submitters.
use crate::{EngineError, MailSubmitter, Result};
use async_trait::async_trait;
use mw_smtp::{Outgoing, SubmissionResult, SubmitOptions};
use mw_store::{
    Store,
    settings_policy::{ResolveInput, SettingScope, SettingSubject, resolve},
};
use std::sync::Arc;

pub(crate) const KEY: &str = "privacy.stripMailerHeaders";
pub(crate) struct HeaderPrivacySubmitter {
    store: Store,
    account: String,
    inner: Arc<dyn MailSubmitter>,
}
impl HeaderPrivacySubmitter {
    pub(crate) fn new(store: Store, account: String, inner: Arc<dyn MailSubmitter>) -> Self {
        Self {
            store,
            account,
            inner,
        }
    }
    async fn prepare(&self, mut outgoing: Outgoing) -> Result<Outgoing> {
        // Registration accepts native engine account IDs, never proxy session IDs.
        // Read policy on every attempt so locks and overrides affect queued sends.
        let account = self.store.get_account(&self.account).await?;
        let address = account.username.trim().to_ascii_lowercase();
        let subject = SettingSubject {
            personal_owner: None,
            account_id: &self.account,
            domain: address.rsplit_once('@').map_or("", |(_, domain)| domain),
            address: &address,
        };
        let canonical = self.store.canonical_setting_subject(&subject).await?;
        let subject = canonical.as_subject();
        let policy = self.store.get_setting_policy(KEY).await?;
        let rows = self.store.setting_values_for_subject(&subject).await?;
        let values = rows
            .iter()
            .filter(|row| row.key == KEY)
            .filter(|row| match row.scope {
                SettingScope::User | SettingScope::AdminUser => row.scope_id == subject.account_id,
                SettingScope::Domain => {
                    !subject.domain.is_empty() && row.scope_id == subject.domain
                }
                SettingScope::Profile | SettingScope::Group => true,
            })
            .collect::<Vec<_>>();
        let effective = resolve(&ResolveInput {
            floor: "true",
            policy: policy.as_ref(),
            values: &values,
            ..Default::default()
        });
        let strip: bool = serde_json::from_str(&effective.value).map_err(|_| {
            EngineError::Unsupported("invalid outgoing header privacy policy".into())
        })?;
        if strip {
            let raw = mw_mime::strip_mailer_headers(&outgoing.raw)
                .map_err(|error| EngineError::Unsupported(error.to_string()))?;
            if let std::borrow::Cow::Owned(raw) = raw {
                outgoing.raw = raw;
            }
        }
        Ok(outgoing)
    }
}
#[async_trait]
impl MailSubmitter for HeaderPrivacySubmitter {
    fn probe_timeout(&self) -> std::time::Duration {
        self.inner.probe_timeout()
    }
    fn cached_dsn_support(&self) -> bool {
        self.inner.cached_dsn_support()
    }
    async fn probe_dsn(&self) -> Result<bool> {
        self.inner.probe_dsn().await
    }
    async fn submit(&self, outgoing: Outgoing) -> Result<SubmissionResult> {
        self.inner.submit(self.prepare(outgoing).await?).await
    }
    async fn submit_with(
        &self,
        outgoing: Outgoing,
        options: SubmitOptions,
    ) -> Result<SubmissionResult> {
        self.inner
            .submit_with(self.prepare(outgoing).await?, options)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mw_store::{
        AccountKind, Credentials, NewAccount, ServerKey,
        settings_policy::{SettingPolicyRow, SettingScope, SettingValueRow},
    };
    use std::sync::Mutex;
    #[derive(Default)]
    struct Capture {
        messages: Mutex<Vec<(Outgoing, SubmitOptions)>>,
    }
    #[async_trait]
    impl MailSubmitter for Capture {
        fn probe_timeout(&self) -> std::time::Duration {
            std::time::Duration::from_secs(32)
        }
        fn cached_dsn_support(&self) -> bool {
            true
        }
        async fn probe_dsn(&self) -> Result<bool> {
            Ok(true)
        }
        async fn submit(&self, outgoing: Outgoing) -> Result<SubmissionResult> {
            self.submit_with(outgoing, SubmitOptions::default()).await
        }
        async fn submit_with(
            &self,
            outgoing: Outgoing,
            options: SubmitOptions,
        ) -> Result<SubmissionResult> {
            self.messages.lock().unwrap().push((outgoing, options));
            Err(EngineError::SubmissionUncertain("recorded once".into()))
        }
    }
    async fn account(store: &Store, name: &str) -> String {
        store
            .create_account(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host: "mail.example.test",
                    port: 993,
                    tls: "implicit",
                    username: name,
                    sync_policy_json: "{}",
                },
                &Credentials {
                    username: name.into(),
                    password: "test".into(),
                },
            )
            .await
            .unwrap()
    }
    fn message() -> Outgoing {
        Outgoing {
            mail_from: "sender@example.test".into(),
            rcpt_to: vec!["recipient@example.test".into()],
            raw: b"From: sender@example.test\r\nUser-Agent: private\r\n\r\nX-Mailer: body\r\n"
                .to_vec(),
        }
    }
    #[tokio::test]
    async fn current_scoped_policy_default_override_lock_and_foreign_isolation() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let id = account(&store, "sender@example.test").await;
        let foreign = account(&store, "other@example.test").await;
        let capture = Arc::new(Capture::default());
        let wrapper = HeaderPrivacySubmitter::new(store.clone(), id.clone(), capture.clone());
        assert!(matches!(
            wrapper.submit(message()).await,
            Err(EngineError::SubmissionUncertain(_))
        ));
        assert_eq!(
            capture.messages.lock().unwrap()[0].0.raw,
            b"From: sender@example.test\r\n\r\nX-Mailer: body\r\n"
        );
        let mut row = SettingValueRow {
            scope: SettingScope::User,
            scope_id: foreign,
            key: KEY.into(),
            value: "false".into(),
            locked: false,
            updated_by: "test".into(),
            updated_at: 1,
        };
        store.put_setting_value(&row).await.unwrap();
        let _ = wrapper.submit(message()).await;
        assert_eq!(
            capture.messages.lock().unwrap()[1].0.raw,
            b"From: sender@example.test\r\n\r\nX-Mailer: body\r\n"
        );
        // An obsolete address-keyed override must not govern this account-keyed setting.
        row.scope_id = "sender@example.test".into();
        store.put_setting_value(&row).await.unwrap();
        let _ = wrapper.submit(message()).await;
        assert_eq!(
            capture.messages.lock().unwrap().last().unwrap().0.raw,
            b"From: sender@example.test\r\n\r\nX-Mailer: body\r\n"
        );
        capture.messages.lock().unwrap().pop();
        row.scope_id = id;
        store.put_setting_value(&row).await.unwrap();
        let _ = wrapper.submit(message()).await;
        assert_eq!(capture.messages.lock().unwrap()[2].0.raw, message().raw);
        store
            .put_setting_policy(&SettingPolicyRow {
                key: KEY.into(),
                global_value: Some("true".into()),
                allow_user_override: false,
                global_locked: true,
                updated_by: "admin".into(),
                updated_at: 2,
            })
            .await
            .unwrap();
        let _ = wrapper.submit(message()).await;
        assert_eq!(
            capture.messages.lock().unwrap()[3].0.raw,
            b"From: sender@example.test\r\n\r\nX-Mailer: body\r\n"
        );
    }
    #[tokio::test]
    async fn extension_options_and_uncertainty_delegate_without_retry() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let id = account(&store, "sender@example.test").await;
        let capture = Arc::new(Capture::default());
        let wrapper = HeaderPrivacySubmitter::new(store, id, capture.clone());
        assert!(wrapper.cached_dsn_support());
        assert_eq!(wrapper.probe_timeout(), std::time::Duration::from_secs(32));
        assert!(wrapper.probe_dsn().await.unwrap());
        let options = SubmitOptions {
            dsn: Some(mw_smtp::Dsn {
                ret: Some(mw_smtp::DsnRet::Hdrs),
                envid: Some("opaque".into()),
                notify: vec![mw_smtp::DsnNotify::Failure],
                orcpt: true,
            }),
            require_tls: true,
            use_chunking: true,
        };
        assert!(matches!(
            wrapper.submit_with(message(), options).await,
            Err(EngineError::SubmissionUncertain(_))
        ));
        let messages = capture.messages.lock().unwrap();
        assert_eq!(messages.len(), 1);
        assert!(messages[0].1.require_tls && messages[0].1.use_chunking);
        assert_eq!(
            messages[0].1.dsn.as_ref().unwrap().envid.as_deref(),
            Some("opaque")
        );
        assert_eq!(messages[0].0.mail_from, "sender@example.test");
    }
}
