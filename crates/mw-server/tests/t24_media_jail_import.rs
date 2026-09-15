//! t24-e10 — `.oft`/`.msg` import through the **kernel-jailed** render worker.
//!
//! The compound-file parse has no in-process path: `/api/import/oft` hands it to the
//! `mw-render` worker, which confines itself (seccomp + Landlock + rlimits) and runs
//! the parse inside its wasmtime media jail. From 26.16 until t24-e10 that worker was
//! SIGSYS-killed on every CFB job on Linux — wasmtime's copy-on-write memory init
//! calls `memfd_create`, which the seccomp allowlist does not permit — so every
//! import on a Linux host answered 422.
//!
//! Nothing caught it because nothing ran the jailed worker: `cargo test --tests`
//! never builds another package's binary, and a test that looks for the worker and
//! skips when it is absent passes in exactly the environment the defect lived in. So
//! this test builds the worker itself, and on Linux a worker it cannot build or run
//! is a failure, never a skip. Off Linux there is no kernel jail to test, and it says
//! so through `common::gate::skip`.
//!
//! Run:
//!   cargo test -p mw-server --test t24_media_jail_import -- --nocapture --test-threads=1

mod common;

#[cfg(not(target_os = "linux"))]
#[test]
fn oft_import_runs_in_the_kernel_jailed_worker() {
    common::gate::skip(format_args!(
        "[t24 media jail] {}: the render worker has no kernel jail on this platform, so \
         the jailed CFB import this test exists for cannot run here. It runs on Linux CI.",
        std::env::consts::OS
    ));
}

#[cfg(target_os = "linux")]
mod linux {
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    use base64::Engine as _;
    use serde_json::{Value, json};

    use mw_server::{AppConfig, build_app};

    use super::common::test_db;

    /// Upper bound for the nested build. A cold build of the worker is a few
    /// minutes; a build that never finishes (for example, blocked on a lock some
    /// other process holds) must fail this test rather than hang the suite.
    const BUILD_DEADLINE: Duration = Duration::from_secs(20 * 60);

    /// Whether this test binary is coverage-instrumented. An instrumented worker
    /// cannot run under its own jail: the profile runtime writes at exit through a
    /// descriptor opened before the jail, and `RLIMIT_FSIZE=0` kills it with SIGXFSZ
    /// after it has already answered — which the server correctly reports as a
    /// failed worker.
    fn instrumented() -> bool {
        std::env::var_os("LLVM_PROFILE_FILE").is_some()
            || ["RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"]
                .iter()
                .any(|v| std::env::var(v).is_ok_and(|f| f.contains("instrument-coverage")))
    }

    /// Build `mw-render` and return the binary's path. Uses the same cargo, profile
    /// and (uninstrumented) target directory as this test binary, so under a plain
    /// `cargo test` the build reuses everything already compiled and only links the
    /// worker. Under coverage it builds an uninstrumented worker in a directory of
    /// its own, so the instrumented artifacts are not invalidated.
    fn build_worker() -> PathBuf {
        let exe = std::env::current_exe().expect("path to this test binary");
        // <target>/<profile>/deps/<this test>
        let profile_dir = exe
            .parent()
            .and_then(Path::parent)
            .expect("test binary lives in <target>/<profile>/deps");
        let target_dir = profile_dir.parent().expect("<target> above <profile>");
        let profile_name = profile_dir
            .file_name()
            .and_then(|n| n.to_str())
            .expect("profile directory name");
        let profile = if profile_name == "debug" {
            "dev"
        } else {
            profile_name
        };

        let cargo = std::env::var_os("CARGO").unwrap_or_else(|| env!("CARGO").into());
        let mut cmd = Command::new(cargo);
        cmd.current_dir(env!("CARGO_MANIFEST_DIR"))
            .args(["build", "--frozen", "-p", "mw-render", "--bin", "mw-render"])
            .args(["--profile", profile])
            .env_remove("CARGO_TARGET_DIR")
            .env_remove("CARGO_BUILD_TARGET_DIR")
            .stdout(Stdio::null());
        let out_dir = if instrumented() {
            for var in ["RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "LLVM_PROFILE_FILE"] {
                cmd.env_remove(var);
            }
            // cargo-llvm-cov instruments through a rustc wrapper (itself) configured
            // by its own variables, not through RUSTFLAGS, so those go too.
            if std::env::var_os("__CARGO_LLVM_COV_RUSTC_WRAPPER").is_some() {
                cmd.env_remove("RUSTC_WRAPPER");
            }
            for (var, _) in std::env::vars_os() {
                let name = var.to_string_lossy();
                if name.starts_with("__CARGO_LLVM_COV") || name.starts_with("CARGO_LLVM_COV") {
                    cmd.env_remove(&var);
                }
            }
            target_dir.join("t24-uninstrumented-worker")
        } else {
            target_dir.to_path_buf()
        };
        cmd.arg("--target-dir").arg(&out_dir);
        // Cargo's output goes to a file, not a pipe: a cold build can print more than
        // a pipe buffer holds, and nothing reads the pipe until the build ends.
        std::fs::create_dir_all(&out_dir).expect("create the worker's target dir");
        let log_path = out_dir.join("t24-mw-render-build.log");
        let log = std::fs::File::create(&log_path).expect("create the build log");
        cmd.stderr(log);

        let started = Instant::now();
        let mut child = cmd.spawn().expect("spawn cargo to build mw-render");
        let status = loop {
            if let Some(status) = child.try_wait().expect("wait for the mw-render build") {
                break status;
            }
            if started.elapsed() > BUILD_DEADLINE {
                let _ = child.kill();
                panic!(
                    "building mw-render did not finish within {BUILD_DEADLINE:?}; the jailed \
                     CFB import cannot be tested without it"
                );
            }
            std::thread::sleep(Duration::from_millis(250));
        };
        let stderr = std::fs::read_to_string(&log_path).unwrap_or_default();
        assert!(
            status.success(),
            "building mw-render failed ({status}); on Linux this test must run the jailed \
             worker, not skip:\n{stderr}"
        );
        let worker = out_dir.join(profile_name).join("mw-render");
        assert!(
            worker.is_file(),
            "cargo reported success but {} does not exist",
            worker.display()
        );
        eprintln!(
            "[t24 media jail] worker {} built in {:.1?} (instrumented test binary: {})",
            worker.display(),
            started.elapsed(),
            instrumented()
        );
        worker
    }

    async fn spawn_mock() -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
        });
        format!("http://{addr}")
    }

    async fn spawn_server() -> String {
        let base = test_db::unique_dir("mw-t24-media-jail");
        let web = base.join("web");
        std::fs::create_dir_all(&web).unwrap();
        std::fs::write(
            web.join("index.html"),
            "<!doctype html><title>Mailwoman</title>",
        )
        .unwrap();
        let config = AppConfig {
            db_path: base.join("mw.db").to_string_lossy().into_owned(),
            server_key_hex: None,
            web_dir: Some(web),
            cookie_secure: false,
            mode: mw_server::ServerMode::Proxy,
            hardening: mw_server::HardeningConfig::default(),
            security: mw_server::SecurityConfig::default(),
        };
        let app = build_app(config).await.unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        format!("http://{addr}")
    }

    #[tokio::test]
    async fn oft_import_runs_in_the_kernel_jailed_worker() {
        // The point of this test is the JAILED worker. With the jail switched off it
        // would pass on a worker that seccomp would have killed.
        assert!(
            mw_sandbox::jail_expected(),
            "MW_RENDER_JAIL={:?} disables the render jail; this test exists to drive the \
             kernel-jailed worker, so unset it",
            std::env::var("MW_RENDER_JAIL").ok()
        );

        let worker = build_worker();
        // SAFETY: the only test in this binary, set before the server (which reads
        // it when the app is built) or any worker process exists.
        unsafe { std::env::set_var("MW_RENDER_BIN", &worker) };

        let mock = spawn_mock().await;
        let server = spawn_server().await;
        let c = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .unwrap();
        let login: Value = c
            .post(format!("{server}/api/login"))
            .json(&json!({
                "jmapUrl": mock,
                "username": mw_mock_jmap::USER,
                "password": mw_mock_jmap::PASS,
            }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(login["ok"], json!(true), "login: {login}");

        let raw =
            b"Subject: Weekly status template\r\n\r\n<p>Fill me in.</p><script>bad()</script>\r\n";
        let oft = mw_export::export_one(
            &mw_export::RawEmail::new(raw.to_vec()),
            mw_export::Format::Oft,
        )
        .expect("write .oft");
        let resp = c
            .post(format!("{server}/api/import/oft"))
            .json(&json!({
                "contentBase64": base64::engine::general_purpose::STANDARD.encode(&oft)
            }))
            .send()
            .await
            .unwrap();
        let status = resp.status();
        let out: Value = resp.json().await.unwrap();

        // 422 "could not import the template" here is the 26.16–26.19 defect: the
        // worker died by SIGSYS inside the jail.
        assert_eq!(
            status, 200,
            "the kernel-jailed render worker must import a valid .oft: {out}"
        );
        assert_eq!(out["subject"], json!("Weekly status template"), "{out}");
        let html = out["html"].as_str().unwrap();
        assert!(html.contains("Fill me in."), "body dropped: {html}");
        assert!(!html.contains("script"), "script survived: {html}");
    }
}
