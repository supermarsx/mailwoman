//! Opt-in real middleware/router HTTP smoke; upstream is the in-repo JMAP mock.
//! No production hosts, upstream credentials, message sends or public listeners.
use mw_server::{AppConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::{ServerKey, Store, settings_policy::SettingPolicyRow};
use serde_json::json;
use std::{io::Write, net::SocketAddr, path::PathBuf, time::Duration};

#[path = "common/request_load_diagnostics.rs"]
mod diagnostics_hook;
#[path = "common/request_load_control.rs"]
mod external_control;

async fn serve(router: axum::Router) -> (String, tokio::task::JoinHandle<()>) {
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
    (format!("http://{address}"), task)
}

#[test]
#[ignore = "opt-in isolated real HTTP load; short by default, explicitly bounded long campaigns"]
fn real_http_pipeline_load_fixture() {
    let workers: usize = std::env::var("MW_LOAD_FIXTURE_WORKERS")
        .ok()
        .map(|v| v.parse().expect("integer Tokio workers"))
        .unwrap_or(4);
    assert!(
        (1..=32).contains(&workers),
        "bounded Tokio worker count required"
    );
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(workers)
        .enable_all()
        .build()
        .unwrap()
        .block_on(run_real_http_pipeline_load_fixture(workers));
}
async fn run_real_http_pipeline_load_fixture(workers: usize) {
    assert_eq!(
        std::env::var("MW_RUN_HTTP_LOAD_FIXTURE").as_deref(),
        Ok("1"),
        "set MW_RUN_HTTP_LOAD_FIXTURE=1 to acknowledge local load"
    );
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let report_dir = std::env::var_os("MW_LOAD_REPORT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join(".orchestration/reports/real-http-load"));
    std::fs::create_dir_all(&report_dir).unwrap();
    let dir = std::env::temp_dir().join(format!("mw-real-load-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    let deadline: u64 = std::env::var("MW_LOAD_FIXTURE_DEADLINE_SECONDS")
        .ok()
        .map(|value| value.parse().expect("integer campaign deadline"))
        .unwrap_or_else(|| match std::env::var("MW_LOAD_FIXTURE_MODE").as_deref() {
            Ok("capacity") => 600,
            Ok("ramp") => 300,
            _ => 90,
        });
    assert!(
        (10..=1800).contains(&deadline),
        "campaign deadline outside10..1800 seconds"
    );
    let backend = std::env::var("MW_LOAD_FIXTURE_BACKEND").unwrap_or_else(|_| "sqlite".into());
    assert!(
        matches!(backend.as_str(), "sqlite" | "postgres"),
        "fixture backend must be sqlite/postgres"
    );
    let variants = match std::env::var("MW_LOAD_FIXTURE_PROTECTION").as_deref() {
        Ok("off") => vec![false],
        Ok("on") => vec![true],
        Ok("both") | Err(_) => vec![false, true],
        _ => panic!("fixture protection must be off/on/both"),
    };
    let (upstream, upstream_task) = serve(mw_mock_jmap::router()).await;
    for protection in variants {
        let label = if protection {
            "protection-on"
        } else {
            "protection-off"
        };
        let path = dir.join(format!("{label}.sqlite"));
        let mut pg_cleanup = None;
        let db_path = if backend == "postgres" {
            assert_eq!(
                std::env::var("MW_LOAD_FIXTURE_PG_ACK_SCHEMA").as_deref(),
                Ok("1"),
                "acknowledge isolated PostgreSQL schema creation"
            );
            let raw =
                std::env::var("MW_LOAD_FIXTURE_PG_DSN").expect("fixture PostgreSQL DSN required");
            let mut url = reqwest::Url::parse(&raw).expect("valid private fixture DSN");
            assert!(
                matches!(url.scheme(), "postgres" | "postgresql")
                    && matches!(
                        url.host_str(),
                        Some(
                            "127.0.0.1"
                                | "localhost"
                                | "::1"
                                | "postgres"
                                | "pg"
                                | "mw-load-pg"
                                | "host.docker.internal"
                        )
                    )
                    && url.path().trim_start_matches('/').starts_with("mw_fixture"),
                "PostgreSQL load fixture requires an acknowledged local mw_fixture database"
            );
            assert!(
                !url.query_pairs()
                    .any(|(key, _)| matches!(key.as_ref(), "options" | "search_path" | "schema")),
                "fixture DSN must not override schema selection"
            );
            let bootstrap = sqlx::postgres::PgPoolOptions::new()
                .max_connections(1)
                .connect(&raw)
                .await
                .expect("connect isolated PostgreSQL fixture");
            let schema = format!("mw_load_{}", uuid::Uuid::new_v4().simple());
            sqlx::query(&format!("CREATE SCHEMA {schema}"))
                .execute(&bootstrap)
                .await
                .unwrap();
            // A UUID-owned schema separates each campaign's migrations/data;
            // no pre-existing schema, database or table is dropped.
            url.query_pairs_mut()
                .append_pair("options", &format!("-csearch_path={schema}"));
            pg_cleanup = Some((bootstrap, schema));
            url.to_string()
        } else {
            path.to_string_lossy().into_owned()
        };
        let key = ServerKey::generate();
        let key_hex = key.to_hex();
        let store = Store::open(&db_path, key).await.unwrap();
        for (name, value) in [
            ("signin.retryCooldownSecs", json!(0)),
            (
                "security.protection.rateMode",
                json!(if protection { "observe" } else { "off" }),
            ),
            (
                "security.protection.adaptiveMode",
                json!(if protection { "observe" } else { "off" }),
            ),
            (
                "security.protection.probePathMode",
                json!(if protection { "enforce" } else { "off" }),
            ),
            (
                "security.protection.userAgentMode",
                json!(if protection { "observe" } else { "off" }),
            ),
        ] {
            store
                .put_setting_policy(&SettingPolicyRow {
                    key: name.into(),
                    global_value: Some(value.to_string()),
                    allow_user_override: false,
                    global_locked: false,
                    updated_by: "isolated-load-fixture".into(),
                    updated_at: 1,
                })
                .await
                .unwrap();
        }
        // Explicit fixture-only overrides isolate admission ceilings from raw
        // backend throughput. Defaults remain untouched unless requested.
        for (env_name, key, maximum) in [
            (
                "MW_LOAD_FIXTURE_JMAP_PER_ACCOUNT",
                "ratelimit.jmap.perAccountConcurrent",
                64u64,
            ),
            (
                "MW_LOAD_FIXTURE_JMAP_GLOBAL",
                "ratelimit.jmap.globalConcurrent",
                4096,
            ),
            ("MW_LOAD_FIXTURE_QUEUE_MS", "ratelimit.queueMs", 30000),
            (
                "MW_LOAD_FIXTURE_ACCOUNT_REQUESTS",
                "security.protection.accountRequests",
                1000000,
            ),
        ] {
            if let Ok(text) = std::env::var(env_name) {
                let value: u64 = text.parse().expect("integer fixture admission override");
                assert!(
                    value <= maximum,
                    "fixture admission override outside bounded range"
                );
                mw_server::settings_registry::validate(
                    mw_server::settings_registry::lookup(key).expect("registered fixture policy"),
                    &json!(value),
                    "/fixture-policy",
                )
                .expect("fixture override must be a supported administrator setting");
                store
                    .put_setting_policy(&SettingPolicyRow {
                        key: key.into(),
                        global_value: Some(value.to_string()),
                        allow_user_override: false,
                        global_locked: false,
                        updated_by: "isolated-load-fixture".into(),
                        updated_at: 1,
                    })
                    .await
                    .unwrap();
            }
        }
        let budgets =
            mw_server::settings_registry::effective_all(&store, &Default::default(), |def| {
                def.key.starts_with("ratelimit.")
                    || matches!(
                        def.key,
                        "security.protection.accountRequests"
                            | "security.protection.clientRequests"
                            | "security.protection.rateWindowSecs"
                    )
            })
            .await
            .unwrap()
            .into_iter()
            .map(|row| (row.key, row.value))
            .collect::<serde_json::Map<String, serde_json::Value>>();
        let (router, _) = build_app_full(
            AppConfig {
                db_path: db_path.clone(),
                server_key_hex: Some(key_hex),
                web_dir: None,
                cookie_secure: false,
                mode: ServerMode::Proxy,
                hardening: Default::default(),
                security: SecurityConfig {
                    upstream_default: Some(upstream.clone()),
                    jmap_upstreams: Some(vec![upstream.clone()]),
                    ..Default::default()
                },
            },
            V6Config::default(),
        )
        .await
        .unwrap();
        let diagnostic_path = format!("/__fixture-diagnostic-{}", uuid::Uuid::new_v4());
        let diagnostic_token = uuid::Uuid::new_v4().to_string();
        let diag_store = store.clone();
        let diag_token = diagnostic_token.clone();
        let diag_dir = dir.clone();
        let reset_token = diagnostic_token.clone();
        // These routes exist only in this ignored integration test, after the
        // production router's layers, and never ship in the application router.
        let router = router.route(
            &diagnostic_path,
            axum::routing::get(move |headers: axum::http::HeaderMap| {
                let store = diag_store.clone();
                let token = diag_token.clone();
                let directory = diag_dir.clone();
                async move {
                    if headers.get("x-fixture-token").and_then(|v| v.to_str().ok())
                        != Some(token.as_str())
                    {
                        return (
                            axum::http::StatusCode::FORBIDDEN,
                            axum::Json(json!({"error":"denied"})),
                        );
                    }
                    (
                        axum::http::StatusCode::OK,
                        axum::Json(diagnostics_hook::snapshot(&store, &directory).await),
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
        let (base, server_task) = serve(router).await;
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let response = client
            .post(format!("{base}/api/login"))
            .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200, "isolated fixture sign-in refused");
        let cookies = response
            .headers()
            .get_all("set-cookie")
            .iter()
            .map(|value| value.to_str().unwrap().split(';').next().unwrap())
            .collect::<Vec<_>>()
            .join("; ");
        let _ = response.bytes().await.unwrap();
        let handoff_document = json!({"base":base,"cookie":cookies,"protection":protection,"backend":backend,"buildProfile":if cfg!(debug_assertions){"debug"}else{option_env!("MW_CARGO_BUILD_MODE").unwrap_or("release")},"budgets":budgets,"runtimeWorkerThreads":workers,"diagnostics":{"path":diagnostic_path,"token":diagnostic_token}});
        let outcome = if std::env::var("MW_LOAD_FIXTURE_EXTERNAL_DRIVER").as_deref() == Ok("1") {
            external_control::run(&handoff_document, label, &report_dir, deadline).await
        } else {
            async {
                let handoff = dir.join(format!("{label}.private.json"));
                let mut options = std::fs::OpenOptions::new();
                options.write(true).create_new(true);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::OpenOptionsExt;
                    options.mode(0o600);
                }
                let mut file = options.open(&handoff).unwrap();
                file.write_all(&serde_json::to_vec(&handoff_document).unwrap())
                    .unwrap();
                file.sync_all().unwrap();
                drop(file);
                let mut command = tokio::process::Command::new("node");
                command
                    .arg(root.join("scripts/perf/http-real-fixture.mjs"))
                    .arg(&handoff)
                    .arg(report_dir.join(format!("{label}.json")))
                    .current_dir(&root)
                    .kill_on_drop(true);
                let outcome =
                    tokio::time::timeout(Duration::from_secs(deadline + 10), command.output())
                        .await;
                std::fs::remove_file(&handoff).unwrap();
                let output = outcome
                    .map_err(|_| "bounded Node fixture deadline exceeded".to_string())?
                    .map_err(|_| "Node must be installed".to_string())?;
                if !output.status.success() {
                    return Err(format!(
                        "load driver failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }
                Ok::<(), String>(())
            }
            .await
        };
        server_task.abort();
        let _ = server_task.await;
        store.close().await;
        if let Some((bootstrap, schema)) = pg_cleanup {
            sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
                .execute(&bootstrap)
                .await
                .unwrap();
            bootstrap.close().await;
        }
        assert!(outcome.is_ok(), "{}", outcome.err().unwrap_or_default());
    }
    upstream_task.abort();
    let _ = upstream_task.await;
    // Keep temp database artifacts for diagnosis; no credentials remain in the
    // handoff directory. OS temp cleanup may reclaim these disposable files.
}
