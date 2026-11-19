//! Opt-in real middleware/router HTTP smoke; upstream is the in-repo JMAP mock.
//! No production hosts, upstream credentials, message sends or public listeners.
use mw_server::{AppConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::{ServerKey, Store, settings_policy::SettingPolicyRow};
use serde_json::json;
use std::{io::Write, net::SocketAddr, path::PathBuf, time::Duration};

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

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "opt-in isolated real HTTP load; requires Node, bounded to 90 seconds"]
async fn real_http_pipeline_load_fixture() {
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
    let (upstream, upstream_task) = serve(mw_mock_jmap::router()).await;
    for protection in [false, true] {
        let label = if protection {
            "protection-on"
        } else {
            "protection-off"
        };
        let path = dir.join(format!("{label}.sqlite"));
        let key = ServerKey::generate();
        let key_hex = key.to_hex();
        let store = Store::open(path.to_str().unwrap(), key).await.unwrap();
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
        let (router, _) = build_app_full(
            AppConfig {
                db_path: path.to_string_lossy().into_owned(),
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
        // The handoff contains a short-lived session secret, never stdout or a
        // report. Unix mode is explicit; Windows inherits the user's temp ACL.
        let handoff = dir.join(format!("{label}.private.json"));
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&handoff).unwrap();
        file.write_all(
            &serde_json::to_vec(&json!({"base":base,"cookie":cookies,"protection":protection}))
                .unwrap(),
        )
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
        let outcome = tokio::time::timeout(Duration::from_secs(90), command.output()).await;
        std::fs::remove_file(&handoff).unwrap();
        server_task.abort();
        let _ = server_task.await;
        store.close().await;
        let output = outcome
            .expect("bounded Node fixture deadline")
            .expect("Node must be installed");
        assert!(
            output.status.success(),
            "load driver failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    upstream_task.abort();
    let _ = upstream_task.await;
    // Keep temp database artifacts for diagnosis; no credentials remain in the
    // handoff directory. OS temp cleanup may reclaim these disposable files.
}
