//! Sign-in pacing and step-up proofs, for tests whose subject is something else.
//!
//! Two server rules sit in front of most authenticated flows, and a test that is
//! not about them has to get past them the way a real client does:
//!
//! * **Sign-in pacing** (`signin.retryCooldownSecs`, default 3 s per client IP,
//!   `signin_cooldown.rs`). Every in-process server listens on 127.0.0.1, so all of
//!   a test's sign-ins share one key. [`no_signin_cooldown`] stores the policy value
//!   `0`, which switches the pacing off. That is a setting an operator can choose, so
//!   the server under test is still a supported configuration. It holds whether the
//!   pacing counts every attempt or only failed ones. Tests whose subject is the
//!   pacing itself (`t32_signin_cooldown.rs`) must not call it.
//! * **Step-up** (`step_up.rs`). Minting an API key, and the admin writes marked
//!   sensitive, answer `403 step_up_required` until the session has confirmed
//!   its password in the last five minutes. [`user_step_up`] and [`admin_step_up`]
//!   send that proof through the real routes. Nothing is bypassed: the proof is
//!   checked, and it is bound to that session.

use mw_store::{ServerKey, Store, settings_policy::SettingPolicyRow};
use serde_json::{Value, json};

/// Store `signin.retryCooldownSecs = 0` (pacing off) in `store`'s policy table.
/// The server reads the policy on every sign-in, so this works before or after
/// the server under test has started.
pub async fn no_signin_cooldown(store: &Store) {
    store
        .put_setting_policy(&SettingPolicyRow {
            key: "signin.retryCooldownSecs".into(),
            global_value: Some("0".into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "test-fixture".into(),
            updated_at: 0,
        })
        .await
        .expect("store signin.retryCooldownSecs = 0");
}

/// [`no_signin_cooldown`] for the SQLite database at `db`, opened with the
/// server key `key_hex` the server under test uses.
pub async fn no_signin_cooldown_at(db: &str, key_hex: &str) {
    let store = Store::open(db, ServerKey::from_hex(key_hex).expect("server key"))
        .await
        .expect("open the server's store");
    no_signin_cooldown(&store).await;
}

/// The request header a personal-settings write must carry (0a11b581,
/// `prefs_routes/personal_storage.rs`): the `storageAuthority` the login (or
/// `GET /api/me`) returned. Without it a write answers `428 storageAuthorityRequired`;
/// with another session's value, `409 accountChanged`.
pub const STORAGE_AUTHORITY_HEADER: &str = "x-mailwoman-storage-authority";

/// The `storageAuthority` in a login or `/api/me` body. Panics if there is none.
pub fn storage_authority(body: &Value) -> String {
    body["storageAuthority"]
        .as_str()
        .unwrap_or_else(|| panic!("the body names no storageAuthority: {body}"))
        .to_string()
}

/// A session opened through the real `POST /api/login`.
pub struct SignedIn {
    /// The `mw_session` cookie's value (without the `mw_session=` prefix).
    pub session: String,
    /// The account the login names.
    pub account_id: String,
    /// The login's `storageAuthority`, for [`STORAGE_AUTHORITY_HEADER`].
    pub storage_authority: Option<String>,
}

impl SignedIn {
    /// The `Cookie` header value that carries this session.
    pub fn cookie(&self) -> String {
        format!("mw_session={}", self.session)
    }
}

/// Sign `username` in through `POST {base}/api/login` with a cookie-less client
/// and return the session cookie and account id. A test that used to write a
/// session row by hand uses this instead: since 7e9af0d0 every request
/// re-checks a session against the configured upstream and the stored account,
/// so a hand-made row naming no real upstream is refused (`401`).
pub async fn sign_in(base: &str, username: &str, password: &str) -> SignedIn {
    let response = reqwest::Client::builder()
        .no_proxy()
        .build()
        .unwrap()
        .post(format!("{base}/api/login"))
        .json(&json!({ "username": username, "password": password }))
        .send()
        .await
        .expect("POST /api/login");
    let status = response.status();
    let session = response
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .filter_map(|v| v.split(';').next())
        .find_map(|v| v.strip_prefix("mw_session="))
        .map(str::to_string);
    let body: Value = response.json().await.unwrap_or(Value::Null);
    assert!(status.is_success(), "login {username}: {status} {body}");
    SignedIn {
        session: session.unwrap_or_else(|| panic!("login set no mw_session cookie: {body}")),
        account_id: body["accountId"]
            .as_str()
            .unwrap_or_else(|| panic!("login names no account: {body}"))
            .to_string(),
        storage_authority: body["storageAuthority"].as_str().map(str::to_string),
    }
}

/// Confirm the signed-in user's password through `POST {base}/api/security/step-up`.
/// `client` must carry the session (a cookie store, or `extra` headers such as
/// `Authorization`). Panics with the server's answer unless the proof is granted.
pub async fn user_step_up(client: &reqwest::Client, base: &str, password: &str) {
    user_step_up_with(client, base, password, &[]).await;
}

/// [`user_step_up`] with extra request headers (a bearer token, a CSRF header).
pub async fn user_step_up_with(
    client: &reqwest::Client,
    base: &str,
    password: &str,
    extra: &[(&str, &str)],
) {
    let mut request = client
        .post(format!("{base}/api/security/step-up"))
        .json(&json!({ "password": password }));
    for (name, value) in extra {
        request = request.header(*name, *value);
    }
    granted("user step-up", request).await;
}

/// Confirm the signed-in admin's password through `POST {admin_base}/step-up`,
/// where `admin_base` is the admin prefix including the origin, e.g.
/// `http://127.0.0.1:1234/admin`. `extra` carries headers the admin API wants
/// (for instance the CSRF header).
pub async fn admin_step_up(
    client: &reqwest::Client,
    admin_base: &str,
    password: &str,
    extra: &[(&str, &str)],
) {
    let mut request = client
        .post(format!("{admin_base}/step-up"))
        .json(&json!({ "password": password }));
    for (name, value) in extra {
        request = request.header(*name, *value);
    }
    granted("admin step-up", request).await;
}

async fn granted(what: &str, request: reqwest::RequestBuilder) {
    let response = request.send().await.expect(what);
    let status = response.status();
    let body: Value = response.json().await.unwrap_or(Value::Null);
    assert!(
        status.is_success() && body["ok"] == json!(true),
        "{what} must be granted: {status} {body}"
    );
}
