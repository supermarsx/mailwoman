//! t32-h16 (QR1) — an account the admin disabled is refused under every spelling
//! of its address the mail server accepts, not only under the one the admin
//! typed.
//!
//! Before this, the flag was read under the typed name folded to lower case and
//! under the names the account was known by. In engine mode a mail server with a
//! default realm accepts the bare `alice` for `alice@example.com`, one that strips
//! tags accepts `alice+x@example.com`, and a domain has a Unicode and an ASCII
//! (`xn--`) form. Such a spelling had no account row yet, so it was gated under
//! itself alone, passed, and a new account was created for the same mailbox.
//!
//! The upstream is the scripted POP3 server the other engine-mode tests use. It
//! accepts every spelling listed for a person with the same password, as a mail
//! server that maps them to one mailbox does. Every refusal is followed, in the
//! same server and for the same credential, by the same login succeeding once
//! the flag is cleared: a refusal that would also happen with the flag clear
//! proves nothing.
//!
//! Run:
//!   cargo test -p mw-server --test t32_h16_disable_spelling --locked -- --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde_json::{Value, json};
use tokio::sync::{Mutex, MutexGuard};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config};
use mw_store::{ServerKey, Store};

mod common;
use common::mock_mail::MockPop3;
use common::test_db;

const KEY_HEX: &str = "3216a1b2c3d4e5f60718293a4b5c6d7e3216a1b2c3d4e5f60718293a4b5c6d7e";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW_T32H16</div>";
const PASS: &str = "Spelling-Passw0rd!";

/// Serialises the tests in this binary: the server reads `MW_ENGINE_TLS` per login.
async fn serial() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let guard = LOCK.get_or_init(Default::default).lock().await;
    // SAFETY: the guard serialises every mutation and every dependent read here.
    unsafe {
        std::env::set_var("MW_ENGINE_TLS", "plaintext");
    }
    guard
}

struct Server {
    base: String,
    db: String,
}

impl Server {
    async fn store(&self) -> Store {
        Store::open(&self.db, ServerKey::from_hex(KEY_HEX).unwrap())
            .await
            .expect("open the server's store")
    }

    /// The usernames of the engine's `accounts` rows.
    async fn accounts(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .store()
            .await
            .list_accounts()
            .await
            .unwrap()
            .into_iter()
            .map(|a| a.username)
            .collect();
        names.sort();
        names
    }
}

/// An engine-mode server whose configured upstream is a scripted POP3 server
/// that accepts each of `spellings` with [`PASS`].
async fn spawn_engine(spellings: &[&str]) -> (Server, MockPop3) {
    let accounts: Vec<(&str, &str)> = spellings.iter().map(|s| (*s, PASS)).collect();
    let pop = MockPop3::start_accounts(&accounts).await;
    let dir = test_db::unique_dir("mw-t32-h16");
    let web: PathBuf = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let db = dir.join("mw.db").to_string_lossy().into_owned();
    // Each refusal here is a failed sign-in, which keeps its pacing charge (t33
    // D1); these legs are not about pacing (`signin.retryCooldownSecs`,
    // d94472d7), and 0 switches it off.
    Store::open(&db, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap()
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: "signin.retryCooldownSecs".into(),
            global_value: Some("0".into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "t32-h16-fixture".into(),
            updated_at: 0,
        })
        .await
        .unwrap();
    let config = AppConfig {
        db_path: db.clone(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: ServerMode::Engine,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
            jmap_upstreams: Some(Vec::new()),
            upstream_default: Some(pop.url()),
            ..SecurityConfig::default()
        },
    };
    let v6 = V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("hunter2".into()),
        redis_url: None,
    };
    let app = mw_server::build_app_full(config, v6)
        .await
        .expect("build_app_full")
        .0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    (
        Server {
            base: format!("http://{addr}"),
            db,
        },
        pop,
    )
}

fn browser() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

/// Set the flags recorded under the address `id`, in the panel's own record
/// format (`account_gate` reads it under every spelling). Since 991223b9 the
/// panel's `PUT /admin/users/{id}/flags` addresses a known user identity, and an
/// address that has never signed in (or only under another spelling) is not one,
/// so the record is written directly.
async fn set_flags(srv: &Server, id: &str, disabled: bool, force: bool) {
    srv.store()
        .await
        .set_setting(
            &format!("v6:admin:flags:{id}"),
            &json!({
                "zero_access": false,
                "force_password_change": force,
                "remote_cache_wipe": false,
                "disabled": disabled,
            })
            .to_string(),
        )
        .await
        .unwrap();
}

/// Sign in as `user` on a fresh client; the client and the response.
async fn login(base: &str, user: &str) -> (reqwest::Client, reqwest::StatusCode, Value) {
    let c = browser();
    let r = c
        .post(format!("{base}/api/login"))
        .json(&json!({ "username": user, "password": PASS }))
        .send()
        .await
        .unwrap();
    let status = r.status();
    let body = r.json::<Value>().await.unwrap_or(Value::Null);
    (c, status, body)
}

async fn status_of(base: &str, user: &str) -> u16 {
    login(base, user).await.1.as_u16()
}

async fn me(c: &reqwest::Client, base: &str) -> u16 {
    c.get(format!("{base}/api/me"))
        .send()
        .await
        .unwrap()
        .status()
        .as_u16()
}

/// The bare local part and a `+tag` of a disabled address are refused before
/// any account row is written; the case variants still are; another person is
/// not; clearing the flag lets every spelling in.
#[tokio::test]
async fn a_disabled_address_is_refused_under_its_bare_and_tagged_spellings() {
    let _g = serial().await;
    let (srv, _pop) = spawn_engine(&[
        "alice@example.com",
        "alice",
        "alice+x@example.com",
        "Alice@Example.COM",
        "bob@example.com",
        "bob",
    ])
    .await;
    let base = &srv.base;

    let (_, status, _) = login(base, "alice@example.com").await;
    assert_eq!(status, 200, "control: the address itself signs in");
    assert_eq!(srv.accounts().await, vec!["alice@example.com".to_string()]);

    set_flags(&srv, "alice@example.com", true, false).await;

    for spelling in [
        "alice",
        "alice+x@example.com",
        "Alice@Example.COM",
        "alice@example.com",
    ] {
        assert_eq!(
            status_of(base, spelling).await,
            401,
            "{spelling} is refused while alice@example.com is disabled"
        );
    }
    assert_eq!(
        srv.accounts().await,
        vec!["alice@example.com".to_string()],
        "a refused spelling writes no accounts row"
    );

    // Another person, under the full address and the bare name.
    assert_eq!(status_of(base, "bob@example.com").await, 200);
    assert_eq!(status_of(base, "bob").await, 200);

    // Clearing the flag lets each refused spelling in: the refusals above were
    // the flag's.
    set_flags(&srv, "alice@example.com", false, false).await;
    for spelling in ["alice", "alice+x@example.com", "Alice@Example.COM"] {
        assert_eq!(
            status_of(base, spelling).await,
            200,
            "{spelling} signs in once alice@example.com is enabled again"
        );
    }
}

/// A session opened under the bare name before the address was disabled is
/// refused on its next request, and a hold set under the address applies to a
/// bare-name sign-in.
#[tokio::test]
async fn a_bare_name_session_and_a_hold_follow_the_address() {
    let _g = serial().await;
    let (srv, _pop) = spawn_engine(&["alice@example.com", "alice"]).await;
    let base = &srv.base;

    let (session, status, _) = login(base, "alice").await;
    assert_eq!(status, 200, "control: the bare name signs in");
    assert_eq!(me(&session, base).await, 200, "control: its session works");

    set_flags(&srv, "alice@example.com", true, false).await;
    assert_eq!(
        me(&session, base).await,
        401,
        "the bare-name session is refused once the address is disabled"
    );

    set_flags(&srv, "alice@example.com", false, true).await;
    let (_, status, body) = login(base, "alice").await;
    assert_eq!(status, 200, "a held account still signs in");
    assert_eq!(
        body["passwordChangeRequired"],
        json!(true),
        "the hold under the address applies to the bare name: {body}"
    );

    set_flags(&srv, "alice@example.com", false, false).await;
    let (_, status, body) = login(base, "alice").await;
    assert_eq!(status, 200);
    assert!(body.get("passwordChangeRequired").is_none(), "{body}");
}

/// The Unicode and the ASCII form of a domain are one address, whichever of the
/// two the admin flagged.
#[tokio::test]
async fn a_disabled_address_is_refused_under_the_other_form_of_its_domain() {
    let _g = serial().await;
    let (srv, _pop) = spawn_engine(&["carol@bücher.example", "carol@xn--bcher-kva.example"]).await;
    let base = &srv.base;

    set_flags(&srv, "carol@bücher.example", true, false).await;
    assert_eq!(status_of(base, "carol@xn--bcher-kva.example").await, 401);
    assert!(srv.accounts().await.is_empty(), "no accounts row");
    set_flags(&srv, "carol@bücher.example", false, false).await;
    assert_eq!(status_of(base, "carol@xn--bcher-kva.example").await, 200);

    set_flags(&srv, "carol@xn--bcher-kva.example", true, false).await;
    assert_eq!(status_of(base, "carol@bücher.example").await, 401);
    set_flags(&srv, "carol@xn--bcher-kva.example", false, false).await;
    assert_eq!(status_of(base, "carol@bücher.example").await, 200);
}

/// The bare-name rule fails closed (module docs of `account_gate`): with two
/// domains, a bare `dave` is refused while either `dave@<domain>` is disabled,
/// and the full address of the other one still signs in.
#[tokio::test]
async fn a_bare_name_is_refused_while_any_address_with_its_local_part_is_disabled() {
    let _g = serial().await;
    let (srv, _pop) = spawn_engine(&["dave", "dave@one.example", "dave@two.example"]).await;
    let base = &srv.base;

    assert_eq!(status_of(base, "dave").await, 200, "control");
    set_flags(&srv, "dave@two.example", true, false).await;
    assert_eq!(status_of(base, "dave").await, 401);
    assert_eq!(status_of(base, "dave@one.example").await, 200);
    assert_eq!(status_of(base, "dave@two.example").await, 401);
    set_flags(&srv, "dave@two.example", false, false).await;
    assert_eq!(status_of(base, "dave").await, 200);
}
