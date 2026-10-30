//! t96: the console-managed country database, end to end against a local mock
//! HTTPS provider.
//!
//! The provider is a TLS origin on loopback whose certificate is issued for
//! `8.8.8.8` by a CA this test generates. The server reaches it as
//! `https://8.8.8.8/…` through an HTTP CONNECT egress route for the `geoip`
//! class, so the egress address policy is exercised rather than bypassed (the
//! same arrangement as the live Nextcloud spec). The CA is the custom source's
//! "extra CA certificate". Every `.mmdb` here is synthetic
//! (`mw_engine::security::geoip_fixture`); no real database is in the repository.
//!
//! One test function: the managed database is process-global, and the steps
//! build on each other (missing → download → corrupt/oversize/redirect refusals
//! → update → rollback → hot reload → schedule with backoff → licence key →
//! environment precedence).

use std::collections::HashMap;
use std::io::Write as _;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use mw_engine::security::geoip_fixture::{self, Entry};
use mw_engine::security::{CountryLookup, country_lookup};
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

mod common;

const SECRET_KEY: &str = "MwT96SecretLicenceKey_0123456789";
const KEY_HEX: &str = "96";

fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    e.write_all(bytes).unwrap();
    e.finish().unwrap()
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest;
    sha2::Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Two releases: v1 puts 89.114.0.0/16 in PT, v2 in ES. 81.2.69.0/24 is GB in both.
fn release(country: &str, epoch: u64) -> Vec<u8> {
    geoip_fixture::country_mmdb(
        &[
            Entry::v4("81.2.69.0/24", "GB"),
            Entry::v4("89.114.0.0/16", country),
        ],
        epoch,
    )
}

// ── the mock provider ─────────────────────────────────────────────────────────

#[derive(Clone)]
struct Origin {
    files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    hits: Arc<Mutex<Vec<String>>>,
}

async fn origin_handler(
    axum::extract::State(o): axum::extract::State<Origin>,
    uri: axum::http::Uri,
) -> Response {
    let path = uri.path().to_string();
    o.hits.lock().unwrap().push(path.clone());
    match path.as_str() {
        "/redirect-evil" => (
            StatusCode::FOUND,
            [(header::LOCATION, "https://evil.example/db.mmdb")],
        )
            .into_response(),
        "/redirect-same" => (StatusCode::FOUND, [(header::LOCATION, "/v2.mmdb")]).into_response(),
        "/huge" => {
            // 129 MiB of zeros, streamed without a Content-Length.
            let chunk = bytes::Bytes::from(vec![0u8; 1024 * 1024]);
            let stream = futures_util::stream::iter(
                (0..129).map(move |_| Ok::<_, std::io::Error>(chunk.clone())),
            );
            Body::from_stream(stream).into_response()
        }
        p => match o.files.lock().unwrap().get(p) {
            Some(bytes) => bytes.clone().into_response(),
            None => StatusCode::NOT_FOUND.into_response(),
        },
    }
}

struct Tls {
    ca_pem: String,
    config: Arc<rustls::ServerConfig>,
}

fn tls_for_8888() -> Tls {
    let ca_key = rcgen::KeyPair::generate().unwrap();
    let mut ca_params = rcgen::CertificateParams::new(Vec::<String>::new()).unwrap();
    ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    ca_params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "t96 test CA");
    let ca = rcgen::CertifiedIssuer::self_signed(ca_params, ca_key).unwrap();
    let leaf_key = rcgen::KeyPair::generate().unwrap();
    let mut leaf = rcgen::CertificateParams::new(Vec::<String>::new()).unwrap();
    leaf.subject_alt_names = vec![rcgen::SanType::IpAddress(IpAddr::V4(Ipv4Addr::new(
        8, 8, 8, 8,
    )))];
    let leaf = leaf.signed_by(&leaf_key, &ca).unwrap();
    let config = rustls::ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .unwrap()
    .with_no_client_auth()
    .with_single_cert(
        vec![leaf.der().clone()],
        rustls_pki_types::PrivateKeyDer::Pkcs8(leaf_key.serialize_der().into()),
    )
    .unwrap();
    Tls {
        ca_pem: ca.pem(),
        config: Arc::new(config),
    }
}

/// The TLS origin, and an HTTP CONNECT proxy that only tunnels to `8.8.8.8:443`
/// (which it maps to the origin). Returns the proxy's port.
async fn provider(origin: Origin, tls: &Tls, connects: Arc<Mutex<Vec<String>>>) -> u16 {
    let app = axum::Router::new()
        .fallback(origin_handler)
        .with_state(origin);
    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::clone(&tls.config));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin_addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        loop {
            let (tcp, _) = listener.accept().await.unwrap();
            let acceptor = acceptor.clone();
            let app = app.clone();
            tokio::spawn(async move {
                let Ok(stream) = acceptor.accept(tcp).await else {
                    return;
                };
                let service = hyper_util::service::TowerToHyperService::new(app);
                let _ = hyper_util::server::conn::auto::Builder::new(
                    hyper_util::rt::TokioExecutor::new(),
                )
                .serve_connection(hyper_util::rt::TokioIo::new(stream), service)
                .await;
            });
        }
    });
    let proxy = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = proxy.local_addr().unwrap().port();
    tokio::spawn(async move {
        loop {
            let (mut client, _) = proxy.accept().await.unwrap();
            let connects = Arc::clone(&connects);
            tokio::spawn(async move {
                let mut head = Vec::new();
                let mut byte = [0u8; 1];
                while !head.ends_with(b"\r\n\r\n") && head.len() < 8192 {
                    if client.read(&mut byte).await.unwrap_or(0) == 0 {
                        return;
                    }
                    head.push(byte[0]);
                }
                let line = String::from_utf8_lossy(&head)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_string();
                connects.lock().unwrap().push(line.clone());
                if !line.starts_with("CONNECT 8.8.8.8:443 ") {
                    let _ = client.write_all(b"HTTP/1.1 403 Forbidden\r\n\r\n").await;
                    return;
                }
                let Ok(mut upstream) = tokio::net::TcpStream::connect(origin_addr).await else {
                    return;
                };
                let _ = client
                    .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
                    .await;
                let _ = tokio::io::copy_bidirectional(&mut client, &mut upstream).await;
            });
        }
    });
    port
}

// ── log capture ───────────────────────────────────────────────────────────────

#[derive(Clone, Default)]
struct Logs(Arc<Mutex<Vec<u8>>>);
impl std::io::Write for Logs {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

async fn serve(router: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap()
    });
    format!("http://{address}")
}

async fn setting(store: &mw_store::Store, key: &str, value: Value) {
    store
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: key.into(),
            global_value: Some(value.to_string()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "fixture".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
}

async fn json_of(r: reqwest::Response) -> (StatusCode, Value, Option<String>) {
    let status = StatusCode::from_u16(r.status().as_u16()).unwrap();
    let etag = r
        .headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let body = r.json().await.unwrap_or(Value::Null);
    (status, body, etag)
}

struct Console {
    c: reqwest::Client,
    base: String,
}

impl Console {
    async fn get(&self) -> (Value, String) {
        let (s, b, e) = json_of(
            self.c
                .get(format!("{}/admin/security/geoip", self.base))
                .send()
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "{b}");
        (b, e.expect("etag"))
    }
    async fn post(&self, what: &str) -> (StatusCode, Value) {
        let (s, b, _) = json_of(
            self.c
                .post(format!("{}/admin/security/geoip/{what}", self.base))
                .send()
                .await
                .unwrap(),
        )
        .await;
        (s, b)
    }
    async fn configure(&self, body: Value) -> (StatusCode, Value) {
        let (_, etag) = self.get().await;
        let (s, b, _) = json_of(
            self.c
                .put(format!("{}/admin/security/geoip/config", self.base))
                .header("if-match", etag)
                .json(&body)
                .send()
                .await
                .unwrap(),
        )
        .await;
        (s, b)
    }
    async fn custom(&self, path: &str, checksum: &str, hosts: &[&str]) {
        let (s, b) = self
            .configure(json!({"provider":"custom","customUrl":format!("https://8.8.8.8{path}"),
                "customChecksumUrl":if checksum.is_empty(){String::new()}else{format!("https://8.8.8.8{checksum}")},
                "customRedirectHosts":hosts,"autoUpdate":true,"intervalDays":7}))
            .await;
        assert_eq!(s, StatusCode::OK, "configure {path}: {b}");
    }
    async fn lookup(&self, ip: &str) -> Value {
        let (s, b, _) = json_of(
            self.c
                .post(format!("{}/admin/security/geoip/lookup", self.base))
                .json(&json!({ "ip": ip }))
                .send()
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "{b}");
        b
    }
}

async fn wait_for(what: &str, mut check: impl AsyncFnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while Instant::now() < deadline {
        if check().await {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("timed out waiting for {what}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn country_database_lifecycle() {
    let logs = Logs::default();
    let writer = logs.clone();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_writer(move || writer.clone())
        .with_ansi(false)
        .init();

    let data_dir = common::test_db::unique_db_path("t96-geoip-dir").with_extension("d");
    // This executable holds one test; the environment is set before any server
    // exists, and only MW_GEOIP_DB changes later (the precedence leg, at the end).
    unsafe {
        std::env::remove_var("MW_GEOIP_DB");
        std::env::set_var("MW_GEOIP_DIR", &data_dir);
        std::env::set_var("MW_TRUSTED_PROXIES", "127.0.0.1");
        std::env::set_var("MW_FORWARDED_MODE", "xff");
    }
    mw_server::geoip::set_loop_interval(Duration::from_millis(150));

    let v1 = release("PT", 1_790_000_000);
    let v2 = release("ES", 1_790_100_000);
    let files = Arc::new(Mutex::new(HashMap::from([
        ("/v1.mmdb.gz".to_string(), gzip(&v1)),
        ("/v2.mmdb".to_string(), v2.clone()),
        (
            "/corrupt.mmdb.gz".to_string(),
            gzip(b"not a MaxMind database at all"),
        ),
        (
            "/asn.mmdb".to_string(),
            geoip_fixture::asn_mmdb(&[("81.2.69.0/24", 1)], 1),
        ),
        (
            "/sum-v1".to_string(),
            format!("{}  v1.mmdb.gz\n", sha256_hex(&gzip(&v1))).into_bytes(),
        ),
        (
            "/sum-bad".to_string(),
            format!("{}  v1.mmdb.gz\n", "0".repeat(64)).into_bytes(),
        ),
    ])));
    let tls = tls_for_8888();
    let connects = Arc::new(Mutex::new(Vec::new()));
    let origin = Origin {
        files: Arc::clone(&files),
        hits: Arc::new(Mutex::new(Vec::new())),
    };
    let proxy_port = provider(origin.clone(), &tls, Arc::clone(&connects)).await;

    let upstream = serve(mw_mock_jmap::router()).await;
    let db = common::test_db::unique_db_path("t96-geoip");
    let key = KEY_HEX.repeat(32);
    let store = mw_store::Store::open(
        db.to_str().unwrap(),
        mw_store::ServerKey::from_hex(&key).unwrap(),
    )
    .await
    .unwrap();
    setting(&store, "signin.retryCooldownSecs", json!(0)).await;
    setting(&store, "security.signin.geoScope", json!("mailbox")).await;
    setting(&store, "security.signin.geoMode", json!("allow")).await;
    setting(&store, "security.signin.geoCountries", json!(["PT"])).await;
    // Unmapped addresses allowed: a missing database must still refuse.
    setting(&store, "security.signin.geoUnknown", json!("allow")).await;
    let app = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into(),
            server_key_hex: Some(key),
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig {
                upstream_default: Some(upstream.clone()),
                jmap_upstreams: Some(vec![upstream]),
                ..Default::default()
            },
        },
        V6Config {
            admin_enabled: true,
            admin_username: Some("root".into()),
            admin_password: Some("hunter2".into()),
            redis_url: None,
        },
    )
    .await
    .unwrap()
    .0;
    let base = serve(app).await;
    let c = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .local_address(IpAddr::V4(Ipv4Addr::LOCALHOST))
        .build()
        .unwrap();
    let r = c
        .post(format!("{base}/admin/login"))
        .json(&json!({"username":"root","password":"hunter2"}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "admin login");
    let console = Console {
        c: c.clone(),
        base: base.clone(),
    };
    let mailbox = reqwest::Client::builder()
        .no_proxy()
        .local_address(IpAddr::V4(Ipv4Addr::LOCALHOST))
        .build()
        .unwrap();
    let sign_in = |ip: &'static str| {
        let mailbox = mailbox.clone();
        let base = base.clone();
        async move {
            mailbox
                .post(format!("{base}/api/login"))
                .header("x-forwarded-for", ip)
                .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS}))
                .send()
                .await
                .unwrap()
        }
    };

    // ── 1. missing: fail closed, even with unmapped addresses allowed ─────────
    let (status, _) = console.get().await;
    assert_eq!(status["state"], "missing", "{status}");
    assert_eq!(status["managedBy"], "console");
    assert_eq!(status["policyActive"], true);
    assert_eq!(status["config"]["provider"], "dbip", "DB-IP is the default");
    let refused = sign_in("89.114.7.1").await;
    assert_eq!(
        refused.status(),
        503,
        "fail closed while the database is missing"
    );
    let body: Value = refused.json().await.unwrap();
    assert_eq!(body["code"], "signinGeographyUnavailable", "{body}");
    let (_, dash, _) = json_of(
        c.get(format!("{base}/admin/dashboard"))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert!(
        dash["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w["code"] == "geoip-database-missing"),
        "dashboard warns: {dash}"
    );
    assert_eq!(
        console.lookup("89.114.7.1").await["database"],
        "unavailable"
    );

    // ── 2. changing the source needs step-up ─────────────────────────────────
    let (s, b) = console
        .configure(json!({"provider":"custom","customUrl":"https://8.8.8.8/v1.mmdb.gz","autoUpdate":true,"intervalDays":7}))
        .await;
    assert_eq!(s, StatusCode::FORBIDDEN, "{b}");
    assert_eq!(b["code"], "step_up_required", "{b}");
    common::auth::admin_step_up(&c, &format!("{base}/admin"), "hunter2", &[]).await;
    let (s, b) = console
        .configure(
            json!({"provider":"custom","customUrl":"https://8.8.8.8/v1.mmdb.gz",
            "customCaPem":tls.ca_pem,"autoUpdate":true,"intervalDays":7}),
        )
        .await;
    assert_eq!(s, StatusCode::OK, "{b}");
    assert_eq!(b["config"]["customCaConfigured"], true);

    // The egress route for the geoip class: CONNECT through the local proxy.
    let r = c
        .post(format!("{base}/admin/egress/proxies"))
        .json(
            &json!({"id":"t96-route","scheme":"http","host":"127.0.0.1","port":proxy_port,
            "scope":"global","trafficClasses":["geoip"]}),
        )
        .send()
        .await
        .unwrap();
    assert!(
        r.status().is_success(),
        "route: {}",
        r.text().await.unwrap()
    );
    let r = c
        .post(format!("{base}/admin/egress/proxies/t96-route/activate"))
        .send()
        .await
        .unwrap();
    assert!(
        r.status().is_success(),
        "activate: {}",
        r.text().await.unwrap()
    );

    // ── 3. download: valid gzip fixture → loaded, lookup, sign-in ────────────
    let (s, b) = console.post("download").await;
    assert_eq!(s, StatusCode::OK, "{b}");
    assert_eq!(b["state"], "loaded", "{b}");
    assert_eq!(b["outcome"]["result"], "installed");
    assert_eq!(b["database"]["source"], "custom");
    assert_eq!(b["database"]["networks"], 2);
    assert_eq!(b["database"]["buildEpoch"], 1_790_000_000u64);
    assert_eq!(b["database"]["sizeBytes"], v1.len());
    assert!(
        connects
            .lock()
            .unwrap()
            .iter()
            .all(|l| l.starts_with("CONNECT 8.8.8.8:443 "))
    );
    assert_eq!(console.lookup("89.114.7.1").await["country"], "PT");
    assert_eq!(console.lookup("::ffff:81.2.69.160").await["country"], "GB");
    assert_eq!(sign_in("89.114.7.1").await.status(), 200, "PT is allowed");
    let denied = sign_in("81.2.69.160").await;
    assert_eq!(denied.status(), 403, "GB is not in the allow list");
    assert!(data_dir.join("active.mmdb").exists());
    assert!(!data_dir.join("previous.mmdb").exists());

    // ── 4. refusals leave the loaded database in place ────────────────────────
    console.custom("/corrupt.mmdb.gz", "", &[]).await;
    let (s, b) = console.post("download").await;
    assert_eq!(s, StatusCode::UNPROCESSABLE_ENTITY, "corrupt: {b}");
    assert_eq!(b["code"], "geoipInvalidDatabase");
    console.custom("/asn.mmdb", "", &[]).await;
    let (s, b) = console.post("download").await;
    assert_eq!(s, StatusCode::UNPROCESSABLE_ENTITY, "no country data: {b}");
    assert!(
        b["detail"].as_str().unwrap().contains("no country codes"),
        "{b}"
    );
    console.custom("/huge", "", &[]).await;
    let (s, b) = console.post("download").await;
    assert_eq!(s, StatusCode::BAD_GATEWAY, "oversize: {b}");
    assert!(b["detail"].as_str().unwrap().contains("128 MB"), "{b}");
    console.custom("/redirect-evil", "", &[]).await;
    let (s, b) = console.post("download").await;
    assert_eq!(s, StatusCode::BAD_GATEWAY, "redirect to another host: {b}");
    assert!(
        b["detail"]
            .as_str()
            .unwrap()
            .contains("redirect to an unlisted host"),
        "{b}"
    );
    assert!(
        !connects
            .lock()
            .unwrap()
            .iter()
            .any(|l| l.contains("evil.example")),
        "the refused redirect was never followed"
    );
    console.custom("/v1.mmdb.gz", "/sum-bad", &[]).await;
    let (s, b) = console.post("download").await;
    assert_eq!(s, StatusCode::BAD_GATEWAY, "checksum mismatch: {b}");
    assert!(b["detail"].as_str().unwrap().contains("SHA-256"), "{b}");
    assert_eq!(
        console.lookup("89.114.7.1").await["country"],
        "PT",
        "still v1"
    );
    assert!(
        !data_dir.join("previous.mmdb").exists(),
        "nothing was replaced"
    );

    // A matching checksum is verified; the same release is "up to date".
    console.custom("/v1.mmdb.gz", "/sum-v1", &[]).await;
    let (s, b) = console.post("check").await;
    assert_eq!(s, StatusCode::OK, "{b}");
    assert_eq!(b["outcome"]["result"], "up-to-date", "{b}");

    // ── 5. update through a same-host redirect; previous kept ─────────────────
    console.custom("/redirect-same", "", &[]).await;
    let (s, b) = console.post("download").await;
    assert_eq!(s, StatusCode::OK, "{b}");
    assert_eq!(
        console.lookup("89.114.7.1").await["country"],
        "ES",
        "v2 in use"
    );
    assert!(data_dir.join("previous.mmdb").exists());
    assert_eq!(b["previous"]["buildEpoch"], 1_790_000_000u64);

    // ── 6. rollback toggles active and previous ──────────────────────────────
    let (s, b) = console.post("rollback").await;
    assert_eq!(s, StatusCode::OK, "{b}");
    assert_eq!(console.lookup("89.114.7.1").await["country"], "PT");
    let (s, _) = console.post("rollback").await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(console.lookup("89.114.7.1").await["country"], "ES");

    // ── 7. hot reload: no lookup sees a gap while databases swap ──────────────
    let stop = Arc::new(AtomicBool::new(false));
    let seen = Arc::new(AtomicUsize::new(0));
    let bad = Arc::new(Mutex::new(Vec::<String>::new()));
    let mut readers = Vec::new();
    for _ in 0..4 {
        let (stop, seen, bad) = (Arc::clone(&stop), Arc::clone(&seen), Arc::clone(&bad));
        readers.push(std::thread::spawn(move || {
            let ip = Some("89.114.7.1".parse().unwrap());
            while !stop.load(Ordering::Relaxed) {
                match country_lookup(ip) {
                    CountryLookup::Known(c) if c == "PT" || c == "ES" => {}
                    other => bad.lock().unwrap().push(format!("{other:?}")),
                }
                seen.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }
    for _ in 0..6 {
        let (s, b) = console.post("rollback").await;
        assert_eq!(s, StatusCode::OK, "{b}");
    }
    let (s, b) = console.post("download").await;
    assert_eq!(s, StatusCode::OK, "{b}");
    stop.store(true, Ordering::Relaxed);
    for r in readers {
        r.join().unwrap();
    }
    assert!(
        seen.load(Ordering::Relaxed) > 1000,
        "lookups ran during the swaps"
    );
    assert!(
        bad.lock().unwrap().is_empty(),
        "lost lookups: {:?}",
        bad.lock().unwrap()
    );

    // ── 8. schedule: a due check installs; a failure backs off; audited ──────
    console.custom("/v1.mmdb.gz", "", &[]).await;
    store
        .set_setting(
            "admin.geoip.schedule.v1",
            &json!({"nextCheckAt":0}).to_string(),
        )
        .await
        .unwrap();
    wait_for("the scheduled update", async || {
        console.lookup("89.114.7.1").await["country"] == "PT"
    })
    .await;
    let (status, _) = console.get().await;
    assert_eq!(status["schedule"]["failures"], 0, "{status}");
    let next = status["schedule"]["nextCheckAt"].as_i64().unwrap();
    assert!(
        next > chrono::Utc::now().timestamp() + 6 * 86_400,
        "next in 7 days: {status}"
    );
    console.custom("/corrupt.mmdb.gz", "", &[]).await;
    store
        .set_setting(
            "admin.geoip.schedule.v1",
            &json!({"nextCheckAt":0}).to_string(),
        )
        .await
        .unwrap();
    wait_for("the failed scheduled update", async || {
        console.get().await.0["schedule"]["failures"] == 1
    })
    .await;
    let (status, _) = console.get().await;
    let next = status["schedule"]["nextCheckAt"].as_i64().unwrap();
    let now = chrono::Utc::now().timestamp();
    assert!(
        (now + 3000..=now + 3700).contains(&next),
        "1 h backoff: {status}"
    );
    assert!(
        status["schedule"]["lastError"]
            .as_str()
            .unwrap()
            .contains("MaxMind DB")
    );
    assert_eq!(console.lookup("89.114.7.1").await["country"], "PT", "kept");

    // ── 9. upload: step-up, validation, install ──────────────────────────────
    common::auth::admin_step_up(&c, &format!("{base}/admin"), "hunter2", &[]).await;
    let r = c
        .post(format!("{base}/admin/security/geoip/upload"))
        .body(b"garbage".to_vec())
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 422, "an invalid upload is refused");
    let r = c
        .post(format!("{base}/admin/security/geoip/upload"))
        .body(gzip(&v2))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "{}", r.text().await.unwrap());
    assert_eq!(console.lookup("89.114.7.1").await["country"], "ES");
    assert_eq!(console.get().await.0["database"]["source"], "upload");

    // ── 10. the licence key never leaves the server ──────────────────────────
    common::auth::admin_step_up(&c, &format!("{base}/admin"), "hunter2", &[]).await;
    let (s, b) = console
        .configure(json!({"provider":"maxmind","licenceKey":SECRET_KEY,"autoUpdate":true,"intervalDays":7}))
        .await;
    assert_eq!(s, StatusCode::OK, "{b}");
    assert_eq!(b["config"]["licenceKeyConfigured"], true);
    assert!(!b.to_string().contains(SECRET_KEY));
    let (status, _) = console.get().await;
    assert!(!status.to_string().contains(SECRET_KEY));
    // The MaxMind host is not reachable through the test route: the error must
    // name the failure, not the URL that carries the key.
    let (s, b) = console.post("download").await;
    assert_eq!(s, StatusCode::BAD_GATEWAY, "{b}");
    assert!(!b.to_string().contains(SECRET_KEY), "{b}");
    assert!(
        !connects
            .lock()
            .unwrap()
            .iter()
            .any(|l| l.contains(SECRET_KEY)),
        "the proxy never sees the key"
    );
    let (_, audit, _) = json_of(
        c.get(format!("{base}/admin/audit?limit=500"))
            .send()
            .await
            .unwrap(),
    )
    .await;
    let audit_text = audit.to_string();
    assert!(!audit_text.contains(SECRET_KEY), "audit rows");
    for action in [
        "geoip-configured",
        "geoip-database-downloaded",
        "geoip-database-checked",
        "geoip-database-rolled-back",
        "geoip-database-uploaded",
        "geoip-database-update",
    ] {
        assert!(audit_text.contains(action), "audited: {action}");
    }
    // At rest: neither the database nor its WAL holds the key in clear.
    for file in [
        db.clone(),
        std::path::PathBuf::from(format!("{}-wal", db.display())),
    ] {
        if let Ok(bytes) = std::fs::read(&file) {
            assert!(
                !bytes
                    .windows(SECRET_KEY.len())
                    .any(|w| w == SECRET_KEY.as_bytes()),
                "{} holds the key in clear",
                file.display()
            );
        }
    }
    let captured = String::from_utf8_lossy(&logs.0.lock().unwrap()).to_string();
    assert!(!captured.contains(SECRET_KEY), "logs hold the key");
    assert_eq!(console.lookup("89.114.7.1").await["country"], "ES", "kept");

    // ── 11. MW_GEOIP_DB takes precedence and locks the page ───────────────────
    let env_file = data_dir.with_extension("env.mmdb");
    std::fs::write(&env_file, release("FR", 1)).unwrap();
    unsafe { std::env::set_var("MW_GEOIP_DB", &env_file) };
    let (status, _) = console.get().await;
    assert_eq!(status["managedBy"], "environment", "{status}");
    assert_eq!(status["writable"], false);
    assert_eq!(status["database"]["source"], "environment");
    assert_eq!(console.lookup("89.114.7.1").await["country"], "FR");
    let (s, b) = console.post("download").await;
    assert_eq!(s, StatusCode::CONFLICT, "{b}");
    assert_eq!(b["code"], "geoipManagedByEnvironment");
    let (s, _) = console.post("rollback").await;
    assert_eq!(s, StatusCode::CONFLICT);
    unsafe { std::env::remove_var("MW_GEOIP_DB") };
    assert_eq!(
        console.lookup("89.114.7.1").await["country"],
        "ES",
        "managed again"
    );
}
