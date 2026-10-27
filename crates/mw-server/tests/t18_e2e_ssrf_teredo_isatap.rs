//! t18-e-e2e — SSRF Teredo / ISATAP embedded-IPv4 refusal (R4) LIVE through the real
//! image proxy.
//!
//! 26.17 (L3) decoded NAT64 (`64:ff9b::/96`) + 6to4 (`2002::/16`) embeddings. 26.18
//! (R4) extends the decode to two more transitional IPv6 forms that also carry a
//! routable IPv4 an attacker can point at an internal target:
//!   * **Teredo** `2001:0000::/32` — a plain *server* v4 (bits 32..64) AND an
//!     XOR-obfuscated mapped *client* v4 (bits 96..128); a private/metadata v4 in
//!     EITHER position is refused;
//!   * **ISATAP** `…:{0000,0200}:5efe:a.b.c.d` — v4 in the last 32 bits.
//!
//! `image_proxy.rs` unit-tests the decode (`blocks_teredo_and_isatap_embedded_private_ipv4`);
//! THIS leg drives the REAL `/api/image-proxy` route behind a real session and proves
//! the whole thing WIRED: a loopback/metadata/private v4 smuggled through Teredo or
//! ISATAP is refused live by the SSRF gate, while a Teredo/ISATAP address whose
//! embedded v4(s) are all public unicast is PERMITTED by it.
//!
//! # Why every assertion here reads the BODY (26.20, t22-e7 P3)
//! The route now has TWO gates that both answer `403` on the path this suite drives:
//! the egress/SSRF policy and the remote-image grant gate (`ungranted_response`).
//! A bare `assert_eq!(status, 403)` therefore does not distinguish them — every
//! smuggling leg below would stay green with the decode deleted, refused by the grant
//! gate instead of by the thing this file exists to test. And the grant gate runs
//! FIRST and now refuses without consulting the egress policy at all, so an
//! ungranted request never reaches the code under test. So the session here holds an
//! `all` grant on a seeded message ([`seed_grant`]) and the legs pass its id: a
//! smuggled address must then be refused with [`EGRESS_REFUSAL`], and a public one
//! must get neither refusal. One ungranted request checks, live, that the two bodies
//! differ, because a reword or a refactor routing both gates through one message
//! would silently degrade every assertion back to "this route returns 403".
//!
//! Run:
//!   cargo test -p mw-server --test t18_e2e_ssrf_teredo_isatap -- --nocapture --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::{Value, json};

use mw_server::{AppConfig, build_app};

mod common;
use common::test_db;

const KEY_HEX: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0";

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

/// Origins of the mock upstreams this test binary has started. Proxy mode refuses a
/// loopback JMAP upstream unless the operator allowlist names it (t24 B2), so the
/// server under test lists these in `SecurityConfig::jmap_upstreams` — the
/// production control an operator uses for an internal upstream, not a test switch.
/// A test must start its mock before its server.
static MOCK_UPSTREAMS: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

/// The mock upstreams started so far, as an allowlist.
fn mock_upstreams() -> Option<Vec<String>> {
    Some(MOCK_UPSTREAMS.lock().unwrap().clone())
}

// The mock this test started last. The server under test signs in to it as
// its configured upstream (`MW_UPSTREAM_DEFAULT`): a login can no longer name
// one (t32-h9). Per thread, so tests running in parallel keep their own.
thread_local! {
    static LAST_MOCK: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

fn last_mock() -> Option<String> {
    LAST_MOCK.with(|m| m.borrow().clone())
}

async fn spawn_mock() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    let origin = format!("http://{addr}");
    MOCK_UPSTREAMS.lock().unwrap().push(origin.clone());
    LAST_MOCK.with(|m| *m.borrow_mut() = Some(origin.clone()));
    origin
}

async fn spawn_server() -> (SocketAddr, String) {
    let base = test_db::unique_dir("mw-t18-teredo");
    let db = base.join("mw.db").to_string_lossy().into_owned();
    let web = base.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let config = AppConfig {
        db_path: db.clone(),
        // FIXED key so the grant fixture can open the same store (`seed_grant`).
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web as PathBuf),
        cookie_secure: false,
        mode: mw_server::ServerMode::Proxy,
        hardening: mw_server::HardeningConfig::default(),
        security: mw_server::SecurityConfig {
            jmap_upstreams: mock_upstreams(),
            upstream_default: last_mock(),
            ..mw_server::SecurityConfig::default()
        },
    };
    let app = build_app(config).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (addr, db)
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap()
}

async fn login(c: &reqwest::Client, base: &str, _mock: &str) -> String {
    let body: Value = c
        .post(format!("{base}/api/login"))
        .json(&json!({ "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(body["ok"], json!(true), "plain login (no 2FA): {body}");
    body["accountId"]
        .as_str()
        .expect("login names the session's accountId")
        .to_string()
}

/// A message on the session's account, covered by an `all` remote-image grant, so
/// requests carrying its id pass the grant gate and reach the EGRESS gate.
///
/// `ungranted_response` now refuses WITHOUT running the egress policy (no DNS
/// or network I/O for an ungranted request), so an ungranted request can no longer
/// tell us anything about the address. The legs below therefore present a grant: the
/// product order is grant first, then egress, and a granted request is exactly the
/// one whose target the egress gate has to judge.
///
/// The `accounts` row is raw-inserted with the SESSION's account id: `messages` has a
/// foreign key to `accounts(id)` and the public `create_account` mints a random id
/// (same precedent as `t22_egress_fetch_no_leak.rs`).
async fn seed_grant(db: &str, account: &str) -> String {
    let opts = sqlx::sqlite::SqliteConnectOptions::new().filename(db);
    let raw = sqlx::SqlitePool::connect_with(opts).await.unwrap();
    sqlx::query(
        "INSERT INTO accounts (id, kind, host, port, tls, username, sealed_creds, sync_policy_json)
         VALUES (?1, 'imap', 'mail.example', 993, 1, ?1, X'00', '{}')",
    )
    .bind(account)
    .execute(&raw)
    .await
    .expect("seed account row");
    raw.close().await;
    let store = mw_store::Store::open(db, mw_store::ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .expect("open store");
    let mailbox = store
        .upsert_mailbox(&mw_store::MailboxUpsert {
            account_id: account,
            name: "INBOX",
            role: Some("inbox"),
            uidvalidity: 1,
            uidnext: 2,
            highestmodseq: 1,
            total: 1,
            unread: 0,
            parent_id: None,
        })
        .await
        .expect("seed mailbox");
    let id = store
        .upsert_message(&mw_store::MessageUpsert {
            account_id: account,
            mailbox_id: &mailbox,
            uid: 1,
            uidvalidity: 1,
            message_id: Some("<ssrf-grant@test>"),
            thread_id: None,
            internaldate: Some("2026-08-16T12:00:00Z"),
            size: 42,
            flags_json: "[]",
            envelope: None,
            blob_ref: None,
        })
        .await
        .expect("seed message");
    store
        .grant_remote_image(account, "all", "")
        .await
        .expect("grant remote images");
    id
}

fn urlencoding(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// The refusal the EGRESS/SSRF policy returns for a blocked address
/// (`image_proxy.rs::refusal_response`, `Refusal::Blocked`). This is the one this
/// file is about.
const EGRESS_REFUSAL: &str = "target address is not permitted";
/// The refusal the 26.20 remote-image GRANT gate returns
/// (`image_proxy.rs::ungranted_response`). That path now refuses without
/// running the egress policy (no DNS or network I/O for an ungranted request), so this
/// body says nothing about the address: every leg that is about the address presents
/// a grant ([`seed_grant`]) and must NOT see this body.
const GRANT_REFUSAL: &str = "no remote-image grant covers this message";

/// Drive the real route and return both halves of the answer. The status alone is no
/// longer a sufficient instrument — see the module docs.
async fn proxy_get(
    c: &reqwest::Client,
    base: &str,
    target: &str,
    email_id: Option<&str>,
) -> (reqwest::StatusCode, String) {
    let mut url = format!("{base}/api/image-proxy?url={}", urlencoding(target));
    if let Some(id) = email_id {
        url.push_str(&format!("&emailId={}", urlencoding(id)));
    }
    let resp = c.get(url).send().await.unwrap();
    let status = resp.status();
    (status, resp.text().await.unwrap())
}

#[tokio::test]
async fn teredo_and_isatap_smuggled_private_targets_are_refused_live() {
    let mock = spawn_mock().await;
    let (addr, db) = spawn_server().await;
    let base = format!("http://{addr}");
    let c = client();
    let account = login(&c, &base, &mock).await;
    let granted = seed_grant(&db, &account).await;
    let granted = Some(granted.as_str());

    // Each of these embeds a forbidden IPv4 (127.0.0.1 / 169.254.169.254 / 10.0.0.1 /
    // 192.168.1.1) inside a Teredo (2001:0000::/32) or ISATAP (…:{0,200}:5efe:v4)
    // IPv6 address. All must be refused live (403) BY THE SSRF GATE — the R4 decode +
    // re-check closes the smuggling path in BOTH embedded positions Teredo carries.
    // One refusal body is kept for the instrument check below.
    let mut denied_body = String::new();
    for (target, what) in [
        (
            "http://[2001:0:4136:e378:8000:ffff:80ff:fffe]/x.png",
            "Teredo client v4 = 127.0.0.1 (obfuscated), public server",
        ),
        (
            "http://[2001:0:4136:e378:8000:ffff:5601:5601]/latest/meta-data/",
            "Teredo client v4 = 169.254.169.254 metadata",
        ),
        (
            "http://[2001:0:a00:1:8000:ffff:f7f7:f7f7]/x.png",
            "Teredo PRIVATE server v4 = 10.0.0.1 (public client)",
        ),
        (
            "http://[2001:470::5efe:7f00:1]/x.png",
            "ISATAP global-prefix IID wrapping 127.0.0.1",
        ),
        (
            "http://[2001:470:0:0:200:5efe:c0a8:101]/logo.png",
            "ISATAP (0x0200 flag) wrapping 192.168.1.1",
        ),
        (
            "http://[2001:470::5efe:a9fe:a9fe]/x",
            "ISATAP wrapping 169.254.169.254 metadata",
        ),
    ] {
        let (status, body) = proxy_get(&c, &base, target, granted).await;
        assert_eq!(
            status, 403,
            "SSRF R4: {what} ({target}) must be refused live (got {status})"
        );
        // The half that makes the leg non-vacuous: refused BY THE SSRF GATE. Without
        // this, deleting the Teredo/ISATAP decode leaves the address permitted by
        // egress and refused by the grant gate — same 403, test still green, control
        // gone.
        assert_eq!(
            body, EGRESS_REFUSAL,
            "SSRF R4: {what} ({target}) must be refused by the EGRESS gate, not by \
             another 403 (got {body:?}); a {GRANT_REFUSAL:?} here means the decode \
             PERMITTED the smuggled address and this leg is no longer testing it"
        );
        denied_body = body;
    }

    // A Teredo address whose BOTH embedded v4s are public (server 65.54.227.120 +
    // client 8.8.8.8), and a public-wrapping ISATAP (8.8.8.8), are PERMITTED by the
    // SSRF gate — the decode re-checks each v4, it does not blanket-refuse the
    // transitional prefix. With the grant presented the request passes both gates and
    // the server attempts the fetch, so whatever it answers, it must be neither
    // refusal: not the egress one (the address was permitted) and not the grant one.
    for (target, what) in [
        (
            "http://[2001:0:4136:e378:8000:ffff:f7f7:f7f7]/x.png",
            "Teredo, both embedded v4s public (server + 8.8.8.8)",
        ),
        (
            "http://[2001:470::5efe:808:808]/x.png",
            "ISATAP wrapping public 8.8.8.8",
        ),
    ] {
        let (status, body) = proxy_get(&c, &base, target, granted).await;
        assert_ne!(
            body, EGRESS_REFUSAL,
            "a public-wrapping {what} ({target}) must NOT be refused by the SSRF gate              (got {status} {body:?})"
        );
        assert_ne!(
            body, GRANT_REFUSAL,
            "the granted request for {what} must pass the grant gate (got {status}              {body:?}) — otherwise this leg never reached the egress gate"
        );
    }

    // THE INSTRUMENT ITSELF. Everything above distinguishes two refusals by their
    // body text; that is a valid instrument only while the two texts actually differ.
    // The same smuggled target WITHOUT the grant is refused by the grant gate, live, so
    // if a reword or a refactor ever routes both gates through one message, this fails
    // here rather than quietly turning every assertion above back into "the route
    // returns 403". It also shows the grant gate still runs first.
    let (ungranted_status, ungranted_body) =
        proxy_get(&c, &base, "http://[2001:470::5efe:7f00:1]/x.png", None).await;
    assert_eq!(ungranted_status, 403);
    assert_eq!(
        ungranted_body, GRANT_REFUSAL,
        "without a covering grant the request is refused by the grant gate"
    );
    assert_ne!(
        denied_body, ungranted_body,
        "the egress refusal and the grant refusal must be distinguishable, or every          assertion in this test degrades to a bare status check"
    );

    // Neither body is reachable without a session: `proxy_image` calls `crate::authed`
    // before both gates, so an unauthenticated caller is told nothing about the
    // address. (Between two AUTHENTICATED sessions the distinction IS visible — that
    // is precisely what the assertions above depend on.)
    let (anon_status, anon_body) = proxy_get(
        &client(),
        &base,
        "http://[2001:470::5efe:7f00:1]/x.png",
        granted,
    )
    .await;
    assert_eq!(
        anon_status, 401,
        "the image proxy requires a session (got {anon_status})"
    );
    assert_ne!(
        anon_body, EGRESS_REFUSAL,
        "no refusal detail without a session"
    );
    assert_ne!(
        anon_body, GRANT_REFUSAL,
        "no refusal detail without a session"
    );
}
