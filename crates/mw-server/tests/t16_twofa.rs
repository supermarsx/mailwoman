//! t16-e-e2e — LOGIN 2FA end-to-end over the REAL HTTP surface (26.16 headline).
//!
//! "unit-green != wired." The `mw-mfa` crate unit-tests the WebAuthn RP verify /
//! TOTP / recovery primitives; `twofa_routes.rs` unit-tests its pending-state map.
//! THIS leg proves the whole thing is WIRED: the login handler (`build_app`, proxy
//! mode over the in-repo `mw-mock-jmap` upstream, a real loopback socket) actually
//! routes a credential-validated login through the second-factor gate and withholds
//! the session until the factor clears — driven end-to-end over HTTP.
//!
//! Legs (each drives the real router; the sealed-at-rest leg drives the real store):
//!   * NO-DOWNGRADE (the security headline): once a factor is enrolled, a
//!     password-only login returns `twofaRequired` and sets **no** `mw_session`
//!     cookie — there is no path back to a password-only session.
//!   * TOTP: a second login clears with a live RFC-6238 code → session issued.
//!   * WEBAUTHN: a VIRTUAL ES256 authenticator (hand-built `navigator.credentials.get`
//!     ceremony) produces an assertion the server's `mw-mfa` RP verify accepts → the
//!     login completes. A tampered signature is refused.
//!   * RECOVERY: a break-glass code logs in and is SINGLE-USE (a second login with
//!     the same code is refused).
//!   * FORCED ENROLMENT: a 2FA-required (admin policy) but unenrolled user is forced
//!     to enrol on next login before any session is issued.
//!   * SEALED AT REST: on a real on-disk store the TOTP shared secret is sealed
//!     (never plaintext in the DB file) and recovery codes persist only as Argon2
//!     hashes. Exercised on SQLite (raw-file byte scan) AND live Postgres
//!     (`MW_E14_PG_DSN`, the 0015 SQL in the second dialect).
//!
//! Run:
//!   cargo test -p mw-server --test t16_twofa -- --nocapture --test-threads=1
//!   docker compose -f docker-compose.ci.yml up -d --wait postgres
//!   MW_E14_PG_DSN=postgres://mailwoman:mailwoman@localhost:5432/mailwoman \
//!     cargo test -p mw-server --test t16_twofa -- --nocapture --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;

use base64::Engine as _;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use mw_mfa::recovery;
use mw_mfa::totp::{self, TotpParams};
use mw_server::{AppConfig, build_app};
use mw_store::{ServerKey, Store, WebauthnCredentialRow};

// A FIXED server key so the seed `Store` this test opens and the `Store` inside
// `build_app` seal/unseal 2FA secrets under the SAME key (an ephemeral generated key
// would make the login gate unable to read what we seeded).
const KEY_HEX: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0";

fn b64url(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn unique() -> String {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    format!(
        "{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW_TEST_INDEX</div>";

/// Spawn the mock JMAP upstream; return its base URL.
async fn spawn_mock() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    format!("http://{addr}")
}

/// Spawn `build_app` (proxy mode, fixed key) on a fresh DB at `db_path`; return its
/// bound address (so tests know the exact origin/rp-id the login gate derives).
async fn spawn_server(db_path: &str) -> SocketAddr {
    let base = PathBuf::from(db_path)
        .parent()
        .unwrap()
        .join(format!("web-{}", unique()));
    std::fs::create_dir_all(&base).unwrap();
    std::fs::write(base.join("index.html"), INDEX_HTML).unwrap();

    let config = AppConfig {
        db_path: db_path.to_string(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(base),
        cookie_secure: false,
        mode: mw_server::ServerMode::Proxy,
        hardening: mw_server::HardeningConfig::default(),
        security: mw_server::SecurityConfig::default(),
    };
    let app = build_app(config).await.expect("build_app");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

fn temp_db(tag: &str) -> String {
    let dir = std::env::temp_dir().join(format!("mw-t16-2fa-{tag}-{}", unique()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("mw.db").to_string_lossy().into_owned()
}

/// A fresh, non-cookie client (a distinct browser).
fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap()
}

async fn post(c: &reqwest::Client, url: String, body: Value) -> reqwest::Response {
    c.post(url).json(&body).send().await.unwrap()
}

fn login_body(mock: &str) -> Value {
    json!({
        "jmapUrl": mock,
        "username": mw_mock_jmap::USER,
        "password": mw_mock_jmap::PASS,
    })
}

/// Whether a response set an `mw_session` cookie (i.e. issued a real session).
fn set_session_cookie(resp: &reqwest::Response) -> bool {
    resp.headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .any(|c| c.starts_with("mw_session="))
}

/// Open a seed store on the same DB + key `build_app` uses.
async fn seed_store(db_path: &str) -> Store {
    Store::open(db_path, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .expect("open seed store")
}

// ── NO-DOWNGRADE + TOTP login ────────────────────────────────────────────────

#[tokio::test]
async fn enrolled_totp_forces_second_factor_and_no_password_only_downgrade() {
    let db = temp_db("totp");
    let mock = spawn_mock().await;
    let addr = spawn_server(&db).await;
    let base = format!("http://{addr}");

    // Seed a CONFIRMED TOTP secret for the mock account (as enrolment would).
    let secret = totp::generate_secret();
    let store = seed_store(&db).await;
    store
        .put_totp_secret(mw_mock_jmap::ACCOUNT_ID, &secret, true)
        .await
        .unwrap();

    // NO-DOWNGRADE: a password-only login is refused a session; it returns a challenge.
    let c = client();
    let resp = post(&c, format!("{base}/api/login"), login_body(&mock)).await;
    assert_eq!(resp.status(), 200, "login handler ran the gate");
    assert!(
        !set_session_cookie(&resp),
        "an enrolled user must NOT get a session cookie from password-only login (no downgrade)"
    );
    let body: Value = resp.json().await.unwrap();
    assert_eq!(
        body["twofaRequired"],
        json!(true),
        "gate demanded 2FA: {body}"
    );
    assert!(
        body["factors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f == "totp"),
        "totp offered as a factor: {body}"
    );
    let pending = body["pendingToken"].as_str().unwrap().to_string();

    // Before clearing the factor, no session exists → an authed call is 401.
    let pre = c
        .get(format!("{base}/api/account/2fa"))
        .send()
        .await
        .unwrap();
    assert_eq!(pre.status(), 401, "no session until the 2nd factor clears");

    // STEP 2: present a live TOTP code → the session is issued.
    let code = totp::totp_at(&secret, now_unix(), &TotpParams::default());
    let step2 = post(
        &c,
        format!("{base}/api/login/2fa"),
        json!({ "pendingToken": pending, "method": "totp", "code": code }),
    )
    .await;
    assert_eq!(step2.status(), 200, "correct TOTP completes the login");
    assert!(set_session_cookie(&step2), "the session cookie is now set");

    // The session is genuinely usable.
    let after = c
        .get(format!("{base}/api/account/2fa"))
        .send()
        .await
        .unwrap();
    assert_eq!(after.status(), 200, "session works after 2FA");
    let st: Value = after.json().await.unwrap();
    assert_eq!(
        st["totp"],
        json!(true),
        "status reflects the enrolled factor"
    );

    // A wrong code on a fresh login is refused (uniform 401, no session).
    let c2 = client();
    let b2: Value = post(&c2, format!("{base}/api/login"), login_body(&mock))
        .await
        .json()
        .await
        .unwrap();
    let bad = post(
        &c2,
        format!("{base}/api/login/2fa"),
        json!({ "pendingToken": b2["pendingToken"], "method": "totp", "code": "000000" }),
    )
    .await;
    assert_eq!(bad.status(), 401, "a wrong TOTP code is refused");
}

// ── VIRTUAL WEBAUTHN AUTHENTICATOR ───────────────────────────────────────────

#[tokio::test]
async fn virtual_webauthn_authenticator_asserts_and_logs_in() {
    let db = temp_db("webauthn");
    let mock = spawn_mock().await;
    let addr = spawn_server(&db).await;
    let base = format!("http://{addr}");
    // The login gate derives origin/rp-id from the request Host (same-origin).
    let origin = format!("http://{addr}");
    let rp_id = addr.ip().to_string(); // "127.0.0.1"

    // Register the virtual authenticator's credential directly in the store (the
    // enrolment ceremony is separately exercised; this leg proves ASSERTION verify).
    let authn = Es256Authenticator::new();
    let cred_id = b"virtual-cred-0001";
    let store = seed_store(&db).await;
    store
        .add_webauthn_credential(&WebauthnCredentialRow {
            credential_id: b64url(cred_id),
            account_id: mw_mock_jmap::ACCOUNT_ID.to_string(),
            cose_public_key: authn.cose(),
            sign_count: 5,
            transports: "internal".into(),
            label: "Virtual authenticator".into(),
            created_at: String::new(),
        })
        .await
        .unwrap();

    // Login → the gate returns a WebAuthn challenge (no session yet).
    let c = client();
    let body: Value = post(&c, format!("{base}/api/login"), login_body(&mock))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(body["twofaRequired"], json!(true));
    let pending = body["pendingToken"].as_str().unwrap().to_string();
    let challenge_b64 = body["webauthn"]["challenge"].as_str().unwrap().to_string();
    assert!(
        body["webauthn"]["credentialIds"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == &json!(b64url(cred_id))),
        "the registered credential id is offered: {body}"
    );
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(&challenge_b64)
        .unwrap();

    // The virtual authenticator signs authData ‖ SHA256(clientDataJSON) with a fresh
    // (advanced) counter — exactly a `navigator.credentials.get` result.
    let assertion = authn.assert(&rp_id, &origin, &challenge, /*counter=*/ 6);

    let good = post(
        &c,
        format!("{base}/api/login/2fa"),
        json!({
            "pendingToken": pending,
            "method": "webauthn",
            "credentialId": b64url(cred_id),
            "clientDataJson": b64url(&assertion.client_data_json),
            "authenticatorData": b64url(&assertion.authenticator_data),
            "signature": b64url(&assertion.signature),
        }),
    )
    .await;
    assert_eq!(
        good.status(),
        200,
        "the server's mw-mfa RP verify accepted the virtual assertion"
    );
    assert!(
        set_session_cookie(&good),
        "session issued after the assertion"
    );

    // The advanced counter was persisted (regression protection is live).
    assert_eq!(
        store
            .get_webauthn_credential(&b64url(cred_id))
            .await
            .unwrap()
            .unwrap()
            .sign_count,
        6
    );

    // A TAMPERED signature on a fresh login is refused.
    let c2 = client();
    let b2: Value = post(&c2, format!("{base}/api/login"), login_body(&mock))
        .await
        .json()
        .await
        .unwrap();
    let ch2 = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(b2["webauthn"]["challenge"].as_str().unwrap())
        .unwrap();
    let mut a2 = authn.assert(&rp_id, &origin, &ch2, 7);
    *a2.signature.last_mut().unwrap() ^= 0xff;
    let bad = post(
        &c2,
        format!("{base}/api/login/2fa"),
        json!({
            "pendingToken": b2["pendingToken"],
            "method": "webauthn",
            "credentialId": b64url(cred_id),
            "clientDataJson": b64url(&a2.client_data_json),
            "authenticatorData": b64url(&a2.authenticator_data),
            "signature": b64url(&a2.signature),
        }),
    )
    .await;
    assert_eq!(bad.status(), 401, "a tampered assertion is refused");
}

// ── RECOVERY CODE (single-use) ───────────────────────────────────────────────

#[tokio::test]
async fn recovery_code_logs_in_and_is_single_use() {
    let db = temp_db("recovery");
    let mock = spawn_mock().await;
    let addr = spawn_server(&db).await;
    let base = format!("http://{addr}");

    // Enrol a confirmed TOTP factor (so 2FA is engaged) + a known recovery code set.
    let secret = totp::generate_secret();
    let plain = recovery::generate_codes(recovery::DEFAULT_RECOVERY_CODES);
    let hashes: Vec<String> = plain.iter().map(|c| recovery::hash_code(c)).collect();
    let store = seed_store(&db).await;
    store
        .put_totp_secret(mw_mock_jmap::ACCOUNT_ID, &secret, true)
        .await
        .unwrap();
    store
        .add_recovery_codes(mw_mock_jmap::ACCOUNT_ID, &hashes)
        .await
        .unwrap();

    // First login: the break-glass recovery code clears the factor.
    let c = client();
    let b1: Value = post(&c, format!("{base}/api/login"), login_body(&mock))
        .await
        .json()
        .await
        .unwrap();
    assert!(
        b1["factors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f == "recovery"),
        "recovery offered as a factor: {b1}"
    );
    let step2 = post(
        &c,
        format!("{base}/api/login/2fa"),
        json!({ "pendingToken": b1["pendingToken"], "method": "recovery", "code": plain[0] }),
    )
    .await;
    assert_eq!(step2.status(), 200, "recovery code logs in");
    assert!(set_session_cookie(&step2));

    // Second login with the SAME code: it was consumed → refused.
    let c2 = client();
    let b2: Value = post(&c2, format!("{base}/api/login"), login_body(&mock))
        .await
        .json()
        .await
        .unwrap();
    let replay = post(
        &c2,
        format!("{base}/api/login/2fa"),
        json!({ "pendingToken": b2["pendingToken"], "method": "recovery", "code": plain[0] }),
    )
    .await;
    assert_eq!(
        replay.status(),
        401,
        "a recovery code is single-use (replay refused)"
    );

    // A different, still-unused code works.
    let ok = post(
        &c2,
        format!("{base}/api/login/2fa"),
        json!({ "pendingToken": b2["pendingToken"], "method": "recovery", "code": plain[1] }),
    )
    .await;
    assert_eq!(ok.status(), 200, "an unused recovery code still works");
}

// ── FORCED ENROLMENT (admin policy) ──────────────────────────────────────────

#[tokio::test]
async fn policy_required_user_is_forced_to_enrol_before_a_session() {
    let db = temp_db("forced");
    let mock = spawn_mock().await;
    let addr = spawn_server(&db).await;
    let base = format!("http://{addr}");

    // Admin require-2FA globally; the account has NO factor enrolled.
    let store = seed_store(&db).await;
    store
        .set_twofa_policy(&mw_store::TwofaPolicyRow {
            scope_kind: "global".into(),
            scope_value: String::new(),
            require_2fa: true,
            updated_by: "admin@example.org".into(),
            updated_at: String::new(),
        })
        .await
        .unwrap();

    // Login: the gate forces enrolment (still no session).
    let c = client();
    let resp = post(&c, format!("{base}/api/login"), login_body(&mock)).await;
    assert!(
        !set_session_cookie(&resp),
        "no session while enrolment is pending"
    );
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["twofaRequired"], json!(true));
    assert_eq!(
        body["enrollmentRequired"],
        json!(true),
        "a required-but-unenrolled user is forced to enrol: {body}"
    );
    let pending = body["pendingToken"].as_str().unwrap().to_string();

    // Forced TOTP enrolment over the pending token: begin → confirm → session.
    let begin: Value = post(
        &c,
        format!("{base}/api/login/2fa/enroll/totp/begin"),
        json!({ "pendingToken": pending }),
    )
    .await
    .json()
    .await
    .unwrap();
    let secret_b32 = begin["secret"].as_str().unwrap();
    let secret = totp::base32_decode(secret_b32).unwrap();
    let code = totp::totp_at(&secret, now_unix(), &TotpParams::default());

    let confirm = post(
        &c,
        format!("{base}/api/login/2fa/enroll/totp/confirm"),
        json!({ "pendingToken": pending, "code": code }),
    )
    .await;
    assert_eq!(confirm.status(), 200, "forced enrolment completes");
    assert!(
        set_session_cookie(&confirm),
        "session issued after forced enrolment"
    );
    let cbody: Value = confirm.json().await.unwrap();
    assert!(
        cbody["recoveryCodes"]
            .as_array()
            .map(|a| !a.is_empty())
            .unwrap_or(false),
        "recovery codes issued once at forced enrolment: {cbody}"
    );

    // The factor is now persisted + confirmed.
    assert!(
        store
            .get_totp_secret(mw_mock_jmap::ACCOUNT_ID)
            .await
            .unwrap()
            .unwrap()
            .confirmed
    );
}

// ── SEALED AT REST (real store: SQLite file scan + live Postgres round-trip) ──

#[tokio::test]
async fn totp_secret_and_recovery_are_sealed_on_the_real_store_sqlite() {
    // A real on-disk SQLite store (not in-memory): we scan the DB file's raw bytes.
    let db = temp_db("sealed");
    let store = Store::open(&db, ServerKey::generate()).await.unwrap();

    // A recognisable secret + a known recovery code plaintext.
    let secret: Vec<u8> = b"SEALED-TOTP-SECRET-01".to_vec();
    let plain_code = "SEAL7-EDCOD"; // shape doesn't matter for the scan
    store
        .put_totp_secret("acct-seal", &secret, true)
        .await
        .unwrap();
    store
        .add_recovery_codes("acct-seal", &[recovery::hash_code(plain_code)])
        .await
        .unwrap();

    // Round-trip still works (seal/unseal correct).
    assert_eq!(
        store
            .get_totp_secret("acct-seal")
            .await
            .unwrap()
            .unwrap()
            .secret,
        secret
    );

    // Force any WAL content to the main file, then scan every DB artefact on disk.
    drop(store);
    let bytes = read_all_db_bytes(&db);
    assert!(
        !contains(&bytes, &secret),
        "the raw TOTP secret must never appear in plaintext in the store file (sealed at rest)"
    );
    assert!(
        !contains(&bytes, plain_code.as_bytes()),
        "a recovery code plaintext must never be persisted (only its Argon2 hash)"
    );
    // The Argon2 hash marker IS present (codes persisted as hashes).
    assert!(
        contains(&bytes, b"$argon2"),
        "recovery codes are stored as Argon2 hashes"
    );
}

#[tokio::test]
async fn twofa_secrets_round_trip_on_live_postgres() {
    let Ok(dsn) = std::env::var("MW_E14_PG_DSN") else {
        eprintln!(
            "\n[t16 2fa SKIP] MW_E14_PG_DSN unset — 0015 SQL + seal not exercised on live Postgres.\n"
        );
        return;
    };
    let store = Store::open(&dsn, ServerKey::generate())
        .await
        .expect("open live Postgres store");
    let acct = format!("pg-2fa-{}", unique());
    let secret = totp::generate_secret();
    store.put_totp_secret(&acct, &secret, true).await.unwrap();
    // seal/unseal + the 0015 totp_secrets SQL round-trips in the Postgres dialect.
    let got = store.get_totp_secret(&acct).await.unwrap().unwrap();
    assert_eq!(got.secret, secret.to_vec());
    assert!(got.confirmed);

    // recovery_codes single-use SQL in the second dialect.
    let plain = recovery::generate_codes(3);
    let hashes: Vec<String> = plain.iter().map(|c| recovery::hash_code(c)).collect();
    store.add_recovery_codes(&acct, &hashes).await.unwrap();
    let unused = store.list_unused_recovery_codes(&acct).await.unwrap();
    let hit = unused
        .iter()
        .find(|h| recovery::verify_code(&plain[0], h))
        .unwrap()
        .clone();
    assert!(store.consume_recovery_code(&acct, &hit).await.unwrap());
    assert!(
        !store.consume_recovery_code(&acct, &hit).await.unwrap(),
        "consume is single-use on Postgres too"
    );
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && haystack.windows(needle.len()).any(|w| w == needle)
}

/// Concatenate the SQLite main DB file plus any `-wal`/`-journal` sidecars so the scan
/// sees content that has not yet been checkpointed into the main file.
fn read_all_db_bytes(db_path: &str) -> Vec<u8> {
    let mut out = Vec::new();
    for suffix in ["", "-wal", "-journal", "-shm"] {
        let p = format!("{db_path}{suffix}");
        if let Ok(b) = std::fs::read(&p) {
            out.extend_from_slice(&b);
        }
    }
    out
}

// ── virtual ES256 WebAuthn authenticator ─────────────────────────────────────
//
// Hand-builds the COSE key + a `navigator.credentials.get` assertion the same way a
// real platform authenticator does, so the server's `mw-mfa` RP verify runs against a
// genuine ceremony (not a canned vector). ES256 (p256) is already a dev-dep.

struct Assertion {
    client_data_json: Vec<u8>,
    authenticator_data: Vec<u8>,
    signature: Vec<u8>,
}

struct Es256Authenticator {
    signing: p256::ecdsa::SigningKey,
    x: [u8; 32],
    y: [u8; 32],
}

impl Es256Authenticator {
    fn new() -> Self {
        let signing = p256::ecdsa::SigningKey::from_slice(&[0x24u8; 32]).unwrap();
        let ep = signing.verifying_key().to_sec1_point(false);
        let b = ep.as_bytes(); // 0x04 || X(32) || Y(32)
        let mut x = [0u8; 32];
        let mut y = [0u8; 32];
        x.copy_from_slice(&b[1..33]);
        y.copy_from_slice(&b[33..65]);
        Es256Authenticator { signing, x, y }
    }

    /// COSE_Key CBOR for the EC2/P-256 public key (RFC 9052 integer labels).
    fn cose(&self) -> Vec<u8> {
        let mut v = vec![0xa5]; // map(5)
        v.extend_from_slice(&[0x01, 0x02]); // kty(1) = EC2(2)
        v.extend_from_slice(&[0x03, 0x26]); // alg(3) = ES256(-7)
        v.extend_from_slice(&[0x20, 0x01]); // crv(-1) = P-256(1)
        v.extend_from_slice(&[0x21, 0x58, 0x20]); // x(-2) = bstr(32)
        v.extend_from_slice(&self.x);
        v.extend_from_slice(&[0x22, 0x58, 0x20]); // y(-3) = bstr(32)
        v.extend_from_slice(&self.y);
        v
    }

    /// Produce a signed assertion (UP|UV flags) for the challenge/origin/rp-id.
    fn assert(&self, rp_id: &str, origin: &str, challenge: &[u8], counter: u32) -> Assertion {
        // authenticatorData = SHA256(rpId) ‖ flags ‖ signCount (no attested data).
        let mut authn_data = Vec::new();
        authn_data.extend_from_slice(&Sha256::digest(rp_id.as_bytes()));
        authn_data.push(0x01 | 0x04); // UP | UV
        authn_data.extend_from_slice(&counter.to_be_bytes());

        let cdj = format!(
            r#"{{"type":"webauthn.get","challenge":"{}","origin":"{origin}"}}"#,
            b64url(challenge)
        )
        .into_bytes();

        // The signed message is authData ‖ SHA256(clientDataJSON), ES256 DER signature.
        let mut message = authn_data.clone();
        message.extend_from_slice(&Sha256::digest(&cdj));
        let signature = {
            use p256::ecdsa::signature::Signer;
            let sig: p256::ecdsa::Signature = self.signing.sign(&message);
            sig.to_der().as_bytes().to_vec()
        };
        Assertion {
            client_data_json: cdj,
            authenticator_data: authn_data,
            signature,
        }
    }
}
