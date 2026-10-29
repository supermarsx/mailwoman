//! t48: a linked mailbox keeps working after its private child session ends.
//!
//! Real router over HTTP, engine mode, real sign-ins, step-up and link. The
//! child mailbox sits behind a switchable TCP forwarder, so "the password was
//! changed upstream" is the same endpoint (same storage authority) answering
//! with a new password. Session expiry is simulated in the database the way
//! the clocks see it: an old `activity_at` (idle lapse) or the rows gone
//! (prune / absolute cap). A second app on the same database is a restart.
//!
//! Run: cargo test -p mw-server --test t48_linked_credentials -- --test-threads=1
//! Live Greenmail leg: MW_T48_GREENMAIL=127.0.0.1:<imap> (user/password
//! `t48child@greenmail.test` / `t48-pass`, Greenmail auth enabled).
mod common;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use common::mock_mail::MockPop3;
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config};
use mw_store::{DomainConfigRow, ServerKey, Store, settings_policy::SettingPolicyRow};
use serde_json::{Value, json};

const KEY: &str = "48a1b2c3d4e5f6071829304a5b6c7d8e48a1b2c3d4e5f6071829304a5b6c7d8e";
const PARENT_PASS: &str = "Parent-Passw0rd!";
const CHILD_PASS: &str = "Child-Secret-7f3a91";
const CHILD_PASS_NEW: &str = "Child-Secret-new-22c8";

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
            assert!(!body.contains(secret), "a response carried the secret: {body}");
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
    headers.insert(
        "x-mailwoman-storage-authority",
        authority.parse().unwrap(),
    );
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
    async fn renew(&self, seen: &Seen, id: &str, pass: &str, expected: u16) -> Value {
        seen.json(
            self.http
                .post(format!("{}/api/accounts/{id}/renew", self.base))
                .json(&json!({"password":pass}))
                .send()
                .await
                .unwrap(),
            expected,
        )
        .await
    }
}

/// Lapse every private child session's idle clock.
async fn idle_out(db: &str) {
    assert!(
        sql(
            db,
            "UPDATE sessions SET activity_at='2000-01-01T00:00:00+00:00',created_at='2000-01-01T00:00:00+00:00' WHERE linked_internal<>0"
        )
        .await
            > 0
    );
}
async fn audit_actions(db: &str) -> Vec<String> {
    sql_rows(db, "SELECT action FROM audit_log WHERE action LIKE 'linked-%' ORDER BY ts").await
}

#[tokio::test]
async fn linked_access_renews_from_the_sealed_credential_and_renews_inline_when_refused() {
    unsafe {
        std::env::set_var("MW_ENGINE_TLS", "plaintext");
    }
    let seen = Seen::default();
    let parents = MockPop3::start_accounts(&[
        ("parent@example.org", PARENT_PASS),
        ("stranger@example.org", PARENT_PASS),
    ])
    .await;
    let child_old = MockPop3::start_accounts(&[("child@child.test", CHILD_PASS)]).await;
    let child_new = MockPop3::start_accounts(&[("child@child.test", CHILD_PASS_NEW)]).await;
    let (child_addr, child_target) = forwarder(child_old.addr).await;
    let dir = common::test_db::unique_dir("mw-t48");
    let db = dir.join("mw.db").to_string_lossy().into_owned();
    policy(&db, "signin.retryCooldownSecs", "0").await;
    policy(&db, "accounts.linkingEnabled", "true").await;
    domain(&db, "example.org", "pop3", parents.addr).await;
    domain(&db, "child.test", "pop3", child_addr).await;
    let base = serve(&db).await;

    let owner = sign_in(&seen, &base, "parent@example.org", PARENT_PASS).await;
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
    assert_eq!(linked["account"]["renewal"], "automatic", "{linked}");
    let listing = owner.list(&seen).await;
    assert_eq!(listing["policy"]["storeCredentials"], true);
    assert_eq!(
        sql_rows(&db, "SELECT state FROM linked_credential").await,
        vec!["active".to_string()]
    );
    // The password is nowhere in the database file in clear.
    let bytes = std::fs::read(&db).unwrap();
    assert!(!bytes.windows(CHILD_PASS.len()).any(|w| w == CHILD_PASS.as_bytes()));

    // 1. Idle lapse: the next listing renews silently.
    idle_out(&db).await;
    assert_eq!(owner.entry(&seen, &id).await["status"], "ready");
    owner.switch(&seen, &id, 200).await;
    assert!(audit_actions(&db).await.contains(&"linked-credential-renewed".to_string()));

    // 2. The child session rows are gone (prune / absolute cap): opening the
    //    mailbox renews it, no listing needed first.
    assert!(sql(&db, "DELETE FROM sessions WHERE linked_internal<>0").await > 0);
    owner.switch(&seen, &id, 200).await;
    assert_eq!(owner.entry(&seen, &id).await["revision"], revision);

    // 3. Restart: a fresh process on the same database, a fresh parent sign-in.
    let base2 = serve(&db).await;
    let owner2 = sign_in(&seen, &base2, "parent@example.org", PARENT_PASS).await;
    idle_out(&db).await;
    assert_eq!(owner2.entry(&seen, &id).await["status"], "ready");
    owner2.switch(&seen, &id, 200).await;

    // 4. Upstream refuses the stored password (changed on the same server).
    *child_target.lock().unwrap() = child_new.addr;
    idle_out(&db).await;
    let refused = owner2.entry(&seen, &id).await;
    assert_eq!(refused["status"], "reauthRequired", "{refused}");
    assert_eq!(refused["renewal"], "signIn");
    assert_eq!(refused["renewalReason"], "credentialRejected");
    assert_eq!(refused["address"], "child@child.test");
    owner2.switch(&seen, &id, 409).await;
    assert_eq!(
        sql_rows(&db, "SELECT state FROM linked_credential").await,
        vec!["rejected".to_string()]
    );
    assert_eq!(
        sql(&db, "UPDATE linked_credential SET state=state WHERE sealed_secret IS NULL").await,
        1,
        "the refused secret is erased"
    );
    assert!(audit_actions(&db).await.contains(&"linked-credential-refresh-failed".to_string()));

    // 5. Inline renew: wrong password is refused, the new one restores the
    //    same link (id, revision) without unlinking.
    owner2.renew(&seen, &id, "wrong", 401).await;
    let renewed = owner2.renew(&seen, &id, CHILD_PASS_NEW, 200).await;
    assert_eq!(renewed["account"]["id"], id.as_str());
    assert_eq!(renewed["account"]["revision"], revision.as_str());
    assert_eq!(renewed["account"]["status"], "ready");
    assert_eq!(renewed["account"]["renewal"], "automatic");
    owner2.switch(&seen, &id, 200).await;
    idle_out(&db).await;
    assert_eq!(owner2.entry(&seen, &id).await["status"], "ready");

    // 5b. The linked mailbox's own owner signs out everywhere else: that ends
    //     the link's automatic renewal too (no silent sign-in afterwards).
    let child_user = sign_in(&seen, &base2, "child@child.test", CHILD_PASS_NEW).await;
    seen.json(
        child_user
            .http
            .post(format!("{base2}/api/account/sessions/revoke"))
            .json(&json!({"all":true}))
            .send()
            .await
            .unwrap(),
        200,
    )
    .await;
    assert!(
        sql_rows(&db, "SELECT link_id FROM linked_credential WHERE sealed_secret IS NOT NULL")
            .await
            .is_empty(),
        "sign-out-everywhere of the linked mailbox erases the stored credential"
    );
    let ended = owner2.entry(&seen, &id).await;
    assert_eq!(ended["status"], "reauthRequired", "{ended}");
    let renewed = owner2.renew(&seen, &id, CHILD_PASS_NEW, 200).await;
    assert_eq!(renewed["account"]["renewal"], "automatic");

    // 6. Another owner cannot renew, read or open this link.
    let stranger = sign_in(&seen, &base2, "stranger@example.org", PARENT_PASS).await;
    stranger.renew(&seen, &id, CHILD_PASS_NEW, 404).await;
    assert_eq!(stranger.list(&seen).await["accounts"], json!([]));
    assert_ne!(stranger.authority, owner2.authority);

    // 7. Policy off: the secret is erased at the next use, access then needs
    //    renewing, and renewing does not store it again.
    policy(&db, "accounts.storeLinkedCredentials", "false").await;
    idle_out(&db).await;
    let off = owner2.entry(&seen, &id).await;
    assert_eq!(off["status"], "reauthRequired");
    assert_eq!(off["renewalReason"], "policyOff");
    assert_eq!(owner2.list(&seen).await["policy"]["storeCredentials"], false);
    let renewed = owner2.renew(&seen, &id, CHILD_PASS_NEW, 200).await;
    assert_eq!(renewed["account"]["status"], "ready");
    assert_eq!(renewed["account"]["renewal"], "signIn");
    assert_eq!(renewed["account"]["renewalReason"], "policyOff");
    assert_eq!(
        sql(&db, "UPDATE linked_credential SET state=state WHERE sealed_secret IS NOT NULL").await,
        0
    );
    idle_out(&db).await;
    assert_eq!(owner2.entry(&seen, &id).await["status"], "reauthRequired");

    // 8. Zero-access on the child: nothing server-readable is stored.
    policy(&db, "accounts.storeLinkedCredentials", "true").await;
    let child_account = sql_rows(&db, "SELECT account_id FROM linked_account").await[0].clone();
    store(&db)
        .await
        .upsert_zeroaccess(&mw_store::ZeroAccessRow {
            account_id: child_account,
            enabled: true,
            wrapped_root_key: vec![1; 32],
            kdf_params_json: "{}".into(),
            recovery_wrapped: None,
            paired_devices_json: "[]".into(),
        })
        .await
        .unwrap();
    let renewed = owner2.renew(&seen, &id, CHILD_PASS_NEW, 200).await;
    assert_eq!(renewed["account"]["renewalReason"], "zeroAccess");
    assert_eq!(
        sql(&db, "UPDATE linked_credential SET state=state WHERE sealed_secret IS NOT NULL").await,
        0
    );

    // 9. Unlink removes the credential row.
    owner2.step_up(&seen, PARENT_PASS).await;
    seen.json(
        owner2
            .http
            .delete(format!("{base2}/api/accounts/{id}"))
            .header("if-match", format!("\"{revision}\""))
            .send()
            .await
            .unwrap(),
        204,
    )
    .await;
    assert!(sql_rows(&db, "SELECT link_id FROM linked_credential").await.is_empty());
    let actions = audit_actions(&db).await;
    assert!(actions.contains(&"linked-credential-stored".to_string()), "{actions:?}");
    assert!(actions.contains(&"linked-credential-deleted".to_string()), "{actions:?}");
    for detail in sql_rows(&db, "SELECT detail_json FROM audit_log").await {
        assert!(!detail.contains(CHILD_PASS) && !detail.contains(CHILD_PASS_NEW));
    }

    // No response ever carried either password.
    seen.assert_never(CHILD_PASS);
    seen.assert_never(CHILD_PASS_NEW);
}

/// Live: a Greenmail IMAP child stays linked across a server restart and an
/// expired child session, with no user action.
#[tokio::test]
async fn greenmail_imap_link_survives_restart_and_expiry() {
    let Ok(imap) = std::env::var("MW_T48_GREENMAIL") else {
        if std::env::var("MW_REQUIRE_LIVE").is_ok() {
            panic!("MW_T48_GREENMAIL is required");
        }
        return;
    };
    unsafe {
        std::env::set_var("MW_ENGINE_TLS", "plaintext");
    }
    let imap: SocketAddr = imap.parse().unwrap();
    let seen = Seen::default();
    let parents = MockPop3::start_accounts(&[("parent@example.org", PARENT_PASS)]).await;
    let dir = common::test_db::unique_dir("mw-t48-live");
    let db = dir.join("mw.db").to_string_lossy().into_owned();
    policy(&db, "signin.retryCooldownSecs", "0").await;
    policy(&db, "accounts.linkingEnabled", "true").await;
    domain(&db, "example.org", "pop3", parents.addr).await;
    domain(&db, "greenmail.test", "imap", imap).await;
    let base = serve(&db).await;
    let owner = sign_in(&seen, &base, "parent@example.org", PARENT_PASS).await;
    owner.step_up(&seen, PARENT_PASS).await;
    let linked = seen
        .json(
            owner
                .http
                .post(format!("{base}/api/accounts/link"))
                .json(&json!({"username":"t48child@greenmail.test","password":"t48-pass"}))
                .send()
                .await
                .unwrap(),
            201,
        )
        .await;
    let id = linked["account"]["id"].as_str().unwrap().to_string();
    assert_eq!(linked["account"]["renewal"], "automatic");
    let base2 = serve(&db).await;
    let owner2 = sign_in(&seen, &base2, "parent@example.org", PARENT_PASS).await;
    assert!(sql(&db, "DELETE FROM sessions WHERE linked_internal<>0").await > 0);
    let context = owner2.switch(&seen, &id, 200).await;
    assert!(context["context"]["storageAuthority"].as_str().is_some());
    assert_eq!(owner2.entry(&seen, &id).await["status"], "ready");
    seen.assert_never("t48-pass");
}
