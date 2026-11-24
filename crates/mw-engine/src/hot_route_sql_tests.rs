//! Hot JMAP read routes as whole requests: statement counts that must not
//! grow with the page size, plus correctness of the shared reads (t34-e12, D3).
//!
//! Every count is taken through [`Engine::handle_jmap`], the entry point the
//! server uses, with the shared sqlx-level recorder of
//! [`crate::state::session_state_tests`] (one event per executed statement),
//! on a Store-backed posture source like the server's `StorePostureSource`
//! and with the header cache attached (as in the server). Each route is sent
//! three times and the third ("warm2", as in e0's router harness) is counted.
//!
//! Correctness is checked against an **oracle on the same data**: the same
//! engine with a posture source that is not Store-backed (so every posture
//! answer is a separate live read, as before t34-e12) and each method sent as
//! its own request (so every state token is its own read, as before).

use std::collections::HashSet;
use std::sync::Arc;
use std::thread::ThreadId;

use mw_store::{
    AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store,
    ZeroAccessRow,
};
use serde_json::{Value, json};

use crate::backend::AccountBackend;
use crate::change::{ChangeOp, ChangeType};
use crate::engine::Engine;
use crate::state::session_state_tests::{Stmts, counted, db_threads};

// ── posture sources ─────────────────────────────────────────────────────────

/// What the server attaches (`mw-server` `StorePostureSource`): every live
/// answer is a fresh `zeroaccess_accounts` read of this Store, fail closed,
/// and the source declares itself Store-backed.
struct StoreLive(Store);
/// The same live reads, but not declared Store-backed: the engine must then
/// keep every live posture read. This is the pre-t34-e12 behaviour and the
/// oracle for responses.
struct OpaqueLive(Store);

async fn live(store: &Store, account: &str) -> crate::v6::AccountPosture {
    match store.get_zeroaccess(account).await {
        Ok(row) if row.as_ref().is_some_and(|r| r.enabled) => crate::v6::AccountPosture::ZeroAccess,
        Ok(_) => crate::v6::AccountPosture::Standard,
        Err(_) => crate::v6::AccountPosture::ZeroAccess,
    }
}

impl crate::v6::AccountPostureSource for StoreLive {
    fn prepared_mail_posture(&self) -> crate::v6::PreparedMailPosture {
        crate::v6::PreparedMailPosture::Store(*self.0.cache_scope().as_bytes())
    }
    fn posture(&self, _: &str) -> crate::v6::AccountPosture {
        crate::v6::AccountPosture::Standard
    }
    fn posture_live<'a>(
        &'a self,
        account: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::v6::AccountPosture> + Send + 'a>>
    {
        Box::pin(live(&self.0, account))
    }
}

impl crate::v6::AccountPostureSource for OpaqueLive {
    fn posture(&self, _: &str) -> crate::v6::AccountPosture {
        crate::v6::AccountPosture::Standard
    }
    fn posture_live<'a>(
        &'a self,
        account: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::v6::AccountPosture> + Send + 'a>>
    {
        Box::pin(live(&self.0, account))
    }
}

// ── a backend that only accepts flag writes ─────────────────────────────────

/// Every read route is served from the store, so any backend call other than
/// the flag write the in-batch mutation test makes is a failure.
struct FlagsOnlyBackend;

#[async_trait::async_trait]
impl AccountBackend for FlagsOnlyBackend {
    async fn capabilities(&self) -> crate::backend::Result<crate::backend::BackendCaps> {
        unreachable!("hot read routes must not ask the backend for capabilities")
    }
    async fn list_mailboxes(&self) -> crate::backend::Result<Vec<crate::backend::RawMailbox>> {
        unreachable!("hot read routes are served from the store")
    }
    async fn sync_mailbox(
        &self,
        _: &crate::backend::RawMailboxRef,
        _: &crate::backend::SyncCursor,
    ) -> crate::backend::Result<crate::backend::MailboxDelta> {
        unreachable!("hot read routes must not sync")
    }
    async fn fetch_raw(
        &self,
        _: &[crate::backend::MessageRef],
    ) -> crate::backend::Result<Vec<crate::backend::RawMessage>> {
        unreachable!("bodies are stored")
    }
    async fn store_flags(
        &self,
        _: &[crate::backend::MessageRef],
        _: &[crate::backend::Flag],
        _: &[crate::backend::Flag],
    ) -> crate::backend::Result<()> {
        Ok(())
    }
    async fn move_messages(
        &self,
        _: &[crate::backend::MessageRef],
        _: &crate::backend::RawMailboxRef,
    ) -> crate::backend::Result<crate::backend::MoveOutcome> {
        unreachable!("no moves")
    }
    async fn append(
        &self,
        _: &crate::backend::RawMailboxRef,
        _: &[u8],
        _: &[crate::backend::Flag],
    ) -> crate::backend::Result<crate::backend::MessageRef> {
        unreachable!("no appends")
    }
    async fn watch(
        &self,
        _: crate::backend::ChangeSink,
    ) -> crate::backend::Result<crate::backend::WatchHandle> {
        unreachable!("no watch")
    }
}

struct NoSubmit;

#[async_trait::async_trait]
impl crate::MailSubmitter for NoSubmit {
    async fn submit(
        &self,
        _: mw_smtp::Outgoing,
    ) -> crate::backend::Result<mw_smtp::SubmissionResult> {
        unreachable!("no submission")
    }
}

// ── fixture ─────────────────────────────────────────────────────────────────

/// The header properties the web client sends (checked-in source of truth).
fn properties(name: &str) -> Vec<String> {
    let source = include_str!("../../../scripts/perf/fixture-properties.mjs");
    let prefix = format!("export const {name}=Object.freeze(");
    let body = source
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .and_then(|line| line.strip_suffix(");"))
        .expect("known property declaration");
    let body = body.strip_prefix('[').and_then(|v| v.strip_suffix(']')).expect("array");
    if !body.contains("...") {
        return serde_json::from_str(&format!("[{body}]").replace('\'', "\"")).expect("literal");
    }
    body.split(',')
        .flat_map(|item| properties(item.trim().strip_prefix("...").expect("spread")))
        .collect()
}

/// Messages per account; pages of 5 and 50 are both strictly inside it.
const SEEDED: usize = 110;

struct Hot {
    engine: Engine,
    threads: HashSet<ThreadId>,
    /// The measured account and a second account with the same shape.
    a: String,
    b: String,
    /// Inbox ids, newest first (the `Email/query` order).
    a_ids: Vec<String>,
    b_ids: Vec<String>,
    cache: mw_cache::Cache,
}

async fn seed_account(store: &Store, name: &str) -> (String, Vec<String>) {
    // Unique login per fixture: PostgreSQL runs share one database.
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let login = format!(
        "{name}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() * 1000
    );
    let account = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "hot.invalid",
                port: 993,
                tls: "implicit",
                username: &login,
                sync_policy_json: "{}",
            },
            &Credentials { username: login.clone(), password: "p".into() },
        )
        .await
        .unwrap();
    let mut boxes = Vec::new();
    for (mailbox, role) in [("INBOX", "inbox"), ("Sent", "sent"), ("Drafts", "drafts")] {
        boxes.push(
            store
                .upsert_mailbox(&MailboxUpsert {
                    account_id: &account,
                    name: mailbox,
                    role: Some(role),
                    uidvalidity: 100,
                    uidnext: 1,
                    highestmodseq: 0,
                    total: 0,
                    unread: 0,
                    parent_id: None,
                })
                .await
                .unwrap(),
        );
    }
    let mut ids = Vec::new();
    let put = async |mailbox: &str, uid: u32, from: &str| {
        let raw = format!(
            "From: {from}\r\nTo: owner@hot.invalid\r\nSubject: {name} {uid}\r\nMessage-ID: <{uid}.{name}@hot.invalid>\r\nDate: Sun, 01 Nov 2026 00:00:00 +0000\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nBody {uid}.\r\n"
        );
        let parsed = mw_mime::parse(raw.as_bytes()).unwrap();
        let envelope = crate::encode_message_envelope(&parsed.email, raw.as_bytes());
        let blob = store.put_body(&account, raw.as_bytes()).await.unwrap();
        let date = format!("2026-11-01T{:02}:{:02}:00Z", uid / 60, uid % 60);
        let message_id = format!("<{uid}.{name}@hot.invalid>");
        store
            .upsert_message(&MessageUpsert {
                account_id: &account,
                mailbox_id: mailbox,
                uid,
                uidvalidity: 100,
                message_id: Some(&message_id),
                thread_id: None,
                internaldate: Some(&date),
                size: raw.len() as u64,
                flags_json: "[]",
                envelope: Some(&envelope),
                blob_ref: Some(&blob),
            })
            .await
            .unwrap()
    };
    for uid in 1..=SEEDED as u32 {
        ids.push(put(&boxes[0], uid, "Sender <sender@hot.invalid>").await);
    }
    for uid in 1..=2 {
        put(&boxes[1], 1000 + uid, "owner@hot.invalid").await;
    }
    // A change-log history so state tokens are not all zero.
    store.record_change(&account, "Email", &ids[0], "created").await.unwrap();
    store.record_change(&account, "Mailbox", &boxes[0], "updated").await.unwrap();
    ids.reverse();
    (account, ids)
}

impl Hot {
    async fn open(store: Store) -> Self {
        let threads = db_threads(&store).await;
        let (a, a_ids) = seed_account(&store, "hot-a").await;
        let (b, b_ids) = seed_account(&store, "hot-b").await;
        // One cached verdict per account, so `mailwomanAuth` has something to
        // show (and to hide under zero-access).
        for (account, id) in [(&a, &a_ids[0]), (&b, &b_ids[0])] {
            let (source, raw) = store.security_verdict_source(account, id).await.unwrap().unwrap();
            let verdict = crate::security::verdict::compute_verdict(id, &raw, None, None, false).await;
            assert!(
                store
                    .security_verdict_cache_if_current(&source, "fixture", &serde_json::to_vec(&verdict).unwrap())
                    .await
                    .unwrap()
            );
        }
        let engine = Engine::new(store);
        for account in [&a, &b] {
            engine.register_backend(
                account.clone(),
                crate::account::AccountRuntime::new(
                    Arc::new(FlagsOnlyBackend) as Arc<dyn AccountBackend>,
                    Arc::new(NoSubmit) as Arc<dyn crate::MailSubmitter>,
                    "owner@hot.invalid",
                ),
            );
        }
        let cache = mw_cache::Cache::default();
        let hot = Self { engine, threads, a, b, a_ids, b_ids, cache };
        hot.attach(true);
        hot
    }

    async fn sqlite() -> Self {
        Self::open(Store::open_in_memory(ServerKey::generate()).await.unwrap()).await
    }

    async fn pg() -> Option<Self> {
        let dsn = mw_test_gate::pg_dsn()?;
        Some(Self::open(Store::open(&dsn, ServerKey::generate()).await.unwrap()).await)
    }

    fn store(&self) -> &Store {
        self.engine.store()
    }

    /// `store_backed`: the server's posture source; otherwise the oracle's.
    fn attach(&self, store_backed: bool) {
        let source: Arc<dyn crate::v6::AccountPostureSource> = if store_backed {
            Arc::new(StoreLive(self.store().clone()))
        } else {
            Arc::new(OpaqueLive(self.store().clone()))
        };
        self.engine.attach_v6(
            crate::v6::V6Hooks::new().with_cache(self.cache.clone()).with_posture_source(source),
        );
    }

    async fn send(&self, account: &str, calls: Vec<Value>) -> Value {
        self.engine
            .handle_jmap(
                account,
                &json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":calls}),
            )
            .await
    }

    /// Three identical requests; the third is counted (warm2).
    async fn warm(&self, account: &str, calls: Vec<Value>) -> (Value, Stmts) {
        for _ in 0..2 {
            self.send(account, calls.clone()).await;
        }
        counted(&self.threads, self.send(account, calls)).await
    }

    async fn set_zero_access(&self, account: &str, enabled: bool) {
        self.store()
            .upsert_zeroaccess(&ZeroAccessRow {
                account_id: account.into(),
                enabled,
                wrapped_root_key: vec![0; 16],
                kdf_params_json: "{}".into(),
                recovery_wrapped: None,
                paired_devices_json: "[]".into(),
            })
            .await
            .unwrap();
    }
}

fn header_props() -> Value {
    json!(properties("DEFAULT_HEADER_PROPERTIES"))
}

/// The hot routes at one page size, as the router harness sends them.
fn routes(account: &str, ids: &[String], page: usize) -> Vec<(&'static str, Vec<Value>)> {
    let get_by_ref = |id: &str, query: &str| {
        json!(["Email/get", {"accountId":account,"#ids":{"resultOf":query,"name":"Email/query","path":"/ids"},"properties":header_props()}, id])
    };
    vec![
        ("mailbox-get", vec![json!(["Mailbox/get", {"accountId":account}, "m"])]),
        ("email-query", vec![json!(["Email/query", {"accountId":account,"limit":page}, "q"])]),
        (
            "email-get",
            vec![json!(["Email/get", {"accountId":account,"ids":ids[..page],"properties":header_props()}, "g"])],
        ),
        (
            "batch-2",
            vec![
                json!(["Email/query", {"accountId":account,"limit":page,"position":0,"calculateTotal":true}, "page1"]),
                get_by_ref("headers1", "page1"),
            ],
        ),
        (
            "batch-4",
            vec![
                json!(["Email/query", {"accountId":account,"limit":page,"position":0,"calculateTotal":true}, "page1"]),
                get_by_ref("headers1", "page1"),
                json!(["Email/query", {"accountId":account,"limit":page,"position":page,"calculateTotal":true}, "page2"]),
                get_by_ref("headers2", "page2"),
            ],
        ),
    ]
}

/// Every method response of a request must be a success (no `error`), and
/// every `Email/get` must have answered the whole page.
fn assert_answered(what: &str, response: &Value, page: usize) {
    for method in response["methodResponses"].as_array().unwrap() {
        assert_ne!(method[0], "error", "{what}: {response}");
        if method[0] == "Email/get" {
            assert_eq!(method[1]["list"].as_array().unwrap().len(), page, "{what}: {method}");
        }
    }
}

/// Statement families of one measurement, for failure messages and the log.
fn families(stmts: &Stmts) -> Vec<String> {
    stmts
        .0
        .iter()
        .map(|s| s.sql.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(140).collect())
        .collect()
}

/// The warm per-route counts at one page size.
async fn measure(hot: &Hot, page: usize) -> Vec<(&'static str, usize, Stmts)> {
    let mut out = Vec::new();
    for (name, calls) in routes(&hot.a, &hot.a_ids, page) {
        let (response, stmts) = hot.warm(&hot.a, calls).await;
        assert_answered(name, &response, page);
        out.push((name, stmts.len(), stmts));
    }
    out
}

/// Warm statements per route, engine level, measured with this fixture:
/// before t34-e12 (HEAD e7a92203) 8 / 3 / 11 / 14 / 27, after 4 / 2 / 5 / 7 /
/// 13. Exact, so a reduction or a regression both show up here first.
const BUDGET: [(&str, usize); 5] = [
    // account+mailboxes, settings revision, owner+saved searches, state frame
    ("mailbox-get", 4),
    // id page, state frame
    ("email-query", 2),
    // rows+meta+posture, Focus sent scan, Focus contacts, verdicts, state frame
    ("email-get", 5),
    // query page + total, get (4), one shared state frame
    ("batch-2", 7),
    ("batch-4", 13),
];

async fn assert_page_independent_budgets(hot_small: &Hot, hot_large: &Hot, backend: &str) {
    let small = measure(hot_small, 5).await;
    let large = measure(hot_large, 50).await;
    for ((name, s, s_stmts), (_, l, l_stmts)) in small.iter().zip(&large) {
        eprintln!("t34-e12 {backend} {name} page5={s} page50={l}");
        for f in families(s_stmts) {
            eprintln!("t34-e12 {backend} {name}   {f}");
        }
        assert_eq!(
            s, l,
            "{backend} {name}: statement count grows with page size (5 -> 50): {:#?} vs {:#?}",
            families(s_stmts),
            families(l_stmts)
        );
        let budget = BUDGET.iter().find(|(n, _)| n == name).unwrap().1;
        assert_eq!(*l, budget, "{backend} {name}: measured {l}, expected {budget}: {:#?}", families(l_stmts));
    }
}

#[tokio::test]
async fn hot_routes_cost_the_same_statements_at_five_and_fifty_items_sqlite() {
    let small = Hot::sqlite().await;
    let large = Hot::sqlite().await;
    assert_page_independent_budgets(&small, &large, "sqlite").await;
}

#[tokio::test]
async fn hot_routes_cost_the_same_statements_at_five_and_fifty_items_postgres() {
    let (Some(small), Some(large)) = (Hot::pg().await, Hot::pg().await) else {
        mw_test_gate::skip("DATABASE_URL_PG and MW_E14_PG_DSN unset: hot route statement budgets on PostgreSQL");
        return;
    };
    assert_page_independent_budgets(&small, &large, "postgres").await;
}

// ── correctness ─────────────────────────────────────────────────────────────

/// Responses carry no volatile field except `sessionState`, which is compared
/// separately; `Mailbox/get`'s initial-sync status is stable in this fixture.
fn method_responses(response: &Value) -> Vec<Value> {
    response["methodResponses"].as_array().unwrap().clone()
}

/// The same calls through the oracle: opaque posture source and one request
/// per method (references resolved from the oracle's own earlier answers).
async fn oracle(hot: &Hot, account: &str, calls: &[Value]) -> (Vec<Value>, String) {
    hot.attach(false);
    let mut answers: Vec<Value> = Vec::new();
    for call in calls {
        let mut call = call.clone();
        if let Some(reference) = call[1].get("#ids").cloned() {
            let source = answers
                .iter()
                .find(|answer| answer[2] == reference["resultOf"])
                .expect("referenced answer");
            let ids = source[1]["ids"].clone();
            let args = call[1].as_object_mut().unwrap();
            args.remove("#ids");
            args.insert("ids".into(), ids);
        }
        let response = hot.send(account, vec![call]).await;
        answers.extend(method_responses(&response));
    }
    let session = hot.engine.session_state(account).await;
    hot.attach(true);
    (answers, session)
}

async fn assert_matches_oracle(hot: &Hot, what: &str, account: &str, calls: Vec<Value>) -> Value {
    let response = hot.send(account, calls.clone()).await;
    let (expected, session) = oracle(hot, account, &calls).await;
    assert_eq!(method_responses(&response), expected, "{what}: batched response differs from the oracle");
    assert_eq!(response["sessionState"], json!(session), "{what}: sessionState");
    response
}

async fn differential(hot: &Hot, backend: &str) {
    for page in [5, 50] {
        for (name, calls) in routes(&hot.a, &hot.a_ids, page) {
            assert_matches_oracle(hot, &format!("{backend} {name} page {page}"), &hot.a, calls).await;
        }
    }
    // The fixture must exercise what the shared posture gates: a cached
    // verdict summary on the first message of the page.
    let response = hot.send(&hot.a, routes(&hot.a, &hot.a_ids, 5)[2].1.clone()).await;
    let first = &response["methodResponses"][0][1]["list"][0];
    assert!(first[crate::security::list_auth::AUTH_PROPERTY].is_object(), "{first}");

    // Zero-access: the projection is skipped exactly as before, on every route.
    hot.set_zero_access(&hot.a, true).await;
    for (name, calls) in routes(&hot.a, &hot.a_ids, 5) {
        let response = assert_matches_oracle(hot, &format!("{backend} {name} zero-access"), &hot.a, calls).await;
        for method in method_responses(&response) {
            if method[0] == "Email/get" {
                for email in method[1]["list"].as_array().unwrap() {
                    assert_eq!(email[crate::security::list_auth::AUTH_PROPERTY], Value::Null, "{email}");
                    assert_eq!(email["mailwomanMentions"], json!([]), "{email}");
                }
            }
        }
    }
    // An explicit `enabled=0` row is standard again.
    hot.set_zero_access(&hot.a, false).await;
    let response = assert_matches_oracle(hot, &format!("{backend} zero-access off"), &hot.a, routes(&hot.a, &hot.a_ids, 5)[2].1.clone()).await;
    assert!(response["methodResponses"][0][1]["list"][0][crate::security::list_auth::AUTH_PROPERTY].is_object());
}

#[tokio::test]
async fn shared_reads_answer_exactly_as_the_unshared_oracle_sqlite() {
    differential(&Hot::sqlite().await, "sqlite").await;
}

#[tokio::test]
async fn shared_reads_answer_exactly_as_the_unshared_oracle_postgres() {
    let Some(hot) = Hot::pg().await else {
        mw_test_gate::skip("DATABASE_URL_PG and MW_E14_PG_DSN unset: hot route differential on PostgreSQL");
        return;
    };
    differential(&hot, "postgres").await;
}

async fn isolation(hot: &Hot, backend: &str) {
    // Account B is zero-access; A is not. A's request must neither see B's
    // rows nor take B's posture, and B's must not take A's.
    hot.set_zero_access(&hot.b, true).await;
    let mixed: Vec<String> = hot.a_ids[..3].iter().chain(&hot.b_ids[..3]).cloned().collect();
    let calls = vec![json!(["Email/get", {"accountId":hot.a,"ids":mixed,"properties":header_props()}, "g"])];
    let response = assert_matches_oracle(hot, &format!("{backend} mixed ids"), &hot.a, calls).await;
    let answer = &response["methodResponses"][0][1];
    let listed: Vec<&str> = answer["list"].as_array().unwrap().iter().map(|e| e["id"].as_str().unwrap()).collect();
    assert_eq!(listed, hot.a_ids[..3].iter().map(String::as_str).collect::<Vec<_>>(), "{answer}");
    assert_eq!(answer["notFound"], json!(hot.b_ids[..3]), "foreign ids are notFound: {answer}");
    assert!(answer["list"][0][crate::security::list_auth::AUTH_PROPERTY].is_object(), "A keeps its own standard posture");
    assert!(!answer.to_string().contains("hot-b "), "no field of B's messages: {answer}");

    // B's own request: zero-access applies to it.
    let calls = vec![json!(["Email/get", {"accountId":hot.b,"ids":hot.b_ids[..3],"properties":header_props()}, "g"])];
    let response = assert_matches_oracle(hot, &format!("{backend} zero-access B"), &hot.b, calls).await;
    assert_eq!(response["methodResponses"][0][1]["list"][0][crate::security::list_auth::AUTH_PROPERTY], Value::Null);

    // Only ids of another account: nothing owned, so nothing is listed and no
    // posture is inferred from a row that does not exist.
    let calls = vec![json!(["Email/get", {"accountId":hot.a,"ids":hot.b_ids[..2],"properties":header_props()}, "g"])];
    let response = assert_matches_oracle(hot, &format!("{backend} only foreign ids"), &hot.a, calls).await;
    assert_eq!(response["methodResponses"][0][1]["list"], json!([]));

    // State tokens are per account: B's counters never answer A's request.
    hot.store().record_change(&hot.b, "Email", "b-only", "created").await.unwrap();
    let response = hot.send(&hot.a, vec![json!(["Email/query", {"accountId":hot.a,"limit":5}, "q"])]).await;
    assert_eq!(
        response["methodResponses"][0][1]["queryState"],
        json!(hot.store().current_state(&hot.a, "Email").await.unwrap().to_string())
    );
    hot.set_zero_access(&hot.b, false).await;
}

#[tokio::test]
async fn shared_reads_stay_inside_their_account_sqlite() {
    isolation(&Hot::sqlite().await, "sqlite").await;
}

#[tokio::test]
async fn shared_reads_stay_inside_their_account_postgres() {
    let Some(hot) = Hot::pg().await else {
        mw_test_gate::skip("DATABASE_URL_PG and MW_E14_PG_DSN unset: hot route isolation on PostgreSQL");
        return;
    };
    isolation(&hot, "postgres").await;
}

async fn writes_inside_the_batch(hot: &Hot, backend: &str) {
    let a = &hot.a;
    let id = &hot.a_ids[0];
    let get = |call: &str| json!(["Email/get", {"accountId":a,"ids":[id],"properties":["id","keywords"]}, call]);
    // read, mail write, read: the second read sees the write and a new state.
    let response = hot
        .send(a, vec![get("before"), json!(["Email/set", {"accountId":a,"update":{id.clone():{"keywords/$flagged":true}}}, "write"]), get("after")])
        .await;
    let methods = method_responses(&response);
    assert_eq!(methods[1][0], "Email/set", "{backend}: {response}");
    assert!(methods[1][1]["updated"].as_object().unwrap().contains_key(id), "{backend}: {response}");
    assert_eq!(methods[0][1]["list"][0]["keywords"].get("$flagged"), None, "{response}");
    assert_eq!(methods[2][1]["list"][0]["keywords"]["$flagged"], json!(true), "{response}");
    let before: u64 = methods[0][1]["state"].as_str().unwrap().parse().unwrap();
    let after: u64 = methods[2][1]["state"].as_str().unwrap().parse().unwrap();
    assert!(after > before, "{backend}: the read after the write carries a fresh state: {response}");
    assert_eq!(methods[2][1]["state"], methods[1][1]["newState"], "{backend}: {response}");
    assert_eq!(response["sessionState"], json!(hot.engine.session_state(a).await), "{backend}: fresh sessionState after a write");

    // A non-mail write (PIM) between reads: the closing sessionState sees it.
    let response = hot
        .send(a, vec![get("before"), json!(["Note/set", {"accountId":a,"create":{"n":{"title":"in-batch"}}}, "note"]), get("after")])
        .await;
    assert!(method_responses(&response)[1][1]["created"]["n"].is_object(), "{backend}: {response}");
    assert_eq!(response["sessionState"], json!(hot.engine.session_state(a).await), "{backend}: {response}");
    assert!(response["sessionState"].as_str().unwrap().contains("p1"), "{backend}: {response}");

    // Nothing is kept between requests: an out-of-band write between two
    // requests is visible in the second.
    let first = hot.send(a, vec![get("g")]).await;
    let state = hot.engine.record_change(a, ChangeType::Email, "elsewhere", ChangeOp::Updated).await.unwrap();
    let second = hot.send(a, vec![get("g")]).await;
    assert_ne!(first["methodResponses"][0][1]["state"], second["methodResponses"][0][1]["state"]);
    assert_eq!(second["methodResponses"][0][1]["state"], json!(state.to_string()), "{backend}");
    assert_eq!(second["sessionState"], json!(hot.engine.session_state(a).await), "{backend}");
}

#[tokio::test]
async fn a_write_in_the_batch_is_seen_by_later_reads_sqlite() {
    writes_inside_the_batch(&Hot::sqlite().await, "sqlite").await;
}

#[tokio::test]
async fn a_write_in_the_batch_is_seen_by_later_reads_postgres() {
    let Some(hot) = Hot::pg().await else {
        mw_test_gate::skip("DATABASE_URL_PG and MW_E14_PG_DSN unset: in-batch writes on PostgreSQL");
        return;
    };
    writes_inside_the_batch(&hot, "postgres").await;
}

/// Outside a JMAP request (no frame), the state readers keep their original
/// one-statement-per-call cost and value.
#[tokio::test]
async fn without_a_request_frame_every_state_read_is_fresh() {
    let hot = Hot::sqlite().await;
    let (_, stmts) = counted(&hot.threads, async {
        hot.engine.response_type_state(&hot.a, ChangeType::Email).await.unwrap();
        hot.engine.response_type_state(&hot.a, ChangeType::Email).await.unwrap();
    })
    .await;
    assert_eq!(stmts.len(), 2, "{stmts:#?}");
}

// ── the client's refresh pair (t50hp-e7) ─────────────────────────────────────

/// The push-fallback poll of the web client (`realtime/queryDelta.ts`):
/// `Email/queryChanges` over the open folder plus `Email/changes`, from the
/// states the last list read returned.
async fn refresh_pair(hot: &Hot, account: &str) -> Vec<Value> {
    let boxes = hot.send(account, vec![json!(["Mailbox/get", {"accountId":account}, "m"])]).await;
    let inbox = boxes["methodResponses"][0][1]["list"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["role"] == "inbox")
        .unwrap()["id"]
        .clone();
    let listed = hot
        .send(
            account,
            vec![
                json!(["Email/query", {"accountId":account,"filter":{"inMailbox":inbox},"limit":25,"calculateTotal":true}, "q"]),
                json!(["Email/get", {"accountId":account,"#ids":{"resultOf":"q","name":"Email/query","path":"/ids"},"properties":["id"]}, "g"]),
            ],
        )
        .await;
    let query = &listed["methodResponses"][0][1];
    let state = &listed["methodResponses"][1][1]["state"];
    vec![
        json!(["Email/queryChanges", {"accountId":account,"filter":{"inMailbox":inbox},"sinceQueryState":query["queryState"],"maxChanges":1000,"calculateTotal":true,"upToId":query["ids"][24]}, "delta-query"]),
        json!(["Email/changes", {"accountId":account,"sinceState":state,"maxChanges":1000}, "delta-emails"]),
    ]
}

/// Warm statements of a refresh pair with nothing changed. Before t50hp-e7: 9
/// (queryChanges: state, state, change log, mailbox, count; Email/changes:
/// state, state, change log; closing sessionState). After: the shared state
/// frame, the mailbox ownership read and the `calculateTotal` count.
const REFRESH_PAIR_UNCHANGED: usize = 3;

async fn refresh_pair_budget(hot: &Hot, backend: &str) {
    let calls = refresh_pair(hot, &hot.a).await;
    let (response, stmts) = hot.warm(&hot.a, calls).await;
    eprintln!("t50hp-e7 {backend} refresh-pair unchanged={}", stmts.len());
    for f in families(&stmts) {
        eprintln!("t50hp-e7 {backend} refresh-pair   {f}");
    }
    let methods = method_responses(&response);
    assert_eq!(methods[0][0], "Email/queryChanges", "{response}");
    assert_eq!(methods[0][1]["oldQueryState"], methods[0][1]["newQueryState"], "{response}");
    assert_eq!(methods[0][1]["removed"], json!([]), "{response}");
    assert_eq!(methods[0][1]["total"], json!(SEEDED), "{response}");
    assert_eq!(methods[1][0], "Email/changes", "{response}");
    assert_eq!(methods[1][1]["oldState"], methods[1][1]["newState"], "{response}");
    assert_eq!(methods[1][1]["hasMoreChanges"], json!(false), "{response}");
    assert_eq!(stmts.len(), REFRESH_PAIR_UNCHANGED, "{backend}: {:#?}", families(&stmts));
}

#[tokio::test]
async fn the_unchanged_refresh_pair_reads_state_once_sqlite() {
    refresh_pair_budget(&Hot::sqlite().await, "sqlite").await;
}

#[tokio::test]
async fn the_unchanged_refresh_pair_reads_state_once_postgres() {
    let Some(hot) = Hot::pg().await else {
        mw_test_gate::skip("DATABASE_URL_PG and MW_E14_PG_DSN unset: refresh pair budget on PostgreSQL");
        return;
    };
    refresh_pair_budget(&hot, "postgres").await;
}

/// The pair answers as the oracle (each method its own request, every state
/// read fresh): unchanged, after a change in the folder, after a destroy, from
/// a state the account never reached, and after a change of another account.
async fn refresh_pair_differential(hot: &Hot, backend: &str) {
    let a = &hot.a;
    let calls = refresh_pair(hot, a).await;
    assert_matches_oracle(hot, &format!("{backend} refresh unchanged"), a, calls.clone()).await;
    hot.store().record_change(&hot.b, "Email", "b-only", "created").await.unwrap();
    assert_matches_oracle(hot, &format!("{backend} refresh other account"), a, calls.clone()).await;
    hot.engine.record_change(a, ChangeType::Email, &hot.a_ids[3], ChangeOp::Updated).await.unwrap();
    hot.engine.record_change(a, ChangeType::Email, "gone", ChangeOp::Destroyed).await.unwrap();
    let response = assert_matches_oracle(hot, &format!("{backend} refresh changed"), a, calls.clone()).await;
    let methods = method_responses(&response);
    assert_eq!(methods[0][1]["added"], json!([{"id": hot.a_ids[3], "index": 3}]), "{response}");
    assert_eq!(methods[1][1]["updated"], json!([hot.a_ids[3]]), "{response}");
    assert_eq!(methods[1][1]["destroyed"], json!(["gone"]), "{response}");
    assert_ne!(methods[1][1]["oldState"], methods[1][1]["newState"], "{response}");
    // A state from the future is refused, as before.
    let mut future = calls.clone();
    future[0][1]["sinceQueryState"] = json!("999999");
    future[1][1]["sinceState"] = json!("999999");
    let response = assert_matches_oracle(hot, &format!("{backend} refresh unknown state"), a, future).await;
    for method in method_responses(&response) {
        assert_eq!(method[0], "error", "{response}");
        assert_eq!(method[1]["type"], "cannotCalculateChanges", "{response}");
    }
    // A write earlier in the same request: the changes read after it is fresh.
    let id = &hot.a_ids[0];
    let current = hot.store().current_state(a, "Email").await.unwrap().to_string();
    let response = hot
        .send(a, vec![
            json!(["Email/set", {"accountId":a,"update":{id.clone():{"keywords/$seen":true}}}, "write"]),
            json!(["Email/changes", {"accountId":a,"sinceState":current,"maxChanges":1000}, "after"]),
        ])
        .await;
    let methods = method_responses(&response);
    assert_eq!(methods[1][1]["updated"], json!([id]), "{backend}: {response}");
    assert_eq!(methods[1][1]["newState"], methods[0][1]["newState"], "{backend}: {response}");
    assert_eq!(response["sessionState"], json!(hot.engine.session_state(a).await), "{backend}: {response}");
}

#[tokio::test]
async fn the_refresh_pair_answers_as_the_unshared_oracle_sqlite() {
    refresh_pair_differential(&Hot::sqlite().await, "sqlite").await;
}

#[tokio::test]
async fn the_refresh_pair_answers_as_the_unshared_oracle_postgres() {
    let Some(hot) = Hot::pg().await else {
        mw_test_gate::skip("DATABASE_URL_PG and MW_E14_PG_DSN unset: refresh pair differential on PostgreSQL");
        return;
    };
    refresh_pair_differential(&hot, "postgres").await;
}

// ── the client's list batch with its unread count (t50hp-e26, S3-a) ─────────

/// `workspaceMessages`' three calls: the page, its headers, and the unread
/// count of the folder (`notKeyword: $seen`, `limit: 1`, `calculateTotal`).
fn list_batch(account: &str, inbox: &str, page: usize) -> Vec<Value> {
    vec![
        json!(["Email/query", {"accountId":account,"filter":{"inMailbox":inbox},"limit":page,"position":0,"calculateTotal":true}, "query"]),
        json!(["Email/get", {"accountId":account,"#ids":{"resultOf":"query","name":"Email/query","path":"/ids"},"properties":header_props()}, "rows"]),
        json!(["Email/query", {"accountId":account,"filter":{"inMailbox":inbox,"notKeyword":"$seen"},"limit":1,"calculateTotal":true}, "unread"]),
    ]
}

/// Warm statements of the list batch through the generic path and with the
/// S3-a shortcut (total from the counter).
///
/// Since t50hp-e31 the unread query reads SQL, not the index: generic = R1
/// (the folder read, the projection page, its COUNT; before: the folder read,
/// the account's mailbox list for the index scope, then the index), and the
/// shortcut's limit-1 id is one projection statement where it was an index
/// top-1. Each count is one higher than before (10 -> 11, 9 -> 10); the index
/// work both replaced is not SQL and does not show here. The planner's
/// trigger check is reused for 5 s, so the warm runs do not repeat it.
const LIST_BATCH_GENERIC: usize = 11;
const LIST_BATCH_SHORTCUT: usize = 10;

async fn list_batch_budget(hot: &Hot, backend: &str) {
    let inbox = hot
        .store()
        .list_mailboxes(&hot.a)
        .await
        .unwrap()
        .into_iter()
        .find(|m| m.role.as_deref() == Some("inbox"))
        .unwrap()
        .id;
    let generic_only = |on: bool| crate::unread_count_shortcut::GENERIC_ONLY.with(|g| g.set(on));
    for page in [5, 50] {
        generic_only(true);
        let (_, before) = hot.warm(&hot.a, list_batch(&hot.a, &inbox, page)).await;
        generic_only(false);
        let (response, after) = hot.warm(&hot.a, list_batch(&hot.a, &inbox, page)).await;
        assert_answered("list-batch", &response, page);
        eprintln!("t50hp-e26 {backend} list-batch page{page}: generic={} shortcut={}", before.len(), after.len());
        for f in families(&after) {
            eprintln!("t50hp-e26 {backend} list-batch   {f}");
        }
        assert_eq!(before.len(), LIST_BATCH_GENERIC, "{backend} generic list batch: {:#?}", families(&before));
        assert_eq!(after.len(), LIST_BATCH_SHORTCUT, "{backend} list batch with the shortcut: {:#?}", families(&after));
        // Every seeded inbox message is unread; the counter says so.
        assert_eq!(response["methodResponses"][2][1]["total"], json!(SEEDED), "{response}");
    }
    // Zero-access: the same read sees the posture and the generic path answers.
    hot.set_zero_access(&hot.a, true).await;
    let response = hot.send(&hot.a, list_batch(&hot.a, &inbox, 5)).await;
    generic_only(true);
    let generic = hot.send(&hot.a, list_batch(&hot.a, &inbox, 5)).await;
    generic_only(false);
    assert_eq!(response["methodResponses"][2], generic["methodResponses"][2], "zero-access keeps the generic answer");
    hot.set_zero_access(&hot.a, false).await;
}

#[tokio::test]
async fn list_batch_unread_count_statements_sqlite() {
    list_batch_budget(&Hot::sqlite().await, "sqlite").await;
}

#[tokio::test]
async fn list_batch_unread_count_statements_postgres() {
    let Some(hot) = Hot::pg().await else {
        mw_test_gate::skip("DATABASE_URL_PG and MW_E14_PG_DSN unset: list batch statements on PostgreSQL");
        return;
    };
    list_batch_budget(&hot, "postgres").await;
}

// ── t50hp-e31: statements per planner route (measurement) ───────────────────

/// Warm statements of common metadata queries on the planner and on the old
/// path (index only, `query_plan::LEGACY`). The SQL statement count is what
/// this measures; the index work the planner removes is not SQL and does not
/// show here. Printed for the log; the planner must answer each.
async fn planner_route_statements(hot: &Hot, backend: &str) {
    let inbox = hot
        .store()
        .list_mailboxes(&hot.a)
        .await
        .unwrap()
        .into_iter()
        .find(|m| m.role.as_deref() == Some("inbox"))
        .unwrap()
        .id;
    let legacy = |on: bool| crate::query_plan::LEGACY.with(|l| l.set(on));
    for (name, filter) in [
        ("R1 unread in folder", json!({"inMailbox": inbox, "notKeyword": "$seen"})),
        ("R1 flagged account-wide", json!({"hasKeyword": "$flagged"})),
        ("R1 label in folder", json!({"inMailbox": inbox, "hasKeyword": "work"})),
        ("R3 unread + text in folder", json!({"inMailbox": inbox, "notKeyword": "$seen", "text": "body"})),
        ("R3 text is:unread OR from:", json!({"inMailbox": inbox, "text": "is:unread OR from:nobody"})),
        ("R2 text in folder", json!({"inMailbox": inbox, "text": "body"})),
    ] {
        let calls = vec![json!(["Email/query", {"accountId": hot.a, "filter": filter, "limit": 50, "position": 0, "calculateTotal": true}, "q"])];
        legacy(true);
        let (old, before) = hot.warm(&hot.a, calls.clone()).await;
        legacy(false);
        let (new, after) = hot.warm(&hot.a, calls).await;
        assert_eq!(new["methodResponses"][0][0], json!("Email/query"), "{name}: {new}");
        // This fixture's index is empty (rows are seeded into the store only),
        // so the old path's answers are not compared; its statements are.
        assert_eq!(old["methodResponses"][0][0], json!("Email/query"), "{name}: {old}");
        eprintln!("t50hp-e31 {backend} {name}: old path {} statements, planner {}", before.len(), after.len());
        for f in families(&after) {
            eprintln!("t50hp-e31 {backend}   {f}");
        }
    }
}

#[tokio::test]
async fn planner_route_statements_sqlite() {
    planner_route_statements(&Hot::sqlite().await, "sqlite").await;
}

#[tokio::test]
async fn planner_route_statements_postgres() {
    let Some(hot) = Hot::pg().await else {
        mw_test_gate::skip("DATABASE_URL_PG and MW_E14_PG_DSN unset: planner route statements on PostgreSQL");
        return;
    };
    planner_route_statements(&hot, "postgres").await;
}
