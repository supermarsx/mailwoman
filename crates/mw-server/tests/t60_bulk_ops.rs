//! t60: opt-in bulk mailbox operation benchmark on large mailboxes.
//!
//! One ignored test owns a temporary encrypted SQLite database (or a fresh
//! PostgreSQL schema), seeds accounts with 10,000/100,000 messages through the
//! real Store APIs, builds the production router with the fixture engine
//! factory, and drives bulk scenarios either through `Engine::handle_jmap`
//! (`MW_BULK_LAYER=engine`) or through HTTP with the enforced security profile
//! (`MW_BULK_LAYER=http`, the default). Each scenario prints one
//! `BULK_OPS_RECORD {json}` line. A scenario whose end state does not verify is
//! reported as FAILED without numbers; a server-side refusal is REFUSED with the
//! point where it stopped. See docs/perf/bulk-operations.md.
#![cfg(feature = "load-test-fixtures")]

use async_trait::async_trait;
use mw_engine::{Engine, MailSubmitter, account::AccountRuntime, backend::*};
use mw_server::{AppConfig, SecurityConfig, ServerMode, V6Config, build_app_with_fixture_engine};
use mw_store::{AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store, identity_principal::VerifiedPrimary};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[path = "common/bulk_ops_corpus.rs"]
mod corpus;
#[path = "common/bulk_ops_backend.rs"]
mod backend;
#[path = "common/native_enforced_policy.rs"]
mod native_enforced_policy;
#[path = "common/t34_modes.rs"]
mod t34_modes;
#[path = "common/bulk_ops_search.rs"]
mod search_ops;
#[path = "common/process_cpu.rs"]
mod process_cpu;
use process_cpu::process_cpu_seconds;

use backend::BulkBackend;

const WORKER_ENV: &str = "MW_BULK_OPS_WORKER";
const DOMAIN: &str = "native-load.invalid";
const USING: [&str; 2] = ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"];
const PAGE: u64 = 50;

// ---------------------------------------------------------------- config ---

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Layer {
    Engine,
    Http,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OnLimit {
    Stop,
    RetryAfter,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Batch {
    Max,
    Fixed(usize),
}
const SCENARIOS: [&str; 10] = ["ingest", "open", "search", "flag", "move", "mark-folder-read", "import", "trash", "destroy", "delete-folder"];

#[derive(Clone, Debug)]
struct Config {
    layer: Layer,
    mailbox: u32,
    operation: u32,
    reader_messages: u32,
    reader_accounts: usize,
    scenarios: Vec<&'static str>,
    batch: Batch,
    on_limit: OnLimit,
    concurrent_reader: bool,
    reader_rate_per_second: u32,
    scenario_deadline: Duration,
    ready_deadline: Duration,
    workers: usize,
    profile: native_enforced_policy::Profile,
}

fn bounded(name: &str, raw: Option<&str>, default: u64, low: u64, high: u64) -> std::result::Result<u64, String> {
    match raw.map(str::trim).filter(|v| !v.is_empty()) {
        None => Ok(default),
        Some(value) if value.bytes().all(|b| b.is_ascii_digit()) => value
            .parse::<u64>()
            .ok()
            .filter(|n| (low..=high).contains(n))
            .ok_or_else(|| format!("{name}={value:?} must be an integer from {low} to {high}")),
        Some(value) => Err(format!("{name}={value:?} must be an integer from {low} to {high}")),
    }
}

impl Config {
    fn parse(var: impl Fn(&str) -> Option<String>) -> std::result::Result<Self, String> {
        let get = |name: &str| var(name);
        let layer = match get("MW_BULK_LAYER").as_deref() {
            None | Some("http") => Layer::Http,
            Some("engine") => Layer::Engine,
            Some(other) => return Err(format!("MW_BULK_LAYER={other:?} must be http or engine")),
        };
        let mailbox = bounded("MW_BULK_MAILBOX_MESSAGES", get("MW_BULK_MAILBOX_MESSAGES").as_deref(), 10_000, 100, 100_000)? as u32;
        let operation = bounded("MW_BULK_OPERATION_MESSAGES", get("MW_BULK_OPERATION_MESSAGES").as_deref(), 1_000, 1, 100_000)? as u32;
        if operation > mailbox {
            return Err("MW_BULK_OPERATION_MESSAGES cannot exceed MW_BULK_MAILBOX_MESSAGES".into());
        }
        let reader_messages = bounded("MW_BULK_READER_MESSAGES", get("MW_BULK_READER_MESSAGES").as_deref(), 1_000, 100, 10_000)? as u32;
        let reader_accounts = bounded("MW_BULK_READER_ACCOUNTS", get("MW_BULK_READER_ACCOUNTS").as_deref(), 2, 1, 8)? as usize;
        let scenarios = match get("MW_BULK_SCENARIOS").as_deref().map(str::trim) {
            None | Some("") | Some("all") => SCENARIOS.to_vec(),
            Some(list) => {
                let mut chosen = Vec::new();
                for name in list.split(',').map(str::trim) {
                    let Some(known) = SCENARIOS.iter().find(|s| **s == name) else {
                        return Err(format!("unknown scenario {name:?}; choose from {}", SCENARIOS.join(",")));
                    };
                    if !chosen.contains(known) {
                        chosen.push(*known);
                    }
                }
                // Run in the declared order: later scenarios depend on earlier state.
                SCENARIOS.iter().copied().filter(|s| chosen.contains(s)).collect()
            }
        };
        let batch = match get("MW_BULK_BATCH").as_deref().map(str::trim) {
            None | Some("") | Some("max") => Batch::Max,
            Some(raw) => Batch::Fixed(bounded("MW_BULK_BATCH", Some(raw), 0, 1, 5_000)? as usize),
        };
        let on_limit = match get("MW_BULK_ON_LIMIT").as_deref() {
            None | Some("stop") => OnLimit::Stop,
            Some("retry-after") => OnLimit::RetryAfter,
            Some(other) => return Err(format!("MW_BULK_ON_LIMIT={other:?} must be stop or retry-after")),
        };
        let concurrent_reader = match get("MW_BULK_CONCURRENT_READER").as_deref() {
            None | Some("on") => true,
            Some("off") => false,
            Some(other) => return Err(format!("MW_BULK_CONCURRENT_READER={other:?} must be on or off")),
        };
        let reader_rate_per_second = bounded("MW_BULK_READER_RATE", get("MW_BULK_READER_RATE").as_deref(), 2, 1, 20)? as u32;
        let scenario_deadline = Duration::from_secs(bounded("MW_BULK_SCENARIO_DEADLINE_SECONDS", get("MW_BULK_SCENARIO_DEADLINE_SECONDS").as_deref(), 3_600, 10, 86_400)?);
        let ready_deadline = Duration::from_secs(bounded("MW_BULK_READY_SECONDS", get("MW_BULK_READY_SECONDS").as_deref(), 7_200, 10, 86_400)?);
        let workers = bounded("MW_BULK_WORKERS", get("MW_BULK_WORKERS").as_deref(), 4, 1, 32)? as usize;
        let profile = native_enforced_policy::Profile::parse(Some(get("MW_BULK_SECURITY_PROFILE").as_deref().unwrap_or("enforced")))
            .map_err(|_| "MW_BULK_SECURITY_PROFILE must be enforced or defaults".to_string())?;
        Ok(Self { layer, mailbox, operation, reader_messages, reader_accounts, scenarios, batch, on_limit, concurrent_reader, reader_rate_per_second, scenario_deadline, ready_deadline, workers, profile })
    }
    fn from_environment() -> std::result::Result<Self, String> {
        Self::parse(|name| std::env::var(name).ok())
    }
    fn wants(&self, scenario: &str) -> bool {
        self.scenarios.contains(&scenario)
    }
    fn summary(&self) -> Value {
        json!({"layer":match self.layer {Layer::Engine=>"engine",Layer::Http=>"http"},"mailboxMessages":self.mailbox,"operationMessages":self.operation,
            "readerMessages":self.reader_messages,"readerAccounts":self.reader_accounts,"scenarios":self.scenarios,"batch":match self.batch {Batch::Max=>json!("maxObjectsInSet"),Batch::Fixed(n)=>json!(n)},
            "onLimit":match self.on_limit {OnLimit::Stop=>"stop",OnLimit::RetryAfter=>"retry-after"},"concurrentReader":self.concurrent_reader,
            "readerRatePerSecond":self.reader_rate_per_second,"scenarioDeadlineSeconds":self.scenario_deadline.as_secs(),
            "runtimeWorkerThreads":self.workers,"securityProfile":match self.profile {native_enforced_policy::Profile::Enforced=>"enforced",native_enforced_policy::Profile::Defaults=>"defaults"}})
    }
}

#[test]
fn configuration_is_strict_bounded_and_ordered() {
    let env = |pairs: &'static [(&'static str, &'static str)]| move |name: &str| pairs.iter().find(|(k, _)| *k == name).map(|(_, v)| v.to_string());
    let default = Config::parse(env(&[])).unwrap();
    assert_eq!((default.layer, default.mailbox, default.operation, default.batch), (Layer::Http, 10_000, 1_000, Batch::Max));
    assert_eq!(default.scenarios, SCENARIOS.to_vec());
    assert_eq!(default.profile, native_enforced_policy::Profile::Enforced);
    let chosen = Config::parse(env(&[("MW_BULK_SCENARIOS", "destroy,open,open"), ("MW_BULK_LAYER", "engine"), ("MW_BULK_BATCH", "100")])).unwrap();
    assert_eq!(chosen.scenarios, vec!["open", "destroy"]);
    assert_eq!((chosen.layer, chosen.batch), (Layer::Engine, Batch::Fixed(100)));
    for bad in [
        &[("MW_BULK_LAYER", "grpc")][..],
        &[("MW_BULK_MAILBOX_MESSAGES", "99")],
        &[("MW_BULK_MAILBOX_MESSAGES", "100001")],
        &[("MW_BULK_MAILBOX_MESSAGES", "200"), ("MW_BULK_OPERATION_MESSAGES", "201")],
        &[("MW_BULK_SCENARIOS", "open,nuke")],
        &[("MW_BULK_BATCH", "5001")],
        &[("MW_BULK_BATCH", "0")],
        &[("MW_BULK_ON_LIMIT", "ignore")],
        &[("MW_BULK_SECURITY_PROFILE", "off")],
        &[("MW_BULK_OPERATION_MESSAGES", "+5")],
    ] {
        let owned: Vec<(String, String)> = bad.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        assert!(Config::parse(|name| owned.iter().find(|(k, _)| k == name).map(|(_, v)| v.clone())).is_err(), "{bad:?}");
    }
}

// ------------------------------------------------------------ statistics ---

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let rank = ((p / 100.0) * sorted.len() as f64).ceil() as usize;
    sorted[rank.clamp(1, sorted.len()) - 1]
}
fn distribution(samples: &[f64]) -> Value {
    let mut sorted = samples.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if sorted.is_empty() {
        return json!({"count":0});
    }
    json!({"count":sorted.len(),"p50":percentile(&sorted,50.0),"p95":percentile(&sorted,95.0),"p99":percentile(&sorted,99.0),"max":sorted[sorted.len()-1],"min":sorted[0]})
}
fn ms(duration: Duration) -> f64 {
    (duration.as_secs_f64() * 1_000_000.0).round() / 1000.0
}

#[test]
fn nearest_rank_percentiles_never_interpolate() {
    let samples: Vec<f64> = (1..=100).map(f64::from).collect();
    let d = distribution(&samples);
    assert_eq!((d["p50"].as_f64(), d["p95"].as_f64(), d["p99"].as_f64(), d["max"].as_f64()), (Some(50.0), Some(95.0), Some(99.0), Some(100.0)));
    assert_eq!(distribution(&[7.0])["p99"], 7.0);
    assert_eq!(distribution(&[])["count"], 0);
}

// ------------------------------------------------------------- principals ---

#[derive(Clone)]
struct Principal {
    role: &'static str,
    account: String,
    cookie: String,
    csrf: String,
    client: String,
    backend: Arc<BulkBackend>,
}

struct NoSend;
#[async_trait]
impl MailSubmitter for NoSend {
    async fn submit(&self, _: mw_smtp::Outgoing) -> Result<mw_smtp::SubmissionResult> {
        Err(EngineError::Unsupported("bulk fixture never sends mail".into()))
    }
}

fn store_role(role: MailboxRole) -> Option<&'static str> {
    match role {
        MailboxRole::Inbox => Some("inbox"),
        MailboxRole::Archive => Some("archive"),
        MailboxRole::Trash => Some("trash"),
        MailboxRole::Junk => Some("junk"),
        _ => None,
    }
}

/// Seed through the Store's checked batch API (the native fixture's path).
async fn seed_folder(store: &Store, account: &str, mailbox: &str, tag: &str, to: &str, seeds: std::ops::RangeInclusive<u32>, pattern: bool) -> u64 {
    let first = *seeds.start();
    let mut written = 0u64;
    let seeds: Vec<u32> = seeds.collect();
    for chunk in seeds.chunks(100) {
        let raws: Vec<Vec<u8>> = chunk.iter().map(|seed| corpus::raw(tag, *seed, to)).collect();
        let envelopes: Vec<Vec<u8>> = raws
            .iter()
            .map(|raw| {
                let parsed = mw_mime::parse(raw).expect("corpus parses");
                mw_engine::encode_message_envelope(&parsed.email, raw)
            })
            .collect();
        let ids: Vec<String> = chunk.iter().map(|seed| corpus::message_id(tag, *seed)).collect();
        let dates: Vec<String> = chunk.iter().map(|seed| corpus::received_at(*seed)).collect();
        let flags: Vec<String> = chunk.iter().map(|seed| flags_json(*seed, pattern)).collect();
        let inputs: Vec<(MessageUpsert<'_>, &[u8])> = chunk
            .iter()
            .enumerate()
            .map(|(i, seed)| {
                (
                    MessageUpsert {
                        account_id: account,
                        mailbox_id: mailbox,
                        // The fake upstream assigns UID = position + 1.
                        uid: *seed - first + 1,
                        uidvalidity: 1,
                        message_id: Some(&ids[i]),
                        thread_id: None,
                        internaldate: Some(&dates[i]),
                        size: raws[i].len() as u64,
                        flags_json: &flags[i],
                        envelope: Some(&envelopes[i]),
                        blob_ref: None,
                    },
                    raws[i].as_slice(),
                )
            })
            .collect();
        let mut batch: Vec<(MessageUpsert<'_>, &[u8])> = Vec::new();
        for input in inputs {
            batch.push(input);
            if Store::validate_message_body_batch(&batch).is_err() {
                let next = batch.pop().unwrap();
                assert!(!batch.is_empty(), "one corpus row exceeds the Store batch bound");
                written += write_batch(store, &batch).await;
                batch.clear();
                batch.push(next);
            }
        }
        if !batch.is_empty() {
            written += write_batch(store, &batch).await;
        }
    }
    written
}
fn flags_json(seed: u32, pattern: bool) -> String {
    let (seen, flagged) = if pattern { corpus::pattern_flags(seed) } else { (false, false) };
    let flags: Vec<Flag> = [(seen, Flag::Seen), (flagged, Flag::Flagged)].into_iter().filter(|(on, _)| *on).map(|(_, f)| f).collect();
    serde_json::to_string(&flags).unwrap()
}
fn pattern_unread(seeds: std::ops::RangeInclusive<u32>, pattern: bool) -> u32 {
    if pattern { seeds.filter(|s| !corpus::pattern_flags(*s).0).count() as u32 } else { seeds.count() as u32 }
}
async fn write_batch(store: &Store, batch: &[(MessageUpsert<'_>, &[u8])]) -> u64 {
    let rows = store.upsert_message_bodies_batch(batch).await.expect("seed batch write must succeed once");
    assert!(rows.iter().all(|(_, existed)| !existed), "fresh seed coordinates must not overwrite");
    rows.len() as u64
}

const READER_ROLES: [&str; 8] = ["reader", "reader2", "reader3", "reader4", "reader5", "reader6", "reader7", "reader8"];
/// The search account's Archive holds N/10 more messages (at least 10).
fn search_archive(n: u32) -> u32 {
    (n / 10).max(10)
}

struct Seeded {
    principals: Vec<Principal>,
    expected_documents: u64,
    seeded_messages: u64,
}

/// Accounts: `main` (open, flag, move, mark-folder-read, import), `trash`,
/// `destroy`, `folder` (Bulk folder of K), `ingest` (upstream only) and the
/// concurrent `reader`. Only the accounts the selected scenarios need exist.
async fn seed(store: &Store, config: &Config) -> Seeded {
    let n = config.mailbox;
    let k = config.operation;
    let mut plan: Vec<(&'static str, Vec<(&'static str, std::ops::RangeInclusive<u32>)>, bool)> = Vec::new();
    if config.wants("search") {
        plan.push(("search", vec![("INBOX", 1..=n), ("Archive", n + 1..=n + search_archive(n))], true));
    }
    if ["open", "flag", "move", "mark-folder-read", "import"].iter().any(|s| config.wants(s)) {
        plan.push(("main", vec![("INBOX", 1..=n)], true));
    }
    if config.wants("trash") {
        plan.push(("trash", vec![("INBOX", 1..=n)], true));
    }
    if config.wants("destroy") {
        plan.push(("destroy", vec![("INBOX", 1..=n)], true));
    }
    if config.wants("delete-folder") {
        plan.push(("folder", vec![("Bulk", 1..=k)], true));
    }
    if config.wants("ingest") {
        plan.push(("ingest", vec![("INBOX", 1..=n)], false));
    }
    if config.concurrent_reader {
        for role in &READER_ROLES[..config.reader_accounts] {
            plan.push((role, vec![("INBOX", 1..=config.reader_messages)], true));
        }
    }
    let now = chrono::Utc::now().timestamp();
    let mut principals = Vec::new();
    let (mut expected, mut seeded) = (0u64, 0u64);
    for (index, (role, folders, store_seeded)) in plan.into_iter().enumerate() {
        let username = format!("bulk-{role}@{DOMAIN}");
        let credentials = Credentials { username: username.clone(), password: uuid::Uuid::new_v4().to_string() };
        let account = store
            .create_account(&NewAccount { kind: AccountKind::Imap, host: "127.0.0.1", port: 9, tls: "starttls", username: &username, sync_policy_json: "{}" }, &credentials)
            .await
            .unwrap();
        let tag = role.to_string();
        let pattern = role == "search";
        let backend = BulkBackend::new(&tag, &username, &folders, store_seeded);
        let backend = Arc::new(if pattern { backend.with_flag_pattern() } else { backend });
        if store_seeded {
            for (name, role_kind) in backend::FOLDERS {
                let seeds = folders.iter().find(|(f, _)| *f == name).map(|(_, r)| r.clone());
                let count = seeds.as_ref().map_or(0, |r| r.clone().count() as u32);
                let unread = seeds.as_ref().map_or(0, |r| pattern_unread(r.clone(), pattern));
                let mailbox = store
                    .upsert_mailbox(&MailboxUpsert { account_id: &account, name, role: store_role(role_kind), uidvalidity: 1, uidnext: count + 1, highestmodseq: 0, total: count, unread, parent_id: None })
                    .await
                    .unwrap();
                if let Some(seeds) = seeds {
                    let written = seed_folder(store, &account, &mailbox, &tag, &username, seeds, pattern).await;
                    assert_eq!(written, u64::from(count));
                    seeded += written;
                    expected += written;
                }
                let cursor = serde_json::to_string(&SyncCursor::UidWindow { uidvalidity: 1, uidnext: count + 1 }).unwrap();
                store.save_cursor(&account, &mailbox, &cursor).await.unwrap();
            }
        }
        let session = store.create_session(&account, &username, "engine", "engine", &credentials).await.unwrap();
        store.set_session_authentication(&session, "password").await.unwrap();
        store
            .bind_verified_primary(&session, VerifiedPrimary { account_id: &account, kind: "mailbox", authority: "isolated-native-load", subject: &account }, now)
            .await
            .unwrap()
            .expect("verified durable principal");
        let csrf = store.session_csrf_token(&session);
        eprintln!("bulk-ops: seeded account {role} ({} messages so far)", seeded);
        principals.push(Principal { role, account, cookie: format!("mw_session={session}; mw_csrf={csrf}"), csrf, client: format!("198.18.0.{}", index + 1), backend });
    }
    Seeded { principals, expected_documents: expected, seeded_messages: seeded }
}

// ---------------------------------------------------------------- drivers ---

#[derive(Debug, Clone)]
enum Refusal {
    Http { status: u16, body: String, retry_after: Option<u64> },
    Transport(String),
    Method { call: String, error: Value },
}
impl Refusal {
    fn report(&self) -> Value {
        match self {
            Refusal::Http { status, body, retry_after } => json!({"kind":"http","status":status,"body":body,"retryAfter":retry_after}),
            Refusal::Transport(error) => json!({"kind":"transport","error":error}),
            Refusal::Method { call, error } => json!({"kind":"jmapMethodError","callId":call,"error":error}),
        }
    }
    fn retry_after(&self) -> Option<u64> {
        match self {
            Refusal::Http { status: 429 | 503, retry_after, .. } => Some(retry_after.unwrap_or(1)),
            _ => None,
        }
    }
}

#[async_trait]
trait Driver: Send + Sync {
    async fn post_jmap(&self, p: &Principal, body: &Value) -> std::result::Result<Value, Refusal>;
    /// `Ok(false)`: identity has no meaning at this layer.
    async fn identity(&self, p: &Principal) -> std::result::Result<bool, Refusal>;
    async fn import_mbox(&self, p: &Principal, mailbox: &str, messages: &[Vec<u8>]) -> std::result::Result<Value, Refusal>;
}

struct EngineDriver {
    engine: Arc<Engine>,
}
#[async_trait]
impl Driver for EngineDriver {
    async fn post_jmap(&self, p: &Principal, body: &Value) -> std::result::Result<Value, Refusal> {
        Ok(self.engine.handle_jmap(&p.account, body).await)
    }
    async fn identity(&self, _: &Principal) -> std::result::Result<bool, Refusal> {
        Ok(false)
    }
    async fn import_mbox(&self, p: &Principal, mailbox: &str, messages: &[Vec<u8>]) -> std::result::Result<Value, Refusal> {
        // The same two steps the import route takes: seal each upload, then one
        // Email/import for the batch.
        let mut emails = serde_json::Map::new();
        for (i, message) in messages.iter().enumerate() {
            let blob = self.engine.store_upload(&p.account, "message/rfc822", message).await.map_err(|e| Refusal::Transport(format!("store_upload: {e}")))?;
            emails.insert(format!("i{i}"), json!({"blobId":blob,"mailboxIds":{mailbox:true}}));
        }
        let response = self.engine.handle_jmap(&p.account, &json!({"using":USING,"methodCalls":[["Email/import",{"accountId":p.account,"emails":emails},"imp0"]]})).await;
        let result = &response["methodResponses"][0][1];
        Ok(json!({"imported":result["created"].as_object().map_or(0,|m|m.len()),"failed":result["notCreated"].as_object().map_or(0,|m|m.len()),"total":messages.len()}))
    }
}

struct HttpDriver {
    client: reqwest::Client,
    base: String,
}
impl HttpDriver {
    async fn finish(response: reqwest::Response) -> std::result::Result<Value, Refusal> {
        let status = response.status().as_u16();
        let retry_after = response.headers().get("retry-after").and_then(|v| v.to_str().ok()).and_then(|v| v.parse().ok());
        let bytes = response.bytes().await.map_err(|e| Refusal::Transport(e.to_string()))?;
        if status != 200 {
            let mut body = String::from_utf8_lossy(&bytes).into_owned();
            body.truncate(2048);
            return Err(Refusal::Http { status, body, retry_after });
        }
        serde_json::from_slice(&bytes).map_err(|e| Refusal::Transport(format!("invalid JSON: {e}")))
    }
}
#[async_trait]
impl Driver for HttpDriver {
    async fn post_jmap(&self, p: &Principal, body: &Value) -> std::result::Result<Value, Refusal> {
        let response = self
            .client
            .post(format!("{}/jmap/api", self.base))
            .header("cookie", &p.cookie)
            .header("x-csrf-token", &p.csrf)
            .header("x-forwarded-for", &p.client)
            .header("origin", &self.base)
            .json(body)
            .send()
            .await
            .map_err(|e| Refusal::Transport(e.to_string()))?;
        Self::finish(response).await
    }
    async fn identity(&self, p: &Principal) -> std::result::Result<bool, Refusal> {
        let response = self
            .client
            .get(format!("{}/api/me", self.base))
            .header("cookie", &p.cookie)
            .header("x-forwarded-for", &p.client)
            .send()
            .await
            .map_err(|e| Refusal::Transport(e.to_string()))?;
        Self::finish(response).await.map(|_| true)
    }
    async fn import_mbox(&self, p: &Principal, mailbox: &str, messages: &[Vec<u8>]) -> std::result::Result<Value, Refusal> {
        let response = self
            .client
            .post(format!("{}/api/import/mbox?mailboxId={mailbox}", self.base))
            .header("cookie", &p.cookie)
            .header("x-csrf-token", &p.csrf)
            .header("x-forwarded-for", &p.client)
            .header("origin", &self.base)
            .header("content-type", "application/mbox")
            .body(corpus::mbox(messages.iter().cloned()))
            .send()
            .await
            .map_err(|e| Refusal::Transport(e.to_string()))?;
        Self::finish(response).await
    }
}

// --------------------------------------------------------------- context ---

struct Ctx {
    config: Config,
    driver: Arc<dyn Driver>,
    engine: Arc<Engine>,
    store: Store,
    db_path: Option<PathBuf>,
    index_dir: PathBuf,
    pg: Option<(String, String)>,
    backend_kind: &'static str,
    inputs: Value,
    max_set: usize,
    max_get: usize,
    properties: Value,
    principals: Vec<Principal>,
    report: Option<Mutex<std::fs::File>>,
    rss_peak: Arc<AtomicU64>,
    /// Setup-only HTTP access (saved-search creation), for either layer.
    http: reqwest::Client,
    base: String,
}
impl Ctx {
    fn principal(&self, role: &str) -> &Principal {
        self.principals.iter().find(|p| p.role == role).expect("seeded principal")
    }
    fn batch(&self) -> usize {
        match self.config.batch {
            Batch::Max => self.max_set,
            Batch::Fixed(n) => n.min(self.max_set),
        }
    }
}

#[derive(Default)]
struct Calls {
    requests: u64,
    latencies: Vec<f64>,
    throttled: u64,
    waited_ms: f64,
}
impl Calls {
    fn report(&self) -> Value {
        json!({"requests":self.requests,"latencyMs":distribution(&self.latencies),"throttled":self.throttled,"retryAfterWaitMs":self.waited_ms})
    }
}

/// One JMAP request (`calls` = the `methodCalls` array). A method-level
/// `error` response is a refusal. 429/503 follow `MW_BULK_ON_LIMIT`.
async fn call(ctx: &Ctx, p: &Principal, calls: Value, stats: &mut Calls) -> std::result::Result<Value, Refusal> {
    let body = json!({"using":USING,"methodCalls":calls});
    loop {
        let started = Instant::now();
        let outcome = ctx.driver.post_jmap(p, &body).await;
        stats.requests += 1;
        stats.latencies.push(ms(started.elapsed()));
        match outcome {
            Err(refusal) if ctx.config.on_limit == OnLimit::RetryAfter && refusal.retry_after().is_some() => {
                stats.throttled += 1;
                let wait = Duration::from_secs(refusal.retry_after().unwrap().clamp(1, 60));
                stats.waited_ms += ms(wait);
                tokio::time::sleep(wait).await;
            }
            Err(refusal) => return Err(refusal),
            Ok(value) => {
                if let Some(error) = value["methodResponses"].as_array().and_then(|all| all.iter().find(|m| m[0] == "error")) {
                    return Err(Refusal::Method { call: error[2].as_str().unwrap_or_default().into(), error: error[1].clone() });
                }
                if !value["methodResponses"].is_array() {
                    return Err(Refusal::Transport(format!("no methodResponses: {value}")));
                }
                return Ok(value);
            }
        }
    }
}
fn result(value: &Value, index: usize) -> &Value {
    &value["methodResponses"][index][1]
}

// ------------------------------------------------------------- counters ---

struct Snapshot {
    statements: mw_store::pool_telemetry::QuerySnapshot,
    commits: u64,
    search: (u64, u64, u64),
    publication: Value,
    upstream: Value,
}
fn snapshot(engine: &Engine, p: Option<&Principal>) -> Snapshot {
    Snapshot {
        statements: mw_store::pool_telemetry::process_snapshot(),
        commits: mw_search::commit_count(),
        search: mw_search::write_and_hit_counts(),
        publication: serde_json::to_value(engine.index_publication_status()).unwrap(),
        upstream: p.map_or(json!({}), |p| p.backend.commands.snapshot()),
    }
}
const PUBLICATION_COUNTERS: [&str; 13] = [
    "submittedJobs", "completedJobs", "failedJobs", "publishedBatches", "publishedChanges", "revalidations", "admissionRefusals",
    "journalCoveredRevalidations", "journalAcknowledgedRows", "journalReplayedKeys", "readFenceWaits", "readFenceTimeouts", "readFenceRefusals",
];
fn delta(before: &Snapshot, after: &Snapshot, messages: u64) -> Value {
    let s = |a: &mw_store::pool_telemetry::OperationSnapshot, b: &mw_store::pool_telemetry::OperationSnapshot| b.calls.saturating_sub(a.calls);
    let (read, write, other) = (s(&before.statements.read, &after.statements.read), s(&before.statements.write, &after.statements.write), s(&before.statements.other, &after.statements.other));
    let per = |n: u64| if messages == 0 { Value::Null } else { json!(n as f64 / messages as f64) };
    let mut publication = serde_json::Map::new();
    for key in PUBLICATION_COUNTERS {
        let (a, b) = (before.publication[key].as_u64().unwrap_or(0), after.publication[key].as_u64().unwrap_or(0));
        publication.insert(key.into(), json!(b.saturating_sub(a)));
    }
    for key in ["pendingJobs", "queuedJobs", "pendingBytes", "poisoned", "earlyAck", "ackMode", "journalRows"] {
        publication.insert(format!("{key}After"), after.publication[key].clone());
    }
    let docs = after.search.0.saturating_sub(before.search.0);
    let body = after.search.1.saturating_sub(before.search.1);
    let commits = after.commits.saturating_sub(before.commits);
    json!({
        "statements":{"scope":"process-wide (includes background work)","read":read,"write":write,"other":other,"total":read+write+other,
            "perMessage":per(read+write+other),"writePerMessage":per(write)},
        "commits":{"store":Value::Null,"storeNote":"Store commits are not counted separately (docs/perf/store-operation-benchmark.md)","index":commits,"indexPerMessage":per(commits)},
        "searchIndex":{"documentsWritten":docs,"documentsWrittenPerMessage":per(docs),"bodyBytesTokenised":body,"hitsMaterialised":after.search.2.saturating_sub(before.search.2)},
        "publication":publication,
        "upstream":backend::delta(&before.upstream,&after.upstream),
    })
}

fn dir_bytes(path: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(path) else { return 0 };
    entries
        .flatten()
        .map(|entry| match entry.metadata() {
            Ok(meta) if meta.is_dir() => dir_bytes(&entry.path()),
            Ok(meta) => meta.len(),
            Err(_) => 0,
        })
        .sum()
}
fn file_bytes(path: &Path) -> Option<u64> {
    std::fs::metadata(path).ok().map(|m| m.len())
}
async fn disk(ctx: &Ctx) -> Value {
    let index = dir_bytes(&ctx.index_dir);
    match (&ctx.db_path, &ctx.pg) {
        (Some(db), _) => {
            let side = |suffix: &str| file_bytes(&PathBuf::from(format!("{}{suffix}", db.display())));
            json!({"databaseBytes":file_bytes(db),"walBytes":side("-wal"),"shmBytes":side("-shm"),"journalBytes":side("-journal"),"searchIndexBytes":index})
        }
        (None, Some((raw, schema))) => {
            let bytes = async {
                let pool = sqlx::postgres::PgPoolOptions::new().max_connections(1).connect(raw).await.ok()?;
                let value: Option<i64> = sqlx::query_scalar("SELECT COALESCE(SUM(pg_total_relation_size(c.oid)),0)::bigint FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname=$1 AND c.relkind='r'")
                    .bind(schema)
                    .fetch_one(&pool)
                    .await
                    .ok();
                pool.close().await;
                value
            }
            .await;
            json!({"schemaBytes":bytes,"walBytes":Value::Null,"walNote":"PostgreSQL WAL is cluster-wide; not attributed","searchIndexBytes":index})
        }
        _ => json!({}),
    }
}
fn lifetime_peak_rss() -> Option<u64> {
    if !cfg!(target_os = "linux") {
        return None;
    }
    let text = std::fs::read_to_string("/proc/self/status").ok()?;
    text.lines()
        .find_map(|line| line.strip_prefix("VmHWM:").and_then(|v| v.trim().trim_end_matches("kB").trim().parse::<u64>().ok()))
        .map(|kib| kib * 1024)
}
fn current_rss() -> Option<u64> {
    mw_observability::resource_snapshot::snapshot(Path::new(".")).process_resident_bytes
}
/// Start a scenario's sampled peak at the current resident size.
fn reset_peak(ctx: &Ctx) {
    ctx.rss_peak.store(current_rss().unwrap_or(0), Ordering::SeqCst);
}
fn memory(ctx: &Ctx) -> Value {
    json!({"sampledPeakResidentBytes":ctx.rss_peak.load(Ordering::SeqCst),"samplingMs":200,"residentAfterBytes":current_rss(),"processLifetimePeakResidentBytes":lifetime_peak_rss()})
}

// ---------------------------------------------------------------- records ---

struct Record {
    scenario: String,
    results: serde_json::Map<String, Value>,
    checks: Vec<Value>,
    failed: bool,
    phase: Option<Arc<AtomicU8>>,
    op_ended: Option<Instant>,
}
enum Stop {
    Refused { stage: String, refusal: Value },
    Failed(String),
}
impl Record {
    fn new(scenario: &str) -> Self {
        Self { scenario: scenario.into(), results: Default::default(), checks: Vec::new(), failed: false, phase: None, op_ended: None }
    }
    fn put(&mut self, key: &str, value: impl serde::Serialize) {
        self.results.insert(key.into(), serde_json::to_value(value).unwrap());
    }
    fn check(&mut self, name: &str, pass: bool, detail: Value) -> bool {
        if !pass {
            self.failed = true;
            eprintln!("bulk-ops: {} check {name} FAILED: {detail}", self.scenario);
        }
        self.checks.push(json!({"name":name,"pass":pass,"detail":detail}));
        pass
    }
    /// Quiet window for the concurrent reader, then the operation phase.
    async fn begin_op(&mut self) {
        if let Some(phase) = &self.phase {
            phase.store(PHASE_BEFORE, Ordering::SeqCst);
            tokio::time::sleep(READER_WINDOW).await;
            phase.store(PHASE_DURING, Ordering::SeqCst);
        }
    }
    fn end_op(&mut self) {
        if let Some(phase) = &self.phase {
            phase.store(PHASE_AFTER, Ordering::SeqCst);
        }
        self.op_ended = Some(Instant::now());
    }
}
fn refused(stage: &str, refusal: Refusal) -> Stop {
    Stop::Refused { stage: stage.into(), refusal: refusal.report() }
}

type Outcome = std::result::Result<std::result::Result<(), Stop>, tokio::time::error::Elapsed>;
fn emit(ctx: &Ctx, rec: Record, outcome: Outcome) -> &'static str {
    let (status, failure) = match outcome {
        Err(_) => ("INCOMPLETE", json!({"reason":"scenario deadline exceeded","deadlineSeconds":ctx.config.scenario_deadline.as_secs()})),
        Ok(Err(Stop::Refused { stage, refusal })) => ("REFUSED", json!({"stage":stage,"refusal":refusal})),
        Ok(Err(Stop::Failed(reason))) => ("FAILED", json!({"reason":reason})),
        Ok(Ok(())) if rec.failed => ("FAILED", json!({"reason":"end-state verification failed"})),
        Ok(Ok(())) => ("OK", Value::Null),
    };
    let results = Value::Object(rec.results);
    let mut record = json!({
        "kind":"mw-bulk-ops-v1","scenario":rec.scenario,"layer":ctx.config.summary()["layer"],"backend":ctx.backend_kind,"status":status,
        "inputs":ctx.inputs,"checks":rec.checks,"failure":failure,
    });
    match status {
        // A failed verification reports no numbers.
        "FAILED" => record["results"] = Value::Null,
        "OK" => record["results"] = results,
        _ => record["partialResults"] = results,
    }
    let line = serde_json::to_string(&record).unwrap();
    println!("BULK_OPS_RECORD {line}");
    if let Some(file) = &ctx.report {
        use std::io::Write;
        let mut file = file.lock().unwrap();
        writeln!(file, "{line}").unwrap();
        file.flush().unwrap();
    }
    status
}

// ------------------------------------------------------- concurrent reader ---

const PHASE_SETUP: u8 = 0;
const PHASE_BEFORE: u8 = 1;
const PHASE_DURING: u8 = 2;
const PHASE_AFTER: u8 = 3;
const READER_WINDOW: Duration = Duration::from_secs(3);

type ReaderSamples = Arc<Mutex<Vec<(u8, &'static str, f64, bool)>>>;
struct ReaderRun {
    samples: ReaderSamples,
    stop: Arc<tokio::sync::Notify>,
    tasks: Vec<tokio::task::JoinHandle<()>>,
    accounts: usize,
}
fn list_calls(account: &str, filter: Value, sort: Value, position: u64, properties: &Value) -> Value {
    json!([
        ["Email/query",{"accountId":account,"filter":filter,"sort":sort,"position":position,"limit":PAGE,"calculateTotal":true},"q"],
        ["Email/get",{"accountId":account,"#ids":{"resultOf":"q","name":"Email/query","path":"/ids"},"properties":properties},"g"]
    ])
}
fn newest_first() -> Value {
    json!([{"property":"receivedAt","isAscending":false}])
}
async fn start_reader(ctx: &Arc<Ctx>, rec: &mut Record) -> Option<ReaderRun> {
    if !ctx.config.concurrent_reader {
        return None;
    }
    let phase = Arc::new(AtomicU8::new(PHASE_SETUP));
    rec.phase = Some(phase.clone());
    let samples: ReaderSamples = Arc::new(Mutex::new(Vec::new()));
    let stop = Arc::new(tokio::sync::Notify::new());
    let mut tasks = Vec::new();
    for role in &READER_ROLES[..ctx.config.reader_accounts] {
        let reader = ctx.principal(role).clone();
        let inbox = mailbox_ids(ctx, &reader).await.get("INBOX").cloned()?;
        let (ctx, samples, stop, phase) = (ctx.clone(), samples.clone(), stop.clone(), phase.clone());
        tasks.push(tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_millis(1000 / u64::from(ctx.config.reader_rate_per_second)));
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            let list = json!({"using":USING,"methodCalls":list_calls(&reader.account, json!({"inMailbox":inbox}), newest_first(), 0, &ctx.properties)});
            // The web client's search request: the typed text in filter.text.
            let search = json!({"using":USING,"methodCalls":list_calls(&reader.account, json!({"inMailbox":inbox,"text":corpus::SEARCH_WORD}), newest_first(), 0, &ctx.properties)});
            let expected = PAGE.min(u64::from(ctx.config.reader_messages)) as usize;
            let hits = word_hits(1..=ctx.config.reader_messages);
            loop {
                tokio::select! { _ = stop.notified() => return, _ = ticker.tick() => {} }
                let current = phase.load(Ordering::SeqCst);
                let started = Instant::now();
                let ok = matches!(ctx.driver.post_jmap(&reader, &list).await, Ok(v) if result(&v, 1)["list"].as_array().is_some_and(|l| l.len() == expected));
                samples.lock().unwrap().push((current, "inboxFirstPage", ms(started.elapsed()), ok));
                let started = Instant::now();
                let ok = matches!(ctx.driver.post_jmap(&reader, &search).await, Ok(v) if result(&v, 0)["total"].as_u64() == Some(hits));
                samples.lock().unwrap().push((current, "textSearch", ms(started.elapsed()), ok));
                let started = Instant::now();
                match ctx.driver.identity(&reader).await {
                    Ok(true) => samples.lock().unwrap().push((current, "identity", ms(started.elapsed()), true)),
                    Ok(false) => {}
                    Err(_) => samples.lock().unwrap().push((current, "identity", ms(started.elapsed()), false)),
                }
            }
        }));
    }
    Some(ReaderRun { samples, stop, accounts: tasks.len(), tasks })
}
async fn finish_reader(run: Option<ReaderRun>, rec: &mut Record) {
    let Some(run) = run else { return };
    if let Some(ended) = rec.op_ended {
        let elapsed = ended.elapsed();
        if elapsed < READER_WINDOW {
            tokio::time::sleep(READER_WINDOW - elapsed).await;
        }
    }
    run.stop.notify_waiters();
    for task in run.tasks {
        task.abort();
        let _ = task.await;
    }
    let samples = run.samples.lock().unwrap().clone();
    let accounts = run.accounts;
    let mut windows = serde_json::Map::new();
    for (phase, name) in [(PHASE_BEFORE, "before"), (PHASE_DURING, "during"), (PHASE_AFTER, "after")] {
        let mut entry = serde_json::Map::new();
        for kind in ["inboxFirstPage", "textSearch", "identity"] {
            let chosen: Vec<_> = samples.iter().filter(|s| s.0 == phase && s.1 == kind).collect();
            let latencies: Vec<f64> = chosen.iter().filter(|s| s.3).map(|s| s.2).collect();
            entry.insert(kind.into(), json!({"latencyMs":distribution(&latencies),"failures":chosen.iter().filter(|s| !s.3).count()}));
        }
        windows.insert(name.into(), Value::Object(entry));
    }
    let failures = samples.iter().filter(|s| s.0 != PHASE_SETUP && !s.3).count();
    rec.check("concurrent-reader-requests-succeeded", failures == 0, json!({"failures":failures}));
    rec.put("concurrentReader", json!({"windows":windows,"quietWindowSeconds":READER_WINDOW.as_secs(),
        "accounts":accounts,"note":"other accounts each read their inbox first page, run the client's text search, and (over HTTP) /api/me at a fixed rate per account; 'during' spans the bulk requests only"}));
}

// ---------------------------------------------------------------- helpers ---

async fn mailbox_ids(ctx: &Ctx, p: &Principal) -> HashMap<String, String> {
    ctx.store.list_mailboxes(&p.account).await.unwrap_or_default().into_iter().map(|m| (m.name, m.id)).collect()
}
async fn require_mailbox(ctx: &Ctx, p: &Principal, name: &str) -> std::result::Result<String, Stop> {
    mailbox_ids(ctx, p).await.get(name).cloned().ok_or_else(|| Stop::Failed(format!("mailbox {name} missing for {}", p.role)))
}
/// Every id matching `filter`, newest first, paged by `page`, at most `max`.
async fn query_all(ctx: &Ctx, p: &Principal, filter: Value, max: u64, page: u64, stats: &mut Calls) -> std::result::Result<Vec<String>, Refusal> {
    let mut ids = Vec::new();
    loop {
        let limit = page.min(max - ids.len() as u64);
        if limit == 0 {
            return Ok(ids);
        }
        let value = call(ctx, p, json!([["Email/query",{"accountId":p.account,"filter":filter,"sort":newest_first(),"position":ids.len(),"limit":limit},"q"]]), stats).await?;
        let got: Vec<String> = result(&value, 0)["ids"].as_array().into_iter().flatten().filter_map(|v| v.as_str().map(str::to_owned)).collect();
        // Stop on an empty page, not a short one: a server cap below `page`
        // must not end the listing early.
        if got.is_empty() {
            return Ok(ids);
        }
        ids.extend(got);
    }
}
async fn query_total(ctx: &Ctx, p: &Principal, filter: Value) -> Option<u64> {
    let mut stats = Calls::default();
    let value = call(ctx, p, json!([["Email/query",{"accountId":p.account,"filter":filter,"position":0,"limit":1,"calculateTotal":true},"t"]]), &mut stats).await.ok()?;
    result(&value, 0)["total"].as_u64()
}
async fn wait_for<F, Fut>(limit: Duration, mut probe: F) -> Option<Duration>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let started = Instant::now();
    loop {
        if probe().await {
            return Some(started.elapsed());
        }
        if started.elapsed() >= limit {
            return None;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
async fn store_messages(ctx: &Ctx, ids: &[String]) -> Vec<Option<mw_store::Message>> {
    let mut out = Vec::with_capacity(ids.len());
    for chunk in ids.chunks(500) {
        out.extend(ctx.store.get_messages(chunk).await.expect("store read"));
    }
    out
}
async fn mailbox_message_ids(ctx: &Ctx, mailbox: &str) -> Vec<String> {
    let mut all = Vec::new();
    loop {
        let page = ctx.store.list_message_ids(mailbox, 5000, all.len() as i64).await.expect("store list");
        let done = page.len() < 5000;
        all.extend(page);
        if done {
            return all;
        }
    }
}
fn states_increase(states: &[String]) -> bool {
    let parsed: Vec<Option<u64>> = states.iter().map(|s| s.parse().ok()).collect();
    if parsed.iter().all(Option::is_some) {
        parsed.windows(2).all(|w| w[1] > w[0])
    } else {
        states.windows(2).all(|w| w[1] != w[0])
    }
}
fn word_hits(seeds: std::ops::RangeInclusive<u32>) -> u64 {
    seeds.filter(|seed| seed % 5 == 0).count() as u64
}

#[test]
fn state_order_and_word_hits_are_checked_exactly() {
    assert!(states_increase(&["1".into(), "2".into(), "10".into()]));
    assert!(!states_increase(&["2".into(), "2".into()]));
    assert!(!states_increase(&["3".into(), "2".into()]));
    assert!(states_increase(&["a".into(), "b".into()]));
    assert_eq!(word_hits(1..=10_000), 2_000);
    assert_eq!(word_hits(1..=4), 0);
}

/// Folder counts that every layer must agree on: Store rows, the cached
/// mailbox counters, `Mailbox/get` and the fake upstream.
async fn verify_folder(ctx: &Ctx, rec: &mut Record, p: &Principal, name: &str, total: u64, unread: Option<u64>) {
    let ids = mailbox_ids(ctx, p).await;
    let Some(id) = ids.get(name) else {
        rec.check(&format!("{name}-exists"), false, json!({}));
        return;
    };
    let rows = ctx.store.count_messages_in_mailbox(id).await.unwrap_or(u64::MAX);
    let cached = ctx.store.get_mailbox(id).await.ok();
    let mut stats = Calls::default();
    let jmap = call(ctx, p, json!([["Mailbox/get",{"accountId":p.account,"ids":[id],"properties":["id","totalEmails","unreadEmails"]},"m"]]), &mut stats).await.ok();
    let listed = jmap.as_ref().map(|v| result(v, 0)["list"][0].clone()).unwrap_or(Value::Null);
    let upstream = p.backend.folder_counts(name);
    let detail = json!({"expectedTotal":total,"expectedUnread":unread,"storeRows":rows,"cachedTotal":cached.as_ref().map(|m|m.total),"cachedUnread":cached.as_ref().map(|m|m.unread),
        "jmapTotal":listed["totalEmails"],"jmapUnread":listed["unreadEmails"],"upstream":upstream.map(|(t,u,f)|json!({"total":t,"unread":u,"flagged":f}))});
    let mut pass = rows == total && listed["totalEmails"].as_u64() == Some(total) && upstream.map(|u| u.0 as u64) == Some(total);
    if let Some(unread) = unread {
        pass &= cached.as_ref().map(|m| u64::from(m.unread)) == Some(unread) && listed["unreadEmails"].as_u64() == Some(unread) && upstream.map(|u| u.1 as u64) == Some(unread);
    }
    rec.check(&format!("{name}-counts"), pass, detail);
}
/// Index documents for the account equal its Store rows, with no orphans.
async fn verify_index(ctx: &Ctx, rec: &mut Record, p: &Principal, gone: &[String]) {
    let indexed: HashSet<String> = ctx.engine.search().account_stable_ids(&p.account).unwrap_or_default().into_iter().collect();
    let mut stored = HashSet::new();
    for id in mailbox_ids(ctx, p).await.values() {
        stored.extend(mailbox_message_ids(ctx, id).await);
    }
    let orphans = indexed.difference(&stored).count();
    let missing = stored.difference(&indexed).count();
    let still_present = gone.iter().filter(|id| indexed.contains(*id) || stored.contains(*id)).count();
    rec.check("index-matches-store", orphans == 0 && missing == 0 && still_present == 0,
        json!({"indexDocuments":indexed.len(),"storeRows":stored.len(),"orphanDocuments":orphans,"missingDocuments":missing,"removedIdsStillPresent":still_present}));
}
/// Current (folder total, unread) from the fake upstream, checked against Store.
async fn baseline(ctx: &Ctx, rec: &mut Record, p: &Principal, name: &str) -> std::result::Result<(u64, u64), Stop> {
    let (total, unread, _) = p.backend.folder_counts(name).ok_or_else(|| Stop::Failed(format!("upstream folder {name} missing")))?;
    let id = require_mailbox(ctx, p, name).await?;
    let rows = ctx.store.count_messages_in_mailbox(&id).await.map_err(|e| Stop::Failed(e.to_string()))?;
    if !rec.check(&format!("precondition-{name}"), rows == total as u64, json!({"storeRows":rows,"upstream":total})) {
        return Err(Stop::Failed(format!("precondition: {name} store/upstream differ")));
    }
    Ok((total as u64, unread as u64))
}

// -------------------------------------------------------------- scenarios ---

const VISIBILITY_LIMIT: Duration = Duration::from_secs(120);

#[derive(Default)]
struct SetRun {
    states: Vec<String>,
    done: u64,
    calls: Calls,
}

/// The client's Email/set shape (plus its Mailbox/get) over `ids` in chunks of
/// the configured batch. A partial acceptance is a refusal, not a success.
async fn bulk_email_set(ctx: &Ctx, p: &Principal, ids: &[String], patch: &Value, destroy: bool, run: &mut SetRun) -> std::result::Result<(), Stop> {
    for chunk in ids.chunks(ctx.batch()) {
        let args = if destroy {
            json!({"accountId":p.account,"destroy":chunk,"mailwomanPermanent":true})
        } else {
            let update: serde_json::Map<String, Value> = chunk.iter().map(|id| (id.clone(), patch.clone())).collect();
            json!({"accountId":p.account,"update":update})
        };
        let value = call(ctx, p, json!([["Email/set", args, "s"], ["Mailbox/get", {"accountId":p.account,"ids":null}, "mb"]]), &mut run.calls)
            .await
            .map_err(|r| refused(&format!("Email/set after {} of {} messages", run.done, ids.len()), r))?;
        let set = result(&value, 0);
        let (accepted, rejected) = if destroy {
            (set["destroyed"].as_array().map_or(0, Vec::len), &set["notDestroyed"])
        } else {
            (set["updated"].as_object().map_or(0, |m| m.len()), &set["notUpdated"])
        };
        if run.states.is_empty()
            && let Some(old) = set["oldState"].as_str()
        {
            run.states.push(old.into());
        }
        if let Some(new) = set["newState"].as_str() {
            run.states.push(new.into());
        }
        if accepted != chunk.len() {
            let sample: serde_json::Map<String, Value> = rejected.as_object().into_iter().flatten().take(5).map(|(k, v)| (k.clone(), v.clone())).collect();
            return Err(Stop::Refused {
                stage: format!("Email/set after {} of {} messages", run.done, ids.len()),
                refusal: json!({"kind":"partialAcceptance","requested":chunk.len(),"accepted":accepted,"rejectedSample":sample}),
            });
        }
        run.done += chunk.len() as u64;
    }
    Ok(())
}

/// The newest `k` ids of a folder (what a user's list selection holds).
async fn top_ids(ctx: &Ctx, p: &Principal, folder: &str, k: u64) -> std::result::Result<Vec<String>, Stop> {
    let mut stats = Calls::default();
    let ids = query_all(ctx, p, json!({"inMailbox":folder}), k, 1000, &mut stats).await.map_err(|r| refused("setup: listing the selection", r))?;
    if ids.len() as u64 != k {
        return Err(Stop::Failed(format!("setup: listed {} ids instead of {k}", ids.len())));
    }
    Ok(ids)
}

async fn change_rows(ctx: &Ctx, p: &Principal, email: u64, mailbox: u64) -> Value {
    let rows = |kind: &'static str, since: u64| async move { ctx.store.changes_since(&p.account, kind, since).await.map(|r| r.len()).ok() };
    json!({"email":rows("Email",email).await,"mailbox":rows("Mailbox",mailbox).await})
}
async fn states_now(ctx: &Ctx, p: &Principal) -> (u64, u64) {
    (ctx.store.current_state(&p.account, "Email").await.unwrap_or(0), ctx.store.current_state(&p.account, "Mailbox").await.unwrap_or(0))
}
async fn publication_idle(ctx: &Ctx) -> Option<Duration> {
    wait_for(VISIBILITY_LIMIT, || async move {
        let status = ctx.engine.index_publication_status();
        status.pending_jobs == 0 && status.queued_jobs == 0
    })
    .await
}

/// Everything recorded after a bulk operation's last response.
#[allow(clippy::too_many_arguments)]
async fn finish_op(ctx: &Ctx, rec: &mut Record, p: &Principal, messages: u64, elapsed: Duration, calls: &Calls, before: &Snapshot, states: (u64, u64), probe: Vec<(Value, u64)>) {
    let after = snapshot(&ctx.engine, Some(p));
    rec.put("messages", messages);
    rec.put("elapsedMs", ms(elapsed));
    rec.put("messagesPerSecond", messages as f64 / elapsed.as_secs_f64().max(1e-9));
    rec.put("requests", calls.report());
    rec.put("counters", delta(before, &after, messages));
    let visible = wait_for(VISIBILITY_LIMIT, || {
        let probe = probe.clone();
        async move {
            for (filter, expected) in probe {
                if query_total(ctx, p, filter).await != Some(expected) {
                    return false;
                }
            }
            true
        }
    })
    .await;
    rec.check("list-reflects-change", visible.is_some(), json!({"probes":probe.iter().map(|(f,e)|json!({"filter":f,"expectedTotal":e})).collect::<Vec<_>>(),"limitSeconds":VISIBILITY_LIMIT.as_secs()}));
    rec.put("listCorrectAfterLastResponseMs", visible.map(ms));
    let idle = publication_idle(ctx).await;
    rec.check("index-publication-drained", idle.is_some(), json!({"status":ctx.engine.index_publication_status()}));
    rec.put("indexPublicationIdleAfterLastResponseMs", idle.map(ms));
    rec.put("changeRows", change_rows(ctx, p, states.0, states.1).await);
    rec.put("disk", disk(ctx).await);
}

async fn keywords_of(ctx: &Ctx, p: &Principal, ids: &[String]) -> std::result::Result<HashMap<String, Value>, Stop> {
    let mut out = HashMap::new();
    let mut stats = Calls::default();
    for chunk in ids.chunks(ctx.max_get) {
        let value = call(ctx, p, json!([["Email/get",{"accountId":p.account,"ids":chunk,"properties":["id","keywords","mailboxIds"]},"g"]]), &mut stats)
            .await
            .map_err(|r| refused("verification Email/get", r))?;
        for row in result(&value, 0)["list"].as_array().into_iter().flatten() {
            out.insert(row["id"].as_str().unwrap_or_default().to_string(), row.clone());
        }
    }
    Ok(out)
}
async fn not_found_count(ctx: &Ctx, p: &Principal, ids: &[String]) -> std::result::Result<usize, Stop> {
    let mut stats = Calls::default();
    let mut count = 0;
    for chunk in ids.chunks(ctx.max_get) {
        let value = call(ctx, p, json!([["Email/get",{"accountId":p.account,"ids":chunk,"properties":["id"]},"g"]]), &mut stats)
            .await
            .map_err(|r| refused("verification Email/get", r))?;
        count += result(&value, 0)["notFound"].as_array().map_or(0, Vec::len);
    }
    Ok(count)
}

async fn s_open(ctx: &Ctx, rec: &mut Record, kind: &str) -> std::result::Result<(), Stop> {
    let p = ctx.principal("main");
    let inbox = require_mailbox(ctx, p, "INBOX").await?;
    let (total, unread) = baseline(ctx, rec, p, "INBOX").await?;
    let (filter, sort, position, expected_total, newest_seed) = match kind {
        "first-page" => (json!({"inMailbox":inbox}), newest_first(), 0, total, total),
        "page-2" => (json!({"inMailbox":inbox}), newest_first(), PAGE, total, total - PAGE),
        "deep-page" => (json!({"inMailbox":inbox}), newest_first(), total / 2, total, total - total / 2),
        "unread" => (json!({"inMailbox":inbox,"notKeyword":"$seen"}), newest_first(), 0, unread, total),
        "sort-from" => (json!({"inMailbox":inbox}), json!([{"property":"from","isAscending":true}]), 0, total, 0),
        "text-search" => (json!({"inMailbox":inbox,"text":corpus::SEARCH_WORD}), newest_first(), 0, word_hits(1..=total as u32), total - total % 5),
        _ => unreachable!(),
    };
    let calls = list_calls(&p.account, filter.clone(), sort.clone(), position, &ctx.properties);
    let s0 = snapshot(&ctx.engine, Some(p));
    let mut cold = Calls::default();
    let first = call(ctx, p, calls.clone(), &mut cold).await.map_err(|r| refused("cold request", r))?;
    let s1 = snapshot(&ctx.engine, Some(p));
    let mut warm = Calls::default();
    let mut last = first.clone();
    for _ in 0..5 {
        last = call(ctx, p, calls.clone(), &mut warm).await.map_err(|r| refused("warm request", r))?;
    }
    let s2 = snapshot(&ctx.engine, Some(p));
    let query = result(&first, 0);
    let list = result(&first, 1)["list"].as_array().cloned().unwrap_or_default();
    rec.put("request", json!({"filter":filter,"sort":sort,"position":position,"limit":PAGE,"properties":ctx.properties.as_array().map_or(0,Vec::len)}));
    rec.put("coldMs", cold.latencies[0]);
    rec.put("coldNote", "first request of this shape in this process after startup readiness; the OS page cache was not dropped");
    rec.put("warm", warm.report());
    rec.put("coldCounters", delta(&s0, &s1, 0));
    rec.put("warmCountersFiveRequests", delta(&s1, &s2, 0));
    rec.put("returned", list.len());
    rec.put("total", &query["total"]);
    rec.put("mailboxMessages", total);
    let extra: serde_json::Map<String, Value> = query.as_object().into_iter().flatten().filter(|(k, _)| !matches!(k.as_str(), "ids" | "accountId")).map(|(k, v)| (k.clone(), v.clone())).collect();
    rec.put("queryResponse", extra);
    let want = PAGE.min(expected_total.saturating_sub(position)) as usize;
    rec.check("total", query["total"].as_u64() == Some(expected_total), json!({"expected":expected_total,"got":query["total"]}));
    rec.check("page-length", list.len() == want, json!({"expected":want,"got":list.len()}));
    rec.check("warm-equals-cold", result(&last, 0)["ids"] == query["ids"], json!({}));
    rec.check("rows-in-folder", list.iter().all(|row| row["mailboxIds"][&inbox] == true), json!({}));
    if kind == "sort-from" {
        let key = |row: &Value, field: &str| row["from"][0][field].as_str().unwrap_or_default().to_lowercase();
        let ordered = |field: &str| list.windows(2).all(|w| key(&w[0], field) <= key(&w[1], field));
        rec.check("sorted-by-from", ordered("email") || ordered("name"), json!({"first":list.first().map(|r|r["from"].clone())}));
    } else {
        let dates: Vec<&str> = list.iter().map(|row| row["receivedAt"].as_str().unwrap_or_default()).collect();
        rec.check("newest-first", dates.windows(2).all(|w| w[0] > w[1]), json!({}));
        rec.check("first-row", dates.first().copied() == Some(corpus::received_at(newest_seed as u32).as_str()) || want == 0,
            json!({"expected":corpus::received_at(newest_seed as u32),"got":dates.first()}));
    }
    Ok(())
}

async fn s_changes(ctx: &Ctx, rec: &mut Record) -> std::result::Result<(), Stop> {
    let p = ctx.principal("main");
    let inbox = require_mailbox(ctx, p, "INBOX").await?;
    let (total, _) = baseline(ctx, rec, p, "INBOX").await?;
    let mut setup = Calls::default();
    let state = call(ctx, p, json!([["Email/get",{"accountId":p.account,"ids":[],"properties":["id"]},"g"]]), &mut setup).await.map_err(|r| refused("setup state", r))?;
    let since = result(&state, 0)["state"].as_str().unwrap_or_default().to_string();
    let seed = 2_000_000 + total as u32;
    p.backend.deliver("INBOX", seed);
    let started = Instant::now();
    ctx.engine.refresh_mailbox(&p.account, Some(inbox.as_str())).await.map_err(|e| Stop::Failed(format!("refresh after arrival: {e}")))?;
    rec.put("arrivalSyncMs", ms(started.elapsed()));
    rec.put("arrivalNote", "engine refresh of the folder after one upstream arrival (what a watch event triggers)");
    let calls = json!([["Email/changes",{"accountId":p.account,"sinceState":since,"maxChanges":1000},"c"]]);
    let s0 = snapshot(&ctx.engine, Some(p));
    let mut cold = Calls::default();
    let first = call(ctx, p, calls.clone(), &mut cold).await.map_err(|r| refused("Email/changes", r))?;
    let s1 = snapshot(&ctx.engine, Some(p));
    let mut warm = Calls::default();
    for _ in 0..5 {
        call(ctx, p, calls.clone(), &mut warm).await.map_err(|r| refused("Email/changes", r))?;
    }
    rec.put("coldMs", cold.latencies[0]);
    rec.put("warm", warm.report());
    rec.put("coldCounters", delta(&s0, &s1, 0));
    rec.put("mailboxMessages", total);
    let changes = result(&first, 0);
    let created: Vec<String> = changes["created"].as_array().into_iter().flatten().filter_map(|v| v.as_str().map(str::to_owned)).collect();
    rec.check("one-created", created.len() == 1 && changes["destroyed"].as_array().is_some_and(Vec::is_empty), json!({"created":created.len(),"updated":changes["updated"],"destroyed":changes["destroyed"]}));
    rec.check("state-advanced", changes["newState"].as_str() != Some(since.as_str()), json!({"since":since,"newState":changes["newState"]}));
    if let Some(id) = created.first() {
        let got = call(ctx, p, json!([["Email/get",{"accountId":p.account,"ids":[id],"properties":["messageId"]},"g"]]), &mut setup).await.map_err(|r| refused("verification", r))?;
        let expected = corpus::message_id("main", seed);
        let expected = expected.trim_matches(|c| c == '<' || c == '>');
        rec.check("created-is-the-arrival", result(&got, 0)["list"][0]["messageId"][0].as_str() == Some(expected), json!({"expected":expected,"got":result(&got,0)["list"][0]["messageId"]}));
    }
    verify_folder(ctx, rec, p, "INBOX", total + 1, None).await;
    Ok(())
}

async fn s_flag(ctx: &Ctx, rec: &mut Record, keyword: &str, on: bool) -> std::result::Result<(), Stop> {
    let p = ctx.principal("main");
    let inbox = require_mailbox(ctx, p, "INBOX").await?;
    let (total, unread) = baseline(ctx, rec, p, "INBOX").await?;
    let k = u64::from(ctx.config.operation).min(total);
    let ids = top_ids(ctx, p, &inbox, k).await?;
    let before_rows = keywords_of(ctx, p, &ids).await?;
    let already = before_rows.values().filter(|row| row["keywords"][keyword] == true).count() as u64;
    let changing = if on { k - already } else { already };
    let flagged_before = p.backend.folder_counts("INBOX").map_or(0, |c| c.2 as u64);
    let (unread_after, flagged_after) = match (keyword, on) {
        ("$seen", true) => (unread - changing, flagged_before),
        ("$seen", false) => (unread + changing, flagged_before),
        (_, true) => (unread, flagged_before + changing),
        (_, false) => (unread, flagged_before - changing),
    };
    let states = states_now(ctx, p).await;
    rec.begin_op().await;
    let before = snapshot(&ctx.engine, Some(p));
    let started = Instant::now();
    let mut run = SetRun::default();
    let mut patch = serde_json::Map::new();
    patch.insert(format!("keywords/{keyword}"), if on { json!(true) } else { Value::Null });
    let outcome = bulk_email_set(ctx, p, &ids, &Value::Object(patch.clone()), false, &mut run).await;
    let elapsed = started.elapsed();
    rec.end_op();
    rec.put("patch", Value::Object(patch));
    rec.put("batchSize", ctx.batch());
    rec.put("mailboxMessages", total);
    rec.put("messagesChangingState", changing);
    outcome?;
    let probe = if keyword == "$seen" {
        vec![(json!({"inMailbox":inbox,"notKeyword":"$seen"}), unread_after)]
    } else {
        vec![(json!({"inMailbox":inbox,"hasKeyword":keyword}), flagged_after)]
    };
    finish_op(ctx, rec, p, k, elapsed, &run.calls, &before, states, probe).await;
    let body_per_message = rec.results["counters"]["searchIndex"]["bodyBytesTokenised"].as_u64().map(|b| b as f64 / k as f64);
    rec.put("bodyBytesTokenisedPerMessage", body_per_message);
    verify_folder(ctx, rec, p, "INBOX", total, Some(unread_after)).await;
    let rows = keywords_of(ctx, p, &ids).await?;
    let wrong = ids.iter().filter(|id| (rows.get(*id).map(|r| r["keywords"][keyword] == true)) != Some(on)).count();
    rec.check("every-message-has-the-new-keywords", wrong == 0, json!({"wrong":wrong}));
    let flagged = p.backend.folder_counts("INBOX").map_or(0, |c| c.2 as u64);
    rec.check("upstream-flagged", keyword == "$seen" || flagged == flagged_after, json!({"expected":flagged_after,"got":flagged}));
    rec.check("states-strictly-increase", states_increase(&run.states), json!({"states":run.states.len()}));
    verify_index(ctx, rec, p, &[]).await;
    Ok(())
}

async fn s_move(ctx: &Ctx, rec: &mut Record, role: &str, from: &str, to: &str) -> std::result::Result<(), Stop> {
    let p = ctx.principal(role);
    let (from_id, to_id) = (require_mailbox(ctx, p, from).await?, require_mailbox(ctx, p, to).await?);
    let (from_total, _) = baseline(ctx, rec, p, from).await?;
    let (to_total, _) = baseline(ctx, rec, p, to).await?;
    let k = u64::from(ctx.config.operation).min(from_total);
    let ids = top_ids(ctx, p, &from_id, k).await?;
    let states = states_now(ctx, p).await;
    rec.begin_op().await;
    let before = snapshot(&ctx.engine, Some(p));
    let started = Instant::now();
    let mut run = SetRun::default();
    let patch = json!({"mailboxIds":{&to_id:true}});
    let outcome = bulk_email_set(ctx, p, &ids, &patch, false, &mut run).await;
    let elapsed = started.elapsed();
    rec.end_op();
    rec.put("patch", json!({"mailboxIds":{"<destination>":true}}));
    rec.put("batchSize", ctx.batch());
    rec.put("sourceMessages", from_total);
    outcome?;
    let probe = vec![(json!({"inMailbox":to_id,"notKeyword":"$draft"}), to_total + k), (json!({"inMailbox":from_id,"notKeyword":"$draft"}), from_total - k)];
    finish_op(ctx, rec, p, k, elapsed, &run.calls, &before, states, probe).await;
    verify_folder(ctx, rec, p, from, from_total - k, None).await;
    verify_folder(ctx, rec, p, to, to_total + k, None).await;
    let rows = store_messages(ctx, &ids).await;
    let in_destination = rows.iter().filter(|row| row.as_ref().is_some_and(|m| m.mailbox_id == to_id)).count() as u64;
    rec.check("store-rows-in-destination", in_destination == k, json!({"expected":k,"got":in_destination}));
    let upstream = p.backend.folder_message_ids(to);
    let mut uids: Vec<u32> = rows.iter().flatten().map(|m| m.uid).collect();
    // Engine ingest and the store seed may spell Message-ID with or without <>.
    let bare = |id: &str| id.trim().trim_start_matches('<').trim_end_matches('>').to_string();
    let located = rows.iter().flatten().filter(|m| m.uidvalidity == 1 && upstream.get(&m.uid).map(|id| bare(id)) == m.message_id.as_deref().map(bare)).count() as u64;
    uids.sort_unstable();
    let unique = uids.windows(2).all(|w| w[0] < w[1]);
    let contiguous = uids.last().zip(uids.first()).is_some_and(|(hi, lo)| u64::from(hi - lo + 1) == k);
    rec.check("destination-uids-match-upstream", located == k && unique && contiguous, json!({"located":located,"unique":unique,"contiguous":contiguous,"range":[uids.first(),uids.last()]}));
    let mut stats = Calls::default();
    let listed: HashSet<String> = query_all(ctx, p, json!({"inMailbox":to_id}), to_total + k + 1, 1000, &mut stats).await.map_err(|r| refused("verification listing", r))?.into_iter().collect();
    rec.check("destination-lists-every-moved-message", ids.iter().all(|id| listed.contains(id)) && listed.len() as u64 == to_total + k, json!({"listed":listed.len()}));
    rec.check("states-strictly-increase", states_increase(&run.states), json!({"states":run.states.len()}));
    verify_index(ctx, rec, p, &[]).await;
    Ok(())
}

async fn s_mark_folder_read(ctx: &Ctx, rec: &mut Record) -> std::result::Result<(), Stop> {
    // The web client's folderBulk.ts: page the unread ids 100 at a time, then
    // mark them read 100 per Email/set.
    const CLIENT_BATCH: usize = 100;
    let p = ctx.principal("main");
    let inbox = require_mailbox(ctx, p, "INBOX").await?;
    let (total, unread) = baseline(ctx, rec, p, "INBOX").await?;
    let states = states_now(ctx, p).await;
    rec.begin_op().await;
    let before = snapshot(&ctx.engine, Some(p));
    let started = Instant::now();
    let mut query_calls = Calls::default();
    let mut ids = Vec::new();
    loop {
        let value = call(ctx, p, json!([["Email/query",{"accountId":p.account,"filter":{"inMailbox":inbox,"notKeyword":"$seen"},"position":ids.len(),"limit":CLIENT_BATCH},"q"]]), &mut query_calls)
            .await
            .map_err(|r| refused(&format!("Email/query page at position {}", ids.len()), r))?;
        let page: Vec<String> = result(&value, 0)["ids"].as_array().into_iter().flatten().filter_map(|v| v.as_str().map(str::to_owned)).collect();
        let short = page.len() < CLIENT_BATCH;
        ids.extend(page);
        if short || query_calls.requests >= 2000 {
            break;
        }
    }
    let listed = started.elapsed();
    let mut run = SetRun::default();
    let mut outcome = Ok(());
    for chunk in ids.chunks(CLIENT_BATCH) {
        let update: serde_json::Map<String, Value> = chunk.iter().map(|id| (id.clone(), json!({"keywords/$seen":true}))).collect();
        match call(ctx, p, json!([["Email/set",{"accountId":p.account,"update":update},"s"]]), &mut run.calls).await {
            Ok(value) => {
                let set = result(&value, 0);
                if let Some(new) = set["newState"].as_str() {
                    run.states.push(new.into());
                }
                if set["updated"].as_object().map_or(0, |m| m.len()) != chunk.len() {
                    outcome = Err(Stop::Refused { stage: format!("Email/set after {} of {}", run.done, ids.len()), refusal: json!({"kind":"partialAcceptance","notUpdated":set["notUpdated"]}) });
                    break;
                }
                run.done += chunk.len() as u64;
            }
            Err(r) => {
                outcome = Err(refused(&format!("Email/set after {} of {}", run.done, ids.len()), r));
                break;
            }
        }
    }
    let elapsed = started.elapsed();
    rec.end_op();
    rec.put("clientBatch", CLIENT_BATCH);
    rec.put("listingMs", ms(listed));
    rec.put("listingRequests", query_calls.report());
    rec.put("mailboxMessages", total);
    outcome?;
    let mut all = Calls::default();
    all.requests = query_calls.requests + run.calls.requests;
    all.latencies = query_calls.latencies.iter().chain(&run.calls.latencies).copied().collect();
    rec.put("setRequests", run.calls.report());
    let unique: HashSet<&String> = ids.iter().collect();
    rec.check("listed-every-unread-message-once", unique.len() as u64 == unread && ids.len() as u64 == unread, json!({"expected":unread,"listed":ids.len(),"unique":unique.len()}));
    finish_op(ctx, rec, p, unread, elapsed, &all, &before, states, vec![(json!({"inMailbox":inbox,"notKeyword":"$seen"}), 0)]).await;
    verify_folder(ctx, rec, p, "INBOX", total, Some(0)).await;
    rec.check("states-strictly-increase", states_increase(&run.states), json!({"states":run.states.len()}));
    verify_index(ctx, rec, p, &[]).await;
    Ok(())
}

async fn s_import(ctx: &Ctx, rec: &mut Record) -> std::result::Result<(), Stop> {
    let p = ctx.principal("main");
    let folder = require_mailbox(ctx, p, "Import").await?;
    let (existing, _) = baseline(ctx, rec, p, "Import").await?;
    let k = ctx.config.operation;
    let max_messages = ctx.inputs["limits"]["importMaxMessages"].as_u64().unwrap_or(20_000) as usize;
    const MAX_BODY: usize = 60 * 1024 * 1024; // below the route's 64 MiB body limit
    let to = format!("bulk-main@{DOMAIN}");
    let mut chunks: Vec<Vec<Vec<u8>>> = vec![Vec::new()];
    let mut bytes = 0;
    for seed in 3_000_001..=3_000_000 + k {
        let raw = corpus::raw("main", seed, &to);
        let last = chunks.last_mut().unwrap();
        if !last.is_empty() && (last.len() >= max_messages || bytes + raw.len() + 64 > MAX_BODY) {
            chunks.push(Vec::new());
            bytes = 0;
        }
        bytes += raw.len() + 64;
        chunks.last_mut().unwrap().push(raw);
    }
    let states = states_now(ctx, p).await;
    rec.begin_op().await;
    let before = snapshot(&ctx.engine, Some(p));
    let started = Instant::now();
    let mut calls = Calls::default();
    let mut done = 0u64;
    let mut outcome = Ok(());
    for chunk in &chunks {
        let begun = Instant::now();
        let response = ctx.driver.import_mbox(p, &folder, chunk).await;
        calls.requests += 1;
        calls.latencies.push(ms(begun.elapsed()));
        match response {
            Ok(value) if value["imported"].as_u64() == Some(chunk.len() as u64) => done += chunk.len() as u64,
            Ok(value) => {
                outcome = Err(Stop::Refused { stage: format!("import after {done} of {k}"), refusal: json!({"kind":"partialImport","response":value}) });
                break;
            }
            Err(r) => {
                outcome = Err(refused(&format!("import after {done} of {k}"), r));
                break;
            }
        }
    }
    let elapsed = started.elapsed();
    rec.end_op();
    rec.put("format", "mbox");
    rec.put("importRequests", chunks.len());
    rec.put("importMaxMessages", max_messages);
    outcome?;
    finish_op(ctx, rec, p, u64::from(k), elapsed, &calls, &before, states, vec![(json!({"inMailbox":folder,"notKeyword":"$draft"}), existing + u64::from(k))]).await;
    verify_folder(ctx, rec, p, "Import", existing + u64::from(k), None).await;
    let appended = rec.results["counters"]["upstream"]["append"].as_u64();
    rec.check("upstream-appends", appended == Some(u64::from(k)), json!({"expected":k,"got":appended}));
    verify_index(ctx, rec, p, &[]).await;
    Ok(())
}

async fn s_empty_trash(ctx: &Ctx, rec: &mut Record) -> std::result::Result<(), Stop> {
    let p = ctx.principal("trash");
    let trash = require_mailbox(ctx, p, "Trash").await?;
    let (total, _) = baseline(ctx, rec, p, "Trash").await?;
    let gone = mailbox_message_ids(ctx, &trash).await;
    let states = states_now(ctx, p).await;
    rec.begin_op().await;
    let before = snapshot(&ctx.engine, Some(p));
    let started = Instant::now();
    let mut calls = Calls::default();
    let args = |expected: u64, limit: u64| json!([["Mailbox/empty",{"accountId":p.account,"mailboxId":trash,"outcome":"destroy","expectedTotal":expected,"limit":limit},"e"]]);
    let preview = call(ctx, p, args(total, 0), &mut calls).await.map_err(|r| refused("Mailbox/empty preview", r))?;
    let mut remaining = result(&preview, 0)["remaining"].as_u64().unwrap_or(total);
    let mut processed = 0u64;
    let mut outcome = Ok(());
    while remaining > 0 && calls.requests < total / 50 + 20 {
        match call(ctx, p, args(remaining, 100), &mut calls).await {
            Ok(value) => {
                processed += result(&value, 0)["processed"].as_u64().unwrap_or(0);
                remaining = result(&value, 0)["remaining"].as_u64().unwrap_or(remaining);
            }
            Err(r) => {
                outcome = Err(refused(&format!("Mailbox/empty after {processed} of {total}"), r));
                break;
            }
        }
    }
    let elapsed = started.elapsed();
    rec.end_op();
    rec.put("clientBatch", 100);
    rec.put("previewOutcome", &result(&preview, 0)["outcome"]);
    outcome?;
    rec.check("emptied", remaining == 0 && processed == total, json!({"processed":processed,"remaining":remaining,"expected":total}));
    finish_op(ctx, rec, p, total, elapsed, &calls, &before, states, vec![(json!({"inMailbox":trash,"notKeyword":"$draft"}), 0)]).await;
    verify_folder(ctx, rec, p, "Trash", 0, None).await;
    let expunged = rec.results["counters"]["upstream"]["expungedMessages"].as_u64();
    rec.check("upstream-expunged", expunged == Some(total), json!({"expected":total,"got":expunged}));
    rec.check("removed-ids-not-found", not_found_count(ctx, p, &gone).await? == gone.len(), json!({"ids":gone.len()}));
    verify_index(ctx, rec, p, &gone).await;
    Ok(())
}

async fn s_destroy(ctx: &Ctx, rec: &mut Record) -> std::result::Result<(), Stop> {
    let p = ctx.principal("destroy");
    let inbox = require_mailbox(ctx, p, "INBOX").await?;
    let (total, _) = baseline(ctx, rec, p, "INBOX").await?;
    let k = u64::from(ctx.config.operation).min(total);
    let ids = top_ids(ctx, p, &inbox, k).await?;
    let states = states_now(ctx, p).await;
    rec.begin_op().await;
    let before = snapshot(&ctx.engine, Some(p));
    let started = Instant::now();
    let mut run = SetRun::default();
    let outcome = bulk_email_set(ctx, p, &ids, &Value::Null, true, &mut run).await;
    let elapsed = started.elapsed();
    rec.end_op();
    rec.put("request", "Email/set destroy with mailwomanPermanent (the web client's permanent delete)");
    rec.put("batchSize", ctx.batch());
    rec.put("mailboxMessages", total);
    outcome?;
    finish_op(ctx, rec, p, k, elapsed, &run.calls, &before, states, vec![(json!({"inMailbox":inbox,"notKeyword":"$draft"}), total - k)]).await;
    verify_folder(ctx, rec, p, "INBOX", total - k, None).await;
    let expunged = rec.results["counters"]["upstream"]["expungedMessages"].as_u64();
    rec.check("upstream-expunged", expunged == Some(k), json!({"expected":k,"got":expunged}));
    rec.check("removed-ids-not-found", not_found_count(ctx, p, &ids).await? == ids.len(), json!({"ids":ids.len()}));
    rec.check("states-strictly-increase", states_increase(&run.states), json!({"states":run.states.len()}));
    verify_index(ctx, rec, p, &ids).await;
    Ok(())
}

async fn s_delete_folder(ctx: &Ctx, rec: &mut Record) -> std::result::Result<(), Stop> {
    let p = ctx.principal("folder");
    let folder = require_mailbox(ctx, p, "Bulk").await?;
    let (total, _) = baseline(ctx, rec, p, "Bulk").await?;
    let gone = mailbox_message_ids(ctx, &folder).await;
    let docs_before = ctx.engine.search().account_doc_count(&p.account).unwrap_or(0);
    let states = states_now(ctx, p).await;
    rec.begin_op().await;
    let before = snapshot(&ctx.engine, Some(p));
    let started = Instant::now();
    let mut calls = Calls::default();
    let outcome = call(ctx, p, json!([["Mailbox/set",{"accountId":p.account,"destroy":[folder],"onDestroyRemoveEmails":true},"d"]]), &mut calls).await;
    let elapsed = started.elapsed();
    rec.end_op();
    rec.put("request", "Mailbox/set destroy with onDestroyRemoveEmails (the web client's folder delete)");
    let value = outcome.map_err(|r| refused("Mailbox/set destroy", r))?;
    if !result(&value, 0)["destroyed"].as_array().is_some_and(|d| d.iter().any(|id| id == &json!(folder))) {
        return Err(Stop::Refused { stage: "Mailbox/set destroy".into(), refusal: json!({"kind":"notDestroyed","response":result(&value,0)["notDestroyed"]}) });
    }
    let after = snapshot(&ctx.engine, Some(p));
    rec.put("messages", total);
    rec.put("elapsedMs", ms(elapsed));
    rec.put("messagesPerSecond", total as f64 / elapsed.as_secs_f64().max(1e-9));
    rec.put("requests", calls.report());
    rec.put("counters", delta(&before, &after, total));
    let docs_gone = wait_for(VISIBILITY_LIMIT, || async move { ctx.engine.search().account_doc_count(&p.account).ok() == Some(docs_before - total) }).await;
    rec.check("index-reflects-removal", docs_gone.is_some(), json!({"documentsBefore":docs_before,"removed":total}));
    rec.put("indexReflectsAfterLastResponseMs", docs_gone.map(ms));
    rec.put("indexPublicationIdleAfterLastResponseMs", publication_idle(ctx).await.map(ms));
    rec.put("changeRows", change_rows(ctx, p, states.0, states.1).await);
    rec.put("disk", disk(ctx).await);
    rec.check("folder-gone", !mailbox_ids(ctx, p).await.contains_key("Bulk") && p.backend.folder_counts("Bulk").is_none(), json!({}));
    let remaining = store_messages(ctx, &gone).await.iter().filter(|m| m.is_some()).count();
    rec.check("messages-gone", remaining == 0, json!({"remaining":remaining}));
    verify_index(ctx, rec, p, &gone).await;
    Ok(())
}

async fn s_ingest(ctx: &Ctx, rec: &mut Record) -> std::result::Result<(), Stop> {
    let p = ctx.principal("ingest");
    let n = u64::from(ctx.config.mailbox);
    let present = ctx.store.count_account_messages(&p.account).await.map_err(|e| Stop::Failed(e.to_string()))?;
    if !rec.check("precondition-empty-cache", present == 0, json!({"messages":present})) {
        return Err(Stop::Failed("ingest began before measurement".into()));
    }
    let listed = p.backend.list_mailboxes().await.map_err(|e| Stop::Failed(e.to_string()))?;
    let disk_before = disk(ctx).await;
    let states = states_now(ctx, p).await;
    rec.begin_op().await;
    let before = snapshot(&ctx.engine, Some(p));
    let started = Instant::now();
    // What engine-mode sign-in does once the upstream accepted the login.
    ctx.engine.cache_mailbox_list(&p.account, listed).await.map_err(|e| Stop::Failed(format!("cache_mailbox_list: {e}")))?;
    ctx.engine.start_initial_sync(&p.account);
    let (mut first_rows, mut all_rows, mut first_page, mut ready, mut indexed, mut fence) = (None, None, None, None, None, None);
    let mut first_page_latency = None;
    let mut polls = Calls::default();
    let mut retries = 0u32;
    let mut last_status = "";
    let hits = word_hits(1..=n as u32);
    let mut inbox = None;
    loop {
        let now = started.elapsed();
        if inbox.is_none() {
            inbox = mailbox_ids(ctx, p).await.get("INBOX").cloned();
        }
        let status = ctx.engine.initial_sync_status(&p.account);
        if status != last_status {
            if status == "retrying" {
                retries += 1;
            }
            last_status = status;
        }
        if status == "failed" {
            rec.put("initialSyncRetries", retries);
            return Err(Stop::Refused { stage: "initial synchronization".into(), refusal: json!({"kind":"initialSyncFailed","afterMs":ms(now),"rowsVisible":first_rows.is_some()}) });
        }
        if let Some(inbox) = &inbox {
            let rows = ctx.store.count_messages_in_mailbox(inbox).await.unwrap_or(0);
            if rows > 0 && first_rows.is_none() {
                first_rows = Some(now);
            }
            if rows == n && all_rows.is_none() {
                all_rows = Some(now);
            }
            if first_page.is_none() && rows > 0 {
                let begun = Instant::now();
                if let Ok(value) = call(ctx, p, list_calls(&p.account, json!({"inMailbox":inbox}), newest_first(), 0, &ctx.properties), &mut polls).await
                    && result(&value, 1)["list"].as_array().is_some_and(|l| l.len() as u64 == PAGE.min(n))
                {
                    first_page = Some(started.elapsed());
                    first_page_latency = Some(ms(begun.elapsed()));
                }
            }
        }
        if status == "ready" && all_rows.is_some() && ready.is_none() {
            ready = Some(now);
        }
        if indexed.is_none() && ctx.engine.search().account_doc_count(&p.account).ok() == Some(n) {
            indexed = Some(now);
        }
        if indexed.is_some() && fence.is_none()
            && let Some(inbox) = &inbox
            && query_total(ctx, p, json!({"inMailbox":inbox,"text":corpus::SEARCH_WORD})).await == Some(hits)
        {
            fence = Some(started.elapsed());
        }
        if all_rows.is_some() && ready.is_some() && indexed.is_some() && fence.is_some() && first_page.is_some() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let elapsed = started.elapsed();
    rec.end_op();
    let after = snapshot(&ctx.engine, Some(p));
    let all = all_rows.unwrap();
    rec.put("messages", n);
    rec.put("sizeDistribution", corpus::distribution());
    rec.put("timeToFirstRowsMs", first_rows.map(ms));
    rec.put("timeToFirstUsablePageMs", first_page.map(ms));
    rec.put("firstUsablePageRequestMs", first_page_latency);
    rec.put("allRowsVisibleMs", ms(all));
    rec.put("initialSyncReadyMs", ready.map(ms));
    rec.put("searchIndexCaughtUpMs", indexed.map(ms));
    rec.put("readFenceSatisfiedMs", fence.map(ms));
    rec.put("elapsedMs", ms(elapsed));
    rec.put("messagesPerSecond", n as f64 / all.as_secs_f64().max(1e-9));
    rec.put("indexedMessagesPerSecond", indexed.map(|d| n as f64 / d.as_secs_f64().max(1e-9)));
    rec.put("initialSyncRetries", retries);
    rec.put("pollRequests", polls.report());
    rec.put("pollingNote", "the store, index and first page are polled every 100 ms while the sync runs; the poll requests are part of the load");
    rec.put("counters", delta(&before, &after, n));
    rec.put("changeRows", change_rows(ctx, p, states.0, states.1).await);
    rec.put("diskBefore", disk_before);
    rec.put("disk", disk(ctx).await);
    let inbox = inbox.unwrap();
    verify_folder(ctx, rec, p, "INBOX", n, Some(n)).await;
    let fetched = rec.results["counters"]["upstream"]["fetchedMessages"].as_u64();
    rec.check("upstream-fetched-each-message-once", fetched == Some(n), json!({"expected":n,"got":fetched}));
    let mut stats = Calls::default();
    let page = call(ctx, p, list_calls(&p.account, json!({"inMailbox":inbox}), newest_first(), 0, &ctx.properties), &mut stats).await.map_err(|r| refused("verification", r))?;
    let first = result(&page, 1)["list"][0]["receivedAt"].clone();
    rec.check("first-page-is-newest", first.as_str() == Some(corpus::received_at(n as u32).as_str()), json!({"expected":corpus::received_at(n as u32),"got":first}));
    verify_index(ctx, rec, p, &[]).await;
    Ok(())
}

// ---------------------------------------------------------------- runner ---

macro_rules! scenario {
    ($ctx:expr, $name:expr, $mutating:expr, $f:path $(, $arg:expr)*) => {{
        let ctx: &Arc<Ctx> = $ctx;
        eprintln!("bulk-ops: scenario {} starting", $name);
        let mut rec = Record::new($name);
        reset_peak(ctx);
        let reader = if $mutating { start_reader(ctx, &mut rec).await } else { None };
        let outcome = tokio::time::timeout(ctx.config.scenario_deadline, $f(ctx, &mut rec $(, $arg)*)).await;
        finish_reader(reader, &mut rec).await;
        rec.put("memory", memory(ctx));
        let status = emit(ctx, rec, outcome);
        eprintln!("bulk-ops: scenario {} {status}", $name);
        status
    }};
}

async fn set_protocol_policy(store: &Store, allowed: &[&str]) {
    store
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: "access.protocols".into(),
            global_value: Some(serde_json::to_string(allowed).unwrap()),
            allow_user_override: false,
            global_locked: true,
            updated_by: "bulk-ops-fixture".into(),
            updated_at: chrono::Utc::now().timestamp(),
        })
        .await
        .unwrap();
}

/// SQLite in the owned directory, or a fresh schema in an acknowledged local
/// `mw_fixture*` PostgreSQL database (the native fixture's rules).
async fn isolated_database(directory: &Path) -> (String, Option<(String, String)>) {
    match std::env::var("MW_LOAD_FIXTURE_BACKEND").as_deref() {
        Err(_) | Ok("sqlite") => (directory.join("mailwoman.db").to_str().unwrap().to_owned(), None),
        Ok("postgres") => {
            assert_eq!(std::env::var("MW_LOAD_FIXTURE_PG_ACK_SCHEMA").as_deref(), Ok("1"), "explicit isolated PostgreSQL acknowledgement required");
            let raw = std::env::var("MW_LOAD_FIXTURE_PG_DSN").expect("local fixture DSN required");
            let mut url = reqwest::Url::parse(&raw).expect("fixture PostgreSQL URL");
            assert!(
                matches!(url.scheme(), "postgres" | "postgresql")
                    && url.host_str().is_some_and(|host| matches!(host, "127.0.0.1" | "localhost" | "::1" | "postgres" | "pg" | "mw-load-pg" | "host.docker.internal"))
                    && url.path().trim_start_matches('/').starts_with("mw_fixture"),
                "only acknowledged local mw_fixture databases are allowed"
            );
            assert!(!url.query_pairs().any(|(key, _)| matches!(key.as_ref(), "options" | "search_path" | "schema")), "fixture owns schema selection");
            let bootstrap = sqlx::postgres::PgPoolOptions::new().max_connections(1).connect(&raw).await.unwrap();
            let schema = format!("mw_bulk_ops_{}", uuid::Uuid::new_v4().simple());
            sqlx::query(&format!("CREATE SCHEMA {schema}")).execute(&bootstrap).await.unwrap();
            bootstrap.close().await;
            url.query_pairs_mut().append_pair("options", &format!("-csearch_path={schema}"));
            (url.to_string(), Some((raw, schema)))
        }
        _ => panic!("MW_LOAD_FIXTURE_BACKEND must be sqlite or postgres"),
    }
}

fn hardware() -> Value {
    match std::env::var("MW_BULK_HARDWARE_JSON") {
        Ok(raw) if raw.len() <= 8192 => serde_json::from_str::<Value>(&raw).ok().filter(Value::is_object).unwrap_or_else(|| json!({"error":"MW_BULK_HARDWARE_JSON is not a JSON object"})),
        Ok(_) => json!({"error":"MW_BULK_HARDWARE_JSON exceeds 8 KiB"}),
        Err(_) => json!({"supplied":false}),
    }
}

async fn run(config: Config) {
    let t34_selection = t34_modes::T34Selection::from_environment().unwrap_or_else(|e| panic!("invalid t34 mode: {e}"));
    let publication = t34_selection.publication_options().unwrap_or_else(|e| panic!("invalid index publication options: {e}"));
    let dir = PathBuf::from(std::env::var("MW_BULK_DIR").expect("worker directory"));
    // Sealed-index read bytes per query come from mw-search's diagnostic timing counters.
    mw_search::timing::set_enabled(true);
    let (db_path, pg) = isolated_database(&dir).await;
    let backend_kind: &'static str = if pg.is_some() { "postgres" } else { "sqlite" };
    let key = ServerKey::generate();
    let key_hex = key.to_hex();
    let store = Store::open(&db_path, key).await.unwrap();
    if config.profile == native_enforced_policy::Profile::Enforced {
        native_enforced_policy::install(&store).await;
    }
    set_protocol_policy(&store, &["imap"]).await;
    eprintln!("bulk-ops: seeding ({} per mailbox, {} per operation)", config.mailbox, config.operation);
    let seed_started = Instant::now();
    let seeded = seed(&store, &config).await;
    let seed_elapsed = seed_started.elapsed();
    let budgets = mw_server::settings_registry::effective_all(&store, &Default::default(), |def| def.key.starts_with("ratelimit.") || def.key.starts_with("security.protection.") || def.key == "access.protocols")
        .await
        .unwrap()
        .into_iter()
        .map(|row| (row.key, row.value))
        .collect::<serde_json::Map<String, Value>>();
    let import_max = mw_server::settings_registry::effective_for(&store, "import.maxMessages", &Default::default()).await.ok().flatten().map(|row| row.value);
    store.close().await;
    drop(store);

    let index_dir = dir.join(if pg.is_some() { "search-index" } else { "mailwoman.db.search-index" });
    std::fs::create_dir_all(&index_dir).unwrap();
    let measured_store = Arc::new(std::sync::OnceLock::<Store>::new());
    let measured_engine = Arc::new(std::sync::OnceLock::<Arc<Engine>>::new());
    let (installed_store, installed_engine) = (measured_store.clone(), measured_engine.clone());
    let runtimes: Vec<(String, Arc<BulkBackend>, String)> = seeded.principals.iter().map(|p| (p.account.clone(), p.backend.clone(), format!("bulk-{}@{DOMAIN}", p.role))).collect();
    let factory_index = index_dir.clone();
    let factory_publication = publication.clone();
    let index_open = Arc::new(Mutex::new(None::<Duration>));
    let factory_open = index_open.clone();
    let startup = Instant::now();
    let (router, _push) = build_app_with_fixture_engine(
        AppConfig { db_path: db_path.clone(), server_key_hex: Some(key_hex), web_dir: None, cookie_secure: false, mode: ServerMode::Engine, hardening: Default::default(), security: SecurityConfig::default() },
        V6Config::default(),
        move |store| {
            assert!(installed_store.set(store.clone()).is_ok());
            let opened = Instant::now();
            let engine = Arc::new(
                Engine::open_with_search_and_publication_options(store.clone(), &factory_index, mw_search::WriterOptions::default(), factory_publication).expect("sealed search index"),
            );
            *factory_open.lock().unwrap() = Some(opened.elapsed());
            for (account, backend, identity) in runtimes {
                engine.register_backend(account, AccountRuntime::new(backend, Arc::new(NoSend), &identity));
            }
            assert!(installed_engine.set(engine.clone()).is_ok());
            engine
        },
    )
    .await
    .unwrap();
    let store = measured_store.get().unwrap().clone();
    let engine = measured_engine.get().unwrap().clone();
    let journal = engine.start_index_publication().await.expect("start index publication");
    eprintln!("bulk-ops: index publication start: {journal:?}; waiting for {} index documents", seeded.expected_documents);
    let ready_started = Instant::now();
    let ready = tokio::time::timeout(config.ready_deadline, async {
        let mut next_log = Instant::now();
        loop {
            let observed = engine.search().num_docs();
            if Instant::now() >= next_log {
                eprintln!("bulk-ops: startup index {observed}/{} documents, {:.1}s", seeded.expected_documents, ready_started.elapsed().as_secs_f64());
                next_log = Instant::now() + Duration::from_secs(10);
            }
            if observed == seeded.expected_documents {
                let status = engine.search_index_status().await.expect("index status");
                if !status.rebuilding && status.documents == seeded.expected_documents && status.messages == seeded.expected_documents && status.journal_rows == 0 {
                    return status;
                }
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    })
    .await;
    let index_wait = ready_started.elapsed();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(listener, router.into_make_service_with_connect_info::<std::net::SocketAddr>()).await.unwrap();
    });
    let client = reqwest::Client::builder().no_proxy().timeout(Duration::from_secs(900)).build().unwrap();
    let first = &seeded.principals[0];
    let session: Value = client.get(format!("{base}/jmap/session")).header("cookie", &first.cookie).header("x-forwarded-for", &first.client).send().await.unwrap().json().await.expect("JMAP session JSON");
    let core = session["capabilities"]["urn:ietf:params:jmap:core"].clone();
    let max_set = core["maxObjectsInSet"].as_u64().expect("maxObjectsInSet advertised") as usize;
    let max_get = core["maxObjectsInGet"].as_u64().expect("maxObjectsInGet advertised") as usize;
    let enforced = if config.profile == native_enforced_policy::Profile::Enforced { native_enforced_policy::preflight(&store, &client, &base).await } else { json!({"profile":"defaults"}) };
    let encrypted = if pg.is_some() {
        json!({"database":"postgres","atRest":"not observable by the application"})
    } else {
        let mut header = [0u8; 16];
        use std::io::Read;
        std::fs::File::open(&db_path).unwrap().read_exact(&mut header).unwrap();
        assert_ne!(&header, mw_store::encryption::SQLITE_MAGIC, "the SQLite database must be SQLCipher-encrypted");
        json!({"database":"sqlite","sqlcipher":true,"verified":"file header is not the plaintext SQLite magic","mode":std::env::var("MW_DB_ENCRYPTION").ok()})
    };
    let contract: Value = serde_json::from_str(include_str!("../../../apps/web/src/api/native-read-intent-contract.json")).unwrap();
    let properties = contract["properties"].clone();
    let p = &publication;
    let inputs = json!({
        "sourceRevision":std::env::var("MW_BULK_SOURCE_REVISION").unwrap_or_else(|_|"unknown".into()),
        "sourceDirty":std::env::var("MW_BULK_SOURCE_DIRTY").ok(),
        "buildProfile":if cfg!(debug_assertions){"debug"}else{"release"},
        "backend":backend_kind,"encryption":encrypted,"searchIndex":"sealed-disk","searchTimingCounters":"enabled (mw_search::timing)",
        "config":config.summary(),
        "sizeDistribution":corpus::distribution(),
        "seedPath":"Store::upsert_message_bodies_batch (the native fixture's path); the ingest scenario uses the engine's initial sync",
        "limits":{"jmapCore":core,"mailboxEmptyMaxBatch":200,"importMaxMessages":import_max,"importBodyLimitBytes":64*1024*1024,
            "indexPublication":{"ack":p.ack.as_str(),"maxPendingJobs":p.max_pending_jobs,"maxPendingBytes":p.max_pending_bytes,"maxBatchJobs":p.max_batch_jobs,
                "maxBatchDocuments":p.max_batch_documents,"admissionTimeoutMs":p.admission_timeout.as_millis() as u64,"readFenceTimeoutMs":p.read_fence_timeout.as_millis() as u64,
                "maxPendingJobsPerAccount":p.max_pending_jobs_per_account},
            "requestProtection":budgets},
        "modes":{"t34":t34_modes::handoff(&t34_selection, t34_modes::witness(&engine)),"sqliteWriter":std::env::var("MW_SQLITE_WRITER").ok()},
        "securityPreflight":enforced,
        "hardware":hardware(),
    });
    let report = std::env::var_os("MW_BULK_REPORT").map(|path| Mutex::new(std::fs::OpenOptions::new().create(true).append(true).open(path).expect("report file")));
    let driver: Arc<dyn Driver> = match config.layer {
        Layer::Engine => Arc::new(EngineDriver { engine: engine.clone() }),
        Layer::Http => Arc::new(HttpDriver { client: client.clone(), base: base.clone() }),
    };
    let rss_peak = Arc::new(AtomicU64::new(0));
    let sampler = {
        let peak = rss_peak.clone();
        tokio::spawn(async move {
            loop {
                if let Some(resident) = current_rss() {
                    peak.fetch_max(resident, Ordering::SeqCst);
                }
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        })
    };
    let ctx = Arc::new(Ctx {
        config: config.clone(),
        driver,
        engine: engine.clone(),
        store: store.clone(),
        db_path: pg.is_none().then(|| PathBuf::from(&db_path)),
        index_dir,
        pg: pg.clone(),
        backend_kind,
        inputs,
        max_set,
        max_get,
        properties,
        principals: seeded.principals.clone(),
        report,
        rss_peak,
        http: client.clone(),
        base: base.clone(),
    });

    // The seed and startup index build are results in their own right.
    let mut statuses = Vec::new();
    {
        let mut rec = Record::new("seed");
        rec.put("seededMessages", seeded.seeded_messages);
        rec.put("accounts", seeded.principals.iter().map(|p| p.role).collect::<Vec<_>>());
        rec.put("seedMs", ms(seed_elapsed));
        rec.put("seedMessagesPerSecond", seeded.seeded_messages as f64 / seed_elapsed.as_secs_f64().max(1e-9));
        rec.put("serverStartupMs", ms(ready_started.duration_since(startup)));
        rec.put("startupIndexBuildMs", ms(index_wait));
        rec.put("startupIndexDocumentsPerSecond", seeded.expected_documents as f64 / index_wait.as_secs_f64().max(1e-9));
        rec.put("indexJournalStartup", format!("{journal:?}"));
        rec.put("searchIndexOpenMs", index_open.lock().unwrap().map(ms));
        rec.put("searchIndexOpenNote", "Engine construction with the sealed on-disk index, measured in the fixture factory");
        rec.put("disk", disk(&ctx).await);
        rec.put("memory", memory(&ctx));
        let outcome = match &ready {
            Ok(status) => {
                rec.put("indexStatus", json!({"documents":status.documents,"messages":status.messages,"rebuildDone":status.rebuild_done,"rebuildTotal":status.rebuild_total}));
                Ok(Ok(()))
            }
            Err(_) => Ok(Err(Stop::Refused { stage: "startup index build".into(), refusal: json!({"kind":"readinessDeadline","seconds":config.ready_deadline.as_secs(),"documents":engine.search().num_docs(),"expected":seeded.expected_documents}) })),
        };
        statuses.push(emit(&ctx, rec, outcome));
    }
    if ready.is_ok() {
        if config.wants("ingest") {
            statuses.push(scenario!(&ctx, "ingest", true, s_ingest));
        }
        if config.wants("open") {
            for kind in ["first-page", "page-2", "deep-page", "unread", "sort-from", "text-search"] {
                statuses.push(scenario!(&ctx, &format!("open-{kind}"), false, s_open, kind));
            }
            statuses.push(scenario!(&ctx, "open-changes", false, s_changes));
        }
        if config.wants("search") {
            match search_ops::prepare(&ctx).await {
                Ok(state) => {
                    for spec in search_ops::specs(config.mailbox) {
                        let name = spec.name.clone();
                        statuses.push(scenario!(&ctx, &name, false, search_ops::s_query, &spec, &state));
                    }
                    statuses.push(scenario!(&ctx, "search-as-you-type", false, search_ops::s_typing, &state));
                    statuses.push(scenario!(&ctx, "search-snippets", false, search_ops::s_snippets, &state));
                    statuses.push(scenario!(&ctx, "search-after-reader-reload", false, search_ops::s_after_reload, &state));
                    for change in ["arrival", "mark-read", "move-1000"] {
                        statuses.push(scenario!(&ctx, &format!("search-fresh-after-{change}"), true, search_ops::s_fresh, &state, change));
                    }
                    statuses.push(scenario!(&ctx, "search-index-rebuild", false, search_ops::s_rebuild, &state));
                }
                Err(error) => {
                    let mut rec = Record::new("search-setup");
                    rec.check("search-setup", false, json!({"error":error}));
                    statuses.push(emit(&ctx, rec, Ok(Ok(()))));
                }
            }
        }
        if config.wants("flag") {
            for (name, keyword, on) in [("flag-seen", "$seen", true), ("flag-unseen", "$seen", false), ("flag-flagged", "$flagged", true), ("flag-unflagged", "$flagged", false)] {
                statuses.push(scenario!(&ctx, name, true, s_flag, keyword, on));
            }
        }
        if config.wants("move") {
            statuses.push(scenario!(&ctx, "move", true, s_move, "main", "INBOX", "Archive"));
            statuses.push(scenario!(&ctx, "move-back", true, s_move, "main", "Archive", "INBOX"));
        }
        if config.wants("mark-folder-read") {
            statuses.push(scenario!(&ctx, "mark-folder-read", true, s_mark_folder_read));
        }
        if config.wants("import") {
            statuses.push(scenario!(&ctx, "import", true, s_import));
        }
        if config.wants("trash") {
            statuses.push(scenario!(&ctx, "trash-move", true, s_move, "trash", "INBOX", "Trash"));
            statuses.push(scenario!(&ctx, "empty-trash", true, s_empty_trash));
        }
        if config.wants("destroy") {
            statuses.push(scenario!(&ctx, "destroy", true, s_destroy));
        }
        if config.wants("delete-folder") {
            statuses.push(scenario!(&ctx, "delete-folder", true, s_delete_folder));
        }
    }
    sampler.abort();
    server.abort();
    let _ = server.await;
    drop(ctx);
    store.close().await;
    if let Some((raw, schema)) = pg {
        let bootstrap = sqlx::postgres::PgPoolOptions::new().max_connections(1).connect(&raw).await.unwrap();
        sqlx::query(&format!("DROP SCHEMA {schema} CASCADE")).execute(&bootstrap).await.unwrap();
        bootstrap.close().await;
    }
    eprintln!("bulk-ops: statuses {statuses:?}");
    assert!(!statuses.contains(&"FAILED"), "a scenario failed its end-state verification: {statuses:?}");
}

#[test]
#[ignore = "opt-in bulk mailbox operation benchmark; set MW_RUN_BULK_OPS_BENCH=1 (docs/perf/bulk-operations.md)"]
fn bulk_mailbox_operations_benchmark() {
    assert_eq!(std::env::var("MW_RUN_BULK_OPS_BENCH").as_deref(), Ok("1"), "set MW_RUN_BULK_OPS_BENCH=1");
    let config = Config::from_environment().unwrap_or_else(|e| panic!("invalid bulk-ops configuration: {e}"));
    if std::env::var(WORKER_ENV).as_deref() != Ok("1") {
        // A fresh worker process gets the proxy trust, the database key and the
        // upload directory; the parent's environment is not changed.
        let dir = std::env::temp_dir().join(format!("mw-bulk-ops-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        std::fs::create_dir(dir.join("uploads")).unwrap();
        let mut command = std::process::Command::new(std::env::current_exe().unwrap());
        command
            .args(["bulk_mailbox_operations_benchmark", "--exact", "--ignored", "--nocapture", "--test-threads=1"])
            .env(WORKER_ENV, "1")
            .env("MW_BULK_DIR", &dir)
            .env("MW_UPLOAD_DIR", dir.join("uploads"))
            .env("MW_FORWARDED_MODE", "xff")
            .env("MW_TRUSTED_PROXIES", "127.0.0.1/32");
        if !matches!(std::env::var("MW_LOAD_FIXTURE_BACKEND").as_deref(), Ok("postgres")) {
            let key = format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple());
            command.env("MW_DB_ENCRYPTION", "required").env("MW_DB_KEY", key);
        }
        let status = command.status().expect("start the bulk-ops worker");
        if std::env::var("MW_BULK_KEEP_DIR").as_deref() != Ok("1") {
            let _ = std::fs::remove_dir_all(&dir);
        }
        assert!(status.success(), "bulk-ops worker failed");
        return;
    }
    assert_eq!(std::env::var("MW_FORWARDED_MODE").as_deref(), Ok("xff"));
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(config.workers)
        .thread_stack_size(if cfg!(debug_assertions) { 16 * 1024 * 1024 } else { 4 * 1024 * 1024 })
        .enable_all()
        .build()
        .unwrap()
        .block_on(run(config));
}
