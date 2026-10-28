//! Runtime-owned initial indexing, separate from credential authentication.
use crate::{Engine, backend::EngineError};
use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, AtomicU8, Ordering},
};
use std::time::Duration;
use tokio::sync::{Notify, Semaphore, watch};

pub(crate) struct InitialSyncControl {
    state: AtomicU8,
    cancel: watch::Sender<bool>,
    /// Set while a job that used up its attempts waits for a user retry.
    parked: AtomicBool,
    retry: Notify,
}

impl Default for InitialSyncControl {
    fn default() -> Self {
        Self {
            state: AtomicU8::new(0),
            cancel: watch::channel(false).0,
            parked: AtomicBool::new(false),
            retry: Notify::new(),
        }
    }
}

/// Wait between attempts. Tests shrink it so the retry path runs in real time.
fn backoff(attempt: u32) -> Duration {
    if cfg!(test) {
        Duration::from_millis(10 << attempt)
    } else {
        Duration::from_secs(5 << attempt)
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

    /// Re-arm a job that failed all of its attempts. Returns `false` when there
    /// is nothing to retry: no runtime, a job still running, or one that ended
    /// for good (finished, cancelled, or retired over rejected credentials).
    pub fn retry_initial_sync(&self, account: &str) -> bool {
        let Some(runtime) = self.runtime(account) else {
            return false;
        };
        let control = &runtime.initial_sync;
        if !control.parked.swap(false, Ordering::AcqRel) {
            return false;
        }
        control.state.store(2, Ordering::Release);
        control.retry.notify_one();
        true
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
            loop {
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
                    if attempt == 2 {
                        break;
                    }
                    control.state.store(2, Ordering::Release);
                    engine.broadcast_state(&account).await;
                    tokio::select! { biased;
                        _=cancelled.changed()=>return,
                        _=tokio::time::sleep(backoff(attempt))=>{},
                    }
                }
                // Out of attempts: report the failure, give the slot back and
                // wait for the user to ask again (`retry_initial_sync`).
                drop(_permit);
                control.state.store(3, Ordering::Release);
                control.parked.store(true, Ordering::Release);
                engine.broadcast_state(&account).await;
                tokio::select! { biased;
                    _=cancelled.changed()=>return,
                    _=control.retry.notified()=>{},
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
        fail: AtomicBool,
        two_folders: AtomicBool,
    }
    fn mailbox() -> RawMailbox {
        named("INBOX", MailboxRole::Inbox)
    }
    fn named(name: &str, role: MailboxRole) -> RawMailbox {
        RawMailbox {
            mailbox_ref: RawMailboxRef {
                name: name.into(),
                uidvalidity: 1,
            },
            role,
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
            let mut boxes = vec![mailbox()];
            if self.two_folders.load(Ordering::SeqCst) {
                boxes.push(named("Archive", MailboxRole::Archive));
            }
            Ok(boxes)
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
            if self.fail.load(Ordering::SeqCst) {
                return Err(EngineError::Transport("connection reset".into()));
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
            fail: AtomicBool::new(false),
            two_folders: AtomicBool::new(false),
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

    /// `mailwomanInitialSync` plus each listed mailbox's `mailwomanSynced`,
    /// read the way the web client polls them.
    async fn progress(
        engine: &Engine,
        account: &str,
        retry: bool,
    ) -> (String, Vec<(String, bool)>) {
        let response = engine
            .handle_jmap(
                account,
                &serde_json::json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Mailbox/get",{"accountId":account,"properties":["id","name","mailwomanSynced"],"mailwomanRetryInitialSync":retry},"p"]]}),
            )
            .await;
        let result = &response["methodResponses"][0][1];
        let boxes = result["list"]
            .as_array()
            .unwrap_or_else(|| panic!("{response}"))
            .iter()
            .map(|entry| {
                (
                    entry["name"].as_str().unwrap_or_default().to_string(),
                    entry["mailwomanSynced"].as_bool().unwrap(),
                )
            })
            .collect();
        (
            result["mailwomanInitialSync"].as_str().unwrap().to_string(),
            boxes,
        )
    }
    #[tokio::test]
    async fn each_mailbox_reports_synced_once_its_messages_are_indexed() {
        let (engine, account, backend) = setup().await;
        backend.two_folders.store(true, Ordering::SeqCst);
        engine
            .cache_mailbox_list(
                &account,
                vec![mailbox(), named("Archive", MailboxRole::Archive)],
            )
            .await
            .unwrap();
        engine.start_initial_sync(&account);
        until(|| backend.calls.load(Ordering::SeqCst) == 1).await;
        let (status, boxes) = progress(&engine, &account, false).await;
        assert_eq!(status, "syncing");
        assert!(boxes.iter().all(|(_, synced)| !synced), "{boxes:?}");
        backend.release.add_permits(1);
        until(|| backend.calls.load(Ordering::SeqCst) == 2).await;
        let (status, boxes) = progress(&engine, &account, false).await;
        assert_eq!(status, "syncing");
        assert_eq!(
            boxes.iter().filter(|(_, synced)| *synced).count(),
            1,
            "{boxes:?}"
        );
        backend.release.add_permits(1);
        until(|| engine.initial_sync_status(&account) == "ready").await;
        let (_, boxes) = progress(&engine, &account, false).await;
        assert!(boxes.iter().all(|(_, synced)| *synced), "{boxes:?}");
        engine.unregister(&account);
    }
    #[tokio::test]
    async fn a_failed_job_waits_for_retry_and_then_finishes() {
        let (engine, account, backend) = setup().await;
        backend.fail.store(true, Ordering::SeqCst);
        engine
            .cache_mailbox_list(&account, vec![mailbox()])
            .await
            .unwrap();
        assert!(
            !engine.retry_initial_sync(&account),
            "nothing to retry before a failure"
        );
        engine.start_initial_sync(&account);
        until(|| engine.initial_sync_status(&account) == "failed").await;
        assert_eq!(backend.calls.load(Ordering::SeqCst), 3);
        let (_, boxes) = progress(&engine, &account, false).await;
        assert!(boxes.iter().all(|(_, synced)| !synced), "{boxes:?}");
        backend.fail.store(false, Ordering::SeqCst);
        backend.release.add_permits(1);
        let (status, _) = progress(&engine, &account, true).await;
        assert_ne!(
            status, "failed",
            "the retry is visible in the same response"
        );
        until(|| engine.initial_sync_status(&account) == "ready").await;
        assert_eq!(backend.calls.load(Ordering::SeqCst), 4);
        assert!(
            !engine.retry_initial_sync(&account),
            "a finished job is not re-run"
        );
        let (_, boxes) = progress(&engine, &account, false).await;
        assert!(boxes.iter().all(|(_, synced)| *synced), "{boxes:?}");
        engine.unregister(&account);
    }
}
