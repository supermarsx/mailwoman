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
#[path = "common/fixture_runtime.rs"]
mod fixture_runtime;
#[path = "common/linked_login_preflight.rs"]
mod linked_login_preflight;
#[path = "common/native_enforced_policy.rs"]
mod native_enforced_policy;
#[path = "common/t34_modes.rs"]
mod t34_modes;
#[path = "common/process_cpu.rs"]
mod process_cpu;
#[cfg(feature = "sql-shape-diagnostics")]
#[path = "common/native_sql_shapes.rs"]
mod native_sql_shapes;

fn index_ready(status: &mw_engine::maintenance::SearchIndexStatus, expected: u64) -> bool {
    !status.rebuilding
        && status.documents == expected
        && status.messages == expected
        && status.zero_access_documents == 0
        // t34: a journal-mode start must have replayed every leftover row.
        && status.journal_rows == 0
}

async fn startup_readiness(
    engine: &Engine,
    accounts: &[String],
    messages_of: &(dyn Fn(&str) -> u64 + Sync),
    startup_started: Instant,
    timeout_seconds: u64,
) -> std::result::Result<Value, &'static str> {
    // A zero deadline must not poll Store at all. Cancelling acquisition from a
    // one-connection in-memory SQLx pool can drop its only database connection.
    if timeout_seconds == 0 {
        return Err("Startup index readiness deadline exceeded; campaign not started");
    }
    let waiting = Instant::now();
    tokio::time::timeout(Duration::from_secs(timeout_seconds),async {
        let known=accounts.iter().map(String::as_str).collect::<std::collections::HashSet<_>>();
        let excluded=engine.store().list_zeroaccess_enabled().await.map_err(|_|"Cannot verify index posture")?.iter().filter(|account|known.contains(account.as_str())).map(|account|messages_of(account)).sum::<u64>();
        let seeded=accounts.iter().map(|account|messages_of(account)).sum::<u64>();
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
    let cold = engine.search_index_status().await.unwrap();
    assert_eq!(cold.documents, 0);
    assert!(!index_ready(&cold, 2), "a genuinely cold index is not ready");
    let (expired, queries) = mw_store::pool_telemetry::measure_queries(
        startup_readiness(&engine, &accounts, &|_| 2, Instant::now(), 0),
    ).await;
    assert!(
        expired.unwrap_err().contains("deadline exceeded"),
        "an expired readiness request must not turn into a warmed claim"
    );
    assert_eq!(queries.read.calls, 0, "zero deadline must not acquire a Store connection");
    assert_eq!(queries.write.calls, 0);
    assert!(store.list_zeroaccess_enabled().await.is_ok());
    let rebuilding = engine.clone();
    let task = tokio::spawn(async move { rebuilding.rebuild_search_index_all().await.unwrap() });
    let report = startup_readiness(&engine, &accounts, &|_| 2, Instant::now(), 20)
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
        startup_readiness(&engine, &accounts, &|_| 2, Instant::now(), 20)
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
    seed_profile(store, count, 2, false, 26, 4, false, false).await.0
}
// Validation chooses chunks BEFORE a write. A failed write is terminal and is
// never retried as smaller batches: its commit outcome may be uncertain.
async fn seed_message_batches(store:&Store,inputs:&[(MessageUpsert<'_>,&[u8])])->Vec<String> {
    let mut result=Vec::with_capacity(inputs.len());
    let mut batch=Vec::new();
    for input in inputs {
        batch.push(input.clone());
        if Store::validate_message_body_batch(&batch).is_err() {
            let next=batch.pop().unwrap();
            assert!(!batch.is_empty(),"one fixture row exceeds the checked Store batch admission bound");
            result.extend(store.upsert_message_bodies_batch(&batch).await.expect("fixture message batch write must succeed once").into_iter().map(|(id,existed)|{assert!(!existed,"fresh fixture coordinates must not overwrite messages");id}));
            batch.clear();batch.push(next);
            Store::validate_message_body_batch(&batch).expect("a fixture row must fit bounded Store admission before writes");
        }
    }
    if !batch.is_empty() {
        result.extend(store.upsert_message_bodies_batch(&batch).await.expect("fixture message batch write must succeed once").into_iter().map(|(id,existed)|{assert!(!existed,"fresh fixture coordinates must not overwrite messages");id}));
    }
    result
}

async fn seed_profile(
    store: &Store,
    count: usize,
    messages: u32,
    representative: bool,
    body_sample_bytes: usize,
    attachment_sample_bytes: usize,
    large_mailbox: bool,
    goal_mix: bool,
) -> (
    Vec<Value>,
    std::collections::BTreeMap<String, Arc<representative::RepresentativeBackend>>,
) {
    let mut principals = Vec::with_capacity(count);
    let mut runtimes = std::collections::BTreeMap::new();
    let mut identities = std::collections::HashSet::with_capacity(count);
    let now = chrono::Utc::now().timestamp();
    for index in 0..count {
        // t60: the large-mailbox profile sizes each user by cohort.
        let messages = if large_mailbox { large_mailbox_messages(index) } else { messages };
        let unread = if large_mailbox { (1..=messages).filter(|uid| !large_mailbox_flags(*uid).0).count() as u32 } else if goal_mix { messages - 1 } else { messages };
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
                unread,
                parent_id: None,
            })
            .await
            .unwrap();
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
        // t60: the goal-65-35 mix also needs a Trash and a Drafts folder (both
        // empty), for delete-to-trash and draft-save shapes.
        let mut goal_folders = serde_json::Map::new();
        if goal_mix {
            for (name, role) in [("Trash", "trash"), ("Drafts", "drafts")] {
                let id = store
                    .upsert_mailbox(&MailboxUpsert { account_id: &account, name, role: Some(role), uidvalidity: 1, uidnext: 1, highestmodseq: 0, total: 0, unread: 0, parent_id: None })
                    .await
                    .unwrap();
                goal_folders.insert(format!("{role}Id"), json!(id));
            }
        }
        // One principal only: the transport already retains these exact raw
        // messages. Store batching borrows them, rather than cloning all input
        // rows or allocating the whole campaign in advance.
        let mut raws = Vec::new();
        let mut envelopes = Vec::new();
        let mut message_ids = Vec::new();
        for uid in 1..=messages {
            let message_id = format!("<{index}-{uid}@native-load.invalid>");
            let planted = if large_mailbox || goal_mix { large_mailbox_terms(uid) } else { String::new() };
            let raw = format!(
                "From: sender@native-load.invalid\r\nTo: {username}\r\nSubject: Native fixture {index}/{uid}\r\nMessage-ID: {message_id}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nIsolated seeded message.\r\n{planted}"
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
                let parsed = mw_mime::parse(raw.as_bytes()).unwrap();
                let email = serde_json::to_value(&parsed.email).unwrap();
                if uid == 1 {
                    attachment_part = email["attachments"][0]["partId"]
                        .as_str()
                        .map(str::to_owned);
                }
                mw_engine::encode_message_envelope(&parsed.email, raw.as_bytes())
            } else {
                serde_json::to_vec(&json!({"subject":format!("Native fixture {index}/{uid}"),"from":[{"email":"sender@native-load.invalid"}],"to":[{"email":username}],"receivedAt":"2026-11-01T00:00:00Z","size":raw.len(),"hasAttachment":false})).unwrap()
            };
            raws.push(raw.into_bytes());
            envelopes.push(envelope);
            message_ids.push(message_id);
        }
        // goal-65-35: messageIds[2] (UID 3) starts read and messageIds[3] (UID 4)
        // flagged, so mark-unread-and-restore and unflag-and-restore change state
        // on their first pass too.
        let flag_rows:Vec<Vec<Flag>>=(1..=messages).map(|uid| if large_mailbox {let (seen,flagged)=large_mailbox_flags(uid);[(seen,Flag::Seen),(flagged,Flag::Flagged)].into_iter().filter(|(on,_)|*on).map(|(_,flag)|flag).collect()} else if goal_mix && uid == 3 {vec![Flag::Seen]} else if goal_mix && uid == 4 {vec![Flag::Flagged]} else {Vec::new()}).collect();
        let flag_json:Vec<String>=flag_rows.iter().map(|flags|serde_json::to_string(flags).unwrap()).collect();
        let inputs:Vec<_>=raws.iter().zip(&envelopes).zip(&message_ids).enumerate().map(|(index,((raw,envelope),message_id))| {
            (MessageUpsert {
                account_id:&account,mailbox_id:&mailbox,uid:index as u32+1,uidvalidity:1,
                message_id:Some(message_id),thread_id:None,
                internaldate:Some("2026-11-01T00:00:00Z"),size:raw.len() as u64,
                flags_json:&flag_json[index],envelope:Some(envelope),blob_ref:None,
            },raw.as_slice())
        }).collect();
        let ids=seed_message_batches(store,&inputs).await;
        drop(inputs);
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
        let cohort = if !large_mailbox { "default" } else { ["big-100000", "big-20000", "ordinary"][large_mailbox_cohort(index)] };
        principals.push(json!({"cohort":cohort,"messageCount":messages,"unreadCount":unread,"flaggedCount":flag_rows.iter().filter(|f|f.contains(&Flag::Flagged)).count(),"plantedQuarterly":if large_mailbox||goal_mix {(1..=messages).filter(|uid|uid%5==0).count()} else {0},"plantedZephyrine":if large_mailbox||goal_mix {(1..=messages).filter(|uid|uid%2000==7).count()} else {0},"cookie":format!("mw_session={session}; mw_csrf={csrf}"),"csrfToken":csrf,"accountId":account,"ownerAccountId":account,"messageIds":ids,"inboxId":mailbox,"archiveId":archive,"attachmentBlobId":attachment_blob,"attachmentSampleBytes":attachment_sample_bytes,"bodySampleBytes":body_sample_bytes,"clientAddress":simulated_client(index).to_string()}));
        if let Some(last) = principals.last_mut() {
            for (key, value) in goal_folders {
                last[key] = value;
            }
        }
        if representative {
            let backend = representative::RepresentativeBackend::with_flags(raws, flag_rows);
            let backend = if goal_mix { backend.with_folders(&["Trash", "Drafts"]) } else { backend };
            runtimes.insert(account, Arc::new(backend));
        }
        if (index + 1) % 1000 == 0 {
            eprintln!("native fixture seeded {} distinct principals", index + 1);
        }
    }
    (principals, runtimes)
}

#[tokio::test]
async fn bounded_batch_seed_keeps_distinct_principals_fifty_owned_messages_and_ready_index() {
    let store=Store::open_in_memory(ServerKey::generate()).await.unwrap();
    // Exercise the maximum permitted sample payloads, not a reduced workload.
    let (principals,runtimes)=seed_profile(&store,2,50,true,16384,262144,false,false).await;
    assert_eq!(principals.len(),2);assert_eq!(runtimes.len(),2);
    let mut passwords=std::collections::HashSet::new();
    let mut owned_ids=std::collections::HashSet::new();
    for principal in &principals {
        let account=principal["accountId"].as_str().unwrap();
        let cookie=principal["cookie"].as_str().unwrap();
        let session=cookie.strip_prefix("mw_session=").unwrap().split(';').next().unwrap();
        let source=store.get_primary_session(session).await.unwrap();
        assert_eq!(source.account_id,account);
        assert_eq!(source.username,store.get_account(account).await.unwrap().username);
        assert!(passwords.insert(source.credentials.password));
        let inbox=store.get_mailbox(principal["inboxId"].as_str().unwrap()).await.unwrap();
        assert_eq!((inbox.total,inbox.unread),(50,50));
        assert_eq!(store.get_mailbox(principal["archiveId"].as_str().unwrap()).await.unwrap().total,0);
        let ids=principal["messageIds"].as_array().unwrap();assert_eq!(ids.len(),50);
        for (index,id) in ids.iter().enumerate() {
            let id=id.as_str().unwrap();assert!(owned_ids.insert(id.to_owned()));
            let message=store.get_message(id).await.unwrap();
            assert_eq!(message.account_id,account);assert_eq!(message.mailbox_id,inbox.id);
            assert_eq!((message.uid,message.uidvalidity),(index as u32+1,1));assert_eq!(message.flags_json,"[]");
            let raw=store.get_body_limited(account,message.blob_ref.as_ref().unwrap(),1024*1024).await.unwrap().unwrap();
            assert_eq!(raw.len() as u64,message.size);
            let parsed=mw_mime::parse(&raw).unwrap();
            assert_eq!(parsed.email.has_attachment,index==0);
            assert!(store.get_account_header_envelope(account,id).await.unwrap().unwrap().projected);
        }
        assert_eq!(store.current_state(account,"Email").await.unwrap(),0);
        assert_eq!(store.current_state(account,"Mailbox").await.unwrap(),0);
    }
    let engine=Arc::new(Engine::new(store.clone()));
    assert_eq!(engine.rebuild_search_index_all().await.unwrap().indexed,100);
    for principal in &principals {
        let account=principal["accountId"].as_str().unwrap();
        assert_eq!(engine.search().account_doc_count(account).unwrap(),50);
        let indexed:std::collections::HashSet<_>=engine.search().account_stable_ids(account).unwrap().into_iter().collect();
        assert!(principal["messageIds"].as_array().unwrap().iter().all(|id|indexed.contains(id.as_str().unwrap())));
    }
    assert_eq!(owned_ids.len(),100);
}

// Fixture-only explicit admin-policy experiment; production defaults stay off.
fn prepared_mail_fixture_options(mode:&str,raw:Option<&str>)->Option<mw_engine::PreparedMailOptions>{
    assert!(matches!(mode,"off"|"on"),"choose prepared mail off or on");
    if mode=="off" {assert!(raw.is_none(),"prepared mail JSON requires explicit on");return None;}
    let value:mw_engine::PreparedMailOptions=match raw {
        Some(raw)=>{assert!(raw.len()<=4096,"prepared mail JSON bound");serde_json::from_str(raw).expect("strict prepared mail JSON")},
        None=>mw_engine::PreparedMailOptions {enabled:true,..Default::default()},
    };
    assert!(value.enabled&&value.valid(),"prepared mail on requires enabled valid bounded options");Some(value)
}

#[test]
fn prepared_mail_fixture_configuration_is_opt_in_strict_and_scales_explicitly(){
    assert!(prepared_mail_fixture_options("off",None).is_none());
    let default=prepared_mail_fixture_options("on",None).unwrap();assert!(default.enabled);assert_eq!(default.max_accounts,128);
    let large=r#"{"enabled":true,"budgetMiB":256,"maxEntries":10000,"maxAccounts":10000,"idleSeconds":30}"#;
    assert_eq!(prepared_mail_fixture_options("on",Some(large)).unwrap().max_accounts,10000);
    for (mode,raw) in [("unknown",None),("off",Some(large)),("on",Some("{}")),("on",Some(r#"{"enabled":true,"budgetMiB":16,"maxEntries":1,"maxAccounts":10001,"idleSeconds":30}"#))] {
        assert!(std::panic::catch_unwind(||prepared_mail_fixture_options(mode,raw)).is_err());
    }
}

fn prepared_mail_status_matches(status:&mw_engine::PreparedMailStatus,options:mw_engine::PreparedMailOptions)->bool {
    status.enabled==Some(true) && status.budget_bytes==Some(u64::from(options.budget_mib)*1024*1024)
        && status.max_entries==Some(options.max_entries) && status.max_accounts==Some(options.max_accounts)
        && status.idle_seconds==Some(options.idle_seconds)
}

// On-only legitimate startup HTTP, not a measured operation or direct cache setter.
async fn prime_prepared_mail(client:&reqwest::Client,base:&str,principal:&Value,engine:&Engine,options:mw_engine::PreparedMailOptions)->std::result::Result<Value,&'static str>{
    let contract:Value=serde_json::from_str(include_str!("../../../apps/web/src/api/native-read-intent-contract.json")).map_err(|_|"invalid canonical read contract")?;
    let ids=principal["messageIds"].as_array().filter(|ids|ids.len()>=2).ok_or("missing startup source IDs")?[..2].to_vec();
    let body=json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Email/get",{"accountId":principal["accountId"],"ids":ids,"properties":contract["properties"]},"prepared-startup"]]});
    let started=Instant::now();
    tokio::time::timeout(Duration::from_secs(10),async {
        for attempt in 1..=40u32 {
            let mut response=client.post(format!("{base}/jmap/api"))
                .header("cookie",principal["cookie"].as_str().ok_or("missing startup cookie")?)
                .header("x-csrf-token",principal["csrfToken"].as_str().ok_or("missing startup CSRF")?)
                .header("x-forwarded-for",principal["clientAddress"].as_str().ok_or("missing startup source")?)
                .header("origin",base)
                .header(contract["header"].as_str().ok_or("missing canonical marker")?,contract["value"].as_str().ok_or("missing canonical marker value")?)
                .json(&body).send().await.map_err(|_|"prepared startup HTTP failed")?;
            if response.status()!=200{return Err("prepared startup HTTP refused");}
            let mut bytes=Vec::new();
            while let Some(chunk)=response.chunk().await.map_err(|_|"prepared startup body failed")? {
                if bytes.len().checked_add(chunk.len()).is_none_or(|n|n>256*1024){return Err("prepared startup response bound");}bytes.extend_from_slice(&chunk);
            }
            let value:Value=serde_json::from_slice(&bytes).map_err(|_|"prepared startup JSON failed")?;
            let rows=value["methodResponses"][0][1]["list"].as_array().ok_or("prepared startup missing rows")?;
            if value["methodResponses"][0][0]!="Email/get" || value["methodResponses"][0][2]!="prepared-startup"
                || value["methodResponses"][0][1]["accountId"]!=principal["accountId"] || rows.len()!=2
                || !ids.iter().all(|id|rows.iter().any(|row|&row["id"]==id)) {return Err("prepared startup source mismatch");}
            if prepared_mail_status_matches(&engine.prepared_mail_status(),options) {
                return Ok(json!({"verified":true,"attempts":attempt,"elapsedMs":started.elapsed().as_millis() as u64,"primedPrincipals":1,"measuredRequests":0}));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Err("prepared startup configuration not applied")
    }).await.map_err(|_|"prepared startup deadline exceeded")?
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
    let workers = fixture_workers(std::env::var("MW_LOAD_FIXTURE_WORKERS").ok().as_deref())
        .unwrap_or_else(|error| panic!("{error}"));
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
/// Highest MW_LOAD_FIXTURE_WORKERS (tokio worker threads of the fixture
/// process). It was 32; 128 lets the fixture use a 56-CPU or larger cpuset.
const FIXTURE_WORKERS_MAX: usize = 128;
/// MW_LOAD_FIXTURE_WORKERS: unset means 4; otherwise 1..=FIXTURE_WORKERS_MAX.
fn fixture_workers(value: Option<&str>) -> std::result::Result<usize, String> {
    match value {
        None => Ok(4),
        Some(text) => match text.trim().parse::<usize>() {
            Ok(n) if (1..=FIXTURE_WORKERS_MAX).contains(&n) => Ok(n),
            _ => Err(format!(
                "MW_LOAD_FIXTURE_WORKERS must be a whole number from 1 to {FIXTURE_WORKERS_MAX}"
            )),
        },
    }
}
#[test]
fn fixture_worker_count_defaults_to_four_and_is_bounded() {
    assert_eq!(fixture_workers(None), Ok(4));
    for n in [1usize, 32, 33, 56, 128] {
        assert_eq!(fixture_workers(Some(&n.to_string())), Ok(n));
    }
    for bad in ["0", "129", "-1", "", "x", "4.0"] {
        assert!(fixture_workers(Some(bad)).is_err(), "{bad}");
    }
}
fn worker_stack_bytes() -> usize {
    if cfg!(debug_assertions) {
        8 * 1024 * 1024
    } else {
        2 * 1024 * 1024
    }
}
// t60: large-mailbox profile, the hot-path planner's shape: 1,000 users in
// cohorts (2 x 100,000, 8 x 20,000, 990 x 50 messages), 30% unread, 2% flagged,
// planted search terms. The default profile keeps its own bounds.
const LARGE_MAILBOX_PRINCIPALS: usize = 1_000;
const LARGE_MAILBOX_COHORTS: [(usize, u32); 3] = [(2, 100_000), (8, 20_000), (990, 50)];
/// Smoke shape (`MW_NATIVE_LOAD_LARGE_MAILBOX_SCALE=50`): big cohorts divided
/// by 50 and 90 ordinary users, 100 in total. Never a measurement profile.
const LARGE_MAILBOX_SMOKE_PRINCIPALS: usize = 100;
fn large_mailbox_cohorts() -> [(usize, u32); 3] {
    if large_mailbox_scale() == 1 { LARGE_MAILBOX_COHORTS } else { [(2, 100_000 / 50), (8, 20_000 / 50), (90, 50)] }
}
fn large_mailbox_principals() -> usize {
    if large_mailbox_scale() == 1 { LARGE_MAILBOX_PRINCIPALS } else { LARGE_MAILBOX_SMOKE_PRINCIPALS }
}
fn large_mailbox_cohort(index: usize) -> usize {
    let mut start = 0;
    for (cohort, (count, _)) in large_mailbox_cohorts().iter().enumerate() {
        if index < start + count {
            return cohort;
        }
        start += count;
    }
    panic!("large-mailbox profile has {} users", large_mailbox_principals())
}
/// Smoke-only divisor for the big cohorts (`MW_NATIVE_LOAD_LARGE_MAILBOX_SCALE`
/// 1 or 50; ordinary users keep 50 messages). Recorded in the handoff.
fn large_mailbox_scale() -> u32 {
    match std::env::var("MW_NATIVE_LOAD_LARGE_MAILBOX_SCALE").as_deref() {
        Err(_) | Ok("1") => 1,
        Ok("50") => 50,
        Ok(other) => panic!("MW_NATIVE_LOAD_LARGE_MAILBOX_SCALE={other:?} must be 1 or 50"),
    }
}
fn large_mailbox_messages(index: usize) -> u32 {
    large_mailbox_cohorts()[large_mailbox_cohort(index)].1
}
/// `(seen, flagged)`: UIDs ending 0-2 unread (30%), every 50th flagged (2%).
fn large_mailbox_flags(uid: u32) -> (bool, bool) {
    (uid % 10 >= 3, uid % 50 == 0)
}
/// Planted words with exact frequencies: `quarterly` on every 5th message,
/// `zephyrine` on every 2000th (UID % 2000 == 7).
fn large_mailbox_terms(uid: u32) -> String {
    let mut out = String::new();
    if uid % 5 == 0 { out.push_str("The quarterly figures follow.\r\n"); }
    if uid % 2000 == 7 { out.push_str("Reference zephyrine attached.\r\n"); }
    out
}
fn large_mailbox_profile(raw: Option<&str>) -> std::result::Result<bool, String> {
    match raw {
        None | Some("default") => Ok(false),
        Some("large-mailbox") => Ok(true),
        Some(_) => Err("MW_NATIVE_LOAD_PROFILE must be default or large-mailbox".into()),
    }
}

#[test]
fn large_mailbox_profile_is_explicit_and_keeps_default_bounds() {
    assert!(!large_mailbox_profile(None).unwrap());
    assert!(!large_mailbox_profile(Some("default")).unwrap());
    assert!(large_mailbox_profile(Some("large-mailbox")).unwrap());
    for raw in ["", "large", "Large-mailbox", " large-mailbox"] { assert!(large_mailbox_profile(Some(raw)).is_err()); }
    assert_eq!(LARGE_MAILBOX_COHORTS.iter().map(|(count, _)| count).sum::<usize>(), LARGE_MAILBOX_PRINCIPALS);
    let total: u64 = (0..LARGE_MAILBOX_PRINCIPALS).map(|i| u64::from(large_mailbox_messages(i))).sum();
    assert_eq!(total, 409_500);
    assert!(total <= 500_000);
    assert_eq!([large_mailbox_messages(0), large_mailbox_messages(1), large_mailbox_messages(2), large_mailbox_messages(9), large_mailbox_messages(10), large_mailbox_messages(999)], [100_000, 100_000, 20_000, 20_000, 50, 50]);
    assert_eq!((1..=1000).filter(|u| !large_mailbox_flags(*u).0).count(), 300);
    assert_eq!((1..=1000).filter(|u| large_mailbox_flags(*u).1).count(), 20);
}

fn fixture_index_timing(raw: Option<&str>) -> std::result::Result<bool, String> {
    match raw {
        None | Some("0") => Ok(false),
        Some("1") => Ok(true),
        Some(_) => Err("MW_NATIVE_LOAD_INDEX_TIMING must be 0 or 1".into()),
    }
}

#[test]
fn index_timing_switch_is_explicit_and_bounded() {
    assert!(!fixture_index_timing(None).unwrap());
    assert!(!fixture_index_timing(Some("0")).unwrap());
    assert!(fixture_index_timing(Some("1")).unwrap());
    for raw in ["", "true", "yes", "2", "01", " 1"] { assert!(fixture_index_timing(Some(raw)).is_err()); }
}

fn fixture_writer_options(raw: Option<&str>) -> std::result::Result<mw_search::WriterOptions, String> {
    let indexing_workers = match raw {
        None => None,
        Some(value) if value.len()==1 && matches!(value.as_bytes()[0],b'1'..=b'8') => Some(value.parse().unwrap()),
        Some(_) => return Err("MW_NATIVE_LOAD_INDEX_WORKERS must be a single integer 1..8".into()),
    };
    let options=mw_search::WriterOptions {indexing_workers,..Default::default()};
    options.validate().map_err(|error|error.to_string())?;
    Ok(options)
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum FixtureIndexMode { Sealed, Memory }
fn fixture_index_mode(raw:Option<&str>)->std::result::Result<FixtureIndexMode,String>{
    match raw {
        None|Some("sealed")=>Ok(FixtureIndexMode::Sealed),
        Some("memory")=>Ok(FixtureIndexMode::Memory),
        Some(_)=>Err("MW_NATIVE_LOAD_INDEX_MODE must be sealed or memory".into()),
    }
}
fn fixture_storage_proof(engine:&Engine,mode:FixtureIndexMode)->Value{
    match mode {
        FixtureIndexMode::Sealed=>{
            use std::io::Read;
            let dir=engine.search_index_dir().expect("sealed engine retains its index directory");
            let mut marker=[0u8;4];
            std::fs::File::open(dir.join("meta.json")).unwrap().read_exact(&mut marker).unwrap();
            assert_eq!(&marker,b"MWB1","sealed index metadata must be authenticated ciphertext");
            json!({"mode":"sealed-disk","encryptedAtRest":true,"verified":true,"proof":"sealed-constructor-and-encrypted-metadata"})
        }
        FixtureIndexMode::Memory=>{
            assert!(engine.search_index_dir().is_none());
            json!({"mode":"memory","encryptedAtRest":false,"verified":true,"proof":"explicit-in-memory-constructor"})
        }
    }
}
#[test]
fn diagnostic_index_storage_defaults_sealed_and_rejects_unknown_modes(){
    assert_eq!(fixture_index_mode(None).unwrap(),FixtureIndexMode::Sealed);
    assert_eq!(fixture_index_mode(Some("sealed")).unwrap(),FixtureIndexMode::Sealed);
    assert_eq!(fixture_index_mode(Some("memory")).unwrap(),FixtureIndexMode::Memory);
    for mode in ["","disk","plain","Memory"," sealed","sealed "]{assert!(fixture_index_mode(Some(mode)).is_err());}
}
#[test]
fn diagnostic_index_worker_override_is_strict_and_memory_bounded() {
    assert_eq!(fixture_writer_options(None).unwrap().indexing_workers,None);
    for value in ["1","2","3"] {
        assert_eq!(fixture_writer_options(Some(value)).unwrap().indexing_workers,Some(value.parse().unwrap()));
    }
    for value in ["","0","4","8","9","01","+1"," 1","1 ","many"] {
        assert!(fixture_writer_options(Some(value)).is_err(),"{value}");
    }
}
fn fixture_cache_experiment(raw: Option<&str>) -> std::result::Result<Option<Value>, &'static str> {
    let Some(raw)=raw else{return Ok(None)};
    let requested=if raw=="null" {None} else {
        let valid= !raw.is_empty() && raw.bytes().all(|b|b.is_ascii_digit()) && !raw.starts_with('0');
        if !valid {return Err("invalid SQLite cache experiment expectation")}
        let value=raw.parse::<u32>().map_err(|_|"invalid SQLite cache experiment expectation")?;
        if !(16..=1024).contains(&value) {return Err("invalid SQLite cache experiment expectation")}
        Some(value)
    };
    Ok(Some(json!({"kind":"sqlite-page-cache","requestedAggregateMiB":requested})))
}
#[test]
fn cache_experiment_expectation_is_explicit_and_bounded() {
    assert_eq!(fixture_cache_experiment(None).unwrap(),None);
    assert_eq!(fixture_cache_experiment(Some("null")).unwrap().unwrap()["requestedAggregateMiB"],Value::Null);
    assert_eq!(fixture_cache_experiment(Some("256")).unwrap().unwrap()["requestedAggregateMiB"],256);
    for raw in ["","NULL","0","15","1025","0256","256.0","+256"," 256"] {assert!(fixture_cache_experiment(Some(raw)).is_err());}
}
async fn run(workers: usize) {
    mw_server::request_inputs_fixture::initialize_from_environment()
        .expect("valid immutable ordinary request-input fixture mode before all fixture effects");
    mw_server::freeze_environment(); // t34-e6: per-request env scans become one snapshot (5be7dde6)
    // t34 modes: every selector is validated before any database creation or seeding.
    let t34_selection = t34_modes::T34Selection::from_environment().unwrap_or_else(|error| panic!("invalid t34 fixture mode: {error}"));
    let t34_publication = t34_selection.publication_options().unwrap_or_else(|error| panic!("invalid t34 index publication options: {error}"));
    let cache_expectation_env=std::env::var_os("MW_LOAD_FIXTURE_SQLITE_CACHE_EXPECT")
        .map(|value|value.into_string().unwrap_or_else(|_|panic!("cache experiment expectation must be Unicode")));
    let cache_experiment=fixture_cache_experiment(cache_expectation_env.as_deref()).expect("valid cache experiment expectation");
    let audit_cache=cache_experiment.is_some();
    // Validate diagnostic resources before any database creation or seeding.
    let index_timing_env = std::env::var_os("MW_NATIVE_LOAD_INDEX_TIMING")
        .map(|value| value.into_string().expect("index timing switch must be Unicode"));
    let index_timing = fixture_index_timing(index_timing_env.as_deref()).expect("valid diagnostic index timing switch");
    mw_search::timing::set_enabled(index_timing);
    let index_worker_env=std::env::var_os("MW_NATIVE_LOAD_INDEX_WORKERS")
        .map(|value|value.into_string().expect("index workers must be Unicode"));
    let writer_options=fixture_writer_options(index_worker_env.as_deref()).expect("valid diagnostic index writer options");
    let index_mode_env=std::env::var_os("MW_NATIVE_LOAD_INDEX_MODE")
        .map(|value|value.into_string().expect("index mode must be Unicode"));
    let index_mode=fixture_index_mode(index_mode_env.as_deref()).expect("valid diagnostic index storage mode");
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
        matches!(workload.as_str(), "baseline" | "representative" | "goal-65-35"),
        "choose baseline or representative workload"
    );
    // t60: goal-65-35 seeds as representative does, plus Trash/Drafts and planted terms.
    let goal_mix = workload == "goal-65-35";
    let representative = workload == "representative" || goal_mix;
    // t60: separate large-mailbox profile with its own bounds; the default
    // profile keeps 2..100 messages per user and the 500,000 total below.
    let large_mailbox = large_mailbox_profile(std::env::var("MW_NATIVE_LOAD_PROFILE").ok().as_deref())
        .expect("valid native load profile");
    let messages: u32 = std::env::var("MW_NATIVE_LOAD_MESSAGES")
        .unwrap_or_else(|_| if representative { "50" } else { "2" }.into())
        .parse()
        .unwrap();
    if large_mailbox {
        assert!(representative, "the large-mailbox profile requires the representative workload");
        assert!(std::env::var_os("MW_NATIVE_LOAD_MESSAGES").is_none(), "the large-mailbox profile sizes users by cohort; leave MW_NATIVE_LOAD_MESSAGES unset");
    } else {
        assert!(
            (2..=100).contains(&messages) && (representative || messages == 2),
            "representative messages must be2..100; baseline stays2"
        );
    }
    let policy_mode = std::env::var("MW_NATIVE_LOAD_POLICY").unwrap_or_else(|_| "defaults".into());
    let security_profile = native_enforced_policy::Profile::from_environment(std::env::var("MW_NATIVE_LOAD_SECURITY_PROFILE")).expect("strict native security profile");
    assert!(
        matches!(policy_mode.as_str(), "defaults" | "loaded"),
        "choose defaults or loaded policy profile"
    );
    let prepared_mode=std::env::var("MW_NATIVE_LOAD_PREPARED_MAIL").unwrap_or_else(|_|"off".into());
    let prepared_json=std::env::var("MW_NATIVE_LOAD_PREPARED_MAIL_CONFIG").ok();
    let prepared_options=prepared_mail_fixture_options(&prepared_mode,prepared_json.as_deref());
    assert!(prepared_options.is_none()||(representative&&policy_mode=="loaded"),"prepared mail experiment requires actual representative + loaded policy profile");
    let count: usize = std::env::var("MW_NATIVE_LOAD_PRINCIPALS")
        .unwrap_or_else(|_| "100".into())
        .parse()
        .unwrap();
    if large_mailbox {
        assert_eq!(count, large_mailbox_principals(), "the large-mailbox profile has 1000 users (MW_NATIVE_LOAD_PRINCIPALS=1000; 100 with the smoke scale)");
    } else {
        assert!(
            [1, 100, 1000, 10000].contains(&count),
            "choose representative bounded profile 1, 100, 1000 or 10000"
        );
        assert!(
            count as u64 * messages as u64 <= 500000,
            "representative seeded messages capped at500000"
        );
    }
    if let Some(options)=prepared_options {assert!(options.max_accounts>=count&&options.max_entries>=count,"configure prepared mail capacity explicitly for the whole selected principal working set");}
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
    assert!(!audit_cache||backend=="sqlite","cache experiments require SQLite");
    let key = ServerKey::generate();
    let key_hex = key.to_hex();
    let store = Store::open(&db_path, key).await.unwrap();
    if security_profile == native_enforced_policy::Profile::Enforced {
        assert_eq!(policy_mode,"loaded","enforced security requires loaded protocol policy");
        native_enforced_policy::install(&store).await;
    }
    eprintln!("native fixture: seeding {count} independent users/clients");
    let seeding_started = Instant::now();
    let (principals, mut representative_runtimes) = seed_profile(
        &store,
        count,
        messages,
        representative,
        body_sample_bytes,
        attachment_sample_bytes,
        large_mailbox,
        goal_mix,
    )
    .await;
    let seed_seconds = seeding_started.elapsed().as_secs_f64();
    if policy_mode == "loaded" {
        set_protocol_policy(&store, &["imap"]).await;
    }
    if let Some(options)=prepared_options {
        store.put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key:"performance.preparedMail".into(),global_value:Some(serde_json::to_string(&options).unwrap()),
            allow_user_override:false,global_locked:true,updated_by:"native-load-fixture".into(),updated_at:chrono::Utc::now().timestamp(),
        }).await.unwrap();
    }
    let prepared_policy=mw_server::settings_registry::effective_for(&store,"performance.preparedMail",&Default::default()).await.unwrap().expect("registered prepared mail policy").value;
    if let Some(options)=prepared_options {assert_eq!(prepared_policy,serde_json::to_value(options).unwrap());}
    let mut prepared_experiment=json!({"enabled":prepared_options.is_some(),"requested":prepared_options,"resolvedPolicy":prepared_policy});
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
    let fixture_index_dir=dir.join(if pg_cleanup.is_some(){"search-index"}else{"mailwoman.db.search-index"});
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
            let engine = Arc::new(match index_mode {
                FixtureIndexMode::Sealed=>{
                    std::fs::create_dir_all(&fixture_index_dir).unwrap();
                    #[cfg(unix)]{
                        use std::os::unix::fs::PermissionsExt;
                        std::fs::set_permissions(&fixture_index_dir,std::fs::Permissions::from_mode(0o700)).unwrap();
                    }
                    Engine::open_with_search_and_publication_options(store.clone(),&fixture_index_dir,writer_options,t34_publication.clone())
                        .expect("construct sealed diagnostic search index")
                }
                FixtureIndexMode::Memory=>{
                    let index=mw_search::Index::open_in_ram_with_writer_options(writer_options)
                        .expect("construct explicit memory diagnostic search index");
                    Engine::with_search_and_publication_options(store.clone(),Arc::new(index),t34_publication.clone())
                        .expect("validated t34 publication options")
                }
            });
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
    // Production starts the journal (trigger capture, startup replay) before serving.
    // Always call it so the fixture matches `engine_mode::start_index_publication`.
    let t34_startup = measured_engine.get().unwrap().start_index_publication().await.expect("start index publication");
    eprintln!("native fixture: index publication start: {t34_startup:?}");
    let seeded_counts: std::collections::HashMap<String, u64> = principals.iter().map(|p| (p["accountId"].as_str().unwrap().to_owned(), p["messageIds"].as_array().unwrap().len() as u64)).collect();
    let startup_readiness = startup_readiness(
        measured_engine.get().unwrap(),
        &push_accounts,
        &|account: &str| seeded_counts.get(account).copied().unwrap_or(0),
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
                let mut snapshot = match diagnostics_hook::snapshot_with_cache_audit(&store,&directory,audit_cache).await {
                    Ok(snapshot)=>snapshot,
                    Err(error)=>return (axum::http::StatusCode::SERVICE_UNAVAILABLE,axum::Json(error.report().clone())),
                };
                if security_profile == native_enforced_policy::Profile::Enforced {
                    snapshot["enforcedPolicySnapshot"] = match mw_server::fixture_profile::protection_policy_witness(&store).await {
                        Ok(value)=>value,
                        Err(_)=>return (axum::http::StatusCode::SERVICE_UNAVAILABLE,axum::Json(json!({"error":"enforced policy witness unavailable"}))),
                    };
                }
                snapshot["preparedMail"] = serde_json::to_value(engine.prepared_mail_status()).unwrap();
                snapshot["ordinaryRequestInputs"] = serde_json::to_value(mw_server::request_inputs_fixture::snapshot()).unwrap();
                snapshot["t34Modes"] = t34_modes::witness(&engine);
                snapshot["httpTelemetry"] = mw_server::http_telemetry_snapshot();
                snapshot["realtime"] = mw_server::push::metrics_snapshot();
                snapshot["processCpuSeconds"] = json!(process_cpu::process_cpu_seconds());
                snapshot["upstreamCommands"] = representative::upstream_commands();
                snapshot["searchIndex"] = json!({
                    "storage":fixture_storage_proof(&engine,index_mode),
                    "writerConfiguration":engine.search().writer_configuration(),
                    "commitAttempts":mw_search::commit_count(),
                    "documentFetches":engine.search().doc_fetch_count(),
                    "writeAndHitCounts":{"documentsWritten":mw_search::write_and_hit_counts().0,"bodyBytesTokenised":mw_search::write_and_hit_counts().1,"hitsMaterialised":mw_search::write_and_hit_counts().2},
                    "listProjectionSearches":engine.search().list_projection_search_count(),
                    "listProjectionDocumentReads":engine.search().list_projection_doc_read_count(),
                    "timings":mw_search::timing::snapshot(),
                    "publication":engine.index_publication_status(),
                });
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
    let linked_login_policy_probe = linked_login_preflight::run(&store,&client,&base,&principals).await;
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
    // Exercise the real authenticated download route, outside timed work.
    // Intent changes cache age only; normal account/attachment policies still run.
    let principal = &principals[0];
    let account = principal["accountId"].as_str().unwrap();
    let message = principal["messageIds"][0].as_str().unwrap();
    let stored = store.get_message(message).await.unwrap();
    let expected = store.get_account_body(account, stored.blob_ref.as_deref().unwrap())
        .await.unwrap().unwrap();
    let download = || client.get(format!("{base}/jmap/download/{account}/{message}/message.eml"))
        .header("cookie", principal["cookie"].as_str().unwrap())
        .header("x-forwarded-for", principal["clientAddress"].as_str().unwrap());
    let background = download().header("x-mailwoman-body-use", "false")
        .send().await.unwrap();
    assert_eq!(background.status(), 200, "owned background download must pass normal policy");
    assert!(background.bytes().await.unwrap().as_ref() == expected.as_slice(),
        "background intent must preserve the actual message bytes");
    let invalid = download().header("x-mailwoman-body-use", "invalid")
        .send().await.unwrap();
    assert_eq!(invalid.status(), 400, "malformed download intent must be rejected");
    let duplicate = download().header("x-mailwoman-body-use", "false")
        .header("x-mailwoman-body-use", "false").send().await.unwrap();
    assert_eq!(duplicate.status(), 400, "duplicate download intent must be rejected");
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
    let enforced_policy = if security_profile == native_enforced_policy::Profile::Enforced {
        Some(native_enforced_policy::preflight(&store,&client,&base).await)
    } else { None };
    if let Some(options)=prepared_options {
        prepared_experiment["startup"]=prime_prepared_mail(&client,&base,&principals[0],measured_engine.get().unwrap(),options).await.expect("actual prepared mail startup configuration");
    }
    let root = fixture_runtime::source_root();
    let report = std::env::var_os("MW_LOAD_REPORT_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join(".orchestration/reports/native-http-load"));
    std::fs::create_dir_all(&report).unwrap();
    let handoff = PrivateFile(dir.join("campaign.private.json"));
    let data = json!({"base":base,"cookie":principals[0]["cookie"],"engineMode":"native","fixtureFeatures":["load-test-fixtures"],"uniquePrincipalCount":count,"clientSimulation":{"mode":"fixture-loopback-trusted-proxy","clientCount":count},"principals":principals,"backend":backend,"buildProfile":if cfg!(debug_assertions){"debug"}else{option_env!("MW_CARGO_BUILD_MODE").unwrap_or("release")},"runtimeWorkerThreads":workers,"runtimeWorkerStackBytes":worker_stack_bytes(),"startupReadiness":startup_readiness,"protection":"administrator defaults","budgets":budgets,"diagnostics":{"path":diagnostic_path,"token":diagnostic_token},"pushTrigger":{"path":push_path,"token":push_token}});
    let mut data = data;
    data["ordinaryRequestInputs"] = serde_json::to_value(mw_server::request_inputs_fixture::snapshot()).unwrap();
    data["t34Modes"] = t34_modes::handoff(&t34_selection, t34_modes::witness(measured_engine.get().unwrap()));
    if let Some(experiment)=cache_experiment {data["sqliteCacheExperiment"]=experiment;}
    data["searchStorage"]=fixture_storage_proof(measured_engine.get().unwrap(),index_mode);
    let configuration=measured_engine.get().unwrap().search().writer_configuration();
    data["searchWriter"]=json!({"requestedWorkers":writer_options.indexing_workers,"indexingWorkers":configuration.indexing_workers,"memoryBudgetBytes":configuration.memory_budget_bytes});
    data["policyEvidence"] = policy_evidence;
    data["enforcedPolicy"] = serde_json::to_value(enforced_policy).unwrap();
    data["preparedMailExperiment"] = prepared_experiment;
    data["linkedLoginPolicyProbe"] = linked_login_policy_probe;
    data["workloadProfile"] = json!({"kind":workload,"messagesPerPrincipal":messages,"seededMessages":count as u64*messages as u64,"seedSeconds":seed_seconds,"transport":if representative {"deterministic-read-flag-move"} else {"read-only"},"sendsMeasured":false,"upstreamNetworkMeasured":false});
    data["workloadProfile"]["profile"] = json!(if large_mailbox { "large-mailbox" } else { "default" });
    if goal_mix {
        data["workloadProfile"]["goalMixSeed"] = json!({"folders":["INBOX (inbox)","Archive (archive)","Trash (trash, empty)","Drafts (drafts, empty)"],
            "principalFields":["inboxId","archiveId","trashId","draftsId","plantedQuarterly","plantedZephyrine"],
            "plantedTerms":{"quarterly":"uid % 5 == 0 (not uid 1, the sample message)","zephyrine":"uid % 2000 == 7"},"flags":"messageIds[2] (UID 3) read, messageIds[3] (UID 4) flagged, every other message unread and unflagged"});
    }
    if large_mailbox {
        data["workloadProfile"]["messagesPerPrincipal"] = Value::Null;
        data["workloadProfile"]["seededMessages"] = json!(seeded_counts.values().sum::<u64>());
        data["workloadProfile"]["cohorts"] = json!(large_mailbox_cohorts().iter().map(|(users, messages)| json!({"users":users,"messages":messages})).collect::<Vec<_>>());
        data["workloadProfile"]["smokeScale"] = json!(large_mailbox_scale() != 1);
        data["workloadProfile"]["flagPattern"] = json!({"unread":"uid % 10 < 3 (30%)","flagged":"uid % 50 == 0 (2%)"});
        data["workloadProfile"]["plantedTerms"] = json!({"quarterly":"uid % 5 == 0","zephyrine":"uid % 2000 == 7"});
    }
    if representative {
        data["workloadProfile"]["bodySampleBytes"] = json!(body_sample_bytes);
        data["workloadProfile"]["attachmentSampleBytes"] = json!(attachment_sample_bytes);
    }
    write_private(&handoff.0, &data);
    let outcome = if std::env::var("MW_LOAD_FIXTURE_EXTERNAL_DRIVER").as_deref() == Ok("1") {
        external_control::run(&data, &format!("native-{count}"), &report, deadline).await
    } else {
        let mut command = match std::env::var("MW_NATIVE_LOAD_DRIVER").as_deref() {
            // t60: the large-mailbox cohort driver (ordinary users' latency while
            // the big accounts run heavy queries). The default generator refuses
            // more than 100 seeded ids per principal.
            Ok("large-mailbox") => {
                assert!(large_mailbox, "MW_NATIVE_LOAD_DRIVER=large-mailbox requires MW_NATIVE_LOAD_PROFILE=large-mailbox");
                let mut command = tokio::process::Command::new("node");
                command.arg(root.join("scripts/perf/bulk-ops/large-mailbox-driver.mjs")).arg(&handoff.0)
                    .arg(report.join(format!("native-large-mailbox-{count}.json"))).current_dir(&root).kill_on_drop(true);
                command
            }
            // t60: one pass of each goal-mix write shape (and candidate shapes), validated; a smoke, not a measurement.
            Ok("goal-mix-smoke") => {
                assert!(goal_mix, "MW_NATIVE_LOAD_DRIVER=goal-mix-smoke requires MW_NATIVE_LOAD_WORKLOAD=goal-65-35");
                let mut command = tokio::process::Command::new("node");
                command.arg(root.join("scripts/perf/bulk-ops/goal-mix-smoke.mjs")).arg(&handoff.0)
                    .arg(report.join(format!("native-goal-mix-smoke-{count}.json"))).current_dir(&root).kill_on_drop(true);
                command
            }
            Err(_) | Ok("default") => fixture_runtime::internal_http_driver(&root, &handoff.0, &report.join(format!("native-{count}.json"))),
            Ok(other) => panic!("MW_NATIVE_LOAD_DRIVER={other:?} must be default or large-mailbox"),
        };
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
