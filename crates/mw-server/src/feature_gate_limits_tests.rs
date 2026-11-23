//! t48: `limit.send_per_hour`, `limit.send_per_day` and `limit.storage_bytes`
//! through the production router, on the classic guard path, the deferred
//! cookie-JMAP path and the published request-input path.
use super::*;
use crate::request_inputs::test_control as inputs_control;
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request as HttpRequest, StatusCode},
};
use mw_store::{
    AccountKind, Credentials, DomainConfigRow, NewAccount, ServerKey, Store,
    settings_policy::SettingPolicyRow, usage,
};
use serde_json::{Value, json};
use std::{
    future::Future,
    sync::{Arc, OnceLock},
    time::Duration,
};
use tower::ServiceExt;

const USER: &str = "sender@limits.test";

#[test]
fn submission_creates_and_created_are_counted_per_entry() {
    let request = json!({"methodCalls":[
        ["EmailSubmission/set",{"create":{"a":{},"b":{}}},"0"],
        ["Email/set",{"create":{"d":{}}},"1"],
        ["EmailSubmission/set",{"create":{"c":{}},"update":{"x":{}}},"2"],
    ]});
    assert_eq!(jmap_submission_creates(&request), 3);
    assert_eq!(jmap_submission_creates(&json!({"methodCalls":[["EmailSubmission/set",{"update":{"x":{}}},"0"]]})), 0);
    let response = json!({"methodResponses":[
        ["EmailSubmission/set",{"created":{"a":{"id":"1"}},"notCreated":{"b":{}}},"0"],
        ["error",{"type":"serverFail"},"2"],
    ]});
    assert_eq!(jmap_submissions_created(&response), Some(1));
    assert_eq!(jmap_submissions_created(&json!({"x":1})), None);
}

#[test]
fn published_policy_keys_stay_within_the_published_input_bound() {
    // request_inputs::policy_keys returns None above 64 keys, which silently
    // turns the published-input path off for every request.
    let keys = crate::request_inputs::policy_keys().expect("policy keys must stay at 64 or fewer");
    assert!(keys.len() <= 64, "{} policy keys", keys.len());
    for key in ["limit.send_per_hour", "limit.send_per_day", "limit.recipients_per_message", "mail.allowSend"] {
        assert!(keys.contains(&key), "{key} must be a published policy key");
    }
    eprintln!("published policy keys: {}", keys.len());
}

#[test]
fn period_remaining_ends_at_the_period_boundary() {
    assert_eq!(period_remaining(usage::SEND_HOUR, 7200), 3600);
    assert_eq!(period_remaining(usage::SEND_HOUR, 7199), 1);
    assert_eq!(period_remaining(usage::SEND_DAY, 86_400 + 60), 86_340);
}

struct Fixture {
    router: Router,
    writer: Store,
    measured: Store,
    session: String,
    account: String,
}

impl Fixture {
    async fn new() -> Self {
        let directory = std::env::temp_dir().join(format!("mw-t48-limits-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();
        let db = directory.join("mail.db");
        let key = ServerKey::generate();
        let writer = Store::open(db.to_str().unwrap(), key.clone()).await.unwrap();
        writer.put_domain_config(&DomainConfigRow {
            name: "limits.test".into(),
            upstream_json: json!({"imap":{"host":"127.0.0.1","port":9,"tls":"starttls","auth":"password","allowPrivate":true}}).to_string(),
            allowlist_json: "[]".into(), blocklist_json: "[]".into(), enabled: true, preferred: false,
            settings_json: "{}".into(), updated_at: 1,
        }).await.unwrap();
        let credentials = Credentials { username: USER.into(), password: "fixture-only-not-a-real-password".into() };
        let account = writer.create_account(&NewAccount {
            kind: AccountKind::Imap, host: "127.0.0.1", port: 9, tls: "starttls",
            username: USER, sync_policy_json: "{}",
        }, &credentials).await.unwrap();
        writer.remember_account_names(&account, &[USER]).await.unwrap();
        let session = writer.create_session(&account, USER, "engine", "engine", &credentials).await.unwrap();
        writer.set_session_authentication(&session, "password").await.unwrap();
        writer.bind_verified_primary(&session, mw_store::identity_principal::VerifiedPrimary {
            account_id: &account, kind: "mailbox", authority: "t48-limits-fixture", subject: USER,
        }, chrono::Utc::now().timestamp()).await.unwrap().unwrap();
        policy(&writer, "access.protocols", json!(["imap"])).await;
        let captured = Arc::new(OnceLock::new());
        let install = captured.clone();
        let db_path = db.to_string_lossy().into_owned();
        let engine_db = db_path.clone();
        let (router, _) = crate::build_app_inner(
            crate::AppConfig {
                db_path, server_key_hex: Some(key.to_hex()), web_dir: None, cookie_secure: false,
                mode: crate::ServerMode::Engine, hardening: Default::default(), security: Default::default(),
            },
            crate::V6Config { admin_enabled: false, ..Default::default() },
            crate::sso::SsoProviderSource::Store,
            Some(Box::new(move |store: &Store| {
                assert!(install.set(store.clone()).is_ok());
                crate::engine_mode::build_engine(store, &engine_db)
            })),
            crate::core_test_build_info::info().unwrap(),
        ).await.unwrap();
        let fixture = Self { router, writer, measured: captured.get().unwrap().clone(), session, account };
        let (status, body, _, _) = fixture.send(fixture.echo(), false).await;
        assert_eq!(status, StatusCode::OK, "fixture startup must preserve real primary auth: {body}");
        fixture
    }
    fn jmap(&self, body: Value) -> HttpRequest<Body> {
        let csrf = self.writer.session_csrf_token(&self.session);
        let mut request = HttpRequest::builder().method("POST").uri("/jmap/api").header("host", "localhost")
            .header("origin", "http://localhost").header("content-type", "application/json")
            .header("cookie", format!("mw_session={}; mw_csrf={csrf}", self.session)).header("x-csrf-token", csrf)
            .header("user-agent", "Mozilla/5.0")
            .body(Body::from(serde_json::to_vec(&body).unwrap())).unwrap();
        request.extensions_mut().insert(axum::extract::ConnectInfo(
            "127.0.0.1:48213".parse::<std::net::SocketAddr>().unwrap()));
        request
    }
    fn echo(&self) -> HttpRequest<Body> {
        self.jmap(json!({"using":["urn:ietf:params:jmap:core"],"methodCalls":[["Core/echo",{"proof":"fresh"},"echo"]]}))
    }
    fn submit(&self, legacy: bool) -> HttpRequest<Body> {
        let mut request = self.jmap(json!({
            "using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail","urn:ietf:params:jmap:submission"],
            "methodCalls":[["EmailSubmission/set",{"accountId":"no-such-account",
                "create":{"s1":{"emailId":"Mmissing","identityId":"none"}}},"s"]]}));
        if legacy {
            request.extensions_mut().insert(auth_boundary::ForceLegacy);
        }
        request
    }
    fn upload(&self, bytes: usize) -> HttpRequest<Body> {
        let csrf = self.writer.session_csrf_token(&self.session);
        HttpRequest::builder().method("POST").uri(format!("/jmap/upload/{}/", self.account)).header("host", "localhost")
            .header("origin", "http://localhost").header("content-type", "application/octet-stream")
            .header("content-length", bytes.to_string())
            .header("cookie", format!("mw_session={}; mw_csrf={csrf}", self.session)).header("x-csrf-token", csrf)
            .body(Body::from(vec![b'x'; bytes])).unwrap()
    }
    async fn send(&self, request: HttpRequest<Body>, published: bool)
        -> (StatusCode, Value, inputs_control::Counts, mw_store::pool_telemetry::QuerySnapshot)
    {
        let ((response, counts), sql) = mw_store::pool_telemetry::measure_queries(
            inputs_control::measure(published, self.router.clone().oneshot(request))).await;
        let response = response.unwrap();
        let status = response.status();
        let retry_after = response.headers().get("retry-after").cloned();
        let bytes = to_bytes(response.into_body(), 4 * 1024 * 1024).await.unwrap();
        let mut body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        if let (Some(retry), Some(map)) = (retry_after, body.as_object_mut()) {
            map.insert("retryAfter".into(), json!(retry.to_str().unwrap()));
        }
        (status, body, counts, sql)
    }
    async fn sent(&self, metric: &str) -> i64 {
        self.writer.usage_sum(USER, metric, 0, i64::MAX).await.unwrap()
    }
    async fn preload(&self, metric: &str, amount: i64) {
        let period = usage::period_of(metric, chrono::Utc::now().timestamp());
        self.writer.usage_reserve(USER, "limits.test", metric, period, amount, 0).await.unwrap().unwrap();
    }
    async fn warm(&self) {
        let (status, _, _, _) = self.send(self.echo(), true).await;
        assert_eq!(status, StatusCode::OK);
        let worker = crate::feature_gate::native_metadata_admission::publication::Worker::start(self.measured.clone()).unwrap();
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let environment = crate::request_inputs::Environment::capture().unwrap();
                if let Some(entry) = worker.request_inputs(&self.session, &environment)
                    && self.measured.native_publication_fence(&self.session, &entry.domains, &entry.subjects)
                        .await.unwrap().is_some_and(|source| entry.matches(&source)) { break; }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }).await.expect("the authenticated route must publish request inputs");
    }
}

async fn policy(store: &Store, key: &str, value: Value) {
    store.put_setting_policy(&SettingPolicyRow {
        key: key.into(), global_value: Some(value.to_string()), allow_user_override: false,
        global_locked: true, updated_by: "t48-limits-fixture".into(), updated_at: chrono::Utc::now().timestamp(),
    }).await.unwrap();
}

fn run<F, Fut>(factory: F)
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = ()>,
{
    std::thread::Builder::new().name("t48-limits".into()).stack_size(16 * 1024 * 1024)
        .spawn(move || {
            tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(factory());
        }).unwrap().join().unwrap();
}

fn assert_over(reply: &(StatusCode, Value, inputs_control::Counts, mw_store::pool_telemetry::QuerySnapshot), key: &str, max: u64) {
    assert_eq!(reply.0, StatusCode::TOO_MANY_REQUESTS, "{}", reply.1);
    assert_eq!(reply.1["code"], "limit_exceeded");
    assert_eq!(reply.1["limit"], key);
    assert_eq!(reply.1["max"], max);
    let retry: i64 = reply.1["retryAfter"].as_str().unwrap().parse().unwrap();
    assert!((1..=86_400).contains(&retry));
}

#[test]
fn send_per_hour_refuses_the_third_submission_on_classic_and_deferred_paths() {
    run(|| async {
        let fixture = Fixture::new().await;
        policy(&fixture.writer, "limit.send_per_hour", json!(2)).await;
        // Two submissions already counted this hour (the shared counter, as
        // another instance would have left it).
        fixture.preload(usage::SEND_HOUR, 2).await;
        for legacy in [true, false] {
            let reply = fixture.send(fixture.submit(legacy), false).await;
            assert_over(&reply, "limit.send_per_hour", 2);
            assert_eq!(fixture.sent(usage::SEND_HOUR).await, 2, "a refused submission reserves nothing");
            assert_eq!(fixture.sent(usage::SEND_DAY).await, 0);
        }
        // Room for one: the reservation is taken, the submission fails (no
        // such email, no reachable mail server), and the reservation is
        // given back on both paths.
        fixture.writer.usage_release(USER, usage::SEND_HOUR, usage::period_of(usage::SEND_HOUR, chrono::Utc::now().timestamp()), 1).await.unwrap();
        for legacy in [true, false] {
            let reply = fixture.send(fixture.submit(legacy), false).await;
            assert_ne!(reply.0, StatusCode::TOO_MANY_REQUESTS, "{}", reply.1);
            assert_eq!(fixture.sent(usage::SEND_HOUR).await, 1, "a failed submission releases its reservation: {}", reply.1);
        }
    });
}

#[test]
fn send_per_day_refuses_and_releases_the_hour_reservation() {
    run(|| async {
        let fixture = Fixture::new().await;
        policy(&fixture.writer, "limit.send_per_hour", json!(5)).await;
        policy(&fixture.writer, "limit.send_per_day", json!(3)).await;
        fixture.preload(usage::SEND_DAY, 3).await;
        let reply = fixture.send(fixture.submit(false), false).await;
        assert_over(&reply, "limit.send_per_day", 3);
        assert_eq!(fixture.sent(usage::SEND_HOUR).await, 0, "the hour reservation is taken back when the day limit refuses");
        assert_eq!(fixture.sent(usage::SEND_DAY).await, 3);
        // 0 means no limit.
        policy(&fixture.writer, "limit.send_per_day", json!(0)).await;
        policy(&fixture.writer, "limit.send_per_hour", json!(0)).await;
        let reply = fixture.send(fixture.submit(false), false).await;
        assert_ne!(reply.0, StatusCode::TOO_MANY_REQUESTS, "{}", reply.1);
        assert_eq!(fixture.sent(usage::SEND_HOUR).await, 0, "no limit set: nothing is counted");
    });
}

#[test]
fn send_per_hour_refuses_the_third_submission_on_published_inputs() {
    run(|| async {
        let fixture = Fixture::new().await;
        policy(&fixture.writer, "limit.send_per_hour", json!(2)).await;
        fixture.warm().await;
        fixture.preload(usage::SEND_HOUR, 2).await;
        let reply = fixture.send(fixture.submit(false), true).await;
        assert!(reply.2.session_decodes >= 1 && reply.2.legacy_session_reads == 0,
            "the decision must run on published inputs: {:?}", reply.2);
        assert_over(&reply, "limit.send_per_hour", 2);
        assert_eq!(fixture.sent(usage::SEND_HOUR).await, 2);
        fixture.writer.usage_release(USER, usage::SEND_HOUR, usage::period_of(usage::SEND_HOUR, chrono::Utc::now().timestamp()), 1).await.unwrap();
        let reply = fixture.send(fixture.submit(false), true).await;
        assert_ne!(reply.0, StatusCode::TOO_MANY_REQUESTS, "{}", reply.1);
        assert_eq!(fixture.sent(usage::SEND_HOUR).await, 1, "a failed submission releases its reservation: {}", reply.1);
    });
}

#[test]
fn unset_send_limits_add_no_statement_to_a_published_submission() {
    run(|| async {
        let fixture = Fixture::new().await;
        fixture.warm().await;
        let unset = fixture.send(fixture.submit(false), true).await;
        assert!(unset.2.session_decodes >= 1 && unset.2.legacy_session_reads == 0, "{:?}", unset.2);
        assert_eq!(fixture.sent(usage::SEND_HOUR).await + fixture.sent(usage::SEND_DAY).await, 0);
        // The same request again, limits still unset: identical statement
        // counts, and none of them is a usage_counter statement (a set limit
        // adds exactly the reserve and the release).
        let again = fixture.send(fixture.submit(false), true).await;
        assert_eq!((again.3.read.calls, again.3.write.calls, again.3.other.calls),
            (unset.3.read.calls, unset.3.write.calls, unset.3.other.calls));
        policy(&fixture.writer, "limit.send_per_hour", json!(9)).await;
        fixture.warm().await;
        let set = fixture.send(fixture.submit(false), true).await;
        assert!(set.2.session_decodes >= 1 && set.2.legacy_session_reads == 0, "{:?}", set.2);
        assert_eq!(set.3.write.calls + set.3.other.calls + set.3.read.calls,
            unset.3.write.calls + unset.3.other.calls + unset.3.read.calls + 2,
            "a set limit adds the reserve and the release; unset adds none");
    });
}

#[test]
fn storage_bytes_refuses_an_upload_over_the_bytes_held() {
    run(|| async {
        let fixture = Fixture::new().await;
        // Unset: no storage refusal.
        let reply = fixture.send(fixture.upload(64), false).await;
        assert_ne!(reply.1["limit"], "limit.storage_bytes", "{}", reply.1);
        policy(&fixture.writer, "limit.storage_bytes", json!(100)).await;
        let held = fixture.writer.usage_storage_bytes(&fixture.account).await.unwrap();
        let reply = fixture.send(fixture.upload((100 - held + 1) as usize), false).await;
        assert_eq!(reply.0, StatusCode::PAYLOAD_TOO_LARGE, "{}", reply.1);
        assert_eq!(reply.1["limit"], "limit.storage_bytes");
        assert_eq!(reply.1["max"], 100);
        if held < 100 {
            let reply = fixture.send(fixture.upload((100 - held) as usize), false).await;
            assert_ne!(reply.1["limit"], "limit.storage_bytes", "{}", reply.1);
        }
        policy(&fixture.writer, "limit.storage_bytes", json!(0)).await;
        let reply = fixture.send(fixture.upload(500), false).await;
        assert_ne!(reply.1["limit"], "limit.storage_bytes", "0 means no limit: {}", reply.1);
    });
}
