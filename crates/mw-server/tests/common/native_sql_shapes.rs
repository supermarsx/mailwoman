//! Bounded actual-router attribution, never a throughput/capacity campaign.
use super::*;
use axum::{
    body::{Body, to_bytes},
    extract::ConnectInfo,
    http::{Request, StatusCode},
};
use mw_store::pool_telemetry::shapes::{self, Report};
use tower::ServiceExt;
fn properties(name: &str) -> Vec<String> {
    let source = include_str!("../../../../scripts/perf/fixture-properties.mjs");
    let prefix = format!("export const {name}=Object.freeze(");
    let value = source
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .and_then(|line| line.strip_suffix(");"))
        .unwrap();
    let array = value
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .unwrap();
    if !array.contains("...") {
        return serde_json::from_str(&format!("[{array}]").replace('\'', "\"")).unwrap();
    }
    array
        .split(',')
        .flat_map(|name| properties(name.trim().strip_prefix("...").unwrap()))
        .collect()
}
async fn header(router: axum::Router, principal: &Value) -> (StatusCode, Value, Report) {
    let request=Request::builder().method("POST").uri("/jmap/api")
        .header("host","localhost").header("origin","http://localhost")
        .header("content-type","application/json")
        .header("cookie",principal["cookie"].as_str().unwrap())
        .header("x-csrf-token",principal["csrfToken"].as_str().unwrap())
        .extension(ConnectInfo(SocketAddr::from(([127,0,0,1],35001))))
        .body(Body::from(serde_json::to_vec(&json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Email/get",{"accountId":principal["accountId"],"ids":principal["messageIds"],"properties":properties("DEFAULT_HEADER_PROPERTIES")},"shapes"]]})).unwrap())).unwrap();
    let ((status, value), report) = tokio::time::timeout(
        Duration::from_secs(30),
        shapes::measure(async move {
            let response = router.oneshot(request).await.unwrap();
            let status = response.status();
            let bytes = to_bytes(response.into_body(), 2 * 1024 * 1024)
                .await
                .unwrap();
            let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
            (status, value)
        }),
    )
    .await
    .expect("bounded actual-router diagnostic deadline");
    report
        .require_valid()
        .expect("overflow/mismatch must invalidate HTTP attribution");
    (status, value, report)
}
#[test]
fn actual_router_header_sql_shapes_keep_real_policy_and_ownership_checks() {
    // Existing request profiling installs another counter scope. Refuse that
    // incompatible diagnostic combination rather than report incomplete counts.
    assert_ne!(
        std::env::var("MW_FIXTURE_REQUEST_PROFILE").as_deref(),
        Ok("1")
    );
    // Construct and poll the large production-router debug future on a bounded
    // owned stack; Tokio's worker stack size does not change block_on's caller.
    let fixture = std::thread::Builder::new()
        .name("mw-sql-shape-fixture".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .thread_stack_size(8 * 1024 * 1024)
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    let directory = tempfile::tempdir().unwrap();
                    let path = directory
                        .path()
                        .join("sql-shapes.db")
                        .to_string_lossy()
                        .into_owned();
                    let key = ServerKey::generate();
                    let seed_store = Store::open(&path, key.clone()).await.unwrap();
                    let (principals, mut runtimes) =
                        seed_profile(&seed_store, 1, 50, true, 4096, 65536).await;
                    set_protocol_policy(&seed_store, &["imap"]).await;
                    let principal = principals.into_iter().next().unwrap();
                    let account = principal["accountId"].as_str().unwrap().to_owned();
                    seed_store.close().await;
                    drop(seed_store);
                    let search_dir = directory.path().join("search");
                    std::fs::create_dir(&search_dir).unwrap();
                    let installed = Arc::new(std::sync::OnceLock::<Arc<Engine>>::new());
                    let capture = installed.clone();
                    let runtime_account = account.clone();
                    let (router, push) = build_app_with_fixture_engine(
                        AppConfig {
                            db_path: path.clone(),
                            server_key_hex: Some(key.to_hex()),
                            web_dir: None,
                            cookie_secure: false,
                            mode: ServerMode::Engine,
                            hardening: Default::default(),
                            security: SecurityConfig::default(),
                        },
                        V6Config::default(),
                        move |store| {
                            let engine = Arc::new(
                                Engine::open_with_search(store.clone(), &search_dir).unwrap(),
                            );
                            engine.register_backend(
                                runtime_account.clone(),
                                AccountRuntime::new(
                                    runtimes.remove(&runtime_account).unwrap(),
                                    Arc::new(NoSend),
                                    "user0@native-load.invalid",
                                ),
                            );
                            assert!(capture.set(engine.clone()).is_ok());
                            engine
                        },
                    )
                    .await
                    .unwrap();
                    let engine = installed.get().unwrap();
                    startup_readiness(engine, &[account.clone()], 50, Instant::now(), 30)
                        .await
                        .expect("real rebuild must complete before attribution");
                    let (status, value, first) = header(router.clone(), &principal).await;
                    assert_eq!(status, StatusCode::OK);
                    assert_eq!(value["methodResponses"][0][0], "Email/get");
                    assert_eq!(
                        value["methodResponses"][0][1]["list"]
                            .as_array()
                            .unwrap()
                            .len(),
                        50
                    );
                    assert!(
                        first.operations_completed > 6,
                        "real router must include more than Engine's6cold statements"
                    );
                    let (status, value, warm) = header(router.clone(), &principal).await;
                    assert_eq!(status, StatusCode::OK);
                    assert_eq!(
                        value["methodResponses"][0][1]["list"]
                            .as_array()
                            .unwrap()
                            .len(),
                        50
                    );
                    // A separate Store writer must immediately revoke the same legitimate
                    // cookie's protocol access, then restore it without weakening guards.
                    let writer = Store::open(&path, key).await.unwrap();
                    set_protocol_policy(&writer, &[]).await;
                    let (status, _, denied) = header(router.clone(), &principal).await;
                    assert_eq!(status, StatusCode::FORBIDDEN);
                    set_protocol_policy(&writer, &["imap"]).await;
                    let (status, value, restored) = header(router.clone(), &principal).await;
                    assert_eq!(status, StatusCode::OK);
                    assert_eq!(
                        value["methodResponses"][0][1]["list"]
                            .as_array()
                            .unwrap()
                            .len(),
                        50
                    );
                    for (label, report) in [
                        ("cold", first),
                        ("warm", warm),
                        ("revoked", denied),
                        ("restored", restored),
                    ] {
                        // Totals include the explicit unknown bucket. Phases are
                        // captured before q() awaits; they are exclusive caller
                        // labels, not additive CPU or inherited worker activity.
                        assert_eq!(report.phases.iter().map(|phase|phase.started).sum::<u64>(),report.operations_started);
                        assert_eq!(report.phases.iter().map(|phase|phase.completed).sum::<u64>(),report.operations_completed);
                        assert!(report.phases.iter().any(|phase|phase.phase=="unknown"));
                        let revisions=report.shapes.iter().filter(|shape|
                            shape.static_token_families==["settings_revision"])
                            .flat_map(|shape|shape.phase_calls.iter()).map(|phase|
                                json!({"phase":phase.phase,"calls":phase.calls,"failures":phase.failures}))
                            .collect::<Vec<_>>();
                        eprintln!("actual-router-header-phase-summary {label}: {}",json!({
                            "operationsStarted":report.operations_started,"operationsCompleted":report.operations_completed,
                            "settingsRevisionByPhase":revisions,"phases":report.phases}));
                        eprintln!(
                            "actual-router-header-sql-shapes {label}: {}",
                            serde_json::to_string(&report).unwrap()
                        );
                    }
                    writer.close().await;
                    drop(writer);
                    drop(router);
                    drop(push);
                    engine.store().close().await;
                    // Retained startup workers are the same production lifecycle as the
                    // existing native fixture. This bounded test owns only its temp directory.
                });
        })
        .expect("bounded SQL diagnostic fixture thread");
    if let Err(panic) = fixture.join() {
        std::panic::resume_unwind(panic);
    }
}

// ---- t34-e0: per-route attribution baseline (measurement only) ----
struct Route {
    name: &'static str,
    method: &'static str,
    uri: String,
    body: Option<Value>,
    intent: bool,
    link: Option<String>,
}
async fn route_request(router: axum::Router, principal: &Value, route: &Route) -> (StatusCode, Value, Report) {
    let mut builder = Request::builder()
        .method(route.method)
        .uri(route.uri.as_str())
        .header("host", "localhost")
        .header("origin", "http://localhost")
        .header("user-agent", "Mozilla/5.0 Chrome/140.0 Safari/537.36")
        .header("cookie", principal["cookie"].as_str().unwrap())
        .header("x-csrf-token", principal["csrfToken"].as_str().unwrap());
    if let Some(link) = &route.link {
        builder = builder.header("x-mailwoman-link", link.as_str());
    }
    if route.intent {
        builder = builder.header("x-mailwoman-read-shape", "native-metadata-v1");
    }
    let body = match &route.body {
        Some(value) => {
            builder = builder.header("content-type", "application/json");
            Body::from(serde_json::to_vec(value).unwrap())
        }
        None => Body::empty(),
    };
    let request = builder
        .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 35001))))
        .body(body)
        .unwrap();
    let ((status, value), report) = tokio::time::timeout(
        Duration::from_secs(30),
        shapes::measure(async move {
            let response = router.oneshot(request).await.unwrap();
            let status = response.status();
            let bytes = to_bytes(response.into_body(), 4 * 1024 * 1024).await.unwrap();
            (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
        }),
    )
    .await
    .expect("bounded actual-router route deadline");
    report.require_valid().expect("overflow/mismatch must invalidate HTTP attribution");
    (status, value, report)
}
fn jmap_body(calls: Vec<Value>) -> Value {
    json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":calls})
}
fn emit_route(profile: &str, route: &str, label: &str, status: StatusCode, report: &Report) {
    let shapes = report.shapes.iter().map(|shape| json!({
        "class":shape.class,"families":shape.static_token_families,"calls":shape.calls,
        "failures":shape.failures,"rows":shape.returned_rows,"affected":shape.affected_rows,
        "sha":&shape.template_sha256[..12],
        "phases":shape.phase_calls.iter().map(|p|json!({"phase":p.phase,"calls":p.calls})).collect::<Vec<_>>()}))
        .collect::<Vec<_>>();
    let phases = report.phases.iter().filter(|p| p.completed > 0)
        .map(|p| json!({"phase":p.phase,"calls":p.completed})).collect::<Vec<_>>();
    // Phase entry counts: `auth.total` entries are full authentication chains.
    let entries = report.phases.iter().filter(|p| p.entries > 0)
        .map(|p| json!({"phase":p.phase,"entries":p.entries})).collect::<Vec<_>>();
    eprintln!("t34-route-baseline {}", json!({"profile":profile,"route":route,"reads":report.query_counts.read.calls,
        "writes":report.query_counts.write.calls,"other":report.query_counts.other.calls,"entries":entries,"label":label,"status":status.as_u16(),
        "operations":report.operations_completed,"phases":phases,"shapes":shapes}));
}
#[test]
fn actual_router_route_sql_baseline_t34() {
    run_route_baseline(native_enforced_policy::Profile::Defaults);
}
#[test]
fn actual_router_route_sql_baseline_t34_enforced() {
    run_route_baseline(native_enforced_policy::Profile::Enforced);
}
// Seeds a real linked mailbox through public Store APIs only: a second
// verified principal, a dedicated source session for it (linking promotes the
// source to an internal session), the global linking policy, then the link.
async fn seed_link(store: &Store, owner: &Value, child: &Value) -> String {
    let session_of = |p: &Value| p["cookie"].as_str().unwrap().strip_prefix("mw_session=").unwrap()
        .split(';').next().unwrap().to_owned();
    for key in ["accounts.linkingEnabled", "accounts.useLinked"] {
        store.put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: key.into(), global_value: Some("true".into()), allow_user_override: false,
            global_locked: true, updated_by: "t34-e14".into(), updated_at: chrono::Utc::now().timestamp(),
        }).await.unwrap();
    }
    let child_session = store.get_primary_session(&session_of(child)).await.unwrap();
    let source = store.create_session(&child_session.account_id, &child_session.username, "engine", "engine",
        &child_session.credentials).await.unwrap();
    store.set_session_authentication(&source, "password").await.unwrap();
    store.bind_verified_primary(&source, mw_store::identity_principal::VerifiedPrimary {
        account_id: &child_session.account_id, kind: "mailbox", authority: "isolated-native-load",
        subject: &child_session.account_id }, chrono::Utc::now().timestamp()).await.unwrap().unwrap();
    store.attach_verified_link(&session_of(owner), &source, "t34 linked", 2, chrono::Utc::now().timestamp())
        .await.unwrap().expect("link must attach").id
}
fn run_route_baseline(profile: native_enforced_policy::Profile) {
    install_sql_capture();
    assert_ne!(std::env::var("MW_FIXTURE_REQUEST_PROFILE").as_deref(), Ok("1"));
    let fixture = std::thread::Builder::new()
        .name("mw-sql-route-baseline".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .thread_stack_size(8 * 1024 * 1024)
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    let directory = tempfile::tempdir().unwrap();
                    let path = directory.path().join("sql-routes.db").to_string_lossy().into_owned();
                    let key = ServerKey::generate();
                    let seed_store = Store::open(&path, key.clone()).await.unwrap();
                    let (principals, mut runtimes) =
                        seed_profile(&seed_store, 2, 50, true, 4096, 65536).await;
                    set_protocol_policy(&seed_store, &["imap"]).await;
                    let link = seed_link(&seed_store, &principals[0], &principals[1]).await;
                    if profile == native_enforced_policy::Profile::Enforced {
                        native_enforced_policy::install(&seed_store).await;
                    }
                    let profile_name = if profile == native_enforced_policy::Profile::Enforced { "enforced" } else { "defaults" };
                    let child = principals[1].clone();
                    let principal = principals[0].clone();
                    let account = principal["accountId"].as_str().unwrap().to_owned();
                    seed_store.close().await;
                    drop(seed_store);
                    let search_dir = directory.path().join("search");
                    std::fs::create_dir(&search_dir).unwrap();
                    let installed = Arc::new(std::sync::OnceLock::<Arc<Engine>>::new());
                    let capture = installed.clone();
                    let runtime_accounts = vec![account.clone(), child["accountId"].as_str().unwrap().to_owned()];
                    let (router, push) = build_app_with_fixture_engine(
                        AppConfig {
                            db_path: path.clone(),
                            server_key_hex: Some(key.to_hex()),
                            web_dir: None,
                            cookie_secure: false,
                            mode: ServerMode::Engine,
                            hardening: Default::default(),
                            security: SecurityConfig::default(),
                        },
                        V6Config::default(),
                        move |store| {
                            let engine = Arc::new(Engine::open_with_search(store.clone(), &search_dir).unwrap());
                            for (index, runtime_account) in runtime_accounts.iter().enumerate() {
                                engine.register_backend(
                                    runtime_account.clone(),
                                    AccountRuntime::new(
                                        runtimes.remove(runtime_account).unwrap(),
                                        Arc::new(NoSend),
                                        &format!("user{index}@native-load.invalid"),
                                    ),
                                );
                            }
                            assert!(capture.set(engine.clone()).is_ok());
                            engine
                        },
                    )
                    .await
                    .unwrap();
                    let engine = installed.get().unwrap();
                    startup_readiness(engine, &[account.clone(), child["accountId"].as_str().unwrap().to_owned()], 50, Instant::now(), 30)
                        .await
                        .expect("real rebuild must complete before attribution");
                    let ids = principal["messageIds"].clone();
                    let two = json!(ids.as_array().unwrap()[..2].to_vec());
                    let acct = principal["accountId"].clone();
                    let call = |name: &str, args: Value, id: &str| {
                        let mut args = args;
                        args["accountId"] = acct.clone();
                        json!([name, args, id])
                    };
                    let reader_id = ids[0].clone();
                    let header_props = properties("DEFAULT_HEADER_PROPERTIES");
                    let by_ref = |name: &str, id: &str, query: &str| call(name, json!({"#ids":{"resultOf":query,"name":"Email/query","path":"/ids"},"properties":header_props}), id);
                    let reader_args = {
                        let mut a = json!({"ids":[reader_id],"properties":properties("READER_PROPERTIES")});
                        a["fetchHTMLBodyValues"] = json!(true);
                        a["fetchTextBodyValues"] = json!(true);
                        a["maxBodyValueBytes"] = json!(1000000);
                        a["mailwomanBodyUse"] = json!(true);
                        a
                    };
                    let jmap = |name: &'static str, intent: bool, calls: Vec<Value>| Route {
                        name, method: "POST", uri: "/jmap/api".into(), intent, body: Some(jmap_body(calls)), link: None,
                    };
                    let routes = vec![
                        Route { name: "identity", method: "GET", uri: "/api/me".into(), body: None, intent: false, link: None },
                        Route { name: "jmap-session", method: "GET", uri: "/jmap/session".into(), body: None, intent: false, link: None },
                        jmap("mailbox-get", false, vec![call("Mailbox/get", json!({}), "m")]),
                        jmap("email-query", false, vec![call("Email/query", json!({"limit":25}), "q")]),
                        jmap("email-get-headers-50-noIntentHeader", false,
                            vec![call("Email/get", json!({"ids":ids,"properties":header_props}), "g")]),
                        jmap("email-get-headers-50-nativeIntent", true,
                            vec![call("Email/get", json!({"ids":ids,"properties":header_props}), "g")]),
                        jmap("email-get-headers-2-nativeIntent", true,
                            vec![call("Email/get", json!({"ids":two,"properties":header_props}), "g")]),
                        Route { name: "refresh", method: "POST", uri: "/api/mail/refresh".into(),
                            body: Some(json!({"accountId":acct})), intent: false, link: None },
                        jmap("reader-body", false, vec![call("Email/get", reader_args, "r")]),
                        jmap("batch-query-get-2method", false, vec![
                            call("Email/query", json!({"limit":25,"position":0,"calculateTotal":true}), "page1"),
                            by_ref("Email/get", "headers1", "page1")]),
                        jmap("batch-paged-4method", false, vec![
                            call("Email/query", json!({"limit":25,"position":0,"calculateTotal":true}), "page1"),
                            by_ref("Email/get", "headers1", "page1"),
                            call("Email/query", json!({"limit":25,"position":25,"calculateTotal":true}), "page2"),
                            by_ref("Email/get", "headers2", "page2")]),
                    ];
                    let child_acct = child["accountId"].clone();
                    let linked_call = |name: &str, args: Value, id: &str| {
                        let mut args = args;
                        args["accountId"] = child_acct.clone();
                        json!([name, args, id])
                    };
                    let linked_by_ref = |id: &str, query: &str| linked_call("Email/get", json!({"#ids":{"resultOf":query,"name":"Email/query","path":"/ids"},"properties":header_props}), id);
                    let mut routes = routes;
                    routes.push(Route { name: "linked-identity", method: "GET", uri: "/api/me".into(), body: None, intent: false, link: Some(link.clone()) });
                    routes.push(Route { name: "linked-jmap-session", method: "GET", uri: "/jmap/session".into(), body: None, intent: false, link: Some(link.clone()) });
                    routes.push(Route { name: "linked-mailbox-get", method: "POST", uri: "/jmap/api".into(), intent: false, link: Some(link.clone()),
                        body: Some(jmap_body(vec![linked_call("Mailbox/get", json!({}), "m")])) });
                    routes.push(Route { name: "linked-batch-query-get-2method", method: "POST", uri: "/jmap/api".into(), intent: false, link: Some(link.clone()),
                        body: Some(jmap_body(vec![linked_call("Email/query", json!({"limit":25,"position":0,"calculateTotal":true}), "page1"), linked_by_ref("headers1", "page1")])) });
                    // Identity/session are first in the list, before the published entry exists;
                    // measure them again after every other route so they are warm.
                    routes.push(Route { name: "identity-late", method: "GET", uri: "/api/me".into(), body: None, intent: false, link: None });
                    routes.push(Route { name: "jmap-session-late", method: "GET", uri: "/jmap/session".into(), body: None, intent: false, link: None });
                    for route in &routes {
                        // first = first request of this shape after startup (route-cold);
                        // warm1/warm2 = immediately repeated requests with the same cookie.
                        for label in ["first", "warm1", "warm2"] {
                            let (status, value, report) = route_request(router.clone(), &principal, route).await;
                            if label == "first" {
                                eprintln!("t34-route-response {} {} status={} head={}", profile_name, route.name, status,
                                    value.to_string().chars().take(200).collect::<String>());
                            }
                            emit_route(profile_name, route.name, label, status, &report);
                        }
                    }
                    emit_captured_sql();
                    drop(router);
                    drop(push);
                    engine.store().close().await;
                });
        })
        .expect("bounded SQL route fixture thread");
    if let Err(panic) = fixture.join() {
        std::panic::resume_unwind(panic);
    }
}
// Captures the text of every statement sqlx logs, keyed by the same SHA-256 the
// shape report uses, so unlinked template hashes can be attributed to SQL text.
static CAPTURED_SQL: std::sync::Mutex<Vec<(String, String)>> = std::sync::Mutex::new(Vec::new());
struct SqlCapture;
struct SqlVisitor(Option<String>);
impl tracing::field::Visit for SqlVisitor {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "db.statement" && !value.is_empty() {
            self.0 = Some(value.to_owned());
        }
    }
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "db.statement" && self.0.is_none() {
            let text = format!("{value:?}");
            if !text.is_empty() {
                self.0 = Some(text);
            }
        }
    }
}
impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for SqlCapture {
    fn on_event(&self, event: &tracing::Event<'_>, _: tracing_subscriber::layer::Context<'_, S>) {
        if event.metadata().target() != "sqlx::query" {
            return;
        }
        let mut visitor = SqlVisitor(None);
        event.record(&mut visitor);
        if let Some(text) = visitor.0 {
            use sha2::{Digest, Sha256};
            let sql = text.trim_matches('"').to_owned();
            let sha: String = Sha256::digest(sql.as_bytes()).iter().map(|b| format!("{b:02x}")).collect();
            let mut captured = CAPTURED_SQL.lock().unwrap();
            if !captured.iter().any(|(known, _)| *known == sha) {
                captured.push((sha, sql));
            }
        }
    }
}
fn install_sql_capture() {
    use tracing_subscriber::layer::SubscriberExt;
    let subscriber = tracing_subscriber::registry()
        .with(SqlCapture)
        .with(tracing_subscriber::filter::Targets::new().with_target("sqlx::query", tracing::Level::TRACE));
    let _ = tracing::subscriber::set_global_default(subscriber);
}
fn emit_captured_sql() {
    for (sha, sql) in CAPTURED_SQL.lock().unwrap().iter() {
        eprintln!("t34-sql-text {}", json!({"sha":&sha[..12],"sql":sql}));
    }
}
