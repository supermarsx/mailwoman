//! t39 L4: the shipped fail2ban filter matches what the server writes.
//!
//! Two properties:
//!
//! * `deploy/fail2ban/filter.d/mailwoman.conf` carries exactly
//!   [`mw_admin::FAIL2BAN_FAILREGEX`], so the file and the code cannot drift.
//! * A rejected `POST /admin/login` against the real app writes a stdout line that
//!   the filter matches, with the client address as `<HOST>`. The line is captured
//!   through the same `tracing_subscriber::fmt::layer()` defaults `main.rs` installs
//!   (ANSI and all), so the bytes are the ones journald would store.
//!
//! With `MW_FAIL2BAN_SAMPLE_LOG=<path>` set, the captured log is also written to
//! that path, so `fail2ban-regex <path> deploy/fail2ban/filter.d/mailwoman.conf`
//! can be run against it (docs/deploy/fail2ban.md).

mod common;

use std::io::Write;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde_json::json;
use tokio::net::TcpListener;

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};

use common::test_db;

const FILTER: &str = include_str!("../../../deploy/fail2ban/filter.d/mailwoman.conf");
const KEY_HEX: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0";

/// The `failregex =` value of the shipped filter (single-line form).
fn shipped_failregex() -> String {
    let values: Vec<&str> = FILTER
        .lines()
        .filter_map(|l| l.strip_prefix("failregex"))
        .map(|rest| {
            rest.trim_start()
                .strip_prefix('=')
                .expect("failregex =")
                .trim()
        })
        .collect();
    assert_eq!(values.len(), 1, "the filter has one failregex line");
    values[0].to_string()
}

#[test]
fn shipped_filter_failregex_is_the_one_in_code() {
    assert_eq!(shipped_failregex(), mw_admin::FAIL2BAN_FAILREGEX);
}

/// Collects everything the fmt layer writes.
#[derive(Clone, Default)]
struct Captured(Arc<Mutex<Vec<u8>>>);

impl Write for Captured {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Captured {
    type Writer = Captured;
    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

async fn spawn_server(db: &str) -> String {
    let web = PathBuf::from(db)
        .parent()
        .unwrap()
        .join(format!("web-{}", test_db::unique_tag()));
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(
        web.join("index.html"),
        "<!doctype html><title>Mailwoman</title>",
    )
    .unwrap();
    let config = AppConfig {
        db_path: db.to_string(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig::default(),
    };
    let v6 = V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("hunter2".into()),
        redis_url: None,
    };
    let app = build_app_full(config, v6).await.expect("server boots").0;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await;
    });
    format!("http://{addr}")
}

#[tokio::test]
async fn a_rejected_admin_login_writes_a_line_the_shipped_filter_matches() {
    use tracing_subscriber::prelude::*;

    let captured = Captured::default();
    // Same layers as main.rs minus the OTLP bridge and the in-memory capture.
    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer().with_writer(captured.clone()));
    let _guard = tracing::subscriber::set_default(subscriber);

    let db = test_db::unique_dir("t39-fail2ban")
        .join("mw.db")
        .to_string_lossy()
        .into_owned();
    let base = spawn_server(&db).await;
    common::auth::no_signin_cooldown_at(&db, KEY_HEX).await;
    let c = reqwest::Client::builder().no_proxy().build().unwrap();
    for (user, pass) in [("root", "wrong"), ("nobody@example.com", "guess")] {
        let status = c
            .post(format!("{base}/admin/login"))
            .json(&json!({ "username": user, "password": pass }))
            .send()
            .await
            .unwrap()
            .status();
        assert_eq!(status, 401, "{user} is refused");
    }

    let log = String::from_utf8(captured.0.lock().unwrap().clone()).unwrap();
    if let Ok(path) = std::env::var("MW_FAIL2BAN_SAMPLE_LOG") {
        std::fs::write(&path, &log).expect("write sample log");
    }

    // `parse_host` is the code's expansion of FAIL2BAN_FAILREGEX, which the test
    // above pins to the shipped file; `fail2ban-regex` itself is run against
    // MW_FAIL2BAN_SAMPLE_LOG (docs/deploy/fail2ban.md).
    let hosts: Vec<String> = log
        .lines()
        .filter_map(mw_admin::banlist::parse_host)
        .collect();
    assert_eq!(hosts, ["127.0.0.1", "127.0.0.1"], "log was:\n{log}");
    assert!(log.contains("logname=root "), "log was:\n{log}");
    assert!(
        log.contains("logname=nobody@example.com "),
        "log was:\n{log}"
    );
}
