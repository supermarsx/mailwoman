//! Runtime-owned initial indexing, separate from credential authentication.
use crate::{Engine, backend::EngineError};
use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicU8, Ordering},
};
use std::time::Duration;
use tokio::sync::{Semaphore, watch};

pub(crate) struct InitialSyncControl {
    state: AtomicU8,
    cancel: watch::Sender<bool>,
}

impl Default for InitialSyncControl {
    fn default() -> Self {
        Self {
            state: AtomicU8::new(0),
            cancel: watch::channel(false).0,
        }
    }
}
impl InitialSyncControl {
    pub(crate) fn cancel(&self) {
        self.cancel.send_replace(true);
    }
    pub(crate) fn cancelled(&self) -> bool {
        *self.cancel.borrow()
    }
}

impl Engine {
    pub fn initial_sync_status(&self, account: &str) -> &'static str {
        match self
            .runtime(account)
            .map(|runtime| runtime.initial_sync.state.load(Ordering::Acquire))
        {
            Some(1) => "syncing",
            Some(2) => "retrying",
            Some(3) => "failed",
            _ => "ready",
        }
    }

    /// Start at most one indexing job for this exact runtime. Authentication
    /// failures retire its credential revision; they never enter retry backoff.
    pub fn start_initial_sync(self: &Arc<Self>, account: &str) {
        let Some(runtime) = self.runtime(account) else {
            return;
        };
        let control = runtime.initial_sync.clone();
        if control
            .state
            .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return;
        }
        let engine = self.clone();
        let account = account.to_string();
        tokio::spawn(async move {
            let mut cancelled = control.cancel.subscribe();
            if *cancelled.borrow() {
                return;
            }
            static JOBS: OnceLock<Semaphore> = OnceLock::new();
            let permit = tokio::select! { biased;
                _=cancelled.changed()=>return,
                permit=JOBS.get_or_init(||Semaphore::new(4)).acquire()=>permit,
            };
            let Ok(_permit) = permit else {
                return;
            };
            for attempt in 0..3u32 {
                if *cancelled.borrow() {
                    return;
                }
                if !engine
                    .runtime(&account)
                    .is_some_and(|current| Arc::ptr_eq(&current.backend, &runtime.backend))
                {
                    return;
                }
                control.state.store(1, Ordering::Release);
                let result = tokio::select! { biased;
                    _=cancelled.changed()=>return,
                    result=tokio::time::timeout(Duration::from_secs(1800),engine.resync_runtime(&account,&runtime))=>result.unwrap_or_else(|_|Err(EngineError::Transport("initial synchronization timed out".into()))),
                };
                match result {
                    Ok(()) => {
                        let watched = tokio::select! { biased;
                            _=cancelled.changed()=>return,
                            result=engine.start_watch(&account)=>result,
                        };
                        match watched {
                            Ok(()) | Err(EngineError::Unsupported(_)) => {
                                control.state.store(0, Ordering::Release);
                                engine.broadcast_state(&account).await;
                                return;
                            }
                            Err(EngineError::Auth(_)) => {
                                engine
                                    .reject_runtime_credentials(&account, &runtime.backend)
                                    .await;
                                return;
                            }
                            Err(error) => {
                                tracing::warn!(account=%account,error=%error,"initial sync watch could not start")
                            }
                        }
                    }
                    Err(EngineError::Auth(_)) => {
                        control.state.store(3, Ordering::Release);
                        engine
                            .reject_runtime_credentials(&account, &runtime.backend)
                            .await;
                        return;
                    }
                    Err(error) => {
                        tracing::warn!(account=%account,error=%error,"initial synchronization failed")
                    }
                }
                control
                    .state
                    .store(if attempt == 2 { 3 } else { 2 }, Ordering::Release);
                engine.broadcast_state(&account).await;
                if attempt == 2 {
                    return;
                }
                tokio::select! { biased;
                    _=cancelled.changed()=>return,
                    _=tokio::time::sleep(Duration::from_secs(5*(1u64<<attempt)))=>{},
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AccountRuntime, MailSubmitter, backend::*};
    use async_trait::async_trait;
    use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};
    use std::sync::atomic::{AtomicBool, AtomicUsize};

    struct SlowBackend {
        release: Semaphore,
        calls: AtomicUsize,
        reject: AtomicBool,
    }
    fn mailbox() -> RawMailbox {
        RawMailbox {
            mailbox_ref: RawMailboxRef {
                name: "INBOX".into(),
                uidvalidity: 1,
            },
            role: MailboxRole::Inbox,
            parent: None,
            uidnext: 1,
            highestmodseq: 0,
            total: 0,
            unread: 0,
        }
    }
    #[async_trait]
    impl AccountBackend for SlowBackend {
        async fn capabilities(&self) -> Result<BackendCaps> {
            Ok(BackendCaps::default())
        }
        async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
            Ok(vec![mailbox()])
        }
        async fn sync_mailbox(
            &self,
            _mailbox: &RawMailboxRef,
            cursor: &SyncCursor,
        ) -> Result<MailboxDelta> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.reject.load(Ordering::SeqCst) {
                return Err(EngineError::Auth("rejected".into()));
            }
            let permit = self.release.acquire().await.unwrap();
            permit.forget();
            Ok(MailboxDelta {
                added: vec![],
                flag_changes: vec![],
                removed: vec![],
                next_cursor: cursor.clone(),
            })
        }
        async fn fetch_raw(&self, _refs: &[MessageRef]) -> Result<Vec<RawMessage>> {
            Ok(vec![])
        }
        async fn store_flags(
            &self,
            _refs: &[MessageRef],
            _add: &[Flag],
            _remove: &[Flag],
        ) -> Result<()> {
            Ok(())
        }
        async fn move_messages(
            &self,
            _refs: &[MessageRef],
            _to: &RawMailboxRef,
        ) -> Result<MoveOutcome> {
            Err(EngineError::Unsupported("unused".into()))
        }
        async fn append(
            &self,
            _mailbox: &RawMailboxRef,
            _raw: &[u8],
            _flags: &[Flag],
        ) -> Result<MessageRef> {
            Err(EngineError::Unsupported("unused".into()))
        }
        async fn watch(&self, _sink: ChangeSink) -> Result<WatchHandle> {
            Err(EngineError::Unsupported("test backend".into()))
        }
    }
    struct NoSubmit;
    #[async_trait]
    impl MailSubmitter for NoSubmit {
        async fn submit(&self, _outgoing: mw_smtp::Outgoing) -> Result<mw_smtp::SubmissionResult> {
            Err(EngineError::Unsupported("unused".into()))
        }
    }
    async fn setup() -> (Arc<Engine>, String, Arc<SlowBackend>) {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let account = store
            .create_account(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host: "mail.example.org",
                    port: 993,
                    tls: "implicit",
                    username: "user@example.org",
                    sync_policy_json: "{}",
                },
                &Credentials {
                    username: "user@example.org".into(),
                    password: "test".into(),
                },
            )
            .await
            .unwrap();
        let engine = Arc::new(Engine::new(store));
        let backend = Arc::new(SlowBackend {
            release: Semaphore::new(0),
            calls: AtomicUsize::new(0),
            reject: AtomicBool::new(false),
        });
        let mut runtime =
            AccountRuntime::new(backend.clone(), Arc::new(NoSubmit), "user@example.org");
        runtime.credential_revision = Some(
            engine
                .store()
                .account_credential_revision(&account)
                .await
                .unwrap(),
        );
        engine.register_backend(account.clone(), runtime);
        (engine, account, backend)
    }
    async fn until(mut condition: impl FnMut() -> bool) {
        tokio::time::timeout(Duration::from_secs(3), async {
            while !condition() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
    }
    #[tokio::test]
    async fn metadata_ready_does_not_wait_for_message_indexing() {
        let (engine, account, backend) = setup().await;
        tokio::time::timeout(
            Duration::from_millis(300),
            engine.cache_mailbox_list(&account, vec![mailbox()]),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(
            engine.store().list_mailboxes(&account).await.unwrap().len(),
            1
        );
        engine.start_initial_sync(&account);
        until(|| backend.calls.load(Ordering::SeqCst) == 1).await;
        assert_eq!(engine.initial_sync_status(&account), "syncing");
        engine.start_initial_sync(&account);
        assert_eq!(backend.calls.load(Ordering::SeqCst), 1);
        backend.release.add_permits(1);
        until(|| engine.initial_sync_status(&account) == "ready").await;
        engine.unregister(&account);
    }
    #[tokio::test]
    async fn replacement_cancels_old_job_before_cursor_commit() {
        let (engine, account, backend) = setup().await;
        engine
            .cache_mailbox_list(&account, vec![mailbox()])
            .await
            .unwrap();
        let id = engine.store().list_mailboxes(&account).await.unwrap()[0]
            .id
            .clone();
        engine.start_initial_sync(&account);
        until(|| backend.calls.load(Ordering::SeqCst) == 1).await;
        engine.register_backend(
            account.clone(),
            AccountRuntime::new(backend.clone(), Arc::new(NoSubmit), "user@example.org"),
        );
        backend.release.add_permits(1);
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(
            engine
                .store()
                .load_cursor(&account, &id)
                .await
                .unwrap()
                .is_none()
        );
        assert_eq!(engine.initial_sync_status(&account), "ready");
        engine.unregister(&account);
    }
    #[tokio::test]
    async fn rejected_credentials_retire_the_runtime_without_retry() {
        let (engine, account, backend) = setup().await;
        backend.reject.store(true, Ordering::SeqCst);
        engine.start_initial_sync(&account);
        until(|| !engine.is_registered(&account)).await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(backend.calls.load(Ordering::SeqCst), 1);
        let policy: serde_json::Value = serde_json::from_str(
            &engine
                .store()
                .get_account(&account)
                .await
                .unwrap()
                .sync_policy_json,
        )
        .unwrap();
        assert_eq!(policy["credential_rejected"], true);
    }
}
