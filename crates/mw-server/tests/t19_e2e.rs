//! t19-e-e2e — the 26.19 live end-to-end suite (the target
//! `.github/workflows/t19-conformance.yml` drives).
//!
//! Three 26.19 surfaces shipped with a seam that only a live run can prove, and each
//! was explicitly deferred to this lane by the lane that built it:
//!
//! 1. **A8 semantic re-rank** (`t19-e8`). The re-rank is unit-covered against an
//!    in-crate `HashEmbedder`, but nothing had ever proved that a *deployment* which
//!    enables Assist and grants `search-semantic` actually ends up with an embedding
//!    provider attached to the engine the server booted. That chain —
//!    `assist_config` row → `build_assist` → `AssistHookAdapter::from_gateway` →
//!    `attach_v7` → `Engine::attach_embeddings` — is entirely `pub(crate)`, so the
//!    only way to exercise it is through `build_app` and a real HTTP request. This
//!    suite drives `Email/query` over the wire, with and without `filter.semantic`,
//!    against a **real HTTP embeddings endpoint**, and asserts the flag produces a
//!    different, embedding-justified ordering while the default path stays
//!    byte-identical to a server that has no Assist config at all.
//! 2. **The Assist `actions` HTTP seam** (`t19-e7`). `actions` was covered
//!    server-side through the function the route calls and client-side on both
//!    paths, but nothing posted to `/api/assist/invoke` over a real router and read
//!    SSE bytes off the wire. That leg is here: one live
//!    `disclosure → delta → done` sequence carrying a proposal.
//! 3. **Appearance sync** (`t19-e13`). `GET`/`PUT`/`DELETE /api/account/appearance`
//!    round-trip, the 8 KiB cap, and the reset-stores-a-newer-null behaviour that is
//!    what stops a stale device resurrecting a forgotten appearance.
//!
//! Plus the live-Postgres item `t19-e2` handed on: every existing live-PG test
//! binary points every one of its tests at the **same** database, so they all race on
//! one `_sqlx_migrations` table. This suite does not. [`pg_fresh_db`] gives each
//! live-PG leg its own freshly `CREATE DATABASE`d database, which isolates the legs
//! *and* is exactly the "0022 applies from a fresh DB" condition the migration leg
//! needs — one mechanism discharging both.
//!
//! # What is live, and what loud-skips
//!
//! Nothing here silently passes when its service is missing, and nothing simulates a
//! service it was told to use:
//!
//! * `MW_E14_PG_DSN` **unset** ⇒ the Postgres legs print why they did not run and
//!   return. **Set but unreachable** ⇒ the leg PANICS. A configured gate that cannot
//!   be honoured is a failure, not a skip.
//! * `MW_TEST_ASSIST_URL` (the `scripts/mock-assist` container) **unset** ⇒ the legs
//!   that specifically prove the *shipped* mock loud-skip; the A8 ordering legs still
//!   run against an in-process endpoint (see below). **Set but unreachable** ⇒ PANIC.
//!
//! ## Why an in-process embeddings endpoint exists as well
//!
//! [`spawn_assist_mock`] is a real HTTP server speaking the OpenAI-compatible wire
//! shape, reached through the gateway's real `reqwest` client. It exists for two
//! reasons, neither of which is "stand in for Docker":
//!
//! * `scripts/mock-assist/server.py` returns a fixed reply text with no
//!   `<<<MW_ACTIONS` block, so it **cannot** produce a proposal — the one thing the
//!   `actions` seam has to carry. Only an endpoint this suite controls can.
//! * It lets the A8 ordering legs run on a host with no container runtime. Its
//!   embedding function is a deliberate mirror of the Python one (FNV-1a over a
//!   64-wide hashed bag of words), and [`the_in_process_embedder_matches_the_shipped_mock`]
//!   pins that correspondence on published vectors so the two cannot drift apart
//!   unnoticed.
//!
//! When `MW_TEST_ASSIST_URL` IS set, [`a8_semantic_rerank_reorders_live_over_http`]
//! runs against that endpoint instead, so CI proves the ordering against the shipped
//! mock. That mock had to be FIXED for any of this to mean anything: before
//! `dae48ea` it returned one constant vector for every input, so every cosine was
//! equal and the correct tie-stable answer *was* the lexical order — a live test
//! against it would have gone green while proving only that the plumbing runs.
//!
//! **The ordering assertion was negative-controlled against exactly that.** With
//! `MW_TEST_ASSIST_URL` pointed at a throwaway endpoint reproducing the pre-`dae48ea`
//! behaviour (one fixed vector for every input), the leg FAILS — `semantic: true`
//! returns the lexical order byte for byte, and the assertion says so by name. So a
//! pass here is evidence about the feature, not about the plumbing. Re-run that check
//! if the corpus or the endpoint ever changes; it is the only thing standing between
//! this suite and the failure mode it was written to prevent.
//!
//! # Running
//!
//! ```text
//! cargo test -p mw-server --test t19_e2e -- --nocapture --test-threads=1
//!
//! docker compose -f docker-compose.ci.yml up -d --build --wait postgres mock-assist
//! MW_E14_PG_DSN=postgres://mailwoman:mailwoman@localhost:5432/mailwoman \
//! MW_TEST_ASSIST_URL=http://127.0.0.1:8199 \
//!   cargo test -p mw-server --test t19_e2e -- --nocapture --test-threads=1
//! ```

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use axum::Router;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::post;
use futures_util::StreamExt as _;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, build_app};
use mw_store::{AccountKind, AssistConfigRow, Credentials, NewAccount, ServerKey, Store};

mod common;
use common::test_db;

/// A fixed server key, so a store opened alongside the server unseals the same rows.
const KEY_HEX: &str = "1c2b3a49586776859493a2b1c0dfee0d1c2b3a49586776859493a2b1c0dfee0d";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

/// The embedding model id the seeded deployment config names. It has to be stable:
/// a cached vector is skipped when its `model` disagrees with the live provider's.
const EMBED_MODEL: &str = "t19-mock-embed";
/// Width of the mock's vectors — the same 64 the shipped `scripts/mock-assist` uses.
const EMBED_DIM: usize = 64;

/// The search term every seeded message matches, so the lexical hit set is fixed and
/// only the ORDER can differ.
const QUERY: &str = "report";

// ─────────────────────────────────────────────────────────────────────────────
// Environment gates. A gate that is SET but not honourable fails loudly.
// ─────────────────────────────────────────────────────────────────────────────

fn pg_dsn() -> Option<String> {
    std::env::var("MW_E14_PG_DSN")
        .or_else(|_| std::env::var("DATABASE_URL_PG"))
        .ok()
        .filter(|s| !s.is_empty())
}

fn shipped_assist_url() -> Option<String> {
    std::env::var("MW_TEST_ASSIST_URL")
        .ok()
        .filter(|s| !s.is_empty())
        .map(|s| s.trim_end_matches('/').to_string())
}

fn skip(leg: &str, why: &str) {
    common::gate::skip(format_args!("[t19-e2e] {leg}: {why}"));
}

/// Prove the shipped mock is actually up before any leg claims to have used it.
/// Unreachable-while-configured is a failure: the alternative is a green run that
/// proves nothing, which is the exact defect class this suite exists to catch.
async fn require_shipped_assist(url: &str) {
    let resp = reqwest::Client::new()
        .get(format!("{url}/healthz"))
        .timeout(Duration::from_secs(10))
        .send()
        .await;
    let body = match resp {
        Ok(r) => r.text().await.unwrap_or_default(),
        Err(e) => panic!(
            "MW_TEST_ASSIST_URL={url} is set but the mock Assist endpoint is not reachable: {e}. \
             Bring it up (docker compose -f docker-compose.ci.yml up -d --build --wait mock-assist) \
             or unset the variable — a configured gate must never be skipped."
        ),
    };
    assert_eq!(
        body.trim(),
        "ok",
        "MW_TEST_ASSIST_URL={url}/healthz answered {body:?}, not \"ok\" — that is not the \
         scripts/mock-assist server"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Live Postgres: one FRESH database per leg (t19-e2's hand-off).
// ─────────────────────────────────────────────────────────────────────────────

/// Create a brand-new database on the live server and return a DSN pointing at it.
///
/// This is the fix for the finding `t19-e2` could not reproduce without Docker:
/// `db_target()` in `t10_dcr`, `t11_dcr_admin`, `t12_ews_auth`, `t13_jwz`,
/// `t14_*`, `t15_*`, `t16_*`, `t17_note_seal`, `t18_e2e_vacuum` and `v6_e2e` all hand
/// **every test in the binary the same DSN**, so every one of them runs
/// `sqlx::migrate!` against one database and races on `_sqlx_migrations` —
/// per-test temp paths do nothing about it because the path is not what collides.
/// A per-leg database removes the shared object entirely, and as a bonus it is
/// literally the "fresh DB" condition the 0022 leg has to assert on.
///
/// The database is left behind on purpose (the temp-dir policy, same reasoning):
/// a failed run's state stays inspectable, and CI drops the whole container with
/// `docker compose down -v`. A re-run drops and recreates its own.
async fn pg_fresh_db(admin_dsn: &str, tag: &str) -> String {
    let name = format!("mw_t19_{tag}");
    let pool = match sqlx::postgres::PgPool::connect(admin_dsn).await {
        Ok(p) => p,
        Err(e) => panic!(
            "MW_E14_PG_DSN is set but Postgres is not reachable: {e}. Bring it up \
             (docker compose -f docker-compose.ci.yml up -d --wait postgres) or unset the \
             variable — a configured gate must never be skipped."
        ),
    };
    // Neither statement may run inside a transaction, so they go one at a time.
    sqlx::query(&format!("DROP DATABASE IF EXISTS {name}"))
        .execute(&pool)
        .await
        .unwrap_or_else(|e| panic!("dropping a stale {name}: {e}"));
    sqlx::query(&format!("CREATE DATABASE {name}"))
        .execute(&pool)
        .await
        .unwrap_or_else(|e| panic!("creating a fresh {name}: {e}"));
    pool.close().await;
    swap_database(admin_dsn, &name)
}

/// Rewrite the database name in a `postgres://…/<db>[?params]` DSN.
fn swap_database(dsn: &str, db: &str) -> String {
    let (head, query) = match dsn.split_once('?') {
        Some((h, q)) => (h, Some(q)),
        None => (dsn, None),
    };
    let base = head.trim_end_matches('/');
    let cut = base.rfind('/').expect("a DSN carries a database path");
    let mut out = format!("{}/{db}", &base[..cut]);
    if let Some(q) = query {
        out.push('?');
        out.push_str(q);
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// The in-process Assist endpoint (OpenAI-compatible), reached over real HTTP.
// ─────────────────────────────────────────────────────────────────────────────

/// A deterministic, content-DEPENDENT embedding: each token lands in one bucket of a
/// fixed-width vector, so texts sharing vocabulary sit near each other under cosine.
///
/// This mirrors `scripts/mock-assist/server.py::_embed` and
/// `mw_engine::search_semantic`'s in-crate `HashEmbedder` — FNV-1a 64-bit, one
/// bucket per token, 64 wide, never the zero vector. It must stay content-dependent
/// for the reason written at length in the Python file: a constant vector makes every
/// cosine equal, and the correct tie-stable re-rank of equal cosines IS the lexical
/// order, so the ordering assertions below would pass while proving nothing.
fn hashed_embedding(text: &str) -> Vec<f32> {
    let mut v = vec![0.0_f32; EMBED_DIM];
    for token in text.split(|c: char| !c.is_alphanumeric()) {
        if token.is_empty() {
            continue;
        }
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in token.to_lowercase().bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x100_0000_01b3);
        }
        v[(h % EMBED_DIM as u64) as usize] += 1.0;
    }
    if v.iter().all(|x| *x == 0.0) {
        v[0] = 1.0;
    }
    v
}

/// Spawn the in-process endpoint and return its API root (already `…/v1`).
///
/// `/chat/completions` streams SSE in the OpenAI shape and ends its reply with a real
/// [`mw_assist::ACTION_MARKER`] block, so the `actions` seam has something to carry.
/// The marker is deliberately split across two SSE frames: the proposal filter holds
/// back bytes that could still complete the marker, and a marker leaking as visible
/// prose is exactly the failure a single-frame reply would not catch.
async fn spawn_assist_mock() -> String {
    let app = Router::new()
        .route(
            "/v1/embeddings",
            post(|body: axum::Json<Value>| async move {
                let input = body
                    .0
                    .get("input")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                axum::Json(json!({
                    "object": "list",
                    "model": body.0.get("model").cloned().unwrap_or(json!("mock-embed")),
                    "data": [{ "object": "embedding", "index": 0,
                               "embedding": hashed_embedding(&input) }],
                }))
            }),
        )
        .route(
            "/v1/chat/completions",
            post(|| async move {
                let marker = mw_assist::ACTION_MARKER;
                let (head, tail) = marker.split_at(4);
                let actions =
                    r#"[{"tool":"mail.search","summary":"Search for the invoice thread"}]"#;
                let chunks: Vec<String> = vec![
                    "Here is ".into(),
                    "the reply.".into(),
                    // The marker straddles a frame boundary on purpose.
                    format!("\n{head}"),
                    format!("{tail}\n{actions}"),
                ];
                let stream =
                    futures_util::stream::iter(chunks.into_iter().map(|delta| {
                        Ok::<Event, std::convert::Infallible>(Event::default().data(
                        json!({ "choices": [{ "index": 0, "delta": { "content": delta } }] })
                            .to_string(),
                    ))
                    }))
                    .chain(futures_util::stream::iter([Ok(Event::default(
                    )
                    .data("[DONE]"))]));
                Sse::new(stream).keep_alive(KeepAlive::default())
            }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}/v1")
}

// ─────────────────────────────────────────────────────────────────────────────
// A minimal POP3 maildrop, so engine mode can register an account.
// ─────────────────────────────────────────────────────────────────────────────

/// Every HTTP route into `Engine::handle_jmap` calls `engine_mode::ensure_account`
/// first, which connects a real backend and resyncs before the request is served.
/// Without an upstream, `/jmap` answers `502` and no amount of correct search wiring
/// is observable over HTTP.
///
/// POP3 is the cheap upstream: an empty maildrop resyncs to an empty INBOX (which is
/// also how the account's inbox gets created), and an empty `UIDL` listing means the
/// delta has nothing added and — because the cursor starts empty — nothing removed,
/// so the messages this suite imports afterwards are never touched. The account's
/// poll interval is the 300 s default, so the watcher cannot tick inside a test.
async fn spawn_pop3_mock() -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        loop {
            let Ok((sock, _)) = listener.accept().await else {
                break;
            };
            tokio::spawn(async move {
                let (r, mut w) = sock.into_split();
                let mut lines = BufReader::new(r).lines();
                if w.write_all(b"+OK mailwoman t19 mock POP3 ready\r\n")
                    .await
                    .is_err()
                {
                    return;
                }
                while let Ok(Some(line)) = lines.next_line().await {
                    let verb = line
                        .split_whitespace()
                        .next()
                        .unwrap_or_default()
                        .to_ascii_uppercase();
                    let reply: &[u8] = match verb.as_str() {
                        "CAPA" => b"+OK capability list follows\r\nUIDL\r\nTOP\r\nUSER\r\n.\r\n",
                        "USER" | "PASS" | "NOOP" | "RSET" => b"+OK\r\n",
                        "STAT" => b"+OK 0 0\r\n",
                        // Empty multi-line listings: no messages in the maildrop.
                        "UIDL" | "LIST" => b"+OK 0 messages\r\n.\r\n",
                        "QUIT" => b"+OK bye\r\n",
                        _ => b"-ERR unsupported in the t19 mock\r\n",
                    };
                    if w.write_all(reply).await.is_err() {
                        return;
                    }
                    if verb == "QUIT" {
                        return;
                    }
                }
            });
        }
    });
    addr
}

// ─────────────────────────────────────────────────────────────────────────────
// The server fixture.
// ─────────────────────────────────────────────────────────────────────────────

/// How a fixture wires Assist: the API root of an endpoint, and the granted
/// capabilities (kebab-case wire names, as an operator writes them into 0008).
struct AssistWiring {
    api_root: String,
    grants: Vec<&'static str>,
}

struct Fixture {
    base: String,
    account: String,
    cookie: String,
    store: Store,
}

/// The seeded corpus: message ids in NEWEST-FIRST order (the default lexical order),
/// plus the id of the one message that is genuinely about [`QUERY`].
struct Corpus {
    ids: Vec<String>,
    on_topic: String,
}

impl Corpus {
    /// Newest-first — the default lexical order.
    fn newest_first(&self) -> Vec<String> {
        self.ids.clone()
    }

    /// Oldest-first — what a blanket inversion of the default would produce, and the
    /// thing "the ordering changed" alone would not rule out.
    fn inverted(&self) -> Vec<String> {
        self.ids.iter().rev().cloned().collect()
    }
}

/// Seed a store (account + session + optional Assist config), then boot a real
/// engine-mode server over it and return everything a leg needs to drive HTTP.
async fn fixture(db_path: &str, pop3: SocketAddr, assist: Option<AssistWiring>) -> Fixture {
    let key = ServerKey::from_hex(KEY_HEX).unwrap();
    let store = Store::open(db_path, key).await.expect("store opens");

    let account = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Pop3,
                host: &pop3.ip().to_string(),
                port: pop3.port(),
                tls: "plaintext",
                username: "reader@example.org",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "reader@example.org".into(),
                password: "pw".into(),
            },
        )
        .await
        .expect("account row");

    if let Some(a) = &assist {
        let ceiling = json!({
            "accounts": [&account], "folders": [],
            "include_e2ee": false, "include_attachments": false
        });
        store
            .put_assist_config(&AssistConfigRow {
                scope: "deployment".into(),
                adapters_json: json!({
                    "kind": "open-ai-compatible",
                    "base_url": a.api_root,
                    "api_key": "t19-not-a-real-key",
                    "chat_model": "mock-chat",
                    "embed_model": EMBED_MODEL,
                })
                .to_string(),
                capability_grants_json: json!(a.grants).to_string(),
                data_ceilings_json: ceiling.to_string(),
                enabled: true,
            })
            .await
            .expect("0008 assist_config row");
    }

    let cookie = store
        .create_session(
            &account,
            "reader@example.org",
            "http://upstream.invalid",
            "http://upstream.invalid",
            &Credentials {
                username: "reader@example.org".into(),
                password: "pw".into(),
            },
        )
        .await
        .expect("session");

    let base = spawn_server(db_path).await;
    Fixture {
        base,
        account,
        cookie,
        store,
    }
}

/// Boot the real app in engine mode with a configured upload backend (import needs
/// one) and serve it on a loopback port.
async fn spawn_server(db_path: &str) -> String {
    let web = test_db::unique_dir("mw-t19-web");
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    // `MW_UPLOAD_DIR` is read at `build_app`; the value is process-wide but every
    // fixture writes the same directory, and the store keys objects per account.
    let uploads = upload_dir();
    unsafe { std::env::set_var("MW_UPLOAD_DIR", &uploads) };

    let config = AppConfig {
        db_path: db_path.to_string(),
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
    format!("http://{addr}")
}

/// One upload directory for the whole binary (the env var is process-wide).
fn upload_dir() -> PathBuf {
    static DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    DIR.get_or_init(|| test_db::unique_dir("mw-t19-uploads"))
        .clone()
}

impl Fixture {
    fn client(&self) -> reqwest::Client {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap()
    }

    /// `POST /jmap/api` with the session cookie — the same hop the browser makes.
    async fn jmap(&self, request: Value) -> Value {
        let resp = self
            .client()
            .post(format!("{}/jmap/api", self.base))
            .header(
                reqwest::header::COOKIE,
                format!("mw_session={}", self.cookie),
            )
            .json(&request)
            .send()
            .await
            .expect("jmap request");
        let status = resp.status();
        let body = resp.text().await.expect("jmap body");
        assert_eq!(
            status, 200,
            "POST /jmap/api must reach the engine (502 means ensure_account could not \
             register the POP3 upstream): {body}"
        );
        // Decoding failures are reported with the body: an empty 200 is what a
        // mis-addressed route looks like, and "expected value at line 1 column 1"
        // alone does not say so.
        serde_json::from_str(&body)
            .unwrap_or_else(|e| panic!("jmap response is not json ({e}): {body:?}"))
    }

    /// One `/jmap` round-trip, which is what drives `engine_mode::ensure_account`:
    /// the POP3 upstream is connected and resynced, and that resync is what creates
    /// the account's INBOX. The import route resolves its target mailbox through
    /// `Mailbox/get` WITHOUT calling `ensure_account`, so importing before this has
    /// happened answers `400 no target mailbox` — which is a genuine ordering
    /// property of the routes, not a fixture quirk.
    async fn register_account(&self) -> String {
        let resp = self
            .jmap(json!({
                "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
                "methodCalls": [["Mailbox/get",
                    { "accountId": self.account, "ids": Value::Null }, "m"]]
            }))
            .await;
        resp["methodResponses"][0][1]["list"]
            .as_array()
            .and_then(|list| {
                list.iter()
                    .find(|m| {
                        m["role"]
                            .as_str()
                            .is_some_and(|r| r.eq_ignore_ascii_case("inbox"))
                    })
                    .and_then(|m| m["id"].as_str())
            })
            .unwrap_or_else(|| panic!("the engine resync must have provisioned an INBOX: {resp}"))
            .to_string()
    }

    /// Upload one RFC822 message through `POST /jmap/upload/{accountId}` and import it
    /// with `Email/import`, both over HTTP. Returns the stable id the engine minted.
    ///
    /// The two-step form is used rather than the one-shot `/api/import/eml` because
    /// `Email/import` accepts an explicit `receivedAt`, and the corpus below depends on
    /// a KNOWN newest-first order. Deriving that order from wall-clock import times
    /// would make the ordering assertions depend on the host's timer resolution — the
    /// exact class of hidden dependency `test_db` was written to remove — and it did:
    /// an earlier draft identified the on-topic message by its POSITION in the very
    /// result it was about to assert on, which is circular, and it went green or red
    /// depending on which of three near-simultaneous timestamps sorted first.
    async fn import(&self, mailbox: &str, subject: &str, body: &str, received_at: &str) -> String {
        let eml = format!(
            "Message-ID: <t19-{subject}@example.org>\r\nFrom: sender@example.com\r\n\
             To: reader@example.org\r\nSubject: {subject}\r\n\
             Date: Mon, 01 Jan 2024 00:00:00 +0000\r\n\r\n{body}"
        );
        let resp = self
            .client()
            .post(format!("{}/jmap/upload/{}", self.base, self.account))
            .header(
                reqwest::header::COOKIE,
                format!("mw_session={}", self.cookie),
            )
            .header(reqwest::header::CONTENT_TYPE, "message/rfc822")
            .body(eml)
            .send()
            .await
            .expect("upload request");
        let status = resp.status();
        let uploaded: Value = resp.json().await.expect("upload response is json");
        assert_eq!(status, 200, "upload must succeed: {uploaded}");
        let blob = uploaded["blobId"]
            .as_str()
            .unwrap_or_else(|| panic!("upload returns a blobId: {uploaded}"))
            .to_string();

        let resp = self
            .jmap(json!({
                "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
                "methodCalls": [["Email/import", {
                    "accountId": self.account,
                    "emails": { "i0": {
                        "blobId": blob,
                        "mailboxIds": { mailbox: true },
                        "receivedAt": received_at,
                    }},
                }, "imp"]]
            }))
            .await;
        resp["methodResponses"][0][1]["created"]["i0"]["id"]
            .as_str()
            .unwrap_or_else(|| panic!("Email/import created the message: {resp}"))
            .to_string()
    }

    /// `Email/query` for [`QUERY`], optionally carrying `filter.semantic`.
    async fn query(&self, semantic: Option<bool>) -> Vec<String> {
        let mut filter = json!({ "text": QUERY });
        if let Some(s) = semantic {
            filter["semantic"] = Value::Bool(s);
        }
        let resp = self
            .jmap(json!({
                "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
                "methodCalls": [["Email/query",
                    { "accountId": self.account, "filter": filter }, "q"]]
            }))
            .await;
        resp["methodResponses"][0][1]["ids"]
            .as_array()
            .unwrap_or_else(|| panic!("Email/query returned no ids: {resp}"))
            .iter()
            .map(|v| v.as_str().unwrap_or_default().to_string())
            .collect()
    }

    /// Seed the three-message corpus with EXPLICIT `receivedAt` stamps and return it.
    ///
    /// Every message matches [`QUERY`], so the hit SET is fixed and only the ORDER can
    /// differ — which is what makes an ordering assertion meaningful at all.
    ///
    /// **The on-topic message is deliberately the MIDDLE one by date.** It is purely
    /// about the query term; the other two merely mention it among unrelated
    /// vocabulary, so the query embedding is closest to it. Putting it in the middle is
    /// what makes the assertion discriminating: newest-first buries it second, and the
    /// *inverse* of newest-first buries it second as well. "The on-topic message is
    /// first" therefore cannot be satisfied by leaving the order alone, by inverting
    /// it, or by any other permutation of an unrelated signal — only by an ordering the
    /// vectors actually justify.
    async fn seed_corpus(&self) -> Corpus {
        let inbox = self.register_account().await;
        let lunch = self
            .import(
                &inbox,
                "Lunch",
                "sandwiches coffee friday cafeteria menu queue report",
                "2024-01-01T00:00:00Z",
            )
            .await;
        let on_topic = self
            .import(&inbox, "Report", "report", "2024-02-01T00:00:00Z")
            .await;
        let standup = self
            .import(
                &inbox,
                "Standup",
                "sprint board standup notes retro planning velocity report",
                "2024-03-01T00:00:00Z",
            )
            .await;

        let ids = self.query(None).await;
        assert_eq!(
            ids,
            vec![standup.clone(), on_topic.clone(), lunch.clone()],
            "the corpus must start out newest-first with the on-topic message in the \
             MIDDLE — every ordering assertion below reads its meaning from that"
        );
        Corpus { ids, on_topic }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// LEG 1 — A8: `semantic: true` reorders; the default path does not move.
// ─────────────────────────────────────────────────────────────────────────────

/// Drive the whole A8 chain over HTTP on one dialect. Returns the reranked order so
/// the caller can assert dialect-specific things about it.
async fn drive_semantic(db_path: &str, dialect: &str, api_root: String) -> Vec<String> {
    let pop3 = spawn_pop3_mock().await;
    let f = fixture(
        db_path,
        pop3,
        Some(AssistWiring {
            api_root,
            grants: vec!["search-semantic"],
        }),
    )
    .await;

    // The deployment config was read at mount: the gateway is enabled and reports the
    // capability that gates the provider. (A `false`/absent here would mean every
    // ordering assertion below was measuring a server with no provider attached.)
    let cfg: Value = f
        .client()
        .get(format!("{}/api/assist/config", f.base))
        .header(reqwest::header::COOKIE, format!("mw_session={}", f.cookie))
        .send()
        .await
        .expect("assist config")
        .json()
        .await
        .expect("assist config json");
    assert_eq!(
        cfg["availability"],
        json!("enabled"),
        "[{dialect}] the seeded 0008 row must produce an ENABLED gateway: {cfg}"
    );
    assert!(
        cfg["capabilities"]
            .as_array()
            .is_some_and(|c| c.contains(&json!("search-semantic"))),
        "[{dialect}] search-semantic must be granted, else no provider is attached: {cfg}"
    );

    let corpus = f.seed_corpus().await;

    // Lexical: newest first, which puts the on-topic message in the MIDDLE.
    let lexical = f.query(None).await;
    assert_eq!(
        lexical,
        corpus.newest_first(),
        "[{dialect}] the default path is newest-first"
    );
    assert_eq!(
        f.store.count_message_embeddings(&f.account).await.unwrap(),
        0,
        "[{dialect}] the default path must not embed anything — a search feature that \
         proxies mail to an AI endpoint without being asked is an egress change nobody \
         opted into"
    );
    assert_eq!(
        f.query(Some(false)).await,
        lexical,
        "[{dialect}] semantic:false is byte-identical to the pre-feature result"
    );

    // Semantic: a DIFFERENT order, and one the embeddings justify.
    let semantic = f.query(Some(true)).await;
    assert_ne!(
        semantic, lexical,
        "[{dialect}] semantic:true must change the ordering. Identical output here is \
         the signature of an endpoint returning a CONSTANT vector: every cosine is then \
         equal and the correct tie-stable answer IS the lexical order"
    );
    assert_eq!(
        semantic.first(),
        Some(&corpus.on_topic),
        "[{dialect}] the message that is actually about {QUERY:?} wins on cosine. It sits \
         SECOND in the lexical order and second in the inverted one, so nothing but an \
         embedding-justified ordering can put it first"
    );
    // The control that "different" alone would not give: an implementation that
    // simply inverted the default order would satisfy `assert_ne!` above.
    assert_ne!(
        semantic,
        corpus.inverted(),
        "[{dialect}] the re-rank is not a blanket inversion of the default order"
    );
    let (mut a, mut b) = (semantic.clone(), lexical.clone());
    a.sort();
    b.sort();
    assert_eq!(a, b, "[{dialect}] a re-rank is a permutation, not a filter");
    assert_eq!(
        f.store.count_message_embeddings(&f.account).await.unwrap(),
        3,
        "[{dialect}] the ordering is backed by vectors that are now cached"
    );

    // A second identical query reproduces the ordering from the cache.
    assert_eq!(
        f.query(Some(true)).await,
        semantic,
        "[{dialect}] the cached pass reproduces the same ordering"
    );
    eprintln!("[t19-e2e] {dialect}: lexical {lexical:?} -> semantic {semantic:?}");
    semantic
}

#[tokio::test]
async fn a8_semantic_rerank_reorders_live_over_http() {
    // Prefer the SHIPPED mock when CI provides it, so the ordering is proven against
    // the artifact that actually ships; fall back to the in-process endpoint (see the
    // module docs) so the leg still runs on a host without a container runtime.
    let (api_root, which) = match shipped_assist_url() {
        Some(url) => {
            require_shipped_assist(&url).await;
            (
                format!("{url}/v1"),
                "scripts/mock-assist via MW_TEST_ASSIST_URL",
            )
        }
        None => {
            skip(
                "a8 ordering vs the SHIPPED mock",
                "MW_TEST_ASSIST_URL unset — running against the in-process endpoint \
                 instead (same FNV-1a/64-wide embedding; see \
                 the_in_process_embedder_matches_the_shipped_mock)",
            );
            (spawn_assist_mock().await, "in-process endpoint")
        }
    };
    eprintln!("[t19-e2e] A8 embeddings endpoint: {which}");

    let db = test_db::unique_db_path("mw-t19-a8");
    drive_semantic(&db.to_string_lossy(), "sqlite", api_root).await;
}

#[tokio::test]
async fn a8_semantic_rerank_reorders_live_on_postgres() {
    let Some(admin) = pg_dsn() else {
        skip(
            "A8 re-rank on live Postgres",
            "MW_E14_PG_DSN unset — the re-rank + 0022 read/write path is NOT exercised \
             on Postgres here. CI must run it (docker compose -f docker-compose.ci.yml \
             up -d --wait postgres).",
        );
        return;
    };
    let dsn = pg_fresh_db(&admin, "a8_rerank").await;
    let api_root = match shipped_assist_url() {
        Some(url) => {
            require_shipped_assist(&url).await;
            format!("{url}/v1")
        }
        None => spawn_assist_mock().await,
    };
    drive_semantic(&dsn, "postgres", api_root).await;
}

/// The default path must be untouched by the feature — measured against a server
/// that has NO Assist config at all, i.e. the literal 26.18 code path, rather than
/// against this suite's own expectations.
#[tokio::test]
async fn the_default_search_path_matches_a_server_without_the_feature() {
    let api_root = match shipped_assist_url() {
        Some(url) => {
            require_shipped_assist(&url).await;
            format!("{url}/v1")
        }
        None => spawn_assist_mock().await,
    };

    let pop3 = spawn_pop3_mock().await;
    let plain_db = test_db::unique_db_path("mw-t19-plain");
    let plain = fixture(&plain_db.to_string_lossy(), pop3, None).await;
    let plain_ids = plain.seed_corpus().await;
    let baseline = plain.query(None).await;
    let baseline_positions: Vec<usize> = baseline
        .iter()
        .map(|id| {
            plain_ids
                .ids
                .iter()
                .position(|x| x == id)
                .expect("known id")
        })
        .collect();

    let feat_db = test_db::unique_db_path("mw-t19-feat");
    let feat = fixture(
        &feat_db.to_string_lossy(),
        pop3,
        Some(AssistWiring {
            api_root,
            grants: vec!["search-semantic"],
        }),
    )
    .await;
    let feat_ids = feat.seed_corpus().await;

    // Ids are content-derived and per-store, so the comparable thing is the ordering
    // as POSITIONS in the identical corpus.
    for (label, semantic) in [("absent", None), ("false", Some(false))] {
        let got = feat.query(semantic).await;
        let positions: Vec<usize> = got
            .iter()
            .map(|id| feat_ids.ids.iter().position(|x| x == id).expect("known id"))
            .collect();
        assert_eq!(
            positions, baseline_positions,
            "with semantic {label}, a server WITH the feature must return exactly what a \
             server without it returns"
        );
    }
    assert_eq!(
        feat.store
            .count_message_embeddings(&feat.account)
            .await
            .unwrap(),
        0,
        "and it must not have embedded anything on the way"
    );
}

/// Live degradation: a cache populated at a different width is skipped, and the
/// result falls back to lexical rather than being corrupted by a mismatched compare.
#[tokio::test]
async fn a_dimension_mismatch_degrades_to_lexical_live() {
    let api_root = match shipped_assist_url() {
        Some(url) => {
            require_shipped_assist(&url).await;
            format!("{url}/v1")
        }
        None => spawn_assist_mock().await,
    };
    let pop3 = spawn_pop3_mock().await;
    let db = test_db::unique_db_path("mw-t19-dim");
    let f = fixture(
        &db.to_string_lossy(),
        pop3,
        Some(AssistWiring {
            api_root,
            grants: vec!["search-semantic"],
        }),
    )
    .await;
    let corpus = f.seed_corpus().await;
    let lexical = f.query(None).await;

    // A previous deployment ran a 16-wide model; the live endpoint is 64-wide.
    for id in &corpus.ids {
        f.store
            .put_message_embedding(id, &f.account, EMBED_MODEL, &[0.5_f32; 16])
            .await
            .expect("seed a stale-width vector");
    }
    let got = f.query(Some(true)).await;
    assert_eq!(
        got, lexical,
        "a width change under a populated cache degrades to lexical, never mis-ranks"
    );
    // Skipped rows are NOT silently re-embedded (that would quietly re-proxy the
    // mailbox to the endpoint on every dimension change).
    assert_eq!(
        f.store.count_message_embeddings(&f.account).await.unwrap(),
        3,
        "the stale rows are still the only rows"
    );
    for id in &corpus.ids {
        let row = f
            .store
            .get_message_embedding(id)
            .await
            .expect("read back")
            .expect("row present");
        assert_eq!(
            row.vector.len(),
            16,
            "the stale row was skipped, not overwritten"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// LEG 2 — migration 0022 on a FRESH live Postgres.
// ─────────────────────────────────────────────────────────────────────────────

/// `t19-e8` verified 0022 on fresh SQLite plus a normalised static lockstep check of
/// the two dialect files, and recorded live Postgres as a genuine loud-skip for this
/// lane. This is that leg: a database that did not exist a moment ago, migrated from
/// zero, then driven through the real sealed repository.
#[tokio::test]
async fn migration_0022_applies_on_a_fresh_live_postgres() {
    let Some(admin) = pg_dsn() else {
        skip(
            "migration 0022 on live Postgres",
            "MW_E14_PG_DSN unset — 0022 is proven on fresh SQLite + a normalised static \
             lockstep diff of the two dialect files only. CI must apply it to a real \
             Postgres from a fresh database.",
        );
        return;
    };
    let dsn = pg_fresh_db(&admin, "m0022").await;
    let key = ServerKey::from_hex(KEY_HEX).unwrap();
    // Opening runs every migration 0001..0022 against an empty database.
    let store = Store::open(&dsn, key)
        .await
        .expect("fresh PG migrates to 0022");

    let vector: Vec<f32> = (0..EMBED_DIM).map(|i| (i as f32) / 64.0 - 0.5).collect();
    store
        .put_message_embedding("m-0022", "acct-0022", EMBED_MODEL, &vector)
        .await
        .expect("0022 write");
    let row = store
        .get_message_embedding("m-0022")
        .await
        .expect("0022 read")
        .expect("the row is there");
    assert_eq!(row.model, EMBED_MODEL);
    assert_eq!(row.account_id, "acct-0022");
    assert_eq!(row.vector.len(), EMBED_DIM);
    assert_eq!(
        row.vector, vector,
        "the vector round-trips through BYTEA unchanged"
    );
    assert_eq!(
        store.count_message_embeddings("acct-0022").await.unwrap(),
        1
    );

    // At rest it is SEALED, not the plaintext little-endian f32 run.
    let mut plaintext = Vec::with_capacity(vector.len() * 4);
    for f in &vector {
        plaintext.extend_from_slice(&f.to_le_bytes());
    }
    let pool = sqlx::postgres::PgPool::connect(&dsn).await.unwrap();
    let stored: Vec<u8> =
        sqlx::query_scalar("SELECT vector_sealed FROM message_embeddings WHERE stable_id = $1")
            .bind("m-0022")
            .fetch_one(&pool)
            .await
            .expect("the 0022 table exists on Postgres with the column it declares");
    pool.close().await;
    assert_ne!(
        stored, plaintext,
        "the stored blob is not the plaintext run"
    );
    assert!(
        !stored
            .windows(plaintext.len().min(stored.len().max(1)))
            .any(|w| w == plaintext.as_slice()),
        "the plaintext vector must not appear anywhere in the stored bytes"
    );

    // Account scoping is enforced by the same SQL on this dialect too.
    store
        .delete_account_message_embeddings("acct-0022")
        .await
        .expect("account purge");
    assert_eq!(
        store.count_message_embeddings("acct-0022").await.unwrap(),
        0
    );
    eprintln!("[t19-e2e] migration 0022 applied and round-tripped on live Postgres ({dsn})");
}

/// The in-process endpoint must embed exactly as `scripts/mock-assist/server.py`
/// does, or the fallback path would be proving something the shipped mock does not
/// do. Pinned on published vectors rather than on a re-implementation, so a change to
/// either side has to be deliberate.
#[test]
fn the_in_process_embedder_matches_the_shipped_mock() {
    // The right-hand sides are the vectors the SHIPPED `scripts/mock-assist/server.py`
    // actually returned when queried (`POST /v1/embeddings`), recorded as
    // (bucket, value) pairs. They are observations of the other implementation, not a
    // re-derivation of this one — which is the only way this test can catch a drift
    // rather than restate it.
    let buckets = |text: &str| -> Vec<(usize, f32)> {
        hashed_embedding(text)
            .into_iter()
            .enumerate()
            .filter(|(_, x)| *x != 0.0)
            .collect()
    };
    assert_eq!(hashed_embedding("report").len(), EMBED_DIM, "64-wide");
    assert_eq!(buckets("report"), vec![(11, 1.0)]);
    // Case- and punctuation-insensitive tokenisation.
    assert_eq!(buckets("Report!"), vec![(11, 1.0)]);
    // Repeats accumulate in the same bucket.
    assert_eq!(buckets("report, report"), vec![(11, 2.0)]);
    // Different vocabulary, different direction — the property a constant vector
    // destroys, and with it every ordering assertion in this file.
    assert_eq!(buckets("lunch"), vec![(31, 1.0)]);
    // Never the zero vector: it has no direction, so the consumer would correctly
    // refuse to rank it and the caller would see an unexplained degradation instead
    // of a mock that simply had nothing to say.
    assert_eq!(buckets(""), vec![(0, 1.0)]);
    assert_eq!(buckets("!!!"), vec![(0, 1.0)]);
}

// ─────────────────────────────────────────────────────────────────────────────
// LEG 3 — the Assist `actions` HTTP seam.
// ─────────────────────────────────────────────────────────────────────────────

/// One SSE record: an optional `event:` name plus its concatenated `data:` payload.
fn parse_sse(body: &str) -> Vec<(Option<String>, String)> {
    let mut out = Vec::new();
    for record in body.replace("\r\n", "\n").split("\n\n") {
        let mut name = None;
        let mut data = String::new();
        for line in record.lines() {
            if let Some(v) = line.strip_prefix("event:") {
                name = Some(v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("data:") {
                if !data.is_empty() {
                    data.push('\n');
                }
                data.push_str(v.strip_prefix(' ').unwrap_or(v));
            }
            // `:`-comment keep-alive lines carry neither and are skipped.
        }
        if name.is_some() || !data.is_empty() {
            out.push((name, data));
        }
    }
    out
}

async fn invoke_sse(f: &Fixture, capability: &str) -> Vec<(Option<String>, String)> {
    let resp = f
        .client()
        .post(format!("{}/api/assist/invoke", f.base))
        .header(reqwest::header::COOKIE, format!("mw_session={}", f.cookie))
        .json(&json!({
            "capability": capability,
            "scope": { "accounts": [&f.account] },
            "input": { "prompt": "find the invoice thread" }
        }))
        .send()
        .await
        .expect("invoke request");
    assert_eq!(resp.status(), 200, "the gateway is enabled and authed");
    assert!(
        resp.headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|c| c.starts_with("text/event-stream")),
        "the contract is SSE, not JSON — the web client parses bytes off this stream"
    );
    assert_eq!(
        resp.headers()
            .get("x-accel-buffering")
            .and_then(|v| v.to_str().ok()),
        Some("no"),
        "a buffering proxy must not be able to hold the whole reply until the turn ends"
    );
    let body = tokio::time::timeout(Duration::from_secs(30), resp.text())
        .await
        .expect("the SSE stream terminates")
        .expect("stream body");
    parse_sse(&body)
}

/// The seam `t19-e7` could not close from a unit test: nothing posted to
/// `/api/assist/invoke` over a real router and read SSE bytes off the wire. It needs
/// an enabled gateway behind `authed()`, i.e. a booted app — which is here.
#[tokio::test]
async fn assist_invoke_streams_disclosure_then_deltas_then_a_proposal() {
    let api_root = spawn_assist_mock().await;
    let pop3 = spawn_pop3_mock().await;
    let db = test_db::unique_db_path("mw-t19-invoke");
    let f = fixture(
        &db.to_string_lossy(),
        pop3,
        Some(AssistWiring {
            api_root,
            grants: vec!["assistant"],
        }),
    )
    .await;

    let frames = invoke_sse(&f, "assistant").await;
    assert!(
        frames.len() >= 3,
        "disclosure + at least one delta + done: {frames:?}"
    );

    // 1. Disclosure leads, before any token, and states what actually left the server.
    let (name, data) = &frames[0];
    assert_eq!(
        name.as_deref(),
        Some("disclosure"),
        "the first frame must be the disclosure: {frames:?}"
    );
    let disclosure: Value = serde_json::from_str(data).expect("disclosure json");
    assert!(
        disclosure["endpoint_host"]
            .as_str()
            .is_some_and(|h| !h.is_empty()),
        "the disclosure names the endpoint host: {disclosure}"
    );

    // 2. Deltas — and the proposal marker never appears as visible prose, even though
    //    it was deliberately split across two frames upstream.
    let deltas: Vec<String> = frames
        .iter()
        .filter(|(n, _)| n.is_none())
        .map(|(_, d)| {
            serde_json::from_str::<Value>(d)
                .ok()
                .and_then(|v| v["delta"].as_str().map(str::to_string))
                .unwrap_or_default()
        })
        .collect();
    assert!(!deltas.is_empty(), "at least one token frame: {frames:?}");
    let text = deltas.concat();
    assert!(
        text.contains("Here is the reply."),
        "the visible reply is streamed: {text:?}"
    );
    assert!(
        !text.contains(mw_assist::ACTION_MARKER),
        "the action marker must never leak into visible text: {text:?}"
    );
    assert!(
        !text.contains("mail.search"),
        "nor must the proposal block itself: {text:?}"
    );

    // 3. The terminal frame carries the proposal.
    let (name, data) = frames.last().expect("a terminal frame");
    assert_eq!(
        name.as_deref(),
        Some("done"),
        "the stream ends with `done`: {frames:?}"
    );
    let done: Value = serde_json::from_str(data).expect("done json");
    let actions = done["actions"].as_array().expect("actions array");
    assert_eq!(actions.len(), 1, "one proposal: {done}");
    assert_eq!(actions[0]["tool"], json!("mail.search"));
    assert_eq!(actions[0]["id"], json!("act-1"));
    assert_eq!(
        actions[0]["would_send"],
        json!(false),
        "a search proposal is not a send"
    );
    assert_eq!(
        actions[0]["summary"],
        json!("Search for the invoice thread")
    );
    eprintln!(
        "[t19-e2e] assist invoke over HTTP: disclosure -> {} deltas -> done({} action)",
        deltas.len(),
        actions.len()
    );
}

/// A capability that is not the assistant proposes nothing, over the same wire — so
/// the terminal frame is present and empty rather than absent.
#[tokio::test]
async fn assist_invoke_for_a_non_assistant_capability_proposes_nothing() {
    let api_root = spawn_assist_mock().await;
    let pop3 = spawn_pop3_mock().await;
    let db = test_db::unique_db_path("mw-t19-noprop");
    let f = fixture(
        &db.to_string_lossy(),
        pop3,
        Some(AssistWiring {
            api_root,
            grants: vec!["summarize"],
        }),
    )
    .await;

    let frames = invoke_sse(&f, "summarize").await;
    let (name, data) = frames.last().expect("a terminal frame");
    assert_eq!(name.as_deref(), Some("done"));
    let done: Value = serde_json::from_str(data).expect("done json");
    assert_eq!(
        done["actions"].as_array().map(Vec::len),
        Some(0),
        "only the assistant capability scans for proposals: {done}"
    );
    // The pass-through filter withholds nothing, so the marker text this endpoint
    // emits is visible for a non-assistant capability. That is the documented
    // behaviour, and asserting it keeps the two branches distinguishable.
    let text: String = frames
        .iter()
        .filter(|(n, _)| n.is_none())
        .filter_map(|(_, d)| {
            serde_json::from_str::<Value>(d)
                .ok()
                .and_then(|v| v["delta"].as_str().map(str::to_string))
        })
        .collect();
    assert!(
        text.contains(mw_assist::ACTION_MARKER),
        "a non-assistant capability uses a pass-through filter: {text:?}"
    );
}

/// The same seam against the SHIPPED mock, so CI proves the HTTP hop against the
/// artifact that ships. That mock has a fixed reply with no marker, so it can prove
/// the frame ORDER and the terminal frame, but never a proposal — which is precisely
/// why the in-process endpoint above exists.
#[tokio::test]
async fn assist_invoke_against_the_shipped_mock() {
    let Some(url) = shipped_assist_url() else {
        skip(
            "assist invoke vs the SHIPPED mock",
            "MW_TEST_ASSIST_URL unset — the disclosure/delta/done order is proven over \
             real HTTP against the in-process endpoint only. CI must run it against \
             scripts/mock-assist.",
        );
        return;
    };
    require_shipped_assist(&url).await;
    let pop3 = spawn_pop3_mock().await;
    let db = test_db::unique_db_path("mw-t19-shipped");
    let f = fixture(
        &db.to_string_lossy(),
        pop3,
        Some(AssistWiring {
            api_root: format!("{url}/v1"),
            grants: vec!["assistant"],
        }),
    )
    .await;

    let frames = invoke_sse(&f, "assistant").await;
    assert_eq!(frames[0].0.as_deref(), Some("disclosure"));
    assert_eq!(frames.last().expect("terminal").0.as_deref(), Some("done"));
    let text: String = frames
        .iter()
        .filter(|(n, _)| n.is_none())
        .filter_map(|(_, d)| {
            serde_json::from_str::<Value>(d)
                .ok()
                .and_then(|v| v["delta"].as_str().map(str::to_string))
        })
        .collect();
    assert!(
        text.contains("deterministic mock Assist reply"),
        "the shipped mock's canned reply streamed through: {text:?}"
    );
    eprintln!(
        "[t19-e2e] assist invoke against scripts/mock-assist: {} frames",
        frames.len()
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// LEG 4 — appearance sync (§17.3).
// ─────────────────────────────────────────────────────────────────────────────

async fn drive_appearance(db_path: &str, dialect: &str) {
    let pop3 = spawn_pop3_mock().await;
    let f = fixture(db_path, pop3, None).await;
    let c = f.client();
    let cookie = format!("mw_session={}", f.cookie);
    let url = format!("{}/api/account/appearance", f.base);

    // Nothing stored yet: the account's own value is null, and the deployment default
    // is reported alongside it (a DEFAULT, never an enforcement).
    let got: Value = c
        .get(&url)
        .header(reqwest::header::COOKIE, &cookie)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        got["appearance"],
        Value::Null,
        "[{dialect}] nothing stored: {got}"
    );
    assert_eq!(got["updatedAt"], Value::Null, "[{dialect}] {got}");
    assert!(
        got["deploymentDefault"].is_object(),
        "[{dialect}] the deployment default is always reported: {got}"
    );

    // PUT round-trips through the server, not localStorage.
    let prefs = json!({ "theme": "grove-dark", "mode": "system", "density": "cosy" });
    let put: Value = c
        .put(&url)
        .header(reqwest::header::COOKIE, &cookie)
        .json(&json!({ "appearance": prefs }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(put["ok"], json!(true), "[{dialect}] {put}");
    let first_stamp = put["updatedAt"].as_i64().expect("a server stamp");

    let got: Value = c
        .get(&url)
        .header(reqwest::header::COOKIE, &cookie)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        got["appearance"], prefs,
        "[{dialect}] the stored object round-trips verbatim (the server keeps it \
         opaque on purpose — the theme union lives in the client registry): {got}"
    );
    assert_eq!(got["updatedAt"], json!(first_stamp), "[{dialect}] {got}");

    // A fresh session for the same account sees it — it is account state, not
    // device state.
    let second = f
        .store
        .create_session(
            &f.account,
            "reader@example.org",
            "http://upstream.invalid",
            "http://upstream.invalid",
            &Credentials {
                username: "reader@example.org".into(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap();
    let got: Value = c
        .get(&url)
        .header(reqwest::header::COOKIE, format!("mw_session={second}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        got["appearance"], prefs,
        "[{dialect}] a second session for the same account adopts the saved appearance"
    );

    // The 8 KiB cap answers 413 — and does not disturb what is stored.
    let oversized = json!({ "blob": "x".repeat(9 * 1024) });
    let resp = c
        .put(&url)
        .header(reqwest::header::COOKIE, &cookie)
        .json(&json!({ "appearance": oversized }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        413,
        "[{dialect}] the per-account cap refuses an oversized object"
    );
    let got: Value = c
        .get(&url)
        .header(reqwest::header::COOKIE, &cookie)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        got["appearance"], prefs,
        "[{dialect}] a refused write leaves the stored value alone"
    );

    // A non-object is refused too (nothing else can round-trip into a preference set).
    let resp = c
        .put(&url)
        .header(reqwest::header::COOKIE, &cookie)
        .json(&json!({ "appearance": "grove-dark" }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        400,
        "[{dialect}] the payload must be an object"
    );

    // DELETE stores a NEWER null rather than removing the key. That is the whole
    // point: a removed key reads as "nothing stored", and a device still holding the
    // old object would win the next reconcile and resurrect what the user forgot.
    let del: Value = c
        .delete(&url)
        .header(reqwest::header::COOKIE, &cookie)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(del["ok"], json!(true), "[{dialect}] {del}");
    let reset_stamp = del["updatedAt"].as_i64().expect("a server stamp");
    assert!(
        reset_stamp >= first_stamp,
        "[{dialect}] the reset's stamp must not go backwards ({reset_stamp} vs {first_stamp})"
    );
    let got: Value = c
        .get(&url)
        .header(reqwest::header::COOKIE, &cookie)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        got["appearance"],
        Value::Null,
        "[{dialect}] the account's appearance is forgotten: {got}"
    );
    assert_eq!(
        got["updatedAt"],
        json!(reset_stamp),
        "[{dialect}] and the reset is a stored event with its own timestamp, so a stale \
         device loses the next reconcile instead of winning it: {got}"
    );

    // Unauthenticated callers get nothing on any verb.
    for method in ["GET", "PUT", "DELETE"] {
        let req = match method {
            "GET" => c.get(&url),
            "PUT" => c.put(&url).json(&json!({ "appearance": {} })),
            _ => c.delete(&url),
        };
        let status = req.send().await.unwrap().status();
        assert!(
            status == 401 || status == 403,
            "[{dialect}] {method} without a session is refused (got {status})"
        );
    }
    eprintln!("[t19-e2e] appearance sync round-tripped on {dialect}");
}

#[tokio::test]
async fn appearance_syncs_server_side() {
    let db = test_db::unique_db_path("mw-t19-appear");
    drive_appearance(&db.to_string_lossy(), "sqlite").await;
}

#[tokio::test]
async fn appearance_syncs_server_side_on_postgres() {
    let Some(admin) = pg_dsn() else {
        skip(
            "appearance sync on live Postgres",
            "MW_E14_PG_DSN unset — the settings-KV round-trip is proven on SQLite only.",
        );
        return;
    };
    let dsn = pg_fresh_db(&admin, "appearance").await;
    drive_appearance(&dsn, "postgres").await;
}

// ─────────────────────────────────────────────────────────────────────────────
// LEG 5 — live-Postgres test isolation (t19-e2's hand-off).
// ─────────────────────────────────────────────────────────────────────────────

/// Two stores opened concurrently on two fresh databases both migrate to 0022
/// without touching each other's `_sqlx_migrations`.
///
/// This is the shape that fails today when several tests in one binary share one
/// DSN: `sqlx::migrate!` inserts into `_sqlx_migrations`, and two runs against the
/// same database race there — which per-test temp PATHS cannot fix, because the path
/// is not the thing that collides. Rewriting the twelve existing live-PG binaries is
/// outside this lane's locks; [`pg_fresh_db`] shows the fix and this leg proves it,
/// so a follow-up has a working pattern to copy rather than a description of one.
#[tokio::test]
async fn concurrent_live_pg_stores_do_not_share_a_migration_table() {
    let Some(admin) = pg_dsn() else {
        skip(
            "live-PG migration isolation",
            "MW_E14_PG_DSN unset — the `_sqlx_migrations` race t19-e2 handed on is NOT \
             reproducible without a live Postgres, and this suite's per-leg-database fix \
             is therefore unexercised here. CI must run it.",
        );
        return;
    };
    let a = pg_fresh_db(&admin, "iso_a").await;
    let b = pg_fresh_db(&admin, "iso_b").await;
    assert_ne!(a, b, "each leg gets its own database");

    let key = ServerKey::from_hex(KEY_HEX).unwrap();
    let (ra, rb) = tokio::join!(
        Store::open(&a, key.clone()),
        Store::open(&b, ServerKey::from_hex(KEY_HEX).unwrap()),
    );
    let sa = ra.expect("store A migrates on its own database");
    let sb = rb.expect("store B migrates on its own database");

    // Same primary key on both, which a shared database could not accept.
    for (label, store) in [("A", &sa), ("B", &sb)] {
        store
            .put_message_embedding("shared-id", "acct", EMBED_MODEL, &[1.0_f32; 8])
            .await
            .unwrap_or_else(|e| panic!("store {label} owns its own row space: {e}"));
        assert_eq!(store.count_message_embeddings("acct").await.unwrap(), 1);
    }
    eprintln!("[t19-e2e] two live-PG stores migrated concurrently on private databases");
}
