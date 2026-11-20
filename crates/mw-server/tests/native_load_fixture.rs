//! Opt-in native-engine HTTP campaign: distinct durable users, real mail rows,
//! production router/session checks, deterministic transport and no live accounts.
#![cfg(feature = "load-test-fixtures")]

use async_trait::async_trait;
use mw_engine::{Engine, MailSubmitter, account::AccountRuntime, backend::*};
use mw_server::{AppConfig, SecurityConfig, ServerMode, V6Config, build_app_with_fixture_engine};
use mw_store::{
    AccountKind, Credentials, MailboxUpsert, MessageUpsert, NewAccount, ServerKey, Store,
    identity_principal::VerifiedPrimary,
};
use serde_json::{Value, json};
use std::{
    net::{Ipv4Addr, SocketAddr},
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};

const MAX_PRINCIPALS: usize = 10_000;
const FIXTURE_WORKER_ENV: &str = "MW_NATIVE_LOAD_PROXY_WORKER";
#[path = "common/native_fixture_backend.rs"]
mod representative;

fn index_ready(status: &mw_engine::maintenance::SearchIndexStatus, expected: u64) -> bool {
    !status.rebuilding
        && status.documents == expected
        && status.messages == expected
        && status.zero_access_documents == 0
}

async fn startup_readiness(
    engine: &Engine,
    accounts: &[String],
    messages_per_account: u64,
    startup_started: Instant,
    timeout_seconds: u64,
) -> std::result::Result<Value, &'static str> {
    let waiting = Instant::now();
    tokio::time::timeout(Duration::from_secs(timeout_seconds),async {
        let known=accounts.iter().map(String::as_str).collect::<std::collections::HashSet<_>>();
        let excluded=engine.store().list_zeroaccess_enabled().await.map_err(|_|"Cannot verify index posture")?.iter().filter(|account|known.contains(account.as_str())).count() as u64*messages_per_account;
        let seeded=accounts.len() as u64*messages_per_account;
        let expected=seeded.checked_sub(excluded).ok_or("Invalid fixture index posture")?;
        let restrictions=engine.store().restriction_list(Some("native-load.invalid"),true,chrono::Utc::now().timestamp()).await.map_err(|_|"Cannot verify fixture index restrictions")?;
        if restrictions.iter().any(|restriction|known.contains(restriction.account_id.as_str())&&restriction.covers("indexing")){
            return Err("Indexing is paused by policy; warmed campaign not started");
        }
        let mut ticker=tokio::time::interval(Duration::from_millis(200));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut next_log=Instant::now();
        loop{
            ticker.tick().await;
            let observed=engine.search().num_docs();
            if Instant::now()>=next_log{
                eprintln!("native fixture: waiting for startup index readiness: {observed}/{expected} documents; {:.1}s elapsed",waiting.elapsed().as_secs_f64());
                next_log=Instant::now()+Duration::from_secs(5);
            }
            if observed==expected{
                // This full production observation enumerates accounts and can
                // be expensive. Never run it repeatedly during a partial copy.
                let status=engine.search_index_status().await.map_err(|_|"Cannot verify completed startup index")?;
                if index_ready(&status,expected){
                    eprintln!("native fixture: startup ready: {}/{} index documents; {:.1}s index wait",status.documents,expected,waiting.elapsed().as_secs_f64());
                    return Ok(json!({"verified":true,"state":if expected==0{"policy-excluded"}else{"ready"},"startupSeconds":startup_started.elapsed().as_secs_f64(),"indexWaitSeconds":waiting.elapsed().as_secs_f64(),"seededMessages":seeded,"expectedIndexDocuments":expected,"indexDocuments":status.documents,"zeroAccessExcludedMessages":excluded,"rebuildRunning":status.rebuilding,"rebuildDone":status.rebuild_done,"rebuildTotal":status.rebuild_total}));
                }
                if !status.rebuilding && status.rebuild_total>0 && status.documents!=status.messages{
                    return Err("Startup index stopped with missing documents; campaign not started");
                }
            }
        }
    }).await.map_err(|_|"Startup index readiness deadline exceeded; campaign not started")?
}

#[test]
fn readiness_never_accepts_incomplete_running_or_unexpected_index_documents() {
    use mw_engine::maintenance::SearchIndexStatus;
    let ready = SearchIndexStatus {
        documents: 200,
        messages: 200,
        ..Default::default()
    };
    assert!(index_ready(&ready, 200));
    assert!(!index_ready(&ready, 20000));
    assert!(!index_ready(
        &SearchIndexStatus {
            rebuilding: true,
            ..ready.clone()
        },
        200
    ));
    assert!(!index_ready(
        &SearchIndexStatus {
            zero_access_documents: 1,
            ..ready.clone()
        },
        200
    ));
    assert!(!index_ready(
        &SearchIndexStatus {
            messages: 201,
            ..ready
        },
        200
    ));
    assert!(index_ready(&SearchIndexStatus::default(), 0));
}

#[tokio::test]
async fn readiness_waits_for_real_engine_rebuild_and_blocks_paused_indexing() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let seeded = seed(&store, 1).await;
    let accounts = vec![seeded[0]["accountId"].as_str().unwrap().to_string()];
    let engine = Arc::new(Engine::new(store.clone()));
    assert!(
        startup_readiness(&engine, &accounts, 2, Instant::now(), 0)
            .await
            .unwrap_err()
            .contains("deadline exceeded"),
        "an index with no rebuild must not turn into a warmed readiness claim"
    );
    let rebuilding = engine.clone();
    let task = tokio::spawn(async move { rebuilding.rebuild_search_index_all().await.unwrap() });
    let report = startup_readiness(&engine, &accounts, 2, Instant::now(), 20)
        .await
        .unwrap();
    assert_eq!(report["verified"], true);
    assert_eq!(report["seededMessages"], 2);
    assert_eq!(report["indexDocuments"], 2);
    assert_eq!(task.await.unwrap().indexed, 2);
    store
        .restriction_apply(&mw_store::privacy::RestrictionRow {
            id: uuid::Uuid::new_v4().to_string(),
            account_id: accounts[0].clone(),
            address: "user0@native-load.invalid".into(),
            domain: "native-load.invalid".into(),
            scopes: vec!["indexing".into()],
            mail_delivery: "keep".into(),
            reason: "fixture".into(),
            source: "fixture".into(),
            request_id: None,
            created_by: "fixture".into(),
            created_at: chrono::Utc::now().timestamp(),
            expires_at: None,
            lifted_at: None,
            lifted_by: None,
            lift_reason: None,
        })
        .await
        .unwrap();
    assert!(
        startup_readiness(&engine, &accounts, 2, Instant::now(), 20)
            .await
            .unwrap_err()
            .contains("paused by policy")
    );
}

fn simulated_client(index: usize) -> Ipv4Addr {
    assert!(index < MAX_PRINCIPALS, "bounded fixture client index");
    Ipv4Addr::from(u32::from(Ipv4Addr::new(198, 18, 0, 1)) + index as u32)
}

#[test]
fn simulated_clients_are_distinct_and_only_the_fixture_proxy_can_assert_them() {
    use mw_protection::proxy::{ForwardedMode, ProxyConfig, resolve};
    let config = ProxyConfig::new(ForwardedMode::Xff, "127.0.0.1/32");
    let trusted = "127.0.0.1".parse().unwrap();
    let untrusted = "127.0.0.2".parse().unwrap();
    let mut clients = std::collections::HashSet::with_capacity(MAX_PRINCIPALS);
    for index in 0..MAX_PRINCIPALS {
        let address = simulated_client(index);
        assert_eq!(&address.octets()[..2], &[198, 18]);
        assert!(clients.insert(address));
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("x-forwarded-for", address.to_string().parse().unwrap());
        assert_eq!(resolve(&config, trusted, &headers), address);
        assert_eq!(resolve(&config, untrusted, &headers), untrusted);
    }
}

#[path = "common/request_load_diagnostics.rs"]
mod diagnostics_hook;
#[path = "common/request_load_control.rs"]
mod external_control;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FixturePushRequest {
    count: usize,
    interval_ms: u64,
}
impl FixturePushRequest {
    fn valid(&self, seeded: usize) -> bool {
        self.count > 0
            && self.count <= seeded
            && self.count <= 10000
            && (1..=1000).contains(&self.interval_ms)
            && self.count as u64 * self.interval_ms <= 600000
    }
}
#[test]
fn synthetic_push_bounds_never_accept_external_account_ids_or_unbounded_fanout() {
    assert!(
        FixturePushRequest {
            count: 10000,
            interval_ms: 10
        }
        .valid(10000)
    );
    for request in [
        FixturePushRequest {
            count: 0,
            interval_ms: 1,
        },
        FixturePushRequest {
            count: 101,
            interval_ms: 1,
        },
        FixturePushRequest {
            count: 1,
            interval_ms: 0,
        },
        FixturePushRequest {
            count: 10000,
            interval_ms: 1000,
        },
    ] {
        assert!(!request.valid(100));
    }
    assert!(
        serde_json::from_str::<FixturePushRequest>(
            r#"{"count":1,"intervalMs":1,"accountId":"external"}"#
        )
        .is_err()
    );
}

struct ReadOnlyBackend;
#[async_trait]
impl AccountBackend for ReadOnlyBackend {
    async fn capabilities(&self) -> Result<BackendCaps> {
        Ok(BackendCaps::default())
    }
    async fn list_mailboxes(&self) -> Result<Vec<RawMailbox>> {
        Ok(vec![RawMailbox {
            mailbox_ref: RawMailboxRef {
                name: "INBOX".into(),
                uidvalidity: 1,
            },
            role: MailboxRole::Inbox,
            parent: None,
            uidnext: 3,
            highestmodseq: 0,
            total: 2,
            unread: 2,
        }])
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
            "fixture serves seeded cached messages only".into(),
        ))
    }
    async fn store_flags(&self, _: &[MessageRef], _: &[Flag], _: &[Flag]) -> Result<()> {
        Err(EngineError::Unsupported("read-only fixture".into()))
    }
    async fn move_messages(&self, _: &[MessageRef], _: &RawMailboxRef) -> Result<MoveOutcome> {
        Err(EngineError::Unsupported("read-only fixture".into()))
    }
    async fn append(&self, _: &RawMailboxRef, _: &[u8], _: &[Flag]) -> Result<MessageRef> {
        Err(EngineError::Unsupported("read-only fixture".into()))
    }
    async fn watch(&self, _: ChangeSink) -> Result<WatchHandle> {
        Err(EngineError::Unsupported(
            "no fixture push simulation".into(),
        ))
    }
}
struct NoSend;
#[async_trait]
impl MailSubmitter for NoSend {
    async fn submit(&self, _: mw_smtp::Outgoing) -> Result<mw_smtp::SubmissionResult> {
        Err(EngineError::Unsupported("fixture never sends mail".into()))
    }
}

async fn seed(store: &Store, count: usize) -> Vec<Value> {
    seed_profile(store, count, 2, false, 26, 4).await.0
}
async fn seed_profile(
    store: &Store,
    count: usize,
    messages: u32,
    representative: bool,
    body_sample_bytes: usize,
    attachment_sample_bytes: usize,
) -> (
    Vec<Value>,
    std::collections::BTreeMap<String, Arc<representative::RepresentativeBackend>>,
) {
    let mut principals = Vec::with_capacity(count);
    let mut runtimes = std::collections::BTreeMap::new();
    let mut identities = std::collections::HashSet::with_capacity(count);
    let now = chrono::Utc::now().timestamp();
    for index in 0..count {
        let username = format!("user{index}@native-load.invalid");
        let credentials = Credentials {
            username: username.clone(),
            password: uuid::Uuid::new_v4().to_string(),
        };
        let account = store
            .create_account(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host: "127.0.0.1",
                    port: 9,
                    // This must be a real transport-policy value: startup's
                    // normal authority import rejects unknown strings and
                    // revokes sessions for accounts without configured routing.
                    // The read-only fixture backend never opens this endpoint.
                    tls: "starttls",
                    username: &username,
                    sync_policy_json: "{}",
                },
                &credentials,
            )
            .await
            .unwrap();
        let mailbox = store
            .upsert_mailbox(&MailboxUpsert {
                account_id: &account,
                name: "INBOX",
                role: Some("inbox"),
                uidvalidity: 1,
                uidnext: messages + 1,
                highestmodseq: 0,
                total: messages,
                unread: messages,
                parent_id: None,
            })
            .await
            .unwrap();
        let mut ids = Vec::new();
        let mut attachment_part: Option<String> = None;
        let archive = if representative {
            Some(
                store
                    .upsert_mailbox(&MailboxUpsert {
                        account_id: &account,
                        name: "Archive",
                        role: Some("archive"),
                        uidvalidity: 1,
                        uidnext: 1,
                        highestmodseq: 0,
                        total: 0,
                        unread: 0,
                        parent_id: None,
                    })
                    .await
                    .unwrap(),
            )
        } else {
            None
        };
        let mut raws = Vec::new();
        for uid in 1..=messages {
            let message_id = format!("<{index}-{uid}@native-load.invalid>");
            let raw = format!(
                "From: sender@native-load.invalid\r\nTo: {username}\r\nSubject: Native fixture {index}/{uid}\r\nMessage-ID: {message_id}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nIsolated seeded message.\r\n"
            );
            let raw = if representative && uid == 1 {
                use base64::Engine as _;
                let prefix = "Representative body sample.";
                let sample_body =
                    format!("{prefix}{}", "x".repeat(body_sample_bytes - prefix.len()));
                let attachment = (0..attachment_sample_bytes)
                    .map(|index| b"ABCD"[index % 4])
                    .collect::<Vec<_>>();
                let attachment = base64::engine::general_purpose::STANDARD.encode(attachment);
                format!(
                    "From: sender@native-load.invalid\r\nTo: {username}\r\nSubject: Native fixture {index}/{uid}\r\nMessage-ID: {message_id}\r\nMIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary=fixture\r\n\r\n--fixture\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n{sample_body}\r\n--fixture\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=sample.txt\r\nContent-Transfer-Encoding: base64\r\n\r\n{attachment}\r\n--fixture--\r\n"
                )
            } else {
                raw
            };
            let envelope = if representative {
                let mut email =
                    serde_json::to_value(mw_mime::parse(raw.as_bytes()).unwrap().email).unwrap();
                if uid == 1 {
                    attachment_part = email["attachments"][0]["partId"]
                        .as_str()
                        .map(str::to_owned);
                }
                email["_mailwomanAttachmentSchema"] = json!(1);
                serde_json::to_vec(&email).unwrap()
            } else {
                serde_json::to_vec(&json!({"subject":format!("Native fixture {index}/{uid}"),"from":[{"email":"sender@native-load.invalid"}],"to":[{"email":username}],"receivedAt":"2026-11-01T00:00:00Z","size":raw.len(),"hasAttachment":false})).unwrap()
            };
            if representative {
                raws.push(raw.as_bytes().to_vec());
            }
            ids.push(
                store
                    .upsert_message_with_body(
                        &MessageUpsert {
                            account_id: &account,
                            mailbox_id: &mailbox,
                            uid,
                            uidvalidity: 1,
                            message_id: Some(&message_id),
                            thread_id: None,
                            internaldate: Some("2026-11-01T00:00:00Z"),
                            size: raw.len() as u64,
                            flags_json: "[]",
                            envelope: Some(&envelope),
                            blob_ref: None,
                        },
                        raw.as_bytes(),
                    )
                    .await
                    .unwrap()
                    .0,
            );
        }
        let session = store
            .create_session(&account, &username, "engine", "engine", &credentials)
            .await
            .unwrap();
        store
            .set_session_authentication(&session, "password")
            .await
            .unwrap();
        let verified = store
            .bind_verified_primary(
                &session,
                VerifiedPrimary {
                    account_id: &account,
                    kind: "mailbox",
                    authority: "isolated-native-load",
                    subject: &account,
                },
                now,
            )
            .await
            .unwrap();
        let verified = verified.expect("fixture must create a real verified durable principal");
        assert!(
            identities.insert(verified.principal_id),
            "fixture principals must be distinct"
        );
        let csrf = store.session_csrf_token(&session);
        let attachment_blob = attachment_part
            .as_ref()
            .map(|part| format!("{}.{}", ids[0], part));
        principals.push(json!({"cookie":format!("mw_session={session}; mw_csrf={csrf}"),"csrfToken":csrf,"accountId":account,"ownerAccountId":account,"messageIds":ids,"inboxId":mailbox,"archiveId":archive,"attachmentBlobId":attachment_blob,"attachmentSampleBytes":attachment_sample_bytes,"bodySampleBytes":body_sample_bytes,"clientAddress":simulated_client(index).to_string()}));
        if representative {
            runtimes.insert(
                account,
                Arc::new(representative::RepresentativeBackend::new(raws)),
            );
        }
        if (index + 1) % 1000 == 0 {
            eprintln!("native fixture seeded {} distinct principals", index + 1);
        }
    }
    (principals, runtimes)
}

async fn set_protocol_policy(store: &Store, allowed: &[&str]) {
    store
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: "access.protocols".into(),
            global_value: Some(serde_json::to_string(allowed).unwrap()),
            allow_user_override: false,
            global_locked: true,
            updated_by: "native-load-fixture".into(),
            updated_at: chrono::Utc::now().timestamp(),
        })
        .await
        .unwrap();
}

async fn protocol_probe(
    client: &reqwest::Client,
    base: &str,
    principal: &Value,
) -> reqwest::Response {
    client.post(format!("{base}/jmap/api"))
        .header("cookie",principal["cookie"].as_str().unwrap())
        .header("x-csrf-token",principal["csrfToken"].as_str().unwrap())
        .header("x-forwarded-for",principal["clientAddress"].as_str().unwrap())
        .json(&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Email/query",{"accountId":principal["accountId"],"limit":2},"policy"]]}))
        .send().await.unwrap()
}

fn write_private(path: &Path, value: &Value) {
    use std::io::Write;
    let encoded = serde_json::to_vec(value).unwrap();
    assert!(
        encoded.len() as u64 <= external_control::HANDOFF_LIMIT,
        "private handoff exceeds bounded size"
    );
    let mut options = std::fs::OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).unwrap();
    file.write_all(&encoded).unwrap();
    file.sync_all().unwrap();
}
struct PrivateFile(std::path::PathBuf);
impl Drop for PrivateFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

async fn isolated_database(directory: &Path) -> (String, Option<(String, String)>) {
    match std::env::var("MW_LOAD_FIXTURE_BACKEND").as_deref() {
        Err(_) | Ok("sqlite") => (
            directory.join("mailwoman.db").to_str().unwrap().to_owned(),
            None,
        ),
        Ok("postgres") => {
            assert_eq!(
                std::env::var("MW_LOAD_FIXTURE_PG_ACK_SCHEMA").as_deref(),
                Ok("1"),
                "explicit isolated PostgreSQL acknowledgement required"
            );
            let raw = std::env::var("MW_LOAD_FIXTURE_PG_DSN").expect("local fixture DSN required");
            let mut url = reqwest::Url::parse(&raw).expect("fixture PostgreSQL URL");
            assert!(
                matches!(url.scheme(), "postgres" | "postgresql")
                    && url.host_str().is_some_and(|host| matches!(
                        host,
                        "127.0.0.1"
                            | "localhost"
                            | "::1"
                            | "postgres"
                            | "pg"
                            | "mw-load-pg"
                            | "host.docker.internal"
                    ))
                    && url.path().trim_start_matches('/').starts_with("mw_fixture"),
                "only acknowledged local mw_fixture databases are allowed"
            );
            assert!(
                !url.query_pairs()
                    .any(|(key, _)| matches!(key.as_ref(), "options" | "search_path" | "schema")),
                "fixture owns schema selection"
            );
            let bootstrap = sqlx::postgres::PgPoolOptions::new()
                .max_connections(1)
                .connect(&raw)
                .await
                .unwrap();
            let schema = format!("mw_native_load_{}", uuid::Uuid::new_v4().simple());
            sqlx::query(&format!("CREATE SCHEMA {schema}"))
                .execute(&bootstrap)
                .await
                .unwrap();
            bootstrap.close().await;
            url.query_pairs_mut()
                .append_pair("options", &format!("-csearch_path={schema}"));
            (url.to_string(), Some((raw, schema)))
        }
        _ => panic!("fixture backend must be sqlite/postgres"),
    }
}

#[test]
#[ignore = "opt-in native multi-principal real HTTP campaign; never a throughput assertion"]
fn native_multi_principal_http_load_fixture() {
    assert_eq!(
        std::env::var("MW_RUN_NATIVE_HTTP_LOAD_FIXTURE").as_deref(),
        Ok("1")
    );
    // Configure only this opt-in fixture worker before its process starts. Do not
    // mutate shared test-process environment or widen production proxy trust.
    if std::env::var(FIXTURE_WORKER_ENV).as_deref() != Ok("1") {
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "native_multi_principal_http_load_fixture",
                "--exact",
                "--ignored",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(FIXTURE_WORKER_ENV, "1")
            .env("MW_FORWARDED_MODE", "xff")
            .env("MW_TRUSTED_PROXIES", "127.0.0.1/32")
            .status()
            .expect("start isolated fixture proxy worker");
        assert!(status.success(), "isolated native fixture worker failed");
        return;
    }
    assert_eq!(std::env::var("MW_FORWARDED_MODE").as_deref(), Ok("xff"));
    assert_eq!(
        std::env::var("MW_TRUSTED_PROXIES").as_deref(),
        Ok("127.0.0.1/32")
    );
    let workers: usize = std::env::var("MW_LOAD_FIXTURE_WORKERS")
        .unwrap_or_else(|_| "4".into())
        .parse()
        .unwrap();
    assert!((1..=32).contains(&workers));
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(workers)
        // Native debug futures retain substantially larger stack frames than
        // optimized release handlers. Keep this diagnostic budget explicit.
        .thread_stack_size(worker_stack_bytes())
        .enable_all()
        .build()
        .unwrap()
        .block_on(run(workers));
}
fn worker_stack_bytes() -> usize {
    if cfg!(debug_assertions) {
        8 * 1024 * 1024
    } else {
        2 * 1024 * 1024
    }
}
async fn run(workers: usize) {
    let body_sample_bytes: usize = std::env::var("MW_NATIVE_LOAD_BODY_BYTES")
        .unwrap_or_else(|_| "4096".into())
        .parse()
        .unwrap();
    let attachment_sample_bytes: usize = std::env::var("MW_NATIVE_LOAD_ATTACHMENT_BYTES")
        .unwrap_or_else(|_| "65536".into())
        .parse()
        .unwrap();
    assert!(
        (27..=16384).contains(&body_sample_bytes),
        "representative body sample bounded27..16384 bytes"
    );
    assert!(
        (4..=262144).contains(&attachment_sample_bytes),
        "representative attachment bounded4..262144 bytes"
    );
    let workload = std::env::var("MW_NATIVE_LOAD_WORKLOAD").unwrap_or_else(|_| "baseline".into());
    assert!(
        matches!(workload.as_str(), "baseline" | "representative"),
        "choose baseline or representative workload"
    );
    let representative = workload == "representative";
    let messages: u32 = std::env::var("MW_NATIVE_LOAD_MESSAGES")
        .unwrap_or_else(|_| if representative { "50" } else { "2" }.into())
        .parse()
        .unwrap();
    assert!(
        (2..=100).contains(&messages) && (representative || messages == 2),
        "representative messages must be2..100; baseline stays2"
    );
    let policy_mode = std::env::var("MW_NATIVE_LOAD_POLICY").unwrap_or_else(|_| "defaults".into());
    assert!(
        matches!(policy_mode.as_str(), "defaults" | "loaded"),
        "choose defaults or loaded policy profile"
    );
    let count: usize = std::env::var("MW_NATIVE_LOAD_PRINCIPALS")
        .unwrap_or_else(|_| "100".into())
        .parse()
        .unwrap();
    assert!(
        [1, 100, 10000].contains(&count),
        "choose representative bounded profile 1, 100 or 10000"
    );
    assert!(
        count as u64 * messages as u64 <= 500000,
        "representative seeded messages capped at500000"
    );
    let deadline: u64 = std::env::var("MW_LOAD_FIXTURE_DEADLINE_SECONDS")
        .unwrap_or_else(|_| "90".into())
        .parse()
        .unwrap();
    assert!((10..=1800).contains(&deadline));
    let readiness_seconds: u64 = std::env::var("MW_NATIVE_LOAD_READY_SECONDS")
        .unwrap_or_else(|_| "300".into())
        .parse()
        .unwrap();
    assert!(
        (10..=1800).contains(&readiness_seconds),
        "startup readiness must be bounded to 10..1800 seconds"
    );
    let dir = std::env::temp_dir().join(format!("mw-native-load-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    let (db_path, pg_cleanup) = isolated_database(&dir).await;
    let backend = if pg_cleanup.is_some() {
        "postgres"
    } else {
        "sqlite"
    };
    let key = ServerKey::generate();
    let key_hex = key.to_hex();
    let store = Store::open(&db_path, key).await.unwrap();
    eprintln!("native fixture: seeding {count} independent users/clients");
    let seeding_started = Instant::now();
    let (principals, mut representative_runtimes) = seed_profile(
        &store,
        count,
        messages,
        representative,
        body_sample_bytes,
        attachment_sample_bytes,
    )
    .await;
    let seed_seconds = seeding_started.elapsed().as_secs_f64();
    if policy_mode == "loaded" {
        set_protocol_policy(&store, &["imap"]).await;
    }
    eprintln!("native fixture: seeding complete");
    let accounts = principals
        .iter()
        .map(|p| p["accountId"].as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    let budgets = mw_server::settings_registry::effective_all(&store, &Default::default(), |def| {
        def.key.starts_with("ratelimit.")
            || def.key.starts_with("security.protection.")
            || def.key == "access.protocols"
    })
    .await
    .unwrap()
    .into_iter()
    .map(|row| (row.key, row.value))
    .collect::<serde_json::Map<String, Value>>();
    // The seed connection pool must not distort measured pool occupancy/budget,
    // especially for PostgreSQL where distinct Store opens have distinct pools.
    store.close().await;
    drop(store);
    let measured_store = Arc::new(std::sync::OnceLock::<Store>::new());
    let installed_store = measured_store.clone();
    let measured_engine = Arc::new(std::sync::OnceLock::<Arc<Engine>>::new());
    let installed_engine = measured_engine.clone();
    let push_accounts = Arc::new(accounts.clone());
    let startup_started = Instant::now();
    let (router, push) = build_app_with_fixture_engine(
        AppConfig {
            db_path,
            server_key_hex: Some(key_hex),
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Engine,
            hardening: Default::default(),
            security: SecurityConfig::default(),
        },
        V6Config::default(),
        move |store| {
            assert!(installed_store.set(store.clone()).is_ok());
            let engine = Arc::new(Engine::new(store.clone()));
            for account in accounts {
                let backend: Arc<dyn AccountBackend> =
                    match representative_runtimes.remove(&account) {
                        Some(backend) => backend,
                        None => Arc::new(ReadOnlyBackend),
                    };
                engine.register_backend(
                    account,
                    AccountRuntime::new(backend, Arc::new(NoSend), "sender@native-load.invalid"),
                );
            }
            assert!(installed_engine.set(engine.clone()).is_ok());
            engine
        },
    )
    .await
    .unwrap();
    let store = measured_store.get().unwrap().clone();
    assert!(
        store
            .get_domain_config("native-load.invalid")
            .await
            .unwrap()
            .is_some(),
        "normal startup authority import must retain the fixture domain"
    );
    for principal in [&principals[0], principals.last().unwrap()] {
        let session = principal["cookie"]
            .as_str()
            .unwrap()
            .strip_prefix("mw_session=")
            .unwrap()
            .split(';')
            .next()
            .unwrap();
        let restored = store
            .get_primary_session(session)
            .await
            .expect("normal startup routing must preserve the seeded primary session");
        assert_eq!(
            restored.account_id,
            principal["accountId"].as_str().unwrap()
        );
    }
    let startup_readiness = startup_readiness(
        measured_engine.get().unwrap(),
        &push_accounts,
        messages as u64,
        startup_started,
        readiness_seconds,
    )
    .await
    .expect("native fixture startup readiness");
    let diagnostic_path = format!("/__fixture-diagnostic-{}", uuid::Uuid::new_v4());
    let diagnostic_token = uuid::Uuid::new_v4().to_string();
    let diag_token = diagnostic_token.clone();
    let reset_token = diagnostic_token.clone();
    let diag_store = store.clone();
    let diag_dir = dir.clone();
    let diag_engine = measured_engine.get().unwrap().clone();
    // Fixture-only routes mounted after production layers. The random token is
    // private handoff data, and snapshots contain no session/account identities.
    let router = router.route(
        &diagnostic_path,
        axum::routing::get(move |headers: axum::http::HeaderMap| {
            let token = diag_token.clone();
            let store = diag_store.clone();
            let directory = diag_dir.clone();
            let engine = diag_engine.clone();
            async move {
                if headers.get("x-fixture-token").and_then(|v| v.to_str().ok())
                    != Some(token.as_str())
                {
                    return (
                        axum::http::StatusCode::FORBIDDEN,
                        axum::Json(json!({"error":"denied"})),
                    );
                }
                let mut snapshot = diagnostics_hook::snapshot(&store, &directory).await;
                snapshot["searchIndex"] = json!({"documentFetches":engine.search().doc_fetch_count()});
                (
                    axum::http::StatusCode::OK,
                    axum::Json(snapshot),
                )
            }
        })
        .post(move |headers: axum::http::HeaderMap| {
            let token = reset_token.clone();
            async move {
                if headers.get("x-fixture-token").and_then(|v| v.to_str().ok())
                    != Some(token.as_str())
                {
                    return axum::http::StatusCode::FORBIDDEN;
                }
                diagnostics_hook::reset();
                axum::http::StatusCode::NO_CONTENT
            }
        }),
    );
    // Synthetic wakeups only: account identities come from this fixture's own
    // seed list. No mail is sent, inserted, flagged or otherwise mutated.
    let push_path = format!("/__fixture-push-{}", uuid::Uuid::new_v4());
    let push_token = uuid::Uuid::new_v4().to_string();
    let trigger_token = push_token.clone();
    let push_gate = Arc::new(tokio::sync::Semaphore::new(1));
    let router = router.route(
        &push_path,
        axum::routing::post(
            move |headers: axum::http::HeaderMap,
                  axum::Json(request): axum::Json<FixturePushRequest>| {
                let token = trigger_token.clone();
                let accounts = push_accounts.clone();
                let push = push.clone();
                let gate = push_gate.clone();
                async move {
                    if headers
                        .get("x-fixture-token")
                        .and_then(|value| value.to_str().ok())
                        != Some(token.as_str())
                    {
                        return (
                            axum::http::StatusCode::FORBIDDEN,
                            axum::Json(json!({"error":"denied"})),
                        );
                    }
                    if !request.valid(accounts.len()) {
                        return (
                            axum::http::StatusCode::BAD_REQUEST,
                            axum::Json(json!({"error":"bounds"})),
                        );
                    }
                    let Ok(_permit) = gate.try_acquire_owned() else {
                        return (
                            axum::http::StatusCode::CONFLICT,
                            axum::Json(json!({"error":"busy"})),
                        );
                    };
                    let marker = format!("fixture-{}", uuid::Uuid::new_v4());
                    for account in accounts.iter().take(request.count) {
                        push.send(mw_engine::StateChange {
                            account_id: account.clone(),
                            email: marker.clone(),
                            mailbox: "fixture-stable".into(),
                            submission: "fixture-stable".into(),
                            thread: "fixture-stable".into(),
                            crypto_key: "fixture-stable".into(),
                            mail_rule: "fixture-stable".into(),
                        });
                        tokio::time::sleep(Duration::from_millis(request.interval_ms)).await;
                    }
                    (
                        axum::http::StatusCode::OK,
                        axum::Json(json!({"marker":marker,"count":request.count})),
                    )
                }
            },
        )
        .layer(axum::extract::DefaultBodyLimit::max(256)),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    let base = format!("http://{address}");
    // Verify actual auth+account policy+mail engine before measuring. A successful
    // Core/echo alone would conceal an unregistered/invalid seeded mailbox.
    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();
    let policy_evidence = if policy_mode == "loaded" {
        // Prime actual authorization caches; the same sessions must immediately
        // observe policy revocation and restoration without a TTL wait.
        for p in [&principals[0], principals.last().unwrap()] {
            let response = protocol_probe(&client, &base, p).await;
            assert_eq!(response.status(), 200);
            let value: Value = response.json().await.unwrap();
            assert_eq!(value["methodResponses"][0][0], "Email/query");
            assert_eq!(value["methodResponses"][0][1]["accountId"], p["accountId"]);
        }
        set_protocol_policy(&store, &[]).await;
        for p in [&principals[0], principals.last().unwrap()] {
            let response = protocol_probe(&client, &base, p).await;
            assert_eq!(
                response.status(),
                403,
                "loaded policy revocation must deny immediately"
            );
            let value: Value = response.json().await.unwrap();
            assert_eq!(value["code"], "feature_disabled");
            assert_eq!(value["key"], "access.protocols");
            assert_eq!(value["feature"], "access.protocols");
        }
        set_protocol_policy(&store, &["imap"]).await;
        json!({"mode":"loaded","key":"access.protocols","allowedProtocols":["imap"],"locked":true,"primedRequests":2,"revokedRequests":2,"revokedStatus":403,"restoredRequests":2,"verified":true})
    } else {
        json!({"mode":"defaults","verified":false})
    };
    for p in [&principals[0], principals.last().unwrap()] {
        let response=client.post(format!("{base}/jmap/api")).header("cookie",p["cookie"].as_str().unwrap()).header("x-csrf-token",p["csrfToken"].as_str().unwrap()).header("x-forwarded-for", p["clientAddress"].as_str().unwrap()).json(&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Email/query",{"accountId":p["accountId"],"limit":2},"q"]]})).send().await.unwrap();
        assert_eq!(
            response.status(),
            200,
            "seeded principal must pass actual HTTP policies"
        );
        let body: Value = response.json().await.unwrap();
        assert_eq!(body["methodResponses"][0][0], "Email/query");
        assert_eq!(
            body["methodResponses"][0][1]["ids"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        // Production health admission must resolve this header as an external
        // benchmark client, rather than accidentally seeing the loopback peer.
        let health = client
            .get(format!("{base}/healthz"))
            .header("x-forwarded-for", p["clientAddress"].as_str().unwrap())
            .send()
            .await
            .unwrap();
        assert_eq!(
            health.status(),
            404,
            "production guard must observe simulated source"
        );
    }
    assert_eq!(
        client
            .get(format!("{base}/healthz"))
            .send()
            .await
            .unwrap()
            .status(),
        200,
        "the actual unforwarded loopback peer remains internal"
    );
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let report = std::env::var_os("MW_LOAD_REPORT_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join(".orchestration/reports/native-http-load"));
    std::fs::create_dir_all(&report).unwrap();
    let handoff = PrivateFile(dir.join("campaign.private.json"));
    let data = json!({"base":base,"cookie":principals[0]["cookie"],"engineMode":"native","fixtureFeatures":["load-test-fixtures"],"uniquePrincipalCount":count,"clientSimulation":{"mode":"fixture-loopback-trusted-proxy","clientCount":count},"principals":principals,"backend":backend,"buildProfile":if cfg!(debug_assertions){"debug"}else{option_env!("MW_CARGO_BUILD_MODE").unwrap_or("release")},"runtimeWorkerThreads":workers,"runtimeWorkerStackBytes":worker_stack_bytes(),"startupReadiness":startup_readiness,"protection":"administrator defaults","budgets":budgets,"diagnostics":{"path":diagnostic_path,"token":diagnostic_token},"pushTrigger":{"path":push_path,"token":push_token}});
    let mut data = data;
    data["policyEvidence"] = policy_evidence;
    data["workloadProfile"] = json!({"kind":workload,"messagesPerPrincipal":messages,"seededMessages":count as u64*messages as u64,"seedSeconds":seed_seconds,"transport":if representative {"deterministic-read-flag-move"} else {"read-only"},"sendsMeasured":false,"upstreamNetworkMeasured":false});
    if representative {
        data["workloadProfile"]["bodySampleBytes"] = json!(body_sample_bytes);
        data["workloadProfile"]["attachmentSampleBytes"] = json!(attachment_sample_bytes);
    }
    write_private(&handoff.0, &data);
    let outcome = if std::env::var("MW_LOAD_FIXTURE_EXTERNAL_DRIVER").as_deref() == Ok("1") {
        external_control::run(&data, &format!("native-{count}"), &report, deadline).await
    } else {
        let mut command = tokio::process::Command::new("node");
        command
            .arg(root.join("scripts/perf/http-real-fixture.mjs"))
            .arg(&handoff.0)
            .arg(report.join(format!("native-{count}.json")))
            .current_dir(root)
            .kill_on_drop(true);
        match tokio::time::timeout(Duration::from_secs(deadline + 10), command.output()).await {
            Ok(Ok(output)) if output.status.success() => Ok(()),
            Ok(Ok(_)) => {
                Err("native load generator returned failure; inspect redacted report".to_owned())
            }
            Ok(Err(_)) => {
                Err("native load generator could not start; Node must be installed".to_owned())
            }
            Err(_) => Err("native load generator exceeded bounded deadline".to_owned()),
        }
    };
    task.abort();
    let _ = task.await;
    store.close().await;
    if let Some((raw, schema)) = pg_cleanup {
        let bootstrap = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&raw)
            .await
            .unwrap();
        // Only the UUID schema created above is removed, never an existing schema.
        sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
            .execute(&bootstrap)
            .await
            .unwrap();
        bootstrap.close().await;
    }
    outcome.expect("native HTTP campaign failed");
}
