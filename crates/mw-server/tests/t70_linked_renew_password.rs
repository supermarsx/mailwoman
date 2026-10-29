//! t70: "Renew access" with only the mailbox password.
//!
//! Real router over HTTP, engine mode, real sign-ins, step-up and link (the
//! t48 fixture). The link is then made to look like one from before t48: its
//! credential row and its private child session are gone, so the server has
//! no saved sign-in name. Renewing still needs only the password: the server
//! takes the name from the linked account. Covered: a wrong password (401
//! `linkedPasswordRejected`), an unreachable mailbox server (502
//! `linkedServerUnreachable`), the right password (same link, ready, the
//! password sealed again), the per-link attempt limit (429
//! `linkedRenewLimit`), audit rows without the secret, and no response or
//! database row carrying it in clear.
//!
//! Run: cargo test -p mw-server --test t70_linked_renew_password -- --test-threads=1
mod common;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use common::mock_mail::MockPop3;
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config};
use mw_store::{DomainConfigRow, ServerKey, Store, settings_policy::SettingPolicyRow};
use serde_json::{Value, json};

const KEY: &str = "70a1b2c3d4e5f6071829304a5b6c7d8e70a1b2c3d4e5f6071829304a5b6c7d8e";
const PARENT_PASS: &str = "Parent-Passw0rd!";
const CHILD_PASS: &str = "Child-Secret-t70-5d1e";
const WRONG_PASS: &str = "Wrong-Secret-t70-93ab";

async fn store(db: &str) -> Store {
    Store::open(db, ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap()
}
async fn policy(db: &str, key: &str, value: &str) {
    store(db)
        .await
        .put_setting_policy(&SettingPolicyRow {
            key: key.into(),
            global_value: Some(value.into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "t48-fixture".into(),
            updated_at: 0,
        })
        .await
        .unwrap();
}
async fn domain(db: &str, name: &str, proto: &str, addr: SocketAddr) {
    store(db).await.put_domain_config(&DomainConfigRow{name:name.into(),upstream_json:json!({proto:{"host":"127.0.0.1","port":addr.port(),"tls":"none","allowPrivate":true}}).to_string(),allowlist_json:"[]".into(),blocklist_json:"[]".into(),enabled:true,preferred:false,settings_json:"{}".into(),updated_at:1}).await.unwrap();
}
async fn sql(db: &str, statement: &str) -> u64 {
    use sqlx::Connection;
    let mut conn = sqlx::SqliteConnection::connect(&format!("sqlite://{db}"))
        .await
        .unwrap();
    sqlx::query(statement)
        .execute(&mut conn)
        .await
        .unwrap()
        .rows_affected()
}
async fn sql_rows(db: &str, statement: &str) -> Vec<String> {
    use sqlx::{Connection, Row};
    let mut conn = sqlx::SqliteConnection::connect(&format!("sqlite://{db}"))
        .await
        .unwrap();
    sqlx::query(statement)
        .fetch_all(&mut conn)
        .await
        .unwrap()
        .iter()
        .map(|r| r.try_get::<String, _>(0).unwrap_or_default())
        .collect()
}

/// A loopback forwarder whose target can be switched (the "same server" with a
/// changed password).
async fn forwarder(target: SocketAddr) -> (SocketAddr, Arc<Mutex<SocketAddr>>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let current = Arc::new(Mutex::new(target));
    let shared = current.clone();
    tokio::spawn(async move {
        while let Ok((mut inbound, _)) = listener.accept().await {
            let to = *shared.lock().unwrap();
            tokio::spawn(async move {
                if let Ok(mut outbound) = tokio::net::TcpStream::connect(to).await {
                    let _ = tokio::io::copy_bidirectional(&mut inbound, &mut outbound).await;
                }
            });
        }
    });
    (addr, current)
}

async fn serve(db: &str) -> String {
    let config = AppConfig {
        db_path: db.into(),
        server_key_hex: Some(KEY.into()),
        web_dir: None,
        cookie_secure: false,
        mode: ServerMode::Engine,
        hardening: HardeningConfig {
            csrf_strict: true,
            ..Default::default()
        },
        security: SecurityConfig {
            jmap_upstreams: Some(Vec::new()),
            ..Default::default()
        },
    };
    let v6 = V6Config {
        admin_enabled: false,
        admin_username: None,
        admin_password: None,
        redis_url: None,
    };
    let (app, _push) = mw_server::build_app_full(config, v6).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap()
    });
    base
}

/// Every response body the test saw, to prove the secret is never in one.
#[derive(Default, Clone)]
struct Seen(Arc<Mutex<Vec<String>>>);
impl Seen {
    async fn json(&self, response: reqwest::Response, expected: u16) -> Value {
        let status = response.status();
        let headers = format!("{:?}", response.headers());
        let text = response.text().await.unwrap();
        self.0.lock().unwrap().push(format!("{headers}\n{text}"));
        assert_eq!(status, expected, "{text}");
        if text.is_empty() {
            Value::Null
        } else {
            serde_json::from_str(&text).unwrap()
        }
    }
    fn assert_never(&self, secret: &str) {
        for body in self.0.lock().unwrap().iter() {
            assert!(
                !body.contains(secret),
                "a response carried the secret: {body}"
            );
        }
    }
}

struct Owner {
    http: reqwest::Client,
    authority: String,
    base: String,
}
async fn sign_in(seen: &Seen, base: &str, user: &str, pass: &str) -> Owner {
    let jar = Arc::new(reqwest::cookie::Jar::default());
    let bare = reqwest::Client::builder()
        .no_proxy()
        .cookie_provider(jar.clone())
        .build()
        .unwrap();
    let login = seen
        .json(
            bare.post(format!("{base}/api/login"))
                .json(&json!({"username":user,"password":pass}))
                .send()
                .await
                .unwrap(),
            200,
        )
        .await;
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "x-csrf-token",
        login["csrfToken"].as_str().unwrap().parse().unwrap(),
    );
    let authority = login["storageAuthority"].as_str().unwrap().to_string();
    headers.insert("x-mailwoman-storage-authority", authority.parse().unwrap());
    let http = reqwest::Client::builder()
        .no_proxy()
        .cookie_provider(jar)
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();
    Owner {
        http,
        authority,
        base: base.into(),
    }
}
impl Owner {
    async fn list(&self, seen: &Seen) -> Value {
        seen.json(
            self.http
                .get(format!("{}/api/accounts", self.base))
                .send()
                .await
                .unwrap(),
            200,
        )
        .await
    }
    async fn entry(&self, seen: &Seen, id: &str) -> Value {
        self.list(seen).await["accounts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["id"] == id)
            .cloned()
            .unwrap()
    }
    async fn switch(&self, seen: &Seen, id: &str, expected: u16) -> Value {
        seen.json(
            self.http
                .post(format!("{}/api/accounts/{id}/switch", self.base))
                .send()
                .await
                .unwrap(),
            expected,
        )
        .await
    }
    async fn step_up(&self, seen: &Seen, pass: &str) {
        seen.json(
            self.http
                .post(format!("{}/api/security/step-up", self.base))
                .json(&json!({"password":pass}))
                .send()
                .await
                .unwrap(),
            200,
        )
        .await;
    }
}

impl Owner {
    async fn renew_with(&self, seen: &Seen, id: &str, body: Value, expected: u16) -> Value {
        seen.json(
            self.http
                .post(format!("{}/api/accounts/{id}/renew", self.base))
                .json(&body)
                .send()
                .await
                .unwrap(),
            expected,
        )
        .await
    }
}

async fn audit_rows(db: &str, action: &str) -> Vec<String> {
    sql_rows(
        db,
        &format!("SELECT detail_json FROM audit_log WHERE action='{action}' ORDER BY ts"),
    )
    .await
}

/// A loopback port nothing listens on.
async fn closed_port() -> SocketAddr {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    addr
}

#[tokio::test]
async fn renew_access_needs_only_the_password() {
    unsafe {
        std::env::set_var("MW_ENGINE_TLS", "plaintext");
    }
    let seen = Seen::default();
    let parents = MockPop3::start_accounts(&[("parent@example.org", PARENT_PASS)]).await;
    let child = MockPop3::start_accounts(&[("child@child.test", CHILD_PASS)]).await;
    let (child_addr, child_target) = forwarder(child.addr).await;
    let dir = common::test_db::unique_dir("mw-t70");
    let db = dir.join("mw.db").to_string_lossy().into_owned();
    policy(&db, "signin.retryCooldownSecs", "0").await;
    policy(&db, "accounts.linkingEnabled", "true").await;
    domain(&db, "example.org", "pop3", parents.addr).await;
    domain(&db, "child.test", "pop3", child_addr).await;
    let base = serve(&db).await;

    let owner = sign_in(&seen, &base, "parent@example.org", PARENT_PASS).await;
    assert!(!owner.authority.is_empty());
    owner.step_up(&seen, PARENT_PASS).await;
    let linked = seen
        .json(
            owner
                .http
                .post(format!("{base}/api/accounts/link"))
                .json(&json!({"username":"child@child.test","password":CHILD_PASS,"displayName":"Child"}))
                .send()
                .await
                .unwrap(),
            201,
        )
        .await;
    let id = linked["account"]["id"].as_str().unwrap().to_string();
    let revision = linked["account"]["revision"].as_str().unwrap().to_string();

    // A link from before t48 whose access ended: no credential row, no child
    // session. The listing still names the mailbox, so the client can ask for
    // the password alone.
    assert_eq!(sql(&db, "DELETE FROM linked_credential").await, 1);
    assert!(sql(&db, "DELETE FROM sessions WHERE linked_internal<>0").await > 0);
    let expired = owner.entry(&seen, &id).await;
    assert_eq!(expired["status"], "reauthRequired", "{expired}");
    assert_eq!(expired["address"], "child@child.test", "{expired}");
    assert_eq!(expired["renewal"], "signIn", "{expired}");

    // 1. Wrong password: refused with its own code, nothing changes.
    let refused = owner
        .renew_with(&seen, &id, json!({"password":WRONG_PASS}), 401)
        .await;
    assert_eq!(refused["code"], "linkedPasswordRejected", "{refused}");
    assert_eq!(owner.entry(&seen, &id).await["status"], "reauthRequired");
    assert!(
        sql_rows(&db, "SELECT state FROM linked_credential")
            .await
            .is_empty()
    );

    // 2. The mailbox's server cannot be reached: a different answer, so the
    //    person is not told their password is wrong.
    *child_target.lock().unwrap() = closed_port().await;
    let down = owner
        .renew_with(&seen, &id, json!({"password":CHILD_PASS}), 502)
        .await;
    assert_eq!(down["code"], "linkedServerUnreachable", "{down}");
    *child_target.lock().unwrap() = child.addr;

    // 3. The right password alone renews the same link and seals it again.
    let renewed = owner
        .renew_with(&seen, &id, json!({"password":CHILD_PASS}), 200)
        .await;
    assert_eq!(renewed["account"]["id"], id.as_str());
    assert_eq!(renewed["account"]["revision"], revision.as_str());
    assert_eq!(renewed["account"]["status"], "ready", "{renewed}");
    assert_eq!(renewed["account"]["renewal"], "automatic", "{renewed}");
    assert_eq!(renewed["account"]["address"], "child@child.test");
    assert_eq!(
        sql_rows(&db, "SELECT state FROM linked_credential").await,
        vec!["active".to_string()]
    );
    owner.switch(&seen, &id, 200).await;

    // 4. The existing flow still works: a typed sign-in name is used, and it
    //    must be the mailbox the link was made for.
    assert!(sql(&db, "DELETE FROM sessions WHERE linked_internal<>0").await > 0);
    let typed = owner
        .renew_with(
            &seen,
            &id,
            json!({"username":"child@child.test","password":CHILD_PASS}),
            200,
        )
        .await;
    assert_eq!(typed["account"]["status"], "ready", "{typed}");

    // 5. Attempts per owner and link are limited (10 per window), whatever
    //    the outcome; the refusal is audited too.
    let mut limited = None;
    for _ in 0..11 {
        let response = owner
            .http
            .post(format!("{base}/api/accounts/{id}/renew"))
            .json(&json!({"password":WRONG_PASS}))
            .send()
            .await
            .unwrap();
        if response.status() == 429 {
            assert!(response.headers().get("retry-after").is_some());
            limited = Some(seen.json(response, 429).await);
            break;
        }
        seen.json(response, 401).await;
    }
    let limited = limited.expect("the eleventh attempt is refused");
    assert_eq!(limited["code"], "linkedRenewLimit", "{limited}");

    // Audit: successes and failures with reason codes, never the secret.
    let failed = audit_rows(&db, "linked-access-renew-failed").await;
    assert!(
        failed.iter().any(|d| d.contains("linkedPasswordRejected")),
        "{failed:?}"
    );
    assert!(
        failed.iter().any(|d| d.contains("linkedServerUnreachable")),
        "{failed:?}"
    );
    assert!(
        failed.iter().any(|d| d.contains("rateLimited")),
        "{failed:?}"
    );
    assert!(!audit_rows(&db, "linked-access-renewed").await.is_empty());
    for detail in sql_rows(&db, "SELECT detail_json FROM audit_log").await {
        assert!(
            !detail.contains(CHILD_PASS) && !detail.contains(WRONG_PASS),
            "an audit row carried a password: {detail}"
        );
    }
    let bytes = std::fs::read(&db).unwrap();
    for secret in [CHILD_PASS, WRONG_PASS] {
        assert!(
            !bytes.windows(secret.len()).any(|w| w == secret.as_bytes()),
            "the database file holds {secret} in clear"
        );
    }
    seen.assert_never(CHILD_PASS);
    seen.assert_never(WRONG_PASS);
}
