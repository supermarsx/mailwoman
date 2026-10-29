//! t39-e7 (L10): `Quota/get` takes the upstream server's quota from the
//! backend (`AccountBackend::quota`, IMAP `GETQUOTAROOT INBOX`), converts
//! RFC 9208 `STORAGE` (KiB) to octets, keeps the admin-configured row beside
//! it, and reports nothing when the server has no quota. Driven through
//! `Engine::handle_jmap` over a scripted backend.

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

use async_trait::async_trait;
use serde_json::{Value, json};

use mw_engine::account::AccountRuntime;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, EngineError, Flag, MailboxDelta, MessageRef,
    MoveOutcome, QuotaUsage, RawMailbox, RawMailboxRef, RawMessage, Result, SyncCursor,
    WatchHandle,
};
use mw_engine::{Engine, MailSubmitter};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{AccountKind, Credentials, NewAccount, QuotaRow, ServerKey, Store};

/// What the scripted server answers to the quota read.
#[derive(Clone)]
enum Answer {
    /// The server does not advertise QUOTA (the trait default).
    NoQuotaCapability,
    /// GETQUOTAROOT answered with these resources (possibly none).
    Quota(Vec<QuotaUsage>),
    /// The command failed on the wire.
    Broken,
}

struct QuotaBackend {
    answer: Mutex<Answer>,
    calls: AtomicU32,
}

#[async_trait]
impl AccountBackend for QuotaBackend {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps::default())
    }
    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        Ok(Vec::new())
    }
    async fn sync_mailbox(&self, _m: &RawMailboxRef, _c: &SyncCursor) -> Result<MailboxDelta> {
        Err(EngineError::Unsupported("sync".into()))
    }
    async fn fetch_raw(&self, _r: &[MessageRef]) -> Result<Vec<RawMessage>> {
        Ok(Vec::new())
    }
    async fn store_flags(&self, _r: &[MessageRef], _a: &[Flag], _rm: &[Flag]) -> Result<()> {
        Ok(())
    }
    async fn move_messages(&self, _r: &[MessageRef], _to: &RawMailboxRef) -> Result<MoveOutcome> {
        Err(EngineError::Unsupported("move".into()))
    }
    async fn append(&self, _m: &RawMailboxRef, _raw: &[u8], _f: &[Flag]) -> Result<MessageRef> {
        Err(EngineError::Unsupported("append".into()))
    }
    async fn watch(&self, _sink: ChangeSink) -> Result<WatchHandle> {
        Err(EngineError::Unsupported("watch".into()))
    }
    async fn quota(&self) -> Result<Vec<QuotaUsage>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        match self.answer.lock().unwrap().clone() {
            Answer::NoQuotaCapability => Err(EngineError::Unsupported("quota".into())),
            Answer::Quota(q) => Ok(q),
            Answer::Broken => Err(EngineError::Protocol("BAD unknown command".into())),
        }
    }
}

struct NoSubmit;

#[async_trait]
impl MailSubmitter for NoSubmit {
    async fn submit(&self, msg: Outgoing) -> Result<SubmissionResult> {
        Ok(SubmissionResult {
            accepted: msg.rcpt_to,
            rejected: Vec::new(),
        })
    }
}

async fn setup(answer: Answer) -> (Arc<Engine>, String, Arc<QuotaBackend>) {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account_id = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example.org",
                port: 993,
                tls: "implicit",
                username: "me@example.org",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "me@example.org".into(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap();
    let backend = Arc::new(QuotaBackend {
        answer: Mutex::new(answer),
        calls: AtomicU32::new(0),
    });
    let engine = Arc::new(Engine::new(store));
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            backend.clone() as Arc<dyn AccountBackend>,
            Arc::new(NoSubmit) as Arc<dyn MailSubmitter>,
            "me@example.org",
        ),
    );
    (engine, account_id, backend)
}

async fn quota_get(engine: &Engine, account_id: &str, args: Value) -> Value {
    let req = json!({
        "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:quota"],
        "methodCalls": [["Quota/get", args, "q"]]
    });
    let resp = engine.handle_jmap(account_id, &req).await;
    assert_eq!(resp["methodResponses"][0][0], "Quota/get", "{resp}");
    resp["methodResponses"][0][1].clone()
}

fn usage(resource: &str, usage: u64, limit: u64) -> QuotaUsage {
    QuotaUsage {
        root: "User quota".into(),
        resource: resource.into(),
        usage,
        limit,
    }
}

#[tokio::test]
async fn upstream_quota_is_reported_in_octets_and_read_once_per_minute() {
    let (engine, account, backend) = setup(Answer::Quota(vec![
        usage("STORAGE", 512, 1024),
        usage("MESSAGE", 7, 1000),
    ]))
    .await;
    let resp = quota_get(&engine, &account, json!({ "accountId": account })).await;
    let list = resp["list"].as_array().unwrap();
    assert_eq!(list.len(), 2, "{resp}");
    let storage = list.iter().find(|q| q["id"] == "upstream-storage").unwrap();
    assert_eq!(storage["resourceType"], "octets");
    assert_eq!(storage["used"], 512 * 1024);
    assert_eq!(storage["hardLimit"], 1024 * 1024);
    assert_eq!(storage["name"], "User quota");
    let count = list.iter().find(|q| q["id"] == "upstream-count").unwrap();
    assert_eq!(count["used"], 7);
    assert_eq!(count["hardLimit"], 1000);

    // A second call within the minute reuses the reading: the shared IMAP
    // session is not asked again.
    *backend.answer.lock().unwrap() = Answer::Quota(vec![usage("STORAGE", 1, 2)]);
    let again = quota_get(&engine, &account, json!({ "accountId": account })).await;
    assert_eq!(again["list"], resp["list"]);
    assert_eq!(backend.calls.load(Ordering::SeqCst), 1);

    // `ids` narrows the result and names unknown ids.
    let some = quota_get(
        &engine,
        &account,
        json!({ "accountId": account, "ids": ["upstream-count", "nope"] }),
    )
    .await;
    assert_eq!(some["list"].as_array().unwrap().len(), 1);
    assert_eq!(some["notFound"], json!(["nope"]));
}

#[tokio::test]
async fn a_server_without_quota_reports_nothing() {
    for answer in [
        Answer::NoQuotaCapability,
        Answer::Quota(Vec::new()),
        Answer::Broken,
    ] {
        let (engine, account, _) = setup(answer).await;
        let resp = quota_get(&engine, &account, json!({ "accountId": account })).await;
        assert_eq!(resp["list"], json!([]), "{resp}");
        assert_eq!(resp["notFound"], json!([]));
    }
}

#[tokio::test]
async fn admin_limits_and_upstream_quota_are_both_reported() {
    let (engine, account, _) = setup(Answer::Quota(vec![usage("STORAGE", 10, 20)])).await;
    engine
        .store()
        .set_quota(
            &account,
            QuotaRow {
                bytes_limit: 5_000,
                msg_limit: 50,
            },
        )
        .await
        .unwrap();
    let resp = quota_get(&engine, &account, json!({ "accountId": account })).await;
    let ids: Vec<&str> = resp["list"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|q| q["id"].as_str())
        .collect();
    assert_eq!(ids, ["upstream-storage", "mail-storage", "mail-count"]);
}
