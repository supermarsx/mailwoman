//! Actual queued send / SMTP-spy behavior for local inspection backpressure.
use super::*;
use crate::account::MailSubmitter;
use crate::backend::*;
use crate::security::{cpu_work::CpuExecutor, dlp};
use async_trait::async_trait;
use mw_store::{
    AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc,
};
use std::time::Duration;
#[derive(Default)]
struct Remote;
#[async_trait]
impl AccountBackend for Remote {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps::default())
    }
    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        Ok(vec![])
    }
    async fn sync_mailbox(&self, _: &RawMailboxRef, c: &SyncCursor) -> Result<MailboxDelta> {
        Ok(MailboxDelta {
            added: vec![],
            flag_changes: vec![],
            removed: vec![],
            next_cursor: c.clone(),
        })
    }
    async fn fetch_raw(&self, _: &[MessageRef]) -> Result<Vec<RawMessage>> {
        Err(EngineError::Unsupported(
            "fixture has only authoritative cached raw".into(),
        ))
    }
    async fn store_flags(&self, _: &[MessageRef], _: &[Flag], _: &[Flag]) -> Result<()> {
        Ok(())
    }
    async fn move_messages(&self, _: &[MessageRef], _: &RawMailboxRef) -> Result<MoveOutcome> {
        Err(EngineError::Unsupported("fixture".into()))
    }
    async fn append(&self, _: &RawMailboxRef, _: &[u8], _: &[Flag]) -> Result<MessageRef> {
        Err(EngineError::Unsupported(
            "fixture does not file sent copies".into(),
        ))
    }
    async fn watch(&self, _: ChangeSink) -> Result<WatchHandle> {
        Err(EngineError::Unsupported("fixture".into()))
    }
}
#[derive(Default)]
struct SubmitSpy {
    calls: AtomicUsize,
    uncertain: AtomicBool,
}
#[async_trait]
impl MailSubmitter for SubmitSpy {
    async fn submit(&self, value: mw_smtp::Outgoing) -> Result<mw_smtp::SubmissionResult> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.uncertain.load(Ordering::SeqCst) {
            return Err(EngineError::SubmissionUncertain(
                "fixture acknowledgement lost".into(),
            ));
        }
        Ok(mw_smtp::SubmissionResult {
            accepted: value.rcpt_to,
            rejected: vec![],
        })
    }
}
struct Fixture {
    engine: Engine,
    account: String,
    id: String,
    runtime: AccountRuntime,
    spy: Arc<SubmitSpy>,
}
async fn fixture(receipt: bool) -> Fixture {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "fixture.invalid",
                port: 993,
                tls: "implicit",
                username: "owner@fixture.invalid",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "owner@fixture.invalid".into(),
                password: "fixture-only".into(),
            },
        )
        .await
        .unwrap();
    let mailbox = store
        .upsert_mailbox(&MailboxUpsert {
            account_id: &account,
            name: if receipt { "INBOX" } else { "Drafts" },
            role: Some(if receipt { "inbox" } else { "drafts" }),
            uidvalidity: 1,
            uidnext: 2,
            highestmodseq: 0,
            total: 0,
            unread: 0,
            parent_id: None,
        })
        .await
        .unwrap();
    let raw = if receipt {
        b"From: sender@example.invalid\r\nTo: owner@fixture.invalid\r\nMessage-ID: <receipt@fixture.invalid>\r\nDisposition-Notification-To: sender@example.invalid\r\nSubject: Receipt\r\nContent-Type: text/plain\r\n\r\nHello\r\n".as_slice()
    } else {
        b"From: owner@fixture.invalid\r\nTo: recipient@example.invalid\r\nMessage-ID: <draft@fixture.invalid>\r\nSubject: Draft\r\nContent-Type: text/plain\r\n\r\n4111 1111 1111 1111\r\n".as_slice()
    };
    let blob = store.put_body(&account, raw).await.unwrap();
    let parsed = mw_mime::parse(raw).unwrap();
    let envelope = crate::encode_message_envelope(&parsed.email, raw);
    let id = store
        .upsert_message(&MessageUpsert {
            account_id: &account,
            mailbox_id: &mailbox,
            uid: 1,
            uidvalidity: 1,
            message_id: parsed
                .email
                .message_id
                .as_ref()
                .and_then(|values| values.first())
                .map(String::as_str),
            thread_id: None,
            internaldate: Some("2026-11-20T00:00:00Z"),
            size: raw.len() as u64,
            flags_json: "[]",
            envelope: Some(&envelope),
            blob_ref: Some(&blob),
        })
        .await
        .unwrap();
    let engine = Engine::new(store);
    let spy = Arc::new(SubmitSpy::default());
    let runtime = AccountRuntime::new(Arc::new(Remote), spy.clone(), "owner@fixture.invalid");
    engine.register_backend(account.clone(), runtime.clone());
    Fixture {
        engine,
        account,
        id,
        runtime,
        spy,
    }
}
async fn held(f: &Fixture) -> String {
    f.engine
        .create_submission(
            &f.account,
            &f.runtime,
            &f.id,
            &json!({"mailwomanHold":"manual"}),
            None,
            None,
        )
        .await
        .unwrap()
        .id
}
struct Saturation {
    guard: Option<dlp::ExecutorOverride>,
    release: Option<mpsc::Sender<()>>,
    task: Option<tokio::task::JoinHandle<()>>,
}
impl Drop for Saturation {
    fn drop(&mut self) {
        if let Some(release) = self.release.take() {
            let _ = release.send(());
        }
    }
}
impl Saturation {
    async fn finish(mut self) {
        self.release.take().unwrap().send(()).unwrap();
        self.task.take().unwrap().await.unwrap();
        self.guard.take();
    }
}
async fn saturate(f: &Fixture) -> Saturation {
    let pool = CpuExecutor::new(1, 1, 256 * 1024 * 1024, Duration::from_secs(10)).unwrap();
    let guard = dlp::override_executor(f.engine.store(), pool.clone());
    let (entered_tx, entered_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let task = tokio::spawn(async move {
        pool.run(1, move || {
            let _ = entered_tx.send(());
            release_rx.recv().unwrap();
        })
        .await
        .unwrap();
    });
    entered_rx.await.unwrap();
    Saturation {
        guard: Some(guard),
        release: Some(release_tx),
        task: Some(task),
    }
}
#[tokio::test]
async fn queued_send_backs_off_without_smtp_and_delivers_once_after_cpu_capacity_returns() {
    let f = fixture(false).await;
    let id = held(&f).await;
    let saturation = saturate(&f).await;
    f.engine
        .release_submission(&f.account, &id, None)
        .await
        .unwrap();
    assert_eq!(f.spy.calls.load(Ordering::SeqCst), 0);
    let row = f.engine.store().get_submission(&id).await.unwrap().unwrap();
    assert_eq!(row.undo_status, SUBMISSION_PENDING);
    let attempt = f
        .engine
        .store()
        .get_submission_attempts(&id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(attempt.attempts, 1);
    let next = chrono::DateTime::parse_from_rfc3339(
        attempt
            .next_attempt_at
            .as_deref()
            .expect("busy CPU must preserve a retry"),
    )
    .unwrap()
    .with_timezone(&chrono::Utc);
    assert!(
        attempt
            .last_error
            .as_deref()
            .unwrap()
            .contains("inspection")
    );
    saturation.finish().await;
    f.engine
        .attempt_submission(&id, next + chrono::Duration::seconds(1))
        .await
        .unwrap();
    assert_eq!(f.spy.calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        f.engine
            .store()
            .get_submission(&id)
            .await
            .unwrap()
            .unwrap()
            .undo_status,
        "final"
    );
    f.engine
        .attempt_submission(&id, next + chrono::Duration::hours(1))
        .await
        .unwrap();
    assert_eq!(f.spy.calls.load(Ordering::SeqCst), 1);
}
#[tokio::test]
async fn active_dlp_block_is_permanent_and_never_dispatches_or_schedules_retry() {
    let f = fixture(false).await;
    let id = held(&f).await;
    let rule = json!({"id":"block-pan","name":"PAN","enabled":true,"priority":1,"conditions":{"detectors":["pan"],"customRegex":null,"dictionaries":[],"attachmentTypes":[],"maxAttachmentSize":null,"recipientDomains":[],"recipientDomainMode":null,"classification":null},"action":"block","message":"Restricted"});
    f.engine
        .store()
        .set_setting(
            dlp::SECURITY_POLICY_SETTINGS_KEY,
            &json!({"dlp_rules_json":json!([rule]).to_string()}).to_string(),
        )
        .await
        .unwrap();
    f.engine
        .release_submission(&f.account, &id, None)
        .await
        .unwrap();
    assert_eq!(f.spy.calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        f.engine
            .store()
            .get_submission(&id)
            .await
            .unwrap()
            .unwrap()
            .undo_status,
        SUBMISSION_FAILED
    );
    assert!(
        f.engine
            .store()
            .get_submission_attempts(&id)
            .await
            .unwrap()
            .unwrap()
            .next_attempt_at
            .is_none()
    );
    f.engine
        .attempt_submission(&id, chrono::Utc::now() + chrono::Duration::days(1))
        .await
        .unwrap();
    assert_eq!(f.spy.calls.load(Ordering::SeqCst), 0);
}
#[tokio::test]
async fn uncertain_smtp_outcome_is_never_reclassified_as_pre_dispatch_busy() {
    let f = fixture(false).await;
    let id = held(&f).await;
    f.spy.uncertain.store(true, Ordering::SeqCst);
    f.engine
        .release_submission(&f.account, &id, None)
        .await
        .unwrap();
    assert_eq!(f.spy.calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        f.engine
            .store()
            .get_submission(&id)
            .await
            .unwrap()
            .unwrap()
            .undo_status,
        "uncertain"
    );
    assert!(
        f.engine
            .store()
            .get_submission_attempts(&id)
            .await
            .unwrap()
            .unwrap()
            .next_attempt_at
            .is_none()
    );
    f.engine
        .attempt_submission(&id, chrono::Utc::now() + chrono::Duration::days(1))
        .await
        .unwrap();
    assert_eq!(f.spy.calls.load(Ordering::SeqCst), 1);
}
#[tokio::test]
async fn scheduled_receipt_cpu_busy_releases_claim_and_retains_job_for_retry() {
    let f = fixture(true).await;
    let now = chrono::Utc::now();
    f.engine
        .store()
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: "receipts.scheduleAllowed".into(),
            global_value: Some("true".into()),
            global_locked: false,
            allow_user_override: false,
            updated_by: "fixture".into(),
            updated_at: now.timestamp(),
        })
        .await
        .unwrap();
    let due = now + chrono::Duration::seconds(60);
    assert!(
        f.engine
            .schedule_read_receipt(&f.account, &f.id, "message", Some(due.timestamp()))
            .await
            .unwrap()
    );
    let saturation = saturate(&f).await;
    f.engine.dispatch_scheduled_mdn(due).await.unwrap();
    assert_eq!(f.spy.calls.load(Ordering::SeqCst), 0);
    assert!(
        f.engine
            .store()
            .scheduled_mdn_time(&f.account, &f.id)
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        f.engine
            .store()
            .due_scheduled_mdn(due.timestamp() + 1)
            .await
            .unwrap()
            .is_empty()
    );
    let message = f.engine.store().get_message(&f.id).await.unwrap();
    assert!(
        !f.engine
            .mdn_sent(&f.account, &f.id, &message.flags_json)
            .await
            .unwrap(),
        "a known pre-SMTP failure must not leave suppression markers"
    );
    saturation.finish().await;
    f.engine
        .dispatch_scheduled_mdn(due + chrono::Duration::seconds(301))
        .await
        .unwrap();
    assert_eq!(f.spy.calls.load(Ordering::SeqCst), 1);
    assert!(
        f.engine
            .store()
            .scheduled_mdn_time(&f.account, &f.id)
            .await
            .unwrap()
            .is_none()
    );
}
