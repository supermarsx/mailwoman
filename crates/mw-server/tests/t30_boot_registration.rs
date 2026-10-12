//! t30-e3 — a request never observes an account that is registered but not yet
//! synced.
//!
//! After a start in engine mode the server connects every stored account in the
//! background (`engine_mode::register_stored_accounts`). Connecting an account
//! registers its backend with the engine and then runs the first sync, which is
//! what writes the account's mailboxes. A request that arrived between the two
//! found the account registered, skipped the wait and read an empty mailbox list
//! (CI: `t19_e2e.rs:578`, "no INBOX", in three jobs). And when that first sync
//! failed the account stayed registered: every later request was answered from a
//! cache nothing had filled, and nothing tried the mail server again.
//!
//! Both legs start a real engine-mode server over a stored account whose mail
//! server is a scripted POP3 listener, and issue `Mailbox/get` on `POST /jmap/api`
//! with the account's session cookie — the request the web client makes first.
//! The POP3 backend dials for every operation, so the listener sees the first
//! sync as its first connection and decides how it goes.
//!
//! Legs:
//!   * the first sync is held: a request made meanwhile is answered only after
//!     the sync finished, and lists the INBOX
//!   * the first sync fails: the next request connects the account itself and
//!     lists the INBOX
//!
//! Run:
//!   cargo test -p mw-server --test t30_boot_registration --locked -- --test-threads=1

use std::net::SocketAddr;
use std::time::Duration;

use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::watch;

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, build_app};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

mod common;
use common::test_db;

const KEY_HEX: &str = "30e3c1b2a3d4e5f60718293a4b5c6d7e30e3c1b2a3d4e5f60718293a4b5c6d7e";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW_T30</div>";
const USER: &str = "reader@example.org";

/// How long a request is watched while the first sync is held. A request that is
/// going to be answered from the unsynced account is answered well inside this.
const HOLD: Duration = Duration::from_secs(2);
/// The longest any single step of a leg may take.
const STEP: Duration = Duration::from_secs(60);

// ── the scripted mail server ─────────────────────────────────────────────────

/// What the scripted server does with its connections.
#[derive(Clone, Copy)]
enum Script {
    /// Accept every connection and say nothing on it until [`ScriptedPop3::release`].
    HoldUntilReleased,
    /// Close the first connection without a greeting; serve the later ones.
    DropTheFirst,
}

/// A POP3 server with an empty maildrop whose connections follow a [`Script`].
struct ScriptedPop3 {
    addr: SocketAddr,
    /// How many connections have been accepted.
    connections: watch::Receiver<usize>,
    release: watch::Sender<bool>,
}

impl ScriptedPop3 {
    async fn start(script: Script) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (count, connections) = watch::channel(0usize);
        let (release, released) = watch::channel(false);
        tokio::spawn(async move {
            let mut n = 0usize;
            loop {
                let Ok((sock, _)) = listener.accept().await else {
                    return;
                };
                n += 1;
                let _ = count.send(n);
                let mut released = released.clone();
                tokio::spawn(async move {
                    match script {
                        Script::HoldUntilReleased => {
                            if released.wait_for(|open| *open).await.is_err() {
                                return;
                            }
                        }
                        Script::DropTheFirst if n == 1 => return,
                        Script::DropTheFirst => {}
                    }
                    let _ = serve_empty_maildrop(sock).await;
                });
            }
        });
        ScriptedPop3 {
            addr,
            connections,
            release,
        }
    }

    fn connections(&self) -> usize {
        *self.connections.borrow()
    }

    /// Wait until at least `n` connections have been accepted.
    async fn wait_for_connections(&self, n: usize) {
        let mut seen = self.connections.clone();
        tokio::time::timeout(STEP, seen.wait_for(|c| *c >= n))
            .await
            .unwrap_or_else(|_| panic!("the mail server was not dialled {n} time(s)"))
            .unwrap();
    }

    /// Let every held connection, and every later one, proceed.
    fn release(&self) {
        self.release.send(true).unwrap();
    }
}

/// Enough of RFC 1939 for the engine's sync of an empty maildrop.
async fn serve_empty_maildrop(sock: tokio::net::TcpStream) -> std::io::Result<()> {
    let (read, mut write) = sock.into_split();
    let mut lines = BufReader::new(read).lines();
    write.write_all(b"+OK t30 scripted POP3 ready\r\n").await?;
    while let Some(line) = lines.next_line().await? {
        let verb = line
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_ascii_uppercase();
        let reply: &[u8] = match verb.as_str() {
            "CAPA" => b"+OK capability list follows\r\nUIDL\r\nTOP\r\nUSER\r\n.\r\n",
            "USER" | "PASS" | "NOOP" | "RSET" => b"+OK\r\n",
            "STAT" => b"+OK 0 0\r\n",
            "UIDL" | "LIST" => b"+OK 0 messages\r\n.\r\n",
            "QUIT" => b"+OK bye\r\n",
            _ => b"-ERR unsupported in the t30 mock\r\n",
        };
        write.write_all(reply).await?;
        if verb == "QUIT" {
            break;
        }
    }
    Ok(())
}

// ── the server fixture ───────────────────────────────────────────────────────

struct Fixture {
    base: String,
    account: String,
    cookie: String,
    store: Store,
}

/// Store one POP3 account and one session for it, as a previous run of the server
/// would have left them, then start an engine-mode server over that database. The
/// start is what begins the background registration.
async fn start_server_with_a_stored_account(pop3: SocketAddr) -> Fixture {
    let dir = test_db::unique_dir("mw-t30-e3-boot");
    let web = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let db = dir.join("mw.db").to_string_lossy().into_owned();

    let creds = Credentials {
        username: USER.into(),
        password: "pw".into(),
    };
    let store = Store::open(&db, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .expect("store opens");
    let account = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Pop3,
                host: &pop3.ip().to_string(),
                port: pop3.port(),
                tls: "plaintext",
                username: USER,
                sync_policy_json: "{}",
            },
            &creds,
        )
        .await
        .expect("account row");
    let cookie = store
        .create_session(
            &account,
            USER,
            "http://upstream.invalid",
            "http://upstream.invalid",
            &creds,
        )
        .await
        .expect("session row");

    let config = AppConfig {
        db_path: db,
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: ServerMode::Engine,
        hardening: HardeningConfig::default(),
        security: SecurityConfig::default(),
    };
    let app = build_app(config).await.expect("build_app");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    Fixture {
        base: format!("http://{addr}"),
        account,
        cookie,
        store,
    }
}

impl Fixture {
    /// `Mailbox/get` for the whole account on `POST /jmap/api`, with the session
    /// cookie. Returns the decoded response; any status but 200 is a failure.
    fn mailbox_get(&self) -> impl Future<Output = Value> + Send + 'static {
        let request = reqwest::Client::new()
            .post(format!("{}/jmap/api", self.base))
            .header(
                reqwest::header::COOKIE,
                format!("mw_session={}", self.cookie),
            )
            .json(&json!({
                "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
                "methodCalls": [["Mailbox/get",
                    { "accountId": self.account, "ids": Value::Null }, "m"]]
            }));
        async move {
            let resp = request.send().await.expect("jmap request");
            let status = resp.status();
            let body = resp.text().await.expect("jmap body");
            assert_eq!(status, 200, "POST /jmap/api: {body}");
            serde_json::from_str(&body)
                .unwrap_or_else(|e| panic!("jmap response is not json ({e}): {body:?}"))
        }
    }
}

/// Whether a `Mailbox/get` response lists a mailbox with the inbox role.
fn lists_inbox(response: &Value) -> bool {
    response["methodResponses"][0][1]["list"]
        .as_array()
        .is_some_and(|list| {
            list.iter().any(|m| {
                m["role"]
                    .as_str()
                    .is_some_and(|r| r.eq_ignore_ascii_case("inbox"))
            })
        })
}

// ── 1. a request made while the first sync is in flight ──────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_request_made_during_the_first_sync_is_answered_after_it() {
    let pop3 = ScriptedPop3::start(Script::HoldUntilReleased).await;
    let fx = start_server_with_a_stored_account(pop3.addr).await;

    // The start-up registration has dialled the mail server, which is holding the
    // connection: the account's backend is registered and its first sync has not
    // written anything yet.
    pop3.wait_for_connections(1).await;
    assert!(
        fx.store
            .list_mailboxes(&fx.account)
            .await
            .unwrap()
            .is_empty(),
        "the premise: no mailbox of the account is stored before the first sync"
    );

    let mut request = tokio::spawn(fx.mailbox_get());
    let while_held = tokio::time::timeout(HOLD, &mut request)
        .await
        .ok()
        .map(|joined| joined.expect("the request task"));
    let (when, response) = match while_held {
        Some(response) => ("while the first sync was held", response),
        None => {
            pop3.release();
            let response = tokio::time::timeout(STEP, request)
                .await
                .expect("the request is answered once the first sync can finish")
                .expect("the request task");
            ("after the first sync was released", response)
        }
    };
    assert!(
        lists_inbox(&response),
        "Mailbox/get, answered {when}, must list the INBOX the first sync creates: {response}"
    );
    assert_eq!(
        when, "after the first sync was released",
        "nothing can list the INBOX before the mail server has answered"
    );

    // The next request takes the ordinary path: the account is connected.
    assert!(lists_inbox(&fx.mailbox_get().await), "and it stays listed");
}

// ── 2. a first sync that fails ───────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_request_after_a_failed_first_sync_connects_the_account_again() {
    let pop3 = ScriptedPop3::start(Script::DropTheFirst).await;
    let fx = start_server_with_a_stored_account(pop3.addr).await;

    // The start-up registration dialled and was cut off before the greeting, so
    // its first sync failed. The mail server answers from now on.
    pop3.wait_for_connections(1).await;

    let response = tokio::time::timeout(STEP, fx.mailbox_get())
        .await
        .expect("the request is answered");
    assert!(
        lists_inbox(&response),
        "Mailbox/get after a failed first sync must list the INBOX, from a sync \
         the request made itself: {response}"
    );
    assert!(
        pop3.connections() >= 2,
        "the request dialled the mail server again ({} connection(s) in all)",
        pop3.connections()
    );
}
