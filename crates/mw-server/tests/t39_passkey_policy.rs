//! t39-e6 (R2) — the administrator passkey policy over the real HTTP surface.
//!
//! A proxy-mode server over the in-repo mock JMAP upstream, a software ES256
//! authenticator that builds real `none` attestations and assertions, and the
//! policy written as registry rows (`twofa.passkey*`, `twofa.lastFactorRemoval`).
//!
//! Legs:
//!   * the registration options carry each policy value;
//!   * an attestation without UV, from the wrong attachment, or reported as not
//!     discoverable is refused with a named code and an audit line; a compliant
//!     one is stored;
//!   * an assertion without UV is refused when the policy requires UV and the
//!     credential has shown UV before;
//!   * grandfathering: an older passkey that does not meet a stricter policy
//!     still signs in under `twofa.passkeyLegacy = allow` (the default) and is
//!     marked in the status; under `refuse` it is refused with
//!     `passkeyPolicyLegacy`;
//!   * the last-factor rule: `block` refuses, `requireRecoveryCodes` refuses
//!     until recovery codes exist, `allow` (default) removes.
//!
//! Run: cargo test -p mw-server --test t39_passkey_policy -- --test-threads=1

use std::net::SocketAddr;

use base64::Engine as _;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use mw_mfa::totp::{self, TotpParams};
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::{ServerKey, Store, WebauthnCredentialRow};

mod common;
use common::test_db;

const KEY_HEX: &str = "39e6d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f00";

fn b64url(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn unb64(s: &str) -> Vec<u8> {
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(s)
        .unwrap()
}

struct Server {
    base: String,
    addr: SocketAddr,
    db: String,
}

impl Server {
    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }
    fn origin(&self) -> String {
        format!("http://{}", self.addr)
    }
    fn rp_id(&self) -> String {
        self.addr.ip().to_string()
    }
    async fn store(&self) -> Store {
        Store::open(&self.db, ServerKey::from_hex(KEY_HEX).unwrap())
            .await
            .unwrap()
    }
    /// Write a global, locked registry value (a string; stored as JSON).
    async fn policy(&self, key: &str, value: &str) {
        self.policy_json(key, &json!(value).to_string()).await;
    }
    async fn policy_json(&self, key: &str, value: &str) {
        self.store()
            .await
            .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
                key: key.into(),
                global_value: Some(value.into()),
                allow_user_override: false,
                global_locked: true,
                updated_by: "t39-e6-fixture".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
    }
    /// The `passkey-policy-refused` audit reasons, oldest first.
    async fn refusals(&self) -> Vec<Value> {
        let store = self.store().await;
        let mut out: Vec<Value> = store
            .list_audit(500)
            .await
            .unwrap()
            .into_iter()
            .filter(|r| r.action == "passkey-policy-refused")
            .map(|r| serde_json::from_str(&r.detail_json).unwrap())
            .collect();
        out.reverse();
        out
    }
}

async fn spawn() -> Server {
    spawn_on(None, None).await
}

async fn spawn_on(port: Option<u16>, web: Option<std::path::PathBuf>) -> Server {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mock = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    let dir = test_db::unique_dir("mw-t39-passkey-policy");
    std::fs::write(dir.join("index.html"), "<!doctype html><title>MW</title>").unwrap();
    let db = dir.join("mw.db").to_string_lossy().into_owned();
    let config = AppConfig {
        db_path: db.clone(),
        server_key_hex: Some(KEY_HEX.into()),
        web_dir: Some(web.unwrap_or(dir)),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
            jmap_upstreams: Some(vec![mock.clone()]),
            upstream_default: Some(mock),
            ..SecurityConfig::default()
        },
    };
    let v6 = V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("t39-e6-admin-pass".into()),
        redis_url: None,
    };
    let app = build_app_full(config, v6).await.expect("server boots").0;
    let server_listener = tokio::net::TcpListener::bind(("127.0.0.1", port.unwrap_or(0)))
        .await
        .unwrap();
    let addr = server_listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            server_listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    let server = Server {
        base: format!("http://{addr}"),
        addr,
        db,
    };
    // Sign-in pacing is not what these tests are about.
    server.policy_json("signin.retryCooldownSecs", "0").await;
    server
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap()
}

async fn post(c: &reqwest::Client, url: String, body: Value) -> (u16, Value) {
    let resp = c.post(url).json(&body).send().await.unwrap();
    let status = resp.status().as_u16();
    (status, resp.json::<Value>().await.unwrap_or(Value::Null))
}

async fn get(c: &reqwest::Client, url: String) -> Value {
    c.get(url).send().await.unwrap().json().await.unwrap()
}

async fn login(s: &Server, c: &reqwest::Client) -> Value {
    let (status, body) = post(
        c,
        s.url("/api/login"),
        json!({ "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS }),
    )
    .await;
    assert_eq!(status, 200, "login: {body}");
    body
}

async fn step_up_password(s: &Server, c: &reqwest::Client, code: Option<String>) {
    let mut body = json!({ "password": mw_mock_jmap::PASS });
    if let Some(code) = code {
        body["code"] = json!(code);
    }
    let (status, out) = post(c, s.url("/api/security/step-up"), body).await;
    assert_eq!(status, 200, "step-up: {out}");
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// A software ES256 authenticator: real `none` attestations and assertions.
struct Authenticator {
    signing: p256::ecdsa::SigningKey,
    id: Vec<u8>,
}

const UP: u8 = 0x01;
const UV: u8 = 0x04;
const AT: u8 = 0x40;

fn cbor_bytes(out: &mut Vec<u8>, b: &[u8]) {
    let n = b.len();
    if n < 24 {
        out.push(0x40 | n as u8);
    } else if n < 256 {
        out.extend_from_slice(&[0x58, n as u8]);
    } else {
        out.push(0x59);
        out.extend_from_slice(&(n as u16).to_be_bytes());
    }
    out.extend_from_slice(b);
}

fn cbor_text(out: &mut Vec<u8>, t: &str) {
    assert!(t.len() < 24);
    out.push(0x60 | t.len() as u8);
    out.extend_from_slice(t.as_bytes());
}

impl Authenticator {
    fn new(seed: u8, id: &[u8]) -> Self {
        Authenticator {
            signing: p256::ecdsa::SigningKey::from_slice(&[seed; 32]).unwrap(),
            id: id.to_vec(),
        }
    }

    fn cose(&self) -> Vec<u8> {
        let ep = self.signing.verifying_key().to_sec1_point(false);
        let b = ep.as_bytes();
        let mut v = vec![0xa5, 0x01, 0x02, 0x03, 0x26, 0x20, 0x01, 0x21, 0x58, 0x20];
        v.extend_from_slice(&b[1..33]);
        v.extend_from_slice(&[0x22, 0x58, 0x20]);
        v.extend_from_slice(&b[33..65]);
        v
    }

    /// `navigator.credentials.create` output for a challenge (fmt "none").
    fn attest(&self, s: &Server, challenge: &[u8], flags: u8) -> (String, String) {
        let mut auth = Vec::new();
        auth.extend_from_slice(&Sha256::digest(s.rp_id().as_bytes()));
        auth.push(flags | UP | AT);
        auth.extend_from_slice(&0u32.to_be_bytes());
        auth.extend_from_slice(&[0u8; 16]); // aaguid
        auth.extend_from_slice(&(self.id.len() as u16).to_be_bytes());
        auth.extend_from_slice(&self.id);
        auth.extend_from_slice(&self.cose());
        let mut att = vec![0xa3];
        cbor_text(&mut att, "fmt");
        cbor_text(&mut att, "none");
        cbor_text(&mut att, "attStmt");
        att.push(0xa0);
        cbor_text(&mut att, "authData");
        cbor_bytes(&mut att, &auth);
        let cdj = format!(
            r#"{{"type":"webauthn.create","challenge":"{}","origin":"{}"}}"#,
            b64url(challenge),
            s.origin()
        );
        (b64url(cdj.as_bytes()), b64url(&att))
    }

    /// `navigator.credentials.get` output, as the JSON fields the routes take.
    fn assert(&self, s: &Server, challenge: &[u8], counter: u32, flags: u8) -> Value {
        let mut auth = Vec::new();
        auth.extend_from_slice(&Sha256::digest(s.rp_id().as_bytes()));
        auth.push(flags | UP);
        auth.extend_from_slice(&counter.to_be_bytes());
        let cdj = format!(
            r#"{{"type":"webauthn.get","challenge":"{}","origin":"{}"}}"#,
            b64url(challenge),
            s.origin()
        );
        let mut message = auth.clone();
        message.extend_from_slice(&Sha256::digest(cdj.as_bytes()));
        let sig: p256::ecdsa::Signature = {
            use p256::ecdsa::signature::Signer;
            self.signing.sign(&message)
        };
        json!({
            "credentialId": b64url(&self.id),
            "clientDataJson": b64url(cdj.as_bytes()),
            "authenticatorData": b64url(&auth),
            "signature": b64url(sig.to_der().as_bytes()),
        })
    }

    async fn seed(&self, s: &Server, transports: &str) {
        s.store()
            .await
            .add_webauthn_credential(&WebauthnCredentialRow {
                credential_id: b64url(&self.id),
                account_id: mw_mock_jmap::ACCOUNT_ID.to_string(),
                cose_public_key: self.cose(),
                sign_count: 0,
                transports: transports.into(),
                label: "Older key".into(),
                created_at: String::new(),
            })
            .await
            .unwrap();
    }
}

/// Begin a registration as a signed-in, stepped-up user; returns the options.
async fn begin(s: &Server, c: &reqwest::Client) -> Value {
    let (status, body) = post(c, s.url("/api/account/2fa/passkey/begin"), json!({})).await;
    assert_eq!(status, 200, "begin: {body}");
    body
}

async fn finish(
    s: &Server,
    c: &reqwest::Client,
    a: &Authenticator,
    flags: u8,
    attachment: Option<&str>,
    transports: &str,
    rk: Option<bool>,
) -> (u16, Value) {
    let options = begin(s, c).await;
    let challenge = unb64(options["challenge"].as_str().unwrap());
    let (cdj, att) = a.attest(s, &challenge, flags);
    let mut body = json!({
        "clientDataJson": cdj,
        "attestationObject": att,
        "transports": transports,
        "label": "Policy test",
    });
    if let Some(attachment) = attachment {
        body["authenticatorAttachment"] = json!(attachment);
    }
    if let Some(rk) = rk {
        body["residentKey"] = json!(rk);
    }
    post(c, s.url("/api/account/2fa/passkey/finish"), body).await
}

#[tokio::test]
async fn registration_options_and_refusals_follow_the_policy() {
    let s = spawn().await;
    let c = client();
    login(&s, &c).await;
    step_up_password(&s, &c, None).await;

    // Defaults: what proxy mode always asked for.
    let options = begin(&s, &c).await;
    assert_eq!(options["userVerification"], "preferred");
    assert_eq!(options["residentKey"], "discouraged");
    assert!(options["authenticatorAttachment"].is_null(), "{options}");

    s.policy("twofa.passkeyAttachment", "cross-platform").await;
    s.policy("twofa.passkeyUserVerification", "discouraged")
        .await;
    s.policy("twofa.passkeyResidentKey", "preferred").await;
    let options = begin(&s, &c).await;
    assert_eq!(options["authenticatorAttachment"], "cross-platform");
    assert_eq!(
        options["authenticatorSelection"]["authenticatorAttachment"],
        "cross-platform"
    );
    assert_eq!(options["userVerification"], "discouraged");
    assert_eq!(options["residentKey"], "preferred");

    s.policy("twofa.passkeyAttachment", "platform").await;
    s.policy("twofa.passkeyUserVerification", "required").await;
    s.policy("twofa.passkeyResidentKey", "required").await;
    let options = begin(&s, &c).await;
    assert_eq!(options["authenticatorAttachment"], "platform");
    assert_eq!(options["userVerification"], "required");
    assert_eq!(options["residentKey"], "required");
    assert_eq!(
        options["authenticatorSelection"]["requireResidentKey"],
        true
    );

    let a = Authenticator::new(0x31, b"t39-e6-registration");
    // No UV flag in the signed authenticatorData.
    let (status, body) = finish(&s, &c, &a, 0, Some("platform"), "internal", Some(true)).await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["code"], "passkeyPolicyUserVerification");
    // A roaming key under a platform-only policy.
    let (status, body) = finish(&s, &c, &a, UV, Some("cross-platform"), "usb", Some(true)).await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["code"], "passkeyPolicyPlatform");
    // A browser that reports nothing about the attachment.
    let (status, body) = finish(&s, &c, &a, UV, None, "", None).await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["code"], "passkeyPolicyAttachmentUnknown");
    // Not stored on the authenticator.
    let (status, body) = finish(&s, &c, &a, UV, Some("platform"), "internal", Some(false)).await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["code"], "passkeyPolicyResidentKey");
    let status_json = get(&c, s.url("/api/account/2fa")).await;
    assert_eq!(status_json["passkeys"], json!([]), "nothing was stored");

    let reasons: Vec<Value> = s
        .refusals()
        .await
        .iter()
        .map(|d| d["reason"].clone())
        .collect();
    assert_eq!(
        reasons,
        vec![
            json!("passkeyPolicyUserVerification"),
            json!("passkeyPolicyPlatform"),
            json!("passkeyPolicyAttachmentUnknown"),
            json!("passkeyPolicyResidentKey"),
        ]
    );
    assert!(
        s.refusals()
            .await
            .iter()
            .all(|d| d["flow"] == "registration")
    );

    // A compliant one is stored and meets the policy.
    let (status, body) = finish(&s, &c, &a, UV, Some("platform"), "internal", Some(true)).await;
    assert_eq!(status, 200, "{body}");
    let status_json = get(&c, s.url("/api/account/2fa")).await;
    assert_eq!(
        status_json["passkeys"][0]["meetsPolicy"], true,
        "{status_json}"
    );
    assert_eq!(status_json["passkeyPolicy"]["attachment"], "platform");
    assert_eq!(status_json["passkeyPolicy"]["legacy"], "allow");
}

async fn second_factor(
    s: &Server,
    a: &Authenticator,
    counter: u32,
    flags: u8,
) -> (u16, Value, Value) {
    let c = client();
    let pending = login(s, &c).await;
    assert_eq!(pending["twofaRequired"], true, "{pending}");
    let challenge = unb64(pending["webauthn"]["challenge"].as_str().unwrap());
    let mut body = a.assert(s, &challenge, counter, flags);
    body["pendingToken"] = pending["pendingToken"].clone();
    body["method"] = json!("webauthn");
    let (status, out) = post(&c, s.url("/api/login/2fa"), body).await;
    (status, out, pending)
}

#[tokio::test]
async fn assertions_without_uv_are_refused_and_older_passkeys_are_grandfathered() {
    let s = spawn().await;
    // A passkey from before any policy: a USB key, not discoverable, never UV.
    let old = Authenticator::new(0x42, b"t39-e6-older-key");
    old.seed(&s, "usb").await;

    s.policy("twofa.passkeyAttachment", "platform").await;
    s.policy("twofa.passkeyUserVerification", "required").await;

    // Default legacy = allow: it still signs in without UV (it never had it)...
    let (status, out, pending) = second_factor(&s, &old, 1, 0).await;
    assert_eq!(pending["webauthn"]["userVerification"], "required");
    assert_eq!(status, 200, "grandfathered: {out}");
    // ...and Settings marks it.
    let c = client();
    let pending = login(&s, &c).await;
    let challenge = unb64(pending["webauthn"]["challenge"].as_str().unwrap());
    let mut body = old.assert(&s, &challenge, 2, 0);
    body["pendingToken"] = pending["pendingToken"].clone();
    body["method"] = json!("webauthn");
    assert_eq!(post(&c, s.url("/api/login/2fa"), body).await.0, 200);
    let status_json = get(&c, s.url("/api/account/2fa")).await;
    assert_eq!(
        status_json["passkeys"][0]["meetsPolicy"], false,
        "{status_json}"
    );
    assert_eq!(
        status_json["passkeys"][0]["policyGaps"],
        json!(["passkeyPolicyUserVerification", "passkeyPolicyPlatform"])
    );

    // Once it has shown UV it may not drop it again.
    assert_eq!(second_factor(&s, &old, 3, UV).await.0, 200);
    let (status, out, _) = second_factor(&s, &old, 4, 0).await;
    assert_eq!(status, 403, "{out}");
    assert_eq!(out["code"], "passkeyPolicyUserVerification");

    // legacy = refuse: the roaming key is refused by name, even with UV.
    s.policy("twofa.passkeyLegacy", "refuse").await;
    let (status, out, _) = second_factor(&s, &old, 5, UV).await;
    assert_eq!(status, 403, "{out}");
    assert_eq!(out["code"], "passkeyPolicyLegacy");

    // A tampered assertion is still the uniform 401, never a policy answer.
    let c = client();
    let pending = login(&s, &c).await;
    let challenge = unb64(pending["webauthn"]["challenge"].as_str().unwrap());
    let mut body = old.assert(&s, &challenge, 6, UV);
    let mut sig = unb64(body["signature"].as_str().unwrap());
    *sig.last_mut().unwrap() ^= 0xff;
    body["signature"] = json!(b64url(&sig));
    body["pendingToken"] = pending["pendingToken"].clone();
    body["method"] = json!("webauthn");
    let (status, out) = post(&c, s.url("/api/login/2fa"), body).await;
    assert_eq!(status, 401, "{out}");

    let reasons: Vec<(Value, Value)> = s
        .refusals()
        .await
        .iter()
        .map(|d| (d["flow"].clone(), d["reason"].clone()))
        .collect();
    assert_eq!(
        reasons,
        vec![
            (
                json!("second-factor"),
                json!("passkeyPolicyUserVerification")
            ),
            (json!("second-factor"), json!("passkeyPolicyLegacy")),
        ]
    );
}

#[tokio::test]
async fn last_factor_removal_follows_the_recovery_policy() {
    let s = spawn().await;
    let c = client();
    login(&s, &c).await;
    let secret = totp::generate_secret();
    s.store()
        .await
        .put_totp_secret(mw_mock_jmap::ACCOUNT_ID, &secret, true)
        .await
        .unwrap();
    let code = totp::totp_at(&secret, now_unix(), &TotpParams::default());
    step_up_password(&s, &c, Some(code)).await;

    s.policy("twofa.lastFactorRemoval", "block").await;
    let (status, body) = post(&c, s.url("/api/account/2fa/totp/disable"), json!({})).await;
    assert_eq!(status, 409, "{body}");
    assert_eq!(body["code"], "lastFactorBlocked");

    s.policy("twofa.lastFactorRemoval", "requireRecoveryCodes")
        .await;
    let (status, body) = post(&c, s.url("/api/account/2fa/totp/disable"), json!({})).await;
    assert_eq!(status, 409, "{body}");
    assert_eq!(body["code"], "lastFactorNeedsRecoveryCodes");
    let status_json = get(&c, s.url("/api/account/2fa")).await;
    assert_eq!(status_json["totp"], true, "nothing was removed");
    assert_eq!(
        status_json["passkeyPolicy"]["lastFactorRemoval"],
        "requireRecoveryCodes"
    );

    s.store()
        .await
        .add_recovery_codes(
            mw_mock_jmap::ACCOUNT_ID,
            &[mw_mfa::recovery::hash_code("abcd-efgh-ijkl")],
        )
        .await
        .unwrap();
    let (status, body) = post(&c, s.url("/api/account/2fa/totp/disable"), json!({})).await;
    assert_eq!(status, 200, "{body}");

    let reasons: Vec<(Value, Value)> = s
        .refusals()
        .await
        .iter()
        .map(|d| (d["flow"].clone(), d["reason"].clone()))
        .collect();
    assert_eq!(
        reasons,
        vec![
            (json!("last-factor-removal"), json!("lastFactorBlocked")),
            (
                json!("last-factor-removal"),
                json!("lastFactorNeedsRecoveryCodes")
            ),
        ]
    );
}

#[tokio::test]
async fn the_default_last_factor_rule_removes_as_before() {
    let s = spawn().await;
    let c = client();
    login(&s, &c).await;
    let secret = totp::generate_secret();
    s.store()
        .await
        .put_totp_secret(mw_mock_jmap::ACCOUNT_ID, &secret, true)
        .await
        .unwrap();
    let code = totp::totp_at(&secret, now_unix(), &TotpParams::default());
    step_up_password(&s, &c, Some(code)).await;
    let (status, body) = post(&c, s.url("/api/account/2fa/totp/disable"), json!({})).await;
    assert_eq!(status, 200, "{body}");
    assert!(s.refusals().await.is_empty());
}

/// A manual browser fixture: a HEAD-built server on `MW_T39_PASSKEY_PORT`
/// serving the built web app from `MW_T39_PASSKEY_WEB`, with the policy
/// "platform + user verification required", until it is killed. Driven by
/// `apps/web/e2e/passkeys/policy.spec.ts` (Chromium's virtual authenticator).
#[tokio::test]
#[ignore = "manual persistent browser fixture, requires MW_T39_PASSKEY_WEB"]
async fn serve_passkey_policy_browser_fixture() {
    let Some(web) = std::env::var_os("MW_T39_PASSKEY_WEB").filter(|v| !v.is_empty()) else {
        common::gate::skip("[t39 passkey fixture] MW_T39_PASSKEY_WEB unset - not started.");
        return;
    };
    let web = std::path::PathBuf::from(web);
    assert!(web.join("index.html").is_file());
    let port = std::env::var("MW_T39_PASSKEY_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(42239);
    let s = spawn_on(Some(port), Some(web)).await;
    s.policy("twofa.passkeyAttachment", "platform").await;
    s.policy("twofa.passkeyUserVerification", "required").await;
    eprintln!("[t39 passkey fixture] serving {}", s.base);
    std::future::pending::<()>().await;
}
