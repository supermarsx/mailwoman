//! t29-e6 — the admin security policy is applied, over the real router.
//!
//! Each test signs in through the real `POST /admin/login` and writes the policy
//! through `PUT /admin/security-policy`, then observes the effect where a user or
//! a peer meets it: a TLS handshake against a real [`TlsListener`], the PHC string
//! a password change stores, an engine send that DLP blocks, `GET /api/policy`, a
//! mailbox login sent to two-factor enrolment, the admin cookie every gate reads.
//! Every negative is preceded by the same request succeeding.
//!
//! The TLS floor, the Argon2 cost, the admin cookie name and the environment are
//! process-wide, so the tests in this binary take turns ([`serial`]) and put the
//! defaults back when they finish.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::CryptoProvider;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, SignatureScheme, SupportedProtocolVersion};
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

use mw_engine::account::AccountRuntime;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, EngineError, Flag, MailboxDelta, MessageRef,
    MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, SyncCursor, WatchHandle,
};
use mw_engine::{Engine, MailSubmitter};
use mw_server::tls::{self, MinTls};
use mw_server::{
    AppConfig, HardeningConfig, SecurityConfig, ServerMode, TlsConfig, TlsListener, V6Config,
    build_app_full,
};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

mod common;
use common::test_db;

const KEY_HEX: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

/// Every variable a test here sets; cleared at the start of each test.
const VARS: &[&str] = &[
    "MW_ADMIN_MIN_TLS",
    "MW_ADMIN_REQUIRE_2FA",
    "MW_ADMIN_SESSION_COOKIE",
    "MW_ADMIN_PORT",
    "MW_DLP_RULES",
];

// ── serialisation of process-wide state ─────────────────────────────────────────

struct Turn(#[allow(dead_code)] tokio::sync::MutexGuard<'static, ()>);

impl Drop for Turn {
    /// Put the process-wide settings back, whatever the test did.
    fn drop(&mut self) {
        clear_env();
        tls::apply_min_tls(MinTls::V12);
        mw_passwd::set_argon_cost(mw_passwd::ArgonCost::DEFAULT);
        mw_mfa::recovery::set_cost(mw_mfa::recovery::ArgonCost::DEFAULT);
        mw_admin::config::set_session_cookie_name(None);
    }
}

async fn serial() -> Turn {
    static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
    let guard = LOCK.lock().await;
    clear_env();
    Turn(guard)
}

fn clear_env() {
    // SAFETY: only while `serial()`'s guard is held; the tests take turns.
    unsafe {
        for k in VARS {
            std::env::remove_var(k);
        }
    }
}

fn set_env(pairs: &[(&str, &str)]) {
    // SAFETY: only while `serial()`'s guard is held.
    unsafe {
        for (k, v) in pairs {
            std::env::set_var(k, v);
        }
    }
}

// ── server ───────────────────────────────────────────────────────────────────

struct Server {
    base: String,
    db: String,
    app: axum::Router,
}

async fn spawn_mock_jmap() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, mw_mock_jmap::router()).await;
    });
    format!("http://{addr}")
}

fn config(mock: &str, db: &str) -> AppConfig {
    let web = PathBuf::from(db).parent().unwrap().join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    AppConfig {
        db_path: db.to_string(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
            jmap_upstreams: Some(vec![mock.to_string()]),
            ..SecurityConfig::default()
        },
    }
}

fn admin_v6() -> V6Config {
    V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("hunter2".into()),
        redis_url: None,
    }
}

async fn build(tag: &str) -> anyhow::Result<Server> {
    let mock = spawn_mock_jmap().await;
    let db = test_db::unique_dir(tag)
        .join("mw.db")
        .to_string_lossy()
        .into_owned();
    let app = build_app_full(config(&mock, &db), admin_v6()).await?.0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let served = app.clone();
    tokio::spawn(async move {
        let _ = axum::serve(
            listener,
            served.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await;
    });
    Ok(Server {
        base: format!("http://{addr}"),
        db,
        app,
    })
}

async fn spawn(tag: &str) -> Server {
    build(tag).await.expect("server boots")
}

fn browser() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

fn plain() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().build().unwrap()
}

/// Sign in as the operator; returns the client (holding the cookie) and the
/// `Set-Cookie` value.
async fn admin(s: &Server) -> (reqwest::Client, String) {
    let c = browser();
    let r = c
        .post(format!("{}/admin/login", s.base))
        .json(&json!({ "username": "root", "password": "hunter2" }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "admin login");
    let set_cookie = r
        .headers()
        .get(reqwest::header::SET_COOKIE)
        .expect("admin login sets a cookie")
        .to_str()
        .unwrap()
        .to_string();
    (c, set_cookie)
}

/// Sign in to the mailbox; returns the client and the account id.
async fn mailbox(s: &Server) -> (reqwest::Client, String) {
    let c = browser();
    let r = c
        .post(format!("{}/api/login", s.base))
        .json(&json!({
            "username": mw_mock_jmap::USER,
            "password": mw_mock_jmap::PASS,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "mailbox login");
    let body: Value = r.json().await.unwrap();
    (c, body["accountId"].as_str().unwrap().to_string())
}

async fn get_policy(s: &Server, c: &reqwest::Client) -> Value {
    let r = c
        .get(format!("{}/admin/security-policy", s.base))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "GET /admin/security-policy");
    r.json().await.unwrap()
}

/// The full PUT body for the policy `GET` returned, with `changes` applied.
fn with(policy: &Value, changes: Value) -> Value {
    let mut body = json!({});
    for k in [
        "minTls",
        "require2fa",
        "argon2MCost",
        "argon2TCost",
        "argon2PCost",
        "capturePolicy",
        "maxSecurityFloor",
        "dlpRulesJson",
    ] {
        body[k] = policy[k].clone();
    }
    for (k, v) in changes.as_object().unwrap() {
        body[k] = v.clone();
    }
    body
}

async fn put_policy(s: &Server, c: &reqwest::Client, body: &Value) -> reqwest::Response {
    c.put(format!("{}/admin/security-policy", s.base))
        .json(body)
        .send()
        .await
        .unwrap()
}

/// Read the policy, apply `changes`, PUT it, and require a 200.
async fn change(s: &Server, c: &reqwest::Client, changes: Value) -> Value {
    let now = get_policy(s, c).await;
    let r = put_policy(s, c, &with(&now, changes)).await;
    let status = r.status();
    let body: Value = r.json().await.unwrap();
    assert_eq!(status, 200, "PUT /admin/security-policy: {body}");
    body
}

// ── TLS client ────────────────────────────────────────────────────────────────

static ONLY_TLS12: &[&SupportedProtocolVersion] = &[&rustls::version::TLS12];
static ONLY_TLS13: &[&SupportedProtocolVersion] = &[&rustls::version::TLS13];

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/tls")
        .join(name)
}

/// Trusts any certificate: the assertion is on the protocol version accepted.
#[derive(Debug)]
struct TrustAny(Arc<CryptoProvider>);

impl ServerCertVerifier for TrustAny {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

/// Serve the app under test on a real HTTPS listener (the production
/// [`TlsListener`]); returns its address.
async fn https(s: &Server) -> SocketAddr {
    let (listener, _resolver) = TlsListener::bind(
        "127.0.0.1:0",
        &TlsConfig::External {
            cert: fixture("cert1.pem"),
            key: fixture("key1.pem"),
        },
    )
    .await
    .unwrap();
    let addr = {
        use axum::serve::Listener;
        listener.local_addr().unwrap()
    };
    let app = s.app.clone();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    addr
}

/// Open a session as a client speaking only `versions` and fetch `/healthz` over
/// it (a TLS 1.3 refusal can arrive after the client's side of the handshake).
async fn handshake(
    addr: SocketAddr,
    versions: &'static [&'static SupportedProtocolVersion],
) -> std::io::Result<()> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ClientConfig::builder_with_provider(provider.clone())
        .with_protocol_versions(versions)
        .unwrap()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(TrustAny(provider)))
        .with_no_client_auth();
    let tcp = TcpStream::connect(addr).await?;
    let mut tls = TlsConnector::from(Arc::new(config))
        .connect(ServerName::try_from("localhost").unwrap(), tcp)
        .await?;
    tls.write_all(b"GET /healthz HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await?;
    let mut seen = Vec::new();
    tls.read_to_end(&mut seen).await?;
    let text = String::from_utf8_lossy(&seen);
    if text.starts_with("HTTP/1.1 200") && text.ends_with("ok") {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "unexpected response: {text:?}"
        )))
    }
}

// ── min TLS ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn the_minimum_tls_version_set_in_the_panel_is_what_the_https_listener_accepts() {
    let _turn = serial().await;
    let s = spawn("mw-t29e6-tls").await;
    let tls_addr = https(&s).await;
    let (c, _) = admin(&s).await;

    // Precondition: the default policy is 1.2, and a TLS 1.2-only client is served.
    assert_eq!(get_policy(&s, &c).await["minTls"], "1.2");
    handshake(tls_addr, ONLY_TLS12)
        .await
        .expect("a TLS 1.2 client is served under the default policy");

    let body = change(&s, &c, json!({ "minTls": "1.3" })).await;
    assert_eq!(body["minTls"], "1.3", "{body}");
    assert!(
        handshake(tls_addr, ONLY_TLS12).await.is_err(),
        "a new TLS 1.2 handshake is refused once the panel sets 1.3"
    );
    handshake(tls_addr, ONLY_TLS13)
        .await
        .expect("TLS 1.3 is still served");

    change(&s, &c, json!({ "minTls": "1.2" })).await;
    handshake(tls_addr, ONLY_TLS12)
        .await
        .expect("setting 1.2 again lets TLS 1.2 clients in");

    // A version rustls cannot speak is refused and nothing changes.
    let now = get_policy(&s, &c).await;
    let r = put_policy(&s, &c, &with(&now, json!({ "minTls": "1.1" }))).await;
    assert_eq!(r.status(), 422);
    assert_eq!(get_policy(&s, &c).await["minTls"], "1.2");
}

/// A policy written by another process (here the `mailwoman admin` CLI's façade
/// over the same database) is applied by the running server's 30-second refresh.
#[tokio::test]
async fn a_policy_written_by_another_process_is_applied_within_the_refresh_interval() {
    let _turn = serial().await;
    let s = spawn("mw-t29e6-refresh").await;
    assert_eq!(tls::min_tls(), MinTls::V12, "precondition");

    let cli = mw_server::build_admin(&s.db, Some(KEY_HEX)).await.unwrap();
    let mut write = {
        let e = cli.effective_security_policy().await.unwrap();
        mw_admin::SecurityPolicyWrite {
            policy: e.policy,
            require_2fa: e.require_2fa,
        }
    };
    write.policy.min_tls = mw_admin::MinTls::V13;
    cli.write_security_policy("cli", write).await.unwrap();
    // The CLI process applies nothing itself: the floor is unchanged until the
    // server's loop reads the row.
    assert_eq!(tls::min_tls(), MinTls::V12);

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(40);
    while tls::min_tls() != MinTls::V13 {
        assert!(
            std::time::Instant::now() < deadline,
            "the server did not apply the stored policy within 40 s"
        );
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }
}

// ── Argon2 ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn the_argon2_cost_set_in_the_panel_is_what_a_password_change_hashes_with() {
    let _turn = serial().await;
    let s = spawn("mw-t29e6-argon").await;
    let (user, account_id) = mailbox(&s).await;
    // The local password backend (the default) holds the account's hash in the
    // store; seed it the way an earlier release would have written it.
    let store = Store::open(&s.db, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap();
    let key = format!("passwd_local:{account_id}");
    store
        .set_setting(
            &key,
            &mw_passwd::hash_password_with(mw_mock_jmap::PASS, mw_passwd::ArgonCost::DEFAULT)
                .unwrap(),
        )
        .await
        .unwrap();
    let change_pw = |old: &str, new: &str| {
        let req = user
            .post(format!("{}/api/password", s.base))
            .json(&json!({ "oldPassword": old, "newPassword": new }));
        async move { req.send().await.unwrap() }
    };
    let stored = || async { store.get_setting(&key).await.unwrap().unwrap() };

    // Precondition: at the default policy a change hashes at 19456/2/1.
    const FIRST: &str = "first-new-password-1";
    const SECOND: &str = "second-new-password-2";
    assert_eq!(change_pw(mw_mock_jmap::PASS, FIRST).await.status(), 200);
    assert_eq!(mw_passwd::phc_cost(&stored().await), Some((19_456, 2, 1)));

    let (c, _) = admin(&s).await;
    let body = change(
        &s,
        &c,
        json!({ "argon2MCost": 32768, "argon2TCost": 3, "argon2PCost": 1 }),
    )
    .await;
    assert_eq!(body["argon2MCost"], 32768, "{body}");
    assert_eq!(
        mw_mfa::recovery::cost(),
        mw_mfa::recovery::ArgonCost::new(32_768, 3, 1).unwrap(),
        "recovery codes are hashed at the new cost too"
    );

    // A wrong current password is refused and stores nothing.
    let before = stored().await;
    assert_ne!(change_pw("not-the-password", SECOND).await.status(), 200);
    assert_eq!(stored().await, before);

    assert_eq!(change_pw(FIRST, SECOND).await.status(), 200);
    let phc = stored().await;
    assert_eq!(mw_passwd::phc_cost(&phc), Some((32_768, 3, 1)), "{phc}");
    assert!(phc.contains("m=32768,t=3"), "{phc}");

    // Out of range: refused, the cost in force is unchanged.
    let now = get_policy(&s, &c).await;
    let r = put_policy(&s, &c, &with(&now, json!({ "argon2MCost": 1024 }))).await;
    assert_eq!(r.status(), 422);
    assert_eq!(mw_passwd::argon_cost().m_kib(), 32_768);
}

// ── DLP ───────────────────────────────────────────────────────────────────────

struct NoopBackend;

#[async_trait]
impl AccountBackend for NoopBackend {
    async fn capabilities(&self) -> mw_engine::backend::Result<BackendCaps> {
        Ok(BackendCaps::default())
    }
    async fn list_mailboxes(&self) -> mw_engine::backend::Result<Vec<RawMailbox>> {
        Ok(Vec::new())
    }
    async fn sync_mailbox(
        &self,
        _m: &RawMailboxRef,
        c: &SyncCursor,
    ) -> mw_engine::backend::Result<MailboxDelta> {
        Ok(MailboxDelta {
            added: Vec::new(),
            flag_changes: Vec::new(),
            removed: Vec::new(),
            next_cursor: c.clone(),
        })
    }
    async fn fetch_raw(&self, _r: &[MessageRef]) -> mw_engine::backend::Result<Vec<RawMessage>> {
        Ok(Vec::new())
    }
    async fn store_flags(
        &self,
        _r: &[MessageRef],
        _a: &[Flag],
        _d: &[Flag],
    ) -> mw_engine::backend::Result<()> {
        Ok(())
    }
    async fn move_messages(
        &self,
        _r: &[MessageRef],
        _to: &RawMailboxRef,
    ) -> mw_engine::backend::Result<MoveOutcome> {
        Err(EngineError::Unsupported("noop".into()))
    }
    async fn append(
        &self,
        _m: &RawMailboxRef,
        _raw: &[u8],
        _f: &[Flag],
    ) -> mw_engine::backend::Result<MessageRef> {
        Err(EngineError::Unsupported("noop".into()))
    }
    async fn watch(&self, _sink: ChangeSink) -> mw_engine::backend::Result<WatchHandle> {
        Err(EngineError::Unsupported("noop".into()))
    }
}

#[derive(Default)]
struct Recording {
    sent: Mutex<Vec<Outgoing>>,
}

#[async_trait]
impl MailSubmitter for Recording {
    async fn submit(&self, msg: Outgoing) -> mw_engine::backend::Result<SubmissionResult> {
        let accepted = msg.rcpt_to.clone();
        self.sent.lock().unwrap().push(msg);
        Ok(SubmissionResult {
            accepted,
            rejected: Vec::new(),
        })
    }
}

/// An engine over the server's own database, as engine mode builds it, with a
/// recording submitter in place of SMTP.
async fn engine_over(db: &str) -> (Arc<Engine>, String, Arc<Recording>) {
    let store = Store::open(db, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap();
    let account_id = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example.org",
                port: 993,
                tls: "implicit",
                username: "me@example.org",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "me@example.org".into(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap();
    let engine = Arc::new(Engine::new(store));
    let rec = Arc::new(Recording::default());
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            Arc::new(NoopBackend) as Arc<dyn AccountBackend>,
            rec.clone() as Arc<dyn MailSubmitter>,
            "me@example.org",
        ),
    );
    (engine, account_id, rec)
}

/// Create a draft carrying a card number and submit it; returns the submission
/// response.
async fn send_card_number(engine: &Engine, account_id: &str) -> Value {
    let resp = engine
        .handle_jmap(
            account_id,
            &json!({ "methodCalls": [
                ["Email/set", { "create": { "draft": {
                    "from": [{ "email": "me@example.org" }],
                    "to": [{ "email": "friend@example.org" }],
                    "subject": "Payment",
                    "bodyValues": { "1": { "value": "Please charge 4111 1111 1111 1111 now." } },
                    "textBody": [{ "partId": "1", "type": "text/plain" }]
                } } }, "c1"],
                ["EmailSubmission/set", { "create": { "sub1": { "emailId": "#draft" } } }, "c2"]
            ] }),
        )
        .await;
    resp["methodResponses"][1][1].clone()
}

const PAN_RULE: &str = r#"[{"id":"panel-pan","name":"Block card numbers","enabled":true,
  "priority":10,"conditions":{"detectors":["pan"],"customRegex":null,"dictionaries":[],
  "attachmentTypes":[],"maxAttachmentSize":null,"recipientDomains":[],
  "recipientDomainMode":null,"classification":null},
  "action":"block","message":"Contains a card number."}]"#;

#[tokio::test]
async fn a_dlp_rule_stored_from_the_panel_blocks_an_engine_send() {
    let _turn = serial().await;
    let s = spawn("mw-t29e6-dlp").await;
    let (engine, account_id, rec) = engine_over(&s.db).await;
    let (c, _) = admin(&s).await;

    // Precondition: no rule anywhere, the send is delivered.
    let sub = send_card_number(&engine, &account_id).await;
    assert!(sub["created"]["sub1"].is_object(), "delivered: {sub}");
    assert_eq!(rec.sent.lock().unwrap().len(), 1);

    change(&s, &c, json!({ "dlpRulesJson": PAN_RULE })).await;
    let sub = send_card_number(&engine, &account_id).await;
    assert_eq!(sub["notCreated"]["sub1"]["type"], "dlpBlocked", "{sub}");
    assert_eq!(
        rec.sent.lock().unwrap().len(),
        1,
        "nothing more was submitted"
    );
    let audit = engine.store().list_dlp_audit(&account_id).await.unwrap();
    assert_eq!(audit.len(), 1, "one audit row");
    assert_eq!(audit[0].rule_id, "panel-pan");
    assert!(audit[0].blocked);

    // A signed-in client is told about the rule, marked as set from the panel.
    let (user, _) = mailbox(&s).await;
    let cfg: Value = user
        .get(format!("{}/api/security/dlp/config", s.base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(cfg["count"], 1, "{cfg}");
    assert_eq!(cfg["list"][0]["id"], "panel-pan");
    assert_eq!(cfg["list"][0]["source"], "admin");

    // Malformed rules are refused and the stored rules stay as they were.
    for bad in [
        "not json",
        "{}",
        r#"[{"id":"x"}]"#,
        &PAN_RULE.replace(r#""customRegex":null"#, r#""customRegex":"(""#),
    ] {
        let now = get_policy(&s, &c).await;
        let r = put_policy(&s, &c, &with(&now, json!({ "dlpRulesJson": bad }))).await;
        assert_eq!(r.status(), 422, "{bad}");
    }
    assert_eq!(get_policy(&s, &c).await["dlpRulesJson"], PAN_RULE);

    // Removing the rule lets the same send through.
    change(&s, &c, json!({ "dlpRulesJson": "[]" })).await;
    let sub = send_card_number(&engine, &account_id).await;
    assert!(sub["created"]["sub1"].is_object(), "delivered again: {sub}");
    assert_eq!(rec.sent.lock().unwrap().len(), 2);
}

/// A rule from `MW_DLP_RULES` is listed read-only beside the stored ones, and a
/// stored rule cannot replace it by reusing its id.
#[tokio::test]
async fn environment_dlp_rules_are_listed_separately_and_win_over_a_stored_rule() {
    let _turn = serial().await;
    let env_rule = PAN_RULE.replace("panel-pan", "env-pan");
    let mock = spawn_mock_jmap().await;
    let db = test_db::unique_dir("mw-t29e6-dlp-env")
        .join("mw.db")
        .to_string_lossy()
        .into_owned();
    let mut cfg = config(&mock, &db);
    cfg.security.dlp_rules = Some(env_rule.clone());
    let app = build_app_full(cfg, admin_v6()).await.unwrap().0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let served = app.clone();
    tokio::spawn(async move {
        let _ = axum::serve(
            listener,
            served.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await;
    });
    let s = Server {
        base: format!("http://{addr}"),
        db,
        app,
    };
    let (c, _) = admin(&s).await;
    let got = get_policy(&s, &c).await;
    assert_eq!(got["envDlpRules"][0]["id"], "env-pan", "{got}");
    assert_eq!(got["dlpRulesJson"], "[]");

    // A stored rule with the environment rule's id, and one of its own.
    let rule: Value = serde_json::from_str::<Value>(PAN_RULE).unwrap()[0].clone();
    let mut same_id = rule.clone();
    same_id["id"] = json!("env-pan");
    same_id["name"] = json!("Stored copy");
    let mut own = rule;
    own["name"] = json!("Stored copy");
    let stored = serde_json::to_string(&json!([same_id, own])).unwrap();
    change(&s, &c, json!({ "dlpRulesJson": stored })).await;

    let (user, _) = mailbox(&s).await;
    let cfg: Value = user
        .get(format!("{}/api/security/dlp/config", s.base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let list: Vec<(String, String, String)> = cfg["list"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["id"].as_str().unwrap().to_string(),
                r["source"].as_str().unwrap().to_string(),
                r["name"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    assert_eq!(
        list,
        vec![
            ("env-pan".into(), "env".into(), "Block card numbers".into()),
            ("panel-pan".into(), "admin".into(), "Stored copy".into()),
        ],
        "{cfg}"
    );
}

// ── floor and capture policy ─────────────────────────────────────────────────

#[tokio::test]
async fn the_floor_and_capture_policy_set_in_the_panel_reach_signed_in_clients() {
    let _turn = serial().await;
    let s = spawn("mw-t29e6-client").await;
    let url = format!("{}/api/policy", s.base);

    assert_eq!(
        plain().get(&url).send().await.unwrap().status(),
        401,
        "a session is required"
    );
    let (user, _) = mailbox(&s).await;
    let r = user.get(&url).send().await.unwrap();
    assert_eq!(r.status(), 200);
    assert_eq!(
        r.headers().get(reqwest::header::CACHE_CONTROL).unwrap(),
        "no-store"
    );
    let body: Value = r.json().await.unwrap();
    assert_eq!(
        body,
        json!({ "maxSecurityFloor": null, "capturePolicy": "off", "openTracking": false })
    );

    let (c, _) = admin(&s).await;
    change(
        &s,
        &c,
        json!({ "maxSecurityFloor": "plain-text", "capturePolicy": "protect" }),
    )
    .await;
    let body: Value = user.get(&url).send().await.unwrap().json().await.unwrap();
    assert_eq!(
        body,
        json!({ "maxSecurityFloor": "plain-text", "capturePolicy": "protect", "openTracking": false })
    );
}

// ── require two-factor ────────────────────────────────────────────────────────

async fn mailbox_login_body(s: &Server) -> Value {
    plain()
        .post(format!("{}/api/login", s.base))
        .json(&json!({
            "username": mw_mock_jmap::USER,
            "password": mw_mock_jmap::PASS,
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

fn global_rule(policies: &Value) -> Option<bool> {
    policies["policies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["scopeKind"] == "global")
        .map(|p| p["require2fa"].as_bool().unwrap())
}

#[tokio::test]
async fn require2fa_in_the_policy_is_the_global_two_factor_rule() {
    let _turn = serial().await;
    let s = spawn("mw-t29e6-2fa").await;
    let (c, _) = admin(&s).await;
    let rules = || async {
        c.get(format!("{}/admin/2fa/policy", s.base))
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap()
    };

    // Precondition: no rule, and a login without a factor is not sent to enrolment.
    assert_eq!(global_rule(&rules().await), None);
    let body = mailbox_login_body(&s).await;
    assert!(body.get("enrollmentRequired").is_none(), "{body}");

    change(&s, &c, json!({ "require2fa": true })).await;
    assert_eq!(global_rule(&rules().await), Some(true));
    let body = mailbox_login_body(&s).await;
    assert_eq!(body["enrollmentRequired"], true, "{body}");

    // The "Require two-factor" screen writes the same rule; the policy reads it.
    let r = c
        .post(format!("{}/admin/2fa/policy", s.base))
        .json(&json!({ "scopeKind": "global", "require2fa": false }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    assert_eq!(get_policy(&s, &c).await["require2fa"], false);
}

// ── environment overlay ───────────────────────────────────────────────────────

#[tokio::test]
async fn an_environment_pinned_minimum_tls_wins_and_the_panel_cannot_change_it() {
    let _turn = serial().await;
    set_env(&[
        ("MW_ADMIN_MIN_TLS", "1.3"),
        ("MW_ADMIN_REQUIRE_2FA", "true"),
    ]);
    let s = spawn("mw-t29e6-pin").await;
    let tls_addr = https(&s).await;
    let (c, _) = admin(&s).await;

    // Applied at start, without any PUT.
    assert!(
        handshake(tls_addr, ONLY_TLS12).await.is_err(),
        "MW_ADMIN_MIN_TLS=1.3 refuses TLS 1.2 from the start"
    );
    handshake(tls_addr, ONLY_TLS13)
        .await
        .expect("TLS 1.3 served");
    let got = get_policy(&s, &c).await;
    assert_eq!(got["minTls"], "1.3", "{got}");
    assert_eq!(got["require2fa"], true, "{got}");
    assert_eq!(got["pinnedByEnv"], json!(["minTls", "require2fa"]), "{got}");
    assert_eq!(
        mailbox_login_body(&s).await["enrollmentRequired"],
        true,
        "MW_ADMIN_REQUIRE_2FA is the rule the sign-in path reads"
    );

    // Changing a pinned field is refused, naming the variable; nothing is stored.
    let r = put_policy(&s, &c, &with(&got, json!({ "minTls": "1.2" }))).await;
    assert_eq!(r.status(), 409);
    let err: Value = r.json().await.unwrap();
    assert_eq!(err["variable"], "MW_ADMIN_MIN_TLS", "{err}");
    let r = put_policy(&s, &c, &with(&got, json!({ "require2fa": false }))).await;
    assert_eq!(r.status(), 409);
    assert!(
        handshake(tls_addr, ONLY_TLS12).await.is_err(),
        "still refused after the 409"
    );

    // Writing the pinned value with another change is accepted.
    let body = change(&s, &c, json!({ "capturePolicy": "protect" })).await;
    assert_eq!(body["capturePolicy"], "protect");
    assert_eq!(body["minTls"], "1.3");

    // The "Require two-factor" screen writes the same global rule: changing it is
    // refused there too, naming the variable, while the pinned value and a
    // per-domain rule are accepted.
    let two_fa = |body: Value| {
        let req = c.post(format!("{}/admin/2fa/policy", s.base)).json(&body);
        async move { req.send().await.unwrap() }
    };
    let r = two_fa(json!({ "scopeKind": "global", "require2fa": false })).await;
    assert_eq!(r.status(), 409, "the pinned global rule is not changed");
    let err: Value = r.json().await.unwrap();
    assert_eq!(err["variable"], "MW_ADMIN_REQUIRE_2FA", "{err}");
    assert_eq!(get_policy(&s, &c).await["require2fa"], true);
    let r = two_fa(json!({ "scopeKind": "global", "require2fa": true })).await;
    assert_eq!(r.status(), 200, "writing the pinned value is accepted");
    let r = two_fa(json!({
        "scopeKind": "domain", "scopeValue": "example.org", "require2fa": false
    }))
    .await;
    assert_eq!(r.status(), 200, "a per-domain rule is not pinned");
}

#[tokio::test]
async fn an_invalid_or_unsupported_admin_variable_stops_the_start_naming_it() {
    let _turn = serial().await;
    // Control: a clean environment starts.
    build("mw-t29e6-env-ok").await.expect("starts");

    for (var, value) in [
        ("MW_ADMIN_MIN_TLS", "banana"),
        ("MW_ADMIN_REQUIRE_2FA", "sometimes"),
        ("MW_ADMIN_SESSION_COOKIE", "mw_session"),
        ("MW_ADMIN_PORT", "9443"),
    ] {
        clear_env();
        set_env(&[(var, value)]);
        let err = match build("mw-t29e6-env-bad").await {
            Ok(_) => panic!("{var}={value} must stop the start"),
            Err(e) => e.to_string(),
        };
        assert!(err.contains(var), "{var}: {err}");
    }
}

// ── admin cookie name ─────────────────────────────────────────────────────────

#[tokio::test]
async fn a_renamed_admin_cookie_is_the_one_every_gate_and_the_csrf_guard_use() {
    let _turn = serial().await;
    set_env(&[("MW_ADMIN_SESSION_COOKIE", "corp_admin")]);
    let s = spawn("mw-t29e6-cookie").await;
    let (_c, set_cookie) = admin(&s).await;
    assert!(set_cookie.starts_with("corp_admin="), "{set_cookie}");
    let token = set_cookie
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1
        .to_string();
    let renamed = format!("corp_admin={token}");
    let old_name = format!("mw_admin_session={token}");

    for path in [
        "/admin/session",
        "/admin/security-policy",
        "/admin/2fa/policy",
        "/admin/sso",
        "/admin/plugins",
        "/admin/ui-plugins",
        "/admin/oauth-dcr",
        "/admin/maintenance/search-index",
    ] {
        let status = |cookie: &str| {
            let req = plain()
                .get(format!("{}{path}", s.base))
                .header(reqwest::header::COOKIE, cookie);
            async move { req.send().await.unwrap().status().as_u16() }
        };
        assert_ne!(status(&renamed).await, 401, "{path} accepts corp_admin");
        assert_eq!(
            status(&old_name).await,
            401,
            "{path} ignores mw_admin_session"
        );
    }

    // The CSRF guard counts the renamed cookie as one that authorises: beside a
    // bearer header, a cross-origin write is refused, not waved through.
    let policy: Value = plain()
        .get(format!("{}/admin/security-policy", s.base))
        .header(reqwest::header::COOKIE, &renamed)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let cross = |cookie: &str| {
        plain()
            .put(format!("{}/admin/security-policy", s.base))
            .header(reqwest::header::COOKIE, cookie)
            .header(reqwest::header::AUTHORIZATION, "Bearer not-a-token")
            .header(reqwest::header::ORIGIN, "https://attacker.example")
            .json(&with(&policy, json!({ "capturePolicy": "protect" })))
            .send()
    };
    assert_eq!(cross(&renamed).await.unwrap().status(), 403);
    // Control: under the old name the cookie authorises nothing, so the bearer
    // request passes the guard and the admin gate answers 401.
    assert_eq!(cross(&old_name).await.unwrap().status(), 401);
    let after: Value = plain()
        .get(format!("{}/admin/security-policy", s.base))
        .header(reqwest::header::COOKIE, &renamed)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        after["capturePolicy"], "off",
        "the refused write stored nothing"
    );
}
