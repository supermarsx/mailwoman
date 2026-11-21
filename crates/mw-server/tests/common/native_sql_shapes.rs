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
