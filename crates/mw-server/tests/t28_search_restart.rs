//! t28-e13: what an engine-mode server does across a restart.
//!
//! Each test runs the real `mailwoman serve` binary as a child process against a
//! scripted POP3 maildrop (and, for sending, a scripted SMTP server), stops it by
//! killing the process, and starts it again on the same database. A child process
//! is what makes the restart real: an in-process `build_app_full` cannot be torn
//! down, because its dispatcher and watch tasks keep the engine alive, and the
//! search index allows one writer per directory.
//!
//! * A search hit found before the restart is found after it, with no message
//!   fetched again and no rebuild run.
//! * With the index directory deleted, the server rebuilds the index from the
//!   store at start, again without fetching a message.
//! * A zero-access account has no text in the index directory: not for mail that
//!   arrives after the switch, and not for mail indexed before it.
//! * A submission due after the restart is sent with no HTTP request made.
//! * An account the admin disabled is not connected at start.
//!
//! No external service is needed; every leg runs in the default gate.

use std::io::Write as _;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

mod common;
use common::test_db;

const KEY_HEX: &str = "28e13a1b2c3d4e5f60718293a4b5c6d7e28e13a1b2c3d4e5f60718293a4b5c6d";
const USER: &str = "owner@example.org";
const PASS: &str = "Maildrop-Passw0rd!";
const ADMIN_USER: &str = "root";
const ADMIN_PASS: &str = "hunter2";

// ── scripted POP3 maildrop ───────────────────────────────────────────────────

/// A POP3 server over a maildrop the test can add to. It counts accepted logins
/// and `RETR` commands, which is how a test shows that a message was, or was not,
/// fetched again.
#[derive(Clone)]
struct Maildrop {
    addr: SocketAddr,
    messages: Arc<Mutex<Vec<(String, Vec<u8>)>>>,
    logins: Arc<AtomicUsize>,
    retrs: Arc<AtomicUsize>,
}

impl Maildrop {
    async fn start() -> Maildrop {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let drop = Maildrop {
            addr: listener.local_addr().unwrap(),
            messages: Arc::default(),
            logins: Arc::default(),
            retrs: Arc::default(),
        };
        let served = drop.clone();
        tokio::spawn(async move {
            loop {
                let Ok((sock, _)) = listener.accept().await else {
                    return;
                };
                let served = served.clone();
                tokio::spawn(async move {
                    let _ = served.serve(sock).await;
                });
            }
        });
        drop
    }

    fn url(&self) -> String {
        format!("pop3://{}", self.addr)
    }

    /// Deliver one message whose subject and body carry `token`.
    fn deliver(&self, uidl: &str, token: &str) {
        let raw = format!(
            "Message-ID: <{uidl}@example.org>\r\n\
             From: sender@example.org\r\n\
             To: {USER}\r\n\
             Subject: Notes about {token}\r\n\
             Date: Wed, 01 Jul 2026 09:00:00 +0000\r\n\
             MIME-Version: 1.0\r\n\
             Content-Type: text/plain; charset=utf-8\r\n\
             \r\n\
             The word is {token}.\r\n"
        );
        self.messages
            .lock()
            .unwrap()
            .push((uidl.to_string(), raw.into_bytes()));
    }

    fn logins(&self) -> usize {
        self.logins.load(Ordering::SeqCst)
    }

    fn retrs(&self) -> usize {
        self.retrs.load(Ordering::SeqCst)
    }

    async fn serve(&self, sock: tokio::net::TcpStream) -> std::io::Result<()> {
        let (read, mut write) = sock.into_split();
        let mut lines = BufReader::new(read).lines();
        write.write_all(b"+OK maildrop ready\r\n").await?;
        let mut user = String::new();
        let mut authed = false;
        while let Some(line) = lines.next_line().await? {
            let mut parts = line.split_whitespace();
            let cmd = parts.next().unwrap_or("").to_ascii_uppercase();
            let arg = parts.next().unwrap_or("").to_string();
            // The maildrop as this command sees it.
            let snapshot = self.messages.lock().unwrap().clone();
            let message = |n: &str| {
                n.parse::<usize>()
                    .ok()
                    .and_then(|n| n.checked_sub(1))
                    .and_then(|i| snapshot.get(i).cloned())
            };
            let mut reply: Vec<u8> = Vec::new();
            match cmd.as_str() {
                "CAPA" => reply.extend(b"+OK\r\nUSER\r\nUIDL\r\nTOP\r\n.\r\n"),
                "USER" => {
                    user = arg;
                    reply.extend(b"+OK\r\n");
                }
                "PASS" => {
                    // The password may contain no spaces here, so `arg` is all of it.
                    if user == USER && arg == PASS {
                        authed = true;
                        self.logins.fetch_add(1, Ordering::SeqCst);
                        reply.extend(b"+OK logged in\r\n");
                    } else {
                        reply.extend(b"-ERR [AUTH] invalid credentials\r\n");
                    }
                }
                "STAT" if authed => {
                    let octets: usize = snapshot.iter().map(|(_, m)| m.len()).sum();
                    reply.extend(format!("+OK {} {octets}\r\n", snapshot.len()).bytes());
                }
                "UIDL" if authed => {
                    reply.extend(b"+OK\r\n");
                    for (i, (uidl, _)) in snapshot.iter().enumerate() {
                        reply.extend(format!("{} {uidl}\r\n", i + 1).bytes());
                    }
                    reply.extend(b".\r\n");
                }
                "LIST" if authed => {
                    reply.extend(b"+OK\r\n");
                    for (i, (_, m)) in snapshot.iter().enumerate() {
                        reply.extend(format!("{} {}\r\n", i + 1, m.len()).bytes());
                    }
                    reply.extend(b".\r\n");
                }
                "RETR" | "TOP" if authed => match message(&arg) {
                    Some((_, raw)) => {
                        if cmd == "RETR" {
                            self.retrs.fetch_add(1, Ordering::SeqCst);
                        }
                        // The test messages have no line starting with a dot, so
                        // there is nothing to dot-stuff.
                        reply.extend(b"+OK\r\n");
                        reply.extend(&raw);
                        reply.extend(b".\r\n");
                    }
                    None => reply.extend(b"-ERR no such message\r\n"),
                },
                "NOOP" if authed => reply.extend(b"+OK\r\n"),
                "QUIT" => {
                    write.write_all(b"+OK bye\r\n").await?;
                    return Ok(());
                }
                _ => reply.extend(b"-ERR unsupported\r\n"),
            }
            write.write_all(&reply).await?;
        }
        Ok(())
    }
}

// ── scripted SMTP server ─────────────────────────────────────────────────────

/// An SMTP server that accepts everything and counts the messages it was handed.
#[derive(Clone)]
struct Smtp {
    addr: SocketAddr,
    delivered: Arc<AtomicUsize>,
}

impl Smtp {
    async fn start() -> Smtp {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let smtp = Smtp {
            addr: listener.local_addr().unwrap(),
            delivered: Arc::default(),
        };
        let delivered = smtp.delivered.clone();
        tokio::spawn(async move {
            loop {
                let Ok((sock, _)) = listener.accept().await else {
                    return;
                };
                let delivered = delivered.clone();
                tokio::spawn(async move {
                    let _ = Smtp::serve(sock, &delivered).await;
                });
            }
        });
        smtp
    }

    fn delivered(&self) -> usize {
        self.delivered.load(Ordering::SeqCst)
    }

    async fn serve(sock: tokio::net::TcpStream, delivered: &AtomicUsize) -> std::io::Result<()> {
        let (read, mut write) = sock.into_split();
        let mut lines = BufReader::new(read).lines();
        write.write_all(b"220 mock ESMTP\r\n").await?;
        // Lines of a multi-step AUTH exchange still expected from the client.
        let mut auth_lines = 0usize;
        let mut in_data = false;
        while let Some(line) = lines.next_line().await? {
            if in_data {
                if line == "." {
                    in_data = false;
                    delivered.fetch_add(1, Ordering::SeqCst);
                    write.write_all(b"250 queued\r\n").await?;
                }
                continue;
            }
            if auth_lines > 0 {
                auth_lines -= 1;
                let reply: &[u8] = if auth_lines == 0 {
                    b"235 authenticated\r\n"
                } else {
                    b"334 \r\n"
                };
                write.write_all(reply).await?;
                continue;
            }
            let upper = line.to_ascii_uppercase();
            let reply: &[u8] = if upper.starts_with("EHLO") {
                b"250-mock\r\n250 AUTH PLAIN LOGIN\r\n"
            } else if upper.starts_with("HELO") {
                b"250 mock\r\n"
            } else if upper.starts_with("AUTH PLAIN") {
                if upper.trim() == "AUTH PLAIN" {
                    auth_lines = 1;
                    b"334 \r\n"
                } else {
                    b"235 authenticated\r\n"
                }
            } else if upper.starts_with("AUTH LOGIN") {
                auth_lines = if upper.trim() == "AUTH LOGIN" { 2 } else { 1 };
                b"334 \r\n"
            } else if upper.starts_with("DATA") {
                in_data = true;
                b"354 go ahead\r\n"
            } else if upper.starts_with("QUIT") {
                write.write_all(b"221 bye\r\n").await?;
                return Ok(());
            } else {
                // MAIL FROM, RCPT TO, RSET, NOOP.
                b"250 ok\r\n"
            };
            write.write_all(reply).await?;
        }
        Ok(())
    }
}

// ── the server under test, as a child process ────────────────────────────────

/// One deployment: a directory holding the database, the web root and the server
/// log, plus the mail servers the engine talks to.
struct Deployment {
    dir: PathBuf,
    db: String,
    pop: Maildrop,
    smtp: Smtp,
}

/// A running `mailwoman serve`. Killed when dropped.
struct Server {
    child: Child,
    base: String,
}

impl Server {
    fn kill(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Deployment {
    async fn new(tag: &str) -> Deployment {
        let dir = test_db::unique_dir(tag);
        let web = dir.join("web");
        std::fs::create_dir_all(&web).unwrap();
        std::fs::write(web.join("index.html"), "<!doctype html><title>MW</title>").unwrap();
        Deployment {
            db: dir.join("mw.db").to_string_lossy().into_owned(),
            dir,
            pop: Maildrop::start().await,
            smtp: Smtp::start().await,
        }
    }

    /// Where the server puts the search index for this database.
    fn index_dir(&self) -> PathBuf {
        PathBuf::from(format!("{}.search-index", self.db))
    }

    fn log_path(&self) -> PathBuf {
        self.dir.join("server.log")
    }

    /// The end of the server log, for a failing assertion's message.
    fn log_tail(&self) -> String {
        let log = std::fs::read_to_string(self.log_path()).unwrap_or_default();
        let lines: Vec<&str> = log.lines().collect();
        lines[lines.len().saturating_sub(40)..].join("\n")
    }

    /// Start the server process without waiting for it or sending it anything.
    fn spawn(&self) -> Server {
        // A port that was free a moment ago. The child binds it itself.
        let port = std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let mut log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.log_path())
            .unwrap();
        writeln!(log, "──── server start, port {port} ────").unwrap();
        let child = Command::new(env!("CARGO_BIN_EXE_mailwoman"))
            .args(["serve", "--mode", "engine"])
            .args(["--bind", &format!("127.0.0.1:{port}")])
            .args(["--db-path", &self.db])
            .args(["--server-key", KEY_HEX])
            .arg("--web-dir")
            .arg(self.dir.join("web"))
            .env("MW_ENGINE_TLS", "plaintext")
            .env("MW_SMTP_HOST", "127.0.0.1")
            .env("MW_SMTP_PORT", self.smtp.addr.port().to_string())
            .env("MW_SMTP_SECURITY", "plaintext")
            .env("MW_ADMIN_USER", ADMIN_USER)
            .env("MW_ADMIN_PASSWORD", ADMIN_PASS)
            .env_remove("MW_SEARCH_DIR")
            .env_remove("MW_UPLOAD_DIR")
            .env_remove("MW_REDIS_URL")
            .env_remove("MW_ACME")
            .stdin(Stdio::null())
            .stdout(log.try_clone().unwrap())
            .stderr(log)
            .spawn()
            .expect("spawn `mailwoman serve`");
        Server {
            child,
            base: format!("http://127.0.0.1:{port}"),
        }
    }

    /// Start the server and wait until it answers `/healthz`.
    async fn start(&self) -> Server {
        let server = self.spawn();
        let c = reqwest::Client::builder().no_proxy().build().unwrap();
        let deadline = Instant::now() + Duration::from_secs(60);
        loop {
            if let Ok(r) = c.get(format!("{}/healthz", server.base)).send().await
                && r.status() == 200
            {
                return server;
            }
            assert!(
                Instant::now() < deadline,
                "the server did not come up:\n{}",
                self.log_tail()
            );
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

// ── HTTP helpers ─────────────────────────────────────────────────────────────

fn browser() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

/// Log the mailbox user in. The engine resyncs the maildrop during this call.
async fn login(c: &reqwest::Client, base: &str, pop: &Maildrop) {
    let r = c
        .post(format!("{base}/api/login"))
        .json(&json!({ "jmapUrl": pop.url(), "username": USER, "password": PASS }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "mailbox login");
}

async fn account_id(c: &reqwest::Client, base: &str) -> String {
    let session: Value = c
        .get(format!("{base}/jmap/session"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    session["primaryAccounts"]["urn:ietf:params:jmap:mail"]
        .as_str()
        .unwrap_or_else(|| panic!("no primary mail account in {session}"))
        .to_string()
}

async fn jmap(c: &reqwest::Client, base: &str, calls: Value) -> Value {
    let r = c
        .post(format!("{base}/jmap/api"))
        .json(&json!({
            "using": [
                "urn:ietf:params:jmap:core",
                "urn:ietf:params:jmap:mail",
                "urn:ietf:params:jmap:submission"
            ],
            "methodCalls": calls,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "POST /jmap/api");
    r.json().await.unwrap()
}

/// The ids `Email/query` returns for a full-text search for `token`.
async fn search(c: &reqwest::Client, base: &str, account: &str, token: &str) -> Vec<String> {
    let resp = jmap(
        c,
        base,
        json!([["Email/query", { "accountId": account, "filter": { "text": token } }, "q"]]),
    )
    .await;
    resp["methodResponses"][0][1]["ids"]
        .as_array()
        .unwrap_or_else(|| panic!("Email/query answered {resp}"))
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

/// How many messages the store holds for the account: `Email/query` scoped to
/// each of its mailboxes in turn, which reads the store and not the index.
async fn stored(c: &reqwest::Client, base: &str, account: &str) -> usize {
    let resp = jmap(
        c,
        base,
        json!([["Mailbox/get", { "accountId": account, "ids": null }, "m"]]),
    )
    .await;
    let mailboxes: Vec<String> = resp["methodResponses"][0][1]["list"]
        .as_array()
        .unwrap_or_else(|| panic!("Mailbox/get answered {resp}"))
        .iter()
        .map(|m| m["id"].as_str().unwrap().to_string())
        .collect();
    let mut n = 0;
    for mailbox in mailboxes {
        let resp = jmap(
            c,
            base,
            json!([["Email/query", { "accountId": account, "filter": { "inMailbox": mailbox } }, "q"]]),
        )
        .await;
        n += resp["methodResponses"][0][1]["ids"]
            .as_array()
            .unwrap_or_else(|| panic!("Email/query answered {resp}"))
            .len();
    }
    n
}

async fn admin(base: &str) -> reqwest::Client {
    let c = browser();
    let r = c
        .post(format!("{base}/admin/login"))
        .json(&json!({ "username": ADMIN_USER, "password": ADMIN_PASS }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "admin login");
    c
}

async fn index_status(admin: &reqwest::Client, base: &str) -> Value {
    let r = admin
        .get(format!("{base}/admin/maintenance/search-index"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "GET /admin/maintenance/search-index");
    r.json().await.unwrap()
}

/// Poll the index status until `done` accepts it.
async fn wait_for_index(
    admin: &reqwest::Client,
    base: &str,
    d: &Deployment,
    what: &str,
    done: impl Fn(&Value) -> bool,
) -> Value {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let status = index_status(admin, base).await;
        if done(&status) {
            return status;
        }
        assert!(
            Instant::now() < deadline,
            "{what}: the index status stayed {status}\n{}",
            d.log_tail()
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// Whether any file under `dir` (at any depth) contains `needle`.
fn dir_contains(dir: &Path, needle: &str) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if dir_contains(&path, needle) {
                return true;
            }
        } else if std::fs::read(&path)
            .unwrap_or_default()
            .windows(needle.len())
            .any(|w| w == needle.as_bytes())
        {
            return true;
        }
    }
    false
}

/// Three messages, the second of which carries `token`.
fn three_messages(pop: &Maildrop, token: &str) {
    pop.deliver("m1", "ordinary");
    pop.deliver("m2", token);
    pop.deliver("m3", "unremarkable");
}

// ── tests ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn a_search_hit_survives_a_restart_without_a_fetch_or_a_rebuild() {
    const TOKEN: &str = "zqxjkvpersist";
    let d = Deployment::new("mw-t28-e13-persist").await;
    three_messages(&d.pop, TOKEN);

    let first = d.start().await;
    let c = browser();
    login(&c, &first.base, &d.pop).await;
    let account = account_id(&c, &first.base).await;
    let hit = search(&c, &first.base, &account, TOKEN).await;
    assert_eq!(
        hit.len(),
        1,
        "precondition: the message is found before the restart"
    );
    assert_eq!(
        d.pop.retrs(),
        3,
        "precondition: three messages were fetched once each"
    );
    assert!(
        d.index_dir().is_dir(),
        "the index is on disk beside the database, at {}",
        d.index_dir().display()
    );
    first.kill();

    let second = d.start().await;
    // The same cookie: the session is in the database, the key is the same.
    let after = search(&c, &second.base, &account, TOKEN).await;
    assert_eq!(
        after,
        hit,
        "the same message is found after the restart\n{}",
        d.log_tail()
    );
    assert_eq!(d.pop.retrs(), 3, "no message was fetched again");
    // And it was found in what was on disk, not in a rebuild: this process has
    // not run one.
    let status = index_status(&admin(&second.base).await, &second.base).await;
    assert_eq!(status["persistent"], true, "{status}");
    assert_eq!(status["documents"], 3, "{status}");
    assert_eq!(status["messages"], 3, "{status}");
    assert_eq!(
        status["rebuildTotal"], 0,
        "no rebuild ran in this process: {status}"
    );
}

#[tokio::test]
async fn a_deleted_index_directory_is_rebuilt_from_the_store_at_start() {
    const TOKEN: &str = "zqxjkvbackfill";
    let d = Deployment::new("mw-t28-e13-backfill").await;
    three_messages(&d.pop, TOKEN);

    let first = d.start().await;
    let c = browser();
    login(&c, &first.base, &d.pop).await;
    let account = account_id(&c, &first.base).await;
    let hit = search(&c, &first.base, &account, TOKEN).await;
    assert_eq!(
        hit.len(),
        1,
        "precondition: the message is found before the restart"
    );
    first.kill();

    std::fs::remove_dir_all(d.index_dir()).expect("delete the index directory");
    assert!(!d.index_dir().exists());

    let second = d.start().await;
    let a = admin(&second.base).await;
    let status = wait_for_index(&a, &second.base, &d, "start-up rebuild", |s| {
        s["documents"] == 3 && s["rebuilding"] == false
    })
    .await;
    assert_eq!(
        status["rebuildTotal"], 3,
        "a rebuild of three messages ran: {status}"
    );
    assert_eq!(status["rebuildDone"], 3, "{status}");
    assert_eq!(
        d.pop.retrs(),
        3,
        "the rebuild read the store, not the mail server"
    );
    let after = search(&c, &second.base, &account, TOKEN).await;
    assert_eq!(after, hit, "the same message is found after the rebuild");

    // The admin route rebuilds on request and reports what it did.
    let r = a
        .post(format!("{}/admin/maintenance/reindex", second.base))
        .json(&json!({ "accountId": account }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let summary: Value = r.json().await.unwrap();
    assert_eq!(summary["accounts"], 1, "{summary}");
    assert_eq!(summary["messages"], 3, "{summary}");
    assert_eq!(summary["indexed"], 3, "{summary}");
    assert_eq!(summary["removed"], 0, "{summary}");
    // It is behind the admin session.
    let anonymous = browser()
        .post(format!("{}/admin/maintenance/reindex", second.base))
        .json(&json!({}))
        .send()
        .await
        .unwrap();
    assert_eq!(anonymous.status(), 401);
    let anonymous = browser()
        .get(format!("{}/admin/maintenance/search-index", second.base))
        .send()
        .await
        .unwrap();
    assert_eq!(anonymous.status(), 401);
}

#[tokio::test]
async fn a_zero_access_account_has_no_text_in_the_index_directory() {
    const BEFORE: &str = "zqxjkvbefore";
    const AFTER: &str = "wpfhgyafter";
    let d = Deployment::new("mw-t28-e13-posture").await;
    d.pop.deliver("m1", BEFORE);

    let server = d.start().await;
    let c = browser();
    login(&c, &server.base, &d.pop).await;
    let account = account_id(&c, &server.base).await;
    // Precondition, and the proof that the scan below can find something: as a
    // standard account, the message's text is in the index directory.
    assert_eq!(search(&c, &server.base, &account, BEFORE).await.len(), 1);
    assert!(
        dir_contains(&d.index_dir(), BEFORE),
        "a standard account's indexed text is found by the scan"
    );

    // The account switches to zero-access while the server runs.
    let r = c
        .post(format!("{}/api/zeroaccess/enable", server.base))
        .json(&json!({
            "saltB64": "c2FsdA==",
            "kdfParams": { "alg": "argon2id" },
            "wrappedDataKeyB64": "d3JhcHBlZA==",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "zero-access is enabled");

    // A message arrives afterwards. Logging in again resyncs the maildrop.
    d.pop.deliver("m2", AFTER);
    login(&c, &server.base, &d.pop).await;
    assert_eq!(d.pop.retrs(), 2, "the new message was fetched");
    assert_eq!(
        stored(&c, &server.base, &account).await,
        2,
        "and stored: the account has two messages"
    );

    assert!(
        !dir_contains(&d.index_dir(), AFTER),
        "mail that arrived after the switch was never written to the index directory"
    );
    assert!(
        !dir_contains(&d.index_dir(), BEFORE),
        "mail indexed before the switch was removed from the index directory"
    );
    assert!(search(&c, &server.base, &account, AFTER).await.is_empty());
    assert!(search(&c, &server.base, &account, BEFORE).await.is_empty());
    let a = admin(&server.base).await;
    let status = index_status(&a, &server.base).await;
    assert_eq!(status["documents"], 0, "{status}");
    assert_eq!(status["zeroAccessDocuments"], 0, "{status}");
    assert_eq!(
        status["messages"], 0,
        "a zero-access account's mail is not counted: {status}"
    );

    // A rebuild does not put it back, at start-up or on request.
    let r = a
        .post(format!("{}/admin/maintenance/reindex", server.base))
        .json(&json!({}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let summary: Value = r.json().await.unwrap();
    assert_eq!(summary["zeroAccess"], 1, "{summary}");
    assert_eq!(summary["indexed"], 0, "{summary}");
    server.kill();
    let restarted = d.start().await;
    let a = admin(&restarted.base).await;
    let status = wait_for_index(&a, &restarted.base, &d, "after restart", |s| {
        s["rebuilding"] == false
    })
    .await;
    assert_eq!(status["documents"], 0, "{status}");
    assert!(!dir_contains(&d.index_dir(), AFTER));
    assert!(!dir_contains(&d.index_dir(), BEFORE));
}

#[tokio::test]
async fn a_due_submission_is_sent_after_a_restart_with_no_request_made() {
    let d = Deployment::new("mw-t28-e13-sendlater").await;
    let first = d.start().await;
    let c = browser();
    login(&c, &first.base, &d.pop).await;
    let account = account_id(&c, &first.base).await;

    let send_at = (chrono::Utc::now() + chrono::Duration::seconds(6)).to_rfc3339();
    let resp = jmap(
        &c,
        &first.base,
        json!([
            ["Email/set", { "accountId": account, "create": { "draft": {
                "from": [{ "email": USER }],
                "to": [{ "email": "friend@example.org" }],
                "subject": "Sent later",
                "bodyValues": { "1": { "value": "queued before the restart" } },
                "textBody": [{ "partId": "1", "type": "text/plain" }]
            } } }, "c1"],
            ["EmailSubmission/set", { "accountId": account, "create": { "s1": {
                "emailId": "#draft", "sendAt": send_at
            } } }, "c2"]
        ]),
    )
    .await;
    let created = &resp["methodResponses"][1][1]["created"]["s1"];
    assert_eq!(
        created["undoStatus"], "pending",
        "the submission is queued: {resp}"
    );
    first.kill();
    assert_eq!(
        d.smtp.delivered(),
        0,
        "precondition: nothing was sent before the restart"
    );
    let logins_before = d.pop.logins();

    // Start the server and send it nothing at all: not a login, not a health probe.
    let _second = d.spawn();
    let deadline = Instant::now() + Duration::from_secs(60);
    while d.smtp.delivered() == 0 {
        assert!(
            Instant::now() < deadline,
            "the submission was not sent after the restart:\n{}",
            d.log_tail()
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(d.smtp.delivered(), 1, "sent once");
    assert!(
        d.pop.logins() > logins_before,
        "the account was connected by the server itself"
    );
    // The dispatcher keeps running; the message is not sent a second time.
    tokio::time::sleep(Duration::from_secs(2)).await;
    assert_eq!(d.smtp.delivered(), 1, "still sent once");
}

#[tokio::test]
async fn a_disabled_account_is_not_connected_at_start() {
    let d = Deployment::new("mw-t28-e13-disabled").await;
    let first = d.start().await;
    let c = browser();
    login(&c, &first.base, &d.pop).await;
    first.kill();

    // Precondition: an enabled account IS connected at start, and this is how
    // long that takes here.
    let before = d.pop.logins();
    let second = d.start().await;
    let started = Instant::now();
    while d.pop.logins() == before {
        assert!(
            started.elapsed() < Duration::from_secs(60),
            "an enabled account was not connected at start:\n{}",
            d.log_tail()
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let took = started.elapsed();

    // The admin disables it.
    let a = admin(&second.base).await;
    let r = a
        .put(format!("{}/admin/users/{USER}/flags", second.base))
        .json(&json!({
            "zeroAccess": false,
            "forcePasswordChange": false,
            "remoteCacheWipe": false,
            "disabled": true,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 204, "the flag write is accepted");
    second.kill();

    let before = d.pop.logins();
    let _third = d.start().await;
    // Wait several times as long as the connection took when it was allowed.
    tokio::time::sleep((took * 5).max(Duration::from_secs(3))).await;
    assert_eq!(
        d.pop.logins(),
        before,
        "the server did not log in to the mail server for a disabled account\n{}",
        d.log_tail()
    );
}
